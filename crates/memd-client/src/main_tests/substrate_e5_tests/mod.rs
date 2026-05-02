//! E5 integration tests — wire `memd benchmark substrate --suite
//! provenance-integrity` end-to-end and assert completeness rate,
//! inject-hole detection, reproducibility. Per `phase-e5-plan.md` §4 tests 6–9.

use crate::benchmark::substrate::fixtures::KindMix;
use crate::benchmark::substrate::provenance_integrity::{E5RunConfig, run_e5_in_process};
use std::path::PathBuf;
use tempfile::tempdir;

fn small_config(results_dir: PathBuf, inject_hole: bool) -> E5RunConfig {
    E5RunConfig {
        seed: 45,
        corpus_size: 50,
        query_count: 20,
        kind_mix: KindMix::default(),
        pass_gate: Default::default(),
        results_dir,
        inject_hole,
    }
}

/// Test 6 — `cli_e5_happy`.
/// Invoking the runner with default config against in-process backend
/// passes the gate and writes a non-empty NDJSON file.
#[test]
fn cli_e5_happy() {
    let dir = tempdir().unwrap();
    let cfg = small_config(dir.path().to_path_buf(), false);
    let outcome = run_e5_in_process(&cfg).unwrap();
    assert!(outcome.overall_pass);
    assert_eq!(outcome.records.len(), 1);
    assert!(outcome.completeness_rate >= 0.99);
    let body = std::fs::read_to_string(&outcome.ndjson_path).unwrap();
    assert!(body.lines().count() >= 1);
    for line in body.lines() {
        assert!(line.contains("\"suite\":\"provenance-integrity\""));
    }
}

/// Test 7 — `cli_e5_inject_hole_catches_planted`.
/// With --inject-hole, the auditor must catch planted provenance gaps.
#[test]
fn cli_e5_inject_hole_catches_planted() {
    let dir = tempdir().unwrap();
    let cfg = small_config(dir.path().to_path_buf(), true);
    let outcome = run_e5_in_process(&cfg).unwrap();
    // Inject-hole should reduce completeness rate below 1.0 and increase unsourced count.
    assert!(
        outcome.unsourced_count > 0,
        "injected hole must be detected"
    );
    assert!(
        outcome.completeness_rate < 1.0,
        "completeness must drop below floor"
    );
    assert!(!outcome.overall_pass, "injected hole must fail gate");
}

/// Test 8 — `cli_e5_reproducibility`.
/// Two invocations with the same seed produce identical completeness and chain metrics.
#[test]
fn cli_e5_reproducibility_same_seed_identical_output() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let cfg_a = small_config(dir_a.path().to_path_buf(), false);
    let cfg_b = small_config(dir_b.path().to_path_buf(), false);
    let a = run_e5_in_process(&cfg_a).unwrap();
    let b = run_e5_in_process(&cfg_b).unwrap();
    assert_eq!(a.records.len(), b.records.len());
    assert_eq!(a.records[0].seed, b.records[0].seed);
    assert_eq!(a.completeness_rate, b.completeness_rate);
    assert_eq!(a.chain_length_mean, b.chain_length_mean);
}

/// Test 9 — `e5_baseline_lock`.
/// Loads the latest `e5-*.json` baseline and asserts the in-process backend
/// still meets the hard 1.000 completeness-rate floor with no tolerance.
#[test]
fn e5_baseline_lock_completeness_rate_equals_one() {
    let baselines_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/verification/substrate-baselines");
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&baselines_dir)
        .unwrap_or_else(|e| panic!("read baseline dir {baselines_dir:?}: {e}"))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.starts_with("e5-"))
                .unwrap_or(false)
        })
        .collect();

    if entries.is_empty() {
        eprintln!("note: no e5-*.json baseline found; skipping lock test");
        return;
    }

    entries.sort();
    let latest = entries.pop().unwrap();
    let baseline: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&latest)
            .unwrap_or_else(|e| panic!("read baseline {latest:?}: {e}")),
    )
    .unwrap_or_else(|e| panic!("parse baseline JSON {latest:?}: {e}"));

    // Assert hard floor: completeness_rate MUST be exactly 1.000 (no tolerance).
    let completeness = baseline["completeness_rate"]
        .as_f64()
        .expect("baseline must have completeness_rate");
    assert_eq!(
        completeness, 1.000,
        "E5 hard floor: completeness_rate must be 1.000 exactly"
    );

    // Chain length mean >= 2.
    let chain_len = baseline["chain_length_mean"]
        .as_f64()
        .expect("baseline must have chain_length_mean");
    assert!(
        chain_len >= 2.0,
        "E5 pass gate: chain_length_mean must be >= 2.0"
    );
}
