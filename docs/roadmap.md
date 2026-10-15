# Project roadmap

## Current status

- Active phase: Phase 1
- Active day: Day 9
- Current gate: not evaluated
- Completed evidence: Day 1 scaffolding and tests, Day 2 WETH workload, Day 3 provider matrix,
  and Day 4's [32-address real proof corpus](../experiments/EXP-003-real-proof-corpus/summary.md)
- Verifier decision: eth_trie 0.6.1 selected in accepted ADR-003
- Light-client decision: Helios selected by Rohan in accepted ADR-007; integration mode remains open
- Implemented: reusable `pvrpc-verifier` with typed outcomes, strict binding and resource limits;
  real corpus recovery and adversarial regressions run offline
- Completed Day 6: EXP-005 measures encoding, duplication and padding; the experimental codec
  round-trips and verifies all 32 real rows. ADR-004 remains proposed; final geometry is open.
- Completed Day 7: EXP-006 reproduces index/keyword PIR over all 32 records with two engines at
  two sizes (256 verified reads), retained licence/build evidence and bounded message framing.
  This is a local feasibility result; networking and final backend acceptance remain pending.
- Completed Day 8: EXP-007 verifies synthetic Helios updates and root/freshness admission with an
  exact candidate interface. Helios is selected in ADR-007; ADR-006's integration mode remains
  proposed, and live Sepolia authentication is pending.
- Started Day 9: EXP-008 adds a resource-bounded synthetic scale sweep with 12 completed cases,
  384 correct sampled reads and four explicit memory-budget skips. It finds workload-dependent
  estimated byte savings and significant client public-array costs. Real representation, setup
  transport and network comparisons are still pending.
- Next investigation: real bundled-proof versus shared-node retrieval costs
- Evaluation scope: the 32-record corpus establishes functional correctness. Day 9 must add a
  resource-bounded scale sweep; the tiny-corpus download result does not decide PIR suitability.

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
| 3 | Block-hash-pinned `eth_getProof` collection probe. | Complete |
| 4 | First real proof corpus with completeness and consistency checks. | Complete: EXP-003, 32 structurally checked records; cryptographic verification remains Day 5 |
| 5 | Rust account and storage proof verifier decision backed by a real local verification. | Complete: accepted ADR-003; reusable adapter checks all 32 real records and adversarial regressions |
| 6 | Proof-size measurements and candidate record layout. | Complete: EXP-005; 3/4/8 KiB candidate round trips; final geometry awaits backend evidence |
| 7 | Reproduced PIR server and client query with build and licence evidence. | Complete: EXP-006 local CPU reads; 256 verified cases; ADR-005 proposed; final backend/licence policy open |
| 8 | Consensus light client adapter feasibility and exact interface. | Complete: EXP-007 synthetic fixture replay, 10 boundary tests and proposed ADR-006; live authentication remains pending |
| 9 | Bundled proof, trie-node and whole-shard comparison. | In progress: EXP-008 synthetic scale baseline complete; real representation comparison pending |
| 10 | Evidence review, accepted decisions, interface freeze and Phase 1 gate. | Not started |

## Status update rule

A work item is complete only when its stated artefact exists and its verification command succeeds.
Research questions move to answered only after their decision criterion is satisfied. A failed
experiment remains evidence and may support a fallback decision. Open technical choices are not
silently converted into architecture through implementation convenience.

The full day-level design and phase gates are defined in
[`docs/superpowers/specs/2026-10-05-four-phase-research-and-implementation-design.md`](superpowers/specs/2026-10-05-four-phase-research-and-implementation-design.md).
