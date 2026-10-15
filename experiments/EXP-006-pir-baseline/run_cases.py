#!/usr/bin/env python3
"""Run each real-corpus case in its own POSIX process and preserve raw output/RSS."""
import datetime
import json
import os
import pathlib
import sys
import time

if len(sys.argv) != 4:
    raise SystemExit("usage: run_cases.py BINARY CORPUS_DIRECTORY NEW_OUTPUT_DIRECTORY")
binary = pathlib.Path(sys.argv[1]).resolve(strict=True)
corpus = pathlib.Path(sys.argv[2]).resolve(strict=True)
output = pathlib.Path(sys.argv[3])
output.mkdir(parents=True, exist_ok=False)
if sys.platform not in ("darwin", "linux"):
    raise SystemExit("peak RSS normalization is supported only on macOS/Linux")
records = []
for size in (3072, 4096):
    for engine in ("simple", "frodo"):
        for mode in ("index", "keyword"):
            name = f"{engine}-{mode}-{size}"
            started = datetime.datetime.now(datetime.timezone.utc).isoformat()
            begin = time.perf_counter_ns()
            with (output / f"{name}.json").open("xb") as stdout, (output / f"{name}.stderr.log").open("xb") as stderr:
                pid = os.fork()
                if pid == 0:
                    os.dup2(stdout.fileno(), 1)
                    os.dup2(stderr.fileno(), 2)
                    os.execv(str(binary), [str(binary), str(corpus), engine, mode, str(size)])
                _, status, usage = os.wait4(pid, 0)
            exit_code = os.waitstatus_to_exitcode(status)
            records.append({
                "case": name, "started_at": started,
                "completed_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
                "exit_code": exit_code, "wall_ns_including_process_start_and_fixture_io": time.perf_counter_ns() - begin,
                "peak_process_rss_bytes": usage.ru_maxrss * (1024 if sys.platform == "linux" else 1),
                "user_cpu_seconds": usage.ru_utime, "system_cpu_seconds": usage.ru_stime,
                "rss_scope": "whole local process: server, client, fixtures, transient setup copies and code",
            })
            (output / "processes.json").write_text(json.dumps(records, indent=2) + "\n")
            if exit_code:
                raise SystemExit(f"case {name} failed; raw stdout/stderr retained")
            print(f"completed {name}", flush=True)
