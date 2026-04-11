use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CommandCatalog {
    pub(crate) root: String,
    pub(crate) command_count: usize,
    pub(crate) commands: Vec<CommandCatalogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CommandCatalogEntry {
    pub(crate) name: String,
    pub(crate) surface: String,
    pub(crate) kind: String,
    pub(crate) ownership: String,
    pub(crate) role: String,
    pub(crate) compatibility_flags: Vec<String>,
    pub(crate) command: String,
    pub(crate) purpose: String,
    pub(crate) path: Option<String>,
}

pub(crate) fn build_command_catalog(bundle_root: &Path) -> CommandCatalog {
    let commands = vec![
        entry(
            "memd commands",
            "memd",
            "CLI",
            "memd",
            "native-cli",
            &["memd-binary", "bundle-root-present"],
            "memd commands --output .memd",
            "inspect the bundle command catalog itself",
            None,
        ),
        entry(
            "memd status",
            "memd",
            "CLI",
            "memd",
            "native-cli",
            &["memd-binary", "bundle-root-present"],
            "memd status --output .memd",
            "check bundle readiness and missing files",
            None,
        ),
        entry(
            "memd wake",
            "memd",
            "CLI",
            "memd",
            "native-cli",
            &["memd-binary", "bundle-root-present"],
            "memd wake --output .memd --intent current_task --write",
            "refresh the wake surface before a turn",
            None,
        ),
        entry(
            "memd resume",
            "memd",
            "CLI",
            "memd",
            "native-cli",
            &["memd-binary", "bundle-root-present"],
            "memd resume --output .memd --intent current_task",
            "resume compact working memory for the current task",
            None,
        ),
        entry(
            "memd lookup",
            "memd",
            "CLI",
            "memd",
            "native-cli",
            &["memd-binary", "bundle-root-present"],
            "memd lookup --output .memd --query \"...\"",
            "run bundle-aware recall before answering",
            None,
        ),
        entry(
            "memd checkpoint",
            "memd",
            "CLI",
            "memd",
            "native-cli",
            &["memd-binary", "bundle-root-present"],
            "memd checkpoint --output .memd --content \"...\"",
            "write short-term task state into the live backend",
            None,
        ),
        entry(
            "memd remember",
            "memd",
            "CLI",
            "memd",
            "native-cli",
            &["memd-binary", "bundle-root-present"],
            "memd remember --output .memd --kind decision --content \"...\"",
            "persist durable typed memory",
            None,
        ),
        entry(
            "memd handoff",
            "memd",
            "CLI",
            "memd",
            "native-cli",
            &["memd-binary", "bundle-root-present"],
            "memd handoff --output .memd --prompt",
            "emit a compact takeover packet",
            None,
        ),
        entry(
            "memd packs",
            "memd",
            "CLI",
            "memd",
            "native-cli",
            &["memd-binary", "bundle-root-present"],
            "memd packs --root .memd --summary",
            "inspect visible harness packs in a bundle",
            None,
        ),
        entry(
            "memd skills",
            "memd",
            "CLI",
            "memd",
            "native-cli",
            &["memd-binary", "bundle-root-present"],
            "memd skills --summary",
            "inspect the discovered skill catalog",
            None,
        ),
        entry(
            "memd hook capture",
            "memd",
            "CLI",
            "memd",
            "native-cli",
            &["memd-binary", "bundle-root-present"],
            "memd hook capture --output .memd --stdin --summary",
            "record live turn changes and refresh bundle truth",
            None,
        ),
        entry(
            "memd hook spill",
            "memd",
            "CLI",
            "memd",
            "native-cli",
            &["memd-binary", "bundle-root-present"],
            "memd hook spill --output .memd --stdin --apply",
            "spill compaction state into durable memory",
            None,
        ),
        entry(
            "/memory",
            "Claude Code",
            "slash",
            "external",
            "bridge-surface",
            &[
                "bundle-root-present",
                "claude-import-bridge",
                "claude-project-bridge",
            ],
            "/memory",
            "universal Claude bridge command surfaced during migration",
            None,
        ),
        entry(
            "$gsd-autonomous",
            "Codex",
            "external-skill",
            "external",
            "bridge-surface",
            &["codex-skill-installed"],
            "$gsd-autonomous",
            "universal Codex bridge command surfaced during migration",
            None,
        ),
        entry(
            "$gsd-map-codebase",
            "Codex",
            "external-skill",
            "external",
            "bridge-surface",
            &["codex-skill-installed"],
            "$gsd-map-codebase",
            "universal Codex bridge command surfaced during migration",
            None,
        ),
        entry(
            ".memd/agents/codex.sh",
            "Codex",
            "helper",
            "memd",
            "bundle-helper",
            &[
                "bundle-root-present",
                "launcher-script-present",
                "launcher-script-executable",
            ],
            ".memd/agents/codex.sh",
            "launch the Codex harness pack",
            Some(bundle_root.join("agents").join("codex.sh")),
        ),
        entry(
            ".memd/agents/claude-code.sh",
            "Claude Code",
            "helper",
            "memd",
            "bundle-helper",
            &[
                "bundle-root-present",
                "launcher-script-present",
                "launcher-script-executable",
            ],
            ".memd/agents/claude-code.sh",
            "launch the Claude Code harness pack",
            Some(bundle_root.join("agents").join("claude-code.sh")),
        ),
        entry(
            ".memd/agents/agent-zero.sh",
            "Agent Zero",
            "helper",
            "memd",
            "bundle-helper",
            &[
                "bundle-root-present",
                "launcher-script-present",
                "launcher-script-executable",
            ],
            ".memd/agents/agent-zero.sh",
            "launch the Agent Zero harness pack",
            Some(bundle_root.join("agents").join("agent-zero.sh")),
        ),
        entry(
            ".memd/agents/openclaw.sh",
            "OpenClaw",
            "helper",
            "memd",
            "bundle-helper",
            &[
                "bundle-root-present",
                "launcher-script-present",
                "launcher-script-executable",
            ],
            ".memd/agents/openclaw.sh",
            "launch the OpenClaw harness pack",
            Some(bundle_root.join("agents").join("openclaw.sh")),
        ),
        entry(
            ".memd/agents/opencode.sh",
            "OpenCode",
            "helper",
            "memd",
            "bundle-helper",
            &[
                "bundle-root-present",
                "launcher-script-present",
                "launcher-script-executable",
            ],
            ".memd/agents/opencode.sh",
            "launch the OpenCode harness pack",
            Some(bundle_root.join("agents").join("opencode.sh")),
        ),
        entry(
            ".memd/agents/hermes.sh",
            "Hermes",
            "helper",
            "memd",
            "bundle-helper",
            &[
                "bundle-root-present",
                "launcher-script-present",
                "launcher-script-executable",
            ],
            ".memd/agents/hermes.sh",
            "launch the Hermes harness pack",
            Some(bundle_root.join("agents").join("hermes.sh")),
        ),
    ];

    CommandCatalog {
        root: bundle_root.display().to_string(),
        command_count: commands.len(),
        commands,
    }
}

pub(crate) fn filter_command_catalog(
    catalog: CommandCatalog,
    query: Option<&str>,
) -> CommandCatalog {
    let Some(query) = query
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
    else {
        return catalog;
    };

    let commands = catalog
        .commands
        .into_iter()
        .filter(|command| command_matches(command, &query))
        .collect::<Vec<_>>();

    CommandCatalog {
        command_count: commands.len(),
        commands,
        ..catalog
    }
}

fn command_matches(command: &CommandCatalogEntry, query: &str) -> bool {
    let mut fields = vec![
        command.name.to_lowercase(),
        command.surface.to_lowercase(),
        command.kind.to_lowercase(),
        command.ownership.to_lowercase(),
        command.role.to_lowercase(),
        command.compatibility_flags.join(" ").to_lowercase(),
        command.command.to_lowercase(),
        command.purpose.to_lowercase(),
    ];
    if let Some(path) = command.path.as_deref() {
        fields.push(path.to_lowercase());
    }
    fields.into_iter().any(|field| field.contains(query))
}

fn entry(
    name: &str,
    surface: &str,
    kind: &str,
    ownership: &str,
    role: &str,
    compatibility_flags: &[&str],
    command: &str,
    purpose: &str,
    path: Option<PathBuf>,
) -> CommandCatalogEntry {
    CommandCatalogEntry {
        name: name.to_string(),
        surface: surface.to_string(),
        kind: kind.to_string(),
        ownership: ownership.to_string(),
        role: role.to_string(),
        compatibility_flags: compatibility_flags
            .iter()
            .map(|value| value.to_string())
            .collect(),
        command: command.to_string(),
        purpose: purpose.to_string(),
        path: path.map(|path| path.display().to_string()),
    }
}
