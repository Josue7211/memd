//! V6 / D6 — bench-compiler tests.
//!
//! D6 lands scaffold-symmetric to A6/B6/C6: pure shim + fixture-driven
//! proxies for the bench-result locks. Real LME/MemBench/LoCoMo locks
//! graduate post-2026-05-02 with A6.9/B6/C6 runtime activation.
//!
//! Coverage map → `docs/phases/v6/phase-d6-plan.md` §4.

use crate::benchmark::typed_ingest::compiler::{
    compile_for_bench, load_budget_profile, BenchCompilerInput, BudgetProfile,
    BENCH_COMPILER_VERSION,
};

/// Test 1 — D6.1.
/// Loads the per-bench budget profile from the canonical config path
/// shipped in this commit. Each public bench (lme/locomo/membench/
/// convomem) must have a profile; the loader must reject unknown
/// schema majors.
#[test]
fn compiler_loads_budget_profile_per_bench() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(".memd/benchmarks/public/compiler-budgets.json");
    assert!(path.exists(), "missing budgets file at {}", path.display());

    for bench in &["lme", "locomo", "membench", "convomem"] {
        let prof = load_budget_profile(&path, bench)
            .unwrap_or_else(|e| panic!("load {bench}: {e}"));
        assert!(
            prof.budget_tokens >= 1000,
            "{bench} budget too low: {}",
            prof.budget_tokens
        );
        assert!(
            prof.priority.contains(&"canonical".to_string()),
            "{bench} priority must list canonical: {:?}",
            prof.priority
        );
    }

    // Schema major-version pin lives with the loader.
    assert_eq!(BENCH_COMPILER_VERSION, "bench-compiler/v1");
}

fn long_record(prefix: &str, n: usize) -> String {
    let body = "x".repeat(n);
    format!("{prefix}: {body}")
}

/// Test 2 — D6.2.
/// V4 D4's priority order (canonical > preference > focus > episodic >
/// semantic) is the authoritative tie-breaker — the shim must not
/// reorder. Under a tight budget, low-priority semantic/raw_episodic
/// content blows the class cap and gets demoted while canonical
/// survives via the per-bucket floor. Profile.priority is bench-side
/// documentation only — V4 owns the operational priority.
///
/// V4 explicitly admits the canonical floor and preference bucket
/// over the global cap (see runtime/resume/compiler/budget.rs §85-86),
/// so this test asserts demotion *occurred*, not that total tokens
/// stayed under the cap.
#[test]
fn compiler_respects_priority_order_on_overflow() {
    let profile = BudgetProfile {
        budget_tokens: 600,
        priority: vec![
            "canonical".into(),
            "preferences".into(),
            "recent_episodic".into(),
            "semantic".into(),
            "raw_episodic".into(),
        ],
    };
    let input = BenchCompilerInput {
        canonical: vec!["canonical-1".into()],
        preferences: vec!["preference-1".into()],
        recent_episodic: vec!["focus-1".into()],
        semantic: vec![long_record("semantic-1", 400)],
        raw_episodic: vec![long_record("episodic-1", 400)],
    };

    let outcome = compile_for_bench(input, &profile);

    assert!(
        outcome.markdown.contains("canonical-1"),
        "canonical must survive a tight budget; got:\n{}",
        outcome.markdown
    );
    assert!(
        !outcome.sections_dropped.is_empty(),
        "expected at least one demotion under a tight budget; included={:?} dropped={:?}",
        outcome.sections_included,
        outcome.sections_dropped
    );
    assert!(
        outcome
            .sections_dropped
            .iter()
            .any(|s| s == "semantic" || s == "raw_episodic"),
        "expected semantic or raw_episodic to be demoted under class cap; dropped={:?}",
        outcome.sections_dropped
    );
}

/// Test 3 — D6.2.
/// V4 D4's char-as-tokens convention: `outcome.tokens` must equal
/// `outcome.markdown.chars().count()` so per-question telemetry stays
/// consistent with `compute_wake_token_metrics`.
#[test]
fn compiler_uses_v4_token_counter() {
    let profile = BudgetProfile {
        budget_tokens: 4000,
        priority: vec!["canonical".into(), "preferences".into()],
    };
    let input = BenchCompilerInput {
        canonical: vec!["alpha".into(), "beta".into()],
        preferences: vec!["gamma".into()],
        ..Default::default()
    };
    let outcome = compile_for_bench(input, &profile);
    assert_eq!(
        outcome.tokens,
        outcome.markdown.chars().count(),
        "shim tokens must equal char count"
    );
}

/// Test 4 — D6.2.
/// Compiled markdown carries the typed section headers (Durable Truth
/// for canonical, Preferences for preferences, Episodic for focus, …).
/// V4 owns the rendering; the shim only verifies the typed-window
/// shape reaches the prompt.
#[test]
fn compiler_emits_typed_window_to_prompt() {
    let profile = BudgetProfile {
        budget_tokens: 4000,
        priority: vec!["canonical".into(), "preferences".into(), "semantic".into()],
    };
    let input = BenchCompilerInput {
        canonical: vec!["fact-A".into()],
        preferences: vec!["pref-A".into()],
        semantic: vec!["sem-A".into()],
        ..Default::default()
    };
    let outcome = compile_for_bench(input, &profile);

    for header in ["## Durable Truth", "## Preferences", "## Semantic"] {
        assert!(
            outcome.markdown.contains(header),
            "missing header {header} in:\n{}",
            outcome.markdown
        );
    }
    assert!(outcome.sections_included.contains(&"canonical".to_string()));
}
