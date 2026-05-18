#![allow(dead_code, unused_imports)]

use super::*;

#[allow(dead_code)]
mod diagnostics;
#[allow(dead_code)]
mod hive;
#[allow(dead_code)]
mod summary;

#[allow(unused_imports)]
pub(crate) use diagnostics::*;
pub(crate) use hive::*;
#[allow(unused_imports)]
pub(crate) use summary::*;

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
        let visible_entries = entries.iter().collect::<Vec<_>>();
        for collision in awareness_summary_diagnostics(&visible_entries) {
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

pub(crate) fn project_awareness_visible_entries(
    response: &ProjectAwarenessResponse,
) -> Vec<&ProjectAwarenessEntry> {
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
    collisions.extend(branch_collision_warnings(&entries));
    let visible_entries = entries.iter().collect::<Vec<_>>();
    collisions.extend(work_overlap_warnings(&visible_entries));
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
    let (current_bundle, current_project, scan_root) = resolve_awareness_paths(args)?;
    let codebase_live_map = update_codebase_live_map(&current_bundle, &current_project)
        .unwrap_or_else(|err| CodebaseLiveMapUpdate {
            status: "unknown".to_string(),
            diagnostics: vec![format!(
                "codebase_live_map_error {}",
                compact_inline(&err.to_string(), 160)
            )],
        });
    if codebase_live_map.status == "blocked" {
        let mut collisions = codebase_live_map.diagnostics;
        collisions.push(format!(
            "awareness_scan_skipped status=blocked root={} action=use_cached_live_map_and_retry_after_host_recovery",
            scan_root.display()
        ));
        return Ok(ProjectAwarenessResponse {
            root: scan_root.display().to_string(),
            current_bundle: current_bundle.display().to_string(),
            collisions,
            entries: Vec::new(),
        });
    }
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
            auto_commit: BundleAutoCommitConfig::default(),
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
    collisions.extend(branch_collision_warnings(&entries));
    let visible_entries = entries.iter().collect::<Vec<_>>();
    collisions.extend(work_overlap_warnings(&visible_entries));
    collisions.extend(codebase_live_map.diagnostics);

    Ok(ProjectAwarenessResponse {
        root: scan_root.display().to_string(),
        current_bundle: current_bundle.display().to_string(),
        collisions,
        entries,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CodebaseLiveMapState {
    repo_root: String,
    fingerprint: String,
    file_count: usize,
    newest_mtime_unix: i64,
    updated_at: DateTime<Utc>,
    status: String,
    needs_reread: bool,
    autosync: String,
    #[serde(default)]
    blockers: Vec<String>,
    #[serde(default)]
    files: std::collections::BTreeMap<String, CodebaseLiveMapFile>,
    #[serde(default)]
    last_changes: CodebaseLiveMapDiff,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CodebaseLiveMapFile {
    len: u64,
    mtime_unix: i64,
    #[serde(default)]
    content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CodebaseLiveMapDiff {
    added_count: usize,
    modified_count: usize,
    deleted_count: usize,
    #[serde(default)]
    baseline_available: bool,
    #[serde(default)]
    added: Vec<String>,
    #[serde(default)]
    modified: Vec<String>,
    #[serde(default)]
    deleted: Vec<String>,
    truncated: bool,
}

#[derive(Debug, Clone)]
struct CodebaseLiveMapUpdate {
    status: String,
    diagnostics: Vec<String>,
}

pub(crate) fn refresh_codebase_live_map_for_bundle(output: &Path) -> anyhow::Result<Vec<String>> {
    let bundle_root = if output.is_absolute() {
        output.to_path_buf()
    } else {
        std::env::current_dir()?.join(output)
    };
    let bundle_root = fs::canonicalize(&bundle_root).unwrap_or(bundle_root);
    let repo_root = bundle_root
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    update_codebase_live_map(&bundle_root, &repo_root).map(|update| update.diagnostics)
}

pub(crate) fn record_codebase_live_map_event(
    output: &Path,
    source: &str,
    paths: &[String],
) -> anyhow::Result<()> {
    let paths = paths
        .iter()
        .map(|path| path.trim())
        .filter(|path| !path.is_empty())
        .take(64)
        .map(str::to_string)
        .collect::<Vec<_>>();
    if paths.is_empty() {
        return Ok(());
    }
    let bundle_root = if output.is_absolute() {
        output.to_path_buf()
    } else {
        std::env::current_dir()?.join(output)
    };
    let bundle_root = fs::canonicalize(&bundle_root).unwrap_or(bundle_root);
    let state_dir = bundle_root.join("state");
    fs::create_dir_all(&state_dir)
        .with_context(|| format!("create live-map state dir {}", state_dir.display()))?;
    let event_path = state_dir.join("codebase-live-map-events.ndjson");
    let event = CodebaseLiveMapEvent {
        ts: Utc::now(),
        source: source.to_string(),
        paths,
    };
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&event_path)
        .with_context(|| format!("open {}", event_path.display()))?;
    use std::io::Write as _;
    writeln!(file, "{}", serde_json::to_string(&event)?)
        .with_context(|| format!("write {}", event_path.display()))?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CodebaseLiveMapEvent {
    ts: DateTime<Utc>,
    source: String,
    paths: Vec<String>,
}

fn update_codebase_live_map(
    bundle_root: &Path,
    repo_root: &Path,
) -> anyhow::Result<CodebaseLiveMapUpdate> {
    let state_dir = bundle_root.join("state");
    fs::create_dir_all(&state_dir)
        .with_context(|| format!("create live-map state dir {}", state_dir.display()))?;
    let state_path = state_dir.join("codebase-live-map.json");
    let previous = fs::read_to_string(&state_path)
        .ok()
        .and_then(|raw| serde_json::from_str::<CodebaseLiveMapState>(&raw).ok());
    let recent_events = read_recent_codebase_live_map_events(bundle_root, 24);
    let blockers = host_process_live_map_blockers(bundle_root, repo_root);
    if !blockers.is_empty() {
        let previous_status = previous
            .as_ref()
            .map(|state| state.status.as_str())
            .unwrap_or("unknown");
        let previous_file_count = previous.as_ref().map(|state| state.file_count).unwrap_or(0);
        let previous_newest_mtime = previous
            .as_ref()
            .map(|state| state.newest_mtime_unix)
            .unwrap_or(0);
        let blocked_state = CodebaseLiveMapState {
            repo_root: repo_root.display().to_string(),
            fingerprint: previous
                .as_ref()
                .map(|state| state.fingerprint.clone())
                .unwrap_or_else(|| "blocked-no-scan".to_string()),
            file_count: previous_file_count,
            newest_mtime_unix: previous_newest_mtime,
            updated_at: Utc::now(),
            status: "blocked".to_string(),
            needs_reread: true,
            autosync: "blocked_no_scan".to_string(),
            blockers: blockers.clone(),
            files: previous
                .as_ref()
                .map(|state| state.files.clone())
                .unwrap_or_default(),
            last_changes: previous
                .as_ref()
                .map(|state| state.last_changes.clone())
                .unwrap_or_default(),
        };
        fs::write(&state_path, serde_json::to_vec_pretty(&blocked_state)?)
            .with_context(|| format!("write {}", state_path.display()))?;
        let mut diagnostics = vec![format!(
            "codebase_live_map status=blocked autosync=blocked_no_scan reread_required=true previous_status={} files={} newest_mtime={} recent_events={} state={} action=wait_for_host_io_recovery",
            previous_status,
            previous_file_count,
            previous_newest_mtime,
            recent_events.len(),
            state_path.display()
        )];
        diagnostics.extend(blockers);
        if !recent_events.is_empty() {
            diagnostics.push(format!(
                "codebase_event_log events={} sample=\"{}\" action=merge_tool_events_into_live_map_after_host_recovery",
                recent_events.len(),
                compact_inline(&codebase_live_map_event_sample(&recent_events), 220)
            ));
        }
        return Ok(CodebaseLiveMapUpdate {
            status: "blocked".to_string(),
            diagnostics,
        });
    }
    if let Some(cached) = previous.as_ref() {
        let ttl_secs = codebase_live_map_ttl_secs();
        let age_secs = Utc::now()
            .signed_duration_since(cached.updated_at)
            .num_seconds()
            .max(0);
        let newer_events = codebase_live_map_events_after(&recent_events, cached.updated_at);
        if age_secs <= ttl_secs && newer_events == 0 {
            let mut diagnostics = vec![format!(
                "codebase_live_map status={} autosync=cached_no_rescan reread_required={} age_secs={} ttl_secs={} files={} newest_mtime={} recent_events={} state={}",
                cached.status,
                cached.needs_reread,
                age_secs,
                ttl_secs,
                cached.file_count,
                cached.newest_mtime_unix,
                recent_events.len(),
                state_path.display()
            )];
            if !recent_events.is_empty() {
                diagnostics.push(format!(
                    "codebase_event_log events={} sample=\"{}\" action=merge_tool_events_into_cached_live_map",
                    recent_events.len(),
                    compact_inline(&codebase_live_map_event_sample(&recent_events), 220)
                ));
            }
            return Ok(CodebaseLiveMapUpdate {
                status: cached.status.clone(),
                diagnostics,
            });
        }
    }
    let snapshot = scan_codebase_live_map(repo_root)?;
    let previous_trusted = previous
        .as_ref()
        .filter(|state| codebase_live_map_has_trusted_baseline(state));
    let diff = diff_codebase_live_map(previous_trusted, &snapshot.files);
    let changed = previous_trusted.is_some_and(|state| state.fingerprint != snapshot.fingerprint);
    let needs_reread = changed;
    let status = if changed { "out_of_sync" } else { "fresh" };
    let autosync = if needs_reread {
        "updated_map_reread_required"
    } else if previous_trusted.is_none() {
        "initialized_map_no_reread"
    } else {
        "updated_map_no_reread"
    };
    let next_state = CodebaseLiveMapState {
        repo_root: repo_root.display().to_string(),
        fingerprint: snapshot.fingerprint.clone(),
        file_count: snapshot.file_count,
        newest_mtime_unix: snapshot.newest_mtime_unix,
        updated_at: Utc::now(),
        status: status.to_string(),
        needs_reread,
        autosync: autosync.to_string(),
        blockers: blockers.clone(),
        files: snapshot.files.clone(),
        last_changes: diff.clone(),
    };
    fs::write(&state_path, serde_json::to_vec_pretty(&next_state)?)
        .with_context(|| format!("write {}", state_path.display()))?;

    let mut diagnostics = vec![format!(
        "codebase_live_map status={} autosync={} reread_required={} files={} newest_mtime={} state={}",
        status,
        autosync,
        needs_reread,
        snapshot.file_count,
        snapshot.newest_mtime_unix,
        state_path.display()
    )];
    if changed {
        diagnostics.push(format!(
            "codebase_out_of_sync previous={} current={} action=reread_changed_files",
            previous_trusted
                .map(|state| state.fingerprint.as_str())
                .unwrap_or("none"),
            snapshot.fingerprint
        ));
        diagnostics.push(format!(
            "codebase_diff added={} modified={} deleted={} sample=\"{}\" truncated={}",
            diff.added_count,
            diff.modified_count,
            diff.deleted_count,
            compact_inline(&codebase_diff_sample(&diff), 220),
            diff.truncated
        ));
    }
    for blocker in blockers {
        diagnostics.push(blocker);
    }
    if !recent_events.is_empty() {
        diagnostics.push(format!(
            "codebase_event_log events={} sample=\"{}\" action=merge_tool_events_into_live_map",
            recent_events.len(),
            compact_inline(&codebase_live_map_event_sample(&recent_events), 220)
        ));
    }

    Ok(CodebaseLiveMapUpdate {
        status: status.to_string(),
        diagnostics,
    })
}

fn codebase_live_map_has_trusted_baseline(state: &CodebaseLiveMapState) -> bool {
    !state.files.is_empty()
        && !matches!(
            state.fingerprint.as_str(),
            "blocked-no-scan"
                | "host-io-blocked-no-scan"
                | "host-io-clear-no-scan"
                | "missing-no-scan"
        )
}

fn codebase_live_map_ttl_secs() -> i64 {
    std::env::var("MEMD_CODEBASE_LIVE_MAP_TTL_SECS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value >= 0)
        .unwrap_or(15)
}

fn read_recent_codebase_live_map_events(
    bundle_root: &Path,
    limit: usize,
) -> Vec<CodebaseLiveMapEvent> {
    let path = bundle_root
        .join("state")
        .join("codebase-live-map-events.ndjson");
    let Ok(raw) = fs::read_to_string(path) else {
        return Vec::new();
    };
    raw.lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<CodebaseLiveMapEvent>(line).ok())
        .take(limit)
        .collect::<Vec<_>>()
}

fn codebase_live_map_event_sample(events: &[CodebaseLiveMapEvent]) -> String {
    events
        .iter()
        .take(6)
        .map(|event| {
            format!(
                "{}:{}",
                event.source,
                event
                    .paths
                    .iter()
                    .take(4)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(",")
            )
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

fn codebase_live_map_events_after(
    events: &[CodebaseLiveMapEvent],
    timestamp: DateTime<Utc>,
) -> usize {
    events.iter().filter(|event| event.ts > timestamp).count()
}

#[derive(Debug, Clone)]
struct CodebaseLiveMapSnapshot {
    fingerprint: String,
    file_count: usize,
    newest_mtime_unix: i64,
    files: std::collections::BTreeMap<String, CodebaseLiveMapFile>,
}

fn scan_codebase_live_map(repo_root: &Path) -> anyhow::Result<CodebaseLiveMapSnapshot> {
    let mut files = Vec::new();
    collect_codebase_live_map_files(repo_root, repo_root, &mut files, 2048)?;
    files.sort();

    let mut hasher = Sha256::new();
    let mut newest_mtime_unix = 0_i64;
    let mut file_map = std::collections::BTreeMap::new();
    for (relative, len, mtime, content_hash) in &files {
        hasher.update(relative.as_bytes());
        hasher.update([0]);
        hasher.update(len.to_le_bytes());
        hasher.update(content_hash.as_bytes());
        newest_mtime_unix = newest_mtime_unix.max(*mtime);
        file_map.insert(
            relative.clone(),
            CodebaseLiveMapFile {
                len: *len,
                mtime_unix: *mtime,
                content_hash: content_hash.clone(),
            },
        );
    }
    Ok(CodebaseLiveMapSnapshot {
        fingerprint: format!("{:x}", hasher.finalize()),
        file_count: files.len(),
        newest_mtime_unix,
        files: file_map,
    })
}

fn diff_codebase_live_map(
    previous: Option<&CodebaseLiveMapState>,
    current: &std::collections::BTreeMap<String, CodebaseLiveMapFile>,
) -> CodebaseLiveMapDiff {
    let Some(previous) = previous else {
        return CodebaseLiveMapDiff::default();
    };

    let mut diff = CodebaseLiveMapDiff {
        baseline_available: true,
        ..CodebaseLiveMapDiff::default()
    };
    for (path, current_file) in current {
        match previous.files.get(path) {
            None => {
                diff.added_count += 1;
                push_codebase_diff_sample(&mut diff.added, path, &mut diff.truncated);
            }
            Some(previous_file) if codebase_live_map_file_changed(previous_file, current_file) => {
                diff.modified_count += 1;
                push_codebase_diff_sample(&mut diff.modified, path, &mut diff.truncated);
            }
            Some(_) => {}
        }
    }
    for path in previous.files.keys() {
        if !current.contains_key(path) {
            diff.deleted_count += 1;
            push_codebase_diff_sample(&mut diff.deleted, path, &mut diff.truncated);
        }
    }
    diff
}

fn codebase_live_map_file_changed(
    previous: &CodebaseLiveMapFile,
    current: &CodebaseLiveMapFile,
) -> bool {
    if !previous.content_hash.is_empty() && !current.content_hash.is_empty() {
        previous.len != current.len || previous.content_hash != current.content_hash
    } else {
        previous != current
    }
}

fn push_codebase_diff_sample(paths: &mut Vec<String>, path: &str, truncated: &mut bool) {
    const SAMPLE_LIMIT: usize = 12;
    if paths.len() < SAMPLE_LIMIT {
        paths.push(path.to_string());
    } else {
        *truncated = true;
    }
}

fn codebase_diff_sample(diff: &CodebaseLiveMapDiff) -> String {
    let mut parts = Vec::new();
    if !diff.added.is_empty() {
        parts.push(format!("added:[{}]", diff.added.join(",")));
    }
    if !diff.modified.is_empty() {
        parts.push(format!("modified:[{}]", diff.modified.join(",")));
    }
    if !diff.deleted.is_empty() {
        parts.push(format!("deleted:[{}]", diff.deleted.join(",")));
    }
    if parts.is_empty() {
        "none".to_string()
    } else {
        parts.join(" ")
    }
}

fn collect_codebase_live_map_files(
    repo_root: &Path,
    dir: &Path,
    out: &mut Vec<(String, u64, i64, String)>,
    limit: usize,
) -> anyhow::Result<()> {
    if out.len() >= limit {
        return Ok(());
    }
    for entry in fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
        if out.len() >= limit {
            break;
        }
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if should_skip_live_map_path(repo_root, &path, &name) {
            continue;
        }
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        if metadata.is_dir() {
            collect_codebase_live_map_files(repo_root, &path, out, limit)?;
            continue;
        }
        if !metadata.is_file() {
            continue;
        }
        let relative = path
            .strip_prefix(repo_root)
            .unwrap_or(&path)
            .display()
            .to_string();
        let mtime = metadata
            .modified()
            .ok()
            .map(DateTime::<Utc>::from)
            .map(|value| value.timestamp())
            .unwrap_or_default();
        let content_hash = fs::read(&path)
            .ok()
            .map(|bytes| format!("{:x}", Sha256::digest(bytes)))
            .unwrap_or_default();
        out.push((relative, metadata.len(), mtime, content_hash));
    }
    Ok(())
}

fn should_skip_live_map_path(repo_root: &Path, path: &Path, name: &str) -> bool {
    if matches!(
        name,
        ".git"
            | "target"
            | "node_modules"
            | ".next"
            | "dist"
            | "build"
            | ".DS_Store"
            | "codebase-live-map.json"
            | "codebase-live-map-events.ndjson"
            | "host-io-guard.txt"
    ) {
        return true;
    }
    let Ok(relative) = path.strip_prefix(repo_root) else {
        return false;
    };
    let mut components = relative
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => value.to_str(),
            _ => None,
        });
    if components.next() == Some(".memd") {
        return !matches!(components.next(), Some("config.json" | "env" | "env.ps1"));
    }
    false
}

fn host_process_live_map_blockers(bundle_root: &Path, repo_root: &Path) -> Vec<String> {
    if let Some(diagnostics) = host_io_guard_report_blockers(bundle_root) {
        return diagnostics;
    }
    if matches!(
        std::env::var("MEMD_CODEBASE_LIVE_MAP_SKIP_HOST_PROCESS_SCAN")
            .ok()
            .as_deref(),
        Some("1" | "true" | "yes")
    ) {
        return Vec::new();
    }
    let mut command = Command::new("ps");
    command.args(["-axo", "pid,ppid,state,command"]);
    let output = match command_output_with_timeout(command, host_process_scan_timeout()) {
        Ok(Some(output)) => output,
        Ok(None) => {
            return vec![format!(
                "host_process_scan_timeout timeout_ms={} action=use_durable_host_io_snapshot_or_retry_later",
                host_process_scan_timeout().as_millis()
            )];
        }
        Err(_) => return Vec::new(),
    };
    if !output.status.success() {
        return Vec::new();
    }
    let process_lines = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let repo_root_text = repo_root.display().to_string();
    let volume = volume_root_for_path(repo_root);
    let mut diagnostics = process_lines
        .iter()
        .filter_map(|line| {
            host_process_live_map_diagnostic(line, &repo_root_text, volume.as_deref())
        })
        .collect::<Vec<_>>();
    diagnostics.extend(host_filesystem_live_map_diagnostics(
        repo_root,
        &process_lines,
    ));
    diagnostics
}

fn command_output_with_timeout(
    mut command: Command,
    timeout: std::time::Duration,
) -> anyhow::Result<Option<std::process::Output>> {
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());
    let mut child = command.spawn()?;
    let started = std::time::Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            return Ok(Some(child.wait_with_output()?));
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            return Ok(None);
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

fn host_process_scan_timeout() -> std::time::Duration {
    let millis = std::env::var("MEMD_HOST_PROCESS_SCAN_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(1500);
    std::time::Duration::from_millis(millis)
}

fn host_io_guard_report_blockers(bundle_root: &Path) -> Option<Vec<String>> {
    let path = bundle_root.join("state").join("host-io-guard.txt");
    let raw = fs::read_to_string(&path).ok()?;
    let mut status = None;
    let mut ts = None;
    let mut blockers = Vec::new();
    for line in raw.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if let Some(value) = line.strip_prefix("status=") {
            status = Some(value.to_string());
            continue;
        }
        if let Some(value) = line.strip_prefix("ts=") {
            ts = Some(value.to_string());
            continue;
        }
        if line.starts_with("repo=") || line.starts_with("pid=") {
            continue;
        }
        if line.contains("project_hint=host-io-report") {
            continue;
        }
        blockers.push(line.to_string());
    }
    let ts = ts?;
    let parsed = DateTime::parse_from_rfc3339(&ts)
        .ok()
        .map(|value| value.with_timezone(&Utc))?;
    let age_secs = Utc::now()
        .signed_duration_since(parsed)
        .num_seconds()
        .max(0);
    let ttl_secs = host_io_guard_report_ttl_secs();
    if age_secs > ttl_secs {
        return None;
    }
    if status.as_deref() == Some("clear") {
        return Some(Vec::new());
    }
    if status.as_deref() != Some("blocked") || blockers.is_empty() {
        return None;
    }
    let mut diagnostics = vec![format!(
        "host_io_guard_report status=blocked age_secs={} ttl_secs={} state={} action=use_durable_host_io_snapshot",
        age_secs,
        ttl_secs,
        path.display()
    )];
    diagnostics.extend(blockers);
    Some(diagnostics)
}

fn host_io_guard_report_ttl_secs() -> i64 {
    std::env::var("MEMD_HOST_IO_REPORT_TTL_SECS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value >= 0)
        .unwrap_or(120)
}

fn host_process_live_map_diagnostic(
    line: &str,
    repo_root: &str,
    volume: Option<&str>,
) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("PID ") {
        return None;
    }
    let parts = trimmed.split_whitespace().collect::<Vec<_>>();
    if parts.len() < 4 {
        return None;
    }
    let pid = parts[0];
    let state = parts[2];
    let command = parts[3..].join(" ");
    let interesting = [
        "git", "cargo", "rustc", "rustfmt", "clang", "clang++", "cc", "c++", "vitest", "tsc",
    ]
    .iter()
    .any(|tool| host_process_command_mentions_tool(&command, tool));
    if !interesting {
        return None;
    }
    let stuck = state.contains('U');
    let build = ["cargo", "rustc"]
        .iter()
        .any(|tool| host_process_command_mentions_tool(&command, tool));
    if !stuck && !build {
        return None;
    }
    let scope = host_process_scope(&command, repo_root, volume);
    let kind = if stuck {
        "host_process_blocked"
    } else {
        "host_build_active"
    };
    Some(format!(
        "{} pid={} state={} scope={} project_hint={} command=\"{}\" action=warn_and_autosync_live_map",
        kind,
        pid,
        state,
        scope,
        host_process_project_hint(&command).unwrap_or_else(|| "unknown".to_string()),
        compact_inline(&command, 180)
    ))
}

fn host_process_command_mentions_tool(command: &str, tool: &str) -> bool {
    command == tool
        || command.starts_with(&format!("{tool} "))
        || command.contains(&format!(" {tool} "))
        || command.ends_with(&format!("/{tool}"))
        || command.contains(&format!("/{tool} "))
}

fn host_process_scope(command: &str, repo_root: &str, volume: Option<&str>) -> String {
    if !repo_root.is_empty() && command.contains(repo_root) {
        return "repo".to_string();
    }
    if let Some(volume) = volume
        && command.contains(volume)
    {
        return format!("volume:{volume}");
    }
    "unknown".to_string()
}

fn host_process_project_hint(command: &str) -> Option<String> {
    let marker = "/projects/";
    if let Some((_, rest)) = command.split_once(marker) {
        let name = rest
            .split(['/', ' ', '"', '\''])
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())?;
        return Some(name.to_string());
    }
    if command.contains("/Xcode")
        && command.contains(".app/")
        && host_process_command_mentions_tool(command, "git")
    {
        return Some("app-git".to_string());
    }
    if ["cargo", "rustc", "rustfmt"]
        .iter()
        .any(|tool| host_process_command_mentions_tool(command, tool))
    {
        return Some("cargo-tooling".to_string());
    }
    if ["clang", "clang++", "cc", "c++"]
        .iter()
        .any(|tool| host_process_command_mentions_tool(command, tool))
    {
        return Some("native-tooling".to_string());
    }
    if ["vitest", "tsc"]
        .iter()
        .any(|tool| host_process_command_mentions_tool(command, tool))
    {
        return Some("node-tooling".to_string());
    }
    None
}

fn host_filesystem_live_map_diagnostics(repo_root: &Path, process_lines: &[String]) -> Vec<String> {
    let volume = volume_root_for_path(repo_root);
    let Some(volume) = volume else {
        return Vec::new();
    };
    let blocked_on_volume = process_lines
        .iter()
        .filter(|line| {
            line.split_whitespace()
                .nth(2)
                .is_some_and(|state| state.contains('U'))
        })
        .filter(|line| line.contains(&volume))
        .count();
    let uvfs_blocked = process_lines.iter().any(|line| {
        line.contains("UVFSService")
            && line
                .split_whitespace()
                .nth(2)
                .is_some_and(|state| state.contains('U'))
    });
    let spotlight_blocked = process_lines.iter().any(|line| {
        (line.contains("/mds") || line.contains("mds_stores"))
            && line
                .split_whitespace()
                .nth(2)
                .is_some_and(|state| state.contains('U'))
    });
    if blocked_on_volume == 0 && !uvfs_blocked && !spotlight_blocked {
        return Vec::new();
    }
    vec![format!(
        "host_filesystem_blocked volume={} blocked_processes={} uvfs_blocked={} spotlight_blocked={} action=pause_t7_git_cargo_and_recover_filesystem",
        volume, blocked_on_volume, uvfs_blocked, spotlight_blocked
    )]
}

fn volume_root_for_path(path: &Path) -> Option<String> {
    path.display()
        .to_string()
        .strip_prefix("/Volumes/")
        .and_then(|rest| rest.split('/').next())
        .map(|name| format!("/Volumes/{name}"))
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
        let left_touches = hive::awareness_overlap_touch_points(left);
        if left_touches.is_empty() {
            continue;
        }
        for right in active_entries.iter().skip(idx + 1) {
            let right_touches = hive::awareness_overlap_touch_points(right);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_process_diagnostic_marks_same_volume_project_blocker() {
        let line = "67435 67400 U /Volumes/T7/Xcodes/Xcode.app/Contents/Developer/usr/bin/git -C /Volumes/T7/projects/clawcontrol status --short";
        let diagnostic = host_process_live_map_diagnostic(
            line,
            "/Volumes/T7/projects/memd",
            Some("/Volumes/T7"),
        )
        .expect("diagnostic");

        assert!(diagnostic.contains("host_process_blocked"));
        assert!(diagnostic.contains("scope=volume:/Volumes/T7"));
        assert!(diagnostic.contains("project_hint=clawcontrol"));
    }

    #[test]
    fn host_process_diagnostic_marks_same_repo_blocker() {
        let line = "66964 800 U git -C /Volumes/T7/projects/memd status --short";
        let diagnostic = host_process_live_map_diagnostic(
            line,
            "/Volumes/T7/projects/memd",
            Some("/Volumes/T7"),
        )
        .expect("diagnostic");

        assert!(diagnostic.contains("host_process_blocked"));
        assert!(diagnostic.contains("scope=repo"));
        assert!(diagnostic.contains("project_hint=memd"));
    }

    #[test]
    fn host_process_diagnostic_marks_app_owned_git_blocker() {
        let line = "84445 1 U /Volumes/T7/Xcodes/Xcode-26.4.1.app/Contents/Developer/usr/bin/git -c core.hooksPath=/dev/null -c core.fsmonitor=false status --porcelain=v1 -z";
        let diagnostic = host_process_live_map_diagnostic(
            line,
            "/Volumes/T7/projects/memd",
            Some("/Volumes/T7"),
        )
        .expect("diagnostic");

        assert!(diagnostic.contains("host_process_blocked"));
        assert!(diagnostic.contains("scope=volume:/Volumes/T7"));
        assert!(diagnostic.contains("project_hint=app-git"));
    }

    #[test]
    fn host_process_diagnostic_marks_stuck_formatter_blocker() {
        let line = "75178 1 U /Volumes/T7/.rustup/toolchains/stable-aarch64-apple-darwin/bin/rustfmt /Volumes/T7/projects/clawcontrol/src-tauri/build.rs";
        let diagnostic = host_process_live_map_diagnostic(
            line,
            "/Volumes/T7/projects/memd",
            Some("/Volumes/T7"),
        )
        .expect("diagnostic");

        assert!(diagnostic.contains("host_process_blocked"));
        assert!(diagnostic.contains("scope=volume:/Volumes/T7"));
        assert!(diagnostic.contains("project_hint=clawcontrol"));
        assert!(diagnostic.contains("rustfmt"));
    }

    #[test]
    fn host_process_diagnostic_marks_native_tooling_blocker() {
        let line = "85222 1 U /Volumes/T7/Xcodes/Xcode-26.4.1.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/clang -c build/native.o";
        let diagnostic = host_process_live_map_diagnostic(
            line,
            "/Volumes/T7/projects/memd",
            Some("/Volumes/T7"),
        )
        .expect("diagnostic");

        assert!(diagnostic.contains("host_process_blocked"));
        assert!(diagnostic.contains("scope=volume:/Volumes/T7"));
        assert!(diagnostic.contains("project_hint=native-tooling"));
    }

    #[test]
    fn host_io_guard_report_blockers_uses_fresh_blocked_snapshot() {
        let root = unique_awareness_test_dir("fresh-host-io-report");
        let state_dir = root.join("state");
        fs::create_dir_all(&state_dir).expect("create state dir");
        fs::write(
            state_dir.join("host-io-guard.txt"),
            format!(
                "ts={}\nrepo=/Volumes/T7/projects/memd\npid=42\nstatus=blocked\nrepo project_hint=host-io-report pid=41 state=cached command=.memd/state/host-io-guard.txt age_s=1 ttl_s=120\nrepo project_hint=memd pid=12 state=U command=git -C /Volumes/T7/projects/memd status --short\n",
                Utc::now().to_rfc3339()
            ),
        )
        .expect("write report");

        let diagnostics = host_io_guard_report_blockers(&root).expect("diagnostics");
        assert!(diagnostics[0].contains("host_io_guard_report status=blocked"));
        assert!(
            diagnostics
                .iter()
                .any(|line| line.contains("repo project_hint=memd"))
        );
        assert!(
            diagnostics
                .iter()
                .all(|line| !line.contains("project_hint=host-io-report"))
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn host_io_guard_report_blockers_ignores_stale_snapshot() {
        let root = unique_awareness_test_dir("stale-host-io-report");
        let state_dir = root.join("state");
        fs::create_dir_all(&state_dir).expect("create state dir");
        fs::write(
            state_dir.join("host-io-guard.txt"),
            format!(
                "ts={}\nrepo=/Volumes/T7/projects/memd\npid=42\nstatus=blocked\nrepo project_hint=memd pid=12 state=U command=git -C /Volumes/T7/projects/memd status --short\n",
                (Utc::now() - chrono::Duration::seconds(300)).to_rfc3339()
            ),
        )
        .expect("write report");

        assert!(host_io_guard_report_blockers(&root).is_none());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn command_output_with_timeout_stops_slow_process() {
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 1"]);

        let output = command_output_with_timeout(command, std::time::Duration::from_millis(10))
            .expect("timeout command");

        assert!(output.is_none());
    }

    #[test]
    fn command_output_with_timeout_returns_fast_output() {
        let mut command = Command::new("sh");
        command.args(["-c", "printf ok"]);

        let output = command_output_with_timeout(command, std::time::Duration::from_secs(1))
            .expect("fast command")
            .expect("output");

        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout), "ok");
    }

    fn unique_awareness_test_dir(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "memd-awareness-{name}-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        path
    }

    #[test]
    fn live_map_skips_memd_transient_state_files() {
        let repo = Path::new("/Volumes/T7/projects/memd");

        assert!(should_skip_live_map_path(
            repo,
            Path::new("/Volumes/T7/projects/memd/.memd/state/host-io-guard.txt"),
            "host-io-guard.txt",
        ));
        assert!(should_skip_live_map_path(
            repo,
            Path::new("/Volumes/T7/projects/memd/.memd/state/codebase-live-map-events.ndjson"),
            "codebase-live-map-events.ndjson",
        ));
        assert!(should_skip_live_map_path(
            repo,
            Path::new("/Volumes/T7/projects/memd/.memd/logs/hook-trace.ndjson"),
            "hook-trace.ndjson",
        ));
        assert!(should_skip_live_map_path(
            repo,
            Path::new("/Volumes/T7/projects/memd/.memd/compiled/memory/working.md"),
            "working.md",
        ));
        assert!(should_skip_live_map_path(
            repo,
            Path::new("/Volumes/T7/projects/memd/.memd/agents/AGENTS.md.example"),
            "AGENTS.md.example",
        ));
        assert!(should_skip_live_map_path(
            repo,
            Path::new("/Volumes/T7/projects/memd/.memd/wake.md"),
            "wake.md",
        ));
        assert!(should_skip_live_map_path(
            repo,
            Path::new("/Volumes/T7/projects/memd/.memd/models/fastembed/model.onnx"),
            "model.onnx",
        ));
        assert!(!should_skip_live_map_path(
            repo,
            Path::new("/Volumes/T7/projects/memd/.memd/config.json"),
            "config.json",
        ));
        assert!(!should_skip_live_map_path(
            repo,
            Path::new("/Volumes/T7/projects/memd/.memd/env"),
            "env",
        ));
        assert!(!should_skip_live_map_path(
            repo,
            Path::new("/Volumes/T7/projects/memd/crates/memd-client/src/awareness/mod.rs"),
            "mod.rs",
        ));
    }

    #[test]
    fn live_map_seed_scan_initializes_without_whole_repo_diff() {
        let repo = unique_awareness_test_dir("seeded-live-map");
        let bundle = repo.join(".memd");
        let state_dir = bundle.join("state");
        let src_dir = repo.join("src");
        fs::create_dir_all(&state_dir).expect("create state dir");
        fs::create_dir_all(&src_dir).expect("create src dir");
        fs::write(src_dir.join("lib.rs"), "pub fn alive() -> bool { true }\n")
            .expect("write source");

        let seeded = CodebaseLiveMapState {
            repo_root: repo.display().to_string(),
            fingerprint: "host-io-clear-no-scan".to_string(),
            file_count: 0,
            newest_mtime_unix: 0,
            updated_at: Utc::now() - chrono::Duration::seconds(300),
            status: "out_of_sync".to_string(),
            needs_reread: true,
            autosync: "host_io_clear_rescan_required".to_string(),
            blockers: Vec::new(),
            files: std::collections::BTreeMap::new(),
            last_changes: CodebaseLiveMapDiff::default(),
        };
        fs::write(
            state_dir.join("codebase-live-map.json"),
            serde_json::to_vec_pretty(&seeded).expect("serialize seeded map"),
        )
        .expect("write seeded map");
        fs::write(
            state_dir.join("host-io-guard.txt"),
            format!(
                "ts={}\nrepo={}\npid=42\nstatus=clear\n",
                Utc::now().to_rfc3339(),
                repo.display()
            ),
        )
        .expect("write clear host report");

        let update = update_codebase_live_map(&bundle, &repo).expect("update live map");

        assert_eq!(update.status, "fresh");
        let persisted = fs::read_to_string(state_dir.join("codebase-live-map.json"))
            .expect("read persisted map");
        let state: CodebaseLiveMapState =
            serde_json::from_str(&persisted).expect("parse persisted map");
        assert_eq!(state.status, "fresh");
        assert!(!state.needs_reread);
        assert_eq!(state.autosync, "initialized_map_no_reread");
        assert!(state.files.contains_key("src/lib.rs"));
        assert!(!state.last_changes.baseline_available);
        assert_eq!(state.last_changes.added_count, 0);
        assert_eq!(state.last_changes.modified_count, 0);
        assert_eq!(state.last_changes.deleted_count, 0);

        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn blocked_live_map_persists_state_without_rescan() {
        let repo = unique_awareness_test_dir("blocked-live-map");
        let bundle = repo.join(".memd");
        let state_dir = bundle.join("state");
        fs::create_dir_all(&state_dir).expect("create state dir");

        let mut files = std::collections::BTreeMap::new();
        files.insert(
            "src/lib.rs".to_string(),
            CodebaseLiveMapFile {
                len: 12,
                mtime_unix: 123,
                content_hash: "same-content".to_string(),
            },
        );
        let previous = CodebaseLiveMapState {
            repo_root: repo.display().to_string(),
            fingerprint: "previous-fingerprint".to_string(),
            file_count: 1,
            newest_mtime_unix: 123,
            updated_at: Utc::now() - chrono::Duration::seconds(300),
            status: "fresh".to_string(),
            needs_reread: false,
            autosync: "updated_map_no_reread".to_string(),
            blockers: Vec::new(),
            files,
            last_changes: CodebaseLiveMapDiff::default(),
        };
        fs::write(
            state_dir.join("codebase-live-map.json"),
            serde_json::to_vec_pretty(&previous).expect("serialize previous"),
        )
        .expect("write previous map");
        fs::write(
            state_dir.join("host-io-guard.txt"),
            format!(
                "ts={}\nrepo={}\npid=42\nstatus=blocked\nvolume:/Volumes/T7 project_hint=app-git pid=99 state=U command=git status\n",
                Utc::now().to_rfc3339(),
                repo.display()
            ),
        )
        .expect("write host report");

        let update = update_codebase_live_map(&bundle, &repo).expect("update blocked map");

        assert_eq!(update.status, "blocked");
        assert!(
            update
                .diagnostics
                .iter()
                .any(|line| line.contains("host_io_guard_report status=blocked"))
        );
        let persisted = fs::read_to_string(state_dir.join("codebase-live-map.json"))
            .expect("read persisted map");
        let state: CodebaseLiveMapState =
            serde_json::from_str(&persisted).expect("parse persisted map");
        assert_eq!(state.status, "blocked");
        assert!(state.needs_reread);
        assert_eq!(state.autosync, "blocked_no_scan");
        assert_eq!(state.fingerprint, "previous-fingerprint");
        assert!(state.files.contains_key("src/lib.rs"));
        assert!(
            state
                .blockers
                .iter()
                .any(|line| line.contains("project_hint=app-git"))
        );

        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn live_map_newer_event_bypasses_fresh_cache() {
        let repo = unique_awareness_test_dir("event-bypasses-cache");
        let bundle = repo.join(".memd");
        let state_dir = bundle.join("state");
        let src_dir = repo.join("src");
        fs::create_dir_all(&state_dir).expect("create state dir");
        fs::create_dir_all(&src_dir).expect("create src dir");
        let source_path = src_dir.join("lib.rs");
        fs::write(&source_path, "pub fn value() -> u8 { 1 }\n").expect("write initial source");
        fs::write(
            state_dir.join("host-io-guard.txt"),
            format!(
                "ts={}\nrepo={}\npid=42\nstatus=clear\n",
                Utc::now().to_rfc3339(),
                repo.display()
            ),
        )
        .expect("write clear host report");

        let initial = update_codebase_live_map(&bundle, &repo).expect("initial map");
        assert_eq!(initial.status, "fresh");
        std::thread::sleep(std::time::Duration::from_millis(20));
        fs::write(&source_path, "pub fn value() -> u8 { 2 }\n").expect("modify source");
        record_codebase_live_map_event(
            &bundle,
            "test:file-write",
            &[source_path.display().to_string()],
        )
        .expect("record live-map event");

        let update = update_codebase_live_map(&bundle, &repo).expect("refresh map");

        assert_eq!(update.status, "out_of_sync");
        let persisted = fs::read_to_string(state_dir.join("codebase-live-map.json"))
            .expect("read persisted map");
        let state: CodebaseLiveMapState =
            serde_json::from_str(&persisted).expect("parse persisted map");
        assert!(state.needs_reread);
        assert_eq!(state.autosync, "updated_map_reread_required");
        assert_eq!(state.last_changes.modified_count, 1);
        assert!(
            update
                .diagnostics
                .iter()
                .any(|line| line.contains("codebase_event_log")),
            "{update:?}"
        );

        let _ = fs::remove_dir_all(repo);
    }

    #[test]
    fn live_map_diff_ignores_mtime_only_churn_when_hash_matches() {
        let previous = CodebaseLiveMapFile {
            len: 12,
            mtime_unix: 100,
            content_hash: "abc".to_string(),
        };
        let current = CodebaseLiveMapFile {
            len: 12,
            mtime_unix: 200,
            content_hash: "abc".to_string(),
        };
        let changed = CodebaseLiveMapFile {
            len: 12,
            mtime_unix: 200,
            content_hash: "def".to_string(),
        };

        assert!(!codebase_live_map_file_changed(&previous, &current));
        assert!(codebase_live_map_file_changed(&previous, &changed));
    }
}
