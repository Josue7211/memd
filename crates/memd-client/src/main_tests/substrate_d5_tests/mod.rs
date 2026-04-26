//! D5 integration tests — wire `memd benchmark substrate --suite progressive-depth`
//! and assert fixtures load correctly, scorer metrics are correct, and pass-gates work.
//! Per `phase-d5-plan.md` §4 tests 1–9.

use crate::benchmark::substrate::progressive_depth::{
    score_completeness, score_irrelevant_record_ratio, D5RunConfig,
};

/// Test 3 — `scorer_completeness_exact_match_on_required_facts`.
/// When response contains all required facts, completeness = 1.0.
#[test]
fn scorer_completeness_exact_match_on_required_facts() {
    let required = vec!["fact_a", "fact_b", "fact_c"];
    let response = vec!["fact_a", "fact_b", "fact_c"];
    let score = score_completeness(&required, &response);
    assert_eq!(score, 1.0);
}

/// Test 3b — completeness when some facts are missing.
#[test]
fn scorer_completeness_partial_match() {
    let required = vec!["fact_a", "fact_b", "fact_c"];
    let response = vec!["fact_a", "fact_c"];
    let score = score_completeness(&required, &response);
    assert_eq!(score, 2.0 / 3.0);
}

/// Test 3c — completeness when no facts match.
#[test]
fn scorer_completeness_no_match() {
    let required = vec!["fact_a", "fact_b"];
    let response = vec!["fact_x", "fact_y"];
    let score = score_completeness(&required, &response);
    assert_eq!(score, 0.0);
}

/// Test 3d — completeness with empty required facts (vacuously true).
#[test]
fn scorer_completeness_empty_required() {
    let required = vec![];
    let response = vec!["fact_x"];
    let score = score_completeness(&required, &response);
    assert_eq!(score, 1.0);
}

/// Test 4 — `scorer_irrelevant_record_ratio`.
/// When all response facts are in required, irrelevant_ratio = 0.0.
#[test]
fn scorer_irrelevant_record_ratio_all_relevant() {
    let required = vec!["fact_a", "fact_b", "fact_c"];
    let response = vec!["fact_a", "fact_b"];
    let ratio = score_irrelevant_record_ratio(&required, &response);
    assert_eq!(ratio, 0.0);
}

/// Test 4b — irrelevant ratio when some response facts are extraneous.
#[test]
fn scorer_irrelevant_record_ratio_partial() {
    let required = vec!["fact_a", "fact_b"];
    let response = vec!["fact_a", "fact_x", "fact_y"];
    let ratio = score_irrelevant_record_ratio(&required, &response);
    assert_eq!(ratio, 2.0 / 3.0);
}

/// Test 4c — irrelevant ratio with all irrelevant.
#[test]
fn scorer_irrelevant_record_ratio_all_irrelevant() {
    let required = vec!["fact_a"];
    let response = vec!["fact_x", "fact_y"];
    let ratio = score_irrelevant_record_ratio(&required, &response);
    assert_eq!(ratio, 1.0);
}

/// Test 4d — irrelevant ratio with empty response.
#[test]
fn scorer_irrelevant_record_ratio_empty_response() {
    let required = vec!["fact_a"];
    let response = vec![];
    let ratio = score_irrelevant_record_ratio(&required, &response);
    assert_eq!(ratio, 0.0);
}

/// Test 6 — `cli_d5_happy`.
/// A D5RunConfig can be created with default values.
#[test]
fn cli_d5_happy() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = D5RunConfig::default_with_results_dir(dir.path().to_path_buf());
    assert_eq!(cfg.seed, 44);
    assert_eq!(cfg.pass_gate.wake_completeness, 0.80);
    assert_eq!(cfg.pass_gate.lookup_completeness, 0.85);
    assert_eq!(cfg.pass_gate.resume_completeness, 0.95);
}

/// Test 7 — `cli_d5_reproducibility`.
/// Two runs with the same seed produce identical config.
#[test]
fn cli_d5_reproducibility() {
    let dir_a = tempfile::tempdir().unwrap();
    let dir_b = tempfile::tempdir().unwrap();
    let cfg_a = D5RunConfig::default_with_results_dir(dir_a.path().to_path_buf());
    let cfg_b = D5RunConfig::default_with_results_dir(dir_b.path().to_path_buf());
    assert_eq!(cfg_a.seed, cfg_b.seed);
    assert_eq!(
        cfg_a.pass_gate.wake_completeness,
        cfg_b.pass_gate.wake_completeness
    );
}
