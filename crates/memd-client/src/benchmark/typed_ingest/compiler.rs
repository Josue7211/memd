//! V6 / D6 — bench-compiler shim.
//!
//! Wraps V4 D4's `runtime::resume::compiler::compile_wake` for use on
//! public-benchmark answer prompts. Pure: no IO, no network. The
//! runtime layer (D6 dispatch, gated by V5 calendar gate alongside
//! A6.9/B6/C6) wraps these helpers to load the budget profile, build
//! a `CompilerInput` from typed-ingest records, and append telemetry
//! NDJSON.
//!
//! Contract: `docs/contracts/bench-compiler.md`.
//! Plan: `docs/phases/v6/phase-d6-plan.md`.

use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

/// Schema version pin. Bumping the major invalidates prior budget files.
pub(crate) const BENCH_COMPILER_VERSION: &str = "bench-compiler/v1";

/// Per-bench budget profile loaded from
/// `.memd/benchmarks/public/compiler-budgets.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct BudgetProfile {
    pub budget_tokens: usize,
    pub priority: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BudgetTable {
    pub version: String,
    pub benches: std::collections::BTreeMap<String, BudgetProfile>,
}

/// Load the per-bench budget profile by id (`lme`, `locomo`, `membench`,
/// `convomem`, ...). Errors when the bench id is not in the table or
/// the schema major-version doesn't match.
pub(crate) fn load_budget_profile(path: &Path, bench_id: &str) -> Result<BudgetProfile> {
    let body = std::fs::read_to_string(path)
        .with_context(|| format!("read bench-compiler budgets {}", path.display()))?;
    let table: BudgetTable = serde_json::from_str(&body)
        .with_context(|| format!("parse bench-compiler budgets {}", path.display()))?;

    let major = major_version(&table.version);
    let expected_major = major_version(BENCH_COMPILER_VERSION);
    if major != expected_major {
        return Err(anyhow!(
            "bench-compiler budgets schema major mismatch: file={} expected={}",
            table.version,
            BENCH_COMPILER_VERSION
        ));
    }

    table.benches.get(bench_id).cloned().ok_or_else(|| {
        anyhow!(
            "bench-compiler budgets has no profile for bench id {:?} (known: {:?})",
            bench_id,
            table.benches.keys().cloned().collect::<Vec<_>>()
        )
    })
}

fn major_version(v: &str) -> &str {
    v.rsplit('/').next().unwrap_or(v).trim_start_matches('v')
}

/// Default repo-relative path for the budget table. Kept here so tests
/// and the runtime agree on one source of truth.
pub(crate) fn default_budgets_path() -> &'static str {
    ".memd/benchmarks/public/compiler-budgets.json"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_table(dir: &Path, body: &str) -> std::path::PathBuf {
        let p = dir.join("compiler-budgets.json");
        std::fs::write(&p, body).unwrap();
        p
    }

    #[test]
    fn loads_profile_for_known_bench() {
        let tmp = tempfile::tempdir().unwrap();
        let body = serde_json::json!({
            "version": "bench-compiler/v1",
            "benches": {
                "lme": { "budget_tokens": 2000, "priority": ["canonical", "preferences"] }
            }
        });
        let path = write_table(tmp.path(), &body.to_string());
        let prof = load_budget_profile(&path, "lme").unwrap();
        assert_eq!(prof.budget_tokens, 2000);
        assert_eq!(prof.priority, vec!["canonical", "preferences"]);
    }

    #[test]
    fn rejects_unknown_major_version() {
        let tmp = tempfile::tempdir().unwrap();
        let body = serde_json::json!({
            "version": "bench-compiler/v2",
            "benches": { "lme": { "budget_tokens": 1, "priority": [] } }
        });
        let path = write_table(tmp.path(), &body.to_string());
        let err = load_budget_profile(&path, "lme").unwrap_err();
        assert!(
            err.to_string().contains("schema major mismatch"),
            "got: {err}"
        );
    }

    #[test]
    fn rejects_unknown_bench_id() {
        let tmp = tempfile::tempdir().unwrap();
        let body = serde_json::json!({
            "version": "bench-compiler/v1",
            "benches": { "lme": { "budget_tokens": 1, "priority": [] } }
        });
        let path = write_table(tmp.path(), &body.to_string());
        let err = load_budget_profile(&path, "nope").unwrap_err();
        assert!(err.to_string().contains("no profile"), "got: {err}");
    }
}
