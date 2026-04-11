use super::*;

pub(crate) fn coordination_snapshot_path(output: &Path) -> PathBuf {
    output.join("state").join("coordination-snapshot.json")
}

pub(crate) fn read_coordination_snapshot(
    output: &Path,
) -> anyhow::Result<Option<CoordinationSnapshotState>> {
    let path = coordination_snapshot_path(output);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let state =
        serde_json::from_str(&content).with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(state))
}

pub(crate) fn write_coordination_snapshot(
    output: &Path,
    state: &CoordinationSnapshotState,
) -> anyhow::Result<()> {
    let path = coordination_snapshot_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(state)? + "\n")
        .with_context(|| format!("write {}", path.display()))
}

pub(crate) fn build_coordination_alert_snapshot(
    response: &CoordinationResponse,
) -> CoordinationAlertSnapshot {
    CoordinationAlertSnapshot {
        message_count: response.inbox.messages.len(),
        owned_count: response.inbox.owned_tasks.len(),
        help_count: response.inbox.help_tasks.len(),
        review_count: response.inbox.review_tasks.len(),
        lane_fault_count: usize::from(response.lane_fault.is_some()),
        lane_receipt_count: response.lane_receipts.len(),
        stale_hive_count: response.recovery.stale_hives.len(),
        reclaimable_claim_count: response.recovery.reclaimable_claims.len(),
        stalled_task_count: response.recovery.stalled_tasks.len(),
        policy_conflict_count: response.policy_conflicts.len(),
        recommendation_count: response.boundary_recommendations.len(),
        suggestion_count: response.suggestions.len(),
        latest_receipt_id: response.receipts.first().map(|receipt| receipt.id.clone()),
    }
}

pub(crate) fn render_coordination_snapshot_alerts(
    previous: Option<&CoordinationAlertSnapshot>,
    current: &CoordinationAlertSnapshot,
    view: &str,
) -> Vec<String> {
    let Some(previous) = previous else {
        return vec!["alert initial coordination snapshot".to_string()];
    };

    let show_all = matches!(view, "all" | "overview");
    let mut alerts = Vec::new();

    if (show_all || view == "inbox")
        && (previous.message_count != current.message_count
            || previous.owned_count != current.owned_count)
    {
        alerts.push(format!(
            "alert inbox messages {}->{} owned {}->{}",
            previous.message_count,
            current.message_count,
            previous.owned_count,
            current.owned_count
        ));
    }
    if (show_all || view == "requests")
        && (previous.help_count != current.help_count
            || previous.review_count != current.review_count)
    {
        alerts.push(format!(
            "alert requests help {}->{} review {}->{}",
            previous.help_count, current.help_count, previous.review_count, current.review_count
        ));
    }
    if (show_all || view == "recovery")
        && (previous.stale_hive_count != current.stale_hive_count
            || previous.reclaimable_claim_count != current.reclaimable_claim_count
            || previous.stalled_task_count != current.stalled_task_count)
    {
        alerts.push(format!(
            "alert recovery stale {}->{} reclaimable {}->{} stalled {}->{}",
            previous.stale_hive_count,
            current.stale_hive_count,
            previous.reclaimable_claim_count,
            current.reclaimable_claim_count,
            previous.stalled_task_count,
            current.stalled_task_count
        ));
    }
    if (show_all || view == "lanes")
        && (previous.lane_fault_count != current.lane_fault_count
            || previous.lane_receipt_count != current.lane_receipt_count)
    {
        alerts.push(format!(
            "alert lanes faults {}->{} receipts {}->{}",
            previous.lane_fault_count,
            current.lane_fault_count,
            previous.lane_receipt_count,
            current.lane_receipt_count
        ));
    }
    if (show_all || view == "policy")
        && (previous.policy_conflict_count != current.policy_conflict_count
            || previous.recommendation_count != current.recommendation_count
            || previous.suggestion_count != current.suggestion_count)
    {
        alerts.push(format!(
            "alert policy conflicts {}->{} recommendations {}->{} suggestions {}->{}",
            previous.policy_conflict_count,
            current.policy_conflict_count,
            previous.recommendation_count,
            current.recommendation_count,
            previous.suggestion_count,
            current.suggestion_count
        ));
    }
    if (show_all || view == "history") && previous.latest_receipt_id != current.latest_receipt_id {
        alerts.push(format!(
            "alert history latest_receipt={}",
            current.latest_receipt_id.as_deref().unwrap_or("none")
        ));
    }

    alerts
}

pub(crate) fn build_coordination_change_response(
    output: &Path,
    response: &CoordinationResponse,
    view: Option<&str>,
) -> anyhow::Result<CoordinationChangeResponse> {
    let view = view.unwrap_or("all").to_string();
    let previous = read_coordination_snapshot(output)?;
    let snapshot = build_coordination_alert_snapshot(response);
    let alerts = render_coordination_snapshot_alerts(
        previous.as_ref().map(|state| &state.snapshot),
        &snapshot,
        &view,
    );
    let change = CoordinationChangeResponse {
        bundle_root: response.bundle_root.clone(),
        current_session: response.current_session.clone(),
        view: view.clone(),
        changed: !alerts.is_empty(),
        alerts,
        snapshot: snapshot.clone(),
        generated_at: Utc::now(),
        previous_generated_at: previous.as_ref().map(|state| state.generated_at),
    };
    write_coordination_snapshot(
        output,
        &CoordinationSnapshotState {
            generated_at: change.generated_at,
            view,
            snapshot,
        },
    )?;
    Ok(change)
}

pub(crate) fn render_coordination_change_summary(response: &CoordinationChangeResponse) -> String {
    let mut lines = vec![
        format!(
            "coordination_changes bundle={} session={} view={} changed={}",
            response.bundle_root, response.current_session, response.view, response.changed
        ),
        format!(
            "snapshot messages={} owned={} help={} review={} lane_faults={} lane_receipts={} stale={} reclaimable={} stalled={} conflicts={} recommendations={} suggestions={} latest_receipt={}",
            response.snapshot.message_count,
            response.snapshot.owned_count,
            response.snapshot.help_count,
            response.snapshot.review_count,
            response.snapshot.lane_fault_count,
            response.snapshot.lane_receipt_count,
            response.snapshot.stale_hive_count,
            response.snapshot.reclaimable_claim_count,
            response.snapshot.stalled_task_count,
            response.snapshot.policy_conflict_count,
            response.snapshot.recommendation_count,
            response.snapshot.suggestion_count,
            response
                .snapshot
                .latest_receipt_id
                .as_deref()
                .unwrap_or("none"),
        ),
    ];
    for alert in &response.alerts {
        lines.push(format!("- {alert}"));
    }
    lines.join("\n")
}

pub(crate) fn render_tasks_summary(response: &TasksResponse) -> String {
    let help = response
        .tasks
        .iter()
        .filter(|task| task.help_requested)
        .count();
    let review = response
        .tasks
        .iter()
        .filter(|task| task.review_requested)
        .count();
    let exclusive = response
        .tasks
        .iter()
        .filter(|task| task.coordination_mode == "exclusive_write")
        .count();
    let shared = response.tasks.len().saturating_sub(exclusive);
    let open = response
        .tasks
        .iter()
        .filter(|task| task.status != "done" && task.status != "closed")
        .count();
    let active_sessions = response
        .tasks
        .iter()
        .filter(|task| task.status != "done" && task.status != "closed")
        .filter_map(|task| task.session.clone())
        .collect::<BTreeSet<_>>()
        .len();
    let owned = response
        .tasks
        .iter()
        .filter(|task| task.session.as_deref() == response.current_session.as_deref())
        .count();
    let mut lines = vec![format!(
        "tasks bundle={} current_session={} count={} open={} help={} review={} exclusive={} shared={} active_sessions={} owned={}",
        response.bundle_root,
        response.current_session.as_deref().unwrap_or("none"),
        response.tasks.len(),
        open,
        help,
        review,
        exclusive,
        shared,
        active_sessions,
        owned
    )];
    let visible = response.tasks.iter().take(12);
    for task in visible {
        lines.push(format!(
            "- {} [{}:{}] owner={} scopes={} help={} review={} | {}",
            task.task_id,
            task.status,
            task.coordination_mode,
            task.effective_agent
                .as_deref()
                .or(task.session.as_deref())
                .unwrap_or("none"),
            if task.claim_scopes.is_empty() {
                "none".to_string()
            } else {
                task.claim_scopes.join(",")
            },
            if task.help_requested { "yes" } else { "no" },
            if task.review_requested { "yes" } else { "no" },
            compact_inline(&task.title, 80)
        ));
    }
    if response.tasks.len() > 12 {
        lines.push(format!(
            "- ... {} more task(s) hidden",
            response.tasks.len() - 12
        ));
    }
    lines.join("\n")
}

pub(crate) fn build_task_view_counts(
    tasks: &[HiveTaskRecord],
    current_session: Option<&str>,
) -> serde_json::Value {
    let open = tasks
        .iter()
        .filter(|task| task.status != "done" && task.status != "closed")
        .count();
    let help = tasks.iter().filter(|task| task.help_requested).count();
    let review = tasks.iter().filter(|task| task.review_requested).count();
    let exclusive = tasks
        .iter()
        .filter(|task| task.coordination_mode == "exclusive_write")
        .count();
    let shared = tasks.len().saturating_sub(exclusive);
    let owned = tasks
        .iter()
        .filter(|task| task.session.as_deref() == current_session)
        .count();
    serde_json::json!({
        "all": tasks.len(),
        "open": open,
        "help": help,
        "review": review,
        "exclusive": exclusive,
        "shared": shared,
        "owned": owned,
    })
}

pub(crate) fn render_coordination_summary(
    response: &CoordinationResponse,
    view: Option<&str>,
) -> String {
    let view = view.unwrap_or("all");
    let exclusive = response
        .inbox
        .owned_tasks
        .iter()
        .filter(|task| task.coordination_mode == "exclusive_write")
        .count();
    let shared = response.inbox.owned_tasks.len().saturating_sub(exclusive);
    let mut lines = vec![
        format!(
            "coordination bundle={} session={}",
            response.bundle_root, response.current_session,
        ),
        format!(
            "pressure messages={} owned={} help={} review={} exclusive={} shared={}",
            response.inbox.messages.len(),
            response.inbox.owned_tasks.len(),
            response.inbox.help_tasks.len(),
            response.inbox.review_tasks.len(),
            exclusive,
            shared,
        ),
        format!(
            "recovery stale_hives={} reclaimable_claims={} stalled_tasks={} retireable_sessions={}",
            response.recovery.stale_hives.len(),
            response.recovery.reclaimable_claims.len(),
            response.recovery.stalled_tasks.len(),
            response.recovery.retireable_sessions.len(),
        ),
        format!(
            "policy conflicts={} recommendations={} suggestions={} receipts={} lane_fault={} lane_receipts={}",
            response.policy_conflicts.len(),
            response.boundary_recommendations.len(),
            response.suggestions.len(),
            response.receipts.len(),
            if response.lane_fault.is_some() {
                "yes"
            } else {
                "no"
            },
            response.lane_receipts.len(),
        ),
    ];
    if matches!(view, "all" | "overview" | "inbox") {
        lines.push("".to_string());
        lines.push("## Inbox".to_string());
    }
    append_coordination_sections(&mut lines, response, view);
    lines.join("\n")
}

pub(crate) fn append_coordination_sections(
    lines: &mut Vec<String>,
    response: &CoordinationResponse,
    view: &str,
) {
    let show_all = matches!(view, "all" | "overview");
    let show_inbox = show_all || view == "inbox";
    let show_requests = show_all || view == "requests";
    let show_recovery = show_all || view == "recovery";
    let show_active = show_all || view == "active";
    let show_lanes = show_all || view == "lanes";
    let show_policy = show_all || view == "policy";
    let show_suggestions = show_all || view == "suggestions";
    let show_history = show_all || view == "history";

    if show_inbox {
        for message in response.inbox.messages.iter().take(6) {
            lines.push(format!(
                "- msg {} [{}] {}",
                &message.id[..8.min(message.id.len())],
                message.kind,
                compact_inline(&message.content, 90)
            ));
        }
        for task in response.inbox.owned_tasks.iter().take(6) {
            lines.push(format!(
                "- own {} [{}] {}",
                task.task_id,
                task.status,
                compact_inline(&task.title, 90)
            ));
        }
    }
    if show_active && !response.active_hives.is_empty() {
        lines.push("".to_string());
        lines.push("## Active Hive".to_string());
        for hive in response.active_hives.iter().take(8) {
            lines.push(format!(
                "- session={} task={} work=\"{}\" touches={} branch={} agent={}",
                hive.session.as_deref().unwrap_or("none"),
                hive.task_id.as_deref().unwrap_or("none"),
                compact_inline(
                    hive.topic_claim
                        .as_deref()
                        .unwrap_or_else(|| hive.focus.as_deref().unwrap_or("none")),
                    72
                ),
                if hive.scope_claims.is_empty() {
                    "none".to_string()
                } else {
                    compact_inline(&hive.scope_claims.join(","), 72)
                },
                hive.branch.as_deref().unwrap_or("none"),
                hive.effective_agent
                    .as_deref()
                    .or(hive.agent.as_deref())
                    .unwrap_or("none"),
            ));
        }
    }
    if show_requests
        && (!response.inbox.help_tasks.is_empty() || !response.inbox.review_tasks.is_empty())
    {
        lines.push("".to_string());
        lines.push("## Requests".to_string());
        for task in response.inbox.help_tasks.iter().take(6) {
            lines.push(format!(
                "- help {} [{}] owner={}",
                task.task_id,
                task.status,
                task.effective_agent
                    .as_deref()
                    .or(task.session.as_deref())
                    .unwrap_or("none")
            ));
        }
        for task in response.inbox.review_tasks.iter().take(6) {
            lines.push(format!(
                "- review {} [{}] owner={}",
                task.task_id,
                task.status,
                task.effective_agent
                    .as_deref()
                    .or(task.session.as_deref())
                    .unwrap_or("none")
            ));
        }
    }
    if show_recovery
        && (!response.recovery.stale_hives.is_empty()
            || !response.recovery.reclaimable_claims.is_empty()
            || !response.recovery.stalled_tasks.is_empty())
    {
        lines.push("".to_string());
        lines.push("## Recovery".to_string());
        for entry in response.recovery.stale_hives.iter().take(6) {
            lines.push(format!(
                "- stale session={} agent={} presence={} focus=\"{}\"",
                entry.session.as_deref().unwrap_or("none"),
                entry
                    .effective_agent
                    .as_deref()
                    .or(entry.agent.as_deref())
                    .unwrap_or("none"),
                entry.presence,
                compact_inline(entry.focus.as_deref().unwrap_or("none"), 72),
            ));
        }
        for claim in response.recovery.reclaimable_claims.iter().take(6) {
            lines.push(format!(
                "- reclaimable claim {} owner={}",
                claim.scope,
                claim
                    .effective_agent
                    .as_deref()
                    .or(claim.session.as_deref())
                    .unwrap_or("none")
            ));
        }
        for task in response.recovery.stalled_tasks.iter().take(6) {
            lines.push(format!(
                "- stalled task {} [{}] owner={}",
                task.task_id,
                task.status,
                task.effective_agent
                    .as_deref()
                    .or(task.session.as_deref())
                    .unwrap_or("none")
            ));
        }
    }
    if show_lanes && (response.lane_fault.is_some() || !response.lane_receipts.is_empty()) {
        lines.push("".to_string());
        lines.push("## Lanes".to_string());
        if let Some(lane_fault) = response.lane_fault.as_ref() {
            lines.push(format!(
                "- fault {} session={} branch={} worktree={}",
                lane_fault
                    .get("kind")
                    .and_then(JsonValue::as_str)
                    .unwrap_or("none"),
                lane_fault
                    .get("session")
                    .and_then(JsonValue::as_str)
                    .unwrap_or("none"),
                lane_fault
                    .get("branch")
                    .and_then(JsonValue::as_str)
                    .unwrap_or("none"),
                lane_fault
                    .get("worktree_root")
                    .and_then(JsonValue::as_str)
                    .unwrap_or("none"),
            ));
        }
        for receipt in response.lane_receipts.iter().take(6) {
            lines.push(format!(
                "- lane [{}] {}",
                receipt.kind,
                compact_inline(&receipt.summary, 96)
            ));
        }
    }
    if show_policy
        && (!response.policy_conflicts.is_empty() || !response.boundary_recommendations.is_empty())
    {
        lines.push("".to_string());
        lines.push("## Policy".to_string());
        for conflict in response.policy_conflicts.iter().take(6) {
            lines.push(format!("- policy {}", compact_inline(conflict, 96)));
        }
        for recommendation in response.boundary_recommendations.iter().take(6) {
            lines.push(format!(
                "- recommend {}",
                compact_inline(recommendation, 96)
            ));
        }
    }
    if show_suggestions && !response.suggestions.is_empty() {
        lines.push("".to_string());
        lines.push("## Suggestions".to_string());
        for suggestion in response.suggestions.iter().take(6) {
            lines.push(format!(
                "- {} [{}] {}",
                suggestion.priority,
                suggestion.action,
                compact_inline(&suggestion.reason, 110),
            ));
        }
    }
    if show_history && !response.receipts.is_empty() {
        lines.push("".to_string());
        lines.push("## History".to_string());
        for receipt in response.receipts.iter().take(8) {
            lines.push(format!(
                "- receipt {} [{}] {}",
                &receipt.id[..8.min(receipt.id.len())],
                receipt.kind,
                compact_inline(&receipt.summary, 96)
            ));
        }
    }
}

pub(crate) fn render_coordination_alerts(
    previous: Option<&CoordinationResponse>,
    current: &CoordinationResponse,
    view: Option<&str>,
) -> Vec<String> {
    let Some(previous) = previous else {
        return vec!["alert initial coordination snapshot".to_string()];
    };

    let view = view.unwrap_or("all");
    let show_all = matches!(view, "all" | "overview");
    let show_suggestions = matches!(view, "all" | "overview" | "suggestions");
    let mut alerts = Vec::new();

    if (show_all || view == "inbox")
        && (previous.inbox.messages.len() != current.inbox.messages.len()
            || previous.inbox.owned_tasks.len() != current.inbox.owned_tasks.len())
    {
        alerts.push(format!(
            "alert inbox messages {}->{} owned {}->{}",
            previous.inbox.messages.len(),
            current.inbox.messages.len(),
            previous.inbox.owned_tasks.len(),
            current.inbox.owned_tasks.len()
        ));
    }
    if (show_all || view == "requests")
        && (previous.inbox.help_tasks.len() != current.inbox.help_tasks.len()
            || previous.inbox.review_tasks.len() != current.inbox.review_tasks.len())
    {
        alerts.push(format!(
            "alert requests help {}->{} review {}->{}",
            previous.inbox.help_tasks.len(),
            current.inbox.help_tasks.len(),
            previous.inbox.review_tasks.len(),
            current.inbox.review_tasks.len()
        ));
    }
    if (show_all || view == "recovery")
        && (previous.recovery.stale_hives.len() != current.recovery.stale_hives.len()
            || previous.recovery.reclaimable_claims.len()
                != current.recovery.reclaimable_claims.len()
            || previous.recovery.stalled_tasks.len() != current.recovery.stalled_tasks.len())
    {
        alerts.push(format!(
            "alert recovery stale {}->{} reclaimable {}->{} stalled {}->{}",
            previous.recovery.stale_hives.len(),
            current.recovery.stale_hives.len(),
            previous.recovery.reclaimable_claims.len(),
            current.recovery.reclaimable_claims.len(),
            previous.recovery.stalled_tasks.len(),
            current.recovery.stalled_tasks.len()
        ));
    }
    if (show_all || view == "policy")
        && (previous.policy_conflicts.len() != current.policy_conflicts.len()
            || previous.boundary_recommendations.len() != current.boundary_recommendations.len()
            || previous.suggestions.len() != current.suggestions.len())
    {
        alerts.push(format!(
            "alert policy conflicts {}->{} recommendations {}->{} suggestions {}->{}",
            previous.policy_conflicts.len(),
            current.policy_conflicts.len(),
            previous.boundary_recommendations.len(),
            current.boundary_recommendations.len(),
            previous.suggestions.len(),
            current.suggestions.len()
        ));
    }
    if show_suggestions && !show_all {
        let prev_suggestions = previous.suggestions.len();
        let curr_suggestions = current.suggestions.len();
        if prev_suggestions != curr_suggestions {
            alerts.push(format!(
                "alert suggestions {}->{}",
                prev_suggestions, curr_suggestions
            ));
        }
    }
    if (show_all || view == "history")
        && previous.receipts.first().map(|receipt| receipt.id.as_str())
            != current.receipts.first().map(|receipt| receipt.id.as_str())
    {
        alerts.push(format!(
            "alert history latest_receipt={}",
            current
                .receipts
                .first()
                .map(|receipt| receipt.id.as_str())
                .unwrap_or("none")
        ));
    }

    alerts
}

pub(crate) fn suggest_coordination_actions(
    inbox: &HiveCoordinationInboxResponse,
    stale_sessions: &[&str],
    active_hives: &[ProjectAwarenessEntry],
    claims: &[SessionClaim],
    tasks: &[HiveTaskRecord],
    current_session: &str,
    policy_conflicts: &[String],
    lane_fault: Option<&JsonValue>,
    lane_receipts: &[HiveCoordinationReceiptRecord],
) -> Vec<CoordinationSuggestion> {
    let mut suggestions = Vec::new();
    let mut emitted = Vec::<(
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    )>::new();

    let is_stale_session = |session: &str, stale_sessions: &[&str]| {
        stale_sessions.iter().any(|entry| entry == &session)
    };
    let has_scope_conflict = |task_id: &str,
                              scope: &str,
                              list: &Vec<(
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    )>| {
        list.iter()
            .any(|item| item.0 == "assign_scope" && item.1 == task_id && item.2 == scope)
    };
    let push_unique = |suggestion: CoordinationSuggestion,
                       seen: &mut Vec<(
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    )>,
                       out: &mut Vec<CoordinationSuggestion>| {
        let key = (
            suggestion.action.clone(),
            suggestion.task_id.clone().unwrap_or_else(String::new),
            suggestion.scope.clone().unwrap_or_else(String::new),
            suggestion.target_session.clone(),
            suggestion.message_id.clone(),
            suggestion.stale_session.clone(),
        );
        if !seen.contains(&key) {
            seen.push(key);
            out.push(suggestion);
        }
    };

    if !inbox.messages.is_empty() {
        for message in inbox.messages.iter().take(3) {
            let suggestion = CoordinationSuggestion {
                action: "ack_message".to_string(),
                priority: "high".to_string(),
                target_session: None,
                task_id: None,
                scope: None,
                message_id: Some(message.id.clone()),
                reason: format!(
                    "Acknowledge {} message from {}.",
                    message.kind,
                    message
                        .from_agent
                        .clone()
                        .unwrap_or_else(|| message.from_session.clone())
                ),
                stale_session: None,
            };
            push_unique(suggestion, &mut emitted, &mut suggestions);
        }
    }

    let current_touch_scopes = claims
        .iter()
        .filter_map(|claim| normalize_hive_touch(&claim.scope))
        .chain(
            tasks
                .iter()
                .flat_map(|task| task.claim_scopes.iter())
                .filter_map(|scope| normalize_hive_touch(scope)),
        )
        .collect::<Vec<_>>();

    for hive in active_hives.iter().filter(|hive| {
        hive.session
            .as_deref()
            .is_some_and(|session| session != current_session)
    }) {
        let Some(target_session) = hive.session.clone() else {
            continue;
        };
        let shared = hive
            .scope_claims
            .iter()
            .filter_map(|touch| normalize_hive_touch(touch))
            .any(|touch| current_touch_scopes.iter().any(|scope| scope == &touch));
        if !shared {
            continue;
        }
        push_unique(
            CoordinationSuggestion {
                action: "request_cowork".to_string(),
                priority: "low".to_string(),
                target_session: Some(target_session.clone()),
                task_id: hive.task_id.clone(),
                scope: hive.scope_claims.first().cloned(),
                message_id: None,
                reason: format!(
                    "Peer {} is near overlapping touch scopes; request a live cowork handshake before overlap widens.",
                    hive.effective_agent
                        .as_deref()
                        .or(hive.agent.as_deref())
                        .unwrap_or(target_session.as_str())
                ),
                stale_session: None,
            },
            &mut emitted,
            &mut suggestions,
        );
    }

    if let Some(lane_fault) = lane_fault {
        let target_session = lane_fault
            .get("session")
            .and_then(JsonValue::as_str)
            .map(str::to_string);
        if let Some(target_session) = target_session.clone() {
            push_unique(
                CoordinationSuggestion {
                    action: "deny_lane".to_string(),
                    priority: "high".to_string(),
                    target_session: Some(target_session.clone()),
                    task_id: None,
                    scope: None,
                    message_id: None,
                    reason: format!(
                        "Queen should deny unsafe lane overlap with {}.",
                        lane_fault
                            .get("kind")
                            .and_then(JsonValue::as_str)
                            .unwrap_or("unknown")
                    ),
                    stale_session: None,
                },
                &mut emitted,
                &mut suggestions,
            );
            push_unique(
                CoordinationSuggestion {
                    action: "reroute_lane".to_string(),
                    priority: "high".to_string(),
                    target_session: Some(target_session),
                    task_id: None,
                    scope: None,
                    message_id: None,
                    reason:
                        "Queen should reroute the conflicting session onto a fresh worker lane."
                            .to_string(),
                    stale_session: None,
                },
                &mut emitted,
                &mut suggestions,
            );
        }
    }

    if lane_receipts
        .iter()
        .any(|receipt| receipt.kind == "lane_fault")
    {
        for task in inbox.owned_tasks.iter().take(2) {
            if let Some(scope) = task.claim_scopes.first() {
                push_unique(
                    CoordinationSuggestion {
                        action: "handoff_scope".to_string(),
                        priority: "medium".to_string(),
                        target_session: None,
                        task_id: Some(task.task_id.clone()),
                        scope: Some(scope.clone()),
                        message_id: None,
                        reason: format!(
                            "Queen should resolve lane conflict by explicit handoff for scope {}.",
                            scope
                        ),
                        stale_session: None,
                    },
                    &mut emitted,
                    &mut suggestions,
                );
            }
        }
    }

    if !stale_sessions.is_empty() {
        for stale_session in stale_sessions.iter().copied() {
            let reclaimable_claims = claims
                .iter()
                .filter(|claim| claim.session.as_deref() == Some(stale_session))
                .count();
            let stalled_tasks = tasks
                .iter()
                .filter(|task| task.session.as_deref() == Some(stale_session))
                .count();
            if reclaimable_claims == 0 && stalled_tasks == 0 {
                let suggestion = CoordinationSuggestion {
                    action: "retire_session".to_string(),
                    priority: "medium".to_string(),
                    target_session: None,
                    task_id: None,
                    scope: None,
                    message_id: None,
                    reason: format!(
                        "Retire stale session {} because it holds no active claims or stalled tasks.",
                        stale_session
                    ),
                    stale_session: Some(stale_session.to_string()),
                };
                push_unique(suggestion, &mut emitted, &mut suggestions);
                continue;
            }
            let suggestion = CoordinationSuggestion {
                action: "recover_session".to_string(),
                priority: "high".to_string(),
                target_session: None,
                task_id: None,
                scope: None,
                message_id: None,
                reason: format!(
                    "Recover {} claim(s) and {} stalled task(s) from stale session {}.",
                    reclaimable_claims, stalled_tasks, stale_session
                ),
                stale_session: Some(stale_session.to_string()),
            };
            push_unique(suggestion, &mut emitted, &mut suggestions);
        }
    }

    for task in tasks
        .iter()
        .filter(|task| task.coordination_mode == "exclusive_write")
    {
        for scope in &task.claim_scopes {
            let Some(task_owner) = task.session.as_deref() else {
                continue;
            };
            let Some(claim) = claims.iter().find(|claim| {
                claim.scope.as_str() == scope.as_str()
                    && claim
                        .session
                        .as_deref()
                        .is_some_and(|claim_owner| !is_stale_session(claim_owner, stale_sessions))
            }) else {
                continue;
            };
            let Some(claim_owner) = claim.session.as_deref() else {
                continue;
            };
            if claim_owner == task_owner {
                continue;
            }
            if has_scope_conflict(&task.task_id, scope, &emitted) {
                continue;
            }
            let suggestion = CoordinationSuggestion {
                action: "assign_scope".to_string(),
                priority: "medium".to_string(),
                target_session: Some(task_owner.to_string()),
                task_id: Some(task.task_id.clone()),
                scope: Some(scope.clone()),
                message_id: None,
                reason: format!(
                    "Resolve exclusivity conflict for {scope} by moving it to task owner {}.",
                    task_owner
                ),
                stale_session: None,
            };
            push_unique(suggestion, &mut emitted, &mut suggestions);
        }
    }

    if !policy_conflicts.is_empty() && suggestions.len() < 6 && !tasks.is_empty() {
        let Some(hive_session) =
            select_coordination_helper_hive(active_hives, tasks, policy_conflicts, current_session)
        else {
            return suggestions;
        };
        for task in tasks
            .iter()
            .filter(|task| task.session.as_deref() == Some(current_session))
            .take(2)
        {
            let action = if task.coordination_mode == "shared_review" {
                "request_review"
            } else if task.coordination_mode == "help_only" {
                "request_help"
            } else {
                continue;
            };
            if action == "request_review" && task.review_requested {
                continue;
            }
            if action == "request_help" && task.help_requested {
                continue;
            }
            let suggestion = CoordinationSuggestion {
                action: action.to_string(),
                priority: "low".to_string(),
                target_session: hive_session.session.clone(),
                task_id: Some(task.task_id.clone()),
                scope: None,
                message_id: None,
                reason: format!(
                    "Ask {} for collaboration support on task {} before heavy overlap grows.",
                    hive_session.session.as_deref().unwrap_or("none"),
                    task.task_id
                ),
                stale_session: None,
            };
            push_unique(suggestion, &mut emitted, &mut suggestions);
        }
    }

    suggestions
}
