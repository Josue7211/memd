use super::*;
use chrono::Utc;

pub(crate) fn render_skill_policy_query_summary(
    receipts: &SkillPolicyApplyReceiptsResponse,
    activations: &SkillPolicyActivationEntriesResponse,
    follow: bool,
) -> String {
    let mut markdown = String::new();
    markdown.push_str("## Skill Policy Query\n\n");
    markdown.push_str(&format!(
        "- receipts: {}\n- activations: {}\n",
        receipts.receipts.len(),
        activations.activations.len()
    ));
    if !receipts.receipts.is_empty() {
        markdown.push_str("\n### Receipts\n\n");
        for receipt in receipts.receipts.iter().take(if follow { 12 } else { 6 }) {
            markdown.push_str(&format!(
                "- {} applied={} skipped={} runtime_defaulted={} queue={}",
                receipt.id.chars().take(8).collect::<String>(),
                receipt.applied_count,
                receipt.skipped_count,
                receipt.runtime_defaulted,
                receipt.source_queue_path
            ));
            if let Some(project) = receipt.project.as_deref() {
                markdown.push_str(&format!(" project={}", project));
            }
            if let Some(namespace) = receipt.namespace.as_deref() {
                markdown.push_str(&format!(" namespace={}", namespace));
            }
            if let Some(workspace) = receipt.workspace.as_deref() {
                markdown.push_str(&format!(" workspace={}", workspace));
            }
            markdown.push('\n');
        }
    }
    if !activations.activations.is_empty() {
        markdown.push_str("\n### Activations\n\n");
        for entry in activations
            .activations
            .iter()
            .take(if follow { 12 } else { 6 })
        {
            markdown.push_str(&format!(
                "- {} / {} / {} action={} sandbox={} risk={:.2}",
                entry.receipt_id.chars().take(8).collect::<String>(),
                entry.record.harness,
                entry.record.name,
                entry.record.activation,
                entry.record.sandbox,
                entry.record.sandbox_risk
            ));
            markdown.push_str(&format!(" queue={}", entry.source_queue_path));
            if let Some(target) = entry.record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            markdown.push('\n');
        }
    }
    markdown
}

pub(crate) fn apply_capability_bridges() -> CapabilityBridgeRegistry {
    let mut actions = Vec::new();
    let Some(home) = home_dir() else {
        return CapabilityBridgeRegistry {
            generated_at: Utc::now(),
            actions,
        };
    };

    let claude_settings = home.join(".claude").join("settings.json");
    let codex_skill_root = home.join(".agents").join("skills");
    let opencode_modern_plugins = home.join(".config").join("opencode").join("plugins");
    let opencode_legacy_plugins = home.join(".opencode").join("plugins");
    let plugin_records = collect_enabled_plugin_cache_records(&claude_settings, &home);
    for record in plugin_records {
        let source_skills = record.cache_root.join("skills");
        if source_skills.is_dir() {
            let target = codex_skill_root.join(&record.plugin_name);
            actions.push(ensure_directory_skill_bridge(
                "codex",
                &record.plugin_name,
                &source_skills,
                &target,
            ));
        }

        let source_opencode_plugins = record.cache_root.join(".opencode").join("plugins");
        if source_opencode_plugins.is_dir() {
            for target_root in [&opencode_modern_plugins, &opencode_legacy_plugins] {
                let target = target_root.join(&record.plugin_name);
                actions.push(ensure_directory_skill_bridge(
                    "opencode",
                    &record.plugin_name,
                    &source_opencode_plugins,
                    &target,
                ));
            }
        }
    }

    CapabilityBridgeRegistry {
        generated_at: Utc::now(),
        actions,
    }
}

pub(crate) fn detect_capability_bridges() -> CapabilityBridgeRegistry {
    let mut actions = Vec::new();
    let Some(home) = home_dir() else {
        return CapabilityBridgeRegistry {
            generated_at: Utc::now(),
            actions,
        };
    };

    let claude_settings = home.join(".claude").join("settings.json");
    let codex_skill_root = home.join(".agents").join("skills");
    let opencode_modern_plugins = home.join(".config").join("opencode").join("plugins");
    let opencode_legacy_plugins = home.join(".opencode").join("plugins");
    let plugin_records = collect_enabled_plugin_cache_records(&claude_settings, &home);
    for record in plugin_records {
        let source_skills = record.cache_root.join("skills");
        if source_skills.is_dir() {
            let target = codex_skill_root.join(&record.plugin_name);
            actions.push(inspect_directory_skill_bridge(
                "codex",
                &record.plugin_name,
                &source_skills,
                &target,
            ));
        }

        let source_opencode_plugins = record.cache_root.join(".opencode").join("plugins");
        if source_opencode_plugins.is_dir() {
            for target_root in [&opencode_modern_plugins, &opencode_legacy_plugins] {
                let target = target_root.join(&record.plugin_name);
                actions.push(inspect_directory_skill_bridge(
                    "opencode",
                    &record.plugin_name,
                    &source_opencode_plugins,
                    &target,
                ));
            }
        }
    }

    CapabilityBridgeRegistry {
        generated_at: Utc::now(),
        actions,
    }
}

pub(crate) fn collect_enabled_plugin_cache_records(
    settings_path: &Path,
    home: &Path,
) -> Vec<PluginCacheRecord> {
    let mut records = Vec::new();
    let Ok(raw) = fs::read_to_string(settings_path) else {
        return records;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return records;
    };
    let Some(enabled) = json
        .get("enabledPlugins")
        .and_then(|value| value.as_object())
    else {
        return records;
    };

    for (plugin_id, value) in enabled {
        if !value.as_bool().unwrap_or(false) {
            continue;
        }
        let (plugin_name, marketplace) = parse_marketplace_plugin_id(plugin_id);
        let Some(cache_root) = latest_cached_plugin_root(
            &home.join(".codex").join("plugins").join("cache"),
            marketplace.as_deref().unwrap_or("unknown"),
            &plugin_name,
        ) else {
            continue;
        };
        records.push(PluginCacheRecord {
            plugin_name,
            cache_root,
        });
    }

    records
}

pub(crate) fn ensure_directory_skill_bridge(
    harness: &str,
    capability: &str,
    source: &Path,
    target: &Path,
) -> CapabilityBridgeAction {
    let source_path = source.display().to_string();
    let target_path = target.display().to_string();
    let mut notes = Vec::new();

    let parent = match target.parent() {
        Some(parent) => parent,
        None => {
            return CapabilityBridgeAction {
                harness: harness.to_string(),
                capability: capability.to_string(),
                status: "blocked".to_string(),
                source_path,
                target_path,
                notes: vec!["target has no parent directory".to_string()],
            };
        }
    };

    if let Err(err) = fs::create_dir_all(parent) {
        return CapabilityBridgeAction {
            harness: harness.to_string(),
            capability: capability.to_string(),
            status: "blocked".to_string(),
            source_path,
            target_path,
            notes: vec![format!("failed to create target parent: {err}")],
        };
    }

    if let Ok(existing) = fs::symlink_metadata(target) {
        if existing.file_type().is_symlink() {
            if let Ok(current) = fs::read_link(target) {
                if current == source {
                    return CapabilityBridgeAction {
                        harness: harness.to_string(),
                        capability: capability.to_string(),
                        status: "already-bridged".to_string(),
                        source_path,
                        target_path,
                        notes: vec!["bridge already points at the current source".to_string()],
                    };
                }
            }
            if let Err(err) = fs::remove_file(target) {
                return CapabilityBridgeAction {
                    harness: harness.to_string(),
                    capability: capability.to_string(),
                    status: "blocked".to_string(),
                    source_path,
                    target_path,
                    notes: vec![format!("failed to replace existing symlink: {err}")],
                };
            }
            notes.push("replaced stale symlink bridge".to_string());
        } else {
            return CapabilityBridgeAction {
                harness: harness.to_string(),
                capability: capability.to_string(),
                status: "blocked".to_string(),
                source_path,
                target_path,
                notes: vec!["target already exists and is not a symlink".to_string()],
            };
        }
    }

    match create_symlink(source, target) {
        Ok(()) => {
            notes.push("created native skill bridge".to_string());
            CapabilityBridgeAction {
                harness: harness.to_string(),
                capability: capability.to_string(),
                status: "bridged".to_string(),
                source_path,
                target_path,
                notes,
            }
        }
        Err(err) => CapabilityBridgeAction {
            harness: harness.to_string(),
            capability: capability.to_string(),
            status: "blocked".to_string(),
            source_path,
            target_path,
            notes: vec![format!("failed to create symlink bridge: {err}")],
        },
    }
}

pub(crate) fn inspect_directory_skill_bridge(
    harness: &str,
    capability: &str,
    source: &Path,
    target: &Path,
) -> CapabilityBridgeAction {
    let source_path = source.display().to_string();
    let target_path = target.display().to_string();

    let Some(parent) = target.parent() else {
        return CapabilityBridgeAction {
            harness: harness.to_string(),
            capability: capability.to_string(),
            status: "blocked".to_string(),
            source_path,
            target_path,
            notes: vec!["target has no parent directory".to_string()],
        };
    };

    if !parent.exists() {
        return CapabilityBridgeAction {
            harness: harness.to_string(),
            capability: capability.to_string(),
            status: "blocked".to_string(),
            source_path,
            target_path,
            notes: vec!["target parent directory is missing".to_string()],
        };
    }

    if let Ok(existing) = fs::symlink_metadata(target) {
        if existing.file_type().is_symlink() {
            if let Ok(current) = fs::read_link(target) {
                if current == source {
                    return CapabilityBridgeAction {
                        harness: harness.to_string(),
                        capability: capability.to_string(),
                        status: "already-bridged".to_string(),
                        source_path,
                        target_path,
                        notes: vec!["bridge already points at the current source".to_string()],
                    };
                }
            }
            return CapabilityBridgeAction {
                harness: harness.to_string(),
                capability: capability.to_string(),
                status: "available".to_string(),
                source_path,
                target_path,
                notes: vec!["stale bridge can be refreshed by explicit init".to_string()],
            };
        }
        return CapabilityBridgeAction {
            harness: harness.to_string(),
            capability: capability.to_string(),
            status: "blocked".to_string(),
            source_path,
            target_path,
            notes: vec!["target already exists and is not a symlink".to_string()],
        };
    }

    CapabilityBridgeAction {
        harness: harness.to_string(),
        capability: capability.to_string(),
        status: "available".to_string(),
        source_path,
        target_path,
        notes: vec!["bridge target can be created by explicit init".to_string()],
    }
}

#[cfg(unix)]
pub(crate) fn create_symlink(source: &Path, target: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(source, target)
}

#[cfg(windows)]
pub(crate) fn create_symlink(source: &Path, target: &Path) -> io::Result<()> {
    std::os::windows::fs::symlink_dir(source, target)
}

#[derive(Debug, Clone)]
pub(crate) struct PluginCacheRecord {
    plugin_name: String,
    cache_root: PathBuf,
}

pub(crate) fn collect_project_bootstrap_sources(project_root: &Path) -> Vec<PathBuf> {
    let mut sources = Vec::new();
    let candidates = [
        "CLAUDE.md",
        ".claude/CLAUDE.md",
        ".agents/CLAUDE.md",
        "DESIGN.md",
        ".claude/DESIGN.md",
        ".agents/DESIGN.md",
        "AGENTS.md",
        "TEAMS.md",
        "MEMORY.md",
        "SOUL.md",
        "USER.md",
        "IDENTITY.md",
        "TOOLS.md",
        "BOOTSTRAP.md",
        "HEARTBEAT.md",
        "README.md",
        "CONTRIBUTING.md",
        "ROADMAP.md",
        "docs/core/setup.md",
        "docs/policy/config.md",
        "docs/reference/infra-facts.md",
        "docs/policy/release-process.md",
        "docs/reference/maintainer-workflow.md",
        ".planning/STATE.md",
        ".planning/PROJECT.md",
        ".planning/ROADMAP.md",
        ".planning/codebase/ARCHITECTURE.md",
        ".planning/codebase/STRUCTURE.md",
    ];

    for candidate in candidates {
        let path = project_root.join(candidate);
        if path.is_file() {
            sources.push(path);
        }
    }

    let claude_project_memory = claude_project_memory_path(project_root);
    if claude_project_memory.is_file() {
        sources.push(claude_project_memory);
    }

    sources.extend(collect_memory_dir_sources(project_root));
    sources.extend(collect_design_dir_sources(project_root));

    sources
}

pub(crate) fn collect_user_harness_bootstrap_sources(project_root: Option<&Path>) -> Vec<PathBuf> {
    let Some(home) = home_dir() else {
        return Vec::new();
    };

    let mut sources = Vec::new();
    let codex_root = home.join(".codex");
    sources.extend(collect_named_file_sources(
        &codex_root,
        &[
            "AGENTS.md",
            "TEAMS.md",
            "MEMORY.md",
            "USER.md",
            "IDENTITY.md",
            "SOUL.md",
            "TOOLS.md",
            "BOOTSTRAP.md",
            "HEARTBEAT.md",
            "config.toml",
        ],
    ));
    sources.extend(collect_relative_file_sources(
        &codex_root,
        &[
            "skills/memd/SKILL.md",
            "skills/memd-init/SKILL.md",
            "skills/memd-reload/SKILL.md",
            "skills/dream/SKILL.md",
            "skills/autodream/SKILL.md",
            "skills/gsd-autonomous/SKILL.md",
            "skills/gsd-map-codebase/SKILL.md",
        ],
    ));

    for harness_root in detect_claude_family_harness_roots(&home) {
        sources.extend(collect_named_file_sources(
            &harness_root.root,
            &[
                "AGENTS.md",
                "TEAMS.md",
                "MEMORY.md",
                "USER.md",
                "IDENTITY.md",
                "SOUL.md",
                "TOOLS.md",
                "BOOTSTRAP.md",
                "HEARTBEAT.md",
                "settings.json",
            ],
        ));
        sources.extend(collect_relative_file_sources(
            &harness_root.root,
            &[
                "hooks/gsd-session-context.js",
                "hooks/memd-session-context.js",
            ],
        ));
    }
    if let Some(project_root) = project_root {
        let claude_project_memory = claude_project_memory_path(project_root);
        if claude_project_memory.is_file() {
            sources.push(claude_project_memory);
        }
    }

    let openclaw_root = home.join(".openclaw").join("workspace");
    sources.extend(collect_named_file_sources(
        &openclaw_root,
        &[
            "AGENTS.md",
            "TEAMS.md",
            "MEMORY.md",
            "USER.md",
            "IDENTITY.md",
            "SOUL.md",
            "TOOLS.md",
            "BOOTSTRAP.md",
            "HEARTBEAT.md",
        ],
    ));

    let opencode_root = home.join(".config").join("opencode");
    sources.extend(collect_named_file_sources(
        &opencode_root,
        &[
            "AGENTS.md",
            "TEAMS.md",
            "MEMORY.md",
            "settings.json",
            "opencode.json",
        ],
    ));
    sources.extend(collect_relative_file_sources(
        &opencode_root,
        &[
            "plugins/memd-plugin.mjs",
            "command/memd.md",
            "command/gsd-autonomous.md",
            "command/gsd-map-codebase.md",
        ],
    ));

    let legacy_opencode_root = home.join(".opencode");
    sources.extend(collect_named_file_sources(
        &legacy_opencode_root,
        &["AGENTS.md", "TEAMS.md", "MEMORY.md"],
    ));

    let claw_config_root = home.join(".config").join("claw");
    sources.extend(collect_named_file_sources(
        &claw_config_root,
        &[
            "settings.json",
            "AGENTS.md",
            "TEAMS.md",
            "MEMORY.md",
            "CLAUDE.md",
        ],
    ));
    sources.extend(collect_relative_file_sources(
        &claw_config_root,
        &[
            "skills/memd/SKILL.md",
            "skills/memd-reload/SKILL.md",
            "skills/memd-init/SKILL.md",
        ],
    ));

    let claw_home_root = home.join(".claw");
    sources.extend(collect_named_file_sources(
        &claw_home_root,
        &[
            "settings.json",
            "AGENTS.md",
            "TEAMS.md",
            "MEMORY.md",
            "CLAUDE.md",
        ],
    ));
    sources.extend(collect_relative_file_sources(
        &claw_home_root,
        &[
            "skills/memd/SKILL.md",
            "skills/memd-reload/SKILL.md",
            "skills/memd-init/SKILL.md",
        ],
    ));

    sources
}

#[derive(Debug, Clone)]
pub(crate) struct HarnessRoot {
    pub(crate) harness: String,
    pub(crate) root: PathBuf,
}

pub(crate) fn detect_claude_family_harness_roots(home: &Path) -> Vec<HarnessRoot> {
    let mut roots = Vec::new();
    let primary = home.join(".claude");
    if primary.is_dir() {
        roots.push(HarnessRoot {
            harness: "claude".to_string(),
            root: primary,
        });
    }

    let Ok(entries) = fs::read_dir(home) else {
        return roots;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name == ".claude" || name == ".codex" || name == ".openclaw" || name == ".opencode" {
            continue;
        }
        if !looks_like_claude_family_dir(name) {
            continue;
        }
        if !path.join("settings.json").is_file() {
            continue;
        }
        roots.push(HarnessRoot {
            harness: name.trim_start_matches('.').to_string(),
            root: path,
        });
    }

    roots.sort_by(|a, b| a.harness.cmp(&b.harness).then(a.root.cmp(&b.root)));
    roots
}

pub(crate) fn looks_like_claude_family_dir(name: &str) -> bool {
    let normalized = name.trim_start_matches('.').to_ascii_lowercase();
    normalized.contains("claude") || normalized.contains("claw")
}

pub(crate) fn collect_named_file_sources(root: &Path, names: &[&str]) -> Vec<PathBuf> {
    names
        .iter()
        .map(|name| root.join(name))
        .filter(|path| path.is_file())
        .collect()
}

pub(crate) fn collect_relative_file_sources(root: &Path, paths: &[&str]) -> Vec<PathBuf> {
    paths
        .iter()
        .map(|relative| root.join(relative))
        .filter(|path| path.is_file())
        .collect()
}

pub(crate) fn collect_memory_dir_sources(project_root: &Path) -> Vec<PathBuf> {
    let memory_dir = project_root.join("memory");
    let mut sources = Vec::new();
    let Ok(entries) = fs::read_dir(&memory_dir) else {
        return sources;
    };

    let mut entries = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .map(|ext| {
                    matches!(
                        ext.to_ascii_lowercase().as_str(),
                        "md" | "txt" | "json" | "yaml" | "yml"
                    )
                })
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    entries.sort();
    sources.extend(entries.into_iter().take(6));
    sources
}

pub(crate) fn collect_design_dir_sources(project_root: &Path) -> Vec<PathBuf> {
    let design_dir = project_root.join("design");
    let mut sources = Vec::new();
    let Ok(entries) = fs::read_dir(&design_dir) else {
        return sources;
    };

    let mut entries = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .map(|ext| {
                    matches!(
                        ext.to_ascii_lowercase().as_str(),
                        "md" | "txt" | "json" | "yaml" | "yml"
                    )
                })
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();

    entries.sort();
    sources.extend(entries.into_iter().take(6));
    sources
}

pub(crate) fn claude_project_memory_path(project_root: &Path) -> PathBuf {
    let slug = project_root.to_string_lossy().replace('/', "-");
    home_dir()
        .map(|home| {
            home.join(".claude")
                .join("projects")
                .join(slug)
                .join("memory")
        })
        .unwrap_or_else(|| PathBuf::from("."))
        .join("MEMORY.md")
}

pub(crate) fn display_bootstrap_source_path(path: &Path, project_root: Option<&Path>) -> String {
    if let Some(project_root) = project_root
        && let Ok(relative) = path.strip_prefix(project_root)
    {
        return relative.display().to_string();
    }

    if let Some(home) = home_dir()
        && let Ok(relative) = path.strip_prefix(&home)
    {
        return format!("~/{}", relative.display());
    }

    path.display().to_string()
}

pub(crate) fn default_heartbeat_model() -> String {
    "llama-desktop/qwen".to_string()
}

pub(crate) fn default_bundle_session() -> String {
    format!(
        "session-{}",
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    )
}

pub(crate) fn default_bundle_tab_id() -> Option<String> {
    std::env::var("MEMD_TAB_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(crate) fn compose_agent_identity(agent: &str, session: Option<&str>) -> String {
    let agent = agent.trim();
    let session = session.map(str::trim).filter(|value| !value.is_empty());
    match session {
        Some(session) => format!("{agent}@{session}"),
        None => agent.to_string(),
    }
}

pub(crate) fn project_hive_group(project: Option<&str>) -> Option<String> {
    let project = project?.trim();
    if project.is_empty() {
        None
    } else {
        let mut slug = String::new();
        let mut last_dash = false;
        for ch in project.chars() {
            let normalized = ch.to_ascii_lowercase();
            if normalized.is_ascii_alphanumeric() {
                slug.push(normalized);
                last_dash = false;
            } else if !last_dash {
                slug.push('-');
                last_dash = true;
            }
        }
        Some(format!("project:{}", slug.trim_matches('-')))
    }
}

pub(crate) fn effective_hive_groups(
    hive_groups: Vec<String>,
    project: Option<&str>,
) -> Vec<String> {
    let mut groups = hive_groups;
    if let Some(project_group) = project_hive_group(project) {
        groups.push(project_group);
    }
    groups.sort();
    groups.dedup();
    groups
}

#[derive(Debug, Clone)]
pub(crate) struct HiveProfileDefaults {
    pub(crate) hive_system: Option<String>,
    pub(crate) hive_role: Option<String>,
    pub(crate) capabilities: Vec<String>,
    pub(crate) hive_groups: Vec<String>,
    pub(crate) hive_group_goal: Option<String>,
    pub(crate) hive_project_enabled: bool,
    pub(crate) hive_project_anchor: Option<String>,
    pub(crate) hive_project_joined_at: Option<DateTime<Utc>>,
    pub(crate) authority: Option<String>,
}

pub(crate) fn default_hive_profile(agent: &str) -> HiveProfileDefaults {
    let normalized = agent.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "agent-shell" => HiveProfileDefaults {
            hive_system: Some("agent-shell".to_string()),
            hive_role: Some("runtime-shell".to_string()),
            capabilities: vec![
                "shell".to_string(),
                "exec".to_string(),
                "workspace".to_string(),
            ],
            hive_groups: vec!["runtime-core".to_string(), "dependency-owners".to_string()],
            hive_group_goal: Some(
                "stabilize runtime execution and dependency health across active agent sessions"
                    .to_string(),
            ),
            hive_project_enabled: false,
            hive_project_anchor: None,
            hive_project_joined_at: None,
            authority: Some("worker".to_string()),
        },
        "agent-secrets" => HiveProfileDefaults {
            hive_system: Some("agent-secrets".to_string()),
            hive_role: Some("secret-broker".to_string()),
            capabilities: vec![
                "secrets".to_string(),
                "auth".to_string(),
                "policy".to_string(),
            ],
            hive_groups: vec!["runtime-core".to_string(), "dependency-owners".to_string()],
            hive_group_goal: Some(
                "keep secret access and auth dependencies reliable for the active product stack"
                    .to_string(),
            ),
            hive_project_enabled: false,
            hive_project_anchor: None,
            hive_project_joined_at: None,
            authority: Some("restricted".to_string()),
        },
        "claw-control" => HiveProfileDefaults {
            hive_system: Some("claw-control".to_string()),
            hive_role: Some("orchestrator".to_string()),
            capabilities: vec![
                "control".to_string(),
                "routing".to_string(),
                "coordination".to_string(),
            ],
            hive_groups: vec!["openclaw-stack".to_string(), "control-plane".to_string()],
            hive_group_goal: Some(
                "coordinate the OpenClaw stack so hives converge on the proper product-level fix"
                    .to_string(),
            ),
            hive_project_enabled: false,
            hive_project_anchor: None,
            hive_project_joined_at: None,
            authority: Some("coordinator".to_string()),
        },
        "memd" => HiveProfileDefaults {
            hive_system: Some("memd".to_string()),
            hive_role: Some("memory-control-plane".to_string()),
            capabilities: vec![
                "memory".to_string(),
                "coordination".to_string(),
                "handoff".to_string(),
            ],
            hive_groups: vec!["openclaw-stack".to_string(), "control-plane".to_string()],
            hive_group_goal: Some(
                "maintain canonical shared memory and coordination for the OpenClaw stack"
                    .to_string(),
            ),
            hive_project_enabled: false,
            hive_project_anchor: None,
            hive_project_joined_at: None,
            authority: Some("canonical".to_string()),
        },
        _ => HiveProfileDefaults {
            hive_system: Some(normalized),
            hive_role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string(), "coordination".to_string()],
            hive_groups: Vec::new(),
            hive_group_goal: None,
            hive_project_enabled: false,
            hive_project_anchor: None,
            hive_project_joined_at: None,
            authority: Some("participant".to_string()),
        },
    }
}

pub(crate) fn resolve_hive_profile(args: &InitArgs, project: Option<&str>) -> HiveProfileDefaults {
    let defaults = default_hive_profile(&args.agent);
    let mut capabilities = if args.capability.is_empty() {
        defaults.capabilities
    } else {
        args.capability
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>()
    };
    capabilities.sort();
    capabilities.dedup();
    let mut hive_groups = if args.hive_group.is_empty() {
        defaults.hive_groups
    } else {
        args.hive_group
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>()
    };
    hive_groups.sort();
    hive_groups.dedup();
    HiveProfileDefaults {
        hive_system: args.hive_system.clone().or(defaults.hive_system),
        hive_role: args.hive_role.clone().or(defaults.hive_role),
        capabilities,
        hive_groups: effective_hive_groups(hive_groups, project),
        hive_group_goal: args.hive_group_goal.clone().or(defaults.hive_group_goal),
        hive_project_enabled: defaults.hive_project_enabled,
        hive_project_anchor: defaults.hive_project_anchor,
        hive_project_joined_at: defaults.hive_project_joined_at,
        authority: args.authority.clone().or(defaults.authority),
    }
}

pub(crate) fn write_init_bundle(args: &InitArgs) -> anyhow::Result<()> {
    let project_root = detect_init_project_root(args)?;
    let output = resolve_init_output_path(args, project_root.as_deref());
    if output.exists() && !args.force {
        anyhow::bail!(
            "{} already exists; pass --force to overwrite",
            output.display()
        );
    }

    fs::create_dir_all(output.join("hooks"))
        .with_context(|| format!("create {}", output.join("hooks").display()))?;
    fs::create_dir_all(output.join("agents"))
        .with_context(|| format!("create {}", output.join("agents").display()))?;

    let session = args
        .session
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(default_bundle_session);
    let tab_id = args.tab_id.clone().or_else(default_bundle_tab_id);
    let project = init_project_name(args, project_root.as_deref());
    let namespace = init_namespace_name(args, &output);
    let hive_profile = resolve_hive_profile(args, Some(project.as_str()));
    let project_bootstrap = if args.seed_existing {
        build_project_bootstrap_memory(project_root.as_deref(), &project, args).unwrap_or_default()
    } else {
        None
    };

    let rag_url = args
        .rag_url
        .clone()
        .or_else(|| std::env::var("MEMD_RAG_URL").ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let rag_enabled = rag_url.is_some();
    let worker_name =
        default_bundle_worker_name_for_project(Some(&project), &args.agent, Some(&session));
    let config = BundleConfig {
        schema_version: 2,
        project: project.clone(),
        namespace: namespace.clone(),
        agent: args.agent.clone(),
        session: session.clone(),
        tab_id: tab_id.clone(),
        hive_system: hive_profile.hive_system.clone(),
        hive_role: hive_profile.hive_role.clone(),
        capabilities: hive_profile.capabilities.clone(),
        hive_groups: hive_profile.hive_groups.clone(),
        hive_group_goal: hive_profile.hive_group_goal.clone(),
        hive_project_enabled: false,
        hive_project_anchor: None,
        hive_project_joined_at: None,
        authority: hive_profile.authority.clone(),
        base_url: args.base_url.clone(),
        route: args.route.clone(),
        intent: args.intent.clone(),
        workspace: args.workspace.clone(),
        visibility: args.visibility.clone(),
        heartbeat_model: default_heartbeat_model(),
        voice_mode: args.voice_mode.clone().unwrap_or_else(default_voice_mode),
        auto_short_term_capture: true,
        authority_policy: BundleAuthorityPolicy::default(),
        authority_state: BundleAuthorityState {
            mode: default_authority_mode(),
            degraded: false,
            shared_base_url: Some(args.base_url.clone()),
            fallback_base_url: None,
            activated_at: Some(Utc::now()),
            activated_by: Some("init".to_string()),
            reason: Some("shared authority available".to_string()),
            warning_acknowledged_at: None,
            expires_at: None,
            blocked_capabilities: Vec::new(),
        },
        backend: BundleBackendConfig {
            rag: BundleRagConfig {
                enabled: rag_enabled,
                provider: "lightrag-compatible".to_string(),
                url: rag_url.clone(),
            },
        },
        hooks: BundleHooksConfig {
            context: "hooks/memd-context.sh".to_string(),
            capture: "hooks/memd-capture.sh".to_string(),
            spill: "hooks/memd-spill.sh".to_string(),
            context_ps1: "hooks/memd-context.ps1".to_string(),
            capture_ps1: "hooks/memd-capture.ps1".to_string(),
            spill_ps1: "hooks/memd-spill.ps1".to_string(),
        },
        rag_url: rag_url.clone(),
    };
    fs::write(
        output.join("config.json"),
        serde_json::to_string_pretty(&config)? + "\n",
    )
    .with_context(|| format!("write {}", output.join("config.json").display()))?;

    write_bundle_backend_env(&output, &config)?;

    fs::write(
        output.join("env"),
        format!(
            "MEMD_BASE_URL={}\nMEMD_PROJECT={}\n{}MEMD_AGENT={}\nMEMD_WORKER_NAME={}\nMEMD_SESSION={}\n{}MEMD_ROUTE={}\nMEMD_INTENT={}\nMEMD_HEARTBEAT_MODEL={}\nMEMD_VOICE_MODE={}\nMEMD_AUTO_SHORT_TERM_CAPTURE={}\n{}{}{}",
            args.base_url,
            project,
            namespace
                .as_ref()
                .map(|value| format!("MEMD_NAMESPACE={value}\n"))
                .unwrap_or_default(),
            compose_agent_identity(&args.agent, Some(&session)),
            shell_single_quote(&worker_name),
            session,
            tab_id
                .as_ref()
                .map(|value| format!("MEMD_TAB_ID={value}\n"))
                .unwrap_or_default(),
            args.route,
            args.intent,
            config.heartbeat_model,
            config.voice_mode,
            if config.auto_short_term_capture { "true" } else { "false" },
            args.workspace
                .as_ref()
                .map(|value| format!("MEMD_WORKSPACE={value}\n"))
                .unwrap_or_default(),
            args.visibility
                .as_ref()
                .map(|value| format!("MEMD_VISIBILITY={value}\n"))
                .unwrap_or_default(),
            rag_url
                .as_ref()
                .map(|value| format!("MEMD_RAG_URL={value}\n"))
                .unwrap_or_default(),
        ),
    )
    .with_context(|| format!("write {}", output.join("env").display()))?;

    if let Some(hive_system) = hive_profile.hive_system.as_deref() {
        rewrite_env_assignment(
            &output.join("env"),
            "MEMD_PEER_SYSTEM=",
            &format!("MEMD_PEER_SYSTEM={hive_system}\n"),
        )?;
    }
    if let Some(hive_role) = hive_profile.hive_role.as_deref() {
        rewrite_env_assignment(
            &output.join("env"),
            "MEMD_PEER_ROLE=",
            &format!("MEMD_PEER_ROLE={hive_role}\n"),
        )?;
    }
    if !hive_profile.capabilities.is_empty() {
        rewrite_env_assignment(
            &output.join("env"),
            "MEMD_PEER_CAPABILITIES=",
            &format!(
                "MEMD_PEER_CAPABILITIES={}\n",
                hive_profile.capabilities.join(",")
            ),
        )?;
    }
    if !hive_profile.hive_groups.is_empty() {
        rewrite_env_assignment(
            &output.join("env"),
            "MEMD_PEER_GROUPS=",
            &format!("MEMD_PEER_GROUPS={}\n", hive_profile.hive_groups.join(",")),
        )?;
    }
    if let Some(goal) = hive_profile.hive_group_goal.as_deref() {
        rewrite_env_assignment(
            &output.join("env"),
            "MEMD_PEER_GROUP_GOAL=",
            &format!("MEMD_PEER_GROUP_GOAL={goal}\n"),
        )?;
    }
    if let Some(authority) = hive_profile.authority.as_deref() {
        rewrite_env_assignment(
            &output.join("env"),
            "MEMD_PEER_AUTHORITY=",
            &format!("MEMD_PEER_AUTHORITY={authority}\n"),
        )?;
    }
    if let Some(value) = tab_id.as_deref() {
        rewrite_env_assignment(
            &output.join("env"),
            "MEMD_TAB_ID=",
            &format!("MEMD_TAB_ID={value}\n"),
        )?;
    }

    fs::write(
        output.join("env.ps1"),
        format!(
            "$env:MEMD_BASE_URL = \"{}\"\n$env:MEMD_PROJECT = \"{}\"\n{}$env:MEMD_AGENT = \"{}\"\n$env:MEMD_WORKER_NAME = \"{}\"\n$env:MEMD_SESSION = \"{}\"\n{}$env:MEMD_ROUTE = \"{}\"\n$env:MEMD_INTENT = \"{}\"\n$env:MEMD_HEARTBEAT_MODEL = \"{}\"\n$env:MEMD_VOICE_MODE = \"{}\"\n$env:MEMD_AUTO_SHORT_TERM_CAPTURE = \"{}\"\n{}{}{}",
            escape_ps1(&args.base_url),
            escape_ps1(&project),
            namespace
                .as_ref()
                .map(|value| format!("$env:MEMD_NAMESPACE = \"{}\"\n", escape_ps1(value)))
                .unwrap_or_default(),
            escape_ps1(&compose_agent_identity(&args.agent, Some(&session))),
            escape_ps1(&worker_name),
            escape_ps1(&session),
            tab_id
                .as_ref()
                .map(|value| format!("$env:MEMD_TAB_ID = \"{}\"\n", escape_ps1(value)))
                .unwrap_or_default(),
            escape_ps1(&args.route),
            escape_ps1(&args.intent),
            escape_ps1(&config.heartbeat_model),
            escape_ps1(&config.voice_mode),
            if config.auto_short_term_capture { "true" } else { "false" },
            args.workspace
                .as_ref()
                .map(|value| format!("$env:MEMD_WORKSPACE = \"{}\"\n", escape_ps1(value)))
                .unwrap_or_default(),
            args.visibility
                .as_ref()
                .map(|value| format!("$env:MEMD_VISIBILITY = \"{}\"\n", escape_ps1(value)))
                .unwrap_or_default(),
            rag_url
                .as_ref()
                .map(|value| format!("$env:MEMD_RAG_URL = \"{}\"\n", escape_ps1(value)))
                .unwrap_or_default(),
        ),
    )
    .with_context(|| format!("write {}", output.join("env.ps1").display()))?;

    write_bundle_authority_env(&output, &config.authority_policy, &config.authority_state)?;

    if let Some(hive_system) = hive_profile.hive_system.as_deref() {
        rewrite_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_PEER_SYSTEM = ",
            &format!("$env:MEMD_PEER_SYSTEM = \"{}\"\n", escape_ps1(hive_system)),
        )?;
    }
    if let Some(hive_role) = hive_profile.hive_role.as_deref() {
        rewrite_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_PEER_ROLE = ",
            &format!("$env:MEMD_PEER_ROLE = \"{}\"\n", escape_ps1(hive_role)),
        )?;
    }
    if !hive_profile.capabilities.is_empty() {
        rewrite_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_PEER_CAPABILITIES = ",
            &format!(
                "$env:MEMD_PEER_CAPABILITIES = \"{}\"\n",
                escape_ps1(&hive_profile.capabilities.join(","))
            ),
        )?;
    }
    if !hive_profile.hive_groups.is_empty() {
        rewrite_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_PEER_GROUPS = ",
            &format!(
                "$env:MEMD_PEER_GROUPS = \"{}\"\n",
                escape_ps1(&hive_profile.hive_groups.join(","))
            ),
        )?;
    }
    if let Some(goal) = hive_profile.hive_group_goal.as_deref() {
        rewrite_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_PEER_GROUP_GOAL = ",
            &format!("$env:MEMD_PEER_GROUP_GOAL = \"{}\"\n", escape_ps1(goal)),
        )?;
    }
    if let Some(authority) = hive_profile.authority.as_deref() {
        rewrite_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_PEER_AUTHORITY = ",
            &format!("$env:MEMD_PEER_AUTHORITY = \"{}\"\n", escape_ps1(authority)),
        )?;
    }

    let hook_root = output.join("hooks");
    super::agent_profiles::copy_hook_assets(Path::new(&hook_root))?;
    write_agent_profiles(&output)?;
    let capability_registry = build_bundle_capability_registry(project_root.as_deref());
    let capability_bridges = apply_capability_bridges();
    let capability_summary = format!(
        "{}\n{}",
        render_capability_registry_summary(&capability_registry),
        render_capability_bridge_summary(&capability_bridges)
    );
    write_bundle_memory_placeholder(
        &output,
        &config,
        project_bootstrap
            .as_ref()
            .map(|bundle| bundle.markdown.as_str()),
        Some(&capability_summary),
    )?;
    if let Some(bundle) = &project_bootstrap {
        write_bundle_source_registry(&output, &bundle.registry)?;
    }
    write_bundle_capability_registry(&output, &capability_registry)?;
    write_bundle_capability_bridges(&output, &capability_bridges)?;
    write_native_agent_bridge_files(&output)?;
    write_bundle_command_catalog_files(&output)?;
    write_bundle_harness_bridge_registry(&output)?;

    fs::write(
        output.join("README.md"),
        format!(
            "# memd bundle\n\nThis directory contains the memd configuration for `{project}`.\n\n## Quick Start\n\n1. Set up the bundle:\n   - `memd setup --output {bundle}`\n2. Check readiness and repair drift when needed:\n   - `memd doctor --output {bundle}`\n   - `memd doctor --output {bundle} --repair`\n3. Inspect the active config:\n   - `memd config --output {bundle}`\n4. Refresh the live wake-up surface:\n   - `memd wake --output {bundle} --route {route} --intent {intent} --write`\n5. Launch an agent profile:\n   - `.memd/agents/codex.sh`\n   - `.memd/agents/claude-code.sh`\n   - `.memd/agents/agent-zero.sh`\n   - `.memd/agents/hermes.sh`\n   - `.memd/agents/openclaw.sh`\n   - `.memd/agents/opencode.sh`\n6. Inspect the compact working-memory view when needed:\n   - `memd resume --output {bundle} --route {route} --intent {intent}`\n7. Before memory-dependent answers, run bundle-aware recall:\n   - `memd lookup --output {bundle} --query \"...\"`\n\n## Commands\n\n- `memd commands --output {bundle}`\n- `memd commands --output {bundle} --summary`\n- `memd commands --output {bundle} --json`\n- `memd setup --output {bundle}`\n- `memd doctor --output {bundle}`\n- `memd config --output {bundle}`\n\nThe same catalog is written to `COMMANDS.md` in the bundle root.\n\n## Notes\n\n- Prefer the built `memd` binary during normal multi-session use; `cargo run` adds avoidable compile/cache contention.\n- `env` and `env.ps1` export the same bundle defaults if you want to wire another harness manually.\n- Automatic short-term capture is enabled by default and writes bundle state under `state/last-resume.json`.\n- `MEMD_WAKEUP.md` is the startup live-memory surface; `MEMD_MEMORY.md` is the deeper compact memory view.\n- Add `--semantic` only when you want deeper LightRAG fallback.\n- For Codex, start from `.memd/agents/CODEX_WAKEUP.md`, then use `memd lookup --output {bundle} --query \"...\"` before memory-dependent answers.\n- For Claude Code, import `.memd/agents/CLAUDE_IMPORTS.md` from your project `CLAUDE.md`, then use `/memory` to verify the memd files are loaded.\n",
            project = project,
            bundle = output.display(),
            route = config.route,
            intent = config.intent,
        ),
    )
    .with_context(|| format!("write {}", output.join("README.md").display()))?;

    Ok(())
}

pub(crate) fn build_bundle_turn_placeholder_config(
    output: &Path,
    project: Option<&str>,
    namespace: Option<&str>,
    agent: Option<&str>,
    workspace: Option<&str>,
    visibility: Option<&str>,
    route: Option<&str>,
    intent: Option<&str>,
) -> BundleConfig {
    let runtime = read_bundle_runtime_config(output).ok().flatten();
    let project = project
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| runtime.as_ref().and_then(|value| value.project.clone()))
        .or_else(|| {
            output
                .parent()
                .and_then(|parent| parent.file_name())
                .and_then(|value| value.to_str())
                .map(|value| value.to_string())
        })
        .unwrap_or_else(|| "memd".to_string());
    let namespace = namespace
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| runtime.as_ref().and_then(|value| value.namespace.clone()));
    let agent = agent
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| runtime.as_ref().and_then(|value| value.agent.clone()))
        .unwrap_or_else(|| "codex".to_string());
    let session = runtime
        .as_ref()
        .and_then(|value| value.session.clone())
        .unwrap_or_else(default_bundle_session);
    let tab_id = runtime
        .as_ref()
        .and_then(|value| value.tab_id.clone())
        .or_else(default_bundle_tab_id);
    let hive_system = runtime.as_ref().and_then(|value| value.hive_system.clone());
    let hive_role = runtime.as_ref().and_then(|value| value.hive_role.clone());
    let capabilities = runtime
        .as_ref()
        .map(|value| value.capabilities.clone())
        .unwrap_or_default();
    let hive_groups = runtime
        .as_ref()
        .map(|value| value.hive_groups.clone())
        .unwrap_or_default();
    let hive_group_goal = runtime
        .as_ref()
        .and_then(|value| value.hive_group_goal.clone());
    let hive_project_enabled = runtime
        .as_ref()
        .map(|value| value.hive_project_enabled)
        .unwrap_or(false);
    let hive_project_anchor = runtime
        .as_ref()
        .and_then(|value| value.hive_project_anchor.clone());
    let hive_project_joined_at = runtime
        .as_ref()
        .and_then(|value| value.hive_project_joined_at.clone());
    let authority = runtime.as_ref().and_then(|value| value.authority.clone());
    let base_url = runtime
        .as_ref()
        .and_then(|value| value.base_url.clone())
        .unwrap_or_else(default_base_url);
    let route = route
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| runtime.as_ref().and_then(|value| value.route.clone()))
        .unwrap_or_else(|| "auto".to_string());
    let intent = intent
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| runtime.as_ref().and_then(|value| value.intent.clone()))
        .unwrap_or_else(|| "current_task".to_string());
    let workspace = workspace
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| runtime.as_ref().and_then(|value| value.workspace.clone()));
    let visibility = visibility
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| runtime.as_ref().and_then(|value| value.visibility.clone()));
    let heartbeat_model = runtime
        .as_ref()
        .and_then(|value| value.heartbeat_model.clone())
        .unwrap_or_else(default_heartbeat_model);
    let voice_mode = read_bundle_voice_mode(output).unwrap_or_else(default_voice_mode);
    let auto_short_term_capture = runtime
        .as_ref()
        .map(|value| value.auto_short_term_capture)
        .unwrap_or_else(default_auto_short_term_capture);

    BundleConfig {
        schema_version: 2,
        project,
        namespace,
        agent,
        session,
        tab_id,
        hive_system,
        hive_role,
        capabilities,
        hive_groups,
        hive_group_goal,
        hive_project_enabled,
        hive_project_anchor,
        hive_project_joined_at,
        authority,
        base_url,
        route,
        intent,
        workspace,
        visibility,
        heartbeat_model,
        voice_mode,
        auto_short_term_capture,
        authority_policy: runtime
            .as_ref()
            .map(|value| value.authority_policy.clone())
            .unwrap_or_default(),
        authority_state: runtime
            .as_ref()
            .map(|value| value.authority_state.clone())
            .unwrap_or_default(),
        backend: BundleBackendConfig {
            rag: BundleRagConfig {
                enabled: false,
                provider: "lightrag-compatible".to_string(),
                url: None,
            },
        },
        hooks: BundleHooksConfig {
            context: "hooks/memd-context.sh".to_string(),
            capture: "hooks/memd-capture.sh".to_string(),
            spill: "hooks/memd-spill.sh".to_string(),
            context_ps1: "hooks/memd-context.ps1".to_string(),
            capture_ps1: "hooks/memd-capture.ps1".to_string(),
            spill_ps1: "hooks/memd-spill.ps1".to_string(),
        },
        rag_url: None,
    }
}

pub(crate) fn write_bundle_turn_placeholder_memory(
    output: &Path,
    project: Option<&str>,
    namespace: Option<&str>,
    agent: Option<&str>,
    workspace: Option<&str>,
    visibility: Option<&str>,
    route: Option<&str>,
    intent: Option<&str>,
) -> anyhow::Result<()> {
    let config = build_bundle_turn_placeholder_config(
        output, project, namespace, agent, workspace, visibility, route, intent,
    );
    write_bundle_memory_placeholder(output, &config, None, None)
}

pub(crate) fn write_bundle_turn_fallback_artifacts(
    output: &Path,
    project: Option<&str>,
    namespace: Option<&str>,
    agent: Option<&str>,
    workspace: Option<&str>,
    visibility: Option<&str>,
    route: Option<&str>,
    intent: Option<&str>,
    wakeup_markdown: &str,
) -> anyhow::Result<()> {
    write_bundle_turn_placeholder_memory(
        output, project, namespace, agent, workspace, visibility, route, intent,
    )?;
    write_wakeup_markdown_files(output, wakeup_markdown)?;
    Ok(())
}

pub(crate) fn resolve_init_output_path(args: &InitArgs, project_root: Option<&Path>) -> PathBuf {
    if let Some(explicit) = maybe_explicit_init_output(args) {
        return explicit;
    }

    if args.global {
        return default_global_bundle_root();
    }

    if let Some(project_root) = project_root {
        return project_root.join(".memd");
    }

    default_init_output_path()
}

pub(crate) fn maybe_explicit_init_output(args: &InitArgs) -> Option<PathBuf> {
    let default_init_output = default_init_output_path();
    let default_global_output = default_global_bundle_root();

    if args.output != default_init_output && args.output != default_global_output {
        return Some(args.output.clone());
    }

    None
}

pub(crate) fn write_agent_profiles(output: &Path) -> anyhow::Result<()> {
    let agents_dir = output.join("agents");
    fs::create_dir_all(&agents_dir).with_context(|| format!("create {}", agents_dir.display()))?;
    for (slug, env_agent) in [
        ("agent", None),
        ("codex", Some("codex")),
        ("claude-code", Some("claude-code")),
        ("agent-zero", Some("agent-zero")),
        ("hermes", Some("hermes")),
        ("openclaw", Some("openclaw")),
        ("opencode", Some("opencode")),
    ] {
        let shell_profile = render_agent_shell_profile(output, env_agent);
        let shell_path = agents_dir.join(format!("{slug}.sh"));
        fs::write(&shell_path, shell_profile)
            .with_context(|| format!("write {}", shell_path.display()))?;
        set_executable_if_shell_script(
            &shell_path,
            shell_path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("agent.sh"),
        )?;

        let ps1_profile = render_agent_ps1_profile(output, env_agent);
        let ps1_path = agents_dir.join(format!("{slug}.ps1"));
        fs::write(&ps1_path, ps1_profile)
            .with_context(|| format!("write {}", ps1_path.display()))?;
    }

    for (slug, kinds) in [
        ("lookup", Vec::<&str>::new()),
        ("recall-decisions", vec!["decision", "constraint"]),
        ("recall-preferences", vec!["preference"]),
        (
            "recall-design",
            vec!["preference", "constraint", "decision"],
        ),
        ("recall-history", vec!["fact", "decision", "status"]),
    ] {
        let tags = match slug {
            "recall-design" => vec!["design-memory"],
            _ => Vec::new(),
        };
        let shell_profile = render_lookup_shell_profile(output, &kinds, &tags);
        let shell_path = agents_dir.join(format!("{slug}.sh"));
        fs::write(&shell_path, shell_profile)
            .with_context(|| format!("write {}", shell_path.display()))?;
        set_executable_if_shell_script(
            &shell_path,
            shell_path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("lookup.sh"),
        )?;

        let ps1_profile = render_lookup_ps1_profile(output, &kinds, &tags);
        let ps1_path = agents_dir.join(format!("{slug}.ps1"));
        fs::write(&ps1_path, ps1_profile)
            .with_context(|| format!("write {}", ps1_path.display()))?;
    }

    for (slug, kind, extra_tags) in [
        (
            "remember-decision",
            "decision",
            vec!["basic-memory", "decision"],
        ),
        (
            "remember-preference",
            "preference",
            vec!["basic-memory", "preference"],
        ),
        ("remember-long", "fact", vec!["basic-memory", "long-term"]),
    ] {
        let shell_profile = render_remember_shell_profile(output, kind, &extra_tags);
        let shell_path = agents_dir.join(format!("{slug}.sh"));
        fs::write(&shell_path, shell_profile)
            .with_context(|| format!("write {}", shell_path.display()))?;
        set_executable_if_shell_script(
            &shell_path,
            shell_path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("remember.sh"),
        )?;

        let ps1_profile = render_remember_ps1_profile(output, kind, &extra_tags);
        let ps1_path = agents_dir.join(format!("{slug}.ps1"));
        fs::write(&ps1_path, ps1_profile)
            .with_context(|| format!("write {}", ps1_path.display()))?;
    }

    for slug in ["remember-short", "sync-semantic"] {
        let shell_profile = match slug {
            "remember-short" => render_checkpoint_shell_profile(output),
            "sync-semantic" => render_rag_sync_shell_profile(output),
            _ => unreachable!("unsupported helper slug"),
        };
        let shell_path = agents_dir.join(format!("{slug}.sh"));
        fs::write(&shell_path, shell_profile)
            .with_context(|| format!("write {}", shell_path.display()))?;
        set_executable_if_shell_script(
            &shell_path,
            shell_path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("helper.sh"),
        )?;

        let ps1_profile = match slug {
            "remember-short" => render_checkpoint_ps1_profile(output),
            "sync-semantic" => render_rag_sync_ps1_profile(output),
            _ => unreachable!("unsupported helper slug"),
        };
        let ps1_path = agents_dir.join(format!("{slug}.ps1"));
        fs::write(&ps1_path, ps1_profile)
            .with_context(|| format!("write {}", ps1_path.display()))?;
    }

    for slug in ["watch"] {
        let shell_profile = render_watch_shell_profile(output);
        let shell_path = agents_dir.join(format!("{slug}.sh"));
        fs::write(&shell_path, shell_profile)
            .with_context(|| format!("write {}", shell_path.display()))?;
        set_executable_if_shell_script(
            &shell_path,
            shell_path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("watch.sh"),
        )?;

        let ps1_profile = render_watch_ps1_profile(output);
        let ps1_path = agents_dir.join(format!("{slug}.ps1"));
        fs::write(&ps1_path, ps1_profile)
            .with_context(|| format!("write {}", ps1_path.display()))?;
    }

    for (slug, mode) in [
        ("capture-live", "capture-live"),
        ("correct-memory", "correct-memory"),
    ] {
        let shell_profile = render_capture_shell_profile(output, mode);
        let shell_path = agents_dir.join(format!("{slug}.sh"));
        fs::write(&shell_path, shell_profile)
            .with_context(|| format!("write {}", shell_path.display()))?;
        set_executable_if_shell_script(
            &shell_path,
            shell_path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("capture.sh"),
        )?;

        let ps1_profile = render_capture_ps1_profile(output, mode);
        let ps1_path = agents_dir.join(format!("{slug}.ps1"));
        fs::write(&ps1_path, ps1_profile)
            .with_context(|| format!("write {}", ps1_path.display()))?;
    }

    Ok(())
}

pub(crate) fn write_bundle_memory_placeholder(
    output: &Path,
    config: &BundleConfig,
    project_bootstrap: Option<&str>,
    capability_summary: Option<&str>,
) -> anyhow::Result<()> {
    let mut markdown = String::new();
    markdown.push_str("# memd memory\n\n");
    markdown.push_str("This file is maintained by `memd` for agents that do not have built-in durable memory.\n\n");
    markdown.push_str("## Voice\n\n");
    markdown.push_str(&render_voice_mode_section(&config.voice_mode));
    markdown.push('\n');
    if let Some(project_bootstrap) = project_bootstrap {
        markdown.push_str("## Project bootstrap\n\n");
        markdown.push_str(project_bootstrap);
        if !project_bootstrap.ends_with('\n') {
            markdown.push('\n');
        }
        markdown.push('\n');
    }
    if let Some(capability_summary) = capability_summary {
        markdown.push_str(capability_summary);
        if !capability_summary.ends_with('\n') {
            markdown.push('\n');
        }
        markdown.push('\n');
    }
    markdown.push_str("Refresh it with:\n\n");
    markdown.push_str(&format!(
        "- `memd resume --output {} --route {} --intent {}`\n- `memd resume --output {} --route {} --intent {} --semantic`\n- `memd handoff --output {}`\n- `memd handoff --output {} --semantic`\n\n",
        output.display(),
        config.route,
        config.intent,
        output.display(),
        config.route,
        config.intent,
        output.display(),
        output.display()
    ));
    markdown.push_str("## Bundle Defaults\n\n");
    markdown.push_str(&format!(
        "- project: {}\n- namespace: {}\n- agent: {}\n- session: {}\n- tab: {}\n- workspace: {}\n- visibility: {}\n- route: {}\n- intent: {}\n- heartbeat_model: {}\n- voice_mode: {}\n- auto_short_term_capture: {}\n",
        config.project,
        config.namespace.as_deref().unwrap_or("none"),
        config.agent,
        config.session,
        config.tab_id.as_deref().unwrap_or("none"),
        config.workspace.as_deref().unwrap_or("none"),
        config.visibility.as_deref().unwrap_or("all"),
        config.route,
        config.intent,
        config.heartbeat_model,
        config.voice_mode,
        if config.auto_short_term_capture { "true" } else { "false" },
    ));
    markdown.push_str("\n## Notes\n\n");
    markdown
        .push_str("- `resume` keeps the active working memory fresh on the fast local hot path.\n");
    markdown.push_str("- `handoff` adds shared workspace, source-lane, and delegation state.\n");
    markdown.push_str("- automatic short-term capture runs on compaction spill boundaries unless disabled in the bundle env/config.\n");
    markdown.push_str(
        "- In Codex, treat installed `$gsd-*` skills as the primary GSD interface after `memd reload` (alias: `memd refresh`).\n",
    );
    markdown.push_str(
        "- Do not claim autonomous GSD is blocked on standalone `gsd-*` shell binaries unless you verified that interface is required for this harness and missing on `PATH`.\n",
    );
    markdown.push_str(
        "- If `$gsd-autonomous` is installed as a skill, try that skill path before claiming the autonomous pipeline is unavailable.\n",
    );
    markdown.push_str(
        "- add `--semantic` only when you want slower deep recall from the semantic backend.\n",
    );
    markdown.push_str(
        "- future dream/consolidation output should flow back into this same memory surface.\n",
    );
    write_memory_markdown_files(output, &markdown)
}

pub(crate) async fn refresh_project_bootstrap_memory(
    output: &Path,
) -> anyhow::Result<Option<(String, BootstrapSourceRegistry)>> {
    let Some(mut registry) = read_bundle_source_registry(output)? else {
        return Ok(None);
    };

    let Some(project_root) = PathBuf::from(&registry.project_root)
        .exists()
        .then(|| PathBuf::from(&registry.project_root))
    else {
        return Ok(None);
    };

    let mut changed = Vec::new();
    for source in &mut registry.sources {
        let path = project_root.join(&source.path);
        if !path.exists() {
            if source.present {
                source.present = false;
                changed.push((
                    source.path.clone(),
                    "(source no longer present)".to_string(),
                ));
            }
            continue;
        }
        source.present = true;

        let current_modified = file_modified_at(&path);
        let current_bytes = fs::metadata(&path)
            .map(|meta| meta.len() as usize)
            .unwrap_or(0);
        if source.modified_at == current_modified && source.bytes == current_bytes {
            continue;
        }

        if let Some((snippet, meta)) = read_bootstrap_source(&path, 24) {
            source.hash = meta.hash;
            source.bytes = meta.bytes;
            source.lines = meta.lines;
            source.imported_at = Utc::now();
            source.modified_at = current_modified;
            changed.push((source.path.clone(), snippet));
        }
    }

    if changed.is_empty() {
        return Ok(None);
    }

    let mut markdown = String::new();
    markdown.push_str("\n## Project source refresh\n\n");
    markdown.push_str("The following project sources changed since the last import:\n\n");
    for (path, snippet) in &changed {
        markdown.push_str(&format!("### {}\n\n{}\n\n", path, snippet));
    }

    Ok(Some((markdown, registry)))
}

pub(crate) async fn write_bundle_memory_files(
    output: &Path,
    snapshot: &ResumeSnapshot,
    handoff: Option<&HandoffSnapshot>,
    apply_bridges: bool,
) -> anyhow::Result<()> {
    if let Some(tab_id) = default_bundle_tab_id() {
        let existing_tab_id = read_bundle_runtime_config(output)
            .ok()
            .flatten()
            .and_then(|config| config.tab_id)
            .filter(|value| !value.trim().is_empty());
        if existing_tab_id.is_none() {
            set_bundle_tab_id(output, &tab_id)?;
        }
    }
    let hive = read_bundle_hive_memory_surface(output).await;
    let markdown = render_bundle_memory_markdown(output, snapshot, handoff, hive.as_ref());
    let wakeup = render_bundle_wakeup_markdown(output, snapshot, false);
    let project_root = infer_bundle_project_root(output);
    let capability_registry = build_bundle_capability_registry(project_root.as_deref());
    write_bundle_capability_registry(output, &capability_registry)?;
    let capability_bridges = if apply_bridges {
        apply_capability_bridges()
    } else {
        detect_capability_bridges()
    };
    write_bundle_capability_bridges(output, &capability_bridges)?;
    let capability_summary = format!(
        "{}\n{}",
        render_capability_registry_summary(&capability_registry),
        render_capability_bridge_summary(&capability_bridges)
    );
    let mut migration_registry = None;
    let markdown = if let Some((registry_markdown, registry)) =
        refresh_project_bootstrap_memory(output).await?
    {
        write_bundle_source_registry(output, &registry)?;
        migration_registry = Some(registry);
        format!("{markdown}\n{capability_summary}\n{registry_markdown}")
    } else {
        format!("{markdown}\n{capability_summary}")
    };
    let manifest = build_bundle_migration_manifest(
        output,
        project_root.as_deref(),
        snapshot,
        handoff,
        migration_registry.as_ref(),
        &capability_registry,
        &capability_bridges,
    )?;
    write_bundle_migration_manifest(output, &manifest)?;
    prune_bundle_compiled_memory_outputs(output)?;
    write_bundle_memory_object_pages(output, snapshot, handoff, hive.as_ref())?;
    write_agent_profiles(output)?;
    write_memory_markdown_files(output, &markdown)?;
    write_wakeup_markdown_files(output, &wakeup)?;
    write_native_agent_bridge_files(output)?;
    write_bundle_harness_bridge_registry(output)?;
    write_bundle_resume_state(output, snapshot)?;
    write_bundle_heartbeat(output, Some(snapshot), false).await?;
    self::write_memd_bootstrap_marker(output, snapshot)?;
    Ok(())
}

pub(crate) fn write_memd_bootstrap_marker(
    output: &Path,
    snapshot: &ResumeSnapshot,
) -> anyhow::Result<()> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let cache_dir = home.join(".codex").join("cache");
    fs::create_dir_all(&cache_dir).with_context(|| format!("create {}", cache_dir.display()))?;
    let marker_path = cache_dir.join("memd-bootstrap-current.json");
    let payload = serde_json::json!({
        "bundle_root": output.display().to_string(),
        "project": snapshot.project,
        "namespace": snapshot.namespace,
        "agent": snapshot.agent,
        "workspace": snapshot.workspace,
        "visibility": snapshot.visibility,
        "route": snapshot.route,
        "intent": snapshot.intent,
        "created_at": Utc::now().to_rfc3339(),
    });
    fs::write(&marker_path, serde_json::to_string_pretty(&payload)? + "\n")
        .with_context(|| format!("write {}", marker_path.display()))?;
    Ok(())
}

pub(crate) fn prune_bundle_compiled_memory_outputs(output: &Path) -> anyhow::Result<()> {
    let compiled = bundle_compiled_memory_dir(output);
    if compiled.exists() {
        fs::remove_dir_all(&compiled).with_context(|| format!("remove {}", compiled.display()))?;
    }
    Ok(())
}
pub(crate) struct LoopEntry {
    pub(crate) slug: String,
    pub(crate) normalized_slug: String,
    pub(crate) record: LoopRecord,
    pub(crate) path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LoopRecord {
    #[serde(default)]
    pub(crate) slug: Option<String>,
    #[serde(default)]
    pub(crate) name: Option<String>,
    #[serde(default)]
    pub(crate) iteration: Option<u32>,
    #[serde(default)]
    pub(crate) percent_improvement: Option<f64>,
    #[serde(default)]
    pub(crate) token_savings: Option<f64>,
    #[serde(default)]
    pub(crate) status: Option<String>,
    #[serde(default)]
    pub(crate) summary: Option<String>,
    #[serde(default)]
    pub(crate) artifacts: Option<Vec<String>>,
    #[serde(default)]
    pub(crate) created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub(crate) metadata: JsonValue,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub(crate) struct LoopSummary {
    pub(crate) entries: Vec<LoopSummaryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LoopSummaryEntry {
    pub(crate) slug: String,
    pub(crate) percent_improvement: Option<f64>,
    pub(crate) token_savings: Option<f64>,
    pub(crate) status: Option<String>,
    pub(crate) recorded_at: DateTime<Utc>,
}

pub(crate) fn is_project_root_candidate(dir: &Path) -> bool {
    dir.join(".git").exists()
        || dir.join(".planning").exists()
        || dir.join("CLAUDE.md").exists()
        || dir.join("AGENTS.md").exists()
        || dir.join(".claude").join("CLAUDE.md").exists()
        || dir.join(".agents").join("CLAUDE.md").exists()
}

pub(crate) fn build_project_bootstrap_memory(
    project_root: Option<&Path>,
    project: &str,
    args: &InitArgs,
) -> anyhow::Result<Option<ProjectBootstrapBundle>> {
    let mut sources = project_root
        .map(collect_project_bootstrap_sources)
        .unwrap_or_default();
    sources.extend(collect_user_harness_bootstrap_sources(project_root));
    let mut seen = std::collections::HashSet::new();
    sources.retain(|path| seen.insert(path.clone()));
    sources.sort();
    if sources.is_empty() {
        return Ok(None);
    }

    let mut markdown = String::new();
    let mut registry_sources = Vec::new();
    markdown.push_str("# memd project bootstrap\n\n");
    if let Some(project_root) = project_root {
        markdown.push_str(&format!(
            "This bundle was initialized from the existing project context at `{}`.\n\n",
            project_root.display()
        ));
    } else {
        markdown.push_str(
            "This bundle was initialized from the user's configured harness context.\n\n",
        );
    }
    markdown.push_str("## Loaded sources\n\n");
    for source in &sources {
        markdown.push_str(&format!(
            "- {}\n",
            display_bootstrap_source_path(source, project_root)
        ));
    }
    markdown.push_str("\n## Imported summaries\n\n");

    for source in sources.drain(..) {
        let display = display_bootstrap_source_path(&source, project_root);
        if let Some((snippet, meta)) = read_bootstrap_source(&source, 24) {
            registry_sources.push(BootstrapSourceRecord {
                path: display.clone(),
                kind: source_kind_from_path(&source),
                hash: meta.hash,
                bytes: meta.bytes,
                lines: meta.lines,
                present: true,
                imported_at: Utc::now(),
                modified_at: file_modified_at(&source),
            });
            markdown.push_str(&format!("### {}\n\n{}\n\n", display, snippet));
        }
    }

    markdown.push_str("## Notes\n\n");
    markdown.push_str(&format!(
        "- project: `{}`\n- init agent: `{}`\n- bootstrap mode: `{}`\n",
        project,
        args.agent,
        if args.seed_existing {
            "seed_existing"
        } else {
            "manual"
        }
    ));
    markdown.push_str(
        "- source registry: `state/source-registry.json` with content hashes for imported files\n",
    );
    markdown.push_str("- Add a separate import command if you need a deeper file sweep or more context than the default bootstrap budget.\n");

    Ok(Some(ProjectBootstrapBundle {
        markdown,
        registry: BootstrapSourceRegistry {
            project: project.to_string(),
            project_root: project_root
                .map(|root| root.display().to_string())
                .unwrap_or_else(|| default_global_bundle_root().display().to_string()),
            imported_at: Utc::now(),
            sources: registry_sources,
        },
    }))
}

pub(crate) fn build_bundle_capability_registry(project_root: Option<&Path>) -> CapabilityRegistry {
    build_bundle_capability_registry_with_home(project_root, home_dir().as_deref())
}

pub(crate) fn build_bundle_capability_registry_with_home(
    project_root: Option<&Path>,
    home: Option<&Path>,
) -> CapabilityRegistry {
    let mut capabilities = Vec::new();

    if let Some(project_root) = project_root {
        for (name, kind) in [
            ("AGENTS.md", "policy"),
            ("TEAMS.md", "team"),
            ("CLAUDE.md", "policy"),
        ] {
            let path = project_root.join(name);
            if path.is_file() {
                capabilities.push(CapabilityRecord {
                    harness: "project".to_string(),
                    kind: kind.to_string(),
                    name: name.to_string(),
                    status: "discovered".to_string(),
                    portability_class: "universal".to_string(),
                    source_path: display_bootstrap_source_path(&path, Some(project_root)),
                    bridge_hint: None,
                    hash: file_sha256(&path),
                    notes: Vec::new(),
                });
            }
        }
    }

    let Some(home) = home else {
        return CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: project_root.map(|path| path.display().to_string()),
            capabilities,
        };
    };

    collect_skill_capabilities(
        &mut capabilities,
        "codex",
        &home.join(".codex").join("skills"),
    );
    collect_skill_capabilities(
        &mut capabilities,
        "claude",
        &home.join(".claude").join("skills"),
    );

    let codex_agents_superpowers = home.join(".agents").join("skills").join("superpowers");
    if codex_agents_superpowers.exists() {
        capabilities.push(CapabilityRecord {
            harness: "codex".to_string(),
            kind: "skill-bridge".to_string(),
            name: "superpowers".to_string(),
            status: "installed".to_string(),
            portability_class: "universal".to_string(),
            source_path: codex_agents_superpowers.display().to_string(),
            bridge_hint: None,
            hash: None,
            notes: vec![
                "discovered through ~/.agents/skills native Codex skill bridge".to_string(),
            ],
        });
    }

    for harness_root in detect_claude_family_harness_roots(&home) {
        capabilities.extend(collect_claude_family_capabilities(&harness_root, &home));
    }

    let opencode_plugin = home
        .join(".config")
        .join("opencode")
        .join("plugins")
        .join("memd-plugin.mjs");
    if opencode_plugin.is_file() {
        capabilities.push(CapabilityRecord {
            harness: "opencode".to_string(),
            kind: "plugin".to_string(),
            name: "memd".to_string(),
            status: "enabled".to_string(),
            portability_class: "universal".to_string(),
            source_path: opencode_plugin.display().to_string(),
            bridge_hint: None,
            hash: file_sha256(&opencode_plugin),
            notes: vec!["local memd plugin bridge is active".to_string()],
        });
    }

    let openclaw_workspace_root = home.join(".openclaw").join("workspace");
    capabilities.extend(collect_openclaw_capabilities(&openclaw_workspace_root));

    let opencode_workspace_root = home.join(".config").join("opencode");
    capabilities.extend(collect_opencode_capabilities(&opencode_workspace_root));

    capabilities.sort_by(|a, b| {
        a.harness
            .cmp(&b.harness)
            .then(a.kind.cmp(&b.kind))
            .then(a.name.cmp(&b.name))
            .then(a.source_path.cmp(&b.source_path))
    });
    capabilities.dedup_by(|a, b| {
        a.harness == b.harness
            && a.kind == b.kind
            && a.name == b.name
            && a.source_path == b.source_path
    });

    CapabilityRegistry {
        generated_at: Utc::now(),
        project_root: project_root.map(|path| path.display().to_string()),
        capabilities,
    }
}

pub(crate) fn collect_skill_capabilities(
    records: &mut Vec<CapabilityRecord>,
    harness: &str,
    root: &Path,
) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    let mut skills = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && path.join("SKILL.md").is_file())
        .collect::<Vec<_>>();
    skills.sort();

    for skill_dir in skills {
        let skill_name = skill_dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("unknown");
        let skill_file = skill_dir.join("SKILL.md");
        records.push(CapabilityRecord {
            harness: harness.to_string(),
            kind: "skill".to_string(),
            name: skill_name.to_string(),
            status: "installed".to_string(),
            portability_class: "harness-native".to_string(),
            source_path: skill_file.display().to_string(),
            bridge_hint: None,
            hash: file_sha256(&skill_file),
            notes: Vec::new(),
        });
    }
}

pub(crate) fn collect_claude_family_capabilities(
    harness_root: &HarnessRoot,
    home: &Path,
) -> Vec<CapabilityRecord> {
    let mut records = Vec::new();
    let portability_class = if harness_root.harness == "claude" {
        "universal".to_string()
    } else {
        "claude-family".to_string()
    };
    records.extend(
        collect_named_file_sources(
            &harness_root.root,
            &[
                "AGENTS.md",
                "TEAMS.md",
                "MEMORY.md",
                "USER.md",
                "IDENTITY.md",
                "SOUL.md",
                "TOOLS.md",
                "BOOTSTRAP.md",
                "HEARTBEAT.md",
            ],
        )
        .into_iter()
        .map(|path| CapabilityRecord {
            harness: harness_root.harness.clone(),
            kind: source_kind_from_path(&path),
            name: path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("unknown")
                .to_string(),
            status: "discovered".to_string(),
            portability_class: portability_class.clone(),
            source_path: path.display().to_string(),
            bridge_hint: None,
            hash: file_sha256(&path),
            notes: vec!["detected from Claude-family harness root".to_string()],
        }),
    );
    collect_directory_entry_capabilities(
        &mut records,
        &harness_root.harness,
        &harness_root.root,
        "agents",
        "agent",
        &portability_class,
        Some("agent"),
        &["md", "json", "yml", "yaml"],
    );
    collect_directory_entry_capabilities(
        &mut records,
        &harness_root.harness,
        &harness_root.root,
        "teams",
        "team",
        &portability_class,
        Some("team"),
        &["md", "json", "yml", "yaml"],
    );
    collect_directory_entry_capabilities(
        &mut records,
        &harness_root.harness,
        &harness_root.root,
        "hooks",
        "hook",
        &portability_class,
        Some("hook"),
        &["js", "mjs", "ts", "cts", "json"],
    );
    collect_directory_entry_capabilities(
        &mut records,
        &harness_root.harness,
        &harness_root.root,
        "command",
        "command",
        &portability_class,
        Some("command"),
        &["md", "json", "yml", "yaml", "sh", "js"],
    );
    records.extend(collect_claude_plugin_capabilities(
        &harness_root.root.join("settings.json"),
        &harness_root.harness,
        home,
    ));
    records
}

pub(crate) fn collect_openclaw_capabilities(workspace_root: &Path) -> Vec<CapabilityRecord> {
    collect_harness_root_directory_capabilities(
        workspace_root,
        "openclaw",
        "harness-native",
        &[
            "AGENTS.md",
            "TEAMS.md",
            "MEMORY.md",
            "USER.md",
            "IDENTITY.md",
            "SOUL.md",
            "TOOLS.md",
            "BOOTSTRAP.md",
            "HEARTBEAT.md",
        ],
    )
}

pub(crate) fn collect_opencode_capabilities(root: &Path) -> Vec<CapabilityRecord> {
    collect_harness_root_directory_capabilities(
        root,
        "opencode",
        "harness-native",
        &[
            "AGENTS.md",
            "TEAMS.md",
            "MEMORY.md",
            "USER.md",
            "IDENTITY.md",
            "SOUL.md",
            "TOOLS.md",
            "BOOTSTRAP.md",
            "HEARTBEAT.md",
            "opencode.json",
            "settings.json",
        ],
    )
}

pub(crate) fn collect_harness_root_directory_capabilities(
    harness_root: &Path,
    harness: &str,
    portability_class: &str,
    named_files: &[&str],
) -> Vec<CapabilityRecord> {
    let mut records = Vec::new();
    records.extend(
        collect_named_file_sources(harness_root, named_files)
            .into_iter()
            .map(|path| CapabilityRecord {
                harness: harness.to_string(),
                kind: source_kind_from_path(&path),
                name: path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                status: "discovered".to_string(),
                portability_class: portability_class.to_string(),
                source_path: path.display().to_string(),
                bridge_hint: None,
                hash: file_sha256(&path),
                notes: vec![format!("detected from {harness} workspace root")],
            }),
    );

    collect_directory_entry_capabilities(
        &mut records,
        harness,
        harness_root,
        "agents",
        "agent",
        portability_class,
        Some("agent"),
        &["md", "json", "yml", "yaml"],
    );
    collect_directory_entry_capabilities(
        &mut records,
        harness,
        harness_root,
        "teams",
        "team",
        portability_class,
        Some("team"),
        &["md", "json", "yml", "yaml"],
    );
    collect_directory_entry_capabilities(
        &mut records,
        harness,
        harness_root,
        "hooks",
        "hook",
        portability_class,
        Some("hook"),
        &["js", "mjs", "ts", "cts", "json"],
    );
    collect_directory_entry_capabilities(
        &mut records,
        harness,
        harness_root,
        "command",
        "command",
        portability_class,
        Some("command"),
        &["md", "json", "yml", "yaml", "sh", "js"],
    );

    records.sort_by(|a, b| {
        a.harness
            .cmp(&b.harness)
            .then(a.kind.cmp(&b.kind))
            .then(a.name.cmp(&b.name))
            .then(a.source_path.cmp(&b.source_path))
    });
    records.dedup_by(|a, b| {
        a.harness == b.harness
            && a.kind == b.kind
            && a.name == b.name
            && a.source_path == b.source_path
    });
    records
}

pub(crate) fn collect_claude_plugin_capabilities(
    settings_path: &Path,
    harness_name: &str,
    home: &Path,
) -> Vec<CapabilityRecord> {
    let mut records = Vec::new();
    let Ok(raw) = fs::read_to_string(settings_path) else {
        return records;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return records;
    };
    let Some(enabled) = json
        .get("enabledPlugins")
        .and_then(|value| value.as_object())
    else {
        return records;
    };

    let codex_agents_root = home.join(".agents").join("skills");
    for (plugin_id, value) in enabled {
        if !value.as_bool().unwrap_or(false) {
            continue;
        }

        let (plugin_name, marketplace) = parse_marketplace_plugin_id(plugin_id);
        let codex_cache = latest_cached_plugin_root(
            &home.join(".codex").join("plugins").join("cache"),
            marketplace.as_deref().unwrap_or("unknown"),
            &plugin_name,
        );
        let codex_skills = codex_cache
            .as_ref()
            .map(|path| path.join("skills"))
            .filter(|path| path.is_dir());
        let codex_install = codex_cache
            .as_ref()
            .map(|path| path.join(".codex").join("INSTALL.md"))
            .filter(|path| path.is_file());
        let opencode_bridge = codex_cache
            .as_ref()
            .map(|path| path.join(".opencode").join("plugins"))
            .filter(|path| path.is_dir());
        let codex_skill_bridge = codex_agents_root.join(&plugin_name);

        let portability_class = if codex_skill_bridge.exists() {
            "universal"
        } else if codex_skills.is_some() || codex_install.is_some() || opencode_bridge.is_some() {
            "bridgeable"
        } else {
            "harness-native"
        };
        let bridge_hint = if codex_skill_bridge.exists() {
            None
        } else {
            codex_skills
                .as_ref()
                .map(|path| {
                    format!(
                        "bridge into Codex via ~/.agents/skills -> {}",
                        path.display()
                    )
                })
                .or_else(|| {
                    codex_install
                        .as_ref()
                        .map(|path| format!("bridge into Codex via {}", path.display()))
                })
                .or_else(|| {
                    opencode_bridge
                        .as_ref()
                        .map(|path| format!("bridge into OpenCode via {}", path.display()))
                })
        };

        let mut notes = Vec::new();
        if let Some(path) = codex_cache.as_ref() {
            notes.push(format!(
                "cached in Codex plugin cache at {}",
                path.display()
            ));
        }
        if codex_skill_bridge.exists() {
            notes.push("active Codex bridge detected under ~/.agents/skills".to_string());
        }

        records.push(CapabilityRecord {
            harness: harness_name.to_string(),
            kind: "plugin".to_string(),
            name: plugin_name.clone(),
            status: "enabled".to_string(),
            portability_class: if harness_name == "claude" || portability_class == "universal" {
                portability_class.to_string()
            } else if portability_class == "bridgeable" {
                "claude-family-bridgeable".to_string()
            } else {
                "claude-family".to_string()
            },
            source_path: settings_path.display().to_string(),
            bridge_hint,
            hash: file_sha256(settings_path),
            notes,
        });

        let effective_portability = if harness_name == "claude" || portability_class == "universal"
        {
            portability_class.to_string()
        } else if portability_class == "bridgeable" {
            "claude-family-bridgeable".to_string()
        } else {
            "claude-family".to_string()
        };
        collect_claude_plugin_artifact_capabilities(
            &mut records,
            harness_name,
            &plugin_name,
            codex_cache.as_ref(),
            &effective_portability,
        );
    }

    records
}

pub(crate) fn collect_claude_plugin_artifact_capabilities(
    records: &mut Vec<CapabilityRecord>,
    harness_name: &str,
    plugin_name: &str,
    codex_cache: Option<&PathBuf>,
    portability_class: &str,
) {
    let Some(cache_root) = codex_cache else {
        return;
    };
    collect_directory_entry_capabilities(
        records,
        harness_name,
        cache_root,
        "command",
        "command",
        portability_class,
        Some(plugin_name),
        &["md", "json", "yml", "yaml", "sh", "js"],
    );
    collect_directory_entry_capabilities(
        records,
        harness_name,
        cache_root,
        "hooks",
        "hook",
        portability_class,
        Some(plugin_name),
        &["js", "mjs", "ts", "cts", "json"],
    );
}

pub(crate) fn parse_marketplace_plugin_id(plugin_id: &str) -> (String, Option<String>) {
    let mut parts = plugin_id.split('@');
    let name = parts.next().unwrap_or(plugin_id).trim().to_string();
    let marketplace = parts.next().map(|value| value.trim().to_string());
    (name, marketplace)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn collect_directory_entry_capabilities(
    records: &mut Vec<CapabilityRecord>,
    harness: &str,
    root: &Path,
    relative_dir: &str,
    kind: &str,
    portability_class: &str,
    name_prefix: Option<&str>,
    extensions: &[&str],
) {
    let dir = root.join(relative_dir);
    let Ok(entries) = fs::read_dir(&dir) else {
        return;
    };

    let mut files = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| {
                    extensions
                        .iter()
                        .any(|ext_name| ext.eq_ignore_ascii_case(ext_name))
                })
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    files.sort();

    for path in files {
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("unknown");
        let name = match name_prefix {
            Some(prefix) => format!("{prefix}:{file_name}"),
            None => file_name.to_string(),
        };
        records.push(CapabilityRecord {
            harness: harness.to_string(),
            kind: kind.to_string(),
            name,
            status: "discovered".to_string(),
            portability_class: portability_class.to_string(),
            source_path: path.display().to_string(),
            bridge_hint: None,
            hash: file_sha256(&path),
            notes: vec![format!("discovered from {relative_dir} surface")],
        });
    }
}

pub(crate) fn latest_cached_plugin_root(
    cache_root: &Path,
    marketplace: &str,
    plugin_name: &str,
) -> Option<PathBuf> {
    let plugin_root = cache_root.join(marketplace).join(plugin_name);
    let Ok(entries) = fs::read_dir(&plugin_root) else {
        return None;
    };

    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .max()
}

pub(crate) fn file_sha256(path: &Path) -> Option<String> {
    let raw = fs::read(path).ok()?;
    Some(format!("{:x}", Sha256::digest(&raw)))
}

pub(crate) fn render_capability_registry_summary(registry: &CapabilityRegistry) -> String {
    let mut markdown = String::new();
    let total = registry.capabilities.len();
    let universal = registry
        .capabilities
        .iter()
        .filter(|record| is_universal_class(&record.portability_class))
        .count();
    let bridgeable = registry
        .capabilities
        .iter()
        .filter(|record| is_bridgeable_class(&record.portability_class))
        .count();
    let harness_native = registry
        .capabilities
        .iter()
        .filter(|record| is_harness_native_class(&record.portability_class))
        .count();

    markdown.push_str("## Capability Registry\n\n");
    markdown.push_str(&format!(
        "- discovered_capabilities: {}\n- universal: {}\n- bridgeable: {}\n- harness_native: {}\n",
        total, universal, bridgeable, harness_native
    ));

    let bridgeable_items = registry
        .capabilities
        .iter()
        .filter(|record| is_bridgeable_class(&record.portability_class))
        .take(8)
        .collect::<Vec<_>>();
    if !bridgeable_items.is_empty() {
        markdown.push_str("\n### Bridgeable capabilities\n\n");
        for item in bridgeable_items {
            markdown.push_str(&format!(
                "- {} / {} / {}",
                item.harness, item.kind, item.name
            ));
            if !item.portability_class.is_empty() {
                markdown.push_str(&format!(" [{}]", item.portability_class));
            }
            if let Some(hint) = item.bridge_hint.as_deref() {
                markdown.push_str(&format!(" -> {}", hint));
            }
            markdown.push('\n');
        }
    }

    markdown
}

pub(crate) fn is_universal_class(class: &str) -> bool {
    class == "universal"
}

pub(crate) fn is_bridgeable_class(class: &str) -> bool {
    class.contains("bridgeable")
}

pub(crate) fn is_harness_native_class(class: &str) -> bool {
    matches!(
        class,
        "harness-native" | "claude-family" | "claude-family-bridgeable"
    ) || class.starts_with("harness-")
}

pub(crate) fn render_capability_bridge_summary(registry: &CapabilityBridgeRegistry) -> String {
    let mut markdown = String::new();
    let bridged = registry
        .actions
        .iter()
        .filter(|action| action.status == "bridged")
        .count();
    let already = registry
        .actions
        .iter()
        .filter(|action| action.status == "already-bridged")
        .count();
    let available = registry
        .actions
        .iter()
        .filter(|action| action.status == "available")
        .count();
    let blocked = registry
        .actions
        .iter()
        .filter(|action| action.status == "blocked")
        .count();

    markdown.push_str("## Capability Bridges\n\n");
    markdown.push_str(&format!(
        "- bridged: {}\n- already_bridged: {}\n- available: {}\n- blocked: {}\n",
        bridged, already, available, blocked
    ));
    if !registry.actions.is_empty() {
        markdown.push_str("\n### Recent bridge actions\n\n");
        for action in registry.actions.iter().take(8) {
            markdown.push_str(&format!(
                "- {} / {} -> {} ({})\n",
                action.harness, action.capability, action.target_path, action.status
            ));
        }
    }

    markdown
}

pub(crate) fn build_skill_lifecycle_report(policy: &MemoryPolicyResponse) -> SkillLifecycleReport {
    let registry = build_bundle_capability_registry(None);
    let bridges = detect_capability_bridges();
    let bridge_lookup = bridges
        .actions
        .iter()
        .map(|action| {
            (
                (action.harness.clone(), action.capability.clone()),
                (action.status.as_str(), action.target_path.as_str()),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let runtime_is_defaulted = is_default_runtime(&policy.runtime);
    let low_risk_threshold = 0.25_f32;

    let mut records = Vec::new();
    let mut proposed = 0usize;
    let mut sandbox_passed = 0usize;
    let mut sandbox_review = 0usize;
    let mut sandbox_blocked = 0usize;
    let mut activation_candidates = 0usize;
    let mut activated = 0usize;
    let mut review_queue = Vec::new();
    let mut activate_queue = Vec::new();

    for capability in registry
        .capabilities
        .iter()
        .filter(|capability| capability.kind == "skill" || capability.kind == "skill-bridge")
    {
        proposed += 1;
        let proposal = if capability.status == "installed" || capability.status == "enabled" {
            "proposed"
        } else {
            "staged"
        };

        let bridge_state = bridge_lookup
            .get(&(capability.harness.clone(), capability.name.clone()))
            .copied();
        let (sandbox, sandbox_risk, sandbox_reason) = score_skill_sandbox(capability, bridge_state);
        if sandbox == "pass" {
            sandbox_passed += 1;
        } else if sandbox == "review" {
            sandbox_review += 1;
        } else if sandbox == "block" {
            sandbox_blocked += 1;
        }

        let policy_allows_activation =
            !runtime_is_defaulted && policy.runtime.skill_gating.gated_activation;
        let activation = if !policy_allows_activation {
            "review"
        } else if sandbox == "pass"
            && policy.runtime.skill_gating.sandboxed_evaluation
            && (!policy.runtime.skill_gating.auto_activate_low_risk_only
                || sandbox_risk <= low_risk_threshold)
        {
            activated += 1;
            activation_candidates += 1;
            "activate"
        } else if sandbox == "pass" {
            activation_candidates += 1;
            "candidate"
        } else {
            "hold"
        };
        let activation_reason = match activation {
            "activate" => "low-risk sandbox passed and policy allowed auto-activation",
            "candidate" => "sandbox passed but policy still wants explicit activation",
            "review" if runtime_is_defaulted => {
                "legacy backend defaults require review before activation"
            }
            "review" => "policy gate requires review before activation",
            _ => "sandbox did not pass",
        };

        let record = SkillLifecycleRecord {
            harness: capability.harness.clone(),
            name: capability.name.clone(),
            kind: capability.kind.clone(),
            portability_class: capability.portability_class.clone(),
            proposal: proposal.to_string(),
            sandbox: sandbox.to_string(),
            sandbox_risk,
            sandbox_reason,
            activation: activation.to_string(),
            activation_reason: activation_reason.to_string(),
            source_path: capability.source_path.clone(),
            target_path: bridge_state.map(|state| state.1.to_string()),
            notes: capability.notes.clone(),
        };
        if activation == "activate" {
            activate_queue.push(record.clone());
        } else {
            review_queue.push(record.clone());
        }
        records.push(record);
    }

    records.sort_by(|a, b| {
        a.harness
            .cmp(&b.harness)
            .then(a.kind.cmp(&b.kind))
            .then(a.name.cmp(&b.name))
    });

    SkillLifecycleReport {
        generated_at: Utc::now(),
        proposed,
        sandbox_passed,
        sandbox_review,
        sandbox_blocked,
        activation_candidates,
        activated,
        review_queue,
        activate_queue,
        records,
    }
}

pub(crate) fn score_skill_sandbox(
    capability: &CapabilityRecord,
    bridge_state: Option<(&str, &str)>,
) -> (&'static str, f32, String) {
    let mut risk: f32;
    let mut reasons = Vec::new();

    match capability.portability_class.as_str() {
        "universal" => {
            risk = 0.05;
            reasons.push("portable".to_string());
        }
        class if class.contains("bridgeable") => {
            risk = 0.20;
            reasons.push("bridgeable".to_string());
            match bridge_state.map(|state| state.0) {
                Some("bridged") | Some("already-bridged") => {
                    risk -= 0.12;
                    reasons.push("bridge_ready".to_string());
                }
                Some("blocked") => {
                    risk += 0.20;
                    reasons.push("bridge_blocked".to_string());
                }
                _ => {
                    reasons.push("bridge_pending".to_string());
                }
            }
        }
        "harness-native" => {
            risk = 0.38;
            reasons.push("harness_native".to_string());
        }
        other => {
            risk = 0.82;
            reasons.push(format!("portability={other}"));
        }
    }

    if capability.status == "installed" || capability.status == "enabled" {
        risk -= 0.03;
        reasons.push("present".to_string());
    }
    if capability.hash.is_some() {
        risk -= 0.01;
        reasons.push("hashed".to_string());
    }
    if capability
        .notes
        .iter()
        .any(|note| note.contains("active Codex bridge"))
    {
        risk -= 0.04;
        reasons.push("active_bridge".to_string());
    }

    risk = risk.clamp(0.0, 1.0);
    let sandbox = if risk <= 0.15 {
        "pass"
    } else if risk <= 0.5 {
        "review"
    } else {
        "block"
    };

    (sandbox, risk, reasons.join(";"))
}

pub(crate) fn render_skill_lifecycle_report(report: &SkillLifecycleReport, follow: bool) -> String {
    let mut markdown = String::new();
    markdown.push_str("## Skill Lifecycle\n\n");
    markdown.push_str(&format!(
        "- proposed: {}\n- sandbox_passed: {}\n- sandbox_review: {}\n- sandbox_blocked: {}\n- review_queue: {}\n- activate_queue: {}\n- activation_candidates: {}\n- activated: {}\n",
        report.proposed,
        report.sandbox_passed,
        report.sandbox_review,
        report.sandbox_blocked,
        report.review_queue.len(),
        report.activate_queue.len(),
        report.activation_candidates,
        report.activated
    ));

    if !report.activate_queue.is_empty() {
        markdown.push_str("\n### Activate Queue\n\n");
        for record in report
            .activate_queue
            .iter()
            .take(if follow { 12 } else { 8 })
        {
            markdown.push_str(&format!(
                "- {} / {} / {} risk={:.2} sandbox={} activation={} reason={}",
                record.harness,
                record.kind,
                record.name,
                record.sandbox_risk,
                record.sandbox,
                record.activation,
                record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            if follow && !record.notes.is_empty() {
                markdown.push_str(&format!(" notes={}", record.notes.join(" | ")));
            }
            markdown.push('\n');
        }
    }

    if !report.review_queue.is_empty() {
        markdown.push_str("\n### Review Queue\n\n");
        for record in report.review_queue.iter().take(if follow { 12 } else { 8 }) {
            markdown.push_str(&format!(
                "- {} / {} / {} risk={:.2} sandbox={} activation={} reason={}",
                record.harness,
                record.kind,
                record.name,
                record.sandbox_risk,
                record.sandbox,
                record.activation,
                record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            if follow && !record.notes.is_empty() {
                markdown.push_str(&format!(" notes={}", record.notes.join(" | ")));
            }
            markdown.push('\n');
        }
    }

    if follow && !report.records.is_empty() {
        markdown.push_str("\n### Lifecycle records\n\n");
        for record in report.records.iter().take(if follow { 12 } else { 8 }) {
            markdown.push_str(&format!(
                "- {} / {} / {} [{}] proposal={} sandbox={} risk={:.2} activation={}",
                record.harness,
                record.kind,
                record.name,
                record.portability_class,
                record.proposal,
                record.sandbox,
                record.sandbox_risk,
                record.activation
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            markdown.push_str(&format!(" reason={}", record.sandbox_reason));
            markdown.push_str(&format!(" activation_reason={}", record.activation_reason));
            if follow && !record.notes.is_empty() {
                markdown.push_str(&format!(" notes={}", record.notes.join(" | ")));
            }
            markdown.push('\n');
        }
    }

    markdown
}

pub(crate) fn render_skill_policy_batch_markdown(batch: &SkillPolicyBatchArtifact) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd skill policy batch\n\n");
    markdown.push_str(&format!(
        "- generated_at: {}\n- bundle_root: {}\n- runtime_defaulted: {}\n- proposed: {}\n- sandbox_passed: {}\n- sandbox_review: {}\n- sandbox_blocked: {}\n- review_queue: {}\n- activate_queue: {}\n- activation_candidates: {}\n- activated: {}\n",
        batch.generated_at.to_rfc3339(),
        batch.bundle_root,
        batch.runtime_defaulted,
        batch.report.proposed,
        batch.report.sandbox_passed,
        batch.report.sandbox_review,
        batch.report.sandbox_blocked,
        batch.report.review_queue.len(),
        batch.report.activate_queue.len(),
        batch.report.activation_candidates,
        batch.report.activated
    ));
    markdown.push_str("\n## Apply Flow\n\n");
    markdown.push_str(
        "Use the activate queue after sandbox review. Keep review queue as the manual follow-up set.\n",
    );
    if !batch.report.activate_queue.is_empty() {
        markdown.push_str("\n### Activate Queue\n\n");
        for record in batch.report.activate_queue.iter().take(12) {
            markdown.push_str(&format!(
                "- {} / {} / {} risk={:.2} sandbox={} activation={} reason={}",
                record.harness,
                record.kind,
                record.name,
                record.sandbox_risk,
                record.sandbox,
                record.activation,
                record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            markdown.push('\n');
        }
    }
    if !batch.report.review_queue.is_empty() {
        markdown.push_str("\n### Review Queue\n\n");
        for record in batch.report.review_queue.iter().take(12) {
            markdown.push_str(&format!(
                "- {} / {} / {} risk={:.2} sandbox={} activation={} reason={}",
                record.harness,
                record.kind,
                record.name,
                record.sandbox_risk,
                record.sandbox,
                record.activation,
                record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            markdown.push('\n');
        }
    }
    markdown
}

pub(crate) fn render_skill_policy_queue_markdown(queue: &SkillPolicyQueueArtifact) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# memd skill policy {} queue\n\n- generated_at: {}\n- bundle_root: {}\n- runtime_defaulted: {}\n- records: {}\n",
        queue.queue,
        queue.generated_at.to_rfc3339(),
        queue.bundle_root,
        queue.runtime_defaulted,
        queue.records.len()
    ));
    if !queue.records.is_empty() {
        markdown.push_str("\n## Records\n\n");
        for record in queue.records.iter().take(16) {
            markdown.push_str(&format!(
                "- {} / {} / {} risk={:.2} sandbox={} activation={} reason={}",
                record.harness,
                record.kind,
                record.name,
                record.sandbox_risk,
                record.sandbox,
                record.activation,
                record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            markdown.push('\n');
        }
    }
    markdown
}

pub(crate) fn render_skill_policy_apply_markdown(receipt: &SkillPolicyApplyArtifact) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd skill policy apply receipt\n\n");
    markdown.push_str(&format!(
        "- generated_at: {}\n- bundle_root: {}\n- runtime_defaulted: {}\n- source_queue_path: {}\n- applied_count: {}\n- skipped_count: {}\n",
        receipt.generated_at.to_rfc3339(),
        receipt.bundle_root,
        receipt.runtime_defaulted,
        receipt.source_queue_path,
        receipt.applied_count,
        receipt.skipped_count
    ));
    if !receipt.applied.is_empty() {
        markdown.push_str("\n## Applied\n\n");
        for record in receipt.applied.iter().take(16) {
            markdown.push_str(&format!(
                "- {} / {} / {} -> {}",
                record.harness, record.kind, record.name, record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            markdown.push('\n');
        }
    }
    if !receipt.skipped.is_empty() {
        markdown.push_str("\n## Skipped\n\n");
        for record in receipt.skipped.iter().take(16) {
            markdown.push_str(&format!(
                "- {} / {} / {} -> {}",
                record.harness, record.kind, record.name, record.activation_reason
            ));
            if let Some(target) = record.target_path.as_deref() {
                markdown.push_str(&format!(" -> {}", target));
            }
            markdown.push('\n');
        }
    }
    markdown
}
