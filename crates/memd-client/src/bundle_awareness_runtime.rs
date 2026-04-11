use super::*;

pub(crate) fn resolve_awareness_paths(
    args: &AwarenessArgs,
) -> anyhow::Result<(PathBuf, PathBuf, PathBuf)> {
    let current_bundle = if args.output.is_absolute() {
        args.output.clone()
    } else {
        std::env::current_dir()?.join(&args.output)
    };
    let current_bundle = fs::canonicalize(&current_bundle).unwrap_or(current_bundle);
    let current_project = current_bundle
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let scan_root = if let Some(root) = args.root.as_ref() {
        if root.is_absolute() {
            root.clone()
        } else {
            std::env::current_dir()?.join(root)
        }
    } else {
        current_project
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| current_project.clone())
    };
    let scan_root = fs::canonicalize(&scan_root).unwrap_or(scan_root);

    Ok((current_bundle, current_project, scan_root))
}

pub(crate) async fn read_project_awareness(
    args: &AwarenessArgs,
) -> anyhow::Result<ProjectAwarenessResponse> {
    let (current_bundle, _, _) = resolve_awareness_paths(args)?;
    let runtime = read_bundle_runtime_config(&current_bundle)?;
    if runtime
        .as_ref()
        .and_then(|config| config.session.as_deref())
        .is_some()
    {
        let _ = timeout_ok(refresh_bundle_heartbeat(&current_bundle, None, false)).await;
    }
    let include_current = args.include_current
        || runtime
            .as_ref()
            .and_then(|config| config.session.as_deref())
            .is_some();
    let local = read_project_awareness_local(&AwarenessArgs {
        include_current,
        ..args.clone()
    })?;
    if let Some(shared) = read_project_awareness_shared(
        &AwarenessArgs {
            include_current: args.include_current,
            ..args.clone()
        },
        &local,
    )
    .await?
    {
        let mut entries = merge_project_awareness_entries(local.entries, shared.entries);
        entries = suppress_superseded_awareness_entries(entries, &local.current_bundle);
        entries.sort_by(|left, right| left.bundle_root.cmp(&right.bundle_root));

        let mut collisions = local.collisions;
        for collision in shared.collisions {
            if !collisions.contains(&collision) {
                collisions.push(collision);
            }
        }

        return Ok(ProjectAwarenessResponse {
            root: shared.root,
            current_bundle: local.current_bundle,
            collisions,
            entries,
        });
    }
    Ok(local)
}

pub(crate) fn merge_project_awareness_entries(
    local_entries: Vec<ProjectAwarenessEntry>,
    shared_entries: Vec<ProjectAwarenessEntry>,
) -> Vec<ProjectAwarenessEntry> {
    let mut entries = local_entries;
    for entry in shared_entries {
        if let Some(index) = entries
            .iter()
            .position(|candidate| awareness_entries_overlap(candidate, &entry))
        {
            let preferred = prefer_project_awareness_entry(entries[index].clone(), entry);
            entries[index] = preferred;
        } else {
            entries.push(entry);
        }
    }
    entries
}

pub(crate) fn suppress_superseded_awareness_entries(
    entries: Vec<ProjectAwarenessEntry>,
    current_bundle: &str,
) -> Vec<ProjectAwarenessEntry> {
    let Some(current) = entries
        .iter()
        .find(|entry| entry.bundle_root == current_bundle && entry.presence == "active")
        .cloned()
    else {
        return entries;
    };

    entries
        .into_iter()
        .filter(|entry| !is_superseded_stale_remote_session(entry, &current))
        .collect()
}

pub(crate) fn is_superseded_stale_remote_session(
    entry: &ProjectAwarenessEntry,
    current: &ProjectAwarenessEntry,
) -> bool {
    entry.project_dir == "remote"
        && entry.presence == "stale"
        && current.presence == "active"
        && entry.session != current.session
        && entry.project == current.project
        && entry.namespace == current.namespace
        && entry.workspace == current.workspace
        && entry.agent == current.agent
        && entry.base_url == current.base_url
}

pub(crate) fn project_awareness_visible_entries<'a>(
    response: &'a ProjectAwarenessResponse,
) -> Vec<&'a ProjectAwarenessEntry> {
    let current_entry = response
        .entries
        .iter()
        .find(|candidate| candidate.bundle_root == response.current_bundle);
    response
        .entries
        .iter()
        .filter(|entry| !(entry.project_dir == "remote" && entry.presence == "dead"))
        .filter(|entry| {
            !current_entry
                .map(|current| is_superseded_stale_remote_session(entry, current))
                .unwrap_or(false)
        })
        .filter(|entry| {
            !current_entry
                .map(|current| is_shadowed_local_seen_session(entry, current))
                .unwrap_or(false)
        })
        .collect()
}

pub(crate) fn is_shadowed_local_seen_session(
    entry: &ProjectAwarenessEntry,
    current: &ProjectAwarenessEntry,
) -> bool {
    let entry_is_shadow_candidate = matches!(entry.presence.as_str(), "unknown" | "active");
    if !entry_is_shadow_candidate {
        return false;
    }
    let current_is_newer = match (entry.last_updated, current.last_updated) {
        (Some(entry_updated), Some(current_updated)) => current_updated > entry_updated,
        _ => false,
    };
    entry.bundle_root != current.bundle_root
        && entry.project_dir != "remote"
        && current.presence == "active"
        && current_is_newer
        && entry.project == current.project
        && entry.namespace == current.namespace
        && entry.workspace == current.workspace
        && entry.agent == current.agent
        && entry.base_url == current.base_url
        && entry.active_claims == 0
        && entry.hive_system.is_none()
        && entry.hive_role.is_none()
        && entry.hive_groups.is_empty()
        && entry.branch.is_none()
}

pub(crate) fn awareness_entries_overlap(
    left: &ProjectAwarenessEntry,
    right: &ProjectAwarenessEntry,
) -> bool {
    let same_session = left.session.is_some()
        && left.session == right.session
        && left.project == right.project
        && left.namespace == right.namespace
        && left.workspace == right.workspace
        && left.branch == right.branch
        && left.worktree_root == right.worktree_root;
    if same_session {
        return left.tab_id == right.tab_id || left.tab_id.is_none() || right.tab_id.is_none();
    }

    left.bundle_root == right.bundle_root
        || (left.base_url.is_some()
            && left.base_url == right.base_url
            && left.project == right.project
            && left.namespace == right.namespace
            && left.session == right.session)
}

pub(crate) fn prefer_project_awareness_entry(
    left: ProjectAwarenessEntry,
    right: ProjectAwarenessEntry,
) -> ProjectAwarenessEntry {
    if project_awareness_entry_rank(&right) > project_awareness_entry_rank(&left) {
        right
    } else {
        left
    }
}

pub(crate) fn project_awareness_entry_rank(
    entry: &ProjectAwarenessEntry,
) -> (u8, u8, u8, u8, i64, usize, usize) {
    (
        if entry.project_dir == "remote" { 0 } else { 1 },
        awareness_presence_rank(&entry.presence),
        if entry.hive_system.is_some() || entry.hive_role.is_some() {
            1
        } else {
            0
        },
        if entry.hive_groups.is_empty() { 0 } else { 1 },
        entry
            .last_updated
            .map(|value| value.timestamp())
            .unwrap_or_default(),
        entry.active_claims,
        entry.capabilities.len(),
    )
}

pub(crate) fn awareness_presence_rank(presence: &str) -> u8 {
    match presence {
        "active" => 3,
        "stale" => 2,
        "dead" => 1,
        _ => 0,
    }
}

pub(crate) fn prune_dead_local_bundle_heartbeat(
    bundle_root: &Path,
    heartbeat: Option<&BundleHeartbeatState>,
    active_claims: usize,
    is_current_bundle: bool,
    current_runtime: Option<&BundleRuntimeConfig>,
) -> anyhow::Result<bool> {
    if is_current_bundle || active_claims > 0 {
        return Ok(false);
    }
    let Some(heartbeat) = heartbeat else {
        return Ok(false);
    };
    if heartbeat_presence_label(heartbeat.last_seen) != "dead" {
        return Ok(false);
    }
    let shares_current_session = current_runtime
        .and_then(|runtime| runtime.session.as_deref())
        .zip(heartbeat.session.as_deref())
        .is_some_and(|(current_session, heartbeat_session)| current_session == heartbeat_session);
    let shares_current_tab = current_runtime
        .and_then(|runtime| runtime.tab_id.as_deref())
        .zip(heartbeat.tab_id.as_deref())
        .is_some_and(|(current_tab, heartbeat_tab)| current_tab == heartbeat_tab);
    if shares_current_session || shares_current_tab {
        return Ok(false);
    }

    let heartbeat_path = bundle_heartbeat_state_path(bundle_root);
    if heartbeat_path.exists() {
        fs::remove_file(&heartbeat_path)
            .with_context(|| format!("remove {}", heartbeat_path.display()))?;
    }
    Ok(true)
}

pub(crate) fn skip_inactive_local_bundle_entry(
    runtime: &BundleRuntimeConfig,
    heartbeat: Option<&BundleHeartbeatState>,
    state: Option<&BundleResumeState>,
    active_claims: usize,
    is_current_bundle: bool,
) -> bool {
    !is_current_bundle
        && active_claims == 0
        && heartbeat.is_none()
        && state.is_none()
        && runtime
            .session
            .as_deref()
            .map(str::trim)
            .is_none_or(|value| value.is_empty())
}

pub(crate) async fn read_project_awareness_shared(
    args: &AwarenessArgs,
    fallback: &ProjectAwarenessResponse,
) -> anyhow::Result<Option<ProjectAwarenessResponse>> {
    let (current_bundle, _, _) = resolve_awareness_paths(args)?;
    let runtime = read_bundle_runtime_config(&current_bundle)?;
    let Some(base_url) = resolve_project_hive_base_url(
        runtime.as_ref(),
        runtime
            .as_ref()
            .and_then(|config| config.base_url.as_deref()),
    ) else {
        return Ok(None);
    };

    let client = match MemdClient::new(&base_url) {
        Ok(client) => client,
        Err(_) => return Ok(None),
    };
    let workspace = runtime.as_ref().and_then(|config| config.workspace.clone());
    let (shared_project, shared_namespace) = shared_awareness_scope(runtime.as_ref());

    let sessions_request = memd_schema::HiveSessionsRequest {
        session: None,
        project: shared_project.clone(),
        namespace: shared_namespace.clone(),
        repo_root: None,
        worktree_root: None,
        branch: None,
        workspace: workspace.clone(),
        hive_system: None,
        hive_role: None,
        host: None,
        hive_group: None,
        active_only: Some(false),
        limit: Some(512),
    };
    let sessions = match timeout_ok(client.hive_sessions(&sessions_request)).await {
        Some(response) => response.sessions,
        None => return Ok(None),
    };
    if sessions.is_empty() {
        return Ok(None);
    }

    let claims_request = HiveClaimsRequest {
        session: None,
        project: shared_project,
        namespace: shared_namespace,
        workspace,
        active_only: Some(true),
        limit: Some(512),
    };
    let claims = timeout_ok(client.hive_claims(&claims_request))
        .await
        .map(|response| response.claims)
        .unwrap_or_default();
    let current_session = runtime
        .as_ref()
        .and_then(|config| config.session.as_deref());

    let mut entries = Vec::new();
    let mut base_url_counts = std::collections::BTreeMap::<String, usize>::new();
    for session in sessions {
        if !args.include_current && current_session == Some(session.session.as_str()) {
            continue;
        }

        let active_claims = claims
            .iter()
            .filter(|claim| claim.session == session.session && claim.expires_at > Utc::now())
            .count();
        let entry = ProjectAwarenessEntry {
            project_dir: "remote".to_string(),
            bundle_root: format!("remote:{}:{}", base_url, session.session),
            project: session.project.clone(),
            namespace: session.namespace.clone(),
            repo_root: session.repo_root.clone(),
            worktree_root: session.worktree_root.clone(),
            branch: session.branch.clone(),
            base_branch: session.base_branch.clone(),
            agent: session.agent.clone(),
            session: Some(session.session.clone()),
            tab_id: session.tab_id.clone(),
            effective_agent: session.effective_agent.clone(),
            hive_system: session.hive_system.clone(),
            hive_role: session.hive_role.clone(),
            capabilities: session.capabilities.clone(),
            hive_groups: session.hive_groups.clone(),
            hive_group_goal: session.hive_group_goal.clone(),
            authority: session.authority.clone(),
            base_url: session.base_url.clone(),
            presence: heartbeat_presence_label(session.last_seen).to_string(),
            host: session.host.clone(),
            pid: session.pid,
            active_claims,
            workspace: session.workspace.clone(),
            visibility: session.visibility.clone(),
            topic_claim: session.topic_claim.clone(),
            scope_claims: session.scope_claims.clone(),
            task_id: session.task_id.clone(),
            focus: session.focus.clone(),
            pressure: session.pressure.clone(),
            next_recovery: session.next_recovery.clone(),
            last_updated: Some(session.last_seen),
        };
        if let Some(url) = entry
            .base_url
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        {
            *base_url_counts.entry(url).or_insert(0) += 1;
        }
        entries.push(entry);
    }

    entries.sort_by(|left, right| left.bundle_root.cmp(&right.bundle_root));
    let mut collisions = base_url_counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(url, count)| format!("base_url {} used by {} bundles", url, count))
        .collect::<Vec<_>>();
    collisions.extend(session_collision_warnings(&entries));
    let root = if let Some(workspace) = runtime.and_then(|config| config.workspace.clone()) {
        format!("server:{base_url} workspace:{workspace}")
    } else {
        format!("server:{base_url}")
    };

    Ok(Some(ProjectAwarenessResponse {
        root,
        current_bundle: fallback.current_bundle.clone(),
        collisions,
        entries,
    }))
}

pub(crate) fn shared_awareness_scope(
    runtime: Option<&BundleRuntimeConfig>,
) -> (Option<String>, Option<String>) {
    let project = runtime.and_then(|config| config.project.clone());
    let namespace = runtime.and_then(|config| config.namespace.clone());
    let workspace = runtime.and_then(|config| config.workspace.clone());
    if workspace.is_some() {
        (None, None)
    } else {
        (project, namespace)
    }
}

pub(crate) fn session_collision_warnings(entries: &[ProjectAwarenessEntry]) -> Vec<String> {
    let mut groups = std::collections::BTreeMap::<
        (Option<String>, Option<String>, Option<String>),
        Vec<&ProjectAwarenessEntry>,
    >::new();
    for entry in entries {
        groups
            .entry((
                entry.workspace.clone(),
                entry.session.clone(),
                entry.tab_id.clone(),
            ))
            .or_default()
            .push(entry);
    }

    groups
        .into_iter()
        .filter_map(|((workspace, session, tab_id), group)| {
            if group.len() <= 1 {
                return None;
            }

            let mut agents = std::collections::BTreeSet::new();
            let mut urls = std::collections::BTreeSet::new();
            for entry in &group {
                agents.insert(
                    entry
                        .effective_agent
                        .as_deref()
                        .or(entry.agent.as_deref())
                        .unwrap_or("none")
                        .to_string(),
                );
                urls.insert(entry.base_url.as_deref().unwrap_or("none").to_string());
            }

            if agents.len() <= 1 && urls.len() <= 1 {
                return None;
            }

            Some(format!(
                "session {} tab {} in workspace {} seen across {} bundles / {} agents / {} endpoints",
                session.as_deref().unwrap_or("none"),
                tab_id.as_deref().unwrap_or("none"),
                workspace.as_deref().unwrap_or("none"),
                group.len(),
                agents.len(),
                urls.len()
            ))
        })
        .collect()
}

pub(crate) fn read_project_awareness_local(
    args: &AwarenessArgs,
) -> anyhow::Result<ProjectAwarenessResponse> {
    let (current_bundle, _current_project, scan_root) = resolve_awareness_paths(args)?;
    let current_runtime = read_bundle_runtime_config(&current_bundle)?;

    let mut entries = Vec::new();
    let mut base_url_counts = std::collections::BTreeMap::<String, usize>::new();
    for child in fs::read_dir(&scan_root)
        .with_context(|| format!("read awareness root {}", scan_root.display()))?
    {
        let child = child?;
        if !child.file_type()?.is_dir() {
            continue;
        }

        let project_dir = child.path();
        let bundle_root = project_dir.join(".memd");
        let config_path = bundle_root.join("config.json");
        if !config_path.exists() {
            continue;
        }

        let canonical_bundle = fs::canonicalize(&bundle_root).unwrap_or(bundle_root.clone());
        if !args.include_current && canonical_bundle == current_bundle {
            continue;
        }

        let runtime = read_bundle_runtime_config(&bundle_root)?.unwrap_or(BundleRuntimeConfig {
            project: None,
            namespace: None,
            agent: None,
            session: None,
            tab_id: None,
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
            heartbeat_model: Some(default_heartbeat_model()),
            voice_mode: Some(default_voice_mode()),
            auto_short_term_capture: true,
            authority_policy: BundleAuthorityPolicy::default(),
            authority_state: BundleAuthorityState::default(),
        });
        let state = read_bundle_resume_state(&bundle_root)?;
        let heartbeat = read_bundle_heartbeat(&bundle_root)?;
        let claims = read_bundle_claims(&bundle_root)?;
        let active_claims = claims
            .claims
            .iter()
            .filter(|claim| claim.expires_at > Utc::now())
            .count();
        if prune_dead_local_bundle_heartbeat(
            &bundle_root,
            heartbeat.as_ref(),
            active_claims,
            canonical_bundle == current_bundle,
            current_runtime.as_ref(),
        )? {
            continue;
        }
        if skip_inactive_local_bundle_entry(
            &runtime,
            heartbeat.as_ref(),
            state.as_ref(),
            active_claims,
            canonical_bundle == current_bundle,
        ) {
            continue;
        }
        let state_path = bundle_resume_state_path(&bundle_root);
        let heartbeat_path = bundle_heartbeat_state_path(&bundle_root);
        let last_updated = if heartbeat_path.exists() {
            fs::metadata(&heartbeat_path)
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .map(DateTime::<Utc>::from)
        } else if state_path.exists() {
            fs::metadata(&state_path)
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .map(DateTime::<Utc>::from)
        } else {
            fs::metadata(&config_path)
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .map(DateTime::<Utc>::from)
        };

        entries.push(ProjectAwarenessEntry {
            project_dir: project_dir.display().to_string(),
            bundle_root: bundle_root.display().to_string(),
            project: runtime.project,
            namespace: runtime.namespace,
            repo_root: heartbeat.as_ref().and_then(|value| value.repo_root.clone()),
            worktree_root: heartbeat
                .as_ref()
                .and_then(|value| value.worktree_root.clone())
                .or_else(|| Some(project_dir.display().to_string())),
            branch: heartbeat.as_ref().and_then(|value| value.branch.clone()),
            base_branch: heartbeat
                .as_ref()
                .and_then(|value| value.base_branch.clone()),
            tab_id: heartbeat
                .as_ref()
                .and_then(|value| value.tab_id.clone())
                .or(runtime.tab_id),
            effective_agent: runtime
                .agent
                .as_deref()
                .map(|agent| compose_agent_identity(agent, runtime.session.as_deref())),
            agent: runtime.agent,
            session: runtime.session,
            hive_system: heartbeat
                .as_ref()
                .and_then(|value| value.hive_system.clone())
                .or(runtime.hive_system),
            hive_role: heartbeat
                .as_ref()
                .and_then(|value| value.hive_role.clone())
                .or(runtime.hive_role),
            capabilities: heartbeat
                .as_ref()
                .map(|value| value.capabilities.clone())
                .filter(|value| !value.is_empty())
                .unwrap_or(runtime.capabilities),
            hive_groups: heartbeat
                .as_ref()
                .map(|value| value.hive_groups.clone())
                .filter(|value| !value.is_empty())
                .unwrap_or(runtime.hive_groups),
            hive_group_goal: heartbeat
                .as_ref()
                .and_then(|value| value.hive_group_goal.clone())
                .or(runtime.hive_group_goal),
            authority: heartbeat
                .as_ref()
                .and_then(|value| value.authority.clone())
                .or(runtime.authority),
            base_url: runtime.base_url.clone(),
            presence: heartbeat
                .as_ref()
                .map(|value| heartbeat_presence_label(value.last_seen).to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            host: heartbeat.as_ref().and_then(|value| value.host.clone()),
            pid: heartbeat.as_ref().and_then(|value| value.pid),
            active_claims,
            workspace: heartbeat
                .as_ref()
                .and_then(|value| value.workspace.clone())
                .or(runtime.workspace),
            visibility: heartbeat
                .as_ref()
                .and_then(|value| value.visibility.clone())
                .or(runtime.visibility),
            topic_claim: heartbeat
                .as_ref()
                .and_then(|value| value.topic_claim.clone()),
            scope_claims: heartbeat
                .as_ref()
                .map(|value| value.scope_claims.clone())
                .unwrap_or_default(),
            task_id: heartbeat.as_ref().and_then(|value| value.task_id.clone()),
            focus: heartbeat
                .as_ref()
                .and_then(|value| value.focus.clone())
                .or_else(|| state.as_ref().and_then(|value| value.focus.clone())),
            pressure: heartbeat
                .as_ref()
                .and_then(|value| value.pressure.clone())
                .or_else(|| state.as_ref().and_then(|value| value.pressure.clone())),
            next_recovery: heartbeat
                .as_ref()
                .and_then(|value| value.next_recovery.clone())
                .or_else(|| state.as_ref().and_then(|value| value.next_recovery.clone())),
            last_updated,
        });
        if let Some(url) = entries
            .last()
            .and_then(|entry| entry.base_url.as_ref())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        {
            *base_url_counts.entry(url).or_insert(0) += 1;
        }
    }

    entries.sort_by(|left, right| left.project_dir.cmp(&right.project_dir));
    let mut collisions = base_url_counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(url, count)| format!("base_url {} used by {} bundles", url, count))
        .collect::<Vec<_>>();
    collisions.extend(session_collision_warnings(&entries));

    Ok(ProjectAwarenessResponse {
        root: scan_root.display().to_string(),
        current_bundle: current_bundle.display().to_string(),
        collisions,
        entries,
    })
}

pub(crate) fn render_project_awareness_summary(response: &ProjectAwarenessResponse) -> String {
    let current_entry = response
        .entries
        .iter()
        .find(|candidate| candidate.bundle_root == response.current_bundle);
    let visible_entries = project_awareness_visible_entries(response);
    let hidden_remote_dead = response
        .entries
        .iter()
        .filter(|entry| {
            entry.project_dir == "remote"
                && entry.presence == "dead"
                && current_entry
                    .map(|current| {
                        entry.project == current.project
                            && entry.namespace == current.namespace
                            && entry.workspace == current.workspace
                            && entry.base_url == current.base_url
                    })
                    .unwrap_or(true)
        })
        .count();
    let superseded_stale_sessions = response
        .entries
        .iter()
        .filter(|entry| {
            current_entry
                .map(|current| is_superseded_stale_remote_session(entry, current))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    let superseded_stale_count = superseded_stale_sessions.len();
    let superseded_stale_session_ids = superseded_stale_sessions
        .iter()
        .filter_map(|entry| entry.session.as_deref())
        .take(3)
        .collect::<Vec<_>>();
    let superseded_stale_suffix = if superseded_stale_count > superseded_stale_session_ids.len() {
        format!(
            " +{}",
            superseded_stale_count - superseded_stale_session_ids.len()
        )
    } else {
        String::new()
    };
    let current_session = visible_entries
        .iter()
        .find(|entry| entry.bundle_root == response.current_bundle)
        .and_then(|entry| entry.session.as_deref());
    let stale_remote_sessions = visible_entries
        .iter()
        .filter(|entry| entry.project_dir == "remote" && entry.presence == "stale")
        .collect::<Vec<_>>();
    let active_hive_sessions = current_session
        .map(|current| {
            visible_entries
                .iter()
                .filter(|entry| entry.presence == "active")
                .filter(|entry| !entry.hive_groups.is_empty())
                .filter_map(|entry| entry.session.as_deref())
                .filter(|session| *session != current)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let rendered_diagnostics = awareness_summary_diagnostics(&visible_entries);
    let mut lines = vec![format!(
        "awareness root={} bundles={} diagnostics={} hidden_remote_dead={} hidden_superseded_stale={}",
        response.root,
        visible_entries.len(),
        rendered_diagnostics.len(),
        hidden_remote_dead,
        superseded_stale_count,
    )];
    if !active_hive_sessions.is_empty() {
        lines.push(format!(
            "! active_hive_sessions={} sessions={}",
            active_hive_sessions.len(),
            active_hive_sessions.join(",")
        ));
    }
    if !stale_remote_sessions.is_empty() {
        let sessions = stale_remote_sessions
            .iter()
            .take(3)
            .filter_map(|entry| entry.session.as_deref())
            .collect::<Vec<_>>();
        let suffix = if stale_remote_sessions.len() > sessions.len() {
            format!(" +{}", stale_remote_sessions.len() - sessions.len())
        } else {
            String::new()
        };
        lines.push(format!(
            "! stale_remote_sessions={} sessions={}{}",
            stale_remote_sessions.len(),
            if sessions.is_empty() {
                "unknown".to_string()
            } else {
                sessions.join(",")
            },
            suffix,
        ));
    }
    if superseded_stale_count > 0 {
        lines.push(format!(
            "! superseded_stale_sessions={} sessions={}{}",
            superseded_stale_count,
            if superseded_stale_session_ids.is_empty() {
                "unknown".to_string()
            } else {
                superseded_stale_session_ids.join(",")
            },
            superseded_stale_suffix,
        ));
    }
    for diagnostic in &rendered_diagnostics {
        lines.push(format!("! {}", diagnostic));
    }
    let current_entries = visible_entries
        .iter()
        .copied()
        .filter(|entry| entry.bundle_root == response.current_bundle)
        .collect::<Vec<_>>();
    let active_hive_entries = visible_entries
        .iter()
        .copied()
        .filter(|entry| entry.bundle_root != response.current_bundle)
        .filter(|entry| entry.presence == "active" && !entry.hive_groups.is_empty())
        .collect::<Vec<_>>();
    let stale_entries = visible_entries
        .iter()
        .copied()
        .filter(|entry| entry.presence == "stale")
        .collect::<Vec<_>>();
    let seen_entries = visible_entries
        .iter()
        .copied()
        .filter(|entry| entry.bundle_root != response.current_bundle)
        .filter(|entry| !(entry.presence == "active" && !entry.hive_groups.is_empty()))
        .filter(|entry| entry.presence != "stale")
        .filter(|entry| entry.presence != "dead")
        .collect::<Vec<_>>();
    let dead_entries = visible_entries
        .iter()
        .copied()
        .filter(|entry| entry.presence == "dead")
        .collect::<Vec<_>>();

    push_awareness_section(
        &mut lines,
        "current_session",
        &current_entries,
        "current",
        &response.current_bundle,
    );
    push_awareness_section(
        &mut lines,
        "active_hive_sessions",
        &active_hive_entries,
        "hive-session",
        &response.current_bundle,
    );
    push_awareness_section(
        &mut lines,
        "stale_sessions",
        &stale_entries,
        "stale-session",
        &response.current_bundle,
    );
    push_awareness_section(
        &mut lines,
        "dead_sessions",
        &dead_entries,
        "dead-session",
        &response.current_bundle,
    );
    push_awareness_section(
        &mut lines,
        "seen_sessions",
        &seen_entries,
        "seen",
        &response.current_bundle,
    );
    lines.join("\n")
}

pub(crate) fn awareness_summary_diagnostics(entries: &[&ProjectAwarenessEntry]) -> Vec<String> {
    let owned = entries
        .iter()
        .map(|entry| (*entry).clone())
        .collect::<Vec<_>>();
    let mut diagnostics = shared_endpoint_diagnostics(entries);
    diagnostics.extend(session_collision_warnings(&owned));
    diagnostics.extend(branch_collision_warnings(&owned));
    diagnostics.extend(work_overlap_warnings(entries));
    diagnostics
}

pub(crate) fn shared_endpoint_diagnostics(entries: &[&ProjectAwarenessEntry]) -> Vec<String> {
    let mut counts = std::collections::BTreeMap::<String, usize>::new();
    for entry in entries {
        if let Some(url) = entry
            .base_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            *counts.entry(url.to_string()).or_insert(0) += 1;
        }
    }

    counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(url, count)| format!("shared_hive_endpoint {} sessions={}", url, count))
        .collect()
}

pub(crate) fn branch_collision_warnings(entries: &[ProjectAwarenessEntry]) -> Vec<String> {
    let mut same_branch = std::collections::BTreeMap::<(String, String), Vec<String>>::new();
    let mut same_worktree = std::collections::BTreeMap::<String, Vec<String>>::new();

    for entry in entries {
        let lane = entry
            .session
            .clone()
            .or_else(|| entry.effective_agent.clone())
            .or_else(|| entry.agent.clone())
            .unwrap_or_else(|| entry.bundle_root.clone());

        if let (Some(repo_root), Some(branch)) = (
            entry
                .repo_root
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
            entry
                .branch
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        ) {
            same_branch
                .entry((repo_root.to_string(), branch.to_string()))
                .or_default()
                .push(lane.clone());
        }

        if let Some(worktree_root) = entry
            .worktree_root
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            same_worktree
                .entry(worktree_root.to_string())
                .or_default()
                .push(lane);
        }
    }

    let mut warnings = Vec::new();
    warnings.extend(
        same_branch
            .into_iter()
            .filter(|(_, lanes)| lanes.len() > 1)
            .map(|((repo_root, branch), lanes)| {
                format!(
                    "unsafe_same_branch repo={} branch={} sessions={}",
                    repo_root,
                    branch,
                    lanes.join(",")
                )
            }),
    );
    warnings.extend(
        same_worktree
            .into_iter()
            .filter(|(_, lanes)| lanes.len() > 1)
            .map(|(worktree_root, lanes)| {
                format!(
                    "unsafe_same_worktree worktree={} sessions={}",
                    worktree_root,
                    lanes.join(",")
                )
            }),
    );
    warnings
}

pub(crate) fn work_overlap_warnings(entries: &[&ProjectAwarenessEntry]) -> Vec<String> {
    let mut warnings = Vec::new();
    let active_entries = entries
        .iter()
        .copied()
        .filter(|entry| entry.presence == "active")
        .collect::<Vec<_>>();

    for (idx, left) in active_entries.iter().enumerate() {
        let left_touches = awareness_overlap_touch_points(left);
        if left_touches.is_empty() {
            continue;
        }
        for right in active_entries.iter().skip(idx + 1) {
            let right_touches = awareness_overlap_touch_points(right);
            if right_touches.is_empty() {
                continue;
            }
            let shared = left_touches
                .iter()
                .filter(|touch| right_touches.iter().any(|other| other == *touch))
                .cloned()
                .collect::<Vec<_>>();
            if shared.is_empty() {
                continue;
            }
            warnings.push(format!(
                "possible_work_overlap touches={} sessions={},{}",
                shared.join(","),
                left.session.as_deref().unwrap_or("none"),
                right.session.as_deref().unwrap_or("none"),
            ));
        }
    }

    warnings
}

#[derive(Debug, Clone)]
pub(crate) struct BundleHiveMemorySurface {
    pub(crate) board: HiveBoardResponse,
    pub(crate) roster: HiveRosterResponse,
    pub(crate) follow: Option<HiveFollowResponse>,
}

pub(crate) fn push_awareness_section(
    lines: &mut Vec<String>,
    label: &str,
    entries: &[&ProjectAwarenessEntry],
    role: &str,
    current_bundle: &str,
) {
    if entries.is_empty() {
        return;
    }
    lines.push(format!("{label}:"));
    for entry in entries {
        lines.push(render_awareness_entry_line(entry, role, current_bundle));
    }
}

pub(crate) fn awareness_truth_label(
    entry: &ProjectAwarenessEntry,
    current_bundle: &str,
) -> &'static str {
    if entry.bundle_root == current_bundle && entry.presence == "active" {
        return "current";
    }
    match entry.presence.as_str() {
        "active" => match entry.last_updated {
            Some(last_updated) => {
                let age = Utc::now() - last_updated;
                if age.num_seconds() <= 120 {
                    "fresh"
                } else if age.num_minutes() <= 15 {
                    "aging"
                } else {
                    "stale-truth"
                }
            }
            None => "active",
        },
        "stale" => "stale",
        "dead" => "dead",
        _ => "seen",
    }
}

pub(crate) fn render_awareness_entry_line(
    entry: &ProjectAwarenessEntry,
    role: &str,
    current_bundle: &str,
) -> String {
    let focus = entry
        .focus
        .as_deref()
        .map(|value| compact_inline(value, 56))
        .unwrap_or_else(|| "none".to_string());
    let pressure = entry
        .pressure
        .as_deref()
        .map(|value| compact_inline(value, 56))
        .unwrap_or_else(|| "none".to_string());
    let truth = awareness_truth_label(entry, current_bundle);
    let updated = entry
        .last_updated
        .map(|value| value.to_rfc3339())
        .unwrap_or_else(|| "unknown".to_string());
    let work = entry
        .topic_claim
        .as_deref()
        .map(|value| compact_inline(value, 56))
        .unwrap_or_else(|| awareness_work_quickview(entry));
    let next = entry
        .next_recovery
        .as_deref()
        .and_then(simplify_awareness_work_text)
        .map(|value| compact_inline(&value, 56))
        .unwrap_or_else(|| "none".to_string());
    let touches = if entry.scope_claims.is_empty() {
        awareness_touch_quickview(entry)
    } else {
        compact_inline(&entry.scope_claims.join(","), 56)
    };
    format!(
        "- {} [{}] | presence={} truth={} updated={} claims={} ns={} hive={} role={} groups={} goal=\"{}\" authority={} agent={} session={} tab={} branch={} worktree={} base_url={} workspace={} visibility={} task={} work=\"{}\" touches={} next=\"{}\" focus=\"{}\" pressure=\"{}\"",
        entry.project.as_deref().unwrap_or("unknown"),
        role,
        entry.presence,
        truth,
        updated,
        entry.active_claims,
        entry.namespace.as_deref().unwrap_or("none"),
        entry.hive_system.as_deref().unwrap_or("none"),
        entry.hive_role.as_deref().unwrap_or("none"),
        if entry.hive_groups.is_empty() {
            "none".to_string()
        } else {
            entry.hive_groups.join(",")
        },
        entry.hive_group_goal.as_deref().unwrap_or("none"),
        entry.authority.as_deref().unwrap_or("none"),
        entry
            .effective_agent
            .as_deref()
            .or(entry.agent.as_deref())
            .unwrap_or("none"),
        entry.session.as_deref().unwrap_or("none"),
        entry.tab_id.as_deref().unwrap_or("none"),
        entry.branch.as_deref().unwrap_or("none"),
        entry.worktree_root.as_deref().unwrap_or("none"),
        entry.base_url.as_deref().unwrap_or("none"),
        entry.workspace.as_deref().unwrap_or("none"),
        entry.visibility.as_deref().unwrap_or("all"),
        entry.task_id.as_deref().unwrap_or("none"),
        work,
        touches,
        next,
        focus,
        pressure,
    )
}

pub(crate) fn awareness_work_quickview(entry: &ProjectAwarenessEntry) -> String {
    if let Some(value) = entry
        .topic_claim
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return compact_inline(value, 56);
    }
    for candidate in [entry.focus.as_deref(), entry.next_recovery.as_deref()] {
        if let Some(value) = candidate.and_then(simplify_awareness_work_text) {
            return compact_inline(&value, 56);
        }
    }
    let touches = awareness_touch_points(entry);
    if let Some(first) = touches.first() {
        if touches.len() == 1 {
            return compact_inline(&format!("editing {first}"), 56);
        }
        return compact_inline(&format!("editing {first} +{}", touches.len() - 1), 56);
    }
    if let Some(value) = entry
        .pressure
        .as_deref()
        .and_then(simplify_awareness_work_text)
    {
        return compact_inline(&value, 56);
    }
    "none".to_string()
}

pub(crate) fn derive_awareness_worker_name(entry: &ProjectAwarenessEntry) -> Option<String> {
    entry
        .effective_agent
        .as_deref()
        .and_then(|value| value.split('@').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            entry
                .agent
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
}

pub(crate) fn derive_awareness_lane_id(entry: &ProjectAwarenessEntry) -> Option<String> {
    entry
        .worktree_root
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            entry
                .branch
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
}

pub(crate) fn project_awareness_entry_to_hive_session(
    entry: &ProjectAwarenessEntry,
) -> memd_schema::HiveSessionRecord {
    let working = entry
        .topic_claim
        .clone()
        .or_else(|| Some(awareness_work_quickview(entry)));
    let touches = awareness_touch_points(entry)
        .into_iter()
        .filter_map(|value| normalize_hive_touch(&value))
        .collect::<Vec<_>>();
    memd_schema::HiveSessionRecord {
        session: entry
            .session
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        tab_id: entry.tab_id.clone(),
        agent: entry.agent.clone(),
        effective_agent: entry.effective_agent.clone(),
        hive_system: entry.hive_system.clone(),
        hive_role: entry.hive_role.clone(),
        worker_name: derive_awareness_worker_name(entry),
        display_name: None,
        role: entry.hive_role.clone(),
        capabilities: entry.capabilities.clone(),
        hive_groups: entry.hive_groups.clone(),
        lane_id: derive_awareness_lane_id(entry),
        hive_group_goal: entry.hive_group_goal.clone(),
        authority: entry.authority.clone(),
        heartbeat_model: None,
        project: entry.project.clone(),
        namespace: entry.namespace.clone(),
        workspace: entry.workspace.clone(),
        repo_root: entry.repo_root.clone(),
        worktree_root: entry.worktree_root.clone(),
        branch: entry.branch.clone(),
        base_branch: entry.base_branch.clone(),
        visibility: entry.visibility.clone(),
        base_url: entry.base_url.clone(),
        base_url_healthy: None,
        host: entry.host.clone(),
        pid: entry.pid,
        topic_claim: entry.topic_claim.clone(),
        scope_claims: entry.scope_claims.clone(),
        task_id: entry.task_id.clone(),
        focus: entry.focus.clone(),
        pressure: entry.pressure.clone(),
        next_recovery: entry.next_recovery.clone(),
        next_action: None,
        working,
        touches,
        relationship_state: None,
        relationship_peer: None,
        relationship_reason: None,
        suggested_action: None,
        blocked_by: Vec::new(),
        cowork_with: Vec::new(),
        handoff_target: None,
        offered_to: Vec::new(),
        needs_help: false,
        needs_review: false,
        handoff_state: None,
        confidence: None,
        risk: None,
        status: entry.presence.clone(),
        last_seen: entry.last_updated.unwrap_or_else(Utc::now),
    }
}

pub(crate) fn awareness_touch_quickview(entry: &ProjectAwarenessEntry) -> String {
    let touches = awareness_touch_points(entry);
    if touches.is_empty() {
        "none".to_string()
    } else {
        compact_inline(&touches.join(","), 56)
    }
}

pub(crate) fn awareness_touch_points(entry: &ProjectAwarenessEntry) -> Vec<String> {
    let mut touches = Vec::new();
    for scope in &entry.scope_claims {
        push_unique_touch_point(&mut touches, scope);
    }
    for candidate in [
        entry.pressure.as_deref(),
        entry.focus.as_deref(),
        entry.next_recovery.as_deref(),
    ] {
        let Some(value) = candidate else {
            continue;
        };
        append_awareness_touch_points(value, &mut touches);
    }
    touches.truncate(4);
    touches
}

pub(crate) fn normalize_hive_touch(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

pub(crate) fn awareness_overlap_touch_points(entry: &ProjectAwarenessEntry) -> Vec<String> {
    awareness_touch_points(entry)
        .into_iter()
        .filter(|touch| !is_generic_overlap_touch(touch))
        .collect()
}

pub(crate) fn is_generic_overlap_touch(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.is_empty()
        || matches!(
            trimmed,
            "project" | "workspace" | "shared" | "none" | "unknown"
        )
}

pub(crate) fn append_awareness_touch_points(value: &str, touches: &mut Vec<String>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return;
    }

    for part in trimmed
        .split('\n')
        .flat_map(|line| line.split(" | "))
        .map(str::trim)
    {
        if let Some(path) = part.strip_prefix("file_edited:") {
            push_unique_touch_point(touches, path.trim());
            continue;
        }
        if let Some(scope) = part.strip_prefix("scope=") {
            push_unique_touch_point(touches, scope.trim());
            continue;
        }
        if let Some(location) = part.strip_prefix("location=") {
            push_unique_touch_point(touches, location.trim());
        }
    }
}

pub(crate) fn push_unique_touch_point(touches: &mut Vec<String>, value: &str) {
    let trimmed = value.trim();
    if trimmed.is_empty() || touches.iter().any(|existing| existing == trimmed) {
        return;
    }
    touches.push(trimmed.to_string());
}

pub(crate) fn derive_hive_topic_claim(
    focus: Option<&str>,
    next_recovery: Option<&str>,
    pressure: Option<&str>,
) -> Option<String> {
    for candidate in [focus, next_recovery] {
        if let Some(value) = candidate.and_then(simplify_awareness_work_text) {
            return Some(compact_inline(&value, 120));
        }
    }
    let touches = [pressure]
        .into_iter()
        .flatten()
        .flat_map(|value| {
            let mut out = Vec::new();
            append_awareness_touch_points(value, &mut out);
            out
        })
        .collect::<Vec<_>>();
    if let Some(first) = touches.first() {
        return Some(if touches.len() == 1 {
            format!("editing {first}")
        } else {
            format!("editing {first} +{}", touches.len() - 1)
        });
    }
    None
}

pub(crate) fn derive_hive_worker_name(
    agent: Option<&str>,
    _session: Option<&str>,
) -> Option<String> {
    agent
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub(crate) fn humanize_worker_label(value: &str) -> String {
    let parts = value
        .split(|ch: char| ch == '-' || ch == '_' || ch.is_whitespace())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            format!(
                "{}{}",
                first.to_uppercase(),
                chars.as_str().to_ascii_lowercase()
            )
        })
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        value.trim().to_string()
    } else {
        parts.join(" ")
    }
}

pub(crate) fn hive_worker_name_is_generic(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "codex" | "claude" | "claude-code"
    )
}

pub(crate) fn derive_hive_display_name(
    agent: Option<&str>,
    session: Option<&str>,
) -> Option<String> {
    let agent = agent.map(str::trim).filter(|value| !value.is_empty())?;
    if !hive_worker_name_is_generic(agent) {
        return None;
    }
    let session = session.map(str::trim).filter(|value| !value.is_empty())?;
    let session_suffix = session
        .strip_prefix("session-")
        .or_else(|| session.strip_prefix("codex-"))
        .or_else(|| session.strip_prefix("sender-"))
        .unwrap_or(session)
        .trim();
    if session_suffix.is_empty() {
        return None;
    }
    let base = match agent.to_ascii_lowercase().as_str() {
        "claude" | "claude-code" => "Claude",
        _ => "Codex",
    };
    Some(format!("{base} {}", session_suffix))
}

pub(crate) fn derive_project_scoped_worker_name(
    project: Option<&str>,
    agent: &str,
    session: Option<&str>,
) -> Option<String> {
    if !hive_worker_name_is_generic(agent) {
        return None;
    }
    let project = project
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(humanize_worker_label)?;
    let generic = derive_hive_display_name(Some(agent), session)?;
    Some(format!("{project} {generic}"))
}

pub(crate) fn default_bundle_worker_name(agent: &str, session: Option<&str>) -> String {
    derive_hive_display_name(Some(agent), session)
        .or_else(|| {
            derive_hive_worker_name(Some(agent), session).map(|value| humanize_worker_label(&value))
        })
        .unwrap_or_else(|| humanize_worker_label(agent))
}

pub(crate) fn default_bundle_worker_name_for_project(
    project: Option<&str>,
    agent: &str,
    session: Option<&str>,
) -> String {
    derive_project_scoped_worker_name(project, agent, session)
        .unwrap_or_else(|| default_bundle_worker_name(agent, session))
}

pub(crate) fn hive_actor_label(
    display_name: Option<&str>,
    worker_name: Option<&str>,
    agent: Option<&str>,
    session: Option<&str>,
) -> String {
    display_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| derive_hive_display_name(worker_name.or(agent), session))
        .or_else(|| {
            worker_name
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .or_else(|| {
            agent
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .or_else(|| {
            session
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "unnamed".to_string())
}

pub(crate) fn derive_hive_next_action(
    focus: Option<&str>,
    next_recovery: Option<&str>,
    pressure: Option<&str>,
) -> Option<String> {
    for candidate in [focus, next_recovery, pressure] {
        if let Some(value) = candidate.and_then(simplify_awareness_work_text) {
            return Some(compact_inline(&value, 120));
        }
    }
    None
}

pub(crate) fn derive_hive_lane_id(
    branch: Option<&str>,
    worktree_root: Option<&str>,
) -> Option<String> {
    worktree_root
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            branch
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
}

pub(crate) fn hive_topic_claim_needs_runtime_upgrade(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.is_empty()
        || trimmed.starts_with("editing ")
        || trimmed.starts_with("ws=")
        || trimmed.starts_with("workspace=")
        || trimmed == "project"
}

pub(crate) fn derive_hive_scope_claims(
    claims_state: Option<&SessionClaimsState>,
    focus: Option<&str>,
    pressure: Option<&str>,
    next_recovery: Option<&str>,
) -> Vec<String> {
    let mut scopes = claims_state
        .map(|state| {
            state
                .claims
                .iter()
                .filter(|claim| claim.expires_at > Utc::now())
                .map(|claim| claim.scope.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for candidate in [pressure, focus, next_recovery] {
        let Some(value) = candidate else {
            continue;
        };
        append_awareness_touch_points(value, &mut scopes);
    }
    scopes.truncate(8);
    scopes
}

pub(crate) fn derive_hive_task_id(
    scope_claims: &[String],
    topic_claim: Option<&str>,
) -> Option<String> {
    for scope in scope_claims {
        if let Some(task_id) = scope.strip_prefix("task:") {
            let task_id = task_id.trim();
            if !task_id.is_empty() {
                return Some(task_id.to_string());
            }
        }
    }
    if let Some(topic) = topic_claim {
        if let Some(task_id) = topic.strip_prefix("task:") {
            let task_id = task_id.trim();
            if !task_id.is_empty() {
                return Some(task_id.to_string());
            }
        }
    }
    None
}

pub(crate) fn confirmed_hive_overlap_reason(
    target: &ProjectAwarenessEntry,
    task_id: Option<&str>,
    topic_claim: Option<&str>,
    scope_claims: &[String],
) -> Option<String> {
    let current_scopes = scope_claims
        .iter()
        .map(|scope| scope.trim())
        .filter(|scope| !scope.is_empty())
        .filter(|scope| !is_generic_overlap_touch(scope))
        .collect::<Vec<_>>();
    let target_scopes = target
        .scope_claims
        .iter()
        .map(|scope| scope.trim())
        .filter(|scope| !scope.is_empty())
        .filter(|scope| !is_generic_overlap_touch(scope))
        .collect::<Vec<_>>();
    let task_id = task_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let topic_claim = topic_claim
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase());

    if let (Some(current_task), Some(target_task)) = (
        task_id.as_deref(),
        target
            .task_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    ) && current_task != target_task
    {
        if !target_scopes.is_empty()
            && current_scopes.iter().any(|scope| {
                target_scopes
                    .iter()
                    .any(|target_scope| target_scope == scope)
            })
        {
            return Some(format!(
                "confirmed hive overlap: target session {} already owns scope(s) for task {}",
                target.session.as_deref().unwrap_or("none"),
                target_task
            ));
        }
    }

    let shared_scopes = current_scopes
        .iter()
        .filter(|scope| {
            target_scopes
                .iter()
                .any(|target_scope| target_scope == *scope)
        })
        .map(|scope| (*scope).to_string())
        .collect::<Vec<_>>();
    if !shared_scopes.is_empty() {
        return Some(format!(
            "confirmed hive overlap: target session {} already claims {}",
            target.session.as_deref().unwrap_or("none"),
            shared_scopes.join(",")
        ));
    }

    if let (Some(current_topic), Some(target_topic)) = (
        topic_claim.as_deref(),
        target
            .topic_claim
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_ascii_lowercase()),
    ) && current_topic == target_topic
    {
        return Some(format!(
            "confirmed hive overlap: target session {} already owns topic {}",
            target.session.as_deref().unwrap_or("none"),
            target.topic_claim.as_deref().unwrap_or("none")
        ));
    }

    None
}

pub(crate) async fn existing_task_scopes_for_assignment(
    client: &MemdClient,
    project: &Option<String>,
    namespace: &Option<String>,
    workspace: &Option<String>,
    task_id: &str,
) -> anyhow::Result<Vec<String>> {
    let response = client
        .hive_tasks(&HiveTasksRequest {
            session: None,
            project: project.clone(),
            namespace: namespace.clone(),
            workspace: workspace.clone(),
            active_only: Some(false),
            limit: Some(256),
        })
        .await?;
    Ok(response
        .tasks
        .into_iter()
        .find(|task| task.task_id == task_id)
        .map(|task| task.claim_scopes)
        .unwrap_or_default())
}

