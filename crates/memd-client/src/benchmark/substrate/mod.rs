//! V5 substrate-native benchmark suites.
//!
//! Public benches measure flat RAG; substrate suites measure what memd is
//! actually for: session continuity, correction propagation, cross-harness
//! continuity, progressive depth, provenance integrity, typed retrieval,
//! adversarial noise. A5 (cross-session-recall) is the first suite; B5–G5
//! follow the same shape.

use crate::cli::SubstrateArgs;

pub(crate) mod adversarial_noise;
pub(crate) mod aggregator;
pub(crate) mod competitor_card;
pub(crate) mod correction_behavior;
pub(crate) mod correction_propagation;
pub(crate) mod cross_harness;
pub(crate) mod cross_session_recall;
pub(crate) mod f5_live_fire;
pub(crate) mod fixtures;
pub(crate) mod harness_adapter;
pub(crate) mod progressive_depth;
pub(crate) mod provenance_auditor;
pub(crate) mod provenance_integrity;
pub(crate) mod report;
pub(crate) mod scorers;
pub(crate) mod session_driver;
pub(crate) mod ten_star_writer;
pub(crate) mod typed_retrieval;

use crate::benchmark::substrate::adversarial_noise::{G5RunConfig, run_g5_in_process};
use crate::benchmark::substrate::aggregator::{
    AggregatorOptions, regenerate_substrate_benchmarks_md, run_aggregator,
};
use crate::benchmark::substrate::correction_behavior::{
    V7RunConfig, run_v7_correction_behavior_in_process,
};
use crate::benchmark::substrate::correction_propagation::{B5RunConfig, run_b5_in_process};
use crate::benchmark::substrate::cross_harness::{C5RunConfig, run_c5_in_process};
use crate::benchmark::substrate::cross_session_recall::{A5RunConfig, run_a5_in_process};
use crate::benchmark::substrate::progressive_depth::{D5RunConfig, run_d5_in_process};
use crate::benchmark::substrate::provenance_integrity::{E5RunConfig, run_e5_in_process};
use crate::benchmark::substrate::report::upsert_markdown_section;
use crate::benchmark::substrate::ten_star_writer::{
    axis_scores_from_summaries, regenerate_10star_md,
};
use crate::benchmark::substrate::typed_retrieval::{F5RunConfig, run_f5_in_process};

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
    (
        "progressive-depth",
        "D5 — wake/lookup/resume quality ladder; quality-per-token at each depth",
    ),
    (
        "provenance-integrity",
        "E5 — every retrieved record has source_turn/captured_by/captured_at; hard 1.000 floor",
    ),
    (
        "typed-retrieval",
        "F5 — query shape routes to right MemoryKind; correct-type-rate@1 ≥ 0.85",
    ),
    (
        "adversarial-noise",
        "G5 — canonical beats noise siblings under recency-bias trap; canonical_wins ≥ 0.90, noise_leak ≤ 0.05",
    ),
    (
        "correction-behavior-change",
        "V7 C7/E7 — S1 corrections change S2 retrieval without prompt-repeat; provenance chain complete",
    ),
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

    if args.all {
        // G5 aggregator path — runs every suite in fixed order, optionally
        // regenerates SUBSTRATE_BENCHMARKS.md.
        let mut opts = AggregatorOptions::with_results_dir(args.output.clone());
        opts.seed = args.seed;
        opts.fail_fast = args.fail_fast;
        let summaries = run_aggregator(&opts);

        if args.regenerate_report {
            regenerate_substrate_benchmarks_md(&args.report, &summaries).map_err(|e| {
                anyhow::anyhow!("substrate aggregator: report regeneration failed: {e}")
            })?;
        }

        if args.regenerate_10star {
            let scores = axis_scores_from_summaries(&summaries);
            let ten_star_path =
                std::path::Path::new("docs/verification/MEMD-10-STAR.md").to_path_buf();
            match regenerate_10star_md(&ten_star_path, &scores, args.allow_below_target) {
                Ok(composite) => {
                    println!("substrate aggregator: 10-STAR composite {composite:.2}/10");
                }
                Err(e) => {
                    anyhow::bail!("substrate aggregator: 10-STAR regeneration failed: {e}");
                }
            }
        }

        let pass_count = summaries.iter().filter(|s| s.pass).count();
        let total = summaries.iter().filter(|s| !s.skipped).count();
        if args.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&summaries).unwrap_or_else(|_| "[]".into())
            );
        } else {
            for s in &summaries {
                let status = if s.skipped {
                    "skip"
                } else if s.pass {
                    "pass"
                } else {
                    "fail"
                };
                println!("substrate {}: {}", s.id, status);
            }
            println!("substrate aggregator: {pass_count}/{total} suites passing");
        }

        if pass_count < total {
            std::process::exit(1);
        }
        return Ok(());
    }

    let suites: Vec<&str> = vec![args.suite.as_deref().unwrap()];

    let mut overall_pass = true;
    for suite in suites {
        match suite {
            "cross-session-recall" => {
                let mut cfg = A5RunConfig::default_with_results_dir(args.output.clone());
                if let Some(seed) = args.seed {
                    cfg.seed = seed;
                }
                if let Some(only) = args.only_cuts.as_deref() {
                    let parsed: Result<Vec<usize>, _> =
                        only.split(',').map(|s| s.trim().parse::<usize>()).collect();
                    cfg.cuts = parsed.map_err(|e| {
                        anyhow::anyhow!("substrate: --only-cuts parse failure: {e}")
                    })?;
                }
                let outcome = run_a5_in_process(&cfg).map_err(|e| {
                    anyhow::anyhow!("substrate cross-session-recall runner io error: {e}")
                })?;
                upsert_markdown_section(&args.report, "cross-session-recall", &outcome.records)
                    .map_err(|e| anyhow::anyhow!("substrate: report write failed: {e}"))?;
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
                let outcome = run_c5_in_process(&cfg)
                    .map_err(|e| anyhow::anyhow!("substrate cross-harness runner io error: {e}"))?;
                upsert_markdown_section(&args.report, "cross-harness", &outcome.records)
                    .map_err(|e| anyhow::anyhow!("substrate: report write failed: {e}"))?;
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
                upsert_markdown_section(&args.report, "correction-propagation", &outcome.records)
                    .map_err(|e| anyhow::anyhow!("substrate: report write failed: {e}"))?;
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
            "correction-behavior-change" => {
                let mut cfg = V7RunConfig::default_with_results_dir(args.output.clone());
                if let Some(seed) = args.seed {
                    cfg.seed = seed;
                }
                let outcome = run_v7_correction_behavior_in_process(&cfg).map_err(|e| {
                    anyhow::anyhow!(
                        "substrate correction-behavior-change runner io error: {e}"
                    )
                })?;
                upsert_markdown_section(
                    &args.report,
                    "correction-behavior-change",
                    &outcome.records,
                )
                .map_err(|e| anyhow::anyhow!("substrate: report write failed: {e}"))?;
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
                        "substrate {suite}: next_session_behavior_rate={:.3}, chain_completeness={:.3}, pass={}",
                        outcome.next_session_behavior_rate,
                        outcome.chain_completeness_rate,
                        outcome.overall_pass
                    );
                }
            }
            "progressive-depth" => {
                let cfg = D5RunConfig::default_with_results_dir(args.output.clone());
                let outcome = run_d5_in_process(&cfg).map_err(|e| {
                    anyhow::anyhow!("substrate progressive-depth runner io error: {e}")
                })?;
                if !outcome.overall_pass {
                    overall_pass = false;
                }
                if args.json {
                    println!(
                        "{{\"suite\": \"progressive-depth\", \"pass\": {}}}",
                        outcome.overall_pass
                    );
                } else {
                    println!("substrate {suite}: pass={}", outcome.overall_pass);
                }
            }
            "provenance-integrity" => {
                let mut cfg = E5RunConfig::default_with_results_dir(args.output.clone());
                if let Some(seed) = args.seed {
                    cfg.seed = seed;
                }
                if args.inject_hole {
                    cfg.inject_hole = true;
                }
                let outcome = run_e5_in_process(&cfg).map_err(|e| {
                    anyhow::anyhow!("substrate provenance-integrity runner io error: {e}")
                })?;
                upsert_markdown_section(&args.report, "provenance-integrity", &outcome.records)
                    .map_err(|e| anyhow::anyhow!("substrate: report write failed: {e}"))?;
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
                        "substrate {suite}: completeness_rate={:.3}, unsourced={}, pass={}",
                        outcome.completeness_rate, outcome.unsourced_count, outcome.overall_pass
                    );
                }
            }
            "typed-retrieval" => {
                let cfg = F5RunConfig::default_with_results_dir(args.output.clone());
                let outcome = run_f5_in_process(&cfg).map_err(|e| {
                    anyhow::anyhow!("substrate typed-retrieval runner io error: {e}")
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
                    let correct_count = outcome.records.iter().filter(|r| r.correct_at_1).count();
                    let correct_rate = if outcome.records.is_empty() {
                        0.0
                    } else {
                        correct_count as f64 / outcome.records.len() as f64
                    };
                    println!(
                        "substrate {suite}: {}/{} correct (rate={:.3}), pass={}",
                        correct_count,
                        outcome.records.len(),
                        correct_rate,
                        outcome.overall_pass
                    );
                }
            }
            "adversarial-noise" => {
                let mut cfg = G5RunConfig::default_with_results_dir(args.output.clone());
                if let Some(seed) = args.seed {
                    cfg.seed = seed;
                }
                let outcome = run_g5_in_process(&cfg).map_err(|e| {
                    anyhow::anyhow!("substrate adversarial-noise runner io error: {e}")
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
                        "substrate {suite}: {} canonical, wins={:.3}, leak={:.3}, tie_break_prov={:.3}, pass={}",
                        outcome.records.len(),
                        outcome.canonical_wins_rate,
                        outcome.noise_leak_rate,
                        outcome.tie_break_by_provenance_rate,
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
        assert!(
            REGISTERED_SUITES
                .iter()
                .any(|(id, _)| *id == "cross-session-recall")
        );
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
            inject_hole: false,
            depth_only: None,
            regenerate_report: false,
            regenerate_10star: false,
            fail_fast: false,
            allow_below_target: false,
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
            inject_hole: false,
            depth_only: None,
            regenerate_report: false,
            regenerate_10star: false,
            fail_fast: false,
            allow_below_target: false,
        };
        let err = run_substrate_command(&args).await.unwrap_err();
        assert!(err.to_string().contains("--suite or --all is required"));
    }
}
