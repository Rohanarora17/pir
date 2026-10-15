# ADR-004: Candidate complete storage-proof rows with a shared account proof

- Status: `proposed`
- Date: `2026-10-15`
- Proposer: assistant; final acceptance pending backend evidence and Rohan's review
- Authorization: Rohan approved the Day 6 measurement and candidate encode/pad/decode experiment
- Linked question: `RQ-004`
- Evidence: [EXP-005](../../experiments/EXP-005-record-geometry/summary.md)

## Proposal

For the first PIR feasibility probe, retrieve a complete storage-proof bundle per logical record.
Retain the common account proof once in the public snapshot manifest, independently of the selected
wallet. Recover the record's existing schema, snapshot, index, key and value fields plus native
storage nodes, then pass it to the accepted pvrpc-verifier. A missing record or unsupported proof
shape fails explicitly. There is no revealing fallback or conversion of a missing proof into zero.

EXP-005 implements a deliberately isolated candidate binary frame with explicit lengths and zero
padding. It preserves the current protocol types and adds no production dependencies or schemas.
Its `PVR0` bytes are an experimental marker, not an accepted wire version. Parsing does not
authenticate fields; the existing verifier still checks independently supplied request context.

## Evidence and provisional geometry

All 32 real records round-trip and verify at tested capacities 3,072/4,096/8,192 bytes. Unpadded
frames range from 2,254 to 2,731 bytes. At 3 KiB, 18.05% of stored bytes are padding; at 4 KiB,
38.54%. Ten rows exceed 2,560 bytes. The shared account proof totals 3,600 raw bytes.

Use these capacities as inputs to the backend probe, not as an accepted maximum Ethereum proof
size. Final geometry must state alignment/chunking, a declared supported corpus, admission limits,
measured backend costs and behavior when future proofs exceed those limits. Those facts are pending.

## Alternatives and tradeoffs

Current compact hex JSON preserves existing tooling but needs 4,599–5,545 bytes per row here.
Binary lengths avoid hex expansion but require a carefully bounded decoder. The decoder rejects
unknown schema, truncated frames, nonzero padding, extra payload bytes and oversized node fields.

Unique storage nodes save 26,600 raw bytes on this corpus; a simple hash/length/node store would
occupy 52,734 bytes before PIR padding and indexing. A private node-traversal protocol needs path
discovery, fixed retrieval budgets and missing-node handling. Hybrid publication likewise needs
an explicit privacy model. Both remain comparison candidates rather than implicit implementations.

Whole-dataset download remains a required baseline: all unpadded candidate frames plus the
experiment's compact manifest occupy 88,357 bytes. No network or performance advantage follows
from the row-size measurements.

## Acceptance and revisit conditions

Keep this record proposed until a usable PIR backend is reproduced, its licence/build and row
constraints are recorded, and representation/geometry are reviewed. Day 9 will compare complete
bundles, trie nodes and whole-shard retrieval. Revisit for larger proofs, dominant duplication,
backend chunking constraints, malformed-frame findings or a revised protocol representation.
Consensus-root authentication, final PIR parameters, snapshot publication and repository licence
remain separate decisions.
