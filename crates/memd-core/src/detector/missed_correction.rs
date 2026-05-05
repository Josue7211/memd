//! V10/A10 missed-correction detector.
//!
//! This catches the simple but high-value case where an agent states a claim
//! and a later user turn contradicts it. The output is intentionally small:
//! enough to queue a re-ingest candidate without pretending to resolve truth
//! semantically.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const STOPWORDS: &[&str] = &[
    "the", "and", "for", "but", "not", "are", "was", "were", "you", "your", "this", "that",
    "with", "from", "have", "has", "had", "been", "they", "them", "their", "our", "his",
    "her", "its", "into", "about", "then", "than", "will", "would", "should", "could",
    "agent", "user",
];

const CORRECTION_MARKERS: &[&str] = &[
    "actually",
    "correction:",
    "wrong",
    "incorrect",
    "not true",
    "that's stale",
    "that is stale",
    "you missed",
    "you forgot",
    "no,",
    "no ",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranscriptTurn {
    pub id: String,
    pub speaker: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdviceClaim {
    pub claim_id: String,
    pub turn_id: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MissedCorrection {
    pub claim_id: String,
    pub claim_turn_id: String,
    pub correction_turn_id: String,
    pub corrected_text: String,
    pub overlap_tokens: Vec<String>,
    pub confidence: f32,
    pub reingest: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReingestCandidate {
    pub supersedes_claim_id: String,
    pub source_turn_id: String,
    pub content: String,
    pub tags: Vec<String>,
}

pub fn detect_missed_corrections(turns: &[TranscriptTurn]) -> Vec<MissedCorrection> {
    let mut claims = Vec::new();
    let mut out = Vec::new();

    for turn in turns {
        if is_agent(&turn.speaker) {
            for claim in extract_agent_claims(turn) {
                claims.push(claim);
            }
            continue;
        }

        if !is_user(&turn.speaker) || !looks_like_correction(&turn.text) {
            continue;
        }

        let user_tokens = token_set(&turn.text);
        let Some((claim, overlap)) = claims
            .iter()
            .rev()
            .filter_map(|claim| {
                let claim_tokens = token_set(&claim.content);
                let overlap = claim_tokens
                    .intersection(&user_tokens)
                    .cloned()
                    .collect::<Vec<_>>();
                (overlap.len() >= 2).then_some((claim, overlap))
            })
            .max_by_key(|(_, overlap)| overlap.len())
        else {
            continue;
        };

        let confidence = (0.55 + overlap.len() as f32 * 0.08).min(0.95);
        out.push(MissedCorrection {
            claim_id: claim.claim_id.clone(),
            claim_turn_id: claim.turn_id.clone(),
            correction_turn_id: turn.id.clone(),
            corrected_text: turn.text.trim().to_string(),
            overlap_tokens: overlap,
            confidence,
            reingest: true,
        });
    }

    out
}

pub fn build_reingest_candidates(
    corrections: &[MissedCorrection],
) -> Vec<ReingestCandidate> {
    corrections
        .iter()
        .filter(|correction| correction.reingest)
        .map(|correction| ReingestCandidate {
            supersedes_claim_id: correction.claim_id.clone(),
            source_turn_id: correction.correction_turn_id.clone(),
            content: correction.corrected_text.clone(),
            tags: vec![
                "correction".to_string(),
                "missed-correction".to_string(),
                "v10-a10".to_string(),
            ],
        })
        .collect()
}

fn extract_agent_claims(turn: &TranscriptTurn) -> Vec<AdviceClaim> {
    turn.text
        .split(['.', '\n'])
        .map(str::trim)
        .filter(|line| line.len() >= 16)
        .enumerate()
        .map(|(idx, content)| AdviceClaim {
            claim_id: format!("{}#{idx}", turn.id),
            turn_id: turn.id.clone(),
            content: content.to_string(),
        })
        .collect()
}

fn is_agent(speaker: &str) -> bool {
    let speaker = speaker.to_ascii_lowercase();
    matches!(speaker.as_str(), "assistant" | "agent" | "codex" | "claude")
}

fn is_user(speaker: &str) -> bool {
    speaker.eq_ignore_ascii_case("user") || speaker.eq_ignore_ascii_case("human")
}

fn looks_like_correction(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    CORRECTION_MARKERS.iter().any(|marker| text.contains(marker))
}

fn token_set(text: &str) -> BTreeSet<String> {
    text.to_ascii_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|token| token.len() >= 3 && !STOPWORDS.contains(token))
        .map(ToString::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn(id: &str, speaker: &str, text: &str) -> TranscriptTurn {
        TranscriptTurn {
            id: id.to_string(),
            speaker: speaker.to_string(),
            text: text.to_string(),
        }
    }

    #[test]
    fn detects_user_correction_of_prior_agent_claim() {
        let turns = vec![
            turn("t1", "assistant", "The production database is postgres on alpha."),
            turn("t2", "user", "Actually, the production database is sqlite on beta now."),
        ];
        let found = detect_missed_corrections(&turns);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].claim_id, "t1#0");
        assert_eq!(found[0].correction_turn_id, "t2");
        assert!(found[0].confidence >= 0.7);
        assert!(found[0].reingest);
    }

    #[test]
    fn builds_reingest_candidate_with_supersede_pointer() {
        let correction = MissedCorrection {
            claim_id: "t1#0".into(),
            claim_turn_id: "t1".into(),
            correction_turn_id: "t2".into(),
            corrected_text: "Actually, use sqlite.".into(),
            overlap_tokens: vec!["sqlite".into(), "production".into()],
            confidence: 0.8,
            reingest: true,
        };
        let candidates = build_reingest_candidates(&[correction]);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].supersedes_claim_id, "t1#0");
        assert!(candidates[0].tags.iter().any(|tag| tag == "v10-a10"));
    }

    #[test]
    fn ignores_neutral_user_turns() {
        let turns = vec![
            turn("t1", "assistant", "The deploy command is make release."),
            turn("t2", "user", "Sounds good, proceed."),
        ];
        assert!(detect_missed_corrections(&turns).is_empty());
    }
}
