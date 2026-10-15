//! Native Ethereum proof verification, conditional on a caller-supplied execution root.
//!
//! This crate performs no networking, PIR, consensus synchronization or fallback reads.
//! The caller must obtain the expected snapshot independently of the returned proof bundle.
mod node;
#[cfg(test)]
mod node_tests;

use alloy_primitives::{B256, U256, keccak256};
use alloy_rlp::Decodable;
use eth_trie::{EthTrie, MemoryDB, Trie};
use pvrpc_protocol::{
    Address, Hash32, HexBytes, ProofRecord, RecordError, SnapshotManifest, SnapshotRef,
};
use std::{fmt, sync::Arc};

#[derive(Clone, Debug)]
/// Expected request context supplied by the caller, never copied from an untrusted response.
/// A light client must establish root provenance separately; this type makes no such claim.
pub struct ReadContext {
    pub snapshot: SnapshotRef,
    pub contract: Address,
    pub storage_key: Hash32,
    pub record_index: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageOutcome {
    Present(Hash32),
    /// Authenticated nonmembership. Under Ethereum storage semantics this means zero.
    Absent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProofStage {
    Account,
    Storage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimitKind {
    NodeCount,
    NodeBytes,
    TotalBytes,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FailureKind {
    SnapshotMismatch,
    ContractMismatch,
    RecordIndexMismatch,
    InvalidRecord(RecordError),
    LimitExceeded { stage: ProofStage, limit: LimitKind },
    MalformedNode { stage: ProofStage, index: usize },
    InvalidProof { stage: ProofStage },
    AccountAbsent,
    InvalidAccountRlp,
    InvalidStorageRlp,
    NonCanonicalStorageZero,
    ValueMismatch { verified: Hash32 },
}

/// Library-neutral diagnostics retain the request identity even when the backend collapses errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationError {
    pub contract: Address,
    pub storage_key: Hash32,
    pub kind: FailureKind,
}

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "proof verification failed for {:?}, key {:?}: {:?}",
            self.contract, self.storage_key, self.kind
        )
    }
}

impl std::error::Error for VerificationError {}

#[derive(Clone, Copy, Debug)]
/// Local admission limits on already decoded proof bytes, not protocol record geometry.
/// Callers must separately bound transport/JSON sizes before allocating decoded objects.
pub struct VerificationLimits {
    pub max_nodes_per_proof: usize,
    pub max_node_bytes: usize,
    pub max_total_bytes: usize,
}

impl Default for VerificationLimits {
    fn default() -> Self {
        Self {
            max_nodes_per_proof: 128,
            max_node_bytes: 4096,
            max_total_bytes: 1024 * 1024,
        }
    }
}

#[derive(Default)]
pub struct Verifier {
    limits: VerificationLimits,
}

impl Verifier {
    pub fn new(limits: VerificationLimits) -> Self {
        Self { limits }
    }

    /// Verify the envelope, recover the account, then use its checked storage root.
    /// An absent account is an error for this contract-storage workload. A missing or
    /// incomplete storage proof is an error, never `StorageOutcome::Absent`.
    pub fn verify(
        &self,
        expected: &ReadContext,
        manifest: &SnapshotManifest,
        record: &ProofRecord,
    ) -> Result<StorageOutcome, VerificationError> {
        self.verify_inner(expected, manifest, record)
            .map_err(|kind| VerificationError {
                contract: expected.contract,
                storage_key: expected.storage_key,
                kind,
            })
    }

    fn verify_inner(
        &self,
        expected: &ReadContext,
        manifest: &SnapshotManifest,
        record: &ProofRecord,
    ) -> Result<StorageOutcome, FailureKind> {
        if manifest.snapshot != expected.snapshot {
            return Err(FailureKind::SnapshotMismatch);
        }
        if manifest.contract != expected.contract {
            return Err(FailureKind::ContractMismatch);
        }
        if record.record_index != expected.record_index {
            return Err(FailureKind::RecordIndexMismatch);
        }
        // Check counts and byte budgets before scanning or cloning attacker-controlled nodes.
        let mut remaining = self.limits.max_total_bytes;
        self.check_limits(&manifest.account_proof, ProofStage::Account, &mut remaining)?;
        self.check_limits(&record.storage_proof, ProofStage::Storage, &mut remaining)?;
        record
            .validate_for(manifest, &expected.storage_key)
            .map_err(FailureKind::InvalidRecord)?;
        self.check_nodes(&manifest.account_proof, ProofStage::Account)?;
        self.check_nodes(&record.storage_proof, ProofStage::Storage)?;

        let trie = EthTrie::new(Arc::new(MemoryDB::new(true)));
        let account_bytes = recover(
            &trie,
            B256::from(*expected.snapshot.state_root.as_bytes()),
            keccak256(expected.contract.as_bytes()),
            &manifest.account_proof,
            ProofStage::Account,
        )?
        .ok_or(FailureKind::AccountAbsent)?;
        let account =
            decode_exact::<Account>(&account_bytes).ok_or(FailureKind::InvalidAccountRlp)?;
        let storage = recover(
            &trie,
            account.storage_root,
            keccak256(expected.storage_key.as_bytes()),
            &record.storage_proof,
            ProofStage::Storage,
        )?;
        let (outcome, verified) = match storage {
            Some(bytes) => {
                let value = decode_exact::<U256>(&bytes).ok_or(FailureKind::InvalidStorageRlp)?;
                // Ethereum deletes zero-valued slots. Preserve absence as a distinct result.
                if value.is_zero() {
                    return Err(FailureKind::NonCanonicalStorageZero);
                }
                let value = Hash32::new(value.to_be_bytes());
                (StorageOutcome::Present(value), value)
            }
            None => (StorageOutcome::Absent, Hash32::new([0; 32])),
        };
        if record.value != verified {
            return Err(FailureKind::ValueMismatch { verified });
        }
        Ok(outcome)
    }

    fn check_limits(
        &self,
        nodes: &[HexBytes],
        stage: ProofStage,
        remaining: &mut usize,
    ) -> Result<(), FailureKind> {
        if nodes.len() > self.limits.max_nodes_per_proof {
            return Err(FailureKind::LimitExceeded {
                stage,
                limit: LimitKind::NodeCount,
            });
        }
        for node in nodes {
            let size = node.as_bytes().len();
            if size > self.limits.max_node_bytes {
                return Err(FailureKind::LimitExceeded {
                    stage,
                    limit: LimitKind::NodeBytes,
                });
            }
            *remaining = remaining
                .checked_sub(size)
                .ok_or(FailureKind::LimitExceeded {
                    stage,
                    limit: LimitKind::TotalBytes,
                })?;
        }
        Ok(())
    }

    fn check_nodes(&self, nodes: &[HexBytes], stage: ProofStage) -> Result<(), FailureKind> {
        for (index, node) in nodes.iter().enumerate() {
            node::validate(node.as_bytes())
                .map_err(|()| FailureKind::MalformedNode { stage, index })?;
        }
        Ok(())
    }
}

#[derive(alloy_rlp::RlpDecodable)]
struct Account {
    _nonce: u64,
    _balance: U256,
    storage_root: B256,
    _code_hash: B256,
}

fn decode_exact<T: Decodable>(bytes: &[u8]) -> Option<T> {
    let mut input = bytes;
    let value = T::decode(&mut input).ok()?;
    input.is_empty().then_some(value)
}

fn recover(
    trie: &EthTrie<MemoryDB>,
    root: B256,
    key: B256,
    nodes: &[HexBytes],
    stage: ProofStage,
) -> Result<Option<Vec<u8>>, FailureKind> {
    trie.verify_proof(
        root,
        key.as_slice(),
        nodes.iter().map(|node| node.as_bytes().to_vec()).collect(),
    )
    .map_err(|_| FailureKind::InvalidProof { stage })
}
