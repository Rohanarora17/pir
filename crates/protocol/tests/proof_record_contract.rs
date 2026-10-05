use pvrpc_protocol::{
    Address, DatasetSpec, Hash32, HexBytes, MANIFEST_SCHEMA_VERSION, PirParameters, ProofRecord,
    RECORD_SCHEMA_VERSION, RecordError, SnapshotManifest, SnapshotRef,
};

fn snapshot() -> SnapshotRef {
    SnapshotRef {
        chain_id: 1,
        block_number: 22_000_000,
        block_hash: Hash32::new([0x11; 32]),
        state_root: Hash32::new([0x22; 32]),
        generation: 7,
    }
}

fn manifest() -> SnapshotManifest {
    SnapshotManifest {
        schema_version: MANIFEST_SCHEMA_VERSION,
        snapshot: snapshot(),
        contract: Address::new([0x33; 20]),
        mapping_slot: Hash32::new([0x03; 32]),
        dataset: DatasetSpec {
            record_count: 10,
            record_size: 4_096,
        },
        pir: PirParameters {
            scheme: "ikpir".into(),
            parameter_set: "development".into(),
            query_count: 1,
        },
        account_proof: vec![HexBytes::new(vec![0xf8, 0x01])],
    }
}

fn record(storage_key: Hash32) -> ProofRecord {
    ProofRecord {
        schema_version: RECORD_SCHEMA_VERSION,
        snapshot: snapshot(),
        record_index: 4,
        storage_key,
        value: Hash32::new([0x44; 32]),
        storage_proof: vec![HexBytes::new(vec![0xf8, 0x02])],
    }
}

#[test]
fn accepts_a_record_bound_to_the_manifest_and_requested_key() {
    let requested_key = Hash32::new([0x55; 32]);

    assert_eq!(
        record(requested_key).validate_for(&manifest(), &requested_key),
        Ok(())
    );
}

#[test]
fn rejects_a_record_from_another_snapshot() {
    let requested_key = Hash32::new([0x55; 32]);
    let mut record = record(requested_key);
    record.snapshot.generation += 1;

    assert_eq!(
        record.validate_for(&manifest(), &requested_key),
        Err(RecordError::SnapshotMismatch)
    );
}

#[test]
fn rejects_a_record_for_another_storage_key() {
    let requested_key = Hash32::new([0x55; 32]);
    let returned_key = Hash32::new([0x66; 32]);

    assert_eq!(
        record(returned_key).validate_for(&manifest(), &requested_key),
        Err(RecordError::StorageKeyMismatch)
    );
}

#[test]
fn rejects_an_out_of_bounds_or_proofless_record() {
    let requested_key = Hash32::new([0x55; 32]);

    let mut out_of_bounds = record(requested_key);
    out_of_bounds.record_index = 10;
    assert_eq!(
        out_of_bounds.validate_for(&manifest(), &requested_key),
        Err(RecordError::RecordIndexOutOfBounds {
            index: 10,
            record_count: 10,
        })
    );

    let mut missing_proof = record(requested_key);
    missing_proof.storage_proof.clear();
    assert_eq!(
        missing_proof.validate_for(&manifest(), &requested_key),
        Err(RecordError::MissingStorageProof)
    );

    let mut empty_node = record(requested_key);
    empty_node.storage_proof[0] = HexBytes::new(Vec::new());
    assert_eq!(
        empty_node.validate_for(&manifest(), &requested_key),
        Err(RecordError::EmptyStorageProofNode { index: 0 })
    );
}

#[test]
fn permits_a_zero_balance_when_a_proof_is_present() {
    let requested_key = Hash32::new([0x55; 32]);
    let mut zero_balance = record(requested_key);
    zero_balance.value = Hash32::new([0; 32]);

    assert_eq!(
        zero_balance.validate_for(&manifest(), &requested_key),
        Ok(())
    );
}
