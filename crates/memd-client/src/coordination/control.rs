use super::*;

pub(crate) fn select_coordination_helper_hive<'a>(
    active_hives: &'a [ProjectAwarenessEntry],
    tasks: &[HiveTaskRecord],
    policy_conflicts: &[String],
    current_session: &str,
) -> Option<&'a ProjectAwarenessEntry> {
    let mut need_runtime = policy_conflicts.iter().any(|conflict| {
        let lowered = conflict.to_ascii_lowercase();
        lowered.contains("runtime")
            || lowered.contains("dependency")
            || lowered.contains("secret")
            || lowered.contains("auth")
            || lowered.contains("shell")
    });
    if !need_runtime {
        need_runtime = tasks.iter().any(|task| {
            let haystack = format!(
                "{} {} {}",
                task.title,
                task.description.as_deref().unwrap_or(""),
                task.claim_scopes.join(" ")
            )
            .to_ascii_lowercase();
            haystack.contains("runtime")
                || haystack.contains("dependency")
                || haystack.contains("secret")
                || haystack.contains("auth")
                || haystack.contains("shell")
                || haystack.contains("infra")
        });
    }

    let preferred_groups = if need_runtime {
        ["runtime-core", "dependency-owners"]
    } else {
        ["openclaw-stack", "control-plane"]
    };

    active_hives
        .iter()
        .filter(|hive| {
            hive.session
                .as_deref()
                .is_some_and(|value| value != current_session)
        })
        .max_by_key(|hive| {
            let mut score = 0i32;
            for group in &hive.hive_groups {
                if preferred_groups.iter().any(|preferred| preferred == group) {
                    score += 10;
                }
            }
            if hive.authority.as_deref() == Some("canonical") {
                score += 3;
            }
            if hive.authority.as_deref() == Some("coordinator") {
                score += 2;
            }
            if hive.presence == "active" {
                score += 2;
            }
            if hive.hive_role.as_deref() == Some("runtime-shell") && need_runtime {
                score += 4;
            }
            if hive.hive_role.as_deref() == Some("secret-broker") && need_runtime {
                score += 4;
            }
            score
        })
}

pub(crate) fn suggest_boundary_recommendations(
    tasks: &[HiveTaskRecord],
    claims: &[SessionClaim],
    current_session: &str,
) -> Vec<String> {
    tasks
        .iter()
        .map(|task| {
            let branch_prefix = match task.coordination_mode.as_str() {
                "exclusive_write" => "dedicated branch",
                "shared_review" => "review branch",
                "help_only" => "help branch",
                _ => "shared branch",
            };
            let owner = task
                .effective_agent
                .as_deref()
                .or(task.session.as_deref())
                .unwrap_or("none");
            let claim_summary = if task.claim_scopes.is_empty() {
                "without claim scopes".to_string()
            } else {
                let contested = task
                    .claim_scopes
                    .iter()
                    .filter_map(|scope| {
                        claims
                            .iter()
                            .find(|claim| claim.scope == *scope)
                            .and_then(|claim| {
                                let claim_owner = claim
                                    .effective_agent
                                    .as_deref()
                                    .or(claim.session.as_deref());
                                if claim_owner != Some(owner)
                                    && claim.session.as_deref() != Some(current_session)
                                {
                                    Some(format!(
                                        "{} held by {}",
                                        scope,
                                        claim_owner.unwrap_or("unknown")
                                    ))
                                } else {
                                    None
                                }
                            })
                    })
                    .collect::<Vec<_>>();
                if contested.is_empty() {
                    format!("for scopes {}", task.claim_scopes.join(","))
                } else {
                    format!(
                        "for scopes {} with conflicts: {}",
                        task.claim_scopes.join(","),
                        contested.join("; ")
                    )
                }
            };
            format!(
                "{} for task {} owned by {} {}",
                branch_prefix, task.task_id, owner, claim_summary
            )
        })
        .take(8)
        .collect()
}

pub(crate) async fn run_coordination_command(
    args: &CoordinationArgs,
    base_url: &str,
) -> anyhow::Result<CoordinationResponse> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let current_session = runtime
        .as_ref()
        .and_then(|config| config.session.clone())
        .filter(|value| !value.trim().is_empty())
        .context("coordination requires a configured bundle session")?;
    let current_base_url = resolve_bundle_command_base_url(
        base_url,
        runtime
            .as_ref()
            .and_then(|config| config.base_url.as_deref()),
    );
    let current_effective_agent = runtime.as_ref().and_then(|config| {
        config
            .agent
            .as_deref()
            .map(|agent| compose_agent_identity(agent, config.session.as_deref()))
    });
    let mutating_request = args.recover_session.is_some()
        || args.retire_session.is_some()
        || args.to_session.is_some()
        || args.deny_session.is_some()
        || args.reroute_session.is_some()
        || args.handoff_scope.is_some();
    if mutating_request {
        ensure_shared_authority_write_allowed(runtime.as_ref(), "coordination queen actions")?;
    }
    let client = MemdClient::new(&current_base_url)?;
    let server_reachable = timeout_ok(client.healthz()).await.is_some();
    let awareness = read_project_awareness(&AwarenessArgs {
        output: args.output.clone(),
        root: None,
        include_current: true,
        summary: false,
    })
    .await?;
    let current_project = runtime.as_ref().and_then(|config| config.project.clone());
    let current_namespace = runtime.as_ref().and_then(|config| config.namespace.clone());
    let current_workspace = runtime.as_ref().and_then(|config| config.workspace.clone());
    let lane_fault = detect_lane_collision_from_awareness_entries(
        &args.output,
        Some(current_session.as_str()),
        &awareness.entries,
    )
    .and_then(|conflict| {
        build_lane_fault_surface(&args.output, Some(current_session.as_str()), &conflict)
    });
    if !server_reachable && mutating_request {
        anyhow::bail!(
            "coordination backend unreachable at {}; queen actions require a live memd server",
            current_base_url
        );
    }
    let initial_claims_request = HiveClaimsRequest {
        session: None,
        project: current_project.clone(),
        namespace: current_namespace.clone(),
        workspace: current_workspace.clone(),
        active_only: Some(true),
        limit: Some(512),
    };
    let claims = if server_reachable {
        timeout_ok(client.hive_claims(&initial_claims_request))
            .await
            .map(|response| response.claims)
            .unwrap_or_default()
            .into_iter()
            .map(session_claim_from_record)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let initial_tasks_request = HiveTasksRequest {
        session: None,
        project: current_project.clone(),
        namespace: current_namespace.clone(),
        workspace: current_workspace.clone(),
        active_only: Some(true),
        limit: Some(512),
    };
    let tasks = if server_reachable {
        timeout_ok(client.hive_tasks(&initial_tasks_request))
            .await
            .map(|response| response.tasks)
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let stale_hives = awareness
        .entries
        .iter()
        .filter(|entry| entry.session.as_deref() != Some(current_session.as_str()))
        .filter(|entry| entry.presence == "stale" || entry.presence == "dead")
        .cloned()
        .collect::<Vec<_>>();

    if let Some(retire_session) = args
        .retire_session
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let stale_entry = stale_hives
            .iter()
            .find(|entry| entry.session.as_deref() == Some(retire_session))
            .cloned()
            .context("retire_session must target a stale or dead session")?;
        let retired_sessions = retire_hive_session_entry(
            &client,
            &stale_entry,
            format!("retired by live session {current_session}"),
        )
        .await?;
        auto_checkpoint_bundle_event(
            &args.output,
            &current_base_url,
            "coordination",
            format!(
                "Retired {} session record(s) for {} session {}.",
                retired_sessions, stale_entry.presence, retire_session
            ),
            vec![
                "coordination".to_string(),
                "retirement".to_string(),
                "auto-checkpoint".to_string(),
            ],
            0.82,
        )
        .await?;
        emit_coordination_receipt(
            &client,
            "stale_session_retirement",
            &current_session,
            current_effective_agent.clone(),
            None,
            None,
            None,
            runtime.as_ref().and_then(|config| config.project.clone()),
            runtime.as_ref().and_then(|config| config.namespace.clone()),
            runtime.as_ref().and_then(|config| config.workspace.clone()),
            format!(
                "Retired {} session record(s) for {} session {}.",
                retired_sessions, stale_entry.presence, retire_session
            ),
        )
        .await?;
    }

    if let Some(deny_session) = args
        .deny_session
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        emit_coordination_receipt(
            &client,
            "queen_deny",
            &current_session,
            current_effective_agent.clone(),
            Some(deny_session.to_string()),
            None,
            None,
            runtime.as_ref().and_then(|config| config.project.clone()),
            runtime.as_ref().and_then(|config| config.namespace.clone()),
            runtime.as_ref().and_then(|config| config.workspace.clone()),
            format!("Queen denied overlapping lane or scope work for session {deny_session}."),
        )
        .await?;
    }

    if let Some(reroute_session) = args
        .reroute_session
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        emit_coordination_receipt(
            &client,
            "queen_reroute",
            &current_session,
            current_effective_agent.clone(),
            Some(reroute_session.to_string()),
            None,
            None,
            runtime.as_ref().and_then(|config| config.project.clone()),
            runtime.as_ref().and_then(|config| config.namespace.clone()),
            runtime.as_ref().and_then(|config| config.workspace.clone()),
            format!("Queen ordered session {reroute_session} onto a new isolated lane."),
        )
        .await?;
    }

    if let Some(handoff_scope) = args
        .handoff_scope
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let target_session = args
            .to_session
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("coordination --handoff-scope requires --to-session")?;
        emit_coordination_receipt(
            &client,
            "queen_handoff",
            &current_session,
            current_effective_agent.clone(),
            Some(target_session.to_string()),
            None,
            Some(handoff_scope.to_string()),
            runtime.as_ref().and_then(|config| config.project.clone()),
            runtime.as_ref().and_then(|config| config.namespace.clone()),
            runtime.as_ref().and_then(|config| config.workspace.clone()),
            format!("Queen handed off scope {handoff_scope} to session {target_session}."),
        )
        .await?;
    }

    if let Some(recover_session) = args
        .recover_session
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let stale_entry = stale_hives
            .iter()
            .find(|entry| entry.session.as_deref() == Some(recover_session))
            .cloned()
            .context("recover_session must target a stale or dead session")?;
        let destination = if let Some(to_session) = args
            .to_session
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            awareness
                .entries
                .iter()
                .find(|entry| entry.session.as_deref() == Some(to_session))
                .cloned()
                .context("to_session not found in awareness")?
        } else {
            awareness
                .entries
                .iter()
                .find(|entry| entry.session.as_deref() == Some(current_session.as_str()))
                .cloned()
                .context("current session missing from awareness")?
        };

        let recover_claims = claims
            .iter()
            .filter(|claim| claim.session.as_deref() == Some(recover_session))
            .cloned()
            .collect::<Vec<_>>();
        let recover_tasks = tasks
            .iter()
            .filter(|task| task.session.as_deref() == Some(recover_session))
            .cloned()
            .collect::<Vec<_>>();

        for claim in &recover_claims {
            client
                .recover_hive_claim(&HiveClaimRecoverRequest {
                    scope: claim.scope.clone(),
                    from_session: recover_session.to_string(),
                    to_session: destination.session.clone(),
                    to_tab_id: destination.tab_id.clone(),
                    to_agent: destination.agent.clone(),
                    to_effective_agent: destination.effective_agent.clone(),
                })
                .await?;
        }
        for task in &recover_tasks {
            client
                .assign_hive_task(&HiveTaskAssignRequest {
                    task_id: task.task_id.clone(),
                    from_session: Some(recover_session.to_string()),
                    to_session: destination
                        .session
                        .clone()
                        .context("destination session missing for recovery")?,
                    to_agent: destination.agent.clone(),
                    to_effective_agent: destination.effective_agent.clone(),
                    note: Some(format!(
                        "Recovered from {} session {}",
                        stale_entry.presence, recover_session
                    )),
                })
                .await?;
        }
        let retired_sessions = retire_hive_session_entry(
            &client,
            &stale_entry,
            format!(
                "recovered to {}",
                destination.session.as_deref().unwrap_or("unknown")
            ),
        )
        .await?;
        auto_checkpoint_bundle_event(
            &args.output,
            &current_base_url,
            "coordination",
            format!(
                "Recovered {} claims and {} tasks from {} session {} and retired {} session record(s).",
                recover_claims.len(),
                recover_tasks.len(),
                stale_entry.presence,
                recover_session,
                retired_sessions
            ),
            vec![
                "coordination".to_string(),
                "recovery".to_string(),
                "auto-checkpoint".to_string(),
            ],
            0.86,
        )
        .await?;
        emit_coordination_receipt(
            &client,
            "stale_session_recovery",
            &current_session,
            current_effective_agent.clone(),
            destination.session.clone(),
            None,
            None,
            runtime.as_ref().and_then(|config| config.project.clone()),
            runtime.as_ref().and_then(|config| config.namespace.clone()),
            runtime.as_ref().and_then(|config| config.workspace.clone()),
            format!(
                "Recovered {} claims and {} tasks from {} session {} and retired {} session record(s).",
                recover_claims.len(),
                recover_tasks.len(),
                stale_entry.presence,
                recover_session,
                retired_sessions
            ),
        )
        .await?;
    }

    let inbox_request = HiveCoordinationInboxRequest {
        session: current_session.clone(),
        project: current_project.clone(),
        namespace: current_namespace.clone(),
        workspace: current_workspace.clone(),
        limit: Some(128),
    };
    let response = if server_reachable {
        timeout_ok(client.hive_coordination_inbox(&inbox_request))
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
    let claims_request = HiveClaimsRequest {
        session: None,
        project: current_project,
        namespace: current_namespace,
        workspace: current_workspace,
        active_only: Some(true),
        limit: Some(512),
    };
    let claims = if server_reachable {
        timeout_ok(client.hive_claims(&claims_request))
            .await
            .map(|response| response.claims)
            .unwrap_or_default()
            .into_iter()
            .map(session_claim_from_record)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let tasks_request = HiveTasksRequest {
        session: None,
        project: runtime.as_ref().and_then(|config| config.project.clone()),
        namespace: runtime.as_ref().and_then(|config| config.namespace.clone()),
        workspace: runtime.as_ref().and_then(|config| config.workspace.clone()),
        active_only: Some(true),
        limit: Some(512),
    };
    let tasks = if server_reachable {
        timeout_ok(client.hive_tasks(&tasks_request))
            .await
            .map(|response| response.tasks)
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let receipts_request = HiveCoordinationReceiptsRequest {
        session: None,
        project: runtime.as_ref().and_then(|config| config.project.clone()),
        namespace: runtime.as_ref().and_then(|config| config.namespace.clone()),
        workspace: runtime.as_ref().and_then(|config| config.workspace.clone()),
        limit: Some(32),
    };
    let receipts = if server_reachable {
        timeout_ok(client.hive_coordination_receipts(&receipts_request))
            .await
            .map(|response| response.receipts)
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let lane_receipts = receipts
        .iter()
        .filter(|receipt| receipt.kind.starts_with("lane_") || receipt.kind.starts_with("queen_"))
        .cloned()
        .collect::<Vec<_>>();
    let policy_conflicts = tasks
        .iter()
        .filter(|task| task.coordination_mode == "exclusive_write")
        .flat_map(|task| {
            task.claim_scopes.iter().filter_map(|scope| {
                claims
                    .iter()
                    .find(|claim| claim.scope == *scope)
                    .and_then(|claim| {
                        let claim_owner = claim.session.as_deref();
                        let task_owner = task.session.as_deref();
                        if claim_owner.is_some() && claim_owner != task_owner {
                            Some(format!(
                                "task {} requires exclusive_write but scope {} is held by {}",
                                task.task_id,
                                scope,
                                claim
                                    .effective_agent
                                    .as_deref()
                                    .or(claim.session.as_deref())
                                    .unwrap_or("none")
                            ))
                        } else {
                            None
                        }
                    })
            })
        })
        .collect::<Vec<_>>();
    let stale_sessions = stale_hives
        .iter()
        .filter_map(|entry| entry.session.as_deref())
        .collect::<Vec<_>>();
    let active_hives = awareness
        .entries
        .iter()
        .filter(|entry| entry.session.as_deref() != Some(current_session.as_str()))
        .filter(|entry| entry.presence == "active")
        .cloned()
        .collect::<Vec<_>>();
    let suggestions = suggest_coordination_actions(
        &response,
        &stale_sessions,
        &active_hives,
        &claims,
        &tasks,
        &current_session,
        &policy_conflicts,
        lane_fault.as_ref(),
        &lane_receipts,
    );
    Ok(CoordinationResponse {
        bundle_root: args.output.display().to_string(),
        current_session: current_session.clone(),
        inbox: response,
        active_hives: active_hives.clone(),
        recovery: CoordinationRecoverySummary {
            stale_hives: stale_hives.clone(),
            reclaimable_claims: claims
                .clone()
                .into_iter()
                .filter(|claim| {
                    claim.session.as_deref().is_some_and(|session| {
                        stale_hives
                            .iter()
                            .any(|entry| entry.session.as_deref() == Some(session))
                    })
                })
                .collect(),
            stalled_tasks: tasks
                .clone()
                .into_iter()
                .filter(|task| {
                    task.session.as_deref().is_some_and(|session| {
                        stale_hives
                            .iter()
                            .any(|entry| entry.session.as_deref() == Some(session))
                    })
                })
                .collect(),
            retireable_sessions: stale_hives
                .iter()
                .filter(|entry| {
                    let session = entry.session.as_deref();
                    !claims
                        .iter()
                        .any(|claim| claim.session.as_deref() == session)
                        && !tasks.iter().any(|task| task.session.as_deref() == session)
                })
                .cloned()
                .collect(),
        },
        lane_fault,
        lane_receipts,
        policy_conflicts,
        suggestions,
        boundary_recommendations: suggest_boundary_recommendations(
            &tasks,
            &claims,
            &current_session,
        ),
        receipts,
    })
}

pub(crate) async fn run_tasks_command(
    args: &TasksArgs,
    base_url: &str,
) -> anyhow::Result<TasksResponse> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let heartbeat = read_bundle_heartbeat(&args.output)?;
    let current_session = runtime
        .as_ref()
        .and_then(|config| config.session.clone())
        .filter(|value| !value.trim().is_empty());
    let current_agent = runtime.as_ref().and_then(|config| config.agent.clone());
    let current_effective_agent = runtime.as_ref().and_then(|config| {
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
    if args.upsert || args.assign_to_session.is_some() || args.request_help || args.request_review {
        ensure_shared_authority_write_allowed(runtime.as_ref(), "task coordination")?;
    }
    let client = MemdClient::new(&current_base_url)?;

    if args.upsert {
        let task_id = args
            .task_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("tasks --upsert requires --task-id")?;
        let title = args
            .title
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("tasks --upsert requires --title")?;
        let response = client
            .upsert_hive_task(&HiveTaskUpsertRequest {
                task_id: task_id.to_string(),
                title: title.to_string(),
                description: args.description.clone(),
                status: args.status.clone(),
                coordination_mode: args.mode.clone(),
                session: current_session.clone(),
                agent: current_agent.clone(),
                effective_agent: current_effective_agent.clone(),
                project: current_project.clone(),
                namespace: current_namespace.clone(),
                workspace: current_workspace.clone(),
                claim_scopes: args.scope.clone(),
                help_requested: Some(false),
                review_requested: Some(false),
            })
            .await?;
        auto_checkpoint_bundle_event(
            &args.output,
            &current_base_url,
            "tasks",
            format!("Updated shared task {task_id}."),
            vec!["tasks".to_string(), "auto-checkpoint".to_string()],
            0.83,
        )
        .await?;
        emit_coordination_receipt(
            &client,
            "task_update",
            current_session.as_deref().unwrap_or("unknown"),
            current_effective_agent.clone(),
            None,
            Some(task_id.to_string()),
            args.scope.first().cloned(),
            current_project.clone(),
            current_namespace.clone(),
            current_workspace.clone(),
            format!("Updated shared task {task_id}."),
        )
        .await?;
        return Ok(TasksResponse {
            bundle_root: args.output.display().to_string(),
            current_session,
            tasks: response.tasks,
        });
    }

    if let Some(target_session) = args
        .assign_to_session
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let task_id = args
            .task_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("tasks --assign-to-session requires --task-id")?;
        let session = current_session
            .as_deref()
            .context("tasks --assign-to-session requires a configured bundle session")?;
        let target = resolve_target_session_bundle(&args.output, target_session)
            .await?
            .context("target session not found in awareness")?;
        if let Err(error) =
            ensure_target_session_lane_is_safe(&args.output, current_session.as_deref(), &target)
        {
            emit_lane_fault_receipt(
                &client,
                session,
                current_effective_agent.clone(),
                &target,
                Some(task_id.to_string()),
                None,
                current_project.clone(),
                current_namespace.clone(),
                current_workspace.clone(),
            )
            .await;
            return Err(error);
        }
        let requested_scopes = existing_task_scopes_for_assignment(
            &client,
            &current_project,
            &current_namespace,
            &current_workspace,
            task_id,
        )
        .await?;
        if let Some(error) = confirmed_hive_overlap_reason(
            &target,
            Some(task_id),
            args.title.as_deref().or(heartbeat
                .as_ref()
                .and_then(|value| value.topic_claim.as_deref())),
            &requested_scopes,
        ) {
            emit_lane_fault_receipt(
                &client,
                session,
                current_effective_agent.clone(),
                &target,
                Some(task_id.to_string()),
                requested_scopes.first().cloned(),
                current_project.clone(),
                current_namespace.clone(),
                current_workspace.clone(),
            )
            .await;
            anyhow::bail!(error);
        }

        let existing = client
            .hive_tasks(&HiveTasksRequest {
                session: None,
                project: current_project.clone(),
                namespace: current_namespace.clone(),
                workspace: current_workspace.clone(),
                active_only: Some(false),
                limit: Some(256),
            })
            .await?;
        if let Some(task) = existing.tasks.iter().find(|task| task.task_id == task_id) {
            for scope in &task.claim_scopes {
                let _ = client
                    .transfer_hive_claim(&memd_schema::HiveClaimTransferRequest {
                        scope: scope.clone(),
                        from_session: session.to_string(),
                        to_session: target_session.to_string(),
                        to_tab_id: target.tab_id.clone(),
                        to_agent: target.agent.clone(),
                        to_effective_agent: target.effective_agent.clone(),
                    })
                    .await;
            }
        }

        let response = client
            .assign_hive_task(&HiveTaskAssignRequest {
                task_id: task_id.to_string(),
                from_session: Some(session.to_string()),
                to_session: target_session.to_string(),
                to_agent: target.agent.clone(),
                to_effective_agent: target.effective_agent.clone(),
                note: None,
            })
            .await?;
        auto_checkpoint_bundle_event(
            &args.output,
            &current_base_url,
            "tasks",
            format!("Assigned shared task {task_id} to session {target_session}."),
            vec![
                "tasks".to_string(),
                "assignment".to_string(),
                "auto-checkpoint".to_string(),
            ],
            0.85,
        )
        .await?;
        emit_coordination_receipt(
            &client,
            "task_assignment",
            session,
            current_effective_agent.clone(),
            Some(target_session.to_string()),
            Some(task_id.to_string()),
            None,
            current_project.clone(),
            current_namespace.clone(),
            current_workspace.clone(),
            format!("Assigned shared task {task_id} to session {target_session}."),
        )
        .await?;
        return Ok(TasksResponse {
            bundle_root: args.output.display().to_string(),
            current_session,
            tasks: response.tasks,
        });
    }

    if args.request_help || args.request_review {
        let task_id = args
            .task_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("tasks help/review requires --task-id")?;
        let target_session = args
            .target_session
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .context("tasks help/review requires --target-session")?;
        let from_session = current_session
            .as_deref()
            .context("tasks help/review requires a configured bundle session")?;
        let target = resolve_target_session_bundle(&args.output, target_session)
            .await?
            .context("target session not found in awareness")?;
        if let Err(error) =
            ensure_target_session_lane_is_safe(&args.output, current_session.as_deref(), &target)
        {
            emit_lane_fault_receipt(
                &client,
                from_session,
                current_effective_agent.clone(),
                &target,
                Some(task_id.to_string()),
                args.scope.first().cloned(),
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
        let target_client = MemdClient::new(&target_base_url)?;
        let requested_scopes = if args.scope.is_empty() {
            heartbeat
                .as_ref()
                .map(|value| value.scope_claims.clone())
                .unwrap_or_default()
        } else {
            args.scope.clone()
        };
        let requested_topic = args.title.as_deref().or(heartbeat
            .as_ref()
            .and_then(|value| value.topic_claim.as_deref()));
        if let Some(error) = confirmed_hive_overlap_reason(
            &target,
            Some(task_id),
            requested_topic,
            &requested_scopes,
        ) {
            emit_lane_fault_receipt(
                &client,
                from_session,
                current_effective_agent.clone(),
                &target,
                Some(task_id.to_string()),
                requested_scopes.first().cloned(),
                current_project.clone(),
                current_namespace.clone(),
                current_workspace.clone(),
            )
            .await;
            anyhow::bail!(error);
        }

        let tasks = client
            .upsert_hive_task(&HiveTaskUpsertRequest {
                task_id: task_id.to_string(),
                title: args
                    .title
                    .clone()
                    .unwrap_or_else(|| format!("Shared task {task_id}")),
                description: args.description.clone(),
                status: Some(if args.request_help {
                    "needs_help".to_string()
                } else {
                    "needs_review".to_string()
                }),
                coordination_mode: Some(if args.request_help {
                    "help_only".to_string()
                } else {
                    "shared_review".to_string()
                }),
                session: current_session.clone(),
                agent: current_agent.clone(),
                effective_agent: current_effective_agent.clone(),
                project: current_project.clone(),
                namespace: current_namespace.clone(),
                workspace: current_workspace.clone(),
                claim_scopes: args.scope.clone(),
                help_requested: Some(args.request_help),
                review_requested: Some(args.request_review),
            })
            .await?;
        let kind = if args.request_help {
            "help_request"
        } else {
            "review_request"
        };
        let content = if args.request_help {
            format!(
                "Need help on shared task {task_id}. Please coordinate before changing overlapping work."
            )
        } else {
            format!("Need review on shared task {task_id}. Please inspect the task before handoff.")
        };
        target_client
            .send_hive_message(&HiveMessageSendRequest {
                kind: kind.to_string(),
                from_session: from_session.to_string(),
                from_agent: current_effective_agent.clone(),
                to_session: target_session.to_string(),
                project: current_project.clone(),
                namespace: current_namespace.clone(),
                workspace: current_workspace.clone(),
                content,
            })
            .await?;
        auto_checkpoint_bundle_event(
            &args.output,
            &current_base_url,
            "tasks",
            format!(
                "{} requested on shared task {task_id} from session {target_session}.",
                if args.request_help { "Help" } else { "Review" }
            ),
            vec![
                "tasks".to_string(),
                if args.request_help {
                    "help-request".to_string()
                } else {
                    "review-request".to_string()
                },
                "auto-checkpoint".to_string(),
            ],
            0.81,
        )
        .await?;
        emit_coordination_receipt(
            &client,
            if args.request_help {
                "task_help_request"
            } else {
                "task_review_request"
            },
            from_session,
            current_effective_agent.clone(),
            Some(target_session.to_string()),
            Some(task_id.to_string()),
            args.scope.first().cloned(),
            current_project.clone(),
            current_namespace.clone(),
            current_workspace.clone(),
            format!(
                "{} requested on shared task {task_id} from session {target_session}.",
                if args.request_help { "Help" } else { "Review" }
            ),
        )
        .await?;
        return Ok(TasksResponse {
            bundle_root: args.output.display().to_string(),
            current_session,
            tasks: tasks.tasks,
        });
    }

    let response = client
        .hive_tasks(&HiveTasksRequest {
            session: None,
            project: current_project,
            namespace: current_namespace,
            workspace: current_workspace,
            active_only: Some(!args.all),
            limit: Some(256),
        })
        .await?;
    let mut tasks = response.tasks;
    if let Some(view) = args.view.as_deref() {
        tasks = match view {
            "owned" => tasks
                .into_iter()
                .filter(|task| task.session.as_deref() == current_session.as_deref())
                .collect(),
            "help" => tasks
                .into_iter()
                .filter(|task| task.help_requested)
                .collect(),
            "review" => tasks
                .into_iter()
                .filter(|task| task.review_requested)
                .collect(),
            "exclusive" => tasks
                .into_iter()
                .filter(|task| task.coordination_mode == "exclusive_write")
                .collect(),
            "shared" => tasks
                .into_iter()
                .filter(|task| task.coordination_mode != "exclusive_write")
                .collect(),
            "open" => tasks
                .into_iter()
                .filter(|task| task.status != "done" && task.status != "closed")
                .collect(),
            "all" => tasks,
            _ => anyhow::bail!("unsupported tasks --view value: {view}"),
        };
    }

    Ok(TasksResponse {
        bundle_root: args.output.display().to_string(),
        current_session,
        tasks,
    })
}
