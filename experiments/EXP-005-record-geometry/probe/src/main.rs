use pvrpc_record_geometry::{CodecError, corpus, decode, encode, encoded_len};
use pvrpc_verifier::{StorageOutcome, Verifier};
use serde_json::{Value, json};
use std::{collections::BTreeMap, error::Error, path::Path};

fn distribution(values: &[usize]) -> Value {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let sum: usize = sorted.iter().sum();
    json!({"min": sorted[0], "median_nearest_rank": sorted[sorted.len().div_ceil(2)-1],
        "p95_nearest_rank": sorted[(sorted.len()*95).div_ceil(100)-1], "max": sorted[sorted.len()-1],
        "mean": sum as f64 / sorted.len() as f64, "total": sum})
}

fn main() -> Result<(), Box<dyn Error>> {
    let directory = std::env::args()
        .nth(1)
        .ok_or("usage: pvrpc-record-geometry RETAINED_CORPUS_DIRECTORY")?;
    let corpus = corpus::load(Path::new(&directory), 4096)?;
    let mut json_sizes = Vec::new();
    let mut binary_sizes = Vec::new();
    let mut proof_sizes = Vec::new();
    let mut node_counts = Vec::new();
    let mut rows = Vec::new();
    let mut unique: BTreeMap<Vec<u8>, usize> = BTreeMap::new();
    let mut counts = (0, 0);
    for (context, record) in &corpus.records {
        let before = Verifier::default().verify(context, &corpus.manifest, record)?;
        let size = encoded_len(record)?;
        let frame = encode(record, size)?;
        let decoded = decode(&frame, size)?;
        if decoded != *record {
            return Err("round trip changed record fields".into());
        }
        let after = Verifier::default().verify(context, &corpus.manifest, &decoded)?;
        if after != before {
            return Err("round trip changed verified outcome".into());
        }
        match after {
            StorageOutcome::Present(_) => counts.0 += 1,
            StorageOutcome::Absent => counts.1 += 1,
        }
        let proof_bytes: usize = record
            .storage_proof
            .iter()
            .map(|n| n.as_bytes().len())
            .sum();
        let json_bytes = serde_json::to_vec(record)?.len();
        proof_sizes.push(proof_bytes);
        json_sizes.push(json_bytes);
        binary_sizes.push(size);
        node_counts.push(record.storage_proof.len());
        for node in &record.storage_proof {
            *unique.entry(node.as_bytes().to_vec()).or_default() += 1;
        }
        rows.push(json!({"record_index": record.record_index, "storage_nodes": record.storage_proof.len(),
            "storage_proof_bytes": proof_bytes, "json_record_bytes": json_bytes, "binary_frame_bytes": size,
            "outcome": if matches!(after, StorageOutcome::Absent) { "absence" } else { "inclusion" }}));
    }
    let binary_total: usize = binary_sizes.iter().sum();
    let json_total: usize = json_sizes.iter().sum();
    let mut padding = Vec::new();
    for capacity in [2048, 2560, 3072, 4096, 8192] {
        let mut supported = 0;
        let mut rejected = Vec::new();
        let mut verified = 0;
        let mut manifest = corpus.manifest.clone();
        manifest.dataset.record_size = capacity as u64;
        for (context, record) in &corpus.records {
            match encode(record, capacity) {
                Ok(frame) => {
                    let decoded = decode(&frame, capacity)?;
                    Verifier::default().verify(context, &manifest, &decoded)?;
                    supported += 1;
                    verified += 1;
                }
                Err(CodecError::TooLarge { .. }) => rejected.push(record.record_index),
                Err(error) => return Err(error.into()),
            }
        }
        let fits = supported == corpus.records.len();
        let database_bytes = fits.then_some(capacity * supported);
        padding.push(json!({"record_size_bytes": capacity, "supported_rows": supported,
            "rejected_row_indices": rejected, "verified_roundtrips": verified, "complete_corpus_fits": fits,
            "database_bytes": database_bytes, "padding_bytes": database_bytes.map(|n| n-binary_total),
            "padding_fraction_of_database": database_bytes.map(|n| (n-binary_total) as f64/n as f64),
            "amplification_over_binary": database_bytes.map(|n| n as f64/binary_total as f64),
            "json_supported_rows": json_sizes.iter().filter(|n| **n <= capacity).count()}));
    }
    let account_bytes: usize = corpus
        .manifest
        .account_proof
        .iter()
        .map(|n| n.as_bytes().len())
        .sum();
    let unique_bytes: usize = unique.keys().map(Vec::len).sum();
    let proof_total: usize = proof_sizes.iter().sum();
    let manifest_bytes = serde_json::to_vec(&corpus.manifest)?.len();
    let output = json!({
        "experiment_id": "EXP-005", "claim_type": "local result", "units": "bytes unless named otherwise",
        "input": {"snapshot": corpus.manifest.snapshot, "contract": corpus.manifest.contract,
            "mapping_slot": corpus.manifest.mapping_slot, "records": corpus.records.len()},
        "verification": {"inclusions": counts.0, "absences": counts.1, "root_authentication": "not_performed", "network_requests": 0},
        "shared_account_proof": {"nodes": corpus.manifest.account_proof.len(), "decoded_bytes_once": account_bytes,
            "hex_json_array_bytes_once": serde_json::to_vec(&corpus.manifest.account_proof)?.len(),
            "avoided_raw_bytes_vs_repeating_in_all_rows": account_bytes*(corpus.records.len()-1)},
        "public_manifest": {"compact_json_bytes_at_4096_row_setting": manifest_bytes,
            "pir_fields": "structural placeholders; no backend selected", "generation": "experiment-only generation 1"},
        "distributions": {"proof_node_count": distribution(&node_counts), "storage_proof_bytes": distribution(&proof_sizes),
            "compact_json_record_bytes": distribution(&json_sizes), "binary_frame_bytes": distribution(&binary_sizes)},
        "binary_savings_over_compact_json_bytes": json_total-binary_total,
        "storage_node_deduplication": {"references": node_counts.iter().sum::<usize>(), "unique_nodes": unique.len(),
            "unique_raw_bytes": unique_bytes, "repeated_raw_bytes": proof_total-unique_bytes,
            "duplicate_fraction_of_raw_bytes": (proof_total-unique_bytes) as f64/proof_total as f64,
            "simple_hash32_length4_store_bytes": unique_bytes+unique.len()*36,
            "comparison_only": "excludes PIR padding, index, path discovery, extra queries and protocol implementation"},
        "padding_candidates": padding,
        "whole_dataset_baseline": {"unpadded_binary_frames_bytes": binary_total,
            "unpadded_binary_plus_public_manifest_bytes": binary_total+manifest_bytes,
            "scope": "local serialized byte count only; not measured network traffic or latency"},
        "records": rows
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
