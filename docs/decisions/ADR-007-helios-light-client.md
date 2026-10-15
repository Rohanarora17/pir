# ADR-007: Select Helios as the Ethereum light client

- Status: `accepted`
- Date: `2026-10-15`
- Decider: Rohan
- Authorization: after the alternatives discussion, Rohan said, "okay let's go with helios only"
- Linked question: `RQ-007`
- Evidence: [EXP-007](../../experiments/EXP-007-consensus-root/summary.md),
  [SRC-016](../../research/sources.md#src-016-helios-consensus-core-and-retained-fixtures)
- Related proposal: [ADR-006](ADR-006-consensus-root-boundary.md)

## Decision and scope

Use Helios as this project's Ethereum consensus light-client implementation. This accepts the
implementation choice, not a claim that live synchronization or end-to-end root authentication
has been completed. Do not implement alternative light-client backends as part of the current scope.

ADR-006 remains proposed: embedding only the consensus core, adapting a fuller embedded client,
or running a locally controlled Helios process still requires integration evidence. Prefer reusing
upstream synchronization machinery where practical; do not equate a smaller verification API with
a smaller maintenance burden. This record does not silently accept ADR-006's core-only approach.

## Rationale and evidence

Helios fits the Rust client and the project's separate root-authentication boundary. EXP-007
demonstrates local integration with pinned revision
`2af9b41887e069144ee56cd710f3ddca195473f8`: retained synthetic updates verify and produce an exact
finalized execution context, with root/network/freshness rejection tests. The pinned revision is
the experiment baseline; a production revision must have the required target-network fork support.

The preceding discussion compared Nimbus's standalone consensus light client, Lodestar's
JavaScript/TypeScript light client, Snowbridge's runtime-oriented verifier and SP1 Helios's
on-chain proving path. The assistant recommended Helios for integration fit; Rohan explicitly
selected it. This was not a head-to-head performance benchmark or comparative security audit.

## Boundaries and pending work

Helios authenticates the public execution context. The selected storage key stays in the PIR
retrieval path. Ordinary storage RPC must not become a fallback for a failed private read.
Protocol and native-proof verification crates remain independent of consensus-library types.

Live Sepolia synchronization, checkpoint provenance and recency, clock/freshness policy, current
fork coverage, restart/recovery and a real proof verified under the accepted root remain pending.
EXP-007 does not authenticate the old EXP-003 corpus. RQ-007 remains investigating, and no phase
gate advances solely because this implementation choice is accepted.

## Revisit condition

Reopen the implementation choice if Helios cannot meet target-network fork, root-output, security
or lifecycle requirements. Compare integration modes within Helios first. Any replacement requires
new evidence and an explicit decision; there is no automatic switch to another light client.
