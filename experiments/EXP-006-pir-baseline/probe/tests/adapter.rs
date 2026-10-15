use pvrpc_pir_baseline::{experiment, wire};
use std::path::PathBuf;

#[test]
fn opaque_vectors_survive_framing_without_transmitting_a_plaintext_key() {
    let vectors = vec![vec![0x12345678, u32::MAX], vec![0]];
    let bytes = wire::encode(*b"PQ06", [7; 32], &vectors).unwrap();
    assert_eq!(bytes.len(), 60); // magic4 + binding32 + count4 + two lengths8 + cells12.
    assert_eq!(&bytes[44..48], &[0x12, 0x34, 0x56, 0x78]);
    assert_eq!(
        wire::decode(&bytes, *b"PQ06", [7; 32], &[2, 1]).unwrap(),
        vectors
    );
}

#[test]
fn malformed_or_cross_snapshot_messages_fail_before_backend_parsing() {
    let bytes = wire::encode(*b"PQ06", [7; 32], &[vec![1, 2]]).unwrap();
    for n in 0..bytes.len() {
        assert!(wire::decode(&bytes[..n], *b"PQ06", [7; 32], &[2]).is_err());
    }
    assert!(wire::decode(&bytes, *b"PR06", [7; 32], &[2]).is_err());
    assert_eq!(
        wire::decode(&bytes, *b"PQ06", [8; 32], &[2]),
        Err(wire::WireError::BindingMismatch)
    );
    assert_eq!(
        wire::decode(&bytes, *b"PQ06", [7; 32], &[3]),
        Err(wire::WireError::ShapeMismatch)
    );
    let mut excess = bytes.clone();
    excess.push(0);
    assert!(wire::decode(&excess, *b"PQ06", [7; 32], &[2]).is_err());
    let mut length = bytes;
    length[40..44].copy_from_slice(&u32::MAX.to_be_bytes());
    assert!(wire::decode(&length, *b"PQ06", [7; 32], &[2]).is_err());
}

#[test]
fn real_private_reads_decode_to_verified_nonzero_and_absent_records() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../EXP-003-real-proof-corpus/retained");
    let result = experiment::run(&path, "simple", "index", 3072).unwrap();
    assert_eq!(result["verified_inclusions"], 24);
    assert_eq!(result["verified_absences"], 8);
    assert_eq!(result["queries_completed"], 32);
    assert_eq!(result["invalid_query_rejected"], true);
}

#[test]
fn keyword_reads_verify_the_same_values_and_query_both_segments() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../EXP-003-real-proof-corpus/retained");
    let result = experiment::run(&path, "simple", "keyword", 3072).unwrap();
    assert_eq!(result["verified_inclusions"], 24);
    assert_eq!(result["verified_absences"], 8);
    assert_eq!(result["queries_completed"], 32);
    assert_eq!(result["setup"]["segments"], 2);
    assert_eq!(result["cross_preprocessing_query_rejected"], true);
    assert_eq!(result["truncated_response_rejected"], true);
}

#[test]
fn vector_limits_are_enforced_for_both_outbound_and_inbound_messages() {
    assert_eq!(
        wire::encode(*b"PQ06", [0; 32], &[]),
        Err(wire::WireError::Limit)
    );
    assert_eq!(
        wire::encode(*b"PQ06", [0; 32], &[vec![]]),
        Err(wire::WireError::Limit)
    );
    assert_eq!(
        wire::encode(*b"PQ06", [0; 32], &[vec![1], vec![1], vec![1]]),
        Err(wire::WireError::Limit)
    );
    assert_eq!(
        wire::encode(*b"PQ06", [0; 32], &[vec![1; 65537]]),
        Err(wire::WireError::Limit)
    );
    let frame = wire::encode(*b"PQ06", [0; 32], &[vec![1; 65536]]).unwrap();
    assert_eq!(
        wire::decode(&frame, *b"PQ06", [0; 32], &[65536]).unwrap()[0].len(),
        65536
    );
    assert_eq!(
        wire::decode(&frame, *b"PQ06", [0; 32], &[usize::MAX]),
        Err(wire::WireError::Limit)
    );
}
