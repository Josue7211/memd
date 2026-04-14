use super::*;

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
        last_wake_at: None,
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
    if let Some(topic) = topic_claim
        && let Some(task_id) = topic.strip_prefix("task:") {
            let task_id = task_id.trim();
            if !task_id.is_empty() {
                return Some(task_id.to_string());
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
        && !target_scopes.is_empty()
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
