use pvrpc_protocol::{
    Address, DatasetSpec, Hash32, HexBytes, MANIFEST_SCHEMA_VERSION, PirParameters,
    SnapshotManifest, SnapshotRef,
};

#[test]
fn ethereum_bytes_use_lowercase_prefixed_hex_in_json() {
    assert_eq!(
        serde_json::to_string(&Hash32::new([0xab; 32])).unwrap(),
        format!("\"0x{}\"", "ab".repeat(32))
    );
    assert_eq!(
        serde_json::to_string(&Address::new([0xcd; 20])).unwrap(),
        format!("\"0x{}\"", "cd".repeat(20))
    );
    assert_eq!(
        serde_json::to_string(&HexBytes::new(vec![0xf8, 0x01])).unwrap(),
        "\"0xf801\""
    );
}

#[test]
fn ethereum_bytes_round_trip_through_json() {
    let hash = Hash32::new([0x11; 32]);
    let address = Address::new([0x22; 20]);
    let bytes = HexBytes::new(vec![0x00, 0xab, 0xff]);

    assert_eq!(
        serde_json::from_str::<Hash32>(&serde_json::to_string(&hash).unwrap()).unwrap(),
        hash
    );
    assert_eq!(
        serde_json::from_str::<Address>(&serde_json::to_string(&address).unwrap()).unwrap(),
        address
    );
    assert_eq!(
        serde_json::from_str::<HexBytes>(&serde_json::to_string(&bytes).unwrap()).unwrap(),
        bytes
    );
}

#[test]
fn fixed_size_types_reject_the_wrong_number_of_bytes() {
    assert!(serde_json::from_str::<Hash32>("\"0x01\"").is_err());
    assert!(serde_json::from_str::<Address>("\"0x01\"").is_err());
    assert!(serde_json::from_str::<HexBytes>("\"01\"").is_err());
    assert!(serde_json::from_str::<HexBytes>("\"0x1\"").is_err());
    assert!(serde_json::from_str::<HexBytes>("\"0xzz\"").is_err());
}

#[test]
fn manifests_reject_unknown_fields_instead_of_ignoring_typos() {
    let manifest = SnapshotManifest {
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
    };
    let mut json = serde_json::to_value(manifest).unwrap();
    json.as_object_mut().unwrap().insert(
        "state_rooot".into(),
        serde_json::Value::String("typo".into()),
    );

    assert!(serde_json::from_value::<SnapshotManifest>(json).is_err());
}
