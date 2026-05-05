//! V11/E11 per-turn cost ledger.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const TOKENS_PER_CENT: usize = 3000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostTarget {
    pub project_id: String,
    pub per_turn_cents: f32,
}

impl CostTarget {
    pub fn token_budget(&self, default_budget: usize) -> usize {
        if self.per_turn_cents <= 0.0 {
            return default_budget;
        }
        ((self.per_turn_cents * TOKENS_PER_CENT as f32).floor() as usize).max(1)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostLedgerEntry {
    pub cost_ledger_id: String,
    pub turn_seq: u64,
    pub project_id: String,
    pub cost_cents: f32,
    pub token_count: usize,
    pub timestamp: DateTime<Utc>,
}

pub fn ledger_entry(
    turn_seq: u64,
    project_id: impl Into<String>,
    token_count: usize,
    timestamp: DateTime<Utc>,
) -> CostLedgerEntry {
    let project_id = project_id.into();
    CostLedgerEntry {
        cost_ledger_id: format!("cost-{project_id}-{turn_seq}"),
        turn_seq,
        project_id,
        cost_cents: token_count as f32 / TOKENS_PER_CENT as f32,
        token_count,
        timestamp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_cent_target_maps_to_v11_wake_budget() {
        let target = CostTarget {
            project_id: "project-a".into(),
            per_turn_cents: 0.5,
        };
        assert_eq!(target.token_budget(4000), 1500);
    }

    #[test]
    fn ledger_cost_is_derived_from_tokens() {
        let entry = ledger_entry(3, "project-a", 1500, Utc::now());
        assert_eq!(entry.cost_ledger_id, "cost-project-a-3");
        assert!((entry.cost_cents - 0.5).abs() < f32::EPSILON);
    }
}
