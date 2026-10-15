# ADR-006: Prefer a consensus-only Helios core boundary

- Status: `proposed`
- Date: `2026-10-15`
- Proposer: assistant; adoption remains for Rohan's review
- Authorization: Rohan approved Day 8 feasibility/interface work, not a production interface freeze
- Linked question: `RQ-007`
- Evidence: [EXP-007](../../experiments/EXP-007-consensus-root/summary.md), [SRC-016](../../research/sources.md#src-016-helios-consensus-core-and-retained-fixtures)

## Recommendation

Embed Helios's consensus core in a separate adapter that owns its light-client store and exposes
only an accepted finalized execution context. Keep consensus dependencies outside the protocol and
native-proof verifier crates. EXP-007 is an isolated prototype, not the production adapter.

At inspected revision `2af9b41887e069144ee56cd710f3ddca195473f8`,
`helios-consensus-core` exposes `verify_bootstrap`, `apply_bootstrap`, `verify_generic_update` and
`apply_generic_update`. The caller must enforce verify-before-apply; apply functions alone do not
verify. Our wrapper owns the store, never exposes store replacement and never calls `force_update`.
The pinned higher-level consensus client publishes finalized `TrustedBlockRef::Hash` values; that
channel alone does not supply a state root. Full-client/sidecar integration therefore needs further
header retrieval and validation or an explicit adapter endpoint.

## Exact experimental contract

Inputs are a caller-trusted network configuration (chain ID, genesis validators root, fork schedule,
consensus preset), a trusted checkpoint beacon root plus provenance, checkpoint-age/finalized-lag
limits, and current slot from a trusted clock. The fixture harness uses minimal Deneb, chain label
`0` and test-vector provenance. These are not Sepolia configuration or production freshness policy.
A source string documents checkpoint trust; it cannot establish that trust itself.

`RootTracker<S>::bootstrap(...)` verifies checkpoint/committee/execution branches and checkpoint age.
`process_update(update, now_slot)` verifies BLS signatures and applicable execution/finality/committee
branches before applying changes. Normal Helios updates require a two-thirds committee quorum to
advance finalized state. The wrapper requires a post-bootstrap finality advance before exporting.
An optimistic update or elapsed timeout cannot promote an optimistic head. Rejected updates cannot
partially mutate accepted state.

`finalized(expected_network, now_slot)` returns a private-construction, non-deserializable
`FinalizedExecutionContext` carrying:

- configured chain ID and genesis validators root;
- finalized beacon slot and beacon block root;
- execution block number, execution block hash and execution `state_root`;
- original checkpoint root/provenance, observation slot and freshness limits.

`context.check_snapshot(snapshot, now_slot)` rechecks lag/time and exact chain/number/hash/root
agreement. No generation is minted: consensus does not authenticate publisher metadata, corpus,
index map, contract, record geometry or PIR preprocessing. Those commitments and the locally derived
key remain separate client checks before using `pvrpc-verifier::ReadContext`.

The output's execution `state_root` is the MPT trie root. Helios's `execution_root()` accessor returns
the SSZ execution-payload-header root, which is a different hash. Tests explicitly distinguish them.

Errors distinguish unavailable, checkpoint-too-old, stale, wrong network, clock regression,
unsupported header, missing provenance, consensus verification failure and snapshot mismatch.
Only exact selected finalized contexts are admitted; a newer root does not authenticate an older
provider block. Historical use requires retained authenticated contexts or verified ancestry.

## Evidence and limits

Ten Rust boundary tests and four source-integrity tests pass. Five retained synthetic updates match
all expected finalized/optimistic beacon and SSZ payload roots; signatures and branches are actually
verified. Mutated signatures, finality branches, execution state root, genesis domain, provider
identity/root and stale contexts are rejected. A companion fixture tests that a timeout leaves the
finalized slot at 96 rather than promoting optimistic slot 130.

Fixtures have a 32-member committee; Sepolia/mainnet use the mainnet preset with 512. There is no
live sync, independent Sepolia checkpoint, authenticated real corpus or composed PIR/proof read.
Helios MIT and the pinned hashing patch Apache-2.0 licences are retained; no complete dependency or
security audit is claimed. The isolated locked build resolves Alloy 2.5.0 under upstream's declared
2.1-compatible range and runs on Rust 1.95.0. Main-workspace dependencies remain unchanged.

## Alternatives and consequences

A full embedded Helios client supplies sync, persistence and execution support, but integrates a
larger API surface than the root-only consumer needs. It is still a credible fallback if implementing
a safe sync lifecycle around the core costs more than adapting the full client. No performance
comparison was performed.

A locally controlled Helios process could isolate dependencies and manage sync separately. A
versioned local endpoint would still need explicit checkpoint/configuration/freshness semantics and
transport provenance; generic provider JSON with a `finalized` label is insufficient. This remains
an option, not an option rejected on measured speed grounds.

The core approach avoids any storage-address/key input and makes transitions inspectable. It also
requires implementing bounded fork-aware transport decoding, clock conversion, update scheduling,
committee-period catch-up, checkpoint rotation/persistence and recovery. The fixture helper panics
on malformed retained files and is not that network decoder.

Capella/Deneb/Electra-style headers supply the execution payload used here. Bellatrix and the pinned
source's Gloas hash-only representation fail closed on export. Hash-only forks require fetching an
execution header and recomputing its block hash before trusting its state root. This path and live
fork coverage remain unimplemented and unverified.

## Acceptance and revisit conditions

Keep proposed until Rohan reviews the boundary and Phase 1 interfaces are considered together.
Production acceptance requires live target-network sync from an explicitly trusted recent checkpoint,
correct active-fork configuration, and a checked root used in a real pinned proof read. Revisit if
sync lifecycle cost favors a full client/sidecar, upstream interfaces change, hash-only headers become
required, or checkpoint/history/freshness requirements change. The Day 8 feasibility result does not
satisfy the Phase 3 authenticated-root gate.
