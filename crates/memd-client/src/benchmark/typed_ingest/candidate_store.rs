//! V6 / B6 — candidate store. Persists semantic candidates with
//! `stage = candidate` plus the originating `EpisodicProvenance` and
//! the B6 distill sidecar (prompt_version, judge_model, etc.).
//!
//! Contract: `docs/contracts/semantic-distillation.md` §6.
//!
//! The store is a JSONL file keyed by bench-run; each line is one
//! candidate record. C6 promotion reads this file as input.

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::distiller::{CandidateKind, DistillCandidate};
use super::episodic::EpisodicProvenance;

/// Stage discriminant for B6 candidates — always `"candidate"`.
/// Promotion to `"canonical"` is C6's job.
pub(crate) const CANDIDATE_STAGE: &str = "candidate";

/// Single persisted candidate. Pairs a `DistillCandidate` with the
/// originating turn provenance and the B6 distill sidecar.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct CandidateRecord {
    pub stage: String,
    pub kind: CandidateKind,
    pub content: String,
    pub provenance: EpisodicProvenance,
    pub distill: DistillSidecar,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct DistillSidecar {
    pub prompt_version: String,
    pub judge_model: String,
    pub confidence: f32,
    pub rationale: String,
    pub source_turn_ids: Vec<String>,
}

impl CandidateRecord {
    pub(crate) fn from_parts(
        candidate: DistillCandidate,
        provenance: EpisodicProvenance,
        prompt_version: &str,
        judge_model: &str,
    ) -> Self {
        let DistillCandidate {
            kind,
            content,
            confidence,
            source_turn_ids,
            rationale,
        } = candidate;
        Self {
            stage: CANDIDATE_STAGE.to_string(),
            kind,
            content,
            provenance,
            distill: DistillSidecar {
                prompt_version: prompt_version.to_string(),
                judge_model: judge_model.to_string(),
                confidence,
                rationale,
                source_turn_ids,
            },
        }
    }
}

/// Append candidates to a JSONL file. Creates parent dirs and the file
/// itself on first write. One JSON object per line.
pub(crate) fn append_candidates(path: &Path, records: &[CandidateRecord]) -> Result<()> {
    use std::io::Write as _;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create candidate store parent dir {}", parent.display()))?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open candidate store {}", path.display()))?;
    for r in records {
        let line = serde_json::to_string(r).context("serialise candidate record")?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
    }
    Ok(())
}

/// Read all candidates from a JSONL file. Order preserved.
pub(crate) fn read_candidates(path: &Path) -> Result<Vec<CandidateRecord>> {
    let body = std::fs::read_to_string(path)
        .with_context(|| format!("read candidate store {}", path.display()))?;
    let mut out = Vec::new();
    for (i, line) in body.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let r: CandidateRecord = serde_json::from_str(line)
            .with_context(|| format!("parse candidate line {}", i + 1))?;
        out.push(r);
    }
    Ok(out)
}
