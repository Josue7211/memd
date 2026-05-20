use super::*;

#[derive(Clone, Copy)]
enum CapabilityBridgeMode {
    Apply,
    Inspect,
}

struct CapabilityBridgePaths {
    claude_settings: PathBuf,
    claude_skill_root: PathBuf,
    claude_modern_commands: PathBuf,
    claude_legacy_commands: PathBuf,
    codex_skill_root: PathBuf,
    codex_builtin_skill_root: PathBuf,
    opencode_modern_commands: PathBuf,
    opencode_legacy_commands: PathBuf,
    opencode_modern_plugins: PathBuf,
    opencode_legacy_plugins: PathBuf,
}

impl CapabilityBridgePaths {
    fn new(home: &Path) -> Self {
        Self {
            claude_settings: home.join(".claude").join("settings.json"),
            claude_skill_root: home.join(".claude").join("skills"),
            claude_modern_commands: home.join(".claude").join("commands"),
            claude_legacy_commands: home.join(".claude").join("command"),
            codex_skill_root: home.join(".agents").join("skills"),
            codex_builtin_skill_root: home.join(".codex").join("skills"),
            opencode_modern_commands: home.join(".config").join("opencode").join("command"),
            opencode_legacy_commands: home.join(".opencode").join("command"),
            opencode_modern_plugins: home.join(".config").join("opencode").join("plugins"),
            opencode_legacy_plugins: home.join(".opencode").join("plugins"),
        }
    }

    fn opencode_command_roots(&self) -> [&Path; 2] {
        [
            &self.opencode_modern_commands,
            &self.opencode_legacy_commands,
        ]
    }

    fn opencode_plugin_roots(&self) -> [&Path; 2] {
        [&self.opencode_modern_plugins, &self.opencode_legacy_plugins]
    }

    fn claude_command_roots(&self) -> [&Path; 2] {
        [&self.claude_modern_commands, &self.claude_legacy_commands]
    }
}

pub(crate) fn apply_capability_bridges() -> CapabilityBridgeRegistry {
    capability_bridge_registry(CapabilityBridgeMode::Apply)
}

pub(crate) fn detect_capability_bridges() -> CapabilityBridgeRegistry {
    capability_bridge_registry(CapabilityBridgeMode::Inspect)
}

fn capability_bridge_registry(mode: CapabilityBridgeMode) -> CapabilityBridgeRegistry {
    let mut actions = Vec::new();
    let Some(home) = home_dir() else {
        return CapabilityBridgeRegistry {
            generated_at: Utc::now(),
            actions,
        };
    };

    let paths = CapabilityBridgePaths::new(&home);
    for target_root in paths.opencode_command_roots() {
        match mode {
            CapabilityBridgeMode::Apply => {
                actions.extend(ensure_opencode_command_skill_bridges(
                    &paths.codex_builtin_skill_root,
                    target_root,
                    "codex",
                ));
                actions.extend(ensure_opencode_command_skill_bridges(
                    &paths.codex_skill_root,
                    target_root,
                    "codex-bridge",
                ));
            }
            CapabilityBridgeMode::Inspect => {
                actions.extend(inspect_opencode_command_skill_bridges(
                    &paths.codex_builtin_skill_root,
                    target_root,
                    "codex",
                ));
                actions.extend(inspect_opencode_command_skill_bridges(
                    &paths.codex_skill_root,
                    target_root,
                    "codex-bridge",
                ));
            }
        }
    }

    for skill_name in MEMD_BRIDGE_SKILLS {
        let source_skill = paths.codex_builtin_skill_root.join(skill_name);
        if source_skill.is_dir() {
            let target = paths.claude_skill_root.join(skill_name);
            actions.push(match mode {
                CapabilityBridgeMode::Apply => {
                    ensure_directory_skill_bridge("claude", skill_name, &source_skill, &target)
                }
                CapabilityBridgeMode::Inspect => {
                    inspect_directory_skill_bridge("claude", skill_name, &source_skill, &target)
                }
            });
        }
    }

    for target_root in paths.claude_command_roots() {
        actions.extend(match mode {
            CapabilityBridgeMode::Apply => {
                ensure_claude_command_bridges(&paths.codex_builtin_skill_root, target_root, "codex")
            }
            CapabilityBridgeMode::Inspect => inspect_claude_command_bridges(
                &paths.codex_builtin_skill_root,
                target_root,
                "codex",
            ),
        });
    }

    let plugin_records = collect_enabled_plugin_cache_records(&paths.claude_settings, &home);
    for record in plugin_records {
        let source_skills = record.cache_root.join("skills");
        if source_skills.is_dir() {
            let target = paths.codex_skill_root.join(&record.plugin_name);
            actions.push(match mode {
                CapabilityBridgeMode::Apply => ensure_directory_skill_bridge(
                    "codex",
                    &record.plugin_name,
                    &source_skills,
                    &target,
                ),
                CapabilityBridgeMode::Inspect => inspect_directory_skill_bridge(
                    "codex",
                    &record.plugin_name,
                    &source_skills,
                    &target,
                ),
            });
        }

        let source_opencode_plugins = record.cache_root.join(".opencode").join("plugins");
        if source_opencode_plugins.is_dir() {
            for target_root in paths.opencode_plugin_roots() {
                let target = target_root.join(&record.plugin_name);
                actions.push(match mode {
                    CapabilityBridgeMode::Apply => ensure_directory_skill_bridge(
                        "opencode",
                        &record.plugin_name,
                        &source_opencode_plugins,
                        &target,
                    ),
                    CapabilityBridgeMode::Inspect => inspect_directory_skill_bridge(
                        "opencode",
                        &record.plugin_name,
                        &source_opencode_plugins,
                        &target,
                    ),
                });
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
            if let Ok(current) = fs::read_link(target)
                && current == source
            {
                return CapabilityBridgeAction {
                    harness: harness.to_string(),
                    capability: capability.to_string(),
                    status: "already-bridged".to_string(),
                    source_path,
                    target_path,
                    notes: vec!["bridge already points at the current source".to_string()],
                };
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
            if let Ok(current) = fs::read_link(target)
                && current == source
            {
                return CapabilityBridgeAction {
                    harness: harness.to_string(),
                    capability: capability.to_string(),
                    status: "already-bridged".to_string(),
                    source_path,
                    target_path,
                    notes: vec!["bridge already points at the current source".to_string()],
                };
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

const MEMD_OPENCODE_SKILL_BRIDGE_MARKER: &str = "<!-- memd-opencode-skill-bridge -->";
const MEMD_CLAUDE_COMMAND_BRIDGE_MARKER: &str = "<!-- memd-claude-command-bridge -->";

pub(crate) fn ensure_opencode_command_skill_bridges(
    source_root: &Path,
    target_root: &Path,
    capability_prefix: &str,
) -> Vec<CapabilityBridgeAction> {
    let mut actions = Vec::new();
    for skill_dir in collect_skill_dirs_recursive(source_root) {
        let skill_file = skill_dir.join("SKILL.md");
        let capability = format!(
            "{}:{}",
            capability_prefix,
            opencode_command_name_for_skill(source_root, &skill_dir)
        );
        actions.push(ensure_opencode_command_skill_bridge(
            &capability,
            &skill_file,
            target_root,
            source_root,
        ));
    }
    actions
}

pub(crate) fn inspect_opencode_command_skill_bridges(
    source_root: &Path,
    target_root: &Path,
    capability_prefix: &str,
) -> Vec<CapabilityBridgeAction> {
    let mut actions = Vec::new();
    for skill_dir in collect_skill_dirs_recursive(source_root) {
        let skill_file = skill_dir.join("SKILL.md");
        let capability = format!(
            "{}:{}",
            capability_prefix,
            opencode_command_name_for_skill(source_root, &skill_dir)
        );
        actions.push(inspect_opencode_command_skill_bridge(
            &capability,
            &skill_file,
            target_root,
            source_root,
        ));
    }
    actions
}

pub(crate) fn ensure_opencode_command_skill_bridge(
    capability: &str,
    source_skill: &Path,
    target_root: &Path,
    source_root: &Path,
) -> CapabilityBridgeAction {
    let source_path = source_skill.display().to_string();
    let command_name = opencode_command_name_for_skill_root(source_root, source_skill.parent());
    let target = target_root.join(format!("{command_name}.md"));
    let target_path = target.display().to_string();
    let desired = render_opencode_command_skill_bridge(source_skill, &command_name);

    if let Err(err) = fs::create_dir_all(target_root) {
        return CapabilityBridgeAction {
            harness: "opencode".to_string(),
            capability: capability.to_string(),
            status: "blocked".to_string(),
            source_path,
            target_path,
            notes: vec![format!("failed to create target directory: {err}")],
        };
    }

    if let Ok(existing) = fs::read_to_string(&target) {
        if existing == desired {
            return CapabilityBridgeAction {
                harness: "opencode".to_string(),
                capability: capability.to_string(),
                status: "already-bridged".to_string(),
                source_path,
                target_path,
                notes: vec!["OpenCode command bridge already current".to_string()],
            };
        }
        if !existing.contains(MEMD_OPENCODE_SKILL_BRIDGE_MARKER) {
            return CapabilityBridgeAction {
                harness: "opencode".to_string(),
                capability: capability.to_string(),
                status: "blocked".to_string(),
                source_path,
                target_path,
                notes: vec!["target already exists and is not memd-managed".to_string()],
            };
        }
    } else if target.exists() {
        return CapabilityBridgeAction {
            harness: "opencode".to_string(),
            capability: capability.to_string(),
            status: "blocked".to_string(),
            source_path,
            target_path,
            notes: vec!["target exists but is not readable as text".to_string()],
        };
    }

    match fs::write(&target, desired) {
        Ok(()) => CapabilityBridgeAction {
            harness: "opencode".to_string(),
            capability: capability.to_string(),
            status: "bridged".to_string(),
            source_path,
            target_path,
            notes: vec!["generated native OpenCode command bridge".to_string()],
        },
        Err(err) => CapabilityBridgeAction {
            harness: "opencode".to_string(),
            capability: capability.to_string(),
            status: "blocked".to_string(),
            source_path,
            target_path,
            notes: vec![format!("failed to write command bridge: {err}")],
        },
    }
}

pub(crate) fn inspect_opencode_command_skill_bridge(
    capability: &str,
    source_skill: &Path,
    target_root: &Path,
    source_root: &Path,
) -> CapabilityBridgeAction {
    let source_path = source_skill.display().to_string();
    let command_name = opencode_command_name_for_skill_root(source_root, source_skill.parent());
    let target = target_root.join(format!("{command_name}.md"));
    let target_path = target.display().to_string();
    let desired = render_opencode_command_skill_bridge(source_skill, &command_name);

    if !target_root.exists() {
        return CapabilityBridgeAction {
            harness: "opencode".to_string(),
            capability: capability.to_string(),
            status: "blocked".to_string(),
            source_path,
            target_path,
            notes: vec!["target command directory is missing".to_string()],
        };
    }

    if let Ok(existing) = fs::read_to_string(&target) {
        let status = if existing == desired {
            "already-bridged"
        } else if existing.contains(MEMD_OPENCODE_SKILL_BRIDGE_MARKER) {
            "available"
        } else {
            "blocked"
        };
        let notes = match status {
            "already-bridged" => vec!["OpenCode command bridge already current".to_string()],
            "available" => vec!["memd-managed command bridge can be refreshed".to_string()],
            _ => vec!["target already exists and is not memd-managed".to_string()],
        };
        return CapabilityBridgeAction {
            harness: "opencode".to_string(),
            capability: capability.to_string(),
            status: status.to_string(),
            source_path,
            target_path,
            notes,
        };
    }

    CapabilityBridgeAction {
        harness: "opencode".to_string(),
        capability: capability.to_string(),
        status: "available".to_string(),
        source_path,
        target_path,
        notes: vec!["command bridge can be created by explicit init".to_string()],
    }
}

pub(crate) fn ensure_claude_command_bridges(
    source_root: &Path,
    target_root: &Path,
    capability_prefix: &str,
) -> Vec<CapabilityBridgeAction> {
    let mut actions = Vec::new();
    for skill_name in MEMD_BRIDGE_SKILLS {
        let source_skill = source_root.join(skill_name).join("SKILL.md");
        if !source_skill.is_file() {
            continue;
        }
        let capability = format!("{capability_prefix}:{skill_name}");
        actions.push(ensure_claude_command_bridge(
            &capability,
            &source_skill,
            target_root,
            skill_name,
        ));
    }
    actions
}

pub(crate) fn inspect_claude_command_bridges(
    source_root: &Path,
    target_root: &Path,
    capability_prefix: &str,
) -> Vec<CapabilityBridgeAction> {
    let mut actions = Vec::new();
    for skill_name in MEMD_BRIDGE_SKILLS {
        let source_skill = source_root.join(skill_name).join("SKILL.md");
        if !source_skill.is_file() {
            continue;
        }
        let capability = format!("{capability_prefix}:{skill_name}");
        actions.push(inspect_claude_command_bridge(
            &capability,
            &source_skill,
            target_root,
            skill_name,
        ));
    }
    actions
}

pub(crate) fn ensure_claude_command_bridge(
    capability: &str,
    source_skill: &Path,
    target_root: &Path,
    command_name: &str,
) -> CapabilityBridgeAction {
    let source_path = source_skill.display().to_string();
    let target = if command_name == "memd" {
        target_root.join("memd.md")
    } else {
        target_root
            .join("memd")
            .join(format!("{}.md", command_name.trim_start_matches("memd-")))
    };
    let target_path = target.display().to_string();
    let desired = render_claude_command_skill_bridge(source_skill, command_name);

    if let Some(parent) = target.parent()
        && let Err(err) = fs::create_dir_all(parent)
    {
        return CapabilityBridgeAction {
            harness: "claude".to_string(),
            capability: capability.to_string(),
            status: "blocked".to_string(),
            source_path,
            target_path,
            notes: vec![format!("failed to create target directory: {err}")],
        };
    }

    if let Ok(existing) = fs::read_to_string(&target) {
        if existing == desired {
            return CapabilityBridgeAction {
                harness: "claude".to_string(),
                capability: capability.to_string(),
                status: "already-bridged".to_string(),
                source_path,
                target_path,
                notes: vec!["Claude command bridge already current".to_string()],
            };
        }
        if !existing.contains(MEMD_CLAUDE_COMMAND_BRIDGE_MARKER) {
            return CapabilityBridgeAction {
                harness: "claude".to_string(),
                capability: capability.to_string(),
                status: "blocked".to_string(),
                source_path,
                target_path,
                notes: vec!["target already exists and is not memd-managed".to_string()],
            };
        }
    } else if target.exists() {
        return CapabilityBridgeAction {
            harness: "claude".to_string(),
            capability: capability.to_string(),
            status: "blocked".to_string(),
            source_path,
            target_path,
            notes: vec!["target exists but is not readable as text".to_string()],
        };
    }

    match fs::write(&target, desired) {
        Ok(()) => CapabilityBridgeAction {
            harness: "claude".to_string(),
            capability: capability.to_string(),
            status: "bridged".to_string(),
            source_path,
            target_path,
            notes: vec!["generated native Claude command bridge".to_string()],
        },
        Err(err) => CapabilityBridgeAction {
            harness: "claude".to_string(),
            capability: capability.to_string(),
            status: "blocked".to_string(),
            source_path,
            target_path,
            notes: vec![format!("failed to write command bridge: {err}")],
        },
    }
}

pub(crate) fn inspect_claude_command_bridge(
    capability: &str,
    source_skill: &Path,
    target_root: &Path,
    command_name: &str,
) -> CapabilityBridgeAction {
    let source_path = source_skill.display().to_string();
    let target = if command_name == "memd" {
        target_root.join("memd.md")
    } else {
        target_root
            .join("memd")
            .join(format!("{}.md", command_name.trim_start_matches("memd-")))
    };
    let target_path = target.display().to_string();
    let desired = render_claude_command_skill_bridge(source_skill, command_name);

    if !target_root.exists() {
        return CapabilityBridgeAction {
            harness: "claude".to_string(),
            capability: capability.to_string(),
            status: "blocked".to_string(),
            source_path,
            target_path,
            notes: vec!["target command directory is missing".to_string()],
        };
    }

    if let Ok(existing) = fs::read_to_string(&target) {
        let status = if existing == desired {
            "already-bridged"
        } else if existing.contains(MEMD_CLAUDE_COMMAND_BRIDGE_MARKER) {
            "available"
        } else {
            "blocked"
        };
        let notes = match status {
            "already-bridged" => vec!["Claude command bridge already current".to_string()],
            "available" => vec!["memd-managed command bridge can be refreshed".to_string()],
            _ => vec!["target already exists and is not memd-managed".to_string()],
        };
        return CapabilityBridgeAction {
            harness: "claude".to_string(),
            capability: capability.to_string(),
            status: status.to_string(),
            source_path,
            target_path,
            notes,
        };
    }

    CapabilityBridgeAction {
        harness: "claude".to_string(),
        capability: capability.to_string(),
        status: "available".to_string(),
        source_path,
        target_path,
        notes: vec!["command bridge can be created by explicit init".to_string()],
    }
}

pub(crate) fn collect_skill_dirs_recursive(root: &Path) -> Vec<PathBuf> {
    let mut skill_dirs = Vec::new();
    if !root.is_dir() {
        return skill_dirs;
    }

    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if path.join("SKILL.md").is_file() {
                skill_dirs.push(path);
                continue;
            }
            stack.push(path);
        }
    }
    skill_dirs.sort();
    skill_dirs
}

pub(crate) fn opencode_command_name_for_skill(root: &Path, skill_dir: &Path) -> String {
    opencode_command_name_for_skill_root(root, Some(skill_dir))
}

pub(crate) fn opencode_command_name_for_skill_root(
    root: &Path,
    skill_dir: Option<&Path>,
) -> String {
    let Some(skill_dir) = skill_dir else {
        return "skill".to_string();
    };
    let relative = skill_dir.strip_prefix(root).unwrap_or(skill_dir);
    let mut parts = Vec::new();
    for component in relative.components() {
        let part = component.as_os_str().to_string_lossy();
        let cleaned = sanitize_opencode_command_component(&part);
        if !cleaned.is_empty() {
            parts.push(cleaned);
        }
    }
    if parts.is_empty() {
        "skill".to_string()
    } else {
        parts.join("--")
    }
}

pub(crate) fn sanitize_opencode_command_component(raw: &str) -> String {
    let mut cleaned = String::new();
    let mut prev_dash = false;
    for ch in raw.chars() {
        let mapped = if ch.is_ascii_alphanumeric() {
            prev_dash = false;
            Some(ch.to_ascii_lowercase())
        } else if ch == '-' || ch == '_' || ch == ' ' || ch == '.' || ch == '/' {
            if prev_dash {
                None
            } else {
                prev_dash = true;
                Some('-')
            }
        } else {
            None
        };
        if let Some(ch) = mapped {
            cleaned.push(ch);
        }
    }
    cleaned.trim_matches('-').to_string()
}

pub(crate) fn render_opencode_command_skill_bridge(
    source_skill: &Path,
    command_name: &str,
) -> String {
    let description = extract_skill_description(source_skill)
        .unwrap_or_else(|| format!("Run Codex skill `{command_name}` through native OpenCode"));
    let skill_dir = source_skill.parent().unwrap_or(source_skill);
    format!(
        "---\n\
description: {description}\n\
argument-hint: \"[context]\"\n\
tools:\n\
  bash: true\n\
  read: true\n\
  write: true\n\
  glob: true\n\
  grep: true\n\
  edit: true\n\
  task: true\n\
  question: true\n\
---\n\
{marker}\n\
<objective>\n\
Execute Codex skill through native OpenCode command bridge.\n\
\n\
Source of truth: `{source}`\n\
Command id: `{command_name}`\n\
</objective>\n\
\n\
<execution_context>\n\
@{source}\n\
</execution_context>\n\
\n\
<context>\n\
Arguments: $ARGUMENTS\n\
\n\
Resolve any relative references from skill file against `{skill_dir}`.\n\
If local `.memd` exists, prefer project bundle over global bundle for memory, runtime, and voice state.\n\
</context>\n\
\n\
<process>\n\
1. Read execution context skill file.\n\
2. Apply its workflow to current request plus `$ARGUMENTS`.\n\
3. Keep normal spelling and exact technical terms even in caveman voice modes.\n\
4. Return concise result.\n\
</process>\n",
        description = escape_yaml_inline(&description),
        marker = MEMD_OPENCODE_SKILL_BRIDGE_MARKER,
        source = source_skill.display(),
        command_name = command_name,
        skill_dir = skill_dir.display(),
    )
}

pub(crate) fn render_claude_command_skill_bridge(
    source_skill: &Path,
    command_name: &str,
) -> String {
    let description = extract_skill_description(source_skill).unwrap_or_else(|| {
        format!("Run memd skill `{command_name}` through native Claude Code slash command")
    });
    let (name, objective, process) = match command_name {
        "memd" => (
            "memd".to_string(),
            "Use memd as the current project's memory control plane.".to_string(),
            "1. Run `memd status --output .memd --summary`.\n2. If `.memd` is missing and the user did not ask for status-only behavior, run `memd init --output .memd`.\n3. If the user asked to load or refresh memory, run `memd reload --output .memd --summary`.\n4. If the user asked about prior decisions, preferences, or history, run `memd lookup --output .memd --query \"$ARGUMENTS\"`.\n5. Return the resulting readiness state.".to_string(),
        ),
        "memd-init" => (
            "memd:init".to_string(),
            "Initialize memd for the current project and wire the Claude Code bridge.".to_string(),
            "1. Run `memd init --output .memd`.\n2. Run `memd agent --output .memd --name claude-code --apply --summary`.\n3. Run `memd status --output .memd --summary`.\n4. Return the final readiness state.".to_string(),
        ),
        "memd-reload" => (
            "memd:reload".to_string(),
            "Refresh the current project's memd wake surface.".to_string(),
            "1. Run `memd reload --output .memd --summary`.\n2. Run `memd status --output .memd --summary`.\n3. Return the updated readiness state.".to_string(),
        ),
        "memd-status" => (
            "memd:status".to_string(),
            "Check whether memd is installed and ready for the current Claude Code project.".to_string(),
            "Run `memd status --output .memd --summary` and return the result.".to_string(),
        ),
        _ => (
            format!("memd:{command_name}"),
            "Run memd for the current project.".to_string(),
            "Run the appropriate `memd` command for the request and report the result.".to_string(),
        ),
    };
    format!(
        "---\nname: {name}\ndescription: {description}\nallowed-tools:\n  - Bash\n  - Read\n  - Glob\n  - Grep\n---\n{marker}\n<objective>\n{objective}\n</objective>\n\n<execution_context>\n@{source}\n</execution_context>\n\n<process>\n{process}\n</process>\n",
        name = name,
        description = description,
        marker = MEMD_CLAUDE_COMMAND_BRIDGE_MARKER,
        objective = objective,
        source = source_skill.display(),
        process = process,
    )
}

pub(crate) fn extract_skill_description(source_skill: &Path) -> Option<String> {
    let raw = fs::read_to_string(source_skill).ok()?;
    let mut in_frontmatter = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            if in_frontmatter {
                break;
            }
            in_frontmatter = true;
            continue;
        }
        if !in_frontmatter {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("description:") {
            let desc = value.trim().trim_matches('"').trim_matches('\'').trim();
            if !desc.is_empty() && desc != ">" && desc != "|" {
                return Some(desc.to_string());
            }
        }
    }
    None
}

pub(crate) fn escape_yaml_inline(value: &str) -> String {
    let compact = value.replace('\n', " ").trim().to_string();
    let escaped = compact.replace('"', "\\\"");
    format!("\"{escaped}\"")
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
