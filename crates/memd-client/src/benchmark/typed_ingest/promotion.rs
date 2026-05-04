//! V6 / C6 — canonical promotion rule engine.
//!
//! Promotes B6 `CandidateRecord`s (`stage=candidate`) to canonical
//! records (`stage=canonical`) when the rule card matches. Pure: no
//! network, no server, no I/O. The runtime layer (C6 dispatch, closed
//! with V6) wraps this to read the candidate store
//! and append to the canonical index.
//!
//! Contract: `docs/contracts/canonical-promotion.md`.
//!
//! See `phase-c6-plan.md` §2 for the rule card and §4 for the test
//! matrix.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::candidate_store::CandidateRecord;
use super::distiller::CandidateKind;

/// Frozen rule version. Bump invalidates prior canonical records.
pub(crate) const PROMOTION_RULE_VERSION: &str = "canonical-promotion/v1";

/// Rule card v1. Matches `phase-c6-plan.md` §2 and the contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PromotionRule {
    pub version: &'static str,
    pub corroboration_count: usize,
    pub confidence_min_milli: u32,
    pub session_age_min_turns: usize,
}

impl PromotionRule {
    pub(crate) fn v1() -> Self {
        Self {
            version: PROMOTION_RULE_VERSION,
            corroboration_count: 2,
            confidence_min_milli: 800,
            session_age_min_turns: 3,
        }
    }

    pub(crate) fn confidence_min(&self) -> f32 {
        self.confidence_min_milli as f32 / 1000.0
    }
}

/// Outcome of evaluating one canonical-identity group against the rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub(crate) enum PromotionOutcome {
    Promote(PromotionAccepted),
    Reject(PromotionRejected),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct PromotionAccepted {
    pub kind: CandidateKind,
    pub content: String,
    pub content_hash: String,
    pub corroboration_count: usize,
    pub min_confidence: f32,
    pub source_turn_ids: Vec<String>,
    pub rule_version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct PromotionRejected {
    pub kind: CandidateKind,
    pub content_hash: String,
    pub reason: RejectReason,
    pub min_confidence: f32,
    pub corroboration_count: usize,
    pub rule_version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RejectReason {
    LowConfidence,
    InsufficientCorroboration,
    WithinSessionWindow,
    ContradictsCanonical,
}

/// Compute the canonical content hash: sha256 of the normalised
/// content (trim, lowercase, collapse whitespace). Used as the
/// canonical identity key together with `kind`.
pub(crate) fn content_hash(content: &str) -> String {
    let normalised = normalise_content(content);
    let mut h = Sha256::new();
    h.update(normalised.as_bytes());
    format!("{:x}", h.finalize())
}

pub(crate) fn normalise_content(content: &str) -> String {
    content
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Identity key — `(kind, content_hash)` pair used to group
/// candidates and keep the canonical lane.
pub(crate) fn identity_key(kind: CandidateKind, hash: &str) -> String {
    format!("{}::{}", kind_tag(kind), hash)
}

fn kind_tag(kind: CandidateKind) -> &'static str {
    match kind {
        CandidateKind::Fact => "fact",
        CandidateKind::Decision => "decision",
        CandidateKind::Preference => "preference",
    }
}

/// Evaluate every canonical-identity group inside a candidate set.
/// Returns one outcome per group, ordered by first-seen identity.
///
/// `existing_canonical` is the set of identity keys already in the
/// canonical lane — groups whose identity is already promoted are
/// reported as `Promote` outcomes for idempotency, but the runtime
/// layer is responsible for deduping on write.
pub(crate) fn evaluate_candidates(
    candidates: &[CandidateRecord],
    rule: &PromotionRule,
) -> Vec<PromotionOutcome> {
    let mut groups: BTreeMap<String, Vec<&CandidateRecord>> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for c in candidates {
        let hash = content_hash(&c.content);
        let key = identity_key(c.kind, &hash);
        if !groups.contains_key(&key) {
            order.push(key.clone());
        }
        groups.entry(key).or_default().push(c);
    }

    order
        .into_iter()
        .map(|key| {
            let group = groups.get(&key).expect("inserted above");
            evaluate_group(group, rule)
        })
        .collect()
}

fn evaluate_group(group: &[&CandidateRecord], rule: &PromotionRule) -> PromotionOutcome {
    let kind = group[0].kind;
    let content = group[0].content.clone();
    let hash = content_hash(&content);

    let min_conf = group
        .iter()
        .map(|c| c.distill.confidence)
        .fold(f32::INFINITY, f32::min);

    let mut source_turn_ids: Vec<String> = group
        .iter()
        .flat_map(|c| c.distill.source_turn_ids.iter().cloned())
        .collect();
    source_turn_ids.sort();
    source_turn_ids.dedup();
    let corroboration = source_turn_ids.len();

    if min_conf < rule.confidence_min() {
        return PromotionOutcome::Reject(PromotionRejected {
            kind,
            content_hash: hash,
            reason: RejectReason::LowConfidence,
            min_confidence: min_conf,
            corroboration_count: corroboration,
            rule_version: rule.version.to_string(),
        });
    }

    if corroboration < rule.corroboration_count {
        return PromotionOutcome::Reject(PromotionRejected {
            kind,
            content_hash: hash,
            reason: RejectReason::InsufficientCorroboration,
            min_confidence: min_conf,
            corroboration_count: corroboration,
            rule_version: rule.version.to_string(),
        });
    }

    if !session_age_ok(group, rule.session_age_min_turns) {
        return PromotionOutcome::Reject(PromotionRejected {
            kind,
            content_hash: hash,
            reason: RejectReason::WithinSessionWindow,
            min_confidence: min_conf,
            corroboration_count: corroboration,
            rule_version: rule.version.to_string(),
        });
    }

    PromotionOutcome::Promote(PromotionAccepted {
        kind,
        content,
        content_hash: hash,
        corroboration_count: corroboration,
        min_confidence: min_conf,
        source_turn_ids,
        rule_version: rule.version.to_string(),
    })
}

/// Session-age check: within one `session_id`, the spread of
/// `turn_index` across corroborating turns must be ≥ threshold. Cross-
/// session corroboration always passes.
fn session_age_ok(group: &[&CandidateRecord], min_turns: usize) -> bool {
    let mut by_session: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    for c in group {
        let sid = c.provenance.session_id.clone();
        let idx = c.provenance.turn_index;
        let entry = by_session.entry(sid).or_insert((idx, idx));
        if idx < entry.0 {
            entry.0 = idx;
        }
        if idx > entry.1 {
            entry.1 = idx;
        }
    }
    if by_session.len() > 1 {
        return true;
    }
    let (lo, hi) = by_session
        .values()
        .next()
        .copied()
        .expect("group non-empty");
    (hi.saturating_sub(lo)) as usize >= min_turns
}

/// Contradiction check: a candidate `CONTRADICTS` an existing canonical
/// record when same-kind, content_hash differs, and normalised tokens
/// share the first 6 tokens (rule v1 surface; cosine path lives in the
/// runtime alongside dedupe). Returns the canonical-key the candidate
/// would conflict with, if any.
///
/// `existing` is a slice of `(kind, normalised_content)` for the
/// canonical lane. Pure: no I/O.
pub(crate) fn detects_contradiction(
    candidate_kind: CandidateKind,
    candidate_content: &str,
    existing: &[(CandidateKind, String)],
) -> Option<String> {
    let cand_norm = normalise_content(candidate_content);
    let cand_hash = content_hash(candidate_content);
    let cand_prefix = first_n_tokens(&cand_norm, 6);

    for (k, content) in existing {
        if *k != candidate_kind {
            continue;
        }
        let other_hash = content_hash(content);
        if other_hash == cand_hash {
            continue;
        }
        let other_norm = normalise_content(content);
        let other_prefix = first_n_tokens(&other_norm, 6);
        if cand_prefix == other_prefix && !cand_prefix.is_empty() {
            return Some(identity_key(*k, &other_hash));
        }
    }
    None
}

fn first_n_tokens(s: &str, n: usize) -> String {
    s.split_whitespace().take(n).collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_hash_normalises_whitespace_and_case() {
        let a = content_hash("User has a Shiba");
        let b = content_hash("  user has a   shiba  ");
        assert_eq!(a, b);
    }

    #[test]
    fn identity_key_separates_kinds() {
        let h = content_hash("x");
        assert_ne!(
            identity_key(CandidateKind::Fact, &h),
            identity_key(CandidateKind::Preference, &h)
        );
    }

    #[test]
    fn contradiction_only_fires_when_kind_matches_and_hash_differs() {
        let existing = vec![(
            CandidateKind::Fact,
            "User chose Rust over Go for the runtime".to_string(),
        )];
        // Different ending → same first 6 tokens → contradicts
        let hit = detects_contradiction(
            CandidateKind::Fact,
            "user chose Rust over Go for performance",
            &existing,
        );
        assert!(hit.is_some());
        // Same content → same hash → no contradiction
        let same = detects_contradiction(
            CandidateKind::Fact,
            "User chose Rust over Go for the runtime",
            &existing,
        );
        assert!(same.is_none());
        // Different kind → no contradiction
        let kind = detects_contradiction(
            CandidateKind::Preference,
            "user chose Rust over Go for performance",
            &existing,
        );
        assert!(kind.is_none());
    }
}
