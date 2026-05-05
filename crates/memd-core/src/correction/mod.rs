//! Correction lane: detector, judge, and shared records (Phase C4).
//!
//! Auto-detection of in-session corrections is conservative by design.
//! Detector runs deterministic regexes; LLM-judge confirms marginal candidates.

pub mod auto_apply;
pub mod detector;
pub mod judge;

use memd_schema::CaptureSource;
use serde::{Deserialize, Serialize};

/// One detector hit on a turn.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CorrectionCandidate {
    pub score: f32,
    pub reasons: Vec<String>,
    pub references_prior: bool,
    pub corrects_id: Option<String>,
    pub source_turn: Option<String>,
}

/// Provenance bundle attached to a stored correction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CorrectionProvenance {
    pub corrects_id: Option<String>,
    pub source_turn: Option<String>,
    pub captured_by: CaptureSource,
    pub confidence: f32,
}

/// Judge verdict tier — kept at the same surface as the detector candidate
/// so the calling code can branch uniformly.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CorrectionConfidence {
    Low,
    Medium,
    High,
}

impl CorrectionConfidence {
    pub fn from_score(score: f32) -> Self {
        if score >= 0.85 {
            Self::High
        } else if score >= 0.5 {
            Self::Medium
        } else {
            Self::Low
        }
    }
}

#[cfg(test)]
mod cross_harness_tests {
    use super::*;
    use chrono::Utc;
    use memd_schema::{
        CaptureSource as SchemaCaptureSource, CorrectionMetadata, MemoryItem, MemoryKind,
        MemoryScope, MemoryStage, MemoryStatus, MemoryVisibility,
    };
    use uuid::Uuid;

    fn item(id: Uuid, agent: &str, version: u64, kind: MemoryKind, content: &str) -> MemoryItem {
        MemoryItem {
            id,
            content: content.into(),
            redundancy_key: None,
            belief_branch: None,
            preferred: false,
            kind,
            scope: MemoryScope::Project,
            project: Some("memd".into()),
            namespace: Some("main".into()),
            workspace: Some("desk".into()),
            visibility: MemoryVisibility::default(),
            source_agent: Some(agent.into()),
            source_system: Some("memd".into()),
            source_path: None,
            source_quality: None,
            confidence: 0.9,
            ttl_seconds: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_verified_at: None,
            supersedes: vec![],
            tags: vec![],
            status: MemoryStatus::Active,
            stage: MemoryStage::Canonical,
            lane: None,
            version,
            correction_meta: None,
        }
    }

    #[test]
    fn cross_harness_correction_wins_when_lamport_higher() {
        // Belief stored under claude-code, version 3.
        let belief_id = Uuid::new_v4();
        let belief = item(belief_id, "claude-code", 3, MemoryKind::Fact, "host=alpha");

        // Codex stores a correction against the belief, version 7.
        let mut correction = item(
            Uuid::new_v4(),
            "codex",
            7,
            MemoryKind::Correction,
            "host=beta",
        );
        correction.supersedes = vec![belief_id];
        correction.correction_meta = Some(CorrectionMetadata {
            corrects_id: Some(belief_id),
            source_turn: Some("t-12".into()),
            captured_by: Some(SchemaCaptureSource::Detector),
            confidence: Some(0.92),
        });

        let winner = pick_correction_winner(&belief, &correction);
        assert_eq!(winner.source_agent.as_deref(), Some("codex"));
        assert_eq!(winner.id, correction.id);
        assert_eq!(winner.kind, MemoryKind::Correction);
    }

    #[test]
    fn cross_harness_correction_loses_when_lamport_stale() {
        let belief_id = Uuid::new_v4();
        let belief = item(belief_id, "claude-code", 9, MemoryKind::Fact, "host=alpha");

        let mut stale = item(
            Uuid::new_v4(),
            "codex",
            4,
            MemoryKind::Correction,
            "host=beta",
        );
        stale.supersedes = vec![belief_id];
        stale.correction_meta = Some(CorrectionMetadata {
            corrects_id: Some(belief_id),
            source_turn: Some("t-3".into()),
            captured_by: Some(SchemaCaptureSource::Manual),
            confidence: Some(0.7),
        });

        let winner = pick_correction_winner(&belief, &stale);
        assert_eq!(winner.id, belief.id, "stale correction must not win");
    }

    #[test]
    fn correction_without_supersede_does_not_win() {
        // Even with higher Lamport, correction loses if it doesn't claim the
        // prior belief — the chain must be explicit.
        let belief_id = Uuid::new_v4();
        let belief = item(belief_id, "claude-code", 1, MemoryKind::Fact, "x");
        let unrelated = item(Uuid::new_v4(), "codex", 99, MemoryKind::Correction, "y");
        let winner = pick_correction_winner(&belief, &unrelated);
        assert_eq!(winner.id, belief.id);
    }
}

/// Phase C4.10 — cross-harness resolver.
///
/// Picks the winning record between a prior belief and a candidate
/// correction. The correction wins iff it explicitly supersedes the prior
/// (`corrects_id == prior.id`) AND its Lamport `version` is strictly greater
/// — Lamport tiebreak on `source_agent` if equal versions are encountered.
pub fn pick_correction_winner<'a>(
    prior: &'a memd_schema::MemoryItem,
    correction: &'a memd_schema::MemoryItem,
) -> &'a memd_schema::MemoryItem {
    let supersedes_prior = correction
        .correction_meta
        .as_ref()
        .and_then(|meta| meta.corrects_id)
        == Some(prior.id)
        || correction.supersedes.iter().any(|id| *id == prior.id);
    if !supersedes_prior {
        return prior;
    }
    match correction.version.cmp(&prior.version) {
        std::cmp::Ordering::Greater => correction,
        std::cmp::Ordering::Equal => {
            // Deterministic tiebreak: lexicographic on source_agent.
            let p = prior.source_agent.as_deref().unwrap_or("");
            let c = correction.source_agent.as_deref().unwrap_or("");
            if c >= p { correction } else { prior }
        }
        std::cmp::Ordering::Less => prior,
    }
}
