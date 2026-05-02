//! B5 integration tests — wire `memd benchmark substrate --suite
//! correction-propagation` end-to-end and assert reproducibility,
//! pass-gate exit semantics, output dir layout. Per `phase-b5-plan.md`
//! §4 tests 5–8.

use crate::benchmark::substrate::correction_propagation::{
    B5RunConfig, DegradedB5Backend, run_b5_in_process, run_b5_with_backend,
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

/// B5 Test 8 — `b5_baseline_lock`.
/// Loads the latest `b5-*.json` and asserts the in-process backend
/// still meets every scenario floor within tolerance. When the HTTP
/// backend lands and floor changes, drop a fresh `b5-YYYY-MM-DD.json`
/// and the test auto-picks the latest by filename sort.
#[test]
fn b5_baseline_current_memd_canonical_numbers() {
    let baselines_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/verification/substrate-baselines");
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&baselines_dir)
        .unwrap_or_else(|e| panic!("read baseline dir {baselines_dir:?}: {e}"))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|n| n.starts_with("b5-") && n.ends_with(".json"))
        })
        .collect();
    entries.sort();
    let latest = entries.last().expect("at least one b5-*.json baseline");

    #[derive(serde::Deserialize)]
    struct BaselineScenario {
        query_session: usize,
        recall_at_1: f64,
        recall_at_3: f64,
    }
    #[derive(serde::Deserialize)]
    struct Baseline {
        tolerance: f64,
        scenarios: Vec<BaselineScenario>,
    }

    let baseline: Baseline = serde_json::from_slice(&std::fs::read(latest).unwrap())
        .unwrap_or_else(|e| panic!("parse {latest:?}: {e}"));

    let dir = tempdir().unwrap();
    let cfg = B5RunConfig::default_with_results_dir(dir.path().to_path_buf());
    let outcome = run_b5_in_process(&cfg).unwrap();

    for floor in &baseline.scenarios {
        let actual = outcome
            .records
            .iter()
            .find(|r| r.cut_k == floor.query_session)
            .unwrap_or_else(|| panic!("no record for query_session={}", floor.query_session));
        assert!(
            actual.recall_at_1 + baseline.tolerance >= floor.recall_at_1,
            "regression: query_session={} propagation_rate {:.3} < floor {:.3} (tol {:.3})",
            floor.query_session,
            actual.recall_at_1,
            floor.recall_at_1,
            baseline.tolerance,
        );
        assert!(
            actual.recall_at_3 + baseline.tolerance >= floor.recall_at_3,
            "regression: query_session={} provenance_correctness {:.3} < floor {:.3} (tol {:.3})",
            floor.query_session,
            actual.recall_at_3,
            floor.recall_at_3,
            baseline.tolerance,
        );
    }
}
