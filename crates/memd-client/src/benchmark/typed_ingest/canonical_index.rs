//! V6 / C6 — canonical lane index.
//!
//! Separate JSONL store for canonical records (`stage=canonical`).
//! Reads from / writes to `.memd/benchmarks/public/canonical/<bench-id>.jsonl`
//! by convention; the runtime layer wraps these helpers behind the
//! V6-closed dispatch.
//!
//! Contract: `docs/contracts/canonical-promotion.md` §6. Provenance
//! shape is the E5 auditor target (`source_turn`, `captured_by`,
//! `captured_at`).

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::candidate_store::CandidateRecord;
use super::distiller::CandidateKind;
use super::promotion::{PROMOTION_RULE_VERSION, PromotionAccepted};

/// Stage discriminant — always `"canonical"`.
pub(crate) const CANONICAL_STAGE: &str = "canonical";

/// `captured_by` value the E5 auditor sees on every C6-promoted record.
pub(crate) const CAPTURED_BY: &str = "c6-promotion/v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct CanonicalRecord {
    pub stage: String,
    pub kind: CandidateKind,
    pub content: String,
    pub content_hash: String,
    pub provenance: CanonicalProvenance,
    pub rule: CanonicalRule,
    pub candidates: Vec<CanonicalCandidateLink>,
}

/// Provenance shaped to pass the E5 auditor (`audit_record`):
/// requires `source_turn`, `captured_by`, `captured_at`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct CanonicalProvenance {
    pub source_turn: String,
    pub captured_by: String,
    pub captured_at: String,
    pub chain: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct CanonicalRule {
    pub version: String,
    pub corroboration_count: usize,
    pub min_confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct CanonicalCandidateLink {
    pub prompt_version: String,
    pub judge_model: String,
    pub confidence: f32,
    pub source_turn_ids: Vec<String>,
}

impl CanonicalRecord {
    /// Build a canonical record from an accepted promotion plus the
    /// originating candidate group (used to populate the candidate
    /// links and pick the earliest captured_at as `captured_at`).
    pub(crate) fn from_promotion(
        accepted: &PromotionAccepted,
        group: &[&CandidateRecord],
        captured_at: &str,
    ) -> Self {
        let mut chain = accepted.source_turn_ids.clone();
        chain.sort();
        let source_turn = chain.first().cloned().unwrap_or_default();

        let candidates: Vec<CanonicalCandidateLink> = group
            .iter()
            .map(|c| CanonicalCandidateLink {
                prompt_version: c.distill.prompt_version.clone(),
                judge_model: c.distill.judge_model.clone(),
                confidence: c.distill.confidence,
                source_turn_ids: c.distill.source_turn_ids.clone(),
            })
            .collect();

        Self {
            stage: CANONICAL_STAGE.to_string(),
            kind: accepted.kind,
            content: accepted.content.clone(),
            content_hash: accepted.content_hash.clone(),
            provenance: CanonicalProvenance {
                source_turn,
                captured_by: CAPTURED_BY.to_string(),
                captured_at: captured_at.to_string(),
                chain,
            },
            rule: CanonicalRule {
                version: accepted.rule_version.clone(),
                corroboration_count: accepted.corroboration_count,
                min_confidence: accepted.min_confidence,
            },
            candidates,
        }
    }

    /// Project the record into the JSON shape expected by the E5
    /// auditor (`audit_record`). Reused as the C6→E5 bridge so the
    /// auditor stays untouched.
    pub(crate) fn to_audit_json(&self) -> serde_json::Value {
        json!({
            "id": self.content_hash,
            "stage": self.stage,
            "content": self.content,
            "provenance": {
                "source_turn": self.provenance.source_turn,
                "captured_by": self.provenance.captured_by,
                "captured_at": self.provenance.captured_at,
                "chain": self.provenance.chain,
            }
        })
    }
}

/// Append canonical records to a JSONL file. Creates parent dirs and
/// the file itself on first write.
pub(crate) fn append_canonical(path: &Path, records: &[CanonicalRecord]) -> Result<()> {
    use std::io::Write as _;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create canonical index parent dir {}", parent.display()))?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open canonical index {}", path.display()))?;
    for r in records {
        let line = serde_json::to_string(r).context("serialise canonical record")?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
    }
    Ok(())
}

/// Read all canonical records from a JSONL file. Order preserved.
/// Filters to `stage = "canonical"` defensively — any other stage in
/// this file is a bug, but the filter mirrors B6's read_candidates
/// semantics.
pub(crate) fn read_canonical(path: &Path) -> Result<Vec<CanonicalRecord>> {
    let body = std::fs::read_to_string(path)
        .with_context(|| format!("read canonical index {}", path.display()))?;
    let mut out = Vec::new();
    for (i, line) in body.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let r: CanonicalRecord = serde_json::from_str(line)
            .with_context(|| format!("parse canonical line {}", i + 1))?;
        if r.stage == CANONICAL_STAGE {
            out.push(r);
        }
    }
    Ok(out)
}

/// Surface the rule version for telemetry headers.
pub(crate) fn current_rule_version() -> &'static str {
    PROMOTION_RULE_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmark::substrate::provenance_auditor::audit_record;
    use crate::benchmark::typed_ingest::candidate_store::DistillSidecar;
    use crate::benchmark::typed_ingest::episodic::EpisodicProvenance;

    fn mk_candidate(content: &str, sess: &str, idx: u32) -> CandidateRecord {
        CandidateRecord {
            stage: "candidate".to_string(),
            kind: CandidateKind::Fact,
            content: content.to_string(),
            provenance: EpisodicProvenance {
                bench_id: "longmemeval".to_string(),
                session_id: sess.to_string(),
                turn_index: idx,
                speaker: "user".to_string(),
                source_hash: "0".repeat(64),
                captured_at: "2026-04-27T00:00:00Z".to_string(),
            },
            distill: DistillSidecar {
                prompt_version: "semantic-distillation/v1".to_string(),
                judge_model: "gpt-5.4".to_string(),
                confidence: 0.9,
                rationale: "r".to_string(),
                source_turn_ids: vec![format!("{sess}::{idx}")],
            },
        }
    }

    #[test]
    fn canonical_record_audit_passes_e5() {
        let group = vec![mk_candidate("X", "s", 0), mk_candidate("X", "s", 4)];
        let group_refs: Vec<&CandidateRecord> = group.iter().collect();
        let accepted = PromotionAccepted {
            kind: CandidateKind::Fact,
            content: "X".to_string(),
            content_hash: "h".to_string(),
            corroboration_count: 2,
            min_confidence: 0.9,
            source_turn_ids: vec!["s::0".to_string(), "s::4".to_string()],
            rule_version: PROMOTION_RULE_VERSION.to_string(),
        };
        let rec = CanonicalRecord::from_promotion(&accepted, &group_refs, "2026-04-27T00:00:00Z");
        let outcome = audit_record(&rec.to_audit_json());
        assert!(outcome.passed, "missing: {:?}", outcome.missing_fields);
        assert!(outcome.chain_length >= 1);
    }
}
