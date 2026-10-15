use alloy::primitives::B256;
use pvrpc_consensus_probe::{Freshness, RootError, RootTracker, fixture};
fn policy() -> Freshness {
    Freshness {
        max_checkpoint_age_slots: 128,
        max_finalized_lag_slots: 32,
    }
}
#[test]
fn rejects_wrong_checkpoint_instead_of_trusting_a_supplied_header() {
    let f = fixture::load();
    let mut checkpoint = f.checkpoint;
    checkpoint.beacon_root = B256::repeat_byte(7);
    assert!(matches!(
        RootTracker::<fixture::Spec>::bootstrap(f.config, checkpoint, policy(), &f.bootstrap, 41),
        Err(RootError::Consensus(_))
    ));
}
#[test]
fn bootstrap_alone_is_not_exported_as_new_finality() {
    let f = fixture::load();
    let identity = f.config.identity.clone();
    let tracker =
        RootTracker::<fixture::Spec>::bootstrap(f.config, f.checkpoint, policy(), &f.bootstrap, 41)
            .unwrap();
    assert!(matches!(
        tracker.finalized(&identity, 41),
        Err(RootError::Unavailable)
    ));
}
#[test]
fn advances_finality_only_after_verified_update_and_checks_freshness() {
    let f = fixture::load();
    let identity = f.config.identity.clone();
    let mut tracker =
        RootTracker::<fixture::Spec>::bootstrap(f.config, f.checkpoint, policy(), &f.bootstrap, 41)
            .unwrap();
    for (step, update) in &f.steps {
        tracker.process_update(update, step.current_slot).unwrap();
        let observation = tracker.observation();
        assert_eq!(
            observation.finalized_beacon_root,
            step.checks.finalized_header.beacon_root
        );
        assert_eq!(
            observation.finalized_execution_payload_root,
            step.checks.finalized_header.execution_root
        );
        assert_eq!(
            observation.optimistic_slot,
            step.checks.optimistic_header.slot
        );
        assert_eq!(
            observation.optimistic_beacon_root,
            step.checks.optimistic_header.beacon_root
        );
        assert_eq!(
            observation.optimistic_execution_payload_root,
            step.checks.optimistic_header.execution_root
        );
        assert_eq!(
            tracker
                .finalized(&identity, step.current_slot)
                .unwrap()
                .beacon_slot(),
            step.checks.finalized_header.slot
        );
    }
    assert!(matches!(
        tracker.finalized(&identity, 113),
        Err(RootError::Stale)
    ));
}

fn synced() -> (
    RootTracker<fixture::Spec>,
    pvrpc_consensus_probe::NetworkIdentity,
) {
    let f = fixture::load();
    let identity = f.config.identity.clone();
    let mut tracker =
        RootTracker::bootstrap(f.config, f.checkpoint, policy(), &f.bootstrap, 41).unwrap();
    for (step, update) in &f.steps {
        tracker.process_update(update, step.current_slot).unwrap();
    }
    (tracker, identity)
}
fn snapshot(
    context: &pvrpc_consensus_probe::FinalizedExecutionContext,
) -> pvrpc_protocol::SnapshotRef {
    let v = serde_json::to_value(context).unwrap();
    serde_json::from_value(serde_json::json!({
        "chain_id": v["network"]["chain_id"],
        "block_number": v["execution_block_number"],
        "block_hash": v["execution_block_hash"],
        "state_root": v["execution_state_root"],
        "generation": 1
    }))
    .unwrap()
}
#[test]
fn provider_must_match_chain_number_hash_and_root_generation_is_separate() {
    let (tracker, identity) = synced();
    let root = tracker.finalized(&identity, 97).unwrap();
    let good = snapshot(&root);
    root.check_snapshot(&good, 97).unwrap();
    let mut other = good.clone();
    other.chain_id += 1;
    assert_eq!(
        root.check_snapshot(&other, 97),
        Err(RootError::SnapshotMismatch("chain_id"))
    );
    let mut other = good.clone();
    other.block_number += 1;
    assert_eq!(
        root.check_snapshot(&other, 97),
        Err(RootError::SnapshotMismatch("block_number"))
    );
    let mut other = good.clone();
    other.block_hash = pvrpc_protocol::Hash32::new([0x42; 32]);
    assert_eq!(
        root.check_snapshot(&other, 97),
        Err(RootError::SnapshotMismatch("block_hash"))
    );
    let mut other = good.clone();
    other.state_root = pvrpc_protocol::Hash32::new([0x42; 32]);
    assert_eq!(
        root.check_snapshot(&other, 97),
        Err(RootError::SnapshotMismatch("state_root"))
    );
    let mut other = good.clone();
    other.generation += 1;
    root.check_snapshot(&other, 97).unwrap(); // Never claim that consensus authenticates generation.
    assert_eq!(root.check_snapshot(&good, 113), Err(RootError::Stale));
    assert_eq!(
        root.check_snapshot(&good, 96),
        Err(RootError::ClockRegression)
    );
}
#[test]
fn network_identity_includes_genesis_root_not_only_chain_number() {
    let (tracker, identity) = synced();
    let mut other = identity.clone();
    other.chain_id += 1;
    assert!(matches!(
        tracker.finalized(&other, 97),
        Err(RootError::WrongNetwork)
    ));
    let mut other = identity;
    other.genesis_validators_root = B256::repeat_byte(0x42);
    assert!(matches!(
        tracker.finalized(&other, 97),
        Err(RootError::WrongNetwork)
    ));
}
#[test]
fn checkpoint_age_source_and_future_time_are_explicit() {
    let f = fixture::load();
    let old = Freshness {
        max_checkpoint_age_slots: 0,
        max_finalized_lag_slots: 32,
    };
    assert!(matches!(
        RootTracker::<fixture::Spec>::bootstrap(
            f.config.clone(),
            f.checkpoint.clone(),
            old,
            &f.bootstrap,
            41
        ),
        Err(RootError::CheckpointTooOld)
    ));
    let mut checkpoint = f.checkpoint.clone();
    checkpoint.source = " ".into();
    assert!(matches!(
        RootTracker::<fixture::Spec>::bootstrap(
            f.config.clone(),
            checkpoint,
            policy(),
            &f.bootstrap,
            41
        ),
        Err(RootError::MissingCheckpointSource)
    ));
    let mut tracker =
        RootTracker::<fixture::Spec>::bootstrap(f.config, f.checkpoint, policy(), &f.bootstrap, 41)
            .unwrap();
    assert_eq!(
        tracker.process_update(&f.steps[0].1, 40),
        Err(RootError::ClockRegression)
    );
}
#[test]
fn corrupt_signature_and_branches_do_not_partially_change_state() {
    let f = fixture::load();
    let identity = f.config.identity.clone();
    let mut tracker =
        RootTracker::<fixture::Spec>::bootstrap(f.config, f.checkpoint, policy(), &f.bootstrap, 41)
            .unwrap();
    tracker.process_update(&f.steps[0].1, 41).unwrap();
    let before = serde_json::to_value(tracker.finalized(&identity, 41).unwrap()).unwrap();
    for kind in ["signature", "finality_branch", "execution_root"] {
        let mut bad = f.steps[1].1.clone();
        match kind {
            "signature" => bad.sync_aggregate.sync_committee_signature = Default::default(),
            "finality_branch" => bad.finality_branch.as_mut().unwrap()[0] = B256::repeat_byte(0x42),
            "execution_root" => {
                // Mutate the execution state root without replacing its SSZ body branch.
                let mut header = serde_json::to_value(&bad.finalized_header).unwrap();
                header["execution"]["state_root"] = serde_json::json!(B256::repeat_byte(0x42));
                bad.finalized_header = Some(serde_json::from_value(header).unwrap());
            }
            _ => unreachable!(),
        }
        assert!(
            matches!(
                tracker.process_update(&bad, 89),
                Err(RootError::Consensus(_))
            ),
            "{kind}"
        );
        assert_eq!(
            serde_json::to_value(tracker.finalized(&identity, 41).unwrap()).unwrap(),
            before
        );
    }
    tracker.process_update(&f.steps[1].1, 89).unwrap();
    assert_eq!(tracker.finalized(&identity, 89).unwrap().beacon_slot(), 72);
}
#[test]
fn wrong_genesis_domain_rejects_a_correct_fixture_signature() {
    let f = fixture::load();
    let mut config = f.config;
    config.identity.genesis_validators_root = B256::repeat_byte(0x42);
    let mut tracker =
        RootTracker::<fixture::Spec>::bootstrap(config, f.checkpoint, policy(), &f.bootstrap, 41)
            .unwrap();
    let err = tracker.process_update(&f.steps[0].1, 41).unwrap_err();
    assert!(
        matches!(&err, RootError::Consensus(message) if message.contains("signature")),
        "{err}"
    );
}

#[test]
fn timeout_does_not_promote_an_optimistic_head_to_finality() {
    // This companion vector has an optimistic head without new finality, then force_update.
    // Replay only the five verified updates; our API has no force-update transition.
    let f = fixture::load();
    let identity = f.config.identity.clone();
    let directory = fixture::directory()
        .parent()
        .unwrap()
        .join("light_client_sync");
    let steps: Vec<serde_yaml::Value> =
        serde_yaml::from_str(&std::fs::read_to_string(directory.join("steps.yaml")).unwrap())
            .unwrap();
    let policy = Freshness {
        max_checkpoint_age_slots: 128,
        max_finalized_lag_slots: 64,
    };
    let mut tracker =
        RootTracker::<fixture::Spec>::bootstrap(f.config, f.checkpoint, policy, &f.bootstrap, 41)
            .unwrap();
    for step in &steps[..5] {
        let step = &step["process_update"];
        let update = fixture::load_update(
            &directory.join(format!("{}.ssz_snappy", step["update"].as_str().unwrap())),
        );
        tracker
            .process_update(&update, step["current_slot"].as_u64().unwrap())
            .unwrap();
    }
    assert_eq!(tracker.finalized(&identity, 131).unwrap().beacon_slot(), 96);
    assert!(matches!(
        tracker.finalized(&identity, 194),
        Err(RootError::Stale)
    ));
    assert_eq!(tracker.observation().finalized_slot, 96); // Upstream force_update would promote 130.
}

#[test]
fn exported_state_root_is_the_execution_trie_root_not_an_ssz_root() {
    let (tracker, identity) = synced();
    let context = tracker.finalized(&identity, 97).unwrap();
    let v = serde_json::to_value(&context).unwrap();
    assert_eq!(v["execution_block_number"], 28);
    assert_eq!(
        v["execution_block_hash"],
        "0xfe530efb8eb3ef8c0911d9d4fed422ab2c5ae9f7171bb73cd99c28152e6773f8"
    );
    assert_eq!(
        v["execution_state_root"],
        serde_json::json!(B256::repeat_byte(0x20))
    );
    assert_ne!(
        v["execution_state_root"],
        serde_json::json!(tracker.observation().finalized_execution_payload_root)
    );
    assert_ne!(v["execution_state_root"], v["beacon_root"]);
}
