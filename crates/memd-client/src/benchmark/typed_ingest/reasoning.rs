//! V6 / F6 — iterative-reasoning scratchpad (scaffold-symmetric).
//!
//! F6 sits *downstream* of E6: where E6 lets the model re-query memd
//! once mid-answer, F6 lets the model chain N depth-routed lookups
//! into a multi-step scratchpad before committing an answer. The
//! scratchpad records each `lookup` and the final `answer`, so
//! per-question telemetry is observable end-to-end.
//!
//! Pure: takes a model-driver closure that returns either another
//! tool-call (`ReasoningStep::Lookup`) or a terminal answer
//! (`ReasoningStep::Answer`). Hard caps stop the loop early.
//!
//! Runtime activation graduates with A6.9 / B6 / C6 / D6 / E6 post
//! V5 close (2026-05-02).
//!
//! Contract: `docs/contracts/iterative-reasoning.md`.
//! Plan: `docs/phases/v6/phase-f6-plan.md` §3.

use crate::benchmark::typed_ingest::depth_router::DepthCall;

/// Schema version pin. Bumping the major invalidates older traces.
pub(crate) const REASONING_VERSION: &str = "iterative-reasoning/v1";

/// Default hard cap on reasoning steps per question. 5 lookups + a
/// final answer matches the F6 plan §3 schema. Override via
/// `--max-reasoning-steps` / `MEMD_V6_MAX_REASONING_STEPS`.
pub(crate) const DEFAULT_MAX_REASONING_STEPS: usize = 5;
pub(crate) const DEFAULT_MAX_REASONING_TOKENS: usize = 20_000;

/// One step in the reasoning scratchpad. Either a tool-call (which
/// the driver resolves and the engine records) or the model's final
/// answer (which terminates the loop).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ReasoningStep {
    Lookup { call: DepthCall, result_ids: Vec<String> },
    Answer { text: String },
}

/// Reason a reasoning loop terminated. Reported in telemetry NDJSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReasoningTermination {
    /// Driver emitted an `Answer` step.
    Answer,
    /// Hit `max_steps` cap before an answer.
    StepCap,
    /// Hit `max_retrieval_tokens` cap.
    TokenCap,
}

/// Outcome of one reasoning loop. Mirrors `DepthRouterOutcome` in
/// shape; the JSON serialisation matches the §3 scratchpad schema.
#[derive(Debug, Clone)]
pub(crate) struct ReasoningOutcome {
    pub steps: Vec<ReasoningStepRecord>,
    pub terminated_by: ReasoningTermination,
    pub retrieval_tokens: usize,
}

/// Recorded step — flat shape so serde renders the §3 schema.
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub(crate) struct ReasoningStepRecord {
    pub n: usize,
    pub action: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Configuration for one reasoning loop.
#[derive(Debug, Clone)]
pub(crate) struct ReasoningConfig {
    pub max_steps: usize,
    pub max_retrieval_tokens: usize,
}

impl Default for ReasoningConfig {
    fn default() -> Self {
        Self {
            max_steps: DEFAULT_MAX_REASONING_STEPS,
            max_retrieval_tokens: DEFAULT_MAX_REASONING_TOKENS,
        }
    }
}

/// Caller-supplied driver. Given the prior steps so far, returns the
/// next step. Real runtime wires this to the E6 router + judge model;
/// tests use deterministic fixtures.
pub(crate) trait ReasoningDriver {
    fn next_step(&mut self, prior: &[ReasoningStepRecord]) -> ReasoningStep;
}

impl<F> ReasoningDriver for F
where
    F: FnMut(&[ReasoningStepRecord]) -> ReasoningStep,
{
    fn next_step(&mut self, prior: &[ReasoningStepRecord]) -> ReasoningStep {
        self(prior)
    }
}

/// Run the scratchpad reasoning loop. The driver emits one step at a
/// time; the engine records it, enforces caps, and stops on `Answer`
/// or cap fired. Pure — IO is the runtime's job.
///
/// Takes a closure directly (not `impl ReasoningDriver`) so closure
/// inference can attach a higher-ranked lifetime bound at the call
/// site — passing the closure through the trait blanket forced one
/// concrete lifetime and broke `FnMut` inference for tests.
pub(crate) fn run_reasoning<F>(
    config: &ReasoningConfig,
    mut driver: F,
) -> ReasoningOutcome
where
    F: FnMut(&[ReasoningStepRecord]) -> ReasoningStep,
{
    let mut steps = Vec::<ReasoningStepRecord>::new();
    let mut retrieval_tokens = 0usize;

    loop {
        if steps.len() >= config.max_steps {
            return ReasoningOutcome {
                steps,
                terminated_by: ReasoningTermination::StepCap,
                retrieval_tokens,
            };
        }
        let n = steps.len() + 1;
        let next = driver(&steps);
        match next {
            ReasoningStep::Answer { text } => {
                steps.push(ReasoningStepRecord {
                    n,
                    action: "answer",
                    query: None,
                    depth: None,
                    result_ids: None,
                    text: Some(text),
                });
                return ReasoningOutcome {
                    steps,
                    terminated_by: ReasoningTermination::Answer,
                    retrieval_tokens,
                };
            }
            ReasoningStep::Lookup { call, result_ids } => {
                let call_tokens = call.query.chars().count()
                    + result_ids.iter().map(|s| s.chars().count()).sum::<usize>();
                if retrieval_tokens.saturating_add(call_tokens)
                    > config.max_retrieval_tokens
                {
                    return ReasoningOutcome {
                        steps,
                        terminated_by: ReasoningTermination::TokenCap,
                        retrieval_tokens,
                    };
                }
                retrieval_tokens += call_tokens;
                steps.push(ReasoningStepRecord {
                    n,
                    action: "lookup",
                    query: Some(call.query),
                    depth: Some(call.depth),
                    result_ids: Some(result_ids),
                    text: None,
                });
            }
        }
    }
}

/// Render the scratchpad as the JSON shape called out in §3 of the
/// plan: `{ "steps": [...], "terminated_by": "..." }`. Deterministic
/// — no timestamps, no IDs that aren't already in the steps.
pub(crate) fn scratchpad_json(outcome: &ReasoningOutcome) -> serde_json::Value {
    let terminated = match outcome.terminated_by {
        ReasoningTermination::Answer => "answer",
        ReasoningTermination::StepCap => "step_cap",
        ReasoningTermination::TokenCap => "token_cap",
    };
    serde_json::json!({
        "steps": outcome.steps,
        "terminated_by": terminated,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answer_terminates_loop() {
        let mut emitted = false;
        let out = run_reasoning(&ReasoningConfig::default(), |_| {
            assert!(!emitted, "driver called twice");
            emitted = true;
            ReasoningStep::Answer { text: "yes".to_string() }
        });
        assert_eq!(out.terminated_by, ReasoningTermination::Answer);
        assert_eq!(out.steps.len(), 1);
    }

    #[test]
    fn step_cap_fires_at_max() {
        let out = run_reasoning(
            &ReasoningConfig { max_steps: 2, max_retrieval_tokens: 10_000 },
            |_| ReasoningStep::Lookup {
                call: DepthCall { query: "q".into(), depth: "wake".into() },
                result_ids: vec!["a".into()],
            },
        );
        assert_eq!(out.terminated_by, ReasoningTermination::StepCap);
        assert_eq!(out.steps.len(), 2);
    }
}
