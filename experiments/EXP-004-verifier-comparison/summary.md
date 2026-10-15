# EXP-004 result and recommendation

Selected **eth_trie 0.6.1** behind a small project-owned adapter. Accepted ADR-003 records the
final delegated choice and its fit across planned integrations. Production integration remains pending.

## Local results

Both pinned candidates verified the same 32 EXP-003 records against its pinned state root:

| Candidate | Nonzero inclusion | Zero exclusion | Rejection checks passed | Local time per record |
|---|---:|---:|---:|---:|
| alloy-trie 0.9.8 | 24 | 8 | 313 | 18.86 microseconds |
| eth_trie 0.6.1 | 24 | 8 | 313 | 70.05 microseconds |

Times are total microseconds in `raw/results.jsonl` divided by 10 passes and 32 records. This is
one small local release-mode measurement in the local arm64 environment with Rust 1.89 nightly. It includes
both proof checks, already loaded field parsing, RLP encoding and library-specific allocation.
CPU model and RAM queries were restricted by the sandbox, which further limits timing comparisons.
It does not establish comparative production throughput. Each candidate performs the same work,
including re-verifying the common account proof per record.

The 313 checks combine 9 mutations/binding failures per record, 24 false-zero truncated-path cases
and one malformed-RLP parsing check. The actual request-key binding checks belong to the project
wrapper; they are not presented as features enforced by a bare MPT function.

## Integration, features and flexibility

Both candidates pass the selected correctness checks. Compiler upgrades are acceptable and are
excluded from the ranking. The deciding difference is authenticated value recovery.

Our manifest has account-proof nodes but no decoded account fields. Alloy's verifier checks a
supplied expected value. eth_trie returns the authenticated value or absence. This lets us decode
the checked WETH storage root directly, without extending the manifest or implementing custom MPT
value traversal. The follow-up recover_account executable recovers all 32 balances from proof
nodes, the supplied state root and independently derived keys. It also works with reversed node
order. Raw results are in raw/account-recovery.jsonl.

eth_trie's wider API offers custom storage, mutation, proof generation and changed-node root
results for future tests or node-storage experiments. These do not let a partial address shard
reconstruct the full Ethereum state. Both libraries use Alloy primitive types, so later consumption
of Helios-authenticated roots is compatible with either choice.

Alloy offers stateless borrowed-input checking, detailed mismatch diagnostics and bulk HashBuilder/
proof-retention tools. eth_trie pays temporary-database allocation costs and collapses some errors
to InvalidProof. The adapter will add account/storage phase and key context. A measured allocation
or diagnostic problem would justify revisiting the choice.

The original expected-value probe alone did not expose the account-recovery advantage.

## Meaning of the verified zeros

The 8 zero cases are valid exclusions under the checked WETH storage root, and the account proof
binds that storage root to the supplied execution state root. They are no longer merely reported
zeros **relative to that root**. A provider-supplied fabricated state root could still authenticate
its own invented trie. Independent consensus authentication remains required.

## Next work and open decisions

Create a separate reusable Rust verification crate with strict
fallible parsing, input limits and typed errors. Keep the protocol crate dependency-free from
execution and light-client libraries. Carry the same real corpus and negative cases into integration
tests, add independent inline-node fixtures, and build on the agreed toolchain.

The verifier dependency choice is accepted. Root authentication, the PIR backend and final
record geometry remain open. A custom verifier is unnecessary given two working reusable options.

Reproduce the executable checks with a fresh output file:

```bash
OUTPUT_FILE=/private/tmp/EXP-004-next.jsonl sh experiments/EXP-004-verifier-comparison/commands.sh
```
