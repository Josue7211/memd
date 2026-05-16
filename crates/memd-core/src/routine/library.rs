use anyhow::{Context, bail};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RoutineStatus {
    Candidate,
    Active,
    Deprecated,
    Merged,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutineRecord {
    pub id: Uuid,
    pub name: String,
    pub summary: String,
    pub steps: Vec<String>,
    pub status: RoutineStatus,
    #[serde(default)]
    pub source_ids: Vec<Uuid>,
    #[serde(default)]
    pub replaces_id: Option<Uuid>,
    #[serde(default)]
    pub deprecation_reason: Option<String>,
    #[serde(default)]
    pub updated_by: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutineLibrary {
    pub workspace_id: String,
    pub routines: Vec<RoutineRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutineExport {
    pub schema: String,
    pub workspace_id: String,
    pub routines: Vec<RoutineRecord>,
    pub checksum: String,
}

impl RoutineRecord {
    pub fn new(
        name: impl Into<String>,
        summary: impl Into<String>,
        steps: Vec<String>,
        status: RoutineStatus,
        workspace_id: impl Into<String>,
    ) -> anyhow::Result<Self> {
        let now = Utc::now();
        let name = normalize_nonempty("name", name.into())?;
        let summary = normalize_nonempty("summary", summary.into())?;
        validate_steps(status, &steps)?;
        Ok(Self {
            id: Uuid::new_v4(),
            name,
            summary,
            steps,
            status,
            source_ids: Vec::new(),
            replaces_id: None,
            deprecation_reason: None,
            updated_by: None,
            project_id: None,
            workspace_id: Some(workspace_id.into()),
            created_at: now,
            updated_at: now,
        })
    }
}

impl RoutineLibrary {
    pub fn new(workspace_id: impl Into<String>) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            routines: Vec::new(),
        }
    }

    pub fn with_routines(workspace_id: impl Into<String>, routines: Vec<RoutineRecord>) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            routines,
        }
    }

    pub fn browse(&self, status: Option<RoutineStatus>) -> Vec<RoutineRecord> {
        self.routines
            .iter()
            .filter(|routine| match status {
                Some(want) => routine.status == want,
                None => matches!(
                    routine.status,
                    RoutineStatus::Candidate | RoutineStatus::Active
                ),
            })
            .cloned()
            .collect()
    }

    pub fn browse_all(&self) -> Vec<RoutineRecord> {
        self.routines.clone()
    }

    pub fn push(&mut self, mut routine: RoutineRecord) -> anyhow::Result<Uuid> {
        self.ensure_unique_name(&routine.name, None)?;
        if routine.workspace_id.is_none() {
            routine.workspace_id = Some(self.workspace_id.clone());
        }
        validate_steps(routine.status, &routine.steps)?;
        let id = routine.id;
        self.routines.push(routine);
        Ok(id)
    }

    pub fn edit(
        &mut self,
        id: Uuid,
        name: impl Into<String>,
        summary: impl Into<String>,
        steps: Vec<String>,
        updated_by: impl Into<String>,
    ) -> anyhow::Result<RoutineRecord> {
        let existing = self.find(id)?.clone();
        let name = normalize_nonempty("name", name.into())?;
        let summary = normalize_nonempty("summary", summary.into())?;
        self.ensure_unique_name(&name, Some(id))?;
        validate_steps(RoutineStatus::Active, &steps)?;
        let now = Utc::now();
        let revision = RoutineRecord {
            id: Uuid::new_v4(),
            name,
            summary,
            steps,
            status: RoutineStatus::Active,
            source_ids: existing.source_ids.clone(),
            replaces_id: Some(existing.id),
            deprecation_reason: None,
            updated_by: Some(updated_by.into()),
            project_id: existing.project_id.clone(),
            workspace_id: existing
                .workspace_id
                .clone()
                .or_else(|| Some(self.workspace_id.clone())),
            created_at: now,
            updated_at: now,
        };
        self.routines.push(revision.clone());
        Ok(revision)
    }

    pub fn merge(
        &mut self,
        input_ids: &[Uuid],
        name: impl Into<String>,
        summary: impl Into<String>,
        updated_by: impl Into<String>,
    ) -> anyhow::Result<RoutineRecord> {
        if input_ids.len() < 2 {
            bail!("merge requires at least two routines");
        }
        let unique: BTreeSet<Uuid> = input_ids.iter().copied().collect();
        if unique.len() != input_ids.len() {
            bail!("merge input ids must be unique");
        }
        let mut steps = Vec::new();
        for id in input_ids {
            let routine = self.find(*id)?;
            if routine.status == RoutineStatus::Deprecated {
                bail!("cannot merge deprecated routine {id}");
            }
            for step in &routine.steps {
                if !steps.contains(step) {
                    steps.push(step.clone());
                }
            }
        }
        let output = self.compose_record(input_ids, name, summary, steps, updated_by)?;
        for routine in &mut self.routines {
            if unique.contains(&routine.id) {
                routine.status = RoutineStatus::Merged;
                routine.updated_at = Utc::now();
            }
        }
        self.routines.push(output.clone());
        Ok(output)
    }

    pub fn compose(
        &mut self,
        left: Uuid,
        right: Uuid,
        name: impl Into<String>,
        summary: impl Into<String>,
        updated_by: impl Into<String>,
    ) -> anyhow::Result<RoutineRecord> {
        let mut steps = self.find(left)?.steps.clone();
        for step in &self.find(right)?.steps {
            if !steps.contains(step) {
                steps.push(step.clone());
            }
        }
        let output = self.compose_record(&[left, right], name, summary, steps, updated_by)?;
        self.routines.push(output.clone());
        Ok(output)
    }

    pub fn deprecate(
        &mut self,
        id: Uuid,
        reason: impl Into<String>,
        updated_by: impl Into<String>,
    ) -> anyhow::Result<RoutineRecord> {
        let reason = normalize_nonempty("reason", reason.into())?;
        let routine = self.find_mut(id)?;
        routine.status = RoutineStatus::Deprecated;
        routine.deprecation_reason = Some(reason);
        routine.updated_by = Some(updated_by.into());
        routine.updated_at = Utc::now();
        Ok(routine.clone())
    }

    pub fn inherit(global: &RoutineLibrary, project: &RoutineLibrary) -> RoutineLibrary {
        let mut by_name = BTreeMap::<String, RoutineRecord>::new();
        for routine in &global.routines {
            by_name.insert(normalize_name_key(&routine.name), routine.clone());
        }
        for routine in &project.routines {
            by_name.insert(normalize_name_key(&routine.name), routine.clone());
        }
        RoutineLibrary {
            workspace_id: project.workspace_id.clone(),
            routines: by_name.into_values().collect(),
        }
    }

    pub fn export_workspace(&self) -> anyhow::Result<RoutineExport> {
        let mut export = RoutineExport {
            schema: "memd.routine-library.v1".to_string(),
            workspace_id: self.workspace_id.clone(),
            routines: self.routines.clone(),
            checksum: String::new(),
        };
        export.checksum = routine_export_checksum(&export)?;
        Ok(export)
    }

    pub fn import_workspace(export: &RoutineExport) -> anyhow::Result<RoutineLibrary> {
        let expected = routine_export_checksum(export)?;
        if expected != export.checksum {
            bail!("routine export checksum mismatch");
        }
        Ok(RoutineLibrary::with_routines(
            export.workspace_id.clone(),
            export.routines.clone(),
        ))
    }

    fn compose_record(
        &self,
        input_ids: &[Uuid],
        name: impl Into<String>,
        summary: impl Into<String>,
        steps: Vec<String>,
        updated_by: impl Into<String>,
    ) -> anyhow::Result<RoutineRecord> {
        let name = normalize_nonempty("name", name.into())?;
        let summary = normalize_nonempty("summary", summary.into())?;
        self.ensure_unique_name(&name, None)?;
        validate_steps(RoutineStatus::Active, &steps)?;
        let now = Utc::now();
        Ok(RoutineRecord {
            id: Uuid::new_v4(),
            name,
            summary,
            steps,
            status: RoutineStatus::Active,
            source_ids: input_ids.to_vec(),
            replaces_id: None,
            deprecation_reason: None,
            updated_by: Some(updated_by.into()),
            project_id: None,
            workspace_id: Some(self.workspace_id.clone()),
            created_at: now,
            updated_at: now,
        })
    }

    fn find(&self, id: Uuid) -> anyhow::Result<&RoutineRecord> {
        self.routines
            .iter()
            .find(|routine| routine.id == id)
            .with_context(|| format!("routine not found: {id}"))
    }

    fn find_mut(&mut self, id: Uuid) -> anyhow::Result<&mut RoutineRecord> {
        self.routines
            .iter_mut()
            .find(|routine| routine.id == id)
            .with_context(|| format!("routine not found: {id}"))
    }

    fn ensure_unique_name(&self, name: &str, except_id: Option<Uuid>) -> anyhow::Result<()> {
        let key = normalize_name_key(name);
        let duplicate = self.routines.iter().any(|routine| {
            Some(routine.id) != except_id && normalize_name_key(&routine.name) == key
        });
        if duplicate {
            bail!("routine name already exists: {name}");
        }
        Ok(())
    }
}

pub fn routine_export_checksum(export: &RoutineExport) -> anyhow::Result<String> {
    let mut clone = export.clone();
    clone.checksum.clear();
    let bytes = serde_json::to_vec(&clone).context("serialize routine export")?;
    Ok(to_hex(Sha256::digest(bytes)))
}

fn validate_steps(status: RoutineStatus, steps: &[String]) -> anyhow::Result<()> {
    if status == RoutineStatus::Active && steps.iter().all(|step| step.trim().is_empty()) {
        bail!("active routine requires non-empty steps");
    }
    Ok(())
}

fn normalize_nonempty(field: &str, value: String) -> anyhow::Result<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        bail!("{field} must not be empty");
    }
    Ok(value)
}

fn normalize_name_key(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn to_hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn routine(name: &str, steps: &[&str]) -> RoutineRecord {
        RoutineRecord::new(
            name,
            format!("{name} summary"),
            steps.iter().map(|step| (*step).to_string()).collect(),
            RoutineStatus::Active,
            "ws-1",
        )
        .unwrap()
    }

    #[test]
    fn browse_hides_deprecated_and_merged_by_default() {
        let mut old = routine("old", &["old step"]);
        old.status = RoutineStatus::Deprecated;
        let mut merged = routine("merged", &["merged step"]);
        merged.status = RoutineStatus::Merged;
        let lib = RoutineLibrary::with_routines(
            "ws-1",
            vec![routine("lint", &["cargo clippy"]), old, merged],
        );
        let names: Vec<_> = lib.browse(None).into_iter().map(|r| r.name).collect();
        assert_eq!(names, vec!["lint"]);
        assert_eq!(lib.browse_all().len(), 3);
    }

    #[test]
    fn edit_creates_revision_with_replaces_id() {
        let mut lib = RoutineLibrary::new("ws-1");
        let id = lib.push(routine("lint", &["cargo check"])).unwrap();
        let edited = lib
            .edit(
                id,
                "lint",
                "strict lint",
                vec!["cargo clippy".into()],
                "codex",
            )
            .unwrap();
        assert_eq!(edited.replaces_id, Some(id));
        assert_eq!(edited.status, RoutineStatus::Active);
        assert_eq!(lib.routines.len(), 2);
    }

    #[test]
    fn merge_marks_inputs_and_records_sources() {
        let mut lib = RoutineLibrary::new("ws-1");
        let lint = lib.push(routine("lint", &["cargo clippy"])).unwrap();
        let fmt = lib.push(routine("format", &["cargo fmt"])).unwrap();
        let merged = lib
            .merge(&[lint, fmt], "lint-format", "both", "codex")
            .unwrap();
        assert_eq!(merged.source_ids, vec![lint, fmt]);
        assert_eq!(merged.steps, vec!["cargo clippy", "cargo fmt"]);
        assert_eq!(lib.browse(Some(RoutineStatus::Merged)).len(), 2);
    }

    #[test]
    fn project_inheritance_overrides_by_name() {
        let global =
            RoutineLibrary::with_routines("global", vec![routine("lint", &["cargo check"])]);
        let project =
            RoutineLibrary::with_routines("project", vec![routine("lint", &["cargo clippy"])]);
        let inherited = RoutineLibrary::inherit(&global, &project);
        assert_eq!(inherited.routines.len(), 1);
        assert_eq!(inherited.routines[0].steps, vec!["cargo clippy"]);
    }

    #[test]
    fn export_import_detects_tamper() {
        let lib = RoutineLibrary::with_routines("ws-1", vec![routine("lint", &["cargo clippy"])]);
        let mut export = lib.export_workspace().unwrap();
        assert_eq!(
            RoutineLibrary::import_workspace(&export).unwrap().routines[0].name,
            "lint"
        );
        export.routines[0].name = "tampered".to_string();
        let err = RoutineLibrary::import_workspace(&export).unwrap_err();
        assert!(err.to_string().contains("checksum mismatch"));
    }
}
