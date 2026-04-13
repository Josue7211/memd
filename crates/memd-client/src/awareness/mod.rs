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
