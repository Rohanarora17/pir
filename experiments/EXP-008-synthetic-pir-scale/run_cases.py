#!/usr/bin/env python3
"""Sequential isolated synthetic cases, retaining plans, skips, failures and child RSS."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
ROWS = (32, 128, 512, 2048, 8192, 32768, 131072, 524288)
WIDTHS = (3072, 4096)


def timestamp():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def save(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def command(*args):
    return subprocess.check_output(args, text=True).strip()


def main():
    if len(sys.argv) != 3:
        raise SystemExit("usage: run_cases.py BINARY NEW_OUTPUT_DIRECTORY")
    if sys.platform not in ("darwin", "linux"):
        raise SystemExit("child RSS supported on macOS/Linux only")
    binary = Path(sys.argv[1]).resolve(strict=True)
    output = Path(sys.argv[2]).resolve()
    budget_mib = int(os.environ.get("MEMORY_BUDGET_MIB", "1024"))
    timeout = int(os.environ.get("CASE_TIMEOUT_SECONDS", "180"))
    queries = int(os.environ.get("QUERY_COUNT", "32"))
    if not 1 <= budget_mib <= 4096 or not 1 <= timeout <= 600 or not 4 <= queries <= 1024:
        raise SystemExit("resource/sample budgets outside supported ranges")
    budget = budget_mib << 20
    output.mkdir(parents=True, exist_ok=False)
    sources = [ROOT / "probe/src/lib.rs", ROOT / "probe/src/main.rs", ROOT / "probe/Cargo.lock",
               ROOT / "run_cases.py", ROOT / "summarize.py", ROOT / "commands.sh"]
    manifest = {
        "experiment_id": "EXP-008", "research_questions": ["RQ-005", "RQ-008"],
        "claim_type": "synthetic result", "started_at": timestamp(), "completed_at": None,
        "software_revisions": {"project_base_revision": command("git", "-C", str(ROOT), "rev-parse", "HEAD"),
            "rustc": command("rustc", "+stable", "--version"),
            "cargo": command("cargo", "+stable", "--version"),
            "tree_state": command("git", "-C", str(ROOT), "status", "--short"),
            "upstream_revision": "fcfb2b540c1cfa92c2de5ec818db3aa9fc817395"},
        "machine": {"operating_system": platform.platform(), "architecture": platform.machine(),
            "logical_cores_available": os.cpu_count(), "logical_cores_used": 1,
            "cpu_model": platform.processor() or "not available", "host_ram": "not available in sandbox",
            "gpu_used": False},
        "inputs": {"generator": "SplitMix64-v1 low byte, seed 0x505652504353434c + (row << 32 | offset)",
            "ethereum_state": "none; synthetic bytes only",
            "source_sha256": {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in sources},
            "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
            "upstream_archive_sha256": hashlib.sha256((ROOT.parent / "EXP-006-pir-baseline/source/ikpir.tar.gz").read_bytes()).hexdigest()},
        "parameters": {"scheme": "SimplePIR direct index", "rows": ROWS, "record_bytes": WIDTHS,
            "queries_per_case": queries, "memory_budget_bytes": budget, "case_timeout_seconds": timeout,
            "security": {"lwe_dimension": 1275, "sigma": 6.4, "ciphertext_modulus_log2": 32,
                "plaintext_bits": 8, "upstream_correctness_kappa": 40},
            "client_state_size": "per-case hint plus expanded public matrix; not RSS",
            "preprocessing_generation": "one independent cold setup per case, no update protocol",
            "transport": "none; bounded local frames", "setup_bytes": "canonical minimum estimates",
            "build_profile": "release", "query_policy": "sequential, no precompute queues"},
        "metrics": [{"name": "setup_and_read_times", "unit": "microseconds", "raw_result_file": "*.json"},
            {"name": "setup_message_arrays_and_RSS", "unit": "bytes", "raw_result_file": "*.json and processes.json"}],
    }
    save(output / "manifest.json", manifest)
    cases = []
    for width in WIDTHS:
        for rows in ROWS:
            name = f"simple-index-{rows}-{width}"
            args = [str(rows), str(width), str(budget), str(queries)]
            plan_result = subprocess.run([str(binary), "plan", *args], text=True, capture_output=True, timeout=10)
            (output / f"{name}.plan.json").write_text(plan_result.stdout)
            (output / f"{name}.plan.stderr.log").write_text(plan_result.stderr)
            if plan_result.returncode:
                cases.append({"case": name, "status": "preflight_failed", "exit_code": plan_result.returncode})
                save(output / "processes.json", cases)
                continue
            geometry = json.loads(plan_result.stdout)
            if not geometry["admitted"]:
                cases.append({"case": name, "status": "skipped", "plan": geometry})
                save(output / "processes.json", cases)
                print(f"skipped {name}: {geometry['skip_reason']}", flush=True)
                continue
            started = timestamp()
            begin = time.monotonic()
            timed_out = False
            with (output / f"{name}.json").open("xb") as stdout, (output / f"{name}.stderr.log").open("xb") as stderr:
                pid = os.fork()
                if pid == 0:
                    os.dup2(stdout.fileno(), 1)
                    os.dup2(stderr.fileno(), 2)
                    try:
                        os.execv(str(binary), [str(binary), "run", *args])
                    except Exception as exc:
                        os.write(2, f"exec failure: {exc}\n".encode())
                        os._exit(127)
                while True:
                    finished, status, usage = os.wait4(pid, os.WNOHANG)
                    if finished:
                        break
                    if time.monotonic() - begin > timeout:
                        os.kill(pid, signal.SIGKILL)
                        _, status, usage = os.wait4(pid, 0)
                        timed_out = True
                        break
                    time.sleep(0.05)
            exit_code = os.waitstatus_to_exitcode(status)
            cases.append({"case": name, "status": "timed_out" if timed_out else ("completed" if exit_code == 0 else "failed"),
                "exit_code": exit_code, "started_at": started, "completed_at": timestamp(),
                "wall_seconds_with_process_and_generation": time.monotonic() - begin,
                "peak_process_rss_bytes": usage.ru_maxrss * (1024 if sys.platform == "linux" else 1),
                "user_cpu_seconds": usage.ru_utime, "system_cpu_seconds": usage.ru_stime,
                "rss_scope": "whole process, server/client, transient copies and code"})
            save(output / "processes.json", cases)
            print(f"{cases[-1]['status']} {name}", flush=True)
    manifest["completed_at"] = timestamp()
    manifest["case_status_counts"] = {s: sum(c["status"] == s for c in cases) for s in sorted({c["status"] for c in cases})}
    save(output / "manifest.json", manifest)
    if any(c["status"] not in ("completed", "skipped") for c in cases):
        raise SystemExit("some cases failed or timed out; all evidence retained")


if __name__ == "__main__":
    main()
