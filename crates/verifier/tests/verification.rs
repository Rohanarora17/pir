//! Offline integration tests. RPC hints are fixture expectations, never verifier inputs.
use alloy_primitives::{B256, U256, keccak256};
use alloy_rlp::Header;
use pvrpc_protocol::{Hash32, HexBytes, ProofRecord, SnapshotManifest};
use pvrpc_verifier::{
    FailureKind, ProofStage, ReadContext, StorageOutcome, VerificationLimits, Verifier,
};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

fn fixtures() -> Vec<(ReadContext, SnapshotManifest, ProofRecord)> {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../experiments/EXP-003-real-proof-corpus/retained");
    let corpus: Value =
        serde_json::from_slice(&fs::read(directory.join("corpus.json")).unwrap()).unwrap();
    let mut snapshot = corpus["snapshot"].clone();
    snapshot["generation"] = json!(1);
    let manifest: SnapshotManifest = serde_json::from_value(json!({
        "schema_version": 1, "snapshot": snapshot, "contract": corpus["contract"],
        "mapping_slot": format!("0x{:064x}", 3),
        // These structural placeholders do not select PIR parameters or final record geometry.
        "dataset": {"record_count": 32, "record_size": 1},
        "pir": {"scheme": "fixture-only", "parameter_set": "not-used", "query_count": 1},
        "account_proof": corpus["account"]["accountProof"]
    }))
    .unwrap();
    corpus["records"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            let rpc: Value = serde_json::from_slice(
                &fs::read(directory.join(entry["proof_file"].as_str().unwrap())).unwrap(),
            )
            .unwrap();
            let record: ProofRecord = serde_json::from_value(json!({
                "schema_version": 1, "snapshot": snapshot, "record_index": entry["record_index"],
                "storage_key": entry["storage_key"], "value": entry["value"],
                "storage_proof": rpc["result"]["storageProof"][0]["proof"]
            }))
            .unwrap();
            // Derive the requested key from the address, independently of the returned record key.
            let address: alloy_primitives::Address =
                entry["address"].as_str().unwrap().parse().unwrap();
            let mut input = [0u8; 64];
            input[12..32].copy_from_slice(address.as_slice());
            input[63] = 3;
            let expected = ReadContext {
                snapshot: manifest.snapshot.clone(),
                contract: manifest.contract,
                storage_key: Hash32::new(keccak256(input).0),
                record_index: record.record_index,
            };
            (expected, manifest.clone(), record)
        })
        .collect()
}

#[test]
fn recovers_24_inclusions_and_eight_proven_absences_from_real_nodes() {
    let verifier = Verifier::default();
    let (mut present, mut absent) = (0, 0);
    for (expected, manifest, record) in fixtures() {
        match verifier.verify(&expected, &manifest, &record).unwrap() {
            StorageOutcome::Present(value) => {
                present += 1;
                assert_eq!(value, record.value);
            }
            StorageOutcome::Absent => {
                absent += 1;
                assert_eq!(record.value, Hash32::new([0; 32]));
            }
        }
    }
    assert_eq!((present, absent), (24, 8));
}

#[test]
fn proof_node_order_does_not_change_the_result() {
    for (expected, mut manifest, mut record) in fixtures() {
        manifest.account_proof.reverse();
        record.storage_proof.reverse();
        assert!(
            Verifier::default()
                .verify(&expected, &manifest, &record)
                .is_ok()
        );
    }
}

#[test]
fn rejects_forged_values_even_for_proven_absence() {
    for (expected, manifest, mut record) in fixtures() {
        record.value = if record.value == Hash32::new([0; 32]) {
            Hash32::new([1; 32])
        } else {
            Hash32::new([0; 32])
        };
        assert!(
            Verifier::default()
                .verify(&expected, &manifest, &record)
                .is_err()
        );
    }
}

#[test]
fn independently_binds_snapshot_account_key_and_index() {
    let (expected, manifest, record) = fixtures().remove(0);
    let verifier = Verifier::default();
    let mut wrong = expected.clone();
    wrong.contract = pvrpc_protocol::Address::new([1; 20]);
    assert!(verifier.verify(&wrong, &manifest, &record).is_err());
    wrong = expected.clone();
    wrong.storage_key = Hash32::new([1; 32]);
    assert!(verifier.verify(&wrong, &manifest, &record).is_err());
    wrong = expected.clone();
    wrong.record_index += 1;
    assert!(verifier.verify(&wrong, &manifest, &record).is_err());
    // Changing both returned objects must not replace the independently supplied snapshot.
    for field in [
        "chain_id",
        "block_number",
        "block_hash",
        "state_root",
        "generation",
    ] {
        let mut changed = serde_json::to_value(&manifest).unwrap();
        changed["snapshot"][field] = match field {
            "block_hash" | "state_root" => json!(format!("0x{}", "11".repeat(32))),
            _ => json!(999),
        };
        let changed: SnapshotManifest = serde_json::from_value(changed).unwrap();
        let mut changed_record = record.clone();
        changed_record.snapshot = changed.snapshot.clone();
        assert!(
            verifier
                .verify(&expected, &changed, &changed_record)
                .is_err(),
            "{field}"
        );
    }
}

#[test]
fn a_matching_envelope_cannot_rescue_an_invalid_root_account_or_key() {
    let (mut expected, mut manifest, mut record) = fixtures().remove(0);
    expected.snapshot.state_root = Hash32::new([7; 32]);
    manifest.snapshot = expected.snapshot.clone();
    record.snapshot = expected.snapshot.clone();
    assert!(
        Verifier::default()
            .verify(&expected, &manifest, &record)
            .is_err()
    );
    let (mut expected, mut manifest, record) = fixtures().remove(0);
    expected.contract = pvrpc_protocol::Address::new([1; 20]);
    manifest.contract = expected.contract;
    assert!(
        Verifier::default()
            .verify(&expected, &manifest, &record)
            .is_err()
    );
    let (mut expected, manifest, mut record) = fixtures().remove(0);
    expected.storage_key = Hash32::new([7; 32]);
    record.storage_key = expected.storage_key;
    assert!(
        Verifier::default()
            .verify(&expected, &manifest, &record)
            .is_err()
    );
}

#[test]
fn truncation_corruption_and_missing_proofs_never_become_zero() {
    for (expected, manifest, record) in fixtures() {
        for account in [true, false] {
            for operation in 0..4 {
                let mut manifest = manifest.clone();
                let mut record = record.clone();
                let proof = if account {
                    &mut manifest.account_proof
                } else {
                    &mut record.storage_proof
                };
                match operation {
                    0 => {
                        proof.pop();
                    }
                    1 => proof.clear(),
                    2 => proof[0] = HexBytes::new(Vec::new()),
                    _ => {
                        let mut bytes = proof[0].as_bytes().to_vec();
                        bytes[0] ^= 1;
                        proof[0] = HexBytes::new(bytes);
                    }
                }
                assert!(
                    Verifier::default()
                        .verify(&expected, &manifest, &record)
                        .is_err(),
                    "account={account} operation={operation}"
                );
            }
        }
    }
}

#[test]
fn limits_accept_exact_boundaries_and_reject_one_less() {
    let (expected, manifest, record) = fixtures().remove(0);
    let nodes = manifest.account_proof.len().max(record.storage_proof.len());
    let size = manifest
        .account_proof
        .iter()
        .chain(&record.storage_proof)
        .map(|n| n.as_bytes().len())
        .max()
        .unwrap();
    let total = manifest
        .account_proof
        .iter()
        .chain(&record.storage_proof)
        .map(|n| n.as_bytes().len())
        .sum();
    let exact = VerificationLimits {
        max_nodes_per_proof: nodes,
        max_node_bytes: size,
        max_total_bytes: total,
    };
    assert!(
        Verifier::new(exact)
            .verify(&expected, &manifest, &record)
            .is_ok()
    );
    for limited in [
        VerificationLimits {
            max_nodes_per_proof: nodes - 1,
            ..exact
        },
        VerificationLimits {
            max_node_bytes: size - 1,
            ..exact
        },
        VerificationLimits {
            max_total_bytes: total - 1,
            ..exact
        },
    ] {
        assert!(
            Verifier::new(limited)
                .verify(&expected, &manifest, &record)
                .is_err()
        );
    }
}

fn list(items: &[Vec<u8>]) -> Vec<u8> {
    let payload: Vec<u8> = items.concat();
    let mut out = Vec::new();
    Header {
        list: true,
        payload_length: payload.len(),
    }
    .encode(&mut out);
    out.extend(payload);
    out
}

fn leaf(key: B256, value: &[u8]) -> Vec<u8> {
    let mut path = vec![0x20];
    path.extend(key.as_slice());
    list(&[alloy_rlp::encode(path.as_slice()), alloy_rlp::encode(value)])
}

fn rebind_account(
    expected: &mut ReadContext,
    manifest: &mut SnapshotManifest,
    record: &mut ProofRecord,
    node: Vec<u8>,
) {
    expected.snapshot.state_root = Hash32::new(keccak256(&node).0);
    manifest.snapshot = expected.snapshot.clone();
    record.snapshot = expected.snapshot.clone();
    manifest.account_proof = vec![HexBytes::new(node)];
}

#[test]
fn malformed_compact_paths_and_nested_rlp_fail_without_panicking() {
    let (expected, manifest, record) = fixtures().remove(0);
    let mut malformed = vec![
        vec![0xff],
        vec![0xc0],
        list(&[vec![0x80], vec![0x80]]),
        list(&[vec![0x40], vec![0x80]]),
        list(&[vec![0x00], vec![0x80]]),
    ];
    let mut nested = vec![0x80];
    for _ in 0..100 {
        nested = list(&[vec![0x11], nested]);
    }
    malformed.push(nested);
    let mut trailing = manifest.account_proof[0].as_bytes().to_vec();
    trailing.push(0x80);
    malformed.push(trailing);
    for node in malformed {
        let (mut expected, mut manifest, mut record) =
            (expected.clone(), manifest.clone(), record.clone());
        rebind_account(&mut expected, &mut manifest, &mut record, node);
        assert!(
            Verifier::default()
                .verify(&expected, &manifest, &record)
                .is_err()
        );
    }
}

#[test]
fn authenticated_account_bytes_still_require_strict_account_rlp() {
    for value in [vec![0xc0], vec![0xff], vec![0xc0, 0x80]] {
        let (mut expected, mut manifest, mut record) = fixtures().remove(0);
        let node = leaf(keccak256(expected.contract.as_bytes()), &value);
        rebind_account(&mut expected, &mut manifest, &mut record, node);
        assert!(
            Verifier::default()
                .verify(&expected, &manifest, &record)
                .is_err()
        );
    }
}

#[test]
fn authenticated_storage_bytes_require_a_canonical_nonzero_u256() {
    // Valid inclusion, noncanonical integer, trailing bytes, oversized integer, stored zero.
    for (encoded, claimed, succeeds) in [
        (vec![1], 1, true),
        (vec![0x81, 1], 1, false),
        (vec![1, 0x80], 1, false),
        ([vec![0xa1], vec![1; 33]].concat(), 1, false),
        (vec![0x80], 0, false),
        // Upstream must not reinterpret a single high byte including its outer prefix.
        (vec![0x80], 128, false),
        (vec![0xff], 255, false),
        // Valid integers at and above the RLP single-byte boundary still work.
        (vec![0x7f], 127, true),
        (vec![0x81, 0x80], 128, true),
        (vec![0x81, 0xff], 255, true),
    ] {
        let (mut expected, mut manifest, mut record) = fixtures().remove(0);
        let node = leaf(keccak256(expected.storage_key.as_bytes()), &encoded);
        let storage_root = keccak256(&node);
        record.storage_proof = vec![HexBytes::new(node)];
        record.value = Hash32::new(U256::from(claimed).to_be_bytes());
        let account = list(&[
            vec![1],
            vec![0x80],
            alloy_rlp::encode(storage_root.as_slice()),
            alloy_rlp::encode(keccak256([]).as_slice()),
        ]);
        let node = leaf(keccak256(expected.contract.as_bytes()), &account);
        rebind_account(&mut expected, &mut manifest, &mut record, node);
        let result = Verifier::default().verify(&expected, &manifest, &record);
        if succeeds {
            assert_eq!(result.unwrap(), StorageOutcome::Present(record.value));
        } else {
            let error = result.unwrap_err();
            if encoded.len() == 1 && encoded[0] >= 0x80 {
                assert_eq!(
                    error.kind,
                    FailureKind::MalformedNode {
                        stage: ProofStage::Storage,
                        index: 0
                    }
                );
            } else {
                assert_eq!(error.kind, FailureKind::InvalidStorageRlp, "{encoded:?}");
            }
        }
    }
}

#[test]
fn absent_contract_is_a_contextual_error_rather_than_a_zero_read() {
    let (mut expected, mut manifest, mut record) = fixtures().remove(0);
    rebind_account(&mut expected, &mut manifest, &mut record, vec![0x80]);
    let error = Verifier::default()
        .verify(&expected, &manifest, &record)
        .unwrap_err();
    assert_eq!(error.kind, FailureKind::AccountAbsent);
    assert_eq!(error.contract, expected.contract);
    assert_eq!(error.storage_key, expected.storage_key);
    assert!(!error.to_string().is_empty());
}

#[test]
fn all_compact_flag_bytes_are_fallible_even_at_a_matching_root() {
    let (expected, manifest, record) = fixtures().remove(0);
    for flag in 0..=255u8 {
        let (mut expected, mut manifest, mut record) =
            (expected.clone(), manifest.clone(), record.clone());
        let node = list(&[alloy_rlp::encode(&[flag][..]), vec![0x80]]);
        rebind_account(&mut expected, &mut manifest, &mut record, node);
        assert!(
            Verifier::default()
                .verify(&expected, &manifest, &record)
                .is_err()
        );
    }
}
