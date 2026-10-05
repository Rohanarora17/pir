# Private and Verifiable Ethereum State Reads

Repository: <https://github.com/Rohanarora17/pir>

This directory contains the implementation of the CS6666 project. It is a separate Rust
workspace so the earlier Bitcoin FIBRE assignment in the parent directory remains unchanged.

## Project status

- Active phase: Phase 1, foundation, research and design lock
- Completed checkpoint: Day 3 block-hash-pinned proof collection
- Next checkpoint: Day 4 first real proof corpus
- Next investigation: defining corpus completeness and checking every proof response against one snapshot context

Progress is evidence-based. A project day advances when its artefacts and verification are complete,
not merely when a calendar day passes. See the [`project roadmap`](docs/roadmap.md) for phase gates
and the current checkpoint.

The first implementation slice defines the boundary between a published Ethereum snapshot and
a privately retrieved proof record. It does not yet fetch Ethereum state, run PIR or verify a
Merkle Patricia proof. Those parts will be connected only after their inputs and failure rules are
fixed at the protocol layer.

## Current components

- `pvrpc-protocol` defines versioned snapshot manifests, proof records and their validation rules.
- `pvrpc-manifest` validates a manifest JSON file before it is accepted by later components.
- `examples/development-manifest.json` is synthetic data for checking the local setup. It is not an
  authenticated Ethereum snapshot.
- `EXP-001` selects the Uniswap-listed Sepolia WETH9 deployment, confirms mapping slot `3` and
  retains matching zero and non-zero state reads at one pinned block.
- `EXP-002` compares EIP-1898 `eth_getProof` behaviour across four public Sepolia endpoints and
  establishes the fail-closed block-hash selector rule.

## Run the foundation

```bash
cargo test --workspace
cargo run -p pvrpc-manifest -- validate examples/development-manifest.json
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

## Research and design records

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
