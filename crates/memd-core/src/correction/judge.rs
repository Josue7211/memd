//! LLM-judge client (Phase C4.3) — placeholder, see follow-up commit.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JudgeDecision {
    Confirmed,
    Rejected,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JudgeVerdict {
    pub decision: JudgeDecision,
    pub confidence: f32,
    pub rationale: String,
    pub cache_hit: bool,
    pub cost_usd: f32,
}
