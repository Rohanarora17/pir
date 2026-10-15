//! Retained upstream minimal-preset Deneb test vectors; never Sepolia/mainnet identity.
use crate::{Checkpoint, NetworkConfig, NetworkIdentity};
use alloy::primitives::B256;
use helios_consensus_core::{
    consensus_spec::MinimalConsensusSpec,
    types::{Bootstrap, FinalityUpdate, Fork, Forks, GenericUpdate, OptimisticUpdate, Update},
};
use serde::Deserialize;
use ssz::Decode;
use std::path::{Path, PathBuf};

pub const FIXTURE_CHAIN_ID: u64 = 0; // Local test label; not inferred from an execution header.
pub type Spec = MinimalConsensusSpec;
#[derive(Deserialize)]
struct Meta {
    genesis_validators_root: B256,
    trusted_block_root: B256,
}
#[derive(Deserialize)]
struct Step {
    process_update: UpdateStep,
}
#[derive(Deserialize)]
pub struct UpdateStep {
    pub update: String,
    pub current_slot: u64,
    pub checks: Checks,
}
#[derive(Deserialize)]
pub struct Checks {
    pub finalized_header: HeaderCheck,
    pub optimistic_header: HeaderCheck,
}
#[derive(Deserialize)]
pub struct HeaderCheck {
    pub slot: u64,
    pub beacon_root: B256,
    pub execution_root: B256,
}
pub struct Fixture {
    pub config: NetworkConfig,
    pub checkpoint: Checkpoint,
    pub bootstrap: Bootstrap<Spec>,
    pub steps: Vec<(UpdateStep, GenericUpdate<Spec>)>,
}
fn decoded(path: &Path) -> Vec<u8> {
    let compressed = std::fs::read(path).expect("retained fixture");
    snap::raw::Decoder::new()
        .decompress_vec(&compressed)
        .expect("valid snappy fixture")
}
pub fn directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../upstream/helios-2af9b41887e069144ee56cd710f3ddca195473f8/ethereum/consensus-core/testdata/deneb/advance_finality_without_sync_committee")
}
pub fn load() -> Fixture {
    let dir = directory();
    let meta: Meta =
        serde_yaml::from_str(&std::fs::read_to_string(dir.join("meta.yaml")).unwrap()).unwrap();
    let config: serde_yaml::Value =
        serde_yaml::from_str(&std::fs::read_to_string(dir.join("config.yaml")).unwrap()).unwrap();
    let fork = |name: &str, active: bool| {
        let version = config[format!("{name}_FORK_VERSION")]
            .as_u64()
            .unwrap_or_else(|| {
                // Inactive later forks are absent from this retained Deneb config.
                match name {
                    "ELECTRA" => 0x05000001,
                    "FULU" => 0x06000001,
                    "GLOAS" => 0x07000001,
                    _ => panic!("missing fork version"),
                }
            });
        Fork {
            epoch: if active { 0 } else { u64::MAX },
            fork_version: (version as u32).to_be_bytes().into(),
        }
    };
    let forks = Forks {
        genesis: fork("GENESIS", true),
        altair: fork("ALTAIR", true),
        bellatrix: fork("BELLATRIX", true),
        capella: fork("CAPELLA", true),
        deneb: fork("DENEB", true),
        electra: fork("ELECTRA", false),
        fulu: fork("FULU", false),
        gloas: fork("GLOAS", false),
    };
    let bootstrap = Bootstrap::from_ssz_bytes(&decoded(&dir.join("bootstrap.ssz_snappy"))).unwrap();
    let steps: Vec<Step> =
        serde_yaml::from_str(&std::fs::read_to_string(dir.join("steps.yaml")).unwrap()).unwrap();
    let steps = steps
        .into_iter()
        .map(|s| {
            let update = load_update(&dir.join(format!("{}.ssz_snappy", s.process_update.update)));
            (s.process_update, update)
        })
        .collect();
    Fixture {
        config: NetworkConfig {
            identity: NetworkIdentity {
                chain_id: FIXTURE_CHAIN_ID,
                genesis_validators_root: meta.genesis_validators_root,
            },
            forks,
        },
        checkpoint: Checkpoint {
            beacon_root: meta.trusted_block_root,
            source: "retained Helios minimal Deneb meta.yaml; synthetic test trust anchor only"
                .into(),
        },
        bootstrap,
        steps,
    }
}

/// Trusted retained fixture files only; not a bounded network decoder.
pub fn load_update(path: &Path) -> GenericUpdate<Spec> {
    let bytes = decoded(path);
    if let Ok(v) = Update::<Spec>::from_ssz_bytes(&bytes) {
        GenericUpdate::from(&v)
    } else if let Ok(v) = FinalityUpdate::<Spec>::from_ssz_bytes(&bytes) {
        GenericUpdate::from(&v)
    } else {
        GenericUpdate::from(&OptimisticUpdate::<Spec>::from_ssz_bytes(&bytes).unwrap())
    }
}
