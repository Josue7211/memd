const CAPABILITY_PAYLOAD_TEXT_PREFIX: &str = "memd:payload-text:";
const CAPABILITY_PAYLOAD_FILE_JSON_PREFIX: &str = "memd:payload-file-json:";
const HOST_CLI_INSTALL_PLAN_PREFIX: &str = "memd:host-cli-install-plan:";
const CAPABILITY_PAYLOAD_MAX_BYTES: u64 = 64 * 1024;
const CAPABILITY_PAYLOAD_SET_MAX_FILES: usize = 32;
const CAPABILITY_PAYLOAD_SET_MAX_TOTAL_BYTES: u64 = 256 * 1024;

fn notes_with_text_payload(mut notes: Vec<String>, path: &Path) -> Vec<String> {
    let Ok(meta) = fs::metadata(path) else {
        return notes;
    };
    if !meta.is_file() || meta.len() > CAPABILITY_PAYLOAD_MAX_BYTES {
        return notes;
    }
    let Ok(content) = fs::read_to_string(path) else {
        return notes;
    };
    notes.push(format!("{CAPABILITY_PAYLOAD_TEXT_PREFIX}{content}"));
    notes
}

fn notes_with_directory_text_payloads(mut notes: Vec<String>, root: &Path) -> Vec<String> {
    if !root.is_dir() {
        return notes;
    }
    let mut files = Vec::new();
    collect_text_payload_files_recursive(root, root, 0, &mut files, &mut 0);
    files.sort_by(|left, right| left.0.cmp(&right.0));
    for (relative_path, content) in files {
        let payload = serde_json::json!({
            "path": relative_path,
            "content": content,
        });
        notes.push(format!("{CAPABILITY_PAYLOAD_FILE_JSON_PREFIX}{payload}"));
    }
    notes
}

fn collect_text_payload_files_recursive(
    root: &Path,
    dir: &Path,
    depth: usize,
    out: &mut Vec<(String, String)>,
    total_bytes: &mut u64,
) {
    if depth > 6 || out.len() >= CAPABILITY_PAYLOAD_SET_MAX_FILES {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    paths.sort();
    for path in paths {
        if out.len() >= CAPABILITY_PAYLOAD_SET_MAX_FILES {
            return;
        }
        if path.is_dir() {
            collect_text_payload_files_recursive(root, &path, depth + 1, out, total_bytes);
            continue;
        }
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        if !meta.is_file() || meta.len() > CAPABILITY_PAYLOAD_MAX_BYTES {
            continue;
        }
        if *total_bytes + meta.len() > CAPABILITY_PAYLOAD_SET_MAX_TOTAL_BYTES {
            return;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(relative) = path.strip_prefix(root) else {
            continue;
        };
        let relative = relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        if relative.is_empty() {
            continue;
        }
        *total_bytes += meta.len();
        out.push((relative, content));
    }
}

pub(crate) fn collect_skill_capabilities(
    records: &mut Vec<CapabilityRecord>,
    harness: &str,
    root: &Path,
) {
    let mut skills = Vec::new();
    collect_any_skill_files_recursive(root, 0, &mut skills);
    skills.sort();

    for skill_file in skills {
        let skill_dir = skill_file.parent().unwrap_or(root);
        let skill_name = skill_dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("unknown");
        records.push(CapabilityRecord {
            harness: harness.to_string(),
            kind: "skill".to_string(),
            name: skill_name.to_string(),
            status: "installed".to_string(),
            portability_class: "harness-native".to_string(),
            source_path: skill_file.display().to_string(),
            bridge_hint: None,
            hash: file_sha256(&skill_file),
            notes: notes_with_directory_text_payloads(
                notes_with_text_payload(Vec::new(), &skill_file),
                skill_dir,
            ),
        });
    }
}

pub(crate) fn collect_project_harness_pack_capabilities(
    project_root: &Path,
) -> Vec<CapabilityRecord> {
    let bundle_root = project_root.join(".memd");
    let project = project_root
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("project");
    let index =
        crate::harness::index::build_harness_pack_index(&bundle_root, Some(project), Some("main"));
    index
        .packs
        .into_iter()
        .map(|pack| {
            let harness = harness_pack_slug(&pack.name);
            let profile = bundle_root.join("agents").join(format!("{harness}.sh"));
            let source_path = if profile.is_file() {
                display_bootstrap_source_path(&profile, Some(project_root))
            } else {
                display_bootstrap_source_path(&bundle_root, Some(project_root))
            };
            let notes = vec![
                pack.role,
                format!("commands={}", pack.commands.len()),
                format!("files={}", pack.files.len()),
            ];
            let notes = if profile.is_file() {
                notes_with_text_payload(notes, &profile)
            } else {
                notes
            };
            CapabilityRecord {
                harness,
                kind: "harness-pack".to_string(),
                name: pack.name,
                status: if profile.is_file() {
                    "wired".to_string()
                } else {
                    "available".to_string()
                },
                portability_class: "universal".to_string(),
                source_path,
                bridge_hint: Some(
                    "server-syncable harness pack; pull on new machines before agent boot"
                        .to_string(),
                ),
                hash: None,
                notes,
            }
        })
        .collect()
}

fn harness_pack_slug(name: &str) -> String {
    match name {
        "Claude Code" => "claude-code".to_string(),
        "Agent Zero" => "agent-zero".to_string(),
        "OpenClaw" => "openclaw".to_string(),
        _ => name
            .trim()
            .to_ascii_lowercase()
            .chars()
            .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
            .collect::<String>()
            .trim_matches('-')
            .to_string(),
    }
}

fn collect_any_skill_files_recursive(root: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > 5 || out.len() >= 500 || !root.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file()
            && path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value == "SKILL.md")
        {
            out.push(path);
            if out.len() >= 500 {
                return;
            }
        } else if path.is_dir() {
            collect_any_skill_files_recursive(&path, depth + 1, out);
            if out.len() >= 500 {
                return;
            }
        }
    }
}

pub(crate) fn collect_codex_plugin_cache_skill_capabilities(
    cache_root: &Path,
) -> Vec<CapabilityRecord> {
    let mut skill_files = Vec::new();
    collect_skill_files_recursive(cache_root, 0, &mut skill_files);
    skill_files.sort();
    skill_files
        .into_iter()
        .take(500)
        .map(|skill_file| {
            let skill_dir = skill_file.parent().unwrap_or(cache_root);
            let skill_name = skill_dir
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("unknown");
            let plugin_name = skill_dir
                .parent()
                .and_then(|path| path.parent())
                .and_then(|path| path.file_name())
                .and_then(|value| value.to_str())
                .unwrap_or("plugin");
            CapabilityRecord {
                harness: "codex".to_string(),
                kind: "plugin-skill".to_string(),
                name: format!("{plugin_name}:{skill_name}"),
                status: "installed".to_string(),
                portability_class: "harness-native".to_string(),
                source_path: skill_file.display().to_string(),
                bridge_hint: Some(
                    "loaded from Codex plugin cache; expose in Active Capabilities".to_string(),
                ),
                hash: file_sha256(&skill_file),
                notes: notes_with_directory_text_payloads(
                    notes_with_text_payload(
                        vec![
                            "discovered from ~/.codex/plugins/cache/**/skills/*/SKILL.md"
                                .to_string(),
                        ],
                        &skill_file,
                    ),
                    skill_dir,
                ),
            }
        })
        .collect()
}

pub(crate) fn collect_codex_plugin_manifest_capabilities(
    cache_root: &Path,
) -> Vec<CapabilityRecord> {
    let mut plugin_manifests = Vec::new();
    collect_plugin_manifests_recursive(cache_root, 0, &mut plugin_manifests);
    plugin_manifests.sort();
    plugin_manifests
        .into_iter()
        .take(200)
        .map(|manifest| {
            let version_dir = manifest
                .parent()
                .and_then(|path| path.parent())
                .unwrap_or(cache_root);
            let version = version_dir
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("unknown");
            let plugin = version_dir
                .parent()
                .and_then(|path| path.file_name())
                .and_then(|value| value.to_str())
                .unwrap_or("plugin");
            CapabilityRecord {
                harness: "codex".to_string(),
                kind: "plugin".to_string(),
                name: format!("{plugin}:{version}"),
                status: "installed".to_string(),
                portability_class: "harness-native".to_string(),
                source_path: manifest.display().to_string(),
                bridge_hint: Some("loaded from Codex plugin cache manifest".to_string()),
                hash: file_sha256(&manifest),
                notes: notes_with_text_payload(
                    vec![
                        "discovered from ~/.codex/plugins/cache/**/.codex-plugin/plugin.json"
                            .to_string(),
                    ],
                    &manifest,
                ),
            }
        })
        .collect()
}

fn collect_plugin_manifests_recursive(root: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > 8 || out.len() >= 200 || !root.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file()
            && path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value == "plugin.json")
            && path
                .parent()
                .and_then(|parent| parent.file_name())
                .and_then(|value| value.to_str())
                == Some(".codex-plugin")
        {
            out.push(path);
            if out.len() >= 200 {
                return;
            }
        } else if path.is_dir() {
            collect_plugin_manifests_recursive(&path, depth + 1, out);
            if out.len() >= 200 {
                return;
            }
        }
    }
}

pub(crate) fn collect_path_cli_capabilities() -> Vec<CapabilityRecord> {
    collect_host_cli_capabilities(find_cli_on_path)
}

fn collect_host_cli_capabilities(
    mut find_cli: impl FnMut(&str) -> Option<PathBuf>,
) -> Vec<CapabilityRecord> {
    let mut records = Vec::new();
    for cli in EXPECTED_HOST_CLIS {
        let path = find_cli(cli);
        let mut notes = vec![
            "PATH inventory; executable availability is host-local".to_string(),
            host_cli_install_plan_note(cli, path.as_deref()),
        ];
        let (status, source_path, bridge_hint) = if let Some(path) = path {
            (
                "installed",
                path.display().to_string(),
                "discovered from PATH; sync as host capability",
            )
        } else {
            notes.push("expected host CLI missing on this machine".to_string());
            (
                "missing",
                format!("host-cli:{cli}"),
                "expected host CLI; sync install guidance as host capability",
            )
        };
        records.push(CapabilityRecord {
            harness: "local".to_string(),
            kind: "cli".to_string(),
            name: cli.to_string(),
            status: status.to_string(),
            portability_class: "host-local".to_string(),
            source_path,
            bridge_hint: Some(bridge_hint.to_string()),
            hash: None,
            notes,
        });
    }
    records
}

const EXPECTED_HOST_CLIS: &[&str] = &["codex", "gh", "opencode", "claude", "wrangler", "supabase"];

fn host_cli_install_plan_note(name: &str, source_path: Option<&Path>) -> String {
    format!(
        "{HOST_CLI_INSTALL_PLAN_PREFIX}{}",
        render_host_cli_install_plan(name, source_path)
    )
}

fn render_host_cli_install_plan(name: &str, source_path: Option<&Path>) -> String {
    let hint = match name {
        "codex" => "Install Codex desktop/CLI for this machine, then expose codex on PATH.",
        "gh" => {
            "Install GitHub CLI for this machine, then authenticate if this repo needs GitHub access."
        }
        "claude" => "Install Claude Code CLI for this machine, then authenticate if needed.",
        "opencode" => {
            "Install OpenCode CLI for this machine, then sync its local config if needed."
        }
        "wrangler" => "Install Cloudflare Wrangler for this machine, then authenticate if needed.",
        "supabase" => "Install Supabase CLI for this machine, then authenticate if needed.",
        _ => "Install this CLI with this machine approved package manager, then expose it on PATH.",
    };
    let source = source_path
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "not observed on source machine PATH".to_string());
    let install_body = host_cli_install_body(name);
    format!(
        r#"#!/bin/sh
set -eu

name='{name}'
echo "memd host CLI install plan: $name"
cat <<'MEMD_HOST_CLI_INFO'
source machine path: {source}
{hint}
memd does not copy host-local binaries across machines.
Default mode is dry-run only; no host changes made.
Set MEMD_HOST_CLI_INSTALL_APPROVED=1 to let this script run package-manager commands on this machine.
MEMD_HOST_CLI_INFO

if command -v "$name" >/dev/null 2>&1; then
  echo "$name already available on PATH"
  exit 0
fi

if [ "${{MEMD_HOST_CLI_INSTALL_APPROVED:-0}}" != "1" ]; then
  echo "dry-run only; no host changes made"
  exit 2
fi

{install_body}

if command -v "$name" >/dev/null 2>&1; then
  echo "$name now available on PATH"
  echo "After install/auth, run: memd capabilities sync --output .memd"
  exit 0
fi

echo "$name still missing after install attempt"
exit 2
"#
    )
}

fn host_cli_install_body(name: &str) -> &'static str {
    match name {
        "gh" => {
            "if command -v brew >/dev/null 2>&1; then\n  brew install gh\nelif command -v apt-get >/dev/null 2>&1; then\n  sudo apt-get update\n  sudo apt-get install -y gh\nelse\n  echo 'No supported package manager found for gh; install GitHub CLI manually.'\nfi"
        }
        "claude" => {
            "if command -v npm >/dev/null 2>&1; then\n  npm install -g @anthropic-ai/claude-code\nelse\n  echo 'npm missing; install Claude Code manually for this machine.'\nfi"
        }
        "opencode" => {
            "if command -v npm >/dev/null 2>&1; then\n  npm install -g opencode-ai\nelse\n  echo 'npm missing; install OpenCode manually for this machine.'\nfi"
        }
        "wrangler" => {
            "if command -v npm >/dev/null 2>&1; then\n  npm install -g wrangler\nelse\n  echo 'npm missing; install Wrangler manually for this machine.'\nfi"
        }
        "supabase" => {
            "if command -v brew >/dev/null 2>&1; then\n  brew install supabase/tap/supabase\nelif command -v npm >/dev/null 2>&1; then\n  npm install -g supabase\nelse\n  echo 'No supported package manager found for supabase; install Supabase CLI manually.'\nfi"
        }
        "codex" => {
            "echo 'Install Codex desktop/CLI manually for this machine, then expose codex on PATH.'"
        }
        _ => {
            "echo 'No approved installer is known for this CLI; install it manually for this machine.'"
        }
    }
}

fn find_cli_on_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}
