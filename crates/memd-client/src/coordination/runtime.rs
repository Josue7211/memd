use super::*;

pub(crate) async fn run_claims_command(
    args: &ClaimsArgs,
    base_url: &str,
) -> anyhow::Result<ClaimsResponse> {
    let runtime_before_overlay = read_bundle_runtime_config_raw(&args.output)?;
    let runtime = read_bundle_runtime_config(&args.output)?;
    let heartbeat = read_bundle_heartbeat(&args.output)?;
    let bundle_session = runtime_before_overlay
        .as_ref()
        .and_then(|config| config.session.clone());
    let live_session = runtime.as_ref().and_then(|config| config.session.clone());
    let rebased_from = match (bundle_session.as_deref(), live_session.as_deref()) {
        (Some(bundle), Some(live)) if bundle != live => Some(bundle.to_string()),
        _ => None,
    };
    let current_session = runtime
        .as_ref()
        .and_then(|config| config.session.clone())
        .filter(|value| !value.trim().is_empty());
    let current_tab_id = runtime
        .as_ref()
        .and_then(|config| config.tab_id.clone())
        .filter(|value| !value.trim().is_empty());
    let current_base_url = resolve_bundle_command_base_url(
        base_url,
        runtime
            .as_ref()
            .and_then(|config| config.base_url.as_deref()),
    );
    let current_agent = runtime.as_ref().and_then(|config| config.agent.clone());
    let current_effective_agent = runtime.as_ref().and_then(|config| {
        config
            .agent
            .as_deref()
            .map(|agent| compose_agent_identity(agent, config.session.as_deref()))
    });
    let current_workspace = runtime.as_ref().and_then(|config| config.workspace.clone());
    let (shared_project, shared_namespace) = shared_awareness_scope(runtime.as_ref());
    if args.acquire || args.release || args.transfer_to_session.is_some() {
        ensure_shared_authority_write_allowed(runtime.as_ref(), "claims mutation")?;
    }
    let client = MemdClient::new(&current_base_url)?;

    if args.acquire {
        let scope = args
            .scope
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("claims --acquire requires --scope")?;
        let session = current_session
            .as_deref()
            .context("claims --acquire requires a configured bundle session")?;
        let response = client
            .acquire_hive_claim(&memd_schema::HiveClaimAcquireRequest {
                scope: scope.to_string(),
                session: session.to_string(),
                tab_id: current_tab_id.clone(),
                agent: current_agent,
                effective_agent: current_effective_agent,
                project: runtime.as_ref().and_then(|config| config.project.clone()),
                namespace: runtime.as_ref().and_then(|config| config.namespace.clone()),
                workspace: current_workspace.clone(),
                host: heartbeat.as_ref().and_then(|value| value.host.clone()),
                pid: heartbeat.as_ref().and_then(|value| value.pid),
                ttl_seconds: args.ttl_secs,
            })
            .await?;
        auto_checkpoint_bundle_event(
            &args.output,
            &current_base_url,
            "claims",
            format!("Claimed scope {scope} for active work."),
            vec!["claims".to_string(), "auto-checkpoint".to_string()],
            0.82,
        )
        .await?;
        let cache = SessionClaimsState {
            claims: client
                .hive_claims(&memd_schema::HiveClaimsRequest {
                    session: None,
                    project: shared_project.clone(),
                    namespace: shared_namespace.clone(),
                    workspace: current_workspace.clone(),
                    active_only: Some(true),
                    limit: Some(512),
                })
                .await?
                .claims
                .into_iter()
                .map(session_claim_from_record)
                .collect(),
        };
        write_bundle_claims(&args.output, &cache)?;
        return Ok(ClaimsResponse {
            bundle_root: args.output.display().to_string(),
            bundle_session,
            live_session,
            rebased_from,
            current_session,
            current_tab_id,
            claims: response
                .claims
                .into_iter()
                .map(session_claim_from_record)
                .collect(),
        });
    } else if let Some(target_session) = args
        .transfer_to_session
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let scope = args
            .scope
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("claims --transfer-to-session requires --scope")?;
        let session = current_session
            .as_deref()
            .context("claims --transfer-to-session requires a configured bundle session")?;
        let target = resolve_target_session_bundle(&args.output, target_session).await?;
        let response = client
            .transfer_hive_claim(&memd_schema::HiveClaimTransferRequest {
                scope: scope.to_string(),
                from_session: session.to_string(),
                to_session: target_session.to_string(),
                to_tab_id: target
                    .as_ref()
                    .and_then(|entry| entry.tab_id.clone())
                    .or_else(|| current_tab_id.clone()),
                to_agent: target.as_ref().and_then(|entry| entry.agent.clone()),
                to_effective_agent: target
                    .as_ref()
                    .and_then(|entry| entry.effective_agent.clone()),
            })
            .await?;
        auto_checkpoint_bundle_event(
            &args.output,
            &current_base_url,
            "claims",
            format!("Transferred scope {scope} to session {target_session}."),
            vec![
                "claims".to_string(),
                "assignment".to_string(),
                "auto-checkpoint".to_string(),
            ],
            0.84,
        )
        .await?;
        let cache = SessionClaimsState {
            claims: client
                .hive_claims(&memd_schema::HiveClaimsRequest {
                    session: None,
                    project: shared_project.clone(),
                    namespace: shared_namespace.clone(),
                    workspace: current_workspace.clone(),
                    active_only: Some(true),
                    limit: Some(512),
                })
                .await?
                .claims
                .into_iter()
                .map(session_claim_from_record)
                .collect(),
        };
        write_bundle_claims(&args.output, &cache)?;
        return Ok(ClaimsResponse {
            bundle_root: args.output.display().to_string(),
            bundle_session,
            live_session,
            rebased_from,
            current_session,
            current_tab_id,
            claims: response
                .claims
                .into_iter()
                .map(session_claim_from_record)
                .collect(),
        });
    } else if args.release {
        let scope = args
            .scope
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("claims --release requires --scope")?;
        let session = current_session
            .as_deref()
            .context("claims --release requires a configured bundle session")?;
        let response = client
            .release_hive_claim(&memd_schema::HiveClaimReleaseRequest {
                scope: scope.to_string(),
                session: session.to_string(),
            })
            .await?;
        auto_checkpoint_bundle_event(
            &args.output,
            &current_base_url,
            "claims",
            format!("Released scope {scope} after finishing or handing off work."),
            vec!["claims".to_string(), "auto-checkpoint".to_string()],
            0.78,
        )
        .await?;
        let cache = SessionClaimsState {
            claims: client
                .hive_claims(&memd_schema::HiveClaimsRequest {
                    session: None,
                    project: runtime.as_ref().and_then(|config| config.project.clone()),
                    namespace: runtime.as_ref().and_then(|config| config.namespace.clone()),
                    workspace: None,
                    active_only: Some(true),
                    limit: Some(512),
                })
                .await?
                .claims
                .into_iter()
                .map(session_claim_from_record)
                .collect(),
        };
        write_bundle_claims(&args.output, &cache)?;
        return Ok(ClaimsResponse {
            bundle_root: args.output.display().to_string(),
            bundle_session,
            live_session,
            rebased_from,
            current_session,
            current_tab_id,
            claims: response
                .claims
                .into_iter()
                .map(session_claim_from_record)
                .collect(),
        });
    }

    let response = client
        .hive_claims(&memd_schema::HiveClaimsRequest {
            session: None,
            project: shared_project,
            namespace: shared_namespace,
            workspace: current_workspace,
            active_only: Some(true),
            limit: Some(512),
        })
        .await?;
    let claims = response
        .claims
        .into_iter()
        .map(session_claim_from_record)
        .collect::<Vec<_>>();
    write_bundle_claims(
        &args.output,
        &SessionClaimsState {
            claims: claims.clone(),
        },
    )?;

    Ok(ClaimsResponse {
        bundle_root: args.output.display().to_string(),
        bundle_session,
        live_session,
        rebased_from,
        current_session,
        current_tab_id,
        claims,
    })
}

pub(crate) fn render_claims_summary(response: &ClaimsResponse) -> String {
    let mut lines = vec![format!(
        "claims bundle={} bundle_session={} live_session={} rebased_from={} current_session={} current_tab={} active={}",
        response.bundle_root,
        response.bundle_session.as_deref().unwrap_or("none"),
        response.live_session.as_deref().unwrap_or("none"),
        response.rebased_from.as_deref().unwrap_or("none"),
        response.current_session.as_deref().unwrap_or("none"),
        response.current_tab_id.as_deref().unwrap_or("none"),
        response.claims.len()
    )];
    for claim in &response.claims {
        lines.push(format!(
            "- {} | holder={} | tab={} | workspace={} | expires_at={}",
            claim.scope,
            claim
                .effective_agent
                .as_deref()
                .or(claim.session.as_deref())
                .unwrap_or("none"),
            claim.tab_id.as_deref().unwrap_or("none"),
            claim.workspace.as_deref().unwrap_or("none"),
            claim.expires_at.to_rfc3339(),
        ));
    }
    lines.join("\n")
}

pub(crate) async fn run_messages_command(
    args: &MessagesArgs,
    base_url: &str,
) -> anyhow::Result<MessagesResponse> {
    let runtime_before_overlay = read_bundle_runtime_config_raw(&args.output)?;
    let runtime = read_bundle_runtime_config(&args.output)?;
    let bundle_session = runtime_before_overlay
        .as_ref()
        .and_then(|config| config.session.clone());
    let live_session = runtime.as_ref().and_then(|config| config.session.clone());
    let rebased_from = match (bundle_session.as_deref(), live_session.as_deref()) {
        (Some(bundle), Some(live)) if bundle != live => Some(bundle.to_string()),
        _ => None,
    };
    let current_session = runtime
        .as_ref()
        .and_then(|config| config.session.clone())
        .filter(|value| !value.trim().is_empty());
    let current_agent = runtime.as_ref().and_then(|config| {
        config
            .agent
            .as_deref()
            .map(|agent| compose_agent_identity(agent, config.session.as_deref()))
    });
    let current_project = runtime.as_ref().and_then(|config| config.project.clone());
    let current_namespace = runtime.as_ref().and_then(|config| config.namespace.clone());
    let current_workspace = runtime.as_ref().and_then(|config| config.workspace.clone());
    let current_base_url = resolve_bundle_command_base_url(
        base_url,
        runtime
            .as_ref()
            .and_then(|config| config.base_url.as_deref()),
    );
    if args.send || args.ack.is_some() {
        ensure_shared_authority_write_allowed(runtime.as_ref(), "message coordination")?;
    }

    if args.send {
        let target_session = args
            .target_session
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("messages --send requires --target-session")?;
        let from_session = current_session
            .as_deref()
            .context("messages --send requires a configured bundle session")?;
        let (kind, content) = derive_outbound_message(args)
            .context("messages --send requires --content or a request helper")?;
        let target = resolve_target_session_bundle(&args.output, target_session)
            .await?
            .context("target session not found in awareness")?;
        let source_client = MemdClient::new(&current_base_url)?;
        if let Err(error) =
            ensure_target_session_lane_is_safe(&args.output, current_session.as_deref(), &target)
        {
            emit_lane_fault_receipt(
                &source_client,
                from_session,
                current_agent.clone(),
                &target,
                None,
                args.assign_scope.clone().or(args.scope.clone()),
                current_project.clone(),
                current_namespace.clone(),
                current_workspace.clone(),
            )
            .await;
            return Err(error);
        }
        let target_runtime = read_bundle_runtime_config(Path::new(&target.bundle_root))?;
        let target_base_url = target_runtime
            .as_ref()
            .and_then(|config| config.base_url.clone())
            .or(target.base_url.clone())
            .unwrap_or_else(|| current_base_url.clone());
        let client = MemdClient::new(&target_base_url)?;
        if let Some(assign_scope) = args
            .assign_scope
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let transfer_client = MemdClient::new(&current_base_url)?;
            transfer_client
                .transfer_hive_claim(&memd_schema::HiveClaimTransferRequest {
                    scope: assign_scope.to_string(),
                    from_session: from_session.to_string(),
                    to_session: target_session.to_string(),
                    to_tab_id: target.tab_id.clone(),
                    to_agent: target.agent.clone(),
                    to_effective_agent: target.effective_agent.clone(),
                })
                .await?;
        }
        let response = client
            .send_hive_message(&HiveMessageSendRequest {
                kind,
                from_session: from_session.to_string(),
                from_agent: current_agent.clone(),
                to_session: target_session.to_string(),
                project: current_project.clone(),
                namespace: current_namespace.clone(),
                workspace: current_workspace.clone(),
                content,
            })
            .await?;
        let summary = if let Some(assign_scope) = args
            .assign_scope
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            format!("Assigned scope {assign_scope} to session {target_session}.")
        } else if args.request_help {
            format!("Requested help from session {target_session}.")
        } else if args.request_review {
            format!("Requested review from session {target_session}.")
        } else {
            format!(
                "Sent {} message to session {target_session}.",
                response.messages[0].kind
            )
        };
        let mut tags = vec!["messages".to_string(), "auto-checkpoint".to_string()];
        if args.request_help {
            tags.push("help-request".to_string());
        }
        if args.request_review {
            tags.push("review-request".to_string());
        }
        if args.assign_scope.is_some() {
            tags.push("assignment".to_string());
        }
        auto_checkpoint_bundle_event(
            &args.output,
            &current_base_url,
            "messages",
            summary,
            tags,
            0.8,
        )
        .await?;
        emit_coordination_receipt(
            &client,
            if args.assign_scope.is_some() {
                "assignment"
            } else if args.request_help {
                "help_request"
            } else if args.request_review {
                "review_request"
            } else {
                "message"
            },
            from_session,
            current_agent.clone(),
            Some(target_session.to_string()),
            None,
            args.assign_scope.clone().or(args.scope.clone()),
            current_project.clone(),
            current_namespace.clone(),
            current_workspace.clone(),
            response
                .messages
                .first()
                .map(|message| message.content.clone())
                .unwrap_or_else(|| "coordination message sent".to_string()),
        )
        .await?;
        return Ok(MessagesResponse {
            bundle_root: args.output.display().to_string(),
            bundle_session,
            live_session,
            rebased_from,
            current_session,
            messages: response.messages,
        });
    }

    let client = MemdClient::new(&current_base_url)?;
    let messages = if let Some(ack) = args.ack.as_deref() {
        let session = current_session
            .as_deref()
            .context("messages --ack requires a configured bundle session")?;
        client
            .ack_hive_message(&HiveMessageAckRequest {
                id: ack.trim().to_string(),
                session: session.to_string(),
            })
            .await?
            .messages
    } else {
        let session = current_session
            .as_deref()
            .context("messages --inbox requires a configured bundle session")?;
        client
            .hive_inbox(&HiveMessageInboxRequest {
                session: session.to_string(),
                project: current_project,
                namespace: current_namespace,
                workspace: current_workspace,
                include_acknowledged: Some(false),
                limit: Some(128),
            })
            .await?
            .messages
    };

    Ok(MessagesResponse {
        bundle_root: args.output.display().to_string(),
        bundle_session,
        live_session,
        rebased_from,
        current_session,
        messages,
    })
}

pub(crate) fn derive_outbound_message(args: &MessagesArgs) -> Option<(String, String)> {
    let assign_scope = args
        .assign_scope
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let scope = args
        .scope
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let explicit_content = args
        .content
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    if args.request_help {
        let content = explicit_content.or_else(|| {
            scope
                .map(|scope| format!("Need help on {scope}. Please coordinate before changing it."))
        })?;
        return Some(("help_request".to_string(), content));
    }

    if args.request_review {
        let content = explicit_content.or_else(|| {
            scope.map(|scope| {
                format!("Need review on {scope}. Please inspect before I hand it off.")
            })
        })?;
        return Some(("review_request".to_string(), content));
    }

    if let Some(assign_scope) = assign_scope {
        let content = explicit_content.or_else(|| {
            Some(format!(
                "Assigned scope {assign_scope}. Take ownership and continue from there."
            ))
        })?;
        return Some(("assignment".to_string(), content));
    }

    let content = explicit_content?;
    Some((
        args.kind.clone().unwrap_or_else(|| "handoff".to_string()),
        content,
    ))
}

pub(crate) fn render_messages_summary(response: &MessagesResponse) -> String {
    let mut lines = vec![format!(
        "messages bundle={} bundle_session={} live_session={} rebased_from={} current_session={} count={}",
        response.bundle_root,
        response.bundle_session.as_deref().unwrap_or("none"),
        response.live_session.as_deref().unwrap_or("none"),
        response.rebased_from.as_deref().unwrap_or("none"),
        response.current_session.as_deref().unwrap_or("none"),
        response.messages.len()
    )];
    for message in &response.messages {
        lines.push(format!(
            "- {} [{}] {} -> {} | acked={} | {}",
            &message.id[..8.min(message.id.len())],
            message.kind,
            message.from_agent.as_deref().unwrap_or("unknown"),
            message.to_session,
            if message.acknowledged_at.is_some() {
                "yes"
            } else {
                "no"
            },
            compact_inline(&message.content, 80)
        ));
    }
    lines.join("\n")
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn emit_coordination_receipt(
    client: &MemdClient,
    kind: &str,
    actor_session: &str,
    actor_agent: Option<String>,
    target_session: Option<String>,
    task_id: Option<String>,
    scope: Option<String>,
    project: Option<String>,
    namespace: Option<String>,
    workspace: Option<String>,
    summary: String,
) -> anyhow::Result<()> {
    client
        .record_hive_coordination_receipt(&HiveCoordinationReceiptRequest {
            kind: kind.to_string(),
            actor_session: actor_session.to_string(),
            actor_agent,
            target_session,
            task_id,
            scope,
            project,
            namespace,
            workspace,
            summary,
        })
        .await?;
    Ok(())
}

pub(crate) async fn run_session_command(
    args: &SessionArgs,
    base_url: &str,
) -> anyhow::Result<SessionResponse> {
    let runtime_before_overlay = read_bundle_runtime_config_raw(&args.output)?;
    let bundle_session_before = runtime_before_overlay
        .as_ref()
        .and_then(|config| config.session.clone());
    let runtime = read_bundle_runtime_config(&args.output)?
        .context("session requires a readable bundle runtime config")?;
    let resolved_base_url = resolve_bundle_command_base_url(base_url, runtime.base_url.as_deref());
    if args.retire_session.is_some() || args.reconcile {
        ensure_shared_authority_write_allowed(Some(&runtime), "session repair")?;
    }
    let client = MemdClient::new(&resolved_base_url)?;
    let awareness = read_project_awareness(&AwarenessArgs {
        output: args.output.clone(),
        root: None,
        include_current: true,
        summary: false,
    })
    .await?;

    let mut action = "summary".to_string();
    let mut retired_sessions = 0usize;
    let mut retire_target = None;
    let mut heartbeat = None;
    let mut reconciled = false;
    let mut reconciled_retired_sessions = 0usize;

    if args.rebind
        && let Some(live_session) = runtime.session.as_deref()
    {
        set_bundle_session(&args.output, live_session)?;
        if let Some(tab_id) = runtime.tab_id.as_deref() {
            set_bundle_tab_id(&args.output, tab_id)?;
        }
        action = "rebind".to_string();
    }

    if let Some(target_session) = args
        .retire_session
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let target = awareness
            .entries
            .iter()
            .find(|entry| entry.session.as_deref() == Some(target_session))
            .context("session retire target not found in awareness")?;
        retired_sessions = retire_hive_session_entry(
            &client,
            target,
            format!(
                "retired by live session {}",
                runtime.session.as_deref().unwrap_or("unknown")
            ),
        )
        .await?;
        retire_target = Some(target_session.to_string());
        action = if action == "summary" {
            "retire".to_string()
        } else {
            format!("{action}+retire")
        };
    }

    if args.reconcile || args.rebind {
        let (heartbeat_state, retired) =
            reconcile_bundle_heartbeat(&args.output, None, false).await?;
        heartbeat = Some(serde_json::to_value(heartbeat_state)?);
        reconciled = true;
        reconciled_retired_sessions = retired;
        if action == "summary" {
            action = "reconcile".to_string();
        } else if action != "rebind" {
            action = format!("{action}+reconcile");
        }
    }

    let runtime_after_overlay = read_bundle_runtime_config_raw(&args.output)?;
    let runtime_after = read_bundle_runtime_config(&args.output)?
        .context("reload bundle runtime config after session action")?;

    Ok(SessionResponse {
        action,
        bundle_root: args.output.display().to_string(),
        bundle_session: runtime_after_overlay.and_then(|config| config.session),
        live_session: runtime_after.session.clone(),
        rebased_from: match (bundle_session_before, runtime_after.session.clone()) {
            (Some(bundle), Some(live)) if bundle != live => Some(bundle),
            _ => None,
        },
        tab_id: runtime_after.tab_id,
        reconciled,
        reconciled_retired_sessions,
        retired_sessions,
        retire_target,
        heartbeat,
    })
}

pub(crate) fn render_session_summary(response: &SessionResponse) -> String {
    format!(
        "session action={} bundle={} bundle_session={} live_session={} rebased_from={} tab={} reconciled={} reconciled_retired={} retired={} retire_target={} heartbeat={}",
        response.action,
        response.bundle_root,
        response.bundle_session.as_deref().unwrap_or("none"),
        response.live_session.as_deref().unwrap_or("none"),
        response.rebased_from.as_deref().unwrap_or("none"),
        response.tab_id.as_deref().unwrap_or("none"),
        if response.reconciled { "yes" } else { "no" },
        response.reconciled_retired_sessions,
        response.retired_sessions,
        response.retire_target.as_deref().unwrap_or("none"),
        if response.heartbeat.is_some() {
            "published"
        } else {
            "skipped"
        }
    )
}
