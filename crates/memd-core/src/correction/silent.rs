//! V11/C11 silent correction detector.
//!
//! A flag is raised when a user repeatedly rephrases a question about a prior
//! answer without using an explicit correction marker. Direct confirmations do
//! not count.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PriorAnswer {
    pub answer_id: String,
    pub source_turn_id: String,
    pub project_id: String,
    pub topic_terms: Vec<String>,
    pub answer_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserTurnObservation {
    pub turn_id: String,
    pub project_id: String,
    pub text: String,
    pub observed_at_ms: u64,
    pub suggestion_ignored: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SilentCorrectionFlag {
    pub correction_flag_id: String,
    pub memory_item_id: String,
    pub project_id: String,
    pub rephrasing_count: usize,
    pub ignore_count: usize,
    pub flagged_at_ms: u64,
    pub detection_latency_ms: u64,
    pub trigger_turn_ids: Vec<String>,
}

pub fn detect_silent_correction(
    prior: &PriorAnswer,
    turns: &[UserTurnObservation],
    now_ms: u64,
) -> Option<SilentCorrectionFlag> {
    let mut trigger_turn_ids = Vec::new();
    let mut first_trigger_ms = None;
    let mut ignore_count = 0usize;

    for turn in turns
        .iter()
        .filter(|turn| turn.project_id == prior.project_id)
        .filter(|turn| !is_confirmation(&turn.text))
    {
        if turn.suggestion_ignored {
            ignore_count += 1;
        }
        if looks_like_rephrase(prior, &turn.text) {
            trigger_turn_ids.push(turn.turn_id.clone());
            first_trigger_ms.get_or_insert(turn.observed_at_ms);
        }
    }

    if trigger_turn_ids.len() < 2 && ignore_count < 2 {
        return None;
    }

    let first = first_trigger_ms.unwrap_or(now_ms);
    Some(SilentCorrectionFlag {
        correction_flag_id: format!("flag-{}-{}", prior.answer_id, trigger_turn_ids.len()),
        memory_item_id: prior.answer_id.clone(),
        project_id: prior.project_id.clone(),
        rephrasing_count: trigger_turn_ids.len(),
        ignore_count,
        flagged_at_ms: now_ms,
        detection_latency_ms: now_ms.saturating_sub(first),
        trigger_turn_ids,
    })
}

fn looks_like_rephrase(prior: &PriorAnswer, text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let questionish = lower.contains('?')
        || lower.contains("what")
        || lower.contains("which")
        || lower.contains("remind")
        || lower.contains("wait");
    questionish
        && prior
            .topic_terms
            .iter()
            .any(|term| lower.contains(&term.to_ascii_lowercase()))
}

fn is_confirmation(text: &str) -> bool {
    let lower = text.trim().to_ascii_lowercase();
    lower.starts_with("correct")
        || lower.starts_with("yes")
        || lower.starts_with("yep")
        || lower.starts_with("sounds right")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prior() -> PriorAnswer {
        PriorAnswer {
            answer_id: "t4-answer".into(),
            source_turn_id: "t4".into(),
            project_id: "project-a".into(),
            topic_terms: vec!["cache".into(), "backend".into(), "protocol".into()],
            answer_text: "cache is Redis".into(),
        }
    }

    #[test]
    fn two_rephrases_trigger_flag_under_latency_budget() {
        let turns = vec![
            UserTurnObservation {
                turn_id: "t17".into(),
                project_id: "project-a".into(),
                text: "Wait, what's the cache backend?".into(),
                observed_at_ms: 1_000,
                suggestion_ignored: false,
            },
            UserTurnObservation {
                turn_id: "t18".into(),
                project_id: "project-a".into(),
                text: "Remind me the cache protocol.".into(),
                observed_at_ms: 1_500,
                suggestion_ignored: false,
            },
        ];

        let flag = detect_silent_correction(&prior(), &turns, 1_900).expect("flag");
        assert_eq!(flag.rephrasing_count, 2);
        assert_eq!(flag.trigger_turn_ids, vec!["t17", "t18"]);
        assert!(flag.detection_latency_ms <= 1_000);
    }

    #[test]
    fn single_confirmation_does_not_flag() {
        let turns = vec![UserTurnObservation {
            turn_id: "t11".into(),
            project_id: "project-a".into(),
            text: "Correct, gRPC.".into(),
            observed_at_ms: 1_000,
            suggestion_ignored: false,
        }];

        assert!(detect_silent_correction(&prior(), &turns, 1_200).is_none());
    }
}
