# Day 1 protocol audit

## Audit boundary

This is a source and test review of the initial `pvrpc-protocol` crate against the approved project
scope. It is not Ethereum proof verification, a cryptographic security audit or a decision to keep
the current schema. A field can be present and internally consistent while still lacking Ethereum
authentication.

Status values have these exact meanings:

- `implemented and tested`: current code enforces the stated local rule and a test covers it.
- `represented but not authenticated`: the data has a field, but its claimed Ethereum meaning is
  not independently verified.
- `provisional`: an initial field or rule exists, but Phase 1 evidence may change its design.
- `missing`: the current protocol has no representation or enforcement for the requirement.

## Requirement comparison

| Requirement | Current representation | Evidence | Status | Phase 1 action |
|---|---|---|---|---|
| Versioned manifest and proof record | Separate manifest and record schema constants, with unsupported manifest versions rejected. Record validation also rejects unsupported versions. | `crates/protocol/src/lib.rs:5-6`, `:162-170`, `:214-218`, `:255-289`. Manifest rejection is covered by `tests/manifest_contract.rs`. | implemented and tested | Retain versioning. Increment the format only after Phase 1 decisions require an incompatible change. |
| Snapshot identity contains chain ID, block number, block hash, state root and generation | `SnapshotRef` contains all five fields. A returned record must carry the exact same `SnapshotRef` as the manifest. | `crates/protocol/src/lib.rs:135-143`, `:291-293`. Snapshot substitution is covered by `tests/proof_record_contract.rs`. | implemented and tested | `RQ-002` and `RQ-007` must establish how these fields are obtained and authenticated. |
| Finalised block and state root are authentic | The state root and block identity are supplied as data. No finality evidence, checkpoint, light client provenance or header proof is represented. | `SnapshotRef` fields only. No consensus dependency or adapter exists in the workspace. | represented but not authenticated | `RQ-007` defines the light client adapter. Phase 3 connects its authenticated output to the record verifier. |
| One declared contract and mapping slot | The manifest contains a contract address and mapping slot. Validation rejects only the zero contract address. | `crates/protocol/src/lib.rs:165-166`, `:226-228`. | represented but not authenticated | `RQ-001` selects and validates the Sepolia workload. `RQ-003` later authenticates the contract account and storage root. |
| Dataset identity and address-corpus commitment | `DatasetSpec` contains only `record_count` and `record_size`. There is no corpus identifier, corpus hash, ordering rule or index-map commitment. | `crates/protocol/src/lib.rs:145-150`. | missing | `RQ-001` defines the reproducible corpus. Phase 1 must decide which identity fields enter the manifest before the format is frozen. |
| Dataset dimensions are non-zero | Manifest validation rejects zero record count and zero record size. | `crates/protocol/src/lib.rs:229-234`, covered by `tests/manifest_contract.rs`. | implemented and tested | Keep the local validation. `RQ-004` supplies measured values. |
| Record index and requested storage key are bound to the response | `validate_for` rejects an out-of-range index and a storage key different from the client's requested key. | `crates/protocol/src/lib.rs:294-302`, covered by `tests/proof_record_contract.rs`. | implemented and tested | Confirm that the index map is deterministic after `RQ-001` defines the corpus. |
| Account proof is present | The manifest requires at least one non-empty account-proof node. | `crates/protocol/src/lib.rs:169`, `:244-249`, covered by `tests/manifest_contract.rs`. | implemented and tested | `RQ-003` must establish proof correctness and whether the account proof remains shared manifest data. |
| Storage proof is present | A proof record requires at least one non-empty storage-proof node. | `crates/protocol/src/lib.rs:263`, `:303-308`, covered by `tests/proof_record_contract.rs`. | implemented and tested | `RQ-003` must establish proof correctness and authenticated absence semantics. |
| Account proof verifies against `stateRoot` | The crate stores bytes but performs no RLP decoding, trie traversal, node hashing or account decoding. | No verification function or trie dependency exists. | missing | `RQ-003` selects and tests a verifier on Day 5. Phase 2 integrates it before publication. |
| Storage proof verifies against account `storageRoot` | The crate stores bytes but does not obtain the account `storageRoot` or verify the requested storage path and value. | No verification function or trie dependency exists. | missing | `RQ-003` selects and tests a verifier. Phase 2 integrates membership and authenticated absence. |
| Strict Ethereum JSON byte encoding | Hashes and addresses have fixed lengths, proof bytes use `0x` hex and unknown struct fields are rejected. | `crates/protocol/src/lib.rs:43-133`, `:135-162`, covered by `tests/json_contract.rs`. | implemented and tested | Retain unless the selected Ethereum library requires a compatible typed representation at an adapter boundary. |
| PIR scheme and parameter set are declared | The manifest stores free-form scheme and parameter strings plus a query count. It checks only that the strings and count are non-empty. | `crates/protocol/src/lib.rs:152-158`, `:235-243`. | provisional | `RQ-005` and `RQ-006` determine the backend and parameters. Replace free-form fields if backend-independent typed fields can be justified. |
| PIR preprocessing belongs to the same generation as values and proofs | No preprocessing digest, database digest, parameter digest or artefact list is committed by the manifest. | No corresponding field exists. | missing | `RQ-004` and `RQ-005` determine the artefacts. Phase 2 adds a generation-wide commitment and atomic publication rule. |
| Fixed record shape and oversized-proof behaviour | `record_size` is declared, but `ProofRecord` uses variable-length vectors and does not encode or enforce padding. | `crates/protocol/src/lib.rs:149`, `:257-264`. | provisional | `RQ-004` measures real proofs and defines padding, limits and an explicit unsupported outcome. |
| Database digest and atomic generation publication | No database hash, staged state, completeness marker, active-generation pointer or retention metadata exists. | No publisher or snapshot-store crate exists. | missing | Phase 2 designs and implements atomic publication using the record layout accepted after `RQ-004`. |
| Zero balance is not confused with a missing record | A zero `value` is accepted only when a non-empty storage proof is present. The proof is not yet cryptographically checked. | `tests/proof_record_contract.rs` covers a zero value with proof presence. | represented but not authenticated | `RQ-003` must verify the zero value or authenticated absence path. Until then, the local rule prevents only a structurally proofless zero. |
| Authenticated absence is distinct from unavailable data | No absence result or incomplete-path result exists. An empty proof is rejected without explaining whether the key is absent or the service lacks data. | `RecordError` contains proof-presence errors only. | missing | `RQ-003` defines verifier semantics. `RQ-004` defines coverage limits. Phase 2 introduces distinct outcomes. |
| Online failures include stale, unavailable, malformed, unsupported and verification failed | Current errors cover schema, shape, generation, index, key and proof-presence failures. They do not model freshness, availability, proof correctness or an unsupported proof shape. | `ManifestError` and `RecordError` in `crates/protocol/src/lib.rs:172-210`, `:266-275`. | provisional | `RQ-002`, `RQ-003`, `RQ-004` and `RQ-007` determine the complete failure vocabulary before Phase 3 freezes the client API. |
| No revealing RPC fallback | No online client exists, so the rule is neither violated nor enforced. | No client or transport crate exists. | missing | Phase 3 adds an observable transport test proving that private-read failure never invokes ordinary `eth_getProof`. |

## Initial findings

1. Snapshot equality and storage-key equality are implemented and tested. They protect local binding
   between a manifest, returned record and requested key.
2. Proof byte presence is checked, but proof correctness is not verified. A non-empty vector is not
   evidence that a value belongs to Ethereum state.
3. The state root is represented, but no consensus light client authenticates it yet. A malicious
   provider could currently choose both a root and proof that agree with each other.
4. The schema does not commit to the address corpus, database artefacts or PIR preprocessing. It
   cannot yet establish that every file belongs to one complete generation.
5. The proof-bundle representation and `record_size` are provisional until `RQ-004` measures real
   proof data.
6. The current error types do not express stale, unavailable, authenticated absence, malformed proof,
   unsupported proof shape or cryptographic verification failure.
7. Shared account proof data in the manifest may be reasonable for one public contract, but this
   remains a provisional representation until the real corpus and record-cost comparison are complete.

## Ownership of gaps

| Gap | Owner |
|---|---|
| Contract, mapping slot, corpus construction and index-map identity | `RQ-001` |
| Exact block selection and provider pinning | `RQ-002` |
| Account proof, storage proof and authenticated absence verification | `RQ-003` |
| Record representation, padding, proof limits and coverage | `RQ-004` |
| PIR scheme, parameters, preprocessing artefacts and available hardware | `RQ-005` |
| PIR distribution and dependency boundary | `RQ-006` |
| Finalised execution root authentication and freshness | `RQ-007` |
| Measured justification for PIR rather than full download | `RQ-008` |
| Database digest, staged build, atomic publication and retention | Phase 2 implementation requirement |
| Complete online client error vocabulary and no-fallback transport test | Phase 2 and Phase 3 implementation requirement |

No schema change is accepted by this audit. Changes will be proposed after the linked research
question produces evidence and will be recorded through an ADR when they affect compatibility or
the trust boundary.
