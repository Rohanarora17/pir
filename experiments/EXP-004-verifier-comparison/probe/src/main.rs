//! Experimental comparison, not the production verifier or a source of authenticated roots.

use alloy_primitives::{Address, B256, Bytes, U256, keccak256};
use alloy_trie::{Nibbles, TrieAccount};
use eth_trie::{EthTrie, MemoryDB, Trie};
use serde_json::{Value, json};
use std::{fs, path::Path, sync::Arc, time::Instant};

#[derive(Clone, Copy, Debug)]
enum Backend {
    Alloy,
    EthTrie,
}

impl Backend {
    fn name(self) -> &'static str {
        match self {
            Self::Alloy => "alloy-trie 0.9.8",
            Self::EthTrie => "eth_trie 0.6.1",
        }
    }

    fn check(self, root: B256, key: B256, value: Option<Vec<u8>>, nodes: &[Bytes]) -> bool {
        match self {
            Self::Alloy => {
                alloy_trie::proof::verify_proof(root, Nibbles::unpack(key), value, nodes).is_ok()
            }
            Self::EthTrie => {
                let trie = EthTrie::new(Arc::new(MemoryDB::new(true)));
                match trie.verify_proof(
                    root,
                    key.as_slice(),
                    nodes.iter().map(|n| n.to_vec()).collect(),
                ) {
                    Ok(actual) => actual == value,
                    Err(_) => false,
                }
            }
        }
    }
}

fn read(path: impl AsRef<Path>) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read retained fixture"))
        .expect("parse retained JSON")
}

fn text(value: &Value) -> &str {
    value.as_str().expect("fixture string")
}

fn nodes(value: &Value) -> Vec<Bytes> {
    value
        .as_array()
        .expect("proof array")
        .iter()
        .map(|n| text(n).parse().expect("proof bytes"))
        .collect()
}

fn account(proof: &Value) -> Vec<u8> {
    alloy_rlp::encode(TrieAccount {
        nonce: u64::from_str_radix(text(&proof["nonce"]).trim_start_matches("0x"), 16)
            .expect("nonce"),
        balance: text(&proof["balance"]).parse().expect("balance"),
        storage_root: text(&proof["storageHash"]).parse().expect("storage root"),
        code_hash: text(&proof["codeHash"]).parse().expect("code hash"),
    })
}

fn mapping_key(address: Address) -> B256 {
    let mut input = [0u8; 64];
    input[12..32].copy_from_slice(address.as_slice());
    input[63] = 3;
    keccak256(input)
}

fn verify(
    backend: Backend,
    root: B256,
    contract: Address,
    expected_key: B256,
    proof: &Value,
) -> bool {
    let reported_contract: Address = text(&proof["address"]).parse().expect("contract");
    let storage = &proof["storageProof"][0];
    let reported_key: U256 = text(&storage["key"]).parse().expect("storage key");
    if reported_contract != contract || B256::from(reported_key.to_be_bytes::<32>()) != expected_key
    {
        return false;
    }
    let storage_root: B256 = text(&proof["storageHash"]).parse().expect("storage root");
    let value: U256 = text(&storage["value"]).parse().expect("storage value");
    let expected = if value.is_zero() {
        None
    } else {
        Some(alloy_rlp::encode(value))
    };
    backend.check(
        root,
        keccak256(contract),
        Some(account(proof)),
        &nodes(&proof["accountProof"]),
    ) && backend.check(
        storage_root,
        keccak256(expected_key),
        expected,
        &nodes(&storage["proof"]),
    )
}

fn flip(hash: B256) -> B256 {
    let mut bytes = hash.0;
    bytes[31] ^= 1;
    B256::from(bytes)
}

fn require_rejection(rejections: &mut usize, outcome: bool, case: &str) {
    assert!(!outcome, "Accepted corrupt case: {case}");
    *rejections += 1;
}

fn run(backend: Backend, directory: &Path) -> Value {
    let corpus = read(directory.join("corpus.json"));
    let root: B256 = text(&corpus["snapshot"]["state_root"])
        .parse()
        .expect("root");
    let contract: Address = text(&corpus["contract"]).parse().expect("contract");
    let mut verified = 0;
    let mut absent = 0;
    let mut rejections = 0;
    let mut fixtures = Vec::new();
    for record in corpus["records"].as_array().expect("records") {
        let address: Address = text(&record["address"]).parse().expect("address");
        let key = mapping_key(address);
        assert_eq!(key.to_string(), text(&record["storage_key"]));
        let proof = read(directory.join(text(&record["proof_file"])))["result"].clone();
        assert!(
            verify(backend, root, contract, key, &proof),
            "Rejected genuine proof for {address}"
        );
        verified += 1;
        let value: U256 = text(&proof["storageProof"][0]["value"])
            .parse()
            .expect("value");
        absent += usize::from(value.is_zero());

        require_rejection(
            &mut rejections,
            verify(backend, flip(root), contract, key, &proof),
            "wrong state root",
        );
        require_rejection(
            &mut rejections,
            verify(backend, root, Address::ZERO, key, &proof),
            "wrong expected account",
        );
        let mut changed = proof.clone();
        changed["storageHash"] =
            json!(flip(text(&proof["storageHash"]).parse().unwrap()).to_string());
        require_rejection(
            &mut rejections,
            verify(backend, root, contract, key, &changed),
            "forged account storage root",
        );
        let mut changed = proof.clone();
        changed["storageProof"][0]["value"] = json!(format!("{:#x}", value + U256::from(1)));
        require_rejection(
            &mut rejections,
            verify(backend, root, contract, key, &changed),
            "altered value",
        );
        let mut changed = proof.clone();
        changed["storageProof"][0]["key"] = json!(flip(key).to_string());
        require_rejection(
            &mut rejections,
            verify(backend, root, contract, key, &changed),
            "substituted RPC key",
        );
        for field in ["accountProof", "storageProof"] {
            let mut changed = proof.clone();
            let array = if field == "accountProof" {
                &mut changed[field]
            } else {
                &mut changed[field][0]["proof"]
            };
            array.as_array_mut().unwrap().truncate(1);
            require_rejection(
                &mut rejections,
                verify(backend, root, contract, key, &changed),
                "truncated path",
            );
        }
        let mut changed = proof.clone();
        changed["storageProof"][0]["proof"] = json!([]);
        require_rejection(
            &mut rejections,
            verify(backend, root, contract, key, &changed),
            "missing storage proof",
        );
        let mut changed = proof.clone();
        changed["storageProof"][0]["proof"][0] = json!("0xc180");
        require_rejection(
            &mut rejections,
            verify(backend, root, contract, key, &changed),
            "malformed node",
        );
        if !value.is_zero() {
            let mut changed = proof.clone();
            changed["storageProof"][0]["value"] = json!("0x0");
            changed["storageProof"][0]["proof"]
                .as_array_mut()
                .unwrap()
                .truncate(1);
            require_rejection(
                &mut rejections,
                verify(backend, root, contract, key, &changed),
                "false absence with truncated proof",
            );
        }
        fixtures.push((key, proof));
    }
    // A self-consistent hash of invalid RLP must produce rejection rather than panic.
    let malformed = Bytes::from(vec![0xc1, 0x80]);
    require_rejection(
        &mut rejections,
        backend.check(keccak256(&malformed), B256::ZERO, None, &[malformed]),
        "invalid RLP at matching root",
    );
    let start = Instant::now();
    let rounds = 10;
    for _ in 0..rounds {
        for (key, proof) in &fixtures {
            assert!(verify(backend, root, contract, *key, proof));
        }
    }
    json!({
        "backend": backend.name(), "records_verified": verified,
        "nonzero_inclusions": verified - absent, "zero_exclusions": absent,
        "negative_cases_rejected": rejections, "timing_rounds": rounds,
        "total_verification_microseconds": start.elapsed().as_micros(),
        "root_authentication": "not_performed", "fixture_source": directory,
    })
}

fn main() {
    let directory = std::env::args()
        .nth(1)
        .expect("pass retained EXP-003 directory");
    for backend in [Backend::Alloy, Backend::EthTrie] {
        println!("{}", run(backend, Path::new(&directory)));
    }
}
