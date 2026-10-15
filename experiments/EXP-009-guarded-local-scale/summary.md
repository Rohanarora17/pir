# EXP-009: Two larger cases completed under a live laptop watchdog

Claim type: **synthetic result**. This extends the EXP-008 scale evidence to 131,072 records.
It does not complete the real-proof representation comparison or establish network performance.

## Local readiness and safety observations

The measured host is an Apple M4 Pro with 24 GiB physical RAM and 14 logical CPUs. The initial
session check found warning-level memory pressure and approximately 7 GiB existing swap usage,
so no large worker was started then. Rohan closed unused apps. The actual per-case admission
samples subsequently recorded normal pressure and a free-memory indicator of 50%.

The two 131,072-record cases were admitted under the declared policy and ran sequentially with
nice +10. **All 64 sampled reads were correct**. Neither worker triggered the watchdog. Across
348 and 432 recorded samples, the pressure flag stayed normal, the minimum free-percentage
indicators were 49% and 44%, and swap usage did not grow from the respective baselines.

Peak whole-process RSS was **2,029,993,984 bytes (1.89 GiB)** for 3,072-byte records and
**2,626,650,112 bytes (2.45 GiB)** for 4,096-byte records. These include both server and client,
transient setup objects and code. They are not client-only RAM. A lack of growth in swap used
does not prove that there was no swap I/O.

Rohan reported **"No noticeable slowdown"** after the runs. This subjective observation is retained
separately from sensor measurements. The final host check still reported normal memory pressure,
no recorded thermal/performance warning, and no remaining benchmark worker. The free-percentage
indicator is not equivalent to immediately allocatable physical RAM.

Both 524,288-record cases were deferred before allocation. Their estimates, 8.52 and 11.16 GiB,
exceed this local policy's maximum 3.5 GiB estimate. These remain untested configurations for a
later remote-VM run. They are not failures of the PIR algorithm or established laptop limits.

## PIR results

The unchanged EXP-008 release worker used default SimplePIR security settings. Both cases used
synthetic byte records, not additional real Ethereum proofs. Every one of the 32 sampled reads
per case was checked against the independently regenerated expected row, including edge records.

| Record bytes | Database payload | Setup minimum estimate | Framed query + reply | Client public arrays | Last read count with lower estimated communication |
|---:|---:|---:|---:|---:|---:|
| 3,072 | 402,653,184 | 109,670,484 | 161,004 | 205,167,900 | 1,819 |
| 4,096 | 536,870,912 | 125,337,684 | 185,776 | 236,752,200 | 2,215 |

For example, at 3,072 bytes, the minimum setup estimate plus 100 reads is **125,770,884 bytes**,
compared with **402,653,184 bytes** to download and cache the payload once. At this size the client
public arrays are also smaller than the complete payload, unlike the 3,072-byte case at 32,768
records. This is a useful change in the measured geometry, not a completed lightweight-client claim.
Private state, allocation overhead and the real metadata/address map add costs that are excluded here.

The strict communication thresholds are derived from fixed frame sizes and an optimistic minimum
setup estimate. Only 32 reads per case were actually executed. The 100/1,000-read and multi-generation
figures are arithmetic models, not separately executed workloads. Setup was not serialised or
transferred over a network. Caching is charged once per snapshot, never once per query.

Server setup took 89.976 seconds and 111.804 seconds. Subsequent local read p50/p95 were
42.088/42.821 ms and 54.377/55.635 ms, based on 31 samples each after the first read. These include
local query/framing/answer/decode, with no transport or Ethereum proof verification. Deliberately
lowered worker priority and the active monitor affect scheduling. Do not treat timing differences
from EXP-008 as an isolated backend comparison. Setup/rebuild time remains material for frequent
snapshot changes.

## Verification and limits

Seven watchdog tests pass on the host. They cover telemetry parsing, refusing unsafe admission,
all runtime thresholds, killing/reaping a tiny sleeping child on simulated pressure, failing closed
on missing telemetry and a healthy child completing with priority lowering enabled. An initial
sandbox-only test failure is retained: that sandbox denied the `nice` call. The same host tests
passed without weakening the priority or memory policy. A post-run review tightened missing-RSS
handling: an unreadable still-running child now stops immediately. A seventh regression covers
that path. Both completed runs had RSS readings in every recorded sample, so their observations
are unaffected. The exact executed runner is retained in `raw/guarded_run-executed.py`, with its
original fingerprint in the run manifest. The current runner includes this additional check.

The offline auditor checks all four planned cases, pre-case admission, guard samples, worker
exit status, fixed security parameters, message geometry, 32 reads per completed case and
regeneration of the retained accounting summary. SHA-256 checksums cover the retained inputs,
source and output files. Original EXP-008 sources and evidence are unchanged.

The watchdog is sampled protection, not an OS-enforced memory reservation or a universal stability
guarantee. No applications were closed by the assistant and no OS memory settings were changed.
These observations justify this bounded local run only.

## Next work

Run the remaining 524,288-record configurations in a remote VM with an explicit memory limit,
adequate OS headroom, cancellation and recorded machine configuration. Keep its results separate
from the laptop timings. No VM has been created or billed. Real proof sharing versus complete
records, setup encoding/network measurements, snapshot transitions and live root authentication
remain pending. Day 9 and RQ-008 remain in progress, with no final backend or record-size freeze.

Audit without rerunning a load:

```sh
python3 experiments/EXP-009-guarded-local-scale/check_evidence.py
```

A useful learning discussion is why the server/client benchmark's 2.45 GiB peak is different from
the client's 236,752,200-byte public-array cost. These are different scopes, not conflicting results.
