use super::*;

pub(crate) fn simplify_awareness_work_text(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with("status:") {
        return None;
    }
    if !trimmed.contains(" | ") {
        return Some(trimmed.to_string());
    }

    let ignored_prefixes = [
        "id=",
        "stage=",
        "scope=",
        "kind=",
        "status=",
        "project=",
        "ns=",
        "vis=",
        "agent=",
        "tags=",
        "cf=",
        "upd=",
        "workspace=",
        "branch=",
        "tab=",
    ];

    trimmed
        .split(" | ")
        .map(str::trim)
        .find(|part| {
            !ignored_prefixes
                .iter()
                .any(|prefix| part.starts_with(prefix))
        })
        .map(str::to_string)
}

pub(crate) async fn resolve_target_session_bundle(
    output: &Path,
    target_session: &str,
) -> anyhow::Result<Option<ProjectAwarenessEntry>> {
    let current_project = if output.is_absolute() {
        output.to_path_buf()
    } else {
        std::env::current_dir()?.join(output)
    };
    let awareness = read_project_awareness(&AwarenessArgs {
        output: current_project,
        root: None,
        include_current: true,
        summary: false,
    })
    .await?;

    Ok(awareness.entries.into_iter().find(|entry| {
        entry.session.as_deref() == Some(target_session)
            || entry.effective_agent.as_deref() == Some(target_session)
    }))
}

pub(crate) async fn read_bundle_resume(args: &ResumeArgs, base_url: &str) -> anyhow::Result<ResumeSnapshot> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let project_root = infer_bundle_project_root(&args.output);
    let base_agent = args
        .agent
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.agent.clone()));
    let session = runtime.as_ref().and_then(|config| config.session.clone());
    let project = args
        .project
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.project.clone()));
    let namespace = args
        .namespace
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.namespace.clone()));
    let agent = base_agent
        .as_deref()
        .map(|value| compose_agent_identity(value, session.as_deref()));
    let workspace = args
        .workspace
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.workspace.clone()));
    let visibility_raw = args.visibility.clone().or_else(|| {
        runtime
            .as_ref()
            .and_then(|config| config.visibility.clone())
    });
    let route_raw = args
        .route
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.route.clone()))
        .unwrap_or_else(|| "auto".to_string());
    let intent_raw = args
        .intent
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.intent.clone()))
        .unwrap_or_else(|| "general".to_string());
    let base_url = resolve_bundle_command_base_url(
        base_url,
        runtime
            .as_ref()
            .and_then(|config| config.base_url.as_deref()),
    );
    let resume_cache_key = build_resume_snapshot_cache_key(args, runtime.as_ref(), &base_url);

    if let Some(snapshot) = cache::read_resume_snapshot_cache(&args.output, &resume_cache_key, 3)? {
        return Ok(snapshot);
    }

    let visibility = visibility_raw
        .as_deref()
        .map(parse_memory_visibility_value)
        .transpose()?;
    let route = parse_retrieval_route(Some(route_raw.clone()))?;
    let intent = parse_retrieval_intent(Some(intent_raw.clone()))?;
    let limit = args.limit.or(Some(8));
    let rehydration_limit = args.rehydration_limit.or(Some(4));

    sync_recent_repo_live_truth(
        project_root.as_deref(),
        &base_url,
        project.as_deref(),
        namespace.as_deref(),
        workspace.as_deref(),
        visibility,
    )
    .await?;

    let client = MemdClient::new(&base_url)?;
    let context = client
        .context_compact(&memd_schema::ContextRequest {
            project: project.clone(),
            agent: agent.clone(),
            workspace: workspace.clone(),
            visibility,
            route,
            intent,
            limit,
            max_chars_per_item: Some(220),
        })
        .await?;
    let working = client
        .working(&WorkingMemoryRequest {
            project: project.clone(),
            agent: agent.clone(),
            workspace: workspace.clone(),
            visibility,
            route,
            intent,
            limit,
            max_chars_per_item: Some(220),
            max_total_chars: Some(1600),
            rehydration_limit,
            auto_consolidate: Some(false),
        })
        .await?;
    let inbox = client
        .inbox(&memd_schema::MemoryInboxRequest {
            project: project.clone(),
            namespace: namespace.clone(),
            workspace: workspace.clone(),
            visibility,
            belief_branch: None,
            route,
            intent,
            limit: Some(6),
        })
        .await?;
    let workspaces = client
        .workspace_memory(&memd_schema::WorkspaceMemoryRequest {
            project: project.clone(),
            namespace: namespace.clone(),
            workspace: workspace.clone(),
            visibility,
            source_agent: None,
            source_system: None,
            limit: Some(6),
        })
        .await?;
    let sources = client
        .source_memory(&SourceMemoryRequest {
            project: project.clone(),
            namespace: namespace.clone(),
            workspace: workspace.clone(),
            visibility,
            source_agent: None,
            source_system: None,
            limit: Some(6),
        })
        .await?;
    let semantic = if let Some(rag) = maybe_rag_client_for_bundle(&args.output)? {
        if args.semantic {
            let query = build_resume_rag_query(
                project.as_deref(),
                workspace.as_deref(),
                &intent_raw,
                &working,
                &context,
            );
            if query.trim().is_empty() {
                None
            } else {
                rag.retrieve(&RagRetrieveRequest {
                    query,
                    project: project.clone(),
                    namespace: namespace.clone(),
                    mode: RagRetrieveMode::Auto,
                    limit: Some(4),
                    include_cross_modal: false,
                })
                .await
                .ok()
                .filter(|response| !response.items.is_empty())
            }
        } else {
            None
        }
    } else {
        None
    };

    let current_state = BundleResumeState {
        focus: working.records.first().map(|record| record.record.clone()),
        pressure: inbox.items.first().map(|item| item.item.content.clone()),
        next_recovery: working
            .rehydration_queue
            .first()
            .map(|item| format!("{}: {}", item.label, item.summary)),
        lane: workspaces.workspaces.first().map(|lane| {
            format!(
                "{} / {} / {}",
                lane.project.as_deref().unwrap_or("none"),
                lane.namespace.as_deref().unwrap_or("none"),
                lane.workspace.as_deref().unwrap_or("none")
            )
        }),
        working_records: working.records.len(),
        inbox_items: inbox.items.len(),
        rehydration_items: working.rehydration_queue.len(),
        recorded_at: Utc::now(),
    };
    let previous_state = read_bundle_resume_state(&args.output)?;
    let change_summary = describe_resume_state_changes(previous_state.as_ref(), &current_state);
    let resume_state_age_minutes = previous_state.as_ref().map(BundleResumeState::age_minutes);
    let refresh_recommended = resume_state_age_minutes.is_some_and(|age_minutes| age_minutes >= 15)
        || working.truncated
        || working.remaining_chars <= 200
        || working.records.len() >= 8
        || inbox.items.len() >= 5
        || working.rehydration_queue.len() >= 4
        || context.records.len() >= 6;
    let claims = read_bundle_claims(&args.output).unwrap_or_default();

    let snapshot = ResumeSnapshot {
        project,
        namespace,
        agent,
        workspace,
        visibility: visibility_raw,
        route: route_raw,
        intent: intent_raw,
        context,
        working,
        inbox,
        workspaces,
        sources,
        semantic,
        claims,
        recent_repo_changes: project_root
            .as_deref()
            .map(collect_recent_repo_changes)
            .unwrap_or_default(),
        change_summary,
        resume_state_age_minutes,
        refresh_recommended,
    };

    sync_resume_state_record(
        &client,
        project_root.as_deref(),
        snapshot.project.as_deref(),
        snapshot.namespace.as_deref(),
        snapshot.workspace.as_deref(),
        visibility,
        snapshot.agent.as_deref(),
        &snapshot,
    )
    .await?;
    let _ = cache::write_resume_snapshot_cache(&args.output, &resume_cache_key, &snapshot);

    Ok(snapshot)
}

pub(crate) async fn read_bundle_handoff(
    args: &HandoffArgs,
    base_url: &str,
) -> anyhow::Result<HandoffSnapshot> {
    let target = if let Some(target_session) = args.target_session.as_deref() {
        resolve_target_session_bundle(&args.output, target_session).await?
    } else {
        None
    };
    let target_bundle = target
        .as_ref()
        .map(|entry| PathBuf::from(&entry.bundle_root))
        .unwrap_or_else(|| args.output.clone());

    let runtime = read_bundle_runtime_config(&target_bundle)?;
    let base_url = resolve_bundle_command_base_url(
        base_url,
        runtime
            .as_ref()
            .and_then(|config| config.base_url.as_deref()),
    );
    let resume_args = ResumeArgs {
        output: target_bundle.clone(),
        project: args.project.clone(),
        namespace: args.namespace.clone(),
        agent: args.agent.clone(),
        workspace: args.workspace.clone(),
        visibility: args.visibility.clone(),
        route: args.route.clone(),
        intent: args.intent.clone(),
        limit: args.limit,
        rehydration_limit: args.rehydration_limit,
        semantic: args.semantic,
        prompt: false,
        summary: false,
    };
    let resume_cache_key =
        build_resume_snapshot_cache_key(&resume_args, runtime.as_ref(), &base_url);
    let source_limit = args.source_limit.unwrap_or(6);
    let target_bundle_key = target_bundle.display().to_string();
    let target_session_key = target
        .as_ref()
        .and_then(|entry| entry.session.clone())
        .unwrap_or_else(|| "none".to_string());
    let handoff_cache_key = cache::build_turn_key(
        Some(&target_bundle_key),
        None,
        Some(&target_session_key),
        "handoff",
        &format!(
            "resume_key={resume_cache_key}|source_limit={source_limit}|target_session={target_session_key}|target_bundle={target_bundle_key}"
        ),
    );
    if let Some(handoff) = cache::read_handoff_snapshot_cache(&args.output, &handoff_cache_key, 3)?
    {
        return Ok(handoff);
    }

    let resume = read_bundle_resume(&resume_args, &base_url).await?;

    let client = MemdClient::new(&base_url)?;
    let sources = client
        .source_memory(&SourceMemoryRequest {
            project: resume.project.clone(),
            namespace: resume.namespace.clone(),
            workspace: resume.workspace.clone(),
            visibility: resume
                .visibility
                .as_deref()
                .map(parse_memory_visibility_value)
                .transpose()?,
            source_agent: None,
            source_system: None,
            limit: Some(source_limit),
        })
        .await?;

    let handoff = HandoffSnapshot {
        generated_at: Utc::now(),
        resume,
        sources,
        target_session: target.and_then(|entry| entry.session),
        target_bundle: Some(target_bundle_key),
    };
    let _ = cache::write_handoff_snapshot_cache(&args.output, &handoff_cache_key, &handoff);
    Ok(handoff)
}

pub(crate) fn build_resume_snapshot_cache_key(
    args: &ResumeArgs,
    runtime: Option<&BundleRuntimeConfig>,
    base_url: &str,
) -> String {
    let project = args
        .project
        .as_deref()
        .or_else(|| runtime.and_then(|config| config.project.as_deref()));
    let namespace = args
        .namespace
        .as_deref()
        .or_else(|| runtime.and_then(|config| config.namespace.as_deref()));
    let agent = args
        .agent
        .as_deref()
        .or_else(|| runtime.and_then(|config| config.agent.as_deref()));
    let session = runtime.and_then(|config| config.session.as_deref());
    let tab_id = runtime.and_then(|config| config.tab_id.as_deref());
    let workspace = args
        .workspace
        .as_deref()
        .or_else(|| runtime.and_then(|config| config.workspace.as_deref()));
    let visibility = args
        .visibility
        .as_deref()
        .or_else(|| runtime.and_then(|config| config.visibility.as_deref()));
    let route = args
        .route
        .as_deref()
        .or_else(|| runtime.and_then(|config| config.route.as_deref()))
        .unwrap_or("auto");
    let intent = args
        .intent
        .as_deref()
        .or_else(|| runtime.and_then(|config| config.intent.as_deref()))
        .unwrap_or("general");
    let limit = args.limit.unwrap_or(8);
    let rehydration_limit = args.rehydration_limit.unwrap_or(4);
    let semantic = if args.semantic { "true" } else { "false" };
    let query = format!(
        "session={}|tab={}|workspace={}|visibility={}|route={}|intent={}|base_url={}|limit={}|rehydration_limit={}|semantic={}",
        session.unwrap_or("none"),
        tab_id.unwrap_or("none"),
        workspace.unwrap_or("none"),
        visibility.unwrap_or("none"),
        route,
        intent,
        base_url,
        limit,
        rehydration_limit,
        semantic
    );
    cache::build_turn_key(project, namespace, agent, "resume", &query)
}

pub(crate) fn invalidate_bundle_runtime_caches(output: &Path) -> anyhow::Result<()> {
    for path in [
        cache::resume_snapshot_cache_path(output),
        cache::handoff_snapshot_cache_path(output),
    ] {
        if path.exists() {
            fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?;
        }
    }
    Ok(())
}

pub(crate) async fn eval_bundle_memory(args: &EvalArgs, base_url: &str) -> anyhow::Result<BundleEvalResponse> {
    let baseline = read_latest_bundle_eval(&args.output)?;
    let snapshot = read_bundle_resume(
        &ResumeArgs {
            output: args.output.clone(),
            project: None,
            namespace: None,
            agent: None,
            workspace: None,
            visibility: None,
            route: None,
            intent: None,
            limit: args.limit.or(Some(8)),
            rehydration_limit: args.rehydration_limit.or(Some(4)),
            semantic: true,
            prompt: false,
            summary: false,
        },
        base_url,
    )
    .await?;

    let runtime = read_bundle_runtime_config(&args.output)?;
    let mut score = 100i32;
    let mut findings = Vec::new();

    if snapshot.working.records.is_empty() {
        score -= 30;
        findings.push("no working memory records returned from bundle resume".to_string());
    }
    if snapshot.context.records.is_empty() {
        score -= 15;
        findings.push("no compact context records returned from bundle resume".to_string());
    }
    if snapshot.working.rehydration_queue.is_empty() {
        score -= 10;
        findings.push("rehydration queue is empty; deeper evidence recovery is weak".to_string());
    }
    if snapshot.workspace.is_some() && snapshot.workspaces.workspaces.is_empty() {
        score -= 15;
        findings.push("active workspace is set but no workspace lanes were returned".to_string());
    }
    if snapshot
        .semantic
        .as_ref()
        .is_some_and(|semantic| semantic.items.is_empty())
    {
        score -= 5;
        findings.push("semantic recall is configured but returned no items".to_string());
    }
    if snapshot.inbox.items.len() >= 6 {
        score -= 10;
        findings.push("inbox pressure is high; resume lane may need maintenance".to_string());
    }

    let score = score.clamp(0, 100) as u8;
    let status = if score >= 85 {
        "strong"
    } else if score >= 65 {
        "usable"
    } else {
        "weak"
    };

    let baseline_score = baseline.as_ref().map(|value| value.score);
    let score_delta = baseline_score.map(|baseline| score as i32 - baseline as i32);
    let changes = baseline
        .as_ref()
        .map(|baseline| describe_eval_changes(baseline, score, &snapshot))
        .unwrap_or_default();
    let recommendations = build_eval_recommendations(&snapshot, score);

    Ok(BundleEvalResponse {
        bundle_root: args.output.display().to_string(),
        project: snapshot.project.clone(),
        namespace: snapshot.namespace.clone(),
        agent: snapshot
            .agent
            .clone()
            .or_else(|| runtime.as_ref().and_then(|config| config.agent.clone())),
        workspace: snapshot.workspace.clone(),
        visibility: snapshot.visibility.clone(),
        status: status.to_string(),
        score,
        working_records: snapshot.working.records.len(),
        context_records: snapshot.context.records.len(),
        rehydration_items: snapshot.working.rehydration_queue.len(),
        inbox_items: snapshot.inbox.items.len(),
        workspace_lanes: snapshot.workspaces.workspaces.len(),
        semantic_hits: snapshot
            .semantic
            .as_ref()
            .map(|semantic| semantic.items.len())
            .unwrap_or(0),
        findings,
        baseline_score,
        score_delta,
        changes,
        recommendations,
    })
}

pub(crate) fn read_latest_bundle_eval(output: &Path) -> anyhow::Result<Option<BundleEvalResponse>> {
    let path = output.join("evals").join("latest.json");
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let eval = serde_json::from_str::<BundleEvalResponse>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(eval))
}

pub(crate) fn read_latest_scenario_report(output: &Path) -> anyhow::Result<Option<ScenarioReport>> {
    let path = output.join("scenarios").join("latest.json");
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let report = serde_json::from_str::<ScenarioReport>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(report))
}

pub(crate) fn describe_eval_changes(
    baseline: &BundleEvalResponse,
    score: u8,
    snapshot: &ResumeSnapshot,
) -> Vec<String> {
    let mut changes = Vec::new();

    if baseline.score != score {
        changes.push(format!("score {} -> {}", baseline.score, score));
    }

    let working_records = snapshot.working.records.len();
    if baseline.working_records != working_records {
        changes.push(format!(
            "working {} -> {}",
            baseline.working_records, working_records
        ));
    }

    let context_records = snapshot.context.records.len();
    if baseline.context_records != context_records {
        changes.push(format!(
            "context {} -> {}",
            baseline.context_records, context_records
        ));
    }

    let rehydration_items = snapshot.working.rehydration_queue.len();
    if baseline.rehydration_items != rehydration_items {
        changes.push(format!(
            "rehydration {} -> {}",
            baseline.rehydration_items, rehydration_items
        ));
    }

    let inbox_items = snapshot.inbox.items.len();
    if baseline.inbox_items != inbox_items {
        changes.push(format!("inbox {} -> {}", baseline.inbox_items, inbox_items));
    }

    let workspace_lanes = snapshot.workspaces.workspaces.len();
    if baseline.workspace_lanes != workspace_lanes {
        changes.push(format!(
            "lanes {} -> {}",
            baseline.workspace_lanes, workspace_lanes
        ));
    }

    let semantic_hits = snapshot
        .semantic
        .as_ref()
        .map(|semantic| semantic.items.len())
        .unwrap_or(0);
    if baseline.semantic_hits != semantic_hits {
        changes.push(format!(
            "semantic {} -> {}",
            baseline.semantic_hits, semantic_hits
        ));
    }

    changes
}

pub(crate) fn eval_failure_reason(
    response: &BundleEvalResponse,
    fail_below: Option<u8>,
    fail_on_regression: bool,
) -> Option<String> {
    if let Some(threshold) = fail_below {
        if response.score < threshold {
            return Some(format!(
                "bundle evaluation score {} fell below required threshold {}",
                response.score, threshold
            ));
        }
    }

    if fail_on_regression && response.score_delta.is_some_and(|delta| delta < 0) {
        let baseline = response
            .baseline_score
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let delta = response.score_delta.unwrap_or_default();
        return Some(format!(
            "bundle evaluation regressed from baseline {} to {} (delta {})",
            baseline, response.score, delta
        ));
    }

    None
}

pub(crate) fn build_eval_recommendations(snapshot: &ResumeSnapshot, score: u8) -> Vec<String> {
    let mut recommendations = Vec::new();

    if snapshot.working.records.is_empty() {
        recommendations.push(
            "capture durable memory with `memd remember --output .memd ...` before relying on resume"
                .to_string(),
        );
    }
    if snapshot.context.records.is_empty() {
        recommendations.push(
            "review bundle route/intent defaults and verify compact context retrieval for the active lane"
                .to_string(),
        );
    }
    if snapshot.working.rehydration_queue.is_empty() {
        recommendations.push(
            "promote richer evidence or inspect key items with `memd explain --follow` so resume can rehydrate deeper context"
                .to_string(),
        );
    }
    if snapshot.workspace.is_some() && snapshot.workspaces.workspaces.is_empty() {
        recommendations.push(
            "repair workspace or visibility lanes so shared memory is visible to the active bundle"
                .to_string(),
        );
    }
    if snapshot.inbox.items.len() >= 6 {
        recommendations.push(
            "drain inbox pressure with repair or promotion passes before the next handoff or resume"
                .to_string(),
        );
    }
    if snapshot
        .semantic
        .as_ref()
        .is_some_and(|semantic| semantic.items.is_empty())
    {
        recommendations.push(
            "check the LightRAG index or sync path before depending on semantic fallback"
                .to_string(),
        );
    }
    if score < 85 {
        recommendations.push(
            "write a fresh baseline with `memd eval --output .memd --write --summary` after corrective changes"
                .to_string(),
        );
    }

    recommendations
}
