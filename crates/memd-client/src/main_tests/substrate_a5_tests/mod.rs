//! A5 integration tests — wire `memd benchmark substrate` end-to-end
//! and assert reproducibility, output dir layout, and pass-gate exit
//! semantics. Per `phase-a5-plan.md` §4 tests 10–14.

use crate::benchmark::substrate::cross_session_recall::{
    A5RunConfig, DegradedBackend, PassGate, run_a5_in_process, run_a5_with_backend,
};
use crate::benchmark::substrate::fixtures::KindMix;
use std::path::PathBuf;
use tempfile::tempdir;

fn small_config(results_dir: PathBuf) -> A5RunConfig {
    A5RunConfig {
        seed: 42,
        fact_counts: vec![20],
        cuts: vec![2],
        kind_mix: KindMix::default(),
        pass_gate: PassGate::default(),
        results_dir,
    }
}

/// Test 10 — `cli_bench_substrate_cross_session_recall_happy`.
/// Invoking the runner with N=20, K=2 against the in-process backend
/// returns pass=true and writes a non-empty NDJSON file.
#[test]
fn cli_bench_substrate_cross_session_recall_happy() {
    let dir = tempdir().unwrap();
    let cfg = small_config(dir.path().to_path_buf());
    let outcome = run_a5_in_process(&cfg).unwrap();
    assert!(outcome.overall_pass);
    let body = std::fs::read_to_string(&outcome.ndjson_path).unwrap();
    assert!(body.lines().count() >= 1);
}

/// Test 11 — `cli_bench_substrate_honors_seed_reproducibility`.
/// Two invocations with the same seed produce identical recall numbers
/// (per-record run_id + ts_ms vary, but all metric fields match).
#[test]
fn cli_bench_substrate_honors_seed_reproducibility() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let cfg_a = small_config(dir_a.path().to_path_buf());
    let cfg_b = small_config(dir_b.path().to_path_buf());
    let a = run_a5_in_process(&cfg_a).unwrap();
    let b = run_a5_in_process(&cfg_b).unwrap();
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

/// Test 12 — `cli_bench_substrate_fails_when_pass_gate_missed`.
/// The DegradedBackend (always returns None) must trigger overall_pass=false
/// and per-record pass=false. The dispatcher's `std::process::exit(1)`
/// path is exercised by the third-party reproduce script (test 14).
#[test]
fn cli_bench_substrate_fails_when_pass_gate_missed() {
    let dir = tempdir().unwrap();
    let cfg = small_config(dir.path().to_path_buf());
    let outcome = run_a5_with_backend(&cfg, &DegradedBackend).unwrap();
    assert!(!outcome.overall_pass);
    for r in &outcome.records {
        assert!(!r.pass, "degraded backend must fail every scenario");
        assert_eq!(r.recall_at_1, 0.0);
    }
}

/// Test 13 — `cli_bench_substrate_writes_results_dir_tree`.
/// After a run, the results dir must contain both the per-suite NDJSON
/// file and the aggregate `runs.jsonl`.
#[test]
fn cli_bench_substrate_writes_results_dir_tree() {
    let dir = tempdir().unwrap();
    let cfg = small_config(dir.path().to_path_buf());
    let outcome = run_a5_in_process(&cfg).unwrap();
    assert!(outcome.ndjson_path.exists());
    let runs_jsonl = dir.path().join("runs.jsonl");
    assert!(runs_jsonl.exists(), "runs.jsonl missing");
    let runs_body = std::fs::read_to_string(&runs_jsonl).unwrap();
    assert!(runs_body.contains("cross-session-recall"));
}

/// Test 14 — `cli_bench_substrate_third_party_reproduce_script`.
/// Verifies the existence + executable bit + key invocation shape of
/// `scripts/substrate-bench-reproduce.sh`. Actually invoking it would
/// require building the release binary, which is out of scope for unit
/// tests; CI nightly (A5.8) runs it for real.
#[test]
fn cli_bench_substrate_third_party_reproduce_script() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/substrate-bench-reproduce.sh")
        .canonicalize()
        .expect("substrate-bench-reproduce.sh missing");
    assert!(script.is_file());
    let body = std::fs::read_to_string(&script).expect("read script");
    assert!(body.contains("benchmark substrate"));
    assert!(body.contains("--suite cross-session-recall"));
    assert!(body.contains("--seed"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::metadata(&script).unwrap().permissions();
        assert!(
            perms.mode() & 0o111 != 0,
            "substrate-bench-reproduce.sh must be executable"
        );
    }
}

/// Test 15 — `a5_baseline_current_memd_canonical_numbers`.
/// Loads the most-recent locked baseline JSON and asserts the
/// in-process backend still meets every scenario floor (within
/// `tolerance`). When the HTTP backend lands and the floor changes,
/// drop a fresh `a5-YYYY-MM-DD.json` and the test picks it up
/// automatically.
#[test]
fn a5_baseline_current_memd_canonical_numbers() {
    let baselines_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/verification/substrate-baselines");
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&baselines_dir)
        .unwrap_or_else(|e| panic!("read baseline dir {baselines_dir:?}: {e}"))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|n| n.starts_with("a5-") && n.ends_with(".json"))
        })
        .collect();
    entries.sort();
    let latest = entries.last().expect("at least one a5-*.json baseline");

    #[derive(serde::Deserialize)]
    struct BaselineScenario {
        fact_count: usize,
        cut_k: usize,
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
    let cfg = A5RunConfig::default_with_results_dir(dir.path().to_path_buf());
    let outcome = run_a5_in_process(&cfg).unwrap();

    for floor in &baseline.scenarios {
        let actual = outcome
            .records
            .iter()
            .find(|r| r.fact_count == floor.fact_count && r.cut_k == floor.cut_k)
            .unwrap_or_else(|| {
                panic!(
                    "no record for fact_count={} cut_k={}",
                    floor.fact_count, floor.cut_k
                )
            });
        assert!(
            actual.recall_at_3 + baseline.tolerance >= floor.recall_at_3,
            "regression: scenario (n={}, k={}) recall_at_3 {:.3} < floor {:.3} (tol {:.3})",
            floor.fact_count,
            floor.cut_k,
            actual.recall_at_3,
            floor.recall_at_3,
            baseline.tolerance,
        );
    }
}
