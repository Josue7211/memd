//! V10/B10 cross-session correction auto-apply.
//!
//! Auto-apply does not invent new truth. It takes an already captured
//! correction and decides whether a reopened or sibling session needs that
//! correction surfaced before stale context can win.

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CorrectionRecord {
    pub correction_id: String,
    pub supersedes_claim_id: String,
    pub origin_session: String,
    pub source_turn_id: String,
    pub corrected_content: String,
    pub confidence: f32,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionContext {
    pub session_id: String,
    pub visible_claim_ids: Vec<String>,
    pub already_applied_correction_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutoApplyDecision {
    pub correction_id: String,
    pub target_session: String,
    pub supersedes_claim_id: String,
    pub apply: bool,
    pub reason: String,
    pub confidence: f32,
}

pub fn auto_apply_corrections(
    corrections: &[CorrectionRecord],
    session: &SessionContext,
) -> Vec<AutoApplyDecision> {
    let visible = set(&session.visible_claim_ids);
    let applied = set(&session.already_applied_correction_ids);

    corrections
        .iter()
        .filter(|correction| correction.confidence >= 0.65)
        .filter(|correction| visible.contains(&correction.supersedes_claim_id))
        .filter(|correction| !applied.contains(&correction.correction_id))
        .map(|correction| AutoApplyDecision {
            correction_id: correction.correction_id.clone(),
            target_session: session.session_id.clone(),
            supersedes_claim_id: correction.supersedes_claim_id.clone(),
            apply: true,
            reason: if correction.origin_session == session.session_id {
                "reopened session contains stale claim".to_string()
            } else {
                "cross-session stale claim auto-corrected".to_string()
            },
            confidence: correction.confidence,
        })
        .collect()
}

pub fn append_auto_apply_log(
    bundle_root: &Path,
    decisions: &[AutoApplyDecision],
) -> anyhow::Result<()> {
    if decisions.is_empty() {
        return Ok(());
    }
    let log_path = bundle_root
        .join("logs")
        .join("auto-applied-corrections.ndjson");
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create auto-apply log dir {}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("open auto-apply log {}", log_path.display()))?;
    for decision in decisions {
        writeln!(file, "{}", serde_json::to_string(decision)?)?;
    }
    Ok(())
}

fn set(values: &[String]) -> BTreeSet<String> {
    values.iter().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn correction(id: &str, claim: &str, origin: &str, confidence: f32) -> CorrectionRecord {
        CorrectionRecord {
            correction_id: id.to_string(),
            supersedes_claim_id: claim.to_string(),
            origin_session: origin.to_string(),
            source_turn_id: "t5".to_string(),
            corrected_content: "corrected value".to_string(),
            confidence,
            tags: vec!["correction".to_string(), "v10-b10".to_string()],
            created_at: Utc::now(),
        }
    }

    #[test]
    fn auto_applies_visible_unapplied_cross_session_correction() {
        let corrections = vec![correction("c1", "claim-1", "s1", 0.9)];
        let session = SessionContext {
            session_id: "s2".into(),
            visible_claim_ids: vec!["claim-1".into()],
            already_applied_correction_ids: vec![],
        };
        let decisions = auto_apply_corrections(&corrections, &session);
        assert_eq!(decisions.len(), 1);
        assert!(decisions[0].apply);
        assert_eq!(decisions[0].target_session, "s2");
        assert!(decisions[0].reason.contains("cross-session"));
    }

    #[test]
    fn does_not_reapply_or_apply_low_confidence() {
        let corrections = vec![
            correction("c1", "claim-1", "s1", 0.9),
            correction("c2", "claim-1", "s1", 0.4),
        ];
        let session = SessionContext {
            session_id: "s2".into(),
            visible_claim_ids: vec!["claim-1".into()],
            already_applied_correction_ids: vec!["c1".into()],
        };
        assert!(auto_apply_corrections(&corrections, &session).is_empty());
    }

    #[test]
    fn writes_ndjson_evidence_log() {
        let dir = tempfile::tempdir().expect("tmp");
        let decisions = vec![AutoApplyDecision {
            correction_id: "c1".into(),
            target_session: "s2".into(),
            supersedes_claim_id: "claim-1".into(),
            apply: true,
            reason: "cross-session stale claim auto-corrected".into(),
            confidence: 0.9,
        }];
        append_auto_apply_log(dir.path(), &decisions).expect("write log");
        let body = fs::read_to_string(dir.path().join("logs/auto-applied-corrections.ndjson"))
            .expect("read log");
        assert!(body.contains("\"correction_id\":\"c1\""));
        assert!(body.ends_with('\n'));
    }
}
