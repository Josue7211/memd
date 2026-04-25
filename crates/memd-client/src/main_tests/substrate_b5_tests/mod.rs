//! B5 integration tests — wire `memd benchmark substrate --suite
//! correction-propagation` end-to-end and assert reproducibility,
//! pass-gate exit semantics, output dir layout. Per `phase-b5-plan.md`
//! §4 tests 5–8.

use crate::benchmark::substrate::correction_propagation::{
    run_b5_in_process, run_b5_with_backend, B5RunConfig, DegradedB5Backend,
};
use crate::benchmark::substrate::fixtures::KindMix;
use std::path::PathBuf;
use tempfile::tempdir;

fn small_config(results_dir: PathBuf) -> B5RunConfig {
    let mut cfg = B5RunConfig::default_with_results_dir(results_dir);
    cfg.fact_count = 10;
    cfg.kind_mix = KindMix::default();
    cfg
}

/// B5 Test 5 — `cli_b5_happy_path`.
/// Default invocation against the in-process perfect-recall backend
/// passes the gate and writes a non-empty NDJSON.
#[test]
fn cli_b5_happy_path() {
    let dir = tempdir().unwrap();
    let cfg = small_config(dir.path().to_path_buf());
    let outcome = run_b5_in_process(&cfg).unwrap();
    assert!(outcome.overall_pass);
    let body = std::fs::read_to_string(&outcome.ndjson_path).unwrap();
    assert!(body.lines().count() >= 1);
    for line in body.lines() {
        assert!(line.contains("\"suite\":\"correction-propagation\""));
    }
}

/// B5 Test 6 — `cli_b5_fails_when_propagation_under_floor`.
/// Degraded backend (no recall, no provenance) misses every gate.
#[test]
fn cli_b5_fails_when_propagation_under_floor() {
    let dir = tempdir().unwrap();
    let cfg = small_config(dir.path().to_path_buf());
    let outcome = run_b5_with_backend(&cfg, &DegradedB5Backend).unwrap();
    assert!(!outcome.overall_pass);
    for r in &outcome.records {
        assert!(!r.pass, "degraded backend must fail every scenario");
        assert!((r.recall_at_1 - 0.0).abs() < f64::EPSILON);
    }
}

/// B5 Test 7 — `cli_b5_reproducibility_same_seed_identical_output`.
/// Two invocations with the same seed produce identical metric fields
/// across all records (run_id + ts_ms vary).
#[test]
fn cli_b5_reproducibility_same_seed_identical_output() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let cfg_a = small_config(dir_a.path().to_path_buf());
    let cfg_b = small_config(dir_b.path().to_path_buf());
    let a = run_b5_in_process(&cfg_a).unwrap();
    let b = run_b5_in_process(&cfg_b).unwrap();
    assert_eq!(a.records.len(), b.records.len());
    for (ra, rb) in a.records.iter().zip(b.records.iter()) {
        assert_eq!(ra.seed, rb.seed);
        assert_eq!(ra.fact_count, rb.fact_count);
        assert_eq!(ra.cut_k, rb.cut_k);
        assert_eq!(ra.recall_at_1, rb.recall_at_1);
        assert_eq!(ra.recall_at_3, rb.recall_at_3);
        assert_eq!(ra.answer_exact_match, rb.answer_exact_match);
        assert_eq!(ra.pass, rb.pass);
    }
}

/// Output dir layout: per-suite NDJSON + aggregate runs.jsonl.
#[test]
fn cli_b5_writes_results_dir_tree() {
    let dir = tempdir().unwrap();
    let cfg = small_config(dir.path().to_path_buf());
    let outcome = run_b5_in_process(&cfg).unwrap();
    assert!(outcome.ndjson_path.exists());
    let runs_jsonl = dir.path().join("runs.jsonl");
    assert!(runs_jsonl.exists());
    let runs_body = std::fs::read_to_string(&runs_jsonl).unwrap();
    assert!(runs_body.contains("correction-propagation"));
}
