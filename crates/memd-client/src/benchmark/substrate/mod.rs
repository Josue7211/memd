//! V5 substrate-native benchmark suites.
//!
//! Public benches measure flat RAG; substrate suites measure what memd is
//! actually for: session continuity, correction propagation, cross-harness
//! continuity, progressive depth, provenance integrity, typed retrieval,
//! adversarial noise. A5 (cross-session-recall) is the first suite; B5–G5
//! follow the same shape.

use crate::cli::SubstrateArgs;

pub(crate) mod fixtures;
pub(crate) mod report;
pub(crate) mod scorers;
pub(crate) mod session_driver;

/// Static registry of every substrate suite the dispatcher knows about.
/// Each `(suite_id, summary)` pair shows up in `--help` and `--all`.
pub(crate) const REGISTERED_SUITES: &[(&str, &str)] = &[
    (
        "cross-session-recall",
        "A5 — recall across simulated session cuts (PostCompact restore path)",
    ),
    // B5..G5 register themselves here as they land.
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

    for suite in suites {
        match suite {
            "cross-session-recall" => {
                anyhow::bail!(
                    "substrate suite '{suite}': runner not yet implemented (A5.2+); \
                     scaffolding landed in A5.1."
                );
            }
            other => anyhow::bail!("substrate: unknown suite '{other}'"),
        }
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
