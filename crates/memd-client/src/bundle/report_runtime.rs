use super::*;

pub(crate) fn summarize_evolution_status(
    output: &Path,
) -> anyhow::Result<Option<serde_json::Value>> {
    let proposal = read_latest_evolution_proposal(output)?;
    let branch_manifest = read_latest_evolution_branch_manifest(output)?;
    let authority = read_evolution_authority_ledger(output)?;
    let merge_queue = read_evolution_merge_queue(output)?;
    let durability_queue = read_evolution_durability_queue(output)?;

    if proposal.is_none()
        && branch_manifest.is_none()
        && authority.is_none()
        && merge_queue.is_none()
        && durability_queue.is_none()
    {
        return Ok(None);
    }

    Ok(Some(serde_json::json!({
        "proposal_state": proposal.as_ref().map(|value| value.state.clone()).unwrap_or_else(|| "none".to_string()),
        "scope_class": proposal.as_ref().map(|value| value.scope_class.clone()).unwrap_or_else(|| "none".to_string()),
        "scope_gate": proposal.as_ref().map(|value| value.scope_gate.clone()).unwrap_or_else(|| "none".to_string()),
        "authority_tier": proposal
            .as_ref()
            .map(|value| value.authority_tier.clone())
            .or_else(|| authority.as_ref().and_then(|ledger| ledger.entries.last()).map(|entry| entry.authority_tier.clone()))
            .unwrap_or_else(|| "none".to_string()),
        "merge_status": merge_queue
            .as_ref()
            .and_then(|queue| queue.entries.last())
            .map(|entry| entry.status.clone())
            .unwrap_or_else(|| "none".to_string()),
        "durability_status": durability_queue
            .as_ref()
            .and_then(|queue| queue.entries.last())
            .map(|entry| entry.status.clone())
            .unwrap_or_else(|| "none".to_string()),
        "branch": proposal
            .as_ref()
            .map(|value| value.branch.clone())
            .or_else(|| branch_manifest.as_ref().map(|value| value.branch.clone()))
            .unwrap_or_else(|| "none".to_string()),
        "durable_truth": proposal.as_ref().is_some_and(|value| value.durable_truth),
    })))
}

pub(crate) fn experiment_evolution_summary(
    output: &Path,
) -> anyhow::Result<Option<ExperimentEvolutionSummary>> {
    let proposal = read_latest_evolution_proposal(output)?;
    let branch_manifest = read_latest_evolution_branch_manifest(output)?;
    let authority = read_evolution_authority_ledger(output)?;
    let merge_queue = read_evolution_merge_queue(output)?;
    let durability_queue = read_evolution_durability_queue(output)?;

    if proposal.is_none()
        && branch_manifest.is_none()
        && authority.is_none()
        && merge_queue.is_none()
        && durability_queue.is_none()
    {
        return Ok(None);
    }

    Ok(Some(ExperimentEvolutionSummary {
        proposal_state: proposal
            .as_ref()
            .map(|value| value.state.clone())
            .unwrap_or_else(|| "none".to_string()),
        scope_class: proposal
            .as_ref()
            .map(|value| value.scope_class.clone())
            .unwrap_or_else(|| "none".to_string()),
        scope_gate: proposal
            .as_ref()
            .map(|value| value.scope_gate.clone())
            .unwrap_or_else(|| "none".to_string()),
        authority_tier: proposal
            .as_ref()
            .map(|value| value.authority_tier.clone())
            .or_else(|| {
                authority
                    .as_ref()
                    .and_then(|ledger| ledger.entries.last())
                    .map(|entry| entry.authority_tier.clone())
            })
            .unwrap_or_else(|| "none".to_string()),
        merge_status: merge_queue
            .as_ref()
            .and_then(|queue| queue.entries.last())
            .map(|entry| entry.status.clone())
            .unwrap_or_else(|| "none".to_string()),
        durability_status: durability_queue
            .as_ref()
            .and_then(|queue| queue.entries.last())
            .map(|entry| entry.status.clone())
            .unwrap_or_else(|| "none".to_string()),
        branch: proposal
            .as_ref()
            .map(|value| value.branch.clone())
            .or_else(|| branch_manifest.as_ref().map(|value| value.branch.clone()))
            .unwrap_or_else(|| "none".to_string()),
        durable_truth: proposal.as_ref().is_some_and(|value| value.durable_truth),
    }))
}

pub(crate) fn read_memd_runtime_wiring() -> serde_json::Value {
    let codex = detect_codex_memd_wiring();
    let claude = detect_claude_memd_wiring();
    let openclaw = detect_openclaw_memd_wiring();
    let opencode = detect_opencode_memd_wiring();
    let claw = detect_claw_memd_wiring();
    let claude_family = detect_claude_family_memd_wiring();
    serde_json::json!({
        "codex": codex,
        "claude": claude,
        "claw": claw,
        "claude_family": claude_family,
        "openclaw": openclaw,
        "opencode": opencode,
        "all_wired": codex.get("wired").and_then(|value| value.as_bool()).unwrap_or(false)
            && claude.get("wired").and_then(|value| value.as_bool()).unwrap_or(false)
            && openclaw.get("wired").and_then(|value| value.as_bool()).unwrap_or(false)
            && opencode.get("wired").and_then(|value| value.as_bool()).unwrap_or(false),
    })
}

pub(crate) fn detect_codex_memd_wiring() -> serde_json::Value {
    let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
    let config = home.join(".codex").join("config.toml");
    let hook = home
        .join(".codex")
        .join("hooks")
        .join("memd-session-context.js");
    let skill = home
        .join(".codex")
        .join("skills")
        .join("memd")
        .join("SKILL.md");
    let hook_wired = hook.exists()
        && fs::read_to_string(&hook)
            .ok()
            .map(|content| content.contains("memd"))
            .unwrap_or(false);
    let skill_wired = skill.exists();
    serde_json::json!({
        "wired": config.exists() && hook_wired && skill_wired,
        "config": config.exists(),
        "hook": hook_wired,
        "skill": skill_wired,
    })
}

pub(crate) fn detect_claude_memd_wiring() -> serde_json::Value {
    let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
    let settings = home.join(".claude").join("settings.json");
    let hook_candidates = [
        home.join(".claude")
            .join("hooks")
            .join("gsd-session-context.js"),
        home.join(".claude")
            .join("hooks")
            .join("memd-session-context.js"),
    ];
    let hook_wired = hook_candidates.iter().any(|path| {
        path.exists()
            && fs::read_to_string(path)
                .ok()
                .map(|content| content.contains("memd"))
                .unwrap_or(false)
    });
    let skill_wired = [
        home.join(".claude").join("skills").join("memd").join("SKILL.md"),
        home.join(".claude")
            .join("skills")
            .join("memd-init")
            .join("SKILL.md"),
        home.join(".claude")
            .join("skills")
            .join("memd-reload")
            .join("SKILL.md"),
        home.join(".claude")
            .join("skills")
            .join("memd-status")
            .join("SKILL.md"),
    ]
    .iter()
    .any(|path| path.is_file());
    let command_wired = [
        home.join(".claude").join("commands").join("memd.md"),
        home.join(".claude").join("commands").join("memd").join("init.md"),
        home.join(".claude").join("commands").join("memd").join("reload.md"),
        home.join(".claude").join("commands").join("memd").join("status.md"),
        home.join(".claude").join("command").join("memd.md"),
    ]
    .iter()
    .any(|path| path.is_file());
    serde_json::json!({
        "wired": settings.exists() && hook_wired && skill_wired && command_wired,
        "settings": settings.exists(),
        "hook": hook_wired,
        "skill": skill_wired,
        "command": command_wired,
    })
}

pub(crate) fn detect_claude_family_memd_wiring() -> serde_json::Value {
    let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
    let harnesses = detect_claude_family_harness_roots(&home)
        .into_iter()
        .filter(|root| root.harness != "claude")
        .map(|root| {
            let settings = root.root.join("settings.json");
            let hook_candidates = [
                root.root.join("hooks").join("gsd-session-context.js"),
                root.root.join("hooks").join("memd-session-context.js"),
            ];
            let hook_wired = hook_candidates.iter().any(|path| {
                path.exists()
                    && fs::read_to_string(path)
                        .ok()
                        .map(|content| content.contains("memd"))
                        .unwrap_or(false)
            });
            serde_json::json!({
                "harness": root.harness,
                "root": root.root.display().to_string(),
                "wired": settings.exists() && hook_wired,
                "settings": settings.exists(),
                "hook": hook_wired,
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "count": harnesses.len(),
        "harnesses": harnesses,
    })
}

pub(crate) fn detect_openclaw_memd_wiring() -> serde_json::Value {
    let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
    let ag = home.join(".openclaw").join("workspace").join("AGENTS.md");
    let bootstrap = home
        .join(".openclaw")
        .join("workspace")
        .join("BOOTSTRAP.md");
    let ag_wired = ag.exists()
        && fs::read_to_string(&ag)
            .ok()
            .map(|content| content.contains("memd"))
            .unwrap_or(false);
    let bootstrap_wired = bootstrap.exists()
        && fs::read_to_string(&bootstrap)
            .ok()
            .map(|content| content.contains("memd"))
            .unwrap_or(false);
    serde_json::json!({
        "wired": ag_wired && bootstrap_wired,
        "agents": ag_wired,
        "bootstrap": bootstrap_wired,
    })
}

pub(crate) fn detect_opencode_memd_wiring() -> serde_json::Value {
    let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
    let config_dir = home.join(".config").join("opencode");
    let legacy_dir = home.join(".opencode");
    let config_files = [
        config_dir.join("opencode.json"),
        config_dir.join("settings.json"),
        legacy_dir.join("opencode.json"),
        legacy_dir.join("settings.json"),
    ];
    let config_exists = config_files.iter().any(|path| path.is_file());
    let config_wired = config_files.iter().any(|path| {
        path.is_file()
            && fs::read_to_string(path)
                .ok()
                .map(|content| content.contains("memd-plugin") || content.contains("\"plugin\""))
                .unwrap_or(false)
    });
    let plugin_files = [
        config_dir.join("plugins").join("memd-plugin.mjs"),
        legacy_dir.join("plugins").join("memd-plugin.mjs"),
    ];
    let plugin_wired = plugin_files.iter().any(|path| {
        path.is_file()
            && fs::read_to_string(path)
                .ok()
                .map(|content| content.contains("mem.md"))
                .unwrap_or(false)
    });
    let command_files = [
        config_dir.join("command").join("memd.md"),
        config_dir.join("commands").join("memd.md"),
        legacy_dir.join("command").join("memd.md"),
        legacy_dir.join("commands").join("memd.md"),
    ];
    let command_wired = command_files.iter().any(|path| {
        path.is_file()
            && fs::read_to_string(path)
                .ok()
                .map(|content| content.contains("memd refresh") || content.contains("memd init"))
                .unwrap_or(false)
    });
    serde_json::json!({
        "wired": config_wired && plugin_wired && command_wired,
        "config": config_wired || config_exists,
        "plugin": plugin_wired,
        "command": command_wired,
    })
}

pub(crate) fn detect_claw_memd_wiring() -> serde_json::Value {
    let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
    let config_candidates = [
        home.join(".config").join("claw").join("settings.json"),
        home.join(".claw").join("settings.json"),
        home.join(".claw.json"),
    ];
    let config_exists = config_candidates.iter().any(|path| path.is_file());
    let binary_exists = Command::new("sh")
        .arg("-lc")
        .arg("command -v claw >/dev/null 2>&1")
        .status()
        .map(|status| status.success())
        .unwrap_or(false);
    let memd_skill_visible = [
        home.join(".claw")
            .join("skills")
            .join("memd")
            .join("SKILL.md"),
        home.join(".agents")
            .join("skills")
            .join("memd")
            .join("SKILL.md"),
        home.join(".codex")
            .join("skills")
            .join("memd")
            .join("SKILL.md"),
    ]
    .iter()
    .any(|path| path.is_file());
    serde_json::json!({
        "wired": binary_exists && config_exists && memd_skill_visible,
        "binary": binary_exists,
        "config": config_exists,
        "skill": memd_skill_visible,
    })
}

pub(crate) fn read_bundle_rag_config(
    output: &Path,
) -> anyhow::Result<Option<BundleRagConfigState>> {
    let config_path = output.join("config.json");
    let resolved = if config_path.exists() {
        let raw = fs::read_to_string(&config_path)
            .with_context(|| format!("read {}", config_path.display()))?;
        let config: BundleConfigFile = serde_json::from_str(&raw)
            .with_context(|| format!("parse {}", config_path.display()))?;
        resolve_bundle_rag_config(config)
    } else {
        None
    };

    if let Some(state) = resolved.as_ref()
        && state.url.is_some() {
            return Ok(Some(state.clone()));
        }

    if let Ok(value) = std::env::var("MEMD_RAG_URL") {
        let value = value.trim().to_string();
        if !value.is_empty() {
            return Ok(Some(BundleRagConfigState {
                configured: true,
                enabled: true,
                url: Some(value),
                source: "env.MEMD_RAG_URL".to_string(),
            }));
        }
    }

    Ok(resolved)
}

pub(crate) fn read_bundle_runtime_config_raw(
    output: &Path,
) -> anyhow::Result<Option<BundleRuntimeConfig>> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let config: BundleConfigFile =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    Ok(Some(BundleRuntimeConfig {
        project: config.project,
        namespace: config.namespace,
        agent: config.agent,
        session: config.session,
        tab_id: config.tab_id.or_else(default_bundle_tab_id),
        hive_system: config.hive_system,
        hive_role: config.hive_role,
        capabilities: config.capabilities,
        hive_groups: config.hive_groups,
        hive_group_goal: config.hive_group_goal,
        authority: config.authority,
        hive_project_enabled: config.hive_project_enabled,
        hive_project_anchor: config.hive_project_anchor,
        hive_project_joined_at: config.hive_project_joined_at,
        base_url: config.base_url,
        route: config.route,
        intent: config.intent,
        workspace: config.workspace,
        visibility: config.visibility,
        heartbeat_model: config.heartbeat_model,
        voice_mode: Some(config.voice_mode.unwrap_or_else(default_voice_mode)),
        auto_short_term_capture: config.auto_short_term_capture,
        authority_policy: config.authority_policy,
        authority_state: config.authority_state,
    }))
}

pub(crate) fn resolve_project_bundle_overlay(
    output: &Path,
    current_dir: &Path,
    global_root: &Path,
) -> anyhow::Result<Option<BundleRuntimeConfig>> {
    let output = fs::canonicalize(output).unwrap_or_else(|_| output.to_path_buf());
    let global_root = fs::canonicalize(global_root).unwrap_or_else(|_| global_root.to_path_buf());
    if output != global_root {
        return Ok(None);
    }

    let local_bundle = current_dir.join(".memd");
    let local_bundle = fs::canonicalize(&local_bundle).unwrap_or(local_bundle);
    if local_bundle == output {
        return Ok(None);
    }

    read_bundle_runtime_config_raw(&local_bundle)
}

pub(crate) fn resolve_live_session_overlay(
    output: &Path,
    current_dir: &Path,
    global_root: &Path,
) -> anyhow::Result<Option<BundleRuntimeConfig>> {
    let output = fs::canonicalize(output).unwrap_or_else(|_| output.to_path_buf());
    let global_root = fs::canonicalize(global_root).unwrap_or_else(|_| global_root.to_path_buf());
    if output == global_root {
        return Ok(None);
    }

    let local_bundle = current_dir.join(".memd");
    let local_bundle = fs::canonicalize(&local_bundle).unwrap_or(local_bundle);
    if local_bundle != output {
        return Ok(None);
    }

    let Some(local_runtime) = read_bundle_runtime_config_raw(&local_bundle)? else {
        return Ok(None);
    };

    let local_workspace_scoped = local_runtime
        .workspace
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());
    let local_visibility_scoped = matches!(
        local_runtime
            .visibility
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
        Some("workspace" | "project")
    );
    if !local_runtime.hive_project_enabled && !local_workspace_scoped && !local_visibility_scoped {
        return Ok(None);
    }

    let Some(global_runtime) = read_bundle_runtime_config_raw(&global_root)? else {
        return Ok(None);
    };

    if global_runtime.session.is_none() && global_runtime.tab_id.is_none() {
        return Ok(None);
    }

    Ok(Some(BundleRuntimeConfig {
        project: None,
        namespace: None,
        agent: None,
        session: global_runtime.session,
        tab_id: global_runtime.tab_id,
        hive_system: None,
        hive_role: None,
        capabilities: Vec::new(),
        hive_groups: Vec::new(),
        hive_group_goal: None,
        authority: None,
        hive_project_enabled: false,
        hive_project_anchor: None,
        hive_project_joined_at: None,
        base_url: None,
        route: None,
        intent: None,
        workspace: None,
        visibility: None,
        heartbeat_model: None,
        voice_mode: Some(default_voice_mode()),
        auto_short_term_capture: false,
        authority_policy: BundleAuthorityPolicy::default(),
        authority_state: BundleAuthorityState::default(),
    }))
}

pub(crate) fn merge_bundle_runtime_config(
    mut runtime: BundleRuntimeConfig,
    overlay: BundleRuntimeConfig,
) -> BundleRuntimeConfig {
    if overlay.project.is_some() {
        runtime.project = overlay.project;
    }
    if overlay.namespace.is_some() {
        runtime.namespace = overlay.namespace;
    }
    if overlay.workspace.is_some() {
        runtime.workspace = overlay.workspace;
    }
    if overlay.visibility.is_some() {
        runtime.visibility = overlay.visibility;
    }
    if overlay.session.is_some() {
        runtime.session = overlay.session;
    }
    if overlay.route.is_some() {
        runtime.route = overlay.route;
    }
    if overlay.intent.is_some() {
        runtime.intent = overlay.intent;
    }
    if overlay.tab_id.is_some() {
        runtime.tab_id = overlay.tab_id;
    }
    if overlay.hive_system.is_some() {
        runtime.hive_system = overlay.hive_system;
    }
    if overlay.hive_role.is_some() {
        runtime.hive_role = overlay.hive_role;
    }
    if !overlay.capabilities.is_empty() {
        runtime.capabilities = overlay.capabilities;
    }
    if !overlay.hive_groups.is_empty() {
        runtime.hive_groups = overlay.hive_groups;
    }
    if overlay.hive_group_goal.is_some() {
        runtime.hive_group_goal = overlay.hive_group_goal;
    }
    if overlay.authority.is_some() {
        runtime.authority = overlay.authority;
    }
    runtime.hive_project_enabled = overlay.hive_project_enabled;
    if overlay.hive_project_anchor.is_some() {
        runtime.hive_project_anchor = overlay.hive_project_anchor;
    }
    if overlay.hive_project_joined_at.is_some() {
        runtime.hive_project_joined_at = overlay.hive_project_joined_at;
    }
    runtime
}

pub(crate) fn read_bundle_runtime_config(
    output: &Path,
) -> anyhow::Result<Option<BundleRuntimeConfig>> {
    let Some(mut runtime) = read_bundle_runtime_config_raw(output)? else {
        return Ok(None);
    };

    let current_dir = std::env::current_dir().context("read current directory")?;
    if let Some(overlay) =
        resolve_project_bundle_overlay(output, &current_dir, &default_global_bundle_root())?
    {
        runtime = merge_bundle_runtime_config(runtime, overlay);
    }
    if let Some(overlay) =
        resolve_live_session_overlay(output, &current_dir, &default_global_bundle_root())?
    {
        runtime = merge_bundle_runtime_config(runtime, overlay);
    }

    Ok(Some(runtime))
}

pub(crate) fn resolve_bundle_command_base_url(
    requested: &str,
    runtime_base_url: Option<&str>,
) -> String {
    let requested = requested.trim();
    if std::env::var("MEMD_BASE_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .as_deref()
        == Some(requested)
    {
        return requested.to_string();
    }

    if requested != default_base_url() {
        return requested.to_string();
    }

    runtime_base_url
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| requested.to_string())
}

pub(crate) fn runtime_prefers_shared_authority(runtime: Option<&BundleRuntimeConfig>) -> bool {
    runtime
        .map(|value| value.authority_policy.shared_primary)
        .unwrap_or(true)
}

pub(crate) fn runtime_allows_localhost_read_only(runtime: Option<&BundleRuntimeConfig>) -> bool {
    runtime
        .map(|value| {
            value.authority_policy.localhost_fallback_policy
                == LocalhostFallbackPolicy::AllowReadOnly
        })
        .unwrap_or(false)
}

pub(crate) fn authority_warning_lines(runtime: Option<&BundleRuntimeConfig>) -> Vec<String> {
    let Some(runtime) = runtime else {
        return Vec::new();
    };
    if runtime.authority_state.mode != "localhost_read_only" {
        return Vec::new();
    }

    let mut lines = vec![
        "shared authority unavailable".to_string(),
        "localhost fallback is lower trust".to_string(),
        "prompt-injection and split-brain risk increased".to_string(),
        "coordination writes blocked".to_string(),
    ];
    if let Some(reason) = runtime.authority_state.reason.as_deref() {
        lines.push(format!("reason={reason}"));
    }
    if let Some(expires_at) = runtime.authority_state.expires_at.as_ref() {
        lines.push(format!("expires_at={}", expires_at.to_rfc3339()));
    }
    lines
}

pub(crate) fn write_bundle_authority_env(
    output: &Path,
    policy: &BundleAuthorityPolicy,
    state: &BundleAuthorityState,
) -> anyhow::Result<()> {
    rewrite_shell_env(&output.join("env"), "MEMD_AUTHORITY_MODE", &state.mode)?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_AUTHORITY_MODE = ",
        &format!(
            "$env:MEMD_AUTHORITY_MODE = \"{}\"\n",
            escape_ps1(&state.mode)
        ),
    )?;
    rewrite_shell_env(&output.join("env"), "MEMD_LOCALHOST_FALLBACK_POLICY", policy.localhost_fallback_policy.as_str())?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_LOCALHOST_FALLBACK_POLICY = ",
        &format!(
            "$env:MEMD_LOCALHOST_FALLBACK_POLICY = \"{}\"\n",
            escape_ps1(policy.localhost_fallback_policy.as_str())
        ),
    )?;
    rewrite_shell_env(&output.join("env"), "MEMD_AUTHORITY_DEGRADED", if state.degraded { "true" } else { "false" })?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_AUTHORITY_DEGRADED = ",
        &format!(
            "$env:MEMD_AUTHORITY_DEGRADED = \"{}\"\n",
            if state.degraded { "true" } else { "false" }
        ),
    )?;
    if let Some(shared_base_url) = state.shared_base_url.as_deref() {
        rewrite_shell_env(&output.join("env"), "MEMD_SHARED_BASE_URL", shared_base_url)?;
        rewrite_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_SHARED_BASE_URL = ",
            &format!(
                "$env:MEMD_SHARED_BASE_URL = \"{}\"\n",
                escape_ps1(shared_base_url)
            ),
        )?;
    } else {
        remove_env_assignment(&output.join("env"), "MEMD_SHARED_BASE_URL=")?;
        remove_env_assignment(&output.join("env.ps1"), "$env:MEMD_SHARED_BASE_URL = ")?;
    }
    if let Some(fallback_base_url) = state.fallback_base_url.as_deref() {
        rewrite_shell_env(&output.join("env"), "MEMD_LOCALHOST_FALLBACK_BASE_URL", fallback_base_url)?;
        rewrite_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_LOCALHOST_FALLBACK_BASE_URL = ",
            &format!(
                "$env:MEMD_LOCALHOST_FALLBACK_BASE_URL = \"{}\"\n",
                escape_ps1(fallback_base_url)
            ),
        )?;
    } else {
        remove_env_assignment(&output.join("env"), "MEMD_LOCALHOST_FALLBACK_BASE_URL=")?;
        remove_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_LOCALHOST_FALLBACK_BASE_URL = ",
        )?;
    }
    Ok(())
}

pub(crate) fn set_bundle_shared_authority_state(
    output: &Path,
    shared_base_url: &str,
    activated_by: &str,
    reason: &str,
) -> anyhow::Result<()> {
    let (config_path, mut config) = read_bundle_config_file(output)?;
    config.authority_state.mode = "shared".to_string();
    config.authority_state.degraded = false;
    config.authority_state.shared_base_url = Some(shared_base_url.to_string());
    config.authority_state.fallback_base_url = None;
    config.authority_state.activated_at = Some(Utc::now());
    config.authority_state.activated_by = Some(activated_by.to_string());
    config.authority_state.reason = Some(reason.to_string());
    config.authority_state.warning_acknowledged_at = None;
    config.authority_state.expires_at = None;
    config.authority_state.blocked_capabilities.clear();
    write_bundle_config_file(&config_path, &config)?;
    write_bundle_authority_env(output, &config.authority_policy, &config.authority_state)?;
    Ok(())
}

pub(crate) fn set_bundle_localhost_read_only_authority_state(
    output: &Path,
    shared_base_url: &str,
    activated_by: &str,
    reason: &str,
) -> anyhow::Result<()> {
    let (config_path, mut config) = read_bundle_config_file(output)?;
    config.authority_policy.shared_primary = true;
    config.authority_policy.localhost_fallback_policy = LocalhostFallbackPolicy::AllowReadOnly;
    config.authority_state.mode = "localhost_read_only".to_string();
    config.authority_state.degraded = true;
    config.authority_state.shared_base_url = Some(shared_base_url.to_string());
    config.authority_state.fallback_base_url = Some(localhost_memd_base_url());
    config.authority_state.activated_at = Some(Utc::now());
    config.authority_state.activated_by = Some(activated_by.to_string());
    config.authority_state.reason = Some(reason.to_string());
    config.authority_state.warning_acknowledged_at = None;
    config.authority_state.expires_at = None;
    config.authority_state.blocked_capabilities = vec![
        "coordination_writes".to_string(),
        "queen_actions".to_string(),
        "shared_claim_mutations".to_string(),
        "shared_task_mutations".to_string(),
        "shared_message_mutations".to_string(),
    ];
    write_bundle_config_file(&config_path, &config)?;
    write_bundle_authority_env(output, &config.authority_policy, &config.authority_state)?;
    Ok(())
}

pub(crate) fn ensure_shared_authority_write_allowed(
    runtime: Option<&BundleRuntimeConfig>,
    operation: &str,
) -> anyhow::Result<()> {
    if runtime
        .map(|value| value.authority_state.mode.as_str() == "localhost_read_only")
        .unwrap_or(false)
    {
        anyhow::bail!(
            "localhost read-only fallback active; {} requires trusted shared authority",
            operation
        );
    }
    Ok(())
}

const LIVE_RPC_TIMEOUT: Duration = Duration::from_secs(2);

pub(crate) async fn timeout_ok<T, E, F>(future: F) -> Option<T>
where
    F: Future<Output = Result<T, E>>,
{
    tokio::time::timeout(LIVE_RPC_TIMEOUT, future)
        .await
        .ok()
        .and_then(Result::ok)
}

pub(crate) fn bundle_auto_short_term_capture_enabled(output: &Path) -> anyhow::Result<bool> {
    if let Ok(value) = std::env::var("MEMD_AUTO_SHORT_TERM_CAPTURE") {
        let value = value.trim().to_ascii_lowercase();
        return Ok(matches!(value.as_str(), "1" | "true" | "yes" | "on"));
    }

    Ok(read_bundle_runtime_config(output)?
        .map(|config| config.auto_short_term_capture)
        .unwrap_or(true))
}

#[path = "../awareness/mod.rs"]
mod awareness;
pub(crate) use awareness::*;

pub(crate) fn render_attach_snippet(shell: &str, bundle_path: &Path) -> anyhow::Result<String> {
    let shell = shell.trim().to_ascii_lowercase();
    let (startup_route, startup_intent) = bundle_startup_route_intent(bundle_path);
    let project_hive_enabled = read_bundle_runtime_config(bundle_path)
        .ok()
        .flatten()
        .map(|runtime| runtime.hive_project_enabled)
        .unwrap_or(false);
    match shell.as_str() {
        "bash" | "zsh" | "sh" => Ok(format!(
            r#"export MEMD_BUNDLE_ROOT="{bundle_path}"
set -a
source "$MEMD_BUNDLE_ROOT/env"
set +a
{base_url_block}nohup memd heartbeat --output "$MEMD_BUNDLE_ROOT" --watch --interval-secs 30 --probe-base-url >/tmp/memd-heartbeat.log 2>&1 &
memd wake --output "$MEMD_BUNDLE_ROOT" --route {startup_route} --intent {startup_intent} --write
# pre-answer durable recall:
# .memd/agents/lookup.sh --query "what did we already decide?"
"#,
            bundle_path = bundle_path.display(),
            startup_route = compact_bundle_value(&startup_route),
            startup_intent = compact_bundle_value(&startup_intent),
            base_url_block = if project_hive_enabled {
                format!(
                    "if [[ -z \"${{MEMD_BASE_URL:-}}\" || \"${{MEMD_BASE_URL}}\" =~ ^https?://(localhost|127\\.0\\.0\\.1|0\\.0\\.0\\.0)(:[0-9]+)?(/|$) ]]; then\n  export MEMD_BASE_URL=\"{}\"\nfi\n",
                    SHARED_MEMD_BASE_URL
                )
            } else {
                String::new()
            },
        )),
        "powershell" | "pwsh" => Ok(format!(
            r#"$env:MEMD_BUNDLE_ROOT = "{bundle_path}"
. (Join-Path $env:MEMD_BUNDLE_ROOT "env.ps1")
{base_url_block}Start-Process -WindowStyle Hidden -FilePath memd -ArgumentList @('heartbeat','--output',$env:MEMD_BUNDLE_ROOT,'--watch','--interval-secs','30','--probe-base-url') -RedirectStandardOutput "$env:TEMP\memd-heartbeat.log" -RedirectStandardError "$env:TEMP\memd-heartbeat.err"
memd wake --output $env:MEMD_BUNDLE_ROOT --route {startup_route} --intent {startup_intent} --write
# pre-answer durable recall:
# .memd/agents/lookup.ps1 --query "what did we already decide?"
"#,
            bundle_path = escape_ps1(&bundle_path.display().to_string()),
            startup_route = escape_ps1(&startup_route),
            startup_intent = escape_ps1(&startup_intent),
            base_url_block = if project_hive_enabled {
                format!(
                    "if ([string]::IsNullOrWhiteSpace($env:MEMD_BASE_URL) -or $env:MEMD_BASE_URL -match '^(https?://)?(localhost|127\\.0\\.0\\.1|0\\.0\\.0\\.0)(:[0-9]+)?(/|$)') {{ $env:MEMD_BASE_URL = \"{}\" }}\n",
                    escape_ps1(SHARED_MEMD_BASE_URL)
                )
            } else {
                String::new()
            },
        )),
        other => anyhow::bail!(
            "unsupported shell '{other}'; expected bash, zsh, sh, powershell, or pwsh"
        ),
    }
}
