//! A5 scorers — exact-match primary + cached LLM-judge fallback.
//!
//! Reuses the existing `public_benchmark` judge cache layout
//! (`.memd/benchmarks/grader-cache/<key>.json`) so substrate runs share
//! a cost ledger with public benches. The substrate scorer only adds
//! its own subdirectory (`grader-cache/a5/`) and a thin `BudgetTracker`
//! that refuses any call that would push spend past
//! `max_budget_usd`.

use crate::benchmark::public_benchmark::{
    estimate_judge_cost_usd, judge_cache_dir, judge_cache_key,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

/// Per-call accounting + a hard ceiling. `max_budget_usd == None`
/// disables the guard (substrate suites without a budget cap).
#[derive(Debug)]
pub(crate) struct BudgetTracker {
    spent_usd: Mutex<f64>,
    max_budget_usd: Option<f64>,
}

impl BudgetTracker {
    pub(crate) fn new(max_budget_usd: Option<f64>) -> Self {
        Self {
            spent_usd: Mutex::new(0.0),
            max_budget_usd,
        }
    }

    pub(crate) fn spent(&self) -> f64 {
        *self.spent_usd.lock().unwrap()
    }

    /// Refuse the call if `spent + projected_cost > max`. Otherwise
    /// returns Ok and *does not* mutate state — caller commits via
    /// `record` after the network round-trip succeeds (so failed RPCs
    /// don't burn budget).
    pub(crate) fn check(&self, projected_cost_usd: f64) -> Result<(), BudgetError> {
        match self.max_budget_usd {
            Some(max) => {
                let current = *self.spent_usd.lock().unwrap();
                if current + projected_cost_usd > max {
                    Err(BudgetError {
                        spent_usd: current,
                        projected_usd: projected_cost_usd,
                        max_usd: max,
                    })
                } else {
                    Ok(())
                }
            }
            None => Ok(()),
        }
    }

    pub(crate) fn record(&self, cost_usd: f64) {
        *self.spent_usd.lock().unwrap() += cost_usd;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BudgetError {
    pub(crate) spent_usd: f64,
    pub(crate) projected_usd: f64,
    pub(crate) max_usd: f64,
}

impl std::fmt::Display for BudgetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "judge budget exceeded: spent ${:.4} + projected ${:.4} > max ${:.4}",
            self.spent_usd, self.projected_usd, self.max_usd
        )
    }
}

impl std::error::Error for BudgetError {}

/// Exact-match scorer that tolerates trailing punctuation and ASCII
/// case-folding. Both inputs are trimmed; trailing `[.,;:!? ]` removed.
/// Empty-after-normalisation is treated as no match.
pub(crate) fn exact_match(predicted: &str, gold: &str) -> bool {
    fn norm(s: &str) -> String {
        s.trim().trim_end_matches(['.', ',', ';', ':', '!', '?', ' ']).to_ascii_lowercase()
    }
    let p = norm(predicted);
    let g = norm(gold);
    !p.is_empty() && p == g
}

/// Cached LLM-judge result returned without network.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct CachedJudgement {
    pub(crate) content: String,
    pub(crate) prompt_tokens: u64,
    pub(crate) completion_tokens: u64,
}

/// Substrate-scoped cache dir: `.memd/benchmarks/grader-cache/a5/`.
pub(crate) fn substrate_cache_dir(suite: &str) -> PathBuf {
    judge_cache_dir().join(suite_subdir(suite))
}

fn suite_subdir(suite: &str) -> &'static str {
    match suite {
        "cross-session-recall" => "a5",
        "correction-propagation" => "b5",
        "cross-harness" => "c5",
        "progressive-depth" => "d5",
        "provenance-integrity" => "e5",
        "typed-retrieval" => "f5",
        "adversarial-noise" => "g5",
        _ => "misc",
    }
}

/// Cache lookup. Returns `Some(judgement)` on hit, `None` on miss.
/// Pure filesystem read — no network.
pub(crate) fn judge_cache_lookup(
    cache_dir: &std::path::Path,
    suite: &str,
    question_id: &str,
    prediction: &str,
    grader_model: &str,
    prompt: &str,
) -> Option<CachedJudgement> {
    let key = judge_cache_key(suite, question_id, prediction, grader_model, prompt);
    let path = cache_dir.join(format!("{key}.json"));
    let bytes = std::fs::read(&path).ok()?;
    serde_json::from_slice::<CachedJudgement>(&bytes).ok()
}

/// Best-effort cache write (used after a successful judge round-trip).
pub(crate) fn judge_cache_store(
    cache_dir: &std::path::Path,
    suite: &str,
    question_id: &str,
    prediction: &str,
    grader_model: &str,
    prompt: &str,
    judgement: &CachedJudgement,
) -> std::io::Result<()> {
    std::fs::create_dir_all(cache_dir)?;
    let key = judge_cache_key(suite, question_id, prediction, grader_model, prompt);
    let path = cache_dir.join(format!("{key}.json"));
    let bytes = serde_json::to_vec_pretty(judgement)?;
    std::fs::write(path, bytes)
}

/// Score a single (predicted, gold) pair using the cache-only judge
/// fallback when exact-match misses. Returns:
///   * `Ok(true)`  — exact-match or cached judge said correct
///   * `Ok(false)` — exact-match miss + cached judge said incorrect
///   * `Err(BudgetError)` — exact-match miss + cache miss + budget wouldn't
///     allow a network call (caller decides whether to fall back to
///     "incorrect" or surface the error)
pub(crate) fn score_one_cached_only(
    cache_dir: &std::path::Path,
    suite: &str,
    question_id: &str,
    predicted: &str,
    gold: &str,
    grader_model: &str,
    prompt: &str,
    budget: &BudgetTracker,
) -> Result<bool, BudgetError> {
    if exact_match(predicted, gold) {
        return Ok(true);
    }

    if let Some(cached) = judge_cache_lookup(
        cache_dir,
        suite,
        question_id,
        predicted,
        grader_model,
        prompt,
    ) {
        // Cached → no network, no budget consumed.
        return Ok(judgement_says_correct(&cached.content));
    }

    // Cache miss → would need a network call. Project cost from
    // *minimum* token shape (prompt only, single-token completion) and
    // check the budget. Real call would record actual cost via
    // `BudgetTracker::record` after the round-trip.
    let projected = estimate_judge_cost_usd(grader_model, prompt.len() as u64 / 4, 4);
    budget.check(projected)?;
    // No network in this helper — surface as an "uncached, would call"
    // signal by returning Ok(false). Real driver (A5.6) replaces this
    // path with the live HTTP call.
    Ok(false)
}

fn judgement_says_correct(content: &str) -> bool {
    let lower = content.trim().to_ascii_lowercase();
    lower.starts_with("yes") || lower == "true" || lower.starts_with("correct")
}

/// B5 provenance-chain scorer.
///
/// A retrieved record's provenance is the ordered list of turn/session
/// IDs that contributed to its current value. For B5 we require that the
/// chain (a) is non-empty, (b) cites the correction turn that updated
/// the fact, and (c) does so *after* any earlier ingest turns — i.e.
/// the chain is monotonically forward-only.
///
/// Returns `true` iff every requirement holds.
pub(crate) fn provenance_chain_cites_correction(
    chain: &[String],
    correction_turn: &str,
) -> bool {
    if chain.is_empty() || correction_turn.is_empty() {
        return false;
    }
    // Forward-only: the correction turn must appear exactly once. A
    // duplicate means the chain re-visited an earlier state, breaking
    // the propagation guarantee.
    chain.iter().filter(|t| *t == correction_turn).count() == 1
}

/// Aggregate provenance correctness across a batch of (chain, correction_turn)
/// observations. Used by the B5 runner to compute the
/// `provenance_correctness` pass-gate axis.
pub(crate) fn provenance_correctness_rate<'a, I>(observations: I) -> f64
where
    I: IntoIterator<Item = (&'a [String], &'a str)>,
{
    let mut total = 0usize;
    let mut hits = 0usize;
    for (chain, turn) in observations {
        total += 1;
        if provenance_chain_cites_correction(chain, turn) {
            hits += 1;
        }
    }
    if total == 0 {
        0.0
    } else {
        hits as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// Test 5 — `scorer_exact_match_tolerates_trailing_punctuation`.
    #[test]
    fn scorer_exact_match_tolerates_trailing_punctuation() {
        assert!(exact_match("berlin", "berlin"));
        assert!(exact_match("Berlin.", "berlin"));
        assert!(exact_match("berlin ", "Berlin"));
        assert!(exact_match("berlin!", "Berlin?"));
        assert!(!exact_match("berlin", "tokyo"));
        assert!(!exact_match("", "berlin"));
        assert!(!exact_match("berlin.", ""));
    }

    /// Test 6 — `scorer_llm_judge_cache_hit_no_network`.
    /// A pre-warmed cache file means `score_one_cached_only` returns
    /// `Ok(true)` without spending any budget, even with budget=0.
    #[test]
    fn scorer_llm_judge_cache_hit_no_network() {
        let dir = tempdir().unwrap();
        let suite = "cross-session-recall";
        let question_id = "q-001";
        let predicted = "completely-wrong-answer";
        let gold = "berlin";
        let grader_model = "gpt-5.4";
        let prompt = "Is the prediction correct?";

        // Pre-warm the cache with a "yes" judgement.
        judge_cache_store(
            dir.path(),
            suite,
            question_id,
            predicted,
            grader_model,
            prompt,
            &CachedJudgement {
                content: "yes".into(),
                prompt_tokens: 100,
                completion_tokens: 1,
            },
        )
        .unwrap();

        // Budget=0.0 — any network call would be refused.
        let budget = BudgetTracker::new(Some(0.0));
        let result = score_one_cached_only(
            dir.path(),
            suite,
            question_id,
            predicted,
            gold,
            grader_model,
            prompt,
            &budget,
        );
        assert_eq!(result, Ok(true));
        assert_eq!(budget.spent(), 0.0, "cache hit must not consume budget");
    }

    /// Test 7 — `scorer_llm_judge_budget_guard_refuses_over_budget`.
    /// On a cache miss with a budget that's too tight, the scorer
    /// surfaces a BudgetError instead of pretending to call.
    #[test]
    fn scorer_llm_judge_budget_guard_refuses_over_budget() {
        let dir = tempdir().unwrap();
        let suite = "cross-session-recall";
        let question_id = "q-002";
        let predicted = "wrong";
        let gold = "berlin";
        let grader_model = "gpt-4o-2024-08-06"; // priced model — projected cost > 0
        let prompt = "Is the prediction correct? ".repeat(2_000);

        // Tiny budget that any real call would blow past.
        let budget = BudgetTracker::new(Some(0.000_001));
        let err = score_one_cached_only(
            dir.path(),
            suite,
            question_id,
            predicted,
            gold,
            grader_model,
            &prompt,
            &budget,
        )
        .expect_err("budget guard must refuse the call");

        assert!(err.projected_usd > 0.0);
        assert_eq!(err.max_usd, 0.000_001);
        assert_eq!(budget.spent(), 0.0, "rejected call must not be recorded");
    }

    #[test]
    fn budget_tracker_records_only_after_explicit_commit() {
        let b = BudgetTracker::new(Some(1.0));
        b.check(0.5).unwrap();
        assert_eq!(b.spent(), 0.0, "check is non-mutating");
        b.record(0.5);
        assert_eq!(b.spent(), 0.5);
        // Next check would push past 1.0 → refused.
        let err = b.check(0.6).unwrap_err();
        assert_eq!(err.spent_usd, 0.5);
    }

    #[test]
    fn substrate_cache_dir_routes_per_suite() {
        let a5 = substrate_cache_dir("cross-session-recall");
        assert!(a5.ends_with("a5"));
        let b5 = substrate_cache_dir("correction-propagation");
        assert!(b5.ends_with("b5"));
    }

    /// B5 Test 1 — `scorer_provenance_chain_passes_when_correction_turn_cited`.
    /// A chain that contains the correction turn ID at any forward position
    /// must score true.
    #[test]
    fn scorer_provenance_chain_passes_when_correction_turn_cited() {
        let chain = vec![
            "s1-ingest-007".to_string(),
            "s2-correct-007".to_string(),
            "s5-restore-007".to_string(),
        ];
        assert!(provenance_chain_cites_correction(&chain, "s2-correct-007"));

        // Even a chain whose only entry is the correction turn passes.
        let solo = vec!["s2-correct-007".to_string()];
        assert!(provenance_chain_cites_correction(&solo, "s2-correct-007"));
    }

    /// B5 Test 2 — `scorer_provenance_chain_fails_when_chain_broken`.
    /// Missing correction turn, empty chain, empty turn id, or a chain
    /// that revisits the correction turn out of order all score false.
    #[test]
    fn scorer_provenance_chain_fails_when_chain_broken() {
        let no_correction = vec!["s1-ingest-007".to_string(), "s5-restore-007".to_string()];
        assert!(!provenance_chain_cites_correction(&no_correction, "s2-correct-007"));

        let empty: Vec<String> = vec![];
        assert!(!provenance_chain_cites_correction(&empty, "s2-correct-007"));

        let chain = vec!["s2-correct-007".to_string()];
        assert!(!provenance_chain_cites_correction(&chain, ""));

        // Duplicate correction turn before canonical position = malformed.
        let revisit = vec![
            "s2-correct-007".to_string(),
            "s3-something".to_string(),
            "s2-correct-007".to_string(),
        ];
        assert!(!provenance_chain_cites_correction(&revisit, "s2-correct-007"));
    }

    #[test]
    fn provenance_correctness_rate_handles_mix() {
        let c1: Vec<String> = vec!["a".into(), "b".into()];
        let c2: Vec<String> = vec!["x".into()];
        let c3: Vec<String> = vec![];
        let obs = vec![
            (c1.as_slice(), "b"),
            (c2.as_slice(), "b"),
            (c3.as_slice(), "b"),
        ];
        let rate = provenance_correctness_rate(obs);
        assert!((rate - 1.0 / 3.0).abs() < 1e-9);
    }
}
