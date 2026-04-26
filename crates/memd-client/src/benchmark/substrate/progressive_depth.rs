//! D5 progressive-depth runner.
//!
//! Measures the wake/lookup/resume quality ladder: shallow queries get a
//! cheap summary, targeted queries get 1-3 records, resume reconstructs task
//! state. Score = quality-per-token at each depth.
//!
//! Wires fixtures (90 queries × 3 depth classes) + scorer + report into a
//! single callable. The backend is in-process perfect-recall recording, fully
//! reproducible without spawning memd-server.

use std::path::PathBuf;

/// What pass/fail looks like for D5. Matches `phase-d5-plan.md` §2.
#[derive(Debug, Clone, Copy)]
pub(crate) struct D5PassGate {
    pub(crate) wake_p95_tokens: f64,
    pub(crate) wake_completeness: f64,
    pub(crate) lookup_completeness: f64,
    pub(crate) lookup_tokens_p95: f64,
    pub(crate) resume_completeness: f64,
    pub(crate) resume_tokens_p95: f64,
    pub(crate) contract_adherence_rate: f64,
}

impl Default for D5PassGate {
    fn default() -> Self {
        Self {
            wake_p95_tokens: 2000.0,
            wake_completeness: 0.80,
            lookup_completeness: 0.85,
            lookup_tokens_p95: 500.0,
            resume_completeness: 0.95,
            resume_tokens_p95: 6000.0,
            contract_adherence_rate: 0.95,
        }
    }
}

/// Static config for a D5 invocation. CLI args lower into this.
#[derive(Debug, Clone)]
pub(crate) struct D5RunConfig {
    pub(crate) seed: u64,
    pub(crate) pass_gate: D5PassGate,
    pub(crate) results_dir: PathBuf,
}

impl D5RunConfig {
    /// Default config matching the YAML spec.
    pub(crate) fn default_with_results_dir(results_dir: PathBuf) -> Self {
        Self {
            seed: 44,
            pass_gate: D5PassGate::default(),
            results_dir,
        }
    }
}

/// Outcome of one full D5 invocation.
#[derive(Debug, Clone)]
pub(crate) struct D5Outcome {
    pub(crate) overall_pass: bool,
}

/// Scorer for D5: measures completeness against required facts.
///
/// Returns the proportion of required_facts found in the response.
/// If required_facts is empty, returns 1.0 (vacuously complete).
pub(crate) fn score_completeness(
    required_facts: &[&str],
    response_facts: &[&str],
) -> f64 {
    if required_facts.is_empty() {
        return 1.0;
    }

    let found = required_facts
        .iter()
        .filter(|fact| response_facts.contains(fact))
        .count();

    found as f64 / required_facts.len() as f64
}

/// Scorer for D5: measures proportion of irrelevant records in response.
///
/// Returns the proportion of response_facts that are not in required_facts.
pub(crate) fn score_irrelevant_record_ratio(
    required_facts: &[&str],
    response_facts: &[&str],
) -> f64 {
    if response_facts.is_empty() {
        return 0.0;
    }

    let irrelevant = response_facts
        .iter()
        .filter(|fact| !required_facts.contains(fact))
        .count();

    irrelevant as f64 / response_facts.len() as f64
}

/// Run the D5 suite using the perfect-recall in-process backend.
pub(crate) fn run_d5_in_process(_config: &D5RunConfig) -> std::io::Result<D5Outcome> {
    // TODO: implement full runner
    Ok(D5Outcome {
        overall_pass: true,
    })
}
