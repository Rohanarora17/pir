# EXP-009: Guarded larger local SimplePIR cases

Research questions: RQ-005, RQ-008. Claim type: synthetic result.

Rohan authorised trying the larger configurations locally first, with laptop stability as the
constraint, and moving later work to VMs afterwards. This experiment reuses EXP-008's unchanged
release worker, generator, default security settings and byte accounting. Worker JSON retains its
original EXP-008 identifier, as does the raw summary reused from its accounting utility.
The enclosing manifest and telemetry identify this guarded run as
EXP-009. Existing retained results are not overwritten.

## Method and limits declared before the run

Check macOS hardware, memory pressure, free-percentage indicator and swap before starting. The
pressure sysctl returns dispatch flags: normal 1, warning 2, critical 4. The local SDK's
`dispatch/source.h` and Apple's XNU pressure-conversion/sysctl implementation establish that
interpretation. See [Apple source](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_memorystatus_notify.c).

Run the four previously skipped configurations, ordered from smaller to larger: 131,072 and
524,288 rows, each at 3,072 and 4,096 bytes. A case is admitted only if:

- Pressure is normal and the free-percentage indicator is at least 30%.
- Its preflight estimate fits the smaller of 3.5 GiB, one sixth of physical RAM, and the memory
  fraction indicated above a 25% reserve. This percentage is an admission proxy, not a measurement
  of immediately allocatable physical bytes.

Only one worker runs at a time, with lowered CPU priority (nice +10). It gets a 240-second timeout.
Poll telemetry and worker RSS approximately every 250 ms. Kill and reap only that child if pressure
is no longer normal, the free-percentage indicator falls below 25%, swap grows by more than
128 MiB from its pre-case baseline, RSS reaches 3 GiB, telemetry fails or the timeout expires.
Stop the batch after such an abort. Recheck all admission conditions before every new case.

This is a watchdog with conservative preflight estimates, not a kernel-enforced memory reservation
or a guarantee against all OS/application failures. No swap settings, caches, running applications
or kernel pressure controls are changed. Existing swap usage alone does not establish current
pressure, so both pressure and swap growth are monitored. Unknown telemetry fails closed.

Record deferred cases distinctly from failed or completed benchmarks. Retain the unmodified worker
stdout/stderr, machine observations, samples, termination reasons, child exit code and peak RSS.
If the laptop is already under pressure, the attempt should record that and allocate no benchmark
database. A stopped run is safety evidence, not a successful performance measurement.

## Reproduce

First prepare/build EXP-008's worker if needed, following its README. Then, from the repository root:

```sh
python3 -m unittest discover -s experiments/EXP-009-guarded-local-scale -p 'test_*.py' -v
python3 experiments/EXP-009-guarded-local-scale/guarded_run.py \
  /private/tmp/pvrpc-pir-target/release/pvrpc-pir-scale /private/tmp/EXP-009-next
```

The output directory must be new. Hardware telemetry may require normal host access outside the
Codex filesystem sandbox. It does not require root or changing OS settings. There is no override
flag for the guard thresholds. Tests use tiny sleeping processes and simulated telemetry to check
termination without applying memory pressure to the laptop.

Use remote VMs for configurations that do not fit this policy. A VM on this same laptop would
still consume the laptop's RAM. No VM is provisioned or billed by these scripts.

Guarded timings include competition from the monitor and deliberately lowered worker priority.
Do not interpret differences from EXP-008 as an isolated backend-speed comparison. The guard tests
need host access for `nice`; an initial sandbox test failure is retained separately from the
successful host tests. A telemetry-access failure defers a load instead of bypassing the checks.

The retained run used `raw/guarded_run-executed.py` (source snapshot only). The current runner
adds immediate termination when RSS cannot be read for a still-live child. The saved run never
encountered missing RSS, and its manifest fingerprint continues to identify the exact executed
source. The seventh tiny-process regression tests the strengthened path without another large run.
