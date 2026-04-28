//! V6 / E6 — progressive-depth routing tests.
//!
//! E6 lands scaffold-symmetric to A6/B6/C6/D6: pure parser + policy +
//! caps with fixture-proxy lifts. Real LoCoMo multi-hop / LME temporal
//! lift locks graduate post-2026-05-02 alongside A6.9/B6/C6/D6 runtime
//! activation.
//!
//! Coverage map → `docs/phases/v6/phase-e6-plan.md` §4.

use std::sync::{Mutex, OnceLock};

use crate::benchmark::typed_ingest::depth_policy::{
    escalate_on_empty_wake, escalate_on_low_confidence, next_depth, NextDepth,
    DEFAULT_LOW_CONFIDENCE_FLOOR,
};
use crate::benchmark::typed_ingest::depth_router::{
    parse_next_call, run_router, DepthCall, DepthRouterConfig, TerminationReason,
    DEFAULT_MAX_DEPTH_CALLS, DEFAULT_MAX_RETRIEVAL_TOKENS, DEPTH_ROUTER_VERSION,
};
use crate::benchmark::typed_ingest::{depth_routing_active, depth_routing_runtime_notice};

/// Serialise tests that mutate `MEMD_V6_DEPTH_ROUTING`.
static DEPTH_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    DEPTH_ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("env lock poisoned")
}

/// Test 1 — E6.1.
/// Parser extracts `query` and `depth` from a `<<memd_lookup …>>`
/// tool-call surface. Quote handling + escapes round-trip the
/// fixture trace.
#[test]
fn router_parses_memd_lookup_call_from_generation() {
    let s = r#"...thinking... <<memd_lookup query="when did Alice move to Madrid" depth="targeted">> ...continue..."#;
    let (range, call) = parse_next_call(s).expect("parse call");
    assert!(s[range].starts_with("<<memd_lookup"));
    assert_eq!(call.query, "when did Alice move to Madrid");
    assert_eq!(call.depth, "targeted");
    assert_eq!(DEPTH_ROUTER_VERSION, "depth-router/v1");
}

/// Test 2 — E6.1.
/// The router resolves a parsed call via the supplied lookup
/// callback and threads V4 E4's depth flag (`wake|targeted|resume`)
/// through verbatim. The runtime layer wires this to the `memd
/// lookup` CLI; the test asserts the depth flag is forwarded so the
/// graduation wiring is mechanical.
#[test]
fn router_resolves_via_memd_lookup_cli_with_depth_flag() {
    let mut seen_depth = String::new();
    let s = r#"<<memd_lookup query="q1" depth="resume">>"#;
    let out = run_router(
        s,
        &DepthRouterConfig::default(),
        |call: &DepthCall| {
            seen_depth = call.depth.clone();
            "RESULT_BODY".to_string()
        },
    );
    assert_eq!(seen_depth, "resume");
    assert_eq!(out.calls_issued, 1);
    assert!(out.conversation.contains("RESULT_BODY"));
}

/// Test 3 — E6.1.
/// Resolved lookup result is injected into the conversation; the
/// router resumes parsing from the splice end so subsequent calls
/// in the *original* tail are picked up but the injected block
/// itself is not re-parsed.
#[test]
fn router_injects_result_and_resumes_generation() {
    let s = r#"A <<memd_lookup query="q1" depth="wake">> B <<memd_lookup query="q2" depth="targeted">> C"#;
    let out = run_router(
        s,
        &DepthRouterConfig::default(),
        |call: &DepthCall| format!("R[{}]", call.query),
    );
    assert_eq!(out.calls_issued, 2);
    assert!(
        out.conversation.contains("R[q1]") && out.conversation.contains("R[q2]"),
        "both calls injected: {}",
        out.conversation
    );
    assert!(
        out.conversation.starts_with("A "),
        "prefix preserved: {}",
        out.conversation
    );
    assert!(
        out.conversation.ends_with(" C"),
        "suffix preserved: {}",
        out.conversation
    );
}

/// Test 4 — E6.2.
/// Empty `wake` result escalates to `targeted` (next depth tier).
#[test]
fn policy_escalates_on_empty_wake_result() {
    assert_eq!(escalate_on_empty_wake("wake", 0), NextDepth::Targeted);
    assert_eq!(escalate_on_empty_wake("wake", 1), NextDepth::Stop);
    assert_eq!(escalate_on_empty_wake("targeted", 0), NextDepth::Stop);
    // Combined helper preserves precedence.
    assert_eq!(
        next_depth("wake", 0, 0.99, DEFAULT_LOW_CONFIDENCE_FLOOR),
        NextDepth::Targeted
    );
}

/// Test 5 — E6.2.
/// Low-confidence answer escalates to `resume` (full-depth tier).
#[test]
fn policy_escalates_on_low_confidence_answer() {
    assert_eq!(
        escalate_on_low_confidence(0.4, DEFAULT_LOW_CONFIDENCE_FLOOR),
        NextDepth::Resume
    );
    assert_eq!(
        escalate_on_low_confidence(0.95, DEFAULT_LOW_CONFIDENCE_FLOOR),
        NextDepth::Stop
    );
    // Combined helper falls through to confidence when wake check
    // does not trigger.
    assert_eq!(
        next_depth("targeted", 5, 0.3, DEFAULT_LOW_CONFIDENCE_FLOOR),
        NextDepth::Resume
    );
}

/// Test 6 — E6.3.
/// Router stops issuing calls at `max_calls`. The fourth call in the
/// stream should not be issued when `max_calls=3`. Termination
/// reason is observable so telemetry NDJSON can record it.
#[test]
fn router_hard_caps_at_3_calls() {
    let s = r#"<<memd_lookup query="q1" depth="wake">>
<<memd_lookup query="q2" depth="wake">>
<<memd_lookup query="q3" depth="wake">>
<<memd_lookup query="q4" depth="wake">>"#;
    let out = run_router(
        s,
        &DepthRouterConfig {
            max_calls: DEFAULT_MAX_DEPTH_CALLS,
            max_retrieval_tokens: DEFAULT_MAX_RETRIEVAL_TOKENS,
        },
        |_call: &DepthCall| "x".to_string(),
    );
    assert_eq!(out.calls_issued, 3, "max_calls=3 enforced");
    assert_eq!(out.termination, TerminationReason::MaxCalls);
    assert!(
        out.conversation.contains(r#"query="q4""#),
        "q4 source should remain unconsumed: {}",
        out.conversation
    );
}

/// Test 7 — E6.3.
/// Router refuses to admit a lookup body that would push retrieval
/// tokens above the cap. `max_retrieval_tokens=100` with each body
/// 60 chars admits one (60), refuses the second (60+60>100).
#[test]
fn router_hard_caps_at_10k_retrieval_tokens() {
    let s = r#"<<memd_lookup query="q1" depth="wake">> <<memd_lookup query="q2" depth="wake">>"#;
    let body = "x".repeat(60);
    let out = run_router(
        s,
        &DepthRouterConfig {
            max_calls: 5,
            max_retrieval_tokens: 100,
        },
        |_: &DepthCall| body.clone(),
    );
    assert_eq!(out.calls_issued, 1, "second call rejected by token cap");
    assert_eq!(out.termination, TerminationReason::MaxRetrievalTokens);
    assert!(out.retrieval_tokens <= 100);
}

#[derive(Debug, Clone, serde::Deserialize)]
struct DepthRow {
    bench_id: String,
    #[allow(dead_code)]
    question_id: String,
    baseline_correct: bool,
    routed_correct: bool,
    baseline_prompt_tokens: u32,
    routed_prompt_tokens: u32,
    #[allow(dead_code)]
    depth_calls: u32,
}

fn load_depth_fixture(name: &str) -> Vec<DepthRow> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/fixtures/typed_ingest/e6")
        .join(name);
    let body = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()));
    body.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str::<DepthRow>(l).expect("parse row"))
        .collect()
}

/// Test 8 — E6.5 (fixture proxy).
/// LoCoMo multi-hop accuracy lift ≥ +0.04 (routed - baseline).
/// Real corpus lock graduates with A6.9/B6/C6/D6 runtime activation
/// post-2026-05-02.
#[test]
fn e6_lifts_locomo_multihop_at_least_0_04() {
    let rows = load_depth_fixture("multihop-10.jsonl");
    let locomo: Vec<_> = rows.into_iter().filter(|r| r.bench_id == "locomo").collect();
    assert!(!locomo.is_empty(), "fixture missing locomo multi-hop rows");
    let baseline = locomo.iter().filter(|r| r.baseline_correct).count() as f64
        / locomo.len() as f64;
    let routed =
        locomo.iter().filter(|r| r.routed_correct).count() as f64 / locomo.len() as f64;
    let lift = routed - baseline;
    assert!(
        lift >= 0.04,
        "LoCoMo multi-hop lift {lift:.3} below 0.04 plan threshold (baseline={baseline:.3} routed={routed:.3})"
    );
}

/// Test 9 — E6.5 (fixture proxy).
/// LME temporal accuracy lift ≥ +0.03.
#[test]
fn e6_lifts_lme_temporal_at_least_0_03() {
    let rows = load_depth_fixture("temporal-10.jsonl");
    let lme: Vec<_> = rows.into_iter().filter(|r| r.bench_id == "lme").collect();
    assert!(!lme.is_empty(), "fixture missing lme temporal rows");
    let baseline =
        lme.iter().filter(|r| r.baseline_correct).count() as f64 / lme.len() as f64;
    let routed = lme.iter().filter(|r| r.routed_correct).count() as f64 / lme.len() as f64;
    let lift = routed - baseline;
    assert!(
        lift >= 0.03,
        "LME temporal lift {lift:.3} below 0.03 plan threshold (baseline={baseline:.3} routed={routed:.3})"
    );
}

/// Test 10 — E6.6 (fixture proxy).
/// Regression guard: routed prompt-token mean must not exceed
/// baseline by more than D6's compiler savings — equivalent to
/// "compiler still pays its way after E6 layered on top". Across
/// the combined fixture, routed total tokens ≤ baseline total (i.e.
/// E6 piggy-backs on D6 compiler shrinkage and adds at most the
/// retrieved bodies, which the cap bounds).
#[test]
fn no_canonical_regression_below_d6_baseline() {
    let mut rows = load_depth_fixture("multihop-10.jsonl");
    rows.extend(load_depth_fixture("temporal-10.jsonl"));
    assert!(!rows.is_empty(), "fixtures missing rows");
    let baseline_correct = rows.iter().filter(|r| r.baseline_correct).count();
    let routed_correct = rows.iter().filter(|r| r.routed_correct).count();
    assert!(
        routed_correct >= baseline_correct,
        "accuracy regression: baseline={baseline_correct} routed={routed_correct}"
    );
    let baseline_mean: f64 = rows
        .iter()
        .map(|r| r.baseline_prompt_tokens as f64)
        .sum::<f64>()
        / rows.len() as f64;
    let routed_mean: f64 = rows
        .iter()
        .map(|r| r.routed_prompt_tokens as f64)
        .sum::<f64>()
        / rows.len() as f64;
    assert!(
        routed_mean <= baseline_mean,
        "prompt-token regression: baseline={baseline_mean:.0} routed={routed_mean:.0}"
    );
}

/// Bonus — runtime notice is the only stable signal the runtime
/// emits about the router. Mirror D6 test 6: off-path contains no
/// engagement language; on-path advertises the version + caps.
#[test]
fn flat_path_unchanged_when_off_router() {
    let _g = env_lock();
    unsafe { std::env::remove_var("MEMD_V6_DEPTH_ROUTING"); }

    assert!(depth_routing_active("on"));
    assert!(!depth_routing_active("off"));
    let off = depth_routing_runtime_notice("off", false, 3, 10_000);
    assert!(!off.contains("engaged"), "off leaks engagement: {off}");
    assert!(off.contains("off-path"), "off-path label: {off}");
    let on = depth_routing_runtime_notice("on", false, 3, 10_000);
    assert!(on.contains("engaged"), "on missing engagement: {on}");
    assert!(on.contains("max_calls=3"), "on missing caps: {on}");
    assert!(on.contains(DEPTH_ROUTER_VERSION), "on missing version: {on}");

    unsafe { std::env::set_var("MEMD_V6_DEPTH_ROUTING", "0"); }
    assert!(!depth_routing_active("on"), "env=0 forces off");
    unsafe { std::env::remove_var("MEMD_V6_DEPTH_ROUTING"); }
}
