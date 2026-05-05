//! V11/A11 project-aware wake isolation.
//!
//! The core invariant is intentionally small: a wake for `(project_id,
//! workspace_id)` can only hydrate records with the same pair. Same project in
//! a different workspace and same workspace in a different project are both
//! hidden.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProjectScope {
    pub project_id: String,
    pub workspace_id: String,
}

impl ProjectScope {
    pub fn new(project_id: impl Into<String>, workspace_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            workspace_id: workspace_id.into(),
        }
    }

    pub fn contains(&self, project_id: &str, workspace_id: &str) -> bool {
        self.project_id == project_id && self.workspace_id == workspace_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScopedMemoryRecord {
    pub id: String,
    pub project_id: String,
    pub workspace_id: String,
    pub content: String,
    pub kind: String,
    pub source_turn_id: Option<String>,
    pub compacted: bool,
    pub correction_active: bool,
    pub token_count: usize,
}

impl ScopedMemoryRecord {
    pub fn scoped(
        id: impl Into<String>,
        scope: &ProjectScope,
        kind: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        let content = content.into();
        Self {
            id: id.into(),
            project_id: scope.project_id.clone(),
            workspace_id: scope.workspace_id.clone(),
            content,
            kind: kind.into(),
            source_turn_id: None,
            compacted: false,
            correction_active: false,
            token_count: 0,
        }
    }

    pub fn with_tokens(mut self, token_count: usize) -> Self {
        self.token_count = token_count;
        self
    }

    pub fn active_correction(mut self, source_turn_id: impl Into<String>) -> Self {
        self.source_turn_id = Some(source_turn_id.into());
        self.correction_active = true;
        self
    }

    pub fn compacted(mut self) -> Self {
        self.compacted = true;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectWake {
    pub scope: ProjectScope,
    pub hydrated: Vec<ScopedMemoryRecord>,
    pub hidden_foreign_count: usize,
    pub focus_restored: bool,
}

pub fn filter_project_records(
    scope: &ProjectScope,
    records: &[ScopedMemoryRecord],
) -> Vec<ScopedMemoryRecord> {
    records
        .iter()
        .filter(|record| scope.contains(&record.project_id, &record.workspace_id))
        .cloned()
        .collect()
}

pub fn build_project_wake(scope: ProjectScope, records: &[ScopedMemoryRecord]) -> ProjectWake {
    let hydrated = filter_project_records(&scope, records);
    let hidden_foreign_count = records.len().saturating_sub(hydrated.len());
    let focus_restored = hydrated
        .iter()
        .any(|record| record.kind == "focus" || record.content.contains("Focus:"));
    ProjectWake {
        scope,
        hydrated,
        hidden_foreign_count,
        focus_restored,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_workspace_different_project_is_hidden() {
        let scope_a = ProjectScope::new("project-a", "workspace-1");
        let scope_b = ProjectScope::new("project-b", "workspace-1");
        let records = vec![
            ScopedMemoryRecord::scoped("a1", &scope_a, "focus", "Focus: finish A"),
            ScopedMemoryRecord::scoped("b1", &scope_b, "fact", "B uses gRPC"),
        ];

        let wake = build_project_wake(scope_a, &records);
        assert_eq!(wake.hydrated.len(), 1);
        assert_eq!(wake.hydrated[0].id, "a1");
        assert_eq!(wake.hidden_foreign_count, 1);
        assert!(wake.focus_restored);
    }

    #[test]
    fn same_project_different_workspace_is_hidden() {
        let scope = ProjectScope::new("project-a", "workspace-1");
        let other_workspace = ProjectScope::new("project-a", "workspace-2");
        let records = vec![
            ScopedMemoryRecord::scoped("a1", &scope, "fact", "A cache Redis"),
            ScopedMemoryRecord::scoped("a2", &other_workspace, "fact", "A secret elsewhere"),
        ];

        let filtered = filter_project_records(&scope, &records);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].workspace_id, "workspace-1");
    }
}
