use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    CapabilityRegistry, build_bundle_capability_registry_with_home,
    command_catalog::{CommandCatalogEntry, build_command_catalog},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MigrationAudit {
    pub(crate) generated_at: DateTime<Utc>,
    pub(crate) bundle_root: String,
    pub(crate) project_root: Option<String>,
    pub(crate) command_count: usize,
    pub(crate) ready_count: usize,
    pub(crate) partial_count: usize,
    pub(crate) missing_count: usize,
    pub(crate) broken_count: usize,
    pub(crate) commands: Vec<MigrationAuditEntry>,
    pub(crate) capability_registry: CapabilityRegistry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MigrationAuditEntry {
    pub(crate) name: String,
    pub(crate) surface: String,
    pub(crate) kind: String,
    pub(crate) ownership: String,
    pub(crate) role: String,
    pub(crate) compatibility_flags: Vec<String>,
    pub(crate) status: String,
    pub(crate) present: Vec<String>,
    pub(crate) missing: Vec<String>,
    pub(crate) broken: Vec<String>,
    pub(crate) notes: Vec<String>,
}

pub(crate) fn build_migration_audit(
    bundle_root: &Path,
    project_root: Option<&Path>,
    home_override: Option<&Path>,
) -> MigrationAudit {
    let catalog = build_command_catalog(bundle_root);
    let home_root = home_override.map(PathBuf::from).or_else(crate::home_dir);
    let capability_registry =
        build_bundle_capability_registry_with_home(project_root, home_root.as_deref());
    let commands = catalog
        .commands
        .iter()
        .map(|command| {
            evaluate_command_compatibility(command, bundle_root, project_root, &capability_registry)
        })
        .collect::<Vec<_>>();

    let ready_count = commands
        .iter()
        .filter(|entry| entry.status == "ready")
        .count();
    let partial_count = commands
        .iter()
        .filter(|entry| entry.status == "partial")
        .count();
    let missing_count = commands
        .iter()
        .filter(|entry| entry.status == "missing")
        .count();
    let broken_count = commands
        .iter()
        .filter(|entry| entry.status == "broken")
        .count();

    MigrationAudit {
        generated_at: Utc::now(),
        bundle_root: bundle_root.display().to_string(),
        project_root: project_root.map(|path| path.display().to_string()),
        command_count: catalog.command_count,
        ready_count,
        partial_count,
        missing_count,
        broken_count,
        commands,
        capability_registry,
    }
}

fn evaluate_command_compatibility(
    command: &CommandCatalogEntry,
    bundle_root: &Path,
    project_root: Option<&Path>,
    capability_registry: &CapabilityRegistry,
) -> MigrationAuditEntry {
    let mut present = Vec::new();
    let mut missing = Vec::new();
    let mut broken = Vec::new();
    let mut notes = Vec::new();

    for flag in &command.compatibility_flags {
        match flag.as_str() {
            "memd-binary" => present.push(flag.clone()),
            "bundle-root-present" => {
                if bundle_root.exists() {
                    present.push(flag.clone());
                } else {
                    missing.push(flag.clone());
                }
            }
            "claude-import-bridge" => {
                let path = bundle_root.join("agents").join("CLAUDE_IMPORTS.md");
                if path.is_file() {
                    present.push(flag.clone());
                } else {
                    missing.push(flag.clone());
                }
            }
            "claude-project-bridge" => {
                let Some(project_root) = project_root else {
                    missing.push(flag.clone());
                    notes.push("project root not provided".to_string());
                    continue;
                };
                let claude = project_root.join("CLAUDE.md");
                if !claude.is_file() {
                    missing.push(flag.clone());
                    continue;
                }
                let Ok(contents) = std::fs::read_to_string(&claude) else {
                    broken.push(flag.clone());
                    notes.push(format!("failed to read {}", claude.display()));
                    continue;
                };
                if contents.contains("@.memd/agents/CLAUDE_IMPORTS.md") {
                    present.push(flag.clone());
                } else {
                    missing.push(flag.clone());
                    notes.push("CLAUDE.md does not import CLAUDE_IMPORTS.md".to_string());
                }
            }
            "codex-skill-installed" => {
                let skill_name = command.name.trim_start_matches('$').to_string();
                if has_codex_skill(capability_registry, &skill_name) {
                    present.push(flag.clone());
                } else {
                    missing.push(flag.clone());
                }
            }
            "launcher-script-present" => {
                let Some(path) = command.path.as_deref() else {
                    broken.push(flag.clone());
                    notes.push("launcher path missing from catalog".to_string());
                    continue;
                };
                let path = Path::new(path);
                if path.is_file() {
                    present.push(flag.clone());
                } else {
                    missing.push(flag.clone());
                }
            }
            "launcher-script-executable" => {
                let Some(path) = command.path.as_deref() else {
                    broken.push(flag.clone());
                    notes.push("launcher path missing from catalog".to_string());
                    continue;
                };
                let path = Path::new(path);
                if launcher_is_executable(path) {
                    present.push(flag.clone());
                } else if path.is_file() {
                    broken.push(flag.clone());
                    notes.push(format!("{} is not executable", path.display()));
                } else {
                    missing.push(flag.clone());
                }
            }
            other => {
                missing.push(other.to_string());
                notes.push(format!("unknown compatibility flag {other}"));
            }
        }
    }

    let mut status = if broken.is_empty() && missing.is_empty() {
        "ready"
    } else if broken.is_empty() {
        "partial"
    } else {
        "broken"
    };

    if command.role == "bridge-surface" && command.name == "/memory" {
        let bundle_bridge = bundle_root.join("agents").join("CLAUDE_IMPORTS.md");
        if bundle_bridge.is_file() && missing.iter().any(|flag| flag == "claude-project-bridge") {
            status = "partial";
        }
    }

    MigrationAuditEntry {
        name: command.name.clone(),
        surface: command.surface.clone(),
        kind: command.kind.clone(),
        ownership: command.ownership.clone(),
        role: command.role.clone(),
        compatibility_flags: command.compatibility_flags.clone(),
        status: status.to_string(),
        present,
        missing,
        broken,
        notes,
    }
}

fn has_codex_skill(registry: &CapabilityRegistry, skill_name: &str) -> bool {
    registry.capabilities.iter().any(|capability| {
        capability.harness == "codex"
            && (capability.kind == "skill" || capability.kind == "skill-bridge")
            && capability.name == skill_name
    })
}

#[cfg(unix)]
fn launcher_is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    std::fs::metadata(path)
        .map(|meta| meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn launcher_is_executable(path: &Path) -> bool {
    path.is_file()
}
