use super::*;

pub(crate) fn improvement_reports_dir(output: &Path) -> PathBuf {
    output.join("improvements")
}

pub(crate) fn scenario_reports_dir(output: &Path) -> PathBuf {
    output.join("scenarios")
}

pub(crate) fn project_root_from_bundle(output: &Path) -> &Path {
    output.parent().unwrap_or_else(|| Path::new("."))
}

pub(crate) fn read_text_file(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(crate) fn read_recent_commits(root: &Path, limit: usize) -> Vec<String> {
    let limit = limit.clamp(1, 64);
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("log")
        .arg(format!("-n{limit}"))
        .arg("--oneline")
        .output();

    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    raw.lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .take(limit)
        .collect()
}

pub(crate) fn gap_to_improvement_snapshot(response: &GapReport) -> ImprovementGapSnapshot {
    ImprovementGapSnapshot {
        candidate_count: response.candidate_count,
        high_priority_count: response.high_priority_count,
        eval_status: response.eval_status.clone(),
        eval_score: response.eval_score,
        eval_score_delta: response.eval_score_delta,
        top_priorities: response.top_priorities.clone(),
        generated_at: response.generated_at,
    }
}

pub(crate) fn improvement_progress(previous: &GapReport, current: &GapReport) -> bool {
    if current.candidate_count < previous.candidate_count {
        return true;
    }
    if current.high_priority_count < previous.high_priority_count {
        return true;
    }
    if let (Some(previous_score), Some(current_score)) = (previous.eval_score, current.eval_score) {
        if current_score > previous_score {
            return true;
        }
    } else if current.eval_score.is_some() && previous.eval_score.is_none() {
        return true;
    }
    previous.top_priorities != current.top_priorities
}

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
            let snapshot = read_bundle_resume(
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

pub(crate) fn collect_gap_plan_evidence(project_root: &Path) -> Vec<String> {
    let planning_root = project_root.join(".planning");
    let mut evidence = Vec::new();
    let mut repo_evidence = collect_gap_repo_evidence(project_root);
    let roadmap = read_text_file(&planning_root.join("ROADMAP.md"));
    let state = read_text_file(&planning_root.join("STATE.md"));
    let project = read_text_file(&planning_root.join("PROJECT.md"));

    if let Some(roadmap) = roadmap {
        let lines = roadmap
            .lines()
            .filter(|value| value.contains("Phase") && value.contains("v6"))
            .take(4)
            .collect::<Vec<_>>();
        if !lines.is_empty() {
            evidence.push(format!("roadmap phases: {}", lines.join(" | ")));
        }
    }
    if let Some(state) = state {
        if let Some(open_loops) = state
            .lines()
            .find(|line| line.starts_with("- ") && line.contains("phase"))
        {
            evidence.push(format!("state signal: {open_loops}"));
        }
        if let Some(open_block) = state.split("## Open Loops").nth(1) {
            let next = open_block
                .lines()
                .take(3)
                .filter(|value| value.starts_with("- "))
                .collect::<Vec<_>>();
            if !next.is_empty() {
                evidence.push(format!("state open loops: {}", next.join(" | ")));
            }
        }
    }
    if let Some(project) = project {
        if let Some(core) = project
            .lines()
            .find(|line| line.starts_with("##") && line.contains("Core"))
        {
            evidence.push(format!("project: {core}"));
        }
    }

    evidence.append(&mut repo_evidence);

    evidence
}

pub(crate) fn collect_gap_repo_evidence(project_root: &Path) -> Vec<String> {
    let mut evidence = Vec::new();
    let branch = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("branch")
        .arg("--show-current")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    evidence.push(format!("git branch: {branch}"));

    let status = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("status")
        .arg("--short")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .take(12)
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if status.is_empty() {
        evidence.push("git status: clean".to_string());
    } else {
        evidence.push(format!("git status: {}", status.join(" | ")));
    }

    for (path, label, keywords) in [
        (
            project_root.join("AGENTS.md"),
            "AGENTS.md",
            &["memd", "memory", "bootstrap"][..],
        ),
        (
            project_root.join("CLAUDE.md"),
            "CLAUDE.md",
            &["memd", "memory", "hook"][..],
        ),
        (
            project_root.join("MEMORY.md"),
            "MEMORY.md",
            &["memory", "memd", "decision"][..],
        ),
        (
            project_root.join("README.md"),
            "README.md",
            &["memd", "setup", "memory"][..],
        ),
        (
            project_root.join("ROADMAP.md"),
            "ROADMAP.md",
            &["v5", "v6", "memd"][..],
        ),
        (
            project_root.join("docs/setup.md"),
            "docs/setup.md",
            &["memd", "bundle", "codex"][..],
        ),
        (
            project_root.join("docs/infra-facts.md"),
            "docs/infra-facts.md",
            &["memd", "openclaw", "tailnet"][..],
        ),
        (
            project_root.join(".planning/STATE.md"),
            ".planning/STATE.md",
            &["memory", "gap", "open loop"][..],
        ),
    ] {
        if let Some(snippet) = read_keyword_snippet(&path, keywords, 4) {
            evidence.push(format!("{label}: {snippet}"));
        }
    }

    let local_bundle = project_root.join(".memd").join("config.json").exists();
    let global_bundle = home_dir()
        .map(|home| home.join(".memd").join("config.json").exists())
        .unwrap_or(false);
    evidence.push(format!(
        "memd bundles: global={} project={}",
        global_bundle, local_bundle
    ));

    let wiring = read_memd_runtime_wiring();
    let codex_wired = wiring
        .get("codex")
        .and_then(|value| value.get("wired"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let claude_wired = wiring
        .get("claude")
        .and_then(|value| value.get("wired"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let openclaw_wired = wiring
        .get("openclaw")
        .and_then(|value| value.get("wired"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let opencode_wired = wiring
        .get("opencode")
        .and_then(|value| value.get("wired"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    evidence.push(format!(
        "runtime wiring: codex={} claude={} openclaw={} opencode={}",
        codex_wired, claude_wired, openclaw_wired, opencode_wired
    ));

    evidence
}

pub(crate) fn collect_recent_repo_changes(project_root: &Path) -> Vec<String> {
    let mut changes = Vec::new();

    let status_entries = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("status")
        .arg("--short")
        .arg("--untracked-files=normal")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .take(8)
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if status_entries.is_empty() {
        changes.push("repo clean".to_string());
    } else {
        changes.extend(
            status_entries
                .into_iter()
                .map(|entry| format!("status {entry}")),
        );
    }

    let diff_stats = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("diff")
        .arg("--stat=72,40")
        .arg("--compact-summary")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .take(4)
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    changes.extend(diff_stats.into_iter().map(|entry| format!("diff {entry}")));

    changes
}

pub(crate) fn summarize_repo_event_line(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.eq_ignore_ascii_case("repo clean") {
        return "repo_state: clean".to_string();
    }

    if let Some(rest) = trimmed.strip_prefix("status ") {
        let mut parts = rest.split_whitespace();
        let code = parts.next().unwrap_or_default();
        let path = parts.collect::<Vec<_>>().join(" ");
        let label = if code.contains('?') {
            "file_created"
        } else if code.contains('D') {
            "file_deleted"
        } else if code.contains('A')
            || code.contains('M')
            || code.contains('R')
            || code.contains('C')
            || code.contains('U')
            || code.contains('T')
        {
            "file_edited"
        } else {
            "repo_change"
        };
        let detail = if path.is_empty() { code } else { path.as_str() };
        return format!("{label}: {detail}");
    }

    if let Some(rest) = trimmed.strip_prefix("diff ") {
        return format!("repo_delta: {}", rest.trim());
    }

    trimmed.to_string()
}

pub(crate) fn build_event_spine(
    change_summary: &[String],
    recent_repo_changes: &[String],
    refresh_recommended: bool,
) -> Vec<String> {
    let mut spine = Vec::new();

    for change in change_summary.iter().take(4) {
        let compact = change.trim();
        if !compact.is_empty() {
            spine.push(format!("resume_delta: {compact}"));
        }
    }

    for change in recent_repo_changes.iter().take(6) {
        let compact = summarize_repo_event_line(change);
        if !compact.is_empty() {
            spine.push(compact);
        }
    }

    if refresh_recommended {
        spine.push("compaction_due: refresh recommended for current resume state".to_string());
    }

    let mut deduped = Vec::new();
    let mut seen = std::collections::HashSet::<String>::new();
    for item in spine {
        let normalized = item
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        if normalized.is_empty() || !seen.insert(normalized) {
            continue;
        }
        deduped.push(item);
    }

    deduped.truncate(8);
    deduped
}

pub(crate) async fn sync_recent_repo_live_truth(
    project_root: Option<&Path>,
    base_url: &str,
    project: Option<&str>,
    namespace: Option<&str>,
    workspace: Option<&str>,
    visibility: Option<memd_schema::MemoryVisibility>,
) -> anyhow::Result<()> {
    let Some(project_root) = project_root else {
        return Ok(());
    };
    let Some(project) = project else {
        return Ok(());
    };

    let changes = collect_recent_repo_changes(project_root);
    let content = {
        let spine = build_event_spine(&[], &changes, false);
        if spine.is_empty() {
            "repo_state: clean".to_string()
        } else {
            spine.join("\n")
        }
    };

    let client = MemdClient::new(base_url)?;
    let live_truth_tags = vec!["live_truth".to_string(), "repo_changes".to_string()];
    let search =
        match search_live_truth_record(&client, project, namespace, workspace, visibility, false)
            .await
        {
            Ok(response) => response,
            Err(err) if is_live_truth_kind_rejection(&err) => {
                search_live_truth_record(&client, project, namespace, workspace, visibility, true)
                    .await?
            }
            Err(err) => return Err(err),
        };

    if let Some(existing) = search.items.first() {
        let repair_request = RepairMemoryRequest {
            id: existing.id,
            mode: MemoryRepairMode::CorrectMetadata,
            confidence: Some(0.98),
            status: Some(MemoryStatus::Active),
            workspace: workspace.map(ToOwned::to_owned),
            visibility,
            source_agent: Some("memd".to_string()),
            source_system: Some("memd-live-truth".to_string()),
            source_path: Some(project_root.display().to_string()),
            source_quality: Some(memd_schema::SourceQuality::Derived),
            content: Some(content.clone()),
            tags: Some(live_truth_tags.clone()),
            supersedes: Vec::new(),
        };
        match client.repair(&repair_request).await {
            Ok(_) => {}
            Err(err) if err.to_string().contains("memory item not found") => {
                store_live_truth_record(
                    &client,
                    content,
                    project,
                    namespace,
                    workspace,
                    visibility,
                    project_root,
                    live_truth_tags,
                )
                .await?;
            }
            Err(err) => return Err(err),
        }
    } else {
        store_live_truth_record(
            &client,
            content,
            project,
            namespace,
            workspace,
            visibility,
            project_root,
            live_truth_tags,
        )
        .await?;
    }

    Ok(())
}

pub(crate) fn is_live_truth_kind_rejection(error: &anyhow::Error) -> bool {
    let message = error.to_string();
    message.contains("unknown variant `live_truth`")
        || message.contains("unknown variant 'live_truth'")
        || message.contains("expected one of fact, decision, preference, runbook, procedural, self_model, topology, status, pattern, constraint")
}

pub(crate) async fn search_live_truth_record(
    client: &MemdClient,
    project: &str,
    namespace: Option<&str>,
    workspace: Option<&str>,
    visibility: Option<memd_schema::MemoryVisibility>,
    legacy_compatible: bool,
) -> anyhow::Result<memd_schema::SearchMemoryResponse> {
    let kinds = if legacy_compatible {
        Vec::new()
    } else {
        vec![MemoryKind::LiveTruth]
    };
    client
        .search(&SearchMemoryRequest {
            query: None,
            route: Some(RetrievalRoute::LocalFirst),
            intent: Some(RetrievalIntent::CurrentTask),
            scopes: vec![MemoryScope::Local],
            kinds,
            statuses: vec![MemoryStatus::Active],
            project: Some(project.to_string()),
            namespace: namespace.map(ToOwned::to_owned),
            workspace: workspace.map(ToOwned::to_owned),
            visibility,
            belief_branch: None,
            source_agent: Some("memd".to_string()),
            tags: vec!["live_truth".to_string()],
            stages: vec![MemoryStage::Canonical],
            limit: Some(1),
            max_chars_per_item: Some(800),
        })
        .await
}

pub(crate) async fn emit_lane_surface_receipt(
    client: &MemdClient,
    surface: &BundleLaneSurface,
    runtime: &BundleRuntimeConfig,
    actor_session: &str,
) -> anyhow::Result<()> {
    let (kind, summary) = if surface.action == "auto_create" {
        (
            "lane_create",
            format!(
                "Auto-created isolated hive lane from {} to {}.",
                surface.previous_branch.as_deref().unwrap_or("none"),
                surface.current_branch.as_deref().unwrap_or("none"),
            ),
        )
    } else {
        (
            "lane_reroute",
            format!(
                "Auto-rerouted hive lane from {} to {} after collision with {}.",
                surface.previous_branch.as_deref().unwrap_or("none"),
                surface.current_branch.as_deref().unwrap_or("none"),
                surface.conflict_session.as_deref().unwrap_or("unknown"),
            ),
        )
    };
    emit_coordination_receipt(
        client,
        kind,
        actor_session,
        runtime
            .agent
            .as_deref()
            .map(|agent| compose_agent_identity(agent, runtime.session.as_deref())),
        surface.conflict_session.clone(),
        None,
        surface.current_branch.clone(),
        runtime.project.clone(),
        runtime.namespace.clone(),
        runtime.workspace.clone(),
        summary,
    )
    .await
}

pub(crate) async fn emit_lane_fault_receipt(
    client: &MemdClient,
    actor_session: &str,
    actor_agent: Option<String>,
    target: &ProjectAwarenessEntry,
    task_id: Option<String>,
    scope: Option<String>,
    project: Option<String>,
    namespace: Option<String>,
    workspace: Option<String>,
) {
    let _ = emit_coordination_receipt(
        client,
        "lane_fault",
        actor_session,
        actor_agent,
        target.session.clone(),
        task_id,
        scope,
        project,
        namespace,
        workspace,
        format!(
            "Queen denied unsafe shared lane target: {}.",
            render_hive_lane_collision(target)
        ),
    )
    .await;
}

pub(crate) async fn store_live_truth_record(
    client: &MemdClient,
    content: String,
    project: &str,
    namespace: Option<&str>,
    workspace: Option<&str>,
    visibility: Option<memd_schema::MemoryVisibility>,
    project_root: &Path,
    tags: Vec<String>,
) -> anyhow::Result<()> {
    let request = StoreMemoryRequest {
        content: content.clone(),
        kind: MemoryKind::LiveTruth,
        scope: MemoryScope::Local,
        project: Some(project.to_string()),
        namespace: namespace.map(ToOwned::to_owned),
        workspace: workspace.map(ToOwned::to_owned),
        visibility,
        belief_branch: None,
        source_agent: Some("memd".to_string()),
        source_system: Some("memd-live-truth".to_string()),
        source_path: Some(project_root.display().to_string()),
        source_quality: Some(memd_schema::SourceQuality::Derived),
        confidence: Some(0.98),
        ttl_seconds: Some(3_600),
        last_verified_at: Some(Utc::now()),
        supersedes: Vec::new(),
        tags: tags.clone(),
        status: Some(MemoryStatus::Active),
    };

    match client.store(&request).await {
        Ok(_) => Ok(()),
        Err(err) if is_live_truth_kind_rejection(&err) => {
            client
                .store(&StoreMemoryRequest {
                    kind: MemoryKind::Status,
                    source_system: Some("memd-live-truth-compat".to_string()),
                    tags,
                    ..request
                })
                .await?;
            Ok(())
        }
        Err(err) => Err(err),
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
    let Some(content) = build_resume_state_record_content(snapshot) else {
        return Ok(());
    };

    let scope = if project.is_some() {
        MemoryScope::Project
    } else {
        MemoryScope::Synced
    };
    let tags = vec!["resume_state".to_string(), "session_state".to_string()];
    let existing = client
        .search(&SearchMemoryRequest {
            query: None,
            route: Some(RetrievalRoute::LocalFirst),
            intent: Some(RetrievalIntent::CurrentTask),
            scopes: vec![scope],
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
        client
            .repair(&RepairMemoryRequest {
                id: existing.id,
                mode: MemoryRepairMode::CorrectMetadata,
                confidence: Some(0.94),
                status: Some(MemoryStatus::Active),
                workspace: workspace.map(ToOwned::to_owned),
                visibility,
                source_agent: effective_agent.map(ToOwned::to_owned),
                source_system: Some("memd-resume-state".to_string()),
                source_path: project_root.map(|path| path.display().to_string()),
                source_quality: Some(memd_schema::SourceQuality::Derived),
                content: Some(content),
                tags: Some(tags),
                supersedes: Vec::new(),
            })
            .await?;
    } else {
        client
            .store(&StoreMemoryRequest {
                content,
                kind: MemoryKind::Status,
                scope,
                project: project.map(ToOwned::to_owned),
                namespace: namespace.map(ToOwned::to_owned),
                workspace: workspace.map(ToOwned::to_owned),
                visibility,
                belief_branch: None,
                source_agent: effective_agent.map(ToOwned::to_owned),
                source_system: Some("memd-resume-state".to_string()),
                source_path: project_root.map(|path| path.display().to_string()),
                source_quality: Some(memd_schema::SourceQuality::Derived),
                confidence: Some(0.94),
                ttl_seconds: Some(86_400),
                last_verified_at: Some(Utc::now()),
                supersedes: Vec::new(),
                tags,
                status: Some(MemoryStatus::Active),
            })
            .await?;
    }

    Ok(())
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

pub(crate) fn read_keyword_snippet(path: &Path, keywords: &[&str], max_lines: usize) -> Option<String> {
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

    let resume = read_bundle_resume(
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

        if let Some(previous_gap) = previous_gap.as_ref() {
            if !improvement_progress(previous_gap, &current_gap) {
                converged = true;
                break;
            }
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
    if args.apply && !accepted {
        if let Some(backup_root) = backup_root.as_ref() {
            restore_bundle_snapshot(backup_root, &args.output)?;
            restored = true;
        }
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
