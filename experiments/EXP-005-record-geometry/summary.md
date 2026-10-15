# EXP-005 results: candidate geometry

The experimental binary frame needs **2,254–2,731 bytes per row** on the 32-record EXP-003 corpus.
The smallest tested capacity that fits the complete corpus is **3,072 bytes (3 KiB)**. A 4,096-byte
capacity also works. This is evidence for the next PIR probe, not a selected production geometry.
All figures below are local serialized sizes; no PIR traffic, timings or throughput were measured.

## Measured sizes

Raw storage proof nodes occupy 2,064–2,529 bytes per record (4–7 nodes). The binary frame retains all
existing binding fields and node bytes, adding 174 fixed bytes and four length bytes per node.
It does not compress proof data or change proof verification.

Across all 32 rows:

- Raw storage proof bytes: 74,222.
- Binary frames before padding: 80,558; minimum 2,254, median 2,516, p95 2,635, maximum 2,731.
- Compact JSON records: 163,858; minimum 4,599, median 5,118, p95 5,353, maximum 5,545.
- Binary saves 83,300 bytes against compact hex JSON before padding. This measures serialization,
  not a cryptographic improvement or compressed network protocol.

Percentiles use the nearest-rank rule. Each record's exact measurements are in `raw/results.json`.

## Padding and unsupported rows

- 2,048 bytes: no rows fit. The experiment refuses to call this a complete database.
- 2,560 bytes: 22 rows fit; ten are explicitly rejected as oversized.
- 3,072 bytes: all 32 fit in 98,304 bytes. Padding is 17,746 bytes, **18.05% of the padded database**,
  or 22.03% overhead relative to the unpadded binary frames.
- 4,096 bytes: all 32 fit in 131,072 bytes. Padding is 50,514 bytes, **38.54% of the padded database**,
  or 62.71% overhead relative to unpadded binary frames.
- 8,192 bytes: all 32 fit in 262,144 bytes, of which 69.27% is padding. Current JSON also fits here.

Three KiB is the smallest **tested** complete capacity, not the mathematical minimum or an Ethereum
proof-size bound. A larger corpus or later block may exceed it. Encoding a row larger than the
chosen capacity returns `TooLarge`; no node is dropped, no row is published partially and no RPC
fallback exists. A backend may prefer powers of two, split one record across chunks, or impose
different alignment. Its measured cost must determine the eventual layout.

## Shared account proof and duplicated storage nodes

The account proof has nine nodes totaling 3,600 raw bytes. Keeping it once in the snapshot manifest
avoids 111,600 raw duplicated bytes compared with copying it into all 32 rows. This is already
consistent with the protocol's manifest/account-proof separation. A public manifest is read for
the snapshot, independent of the requested wallet; its bytes are not free but can be cached.

Storage proofs have 192 node references but only 142 distinct byte strings. Unique nodes occupy
47,622 bytes; repeated references account for 26,600 bytes, or 35.84% of raw storage-proof bytes.
A simple unique-node store with a 32-byte hash and four-byte length per node would occupy 52,734
bytes **before PIR row padding, indexing and protocol overhead**. This establishes potential
storage savings only. It does not establish a private traversal protocol, query count, privacy
budget or performance win. Those comparisons remain Day 9 work.

## Whole-dataset baseline

Downloading all unpadded binary frames plus the experiment's compact public manifest would transfer
88,357 serialized bytes, excluding framing/transport overhead. The manifest is 7,799 bytes with the
4,096-byte row setting and structural PIR placeholders. This small 32-row corpus makes a whole-
dataset baseline important when we measure PIR; geometry alone does not establish a PIR advantage.

## Correctness evidence and limits

The program verifies every original record, serializes/decodes an exact frame, compares all fields,
then verifies the recovered record. For each tested padded capacity it verifies every supported row.
All 32 rows survive the 3/4/8 KiB round trips: 24 inclusions and eight authenticated absences under
the supplied root. The execution root remains provider-reported; consensus authentication is absent.

Six integration tests cover framing boundaries, every truncated prefix of a test row, header lengths,
unknown schema, nonzero padding, hostile counts/lengths, real round trips and 160 forged-value/key/
generation/index/proof cases that still parse but fail the existing verifier. Synthetic framing
controls are test inputs, not Ethereum results. Independent read-only review found no blocking issue.

The fixture loader uses generation 1 and structural PIR placeholders to exercise the current
protocol types. These values select no backend or security parameters. There is no snapshot
publication, PIR preprocessing, server retrieval, consensus synchronization or performance benchmark.

## Recommendation and next work

Carry the complete storage-proof bundle with a shared public account proof into the first PIR probe.
Test both 3,072- and 4,096-byte capacities if the backend supports them; enforce explicit oversize
failure. [Proposed ADR-004](../../docs/decisions/ADR-004-candidate-record-layout.md) records the
candidate and its boundaries. Rohan authorized this experiment; final representation acceptance and
geometry remain open pending backend evidence and review. Day 6's measurement/candidate task is
complete; RQ-004 remains investigating. Day 7 reproduces an actual PIR server/client query.

Reproduce from the project directory, using a fresh output path:

```bash
OUTPUT_FILE=/private/tmp/EXP-005-next.json sh experiments/EXP-005-record-geometry/commands.sh
```

For a learning walkthrough, compare the 17,746 padding bytes at 3 KiB with the 26,600 duplicated
storage-node bytes. Removing either cost has different consequences: a tighter row limit rejects
more proof shapes, while node deduplication needs a different retrieval design. Neither choice
may be justified from byte counts alone.
