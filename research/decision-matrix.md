# Phase 1 decision matrix

This document compares only evidence recorded by the project. `Not reviewed` means no current source
assessment exists. `Not measured` means a source may make a claim, but the result has not been
reproduced using this project's record shape and hardware. No option is ranked until its linked
research question reaches its decision criterion.

## Proof verifier

Linked question: `RQ-003`

| Option | Correctness evidence | Integration cost | Runtime requirements | Licence | Maintenance | Measured result | Decision status |
|---|---|---|---|---|---|---|---|
| alloy-trie 0.9.8 | EXP-004: 32 real proofs, 313 negative/binding checks | Standalone function plus project adapter | Rust library; compiler upgrade acceptable | MIT OR Apache-2.0 | Published source pinned; Helios wrapper inspected | Local release probe, conditional on supplied root | Working comparison reference |
| eth_trie 0.6.1 | EXP-004 plus reusable adapter tests over 32 records and malformed-input regressions | Account/value recovery in separate component; structural decoder guard | Rust library; compiler upgrade acceptable | Apache-2.0 | Published source pinned; one-byte decoder edge guarded and regression tested | 24 inclusions and 8 absences; reordered nodes and rejection tests | Integrated; ADR-003 accepted |
| Minimal local verifier | No implementation | High unless reusable options fail | Rust client | Project licence remains open | Project maintained | Not measured | Fallback only |

## Record representation

Linked question: `RQ-004`

| Option | Correctness evidence | Integration cost | Runtime requirements | Licence | Maintenance | Measured result | Decision status |
|---|---|---|---|---|---|---|---|
| Padded complete proof bundle | EXP-005 round-trips and verifies all 32 real records | Candidate binary codec; production format pending | One logical row retrieval; backend query/chunk count pending | Project licence remains open | Experimental codec with bounds tests | 2254–2731 bytes before padding; 3 KiB fits this corpus | ADR-004 proposed; final geometry open |
| Content-addressed trie nodes | Compatible with native trie traversal in principle | Multiple private reads and padding policy | Fixed traversal budget required | Project licence remains open | Project maintained | EXP-005: 142 unique nodes, 47622 raw bytes; excludes PIR/index overhead | Open; private traversal unimplemented |
| Hybrid public and private nodes | Correctness boundary not specified | Highest design cost | Public cache plus private backend | Project licence remains open | Project maintained | Not measured | Open |

## PIR backend

Linked questions: `RQ-005`, `RQ-006`

| Option | Correctness evidence | Integration cost | Runtime requirements | Licence | Maintenance | Measured result | Decision status |
|---|---|---|---|---|---|---|---|
| Direct SimplePIR/FrodoPIR from pinned IKPIR source | EXP-006 verifies 32 real rows per engine/size | Experimental index adapter and framed vectors; setup/network integration pending | Local CPU baseline reproduced | Apache-2.0 upstream; active declared licence inventory retained | Pinned unaudited pre-1.0 prototype | 3 KiB: 12504 online bytes; Simple setup minimum 15667284 bytes | ADR-005 proposed; final adoption open |
| Full keyword IKPIR | EXP-006 verifies 32 real rows per engine/size | Adds two-segment cuckoo lookup and update-client machinery | Local CPU baseline reproduced; immutable epoch | Apache-2.0 at pinned revision; repository licence still open | Same pinned prototype | 3 KiB: 98720 online bytes; Simple setup minimum 125664173 bytes | Comparison path; final adoption open |
| insPIRe with serving layer | CPU client smoke only | CUDA server and serving integration | NVIDIA GPU for server path | Distribution terms not established at prior inspection | Revision refresh required | Author-reported only | Open |
| Another single-server PIR backend | Not reviewed | Not reviewed | Not reviewed | Not reviewed | Not reviewed | Not measured | Open |

## Light client adapter

Linked question: `RQ-007`

Helios is the accepted implementation choice (ADR-007, Rohan, 15 October 2026). The rows below
compare integration modes within Helios; ADR-006's core-only proposal remains open.

| Option | Correctness evidence | Integration cost | Runtime requirements | Licence | Maintenance | Measured result | Decision status |
|---|---|---|---|---|---|---|---|
| Embedded Helios consensus core | EXP-007: synthetic signed updates, exact root and rejection tests | Private store/verify-before-apply wrapper; sync lifecycle still required | Rust process; future consensus API transport | MIT source plus Apache-2.0 hashing patch retained; full audit open | Revision pinned in SRC-016 | Five synthetic transitions; no live sync | ADR-006 proposed |
| Local Helios process adapter | Current source inspected; no executable process adapter | Versioned endpoint and trusted local provenance required | Separate local process | MIT source retained; dependency policy open | Same pinned revision; finalized channel is hash-only | Not reproduced | Credible alternative; no speed ranking |
| Fixture adapter | EXP-007 minimal Deneb vectors (32 members) | Isolated reproducible harness | No live sync | Retained upstream licences; project licence open | Project maintained | 10 boundary tests, four source-integrity tests | Test-only; not Sepolia authentication |
