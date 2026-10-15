use pvrpc_protocol::{Hash32, HexBytes};
use pvrpc_record_geometry::{CodecError, corpus, decode, encode, encoded_len};
use pvrpc_verifier::{StorageOutcome, Verifier};
use std::path::PathBuf;

fn corpus() -> corpus::Corpus {
    corpus::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../EXP-003-real-proof-corpus/retained"),
        4096,
    )
    .unwrap()
}

#[test]
fn all_real_rows_survive_encoding_padding_decoding_and_native_verification() {
    let corpus = corpus();
    let mut counts = (0, 0);
    for (context, record) in &corpus.records {
        let exact = encoded_len(record).unwrap();
        assert_eq!(
            decode(&encode(record, exact).unwrap(), exact).unwrap(),
            *record
        );
        let frame = encode(record, 4096).unwrap();
        let decoded = decode(&frame, 4096).unwrap();
        assert_eq!(decoded, *record);
        match Verifier::default()
            .verify(context, &corpus.manifest, &decoded)
            .unwrap()
        {
            StorageOutcome::Present(_) => counts.0 += 1,
            StorageOutcome::Absent => counts.1 += 1,
        }
        assert!(matches!(
            encode(record, exact - 1),
            Err(CodecError::TooLarge { .. })
        ));
    }
    assert_eq!(counts, (24, 8));
}

#[test]
fn a_well_formed_frame_does_not_authenticate_a_forged_value_or_binding() {
    let corpus = corpus();
    for (context, record) in &corpus.records {
        for mutation in 0..5 {
            let mut forged = record.clone();
            match mutation {
                0 => {
                    forged.value = if forged.value == Hash32::new([0; 32]) {
                        Hash32::new([1; 32])
                    } else {
                        Hash32::new([0; 32])
                    }
                }
                1 => forged.storage_key = Hash32::new([1; 32]),
                2 => forged.snapshot.generation += 1,
                3 => forged.record_index = (forged.record_index + 1) % 32,
                _ => forged.storage_proof[0] = HexBytes::new(vec![0xff]),
            }
            let decoded = decode(&encode(&forged, 4096).unwrap(), 4096).unwrap();
            assert!(
                Verifier::default()
                    .verify(context, &corpus.manifest, &decoded)
                    .is_err()
            );
        }
    }
}
