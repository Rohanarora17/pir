//! Integration probe: recover authenticated account/storage data without RPC value hints.

use alloy_primitives::{Address, B256, Bytes, U256, keccak256};
use alloy_rlp::Decodable;
use eth_trie::{EthTrie, MemoryDB, Trie};
use serde_json::{Value, json};
use std::{fs, path::Path, sync::Arc};

#[derive(Debug, alloy_rlp::RlpDecodable)]
struct Account {
    nonce: u64,
    balance: U256,
    storage_root: B256,
    code_hash: B256,
}

fn nodes(value: &Value) -> Vec<Vec<u8>> {
    value
        .as_array()
        .expect("fixture proof array")
        .iter()
        .map(|node| node.as_str().unwrap().parse::<Bytes>().unwrap().to_vec())
        .collect()
}

fn main() {
    let directory = std::env::args().nth(1).expect("retained corpus directory");
    let reverse_order = std::env::args().any(|argument| argument == "--reverse-proof-order");
    let directory = Path::new(&directory);
    let corpus: Value =
        serde_json::from_slice(&fs::read(directory.join("corpus.json")).unwrap()).unwrap();
    let root: B256 = corpus["snapshot"]["state_root"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let contract: Address = corpus["contract"].as_str().unwrap().parse().unwrap();
    let verifier = EthTrie::new(Arc::new(MemoryDB::new(true)));
    // Only the proof nodes are consumed; account balance/nonce/storageHash/codeHash hints are ignored.
    let mut account_nodes = nodes(&corpus["account"]["accountProof"]);
    if reverse_order {
        account_nodes.reverse();
    }
    let account_bytes = verifier
        .verify_proof(root, keccak256(contract).as_slice(), account_nodes)
        .expect("valid account proof")
        .expect("WETH account exists");
    let mut account_rlp = account_bytes.as_slice();
    let account = Account::decode(&mut account_rlp).expect("Ethereum account RLP");
    assert!(account_rlp.is_empty());
    let mut inclusions = 0;
    let mut exclusions = 0;
    for record in corpus["records"].as_array().unwrap() {
        let address: Address = record["address"].as_str().unwrap().parse().unwrap();
        let mut mapping_input = [0u8; 64];
        mapping_input[12..32].copy_from_slice(address.as_slice());
        mapping_input[63] = 3;
        let expected_key = keccak256(mapping_input);
        let response: Value = serde_json::from_slice(
            &fs::read(directory.join(record["proof_file"].as_str().unwrap())).unwrap(),
        )
        .unwrap();
        // Consume only native proof nodes, never the RPC's storageHash, storage key or value.
        let mut proof_nodes = nodes(&response["result"]["storageProof"][0]["proof"]);
        if reverse_order {
            proof_nodes.reverse();
        }
        let value = verifier
            .verify_proof(
                account.storage_root,
                keccak256(expected_key).as_slice(),
                proof_nodes,
            )
            .expect("valid storage proof");
        let balance = match value {
            Some(value) => {
                inclusions += 1;
                let mut encoded = value.as_slice();
                let decoded = U256::decode(&mut encoded).expect("storage integer RLP");
                assert!(encoded.is_empty());
                decoded
            }
            None => {
                exclusions += 1;
                U256::ZERO
            }
        };
        // Retained reported values are used only as independent test expectations after recovery.
        let expected: U256 = record["value"].as_str().unwrap().parse().unwrap();
        assert_eq!(balance, expected);
    }
    println!(
        "{}",
        json!({
            "backend": "eth_trie 0.6.1", "account_source": "authenticated proof bytes",
            "account_nonce": account.nonce, "account_native_balance": account.balance.to_string(),
            "storage_root": account.storage_root.to_string(), "code_hash": account.code_hash.to_string(),
            "inclusions_recovered": inclusions, "exclusions_recovered": exclusions,
            "rpc_value_hints_used_for_verification": false, "root_authentication": "not_performed",
            "reversed_proof_order": reverse_order,
        })
    );
}
