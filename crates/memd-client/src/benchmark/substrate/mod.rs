//! V5 substrate-native benchmark suites.
//!
//! Public benches measure flat RAG; substrate suites measure what memd is
//! actually for: session continuity, correction propagation, cross-harness
//! continuity, progressive depth, provenance integrity, typed retrieval,
//! adversarial noise. A5 (cross-session-recall) is the first suite; B5–G5
//! follow the same shape.

use crate::cli::SubstrateArgs;

pub(crate) mod correction_propagation;
pub(crate) mod cross_harness;
pub(crate) mod cross_session_recall;
pub(crate) mod fixtures;
pub(crate) mod harness_adapter;
pub(crate) mod report;
pub(crate) mod scorers;
pub(crate) mod session_driver;

use crate::benchmark::substrate::correction_propagation::{
    run_b5_in_process, B5RunConfig,
};
use crate::benchmark::substrate::cross_harness::{run_c5_in_process, C5RunConfig};
use crate::benchmark::substrate::cross_session_recall::{
    run_a5_in_process, A5RunConfig,
};
use crate::benchmark::substrate::report::upsert_markdown_section;

/// Static registry of every substrate suite the dispatcher knows about.
/// Each `(suite_id, summary)` pair shows up in `--help` and `--all`.
pub(crate) const REGISTERED_SUITES: &[(&str, &str)] = &[
    (
        "cross-session-recall",
        "A5 — recall across simulated session cuts (PostCompact restore path)",
    ),
    (
        "correction-propagation",
        "B5 — corrections in session 2 propagate forward; provenance chain cites correction turn",
    ),
    (
        "cross-harness",
        "C5 — claude_code and codex roundtrip facts via memd; visibility leaks hard 0",
    ),
    // D5..G5 register themselves here as they land.
];

/// Top-level dispatcher for `memd bench substrate`.
///
/// Responsibilities:
/// * Validate `--suite` / `--all` mutual exclusion.
/// * Resolve the bench-spec path (explicit `--spec` or registry default).
/// * Fan out to per-suite runners.
///
/// Per-suite implementations live in sibling modules and are wired in as
/// each phase (A5..G5) lands.
pub(crate) async fn run_substrate_command(args: &SubstrateArgs) -> anyhow::Result<()> {
    if args.all && args.suite.is_some() {
        anyhow::bail!("substrate: --suite and --all are mutually exclusive");
    }

    if !args.all && args.suite.is_none() {
        eprintln!("substrate: pick a suite (or pass --all). known suites:");
        for (id, summary) in REGISTERED_SUITES {
            eprintln!("  - {id:<28} {summary}");
        }
        anyhow::bail!("substrate: --suite or --all is required");
    }

    let suites: Vec<&str> = if args.all {
        REGISTERED_SUITES.iter().map(|(id, _)| *id).collect()
    } else {
        vec![args.suite.as_deref().unwrap()]
    };

    let mut overall_pass = true;
    for suite in suites {
        match suite {
            "cross-session-recall" => {
                let mut cfg = A5RunConfig::default_with_results_dir(args.output.clone());
                if let Some(seed) = args.seed {
                    cfg.seed = seed;
                }
                if let Some(only) = args.only_cuts.as_deref() {
                    let parsed: Result<Vec<usize>, _> = only
                        .split(',')
                        .map(|s| s.trim().parse::<usize>())
                        .collect();
                    cfg.cuts = parsed.map_err(|e| {
                        anyhow::anyhow!("substrate: --only-cuts parse failure: {e}")
                    })?;
                }
                let outcome = run_a5_in_process(&cfg).map_err(|e| {
                    anyhow::anyhow!("substrate cross-session-recall runner io error: {e}")
                })?;
                upsert_markdown_section(
                    &args.report,
                    "cross-session-recall",
                    &outcome.records,
                )
                .map_err(|e| {
                    anyhow::anyhow!("substrate: report write failed: {e}")
                })?;
                if !outcome.overall_pass {
                    overall_pass = false;
                }
                if args.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&outcome.records)
                            .unwrap_or_else(|_| "[]".into())
                    );
                } else {
                    println!(
                        "substrate {suite}: {} scenarios, pass={}",
                        outcome.records.len(),
                        outcome.overall_pass
                    );
                }
            }
            "cross-harness" => {
                let mut cfg = C5RunConfig::default_with_results_dir(args.output.clone());
                if let Some(seed) = args.seed {
                    cfg.seed = seed;
                }
                let outcome = run_c5_in_process(&cfg).map_err(|e| {
                    anyhow::anyhow!("substrate cross-harness runner io error: {e}")
                })?;
                upsert_markdown_section(
                    &args.report,
                    "cross-harness",
                    &outcome.records,
                )
                .map_err(|e| {
                    anyhow::anyhow!("substrate: report write failed: {e}")
                })?;
                if !outcome.overall_pass {
                    overall_pass = false;
                }
                if args.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&outcome.records)
                            .unwrap_or_else(|_| "[]".into())
                    );
                } else {
                    println!(
                        "substrate {suite}: {} scenarios, leaks={}, pass={}",
                        outcome.records.len(),
                        outcome.leaks.len(),
                        outcome.overall_pass
                    );
                }
            }
            "correction-propagation" => {
                let mut cfg = B5RunConfig::default_with_results_dir(args.output.clone());
                if let Some(seed) = args.seed {
                    cfg.seed = seed;
                }
                let outcome = run_b5_in_process(&cfg).map_err(|e| {
                    anyhow::anyhow!("substrate correction-propagation runner io error: {e}")
                })?;
                upsert_markdown_section(
                    &args.report,
                    "correction-propagation",
                    &outcome.records,
                )
                .map_err(|e| {
                    anyhow::anyhow!("substrate: report write failed: {e}")
                })?;
                if !outcome.overall_pass {
                    overall_pass = false;
                }
                if args.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&outcome.records)
                            .unwrap_or_else(|_| "[]".into())
                    );
                } else {
                    println!(
                        "substrate {suite}: {} scenarios, pass={}",
                        outcome.records.len(),
                        outcome.overall_pass
                    );
                }
            }
            other => anyhow::bail!("substrate: unknown suite '{other}'"),
        }
    }

    if !overall_pass {
        // Exit-1 contract from phase-a5-plan.md §3.
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_lists_a5_suite() {
        assert!(REGISTERED_SUITES.iter().any(|(id, _)| *id == "cross-session-recall"));
    }

    #[tokio::test]
    async fn dispatcher_rejects_both_suite_and_all() {
        let args = SubstrateArgs {
            suite: Some("cross-session-recall".into()),
            all: true,
            spec: None,
            seed: None,
            output: ".memd/benchmarks/substrate/results".into(),
            report: "docs/verification/SUBSTRATE_BENCHMARKS.md".into(),
            only_cuts: None,
            json: false,
            max_budget_usd: None,
            emit_fixtures: false,
        };
        let err = run_substrate_command(&args).await.unwrap_err();
        assert!(err.to_string().contains("mutually exclusive"));
    }

    #[tokio::test]
    async fn dispatcher_requires_suite_or_all() {
        let args = SubstrateArgs {
            suite: None,
            all: false,
            spec: None,
            seed: None,
            output: ".memd/benchmarks/substrate/results".into(),
            report: "docs/verification/SUBSTRATE_BENCHMARKS.md".into(),
            only_cuts: None,
            json: false,
            max_budget_usd: None,
            emit_fixtures: false,
        };
        let err = run_substrate_command(&args).await.unwrap_err();
        assert!(err.to_string().contains("--suite or --all is required"));
    }
}
