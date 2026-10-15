#!/usr/bin/env python3
"""Derive payload-only estimates from preserved measurements, never charge cache per read."""
import json
import math
from pathlib import Path
import sys


def communication(database, setup, online, reads, generations=1):
    if min(database, setup, online) < 0 or reads < 1 or generations < 1:
        raise ValueError("positive workload and nonnegative byte counts required")
    pir = generations * (setup + reads * online)
    cache = generations * database
    return {"reads_per_generation": reads, "generations": generations,
            "pir_min_estimated_bytes": pir, "cached_download_bytes": cache,
            "pir_estimate_lower": pir < cache}


def last_lower_read_count(database, setup, online):
    if online <= 0:
        raise ValueError("online bytes must be positive")
    return max(0, (database - setup - 1) // online)


def percentile(values, p):
    if not values:
        raise ValueError("empty timing samples")
    values = sorted(values)
    return values[math.ceil(len(values) * p) - 1]


def summarise(directory):
    directory = Path(directory)
    processes = json.loads((directory / "processes.json").read_text())
    manifest = json.loads((directory / "manifest.json").read_text())
    results = []
    for process in processes:
        if process["status"] != "completed":
            results.append(process)
            continue
        item = json.loads((directory / f"{process['case']}.json").read_text())
        if item["status"] != "completed":
            raise ValueError("completed process did not produce completed case")
        measurements = item["measurements"]
        if len(measurements) != manifest["parameters"]["queries_per_case"]:
            raise ValueError("incomplete query sample")
        pairs = {(m["request_bytes"], m["response_bytes"]) for m in measurements}
        if len(pairs) != 1:
            raise ValueError("per-case frame sizes changed")
        request, response = pairs.pop()
        plan = item["plan"]
        online = request + response
        setup = item["setup_payload_min_bytes"]
        database = plan["database_payload_bytes"]
        warm = [m["total_local_us"] for m in measurements[1:]]
        results.append({"case": process["case"], "status": "completed", "records": plan["rows"],
            "record_bytes": plan["record_bytes"], "matrix_rows": plan["matrix_rows"], "matrix_columns": plan["matrix_columns"],
            "database_payload_bytes": database, "setup_payload_min_bytes": setup,
            "online_request_bytes": request, "online_response_bytes": response,
            "online_total_bytes": online, "client_public_arrays_bytes": item["client_public_arrays_bytes"],
            "estimated_peak_working_bytes": plan["estimated_peak_working_bytes"],
            "peak_process_rss_bytes": process["peak_process_rss_bytes"],
            "rss_above_preflight_estimate": process["peak_process_rss_bytes"] > plan["estimated_peak_working_bytes"],
            "server_setup_us": item["server_setup_us"], "client_setup_us": item["client_setup_us"],
            "first_local_read_us": measurements[0]["total_local_us"],
            "subsequent_sample_count": len(warm), "subsequent_local_p50_us": percentile(warm, 0.5),
            "subsequent_local_p95_us": percentile(warm, 0.95),
            "max_reads_with_strictly_lower_estimated_bytes": last_lower_read_count(database, setup, online),
            "workload_models": [communication(database, setup, online, q) for q in (1, 10, 100, 1000)],
            "full_rebuild_ten_generation_model": communication(database, setup, online, 10, 10),
            "queries_completed": item["queries_completed"]})
    return {"experiment_id": "EXP-008", "claim_type": "synthetic result",
        "communication_scope": "payload only; optimistic canonical setup minimum; no real manifest/address map/network",
        "timing_scope": "local compute/framing; no proof verification or transport",
        "generation_model_scope": "arithmetic independent full rebuild per generation, not measured transitions",
        "cases": results}


if __name__ == "__main__":
    output = Path(sys.argv[1])
    result = summarise(output)
    (output / "summary.json").write_text(json.dumps(result, indent=2) + "\n")
    for case in result["cases"]:
        if case["status"] == "completed":
            print(f"{case['case']}: database={case['database_payload_bytes']} setup_min={case['setup_payload_min_bytes']} "
                  f"online={case['online_total_bytes']} last_lower_read={case['max_reads_with_strictly_lower_estimated_bytes']} "
                  f"p50_us={case['subsequent_local_p50_us']} rss={case['peak_process_rss_bytes']}")
        else:
            print(f"{case['case']}: {case['status']}")
