#!/usr/bin/env python3
"""Fail-closed macOS admission and watchdog for the unchanged EXP-008 worker."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
PREVIOUS = ROOT.parent / "EXP-008-synthetic-pir-scale"
sys.path.insert(0, str(PREVIOUS))
from summarize import summarise

GIB = 1 << 30
POLICY = {"normal_pressure_flag": 1, "minimum_start_free_percent": 30,
          "stop_free_percent_below": 25, "max_estimated_bytes": 7 * GIB // 2,
          "rss_stop_bytes": 3 * GIB, "swap_growth_stop_bytes": 128 << 20,
          "timeout_seconds": 240, "poll_seconds": 0.25, "nice_increment": 10}


def stamp():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def save(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def read_command(*args):
    result = subprocess.run(args, text=True, capture_output=True, timeout=3)
    if result.returncode:
        raise RuntimeError(f"telemetry command failed: {args[0]}: {result.stderr.strip()}")
    return result.stdout


def parse_telemetry(sysctl, pressure):
    level = re.search(r"kern.memorystatus_vm_pressure_level:\s*(\d+)", sysctl)
    swap = re.search(r"used\s*=\s*([\d.]+)M", sysctl)
    free = re.search(r"System-wide memory free percentage:\s*(\d+)%", pressure)
    if not level or not swap or not free or not 0 <= int(free[1]) <= 100:
        raise ValueError("unrecognised memory telemetry; refusing to guess")
    return {"pressure_flag": int(level[1]), "swap_used_bytes": int(float(swap[1]) * (1 << 20)),
            "free_percent_indicator": int(free[1]), "observed_at": stamp()}


def telemetry():
    values = read_command("/usr/sbin/sysctl", "kern.memorystatus_vm_pressure_level", "vm.swapusage")
    pressure = read_command("/usr/bin/memory_pressure", "-Q")
    return parse_telemetry(values, pressure)


def admission(sample, physical_bytes, estimated_bytes):
    if sample["pressure_flag"] != 1:
        return "memory_pressure_not_normal", 0
    if sample["free_percent_indicator"] < POLICY["minimum_start_free_percent"]:
        return "insufficient_start_headroom", 0
    reserve_budget = physical_bytes * max(0, sample["free_percent_indicator"] - 25) // 100
    budget = min(POLICY["max_estimated_bytes"], physical_bytes // 6, reserve_budget)
    if estimated_bytes > budget:
        return "estimate_above_safe_budget", budget
    return None, budget


def stop_reason(sample, baseline, rss, elapsed):
    if sample["pressure_flag"] != 1:
        return "memory_pressure_not_normal"
    if sample["free_percent_indicator"] < POLICY["stop_free_percent_below"]:
        return "free_indicator_below_reserve"
    if sample["swap_used_bytes"] - baseline["swap_used_bytes"] > POLICY["swap_growth_stop_bytes"]:
        return "swap_growth"
    if rss >= POLICY["rss_stop_bytes"]:
        return "worker_rss_limit"
    if elapsed >= POLICY["timeout_seconds"]:
        return "timeout"
    return None


def rss_bytes(pid):
    result = subprocess.run(["/bin/ps", "-o", "rss=", "-p", str(pid)],
                            text=True, capture_output=True, timeout=3)
    if result.returncode == 1 and not result.stdout.strip():
        return None  # Child may have exited between wait4 and ps. Parent reaps next.
    if result.returncode or not result.stdout.strip().isdigit():
        raise RuntimeError("cannot read worker RSS")
    return int(result.stdout.strip()) * 1024


def watch_child(argv, directory, baseline, sample_fn=telemetry, rss_fn=rss_bytes, poll=None):
    """Every exceptional exit kills/reaps the only child. Does not touch other processes."""
    directory = Path(directory)
    interval = POLICY["poll_seconds"] if poll is None else poll
    start = time.monotonic()
    samples = []
    reason = None
    reaped = False
    with (directory / "worker.json").open("xb") as stdout, (directory / "worker.stderr.log").open("xb") as stderr:
        pid = os.fork()
        if pid == 0:
            os.dup2(stdout.fileno(), 1)
            os.dup2(stderr.fileno(), 2)
            try:
                os.nice(POLICY["nice_increment"])
                os.execv(argv[0], argv)
            except BaseException as exc:
                os.write(2, f"worker exec failed: {exc}\n".encode())
                os._exit(127)
        try:
            while True:
                finished, status, usage = os.wait4(pid, os.WNOHANG)
                if finished:
                    reaped = True
                    break
                try:
                    sample = sample_fn()
                    rss = rss_fn(pid)
                    if rss is None:
                        finished, status, usage = os.wait4(pid, os.WNOHANG)
                        if finished:
                            reaped = True
                            break
                        raise RuntimeError("RSS unavailable for a still-running worker")
                    elapsed = time.monotonic() - start
                    sample = dict(sample, elapsed_seconds=elapsed, worker_rss_bytes=rss)
                    samples.append(sample)
                    reason = stop_reason(sample, baseline, rss or 0, elapsed)
                except Exception as exc:
                    reason = "telemetry_unavailable"
                    samples.append({"observed_at": stamp(), "error": str(exc)})
                if reason:
                    os.kill(pid, signal.SIGKILL)
                    _, status, usage = os.wait4(pid, 0)
                    reaped = True
                    break
                time.sleep(interval)
        finally:
            if not reaped:
                try:
                    os.kill(pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                os.wait4(pid, 0)
    return {"status": "aborted" if reason else ("completed" if os.waitstatus_to_exitcode(status) == 0 else "failed"),
            "reason": reason, "exit_code": os.waitstatus_to_exitcode(status),
            "wall_seconds": time.monotonic() - start, "samples": samples,
            "peak_process_rss_bytes": usage.ru_maxrss * (1024 if sys.platform == "linux" else 1)}


def main():
    if len(sys.argv) != 3 or sys.platform != "darwin":
        raise SystemExit("usage on macOS: guarded_run.py EXP008_WORKER NEW_OUTPUT_DIRECTORY")
    binary = Path(sys.argv[1]).resolve(strict=True)
    output = Path(sys.argv[2]).resolve()
    output.mkdir(parents=True, exist_ok=False)
    hardware = read_command("/usr/sbin/sysctl", "hw.memsize", "hw.logicalcpu", "machdep.cpu.brand_string")
    memory_match = re.search(r"hw.memsize:\s*(\d+)", hardware)
    if not memory_match:
        raise RuntimeError("physical memory unavailable")
    physical = int(memory_match[1])
    manifest = {"experiment_id": "EXP-009", "research_questions": ["RQ-005", "RQ-008"],
        "claim_type": "synthetic result", "started_at": stamp(), "completed_at": None,
        "machine": {"hardware_sysctl": hardware, "physical_memory_bytes": physical,
            "operating_system": platform.platform(), "logical_cores_used": 1},
        "software_revisions": {"project_base_revision": read_command("git", "-C", str(ROOT), "rev-parse", "HEAD").strip(),
            "worker_experiment": "EXP-008 unchanged binary", "python": sys.version.split()[0]},
        "inputs": {"binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
            "runner_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
            "generator": "EXP-008 SplitMix64-v1; synthetic bytes, no Ethereum state"},
        "parameters": {"rows": [131072, 524288], "record_bytes": [3072, 4096],
            "queries_per_case": 32, "policy": POLICY,
            "security": {"lwe_dimension": 1275, "sigma": 6.4, "ciphertext_modulus_log2": 32,
                "plaintext_bits": 8, "upstream_correctness_kappa": 40}},
        "metrics": [{"name": "guard_telemetry_and_status", "unit": "bytes, percentage, seconds", "raw_result_file": "*/guard.json"},
            {"name": "unchanged_worker_performance", "unit": "bytes and microseconds", "raw_result_file": "*-*.json"}]}
    save(output / "manifest.json", manifest)
    cases = []
    abort_batch = False
    for rows in (131072, 524288):
        for width in (3072, 4096):
            name = f"simple-index-{rows}-{width}"
            directory = output / name
            directory.mkdir()
            # Large planning allowance is arithmetic only; never used for admission directly.
            plan = json.loads(read_command(str(binary), "plan", str(rows), str(width), str(16 * GIB), "32"))
            save(output / f"{name}.plan.json", plan)
            if abort_batch:
                record = {"case": name, "status": "deferred", "reason": "previous_case_aborted"}
            else:
                try:
                    baseline = telemetry()
                    reason, budget = admission(baseline, physical, plan["estimated_peak_working_bytes"])
                    save(directory / "baseline.json", baseline)
                except Exception as exc:
                    reason, budget = "telemetry_unavailable", 0
                    save(directory / "baseline.json", {"error": str(exc)})
                if reason or not plan["admitted"]:
                    record = {"case": name, "status": "deferred", "reason": reason or plan["skip_reason"],
                              "safe_budget_bytes": budget, "estimated_peak_working_bytes": plan["estimated_peak_working_bytes"]}
                else:
                    admitted_plan = json.loads(read_command(str(binary), "plan", str(rows), str(width), str(budget), "32"))
                    save(output / f"{name}.plan.json", admitted_plan)
                    result = watch_child([str(binary), "run", str(rows), str(width), str(budget), "32"], directory, baseline)
                    save(directory / "guard.json", result)
                    record = {"case": name, **{k: v for k, v in result.items() if k != "samples"}}
                    if result["status"] == "completed":
                        raw = (directory / "worker.json").read_bytes()
                        (output / f"{name}.json").write_bytes(raw)
                        item = json.loads(raw)
                        if item["status"] != "completed" or item["queries_completed"] != 32:
                            raise RuntimeError("worker exited without complete measurements")
                    else:
                        abort_batch = True
            cases.append(record)
            save(output / "processes.json", cases)
            print(name, record["status"], record.get("reason", ""), flush=True)
    manifest["completed_at"] = stamp()
    manifest["case_status_counts"] = {s: sum(c["status"] == s for c in cases) for s in sorted({c["status"] for c in cases})}
    save(output / "manifest.json", manifest)
    save(output / "summary.json", summarise(output))
    print(json.dumps(manifest["case_status_counts"]), flush=True)


if __name__ == "__main__":
    def interrupted(signum, frame):
        raise KeyboardInterrupt("guarded run interrupted")
    signal.signal(signal.SIGTERM, interrupted)
    main()
