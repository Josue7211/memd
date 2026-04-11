use super::*;

pub(crate) fn render_hive_project_summary(response: &HiveProjectResponse) -> String {
    format!(
        "hive-project action={} output={} enabled={} anchor={} joined_at={} session={}",
        response.action,
        response.output,
        if response.enabled { "true" } else { "false" },
        response.anchor.as_deref().unwrap_or("none"),
        response
            .joined_at
            .as_ref()
            .map(DateTime::<Utc>::to_rfc3339)
            .unwrap_or_else(|| "none".to_string()),
        response.live_session.as_deref().unwrap_or("none"),
    )
}

#[cfg(test)]
pub(crate) fn render_hive_wire_summary(response: &HiveWireResponse) -> String {
    let rebased = response
        .rebased_from_session
        .as_deref()
        .map(|value| format!("rebased_from={value}"))
        .unwrap_or_else(|| "rebased_from=none".to_string());
    let lane = response
        .lane_surface
        .as_ref()
        .and_then(|value| value.get("current_branch"))
        .and_then(JsonValue::as_str)
        .map(|value| format!("lane=current:{value}"))
        .unwrap_or_else(|| "lane=current:none".to_string());
    format!(
        "hive {} bundle={} agent={} bundle_session={} live_session={} session={} tab={} hive={} role={} groups={} goal=\"{}\" authority={} lane_rerouted={} lane_created={} {} heartbeat={} {}",
        response.action,
        response.output,
        response.agent,
        response.bundle_session.as_deref().unwrap_or("none"),
        response.live_session.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.tab_id.as_deref().unwrap_or("none"),
        response.hive_system.as_deref().unwrap_or("none"),
        response.hive_role.as_deref().unwrap_or("none"),
        if response.hive_groups.is_empty() {
            "none".to_string()
        } else {
            response.hive_groups.join(",")
        },
        response.hive_group_goal.as_deref().unwrap_or("none"),
        response.authority.as_deref().unwrap_or("none"),
        if response.lane_rerouted { "yes" } else { "no" },
        if response.lane_created { "yes" } else { "no" },
        lane,
        if response.heartbeat.is_some() {
            "published"
        } else {
            "skipped"
        },
        rebased,
    )
}

pub(crate) fn render_hive_roster_summary(response: &HiveRosterResponse) -> String {
    let mut lines = vec![format!(
        "hive_roster project={} namespace={} queen={}",
        response.project,
        response.namespace,
        response.queen_session.as_deref().unwrap_or("none"),
    )];
    for bee in &response.bees {
        let worker = hive_actor_label(
            bee.display_name.as_deref(),
            bee.worker_name.as_deref(),
            bee.agent.as_deref(),
            Some(bee.session.as_str()),
        );
        let lane = bee
            .lane_id
            .as_deref()
            .or(bee.branch.as_deref())
            .unwrap_or("none");
        let capabilities = if bee.capabilities.is_empty() {
            "none".to_string()
        } else {
            bee.capabilities.join(",")
        };
        lines.push(format!(
            "- {} ({}) role={} lane={} task={} caps={} status={}",
            worker,
            bee.session,
            bee.role
                .as_deref()
                .or(bee.hive_role.as_deref())
                .unwrap_or("worker"),
            lane,
            bee.task_id.as_deref().unwrap_or("none"),
            capabilities,
            bee.status,
        ));
    }
    lines.join("\n")
}

pub(crate) fn render_hive_follow_summary(response: &HiveFollowResponse) -> String {
    let worker = hive_actor_label(
        response.target.display_name.as_deref(),
        response.target.worker_name.as_deref(),
        response.target.agent.as_deref(),
        Some(response.target.session.as_str()),
    );
    let lane = response
        .target
        .lane_id
        .as_deref()
        .or(response.target.branch.as_deref())
        .unwrap_or("none");
    let mut lines = vec![
        format!(
            "hive_follow worker={} session={} role={} lane={} task={} status={}",
            worker,
            response.target.session,
            response
                .target
                .role
                .as_deref()
                .or(response.target.hive_role.as_deref())
                .unwrap_or("worker"),
            lane,
            response.target.task_id.as_deref().unwrap_or("none"),
            response.target.status,
        ),
        format!(
            "work=\"{}\" touches={} next=\"{}\" overlap_risk={} recommended_action={}",
            response.work_summary,
            if response.touch_points.is_empty() {
                "none".to_string()
            } else {
                response.touch_points.join(",")
            },
            response.next_action.as_deref().unwrap_or("none"),
            response.overlap_risk.as_deref().unwrap_or("none"),
            response.recommended_action,
        ),
    ];

    if !response.messages.is_empty() {
        lines.push(String::new());
        lines.push("## Messages".to_string());
        for message in &response.messages {
            lines.push(format!(
                "- {} from={} ack={} content=\"{}\"",
                message.kind,
                message
                    .from_agent
                    .as_deref()
                    .unwrap_or(message.from_session.as_str()),
                if message.acknowledged_at.is_some() {
                    "yes"
                } else {
                    "no"
                },
                compact_inline(&message.content, 96),
            ));
        }
    }

    if !response.owned_tasks.is_empty()
        || !response.help_tasks.is_empty()
        || !response.review_tasks.is_empty()
    {
        lines.push(String::new());
        lines.push("## Tasks".to_string());
        for task in &response.owned_tasks {
            lines.push(format!(
                "- owned {} status={} scopes={}",
                task.task_id,
                task.status,
                if task.claim_scopes.is_empty() {
                    "none".to_string()
                } else {
                    compact_inline(&task.claim_scopes.join(","), 96)
                }
            ));
        }
        for task in &response.help_tasks {
            lines.push(format!("- help {} status={}", task.task_id, task.status));
        }
        for task in &response.review_tasks {
            lines.push(format!("- review {} status={}", task.task_id, task.status));
        }
    }

    if !response.recent_receipts.is_empty() {
        lines.push(String::new());
        lines.push("## Receipts".to_string());
        for receipt in &response.recent_receipts {
            lines.push(format!(
                "- {} actor={} target={} summary=\"{}\"",
                receipt.kind,
                receipt
                    .actor_agent
                    .as_deref()
                    .unwrap_or(&receipt.actor_session),
                receipt.target_session.as_deref().unwrap_or("none"),
                compact_inline(&receipt.summary, 96),
            ));
        }
    }

    lines.join("\n")
}

pub(crate) fn render_hive_follow_watch_frame(
    response: &HiveFollowResponse,
    previous: Option<&HiveFollowResponse>,
    observed_at: DateTime<Utc>,
) -> String {
    let mut lines = vec![format!("== hive follow {} ==", observed_at.to_rfc3339())];
    match previous {
        None => {
            lines.push("state=initial".to_string());
            lines.push(render_hive_follow_summary(response));
        }
        Some(previous) => {
            lines.push(format!(
                "state=changed severity={}",
                hive_follow_watch_severity(response)
            ));
            lines.extend(render_hive_follow_watch_changes(previous, response));
            lines.push(String::new());
            lines.push(render_hive_follow_watch_snapshot(response));
        }
    }
    lines.join("\n")
}

fn render_hive_follow_watch_snapshot(response: &HiveFollowResponse) -> String {
    let summary = render_hive_follow_summary(response);
    summary.lines().take(2).collect::<Vec<_>>().join("\n")
}

fn hive_follow_watch_severity(response: &HiveFollowResponse) -> &'static str {
    let has_unacked_messages = response
        .messages
        .iter()
        .any(|message| message.acknowledged_at.is_none());
    if response
        .overlap_risk
        .as_deref()
        .is_some_and(|risk| risk.starts_with("unsafe hive cowork target collision"))
    {
        "urgent"
    } else if response.overlap_risk.is_some()
        || response.recommended_action == "coordinate_now"
        || response.recommended_action == "stop_and_reroute"
    {
        "high"
    } else if has_unacked_messages
        || !response.help_tasks.is_empty()
        || !response.review_tasks.is_empty()
        || response.recommended_action == "watch_and_coordinate"
    {
        "medium"
    } else {
        "low"
    }
}

fn render_hive_follow_watch_changes(
    previous: &HiveFollowResponse,
    current: &HiveFollowResponse,
) -> Vec<String> {
    let mut lines = Vec::new();

    if previous.work_summary != current.work_summary {
        lines.push(format!(
            "change work: \"{}\" -> \"{}\"",
            previous.work_summary, current.work_summary
        ));
    }
    if previous.next_action != current.next_action {
        lines.push(format!(
            "change next_action: \"{}\" -> \"{}\"",
            previous.next_action.as_deref().unwrap_or("none"),
            current.next_action.as_deref().unwrap_or("none")
        ));
    }
    if previous.overlap_risk != current.overlap_risk {
        match (
            previous.overlap_risk.as_deref(),
            current.overlap_risk.as_deref(),
        ) {
            (Some(previous_risk), None) => {
                lines.push(format!("risk_cleared \"{}\"", previous_risk));
            }
            (None, Some(current_risk)) => {
                lines.push(format!("risk_detected \"{}\"", current_risk));
            }
            _ => {
                lines.push(format!(
                    "change overlap_risk: \"{}\" -> \"{}\"",
                    previous.overlap_risk.as_deref().unwrap_or("none"),
                    current.overlap_risk.as_deref().unwrap_or("none")
                ));
            }
        }
    }
    if previous.recommended_action != current.recommended_action {
        lines.push(format!(
            "change recommended_action: {} -> {}",
            previous.recommended_action, current.recommended_action
        ));
    }
    if previous.touch_points != current.touch_points {
        lines.push(format!(
            "change touches: {} -> {}",
            if previous.touch_points.is_empty() {
                "none".to_string()
            } else {
                previous.touch_points.join(",")
            },
            if current.touch_points.is_empty() {
                "none".to_string()
            } else {
                current.touch_points.join(",")
            }
        ));
    }

    for message in current
        .messages
        .iter()
        .filter(|message| !previous.messages.iter().any(|prior| prior.id == message.id))
    {
        lines.push(format!(
            "new_message {} from={} ack={} content=\"{}\"",
            message.kind,
            message
                .from_agent
                .as_deref()
                .unwrap_or(message.from_session.as_str()),
            if message.acknowledged_at.is_some() {
                "yes"
            } else {
                "no"
            },
            compact_inline(&message.content, 96),
        ));
    }

    for message in current.messages.iter().filter(|message| {
        previous
            .messages
            .iter()
            .find(|prior| prior.id == message.id)
            .is_some_and(|prior| prior.acknowledged_at.is_none() && message.acknowledged_at.is_some())
    }) {
        lines.push(format!(
            "message_acked {} from={} acked_at={}",
            message.kind,
            message
                .from_agent
                .as_deref()
                .unwrap_or(message.from_session.as_str()),
            message
                .acknowledged_at
                .as_ref()
                .map(chrono::DateTime::to_rfc3339)
                .unwrap_or_else(|| "unknown".to_string())
        ));
    }

    for message in previous.messages.iter().filter(|message| {
        message.acknowledged_at.is_none()
            && !current.messages.iter().any(|current_message| current_message.id == message.id)
    }) {
        lines.push(format!(
            "message_resolved {} from={} previous_ack=no",
            message.kind,
            message
                .from_agent
                .as_deref()
                .unwrap_or(message.from_session.as_str()),
        ));
    }

    for receipt in current.recent_receipts.iter().filter(|receipt| {
        !previous
            .recent_receipts
            .iter()
            .any(|prior| prior.id == receipt.id)
    }) {
        lines.push(format!(
            "new_receipt {} actor={} target={} summary=\"{}\"",
            receipt.kind,
            receipt
                .actor_agent
                .as_deref()
                .unwrap_or(&receipt.actor_session),
            receipt.target_session.as_deref().unwrap_or("none"),
            compact_inline(&receipt.summary, 96),
        ));
    }

    if lines.is_empty() {
        lines.push("change snapshot_updated".to_string());
    }

    lines
}

pub(crate) fn render_hive_handoff_summary(response: &HiveHandoffResponse) -> String {
    let lines = vec![
        format!(
            "hive_handoff from={} ({}) to={} ({}) task={} message_id={}",
            response.packet.from_worker.as_deref().unwrap_or("unknown"),
            response.packet.from_session,
            response.packet.to_worker.as_deref().unwrap_or("unknown"),
            response.packet.to_session,
            response.packet.task_id.as_deref().unwrap_or("none"),
            response.message_id.as_deref().unwrap_or("none"),
        ),
        format!(
            "topic=\"{}\" scopes={} next=\"{}\" blocker=\"{}\" note=\"{}\" receipt_kind={} follow=\"{}\"",
            response.packet.topic_claim.as_deref().unwrap_or("none"),
            if response.packet.scope_claims.is_empty() {
                "none".to_string()
            } else {
                response.packet.scope_claims.join(",")
            },
            response.packet.next_action.as_deref().unwrap_or("none"),
            response.packet.blocker.as_deref().unwrap_or("none"),
            response.packet.note.as_deref().unwrap_or("none"),
            response.receipt_kind,
            response.recommended_follow,
        ),
        format!("receipt_summary=\"{}\"", response.receipt_summary),
    ];
    lines.join("\n")
}

pub(crate) fn render_hive_queen_summary(response: &HiveQueenResponse) -> String {
    let mut lines = vec![format!("hive_queen queen={}", response.queen_session)];

    if !response.suggested_actions.is_empty() {
        lines.push(String::new());
        lines.push("## Suggested Actions".to_string());
        for action in &response.suggested_actions {
            lines.push(format!("- {}", action));
        }
    }

    if !response.action_cards.is_empty() {
        lines.push(String::new());
        lines.push("## Action Cards".to_string());
        for card in &response.action_cards {
            lines.push(format!(
                "- action={} priority={} target={} task={} scope={} reason=\"{}\"",
                card.action,
                card.priority,
                card.target_worker
                    .as_deref()
                    .or(card.target_session.as_deref())
                    .unwrap_or("none"),
                card.task_id.as_deref().unwrap_or("none"),
                card.scope.as_deref().unwrap_or("none"),
                compact_inline(&card.reason, 96),
            ));
            if let Some(command) = card.follow_command.as_deref() {
                lines.push(format!("  follow={command}"));
            }
            if let Some(command) = card.reroute_command.as_deref() {
                lines.push(format!("  reroute={command}"));
            }
            if let Some(command) = card.deny_command.as_deref() {
                lines.push(format!("  deny={command}"));
            }
            if let Some(command) = card.retire_command.as_deref() {
                lines.push(format!("  retire={command}"));
            }
        }
    }

    if !response.recent_receipts.is_empty() {
        lines.push(String::new());
        lines.push("## Recent Receipts".to_string());
        for receipt in &response.recent_receipts {
            lines.push(format!("- {}", receipt));
        }
    }

    lines.join("\n")
}

pub(crate) fn render_lane_fault_inline(fault: &JsonValue) -> String {
    format!(
        "{} session={} branch={} worktree={}",
        fault
            .get("kind")
            .and_then(JsonValue::as_str)
            .unwrap_or("lane_fault"),
        fault
            .get("session")
            .and_then(JsonValue::as_str)
            .unwrap_or("none"),
        fault
            .get("branch")
            .and_then(JsonValue::as_str)
            .unwrap_or("none"),
        fault
            .get("worktree_root")
            .and_then(JsonValue::as_str)
            .unwrap_or("none"),
    )
}

pub(crate) fn render_hive_board_summary(response: &HiveBoardResponse) -> String {
    let mut lines = vec![format!(
        "hive_board queen={}",
        response.queen_session.as_deref().unwrap_or("none")
    )];

    lines.push(String::new());
    lines.push("## Active Bees".to_string());
    if response.active_bees.is_empty() {
        lines.push("- none".to_string());
    } else {
        for bee in &response.active_bees {
            let worker = hive_actor_label(
                bee.display_name.as_deref(),
                bee.worker_name.as_deref(),
                bee.agent.as_deref(),
                Some(bee.session.as_str()),
            );
            lines.push(format!(
                "- {} ({}) role={} lane={} task={}",
                worker,
                bee.session,
                bee.role
                    .as_deref()
                    .or(bee.hive_role.as_deref())
                    .unwrap_or("worker"),
                bee.lane_id
                    .as_deref()
                    .or(bee.branch.as_deref())
                    .unwrap_or("none"),
                bee.task_id.as_deref().unwrap_or("none"),
            ));
        }
    }

    lines.push(String::new());
    lines.push("## Blocked Bees".to_string());
    if response.blocked_bees.is_empty() {
        lines.push("- none".to_string());
    } else {
        for blocked in &response.blocked_bees {
            lines.push(format!("- {}", blocked));
        }
    }

    lines.push(String::new());
    lines.push("## Review Queue".to_string());
    if response.review_queue.is_empty() {
        lines.push("- none".to_string());
    } else {
        for item in &response.review_queue {
            lines.push(format!("- {}", item));
        }
    }

    lines.push(String::new());
    lines.push("## Overlap Risks".to_string());
    if response.overlap_risks.is_empty() {
        lines.push("- none".to_string());
    } else {
        for risk in &response.overlap_risks {
            lines.push(format!("- {}", risk));
        }
    }

    lines.push(String::new());
    lines.push("## Lane Faults".to_string());
    if response.lane_faults.is_empty() {
        lines.push("- none".to_string());
    } else {
        for fault in &response.lane_faults {
            lines.push(format!("- {}", fault));
        }
    }

    lines.push(String::new());
    lines.push("## Stale Bees".to_string());
    if response.stale_bees.is_empty() {
        lines.push("- none".to_string());
    } else {
        for stale in &response.stale_bees {
            lines.push(format!("- {}", stale));
        }
    }

    lines.push(String::new());
    lines.push("## Recommended Actions".to_string());
    if response.recommended_actions.is_empty() {
        lines.push("- none".to_string());
    } else {
        for action in &response.recommended_actions {
            lines.push(format!("- {}", action));
        }
    }

    lines.join("\n")
}

pub(crate) async fn read_bundle_hive_memory_surface(
    output: &Path,
) -> Option<BundleHiveMemorySurface> {
    let runtime = read_bundle_runtime_config(output).ok().flatten()?;
    let resolved_base_url =
        resolve_bundle_command_base_url(&default_base_url(), runtime.base_url.as_deref());
    let client = MemdClient::new(&resolved_base_url).ok()?;
    timeout_ok(client.healthz()).await?;

    let board_request = HiveBoardRequest {
        project: runtime.project.clone(),
        namespace: runtime.namespace.clone(),
        workspace: runtime.workspace.clone(),
    };
    let board = timeout_ok(client.hive_board(&board_request)).await?;
    let roster = timeout_ok(client.hive_roster(&HiveRosterRequest {
        project: runtime.project.clone(),
        namespace: runtime.namespace.clone(),
        workspace: runtime.workspace.clone(),
    }))
    .await?;
    let follow_target = runtime.session.clone().or_else(|| {
        board
            .active_bees
            .first()
            .map(|bee| bee.session.clone())
            .filter(|value| !value.trim().is_empty())
    });
    let follow = if let Some(session) = follow_target {
        timeout_ok(client.hive_follow(&HiveFollowRequest {
            session,
            current_session: runtime.session.clone(),
            project: runtime.project.clone(),
            namespace: runtime.namespace.clone(),
            workspace: runtime.workspace.clone(),
        }))
        .await
    } else {
        None
    };

    Some(BundleHiveMemorySurface {
        board,
        roster,
        follow,
    })
}

pub(crate) fn render_hive_join_summary(response: &HiveJoinResponse) -> String {
    match response {
        HiveJoinResponse::Single(response) => format!(
            "hive join bundle={} base_url={} project={} namespace={} agent={} session={} tab={} hive={} role={} groups={} goal=\"{}\" authority={} lane_rerouted={} lane_created={} heartbeat={}",
            response.output,
            response.base_url,
            response.project.as_deref().unwrap_or("none"),
            response.namespace.as_deref().unwrap_or("none"),
            response.agent.as_deref().unwrap_or("none"),
            response.session.as_deref().unwrap_or("none"),
            response.tab_id.as_deref().unwrap_or("none"),
            response.hive_system.as_deref().unwrap_or("none"),
            response.hive_role.as_deref().unwrap_or("none"),
            if response.hive_groups.is_empty() {
                "none".to_string()
            } else {
                response.hive_groups.join(",")
            },
            response.hive_group_goal.as_deref().unwrap_or("none"),
            response.authority.as_deref().unwrap_or("none"),
            if response.lane_rerouted { "yes" } else { "no" },
            if response.lane_created { "yes" } else { "no" },
            if response.heartbeat.is_some() {
                "published"
            } else {
                "skipped"
            },
        ),
        HiveJoinResponse::Batch(response) => {
            let published = response
                .joined
                .iter()
                .filter(|entry| entry.heartbeat.is_some())
                .count();
            format!(
                "hive join {} base_url={} bundles={} published={}",
                response.mode,
                response.base_url,
                response.joined.len(),
                published
            )
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum HiveJoinResponse {
    Single(HiveJoinBundleResponse),
    Batch(HiveJoinBatchResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HiveJoinBatchResponse {
    pub(crate) base_url: String,
    pub(crate) mode: String,
    pub(crate) joined: Vec<HiveJoinBundleResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HiveJoinBundleResponse {
    pub(crate) output: String,
    pub(crate) base_url: String,
    pub(crate) project: Option<String>,
    pub(crate) namespace: Option<String>,
    pub(crate) agent: Option<String>,
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
    pub(crate) heartbeat: Option<JsonValue>,
}
