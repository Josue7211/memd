use super::*;

mod evidence;
#[allow(unused_imports)]
pub(crate) use evidence::*;

mod support;
#[allow(unused_imports)]
pub(crate) use support::*;

pub(crate) fn build_improvement_actions(
    gap: &GapReport,
    coordination: Option<&CoordinationResponse>,
) -> Vec<ImprovementAction> {
    let mut actions = Vec::new();
    let mut seen = std::collections::HashSet::<(
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )>::new();

    let add = |actions: &mut Vec<ImprovementAction>,
               seen: &mut std::collections::HashSet<(
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )>,
               action: &str,
               priority: &str,
               target_session: Option<String>,
               scope: Option<String>,
               task_id: Option<String>,
               message_id: Option<String>,
               reason: &str| {
        let key = (
            action.to_string(),
            target_session.clone(),
            scope.clone(),
            task_id.clone(),
            message_id.clone(),
            Some(reason.to_string()),
        );
        if seen.insert(key) {
            actions.push(ImprovementAction {
                action: action.to_string(),
                priority: priority.to_string(),
                target_session,
                scope,
                task_id,
                message_id,
                reason: reason.to_string(),
            });
        }
    };

    for candidate in &gap.candidates {
        match candidate.id.as_str() {
            "memory:low_eval_score"
            | "memory:below_target_eval_score"
            | "memory:no_eval_snapshot" => {
                add(
                    &mut actions,
                    &mut seen,
                    "refresh_eval",
                    "high",
                    None,
                    None,
                    None,
                    None,
                    &candidate.recommendation,
                );
            }
            "memory:weak_working_lane"
            | "memory:empty_context_lane"
            | "memory:empty_rehydration_queue"
            | "memory:missing_active_workspace_lane"
            | "memory:inbox_growth"
            | "memory:resume_state_weak"
            | "memory:resume_state_inbox_backlog"
            | "memory:missing_resume_state" => {
                add(
                    &mut actions,
                    &mut seen,
                    "refresh_resume",
                    "high",
                    None,
                    None,
                    None,
                    None,
                    &candidate.recommendation,
                );
            }
            "coordination:message_backlog" => {
                add(
                    &mut actions,
                    &mut seen,
                    "refresh_resume",
                    "medium",
                    None,
                    None,
                    None,
                    None,
                    &candidate.recommendation,
                );
            }
            "coordination:stale_remote_sessions" => {
                if candidate
                    .evidence
                    .iter()
                    .any(|value| value.starts_with("recovery=memd coordination --recover-session"))
                {
                    add(
                        &mut actions,
                        &mut seen,
                        "recover_session",
                        "high",
                        None,
                        None,
                        None,
                        None,
                        &candidate.recommendation,
                    );
                }
                if candidate
                    .evidence
                    .iter()
                    .any(|value| value.starts_with("retirement=memd coordination --retire-session"))
                {
                    add(
                        &mut actions,
                        &mut seen,
                        "retire_session",
                        "medium",
                        None,
                        None,
                        None,
                        None,
                        &candidate.recommendation,
                    );
                }
            }
            _ => {}
        }
    }

    if let Some(coordination) = coordination {
        for suggestion in &coordination.suggestions {
            match suggestion.action.as_str() {
                "ack_message" => {
                    if let Some(message_id) = suggestion.message_id.clone() {
                        add(
                            &mut actions,
                            &mut seen,
                            "ack_message",
                            &suggestion.priority,
                            None,
                            None,
                            None,
                            Some(message_id),
                            &suggestion.reason,
                        );
                    }
                }
                "recover_session" => {
                    if let Some(session) = suggestion.stale_session.clone() {
                        add(
                            &mut actions,
                            &mut seen,
                            "recover_session",
                            &suggestion.priority,
                            Some(session),
                            None,
                            None,
                            None,
                            &suggestion.reason,
                        );
                    }
                }
                "retire_session" => {
                    if let Some(session) = suggestion.stale_session.clone() {
                        add(
                            &mut actions,
                            &mut seen,
                            "retire_session",
                            &suggestion.priority,
                            Some(session),
                            None,
                            None,
                            None,
                            &suggestion.reason,
                        );
                    }
                }
                "assign_scope" => {
                    if suggestion.task_id.is_some() && suggestion.scope.is_some() {
                        add(
                            &mut actions,
                            &mut seen,
                            "assign_scope",
                            &suggestion.priority,
                            suggestion.target_session.clone(),
                            suggestion.scope.clone(),
                            suggestion.task_id.clone(),
                            None,
                            &suggestion.reason,
                        );
                    }
                }
                "request_help" | "request_review" => {
                    if suggestion.task_id.is_some() && suggestion.target_session.is_some() {
                        add(
                            &mut actions,
                            &mut seen,
                            &suggestion.action,
                            &suggestion.priority,
                            suggestion.target_session.clone(),
                            None,
                            suggestion.task_id.clone(),
                            None,
                            &suggestion.reason,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    if actions.len() > 8 {
        actions.truncate(8);
    }
    actions
}

pub(crate) async fn apply_improvement_action(
    action: &ImprovementAction,
    output: &Path,
    base_url: &str,
) -> anyhow::Result<String> {
    match action.action.as_str() {
        "refresh_eval" => {
            let response = eval_bundle_memory(
                &EvalArgs {
                    output: output.to_path_buf(),
                    limit: None,
                    rehydration_limit: None,
                    write: false,
                    fail_below: None,
                    fail_on_regression: false,
                    summary: false,
                },
                base_url,
            )
            .await?;
            Ok(format!(
                "status={} score={}",
                response.status, response.score
            ))
        }
        "refresh_resume" => {
            let runtime = read_bundle_runtime_config(output)?;
            let snapshot = crate::runtime::read_bundle_resume(
                &ResumeArgs {
                    output: output.to_path_buf(),
                    project: runtime.as_ref().and_then(|value| value.project.clone()),
                    namespace: runtime.as_ref().and_then(|value| value.namespace.clone()),
                    agent: runtime.as_ref().and_then(|value| value.agent.clone()),
                    workspace: runtime.as_ref().and_then(|value| value.workspace.clone()),
                    visibility: runtime.as_ref().and_then(|value| value.visibility.clone()),
                    route: runtime
                        .as_ref()
                        .and_then(|value| value.route.clone())
                        .or(Some("auto".to_string())),
                    intent: runtime
                        .as_ref()
                        .and_then(|value| value.intent.clone())
                        .or(Some("current_task".to_string())),
                    limit: Some(8),
                    rehydration_limit: Some(4),
                    semantic: false,
                    prompt: false,
                    summary: false,
                },
                base_url,
            )
            .await?;
            write_bundle_memory_files(output, &snapshot, None, false).await?;
            Ok(format!(
                "working={} inbox={} rehydration={}",
                snapshot.working.records.len(),
                snapshot.inbox.items.len(),
                snapshot.working.rehydration_queue.len(),
            ))
        }
        "ack_message" => {
            let response = run_messages_command(
                &MessagesArgs {
                    output: output.to_path_buf(),
                    send: false,
                    inbox: true,
                    ack: action.message_id.clone(),
                    target_session: None,
                    kind: None,
                    request_help: false,
                    request_review: false,
                    assign_scope: None,
                    scope: None,
                    content: None,
                    summary: false,
                },
                base_url,
            )
            .await?;
            Ok(format!("acked {} message(s)", response.messages.len()))
        }
        "recover_session" => {
            let response = run_coordination_command(
                &CoordinationArgs {
                    output: output.to_path_buf(),
                    view: Some("all".to_string()),
                    changes_only: false,
                    watch: false,
                    interval_secs: 30,
                    recover_session: action.target_session.clone(),
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
            Ok(format!(
                "recovered stale session pressure (stale_hives={})",
                response.recovery.stale_hives.len()
            ))
        }
        "retire_session" => {
            let target_session = action
                .target_session
                .clone()
                .context("retire_session requires a target_session")?;
            let response = run_coordination_command(
                &CoordinationArgs {
                    output: output.to_path_buf(),
                    view: Some("all".to_string()),
                    changes_only: false,
                    watch: false,
                    interval_secs: 30,
                    recover_session: None,
                    retire_session: Some(target_session.clone()),
                    to_session: None,
                    deny_session: None,
                    reroute_session: None,
                    handoff_scope: None,
                    summary: false,
                },
                base_url,
            )
            .await?;
            Ok(format!(
                "retired stale session {target_session} (stale_hives={})",
                response.recovery.stale_hives.len()
            ))
        }
        "assign_scope" => {
            let response = run_tasks_command(
                &TasksArgs {
                    output: output.to_path_buf(),
                    upsert: false,
                    assign_to_session: action.target_session.clone(),
                    target_session: None,
                    task_id: action.task_id.clone(),
                    title: None,
                    description: None,
                    status: None,
                    mode: None,
                    scope: Vec::new(),
                    request_help: false,
                    request_review: false,
                    all: false,
                    view: None,
                    summary: false,
                    json: false,
                },
                base_url,
            )
            .await?;
            Ok(format!("assigned task count={}", response.tasks.len()))
        }
        "request_help" => {
            let response = run_tasks_command(
                &TasksArgs {
                    output: output.to_path_buf(),
                    upsert: false,
                    assign_to_session: None,
                    target_session: action.target_session.clone(),
                    task_id: action.task_id.clone(),
                    title: None,
                    description: None,
                    status: None,
                    mode: None,
                    scope: Vec::new(),
                    request_help: true,
                    request_review: false,
                    all: false,
                    view: None,
                    summary: false,
                    json: false,
                },
                base_url,
            )
            .await?;
            Ok(format!(
                "requested help on {} task(s)",
                response.tasks.len()
            ))
        }
        "request_review" => {
            let response = run_tasks_command(
                &TasksArgs {
                    output: output.to_path_buf(),
                    upsert: false,
                    assign_to_session: None,
                    target_session: action.target_session.clone(),
                    task_id: action.task_id.clone(),
                    title: None,
                    description: None,
                    status: None,
                    mode: None,
                    scope: Vec::new(),
                    request_help: false,
                    request_review: true,
                    all: false,
                    view: None,
                    summary: false,
                    json: false,
                },
                base_url,
            )
            .await?;
            Ok(format!(
                "requested review on {} task(s)",
                response.tasks.len()
            ))
        }
        _ => anyhow::bail!("unknown improvement action: {}", action.action),
    }
}

pub(crate) async fn sync_resume_state_record(
    client: &MemdClient,
    project_root: Option<&Path>,
    project: Option<&str>,
    namespace: Option<&str>,
    workspace: Option<&str>,
    visibility: Option<memd_schema::MemoryVisibility>,
    effective_agent: Option<&str>,
    snapshot: &ResumeSnapshot,
) -> anyhow::Result<()> {
    let Some(store_request) = build_resume_state_store_request(
        project_root,
        project,
        namespace,
        workspace,
        visibility,
        effective_agent,
        snapshot,
    ) else {
        return Ok(());
    };

    let tags = vec!["resume_state".to_string(), "session_state".to_string()];
    let existing = client
        .search(&SearchMemoryRequest {
            query: None,
            route: Some(RetrievalRoute::SyncedOnly),
            intent: Some(RetrievalIntent::CurrentTask),
            scopes: vec![MemoryScope::Synced],
            kinds: vec![MemoryKind::Status],
            statuses: vec![MemoryStatus::Active],
            project: project.map(ToOwned::to_owned),
            namespace: namespace.map(ToOwned::to_owned),
            workspace: workspace.map(ToOwned::to_owned),
            visibility,
            belief_branch: None,
            source_agent: effective_agent.map(ToOwned::to_owned),
            tags: vec!["resume_state".to_string()],
            stages: vec![MemoryStage::Canonical],
            limit: Some(1),
            max_chars_per_item: Some(800),
        })
        .await?;

    if let Some(existing) = existing.items.first() {
        match client
            .repair(&RepairMemoryRequest {
                id: existing.id,
                mode: MemoryRepairMode::CorrectMetadata,
                confidence: Some(0.62),
                status: Some(MemoryStatus::Active),
                workspace: workspace.map(ToOwned::to_owned),
                visibility,
                source_agent: effective_agent.map(ToOwned::to_owned),
                source_system: Some("memd-resume-state".to_string()),
                source_path: project_root.map(|path| path.display().to_string()),
                source_quality: Some(memd_schema::SourceQuality::Derived),
                content: Some(store_request.content.clone()),
                tags: Some(tags),
                supersedes: Vec::new(),
            })
            .await
        {
            Ok(_) => {}
            Err(err) if err.to_string().contains("memory item not found") => {
                client.store(&store_request).await?;
            }
            Err(err) => return Err(err),
        }
    } else {
        client.store(&store_request).await?;
    }

    Ok(())
}

pub(crate) fn build_resume_state_store_request(
    project_root: Option<&Path>,
    project: Option<&str>,
    namespace: Option<&str>,
    workspace: Option<&str>,
    visibility: Option<memd_schema::MemoryVisibility>,
    effective_agent: Option<&str>,
    snapshot: &ResumeSnapshot,
) -> Option<StoreMemoryRequest> {
    let content = build_resume_state_record_content(snapshot)?;
    Some(StoreMemoryRequest {
        content,
        kind: MemoryKind::Status,
        scope: MemoryScope::Synced,
        project: project.map(ToOwned::to_owned),
        namespace: namespace.map(ToOwned::to_owned),
        workspace: workspace.map(ToOwned::to_owned),
        visibility,
        belief_branch: None,
        source_agent: effective_agent.map(ToOwned::to_owned),
        source_system: Some("memd-resume-state".to_string()),
        source_path: project_root.map(|path| path.display().to_string()),
        source_quality: Some(memd_schema::SourceQuality::Derived),
        confidence: Some(0.62),
        ttl_seconds: Some(3_600),
        last_verified_at: Some(Utc::now()),
        supersedes: Vec::new(),
        tags: vec!["resume_state".to_string(), "session_state".to_string()],
        status: Some(MemoryStatus::Active),
    })
}

pub(crate) fn build_resume_state_record_content(snapshot: &ResumeSnapshot) -> Option<String> {
    let mut lines = Vec::new();

    if let Some(focus) = snapshot.compact_working_records().first() {
        lines.push(format!("focus: {}", compact_inline(focus, 180)));
    }
    lines.push(format!("pressure: {}", snapshot.context_pressure()));
    if let Some(next) = snapshot.compact_rehydration_summaries().first() {
        lines.push(format!("next_recovery: {}", compact_inline(next, 180)));
    }
    if let Some(inbox) = snapshot.compact_inbox_items().first() {
        lines.push(format!("top_inbox: {}", compact_inline(inbox, 180)));
    }
    if let Some(change) = snapshot.recent_repo_changes.first() {
        lines.push(format!("repo_change: {}", compact_inline(change, 180)));
    }

    lines.retain(|line| !line.ends_with(": "));
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memd_schema::MemoryVisibility;

    fn sample_snapshot() -> ResumeSnapshot {
        ResumeSnapshot {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            context: memd_schema::CompactContextResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                retrieval_order: vec![MemoryScope::Project],
                records: vec![memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "remembered project fact".to_string(),
                }],
            },
            working: memd_schema::WorkingMemoryResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                retrieval_order: vec![MemoryScope::Project],
                budget_chars: 1600,
                used_chars: 120,
                remaining_chars: 1480,
                truncated: false,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 8,
                    max_chars_per_item: 220,
                    budget_chars: 1600,
                    rehydration_limit: 4,
                },
                records: vec![memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "follow durable truth".to_string(),
                }],
                evicted: Vec::new(),
                rehydration_queue: Vec::new(),
                traces: Vec::new(),
                semantic_consolidation: None,
            procedures: vec![],
            },
            inbox: memd_schema::MemoryInboxResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                items: Vec::new(),
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: Vec::new(),
            },
            sources: memd_schema::SourceMemoryResponse {
                sources: Vec::new(),
            },
            semantic: None,
            claims: SessionClaimsState::default(),
            recent_repo_changes: vec!["status M crates/memd-client/src/main.rs".to_string()],
            change_summary: Vec::new(),
            resume_state_age_minutes: None,
            refresh_recommended: false,
        }
    }

    #[test]
    fn resume_state_store_request_stays_in_synced_scope() {
        let snapshot = sample_snapshot();
        let request = build_resume_state_store_request(
            Some(Path::new("/tmp/demo")),
            Some("memd"),
            Some("main"),
            Some("team-alpha"),
            Some(MemoryVisibility::Workspace),
            Some("codex@session-1"),
            &snapshot,
        )
        .expect("build resume state request");

        assert_eq!(request.scope, MemoryScope::Synced);
        assert_eq!(request.confidence, Some(0.62));
        assert_eq!(request.ttl_seconds, Some(3_600));
        assert_eq!(request.source_system.as_deref(), Some("memd-resume-state"));
        assert!(request.tags.iter().any(|tag| tag == "resume_state"));
        assert!(request.tags.iter().any(|tag| tag == "session_state"));
    }
}

pub(crate) fn read_keyword_snippet(
    path: &Path,
    keywords: &[&str],
    max_lines: usize,
) -> Option<String> {
    let contents = read_text_file(path)?;
    let keywords = keywords
        .iter()
        .map(|keyword| keyword.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let lines = contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            keywords.iter().any(|keyword| lower.contains(keyword))
        })
        .take(max_lines)
        .collect::<Vec<_>>();
    if lines.is_empty() {
        None
    } else {
        Some(lines.join(" | "))
    }
}

pub(crate) async fn gap_report(args: &GapArgs) -> anyhow::Result<GapReport> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let project_root = project_root_from_bundle(&args.output);
    let base_url = runtime
        .as_ref()
        .and_then(|value| value.base_url.clone())
        .unwrap_or_else(default_base_url);
    let limit = args.limit.unwrap_or(8);
    let recent_commits = read_recent_commits(project_root, args.recent_commits.unwrap_or(8));
    let mut evidence = collect_gap_plan_evidence(project_root);

    if evidence.is_empty() {
        evidence.push("planning evidence unavailable in .planning".to_string());
    }

    let baseline = read_latest_gap_report(&args.output).ok().flatten();
    let eval = read_latest_bundle_eval(&args.output).ok().flatten();
    if let Some(eval) = &eval {
        evidence.push(format!(
            "eval baseline score: {} ({})",
            eval.score, eval.status
        ));
    } else {
        evidence.push("no previous memd eval snapshot in .memd/evals/latest.json".to_string());
    }

    let resume = crate::runtime::read_bundle_resume(
        &ResumeArgs {
            output: args.output.clone(),
            project: runtime.as_ref().and_then(|value| value.project.clone()),
            namespace: runtime.as_ref().and_then(|value| value.namespace.clone()),
            agent: runtime.as_ref().and_then(|value| value.agent.clone()),
            workspace: runtime.as_ref().and_then(|value| value.workspace.clone()),
            visibility: runtime.as_ref().and_then(|value| value.visibility.clone()),
            route: runtime
                .as_ref()
                .and_then(|value| value.route.clone())
                .or(Some("auto".to_string())),
            intent: runtime
                .as_ref()
                .and_then(|value| value.intent.clone())
                .or(Some("current_task".to_string())),
            limit: Some(limit),
            rehydration_limit: Some(4),
            semantic: false,
            prompt: false,
            summary: false,
        },
        &base_url,
    )
    .await
    .ok();

    let snapshot_state = read_bundle_resume_state(&args.output).ok().flatten();

    let runtime_session = runtime.as_ref().and_then(|value| value.session.clone());
    let coordination = if runtime_session.is_some() {
        run_coordination_command(
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
            &base_url,
        )
        .await
        .ok()
    } else {
        None
    };
    let awareness = read_project_awareness(&AwarenessArgs {
        output: args.output.clone(),
        root: None,
        include_current: true,
        summary: false,
    })
    .await
    .ok();
    let research_loops_doc_count = research_loops_doc_loop_count(project_root);
    let benchmark_registry = load_benchmark_registry_for_output(&args.output)
        .ok()
        .flatten()
        .map(|(_, registry)| registry);

    let candidates = build_gap_candidates(
        &args.output,
        &runtime,
        &resume,
        snapshot_state.as_ref(),
        eval.as_ref(),
        coordination.as_ref(),
        awareness.as_ref(),
        research_loops_doc_count,
        &recent_commits,
        &mut evidence,
        benchmark_registry.as_ref(),
    );
    let candidates = prioritize_gap_candidates(candidates, limit);

    let mut recommendations = candidates
        .iter()
        .take(3)
        .map(|candidate| candidate.recommendation.clone())
        .collect::<Vec<_>>();
    if recommendations.is_empty() {
        recommendations
            .push("run memd gap after collecting 12+ recent commits and a fresh eval".to_string());
    }

    if !recent_commits.is_empty() {
        evidence.push(format!("recent_commits={} checked", recent_commits.len()));
    }

    let high_priorities = candidates
        .iter()
        .filter(|candidate| candidate.severity == "high")
        .map(|candidate| candidate.id.clone())
        .collect::<Vec<_>>();
    let mut response = GapReport {
        bundle_root: args.output.display().to_string(),
        project: runtime.as_ref().and_then(|value| value.project.clone()),
        namespace: runtime.as_ref().and_then(|value| value.namespace.clone()),
        agent: runtime.as_ref().and_then(|value| value.agent.clone()),
        session: runtime_session,
        workspace: runtime.as_ref().and_then(|value| value.workspace.clone()),
        visibility: runtime.as_ref().and_then(|value| value.visibility.clone()),
        limit,
        commits_checked: recent_commits.len(),
        eval_status: eval.as_ref().map(|value| value.status.clone()),
        eval_score: eval.as_ref().map(|value| value.score),
        eval_score_delta: baseline
            .as_ref()
            .and_then(|value| value.eval_score)
            .and_then(|value| eval_score_delta(value, eval.as_ref())),
        candidate_count: candidates.len(),
        high_priority_count: high_priorities.len(),
        candidates,
        top_priorities: high_priorities,
        recommendations,
        changes: Vec::new(),
        evidence,
        generated_at: Utc::now(),
        previous_candidate_count: baseline.as_ref().map(|value| value.candidate_count),
    };
    response.changes = evaluate_gap_changes(&response, baseline.as_ref());
    Ok(response)
}

pub(crate) async fn run_improvement_loop(
    args: &ImproveArgs,
    base_url: &str,
) -> anyhow::Result<ImprovementReport> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let started_at = Utc::now();
    let mut iterations = Vec::new();
    let mut converged = false;

    let initial_report = gap_report(&GapArgs {
        output: args.output.clone(),
        limit: args.limit,
        recent_commits: args.recent_commits,
        write: false,
        summary: false,
    })
    .await?;

    let initial_snapshot = gap_to_improvement_snapshot(&initial_report);
    let mut current_gap = initial_report.clone();
    let mut final_changes = initial_report.changes.clone();
    let mut previous_gap: Option<GapReport> = Some(initial_report.clone());
    let mut final_gap: Option<GapReport> = Some(initial_report);

    for iteration in 0..args.max_iterations {
        let coordination = if runtime
            .as_ref()
            .and_then(|value| value.session.as_ref())
            .is_some()
        {
            run_coordination_command(
                &CoordinationArgs {
                    output: args.output.clone(),
                    view: Some("all".to_string()),
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
            .await
            .ok()
        } else {
            None
        };

        let mut planned_actions = build_improvement_actions(&current_gap, coordination.as_ref());
        planned_actions.truncate(6);

        let pre_gap = gap_to_improvement_snapshot(&current_gap);
        let mut executed_actions = Vec::new();

        if !args.apply || planned_actions.is_empty() {
            final_gap = Some(current_gap.clone());
            final_changes = current_gap.changes.clone();
            converged = true;
            iterations.push(ImprovementIteration {
                iteration,
                pre_gap,
                planned_actions,
                executed_actions,
                post_gap: None,
                generated_at: Utc::now(),
            });
            break;
        }

        let mut stop_due_to_failure = false;
        for action in &planned_actions {
            let result = match apply_improvement_action(action, &args.output, base_url).await {
                Ok(detail) => ImprovementActionResult {
                    action: action.action.clone(),
                    status: "applied".to_string(),
                    detail,
                },
                Err(error) => {
                    stop_due_to_failure = true;
                    ImprovementActionResult {
                        action: action.action.clone(),
                        status: "failed".to_string(),
                        detail: error.to_string(),
                    }
                }
            };
            executed_actions.push(result);
        }

        if stop_due_to_failure {
            final_gap = Some(current_gap.clone());
            final_changes = current_gap.changes.clone();
            iterations.push(ImprovementIteration {
                iteration,
                pre_gap,
                planned_actions,
                executed_actions,
                post_gap: None,
                generated_at: Utc::now(),
            });
            break;
        }

        current_gap = gap_report(&GapArgs {
            output: args.output.clone(),
            limit: args.limit,
            recent_commits: args.recent_commits,
            write: false,
            summary: false,
        })
        .await?;
        final_changes = current_gap.changes.clone();
        let post_gap = gap_to_improvement_snapshot(&current_gap);
        final_gap = Some(current_gap.clone());

        iterations.push(ImprovementIteration {
            iteration,
            pre_gap,
            planned_actions,
            executed_actions,
            post_gap: Some(post_gap),
            generated_at: Utc::now(),
        });

        if let Some(previous_gap) = previous_gap.as_ref()
            && !improvement_progress(previous_gap, &current_gap) {
                converged = true;
                break;
            }
        previous_gap = Some(current_gap.clone());

        if iteration + 1 >= args.max_iterations {
            break;
        }
    }

    let final_snapshot = final_gap
        .as_ref()
        .map(gap_to_improvement_snapshot)
        .or_else(|| Some(initial_snapshot.clone()));

    if iterations.is_empty() {
        iterations.push(ImprovementIteration {
            iteration: 0,
            pre_gap: initial_snapshot.clone(),
            planned_actions: Vec::new(),
            executed_actions: Vec::new(),
            post_gap: Some(initial_snapshot.clone()),
            generated_at: Utc::now(),
        });
        final_gap = Some(current_gap);
        final_changes = final_gap
            .as_ref()
            .map_or_else(Vec::new, |gap| gap.changes.clone());
    }
    if final_changes.is_empty()
        && let Some(gap) = final_gap.as_ref()
    {
        final_changes = gap.changes.clone();
    }

    Ok(ImprovementReport {
        bundle_root: args.output.display().to_string(),
        project: runtime.as_ref().and_then(|value| value.project.clone()),
        namespace: runtime.as_ref().and_then(|value| value.namespace.clone()),
        agent: runtime.as_ref().and_then(|value| value.agent.clone()),
        session: runtime.as_ref().and_then(|value| value.session.clone()),
        workspace: runtime.as_ref().and_then(|value| value.workspace.clone()),
        visibility: runtime.as_ref().and_then(|value| value.visibility.clone()),
        max_iterations: args.max_iterations,
        apply: args.apply,
        started_at,
        completed_at: Utc::now(),
        converged,
        initial_gap: Some(initial_snapshot),
        final_gap: final_snapshot,
        final_changes,
        iterations,
    })
}

pub(crate) async fn run_experiment_command(
    args: &ExperimentArgs,
    base_url: &str,
) -> anyhow::Result<ExperimentReport> {
    let started_at = Utc::now();
    let runtime = read_bundle_runtime_config(&args.output)?;
    let effective_max_iterations = if args.apply {
        args.max_iterations.max(1)
    } else {
        args.max_iterations
    };
    let backup_root = if args.apply {
        Some(snapshot_bundle_for_reversion(&args.output)?)
    } else {
        None
    };

    let improvement = run_improvement_loop(
        &ImproveArgs {
            output: args.output.clone(),
            max_iterations: effective_max_iterations,
            limit: args.limit,
            recent_commits: args.recent_commits,
            write: false,
            apply: args.apply,
            summary: false,
        },
        base_url,
    )
    .await?;

    let eval = read_latest_bundle_eval(&args.output).ok().flatten();
    let self_evolution_scenario = build_self_evolution_scenario_report(
        &args.output,
        runtime.as_ref(),
        &improvement,
        eval.as_ref(),
        Utc::now(),
    );
    write_scenario_artifacts(&args.output, &self_evolution_scenario)?;

    let composite = run_composite_command(
        &CompositeArgs {
            output: args.output.clone(),
            scenario: Some("self_evolution".to_string()),
            write: false,
            summary: false,
        },
        base_url,
    )
    .await?;

    let acceptance_gate = composite
        .gates
        .iter()
        .find(|gate| gate.name == "acceptance")
        .map(|gate| gate.status.as_str())
        .unwrap_or("fail");
    let hard_correctness_gate = composite
        .gates
        .iter()
        .find(|gate| gate.name == "hard_correctness")
        .map(|gate| gate.status.as_str())
        .unwrap_or("fail");
    let accepted = composite.score >= args.accept_below
        && acceptance_gate == "pass"
        && hard_correctness_gate == "pass";

    let mut restored = false;
    if args.apply && !accepted
        && let Some(backup_root) = backup_root.as_ref() {
            restore_bundle_snapshot(backup_root, &args.output)?;
            restored = true;
        }

    let mut learnings = Vec::new();
    if accepted && args.consolidate {
        learnings = derive_experiment_learnings(&improvement, &composite);
        append_experiment_learning_notes(&args.output, &learnings, &composite)?;
    }

    let mut trail = Vec::new();
    trail.push(format!(
        "improvement iterations={} apply={} max_iterations={} final_candidates={}",
        improvement.iterations.len(),
        improvement.apply,
        effective_max_iterations,
        improvement
            .final_gap
            .as_ref()
            .map(|value| value.candidate_count)
            .unwrap_or(0),
    ));
    trail.push(format!(
        "composite score={}/{} acceptance={} hard_correctness={}",
        composite.score, composite.max_score, acceptance_gate, hard_correctness_gate
    ));
    trail.push(format!(
        "decision={} accept_below={} restored={}",
        if accepted { "accepted" } else { "rejected" },
        args.accept_below,
        restored
    ));
    if !learnings.is_empty() {
        trail.push(format!("consolidated learnings={}", learnings.len()));
    }

    let mut findings = composite.findings.clone();
    if !accepted {
        findings.push("experiment rejected by bounded composite gate".to_string());
    }

    let mut recommendations = composite.recommendations.clone();
    if !accepted {
        recommendations.push(
            "tighten the improvement loop until the composite gate clears the accept threshold"
                .to_string(),
        );
    }

    let mut evidence = composite.evidence.clone();
    evidence.push(format!(
        "improvement_iterations={}",
        improvement.iterations.len()
    ));
    evidence.push(format!("accepted={accepted}"));
    if restored {
        evidence.push("bundle restored from snapshot after rejection".to_string());
    }

    Ok(ExperimentReport {
        bundle_root: args.output.display().to_string(),
        project: runtime.as_ref().and_then(|value| value.project.clone()),
        namespace: runtime.as_ref().and_then(|value| value.namespace.clone()),
        agent: runtime.as_ref().and_then(|value| value.agent.clone()),
        session: runtime.as_ref().and_then(|value| value.session.clone()),
        workspace: runtime.as_ref().and_then(|value| value.workspace.clone()),
        visibility: runtime.as_ref().and_then(|value| value.visibility.clone()),
        max_iterations: args.max_iterations,
        accept_below: args.accept_below,
        apply: args.apply,
        consolidate: args.consolidate,
        accepted,
        restored,
        started_at,
        completed_at: Utc::now(),
        improvement,
        composite,
        trail,
        learnings,
        findings,
        recommendations,
        evidence,
        evolution: None,
    })
}

pub(crate) fn build_self_evolution_scenario_report(
    output: &Path,
    runtime: Option<&BundleRuntimeConfig>,
    improvement: &ImprovementReport,
    eval: Option<&BundleEvalResponse>,
    completed_at: DateTime<Utc>,
) -> ScenarioReport {
    let mut checks = Vec::new();
    let mut findings = Vec::new();
    let mut next_actions = Vec::new();
    let mut evidence = vec![
        format!("bundle_root={}", output.display()),
        "scenario=self_evolution".to_string(),
        format!("improvement_iterations={}", improvement.iterations.len()),
        format!("final_changes={}", improvement.final_changes.len()),
    ];
    let mut passed_checks: usize = 0;
    let mut failed_checks: usize = 0;
    let mut score: u16 = 0;
    let mut max_score: u16 = 0;

    let mut add_check = |name: &str, status: &str, points: u16, details: String| {
        checks.push(ScenarioCheck {
            name: name.to_string(),
            status: status.to_string(),
            points,
            details: details.clone(),
        });
        max_score += points;
        match status {
            "pass" => {
                score += points;
                passed_checks += 1;
            }
            "warn" => {
                score += points;
                findings.push(details);
                next_actions.push(format!("improve {name} before promoting self evolution"));
            }
            _ => {
                failed_checks += 1;
                findings.push(details);
                next_actions.push(format!("resolve {name} before promoting self evolution"));
            }
        }
    };

    if !improvement.final_changes.is_empty() {
        add_check(
            "improvement_signal",
            "pass",
            28,
            format!(
                "{} final change(s) captured from improvement loop",
                improvement.final_changes.len()
            ),
        );
    } else {
        add_check(
            "improvement_signal",
            "fail",
            0,
            "no final changes captured for self evolution".to_string(),
        );
    }

    if improvement.converged {
        add_check(
            "improvement_convergence",
            "pass",
            12,
            "improvement loop converged on a proposal".to_string(),
        );
    } else if !improvement.iterations.is_empty() {
        add_check(
            "improvement_convergence",
            "warn",
            8,
            "improvement loop produced iterations but did not converge".to_string(),
        );
    } else {
        add_check(
            "improvement_convergence",
            "fail",
            0,
            "improvement loop produced no usable iterations".to_string(),
        );
    }

    let scope = classify_evolution_scope(&ExperimentReport {
        bundle_root: output.display().to_string(),
        project: runtime.and_then(|value| value.project.clone()),
        namespace: runtime.and_then(|value| value.namespace.clone()),
        agent: runtime.and_then(|value| value.agent.clone()),
        session: runtime.and_then(|value| value.session.clone()),
        workspace: runtime.and_then(|value| value.workspace.clone()),
        visibility: runtime.and_then(|value| value.visibility.clone()),
        max_iterations: improvement.max_iterations,
        accept_below: 80,
        apply: false,
        consolidate: false,
        accepted: false,
        restored: false,
        started_at: improvement.started_at,
        completed_at,
        improvement: improvement.clone(),
        composite: CompositeReport {
            bundle_root: output.display().to_string(),
            project: runtime.and_then(|value| value.project.clone()),
            namespace: runtime.and_then(|value| value.namespace.clone()),
            agent: runtime.and_then(|value| value.agent.clone()),
            session: runtime.and_then(|value| value.session.clone()),
            workspace: runtime.and_then(|value| value.workspace.clone()),
            visibility: runtime.and_then(|value| value.visibility.clone()),
            scenario: Some("self_evolution".to_string()),
            score: 100,
            max_score: 100,
            dimensions: Vec::new(),
            gates: Vec::new(),
            findings: Vec::new(),
            recommendations: Vec::new(),
            evidence: Vec::new(),
            generated_at: completed_at,
            completed_at,
        },
        trail: Vec::new(),
        learnings: Vec::new(),
        findings: Vec::new(),
        recommendations: Vec::new(),
        evidence: Vec::new(),
        evolution: None,
    });
    evidence.push(format!(
        "scope_class={} scope_gate={}",
        scope.scope_class, scope.scope_gate
    ));
    if scope.scope_gate == "auto_merge" {
        add_check(
            "proposal_scope",
            "pass",
            16,
            format!("proposal classified as {}", scope.scope_class),
        );
    } else {
        add_check(
            "proposal_scope",
            "warn",
            8,
            format!(
                "proposal classified as {} and requires review",
                scope.scope_class
            ),
        );
    }

    if let Some(eval) = eval {
        evidence.push(format!("eval score={} status={}", eval.score, eval.status));
        if eval.score >= 80 {
            add_check(
                "eval_score",
                "pass",
                16,
                format!("eval score {} meets strong target", eval.score),
            );
        } else if eval.score >= 70 {
            add_check(
                "eval_score",
                "warn",
                10,
                format!("eval score {} below strong target", eval.score),
            );
        } else {
            add_check(
                "eval_score",
                "fail",
                0,
                format!("eval score {} below stable threshold", eval.score),
            );
        }
    } else {
        add_check(
            "eval_score",
            "warn",
            8,
            "no eval snapshot found for self evolution".to_string(),
        );
    }

    ScenarioReport {
        bundle_root: output.display().to_string(),
        project: runtime.and_then(|value| value.project.clone()),
        namespace: runtime.and_then(|value| value.namespace.clone()),
        agent: runtime.and_then(|value| value.agent.clone()),
        session: runtime.and_then(|value| value.session.clone()),
        workspace: runtime.and_then(|value| value.workspace.clone()),
        visibility: runtime.and_then(|value| value.visibility.clone()),
        scenario: "self_evolution".to_string(),
        score,
        max_score,
        checks,
        passed_checks,
        failed_checks,
        findings,
        next_actions,
        evidence,
        generated_at: completed_at,
        completed_at,
    }
}
