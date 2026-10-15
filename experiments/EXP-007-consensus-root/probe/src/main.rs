use helios_consensus_core::get_bits;
use pvrpc_consensus_probe::{Freshness, RootTracker, fixture};
use serde_json::json;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture::load();
    let identity = f.config.identity.clone();
    let policy = Freshness {
        max_checkpoint_age_slots: 128,
        max_finalized_lag_slots: 32,
    };
    let mut tracker =
        RootTracker::<fixture::Spec>::bootstrap(f.config, f.checkpoint, policy, &f.bootstrap, 41)?;
    let mut trace = Vec::new();
    for (step, update) in &f.steps {
        tracker.process_update(update, step.current_slot)?;
        let heads = tracker.observation();
        assert_eq!(heads.finalized_slot, step.checks.finalized_header.slot);
        assert_eq!(
            heads.finalized_beacon_root,
            step.checks.finalized_header.beacon_root
        );
        assert_eq!(
            heads.finalized_execution_payload_root,
            step.checks.finalized_header.execution_root
        );
        assert_eq!(heads.optimistic_slot, step.checks.optimistic_header.slot);
        assert_eq!(
            heads.optimistic_beacon_root,
            step.checks.optimistic_header.beacon_root
        );
        assert_eq!(
            heads.optimistic_execution_payload_root,
            step.checks.optimistic_header.execution_root
        );
        trace.push(
            json!({"fixture": step.update, "current_slot": step.current_slot,
            "participants": get_bits::<fixture::Spec>(&update.sync_aggregate.sync_committee_bits),
            "committee_size": 32, "heads": heads,
            "admitted_context": tracker.finalized(&identity, step.current_slot)?}),
        );
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "experiment_id": "EXP-007", "claim_type": "synthetic result",
            "network": "retained upstream minimal Deneb fixture; not Sepolia/mainnet",
            "helios_revision": "2af9b41887e069144ee56cd710f3ddca195473f8",
            "trust_anchor": "fixture meta.yaml; test-only", "live_sync": false,
            "force_update_used": false, "execution_rpc_used": false, "storage_query_sent": false,
            "old_sepolia_corpus_authenticated": false, "updates_verified": trace.len(), "trace": trace
        }))?
    );
    Ok(())
}
