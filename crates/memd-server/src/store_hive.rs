use memd_schema::{HiveCoordinationReceiptRecord, HiveSessionRecord, HiveTaskRecord};

pub(crate) fn collapse_hive_session_records(
    records: Vec<HiveSessionRecord>,
) -> Vec<HiveSessionRecord> {
    let mut merged = Vec::<HiveSessionRecord>::new();

    for record in records {
        if let Some(existing) = merged
            .iter_mut()
            .find(|existing| hive_session_records_can_merge(existing, &record))
        {
            merge_hive_session_record(existing, &record);
        } else {
            merged.push(record);
        }
    }

    merged
}

fn merge_hive_session_record(target: &mut HiveSessionRecord, fallback: &HiveSessionRecord) {
    merge_option_string(&mut target.tab_id, &fallback.tab_id);
    merge_option_string(&mut target.agent, &fallback.agent);
    merge_option_string(&mut target.effective_agent, &fallback.effective_agent);
    merge_option_string(&mut target.hive_system, &fallback.hive_system);
    merge_option_string(&mut target.hive_role, &fallback.hive_role);
    merge_identity_option_string(&mut target.worker_name, &fallback.worker_name);
    if target
        .worker_name
        .as_deref()
        .is_none_or(worker_identity_needs_display_name)
    {
        merge_identity_option_string(&mut target.display_name, &fallback.display_name);
    }
    merge_option_string(&mut target.role, &fallback.role);
    merge_string_vec(&mut target.capabilities, &fallback.capabilities);
    merge_string_vec(&mut target.hive_groups, &fallback.hive_groups);
    merge_option_string(&mut target.lane_id, &fallback.lane_id);
    merge_option_string(&mut target.hive_group_goal, &fallback.hive_group_goal);
    merge_option_string(&mut target.authority, &fallback.authority);
    merge_option_string(&mut target.heartbeat_model, &fallback.heartbeat_model);
    merge_option_string(&mut target.project, &fallback.project);
    merge_option_string(&mut target.namespace, &fallback.namespace);
    merge_option_string(&mut target.workspace, &fallback.workspace);
    merge_option_string(&mut target.repo_root, &fallback.repo_root);
    merge_option_string(&mut target.worktree_root, &fallback.worktree_root);
    merge_option_string(&mut target.branch, &fallback.branch);
    merge_option_string(&mut target.base_branch, &fallback.base_branch);
    merge_option_string(&mut target.visibility, &fallback.visibility);
    merge_option_string(&mut target.base_url, &fallback.base_url);
    if target.base_url_healthy.is_none() {
        target.base_url_healthy = fallback.base_url_healthy;
    }
    merge_option_string(&mut target.host, &fallback.host);
    if target.pid.is_none() {
        target.pid = fallback.pid;
    }
    merge_option_string(&mut target.topic_claim, &fallback.topic_claim);
    merge_string_vec(&mut target.scope_claims, &fallback.scope_claims);
    merge_option_string(&mut target.task_id, &fallback.task_id);
    merge_option_string(&mut target.focus, &fallback.focus);
    merge_option_string(&mut target.pressure, &fallback.pressure);
    merge_option_string(&mut target.next_recovery, &fallback.next_recovery);
    merge_option_string(&mut target.next_action, &fallback.next_action);
    merge_option_string(&mut target.working, &fallback.working);
    merge_string_vec(&mut target.touches, &fallback.touches);
    merge_option_string(&mut target.relationship_state, &fallback.relationship_state);
    merge_option_string(&mut target.relationship_peer, &fallback.relationship_peer);
    merge_option_string(&mut target.relationship_reason, &fallback.relationship_reason);
    merge_option_string(&mut target.suggested_action, &fallback.suggested_action);
    merge_string_vec(&mut target.blocked_by, &fallback.blocked_by);
    merge_string_vec(&mut target.cowork_with, &fallback.cowork_with);
    merge_option_string(&mut target.handoff_target, &fallback.handoff_target);
    merge_string_vec(&mut target.offered_to, &fallback.offered_to);
    target.needs_help = target.needs_help || fallback.needs_help;
    target.needs_review = target.needs_review || fallback.needs_review;
    merge_option_string(&mut target.handoff_state, &fallback.handoff_state);
    merge_option_string(&mut target.confidence, &fallback.confidence);
    merge_option_string(&mut target.risk, &fallback.risk);
    if fallback.last_seen > target.last_seen {
        target.last_seen = fallback.last_seen;
    }
}

fn hive_session_records_can_merge(left: &HiveSessionRecord, right: &HiveSessionRecord) -> bool {
    if left.session != right.session {
        return false;
    }

    let same_scope = hive_identity_field_matches(left.project.as_deref(), right.project.as_deref())
        && hive_identity_field_matches(left.namespace.as_deref(), right.namespace.as_deref())
        && hive_identity_field_matches(left.workspace.as_deref(), right.workspace.as_deref())
        && hive_identity_field_matches(left.repo_root.as_deref(), right.repo_root.as_deref())
        && hive_identity_field_matches(left.tab_id.as_deref(), right.tab_id.as_deref());

    if !same_scope {
        return false;
    }

    if hive_identity_field_value(left.tab_id.as_deref())
        .zip(hive_identity_field_value(right.tab_id.as_deref()))
        .is_some_and(|(left, right)| left == right)
    {
        return true;
    }

    hive_identity_field_matches(
        left.worktree_root.as_deref(),
        right.worktree_root.as_deref(),
    ) && hive_identity_field_matches(left.branch.as_deref(), right.branch.as_deref())
        && hive_identity_field_matches(left.agent.as_deref(), right.agent.as_deref())
        && hive_identity_field_matches(
            left.effective_agent.as_deref(),
            right.effective_agent.as_deref(),
        )
        && hive_identity_field_matches(left.hive_system.as_deref(), right.hive_system.as_deref())
        && hive_identity_field_matches(left.hive_role.as_deref(), right.hive_role.as_deref())
        && hive_identity_field_matches(left.host.as_deref(), right.host.as_deref())
}

fn hive_identity_field_matches(left: Option<&str>, right: Option<&str>) -> bool {
    match (
        hive_identity_field_value(left),
        hive_identity_field_value(right),
    ) {
        (Some(left), Some(right)) => left == right,
        _ => true,
    }
}

fn hive_identity_field_value(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn merge_option_string(target: &mut Option<String>, fallback: &Option<String>) {
    if target.is_none() {
        *target = fallback.clone();
    }
}

fn merge_identity_option_string(target: &mut Option<String>, fallback: &Option<String>) {
    match (target.as_deref(), fallback.as_deref()) {
        (None, Some(_)) => {
            *target = fallback.clone();
        }
        (Some(current), Some(candidate)) if should_prefer_identity_value(current, candidate) => {
            *target = Some(candidate.to_string());
        }
        _ => {}
    }
}

fn should_prefer_identity_value(current: &str, candidate: &str) -> bool {
    let current = current.trim();
    let candidate = candidate.trim();
    if current.is_empty() {
        return !candidate.is_empty();
    }
    if candidate.is_empty() {
        return false;
    }
    if current.eq_ignore_ascii_case(candidate) && current != candidate {
        return true;
    }
    hive_identity_specificity_score(candidate) > hive_identity_specificity_score(current)
}

fn hive_identity_specificity_score(value: &str) -> u8 {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return 0;
    }
    let normalized = trimmed.to_ascii_lowercase();
    if normalized == "codex"
        || normalized == "claude-code"
        || normalized == "agent-shell"
        || normalized == "agent"
        || normalized == "worker"
    {
        return 1;
    }
    let has_uppercase = trimmed.chars().any(|ch| ch.is_ascii_uppercase());
    let has_digits = trimmed.chars().any(|ch| ch.is_ascii_digit());
    let has_symbol = trimmed
        .chars()
        .any(|ch| matches!(ch, '-' | '_' | '@' | '/' | '\\'));
    match (has_uppercase, has_digits || has_symbol) {
        (true, false) => 4,
        (true, true) => 3,
        (false, false) => 2,
        (false, true) => 1,
    }
}

fn worker_identity_needs_display_name(value: &str) -> bool {
    hive_identity_specificity_score(value) <= 2
}

fn merge_string_vec(target: &mut Vec<String>, fallback: &[String]) {
    for value in fallback {
        if !target.iter().any(|existing| existing == value) {
            target.push(value.clone());
        }
    }
}

pub(crate) fn is_hive_overlap_receipt(receipt: &HiveCoordinationReceiptRecord) -> bool {
    receipt.kind.contains("overlap")
        || receipt.summary.contains("confirmed hive overlap")
        || receipt.summary.contains("possible_work_overlap")
}

pub(crate) fn is_ephemeral_proof_hive_session(session: &HiveSessionRecord) -> bool {
    let session_name = session.session.trim();
    session_name == "codex-fresh"
        || session_name.starts_with("session-live-")
        || session_name.starts_with("session-dogfood-")
}

fn hive_session_live_grace(session: &HiveSessionRecord) -> chrono::TimeDelta {
    if is_ephemeral_proof_hive_session(session) {
        chrono::TimeDelta::minutes(5)
    } else {
        chrono::TimeDelta::minutes(15)
    }
}

pub(crate) fn hive_session_is_active_at(
    session: &HiveSessionRecord,
    now: chrono::DateTime<chrono::Utc>,
) -> bool {
    session.last_seen >= now - hive_session_live_grace(session)
}

pub(crate) fn refresh_hive_session_presence(
    session: &mut HiveSessionRecord,
    now: chrono::DateTime<chrono::Utc>,
) {
    let live_grace = hive_session_live_grace(session);
    let dead_grace = live_grace + live_grace;
    let age = now - session.last_seen;
    session.status = if age <= live_grace {
        "live".to_string()
    } else if age <= dead_grace {
        "stale".to_string()
    } else {
        "dead".to_string()
    };
}

pub(crate) fn is_low_signal_hive_board_session(
    session: &HiveSessionRecord,
    tasks: &[HiveTaskRecord],
) -> bool {
    let session_name = session.session.trim();
    if !session_name.starts_with("sender-") {
        return false;
    }
    let has_identity = session.hive_system.is_some()
        || session.hive_role.is_some()
        || session.role.is_some()
        || session.authority.is_some()
        || !session.capabilities.is_empty();
    if has_identity {
        return false;
    }
    let has_task_like_signal = session.task_id.is_some() || !session.scope_claims.is_empty();
    let has_active_task = tasks
        .iter()
        .any(|task| task.session.as_deref() == Some(session_name));
    !(has_task_like_signal || has_active_task)
}

pub(crate) fn is_active_hive_board_receipt(
    receipt: &HiveCoordinationReceiptRecord,
    active_session_ids: &std::collections::HashSet<String>,
) -> bool {
    receipt
        .target_session
        .as_ref()
        .is_some_and(|session| active_session_ids.contains(session))
}

pub(crate) fn hive_follow_overlap_risk(
    current: &HiveSessionRecord,
    target: &HiveSessionRecord,
) -> Option<String> {
    let current_task = normalized_hive_text(current.task_id.as_deref());
    let target_task = normalized_hive_text(target.task_id.as_deref());
    let current_topic = normalized_hive_text(current.topic_claim.as_deref());
    let target_topic = normalized_hive_text(target.topic_claim.as_deref());
    let current_scopes = hive_overlap_scopes(&current.scope_claims);
    let target_scopes = hive_overlap_scopes(&target.scope_claims);
    let shared_scopes = current_scopes
        .iter()
        .filter(|scope| target_scopes.iter().any(|other| other == *scope))
        .cloned()
        .collect::<Vec<_>>();

    if let (Some(current_task), Some(target_task)) =
        (current_task.as_deref(), target_task.as_deref())
    {
        if current_task != target_task && !shared_scopes.is_empty() {
            return Some(format!(
                "confirmed hive overlap: target session {} already owns scope(s) for task {}",
                target.session, target_task
            ));
        }
    }

    if !shared_scopes.is_empty() {
        return Some(format!(
            "possible_work_overlap touches={}",
            shared_scopes.join(",")
        ));
    }

    if let (Some(current_task), Some(target_task)) =
        (current_task.as_deref(), target_task.as_deref())
    {
        if current_task == target_task {
            return Some(format!(
                "confirmed hive overlap: target session {} already owns task {}",
                target.session, target_task
            ));
        }
    }

    if let (Some(current_topic), Some(target_topic)) =
        (current_topic.as_deref(), target_topic.as_deref())
    {
        if current_topic == target_topic {
            return Some(format!(
                "confirmed hive overlap: target session {} already owns topic {}",
                target.session,
                target.topic_claim.as_deref().unwrap_or("none")
            ));
        }
    }

    None
}

pub(crate) fn annotate_hive_relationships(bees: Vec<HiveSessionRecord>) -> Vec<HiveSessionRecord> {
    let snapshot = bees.clone();
    bees.into_iter()
        .map(|mut bee| {
            let best = snapshot
                .iter()
                .filter(|peer| peer.session != bee.session)
                .filter_map(|peer| {
                    derive_store_hive_relationship(&bee, peer).map(|(state, reason, action)| {
                        let peer_label = peer
                            .display_name
                            .clone()
                            .or_else(|| peer.worker_name.clone())
                            .or_else(|| peer.agent.clone())
                            .unwrap_or_else(|| peer.session.clone());
                        (
                            store_hive_relationship_rank(&state),
                            peer_label,
                            state,
                            reason,
                            action,
                        )
                    })
                })
                .max_by(|left, right| {
                    left.0
                        .cmp(&right.0)
                        .then_with(|| right.1.cmp(&left.1))
                        .then_with(|| right.3.cmp(&left.3))
                });

            if let Some((_, peer, state, reason, action)) = best {
                bee.relationship_state = Some(state);
                bee.relationship_peer = Some(peer);
                bee.relationship_reason = Some(reason);
                bee.suggested_action = Some(action);
            }

            bee
        })
        .collect()
}

fn store_hive_relationship_rank(state: &str) -> u8 {
    match state {
        "conflict" => 5,
        "blocked" => 4,
        "cowork_active" => 3,
        "handoff_ready" => 2,
        "near" => 1,
        _ => 0,
    }
}

fn store_hive_relationship_annotation(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if let Some(topic) = trimmed.strip_prefix("topic:") {
        let topic = topic.trim();
        return (!topic.is_empty()).then(|| topic.to_string());
    }
    if let Some(area) = trimmed.strip_prefix("area:") {
        let area = area.trim();
        return (!area.is_empty()).then(|| area.to_string());
    }
    None
}

fn derive_store_hive_relationship(
    current: &HiveSessionRecord,
    peer: &HiveSessionRecord,
) -> Option<(String, String, String)> {
    if current.blocked_by.iter().any(|value| value == &peer.session) {
        return Some((
            "blocked".to_string(),
            format!("waiting on peer {}", peer.session),
            "wait_for_peer".to_string(),
        ));
    }

    let current_cowork = current.cowork_with.iter().any(|value| value == &peer.session);
    let peer_cowork = peer.cowork_with.iter().any(|value| value == &current.session);
    if current_cowork && peer_cowork {
        return Some((
            "cowork_active".to_string(),
            "mutual cowork coordination".to_string(),
            "coordinate_live".to_string(),
        ));
    }

    if current.handoff_target.as_deref() == Some(peer.session.as_str())
        || peer.handoff_target.as_deref() == Some(current.session.as_str())
    {
        return Some((
            "handoff_ready".to_string(),
            "live handoff boundary detected".to_string(),
            "follow_handoff".to_string(),
        ));
    }

    let exact = current
        .touches
        .iter()
        .filter_map(|touch| normalized_hive_text(Some(touch.as_str())))
        .filter(|touch| {
            peer.touches
                .iter()
                .filter_map(|other| normalized_hive_text(Some(other.as_str())))
                .any(|other| other == *touch)
        })
        .collect::<Vec<_>>();
    if !exact.is_empty() {
        return Some((
            "conflict".to_string(),
            format!("shared touch {}", exact.join(",")),
            "stop_and_cowork".to_string(),
        ));
    }

    let nearby = current
        .touches
        .iter()
        .filter_map(|touch| store_hive_relationship_annotation(touch))
        .filter(|touch| {
            peer.touches
                .iter()
                .filter_map(|other| store_hive_relationship_annotation(other))
                .any(|other| other == *touch)
        })
        .collect::<Vec<_>>();
    if !nearby.is_empty() {
        return Some((
            "near".to_string(),
            format!("shared area {}", nearby.join(",")),
            "cowork".to_string(),
        ));
    }

    None
}

fn hive_overlap_scopes(scopes: &[String]) -> Vec<String> {
    scopes
        .iter()
        .filter_map(|scope| normalized_hive_text(Some(scope.as_str())))
        .filter(|scope| !is_generic_hive_overlap_scope(scope))
        .collect()
}

fn normalized_hive_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn is_generic_hive_overlap_scope(value: &str) -> bool {
    matches!(
        value,
        "project" | "workspace" | "shared" | "none" | "unknown"
    )
}

pub(crate) struct HiveSessionKeyArgs<'a> {
    pub project: Option<&'a str>,
    pub namespace: Option<&'a str>,
    pub workspace: Option<&'a str>,
    pub repo_root: Option<&'a str>,
    pub worktree_root: Option<&'a str>,
    pub branch: Option<&'a str>,
    pub agent: Option<&'a str>,
    pub effective_agent: Option<&'a str>,
    pub hive_system: Option<&'a str>,
    pub hive_role: Option<&'a str>,
    pub host: Option<&'a str>,
}

pub(crate) fn hive_session_key(session: &str, args: HiveSessionKeyArgs<'_>) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        session.trim(),
        args.project.unwrap_or("").trim(),
        args.namespace.unwrap_or("").trim(),
        args.workspace.unwrap_or("").trim(),
        args.repo_root.unwrap_or("").trim(),
        args.worktree_root.unwrap_or("").trim(),
        args.branch.unwrap_or("").trim(),
        args.agent.unwrap_or("").trim(),
        args.effective_agent.unwrap_or("").trim(),
        args.hive_system.unwrap_or("").trim(),
        args.hive_role.unwrap_or("").trim(),
        args.host.unwrap_or("").trim()
    )
}
