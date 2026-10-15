# EXP-006: Private retrieval works; direct indexing is the smaller baseline

All **256 reads** completed correctly across eight cases: direct index PIR and two-segment keyword
IKPIR, using both SimplePIR and FrodoPIR, with 3,072- and 4,096-byte real proof records. Each case
recovers the same 24 nonzero balances and eight proven absences under the retained supplied root.
This is a local client/server computation with actual query/response framing, not a network service
or a consensus-authenticated Ethereum read.

## Main finding

Use direct index PIR for the next immutable-snapshot prototype. Keep full keyword IKPIR as an
experimental comparison. At 3 KiB, direct reads encode 12,504 online bytes versus 98,720 for keyword
reads. Keyword mode returns a bucket in each of two segments, with four slots per bucket; it pays
for more than one proof-sized value per logical lookup. This result concerns the exact declared
50%-loaded, 64-slot keyword table, not an optimized or universally best keyword configuration.

SimplePIR has the smaller setup hint at both tested record sizes. The next prototype can use it
behind the isolated adapter, but [ADR-005](../../docs/decisions/ADR-005-index-pir-baseline.md) remains
proposed. Neither the final backend nor record geometry is frozen by this experiment.

## Byte and memory results

All byte counts are exact integers. Setup bytes are upstream canonical minimum estimates plus
the 32-byte preprocessing tag; setup objects have not been serialized or transferred over a network.
They exclude the common manifest and complete public logical-index map. Query and response counts
include our actual bounded frame headers, vector counts and lengths.

| Engine / mode | Record bytes | Setup minimum bytes | Request bytes | Response bytes | Client public arrays bytes | Whole-process peak RSS bytes |
|---|---:|---:|---:|---:|---:|---:|
| Simple / index | 3072 | 15667284 | 172 | 12332 | 15830400 | 34570240 |
| Frodo / index | 3072 | 19243072 | 172 | 12332 | 19443456 | 41762816 |
| Simple / keyword | 3072 | 125664173 | 112 | 98608 | 125745600 | 387383296 |
| Frodo / keyword | 3072 | 154345093 | 112 | 98608 | 154445184 | 474218496 |
| Simple / index | 4096 | 20889684 | 172 | 16428 | 21052800 | 45154304 |
| Frodo / index | 4096 | 25657408 | 172 | 16428 | 25857792 | 55017472 |
| Simple / keyword | 4096 | 167443373 | 112 | 131376 | 167524800 | 516046848 |
| Frodo / keyword | 4096 | 205659781 | 112 | 131376 | 205759872 | 630915072 |

Client arrays count the hint and expanded public matrix only. They exclude private pending-query
allocation, allocator overhead and code. RSS is the peak of a separate process holding **both**
server and client, including transient setup copies; it must not be presented as client-only RAM.
Separate per-case processes prevent earlier cases' peak RSS from contaminating later measurements.

## Local timings

These are nearest-rank p50/p95 times over 32 sequential, cold reads. They include query construction,
both frame encoders/decoders, server answering, client PIR decoding and record/proof verification.
They exclude setup, fixture loading and all transport latency. No prepared-query/decode queues are
used. Setup runs single-threaded reference methods, once per case. This is an exploratory run on
the recorded arm64/macOS host, not a throughput or comparative production benchmark.

| Engine / mode | Record bytes | Server setup us | Setup export us | Client setup us | Local p50 us | Local p95 us |
|---|---:|---:|---:|---:|---:|---:|
| Simple / index | 3072 | 16393 | 0 | 1766 | 413 | 511 |
| Frodo / index | 3072 | 12987 | 0 | 1521 | 384 | 504 |
| Simple / keyword | 3072 | 31997 | 8796 | 11014 | 2122 | 2284 |
| Frodo / keyword | 3072 | 37011 | 9844 | 14814 | 2580 | 2785 |
| Simple / index | 4096 | 14091 | 0 | 1575 | 415 | 430 |
| Frodo / index | 4096 | 17416 | 0 | 1852 | 490 | 537 |
| Simple / keyword | 4096 | 40882 | 10224 | 14365 | 2970 | 3131 |
| Frodo / keyword | 4096 | 50224 | 13044 | 20990 | 3600 | 3775 |

Export measures the upstream keyword server's setup-bundle clone. Direct mode passes its params
and hint to client setup without that bundle-copy step. These timings do not identify a universally
faster cryptographic engine; record width, database shape, cache state and setup policy matter.

## Whole-dataset download is the right bandwidth choice for this corpus

The complete padded database plus public manifest and logical-index map is **109,390 bytes** at
3 KiB, or **142,158 bytes** at 4 KiB. The smaller direct SimplePIR setup alone is approximately
15.7 MB at 3 KiB, before any online reads. A client can download the whole immutable generation
once and cache it, keeping subsequent selections local. It is incorrect to invent a PIR break-even
by charging a full-database download again for every read of the same cached generation.

PIR remains useful to investigate for larger corpora and clients with different storage constraints.
This experiment establishes compatibility and a baseline, not a bandwidth advantage for 32 rows.

## Implemented boundary and verification

The experimental adapter wraps the pinned engines. Query and response packets contain a public
preprocessing tag and opaque numeric vectors, with exact expected segment counts and dimensions.
No wallet, storage key or plaintext logical index is placed in the request frame. A tag mismatch,
truncated frame, excess data or wrong vector shape fails before backend invocation. The tag is
assigned locally from snapshot, database digest and public preprocessing parameters; an authenticated
generation manifest carrying those commitments remains future work. A hash tag alone is not consensus
authentication or a security analysis of malicious setup.

Recovered bytes must equal the source frame and pass its decoder and the native proof verifier.
Missing keyword results are errors rather than authenticated zero. A session performs one query,
answer and decode before the next query, preserving upstream's single-in-flight-secret invariant.
Keyword mode freezes upstream epoch 0 and queries both segments every time. Our logical snapshot
generation is 1; it is distinct from the backend's mutation epoch. No live update or delta replay
is introduced into a published immutable snapshot.

Five adapter integration tests cover exact framing, truncated prefixes, incorrect bindings and
dimensions, vector limits, and real direct/keyword SimplePIR reads. The retained release run covers
all eight engine/mode/size combinations. Every case records invalid-query-length, cross-preprocessing
and truncated-response rejection. Raw measurement files, child stderr and exit/RSS records are
retained under `raw/run/`; `raw/summary.json` is generated by `summarize.py`. Independent read-only
review found no blocking adapter issue. Correctness tests do not prove query privacy; the privacy
claim remains conditional on the upstream scheme, parameters and threat model.

## Source, licence and limitations

The upstream source is pinned to `fcfb2b540c1cfa92c2de5ec818db3aa9fc817395`, retained as a checksummed
archive with LICENSE/NOTICE and verified unchanged on extraction. The declared source licence is
Apache-2.0. The normal dependency graph's declared licences are recorded in `raw/license-tree.txt`;
see [licence and distribution notes](license-review.md). This is source/build evidence, not a legal
opinion, independent security audit or completed project distribution policy. Project-owned crates
still have no chosen repository licence.

Default upstream LWE dimensions are preserved: 1,275 for SimplePIR (sigma 6.4), 1,566 for FrodoPIR,
with implicit ciphertext modulus 2^32. All cells use 8 plaintext bits, checked against upstream's
geometry-dependent correctness bound (kappa 40); that correctness target is separate from its
claimed security level. Keyword fingerprints are 64 bits. No parameter is weakened for speed.

Public setup objects are still in-process, no HTTP/TCP service exists, no consensus light client
authenticates the root, and no availability/freshness guarantee is established. Timings use one
small real snapshot and no concurrent clients. These limitations bound the result.

## Reproduce and continue

From the project directory, use a new output directory:

```bash
OUTPUT_DIR=/private/tmp/EXP-006-next sh experiments/EXP-006-pir-baseline/commands.sh
python3 experiments/EXP-006-pir-baseline/summarize.py /private/tmp/EXP-006-next
```

The existing sandbox cache can be reused by setting `CARGO_HOME=/private/tmp/pvrpc-verifier-cargo`
and `CARGO_TARGET_DIR=/private/tmp/pvrpc-pir-target`. Runtime queries make no RPC/network requests;
initial dependency downloads may use the network if not cached.

Day 7's local feasibility/build/licence-evidence task is complete. Next is Day 8's consensus-light-
client interface. Before accepting the final PIR backend, benchmark larger declared corpora and
compare setup amortization, full-generation caching and backend geometry. Network setup encoding,
generation commitments, concurrency and production admission remain integration work.

A useful learning exercise is to explain why our 12.5 KB direct query/response is not enough to
claim a bandwidth win: the hint download and the ability to cache the complete generation change
the comparison.

## Pre-push source-integrity review: 2026-10-15

The packaging review found that source preparation compared archive-member bytes but admitted
extra build inputs or matching-byte symlinks in an already extracted directory. Preparation now
requires the exact archived inventory and file types as well as matching bytes. Four Python
regressions cover clean reuse, modified source, an added Cargo build script and a matching-byte
symlink. The two bypass regressions failed before the fix and all four pass after it. The
reproduction command runs these tests before compiling. Retained upstream archives and original
benchmark output are unchanged; this is source-admission hardening, not a new performance claim.
