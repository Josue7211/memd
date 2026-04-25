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

/// C5 Test 10 — `c5_baseline_lock`.
/// Loads the latest `c5-*.json` and asserts the in-memory gateway
/// still meets every scenario's truth-conservation floor and the hard
/// zero-leak floor. Auto-picks newest filename — drop a fresh
/// `c5-YYYY-MM-DD.json` when the HTTP-backed gateway relands the floor.
#[test]
fn c5_baseline_current_memd_canonical_numbers() {
    let baselines_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/verification/substrate-baselines");
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&baselines_dir)
        .unwrap_or_else(|e| panic!("read baseline dir {baselines_dir:?}: {e}"))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|n| n.starts_with("c5-") && n.ends_with(".json"))
        })
        .collect();
    entries.sort();
    let latest = entries.last().expect("at least one c5-*.json baseline");

    #[derive(serde::Deserialize)]
    struct BaselineScenario {
        pair_index: usize,
        scenario: String,
        truth_conservation_rate: f64,
        visibility_leak_count: u64,
    }
    #[derive(serde::Deserialize)]
    struct Baseline {
        tolerance: f64,
        scenarios_floor: Vec<BaselineScenario>,
    }

    let baseline: Baseline = serde_json::from_slice(&std::fs::read(latest).unwrap())
        .unwrap_or_else(|e| panic!("parse {latest:?}: {e}"));

    let dir = tempdir().unwrap();
    let (claude, codex) = ready_adapters(dir.path());
    let gateway = InMemoryGateway::new();
    let cfg = C5RunConfig::default_with_results_dir(dir.path().to_path_buf());
    let adapters: Vec<(&str, &dyn HarnessAdapter)> =
        vec![("claude_code", &claude), ("codex", &codex)];
    let outcome = run_c5_with_adapters(&cfg, &adapters, &gateway).unwrap();

    for floor in &baseline.scenarios_floor {
        let actual = outcome
            .records
            .iter()
            .find(|r| r.cut_k == floor.pair_index && r.suite.contains(&floor.scenario))
            .unwrap_or_else(|| {
                panic!(
                    "no record for pair_index={} scenario={}",
                    floor.pair_index, floor.scenario
                )
            });
        assert!(
            actual.recall_at_1 + baseline.tolerance >= floor.truth_conservation_rate,
            "regression: pair={} scenario={} truth {:.3} < floor {:.3} (tol {:.3})",
            floor.pair_index,
            floor.scenario,
            actual.recall_at_1,
            floor.truth_conservation_rate,
            baseline.tolerance,
        );
        // Hard floor — no tolerance.
        assert_eq!(
            actual.recall_at_3 as u64, floor.visibility_leak_count,
            "regression: pair={} scenario={} leaks {} != floor {} (hard 0)",
            floor.pair_index, floor.scenario, actual.recall_at_3, floor.visibility_leak_count,
        );
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
