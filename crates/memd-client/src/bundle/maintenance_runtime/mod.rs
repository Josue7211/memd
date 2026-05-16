use super::*;
use crate::cli::command_catalog::build_command_catalog;

pub(crate) fn harness_pack_enabled_for_snapshot(
    output: &Path,
    snapshot: &ResumeSnapshot,
    agent_name: &str,
) -> bool {
    let runtime_match = read_bundle_runtime_config(output)
        .ok()
        .flatten()
        .and_then(|config| config.agent)
        .map(|agent| agent.trim().to_ascii_lowercase().starts_with(agent_name))
        .unwrap_or(false);
    let snapshot_match = snapshot
        .agent
        .as_deref()
        .map(|agent| agent.trim().to_ascii_lowercase().starts_with(agent_name))
        .unwrap_or(false);
    runtime_match || snapshot_match
}

pub(crate) fn harness_pack_enabled_for_bundle(
    output: &Path,
    agent: Option<&str>,
    agent_name: &str,
) -> bool {
    let runtime_match = read_bundle_runtime_config(output)
        .ok()
        .flatten()
        .and_then(|config| config.agent)
        .map(|value| value.trim().to_ascii_lowercase().starts_with(agent_name))
        .unwrap_or(false);
    let agent_match = agent
        .map(|value| value.trim().to_ascii_lowercase().starts_with(agent_name))
        .unwrap_or(false);
    runtime_match || agent_match
}

#[derive(Clone, Copy)]
pub(crate) struct HarnessPackRuntime {
    agent_name: &'static str,
    build: fn(&Path, &str, &str) -> crate::harness::shared::HarnessPackData,
}

pub(crate) fn harness_pack_runtimes() -> &'static [HarnessPackRuntime] {
    const RUNTIMES: &[HarnessPackRuntime] = &[
        HarnessPackRuntime {
            agent_name: "codex",
            build: crate::harness::codex::build_codex_harness_pack,
        },
        HarnessPackRuntime {
            agent_name: "agent-zero",
            build: crate::harness::agent_zero::build_agent_zero_harness_pack,
        },
        HarnessPackRuntime {
            agent_name: "openclaw",
            build: crate::harness::openclaw::build_openclaw_harness_pack,
        },
        HarnessPackRuntime {
            agent_name: "hermes",
            build: crate::harness::hermes::build_hermes_harness_pack,
        },
        HarnessPackRuntime {
            agent_name: "opencode",
            build: crate::harness::opencode::build_opencode_harness_pack,
        },
    ];
    RUNTIMES
}

pub(crate) fn harness_pack_query_from_snapshot(snapshot: &ResumeSnapshot) -> String {
    let mut parts = Vec::new();
    if let Some(record) = snapshot.working.records.first() {
        parts.push(record.record.clone());
    }
    if let Some(item) = snapshot.inbox.items.first() {
        parts.push(item.item.content.clone());
    }
    if let Some(next) = snapshot.working.rehydration_queue.first() {
        parts.push(next.summary.clone());
    }
    if let Some(change) = snapshot.change_summary.first() {
        parts.push(change.clone());
    }
    if let Some(change) = snapshot.recent_repo_changes.first() {
        parts.push(change.clone());
    }
    parts.join(" | ")
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn harness_pack_turn_key(
    project: Option<&str>,
    namespace: Option<&str>,
    agent: Option<&str>,
    mode: &str,
    query: &str,
) -> String {
    cache::build_turn_key(project, namespace, agent, mode, query)
}

pub(crate) async fn refresh_harness_pack_files(
    output: &Path,
    snapshot: &ResumeSnapshot,
    manifest: &crate::harness::shared::HarnessPackData,
    agent_name: &str,
    mode: &str,
    query: &str,
) -> anyhow::Result<Vec<PathBuf>> {
    cache::refresh_turn_cached_pack_files(
        output,
        snapshot,
        &manifest.files,
        agent_name,
        mode,
        query,
        write_bundle_memory_files(output, snapshot, None, false),
    )
    .await
}

pub(crate) async fn refresh_harness_pack_files_for_snapshot(
    output: &Path,
    snapshot: &ResumeSnapshot,
    mode: &str,
    allowed_agents: &[&str],
) -> anyhow::Result<Vec<PathBuf>> {
    let query = harness_pack_query_from_snapshot(snapshot);
    let project = snapshot.project.as_deref().unwrap_or("none");
    let namespace = snapshot.namespace.as_deref().unwrap_or("none");
    let mut refreshed = Vec::new();
    for runtime in harness_pack_runtimes() {
        if !allowed_agents.contains(&runtime.agent_name) {
            continue;
        }
        if !harness_pack_enabled_for_snapshot(output, snapshot, runtime.agent_name) {
            continue;
        }
        let manifest = (runtime.build)(output, project, namespace);
        refreshed.extend(
            refresh_harness_pack_files(
                output,
                snapshot,
                &manifest,
                runtime.agent_name,
                mode,
                &query,
            )
            .await?,
        );
    }
    Ok(refreshed)
}

pub(crate) fn read_codex_pack_local_markdown(
    output: &Path,
    file_name: &str,
) -> anyhow::Result<Option<String>> {
    let path = output.join(file_name);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(Some(raw))
}

pub(crate) fn preserve_codex_capture_locally(output: &Path, content: &str) -> anyhow::Result<()> {
    let mut note = String::new();
    note.push_str("\n## Codex Capture Fallback\n\n");
    note.push_str(&format!("- {}\n", compact_inline(content.trim(), 220)));
    append_text_to_memory_surface(&output.join("mem.md"), &note)?;
    Ok(())
}

pub(crate) fn build_bundle_migration_manifest(
    output: &Path,
    project_root: Option<&Path>,
    snapshot: &ResumeSnapshot,
    handoff: Option<&HandoffSnapshot>,
    source_registry: Option<&BootstrapSourceRegistry>,
    capability_registry: &CapabilityRegistry,
    capability_bridges: &CapabilityBridgeRegistry,
) -> anyhow::Result<BundleMigrationManifest> {
    let source_registry_json = source_registry
        .map(serde_json::to_string)
        .transpose()
        .context("serialize source registry")?;
    let source_registry_hash = source_registry_json
        .as_ref()
        .map(|json| format!("{:x}", Sha256::digest(json.as_bytes())));
    let source_registry_path = source_registry
        .as_ref()
        .map(|_| bundle_source_registry_path(output).display().to_string());

    let live_truth_summary = snapshot
        .event_spine()
        .into_iter()
        .take(4)
        .collect::<Vec<_>>();
    let mut project_brain_summary = snapshot.compact_context_records();
    project_brain_summary.extend(snapshot.compact_working_records());
    project_brain_summary.extend(snapshot.compact_inbox_items());
    if let Some(handoff) = handoff {
        project_brain_summary.extend(handoff.sources.sources.iter().map(|source| {
            format!(
                "handoff {} / {}",
                source.source_agent.as_deref().unwrap_or("none"),
                source.source_system.as_deref().unwrap_or("none")
            )
        }));
    }
    let user_policy_summary = capability_registry
        .capabilities
        .iter()
        .take(8)
        .map(|record| format!("{} / {} / {}", record.harness, record.kind, record.name))
        .collect::<Vec<_>>();
    let promoted_abstractions_summary = capability_bridges
        .actions
        .iter()
        .take(8)
        .map(|action| {
            format!(
                "{} / {} -> {}",
                action.harness, action.capability, action.status
            )
        })
        .collect::<Vec<_>>();

    Ok(BundleMigrationManifest {
        generated_at: Utc::now(),
        project_root: project_root.map(|root| root.display().to_string()),
        source_registry_hash,
        source_registry_path,
        layer_summary: vec![
            BundleMigrationLayer {
                layer: "live_truth".to_string(),
                sources: live_truth_summary.len(),
                summary: live_truth_summary,
            },
            BundleMigrationLayer {
                layer: "project_brain".to_string(),
                sources: project_brain_summary.len(),
                summary: project_brain_summary,
            },
            BundleMigrationLayer {
                layer: "user_policy".to_string(),
                sources: user_policy_summary.len(),
                summary: user_policy_summary,
            },
            BundleMigrationLayer {
                layer: "promoted_abstractions".to_string(),
                sources: promoted_abstractions_summary.len(),
                summary: promoted_abstractions_summary,
            },
        ],
        notes: vec![
            "bootstrap remains read-once for unchanged sources".to_string(),
            "delta refresh reuses the existing source registry instead of reimporting stable files"
                .to_string(),
            "explicit init remains the only mutating bridge path for shared runtime surfaces"
                .to_string(),
        ],
    })
}

pub(crate) fn infer_bundle_project_root(output: &Path) -> Option<PathBuf> {
    let parent = output.parent()?;
    if output.file_name().and_then(|value| value.to_str()) != Some(".memd") {
        return None;
    }
    if is_project_root_candidate(parent) {
        return Some(parent.to_path_buf());
    }
    None
}

pub(crate) fn bundle_resume_state_path(output: &Path) -> PathBuf {
    output.join("state").join("last-resume.json")
}

pub(crate) fn bundle_lane_surface_path(output: &Path) -> PathBuf {
    output.join("state").join("lane-surface.json")
}

pub(crate) fn skill_policy_batch_state_path(output: &Path) -> PathBuf {
    output.join("state").join("skill-policy-batch.json")
}

pub(crate) fn skill_policy_batch_markdown_path(output: &Path) -> PathBuf {
    output.join("state").join("skill-policy-batch.md")
}

pub(crate) fn skill_policy_activate_state_path(output: &Path) -> PathBuf {
    output
        .join("state")
        .join("skill-policy-activate-queue.json")
}

pub(crate) fn skill_policy_activate_markdown_path(output: &Path) -> PathBuf {
    output.join("state").join("skill-policy-activate-queue.md")
}

pub(crate) fn skill_policy_review_state_path(output: &Path) -> PathBuf {
    output.join("state").join("skill-policy-review-queue.json")
}

pub(crate) fn skill_policy_review_markdown_path(output: &Path) -> PathBuf {
    output.join("state").join("skill-policy-review-queue.md")
}

pub(crate) fn skill_policy_apply_state_path(output: &Path) -> PathBuf {
    output.join("state").join("skill-policy-apply-receipt.json")
}

pub(crate) fn skill_policy_apply_markdown_path(output: &Path) -> PathBuf {
    output.join("state").join("skill-policy-apply-receipt.md")
}

pub(crate) fn build_hive_session_retire_request_from_entry(
    entry: &ProjectAwarenessEntry,
    reason: impl Into<String>,
) -> Option<memd_schema::HiveSessionRetireRequest> {
    let session = entry
        .session
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())?;
    Some(memd_schema::HiveSessionRetireRequest {
        session: session.to_string(),
        project: entry.project.clone(),
        namespace: entry.namespace.clone(),
        repo_root: entry.repo_root.clone(),
        worktree_root: entry.worktree_root.clone(),
        branch: entry.branch.clone(),
        workspace: entry.workspace.clone(),
        agent: entry.agent.clone(),
        effective_agent: entry.effective_agent.clone(),
        hive_system: entry.hive_system.clone(),
        hive_role: entry.hive_role.clone(),
        host: entry.host.clone(),
        reason: Some(reason.into()),
    })
}

pub(crate) fn build_hive_session_retire_request_from_record(
    record: &memd_schema::HiveSessionRecord,
    reason: impl Into<String>,
) -> memd_schema::HiveSessionRetireRequest {
    memd_schema::HiveSessionRetireRequest {
        session: record.session.clone(),
        project: record.project.clone(),
        namespace: record.namespace.clone(),
        repo_root: record.repo_root.clone(),
        worktree_root: record.worktree_root.clone(),
        branch: record.branch.clone(),
        workspace: record.workspace.clone(),
        agent: record.agent.clone(),
        effective_agent: record.effective_agent.clone(),
        hive_system: record.hive_system.clone(),
        hive_role: record.hive_role.clone(),
        host: record.host.clone(),
        reason: Some(reason.into()),
    }
}

pub(crate) fn is_superseded_hive_session_record(
    record: &memd_schema::HiveSessionRecord,
    current: &BundleHeartbeatState,
) -> bool {
    heartbeat_presence_label(record.last_seen) == "stale"
        && current.status == "live"
        && current
            .session
            .as_deref()
            .is_some_and(|session| session != record.session)
        && record.project == current.project
        && record.namespace == current.namespace
        && record.workspace == current.workspace
        && record.agent == current.agent
        && record.base_url == current.base_url
}

pub(crate) async fn retire_hive_session_entry(
    client: &MemdClient,
    entry: &ProjectAwarenessEntry,
    reason: impl Into<String>,
) -> anyhow::Result<usize> {
    let Some(request) = build_hive_session_retire_request_from_entry(entry, reason) else {
        return Ok(0);
    };
    Ok(client.retire_hive_session(&request).await?.retired)
}

pub(crate) async fn retire_superseded_hive_sessions(
    client: &MemdClient,
    state: &BundleHeartbeatState,
) -> anyhow::Result<usize> {
    let Some(current_session) = state
        .session
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(0);
    };
    let sessions_request = memd_schema::HiveSessionsRequest {
        session: None,
        project: state.project.clone(),
        namespace: state.namespace.clone(),
        repo_root: state.repo_root.clone(),
        worktree_root: state.worktree_root.clone(),
        branch: state.branch.clone(),
        workspace: state.workspace.clone(),
        hive_system: None,
        hive_role: None,
        host: None,
        hive_group: None,
        active_only: Some(false),
        limit: Some(512),
    };
    let sessions = timeout_ok(client.hive_sessions(&sessions_request))
        .await
        .map(|response| response.sessions)
        .unwrap_or_default();
    let mut retired = 0usize;
    for session in sessions {
        if session.session == current_session {
            continue;
        }
        if !is_superseded_hive_session_record(&session, state) {
            continue;
        }
        let retire_request = build_hive_session_retire_request_from_record(
            &session,
            format!("superseded by live session {current_session}"),
        );
        retired += timeout_ok(client.retire_hive_session(&retire_request))
            .await
            .map(|response| response.retired)
            .unwrap_or(0);
    }
    Ok(retired)
}

pub(crate) async fn enrich_hive_heartbeat_with_runtime_intent(
    state: &mut BundleHeartbeatState,
) -> anyhow::Result<()> {
    let Some(base_url) = state
        .base_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };
    let Some(session) = state
        .session
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };

    let client = MemdClient::new(base_url)?;
    let tasks = timeout_ok(client.hive_tasks(&HiveTasksRequest {
        session: Some(session.to_string()),
        project: state.project.clone(),
        namespace: state.namespace.clone(),
        workspace: state.workspace.clone(),
        active_only: Some(true),
        limit: Some(64),
    }))
    .await
    .map(|response| response.tasks)
    .unwrap_or_default();

    let current_task = tasks
        .iter()
        .find(|task| task.status != "done" && task.status != "closed")
        .or_else(|| tasks.first());
    if let Some(task) = current_task {
        state.task_id = Some(task.task_id.clone());
        if state
            .topic_claim
            .as_deref()
            .is_none_or(hive_topic_claim_needs_runtime_upgrade)
        {
            state.topic_claim = Some(task.title.clone());
        }
        if state.display_name.is_none()
            && state
                .worker_name
                .as_deref()
                .is_some_and(hive_worker_name_is_generic)
        {
            state.display_name =
                derive_hive_display_name(state.agent.as_deref(), state.session.as_deref());
        }
        for scope in &task.claim_scopes {
            push_unique_touch_point(&mut state.scope_claims, scope);
        }
    }
    Ok(())
}

pub(crate) fn build_hive_heartbeat(
    output: &Path,
    snapshot: Option<&ResumeSnapshot>,
) -> anyhow::Result<BundleHeartbeatState> {
    let runtime = read_bundle_runtime_config(output)?.unwrap_or(BundleRuntimeConfig {
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
    let session = runtime.session.clone();
    let agent = runtime.agent.clone();
    let resume_state = read_bundle_resume_state(output).ok().flatten();
    let claims_state = read_bundle_claims(output).ok();
    let project_root = infer_bundle_project_root(output);
    let worktree_root = project_root
        .as_deref()
        .and_then(detect_git_worktree_root)
        .as_deref()
        .map(display_path_nonempty);
    let repo_root = project_root
        .as_deref()
        .and_then(detect_git_repo_root)
        .as_deref()
        .map(display_path_nonempty);
    let branch = project_root
        .as_deref()
        .and_then(|root| git_stdout(root, &["branch", "--show-current"]));
    let effective_agent = agent
        .as_deref()
        .map(|value| compose_agent_identity(value, session.as_deref()));
    let focus = snapshot
        .and_then(|value| {
            value
                .working
                .records
                .first()
                .map(|record| record.record.clone())
        })
        .or_else(|| resume_state.as_ref().and_then(|value| value.focus.clone()));
    let pressure = snapshot
        .and_then(|value| {
            value
                .inbox
                .items
                .first()
                .map(|item| item.item.content.clone())
        })
        .or_else(|| {
            resume_state
                .as_ref()
                .and_then(|value| value.pressure.clone())
        });
    let next_recovery = snapshot
        .and_then(|value| {
            value
                .working
                .rehydration_queue
                .first()
                .map(|item| format!("{}: {}", item.label, item.summary))
        })
        .or_else(|| {
            resume_state
                .as_ref()
                .and_then(|value| value.next_recovery.clone())
        });
    let topic_claim = derive_hive_topic_claim(
        focus.as_deref(),
        next_recovery.as_deref(),
        pressure.as_deref(),
    );
    let working = topic_claim
        .clone()
        .or_else(|| focus.clone())
        .or_else(|| next_recovery.clone());
    let scope_claims = derive_hive_scope_claims(
        claims_state.as_ref(),
        focus.as_deref(),
        pressure.as_deref(),
        next_recovery.as_deref(),
    );
    let touches = scope_claims.clone();
    let task_id = derive_hive_task_id(&scope_claims, topic_claim.as_deref());
    let worker_name = infer_worker_agent_from_env().or_else(|| {
        agent.as_deref().map(|value| {
            default_bundle_worker_name_for_project(
                runtime.project.as_deref(),
                value,
                session.as_deref(),
            )
        })
    });
    let display_name = if worker_name
        .as_deref()
        .is_some_and(hive_worker_name_is_generic)
    {
        derive_hive_display_name(
            worker_name.as_deref().or(agent.as_deref()),
            session.as_deref(),
        )
    } else {
        None
    };
    let lane_id = derive_hive_lane_id(branch.as_deref(), worktree_root.as_deref());
    Ok(BundleHeartbeatState {
        session: session.clone(),
        agent: agent.clone(),
        effective_agent,
        tab_id: runtime.tab_id,
        hive_system: runtime.hive_system,
        hive_role: runtime.hive_role.clone(),
        worker_name,
        display_name,
        role: runtime.hive_role.clone(),
        capabilities: runtime.capabilities,
        hive_groups: effective_hive_groups(
            runtime.hive_groups,
            snapshot
                .and_then(|value| value.project.as_deref())
                .or(runtime.project.as_deref()),
        ),
        lane_id,
        hive_group_goal: runtime.hive_group_goal,
        authority: runtime.authority,
        authority_mode: Some(runtime.authority_state.mode),
        authority_degraded: runtime.authority_state.degraded,
        heartbeat_model: runtime.heartbeat_model,
        project: snapshot
            .and_then(|value| value.project.clone())
            .or(runtime.project),
        namespace: snapshot
            .and_then(|value| value.namespace.clone())
            .or(runtime.namespace),
        workspace: snapshot
            .and_then(|value| value.workspace.clone())
            .or(runtime.workspace),
        repo_root,
        worktree_root,
        branch,
        base_branch: None,
        visibility: snapshot
            .and_then(|value| value.visibility.clone())
            .or(runtime.visibility),
        base_url: runtime.base_url,
        base_url_healthy: None,
        host: detect_host_name(),
        pid: Some(std::process::id()),
        topic_claim,
        scope_claims,
        task_id,
        focus: focus.clone(),
        pressure: pressure.clone(),
        next_recovery: next_recovery.clone(),
        next_action: derive_hive_next_action(
            focus.as_deref(),
            next_recovery.as_deref(),
            pressure.as_deref(),
        ),
        working,
        touches,
        blocked_by: Vec::new(),
        cowork_with: Vec::new(),
        handoff_target: None,
        offered_to: Vec::new(),
        needs_help: false,
        needs_review: false,
        handoff_state: None,
        confidence: None,
        risk: None,
        status: "live".to_string(),
        last_seen: Utc::now(),
    })
}

/// Refresh live truth items by detecting changed files in `.memd/lanes/`.
/// Computes content hashes, compares against cached state, and stores
/// changed files via the store API with `kind = LiveTruth`.
pub(crate) async fn refresh_live_truth(output: &Path, base_url: &str) -> anyhow::Result<usize> {
    let lanes_dir = output.join("lanes");
    if !lanes_dir.is_dir() {
        return Ok(0);
    }
    let hash_cache_path = output.join("live_truth_hashes.json");
    let cached: std::collections::HashMap<String, String> = if hash_cache_path.is_file() {
        serde_json::from_str(&fs::read_to_string(&hash_cache_path).unwrap_or_default())
            .unwrap_or_default()
    } else {
        std::collections::HashMap::new()
    };

    let runtime = read_bundle_runtime_config(output)?;
    let project = runtime
        .as_ref()
        .and_then(|c| c.project.clone())
        .unwrap_or_else(|| "unknown".to_string());
    let namespace = runtime.as_ref().and_then(|c| c.namespace.clone());
    let agent = runtime.as_ref().and_then(|c| c.agent.clone());

    let mut updated: std::collections::HashMap<String, String> = cached.clone();
    let mut refreshed = 0usize;
    let client = MemdClient::new(base_url)?;

    for entry in walkdir_lanes(&lanes_dir)? {
        let content = match fs::read_to_string(&entry) {
            Ok(c) if !c.trim().is_empty() => c,
            _ => continue,
        };
        let rel = entry
            .strip_prefix(output)
            .unwrap_or(&entry)
            .to_string_lossy()
            .to_string();
        let hash = format!("{:x}", content_hash(content.as_bytes()));
        if cached.get(&rel).map(|h| h.as_str()) == Some(&hash) {
            updated.insert(rel, hash);
            continue;
        }
        let summary = if content.len() > 300 {
            let truncated: String = content.chars().take(300).collect();
            format!("{truncated}...")
        } else {
            content.clone()
        };
        let req = memd_schema::StoreMemoryRequest {
            content: summary,
            kind: memd_schema::MemoryKind::LiveTruth,
            scope: memd_schema::MemoryScope::Project,
            project: Some(project.clone()),
            namespace: namespace.clone(),
            workspace: None,
            visibility: None,
            source_agent: agent.clone(),
            source_system: Some("heartbeat-refresh".to_string()),
            source_path: Some(rel.clone()),
            source_quality: Some(memd_schema::SourceQuality::Derived),
            confidence: Some(0.8),
            ttl_seconds: None,
            last_verified_at: Some(Utc::now()),
            supersedes: Vec::new(),
            tags: vec!["live-truth".to_string(), "lane-refresh".to_string()],
            belief_branch: None,
            status: None,
            lane: None,
        };
        match client.store(&req).await {
            Ok(_) => {
                refreshed += 1;
                updated.insert(rel, hash);
            }
            Err(e) => {
                eprintln!(
                    "warn: refresh_live_truth store failed for {}: {e:#}",
                    entry.display()
                );
            }
        }
    }
    if let Ok(json) = serde_json::to_string_pretty(&updated) {
        let _ = fs::write(&hash_cache_path, json);
    }
    Ok(refreshed)
}

fn walkdir_lanes(lanes_dir: &Path) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    if !lanes_dir.is_dir() {
        return Ok(files);
    }
    for lane_entry in fs::read_dir(lanes_dir)? {
        let lane_entry = lane_entry?;
        let lane_path = lane_entry.path();
        if !lane_path.is_dir() {
            continue;
        }
        for file_entry in fs::read_dir(&lane_path)? {
            let file_entry = file_entry?;
            let file_path = file_entry.path();
            if file_path.is_file()
                && file_path
                    .extension()
                    .map(|e| e == "md" || e == "txt")
                    .unwrap_or(false)
            {
                files.push(file_path);
            }
        }
    }
    Ok(files)
}

fn content_hash(data: &[u8]) -> u128 {
    // Simple FNV-1a-based content hash (not cryptographic, just change detection)
    let mut hash: u128 = 0xcbf29ce484222325u128;
    for &byte in data {
        hash ^= byte as u128;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub(crate) async fn write_bundle_heartbeat(
    output: &Path,
    snapshot: Option<&ResumeSnapshot>,
    probe_base_url: bool,
) -> anyhow::Result<()> {
    let _ = repair_bundle_worker_name_env(output);
    let path = bundle_heartbeat_state_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut state = build_hive_heartbeat(output, snapshot)?;
    enrich_hive_heartbeat_with_runtime_intent(&mut state).await?;
    if probe_base_url && let Some(url) = state.base_url.as_deref() {
        state.base_url_healthy = Some(MemdClient::new(url)?.healthz().await.is_ok());
    }
    // Refresh live truth items from lane files on each heartbeat cycle
    if let Some(url) = state.base_url.as_deref()
        && state.base_url_healthy == Some(true)
    {
        if let Err(e) = refresh_live_truth(output, url).await {
            eprintln!("warn: refresh_live_truth: {e:#}");
        }
    }
    fs::write(&path, serde_json::to_string_pretty(&state)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    publish_bundle_heartbeat(&state).await?;
    Ok(())
}

pub(crate) async fn refresh_bundle_heartbeat(
    output: &Path,
    snapshot: Option<&ResumeSnapshot>,
    probe_base_url: bool,
) -> anyhow::Result<BundleHeartbeatState> {
    write_bundle_heartbeat(output, snapshot, probe_base_url).await?;
    read_bundle_heartbeat(output)?.context("reload bundle heartbeat after write")
}

pub(crate) async fn reconcile_bundle_heartbeat(
    output: &Path,
    snapshot: Option<&ResumeSnapshot>,
    probe_base_url: bool,
) -> anyhow::Result<(BundleHeartbeatState, usize)> {
    let _ = repair_bundle_worker_name_env(output);
    let path = bundle_heartbeat_state_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut state = build_hive_heartbeat(output, snapshot)?;
    enrich_hive_heartbeat_with_runtime_intent(&mut state).await?;
    if probe_base_url && let Some(url) = state.base_url.as_deref() {
        state.base_url_healthy = Some(MemdClient::new(url)?.healthz().await.is_ok());
    }
    fs::write(&path, serde_json::to_string_pretty(&state)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    let retired = publish_bundle_heartbeat(&state).await?;
    let state =
        read_bundle_heartbeat(output)?.context("reload bundle heartbeat after reconcile")?;
    Ok((state, retired))
}

pub(crate) async fn publish_bundle_heartbeat(
    state: &BundleHeartbeatState,
) -> anyhow::Result<usize> {
    if state
        .authority_mode
        .as_deref()
        .is_some_and(|mode| mode == "localhost_read_only")
    {
        anyhow::bail!(
            "localhost read-only fallback active; heartbeat publication requires trusted shared authority"
        );
    }
    let Some(base_url) = state
        .base_url
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    else {
        return Ok(0);
    };
    let Some(session) = state
        .session
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    else {
        return Ok(0);
    };

    let client = MemdClient::new(base_url)?;
    let request = memd_schema::HiveSessionUpsertRequest {
        session: session.to_string(),
        tab_id: state.tab_id.clone(),
        agent: state.agent.clone(),
        effective_agent: state.effective_agent.clone(),
        hive_system: state.hive_system.clone(),
        hive_role: state.hive_role.clone(),
        worker_name: state.worker_name.clone(),
        display_name: state.display_name.clone(),
        role: state.role.clone(),
        capabilities: state.capabilities.clone(),
        hive_groups: state.hive_groups.clone(),
        lane_id: state.lane_id.clone(),
        hive_group_goal: state.hive_group_goal.clone(),
        authority: state.authority.clone(),
        heartbeat_model: state.heartbeat_model.clone(),
        project: state.project.clone(),
        namespace: state.namespace.clone(),
        workspace: state.workspace.clone(),
        repo_root: state.repo_root.clone(),
        worktree_root: state.worktree_root.clone(),
        branch: state.branch.clone(),
        base_branch: state.base_branch.clone(),
        visibility: state.visibility.clone(),
        base_url: state.base_url.clone(),
        base_url_healthy: state.base_url_healthy,
        host: state.host.clone(),
        pid: state.pid,
        topic_claim: state.topic_claim.clone(),
        scope_claims: state.scope_claims.clone(),
        task_id: state.task_id.clone(),
        focus: state.focus.clone(),
        pressure: state.pressure.clone(),
        next_recovery: state.next_recovery.clone(),
        next_action: state.next_action.clone(),
        working: state.working.clone(),
        touches: state.touches.clone(),
        blocked_by: state.blocked_by.clone(),
        cowork_with: state.cowork_with.clone(),
        handoff_target: state.handoff_target.clone(),
        offered_to: state.offered_to.clone(),
        needs_help: state.needs_help,
        needs_review: state.needs_review,
        handoff_state: state.handoff_state.clone(),
        confidence: state.confidence.clone(),
        risk: state.risk.clone(),
        status: Some(state.status.clone()),
        last_wake_at: None,
    };
    let _ = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        client.upsert_hive_session(&request),
    )
    .await;
    let retired = retire_superseded_hive_sessions(&client, state)
        .await
        .unwrap_or(0);
    Ok(retired)
}

pub(crate) fn render_bundle_heartbeat_summary(state: &BundleHeartbeatState) -> String {
    format!(
        "heartbeat project={} agent={} hive={} role={} groups={} goal=\"{}\" authority={} session={} tab={} presence={} model={} base_url={} topic=\"{}\" scopes={} task={} focus=\"{}\" pressure=\"{}\"",
        state.project.as_deref().unwrap_or("none"),
        state
            .effective_agent
            .as_deref()
            .or(state.agent.as_deref())
            .unwrap_or("none"),
        state.hive_system.as_deref().unwrap_or("none"),
        state.hive_role.as_deref().unwrap_or("none"),
        if state.hive_groups.is_empty() {
            "none".to_string()
        } else {
            state.hive_groups.join(",")
        },
        state.hive_group_goal.as_deref().unwrap_or("none"),
        state.authority.as_deref().unwrap_or("none"),
        state.session.as_deref().unwrap_or("none"),
        state.tab_id.as_deref().unwrap_or("none"),
        heartbeat_presence_label(state.last_seen),
        state.heartbeat_model.as_deref().unwrap_or("none"),
        state.base_url.as_deref().unwrap_or("none"),
        state.topic_claim.as_deref().unwrap_or("none"),
        if state.scope_claims.is_empty() {
            "none".to_string()
        } else {
            compact_inline(&state.scope_claims.join(","), 72)
        },
        state.task_id.as_deref().unwrap_or("none"),
        state
            .focus
            .as_deref()
            .map(|value| compact_inline(value, 72))
            .unwrap_or_else(|| "none".to_string()),
        state
            .pressure
            .as_deref()
            .map(|value| compact_inline(value, 72))
            .unwrap_or_else(|| "none".to_string())
    )
}

pub(crate) fn run_capabilities_command(
    args: &CapabilitiesArgs,
) -> anyhow::Result<CapabilitiesResponse> {
    let output = capabilities_output(args);
    let project_root = infer_bundle_project_root(output);
    let mut registry = build_bundle_capability_registry(project_root.as_deref());
    let bridges = detect_capability_bridges();
    if matches!(&args.command, Some(CapabilitiesSubcommand::Sync(_))) {
        write_bundle_capability_registry(output, &registry)?;
        write_bundle_capability_bridges(output, &bridges)?;
    } else if let Some(persisted) = read_persisted_capability_registry(output)? {
        registry = merge_capability_registries(registry, persisted);
    }
    let record_limit = if matches!(&args.command, Some(CapabilitiesSubcommand::Sync(_))) {
        usize::MAX
    } else {
        args.limit
    };
    build_capabilities_response_from_registry(
        args,
        output,
        &registry,
        &bridges,
        capabilities_command_label(args),
        matches!(&args.command, Some(CapabilitiesSubcommand::Sync(_))),
        record_limit,
    )
}

pub(crate) fn build_capabilities_response_from_registry(
    args: &CapabilitiesArgs,
    output: &Path,
    registry: &CapabilityRegistry,
    bridges: &CapabilityBridgeRegistry,
    command_label: &'static str,
    synced: bool,
    record_limit: usize,
) -> anyhow::Result<CapabilitiesResponse> {
    let query = args.query.as_deref().map(str::to_ascii_lowercase);
    let mut filtered = registry
        .capabilities
        .iter()
        .filter(|capability| {
            args.harness
                .as_deref()
                .is_none_or(|value| capability.harness == value)
        })
        .filter(|capability| {
            args.kind
                .as_deref()
                .is_none_or(|value| capability.kind == value)
        })
        .filter(|capability| {
            args.portability
                .as_deref()
                .is_none_or(|value| capability.portability_class == value)
        })
        .filter(|capability| {
            query.as_ref().is_none_or(|needle| {
                capability.name.to_ascii_lowercase().contains(needle)
                    || capability.harness.to_ascii_lowercase().contains(needle)
                    || capability.kind.to_ascii_lowercase().contains(needle)
                    || capability
                        .portability_class
                        .to_ascii_lowercase()
                        .contains(needle)
                    || capability
                        .bridge_hint
                        .as_deref()
                        .unwrap_or_default()
                        .to_ascii_lowercase()
                        .contains(needle)
            })
        })
        .cloned()
        .collect::<Vec<_>>();
    filtered.sort_by(|left, right| {
        left.harness
            .cmp(&right.harness)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.name.cmp(&right.name))
    });
    let bridge_harnesses = bridges
        .actions
        .iter()
        .map(|action| action.harness.clone())
        .collect::<BTreeSet<_>>();
    let mut harnesses = BTreeMap::<String, CapabilityHarnessSummary>::new();
    for capability in &filtered {
        let entry = harnesses
            .entry(capability.harness.clone())
            .or_insert_with(|| CapabilityHarnessSummary {
                harness: capability.harness.clone(),
                capabilities: 0,
                installed: 0,
                bridge_actions: 0,
            });
        entry.capabilities += 1;
        if capability.status == "installed" || capability.status == "discovered" {
            entry.installed += 1;
        }
    }
    for action in &bridges.actions {
        if args
            .harness
            .as_deref()
            .is_some_and(|value| action.harness != value)
        {
            continue;
        }
        let entry =
            harnesses
                .entry(action.harness.clone())
                .or_insert_with(|| CapabilityHarnessSummary {
                    harness: action.harness.clone(),
                    capabilities: 0,
                    installed: 0,
                    bridge_actions: 0,
                });
        entry.bridge_actions += 1;
    }

    Ok(CapabilitiesResponse {
        bundle_root: output.display().to_string(),
        generated_at: registry.generated_at,
        discovered: filtered.len(),
        universal: filtered
            .iter()
            .filter(|record| is_universal_class(&record.portability_class))
            .count(),
        bridgeable: filtered
            .iter()
            .filter(|record| is_bridgeable_class(&record.portability_class))
            .count(),
        harness_native: filtered
            .iter()
            .filter(|record| is_harness_native_class(&record.portability_class))
            .count(),
        bridge_actions: bridges.actions.len(),
        wired_harnesses: bridge_harnesses.len(),
        filters: serde_json::json!({
            "command": command_label,
            "harness": args.harness,
            "kind": args.kind,
            "portability": args.portability,
            "query": args.query,
            "limit": args.limit,
            "materialize_plan": capabilities_materialize_plan(args),
            "materialize": capabilities_materialize(args),
            "synced": synced,
        }),
        materialization: (capabilities_materialize_plan(args) || capabilities_materialize(args))
            .then(|| {
                build_capability_materialization_report(
                    output,
                    registry,
                    capabilities_materialize(args),
                )
            })
            .transpose()?,
        harnesses: harnesses.into_values().collect(),
        records: filtered.into_iter().take(record_limit).collect(),
    })
}

fn read_persisted_capability_registry(output: &Path) -> anyhow::Result<Option<CapabilityRegistry>> {
    let path = bundle_capability_registry_path(output);
    if !path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let registry = serde_json::from_str::<CapabilityRegistry>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(registry))
}

fn merge_capability_registries(
    local: CapabilityRegistry,
    persisted: CapabilityRegistry,
) -> CapabilityRegistry {
    let mut by_identity = BTreeMap::<String, CapabilityRecord>::new();
    for capability in persisted.capabilities {
        by_identity.insert(capability_identity_key(&capability), capability);
    }
    for capability in local.capabilities {
        let key = capability_identity_key(&capability);
        let keep_persisted = by_identity
            .get(&key)
            .is_some_and(|persisted| persisted_capability_is_stronger(persisted, &capability));
        if !keep_persisted {
            by_identity.insert(key, capability);
        }
    }
    CapabilityRegistry {
        generated_at: Utc::now(),
        project_root: local.project_root.or(persisted.project_root),
        capabilities: by_identity.into_values().collect(),
    }
}

fn capability_identity_key(record: &CapabilityRecord) -> String {
    format!("{}\0{}\0{}", record.harness, record.kind, record.name)
}

fn persisted_capability_is_stronger(
    persisted: &CapabilityRecord,
    local: &CapabilityRecord,
) -> bool {
    local.status == "available"
        && persisted.status != "available"
        && capability_payload_text(persisted).is_some()
        && capability_payload_text(local).is_none()
}

#[cfg(test)]
mod capability_merge_tests {
    use super::*;

    fn capability(harness: &str, kind: &str, name: &str, status: &str) -> CapabilityRecord {
        CapabilityRecord {
            harness: harness.to_string(),
            kind: kind.to_string(),
            name: name.to_string(),
            status: status.to_string(),
            portability_class: "harness-native".to_string(),
            source_path: format!("/{harness}/{kind}/{name}"),
            bridge_hint: None,
            hash: None,
            notes: Vec::new(),
        }
    }

    #[test]
    fn capability_merge_keeps_server_shadow_and_prefers_local_state() {
        let persisted = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: Some("/server-shadow".to_string()),
            capabilities: vec![
                capability("codex", "skill", "remote-only", "available-server"),
                capability("codex", "skill", "shared", "available-server"),
            ],
        };
        let local = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: Some("/local".to_string()),
            capabilities: vec![capability("codex", "skill", "shared", "installed")],
        };

        let merged = merge_capability_registries(local, persisted);

        assert_eq!(merged.capabilities.len(), 2);
        assert!(
            merged.capabilities.iter().any(|record| {
                record.name == "remote-only" && record.status == "available-server"
            })
        );
        assert!(
            merged
                .capabilities
                .iter()
                .any(|record| { record.name == "shared" && record.status == "installed" })
        );
    }

    #[test]
    fn capability_merge_keeps_server_payload_over_unwired_local_stub() {
        let mut persisted = capability("hermes", "harness-pack", "Hermes", "wired");
        persisted.notes = vec!["memd:payload-text:#!/bin/sh\n".to_string()];
        let local = capability("hermes", "harness-pack", "Hermes", "available");

        let merged = merge_capability_registries(
            CapabilityRegistry {
                generated_at: Utc::now(),
                project_root: Some("/fresh".to_string()),
                capabilities: vec![local],
            },
            CapabilityRegistry {
                generated_at: Utc::now(),
                project_root: Some("/server".to_string()),
                capabilities: vec![persisted],
            },
        );

        let record = merged
            .capabilities
            .iter()
            .find(|record| record.harness == "hermes")
            .expect("merged hermes capability");
        assert_eq!(record.status, "wired");
        assert!(capability_payload_text(record).is_some());
    }
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
    match std::process::Command::new(name).args(args).output() {
        Ok(output) if output.status.success() => Some("authenticated".to_string()),
        Ok(_) => Some("unauthenticated".to_string()),
        Err(_) => Some("unknown".to_string()),
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

#[cfg(test)]
mod capability_materialization_tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static HOST_CLI_INSTALL_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn lock_host_cli_install_env() -> std::sync::MutexGuard<'static, ()> {
        HOST_CLI_INSTALL_ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("host CLI install env lock poisoned")
    }

    fn capability(
        harness: &str,
        kind: &str,
        name: &str,
        portability_class: &str,
        source_path: &str,
    ) -> CapabilityRecord {
        CapabilityRecord {
            harness: harness.to_string(),
            kind: kind.to_string(),
            name: name.to_string(),
            status: "available-server".to_string(),
            portability_class: portability_class.to_string(),
            source_path: source_path.to_string(),
            bridge_hint: None,
            hash: None,
            notes: Vec::new(),
        }
    }

    #[test]
    fn materialization_plan_marks_harness_native_assets_missing() {
        let bundle =
            std::env::temp_dir().join(format!("memd-materialize-plan-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: None,
            capabilities: vec![
                capability(
                    "codex",
                    "plugin-skill",
                    "browser-use:browser",
                    "harness-native",
                    "/missing/.codex/plugins/cache/browser/SKILL.md",
                ),
                capability(
                    "hermes",
                    "harness-pack",
                    "Hermes",
                    "universal",
                    ".memd/agents/hermes.sh",
                ),
                capability(
                    "local",
                    "cli",
                    "memd-missing-gh",
                    "host-local",
                    "/missing/bin/memd-missing-gh",
                ),
            ],
        };
        write_bundle_capability_registry(&bundle, &registry).expect("write registry");
        let local_plugin = bundle.join("local-plugin").join("SKILL.md");
        fs::create_dir_all(local_plugin.parent().expect("local plugin parent"))
            .expect("create local plugin parent");
        fs::write(&local_plugin, "present").expect("write local plugin");

        let report = run_capabilities_command(&CapabilitiesArgs {
            command: None,
            output: bundle.clone(),
            harness: None,
            kind: None,
            portability: None,
            query: None,
            limit: 12,
            summary: false,
            json: false,
            materialize_plan: true,
            materialize: false,
        })
        .expect("capability report")
        .materialization
        .expect("materialization report");

        assert_eq!(report.status, "partial");
        assert!(report.actions.iter().any(|action| {
            action.harness == "codex"
                && action.status == "missing"
                && action.action == "install-codex-plugin"
        }));
        assert!(report.actions.iter().any(|action| {
            action.harness == "local"
                && action.status == "missing"
                && action.action == "install-host-cli"
        }));
        assert!(report.actions.iter().any(|action| {
            action.harness == "hermes"
                && action.status == "installable"
                && action.action == "restore-from-bundle"
        }));
        let local_action = materialization_action_for_record(
            &bundle,
            &capability(
                "codex",
                "plugin-skill",
                "local",
                "harness-native",
                &local_plugin.display().to_string(),
            ),
        );
        assert_eq!(local_action.status, "missing");
        assert_eq!(local_action.action, "install-codex-plugin");

        fs::remove_dir_all(bundle).ok();
    }

    #[test]
    fn host_cli_install_plan_materializes_as_installer_ready() {
        let bundle = std::env::temp_dir().join(format!(
            "memd-host-cli-install-plan-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        let mut record = capability(
            "local",
            "cli",
            "memd-test-gh",
            "host-local",
            "/usr/local/bin/memd-test-gh",
        );
        record.notes = vec![
            "PATH inventory; executable availability is host-local".to_string(),
            "memd:host-cli-install-plan:#!/bin/sh\necho install gh\nexit 2\n".to_string(),
        ];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: None,
            capabilities: vec![record],
        };
        write_bundle_capability_registry(&bundle, &registry).expect("write registry");

        let report = run_capabilities_command(&CapabilitiesArgs {
            command: None,
            output: bundle.clone(),
            harness: None,
            kind: None,
            portability: None,
            query: None,
            limit: 12,
            summary: false,
            json: false,
            materialize_plan: false,
            materialize: true,
        })
        .expect("capability report")
        .materialization
        .expect("materialization report");

        assert_eq!(report.status, "partial-applied");
        assert!(report.applied >= 1);
        assert_eq!(report.missing, 0);
        assert!(report.host_local >= 1);
        assert!(!report.fresh_machine_ready);
        let action = report
            .actions
            .iter()
            .find(|action| action.name == "memd-test-gh")
            .expect("memd-test-gh action");
        assert_eq!(action.status, "installer-ready");
        assert_eq!(action.action, "write-host-cli-install-plan");
        assert!(action.reason.contains("MEMD_HOST_CLI_INSTALL_APPROVED=1"));
        let plan_path = bundle
            .join("install")
            .join("host-cli")
            .join("memd-test-gh.sh");
        assert_eq!(
            fs::read_to_string(&plan_path).expect("read install plan"),
            "#!/bin/sh\necho install gh\nexit 2\n"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&plan_path)
                .expect("install plan metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o755);
        }

        let second_report = run_capabilities_command(&CapabilitiesArgs {
            command: None,
            output: bundle.clone(),
            harness: None,
            kind: None,
            portability: None,
            query: None,
            limit: 12,
            summary: false,
            json: false,
            materialize_plan: false,
            materialize: true,
        })
        .expect("second capability report")
        .materialization
        .expect("second materialization report");
        let second_action = second_report
            .actions
            .iter()
            .find(|action| action.name == "memd-test-gh")
            .expect("second memd-test-gh action");
        assert_eq!(second_action.status, "installer-ready");
        assert!(second_action.reason.contains("already materialized"));
        assert_eq!(second_report.applied, 0);
        assert_eq!(second_report.missing, 0);

        fs::remove_dir_all(bundle).ok();
    }

    #[test]
    fn host_cli_auth_gap_reports_next_auth_check() {
        let _guard = lock_host_cli_install_env();
        let old_path = std::env::var_os("PATH");
        unsafe {
            std::env::remove_var("PATH");
        }
        let bundle =
            std::env::temp_dir().join(format!("memd-host-cli-auth-gap-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        let mut record = capability("local", "cli", "gh", "host-local", "host-cli:gh");
        record.notes = vec![
            "PATH inventory; executable availability is host-local".to_string(),
            "memd:host-cli-install-plan:#!/bin/sh\necho install gh\n".to_string(),
        ];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: None,
            capabilities: vec![record],
        };

        let report = build_capabilities_response_from_registry(
            &CapabilitiesArgs {
                command: None,
                output: bundle.clone(),
                harness: None,
                kind: None,
                portability: None,
                query: None,
                limit: 12,
                summary: false,
                json: false,
                materialize_plan: true,
                materialize: false,
            },
            &bundle,
            &registry,
            &CapabilityBridgeRegistry {
                generated_at: Utc::now(),
                actions: Vec::new(),
            },
            "status",
            false,
            12,
        )
        .expect("capability report")
        .materialization
        .expect("materialization report");

        assert_eq!(report.auth_gaps, 1);
        assert_eq!(report.auth_unknown, 1);
        assert_eq!(report.auth_authenticated, 0);
        assert_eq!(report.auth_unauthenticated, 0);
        let action = report.actions.first().expect("host CLI action");
        assert_eq!(action.auth_status.as_deref(), Some("unknown"));
        assert_eq!(action.auth_check.as_deref(), Some("gh auth status"));
        assert!(action.reason.contains("auth_status=unknown"));
        assert!(action.reason.contains("auth_check=gh auth status"));

        unsafe {
            match old_path {
                Some(value) => std::env::set_var("PATH", value),
                None => std::env::remove_var("PATH"),
            }
        }
        fs::remove_dir_all(bundle).ok();
    }

    #[test]
    fn host_cli_auth_probe_marks_authenticated_without_storing_output() {
        let _guard = lock_host_cli_install_env();
        let old_path = std::env::var_os("PATH");
        let root =
            std::env::temp_dir().join(format!("memd-host-cli-auth-probe-{}", uuid::Uuid::new_v4()));
        let bundle = root.join(".memd");
        let bin = root.join("bin");
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        fs::create_dir_all(&bin).expect("create fake bin");
        let gh = bin.join("gh");
        fs::write(&gh, "#!/bin/sh\nexit 0\n").expect("write fake gh");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&gh).expect("fake gh metadata").permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&gh, permissions).expect("chmod fake gh");
        }
        unsafe {
            std::env::set_var("PATH", &bin);
        }

        let mut record = capability(
            "local",
            "cli",
            "gh",
            "host-local",
            &gh.display().to_string(),
        );
        record.notes = vec![
            "PATH inventory; executable availability is host-local".to_string(),
            "memd:host-cli-install-plan:#!/bin/sh\necho install gh\n".to_string(),
        ];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: None,
            capabilities: vec![record],
        };

        let report = build_capabilities_response_from_registry(
            &CapabilitiesArgs {
                command: None,
                output: bundle.clone(),
                harness: None,
                kind: None,
                portability: None,
                query: None,
                limit: 12,
                summary: false,
                json: false,
                materialize_plan: true,
                materialize: false,
            },
            &bundle,
            &registry,
            &CapabilityBridgeRegistry {
                generated_at: Utc::now(),
                actions: Vec::new(),
            },
            "status",
            false,
            12,
        )
        .expect("capability report")
        .materialization
        .expect("materialization report");

        assert_eq!(report.auth_gaps, 0);
        assert_eq!(report.auth_unknown, 0);
        assert_eq!(report.auth_authenticated, 1);
        assert_eq!(report.auth_unauthenticated, 0);
        let action = report.actions.first().expect("host CLI action");
        assert_eq!(action.auth_status.as_deref(), Some("authenticated"));
        assert_eq!(action.auth_check.as_deref(), Some("gh auth status"));
        assert!(!action.reason.contains("stdout="));
        assert!(!action.reason.contains("stderr="));

        unsafe {
            match old_path {
                Some(value) => std::env::set_var("PATH", value),
                None => std::env::remove_var("PATH"),
            }
        }
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn host_cli_auth_probe_marks_unauthenticated_as_gap() {
        let _guard = lock_host_cli_install_env();
        let old_path = std::env::var_os("PATH");
        let root = std::env::temp_dir().join(format!(
            "memd-host-cli-auth-probe-fail-{}",
            uuid::Uuid::new_v4()
        ));
        let bundle = root.join(".memd");
        let bin = root.join("bin");
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        fs::create_dir_all(&bin).expect("create fake bin");
        let gh = bin.join("gh");
        fs::write(&gh, "#!/bin/sh\nexit 1\n").expect("write fake gh");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&gh).expect("fake gh metadata").permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&gh, permissions).expect("chmod fake gh");
        }
        unsafe {
            std::env::set_var("PATH", &bin);
        }

        let mut record = capability(
            "local",
            "cli",
            "gh",
            "host-local",
            &gh.display().to_string(),
        );
        record.notes = vec![
            "PATH inventory; executable availability is host-local".to_string(),
            "memd:host-cli-install-plan:#!/bin/sh\necho install gh\n".to_string(),
        ];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: None,
            capabilities: vec![record],
        };

        let report = build_capabilities_response_from_registry(
            &CapabilitiesArgs {
                command: None,
                output: bundle.clone(),
                harness: None,
                kind: None,
                portability: None,
                query: None,
                limit: 12,
                summary: false,
                json: false,
                materialize_plan: true,
                materialize: false,
            },
            &bundle,
            &registry,
            &CapabilityBridgeRegistry {
                generated_at: Utc::now(),
                actions: Vec::new(),
            },
            "status",
            false,
            12,
        )
        .expect("capability report")
        .materialization
        .expect("materialization report");

        assert_eq!(report.auth_gaps, 1);
        assert_eq!(report.auth_unknown, 0);
        assert_eq!(report.auth_authenticated, 0);
        assert_eq!(report.auth_unauthenticated, 1);
        let action = report.actions.first().expect("host CLI action");
        assert_eq!(action.auth_status.as_deref(), Some("unauthenticated"));
        assert_eq!(action.auth_check.as_deref(), Some("gh auth status"));

        unsafe {
            match old_path {
                Some(value) => std::env::set_var("PATH", value),
                None => std::env::remove_var("PATH"),
            }
        }
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn approved_host_cli_install_plan_runs_and_rechecks_path() {
        let _guard = lock_host_cli_install_env();
        let old_approved = std::env::var_os("MEMD_HOST_CLI_INSTALL_APPROVED");
        let old_path = std::env::var_os("PATH");
        let root = std::env::temp_dir().join(format!(
            "memd-host-cli-approved-install-{}",
            uuid::Uuid::new_v4()
        ));
        let bundle = root.join(".memd");
        let bin = root.join("bin");
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        fs::create_dir_all(&bin).expect("create fake bin");
        let cli_path = bin.join("memd-test-runner");
        let plan = format!(
            "#!/bin/sh\nset -eu\ncat > '{}' <<'EOF'\n#!/bin/sh\nexit 0\nEOF\nchmod +x '{}'\n",
            cli_path.display(),
            cli_path.display()
        );
        let mut record = capability(
            "local",
            "cli",
            "memd-test-runner",
            "host-local",
            "host-cli:memd-test-runner",
        );
        record.notes = vec![
            "PATH inventory; executable availability is host-local".to_string(),
            format!("memd:host-cli-install-plan:{plan}"),
        ];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: None,
            capabilities: vec![record],
        };
        write_bundle_capability_registry(&bundle, &registry).expect("write registry");

        let mut paths = vec![bin.clone()];
        if let Some(old_path) = old_path.as_ref() {
            paths.extend(std::env::split_paths(old_path));
        }
        let joined_path = std::env::join_paths(paths).expect("join PATH");
        unsafe {
            std::env::set_var("MEMD_HOST_CLI_INSTALL_APPROVED", "1");
            std::env::set_var("PATH", &joined_path);
        }

        let report = build_capabilities_response_from_registry(
            &CapabilitiesArgs {
                command: None,
                output: bundle.clone(),
                harness: None,
                kind: None,
                portability: None,
                query: None,
                limit: 12,
                summary: false,
                json: false,
                materialize_plan: false,
                materialize: true,
            },
            &bundle,
            &registry,
            &CapabilityBridgeRegistry {
                generated_at: Utc::now(),
                actions: Vec::new(),
            },
            "status",
            false,
            12,
        )
        .expect("capability report")
        .materialization
        .expect("materialization report");

        unsafe {
            match old_approved {
                Some(value) => std::env::set_var("MEMD_HOST_CLI_INSTALL_APPROVED", value),
                None => std::env::remove_var("MEMD_HOST_CLI_INSTALL_APPROVED"),
            }
            match old_path {
                Some(value) => std::env::set_var("PATH", value),
                None => std::env::remove_var("PATH"),
            }
        }

        let action = report
            .actions
            .iter()
            .find(|action| action.name == "memd-test-runner")
            .expect("memd-test-runner action");
        assert_eq!(action.status, "present");
        assert!(action.reason.contains("now available on PATH"));
        assert!(cli_path.is_file());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn host_cli_on_path_keeps_fresh_machine_materialization_partial() {
        if !host_cli_available_on_path("sh") {
            return;
        }
        let bundle =
            std::env::temp_dir().join(format!("memd-host-cli-on-path-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        let mut record = capability("local", "cli", "sh", "host-local", "/bin/sh");
        record.notes = vec![
            "PATH inventory; executable availability is host-local".to_string(),
            "memd:host-cli-install-plan:#!/bin/sh\necho install sh\n".to_string(),
        ];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: None,
            capabilities: vec![record],
        };
        write_bundle_capability_registry(&bundle, &registry).expect("write registry");

        let report = build_capabilities_response_from_registry(
            &CapabilitiesArgs {
                command: None,
                output: bundle.clone(),
                harness: None,
                kind: None,
                portability: None,
                query: None,
                limit: 12,
                summary: false,
                json: false,
                materialize_plan: true,
                materialize: false,
            },
            &bundle,
            &registry,
            &CapabilityBridgeRegistry {
                generated_at: Utc::now(),
                actions: Vec::new(),
            },
            "status",
            false,
            12,
        )
        .expect("capability report")
        .materialization
        .expect("materialization report");

        assert_eq!(report.status, "partial-host-local");
        assert_eq!(report.missing, 0);
        assert_eq!(report.host_local, 1);
        assert_eq!(report.auth_gaps, 0);
        assert_eq!(report.auth_unknown, 0);
        assert_eq!(report.auth_authenticated, 0);
        assert_eq!(report.auth_unauthenticated, 0);
        assert!(!report.fresh_machine_ready);
        let action = report.actions.first().expect("host CLI action");
        assert_eq!(action.status, "present");
        assert_eq!(action.action, "host-cli-on-path");
        assert!(action.reason.contains("fresh machines still need"));

        fs::remove_dir_all(bundle).ok();
    }

    #[test]
    fn materialize_restores_bundle_relative_assets() {
        let root =
            std::env::temp_dir().join(format!("memd-materialize-apply-{}", uuid::Uuid::new_v4()));
        let bundle = root.join(".memd");
        fs::create_dir_all(bundle.join("agents")).expect("create bundle agents");
        fs::write(bundle.join("agents").join("mercury.sh"), "#!/bin/sh\n").expect("write source");
        let target = root.join("agents").join("mercury.sh");
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: Some(root.display().to_string()),
            capabilities: vec![capability(
                "hermes",
                "harness-pack",
                "Mercury",
                "universal",
                "agents/mercury.sh",
            )],
        };
        write_bundle_capability_registry(&bundle, &registry).expect("write registry");

        let report = run_capabilities_command(&CapabilitiesArgs {
            command: None,
            output: bundle.clone(),
            harness: None,
            kind: None,
            portability: None,
            query: None,
            limit: 12,
            summary: false,
            json: false,
            materialize_plan: false,
            materialize: true,
        })
        .expect("capability report")
        .materialization
        .expect("materialization report");

        assert_eq!(report.applied, 1);
        assert!(target.is_file());
        assert_eq!(
            fs::read_to_string(&target).expect("read restored"),
            "#!/bin/sh\n"
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn materialize_restores_text_payload_assets() {
        let root =
            std::env::temp_dir().join(format!("memd-payload-apply-{}", uuid::Uuid::new_v4()));
        let bundle = root.join(".memd");
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        let target = root.join("skills").join("demo").join("SKILL.md");
        let mut record = capability(
            "codex",
            "skill",
            "demo",
            "harness-native",
            "skills/demo/SKILL.md",
        );
        record.notes = vec!["memd:payload-text:# Demo\n".to_string()];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: Some(root.display().to_string()),
            capabilities: vec![record],
        };
        write_bundle_capability_registry(&bundle, &registry).expect("write registry");

        let report = run_capabilities_command(&CapabilitiesArgs {
            command: None,
            output: bundle.clone(),
            harness: None,
            kind: None,
            portability: None,
            query: None,
            limit: 12,
            summary: false,
            json: false,
            materialize_plan: false,
            materialize: true,
        })
        .expect("capability report")
        .materialization
        .expect("materialization report");

        assert_eq!(report.applied, 1);
        assert_eq!(
            fs::read_to_string(&target).expect("read payload"),
            "# Demo\n"
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn materialize_restores_text_payload_file_sets() {
        let root =
            std::env::temp_dir().join(format!("memd-payload-set-apply-{}", uuid::Uuid::new_v4()));
        let bundle = root.join(".memd");
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        let mut record = capability(
            "codex",
            "skill",
            "demo",
            "harness-native",
            "skills/demo/SKILL.md",
        );
        record.notes = vec![
            "memd:payload-text:# Demo\n".to_string(),
            format!(
                "memd:payload-file-json:{}",
                serde_json::json!({"path": "SKILL.md", "content": "# Demo\n"})
            ),
            format!(
                "memd:payload-file-json:{}",
                serde_json::json!({"path": "scripts/run.sh", "content": "#!/bin/sh\necho demo\n"})
            ),
            format!(
                "memd:payload-file-json:{}",
                serde_json::json!({"path": "../escape.sh", "content": "nope\n"})
            ),
        ];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: Some(root.display().to_string()),
            capabilities: vec![record],
        };
        write_bundle_capability_registry(&bundle, &registry).expect("write registry");

        let report = run_capabilities_command(&CapabilitiesArgs {
            command: None,
            output: bundle.clone(),
            harness: None,
            kind: None,
            portability: None,
            query: None,
            limit: 12,
            summary: false,
            json: false,
            materialize_plan: false,
            materialize: true,
        })
        .expect("capability report")
        .materialization
        .expect("materialization report");

        assert_eq!(report.applied, 1);
        let action = report
            .actions
            .iter()
            .find(|action| {
                action.harness == "codex" && action.kind == "skill" && action.name == "demo"
            })
            .expect("payload set action");
        assert_eq!(action.action, "restore-from-payload-set");
        assert_eq!(action.status, "present");
        assert_eq!(
            fs::read_to_string(root.join("skills/demo/SKILL.md")).expect("read skill"),
            "# Demo\n"
        );
        assert_eq!(
            fs::read_to_string(root.join("skills/demo/scripts/run.sh")).expect("read script"),
            "#!/bin/sh\necho demo\n"
        );
        assert!(!root.join("skills/escape.sh").exists());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn materialize_plan_does_not_serialize_payload_body() {
        let root = std::env::temp_dir().join(format!("memd-payload-plan-{}", uuid::Uuid::new_v4()));
        let bundle = root.join(".memd");
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        let mut record = capability(
            "codex",
            "skill",
            "demo",
            "harness-native",
            "skills/demo/SKILL.md",
        );
        record.notes = vec!["memd:payload-text:# Secret-ish payload\n".to_string()];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: Some(root.display().to_string()),
            capabilities: vec![record],
        };
        write_bundle_capability_registry(&bundle, &registry).expect("write registry");

        let report = run_capabilities_command(&CapabilitiesArgs {
            command: None,
            output: bundle.clone(),
            harness: None,
            kind: None,
            portability: None,
            query: None,
            limit: 12,
            summary: false,
            json: false,
            materialize_plan: true,
            materialize: false,
        })
        .expect("capability report");
        let json = serde_json::to_string(&report.materialization).expect("serialize report");

        assert!(json.contains("restore-from-payload"));
        assert!(!json.contains("Secret-ish payload"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn pull_mode_materialize_flag_restores_payload_assets() {
        let root = std::env::temp_dir().join(format!("memd-payload-pull-{}", uuid::Uuid::new_v4()));
        let bundle = root.join(".memd");
        fs::create_dir_all(bundle.join("state")).expect("create bundle state");
        let target = root.join("skills").join("demo").join("SKILL.md");
        let mut record = capability(
            "codex",
            "skill",
            "demo",
            "harness-native",
            "skills/demo/SKILL.md",
        );
        record.notes = vec!["memd:payload-text:# Pull Demo\n".to_string()];
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: Some(root.display().to_string()),
            capabilities: vec![record],
        };

        let report = build_capabilities_response_from_registry(
            &CapabilitiesArgs {
                command: Some(CapabilitiesSubcommand::Pull(CapabilitiesPullArgs {
                    output: bundle.clone(),
                    json: false,
                    materialize_plan: false,
                    materialize: true,
                })),
                output: PathBuf::from("ignored-by-pull"),
                harness: None,
                kind: None,
                portability: None,
                query: None,
                limit: 12,
                summary: false,
                json: false,
                materialize_plan: false,
                materialize: false,
            },
            &bundle,
            &registry,
            &CapabilityBridgeRegistry {
                generated_at: Utc::now(),
                actions: Vec::new(),
            },
            "pull",
            true,
            12,
        )
        .expect("capability response")
        .materialization
        .expect("materialization report");

        assert_eq!(report.applied, 1);
        assert_eq!(
            fs::read_to_string(&target).expect("read payload"),
            "# Pull Demo\n"
        );

        fs::remove_dir_all(root).ok();
    }
}

pub(crate) fn render_capabilities_runtime_summary(response: &CapabilitiesResponse) -> String {
    let harnesses = response
        .harnesses
        .iter()
        .take(4)
        .map(|harness| {
            format!(
                "{}:{}/{}/{}",
                harness.harness, harness.capabilities, harness.installed, harness.bridge_actions
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let materialization = response
        .materialization
        .as_ref()
        .map(|report| {
            format!(
                " materialize={} installable={} missing={} host_local={} auth_gaps={} auth_unknown={} auth_authenticated={} auth_unauthenticated={} fresh_machine_ready={} applied={} skipped={}",
                report.status,
                report.installable,
                report.missing,
                report.host_local,
                report.auth_gaps,
                report.auth_unknown,
                report.auth_authenticated,
                report.auth_unauthenticated,
                report.fresh_machine_ready,
                report.applied,
                report.skipped
            )
        })
        .unwrap_or_default();
    if harnesses.is_empty() {
        format!(
            "capabilities bundle={} discovered={} universal={} bridgeable={} harness_native={} bridge_actions={} wired_harnesses={} shown={} harnesses=none{}",
            response.bundle_root,
            response.discovered,
            response.universal,
            response.bridgeable,
            response.harness_native,
            response.bridge_actions,
            response.wired_harnesses,
            response.records.len(),
            materialization,
        )
    } else {
        format!(
            "capabilities bundle={} discovered={} universal={} bridgeable={} harness_native={} bridge_actions={} wired_harnesses={} shown={} harnesses={}{}",
            response.bundle_root,
            response.discovered,
            response.universal,
            response.bridgeable,
            response.harness_native,
            response.bridge_actions,
            response.wired_harnesses,
            response.records.len(),
            harnesses,
            materialization,
        )
    }
}

pub(crate) fn read_recent_maintain_reports(
    output: &Path,
    limit: usize,
) -> anyhow::Result<Vec<MaintainReport>> {
    let dir = maintain_reports_dir(output);
    if !dir.exists() || limit == 0 {
        return Ok(Vec::new());
    }
    let mut candidates = fs::read_dir(&dir)
        .with_context(|| format!("read {}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|value| value.path()))
        .filter(|path| {
            path.extension().and_then(|value| value.to_str()) == Some("json")
                && path.file_name().and_then(|value| value.to_str()) != Some("latest.json")
        })
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.reverse();
    let mut reports = Vec::new();
    for path in candidates.into_iter().take(limit) {
        let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let report = serde_json::from_str::<MaintainReport>(&raw)
            .with_context(|| format!("parse {}", path.display()))?;
        reports.push(report);
    }
    Ok(reports)
}

pub(crate) fn write_skill_policy_artifacts(
    output: &Path,
    response: &MemoryPolicyResponse,
    report: &SkillLifecycleReport,
    apply_queues: bool,
) -> anyhow::Result<Option<SkillPolicyApplyArtifact>> {
    let runtime_defaulted = is_default_runtime(&response.runtime);
    let batch = SkillPolicyBatchArtifact {
        generated_at: Utc::now(),
        bundle_root: output.display().to_string(),
        runtime_defaulted,
        report: report.clone(),
    };
    let batch_json = serde_json::to_string_pretty(&batch)? + "\n";
    let batch_markdown = render_skill_policy_batch_markdown(&batch);
    write_state_artifact(
        skill_policy_batch_state_path(output),
        &batch_json,
        "skill-policy batch json",
    )?;
    write_state_artifact(
        skill_policy_batch_markdown_path(output),
        &batch_markdown,
        "skill-policy batch markdown",
    )?;

    let review = SkillPolicyQueueArtifact {
        generated_at: Utc::now(),
        bundle_root: output.display().to_string(),
        runtime_defaulted,
        queue: "review".to_string(),
        records: report.review_queue.clone(),
    };
    let review_json = serde_json::to_string_pretty(&review)? + "\n";
    let review_markdown = render_skill_policy_queue_markdown(&review);
    write_state_artifact(
        skill_policy_review_state_path(output),
        &review_json,
        "skill-policy review queue json",
    )?;
    write_state_artifact(
        skill_policy_review_markdown_path(output),
        &review_markdown,
        "skill-policy review queue markdown",
    )?;

    let activate = SkillPolicyQueueArtifact {
        generated_at: Utc::now(),
        bundle_root: output.display().to_string(),
        runtime_defaulted,
        queue: "activate".to_string(),
        records: report.activate_queue.clone(),
    };
    let activate_json = serde_json::to_string_pretty(&activate)? + "\n";
    let activate_markdown = render_skill_policy_queue_markdown(&activate);
    write_state_artifact(
        skill_policy_activate_state_path(output),
        &activate_json,
        "skill-policy activate queue json",
    )?;
    write_state_artifact(
        skill_policy_activate_markdown_path(output),
        &activate_markdown,
        "skill-policy activate queue markdown",
    )?;

    let receipt = if apply_queues {
        let receipt = consume_skill_policy_activate_queue(output)?;
        if let Some(receipt) = receipt.as_ref() {
            let apply_json = serde_json::to_string_pretty(receipt)? + "\n";
            let apply_markdown = render_skill_policy_apply_markdown(receipt);
            write_state_artifact(
                skill_policy_apply_state_path(output),
                &apply_json,
                "skill-policy apply receipt json",
            )?;
            write_state_artifact(
                skill_policy_apply_markdown_path(output),
                &apply_markdown,
                "skill-policy apply receipt markdown",
            )?;
        }
        receipt
    } else {
        None
    };

    Ok(receipt)
}

pub(crate) fn consume_skill_policy_activate_queue(
    output: &Path,
) -> anyhow::Result<Option<SkillPolicyApplyArtifact>> {
    let path = skill_policy_activate_state_path(output);
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let queue = serde_json::from_str::<SkillPolicyQueueArtifact>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    let applied = queue
        .records
        .iter()
        .filter(|record| record.activation == "activate")
        .cloned()
        .collect::<Vec<_>>();
    let skipped = queue
        .records
        .iter()
        .filter(|record| record.activation != "activate")
        .cloned()
        .collect::<Vec<_>>();
    let receipt = SkillPolicyApplyArtifact {
        generated_at: Utc::now(),
        bundle_root: queue.bundle_root.clone(),
        runtime_defaulted: queue.runtime_defaulted,
        source_queue_path: path.display().to_string(),
        applied_count: applied.len(),
        skipped_count: skipped.len(),
        applied,
        skipped,
    };

    Ok(Some(receipt))
}

pub(crate) fn skill_policy_apply_request(
    receipt: &SkillPolicyApplyArtifact,
) -> SkillPolicyApplyRequest {
    SkillPolicyApplyRequest {
        bundle_root: receipt.bundle_root.clone(),
        runtime_defaulted: receipt.runtime_defaulted,
        source_queue_path: receipt.source_queue_path.clone(),
        applied_count: receipt.applied_count,
        skipped_count: receipt.skipped_count,
        applied: receipt.applied.iter().map(to_activation_record).collect(),
        skipped: receipt.skipped.iter().map(to_activation_record).collect(),
        project: None,
        namespace: None,
        workspace: None,
    }
}

pub(crate) fn to_activation_record(record: &SkillLifecycleRecord) -> SkillPolicyActivationRecord {
    SkillPolicyActivationRecord {
        harness: record.harness.clone(),
        name: record.name.clone(),
        kind: record.kind.clone(),
        portability_class: record.portability_class.clone(),
        proposal: record.proposal.clone(),
        sandbox: record.sandbox.clone(),
        sandbox_risk: record.sandbox_risk,
        sandbox_reason: record.sandbox_reason.clone(),
        activation: record.activation.clone(),
        activation_reason: record.activation_reason.clone(),
        source_path: record.source_path.clone(),
        target_path: record.target_path.clone(),
        notes: record.notes.clone(),
    }
}

pub(crate) fn write_state_artifact(
    path: PathBuf,
    content: &str,
    label: &str,
) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, content).with_context(|| format!("write {label} {}", path.display()))
}

pub(crate) fn describe_resume_state_changes(
    previous: Option<&BundleResumeState>,
    current: &BundleResumeState,
) -> Vec<String> {
    let Some(previous) = previous else {
        return Vec::new();
    };

    let mut changes = Vec::new();

    if previous.focus != current.focus
        && let Some(focus) = current.focus.as_deref()
    {
        changes.push(format!("focus -> {}", compact_inline(focus, 120)));
    }
    if previous.pressure != current.pressure
        && let Some(pressure) = current.pressure.as_deref()
    {
        changes.push(format!("pressure -> {}", compact_inline(pressure, 120)));
    }
    if previous.next_recovery != current.next_recovery
        && let Some(next_recovery) = current.next_recovery.as_deref()
    {
        changes.push(format!(
            "next_recovery -> {}",
            compact_inline(next_recovery, 120)
        ));
    }
    if previous.lane != current.lane
        && let Some(lane) = current.lane.as_deref()
    {
        changes.push(format!("lane -> {}", compact_inline(lane, 120)));
    }
    if previous.working_records != current.working_records {
        changes.push(format!(
            "working {} -> {}",
            previous.working_records, current.working_records
        ));
    }
    if previous.inbox_items != current.inbox_items {
        changes.push(format!(
            "inbox {} -> {}",
            previous.inbox_items, current.inbox_items
        ));
    }
    if previous.rehydration_items != current.rehydration_items {
        changes.push(format!(
            "rehydration {} -> {}",
            previous.rehydration_items, current.rehydration_items
        ));
    }

    changes
}

pub(crate) fn compact_inline(value: &str, max_chars: usize) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() <= max_chars {
        return normalized;
    }

    let mut compact = String::with_capacity(max_chars + 3);
    for ch in normalized.chars().take(max_chars.saturating_sub(3)) {
        compact.push(ch);
    }
    compact.push_str("...");
    compact
}

pub(crate) fn render_voice_mode_section(voice_mode: &str) -> String {
    let voice_mode =
        normalize_voice_mode_value(voice_mode).unwrap_or_else(|_| default_voice_mode());
    match voice_mode.as_str() {
        "normal" => "- default: normal\n- keep replies clear and complete\n- avoid forced compression\n".to_string(),
        "caveman-lite" => "- default: `caveman-lite`\n- no filler/hedging, keep articles + full sentences\n- professional but tight\n- keep exact technical terms\n".to_string(),
        "caveman-full" => "- default: `caveman-full`\n- drop articles, fragments OK, short synonyms\n- keep exact technical terms\n- keep technical accuracy\n".to_string(),
        "caveman-ultra" => "- default: `caveman-ultra`\n- abbreviate (DB/auth/config/req/res/fn/impl), strip conjunctions\n- arrows for causality (X → Y), one word when one word enough\n- keep exact technical terms\n- keep technical accuracy\n- match `.memd/config.json` exactly if the user changes voice_mode\n- reply style is derived from config; if draft slips, rewrite before sending\n".to_string(),
        "wenyan-lite" => "- default: `wenyan-lite`\n- semi-classical Chinese, drop filler/hedging but keep grammar structure\n- classical register\n".to_string(),
        "wenyan-full" => "- default: `wenyan-full`\n- maximum classical terseness, fully 文言文\n- 80-90% character reduction, classical sentence patterns\n".to_string(),
        "wenyan-ultra" => "- default: `wenyan-ultra`\n- extreme abbreviation, classical Chinese feel, maximum compression\n".to_string(),
        _ => "- default: `caveman-full`\n- drop articles, fragments OK, short synonyms\n- keep exact technical terms\n- keep technical accuracy\n".to_string(),
    }
}

pub(crate) fn render_codex_agents_bridge_markdown(output: &Path) -> String {
    let voice_mode = read_bundle_voice_mode(output).unwrap_or_else(default_voice_mode);
    let normalized =
        normalize_voice_mode_value(&voice_mode).unwrap_or_else(|_| default_voice_mode());
    format!(
        "These instructions are managed by memd.\n\n## memd voice bootstrap\n\n- Treat `.memd/config.json` as the source of truth for this repo's active `voice_mode`.\n- Valid repo voice modes are `normal`, `caveman-lite`, `caveman-full`, `caveman-ultra`, `wenyan-lite`, `wenyan-full`, and `wenyan-ultra`.\n- If the user asks which voice is active, answer from `.memd/config.json`.\n- Do not tell the user to manually enable a voice that `.memd/config.json` already sets.\n- Do not invent a second source of truth for voice mode.\n- Do not slip from the repo voice mode; stay in `{current_default}` unless `.memd/config.json` changes.\n- Caveman modes mean compressed wording, not broken spelling.\n- Keep normal spelling and exact technical terms even when voice mode is `caveman-lite` or `caveman-ultra`.\n- Reply style is derived from config. If your draft is not in `{current_default}`, stop and rewrite it before sending.\n\n## current repo default\n\n- The current bundle file `.memd/config.json` sets `voice_mode` to `{current_default}`.\n- Until that bundle setting changes, use `{current_default}` by default in this repo.\n\n## memd runtime\n\n- memd is the memory/bootstrap dependency for this repo.\n- Treat memd bundle state as startup truth before answering.\n- Start from `.memd/wake.md` before relying on transcript recall.\n- Use `.memd/mem.md` for the deeper compact memory view.\n- Use `.memd/events.md` for the event log.\n- Durable truth beats transcript recall.\n- For decisions, preferences, project history, or prior corrections, run `memd lookup --output .memd --query \"...\"` before answering.\n- Use `memd hook spill --output .memd --stdin --apply` at compaction boundaries to turn turn-state deltas into durable candidate memory.\n- If the user corrects you, write the correction back instead of trusting the transcript.\n- Keep responses short, direct, and token-efficient unless the user asks for detail.\n",
        current_default = normalized,
    )
}

pub(crate) fn upsert_project_agents_bridge(output: &Path) -> anyhow::Result<()> {
    const START: &str = "<!-- memd-managed:start -->";
    const END: &str = "<!-- memd-managed:end -->";

    let project_root = project_root_from_bundle(output);
    let agents_path = project_root.join("AGENTS.md");
    let managed = format!(
        "{START}\n{}\n{END}\n",
        render_codex_agents_bridge_markdown(output)
    );

    let next = match fs::read_to_string(&agents_path) {
        Ok(existing) => {
            if let (Some(start), Some(end)) = (existing.find(START), existing.find(END)) {
                let end = end + END.len();
                format!(
                    "{}{}{}",
                    &existing[..start],
                    managed,
                    existing[end..].trim_start_matches('\n')
                )
            } else if existing.trim().is_empty() {
                format!("# AGENTS.md\n\n{managed}")
            } else {
                format!("{}\n\n{}", existing.trim_end(), managed)
            }
        }
        Err(_) => format!("# AGENTS.md\n\n{managed}"),
    };

    fs::write(&agents_path, next).with_context(|| format!("write {}", agents_path.display()))?;
    Ok(())
}

pub(crate) fn upsert_project_claude_bridge(output: &Path) -> anyhow::Result<()> {
    const START: &str = "<!-- memd-managed:claude-import:start -->";
    const END: &str = "<!-- memd-managed:claude-import:end -->";

    let Some(project_root) = infer_bundle_project_root(output) else {
        return Ok(());
    };
    let claude_path = project_root.join("CLAUDE.md");
    let managed = format!("{START}\n@.memd/agents/CLAUDE_IMPORTS.md\n{END}\n");

    let next = match fs::read_to_string(&claude_path) {
        Ok(existing) => {
            if existing.contains("@.memd/agents/CLAUDE_IMPORTS.md") {
                existing
            } else if let (Some(start), Some(end)) = (existing.find(START), existing.find(END)) {
                let end = end + END.len();
                format!(
                    "{}{}{}",
                    &existing[..start],
                    managed,
                    existing[end..].trim_start_matches('\n')
                )
            } else if existing.trim().is_empty() {
                format!("# Claude Instructions\n\n{managed}")
            } else if let Some((first, rest)) = existing.split_once('\n') {
                if first.trim_start().starts_with('#') {
                    format!("{first}\n\n{managed}\n{}", rest.trim_start_matches('\n'))
                } else {
                    format!("{managed}\n{}", existing.trim_start_matches('\n'))
                }
            } else if existing.trim_start().starts_with('#') {
                format!("{}\n\n{managed}", existing.trim_end())
            } else {
                format!("{managed}\n{}", existing.trim_start_matches('\n'))
            }
        }
        Err(_) => format!("# Claude Instructions\n\n{managed}"),
    };

    fs::write(&claude_path, next).with_context(|| format!("write {}", claude_path.display()))?;
    Ok(())
}

pub(crate) fn write_native_agent_bridge_files(output: &Path) -> anyhow::Result<()> {
    let agents_dir = output.join("agents");
    fs::create_dir_all(&agents_dir).with_context(|| format!("create {}", agents_dir.display()))?;
    let claude_imports = agents_dir.join("CLAUDE_IMPORTS.md");
    fs::write(
        &claude_imports,
        format!(
            "# memd imports for Claude Code\n\n@../wake.md\n\nDeeper recall: `memd resume --output {bundle}` or `memd lookup --output {bundle} --query \"...\"`.\n",
            bundle = output.display(),
        ),
    )
    .with_context(|| format!("write {}", claude_imports.display()))?;

    let claude_example = agents_dir.join("CLAUDE.md.example");
    fs::write(
        &claude_example,
        "# Claude Code project memory\n\n@.memd/agents/CLAUDE_IMPORTS.md\n",
    )
    .with_context(|| format!("write {}", claude_example.display()))?;

    let codex_example = agents_dir.join("AGENTS.md.example");
    fs::write(
        &codex_example,
        format!(
            "# AGENTS.md\n\n<!-- memd-managed:start -->\n{}\n<!-- memd-managed:end -->\n",
            render_codex_agents_bridge_markdown(output)
        ),
    )
    .with_context(|| format!("write {}", codex_example.display()))?;

    upsert_project_agents_bridge(output)?;
    upsert_project_claude_bridge(output)?;

    Ok(())
}

pub(crate) fn write_bundle_command_catalog_files(output: &Path) -> anyhow::Result<()> {
    let catalog = build_command_catalog(output);
    let commands = output.join("COMMANDS.md");
    fs::write(&commands, render_command_catalog_markdown(&catalog))
        .with_context(|| format!("write {}", commands.display()))?;
    Ok(())
}

pub(crate) fn render_authority_warning_markdown(output: &Path) -> String {
    let authority_warning = read_bundle_runtime_config(output)
        .ok()
        .flatten()
        .map(|runtime| authority_warning_lines(Some(&runtime)))
        .unwrap_or_default();
    if authority_warning.is_empty() {
        return String::new();
    }

    format!(
        "## Session Start Warning\n\n{}\n\n",
        authority_warning
            .iter()
            .map(|line| format!("- {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

pub(crate) fn write_memory_markdown_files(output: &Path, markdown: &str) -> anyhow::Result<()> {
    let authority_warning = render_authority_warning_markdown(output);
    let markdown = if authority_warning.is_empty() {
        markdown.to_string()
    } else {
        format!("{authority_warning}{markdown}")
    };
    let root_memory = output.join("mem.md");
    fs::write(&root_memory, &markdown)
        .with_context(|| format!("write {}", root_memory.display()))?;

    Ok(())
}

pub(crate) fn write_wakeup_markdown_files(output: &Path, markdown: &str) -> anyhow::Result<()> {
    let authority_warning = render_authority_warning_markdown(output);
    let markdown = if authority_warning.is_empty() {
        markdown.to_string()
    } else {
        format!("{authority_warning}{markdown}")
    };
    let root_wakeup = output.join("wake.md");
    fs::write(&root_wakeup, &markdown)
        .with_context(|| format!("write {}", root_wakeup.display()))?;

    Ok(())
}

pub(crate) fn write_bundle_memory_object_pages(
    output: &Path,
    snapshot: &ResumeSnapshot,
    handoff: Option<&HandoffSnapshot>,
    hive: Option<&BundleHiveMemorySurface>,
) -> anyhow::Result<()> {
    let dir = bundle_compiled_memory_dir(output);
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    for lane in [
        MemoryObjectLane::Context,
        MemoryObjectLane::Working,
        MemoryObjectLane::Inbox,
        MemoryObjectLane::Recovery,
        MemoryObjectLane::Semantic,
        MemoryObjectLane::Workspace,
    ] {
        let path = bundle_compiled_memory_path(output, lane);
        let markdown = render_bundle_memory_object_markdown(output, snapshot, handoff, hive, lane);
        fs::write(&path, markdown).with_context(|| format!("write {}", path.display()))?;
        let item_count = match lane {
            MemoryObjectLane::Context => snapshot.context.records.len(),
            MemoryObjectLane::Working => snapshot.working.records.len(),
            MemoryObjectLane::Inbox => snapshot.inbox.items.len(),
            MemoryObjectLane::Recovery => snapshot.working.rehydration_queue.len(),
            MemoryObjectLane::Semantic => snapshot
                .semantic
                .as_ref()
                .map(|semantic| semantic.items.len())
                .unwrap_or(0),
            MemoryObjectLane::Workspace => snapshot.workspaces.workspaces.len(),
        };
        for index in 0..item_count {
            if let Some(key) = memory_object_lane_item_key(snapshot, lane, index) {
                let item_path = bundle_compiled_memory_item_path(output, lane, index, &key);
                if let Some(parent) = item_path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("create {}", parent.display()))?;
                }
                let item_markdown = render_bundle_memory_object_item_markdown(
                    output, snapshot, handoff, hive, lane, index,
                )
                .unwrap_or_else(|| format!("# memd memory item: {}\n\n- none\n", lane.title()));
                fs::write(&item_path, item_markdown)
                    .with_context(|| format!("write {}", item_path.display()))?;
            }
        }
    }
    Ok(())
}

pub(crate) fn write_bundle_eval_artifacts(
    output: &Path,
    response: &BundleEvalResponse,
) -> anyhow::Result<()> {
    let evals_dir = output.join("evals");
    fs::create_dir_all(&evals_dir).with_context(|| format!("create {}", evals_dir.display()))?;

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let json = serde_json::to_string_pretty(response)? + "\n";
    let markdown = render_bundle_eval_markdown(response);

    let latest_json = evals_dir.join("latest.json");
    let latest_md = evals_dir.join("latest.md");
    let timestamped_json = evals_dir.join(format!("{timestamp}.json"));
    let timestamped_md = evals_dir.join(format!("{timestamp}.md"));

    fs::write(&latest_json, &json).with_context(|| format!("write {}", latest_json.display()))?;
    fs::write(&latest_md, &markdown).with_context(|| format!("write {}", latest_md.display()))?;
    fs::write(&timestamped_json, &json)
        .with_context(|| format!("write {}", timestamped_json.display()))?;
    fs::write(&timestamped_md, &markdown)
        .with_context(|| format!("write {}", timestamped_md.display()))?;

    Ok(())
}

pub(crate) fn maintain_reports_dir(output: &Path) -> PathBuf {
    output.join("maintenance")
}

pub(crate) fn read_latest_maintain_report(output: &Path) -> anyhow::Result<Option<MaintainReport>> {
    let path = maintain_reports_dir(output).join("latest.json");
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let report = serde_json::from_str::<MaintainReport>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(report))
}

pub(crate) fn read_previous_maintain_report(
    output: &Path,
) -> anyhow::Result<Option<MaintainReport>> {
    let dir = maintain_reports_dir(output);
    if !dir.exists() {
        return Ok(None);
    }
    let mut candidates = fs::read_dir(&dir)
        .with_context(|| format!("read {}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|value| value.path()))
        .filter(|path| {
            path.extension().and_then(|value| value.to_str()) == Some("json")
                && path.file_name().and_then(|value| value.to_str()) != Some("latest.json")
        })
        .collect::<Vec<_>>();
    candidates.sort();
    let Some(path) = candidates.into_iter().next_back() else {
        return Ok(None);
    };
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let report = serde_json::from_str::<MaintainReport>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(report))
}

pub(crate) fn write_maintain_artifacts(
    output: &Path,
    response: &MaintainReport,
) -> anyhow::Result<()> {
    let maintain_dir = maintain_reports_dir(output);
    fs::create_dir_all(&maintain_dir)
        .with_context(|| format!("create {}", maintain_dir.display()))?;

    let timestamp = response.generated_at.format("%Y%m%dT%H%M%SZ").to_string();
    let json = serde_json::to_string_pretty(response)? + "\n";
    let markdown = format!(
        "# memd maintain report\n\n- mode: {}\n- receipt: {}\n- compacted: {}\n- refreshed: {}\n- repaired: {}\n\n## Findings\n{}\n",
        response.mode.as_str(),
        response.receipt_id.as_deref().unwrap_or("none"),
        response.compacted_items,
        response.refreshed_items,
        response.repaired_items,
        if response.findings.is_empty() {
            "- none".to_string()
        } else {
            response
                .findings
                .iter()
                .map(|value| format!("- {}", value))
                .collect::<Vec<_>>()
                .join("\n")
        }
    );

    let latest_json = maintain_dir.join("latest.json");
    let latest_md = maintain_dir.join("latest.md");
    let timestamped_json = maintain_dir.join(format!("{timestamp}.json"));
    let timestamped_md = maintain_dir.join(format!("{timestamp}.md"));

    fs::write(&latest_json, &json).with_context(|| format!("write {}", latest_json.display()))?;
    fs::write(&latest_md, &markdown).with_context(|| format!("write {}", latest_md.display()))?;
    fs::write(&timestamped_json, &json)
        .with_context(|| format!("write {}", timestamped_json.display()))?;
    fs::write(&timestamped_md, &markdown)
        .with_context(|| format!("write {}", timestamped_md.display()))?;
    Ok(())
}

pub(crate) async fn run_maintain_command(
    args: &MaintainArgs,
    base_url: &str,
) -> anyhow::Result<MaintainReport> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let client = MemdClient::new(base_url)?;
    let maintenance = client
        .maintenance_report(&MemoryMaintenanceReportRequest {
            project: runtime.as_ref().and_then(|value| value.project.clone()),
            namespace: runtime.as_ref().and_then(|value| value.namespace.clone()),
            inactive_days: Some(7),
            lookback_days: Some(30),
            min_events: Some(2),
            max_decay: Some(0.5),
            mode: Some(args.mode.clone()),
            apply: Some(args.apply),
        })
        .await?;
    let response = MaintainReport {
        mode: args.mode.clone(),
        receipt_id: maintenance.receipt_id.clone(),
        compacted_items: if args.mode == "compact" {
            maintenance
                .compacted_items
                .max(maintenance.consolidated_candidates)
        } else {
            maintenance.compacted_items
        },
        refreshed_items: if args.mode == "refresh" {
            maintenance
                .refreshed_items
                .max(maintenance.reinforced_candidates)
        } else {
            maintenance.refreshed_items
        },
        repaired_items: if args.mode == "repair" {
            maintenance
                .repaired_items
                .max(maintenance.cooled_candidates)
        } else {
            maintenance.repaired_items
        },
        findings: maintenance.highlights.clone(),
        generated_at: maintenance.generated_at,
    };
    write_maintain_artifacts(&args.output, &response)?;
    auto_checkpoint_bundle_event(
        &args.output,
        base_url,
        "maintenance",
        format!(
            "Maintenance {} compacted={} refreshed={} repaired={} findings={}.",
            response.mode.as_str(),
            response.compacted_items,
            response.refreshed_items,
            response.repaired_items,
            response.findings.len()
        ),
        vec!["maintenance".to_string(), response.mode.clone()],
        0.78,
    )
    .await?;
    Ok(response)
}

pub(crate) fn render_maintain_summary(response: &MaintainReport) -> String {
    let findings = if response.findings.is_empty() {
        "none".to_string()
    } else {
        response.findings.join(" | ")
    };
    format!(
        "maintain mode={} receipt={} compacted={} refreshed={} repaired={} findings={}",
        response.mode.as_str(),
        response.receipt_id.as_deref().unwrap_or("none"),
        response.compacted_items,
        response.refreshed_items,
        response.repaired_items,
        findings
    )
}
