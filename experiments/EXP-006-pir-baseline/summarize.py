#!/usr/bin/env python3
"""Validate and summarize retained case outputs; never modify the raw measurements."""
import json
import math
import pathlib
import sys

def distribution(values):
    values = sorted(values)
    return {"min": values[0], "median_nearest_rank": values[math.ceil(len(values)*0.5)-1],
            "p95_nearest_rank": values[math.ceil(len(values)*0.95)-1],
            "max": values[-1], "mean": sum(values)/len(values)}

def summarize(directory):
    processes = {p["case"]: p for p in json.loads((directory/"processes.json").read_text())}
    output = []
    for size in (3072, 4096):
        for engine in ("simple", "frodo"):
            for mode in ("index", "keyword"):
                name = f"{engine}-{mode}-{size}"
                case = json.loads((directory/f"{name}.json").read_text())
                if processes[name]["exit_code"] != 0 or case["queries_completed"] != 32 or (case["verified_inclusions"],case["verified_absences"]) != (24,8):
                    raise ValueError(f"incomplete or unsuccessful case: {name}")
                measurements = case["measurements"]
                if len(measurements) != 32 or {m["record_index"] for m in measurements} != set(range(32)):
                    raise ValueError(f"missing query measurements: {name}")
                if not all(case[k] for k in ("invalid_query_rejected","cross_preprocessing_query_rejected","truncated_response_rejected")):
                    raise ValueError(f"admission checks failed: {name}")
                online = [m["request_bytes"]+m["response_bytes"] for m in measurements]
                output.append({"case":name,"record_size_bytes":size,"verified_reads":32,
                    "setup_payload_min_bytes":case["setup"]["setup_payload_min_bytes"],
                    "public_manifest_and_index_map_bytes":case["public_manifest_bytes"]+case["public_logical_index_map_bytes"],
                    "client_public_arrays_bytes":case["setup"]["client_public_arrays_bytes"],
                    "peak_whole_process_rss_bytes":processes[name]["peak_process_rss_bytes"],
                    "server_setup_us":case["setup"]["server_setup_us"],
                    "setup_bundle_export_us":case["setup"]["bundle_export_us"],
                    "client_setup_us":case["setup"]["client_setup_us"],
                    "request_bytes":distribution([m["request_bytes"] for m in measurements]),
                    "response_bytes":distribution([m["response_bytes"] for m in measurements]),
                    "online_total_bytes":distribution(online),
                    "local_total_us":distribution([m["total_local_us"] for m in measurements]),
                    "whole_dataset_download_once_bytes":case["full_padded_dataset_download_bytes"]})
    return {"cases":output,"verified_reads_total":sum(c["verified_reads"] for c in output),
            "notes":["Nearest-rank percentiles over 32 cold sequential reads per case; no transport latency.",
                     "Setup sizes are canonical minimum estimates; query/response sizes are actually encoded frames.",
                     "RSS is the whole process, not client-only; public arrays exclude allocator and private-state overhead.",
                     "Whole-dataset download can be cached once per immutable generation; do not compare against re-downloading it for every read."]}

if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: summarize.py RUN_DIRECTORY")
    print(json.dumps(summarize(pathlib.Path(sys.argv[1])),indent=2,sort_keys=True))
