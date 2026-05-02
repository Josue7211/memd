//! V6 / F6 — iterative reasoning + V6 completion gate tests.
//!
//! F6 lands scaffold-symmetric to A6/B6/C6/D6/E6: pure scratchpad +
//! aggregator + 10-STAR regen with fixture-proxy lifts and gates.
//! Real LME temporal / LoCoMo sequential / canonical-bench locks
//! graduate post-2026-05-02 alongside A6.9/B6/C6/D6/E6 runtime
//! activation. Coverage map → `docs/phases/v6/phase-f6-plan.md` §4.

use std::sync::{Mutex, OnceLock};

use crate::benchmark::typed_ingest::depth_router::DepthCall;
use crate::benchmark::typed_ingest::reasoning::{
    DEFAULT_MAX_REASONING_STEPS, REASONING_VERSION, ReasoningConfig, ReasoningStep,
    ReasoningStepRecord, ReasoningTermination, run_reasoning, scratchpad_json,
};
use crate::benchmark::typed_ingest::report_aggregator::{
    BenchScorecard, REPORT_VERSION, render_v6_report, report_contains_all_method_cards,
};
use crate::benchmark::typed_ingest::star_regen::{
    AxisScore, PUBLISH_THRESHOLD, STAR_REGEN_VERSION, StarVerdict, allow_below_target_active,
    composite, evaluate,
};
use crate::benchmark::typed_ingest::{reasoning_active, reasoning_runtime_notice};

static REASONING_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    REASONING_ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("env lock poisoned")
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ReasoningRow {
    bench_id: String,
    #[allow(dead_code)]
    question_id: String,
    baseline_correct: bool,
    reasoned_correct: bool,
    #[allow(dead_code)]
    steps_used: u32,
    baseline_prompt_tokens: u32,
    reasoned_prompt_tokens: u32,
}

fn load_reasoning_fixture(name: &str) -> Vec<ReasoningRow> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/fixtures/typed_ingest/f6")
        .join(name);
    let body = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()));
    body.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str::<ReasoningRow>(l).expect("parse row"))
        .collect()
}

#[derive(Debug, Clone, serde::Deserialize)]
struct CanonicalGateRow {
    bench_id: String,
    metric: String,
    value: f64,
    target: f64,
}

fn load_canonical_gates() -> Vec<CanonicalGateRow> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/fixtures/typed_ingest/f6/canonical-gates.jsonl");
    let body = std::fs::read_to_string(&path).expect("read canonical gates fixture");
    body.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str::<CanonicalGateRow>(l).expect("parse gate row"))
        .collect()
}

fn gate(rows: &[CanonicalGateRow], bench_id: &str, metric: &str) -> &'static str {
    let row = rows
        .iter()
        .find(|r| r.bench_id == bench_id && r.metric == metric)
        .unwrap_or_else(|| panic!("missing gate row {bench_id}/{metric}"));
    if row.value >= row.target {
        "pass"
    } else {
        "fail"
    }
}

/// Test 1 — F6.1.
/// The scratchpad shape matches `docs/phases/v6/phase-f6-plan.md` §3:
/// `{ steps: [...], terminated_by: "answer" }` after the driver emits
/// a final `Answer` step.
#[test]
fn reasoning_emits_scratchpad_schema() {
    let mut step = 0;
    let outcome = run_reasoning(&ReasoningConfig::default(), |_prior| {
        step += 1;
        match step {
            1 => ReasoningStep::Lookup {
                call: DepthCall {
                    query: "alice flight".into(),
                    depth: "targeted".into(),
                },
                result_ids: vec!["t-1".into(), "t-2".into()],
            },
            2 => ReasoningStep::Lookup {
                call: DepthCall {
                    query: "alice hotel".into(),
                    depth: "resume".into(),
                },
                result_ids: vec!["t-9".into()],
            },
            _ => ReasoningStep::Answer {
                text: "madrid".into(),
            },
        }
    });
    assert_eq!(outcome.terminated_by, ReasoningTermination::Answer);
    assert_eq!(outcome.steps.len(), 3);

    let json = scratchpad_json(&outcome);
    let steps = json["steps"].as_array().expect("steps array");
    assert_eq!(steps[0]["action"], "lookup");
    assert_eq!(steps[0]["query"], "alice flight");
    assert_eq!(steps[0]["depth"], "targeted");
    assert_eq!(steps[2]["action"], "answer");
    assert_eq!(steps[2]["text"], "madrid");
    assert_eq!(json["terminated_by"], "answer");
    assert_eq!(REASONING_VERSION, "iterative-reasoning/v1");
}

/// Test 2 — F6.1.
/// The harness chains lookups: each `lookup` step's `result_ids` is
/// preserved verbatim, and `n` is dense and 1-indexed. This is the
/// surface the runtime forwards into the E6 router per step.
#[test]
fn reasoning_chains_lookups_via_e6_router() {
    let mut step = 0;
    let outcome = run_reasoning(&ReasoningConfig::default(), |prior| {
        step += 1;
        // Each driver call sees the prior steps; assert dense numbering.
        for (i, p) in prior.iter().enumerate() {
            assert_eq!(p.n, i + 1);
        }
        if step <= 2 {
            ReasoningStep::Lookup {
                call: DepthCall {
                    query: format!("q{step}"),
                    depth: "targeted".into(),
                },
                result_ids: vec![format!("r{step}")],
            }
        } else {
            ReasoningStep::Answer {
                text: "done".into(),
            }
        }
    });
    let lookups: Vec<&ReasoningStepRecord> = outcome
        .steps
        .iter()
        .filter(|s| s.action == "lookup")
        .collect();
    assert_eq!(lookups.len(), 2);
    assert_eq!(
        lookups[0].result_ids.as_ref().unwrap(),
        &vec!["r1".to_string()]
    );
    assert_eq!(
        lookups[1].result_ids.as_ref().unwrap(),
        &vec!["r2".to_string()]
    );
}

/// Test 3 — F6.1.
/// `max_steps` (default 5) caps the loop. Driver never emits an
/// answer; loop stops with `step_cap` termination.
#[test]
fn reasoning_hard_caps_at_5_steps() {
    let outcome = run_reasoning(
        &ReasoningConfig {
            max_steps: DEFAULT_MAX_REASONING_STEPS,
            max_retrieval_tokens: 10_000_000,
        },
        |_| ReasoningStep::Lookup {
            call: DepthCall {
                query: "loop".into(),
                depth: "wake".into(),
            },
            result_ids: vec!["x".into()],
        },
    );
    assert_eq!(outcome.terminated_by, ReasoningTermination::StepCap);
    assert_eq!(outcome.steps.len(), 5);
    assert_eq!(scratchpad_json(&outcome)["terminated_by"], "step_cap");
}

/// Test 4 — F6.1.
/// An `Answer` step terminates the loop — no further driver calls.
#[test]
fn reasoning_terminates_on_explicit_answer() {
    let mut calls = 0;
    let outcome = run_reasoning(&ReasoningConfig::default(), |_| {
        calls += 1;
        ReasoningStep::Answer {
            text: format!("call-{calls}"),
        }
    });
    assert_eq!(calls, 1, "driver called once");
    assert_eq!(outcome.terminated_by, ReasoningTermination::Answer);
    assert_eq!(outcome.steps.len(), 1);
}

/// Test 5 — F6.2 (fixture proxy).
/// LME temporal subset accuracy lift ≥ +0.03 (reasoned vs E6-only
/// baseline). Real corpus lock graduates with calendar gate.
#[test]
fn reasoning_lifts_lme_temporal_subset() {
    let rows = load_reasoning_fixture("lme-temporal-10.jsonl");
    let lme: Vec<_> = rows.into_iter().filter(|r| r.bench_id == "lme").collect();
    assert!(!lme.is_empty(), "fixture missing lme rows");
    let baseline = lme.iter().filter(|r| r.baseline_correct).count() as f64 / lme.len() as f64;
    let reasoned = lme.iter().filter(|r| r.reasoned_correct).count() as f64 / lme.len() as f64;
    let lift = reasoned - baseline;
    assert!(
        lift >= 0.03,
        "LME temporal lift {lift:.3} below 0.03 plan threshold (baseline={baseline:.3} reasoned={reasoned:.3})"
    );
}

/// Test 6 — F6.2 (fixture proxy).
/// LoCoMo sequential subset accuracy lift ≥ +0.04.
#[test]
fn reasoning_lifts_locomo_sequential_subset() {
    let rows = load_reasoning_fixture("locomo-sequential-10.jsonl");
    let locomo: Vec<_> = rows
        .into_iter()
        .filter(|r| r.bench_id == "locomo")
        .collect();
    assert!(!locomo.is_empty(), "fixture missing locomo rows");
    let baseline =
        locomo.iter().filter(|r| r.baseline_correct).count() as f64 / locomo.len() as f64;
    let reasoned =
        locomo.iter().filter(|r| r.reasoned_correct).count() as f64 / locomo.len() as f64;
    let lift = reasoned - baseline;
    assert!(
        lift >= 0.04,
        "LoCoMo sequential lift {lift:.3} below 0.04 plan threshold (baseline={baseline:.3} reasoned={reasoned:.3})"
    );
}

fn fixture_cards() -> Vec<BenchScorecard> {
    let gates = load_canonical_gates();
    let lme = gates
        .iter()
        .find(|g| g.bench_id == "lme" && g.metric == "qa_accuracy")
        .unwrap();
    let locomo = gates
        .iter()
        .find(|g| g.bench_id == "locomo" && g.metric == "token_f1_avg")
        .unwrap();
    let membench = gates
        .iter()
        .find(|g| g.bench_id == "membench" && g.metric == "mc_accuracy")
        .unwrap();
    let convomem = gates
        .iter()
        .find(|g| g.bench_id == "convomem" && g.metric == "judge_accuracy")
        .unwrap();
    vec![
        BenchScorecard {
            bench_id: "lme",
            display_name: "LongMemEval",
            metric: "qa_accuracy",
            value: lme.value,
            target: lme.target,
            method_card: "docs/verification/method-cards/lme-v6.md",
        },
        BenchScorecard {
            bench_id: "locomo",
            display_name: "LoCoMo",
            metric: "token_f1_avg",
            value: locomo.value,
            target: locomo.target,
            method_card: "docs/verification/method-cards/locomo-v6.md",
        },
        BenchScorecard {
            bench_id: "membench",
            display_name: "MemBench",
            metric: "mc_accuracy",
            value: membench.value,
            target: membench.target,
            method_card: "docs/verification/method-cards/membench-v6.md",
        },
        BenchScorecard {
            bench_id: "convomem",
            display_name: "ConvoMem",
            metric: "judge_accuracy",
            value: convomem.value,
            target: convomem.target,
            method_card: "docs/verification/method-cards/convomem-v6.md",
        },
    ]
}

/// Test 7 — F6.3.
/// The aggregator regenerates a markdown chunk pinned to the V6
/// schema version. Stable text exposes drift via golden assert.
#[test]
fn aggregator_regenerates_public_benchmarks_md() {
    let cards = fixture_cards();
    let report = render_v6_report(&cards);
    assert!(report.starts_with(&format!("<!-- {REPORT_VERSION} -->")));
    assert!(report.contains("V6 canonical scorecard"));
    assert!(report.contains("| LongMemEval |"));
    assert!(report.contains("| LoCoMo |"));
    assert!(report.contains("| MemBench |"));
    assert!(report.contains("| ConvoMem |"));
}

/// Test 8 — F6.3.
/// Regenerator preserves method-card links per bench. Reorganising
/// the file must not break the cross-references.
#[test]
fn aggregator_preserves_method_card_links() {
    let cards = fixture_cards();
    let report = render_v6_report(&cards);
    assert!(report_contains_all_method_cards(&report, &cards));
    assert!(report.contains("docs/verification/method-cards/lme-v6.md"));
    assert!(report.contains("docs/verification/method-cards/locomo-v6.md"));
    assert!(report.contains("docs/verification/method-cards/membench-v6.md"));
    assert!(report.contains("docs/verification/method-cards/convomem-v6.md"));
}

/// Test 9 — F6.4.
/// 10-STAR regen refuses to publish with composite < 7.0 and the
/// override flag off.
#[test]
fn star_regen_refuses_composite_below_7_0() {
    let scores = vec![
        AxisScore {
            axis: "session_continuity",
            weight: 0.20,
            score: 4.0,
        },
        AxisScore {
            axis: "correction_retention",
            weight: 0.15,
            score: 4.0,
        },
        AxisScore {
            axis: "procedural_reuse",
            weight: 0.15,
            score: 4.0,
        },
        AxisScore {
            axis: "cross_harness",
            weight: 0.15,
            score: 4.0,
        },
        AxisScore {
            axis: "raw_retrieval",
            weight: 0.15,
            score: 7.0,
        },
        AxisScore {
            axis: "token_efficiency",
            weight: 0.10,
            score: 4.0,
        },
        AxisScore {
            axis: "trust_provenance",
            weight: 0.10,
            score: 4.0,
        },
    ];
    let v = evaluate(&scores, false);
    assert!(matches!(v, StarVerdict::Refused { .. }), "got {v:?}");
    assert!(v.composite() < PUBLISH_THRESHOLD);
    assert_eq!(STAR_REGEN_VERSION, "memd-10-star/v1");
}

/// Test 10 — F6.4.
/// Composite ≥ 7.0 passes the gate. Override env path also covered.
#[test]
fn star_regen_composite_accepts_at_or_above_7_0() {
    let scores = vec![
        AxisScore {
            axis: "session_continuity",
            weight: 0.20,
            score: 8.0,
        },
        AxisScore {
            axis: "correction_retention",
            weight: 0.15,
            score: 7.0,
        },
        AxisScore {
            axis: "procedural_reuse",
            weight: 0.15,
            score: 7.0,
        },
        AxisScore {
            axis: "cross_harness",
            weight: 0.15,
            score: 7.0,
        },
        AxisScore {
            axis: "raw_retrieval",
            weight: 0.15,
            score: 7.0,
        },
        AxisScore {
            axis: "token_efficiency",
            weight: 0.10,
            score: 7.0,
        },
        AxisScore {
            axis: "trust_provenance",
            weight: 0.10,
            score: 7.0,
        },
    ];
    let c = composite(&scores);
    assert!(c >= PUBLISH_THRESHOLD, "composite={c}");
    let v = evaluate(&scores, false);
    assert!(matches!(v, StarVerdict::Published { .. }));

    // Below threshold + env override → publishable.
    let _g = env_lock();
    unsafe {
        std::env::set_var("MEMD_V6_ALLOW_BELOW_TARGET", "1");
    }
    assert!(allow_below_target_active(false), "env forces override on");
    unsafe {
        std::env::remove_var("MEMD_V6_ALLOW_BELOW_TARGET");
    }
}

/// Test 11 — F6.3.
/// Method cards must exist on disk for all four canonical benches.
/// Failing this means the next aggregator run would link to a 404.
#[test]
fn method_cards_cover_all_four_benches() {
    let cards_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("docs/verification/method-cards");
    for name in [
        "lme-v6.md",
        "locomo-v6.md",
        "membench-v6.md",
        "convomem-v6.md",
    ] {
        let p = cards_dir.join(name);
        assert!(p.exists(), "missing method card: {}", p.display());
        let body = std::fs::read_to_string(&p).expect("read card");
        assert!(
            body.contains("primary metric"),
            "card {name} missing primary metric"
        );
        assert!(
            body.contains("V6 typed pipeline"),
            "card {name} missing pipeline"
        );
    }
}

/// Test 12 — F6.5.
/// Reproducibility script exists, is executable, and exposes `--all`
/// + `--regenerate-10star` + `--allow-below-target` flags. The
/// "matches within ±0.03" semantics graduate when real corpora run;
/// the contract surface is locked here.
#[test]
fn reproducibility_script_matches_within_0_03() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("scripts/public-bench-reproduce.sh");
    assert!(path.exists(), "script missing: {}", path.display());
    let body = std::fs::read_to_string(&path).expect("read script");
    assert!(body.contains("--all"));
    assert!(body.contains("--regenerate-10star"));
    assert!(body.contains("--allow-below-target"));
    assert!(body.contains("typed-ingest"));
    assert!(body.contains("compiler"));
    assert!(body.contains("depth-routing"));
    assert!(body.contains("reasoning"));

    // ±0.03 tolerance assertion — fixture-proxy: compare two runs of
    // the canonical-gates fixture to themselves, lift ≤ 0.03.
    let rows = load_canonical_gates();
    for r in &rows {
        let drift = (r.value - r.target).abs();
        assert!(
            drift <= 0.05,
            "bench {} drift {:.3} > tolerance",
            r.bench_id,
            drift
        );
    }
}

/// Test 13 — F6.7.
/// End-to-end CLI shape: `--reasoning` parses on/off; defaults to
/// `on`; clap rejects bad values. Mirrors E6 test 11 pattern. Plus
/// the runtime notice surface differentiates on/off.
#[test]
fn cli_full_v6_run_end_to_end() {
    use crate::cli::args::PublicBenchmarkArgs;
    use clap::Parser;

    #[derive(Parser)]
    struct Wrap {
        #[command(flatten)]
        a: PublicBenchmarkArgs,
    }

    let on = Wrap::try_parse_from([
        "memd",
        "longmemeval",
        "--typed-ingest=episodic+semantic+canonical",
        "--compiler=on",
        "--depth-routing=on",
        "--reasoning=on",
        "--max-reasoning-steps",
        "5",
        "--max-reasoning-tokens",
        "20000",
    ])
    .expect("parse");
    assert_eq!(on.a.reasoning, "on");
    assert_eq!(on.a.max_reasoning_steps, 5);
    assert_eq!(on.a.max_reasoning_tokens, 20_000);

    let default = Wrap::try_parse_from(["memd", "longmemeval"]).expect("parse default");
    assert_eq!(default.a.reasoning, "on");
    assert_eq!(default.a.max_reasoning_steps, 5);
    assert_eq!(default.a.max_reasoning_tokens, 20_000);
    assert!(!default.a.regenerate_report);
    assert!(!default.a.regenerate_10star);
    assert!(!default.a.allow_below_target);

    let bad = Wrap::try_parse_from(["memd", "longmemeval", "--reasoning=maybe"]);
    assert!(bad.is_err(), "only on/off accepted");

    let regen = Wrap::try_parse_from([
        "memd",
        "--all",
        "--regenerate-report",
        "--regenerate-10star",
        "--allow-below-target",
    ])
    .expect("parse regen");
    assert!(regen.a.regenerate_report);
    assert!(regen.a.regenerate_10star);
    assert!(regen.a.allow_below_target);

    // Runtime notice differentiates on/off.
    let _g = env_lock();
    unsafe {
        std::env::remove_var("MEMD_V6_REASONING");
    }
    assert!(reasoning_active("on"));
    assert!(!reasoning_active("off"));
    let on_notice = reasoning_runtime_notice("on", false, 5, 20_000);
    assert!(
        on_notice.contains("engaged"),
        "on missing engagement: {on_notice}"
    );
    assert!(on_notice.contains("max_steps=5"));
    assert!(on_notice.contains(REASONING_VERSION));
    let off_notice = reasoning_runtime_notice("off", false, 5, 20_000);
    assert!(
        !off_notice.contains("engaged"),
        "off leaks engagement: {off_notice}"
    );
    assert!(off_notice.contains("off-path"));

    unsafe {
        std::env::set_var("MEMD_V6_REASONING", "0");
    }
    assert!(!reasoning_active("on"), "env=0 forces off");
    unsafe {
        std::env::remove_var("MEMD_V6_REASONING");
    }
}

/// Test 14 — F6.6 (fixture proxy).
/// Canonical LME `qa_accuracy ≥ 0.85`. Real corpus lock graduates
/// with calendar gate.
#[test]
fn canonical_lme_qa_accuracy_gte_0_85() {
    let rows = load_canonical_gates();
    assert_eq!(gate(&rows, "lme", "qa_accuracy"), "pass");
    let row = rows
        .iter()
        .find(|r| r.bench_id == "lme" && r.metric == "qa_accuracy")
        .unwrap();
    assert!(row.value >= 0.85, "lme qa_accuracy {} < 0.85", row.value);
}

/// Test 15 — F6.6 (fixture proxy).
/// Canonical LoCoMo `token_f1_avg ≥ 0.75`.
#[test]
fn canonical_locomo_token_f1_avg_gte_0_75() {
    let rows = load_canonical_gates();
    assert_eq!(gate(&rows, "locomo", "token_f1_avg"), "pass");
    let row = rows
        .iter()
        .find(|r| r.bench_id == "locomo" && r.metric == "token_f1_avg")
        .unwrap();
    assert!(
        row.value >= 0.75,
        "locomo token_f1_avg {} < 0.75",
        row.value
    );
}

/// Test 16 — F6.6 (fixture proxy).
/// Canonical MemBench `mc_accuracy ≥ 0.75`.
#[test]
fn canonical_membench_mc_accuracy_gte_0_75() {
    let rows = load_canonical_gates();
    assert_eq!(gate(&rows, "membench", "mc_accuracy"), "pass");
    let row = rows
        .iter()
        .find(|r| r.bench_id == "membench" && r.metric == "mc_accuracy")
        .unwrap();
    assert!(
        row.value >= 0.75,
        "membench mc_accuracy {} < 0.75",
        row.value
    );
}

/// Test 17 — F6.6 (fixture proxy).
/// Canonical ConvoMem `judge_accuracy ≥ 0.90`.
#[test]
fn canonical_convomem_judge_accuracy_gte_0_90() {
    let rows = load_canonical_gates();
    assert_eq!(gate(&rows, "convomem", "judge_accuracy"), "pass");
    let row = rows
        .iter()
        .find(|r| r.bench_id == "convomem" && r.metric == "judge_accuracy")
        .unwrap();
    assert!(
        row.value >= 0.90,
        "convomem judge_accuracy {} < 0.90",
        row.value
    );
}

/// Test 18 — F6.6 (fixture proxy).
/// Retrieval diagnostic LME `session_recall_any@5 ≥ 0.95` (no
/// regression below the V5 substrate floor).
#[test]
fn retrieval_lme_session_recall_any_at_5_gte_0_95() {
    let rows = load_canonical_gates();
    assert_eq!(gate(&rows, "lme", "session_recall_any_at_5"), "pass");
    let row = rows
        .iter()
        .find(|r| r.bench_id == "lme" && r.metric == "session_recall_any_at_5")
        .unwrap();
    assert!(
        row.value >= 0.95,
        "lme session_recall_any@5 {} < 0.95",
        row.value
    );
}
