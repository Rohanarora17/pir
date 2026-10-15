use pvrpc_pir_scale::{cell, plan, run, targets};

#[test]
fn resource_and_wire_limits_precede_any_database_allocation() {
    let p = plan(524288, 4096, 1 << 30).unwrap();
    assert_eq!(p["admitted"], false);
    assert_eq!(run(524288, 4096, 0, 4).unwrap()["status"], "skipped");
    assert!(plan(0, 3072, u64::MAX).is_err());
    assert!(plan(u32::MAX, 4096, u64::MAX).is_err());
    assert!(plan(32, 3000, u64::MAX).is_err());
    assert_eq!(
        plan(32, 3072, 0).unwrap()["skip_reason"],
        "estimated_memory_budget"
    );
    assert_eq!(
        plan(1 << 24, 4096, u64::MAX).unwrap()["skip_reason"],
        "existing_wire_width_limit"
    );
}

#[test]
fn deterministic_payload_and_samples_distinguish_rows_and_cover_edges() {
    let first: Vec<_> = (0..32).map(|o| cell(0, o)).collect();
    assert_eq!(&first[..8], &[92, 45, 233, 114, 207, 4, 73, 250]);
    assert_ne!(first, (0..32).map(|o| cell(1, o)).collect::<Vec<_>>());
    assert!(first.iter().all(|b| *b <= 255));
    let sample = targets(8192, 32);
    assert_eq!(&sample[..4], &[0, 8191, 1, 8190]);
    assert!(sample.iter().all(|i| *i < 8192));
}

#[test]
fn default_security_and_partial_reshape_rows_decode_correctly() {
    // Wide records need >6912 rows to make k=2. An odd count exercises
    // a partially populated final reshape row and the last-record sample.
    let result = run(7001, 3072, 1 << 30, 4).unwrap();
    assert_eq!(result["status"], "completed");
    assert_eq!(result["plan"]["reshape_k"], 2);
    assert_eq!(result["queries_completed"], 4);
    assert_eq!(result["security"]["lwe_dimension"], 1275);
    assert_eq!(result["security"]["plaintext_bits"], 8);
}
