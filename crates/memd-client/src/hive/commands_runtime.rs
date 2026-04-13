use super::*;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HiveWireResponse {
    pub(crate) action: String,
    pub(crate) output: String,
    pub(crate) project_root: Option<String>,
    pub(crate) agent: String,
    pub(crate) bundle_session: Option<String>,
    pub(crate) live_session: Option<String>,
    pub(crate) rebased_from_session: Option<String>,
    pub(crate) session: Option<String>,
    pub(crate) tab_id: Option<String>,
    pub(crate) hive_system: Option<String>,
    pub(crate) hive_role: Option<String>,
    pub(crate) hive_groups: Vec<String>,
    pub(crate) hive_group_goal: Option<String>,
    pub(crate) authority: Option<String>,
    pub(crate) lane_rerouted: bool,
    pub(crate) lane_created: bool,
    pub(crate) lane_surface: Option<serde_json::Value>,
    pub(crate) heartbeat: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HiveProjectResponse {
    pub(crate) action: String,
    pub(crate) output: String,
    pub(crate) project_root: Option<String>,
    pub(crate) enabled: bool,
    pub(crate) anchor: Option<String>,
    pub(crate) joined_at: Option<DateTime<Utc>>,
    pub(crate) live_session: Option<String>,
    pub(crate) heartbeat: Option<serde_json::Value>,
}

pub(crate) fn hive_follow_overlap_risk(
    output: &Path,
    current_session: Option<&str>,
    visible_entries: &[&ProjectAwarenessEntry],
    target: &ProjectAwarenessEntry,
) -> Option<String> {
    if current_session.is_some() && target.session.as_deref() == current_session {
        return None;
    }

    let lane = detect_bundle_lane_identity(output)?;
    let current_bundle = fs::canonicalize(output).unwrap_or_else(|_| output.to_path_buf());
    if awareness_entry_has_same_lane(target, &lane, &current_bundle, current_session) {
        return Some(format!(
            "unsafe hive cowork target collision: {}",
            render_hive_lane_collision(target)
        ));
    }

    let current_entry = current_session.and_then(|session| {
        visible_entries
            .iter()
            .copied()
            .find(|entry| entry.session.as_deref() == Some(session))
    })?;
    if let Some(reason) = confirmed_hive_overlap_reason(
        target,
        current_entry.task_id.as_deref(),
        current_entry.topic_claim.as_deref(),
        &current_entry.scope_claims,
    ) {
        return Some(reason);
    }

    let current_touches = awareness_overlap_touch_points(current_entry);
    let target_touches = awareness_overlap_touch_points(target);
    let shared = current_touches
        .iter()
        .filter(|touch| target_touches.iter().any(|other| other == *touch))
        .cloned()
        .collect::<Vec<_>>();
    if shared.is_empty() {
        None
    } else {
        Some(format!(
            "possible_work_overlap touches={}",
            shared.join(",")
        ))
    }
}

pub(crate) fn infer_service_agent_from_path(path: &Path) -> Option<String> {
    let normalized = path
        .file_name()
        .and_then(|value| value.to_str())
        .map(|value| value.trim().to_ascii_lowercase())?;
    if normalized.contains("agent-secrets") {
        Some("agent-secrets".to_string())
    } else if normalized.contains("agentshell") || normalized.contains("agent-shell") {
        Some("agent-shell".to_string())
    } else if normalized.contains("clawcontrol")
        || normalized.contains("claw-control")
        || normalized.contains("rollout")
    {
        Some("claw-control".to_string())
    } else if normalized == "workspace" {
        Some("openclaw".to_string())
    } else {
        None
    }
}

pub(crate) fn infer_worker_agent_from_env() -> Option<String> {
    [
        "MEMD_WORKER_NAME",
        "MEMD_AGENT_NAME",
        "CODEX_WORKER_NAME",
        "CLAUDE_WORKER_NAME",
    ]
    .into_iter()
    .find_map(|key| {
        std::env::var(key)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn maybe_explicit_hive_output(args: &HiveArgs) -> Option<PathBuf> {
    let default_init_output = default_init_output_path();
    let default_global_output = default_global_bundle_root();

    if args.output != default_init_output && args.output != default_global_output {
        return Some(args.output.clone());
    }

    None
}

fn resolve_hive_output_path(args: &HiveArgs, project_root: Option<&Path>) -> PathBuf {
    if let Some(explicit) = maybe_explicit_hive_output(args) {
        return explicit;
    }

    if args.global {
        return default_global_bundle_root();
    }

    if let Some(project_root) = project_root {
        return project_root.join(".memd");
    }

    default_bundle_root_path()
}

fn hive_args_to_init_args(args: &HiveArgs, agent: String) -> InitArgs {
    InitArgs {
        project: args.project.clone(),
        namespace: args.namespace.clone(),
        global: args.global,
        project_root: args.project_root.clone(),
        seed_existing: args.seed_existing,
        agent,
        session: args.session.clone(),
        tab_id: args.tab_id.clone(),
        hive_system: args.hive_system.clone(),
        hive_role: args.hive_role.clone(),
        capability: args.capability.clone(),
        hive_group: args.hive_group.clone(),
        hive_group_goal: args.hive_group_goal.clone(),
        authority: args.authority.clone(),
        output: args.output.clone(),
        base_url: resolve_hive_command_base_url(&args.base_url),
        rag_url: args.rag_url.clone(),
        route: args.route.clone(),
        intent: args.intent.clone(),
        workspace: args.workspace.clone(),
        visibility: args.visibility.clone(),
        voice_mode: Some(default_voice_mode()),
        allow_localhost_read_only_fallback: false,
        force: args.force,
    }
}

pub(crate) fn resolve_hive_command_base_url(base_url: &str) -> String {
    let requested = base_url.trim();
    if requested != SHARED_MEMD_BASE_URL {
        return requested.to_string();
    }

    read_bundle_runtime_config(&default_global_bundle_root())
        .ok()
        .flatten()
        .and_then(|runtime| runtime.base_url)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| requested.to_string())
}

pub(crate) fn default_hive_join_base_url() -> String {
    SHARED_MEMD_BASE_URL.to_string()
}

pub(crate) fn is_loopback_base_url(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() {
        return false;
    }
    let normalized = value
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    normalized.starts_with("localhost")
        || normalized.contains("127.0.0.1")
        || normalized.contains("0.0.0.0")
}

pub(crate) fn resolve_project_hive_base_url(
    runtime: Option<&BundleRuntimeConfig>,
    requested_base_url: Option<&str>,
) -> Option<String> {
    let enabled = runtime
        .map(|value| value.hive_project_enabled)
        .unwrap_or(false);
    let base_url = requested_base_url
        .and_then(|value| {
            let value = value.trim();
            if value.is_empty() { None } else { Some(value) }
        })
        .or_else(|| {
            runtime.and_then(|value| {
                value
                    .base_url
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
            })
        });

    if enabled && base_url.map(is_loopback_base_url).unwrap_or(true) {
        Some(SHARED_MEMD_BASE_URL.to_string())
    } else {
        base_url.map(str::to_string)
    }
}

pub(crate) async fn run_hive_command(args: &HiveArgs) -> anyhow::Result<HiveWireResponse> {
    let current_project_root = detect_current_project_root().ok().flatten();
    let inferred_agent = args
        .agent
        .clone()
        .or_else(infer_worker_agent_from_env)
        .or_else(|| {
            args.project_root
                .as_deref()
                .and_then(infer_service_agent_from_path)
        })
        .or_else(|| {
            current_project_root
                .as_deref()
                .and_then(infer_service_agent_from_path)
        })
        .unwrap_or_else(|| "codex".to_string());
    let init_args = hive_args_to_init_args(args, inferred_agent);
    let project_root = detect_init_project_root(&init_args)?;
    let mut output = resolve_hive_output_path(args, project_root.as_deref());
    let mut action = "updated".to_string();
    let defaults = default_hive_profile(&init_args.agent);
    let mut initial_lane_surface = None;

    if read_bundle_runtime_config(&output)?.is_none() {
        if maybe_explicit_hive_output(args).is_none()
            && let Some(project_root) = project_root.as_deref()
            && let Some(created_lane) =
                auto_create_worker_hive_lane(project_root, &output, &init_args)?
        {
            output = created_lane.output;
            initial_lane_surface = created_lane.lane_surface;
        }
        if read_bundle_runtime_config(&output)?.is_none() {
            write_init_bundle(&InitArgs {
                output: output.clone(),
                ..init_args.clone()
            })?;
        }
        action = "initialized".to_string();
    } else {
        if let Some(value) = args.project.as_deref() {
            set_bundle_project(&output, value)?;
        }
        if let Some(value) = args.namespace.as_deref() {
            set_bundle_namespace(&output, value)?;
        }
        set_bundle_agent(&output, &init_args.agent)?;
        if let Some(value) = args.session.as_deref() {
            set_bundle_session(&output, value)?;
        }
        if let Some(value) = args.tab_id.as_deref() {
            set_bundle_tab_id(&output, value)?;
        }
        if let Some(value) = args.hive_system.as_deref() {
            set_bundle_hive_system(&output, value)?;
        } else if let Some(value) = defaults.hive_system.as_deref() {
            set_bundle_hive_system(&output, value)?;
        }
        if let Some(value) = args.hive_role.as_deref() {
            set_bundle_hive_role(&output, value)?;
        } else if let Some(value) = defaults.hive_role.as_deref() {
            set_bundle_hive_role(&output, value)?;
        }
        if !args.capability.is_empty() {
            set_bundle_capabilities(&output, &args.capability)?;
        } else if !defaults.capabilities.is_empty() {
            set_bundle_capabilities(&output, &defaults.capabilities)?;
        }
        if !args.hive_group.is_empty() {
            set_bundle_hive_groups(&output, &args.hive_group)?;
        } else if !defaults.hive_groups.is_empty() {
            set_bundle_hive_groups(&output, &defaults.hive_groups)?;
        }
        if let Some(value) = args.hive_group_goal.as_deref() {
            set_bundle_hive_group_goal(&output, value)?;
        } else if let Some(value) = defaults.hive_group_goal.as_deref() {
            set_bundle_hive_group_goal(&output, value)?;
        }
        if let Some(value) = args.authority.as_deref() {
            set_bundle_authority(&output, value)?;
        } else if let Some(value) = defaults.authority.as_deref() {
            set_bundle_authority(&output, value)?;
        }
        set_bundle_base_url(&output, &args.base_url)?;
        set_bundle_route(&output, &args.route)?;
        set_bundle_intent(&output, &args.intent)?;
        if let Some(value) = args.workspace.as_deref() {
            set_bundle_workspace(&output, value)?;
        }
        if let Some(value) = args.visibility.as_deref() {
            set_bundle_visibility(&output, value)?;
        }
    }

    let runtime = read_bundle_runtime_config(&output)?
        .context("hive wiring requires a readable bundle runtime config")?;
    let lane = ensure_isolated_hive_bundle_lane(&output, &runtime).await?;
    let output = lane.output;
    let lane_surface = lane.lane_surface.or(initial_lane_surface);
    let runtime_before_overlay = read_bundle_runtime_config_raw(&output)?;
    let runtime = read_bundle_runtime_config(&output)?
        .context("reload bundle runtime config after hive lane isolation")?;
    let resolved_base_url = resolve_project_hive_base_url(Some(&runtime), Some(&args.base_url))
        .unwrap_or_else(|| args.base_url.clone());
    if runtime.base_url.as_deref() != Some(resolved_base_url.as_str()) {
        set_bundle_base_url(&output, &resolved_base_url)?;
        set_bundle_shared_authority_state(
            &output,
            &resolved_base_url,
            "hive",
            "shared authority available",
        )?;
    }
    let runtime = read_bundle_runtime_config(&output)?
        .context("reload bundle runtime config after hive wiring")?;
    let bundle_session = runtime_before_overlay
        .as_ref()
        .and_then(|config| config.session.clone());
    let live_session = runtime.session.clone();
    let rebased_from_session = match (bundle_session.as_deref(), live_session.as_deref()) {
        (Some(bundle), Some(live)) if bundle != live => Some(bundle.to_string()),
        _ => None,
    };
    if let Some(surface) = lane_surface.as_ref()
        && let Some(session) = runtime
            .session
            .as_deref()
            .filter(|value| !value.trim().is_empty())
    {
        let client = MemdClient::new(&resolved_base_url)?;
        emit_lane_surface_receipt(&client, surface, &runtime, session).await?;
    }
    propagate_hive_metadata_to_active_project_bundles(&output, &runtime, true).await?;
    let heartbeat = if args.publish_heartbeat {
        Some(serde_json::to_value(
            refresh_bundle_heartbeat(&output, None, false).await?,
        )?)
    } else {
        None
    };

    Ok(HiveWireResponse {
        action,
        output: output.display().to_string(),
        project_root: project_root.map(|value| value.display().to_string()),
        agent: runtime.agent.unwrap_or(init_args.agent),
        bundle_session,
        live_session,
        rebased_from_session,
        session: runtime.session,
        tab_id: runtime.tab_id,
        hive_system: runtime.hive_system,
        hive_role: runtime.hive_role,
        hive_groups: runtime.hive_groups,
        hive_group_goal: runtime.hive_group_goal,
        authority: runtime.authority,
        lane_rerouted: lane_surface
            .as_ref()
            .is_some_and(|surface| surface.action == "auto_reroute"),
        lane_created: lane_surface.is_some(),
        lane_surface: lane_surface
            .as_ref()
            .map(|surface| serde_json::to_value(surface).unwrap_or(JsonValue::Null)),
        heartbeat,
    })
}

pub(crate) async fn propagate_hive_metadata_to_active_project_bundles(
    output: &Path,
    runtime: &BundleRuntimeConfig,
    _refresh_worker_profiles: bool,
) -> anyhow::Result<()> {
    let Some(project) = runtime.project.as_deref() else {
        return Ok(());
    };
    let awareness = read_project_awareness_local(&AwarenessArgs {
        output: output.to_path_buf(),
        root: None,
        include_current: true,
        summary: false,
    })?;

    let current_bundle = fs::canonicalize(output).unwrap_or_else(|_| output.to_path_buf());
    for entry in awareness.entries {
        if entry.presence != "active" {
            continue;
        }
        if entry.project.as_deref() != Some(project) {
            continue;
        }
        if entry.namespace != runtime.namespace {
            continue;
        }
        if entry.workspace != runtime.workspace {
            continue;
        }
        let bundle_root = PathBuf::from(&entry.bundle_root);
        let canonical_bundle = fs::canonicalize(&bundle_root).unwrap_or(bundle_root.clone());
        if canonical_bundle == current_bundle {
            continue;
        }
        if !bundle_root.join("config.json").exists() {
            continue;
        }

        if let Some(value) = runtime.project.as_deref() {
            set_bundle_project(&bundle_root, value)?;
        }
        if let Some(value) = runtime.namespace.as_deref() {
            set_bundle_namespace(&bundle_root, value)?;
        }
        if let Some(value) = runtime.hive_system.as_deref() {
            set_bundle_hive_system(&bundle_root, value)?;
        }
        if let Some(value) = runtime.hive_role.as_deref() {
            set_bundle_hive_role(&bundle_root, value)?;
        }
        set_bundle_hive_groups(&bundle_root, &runtime.hive_groups)?;
        if let Some(value) = runtime.authority.as_deref() {
            set_bundle_authority(&bundle_root, value)?;
        }
        if let Some(value) = runtime.base_url.as_deref() {
            set_bundle_base_url(&bundle_root, value)?;
        }
        if let Some(value) = runtime.workspace.as_deref() {
            set_bundle_workspace(&bundle_root, value)?;
        }
        if let Some(value) = runtime.visibility.as_deref() {
            set_bundle_visibility(&bundle_root, value)?;
        }
        if let Some(mut heartbeat) = read_bundle_heartbeat(&bundle_root)? {
            heartbeat.project = runtime.project.clone();
            heartbeat.namespace = runtime.namespace.clone();
            heartbeat.hive_system = runtime.hive_system.clone();
            heartbeat.hive_role = runtime.hive_role.clone();
            heartbeat.hive_groups = runtime.hive_groups.clone();
            heartbeat.authority = runtime.authority.clone();
            heartbeat.base_url = runtime.base_url.clone();
            heartbeat.workspace = runtime.workspace.clone();
            heartbeat.visibility = runtime.visibility.clone();
            fs::write(
                bundle_heartbeat_state_path(&bundle_root),
                serde_json::to_string_pretty(&heartbeat)? + "\n",
            )
            .with_context(|| {
                format!(
                    "write {}",
                    bundle_heartbeat_state_path(&bundle_root).display()
                )
            })?;
        }
        write_agent_profiles(&bundle_root)?;
    }

    Ok(())
}

pub(crate) async fn run_hive_project_command(
    args: &HiveProjectArgs,
) -> anyhow::Result<HiveProjectResponse> {
    let runtime = read_bundle_runtime_config(&args.output)?
        .context("hive-project requires a readable bundle runtime config")?;
    let project = runtime.project.as_deref().or_else(|| {
        args.output
            .parent()
            .and_then(|value| value.file_name())
            .and_then(|value| value.to_str())
    });
    let anchor = project_hive_group(project);
    let mut action = "status".to_string();

    if args.enable {
        let Some(anchor) = anchor.as_deref() else {
            anyhow::bail!("hive-project enable requires a project name");
        };
        set_bundle_hive_project_state(&args.output, true, Some(anchor), Some(Utc::now()))?;
        let runtime_after_enable = read_bundle_runtime_config(&args.output)?
            .context("reload bundle runtime config after hive-project enable")?;
        if let Some(shared_base_url) = resolve_project_hive_base_url(
            Some(&runtime_after_enable),
            runtime_after_enable.base_url.as_deref(),
        )
            && shared_base_url != runtime_after_enable.base_url.as_deref().unwrap_or_default() {
                set_bundle_base_url(&args.output, &shared_base_url)?;
                set_bundle_shared_authority_state(
                    &args.output,
                    &shared_base_url,
                    "hive-project",
                    "shared authority available",
                )?;
            }
        write_agent_profiles(&args.output)?;
        action = "enabled".to_string();
    } else if args.disable {
        clear_bundle_hive_project_state(&args.output)?;
        write_agent_profiles(&args.output)?;
        action = "disabled".to_string();
    }

    let runtime = read_bundle_runtime_config(&args.output)?
        .context("reload bundle runtime config after hive-project update")?;
    let heartbeat = if runtime.session.is_some() {
        Some(serde_json::to_value(
            refresh_bundle_heartbeat(&args.output, None, false).await?,
        )?)
    } else {
        None
    };

    Ok(HiveProjectResponse {
        action,
        output: args.output.display().to_string(),
        project_root: detect_current_project_root()
            .ok()
            .flatten()
            .map(|value| value.display().to_string()),
        enabled: runtime.hive_project_enabled,
        anchor: runtime.hive_project_anchor,
        joined_at: runtime.hive_project_joined_at,
        live_session: runtime.session,
        heartbeat,
    })
}

pub(crate) async fn run_hive_join_command(args: &HiveJoinArgs) -> anyhow::Result<HiveJoinResponse> {
    if args.all_local {
        run_hive_join_all_local_command(args).await
    } else if args.all_active {
        run_hive_join_all_active_command(args).await
    } else {
        let runtime = read_bundle_runtime_config(&args.output)?
            .context("hive join requires a readable bundle runtime config")?;
        let join_base_url = resolve_hive_join_base_url(Some(&runtime), &args.base_url);
        let response =
            join_hive_bundle(&args.output, &join_base_url, args.publish_heartbeat).await?;
        Ok(HiveJoinResponse::Single(response))
    }
}

async fn run_hive_join_all_local_command(args: &HiveJoinArgs) -> anyhow::Result<HiveJoinResponse> {
    let (current_bundle, _, _) = resolve_awareness_paths(&AwarenessArgs {
        output: args.output.clone(),
        root: None,
        include_current: true,
        summary: false,
    })?;
    let current_runtime = read_bundle_runtime_config_raw(&current_bundle)?
        .context("hive join requires a readable current bundle runtime config")?;
    let join_base_url = resolve_hive_join_base_url(Some(&current_runtime), &args.base_url);
    let awareness = read_project_awareness_local(&AwarenessArgs {
        output: current_bundle.clone(),
        root: None,
        include_current: true,
        summary: false,
    })?;
    let target_project = current_runtime.project.clone();
    let target_namespace = current_runtime.namespace.clone();

    let mut bundles = std::collections::BTreeSet::<PathBuf>::new();
    for entry in awareness.entries {
        if target_project.as_deref() != entry.project.as_deref() {
            continue;
        }
        if target_namespace.as_deref() != entry.namespace.as_deref() {
            continue;
        }
        if entry.bundle_root.starts_with("remote:") {
            continue;
        }
        bundles.insert(PathBuf::from(entry.bundle_root));
    }
    bundles.insert(current_bundle);

    let mut joined = Vec::new();
    for bundle in bundles {
        let response = join_hive_bundle(&bundle, &join_base_url, args.publish_heartbeat).await?;
        joined.push(response);
    }

    Ok(HiveJoinResponse::Batch(HiveJoinBatchResponse {
        base_url: join_base_url,
        joined,
        mode: "all-local".to_string(),
    }))
}

async fn run_hive_join_all_active_command(args: &HiveJoinArgs) -> anyhow::Result<HiveJoinResponse> {
    let (current_bundle, _, _) = resolve_awareness_paths(&AwarenessArgs {
        output: args.output.clone(),
        root: None,
        include_current: true,
        summary: false,
    })?;
    let current_runtime = read_bundle_runtime_config_raw(&current_bundle)?
        .context("hive join requires a readable current bundle runtime config")?;
    let join_base_url = resolve_hive_join_base_url(Some(&current_runtime), &args.base_url);
    let awareness = read_project_awareness_local(&AwarenessArgs {
        output: current_bundle.clone(),
        root: None,
        include_current: true,
        summary: false,
    })?;
    let target_project = current_runtime.project.clone();
    let target_namespace = current_runtime.namespace.clone();

    let mut bundles = std::collections::BTreeSet::<PathBuf>::new();
    for entry in awareness.entries {
        if entry.presence != "active" {
            continue;
        }
        if target_project.as_deref() != entry.project.as_deref() {
            continue;
        }
        if target_namespace.as_deref() != entry.namespace.as_deref() {
            continue;
        }
        if entry.bundle_root.starts_with("remote:") {
            continue;
        }
        bundles.insert(PathBuf::from(entry.bundle_root));
    }
    bundles.insert(current_bundle);

    let mut joined = Vec::new();
    for bundle in bundles {
        let response = join_hive_bundle(&bundle, &join_base_url, args.publish_heartbeat).await?;
        joined.push(response);
    }

    Ok(HiveJoinResponse::Batch(HiveJoinBatchResponse {
        base_url: join_base_url,
        joined,
        mode: "all-active".to_string(),
    }))
}

fn resolve_hive_join_base_url(runtime: Option<&BundleRuntimeConfig>, base_url: &str) -> String {
    resolve_project_hive_base_url(runtime, Some(base_url)).unwrap_or_else(|| {
        let requested = base_url.trim();
        if requested.is_empty() {
            SHARED_MEMD_BASE_URL.to_string()
        } else {
            requested.to_string()
        }
    })
}

async fn join_hive_bundle(
    output: &Path,
    base_url: &str,
    publish_heartbeat: bool,
) -> anyhow::Result<HiveJoinBundleResponse> {
    let runtime = read_bundle_runtime_config_raw(output)?
        .context("hive join requires a readable bundle runtime config")?;
    let lane = ensure_isolated_hive_bundle_lane(output, &runtime).await?;
    let output = lane.output;
    let runtime = read_bundle_runtime_config_raw(&output)?
        .context("reload bundle runtime config after hive lane isolation")?;
    let join_base_url = resolve_hive_join_base_url(Some(&runtime), base_url);

    set_bundle_base_url(&output, &join_base_url)?;
    set_bundle_shared_authority_state(
        &output,
        &join_base_url,
        "hive-join",
        "shared authority available",
    )?;
    if let Some(project) = runtime.project.as_deref() {
        set_bundle_project(&output, project)?;
    }
    if let Some(namespace) = runtime.namespace.as_deref() {
        set_bundle_namespace(&output, namespace)?;
    }
    if let Some(agent) = runtime.agent.as_deref() {
        set_bundle_agent(&output, agent)?;
    }
    if let Some(session) = runtime.session.as_deref() {
        set_bundle_session(&output, session)?;
    }
    if let Some(tab_id) = runtime.tab_id.as_deref() {
        set_bundle_tab_id(&output, tab_id)?;
    }
    set_bundle_hive_groups(&output, &runtime.hive_groups)?;
    if let Some(goal) = runtime.hive_group_goal.as_deref() {
        set_bundle_hive_group_goal(&output, goal)?;
    }
    if let Some(authority) = runtime.authority.as_deref() {
        set_bundle_authority(&output, authority)?;
    }
    if let Some(route) = runtime.route.as_deref() {
        set_bundle_route(&output, route)?;
    }
    if let Some(intent) = runtime.intent.as_deref() {
        set_bundle_intent(&output, intent)?;
    }
    if let Some(workspace) = runtime.workspace.as_deref() {
        set_bundle_workspace(&output, workspace)?;
    }
    if let Some(visibility) = runtime.visibility.as_deref() {
        set_bundle_visibility(&output, visibility)?;
    }
    write_agent_profiles(&output)?;

    let heartbeat = if publish_heartbeat {
        Some(serde_json::to_value(
            refresh_bundle_heartbeat(&output, None, false).await?,
        )?)
    } else {
        None
    };
    let joined_runtime = read_bundle_runtime_config_raw(&output)?
        .context("reload bundle runtime config after hive join")?;
    if let Some(surface) = lane.lane_surface.as_ref()
        && let Some(session) = joined_runtime
            .session
            .as_deref()
            .filter(|value| !value.trim().is_empty())
    {
        let client = MemdClient::new(&join_base_url)?;
        emit_lane_surface_receipt(&client, surface, &joined_runtime, session).await?;
    }

    Ok(HiveJoinBundleResponse {
        output: output.display().to_string(),
        base_url: join_base_url,
        project: joined_runtime.project,
        namespace: joined_runtime.namespace,
        agent: joined_runtime.agent,
        session: joined_runtime.session,
        tab_id: joined_runtime.tab_id,
        hive_system: joined_runtime.hive_system,
        hive_role: joined_runtime.hive_role,
        hive_groups: joined_runtime.hive_groups,
        hive_group_goal: joined_runtime.hive_group_goal,
        authority: joined_runtime.authority,
        lane_rerouted: lane
            .lane_surface
            .as_ref()
            .is_some_and(|surface| surface.action == "auto_reroute"),
        lane_created: lane.lane_surface.is_some(),
        lane_surface: lane
            .lane_surface
            .as_ref()
            .map(|surface| serde_json::to_value(surface).unwrap_or(JsonValue::Null)),
        heartbeat,
    })
}
