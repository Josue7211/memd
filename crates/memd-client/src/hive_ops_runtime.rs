use super::*;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HiveQueenResponse {
    pub(crate) queen_session: String,
    pub(crate) suggested_actions: Vec<String>,
    pub(crate) action_cards: Vec<HiveQueenActionCard>,
    pub(crate) recent_receipts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HiveQueenActionCard {
    pub(crate) action: String,
    pub(crate) priority: String,
    pub(crate) target_session: Option<String>,
    pub(crate) target_worker: Option<String>,
    pub(crate) task_id: Option<String>,
    pub(crate) scope: Option<String>,
    pub(crate) reason: String,
    pub(crate) follow_command: Option<String>,
    pub(crate) deny_command: Option<String>,
    pub(crate) reroute_command: Option<String>,
    pub(crate) retire_command: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HiveHandoffResponse {
    pub(crate) packet: HiveHandoffPacket,
    pub(crate) receipt_kind: String,
    pub(crate) receipt_summary: String,
    pub(crate) message_id: Option<String>,
    pub(crate) recommended_follow: String,
}

pub(crate) async fn run_hive_roster_command(
    args: &HiveRosterArgs,
) -> anyhow::Result<HiveRosterResponse> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let base_url = resolve_bundle_command_base_url(
        &default_base_url(),
        runtime
            .as_ref()
            .and_then(|config| config.base_url.as_deref()),
    );
    let client = MemdClient::new(&base_url)?;
    if let Some(response) = timeout_ok(client.hive_roster(&HiveRosterRequest {
        project: runtime.as_ref().and_then(|config| config.project.clone()),
        namespace: runtime.as_ref().and_then(|config| config.namespace.clone()),
        workspace: runtime.as_ref().and_then(|config| config.workspace.clone()),
    }))
    .await
    {
        return Ok(response);
    }

    let awareness = read_project_awareness(&AwarenessArgs {
        output: args.output.clone(),
        root: None,
        include_current: true,
        summary: false,
    })
    .await?;
    let visible_entries = filter_project_awareness_entries_for_hive_scope(
        &project_awareness_visible_entries(&awareness),
        runtime.as_ref(),
    );
    let queen_session = visible_entries
        .iter()
        .find(|entry| {
            matches!(
                entry.hive_role.as_deref(),
                Some("orchestrator" | "memory-control-plane")
            ) || matches!(
                entry.authority.as_deref(),
                Some("coordinator" | "canonical")
            )
        })
        .and_then(|entry| entry.session.clone());

    Ok(HiveRosterResponse {
        project: visible_entries
            .iter()
            .find_map(|entry| entry.project.clone())
            .unwrap_or_else(|| "unknown".to_string()),
        namespace: visible_entries
            .iter()
            .find_map(|entry| entry.namespace.clone())
            .unwrap_or_else(|| "default".to_string()),
        queen_session,
        bees: visible_entries
            .into_iter()
            .map(project_awareness_entry_to_hive_session)
            .collect(),
    })
}

pub(crate) async fn run_hive_follow_command(
    args: &HiveFollowArgs,
) -> anyhow::Result<HiveFollowResponse> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let current_session = runtime
        .as_ref()
        .and_then(|config| config.session.clone())
        .filter(|value| !value.trim().is_empty());
    let awareness = read_project_awareness(&AwarenessArgs {
        output: args.output.clone(),
        root: None,
        include_current: true,
        summary: false,
    })
    .await?;
    let visible_entries = project_awareness_visible_entries(&awareness);
    let target_entry = resolve_hive_follow_target(&visible_entries, args)?;
    let target_session = target_entry
        .session
        .clone()
        .context("hive follow target is missing a session id")?;

    let base_url = resolve_bundle_command_base_url(
        &default_base_url(),
        runtime
            .as_ref()
            .and_then(|config| config.base_url.as_deref())
            .or(target_entry.base_url.as_deref()),
    );
    let client = MemdClient::new(&base_url)?;
    if let Some(response) = timeout_ok(
        client.hive_follow(&HiveFollowRequest {
            session: target_session.clone(),
            current_session: current_session.clone(),
            project: target_entry
                .project
                .clone()
                .or_else(|| runtime.as_ref().and_then(|config| config.project.clone())),
            namespace: target_entry
                .namespace
                .clone()
                .or_else(|| runtime.as_ref().and_then(|config| config.namespace.clone())),
            workspace: target_entry
                .workspace
                .clone()
                .or_else(|| runtime.as_ref().and_then(|config| config.workspace.clone())),
        }),
    )
    .await
    {
        return Ok(response);
    }
    let server_reachable = timeout_ok(client.healthz()).await.is_some();
    let project = target_entry
        .project
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.project.clone()));
    let namespace = target_entry
        .namespace
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.namespace.clone()));
    let workspace = target_entry
        .workspace
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.workspace.clone()));

    let inbox = if server_reachable {
        timeout_ok(
            client.hive_coordination_inbox(&HiveCoordinationInboxRequest {
                session: target_session.clone(),
                project: project.clone(),
                namespace: namespace.clone(),
                workspace: workspace.clone(),
                limit: Some(32),
            }),
        )
        .await
        .unwrap_or(HiveCoordinationInboxResponse {
            messages: Vec::new(),
            owned_tasks: Vec::new(),
            help_tasks: Vec::new(),
            review_tasks: Vec::new(),
        })
    } else {
        HiveCoordinationInboxResponse {
            messages: Vec::new(),
            owned_tasks: Vec::new(),
            help_tasks: Vec::new(),
            review_tasks: Vec::new(),
        }
    };
    let recent_receipts = if server_reachable {
        timeout_ok(
            client.hive_coordination_receipts(&HiveCoordinationReceiptsRequest {
                session: None,
                project: project.clone(),
                namespace: namespace.clone(),
                workspace: workspace.clone(),
                limit: Some(64),
            }),
        )
        .await
        .map(|response| {
            response
                .receipts
                .into_iter()
                .filter(|receipt| {
                    receipt.actor_session == target_session
                        || receipt.target_session.as_deref() == Some(target_session.as_str())
                        || receipt.task_id.as_deref().is_some_and(|task_id| {
                            inbox.owned_tasks.iter().any(|task| task.task_id == task_id)
                        })
                })
                .take(8)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
    } else {
        Vec::new()
    };
    let overlap_risk = hive_follow_overlap_risk(
        &args.output,
        current_session.as_deref(),
        &visible_entries,
        target_entry,
    );
    let target = project_awareness_entry_to_hive_session(target_entry);
    let work_summary = target
        .topic_claim
        .clone()
        .unwrap_or_else(|| awareness_work_quickview(target_entry));
    let touch_points = if target.scope_claims.is_empty() {
        awareness_touch_points(target_entry)
    } else {
        target.scope_claims.clone()
    };
    let next_action = target.next_action.clone().or_else(|| {
        target_entry
            .next_recovery
            .as_deref()
            .and_then(simplify_awareness_work_text)
    });
    let recommended_action = if let Some(risk) = overlap_risk.as_deref() {
        if risk.starts_with("unsafe hive cowork target collision") {
            "stop_and_reroute".to_string()
        } else {
            "coordinate_now".to_string()
        }
    } else if !inbox.review_tasks.is_empty()
        || !inbox.help_tasks.is_empty()
        || !inbox.messages.is_empty()
    {
        "watch_and_coordinate".to_string()
    } else {
        "safe_to_continue".to_string()
    };

    Ok(HiveFollowResponse {
        current_session,
        target,
        work_summary,
        touch_points,
        next_action,
        messages: inbox.messages.clone(),
        owned_tasks: inbox.owned_tasks.clone(),
        help_tasks: inbox.help_tasks.clone(),
        review_tasks: inbox.review_tasks.clone(),
        recent_receipts,
        overlap_risk,
        recommended_action,
    })
}

pub(crate) async fn run_hive_follow_watch(args: &HiveFollowArgs) -> anyhow::Result<()> {
    let mut last_snapshot = None::<String>;
    loop {
        let response = run_hive_follow_command(args).await?;
        let snapshot = if args.json {
            serde_json::to_string(&response).context("serialize hive follow watch frame")?
        } else {
            render_hive_follow_summary(&response)
        };
        if last_snapshot.as_deref() != Some(snapshot.as_str()) {
            if args.json {
                println!("{snapshot}");
            } else {
                println!("{}", render_hive_follow_watch_frame(&response, Utc::now()));
                println!();
            }
            last_snapshot = Some(snapshot);
        }
        tokio::time::sleep(Duration::from_secs(args.interval_secs.max(1))).await;
    }
}

pub(crate) async fn run_hive_handoff_command(
    args: &HiveHandoffArgs,
    base_url: &str,
) -> anyhow::Result<HiveHandoffResponse> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    ensure_shared_authority_write_allowed(runtime.as_ref(), "hive handoff")?;
    let current_session = runtime
        .as_ref()
        .and_then(|config| config.session.clone())
        .filter(|value| !value.trim().is_empty())
        .context("hive handoff requires a configured bundle session")?;
    let current_agent = runtime.as_ref().and_then(|config| {
        config
            .agent
            .as_deref()
            .map(|agent| compose_agent_identity(agent, config.session.as_deref()))
    });
    let project = runtime.as_ref().and_then(|config| config.project.clone());
    let namespace = runtime.as_ref().and_then(|config| config.namespace.clone());
    let awareness = read_project_awareness(&AwarenessArgs {
        output: args.output.clone(),
        root: None,
        include_current: true,
        summary: false,
    })
    .await?;
    let visible_entries = filter_project_awareness_entries_for_hive_scope(
        &project_awareness_visible_entries(&awareness),
        runtime.as_ref(),
    );
    let current_entry = visible_entries
        .iter()
        .copied()
        .find(|entry| entry.session.as_deref() == Some(current_session.as_str()));
    let target_entry = resolve_hive_target_entry(
        &visible_entries,
        args.to_session.as_deref(),
        args.to_worker.as_deref(),
        "hive handoff",
    )?;
    let workspace = runtime
        .as_ref()
        .and_then(|config| config.workspace.clone())
        .or_else(|| target_entry.workspace.clone());
    let target_session = target_entry
        .session
        .clone()
        .context("hive handoff target is missing a session id")?;
    if target_session == current_session {
        anyhow::bail!("hive handoff target must be another bee");
    }

    let resolved_base_url = resolve_bundle_command_base_url(
        base_url,
        runtime
            .as_ref()
            .and_then(|config| config.base_url.as_deref())
            .or(target_entry.base_url.as_deref()),
    );
    let client = MemdClient::new(&resolved_base_url)?;
    let current_session_record = if let Some(entry) = current_entry {
        Some(project_awareness_entry_to_hive_session(entry))
    } else {
        timeout_ok(client.hive_sessions(&memd_schema::HiveSessionsRequest {
            session: Some(current_session.clone()),
            project: project.clone(),
            namespace: namespace.clone(),
            workspace: workspace.clone(),
            repo_root: None,
            worktree_root: None,
            branch: None,
            hive_system: None,
            hive_role: None,
            host: None,
            hive_group: None,
            active_only: Some(false),
            limit: Some(1),
        }))
        .await
        .and_then(|response| response.sessions.into_iter().next())
    };

    let packet = HiveHandoffPacket {
        from_session: current_session.clone(),
        from_worker: current_session_record
            .as_ref()
            .and_then(|record| record.worker_name.clone())
            .or_else(|| current_entry.and_then(derive_awareness_worker_name))
            .or_else(|| {
                current_agent
                    .as_deref()
                    .and_then(|value| value.split('@').next())
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
            }),
        to_session: target_session.clone(),
        to_worker: derive_awareness_worker_name(target_entry),
        task_id: args
            .task_id
            .clone()
            .or_else(|| {
                current_session_record
                    .as_ref()
                    .and_then(|record| record.task_id.clone())
            })
            .or_else(|| current_entry.and_then(|entry| entry.task_id.clone())),
        topic_claim: args
            .topic
            .clone()
            .or_else(|| {
                current_session_record
                    .as_ref()
                    .and_then(|record| record.topic_claim.clone())
            })
            .or_else(|| current_entry.and_then(|entry| entry.topic_claim.clone())),
        scope_claims: if args.scope.is_empty() {
            current_session_record
                .as_ref()
                .map(|record| record.scope_claims.clone())
                .or_else(|| current_entry.map(|entry| entry.scope_claims.clone()))
                .unwrap_or_default()
        } else {
            args.scope.clone()
        },
        next_action: args.next_action.clone().or_else(|| {
            current_session_record
                .as_ref()
                .and_then(|record| record.next_action.clone())
                .or_else(|| {
                    current_entry
                        .and_then(|entry| entry.next_recovery.as_deref())
                        .and_then(simplify_awareness_work_text)
                        .map(|value| value.to_string())
                })
        }),
        blocker: args.blocker.clone(),
        note: args.note.clone(),
        created_at: Utc::now(),
    };

    let receipt_summary = format_hive_handoff_receipt_summary(&packet);
    let message = client
        .send_hive_message(&HiveMessageSendRequest {
            kind: "handoff".to_string(),
            from_session: current_session.clone(),
            from_agent: current_agent.clone(),
            to_session: target_session.clone(),
            project: project.clone(),
            namespace: namespace.clone(),
            workspace: workspace.clone(),
            content: format_hive_handoff_message(&packet),
        })
        .await?;
    emit_coordination_receipt(
        &client,
        "queen_handoff",
        &current_session,
        current_agent,
        Some(target_session.clone()),
        packet.task_id.clone(),
        packet.scope_claims.first().cloned(),
        project,
        namespace,
        workspace,
        receipt_summary.clone(),
    )
    .await?;

    Ok(HiveHandoffResponse {
        packet,
        receipt_kind: "queen_handoff".to_string(),
        receipt_summary,
        message_id: message.messages.first().map(|entry| entry.id.clone()),
        recommended_follow: format!("memd hive follow --session {} --summary", target_session),
    })
}

pub(crate) async fn run_hive_queen_command(
    args: &HiveQueenArgs,
    base_url: &str,
) -> anyhow::Result<HiveQueenResponse> {
    let awareness = read_project_awareness(&AwarenessArgs {
        output: args.output.clone(),
        root: None,
        include_current: true,
        summary: false,
    })
    .await?;
    let visible_entries = project_awareness_visible_entries(&awareness);
    let coordination = run_coordination_command(
        &CoordinationArgs {
            output: args.output.clone(),
            view: args.view.clone(),
            changes_only: false,
            watch: false,
            interval_secs: 30,
            recover_session: args.recover_session.clone(),
            retire_session: args.retire_session.clone(),
            to_session: args.to_session.clone(),
            deny_session: args.deny_session.clone(),
            reroute_session: args.reroute_session.clone(),
            handoff_scope: args.handoff_scope.clone(),
            summary: false,
        },
        base_url,
    )
    .await?;

    let action_cards =
        coordination
            .suggestions
            .iter()
            .map(|suggestion| {
                let target_entry = suggestion.target_session.as_deref().and_then(|session| {
                    visible_entries
                        .iter()
                        .find(|entry| entry.session.as_deref() == Some(session))
                        .copied()
                });
                let target_worker = target_entry
                    .and_then(derive_awareness_worker_name)
                    .or_else(|| suggestion.target_session.clone());
                HiveQueenActionCard {
                    action: suggestion.action.clone(),
                    priority: suggestion.priority.clone(),
                    target_session: suggestion.target_session.clone(),
                    target_worker,
                    task_id: suggestion.task_id.clone(),
                    scope: suggestion.scope.clone(),
                    reason: suggestion.reason.clone(),
                    follow_command: suggestion
                        .target_session
                        .as_ref()
                        .map(|session| format!("memd hive follow --session {session} --summary")),
                    deny_command: suggestion.target_session.as_ref().map(|session| {
                        format!("memd hive queen --deny-session {session} --summary")
                    }),
                    reroute_command: suggestion.target_session.as_ref().map(|session| {
                        format!("memd hive queen --reroute-session {session} --summary")
                    }),
                    retire_command: suggestion.stale_session.as_ref().map(|session| {
                        format!("memd hive queen --retire-session {session} --summary")
                    }),
                }
            })
            .collect::<Vec<_>>();

    Ok(HiveQueenResponse {
        queen_session: coordination.current_session,
        suggested_actions: coordination
            .suggestions
            .into_iter()
            .map(|suggestion| format!("{} {}", suggestion.action, suggestion.reason))
            .collect(),
        action_cards,
        recent_receipts: coordination
            .receipts
            .iter()
            .filter(|receipt| receipt.kind.starts_with("queen_"))
            .map(|receipt| {
                format!(
                    "{} {} {}",
                    receipt.kind, receipt.actor_session, receipt.summary
                )
            })
            .collect(),
    })
}

pub(crate) async fn run_hive_board_command(
    args: &HiveArgs,
    base_url: &str,
) -> anyhow::Result<HiveBoardResponse> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let resolved_base_url = resolve_bundle_command_base_url(
        base_url,
        runtime
            .as_ref()
            .and_then(|config| config.base_url.as_deref()),
    );
    let client = MemdClient::new(&resolved_base_url)?;
    let retired_sessions = timeout_ok(client.auto_retire_hive_sessions(
        &HiveSessionAutoRetireRequest {
            project: runtime.as_ref().and_then(|config| config.project.clone()),
            namespace: runtime.as_ref().and_then(|config| config.namespace.clone()),
            workspace: runtime.as_ref().and_then(|config| config.workspace.clone()),
        },
    ))
    .await
    .map(|response| response.retired)
    .unwrap_or_default();

    if let Some(board) = timeout_ok(client.hive_board(&HiveBoardRequest {
        project: runtime.as_ref().and_then(|config| config.project.clone()),
        namespace: runtime.as_ref().and_then(|config| config.namespace.clone()),
        workspace: runtime.as_ref().and_then(|config| config.workspace.clone()),
    }))
    .await
    {
        let mut board = board;
        if !retired_sessions.is_empty() {
            board
                .stale_bees
                .retain(|session| !retired_sessions.iter().any(|retired| retired == session));
        }
        return Ok(board);
    }

    let roster = run_hive_roster_command(&HiveRosterArgs {
        output: args.output.clone(),
        json: false,
        summary: false,
    })
    .await?;
    let coordination = run_coordination_command(
        &CoordinationArgs {
            output: args.output.clone(),
            view: None,
            changes_only: false,
            watch: false,
            interval_secs: 30,
            recover_session: None,
            retire_session: None,
            to_session: None,
            deny_session: None,
            reroute_session: None,
            handoff_scope: None,
            summary: false,
        },
        base_url,
    )
    .await?;
    let review_queue = timeout_ok(client.hive_tasks(&HiveTasksRequest {
        session: None,
        project: runtime.as_ref().and_then(|config| config.project.clone()),
        namespace: runtime.as_ref().and_then(|config| config.namespace.clone()),
        workspace: runtime.as_ref().and_then(|config| config.workspace.clone()),
        active_only: Some(true),
        limit: Some(256),
    }))
    .await
    .map(|response| {
        response
            .tasks
            .into_iter()
            .filter(|task| task.review_requested || task.coordination_mode == "shared_review")
            .map(|task| {
                format!(
                    "{} -> {}",
                    task.task_id,
                    task.session.as_deref().unwrap_or("unassigned")
                )
            })
            .collect::<Vec<_>>()
    })
    .unwrap_or_default();

    let mut blocked_bees = coordination.policy_conflicts.clone();
    if let Some(fault) = coordination.lane_fault.as_ref() {
        blocked_bees.push(render_lane_fault_inline(fault));
    }
    let roster_sessions = roster
        .bees
        .iter()
        .map(|bee| bee.session.clone())
        .collect::<BTreeSet<_>>();

    Ok(HiveBoardResponse {
        queen_session: roster
            .queen_session
            .clone()
            .or_else(|| Some(coordination.current_session.clone())),
        active_bees: roster
            .bees
            .into_iter()
            .filter(|bee| bee.status == "active")
            .collect(),
        blocked_bees,
        stale_bees: coordination
            .recovery
            .stale_hives
            .iter()
            .filter_map(|entry| entry.session.clone())
            .filter(|session| roster_sessions.contains(session))
            .filter(|session| !retired_sessions.iter().any(|retired| retired == session))
            .collect(),
        review_queue,
        overlap_risks: coordination
            .policy_conflicts
            .iter()
            .filter(|value| value.contains("overlap") || value.contains("scope"))
            .cloned()
            .collect(),
        lane_faults: coordination
            .lane_receipts
            .iter()
            .map(|receipt| receipt.summary.clone())
            .collect(),
        recommended_actions: coordination
            .suggestions
            .iter()
            .map(|suggestion| format!("{} {}", suggestion.action, suggestion.reason))
            .collect(),
    })
}

fn resolve_hive_follow_target<'a>(
    visible_entries: &[&'a ProjectAwarenessEntry],
    args: &HiveFollowArgs,
) -> anyhow::Result<&'a ProjectAwarenessEntry> {
    resolve_hive_target_entry(
        visible_entries,
        args.session.as_deref(),
        args.worker.as_deref(),
        "hive follow",
    )
}

fn filter_project_awareness_entries_for_hive_scope<'a>(
    entries: &[&'a ProjectAwarenessEntry],
    runtime: Option<&BundleRuntimeConfig>,
) -> Vec<&'a ProjectAwarenessEntry> {
    let Some(runtime) = runtime else {
        return entries.to_vec();
    };
    let project = runtime
        .project
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let namespace = runtime
        .namespace
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let workspace = runtime
        .workspace
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let filtered = entries
        .iter()
        .copied()
        .filter(|entry| {
            project.is_none_or(|value| entry.project.as_deref().map(str::trim) == Some(value))
                && namespace
                    .is_none_or(|value| entry.namespace.as_deref().map(str::trim) == Some(value))
                && workspace
                    .is_none_or(|value| entry.workspace.as_deref().map(str::trim) == Some(value))
        })
        .collect::<Vec<_>>();
    if filtered.is_empty() {
        entries.to_vec()
    } else {
        filtered
    }
}

fn resolve_hive_target_entry<'a>(
    visible_entries: &[&'a ProjectAwarenessEntry],
    session: Option<&str>,
    worker: Option<&str>,
    command: &str,
) -> anyhow::Result<&'a ProjectAwarenessEntry> {
    if let Some(session) = session.map(str::trim).filter(|value| !value.is_empty()) {
        return visible_entries
            .iter()
            .copied()
            .find(|entry| entry.session.as_deref() == Some(session))
            .with_context(|| format!("{command} session not found in awareness"));
    }

    if let Some(worker) = worker.map(str::trim).filter(|value| !value.is_empty()) {
        return visible_entries
            .iter()
            .copied()
            .filter(|entry| {
                derive_awareness_worker_name(entry)
                    .is_some_and(|name| name.eq_ignore_ascii_case(worker))
            })
            .max_by_key(|entry| {
                let presence_rank = match entry.presence.as_str() {
                    "active" => 4,
                    "fresh" => 3,
                    "seen" => 2,
                    "stale" => 1,
                    _ => 0,
                };
                let task_rank = usize::from(entry.task_id.is_some());
                let scope_rank = entry.scope_claims.len();
                (presence_rank, task_rank, scope_rank)
            })
            .with_context(|| format!("{command} worker not found in awareness"));
    }

    anyhow::bail!("{command} requires --session or --worker");
}

fn format_hive_handoff_message(packet: &HiveHandoffPacket) -> String {
    let mut lines = vec!["handoff_packet".to_string()];
    lines.push(format!(
        "from={}",
        packet.from_worker.as_deref().unwrap_or("unknown")
    ));
    lines.push(format!("session={}", packet.from_session));
    if let Some(task_id) = packet.task_id.as_deref() {
        lines.push(format!("task={task_id}"));
    }
    if let Some(topic_claim) = packet.topic_claim.as_deref() {
        lines.push(format!("topic={topic_claim}"));
    }
    if !packet.scope_claims.is_empty() {
        lines.push(format!("scopes={}", packet.scope_claims.join(",")));
    }
    if let Some(next_action) = packet.next_action.as_deref() {
        lines.push(format!("next={next_action}"));
    }
    if let Some(blocker) = packet.blocker.as_deref() {
        lines.push(format!("blocker={blocker}"));
    }
    if let Some(note) = packet.note.as_deref() {
        lines.push(format!("note={note}"));
    }
    lines.join("\n")
}

fn format_hive_handoff_receipt_summary(packet: &HiveHandoffPacket) -> String {
    let mut parts = vec![format!(
        "Handoff to {} ({})",
        packet.to_worker.as_deref().unwrap_or("unknown"),
        packet.to_session
    )];
    if let Some(task_id) = packet.task_id.as_deref() {
        parts.push(format!("task={task_id}"));
    }
    if let Some(topic_claim) = packet.topic_claim.as_deref() {
        parts.push(format!("topic=\"{}\"", compact_inline(topic_claim, 72)));
    }
    if !packet.scope_claims.is_empty() {
        parts.push(format!(
            "scopes={}",
            compact_inline(&packet.scope_claims.join(","), 72)
        ));
    }
    if let Some(next_action) = packet.next_action.as_deref() {
        parts.push(format!("next=\"{}\"", compact_inline(next_action, 72)));
    }
    if let Some(blocker) = packet.blocker.as_deref() {
        parts.push(format!("blocker=\"{}\"", compact_inline(blocker, 72)));
    }
    parts.join(" ")
}
