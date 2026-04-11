use super::*;

pub(crate) struct BootstrapSourceMeta {
    pub(crate) hash: String,
    pub(crate) bytes: usize,
    pub(crate) lines: usize,
}

pub(crate) fn read_bootstrap_source(
    path: &Path,
    max_lines: usize,
) -> Option<(String, BootstrapSourceMeta)> {
    let raw = fs::read(path).ok()?;
    let text = String::from_utf8_lossy(&raw).into_owned();
    let snippet = text
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty())
        .take(max_lines)
        .collect::<Vec<_>>()
        .join("\n");
    if snippet.trim().is_empty() {
        return None;
    }

    Some((
        snippet,
        BootstrapSourceMeta {
            hash: format!("{:x}", Sha256::digest(&raw)),
            bytes: raw.len(),
            lines: text.lines().count(),
        },
    ))
}

pub(crate) fn source_kind_from_path(path: &Path) -> String {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let path_str = path.to_string_lossy();
    if path_str.contains("/.codex/") {
        return if name.eq_ignore_ascii_case("SKILL.md") {
            "codex-skill".to_string()
        } else {
            "codex-config".to_string()
        };
    }
    if path_str.contains("/.claude/") {
        return if name.eq_ignore_ascii_case("SKILL.md") {
            "claude-skill".to_string()
        } else {
            "claude-config".to_string()
        };
    }
    if path_str.contains("/.openclaw/") {
        return "openclaw-config".to_string();
    }
    if path_str.contains("/.config/opencode/") || path_str.contains("/.opencode/") {
        return "opencode-config".to_string();
    }
    if name.eq_ignore_ascii_case("AGENTS.md") || name.eq_ignore_ascii_case("CLAUDE.md") {
        return "policy".to_string();
    }
    if name.eq_ignore_ascii_case("TEAMS.md") {
        return "team".to_string();
    }
    if name.eq_ignore_ascii_case("MEMORY.md")
        || name.eq_ignore_ascii_case("SOUL.md")
        || name.eq_ignore_ascii_case("USER.md")
        || name.eq_ignore_ascii_case("IDENTITY.md")
        || name.eq_ignore_ascii_case("TOOLS.md")
        || name.eq_ignore_ascii_case("BOOTSTRAP.md")
        || name.eq_ignore_ascii_case("HEARTBEAT.md")
    {
        return "memory".to_string();
    }
    if name.eq_ignore_ascii_case("DESIGN.md") {
        return "design".to_string();
    }
    if path
        .components()
        .any(|part| part.as_os_str().to_string_lossy() == ".planning")
    {
        return "planning".to_string();
    }
    if path.extension().and_then(|value| value.to_str()).is_some() {
        return "doc".to_string();
    }
    "source".to_string()
}

pub(crate) fn bundle_source_registry_path(output: &Path) -> PathBuf {
    output.join("state").join("source-registry.json")
}

pub(crate) fn bundle_capability_registry_path(output: &Path) -> PathBuf {
    output.join("state").join("capability-registry.json")
}

pub(crate) fn bundle_capability_bridges_path(output: &Path) -> PathBuf {
    output.join("state").join("capability-bridges.json")
}

pub(crate) fn bundle_migration_manifest_path(output: &Path) -> PathBuf {
    output.join("state").join("migration-manifest.json")
}

pub(crate) fn write_bundle_source_registry(
    output: &Path,
    registry: &BootstrapSourceRegistry,
) -> anyhow::Result<()> {
    let path = bundle_source_registry_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(registry)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn write_bundle_capability_registry(
    output: &Path,
    registry: &CapabilityRegistry,
) -> anyhow::Result<()> {
    let path = bundle_capability_registry_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(registry)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn write_bundle_capability_bridges(
    output: &Path,
    registry: &CapabilityBridgeRegistry,
) -> anyhow::Result<()> {
    let path = bundle_capability_bridges_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(registry)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn write_bundle_migration_manifest(
    output: &Path,
    manifest: &BundleMigrationManifest,
) -> anyhow::Result<()> {
    let path = bundle_migration_manifest_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(manifest)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn harness_bridge_registry_path(output: &Path) -> PathBuf {
    output.join("state").join("harness-bridge.json")
}

pub(crate) fn write_bundle_harness_bridge_registry(
    output: &Path,
) -> anyhow::Result<HarnessBridgeRegistry> {
    let registry = build_harness_bridge_registry();
    let path = harness_bridge_registry_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(&registry)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    let markdown_path = output.join("agents").join("HARNESS_BRIDGES.md");
    if let Some(parent) = markdown_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&markdown_path, render_harness_bridge_markdown(&registry))
        .with_context(|| format!("write {}", markdown_path.display()))?;
    Ok(registry)
}

pub(crate) fn read_bundle_harness_bridge_registry(
    output: &Path,
) -> anyhow::Result<Option<HarnessBridgeRegistry>> {
    let path = harness_bridge_registry_path(output);
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let registry = serde_json::from_str::<HarnessBridgeRegistry>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(registry))
}

pub(crate) fn build_harness_bridge_registry() -> HarnessBridgeRegistry {
    let harnesses = vec![
        harness_bridge_record(
            "codex",
            detect_codex_memd_wiring(),
            &["config", "hook", "skill"],
            &["Codex is native when the config, hook, and skill surfaces are all present."],
        ),
        harness_bridge_record(
            "claude",
            detect_claude_memd_wiring(),
            &["settings", "hook"],
            &["Claude is native when the settings and session hook surfaces exist."],
        ),
        harness_bridge_record(
            "claw",
            detect_claw_memd_wiring(),
            &["binary", "config", "skill"],
            &[
                "Claw is memd-ready when the binary is installed, config exists, and memd skills are visible through shared skill roots.",
            ],
        ),
        harness_bridge_record(
            "openclaw",
            detect_openclaw_memd_wiring(),
            &["agents", "bootstrap"],
            &["OpenClaw is native when AGENTS.md and BOOTSTRAP.md bridge surfaces exist."],
        ),
        harness_bridge_record(
            "opencode",
            detect_opencode_memd_wiring(),
            &["config", "plugin", "command"],
            &[
                "OpenCode is native when config, plugin, and command surfaces all route through memd.",
            ],
        ),
    ];

    let all_wired = harnesses.iter().all(|record| record.wired);
    HarnessBridgeRegistry {
        generated_at: Utc::now(),
        overall_portability_class: if all_wired {
            "portable".to_string()
        } else {
            "adapter-required".to_string()
        },
        all_wired,
        harnesses,
    }
}

pub(crate) fn harness_bridge_record(
    harness: &str,
    wiring: serde_json::Value,
    required_surfaces: &[&str],
    notes: &[&str],
) -> HarnessBridgeRecord {
    let wired = wiring
        .get("wired")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let missing_surfaces = required_surfaces
        .iter()
        .filter_map(|surface| {
            let present = wiring
                .get(*surface)
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            if present {
                None
            } else {
                Some((*surface).to_string())
            }
        })
        .collect::<Vec<_>>();

    HarnessBridgeRecord {
        harness: harness.to_string(),
        wired,
        portability_class: if wired {
            "harness-native".to_string()
        } else {
            "adapter-required".to_string()
        },
        required_surfaces: required_surfaces
            .iter()
            .map(|value| value.to_string())
            .collect(),
        missing_surfaces,
        notes: notes.iter().map(|value| value.to_string()).collect(),
    }
}

pub(crate) fn render_harness_bridge_markdown(registry: &HarnessBridgeRegistry) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd harness bridge matrix\n\n");
    markdown.push_str(&format!(
        "Generated: {}\n\n",
        registry.generated_at.to_rfc3339()
    ));
    markdown.push_str(&format!(
        "Overall portability class: **{}**\n\n",
        registry.overall_portability_class
    ));
    markdown.push_str("| Harness | Wired | Portability | Missing surfaces | Notes |\n");
    markdown.push_str("|---|---|---|---|---|\n");
    for harness in &registry.harnesses {
        let missing = if harness.missing_surfaces.is_empty() {
            "none".to_string()
        } else {
            harness.missing_surfaces.join(", ")
        };
        let notes = if harness.notes.is_empty() {
            "none".to_string()
        } else {
            harness
                .notes
                .iter()
                .map(|note| compact_inline(note, 120))
                .collect::<Vec<_>>()
                .join(" | ")
        };
        markdown.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            harness.harness,
            if harness.wired { "yes" } else { "no" },
            harness.portability_class,
            missing,
            notes
        ));
    }
    markdown.push_str("\n## Adapter Required Surface\n\n");
    markdown.push_str(
        "If a harness is not wired, `memd` treats it as adapter-required and surfaces the missing bridge surfaces instead of pretending the skill is universally available.\n",
    );
    markdown
}

pub(crate) fn read_bundle_source_registry(
    output: &Path,
) -> anyhow::Result<Option<BootstrapSourceRegistry>> {
    let path = bundle_source_registry_path(output);
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let registry = serde_json::from_str::<BootstrapSourceRegistry>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(registry))
}

pub(crate) fn file_modified_at(path: &Path) -> Option<DateTime<Utc>> {
    fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
        .map(DateTime::<Utc>::from)
}
