# Project roadmap

## Current status

- Active phase: Phase 1
- Active day: Day 3
- Current gate: not evaluated
- Completed evidence: Day 1 research scaffolding and tests, plus the Day 2 Sepolia WETH workload experiment and accepted workload decision
- Next decision: exact block-hash pinning support for `eth_getProof`

The day number records project progress. It does not advance merely because a calendar day has
passed. This status is updated only when evidence is saved, a decision is accepted or a phase gate
is evaluated.

## Four-phase plan

| Phase | Working days | Objective | Research component | Main deliverable | Gate |
|---|---:|---|---|---|---|
| Phase 1 | 1–10 | Resolve the choices that affect every later component. | Reproduce a real proof, measure proof shape, test PIR feasibility and define the light client boundary. | Locally verified proof, reproduced PIR query and accepted interface decisions. | Real proof verification, reproducible corpus, usable PIR backend, recorded licence review, precise light client contract and measured record layout. |
| Phase 2 | 11–20 | Build a verified snapshot pipeline for one declared corpus and one pinned finalised block. | Measure the correctness and cost of converting native proofs into fixed-shape records. | Immutable snapshot generation containing only preverified proof records. | A clean build publishes one internally consistent generation and never exposes a partial build. |
| Phase 3 | 21–30 | Compose private retrieval, proof verification and consensus-authenticated roots. | Compare ordinary RPC, whole-dataset download and CPU PIR on the same corpus and root. | One complete private and verified WETH balance read. | The client rejects key or generation substitution, verifies against an independent finalised root and reports reproducible costs. |
| Phase 4 | 31–40 | Establish practical limits and failure behaviour. | Run adversarial, transition, concurrency and performance experiments on the integrated system. | Reproducible evaluation, demonstration and documented limitations. | Another person can rebuild the snapshot, reproduce the read, observe corrupt-response rejection and repeat the principal benchmarks. |

## Phase 1 days

| Day | Intended outcome | Status |
|---:|---|---|
| 1 | Research register, experiment format, ADR template, roadmap and protocol audit. | Complete |
| 2 | Sepolia WETH contract, storage layout and address corpus selection. | Complete |
| 3 | Block-hash-pinned `eth_getProof` collection probe. | Not started |
| 4 | First real proof corpus with completeness and consistency checks. | Not started |
| 5 | Rust account and storage proof verifier decision backed by a real local verification. | Not started |
| 6 | Proof-size measurements and candidate record layout. | Not started |
| 7 | Reproduced PIR server and client query with build and licence evidence. | Not started |
| 8 | Consensus light client adapter feasibility and exact interface. | Not started |
| 9 | Bundled proof, trie-node and whole-shard comparison. | Not started |
| 10 | Evidence review, accepted decisions, interface freeze and Phase 1 gate. | Not started |

## Status update rule

A work item is complete only when its stated artefact exists and its verification command succeeds.
Research questions move to answered only after their decision criterion is satisfied. A failed
experiment remains evidence and may support a fallback decision. Open technical choices are not
silently converted into architecture through implementation convenience.

The full day-level design and phase gates are defined in
[`docs/superpowers/specs/2026-10-05-four-phase-research-and-implementation-design.md`](superpowers/specs/2026-10-05-four-phase-research-and-implementation-design.md).
