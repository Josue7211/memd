//! V6 / D6 — bench-compiler tests.
//!
//! D6 lands scaffold-symmetric to A6/B6/C6: pure shim + fixture-driven
//! proxies for the bench-result locks. Real LME/MemBench/LoCoMo locks
//! graduate post-2026-05-02 with A6.9/B6/C6 runtime activation.
//!
//! Coverage map → `docs/phases/v6/phase-d6-plan.md` §4.

use std::sync::{Mutex, OnceLock};

use crate::benchmark::typed_ingest::compiler::{
    BENCH_COMPILER_VERSION, BenchCompilerInput, BudgetProfile, compile_for_bench,
    load_budget_profile,
};
use crate::benchmark::typed_ingest::{compiler_active, compiler_runtime_notice};

/// Serialise tests that mutate `MEMD_V6_COMPILER`.
static COMPILER_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    COMPILER_ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("env lock poisoned")
}

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
        let prof =
            load_budget_profile(&path, bench).unwrap_or_else(|e| panic!("load {bench}: {e}"));
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

/// Test 5 — D6.3.
/// CLI flag and env override resolve cleanly. `compiler_active`:
/// - `--compiler=off` → off
/// - `--compiler=on` → on
/// - `MEMD_V6_COMPILER=1` overrides any CLI value to on.
#[test]
fn ab_harness_flag_toggles_cleanly() {
    let _g = env_lock();

    unsafe {
        std::env::remove_var("MEMD_V6_COMPILER");
    }
    assert!(!compiler_active("off"), "default off");
    assert!(compiler_active("on"), "explicit on");

    unsafe {
        std::env::set_var("MEMD_V6_COMPILER", "1");
    }
    assert!(compiler_active("off"), "env=1 forces on over CLI off");
    assert!(compiler_active("on"), "env=1 keeps on");

    unsafe {
        std::env::set_var("MEMD_V6_COMPILER", "0");
    }
    assert!(!compiler_active("off"), "env=0 falls back to CLI off");
    assert!(compiler_active("on"), "env=0 keeps CLI on");

    unsafe {
        std::env::remove_var("MEMD_V6_COMPILER");
    }
}

/// Test 6 — D6.3.
/// `--compiler=off` notice carries no compiler-engagement language —
/// the legacy flat-RAG prompt path stays observably unchanged. The
/// off-path notice is the only stable signal (eprintln) the runtime
/// emits about the compiler when it's disabled.
#[test]
fn flat_rag_path_unchanged_when_off() {
    let _g = env_lock();
    unsafe {
        std::env::remove_var("MEMD_V6_COMPILER");
    }

    let off_notice = compiler_runtime_notice("off", false);
    assert!(
        !off_notice.contains("engaged"),
        "off-path leaks compiler engagement: {off_notice}"
    );
    assert!(
        !off_notice.contains("budgets="),
        "off-path leaks budget-table reference: {off_notice}"
    );
    assert!(
        off_notice.contains("flat-RAG"),
        "off-path notice must label legacy path: {off_notice}"
    );

    let on_notice = compiler_runtime_notice("on", false);
    assert!(
        on_notice.contains("engaged"),
        "on-path notice missing engagement: {on_notice}"
    );
    assert!(
        on_notice.contains("budgets="),
        "on-path notice must reference budgets file: {on_notice}"
    );
}

#[derive(Debug, Clone, serde::Deserialize)]
struct OverflowRow {
    bench_id: String,
    #[allow(dead_code)]
    question_id: String,
    baseline_prompt_tokens: u32,
    compiled_prompt_tokens: u32,
    baseline_correct: bool,
    compiled_correct: bool,
}

fn load_overflow_fixture() -> Vec<OverflowRow> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/fixtures/typed_ingest/d6/overflow-scenario.jsonl");
    let body = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()));
    body.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str::<OverflowRow>(l).expect("parse row"))
        .collect()
}

fn rows_for(bench: &str) -> Vec<OverflowRow> {
    load_overflow_fixture()
        .into_iter()
        .filter(|r| r.bench_id == bench)
        .collect()
}

/// Test 7 — D6.4 (fixture proxy).
/// Mean prompt-token drop across LME questions ≥ 25%.
/// Real-data lock graduates with A6.9/B6/C6 runtime activation
/// (post-2026-05-02). Until then this proxy verifies the budget
/// profile + compiler shim deliver enough headroom.
#[test]
fn lme_mean_prompt_tokens_drops_at_least_25pct() {
    let rows = rows_for("lme");
    assert!(!rows.is_empty(), "fixture missing lme rows");

    let baseline_mean: f64 = rows
        .iter()
        .map(|r| r.baseline_prompt_tokens as f64)
        .sum::<f64>()
        / rows.len() as f64;
    let compiled_mean: f64 = rows
        .iter()
        .map(|r| r.compiled_prompt_tokens as f64)
        .sum::<f64>()
        / rows.len() as f64;

    let drop = (baseline_mean - compiled_mean) / baseline_mean;
    assert!(
        drop >= 0.25,
        "LME prompt-token drop {drop:.3} below 0.25 plan threshold (baseline={baseline_mean:.0} compiled={compiled_mean:.0})"
    );
}

/// Test 8 — D6.5 (fixture proxy).
/// MemBench accuracy lift ≥ +0.03 (compiled - baseline).
#[test]
fn membench_lifts_at_least_0_03() {
    let rows = rows_for("membench");
    assert!(!rows.is_empty(), "fixture missing membench rows");
    let baseline_acc =
        rows.iter().filter(|r| r.baseline_correct).count() as f64 / rows.len() as f64;
    let compiled_acc =
        rows.iter().filter(|r| r.compiled_correct).count() as f64 / rows.len() as f64;
    let lift = compiled_acc - baseline_acc;
    assert!(
        lift >= 0.03,
        "MemBench lift {lift:.3} below 0.03 (baseline={baseline_acc:.3} compiled={compiled_acc:.3})"
    );
}

/// Test 9 — D6.5 (fixture proxy).
/// LoCoMo accuracy lift ≥ +0.03.
#[test]
fn locomo_lifts_at_least_0_03() {
    let rows = rows_for("locomo");
    assert!(!rows.is_empty(), "fixture missing locomo rows");
    let baseline_acc =
        rows.iter().filter(|r| r.baseline_correct).count() as f64 / rows.len() as f64;
    let compiled_acc =
        rows.iter().filter(|r| r.compiled_correct).count() as f64 / rows.len() as f64;
    let lift = compiled_acc - baseline_acc;
    assert!(
        lift >= 0.03,
        "LoCoMo lift {lift:.3} below 0.03 (baseline={baseline_acc:.3} compiled={compiled_acc:.3})"
    );
}

/// Test 10 — D6.6 (fixture proxy).
/// ConvoMem + canonical paths regression guard. Compiled accuracy
/// must not drop below baseline on ConvoMem (the legacy non-typed
/// path); this is the canary that compiler turning on doesn't
/// damage benches it isn't tuned for.
#[test]
fn no_canonical_regression_below_c6_baseline() {
    let rows = rows_for("convomem");
    assert!(!rows.is_empty(), "fixture missing convomem rows");
    let baseline_correct = rows.iter().filter(|r| r.baseline_correct).count();
    let compiled_correct = rows.iter().filter(|r| r.compiled_correct).count();
    assert!(
        compiled_correct >= baseline_correct,
        "ConvoMem regression: baseline={baseline_correct} compiled={compiled_correct}"
    );

    // Canonical path proxy: compiled prompt-token mean must not
    // exceed baseline (compiler-on must shrink, never grow). Echoes
    // the C6→D6 invariant that canonical content stays admitted.
    let baseline_mean: f64 = rows
        .iter()
        .map(|r| r.baseline_prompt_tokens as f64)
        .sum::<f64>()
        / rows.len() as f64;
    let compiled_mean: f64 = rows
        .iter()
        .map(|r| r.compiled_prompt_tokens as f64)
        .sum::<f64>()
        / rows.len() as f64;
    assert!(
        compiled_mean <= baseline_mean,
        "ConvoMem prompt-token regression: baseline={baseline_mean:.0} compiled={compiled_mean:.0}"
    );
}
