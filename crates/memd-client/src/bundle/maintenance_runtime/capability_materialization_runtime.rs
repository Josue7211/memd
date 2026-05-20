fn compact_capability_notes_for_materialization(notes: &[String]) -> Vec<String> {
    notes
        .iter()
        .map(|note| {
            if note.starts_with(CAPABILITY_PAYLOAD_TEXT_PREFIX) {
                "memd:payload-text:<omitted; see materialization actions>".to_string()
            } else if note.starts_with(CAPABILITY_PAYLOAD_FILE_JSON_PREFIX) {
                "memd:payload-file-json:<omitted; see materialization actions>".to_string()
            } else if note.starts_with(HOST_CLI_INSTALL_PLAN_PREFIX) {
                "memd:host-cli-install-plan:<omitted; see materialization actions>".to_string()
            } else {
                note.clone()
            }
        })
        .collect()
}

fn build_capability_materialization_report(
    output: &Path,
    registry: &CapabilityRegistry,
    apply: bool,
) -> anyhow::Result<CapabilityMaterializationReport> {
    let mut actions = registry
        .capabilities
        .iter()
        .map(|record| materialization_action_for_record(output, record))
        .collect::<Vec<_>>();
    let mut applied = 0;
    let mut skipped = 0;
    if apply {
        for action in &mut actions {
            if apply_capability_materialization(action)? {
                applied += 1;
            } else {
                skipped += 1;
            }
        }
    }
    let missing = actions
        .iter()
        .filter(|action| action.status == "missing" || action.status == "install-failed")
        .count();
    let installable = actions
        .iter()
        .filter(|action| {
            matches!(
                action.status.as_str(),
                "present" | "installable" | "installer-ready"
            )
        })
        .count();
    let host_local = actions
        .iter()
        .filter(|action| {
            matches!(
                action.action.as_str(),
                "host-cli-on-path" | "write-host-cli-install-plan" | "install-host-cli"
            )
        })
        .count();
    let auth_gaps = actions
        .iter()
        .filter(|action| {
            matches!(
                action.auth_status.as_deref(),
                Some("unknown" | "unauthenticated")
            )
        })
        .count();
    let auth_unknown = actions
        .iter()
        .filter(|action| action.auth_status.as_deref() == Some("unknown"))
        .count();
    let auth_authenticated = actions
        .iter()
        .filter(|action| action.auth_status.as_deref() == Some("authenticated"))
        .count();
    let auth_unauthenticated = actions
        .iter()
        .filter(|action| action.auth_status.as_deref() == Some("unauthenticated"))
        .count();
    let fresh_machine_ready = missing == 0 && host_local == 0;
    Ok(CapabilityMaterializationReport {
        status: if fresh_machine_ready {
            "ready".to_string()
        } else if apply && applied > 0 {
            "partial-applied".to_string()
        } else if host_local > 0 {
            "partial-host-local".to_string()
        } else {
            "partial".to_string()
        },
        installable,
        missing,
        host_local,
        auth_gaps,
        auth_unknown,
        auth_authenticated,
        auth_unauthenticated,
        fresh_machine_ready,
        applied,
        skipped,
        actions,
    })
}

fn materialization_action_for_record(
    output: &Path,
    record: &CapabilityRecord,
) -> CapabilityMaterializationAction {
    if let Some(plan_text) = host_cli_install_plan_text(record) {
        if host_cli_available_on_path(&record.name) {
            return CapabilityMaterializationAction {
                harness: record.harness.clone(),
                kind: record.kind.clone(),
                name: record.name.clone(),
                status: "present".to_string(),
                action: "host-cli-on-path".to_string(),
                source_path: record.source_path.clone(),
                target_path: None,
                payload_text: None,
                auth_status: host_cli_auth_status(&record.name),
                auth_check: host_cli_auth_check(&record.name),
                reason: host_cli_reason(
                    &record.name,
                    "host-local CLI is available on this machine, but fresh machines still need machine-specific install proof",
                ),
            };
        }
        let target = host_cli_install_plan_target_path(output, record);
        return CapabilityMaterializationAction {
            harness: record.harness.clone(),
            kind: record.kind.clone(),
            name: record.name.clone(),
            status: "missing".to_string(),
            action: "write-host-cli-install-plan".to_string(),
            source_path: record.source_path.clone(),
            target_path: Some(target.display().to_string()),
            payload_text: Some(plan_text.to_string()),
            auth_status: host_cli_auth_status(&record.name),
            auth_check: host_cli_auth_check(&record.name),
            reason: host_cli_reason(
                &record.name,
                "host-local CLI needs machine-specific install; server can restore an install plan but not the executable",
            ),
        };
    }
    if let Some(payload_text) = capability_payload_text(record) {
        if let Some(payload_text) = capability_payload_file_set_text(record) {
            let target = capability_payload_target_path(output, record)
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| capability_payload_target_path(output, record));
            return CapabilityMaterializationAction {
                harness: record.harness.clone(),
                kind: record.kind.clone(),
                name: record.name.clone(),
                status: "installable".to_string(),
                action: "restore-from-payload-set".to_string(),
                source_path: record.source_path.clone(),
                target_path: Some(target.display().to_string()),
                payload_text: Some(payload_text),
                auth_status: None,
                auth_check: None,
                reason:
                    "server-synced text payload set can restore skill/plugin files on this machine"
                        .to_string(),
            };
        }
        let target = capability_payload_target_path(output, record);
        return CapabilityMaterializationAction {
            harness: record.harness.clone(),
            kind: record.kind.clone(),
            name: record.name.clone(),
            status: "installable".to_string(),
            action: "restore-from-payload".to_string(),
            source_path: record.source_path.clone(),
            target_path: Some(target.display().to_string()),
            payload_text: Some(payload_text.to_string()),
            auth_status: None,
            auth_check: None,
            reason: "server-synced text payload can be materialized on this machine".to_string(),
        };
    }
    if is_bundle_relative_capability(record) {
        let (source, target) = capability_materialization_paths(output, record);
        let status = if source.exists() {
            "present"
        } else {
            "installable"
        };
        return CapabilityMaterializationAction {
            harness: record.harness.clone(),
            kind: record.kind.clone(),
            name: record.name.clone(),
            status: status.to_string(),
            action: "restore-from-bundle".to_string(),
            source_path: source.display().to_string(),
            target_path: Some(target.display().to_string()),
            payload_text: None,
            auth_status: None,
            auth_check: None,
            reason: "portable bundle asset can be restored from memd bundle state".to_string(),
        };
    }

    let (action, reason) = if record.portability_class == "host-local" || record.kind == "cli" {
        (
            "install-host-cli",
            "host-local CLI needs machine-specific install or PATH guidance",
        )
    } else if record.harness == "codex" && record.kind.contains("plugin") {
        (
            "install-codex-plugin",
            "Codex plugin/skill cache is harness-native and needs plugin installer payload",
        )
    } else if record.harness.contains("claude") {
        (
            "install-claude-code-asset",
            "Claude Code config/plugin asset is not materialized from server inventory",
        )
    } else if record.harness == "hermes" {
        (
            "install-hermes-asset",
            "Hermes harness asset is not materialized from server inventory",
        )
    } else if record.harness == "opencode" {
        (
            "install-opencode-asset",
            "OpenCode command/plugin is not materialized from server inventory",
        )
    } else {
        (
            "needs-materializer",
            "capability record has no local source and no proven installer",
        )
    };

    CapabilityMaterializationAction {
        harness: record.harness.clone(),
        kind: record.kind.clone(),
        name: record.name.clone(),
        status: "missing".to_string(),
        action: action.to_string(),
        source_path: record.source_path.clone(),
        target_path: None,
        payload_text: None,
        auth_status: if action == "install-host-cli" {
            host_cli_auth_status(&record.name)
        } else {
            None
        },
        auth_check: if action == "install-host-cli" {
            host_cli_auth_check(&record.name)
        } else {
            None
        },
        reason: reason.to_string(),
    }
}

fn capability_materialization_paths(
    output: &Path,
    record: &CapabilityRecord,
) -> (PathBuf, PathBuf) {
    let project_root =
        infer_bundle_project_root(output).or_else(|| output.parent().map(Path::to_path_buf));
    let path = PathBuf::from(&record.source_path);
    let target = if path.is_absolute() {
        path.clone()
    } else if let Some(root) = project_root.as_ref() {
        root.join(&path)
    } else {
        output.join(&path)
    };
    let source = if let Ok(stripped) = path.strip_prefix(".memd") {
        output.join(stripped)
    } else if path.is_absolute() {
        path
    } else {
        output.join(&path)
    };
    (source, target)
}

const CAPABILITY_PAYLOAD_TEXT_PREFIX: &str = "memd:payload-text:";
const CAPABILITY_PAYLOAD_FILE_JSON_PREFIX: &str = "memd:payload-file-json:";
const HOST_CLI_INSTALL_PLAN_PREFIX: &str = "memd:host-cli-install-plan:";
const HOST_CLI_AUTH_STATUS_PREFIX: &str = "memd:host-auth-status:";
const HOST_CLI_AUTH_CHECK_PREFIX: &str = "memd:host-auth-check:";
const HOST_CLI_AUTH_PROOF_PREFIX: &str = "memd:host-auth-proof:";
const HOST_CLI_AUTH_OUTPUT_STORED_PREFIX: &str = "memd:host-auth-output-stored:";
const HOST_CLI_PATH_STATUS_PREFIX: &str = "memd:host-cli-path-status:";

fn capability_payload_text(record: &CapabilityRecord) -> Option<&str> {
    record
        .notes
        .iter()
        .find_map(|note| note.strip_prefix(CAPABILITY_PAYLOAD_TEXT_PREFIX))
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct CapabilityPayloadFile {
    path: String,
    content: String,
}

fn capability_payload_files(record: &CapabilityRecord) -> Vec<CapabilityPayloadFile> {
    record
        .notes
        .iter()
        .filter_map(|note| note.strip_prefix(CAPABILITY_PAYLOAD_FILE_JSON_PREFIX))
        .filter_map(|payload| serde_json::from_str::<CapabilityPayloadFile>(payload).ok())
        .filter(|payload| safe_relative_payload_path(&payload.path).is_some())
        .collect()
}

fn capability_payload_file_set_text(record: &CapabilityRecord) -> Option<String> {
    let files = capability_payload_files(record);
    if files.is_empty() {
        None
    } else {
        serde_json::to_string(&files).ok()
    }
}

fn safe_relative_payload_path(path: &str) -> Option<PathBuf> {
    let path = Path::new(path);
    if path.is_absolute() {
        return None;
    }
    let mut safe = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(part) => safe.push(part),
            _ => return None,
        }
    }
    (!safe.as_os_str().is_empty()).then_some(safe)
}

fn host_cli_install_plan_text(record: &CapabilityRecord) -> Option<&str> {
    if record.portability_class != "host-local" && record.kind != "cli" {
        return None;
    }
    record
        .notes
        .iter()
        .find_map(|note| note.strip_prefix(HOST_CLI_INSTALL_PLAN_PREFIX))
}

pub(crate) fn annotate_capability_registry_host_cli_auth_notes(registry: &mut CapabilityRegistry) {
    for capability in &mut registry.capabilities {
        annotate_host_cli_auth_notes(capability);
    }
}

fn annotate_host_cli_auth_notes(record: &mut CapabilityRecord) {
    if record.portability_class != "host-local" || record.kind != "cli" {
        return;
    }
    let Some(check) = host_cli_auth_check(&record.name) else {
        return;
    };
    let status = host_cli_auth_status(&record.name).unwrap_or_else(|| "unknown".to_string());
    record.notes.retain(|note| {
        !note.starts_with(HOST_CLI_AUTH_STATUS_PREFIX)
            && !note.starts_with(HOST_CLI_AUTH_CHECK_PREFIX)
            && !note.starts_with(HOST_CLI_AUTH_PROOF_PREFIX)
            && !note.starts_with(HOST_CLI_AUTH_OUTPUT_STORED_PREFIX)
            && !note.starts_with(HOST_CLI_PATH_STATUS_PREFIX)
    });
    record
        .notes
        .push(format!("{HOST_CLI_AUTH_STATUS_PREFIX}{status}"));
    record
        .notes
        .push(format!("{HOST_CLI_AUTH_CHECK_PREFIX}{check}"));
    record.notes.push(format!(
        "{HOST_CLI_AUTH_PROOF_PREFIX}{}",
        host_cli_auth_proof()
    ));
    record
        .notes
        .push(format!("{HOST_CLI_AUTH_OUTPUT_STORED_PREFIX}false"));
    record.notes.push(format!(
        "{HOST_CLI_PATH_STATUS_PREFIX}{}",
        host_cli_path_status(&record.name)
    ));
}

fn host_cli_install_plan_target_path(output: &Path, record: &CapabilityRecord) -> PathBuf {
    output
        .join("install")
        .join("host-cli")
        .join(format!("{}.sh", sanitize_capability_filename(&record.name)))
}

fn host_cli_available_on_path(name: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|path| {
        std::env::split_paths(&path)
            .map(|dir| dir.join(name))
            .any(|candidate| candidate.is_file())
    })
}

fn host_cli_path_status(name: &str) -> &'static str {
    if host_cli_available_on_path(name) {
        "on-path"
    } else {
        "missing"
    }
}

fn sanitize_capability_filename(name: &str) -> String {
    let mut out = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();
    while out.contains("--") {
        out = out.replace("--", "-");
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "cli".to_string()
    } else {
        trimmed.to_string()
    }
}

fn capability_payload_target_path(output: &Path, record: &CapabilityRecord) -> PathBuf {
    let path = PathBuf::from(&record.source_path);
    if path.is_absolute() {
        for anchor in [
            ".codex",
            ".claude",
            ".agents",
            ".config",
            ".opencode",
            ".openclaw",
        ] {
            if let Some(relative) = suffix_after_component(&path, anchor)
                && let Some(home) = home_dir()
            {
                return home.join(anchor).join(relative);
            }
        }
    }
    capability_materialization_paths(output, record).1
}

fn suffix_after_component(path: &Path, component: &str) -> Option<PathBuf> {
    let mut seen = false;
    let mut suffix = PathBuf::new();
    for part in path.components() {
        let text = part.as_os_str().to_string_lossy();
        if seen {
            suffix.push(part.as_os_str());
        } else if text == component {
            seen = true;
        }
    }
    seen.then_some(suffix)
}

fn apply_capability_materialization(
    action: &mut CapabilityMaterializationAction,
) -> anyhow::Result<bool> {
    if action.action == "write-host-cli-install-plan" {
        let Some(target) = action.target_path.as_ref().map(PathBuf::from) else {
            return Ok(false);
        };
        let Some(payload) = action.payload_text.as_deref() else {
            action.reason = "host CLI install plan payload is missing".to_string();
            return Ok(false);
        };
        let changed =
            !target.is_file() || fs::read_to_string(&target).ok().as_deref() != Some(payload);
        if changed {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            fs::write(&target, payload)
                .with_context(|| format!("write host CLI install plan {}", target.display()))?;
        }
        set_host_cli_install_plan_executable(&target)?;
        action.status = "installer-ready".to_string();
        action.reason = if changed {
            host_cli_reason(
                &action.name,
                "wrote machine-approved host CLI install plan; run it with MEMD_HOST_CLI_INSTALL_APPROVED=1 to install on this machine, then authenticate and rerun capability sync",
            )
        } else {
            host_cli_reason(
                &action.name,
                "host CLI install plan already materialized; run it with MEMD_HOST_CLI_INSTALL_APPROVED=1 to install on this machine, then authenticate and rerun capability sync",
            )
        };
        if host_cli_install_approved() {
            match run_host_cli_install_plan(&target) {
                Ok(_) if host_cli_available_on_path(&action.name) => {
                    action.status = "present".to_string();
                    action.reason = host_cli_reason(
                        &action.name,
                        &format!(
                            "host CLI installer ran and {} is now available on PATH; authenticate if needed, then rerun capability sync",
                            action.name
                        ),
                    );
                    return Ok(true);
                }
                Ok(output) => {
                    action.status = "install-failed".to_string();
                    action.reason = format!(
                        "host CLI installer ran but {} is still missing on PATH: {}",
                        action.name,
                        compact_command_output(&output)
                    );
                    return Ok(true);
                }
                Err(error) => {
                    action.status = "install-failed".to_string();
                    action.reason = format!("host CLI installer failed: {error:#}");
                    return Ok(true);
                }
            }
        }
        return Ok(changed);
    }
    if action.action == "restore-from-payload" {
        let Some(target) = action.target_path.as_ref().map(PathBuf::from) else {
            return Ok(false);
        };
        let Some(payload) = action.payload_text.as_deref() else {
            action.status = "missing".to_string();
            action.reason = "server payload is missing".to_string();
            return Ok(false);
        };
        if target.is_file() && fs::read_to_string(&target).ok().as_deref() == Some(payload) {
            action.status = "present".to_string();
            action.reason = "payload already materialized".to_string();
            return Ok(false);
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        fs::write(&target, payload)
            .with_context(|| format!("restore payload to {}", target.display()))?;
        action.status = "present".to_string();
        action.reason = "restored from server-synced text payload".to_string();
        return Ok(true);
    }
    if action.action == "restore-from-payload-set" {
        let Some(target_root) = action.target_path.as_ref().map(PathBuf::from) else {
            return Ok(false);
        };
        let Some(payload) = action.payload_text.as_deref() else {
            action.status = "missing".to_string();
            action.reason = "server payload set is missing".to_string();
            return Ok(false);
        };
        let files = serde_json::from_str::<Vec<CapabilityPayloadFile>>(payload)
            .context("parse capability payload set")?;
        let mut changed = false;
        let mut restored = 0usize;
        for file in files {
            let Some(relative) = safe_relative_payload_path(&file.path) else {
                continue;
            };
            let target = target_root.join(relative);
            if target.is_file()
                && fs::read_to_string(&target).ok().as_deref() == Some(file.content.as_str())
            {
                continue;
            }
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            fs::write(&target, file.content)
                .with_context(|| format!("restore payload to {}", target.display()))?;
            changed = true;
            restored += 1;
        }
        action.status = "present".to_string();
        action.reason = if restored == 0 {
            "payload set already materialized".to_string()
        } else {
            format!("restored {restored} files from server-synced text payload set")
        };
        return Ok(changed);
    }
    if action.action != "restore-from-bundle" {
        return Ok(false);
    }
    let Some(target) = action.target_path.as_ref().map(PathBuf::from) else {
        return Ok(false);
    };
    let source = PathBuf::from(&action.source_path);
    if !source.is_file() {
        action.status = "missing".to_string();
        action.reason = "bundle source asset is missing".to_string();
        return Ok(false);
    }
    if target.is_file() && fs::read(&source)? == fs::read(&target)? {
        action.status = "present".to_string();
        action.reason = "asset already materialized".to_string();
        return Ok(false);
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::copy(&source, &target)
        .with_context(|| format!("restore {} to {}", source.display(), target.display()))?;
    action.status = "present".to_string();
    action.reason = "restored from memd bundle state".to_string();
    Ok(true)
}

#[cfg(unix)]
fn set_host_cli_install_plan_executable(path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)
        .with_context(|| format!("stat host CLI install plan {}", path.display()))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .with_context(|| format!("chmod host CLI install plan {}", path.display()))
}

#[cfg(not(unix))]
fn set_host_cli_install_plan_executable(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}

fn host_cli_install_approved() -> bool {
    std::env::var("MEMD_HOST_CLI_INSTALL_APPROVED").as_deref() == Ok("1")
}

fn run_host_cli_install_plan(path: &Path) -> anyhow::Result<std::process::Output> {
    std::process::Command::new("sh")
        .arg(path)
        .output()
        .with_context(|| format!("run host CLI install plan {}", path.display()))
}

fn compact_command_output(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let text = format!(
        "exit={} stdout={} stderr={}",
        output.status.code().unwrap_or(-1),
        stdout.trim(),
        stderr.trim()
    );
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn host_cli_reason(name: &str, base: &str) -> String {
    match host_cli_auth_check_hint(name) {
        Some(hint) => format!(
            "{base}; auth_status={}; auth_check={hint}",
            host_cli_auth_status(name).unwrap_or_else(|| "unknown".to_string())
        ),
        None => base.to_string(),
    }
}

fn host_cli_auth_status(name: &str) -> Option<String> {
    host_cli_auth_check_hint(name)?;
    if !host_cli_available_on_path(name) {
        return Some("unknown".to_string());
    }
    let Some(args) = host_cli_auth_probe_args(name) else {
        return Some("unknown".to_string());
    };
    if !env_flag_enabled("MEMD_CAPABILITIES_AUTH_PROBE") {
        return Some("unknown".to_string());
    }
    match std::process::Command::new(name).args(args).output() {
        Ok(output) if output.status.success() => Some("authenticated".to_string()),
        Ok(_) => Some("unauthenticated".to_string()),
        Err(_) => Some("unknown".to_string()),
    }
}

fn host_cli_auth_proof() -> &'static str {
    if env_flag_enabled("MEMD_CAPABILITIES_AUTH_PROBE") {
        "local-probe"
    } else {
        "probe-skipped"
    }
}

fn host_cli_auth_check(name: &str) -> Option<String> {
    host_cli_auth_check_hint(name).map(str::to_string)
}

fn host_cli_auth_check_hint(name: &str) -> Option<&'static str> {
    match name {
        "codex" => Some("open Codex on this machine and confirm account/plugin access"),
        "gh" => Some("gh auth status"),
        "opencode" => {
            Some("opencode auth status or open OpenCode and confirm configured provider access")
        }
        "claude" => Some("claude /login or run Claude Code and confirm account access"),
        "wrangler" => Some("wrangler whoami"),
        "supabase" => Some("supabase login status or run supabase projects list"),
        _ => None,
    }
}

fn host_cli_auth_probe_args(name: &str) -> Option<&'static [&'static str]> {
    match name {
        "gh" => Some(&["auth", "status"]),
        "opencode" => Some(&["auth", "status"]),
        "wrangler" => Some(&["whoami"]),
        "supabase" => Some(&["projects", "list"]),
        _ => None,
    }
}

fn is_bundle_relative_capability(record: &CapabilityRecord) -> bool {
    record.source_path.starts_with(".memd/")
        || record.source_path.starts_with("agents/")
        || (record.harness == "project" && is_universal_class(&record.portability_class))
}

fn capabilities_output(args: &CapabilitiesArgs) -> &Path {
    match &args.command {
        Some(CapabilitiesSubcommand::Pull(args)) => &args.output,
        Some(CapabilitiesSubcommand::Status(args)) => &args.output,
        Some(CapabilitiesSubcommand::Sync(args)) => &args.output,
        None => &args.output,
    }
}

fn capabilities_command_label(args: &CapabilitiesArgs) -> &'static str {
    match &args.command {
        Some(CapabilitiesSubcommand::Pull(_)) => "pull",
        Some(CapabilitiesSubcommand::Status(_)) => "status",
        Some(CapabilitiesSubcommand::Sync(_)) => "sync",
        None => "list",
    }
}

fn capabilities_materialize_plan(args: &CapabilitiesArgs) -> bool {
    args.materialize_plan
        || matches!(
            &args.command,
            Some(CapabilitiesSubcommand::Pull(pull)) if pull.materialize_plan
        )
}

fn capabilities_materialize(args: &CapabilitiesArgs) -> bool {
    args.materialize
        || matches!(
            &args.command,
            Some(CapabilitiesSubcommand::Pull(pull)) if pull.materialize
        )
}
