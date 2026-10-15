//! Local fixture import shared by the measurement executable and its integration tests.
use alloy_primitives::{Address, keccak256};
use pvrpc_protocol::{Hash32, ProofRecord, SnapshotManifest};
use pvrpc_verifier::ReadContext;
use serde_json::{Value, json};
use std::{
    error::Error,
    fs,
    path::{Component, Path},
};

pub struct Corpus {
    pub manifest: SnapshotManifest,
    pub records: Vec<(ReadContext, ProofRecord)>,
}

pub fn load(directory: &Path, record_size: usize) -> Result<Corpus, Box<dyn Error>> {
    let corpus: Value = serde_json::from_slice(&fs::read(directory.join("corpus.json"))?)?;
    let entries = corpus["records"]
        .as_array()
        .ok_or("records must be an array")?;
    if entries.is_empty() {
        return Err("empty corpus".into());
    }
    let mut snapshot = corpus["snapshot"].clone();
    snapshot["generation"] = json!(1);
    let mapping_slot = corpus["mapping_slot"]
        .as_u64()
        .ok_or("missing mapping slot")?;
    let manifest: SnapshotManifest = serde_json::from_value(json!({
        "schema_version": 1, "snapshot": snapshot, "contract": corpus["contract"],
        "mapping_slot": format!("0x{mapping_slot:064x}"),
        "dataset": { "record_count": entries.len(), "record_size": record_size },
        "pir": { "scheme": "geometry-experiment-only", "parameter_set": "not-selected", "query_count": 1 },
        "account_proof": corpus["account"]["accountProof"]
    }))?;
    manifest.validate()?;
    let mut records = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let proof_path = Path::new(entry["proof_file"].as_str().ok_or("missing proof path")?);
        if !proof_path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
        {
            return Err("proof path must stay relative to the corpus".into());
        }
        let response: Value = serde_json::from_slice(&fs::read(directory.join(proof_path))?)?;
        let record: ProofRecord = serde_json::from_value(json!({
            "schema_version": 1, "snapshot": snapshot, "record_index": entry["record_index"],
            "storage_key": entry["storage_key"], "value": entry["value"],
            "storage_proof": response["result"]["storageProof"][0]["proof"]
        }))?;
        let address: Address = entry["address"]
            .as_str()
            .ok_or("missing address")?
            .parse()?;
        let mut input = [0u8; 64];
        input[12..32].copy_from_slice(address.as_slice());
        input[56..].copy_from_slice(&mapping_slot.to_be_bytes());
        let context = ReadContext {
            snapshot: manifest.snapshot.clone(),
            contract: manifest.contract,
            storage_key: Hash32::new(keccak256(input).0),
            record_index: index as u64,
        };
        if record.record_index != context.record_index {
            return Err("corpus indices must be contiguous and ordered".into());
        }
        records.push((context, record));
    }
    Ok(Corpus { manifest, records })
}
