use std::{fs, path::PathBuf, process::Command};

use pvrpc_protocol::{
    Address, DatasetSpec, Hash32, HexBytes, MANIFEST_SCHEMA_VERSION, PirParameters,
    SnapshotManifest, SnapshotRef,
};

fn manifest() -> SnapshotManifest {
    SnapshotManifest {
        schema_version: MANIFEST_SCHEMA_VERSION,
        snapshot: SnapshotRef {
            chain_id: 1,
            block_number: 22_000_000,
            block_hash: Hash32::new([0x11; 32]),
            state_root: Hash32::new([0x22; 32]),
            generation: 1,
        },
        contract: Address::new([0x33; 20]),
        mapping_slot: Hash32::new([0x03; 32]),
        dataset: DatasetSpec {
            record_count: 1_000,
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

fn temporary_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("pvrpc-{name}-{}.json", std::process::id()))
}

#[test]
fn validate_accepts_a_complete_manifest() {
    let path = temporary_path("valid-manifest");
    fs::write(&path, serde_json::to_vec_pretty(&manifest()).unwrap()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_pvrpc-manifest"))
        .args(["validate", path.to_str().unwrap()])
        .output()
        .unwrap();
    fs::remove_file(path).unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "valid snapshot manifest\n"
    );
}

#[test]
fn validate_rejects_a_structurally_invalid_manifest() {
    let path = temporary_path("invalid-manifest");
    let mut invalid = manifest();
    invalid.dataset.record_count = 0;
    fs::write(&path, serde_json::to_vec_pretty(&invalid).unwrap()).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_pvrpc-manifest"))
        .args(["validate", path.to_str().unwrap()])
        .output()
        .unwrap();
    fs::remove_file(path).unwrap();

    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("dataset has no records")
    );
}
