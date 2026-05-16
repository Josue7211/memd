//! Deterministic correction detector.
//!
//! Cheap rule set: phrase regexes + a window check that the turn references
//! a recent prior claim. All semantic confirmation lives in the LLM-judge.

use once_cell::sync::Lazy;
use regex::Regex;

use super::CorrectionCandidate;

/// Maximum age of a "prior claim" considered relevant for window check, in turns.
pub const DEFAULT_PRIOR_WINDOW: usize = 12;

const STOPWORDS: &[&str] = &[
    "the", "and", "for", "but", "not", "are", "was", "were", "you", "your", "this", "that", "with",
    "from", "have", "has", "had", "been", "they", "them", "their", "our", "his", "her", "its",
    "his", "she", "him", "her",
];

/// A single prior claim from the conversation, used for window check.
#[derive(Debug, Clone)]
pub struct PriorClaim {
    pub id: String,
    pub turn: String,
    pub content: String,
}

struct PhraseRule {
    name: &'static str,
    re: Regex,
    weight: f32,
}

static RULES: Lazy<Vec<PhraseRule>> = Lazy::new(|| {
    vec![
        rule(
            "no_x_is_y",
            r"(?i)\bno[,.\s]+(?:[a-z][a-z0-9_\-]*\s+){1,6}is\s+",
            0.55,
        ),
        rule("no_its", r"(?i)\bno[,.\s]+\s*it'?s\b", 0.5),
        rule("wait_actually", r"(?i)\bwait[,.\s]+actually\b", 0.6),
        rule("actually_x", r"(?i)\bactually[,.\s]+", 0.4),
        rule("i_meant", r"(?i)\bi\s+meant\b", 0.55),
        rule("correction_colon", r"(?i)\bcorrection\s*[:\-]\s*", 0.7),
        rule("scratch_that", r"(?i)\bscratch\s+that\b", 0.65),
        rule(
            "not_x_but_y",
            r"(?i)\bnot\s+[a-z0-9][a-z0-9_\-]*[,]?\s+but\s+",
            0.45,
        ),
        rule("rather", r"(?i)\b(?:or\s+)?rather[,.\s]", 0.3),
    ]
});

fn rule(name: &'static str, pattern: &str, weight: f32) -> PhraseRule {
    PhraseRule {
        name,
        re: Regex::new(pattern).expect("regex compile"),
        weight,
    }
}

/// Score a turn against the rule set. `prior_claims` supplies the recent
/// window — if the turn appears to reference one of them by token overlap,
/// the score is amplified and `corrects_id` is populated.
pub fn score(turn: &str, prior_claims: &[PriorClaim]) -> CorrectionCandidate {
    let trimmed = turn.trim();
    if trimmed.is_empty() {
        return CorrectionCandidate {
            score: 0.0,
            reasons: vec![],
            references_prior: false,
            corrects_id: None,
            source_turn: None,
        };
    }

    let mut total: f32 = 0.0;
    let mut hits: Vec<String> = Vec::new();
    for rule in RULES.iter() {
        let count = rule.re.find_iter(trimmed).count();
        if count > 0 {
            total += rule.weight * count.min(2) as f32;
            hits.push(rule.name.to_string());
        }
    }

    if total <= f32::EPSILON {
        return CorrectionCandidate {
            score: 0.0,
            reasons: vec![],
            references_prior: false,
            corrects_id: None,
            source_turn: None,
        };
    }

    let (references_prior, corrects_id, source_turn) = match_prior(trimmed, prior_claims);

    if !references_prior {
        // No prior reference: clamp to ≤0.5 so judge still gates promotion.
        total = total.min(0.5);
    } else {
        total += 0.15;
    }

    CorrectionCandidate {
        score: total.clamp(0.0, 1.0),
        reasons: hits,
        references_prior,
        corrects_id,
        source_turn,
    }
}

fn match_prior(turn: &str, prior: &[PriorClaim]) -> (bool, Option<String>, Option<String>) {
    if prior.is_empty() {
        return (false, None, None);
    }
    let turn_tokens = tokenize(turn);
    if turn_tokens.is_empty() {
        return (false, None, None);
    }

    let window_end = prior.len();
    let window_start = window_end.saturating_sub(DEFAULT_PRIOR_WINDOW);
    let mut best: Option<(usize, &PriorClaim)> = None;

    for claim in prior[window_start..window_end].iter() {
        let claim_tokens = tokenize(&claim.content);
        let overlap = claim_tokens
            .iter()
            .filter(|tok| turn_tokens.contains(*tok))
            .count();
        if overlap >= 2 && best.is_none_or(|(score, _)| overlap > score) {
            best = Some((overlap, claim));
        }
    }

    match best {
        Some((_, claim)) => (true, Some(claim.id.clone()), Some(claim.turn.clone())),
        None => (false, None, None),
    }
}

fn tokenize(text: &str) -> Vec<String> {
    text.to_ascii_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|tok| tok.len() >= 3 && !STOPWORDS.contains(tok))
        .map(|tok| tok.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prior(id: &str, turn: &str, content: &str) -> PriorClaim {
        PriorClaim {
            id: id.to_string(),
            turn: turn.to_string(),
            content: content.to_string(),
        }
    }

    #[test]
    fn detector_flags_no_x_is_y() {
        let prior = vec![prior(
            "rec-1",
            "t-1",
            "the primary key is autoincrement integer",
        )];
        let cand = score("no, the primary key is uuid", &prior);
        assert!(cand.score > 0.5, "score={}", cand.score);
        assert!(cand.references_prior);
        assert_eq!(cand.corrects_id.as_deref(), Some("rec-1"));
    }

    #[test]
    fn detector_flags_wait_actually_y() {
        let prior = vec![prior("rec-2", "t-3", "we deploy with terraform apply")];
        let cand = score("wait actually, we deploy with helm not terraform", &prior);
        assert!(cand.score > 0.5);
        assert!(cand.reasons.iter().any(|r| r == "wait_actually"));
    }

    #[test]
    fn detector_flags_i_meant_y() {
        let prior = vec![prior("rec-3", "t-5", "the host is alpha")];
        let cand = score("i meant the host is beta not alpha", &prior);
        assert!(cand.score > 0.5);
        assert!(cand.reasons.iter().any(|r| r == "i_meant"));
    }

    #[test]
    fn detector_ignores_neutral_text() {
        let prior = vec![prior("rec-4", "t-7", "we use postgres")];
        let cand = score("yes, that approach works fine", &prior);
        assert_eq!(cand.score, 0.0);
        assert!(cand.reasons.is_empty());
    }

    #[test]
    fn detector_requires_prior_claim_reference_within_window() {
        // Phrase fires but no prior overlap → clamp ≤0.5
        let cand = score("actually, I think we should reconsider", &[]);
        assert!(cand.score <= 0.5);
        assert!(!cand.references_prior);
        assert!(cand.corrects_id.is_none());
    }

    #[test]
    fn detector_scores_monotonically_with_phrase_count() {
        let prior = vec![prior("rec-5", "t-9", "production runs on machine alpha")];
        let single = score("no, production runs on machine beta", &prior);
        let many = score(
            "no, scratch that — i meant production runs on beta. correction: it's beta.",
            &prior,
        );
        assert!(
            many.score >= single.score,
            "many={} single={}",
            many.score,
            single.score
        );
    }
}
