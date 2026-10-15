use pvrpc_protocol::{Hash32, HexBytes, ProofRecord, SnapshotRef};
use pvrpc_record_geometry::{CodecError, decode, encode, encoded_len};

fn record() -> ProofRecord {
    ProofRecord {
        schema_version: 1,
        snapshot: SnapshotRef {
            chain_id: 11155111,
            block_number: 11919648,
            block_hash: Hash32::new([1; 32]),
            state_root: Hash32::new([2; 32]),
            generation: 1,
        },
        record_index: 7,
        storage_key: Hash32::new([3; 32]),
        value: Hash32::new([4; 32]),
        storage_proof: vec![HexBytes::new(vec![0xc2, 1, 2])],
    }
}

#[test]
fn round_trip_preserves_all_binding_fields_and_node_bytes() {
    let record = record();
    let frame = encode(&record, 256).unwrap();
    assert_eq!(frame.len(), 256);
    assert_eq!(&frame[..8], b"PVR0\0\0\0\xad"); // 166 field bytes + 4 length + 3 node bytes.
    assert_eq!(decode(&frame, 256).unwrap(), record);
    assert!(frame[181..].iter().all(|b| *b == 0));
}

#[test]
fn exact_capacity_succeeds_and_one_byte_short_fails_without_truncation() {
    let record = record();
    assert_eq!(encoded_len(&record).unwrap(), 181);
    assert!(encode(&record, 181).is_ok());
    assert_eq!(
        encode(&record, 180),
        Err(CodecError::TooLarge {
            needed: 181,
            capacity: 180
        })
    );
}

#[test]
fn rejects_truncated_frames_bad_lengths_padding_magic_and_versions() {
    let original = encode(&record(), 256).unwrap();
    for length in 0..256 {
        assert!(decode(&original[..length], 256).is_err());
    }
    let mut extra = original.clone();
    extra.push(0);
    assert!(decode(&extra, 256).is_err());
    for position in [0, 7, 11, 255] {
        let mut bad = original.clone();
        bad[position] ^= 1;
        assert!(decode(&bad, 256).is_err(), "position {position}");
    }
    assert!(decode(&original, 255).is_err());
    assert!(decode(&original, usize::MAX).is_err());
}

#[test]
fn rejects_hostile_counts_and_node_lengths_before_allocation() {
    let original = encode(&record(), 256).unwrap();
    // Count is at frame offset 172; first node length starts at 174.
    for (offset, bytes) in [
        (172, vec![0xff, 0xff]),
        (172, vec![0, 0]),
        (174, vec![0xff; 4]),
        (174, vec![0; 4]),
    ] {
        let mut bad = original.clone();
        bad[offset..offset + bytes.len()].copy_from_slice(&bytes);
        assert!(decode(&bad, 256).is_err());
    }
    for length in [0, 4097] {
        let mut bad = record();
        bad.storage_proof = vec![HexBytes::new(vec![1; length])];
        assert_eq!(encoded_len(&bad), Err(CodecError::InvalidProofShape));
    }
    let mut bad = record();
    bad.storage_proof = vec![HexBytes::new(vec![1]); 129];
    assert_eq!(encode(&bad, 1024), Err(CodecError::InvalidProofShape));
    assert_eq!(encode(&record(), usize::MAX), Err(CodecError::InvalidSize));
}
