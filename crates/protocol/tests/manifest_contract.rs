use pvrpc_protocol::{
    Address, DatasetSpec, Hash32, HexBytes, MANIFEST_SCHEMA_VERSION, ManifestError, PirParameters,
    SnapshotManifest, SnapshotRef,
};

fn valid_manifest() -> SnapshotManifest {
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

#[test]
fn accepts_a_complete_snapshot_manifest() {
    let manifest = valid_manifest();

    assert_eq!(manifest.validate(), Ok(()));
}

#[test]
fn rejects_an_unsupported_schema_version() {
    let mut manifest = valid_manifest();
    manifest.schema_version += 1;

    assert_eq!(
        manifest.validate(),
        Err(ManifestError::UnsupportedSchema {
            expected: MANIFEST_SCHEMA_VERSION,
            actual: MANIFEST_SCHEMA_VERSION + 1,
        })
    );
}

#[test]
fn rejects_an_incomplete_snapshot_manifest() {
    let cases = [
        (
            {
                let mut manifest = valid_manifest();
                manifest.snapshot.chain_id = 0;
                manifest
            },
            ManifestError::ZeroChainId,
        ),
        (
            {
                let mut manifest = valid_manifest();
                manifest.snapshot.generation = 0;
                manifest
            },
            ManifestError::ZeroGeneration,
        ),
        (
            {
                let mut manifest = valid_manifest();
                manifest.contract = Address::new([0; 20]);
                manifest
            },
            ManifestError::ZeroContract,
        ),
        (
            {
                let mut manifest = valid_manifest();
                manifest.dataset.record_count = 0;
                manifest
            },
            ManifestError::EmptyDataset,
        ),
        (
            {
                let mut manifest = valid_manifest();
                manifest.dataset.record_size = 0;
                manifest
            },
            ManifestError::ZeroRecordSize,
        ),
        (
            {
                let mut manifest = valid_manifest();
                manifest.pir.scheme.clear();
                manifest
            },
            ManifestError::EmptyPirScheme,
        ),
        (
            {
                let mut manifest = valid_manifest();
                manifest.pir.parameter_set.clear();
                manifest
            },
            ManifestError::EmptyParameterSet,
        ),
        (
            {
                let mut manifest = valid_manifest();
                manifest.pir.query_count = 0;
                manifest
            },
            ManifestError::ZeroQueryCount,
        ),
        (
            {
                let mut manifest = valid_manifest();
                manifest.account_proof.clear();
                manifest
            },
            ManifestError::MissingAccountProof,
        ),
        (
            {
                let mut manifest = valid_manifest();
                manifest.account_proof[0] = HexBytes::new(Vec::new());
                manifest
            },
            ManifestError::EmptyAccountProofNode { index: 0 },
        ),
    ];

    for (manifest, expected) in cases {
        assert_eq!(manifest.validate(), Err(expected));
    }
}
