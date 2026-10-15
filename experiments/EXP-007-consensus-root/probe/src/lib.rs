//! Experimental consensus-only boundary. This crate is outside the production workspace.
//!
//! A checkpoint's provenance, the network config and the clock are trusted inputs. Only this
//! wrapper owns the mutable Helios store. No `force_update` or provider-root import is exposed.
use alloy::primitives::B256;
use helios_consensus_core::{
    apply_bootstrap, apply_generic_update,
    consensus_spec::ConsensusSpec,
    types::{Bootstrap, Forks, GenericUpdate, LightClientStore},
    verify_bootstrap, verify_generic_update,
};
use pvrpc_protocol::{Hash32, SnapshotRef};
use serde::Serialize;
use tree_hash::TreeHash;

pub mod fixture;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NetworkIdentity {
    pub chain_id: u64,
    pub genesis_validators_root: B256,
}
#[derive(Clone, Debug)]
pub struct NetworkConfig {
    pub identity: NetworkIdentity,
    pub forks: Forks,
}
#[derive(Clone, Debug, Serialize)]
pub struct Checkpoint {
    pub beacon_root: B256,
    pub source: String,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Freshness {
    pub max_checkpoint_age_slots: u64,
    pub max_finalized_lag_slots: u64,
}
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RootError {
    #[error("no post-bootstrap quorum-finalized execution context is available")]
    Unavailable,
    #[error("consensus verification failed: {0}")]
    Consensus(String),
    #[error("checkpoint is too old")]
    CheckpointTooOld,
    #[error("clock precedes an accepted header or last verified observation")]
    ClockRegression,
    #[error("finalized context exceeds the caller's freshness limit")]
    Stale,
    #[error("configured network differs from requested network")]
    WrongNetwork,
    #[error("provider snapshot disagrees with finalized {0}")]
    SnapshotMismatch(&'static str),
    #[error("header does not supply a supported execution payload header")]
    UnsupportedHeader,
    #[error("checkpoint provenance must be explicit")]
    MissingCheckpointSource,
}
// Private construction and no Deserialize: provider JSON cannot create this capability.
#[derive(Debug, Serialize)]
pub struct FinalizedExecutionContext {
    network: NetworkIdentity,
    beacon_slot: u64,
    beacon_root: B256,
    execution_block_number: u64,
    execution_block_hash: Hash32,
    execution_state_root: Hash32,
    checked_at_slot: u64,
    checkpoint: Checkpoint,
    freshness: Freshness,
}
impl FinalizedExecutionContext {
    /// Admit only the exact execution context. Consensus does not authenticate a publisher's
    /// generation, corpus, contract, layout or preprocessing commitments.
    pub fn check_snapshot(&self, snapshot: &SnapshotRef, now_slot: u64) -> Result<(), RootError> {
        if now_slot < self.checked_at_slot {
            return Err(RootError::ClockRegression);
        }
        check_lag(
            now_slot,
            self.beacon_slot,
            self.freshness.max_finalized_lag_slots,
        )?;
        if snapshot.chain_id != self.network.chain_id {
            return Err(RootError::SnapshotMismatch("chain_id"));
        }
        if snapshot.block_number != self.execution_block_number {
            return Err(RootError::SnapshotMismatch("block_number"));
        }
        if snapshot.block_hash != self.execution_block_hash {
            return Err(RootError::SnapshotMismatch("block_hash"));
        }
        if snapshot.state_root != self.execution_state_root {
            return Err(RootError::SnapshotMismatch("state_root"));
        }
        Ok(())
    }
    pub fn beacon_slot(&self) -> u64 {
        self.beacon_slot
    }
}
pub struct RootTracker<S: ConsensusSpec> {
    config: NetworkConfig,
    checkpoint: Checkpoint,
    freshness: Freshness,
    store: LightClientStore<S>,
    last_verified_slot: u64,
    advanced_finality: bool,
}
fn check_lag(now: u64, slot: u64, max_lag: u64) -> Result<(), RootError> {
    let lag = now.checked_sub(slot).ok_or(RootError::ClockRegression)?;
    if lag > max_lag {
        return Err(RootError::Stale);
    }
    Ok(())
}
impl<S: ConsensusSpec> RootTracker<S> {
    pub fn bootstrap(
        config: NetworkConfig,
        checkpoint: Checkpoint,
        freshness: Freshness,
        bootstrap: &Bootstrap<S>,
        now_slot: u64,
    ) -> Result<Self, RootError> {
        if checkpoint.source.trim().is_empty() {
            return Err(RootError::MissingCheckpointSource);
        }
        verify_bootstrap(bootstrap, checkpoint.beacon_root, &config.forks)
            .map_err(|e| RootError::Consensus(e.to_string()))?;
        let slot = bootstrap.header().beacon().slot;
        let age = now_slot
            .checked_sub(slot)
            .ok_or(RootError::ClockRegression)?;
        if age > freshness.max_checkpoint_age_slots {
            return Err(RootError::CheckpointTooOld);
        }
        let mut store = LightClientStore::default();
        apply_bootstrap(&mut store, bootstrap);
        Ok(Self {
            config,
            checkpoint,
            freshness,
            store,
            last_verified_slot: now_slot,
            advanced_finality: false,
        })
    }
    pub fn process_update(
        &mut self,
        update: &GenericUpdate<S>,
        now_slot: u64,
    ) -> Result<(), RootError> {
        if now_slot < self.last_verified_slot {
            return Err(RootError::ClockRegression);
        }
        // Verification is read-only. A rejected update cannot partially change the store.
        verify_generic_update(
            update,
            now_slot,
            &self.store,
            self.config.identity.genesis_validators_root,
            &self.config.forks,
        )
        .map_err(|e| RootError::Consensus(e.to_string()))?;
        let before = self.store.finalized_header.beacon().slot;
        apply_generic_update(&mut self.store, update);
        self.advanced_finality |= self.store.finalized_header.beacon().slot > before;
        self.last_verified_slot = now_slot;
        Ok(())
    }
    pub fn finalized(
        &self,
        expected: &NetworkIdentity,
        now_slot: u64,
    ) -> Result<FinalizedExecutionContext, RootError> {
        if expected != &self.config.identity {
            return Err(RootError::WrongNetwork);
        }
        if !self.advanced_finality {
            return Err(RootError::Unavailable);
        }
        if now_slot < self.last_verified_slot {
            return Err(RootError::ClockRegression);
        }
        let header = &self.store.finalized_header;
        check_lag(
            now_slot,
            header.beacon().slot,
            self.freshness.max_finalized_lag_slots,
        )?;
        // Bellatrix and Gloas do not expose the state-root payload header needed here.
        // A future hash-only fork needs independently hash-checked execution-header retrieval.
        let execution = header
            .execution()
            .map_err(|_| RootError::UnsupportedHeader)?;
        Ok(FinalizedExecutionContext {
            network: self.config.identity.clone(),
            beacon_slot: header.beacon().slot,
            beacon_root: header.beacon().tree_hash_root(),
            execution_block_number: *execution.block_number(),
            execution_block_hash: Hash32::new(execution.block_hash().0),
            execution_state_root: Hash32::new(execution.state_root().0),
            checked_at_slot: now_slot,
            checkpoint: self.checkpoint.clone(),
            freshness: self.freshness,
        })
    }
}

/// Diagnostic hashes only. This is not the root-admission capability.
#[derive(Debug, Serialize)]
pub struct HeadObservation {
    pub finalized_slot: u64,
    pub finalized_beacon_root: B256,
    pub finalized_execution_payload_root: B256,
    pub optimistic_slot: u64,
    pub optimistic_beacon_root: B256,
    pub optimistic_execution_payload_root: B256,
}
impl<S: ConsensusSpec> RootTracker<S> {
    pub fn observation(&self) -> HeadObservation {
        let finalized = &self.store.finalized_header;
        let optimistic = &self.store.optimistic_header;
        HeadObservation {
            finalized_slot: finalized.beacon().slot,
            finalized_beacon_root: finalized.beacon().tree_hash_root(),
            finalized_execution_payload_root: finalized.execution_root(),
            optimistic_slot: optimistic.beacon().slot,
            optimistic_beacon_root: optimistic.beacon().tree_hash_root(),
            optimistic_execution_payload_root: optimistic.execution_root(),
        }
    }
}
