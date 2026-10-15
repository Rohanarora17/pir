# Private and Verifiable Ethereum State Reads

Repository: <https://github.com/Rohanarora17/pir>

This directory contains the implementation of the CS6666 project. It is a separate Rust
workspace so the earlier Bitcoin FIBRE assignment in the parent directory remains unchanged.

## Project status

- Active phase: Phase 1, foundation, research and design lock
- Completed checkpoint: Day 8 consensus-root interface feasibility in EXP-007 (synthetic fixtures)
- In progress: Day 9 scale and bundled-proof/trie-node/whole-shard comparison
- Current evidence: local proof/PIR reads plus signed consensus-fixture replay; live root
  authentication pending
- Verifier rationale: accepted ADR-003 and the retained EXP-004 comparison
- Light-client choice: Helios, accepted in [ADR-007](docs/decisions/ADR-007-helios-light-client.md);
  integration mode and live Sepolia validation remain pending

Progress is evidence-based. A project day advances when its artefacts and verification are complete,
not merely when a calendar day passes. See the [`project roadmap`](docs/roadmap.md) for phase gates
and the current checkpoint.

The Rust foundation defines the boundary between a published Ethereum snapshot and a privately
retrieved proof record. Experimental utilities now fetch and structurally check real proof data.
Native Merkle Patricia verification is implemented in a separate component. PIR, consensus
authentication and snapshot construction remain to be connected after their interfaces and failure
rules have supporting evidence.

## Current components

- `pvrpc-protocol` defines versioned snapshot manifests, proof records and their validation rules.
- `pvrpc-manifest` validates a manifest JSON file before it is accepted by later components.
- [`pvrpc-verifier`](crates/verifier/README.md) recovers checked account/storage values with independent
  request binding, typed inclusion/absence/errors and bounded proof admission. Root provenance
  remains the future light client's responsibility.
- `examples/development-manifest.json` is synthetic data for checking the local setup. It is not an
  authenticated Ethereum snapshot.
- `EXP-001` selects the Uniswap-listed Sepolia WETH9 deployment, confirms mapping slot `3` and
  retains matching zero and non-zero state reads at one pinned block.
- `EXP-002` compares EIP-1898 `eth_getProof` behaviour across four public Sepolia endpoints and
  establishes the fail-closed block-hash selector rule.
- `EXP-003` collects 32 structurally checked proof responses at one block hash, with 24 reported
  nonzero and 8 reported zero values. Its offline audit replays the completeness checks from retained
  evidence. EXP-003 alone does not verify the proofs or authenticate the state root.
- `EXP-004` verifies those 32 records with both Alloy and eth_trie, including 8 zero exclusions,
  and runs 313 rejection checks per candidate. Verification is conditional on the supplied root;
  independent consensus authentication remains pending. The reusable adapter is separately tested.
- [`EXP-005`](experiments/EXP-005-record-geometry/summary.md) measures complete storage-proof rows:
  binary frames need 2,254–2,731 bytes; 3 KiB fits this corpus with 18.05% padding. The candidate
  decoder preserves the verifier boundary; final representation and backend geometry remain open.
- [`EXP-006`](experiments/EXP-006-pir-baseline/summary.md) retrieves and verifies all 32 real records
  through index and keyword PIR with SimplePIR/FrodoPIR at 3/4 KiB. Direct indexing has lower
  setup/response costs here. Whole-generation download is cheaper in bandwidth for this small corpus.
  This is local computation and framing; setup serialization, networking and root authentication
  remain pending. ADR-005 recommends the next baseline and remains proposed.

- [`EXP-007`](experiments/EXP-007-consensus-root/summary.md) verifies pinned Helios consensus-core
  fixtures, exports a checked finalized execution context, and tests root/network/freshness rejection.
  Ten Rust and four source-integrity tests pass. This is synthetic feasibility, not live Sepolia sync
  or authentication of EXP-003. The consensus-only interface in ADR-006 remains proposed.

- [`EXP-008`](experiments/EXP-008-synthetic-pir-scale/summary.md) runs synthetic direct SimplePIR
  at six admitted sizes from 32 to 32,768 records and two widths, with 384 correct sampled reads.
  Larger sizes are explicitly skipped by resource admission. Estimated communication savings
  appear for some workloads, with significant client-memory costs. Real representation and
  network comparisons remain pending.

## Run the foundation

```bash
cargo +stable test --workspace --locked
cargo run -p pvrpc-manifest -- validate examples/development-manifest.json
python3 -m unittest discover -s experiments/scripts/tests -v
python3 experiments/scripts/collect_corpus.py audit experiments/EXP-003-real-proof-corpus/retained
```

The validator checks the file format and the protocol invariants implemented so far. A successful
result does not prove that the block or state root came from Ethereum. Consensus light client
authentication will provide that guarantee in a later component.

## What is fixed at this stage

- Each database generation refers to one block hash and one execution state root.
- The manifest identifies one contract, mapping slot, record shape and PIR parameter set.
- A returned record must match the manifest generation and the storage key requested by the client.
- Every returned value must carry a storage proof. A missing record is never converted into an
  authenticated zero balance.
- Ethereum hashes, addresses and proof nodes use strict `0x` prefixed JSON encoding.

The component boundaries and next implementation steps are described in
[`docs/architecture.md`](docs/architecture.md).

## Project reports

[Progress Report 2](reports/progress-report-2.pdf) is the three-page project update covering
proof verification, local PIR experiments, Helios selection and the planned scale evaluation.
The [report source and rebuild instructions](reports/README.md) are retained with the PDF.

## Research and design records

- [Collaboration and learning agreement](docs/collaboration.md)
- [Approved four-phase design](docs/superpowers/specs/2026-10-05-four-phase-research-and-implementation-design.md)
- [Phase roadmap and gates](docs/roadmap.md)
- [Research process and claim labels](research/README.md)
- [Phase 1 research questions](research/questions.md)
- [Source register](research/sources.md)
- [Technical decision matrix](research/decision-matrix.md)
- [Protocol audit](research/protocol-audit.md)
- [Architecture decision records](docs/decisions/README.md)
- [Reproducible experiment contract](experiments/README.md)

The research system separates hypotheses, source findings, local results, design decisions and
implemented behaviour. Technical choices remain open until their stated evidence and decision
criteria are satisfied.
