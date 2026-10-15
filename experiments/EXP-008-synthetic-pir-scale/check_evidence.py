#!/usr/bin/env python3
"""Offline inventory, provenance, derived-result and basic measurement audit."""
import hashlib
import json
from pathlib import Path
import sys
from summarize import summarise

ROOT = Path(__file__).resolve().parent


def check(directory):
    directory = Path(directory)
    manifest = json.loads((directory / "manifest.json").read_text())
    processes = json.loads((directory / "processes.json").read_text())
    expected = {f"simple-index-{n}-{w}" for n in manifest["parameters"]["rows"]
                for w in manifest["parameters"]["record_bytes"]}
    if len(processes) != len(expected) or {p["case"] for p in processes} != expected:
        raise ValueError("missing, duplicated or unexpected cases")
    for name, digest in manifest["inputs"]["source_sha256"].items():
        if hashlib.sha256((ROOT / name).read_bytes()).hexdigest() != digest:
            raise ValueError(f"source differs from executed version: {name}")
    for process in processes:
        name = process["case"]
        plan = json.loads((directory / f"{name}.plan.json").read_text())
        if process["status"] == "skipped":
            if plan["admitted"] or process["plan"] != plan:
                raise ValueError("unjustified skip")
            continue
        if process["status"] != "completed" or process["exit_code"] != 0:
            raise ValueError("batch contains failed or timed-out cases")
        raw = json.loads((directory / f"{name}.json").read_text())
        if raw["plan"] != plan or not plan["admitted"]:
            raise ValueError("case disagrees with admitted preflight")
        if raw["security"] != manifest["parameters"]["security"]:
            raise ValueError("cryptographic settings changed")
        if len(raw["measurements"]) != manifest["parameters"]["queries_per_case"]:
            raise ValueError("incomplete query sample")
        if raw["measurements"][0]["record_index"] != 0 or raw["measurements"][1]["record_index"] != plan["rows"] - 1:
            raise ValueError("edge rows not exercised")
        for sample in raw["measurements"]:
            if sample["request_bytes"] != 44 + 4 * plan["matrix_rows"] or sample["response_bytes"] != 44 + 4 * plan["matrix_columns"]:
                raise ValueError("framed bytes disagree with geometry")
        if raw["client_public_arrays_bytes"] != 4 * 1275 * (plan["matrix_rows"] + plan["matrix_columns"]):
            raise ValueError("client state accounting disagrees with geometry")
    if summarise(directory) != json.loads((directory / "summary.json").read_text()):
        raise ValueError("derived summary differs from retained raw inputs")
    return sum(p["status"] == "completed" for p in processes), sum(p["status"] == "skipped" for p in processes)


if __name__ == "__main__":
    directory = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "raw/run"
    inventory = ROOT / "checksums.json"
    checked = 0
    if inventory.exists():
        checksums = json.loads(inventory.read_text())
        actual = {str(p.relative_to(ROOT)) for p in ROOT.rglob("*")
                  if p.is_file() and p.name != "checksums.json" and "__pycache__" not in p.parts}
        if actual != set(checksums):
            raise ValueError("retained file inventory differs from checksums")
        for name, digest in checksums.items():
            path = (ROOT / name).resolve()
            if not path.is_relative_to(ROOT) or hashlib.sha256(path.read_bytes()).hexdigest() != digest:
                raise ValueError(f"checksum mismatch: {name}")
            checked += 1
    completed, skipped = check(directory)
    print(f"verified {checked} checksums, {completed} completed cases and {skipped} justified skips")
