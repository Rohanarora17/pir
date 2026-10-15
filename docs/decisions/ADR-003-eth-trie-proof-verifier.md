# ADR-003: Use eth_trie for authenticated account and storage recovery

- Status: `accepted`
- Date: `2026-10-15`
- Decider: assistant under Rohan's explicit delegation to make the final integration-focused choice
- Authorization: Rohan requested the final decision based on future integrations and suitability
- Linked research question: `RQ-003`
- Linked experiment: [`EXP-004`](../../experiments/EXP-004-verifier-comparison/)
- Criteria: integration, features and flexibility. Compiler upgrades are acceptable and minimum
  compiler versions do not determine this recommendation.

## Context and evidence

Our manifest contains account-proof nodes but not the decoded account fields. The adapter should
recover the authenticated account, decode its checked storage root, then recover or check storage
values with explicit inclusion/absence/error outcomes.

Both candidates verify the same 32 real records and reject all 313 adversarial/binding checks.
The initial comparison supplied decoded RPC account fields, masking the account-recovery benefit.
A follow-up eth_trie probe consumes only the supplied root, expected account/key and proof nodes.
It recovers the account and all 24 nonzero values plus 8 exclusions without using RPC value hints
for verification. Reversing node order also succeeds. These checks remain conditional on the
supplied execution root; consensus authentication is separate.

## Decision

Pin `eth_trie = "=0.6.1"` in a separate reusable component, with Alloy primitives and RLP decoding.
Keep `pvrpc-protocol` free of trie dependencies. Enforce expected account/key binding, fallible
parsing, resource limits and explicit inclusion, authenticated absence and invalid-proof outcomes.
The experimental fixture assertions are not the production adapter.

`verify_proof(root, key, nodes)` returns checked value bytes or absence. This fits the existing
account-proof-only manifest without adding decoded account fields or writing custom MPT traversal.
The wider API supports custom storage, lookup, insert/remove, proof generation and roots with
changed nodes. These features can support tests and later node-storage experiments; our partial
address shard still cannot reconstruct the complete Ethereum trie.

## Fit across the planned integrations

| Integration | How the selected verifier fits | Boundary to preserve |
|---|---|---|
| Proof-bundle PIR | Recover the checked account and balance directly from native proof nodes. | PIR returns bytes; verification does not know which PIR scheme transported them. |
| Content-addressed trie-node retrieval | Hash-indexed node storage and order-independent proof recovery support assembling privately retrieved nodes. | Fixed retrieval budgets, padding and missing-node handling remain project rules; this mode has not been integrated yet. |
| Consensus light client | Consume its accepted execution state root through the project-owned root-context interface. | The proof verifier does not synchronize consensus or fetch account/storage data. |
| Immutable snapshot generations | Verify each corpus against its generation's supplied root, independently of older generations. | Publication, retention and freshness are snapshot-manager responsibilities. Mutating a partial WETH shard cannot compute Ethereum's global state root. |
| Tests and future node-storage experiments | Custom DB, proof generation, insert/remove and changed-node APIs provide reusable test/storage tools. | These optional capabilities do not expand the service into a full execution client. |

The bundle and recovery path is locally demonstrated by EXP-004. Future node-storage/PIR and
light-client compatibility above is an integration assessment from the public APIs, not a claim
that those system components have been built. Keep eth_trie types internal to the adapter so
the PIR transport, protocol formats and root-authentication implementation can evolve separately.

## Tradeoffs and alternatives

eth_trie allocates a temporary proof database and maps some underlying errors to `InvalidProof`.
Add project-owned account/storage phase and key context. Reopen if allocations become a measured
bottleneck or diagnostics prove insufficient.

Alloy remains a strong alternative: stateless borrowed inputs, detailed mismatch errors and bulk
HashBuilder/proof-retention APIs. Its verification function needs expected account bytes. With
our current manifest, that requires decoded account metadata or a separate safe extraction path.
This integration cost outweighs the simpler function call for this project.

Both candidates use Alloy primitive types and can consume roots from a future Helios adapter.
Helios's internal choice does not force this project's choice. A custom MPT implementation is
unnecessary. Retain Alloy as a differential-test reference rather than a production dependency.

## Revision and revisit condition

The initial unaccepted draft recommended Alloy. Checking value recovery against the actual
manifest changed the recommendation. Rohan then delegated the final choice; the assistant selected
eth_trie for the planned integrations. This records delegated decision-making, not a claim that
Rohan independently reviewed or selected every library detail.

Revisit for unsupported proof shapes, measured allocation costs, insufficient error detail, a
change to decoded-account record formats, or dependency security/maintenance issues. PIR, root
authentication, final record geometry and repository licence remain separate decisions.

## Implementation checkpoint: 2026-10-15

The reusable [pvrpc-verifier](../../crates/verifier/README.md) now implements this decision.
It consumes project protocol types, independently supplied snapshot/contract/key/index context,
and configured byte budgets. The protocol crate retains no trie dependencies. Rust 1.87 is the
workspace floor; compiler floor did not enter the selection ranking. The current lockfile is
verified on installed stable Rust 1.95.0, not an exact 1.87 toolchain.

Offline tests recover all 24 nonzero values and eight absences from EXP-003 nodes, with reordered
proofs and rejection checks. Strict structural admission guards compact-path panics and a review-
reproduced eth_trie 0.6.1 one-byte value decoding ambiguity. Raw payload `0x80` could become
`0x8180` and pass a claim of 128. Rejecting leaf/branch value payloads consisting of one byte
>= `0x80` excludes no valid Ethereum account or canonical nonzero storage integer. The regression
failed before the guard and now asserts the specific structural error; valid 127/128/255 values
remain accepted. Retain this boundary when upgrading the backend. This is a scoped adapter guard,
not a replacement MPT traversal. The supplied root remains unauthenticated by consensus.
