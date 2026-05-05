//! V11/B11 compaction-aware recall.

use crate::isolation::{ProjectScope, ScopedMemoryRecord, filter_project_records};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompactionSnapshot {
    pub cut_id: String,
    pub project_id: String,
    pub workspace_id: String,
    pub focus: Option<String>,
    pub records: Vec<ScopedMemoryRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecoveredContext {
    pub scope: ProjectScope,
    pub cut_ids: Vec<String>,
    pub records: Vec<ScopedMemoryRecord>,
    pub corrections_recovered: usize,
    pub truncated: bool,
}

pub fn recover_project_context(
    scope: &ProjectScope,
    snapshots: &[CompactionSnapshot],
    token_budget: usize,
) -> RecoveredContext {
    let mut cut_ids = Vec::new();
    let mut records = Vec::new();
    let mut used = 0usize;
    let mut truncated = false;

    for snapshot in snapshots
        .iter()
        .filter(|snapshot| scope.contains(&snapshot.project_id, &snapshot.workspace_id))
    {
        cut_ids.push(snapshot.cut_id.clone());
        for record in filter_project_records(scope, &snapshot.records) {
            let cost = record.token_count.max(1);
            if used + cost > token_budget {
                truncated = true;
                continue;
            }
            used += cost;
            records.push(record);
        }
    }

    let corrections_recovered = records
        .iter()
        .filter(|record| record.correction_active)
        .count();
    RecoveredContext {
        scope: scope.clone(),
        cut_ids,
        records,
        corrections_recovered,
        truncated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_active_correction_after_project_round_trip() {
        let a = ProjectScope::new("project-a", "workspace-1");
        let b = ProjectScope::new("project-b", "workspace-1");
        let snapshots = vec![
            CompactionSnapshot {
                cut_id: "cut-a".into(),
                project_id: a.project_id.clone(),
                workspace_id: a.workspace_id.clone(),
                focus: Some("finish project A redesign".into()),
                records: vec![
                    ScopedMemoryRecord::scoped("a-correction", &a, "correction", "cache is Redis")
                        .active_correction("t4")
                        .with_tokens(12)
                        .compacted(),
                ],
            },
            CompactionSnapshot {
                cut_id: "cut-b".into(),
                project_id: b.project_id.clone(),
                workspace_id: b.workspace_id.clone(),
                focus: Some("debug project B API".into()),
                records: vec![
                    ScopedMemoryRecord::scoped("b-fact", &b, "fact", "API uses gRPC")
                        .with_tokens(10),
                ],
            },
        ];

        let recovered = recover_project_context(&a, &snapshots, 100);
        assert_eq!(recovered.cut_ids, vec!["cut-a"]);
        assert_eq!(recovered.corrections_recovered, 1);
        assert!(recovered.records[0].content.contains("Redis"));
        assert!(!recovered.truncated);
    }

    #[test]
    fn marks_truncation_when_budget_cannot_fit_project_records() {
        let a = ProjectScope::new("project-a", "workspace-1");
        let snapshots = vec![CompactionSnapshot {
            cut_id: "cut-a".into(),
            project_id: a.project_id.clone(),
            workspace_id: a.workspace_id.clone(),
            focus: None,
            records: vec![
                ScopedMemoryRecord::scoped("a1", &a, "fact", "large").with_tokens(50),
                ScopedMemoryRecord::scoped("a2", &a, "fact", "larger").with_tokens(50),
            ],
        }];

        let recovered = recover_project_context(&a, &snapshots, 60);
        assert_eq!(recovered.records.len(), 1);
        assert!(recovered.truncated);
    }
}
