//! C5 integration tests — `memd benchmark substrate --suite cross-harness`
//! end-to-end. Mirrors `substrate_b5_tests/mod.rs` shape: cli happy
//! path, graceful skip, reproducibility, dir-tree, baseline lock.
//! Per `phase-c5-plan.md` §4 tests 7-10.

use crate::benchmark::substrate::cross_harness::{
    run_c5_with_adapters, run_c5_with_skip, C5RunConfig, C5Scenario,
};
use crate::benchmark::substrate::harness_adapter::{
    claude_code::ClaudeCodeAdapter, codex::CodexAdapter, HarnessAdapter, InMemoryGateway,
};
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn ready_adapters(dir: &Path) -> (ClaudeCodeAdapter, CodexAdapter) {
    let claude_settings = dir.join("settings.json");
    let codex_hooks = dir.join("hooks.json");
    std::fs::write(&claude_settings, "{}").unwrap();
    std::fs::write(&codex_hooks, "{}").unwrap();
    (
        ClaudeCodeAdapter::with_config_path(claude_settings),
        CodexAdapter::with_config_path(codex_hooks),
    )
}

fn small_config(results_dir: PathBuf) -> C5RunConfig {
    let mut cfg = C5RunConfig::default_with_results_dir(results_dir);
    cfg.per_scenario_facts = 4;
    cfg.scenarios = vec![C5Scenario::FactRoundtrip, C5Scenario::PreferenceRoundtrip];
    cfg
}

/// C5 Test 7 — `cli_c5_gracefully_skips_unavailable_harness`.
/// When codex's hooks.json is missing AND skip is allowed, every pair
/// touching codex is dropped. Default pairs are claude<->codex, so all
/// pairs drop, yielding an empty-but-passing run with a runs.jsonl
/// skip note for CI surfaces.
#[test]
fn cli_c5_gracefully_skips_unavailable_harness() {
    let dir = tempdir().unwrap();
    let claude_settings = dir.path().join("settings.json");
    std::fs::write(&claude_settings, "{}").unwrap();
    let claude = ClaudeCodeAdapter::with_config_path(claude_settings);
    // Codex config does NOT exist on disk.
    let codex = CodexAdapter::with_config_path(dir.path().join("missing-hooks.json"));

    let cfg = small_config(dir.path().to_path_buf());
    let gateway = InMemoryGateway::new();
    let adapters: Vec<(&str, &dyn HarnessAdapter)> =
        vec![("claude_code", &claude), ("codex", &codex)];

    let outcome = run_c5_with_skip(&cfg, &adapters, &gateway, true).unwrap();
    assert!(outcome.overall_pass);
    assert!(outcome.records.is_empty(), "all pairs touch codex, must skip all");

    let runs = std::fs::read_to_string(dir.path().join("runs.jsonl")).unwrap();
    assert!(runs.contains("\"skipped\":true"));
    assert!(runs.contains("\"availability\""));
}

/// Skip-disabled mode must surface the missing harness as an error
/// instead of silently producing zero records — guards against CI
/// configurations that forget to opt out of skip.
#[test]
fn cli_c5_errors_when_skip_disabled_and_harness_missing() {
    let dir = tempdir().unwrap();
    let claude_settings = dir.path().join("settings.json");
    std::fs::write(&claude_settings, "{}").unwrap();
    let claude = ClaudeCodeAdapter::with_config_path(claude_settings);
    let codex = CodexAdapter::with_config_path(dir.path().join("missing-hooks.json"));

    let cfg = small_config(dir.path().to_path_buf());
    let gateway = InMemoryGateway::new();
    let adapters: Vec<(&str, &dyn HarnessAdapter)> =
        vec![("claude_code", &claude), ("codex", &codex)];

    let err = run_c5_with_skip(&cfg, &adapters, &gateway, false).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("required harnesses unavailable"), "unexpected error: {msg}");
    assert!(msg.contains("codex"));
}

/// C5 Test 8 — `cli_c5_happy_both_pairs`.
/// Both harnesses available; full pair matrix runs; every scenario
/// passes; zero leaks.
#[test]
fn cli_c5_happy_both_pairs() {
    let dir = tempdir().unwrap();
    let (claude, codex) = ready_adapters(dir.path());
    let gateway = InMemoryGateway::new();
    let cfg = small_config(dir.path().to_path_buf());

    let adapters: Vec<(&str, &dyn HarnessAdapter)> =
        vec![("claude_code", &claude), ("codex", &codex)];
    let outcome = run_c5_with_adapters(&cfg, &adapters, &gateway).unwrap();

    assert_eq!(outcome.records.len(), cfg.pairs.len() * cfg.scenarios.len());
    assert!(outcome.overall_pass);
    assert_eq!(outcome.leaks.len(), 0);
    let runs = std::fs::read_to_string(dir.path().join("runs.jsonl")).unwrap();
    assert!(runs.contains("\"suite\":\"cross-harness\""));
}

/// C5 Test 9 — `cli_c5_reproducibility_same_seed`.
/// Same seed twice → identical determinable metrics. Latency varies,
/// so it's excluded.
#[test]
fn cli_c5_reproducibility_same_seed() {
    let dir_a = tempdir().unwrap();
    let dir_b = tempdir().unwrap();
    let (claude_a, codex_a) = ready_adapters(dir_a.path());
    let (claude_b, codex_b) = ready_adapters(dir_b.path());

    let cfg_a = small_config(dir_a.path().to_path_buf());
    let cfg_b = small_config(dir_b.path().to_path_buf());

    let gw_a = InMemoryGateway::new();
    let gw_b = InMemoryGateway::new();
    let ad_a: Vec<(&str, &dyn HarnessAdapter)> =
        vec![("claude_code", &claude_a), ("codex", &codex_a)];
    let ad_b: Vec<(&str, &dyn HarnessAdapter)> =
        vec![("claude_code", &claude_b), ("codex", &codex_b)];

    let a = run_c5_with_adapters(&cfg_a, &ad_a, &gw_a).unwrap();
    let b = run_c5_with_adapters(&cfg_b, &ad_b, &gw_b).unwrap();

    assert_eq!(a.records.len(), b.records.len());
    for (ra, rb) in a.records.iter().zip(b.records.iter()) {
        assert_eq!(ra.suite, rb.suite);
        assert_eq!(ra.seed, rb.seed);
        assert_eq!(ra.fact_count, rb.fact_count);
        assert_eq!(ra.cut_k, rb.cut_k);
        assert_eq!(ra.recall_at_1, rb.recall_at_1, "truth {} drifted", ra.suite);
        assert_eq!(ra.recall_at_3, rb.recall_at_3, "leaks {} drifted", ra.suite);
        assert_eq!(ra.pass, rb.pass);
    }
}

/// Output dir layout — NDJSON per suite + runs.jsonl aggregator.
#[test]
fn cli_c5_writes_results_dir_tree() {
    let dir = tempdir().unwrap();
    let (claude, codex) = ready_adapters(dir.path());
    let gateway = InMemoryGateway::new();
    let cfg = small_config(dir.path().to_path_buf());
    let adapters: Vec<(&str, &dyn HarnessAdapter)> =
        vec![("claude_code", &claude), ("codex", &codex)];

    let outcome = run_c5_with_adapters(&cfg, &adapters, &gateway).unwrap();
    assert!(outcome.ndjson_path.exists());
    let runs_jsonl = dir.path().join("runs.jsonl");
    assert!(runs_jsonl.exists());
    let runs_body = std::fs::read_to_string(&runs_jsonl).unwrap();
    assert!(runs_body.contains("cross-harness"));
}
