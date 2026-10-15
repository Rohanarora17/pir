# Native account and storage verifier

`pvrpc-verifier` implements [ADR-003](../../docs/decisions/ADR-003-eth-trie-proof-verifier.md)
with `eth_trie = 0.6.1`. Its public API contains project types, with no trie-library types.
It performs no RPC, PIR, light-client synchronization or fallback reads.

## Caller contract

Supply a `ReadContext` containing the expected snapshot, contract, storage key and record index.
Obtain these independently of the returned manifest/record. A future consensus adapter supplies
the accepted execution root and block identity; the client selects the generation, contract and
index, and derives the storage key. A context copied from the response defeats these checks.

```rust
use pvrpc_protocol::{ProofRecord, SnapshotManifest};
use pvrpc_verifier::{ReadContext, StorageOutcome, VerificationError, Verifier};

fn verify_read(
    expected: &ReadContext,
    manifest: &SnapshotManifest,
    record: &ProofRecord,
) -> Result<StorageOutcome, VerificationError> {
    Verifier::default().verify(expected, manifest, record)
}
```

The adapter checks the envelope and byte budgets, validates node structure, verifies the account
at `keccak256(contract)` under the expected state root, and strictly decodes its account RLP.
It then verifies storage at `keccak256(storage_key)` under that account's checked storage root.
The recovered U256 must equal the record's claimed 32-byte value.

Results are `Present(value)`, `Absent`, or `Err(VerificationError)`. Only authenticated storage
nonmembership produces `Absent`, which means zero under Ethereum semantics. An absent contract,
missing proof, truncated path, stored zero, malformed RLP or mismatched value fails. A zero value
field alone is never enough. Typed diagnostics retain the expected contract/key and identify the
account/storage stage where applicable.

## Real input walkthrough

The offline tests consume EXP-003's retained nodes. Record 0 requests address
`0xce54cf5a0de3843011cf20389c1b6a4aac442d6a` at Sepolia block `11919648`.
The test derives its key from `keccak256(pad32(address) || pad32(3))`:
`0x88080a0f7453a7e6f91b5d6e4f0c47e15d7639174219416b4e29a716c14f0857`.
The WETH account proof recovers storage root
`0x1776d2e52436c5b6d99521fb9004a6c303833b96ad9154a57c19d32ca793d7cc`.
The five storage nodes establish absence for that key. The adapter returns `Absent` after checking
the record claims zero. Substituting a nonzero claim produces `ValueMismatch`; removing a required
node produces a failure rather than zero.

Tests recover 24 inclusions and eight absences across all 32 records. RPC account-field hints are
never verifier inputs. Fixture values are test expectations and record claims; proof recovery
supplies the checked values. The fixture root remains provider-reported: consensus authentication
and finality have not been established by these tests.

## Admission and decoder boundaries

Defaults permit 128 nodes per proof, 4,096 bytes per node and 1 MiB of combined account/storage
node bytes. These are configurable local limits with headroom over the measured corpus, not final
PIR record geometry. Counts and checked byte subtraction run before cloning or parsing. Callers
must separately bound transport and JSON sizes before allocating these typed inputs.

Structural admission checks complete canonical RLP, branch/path shapes, compact-path flags and
padding, nonempty extension paths, child reference widths and inline nesting (at most 64 levels).
Every supplied node is checked, including unused extras. Order is immaterial. This guards the
backend decoder; eth_trie performs hashed path lookup and membership verification. Structural
admission does not establish that an arbitrary root represents canonical Ethereum state.

Independent review reproduced an eth_trie 0.6.1 edge: raw one-byte value `0x80` was recovered as
`0x8180` and could pass a forged claim of 128. The guard rejects leaf **and branch** value payloads
of one byte >= `0x80`. Such payloads cannot encode a valid Ethereum account or canonical nonzero
storage integer. Regressions assert structural failures for raw `0x80`/`0xff` with claims 128/255,
and accept valid encodings at 127/128/255. This prevents a value-mismatch check from masking the
decoder issue. Preserve the guard unless the pinned backend's behavior is rechecked.

The protocol requires a nonempty storage proof even for zero. An explicit `0x80` empty-trie node is
supported; an empty proof array remains an error. No missing-proof exception is introduced here.
Account caching and the final snapshot-builder interface remain future work.

## Reproduce and continue

The workspace compiler floor is Rust 1.87, with Cargo resolver 3 to prefer compatible dependencies.
The locked adapter was tested on installed stable Rust 1.95.0; an exact 1.87 build is unverified.

```bash
cd /Users/rohan/Downloads/cs6666-blockchain-assignment-1/private-verifiable-rpc
cargo +stable test --workspace --locked
cargo +stable test -p pvrpc-verifier recovers_24_inclusions --locked -- --nocapture
cargo +stable fmt --all -- --check
cargo +stable clippy --workspace --all-targets --locked -- -D warnings
```

For the sandbox's existing isolated cache, prefix Cargo commands with
`CARGO_HOME=/private/tmp/pvrpc-verifier-cargo CARGO_TARGET_DIR=/private/tmp/pvrpc-verifier-target`.
This cache is temporary; ordinary use downloads the locked dependencies if needed.

A useful next learning exercise is to change the claimed value for the proven-absent record and
identify its error, then compare with truncating its proof. The next roadmap task measures padded
record sizes. PIR, consensus integration, final record geometry and repository licence remain open.
