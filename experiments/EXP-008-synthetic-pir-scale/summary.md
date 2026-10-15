# EXP-008: A communication window appears, with substantial client memory costs

Claim type: **synthetic result**. This is the first Day 9 scale slice, not the completed
bundled-proof versus shared-node comparison. It does not authenticate Ethereum state or settle the
final PIR backend/layout.

## What ran

The unchanged pinned SimplePIR backend ran at the default LWE dimension 1275, sigma 6.4, 8-bit
plaintext cells and upstream correctness target 40. Six admitted record counts from 32 to 32,768
were tested at each of two widths, 3,072 and 4,096 bytes. All **384 sampled reads** recovered exactly
the generated expected bytes across **12 completed cases**. Every case includes the first and last
record, including partially filled final reshaped rows. All database hashes and parameters are
retained. These are artificial bytes, not 32,768 real Ethereum balance proofs.

Four larger cases, 131,072 and 524,288 records at each width, were skipped before database allocation
because the estimated working set exceeded the declared 1 GiB budget. No case failed or timed out.
Actual whole-process peak RSS stayed below the estimate in every admitted case, ranging from
33,685,504 to 777,568,256 bytes. That measurement includes server, client, transient setup copies
and code. It is not client RAM alone or an enforced process-memory limit.

## Communication and client state

Selected cases, in exact bytes. Setup remains the upstream canonical **minimum estimate**, not an
actual transferred setup message. Online bytes are actual framed query plus reply bytes.

| Records | Record bytes | Payload download once | Setup minimum | Online per read | Client public arrays | Last read count with lower PIR estimate |
|---:|---:|---:|---:|---:|---:|---:|
| 32 | 3,072 | 98,304 | 15,667,284 | 12,504 | 15,830,400 | None |
| 8,192 | 3,072 | 25,165,824 | 31,334,484 | 41,048 | 52,224,000 | None |
| 8,192 | 4,096 | 33,554,432 | 20,889,684 | 49,240 | 62,668,800 | 257 |
| 32,768 | 3,072 | 100,663,296 | 47,001,684 | 80,644 | 102,708,900 | 665 |
| 32,768 | 4,096 | 134,217,728 | 62,668,884 | 92,932 | 118,376,100 | 769 |

For 32,768 records of 3,072 bytes, 100 reads cost an estimated minimum **55,066,084 bytes** through
PIR, compared with **100,663,296 bytes** to download and cache the payload once. At 1,000 reads,
PIR's estimate grows to **127,645,684 bytes**, while the cached-download cost stays unchanged.
The strict threshold is 665 reads per snapshot. This describes the optimistic byte model for this
case, not a measured network crossover or a general recommendation.

Client public arrays are the hint plus locally expanded public matrix. The 3,072-byte, 32,768-record
case already uses more public-array memory than the entire byte payload. Even the 4,096-byte case
at 8,192 records has lower estimated communication but substantially larger client state. Therefore
this result does not establish suitability for a storage-constrained light client. Private query
state, allocator overhead, real address maps and application memory add further costs.

A useful geometry observation is that 4,096-byte rows have a smaller setup than 3,072-byte rows at
8,192 records. The pinned backend chooses different internal reshape dimensions: 8192 x 4096 versus
4096 x 6144. Wider records alone do not determine hint cost. This supports measuring the actual
backend shape rather than freezing a record size from proof padding percentages alone. It does
not justify padding records more widely without comparing the full storage/workload tradeoff.

## Local processing

One setup per case and 32 sequential queries were measured on the recorded arm64/macOS environment.
No precomputation queues or parallel setup were used. At 32,768 records, server setup took
15.418 seconds for 3,072-byte rows and 23.827 seconds for 4,096-byte rows. Subsequent-read p50/p95
were 10.160/11.778 ms and 13.105/13.927 ms respectively. Each percentile uses only 31 samples after
the separately reported first read. These are exploratory local query/framing/answer/decode timings,
with no transport or Ethereum proof verification. The synthetic-data generation and expected-byte
comparison are outside those read timings.

A complete cached-download latency comparison cannot be inferred from these compute timings.
The machine metadata could not obtain host RAM or a specific CPU model from the sandbox. The
manifest records the available processor identifier and architecture instead of inventing details.

## Accounting and verification

The comparison charges `setup_min + reads * online_bytes` for PIR and one payload download per
cached immutable snapshot. A ten-generation full-rebuild calculation is retained as an arithmetic
model only. Neither real Ethereum update costs nor a working generation-transition protocol was
measured. Logical record positions are implicit in this generator, so real manifests and address
maps are excluded from both paths. Missing metadata cannot be assumed cheap in the final design.

Three Rust tests cover preallocation admission/limits, the deterministic generator and sample
positions, and actual fixed-security recovery at a partially populated reshaped boundary. Four
Python tests cover fair caching, equality/threshold rounding, charging both methods for fresh
snapshots and percentile/invalid-input handling. Three additional Python regressions reject missing cases, changed security settings
and edited summary results. Formatting and Clippy with warnings denied pass.
The offline auditor checks case completeness, source fingerprints, security parameters, frame
sizes, client-array accounting, edge samples and regeneration of the retained summary. It also
checks the retained file inventory's SHA-256 hashes.

The existing real-proof integration is separate in EXP-006. Passing these synthetic correctness
tests is not a privacy proof, security audit or live consensus authentication result.

## Pending work and decisions

RQ-008 is now investigating. It is not answered: the useful region still needs measured setup
serialization, real metadata, client storage constraints, network conditions and broader workloads.
Next compare complete proofs with shared proof parts on declared real data, including private
request counts and path-leakage constraints. Larger real corpora, other backends, ordinary RPC,
network serving and measured snapshot transitions remain pending. ADR-004 and ADR-005 remain
proposed. Day 9 is in progress, and the Phase 1 gate has not been evaluated.

Reproduce using the README command with a fresh output directory. Audit the retained evidence:

```sh
python3 experiments/EXP-008-synthetic-pir-scale/check_evidence.py
```

A useful participation exercise is to explain why the 32,768-record case can save communication
at 100 reads but lose that saving at 1,000, and why this alone does not establish a client-memory
benefit. This is offered, not recorded as demonstrated understanding.
