# Phase 1 decision matrix

This document compares only evidence recorded by the project. `Not reviewed` means no current source
assessment exists. `Not measured` means a source may make a claim, but the result has not been
reproduced using this project's record shape and hardware. No option is ranked until its linked
research question reaches its decision criterion.

## Proof verifier

Linked question: `RQ-003`

| Option | Correctness evidence | Integration cost | Runtime requirements | Licence | Maintenance | Measured result | Decision status |
|---|---|---|---|---|---|---|---|
| Helios or Alloy verification path | Prior source inspection only | Not reviewed | Not reviewed | Not reviewed | Revision refresh required | Not measured | Open |
| Alternative maintained Rust MPT verifier | Not reviewed | Not reviewed | Not reviewed | Not reviewed | Not reviewed | Not measured | Open |
| Minimal local verifier | No implementation | High unless reusable options fail | Rust client | Project licence remains open | Project maintained | Not measured | Fallback only |

## Record representation

Linked question: `RQ-004`

| Option | Correctness evidence | Integration cost | Runtime requirements | Licence | Maintenance | Measured result | Decision status |
|---|---|---|---|---|---|---|---|
| Padded complete proof bundle | Compatible with native proof concept | Lowest initial integration | One private retrieval per record if it fits | Project licence remains open | Project maintained | Not measured on real corpus | Open |
| Content-addressed trie nodes | Compatible with native trie traversal in principle | Multiple private reads and padding policy | Fixed traversal budget required | Project licence remains open | Project maintained | Not measured | Open |
| Hybrid public and private nodes | Correctness boundary not specified | Highest design cost | Public cache plus private backend | Project licence remains open | Project maintained | Not measured | Open |

## PIR backend

Linked questions: `RQ-005`, `RQ-006`

| Option | Correctness evidence | Integration cost | Runtime requirements | Licence | Maintenance | Measured result | Decision status |
|---|---|---|---|---|---|---|---|
| IKPIR | Prior source inspection | Not reviewed for proof records | CPU-first path to be reproduced | Apache-2.0 at prior inspected revision, refresh required | Revision refresh required | Not measured end to end | Open |
| insPIRe with serving layer | CPU client smoke only | CUDA server and serving integration | NVIDIA GPU for server path | Distribution terms not established at prior inspection | Revision refresh required | Author-reported only | Open |
| Another single-server PIR backend | Not reviewed | Not reviewed | Not reviewed | Not reviewed | Not reviewed | Not measured | Open |

## Light client adapter

Linked question: `RQ-007`

| Option | Correctness evidence | Integration cost | Runtime requirements | Licence | Maintenance | Measured result | Decision status |
|---|---|---|---|---|---|---|---|
| Embedded Helios library | Prior source inspection | Not reviewed against current API | Rust process with consensus sync | Not reviewed at current revision | Revision refresh required | Not measured | Open |
| Local Helios process adapter | Prior source inspection | Versioned process boundary required | Separate local process | Not reviewed at current revision | Revision refresh required | Not measured | Open |
| Fixture adapter | Deterministic test input only | Low | No live sync | Project licence remains open | Project maintained | Not a final authentication path | Test-only option |
