#!/usr/bin/env python3
"""Audit guard decisions, fixed-parameter reads and retained files without rerunning a load."""
import hashlib
import json
from pathlib import Path
import sys
from guarded_run import ROOT, POLICY, PREVIOUS, admission, stop_reason, summarise


def audit(directory):
    directory = Path(directory)
    manifest = json.loads((directory / "manifest.json").read_text())
    cases = json.loads((directory / "processes.json").read_text())
    if manifest["experiment_id"] != "EXP-009" or manifest["parameters"]["policy"] != POLICY:
        raise ValueError("experiment or guard policy changed")
    candidates = [ROOT / "guarded_run.py", ROOT / "raw/guarded_run-executed.py"]
    if not any(p.exists() and hashlib.sha256(p.read_bytes()).hexdigest() == manifest["inputs"]["runner_sha256"] for p in candidates):
        raise ValueError("executed runner source is not retained")
    expected = {f"simple-index-{r}-{w}" for r in (131072, 524288) for w in (3072, 4096)}
    if len(cases) != 4 or {c["case"] for c in cases} != expected:
        raise ValueError("missing or duplicated larger configurations")
    completed = 0
    reads = 0
    for case in cases:
        name = case["case"]
        plan = json.loads((directory / f"{name}.plan.json").read_text())
        if case["status"] == "deferred":
            if case["reason"] != "previous_case_aborted":
                baseline = json.loads((directory / name / "baseline.json").read_text())
                if "error" not in baseline:
                    reason, budget = admission(baseline, manifest["machine"]["physical_memory_bytes"], plan["estimated_peak_working_bytes"])
                    if reason != case["reason"] or budget != case["safe_budget_bytes"]:
                        raise ValueError("deferred case disagrees with safety admission")
            continue
        guard = json.loads((directory / name / "guard.json").read_text())
        if guard["status"] != case["status"] or guard["exit_code"] != case["exit_code"]:
            raise ValueError("guard/process status disagreement")
        if case["status"] != "completed":
            if case["status"] not in ("aborted", "failed") or not case.get("reason"):
                raise ValueError("unexplained unsuccessful case")
            continue
        baseline = json.loads((directory / name / "baseline.json").read_text())
        reason, budget = admission(baseline, manifest["machine"]["physical_memory_bytes"], plan["estimated_peak_working_bytes"])
        if reason is not None or plan["memory_budget_bytes"] != budget:
            raise ValueError("completed case was not safely admitted")
        if not guard["samples"] or case["peak_process_rss_bytes"] >= POLICY["rss_stop_bytes"]:
            raise ValueError("missing monitoring samples or peak RSS above local limit")
        for sample in guard["samples"]:
            if "error" in sample or stop_reason(sample, baseline, sample["worker_rss_bytes"] or 0, sample["elapsed_seconds"]):
                raise ValueError("completed case crossed a guard condition")
        raw = json.loads((directory / f"{name}.json").read_text())
        if raw != json.loads((directory / name / "worker.json").read_text()):
            raise ValueError("worker output copy differs")
        if raw["plan"] != plan or raw["security"] != manifest["parameters"]["security"]:
            raise ValueError("worker geometry or security settings changed")
        if raw["queries_completed"] != 32 or len(raw["measurements"]) != 32:
            raise ValueError("incomplete larger-case reads")
        for sample in raw["measurements"]:
            if sample["request_bytes"] != 44 + 4 * plan["matrix_rows"] or sample["response_bytes"] != 44 + 4 * plan["matrix_columns"]:
                raise ValueError("wire bytes disagree with backend shape")
        completed += 1
        reads += raw["queries_completed"]
    if summarise(directory) != json.loads((directory / "summary.json").read_text()):
        raise ValueError("summary differs from raw measurements")
    return {"completed_cases": completed, "completed_reads": reads,
            "deferred_cases": sum(c["status"] == "deferred" for c in cases),
            "aborted_cases": sum(c["status"] == "aborted" for c in cases)}


if __name__ == "__main__":
    index = ROOT / "checksums.json"
    if index.exists():
        checksums = json.loads(index.read_text())
        actual = {str(p.relative_to(ROOT)) for p in ROOT.rglob("*")
                  if p.is_file() and p.name != "checksums.json" and "__pycache__" not in p.parts}
        if actual != set(checksums):
            raise ValueError("retained inventory differs")
        for name, digest in checksums.items():
            path = (ROOT / name).resolve()
            if not path.is_relative_to(ROOT) or hashlib.sha256(path.read_bytes()).hexdigest() != digest:
                raise ValueError(f"checksum mismatch: {name}")
        print(f"verified {len(checksums)} checksums")
    print(json.dumps(audit(Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "raw/run"), indent=2))
