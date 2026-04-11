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
    pub(crate) cowork_command: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HiveHandoffResponse {
    pub(crate) packet: HiveHandoffPacket,
    pub(crate) receipt_kind: String,
    pub(crate) receipt_summary: String,
    pub(crate) message_id: Option<String>,
    pub(crate) recommended_follow: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HiveCoworkResponse {
    pub(crate) packet: HiveCoworkPacket,
    pub(crate) receipt_kind: String,
    pub(crate) receipt_summary: String,
    pub(crate) message_id: Option<String>,
    pub(crate) recommended_follow: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HiveCoworkPacket {
    pub(crate) action: String,
    pub(crate) from_session: String,
    pub(crate) from_worker: Option<String>,
    pub(crate) to_session: String,
    pub(crate) to_worker: Option<String>,
    pub(crate) task_id: Option<String>,
    pub(crate) scope_claims: Vec<String>,
    pub(crate) reason: Option<String>,
    pub(crate) note: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
}

pub(crate) async fn run_hive_roster_command(
    args: &HiveRosterArgs,
) -> anyhow::Result<HiveRosterResponse> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let awareness = read_project_awareness(&AwarenessArgs {
        output: args.output.clone(),
        root: None,
        include_current: true,
        summary: false,
    })
    .await?;
    let awareness_response = build_hive_roster_from_awareness(&awareness, runtime.as_ref());
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
        let response_sessions = response
            .bees
            .iter()
            .map(|bee| bee.session.as_str())
            .collect::<BTreeSet<_>>();
        let awareness_has_extra = awareness_response
            .bees
            .iter()
            .any(|bee| !response_sessions.contains(bee.session.as_str()));
        if awareness_has_extra {
            return Ok(awareness_response);
        }
        return Ok(response);
    }
    Ok(awareness_response)
}

fn build_hive_roster_from_awareness(
    awareness: &ProjectAwarenessResponse,
    runtime: Option<&BundleRuntimeConfig>,
) -> HiveRosterResponse {
    let visible_entries = filter_project_awareness_entries_for_hive_scope(
        &project_awareness_visible_entries(awareness),
        runtime,
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

    HiveRosterResponse {
        project: visible_entries
            .iter()
            .find_map(|entry| entry.project.clone())
            .unwrap_or_else(|| "unknown".to_string()),
        namespace: visible_entries
            .iter()
            .find_map(|entry| entry.namespace.clone())
            .unwrap_or_else(|| "default".to_string()),
        queen_session,
        bees: annotate_hive_relationships(
            visible_entries
                .into_iter()
                .map(project_awareness_entry_to_hive_session)
                .collect(),
        ),
    }
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
        .working
        .clone()
        .or_else(|| target.topic_claim.clone())
        .unwrap_or_else(|| awareness_work_quickview(target_entry));
    let touch_points = if target.touches.is_empty() {
        if target.scope_claims.is_empty() {
            awareness_touch_points(target_entry)
        } else {
            target.scope_claims.clone()
        }
    } else {
        target.touches.clone()
    };
    let next_action = target.next_action.clone().or_else(|| {
        target_entry
            .next_recovery
            .as_deref()
            .and_then(simplify_awareness_work_text)
    });
    let recommended_action = if let Some(action) = target.suggested_action.clone() {
        action
    } else if let Some(risk) = overlap_risk.as_deref() {
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
    let mut last_response = None::<HiveFollowResponse>;
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
                println!(
                    "{}",
                    render_hive_follow_watch_frame(&response, last_response.as_ref(), Utc::now())
                );
                println!();
            }
            last_snapshot = Some(snapshot);
            last_response = Some(response.clone());
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

pub(crate) async fn run_hive_cowork_command(
    args: &HiveCoworkArgs,
    base_url: &str,
    action: &str,
) -> anyhow::Result<HiveCoworkResponse> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    ensure_shared_authority_write_allowed(runtime.as_ref(), "hive cowork")?;
    let current_session = runtime
        .as_ref()
        .and_then(|config| config.session.clone())
        .filter(|value| !value.trim().is_empty())
        .context("hive cowork requires a configured bundle session")?;
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
        "hive cowork",
    )?;
    let workspace = runtime
        .as_ref()
        .and_then(|config| config.workspace.clone())
        .or_else(|| target_entry.workspace.clone());
    let target_session = target_entry
        .session
        .clone()
        .context("hive cowork target is missing a session id")?;
    if target_session == current_session {
        anyhow::bail!("hive cowork target must be another bee");
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

    let packet = HiveCoworkPacket {
        action: action.to_string(),
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
        scope_claims: if args.scope.is_empty() {
            current_session_record
                .as_ref()
                .map(|record| record.scope_claims.clone())
                .or_else(|| current_entry.map(|entry| entry.scope_claims.clone()))
                .unwrap_or_default()
        } else {
            args.scope.clone()
        },
        reason: args.reason.clone(),
        note: args.note.clone(),
        created_at: Utc::now(),
    };

    let kind = format!("cowork_{action}");
    let receipt_summary = format_hive_cowork_receipt_summary(&packet);
    let message = client
        .send_hive_message(&HiveMessageSendRequest {
            kind: kind.clone(),
            from_session: current_session.clone(),
            from_agent: current_agent.clone(),
            to_session: target_session.clone(),
            project: project.clone(),
            namespace: namespace.clone(),
            workspace: workspace.clone(),
            content: format_hive_cowork_message(&packet),
        })
        .await?;
    emit_coordination_receipt(
        &client,
        &kind,
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

    Ok(HiveCoworkResponse {
        packet,
        receipt_kind: kind,
        receipt_summary,
        message_id: message.messages.first().map(|entry| entry.id.clone()),
        recommended_follow: format!("memd hive follow --session {} --summary", target_session),
    })
}

async fn dispatch_queen_cowork_actions(
    args: &HiveQueenArgs,
    base_url: &str,
    suggestions: &[CoordinationSuggestion],
) -> anyhow::Result<Vec<String>> {
    let mut receipts = Vec::new();
    let mut seen_eligible = false;
    let mut dispatched_any = false;

    for suggestion in suggestions
        .iter()
        .filter(|suggestion| matches!(suggestion.action.as_str(), "request_cowork" | "ack_cowork"))
    {
        let Some(target_session) = suggestion.target_session.as_deref() else {
            continue;
        };
        seen_eligible = true;
        let cowork_args = HiveCoworkArgs {
            output: args.output.clone(),
            to_session: Some(target_session.to_string()),
            to_worker: None,
            task_id: suggestion.task_id.clone(),
            scope: suggestion.scope.clone().into_iter().collect(),
            reason: Some(suggestion.reason.clone()),
            note: None,
            json: false,
            summary: false,
        };
        let action = if suggestion.action == "ack_cowork" {
            "ack"
        } else {
            "request"
        };
        let Ok(response) = run_hive_cowork_command(&cowork_args, base_url, action).await else {
            continue;
        };
        dispatched_any = true;
        receipts.push(response.receipt_summary);
    }

    if !seen_eligible {
        anyhow::bail!(
            "cowork auto-send found no eligible request_cowork or ack_cowork suggestions"
        );
    }
    if !dispatched_any {
        anyhow::bail!("cowork auto-send did not dispatch any packets");
    }

    Ok(receipts)
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
                    cowork_command: format_hive_queen_cowork_command(suggestion),
                }
            })
            .collect::<Vec<_>>();

    let mut recent_receipts = coordination
        .receipts
        .iter()
        .filter(|receipt| receipt.kind.starts_with("queen_"))
        .map(|receipt| {
            format!(
                "{} {} {}",
                receipt.kind, receipt.actor_session, receipt.summary
            )
        })
        .collect::<Vec<_>>();
    if args.cowork_auto_send {
        let cowork_receipts =
            dispatch_queen_cowork_actions(args, base_url, &coordination.suggestions).await?;
        recent_receipts.extend(cowork_receipts);
    }

    Ok(HiveQueenResponse {
        queen_session: coordination.current_session,
        suggested_actions: coordination
            .suggestions
            .into_iter()
            .map(|suggestion| format!("{} {}", suggestion.action, suggestion.reason))
            .collect(),
        action_cards,
        recent_receipts,
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

fn format_hive_cowork_message(packet: &HiveCoworkPacket) -> String {
    let mut lines = vec!["cowork_packet".to_string()];
    lines.push(format!("action={}", packet.action));
    lines.push(format!(
        "from={}",
        packet.from_worker.as_deref().unwrap_or("unknown")
    ));
    lines.push(format!("session={}", packet.from_session));
    lines.push(format!(
        "to={}",
        packet.to_worker.as_deref().unwrap_or("unknown")
    ));
    lines.push(format!("target_session={}", packet.to_session));
    if let Some(task_id) = packet.task_id.as_deref() {
        lines.push(format!("task={task_id}"));
    }
    if !packet.scope_claims.is_empty() {
        lines.push(format!("scope={}", packet.scope_claims.join(",")));
    }
    if let Some(reason) = packet.reason.as_deref() {
        lines.push(format!("reason={reason}"));
    }
    if let Some(note) = packet.note.as_deref() {
        lines.push(format!("note={note}"));
    }
    lines.join("\n")
}

fn format_hive_cowork_receipt_summary(packet: &HiveCoworkPacket) -> String {
    format!(
        "cowork {} from={} ({}) to={} ({}) task={} scope={} reason={} note={}",
        packet.action,
        packet.from_worker.as_deref().unwrap_or("unknown"),
        packet.from_session,
        packet.to_worker.as_deref().unwrap_or("unknown"),
        packet.to_session,
        packet.task_id.as_deref().unwrap_or("none"),
        if packet.scope_claims.is_empty() {
            "none".to_string()
        } else {
            packet.scope_claims.join(",")
        },
        packet.reason.as_deref().unwrap_or("none"),
        packet.note.as_deref().unwrap_or("none"),
    )
}

fn format_hive_queen_cowork_command(suggestion: &CoordinationSuggestion) -> Option<String> {
    let action = match suggestion.action.as_str() {
        "request_cowork" => "request",
        "ack_cowork" => "ack",
        _ => return None,
    };
    let target_session = suggestion.target_session.as_ref()?;
    let mut command = format!(
        "memd hive cowork {action} --to-session {}",
        shell_single_quote(target_session)
    );
    if let Some(task_id) = suggestion.task_id.as_deref() {
        command.push_str(&format!(" --task-id {}", shell_single_quote(task_id)));
    }
    command.push_str(&format!(
        " --reason {}",
        shell_single_quote(suggestion.reason.as_str())
    ));
    command.push_str(" --summary");
    Some(command)
}

fn annotate_hive_relationships(
    bees: Vec<memd_schema::HiveSessionRecord>,
) -> Vec<memd_schema::HiveSessionRecord> {
    let snapshot = bees.clone();
    bees.into_iter()
        .map(|mut bee| {
            let best = snapshot
                .iter()
                .filter(|peer| peer.session != bee.session)
                .filter_map(|peer| {
                    derive_hive_relationship(&bee, peer).map(|(state, reason, action)| {
                        let peer_label = peer
                            .display_name
                            .clone()
                            .or_else(|| peer.worker_name.clone())
                            .or_else(|| peer.agent.clone())
                            .unwrap_or_else(|| peer.session.clone());
                        (state_rank(&state), peer_label, state, reason, action)
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

fn state_rank(state: &str) -> u8 {
    match state {
        "conflict" => 5,
        "blocked" => 4,
        "cowork_active" => 3,
        "handoff_ready" => 2,
        "near" => 1,
        _ => 0,
    }
}

fn derive_hive_relationship(
    current: &memd_schema::HiveSessionRecord,
    peer: &memd_schema::HiveSessionRecord,
) -> Option<(String, String, String)> {
    if current
        .blocked_by
        .iter()
        .any(|value| value == &peer.session)
    {
        return Some((
            "blocked".to_string(),
            format!("waiting on peer {}", peer.session),
            "wait_for_peer".to_string(),
        ));
    }
    if current
        .cowork_with
        .iter()
        .any(|value| value == &peer.session)
        && peer
            .cowork_with
            .iter()
            .any(|value| value == &current.session)
    {
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
        .filter_map(|touch| normalize_hive_touch(touch))
        .filter(|touch| {
            peer.touches
                .iter()
                .filter_map(|other| normalize_hive_touch(other))
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
    None
}
