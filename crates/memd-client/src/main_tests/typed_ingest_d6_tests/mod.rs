//! V6 / D6 — bench-compiler tests.
//!
//! D6 lands scaffold-symmetric to A6/B6/C6: pure shim + fixture-driven
//! proxies for the bench-result locks. Real LME/MemBench/LoCoMo locks
//! graduate post-2026-05-02 with A6.9/B6/C6 runtime activation.
//!
//! Coverage map → `docs/phases/v6/phase-d6-plan.md` §4.

use crate::benchmark::typed_ingest::compiler::{load_budget_profile, BENCH_COMPILER_VERSION};

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
