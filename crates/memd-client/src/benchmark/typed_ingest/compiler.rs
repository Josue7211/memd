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

use anyhow::{Context, Result, anyhow};
use memd_schema::CompactMemoryRecord;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::runtime::resume::compiler::{
    BucketKind, CompiledWake, CompilerInput, WakeBudget, compile_wake,
};

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

/// Bench-shim input. Each field is a list of free-form strings —
/// callers project their typed-ingest records into the bucket vocab
/// before calling `compile_for_bench`. Order within each bucket is
/// preserved verbatim; V4's priority order across buckets is fixed
/// (canonical > preference > focus > correction > episodic > semantic
/// > candidate).
#[derive(Debug, Clone, Default)]
pub(crate) struct BenchCompilerInput {
    pub canonical: Vec<String>,
    pub preferences: Vec<String>,
    pub recent_episodic: Vec<String>,
    pub semantic: Vec<String>,
    pub raw_episodic: Vec<String>,
}

/// Outcome surfaced to the runtime layer. Fields mirror the telemetry
/// NDJSON schema in the contract — runtime appends them verbatim.
#[derive(Debug, Clone)]
pub(crate) struct BenchCompilerOutcome {
    pub markdown: String,
    pub tokens: usize,
    pub sections_included: Vec<String>,
    pub sections_dropped: Vec<String>,
    pub tokens_before_drop: usize,
}

/// Compile a bench prompt window. Pure: no IO. Maps the bench-shim
/// vocabulary onto V4 D4's `CompilerInput` then defers admission +
/// dedupe + render to `compile_wake`. Profile's `budget_tokens` is the
/// operational lever; profile's `priority` field is bench-side
/// documentation (V4's fixed order is the authoritative tie-breaker).
pub(crate) fn compile_for_bench(
    input: BenchCompilerInput,
    profile: &BudgetProfile,
) -> BenchCompilerOutcome {
    let tokens_before_drop: usize = total_chars(&input);

    let v4_input = CompilerInput {
        canonical: lift(input.canonical),
        preferences: lift(input.preferences),
        focus: lift(input.recent_episodic),
        episodic: lift(input.raw_episodic),
        semantic: lift(input.semantic),
        corrections: Vec::new(),
        candidates: Vec::new(),
        drift_notes: Vec::new(),
    };

    let budget = WakeBudget::default_2000().with_tokens(profile.budget_tokens);
    let compiled: CompiledWake = compile_wake(v4_input, budget);

    let mut sections_included: Vec<String> = Vec::new();
    let mut sections_dropped: Vec<String> = Vec::new();
    for kind in BucketKind::ALL {
        let label = bench_label_for(kind);
        let report = compiled
            .bucket_report
            .get(&kind)
            .cloned()
            .unwrap_or_default();
        if report.admitted > 0 {
            sections_included.push(label.to_string());
        } else if report.demoted > 0 {
            sections_dropped.push(label.to_string());
        }
    }

    BenchCompilerOutcome {
        markdown: compiled.markdown.clone(),
        tokens: compiled.tokens,
        sections_included,
        sections_dropped,
        tokens_before_drop,
    }
}

fn lift(records: Vec<String>) -> Vec<CompactMemoryRecord> {
    records
        .into_iter()
        .filter(|r| !r.trim().is_empty())
        .map(|record| CompactMemoryRecord {
            id: Uuid::new_v4(),
            record,
        })
        .collect()
}

fn total_chars(input: &BenchCompilerInput) -> usize {
    let mut n = 0usize;
    for v in [
        &input.canonical,
        &input.preferences,
        &input.recent_episodic,
        &input.semantic,
        &input.raw_episodic,
    ] {
        for s in v {
            n = n.saturating_add(s.len());
        }
    }
    n
}

/// Translate V4 `BucketKind` into the bench-shim vocabulary. Inverse
/// of the contract's label table (`docs/contracts/bench-compiler.md`
/// §2). `Correction` and `Candidate` are inert at the shim layer —
/// they are not surfaced from typed-ingest at the bench surface.
fn bench_label_for(kind: BucketKind) -> &'static str {
    match kind {
        BucketKind::Canonical => "canonical",
        BucketKind::Preference => "preferences",
        BucketKind::Focus => "recent_episodic",
        BucketKind::Episodic => "raw_episodic",
        BucketKind::Semantic => "semantic",
        BucketKind::Correction => "correction",
        BucketKind::Candidate => "candidate",
    }
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
