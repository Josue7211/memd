//! Correction lane: detector, judge, and shared records (Phase C4).
//!
//! Auto-detection of in-session corrections is conservative by design.
//! Detector runs deterministic regexes; LLM-judge confirms marginal candidates.

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
