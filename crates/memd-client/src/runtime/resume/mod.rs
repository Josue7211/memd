use super::*;
use crate::harness::cache;
use memd_core::file_ledger::FileInteractionLedger;
use memd_schema::MemoryStatus;

mod wakeup;

pub(crate) mod compiler;

pub(crate) fn collect_files_touched(output: &Path) -> Vec<String> {
    let state = output.join("state");
    let Ok(rd) = std::fs::read_dir(&state) else {
        return Vec::new();
    };

    let mut latest: Option<(std::time::SystemTime, std::path::PathBuf)> = None;
    let consider =
        |p: std::path::PathBuf,
         latest: &mut Option<(std::time::SystemTime, std::path::PathBuf)>| {
            if let Ok(meta) = std::fs::metadata(&p) {
                if let Ok(mt) = meta.modified() {
                    if latest.as_ref().map_or(true, |(l, _)| mt > *l) {
                        *latest = Some((mt, p));
                    }
                }
            }
        };

    for entry in rd.flatten() {
        if !entry.file_name().to_string_lossy().starts_with("session-") {
            continue;
        }
        let sealed = entry.path().join("sealed");
        if let Ok(sd) = std::fs::read_dir(&sealed) {
            for s in sd.flatten() {
                consider(s.path(), &mut latest);
            }
        }
        let live = entry.path().join("file_interactions.json");
        if live.exists() {
            consider(live, &mut latest);
        }
    }

    latest
        .and_then(|(_, path)| FileInteractionLedger::load_from_path(&path).ok())
        .map(|l| l.distinct_paths())
        .unwrap_or_default()
}

pub(crate) fn collect_un_read_paths(output: &Path, session_id: &str) -> Vec<String> {
    let sealed = memd_core::enforcement::load_latest_sealed_paths(output);
    let fresh = memd_core::enforcement::FreshReadIndex::for_session(output, session_id);
    sealed.into_iter().filter(|p| !fresh.contains(p)).collect()
}

#[allow(unused_imports)]
pub(crate) use crate::workflow::*;
#[allow(unused_imports)]
pub(crate) use wakeup::*;

fn infer_resume_bundle_identity_defaults(output: &Path) -> (Option<String>, Option<String>) {
    infer_bundle_identity_defaults(output)
}

fn normalize_resume_record(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn compact_record_kind(record: &str) -> Option<&'static str> {
    if record.contains("| kind=status |") {
        Some("status")
    } else if record.contains("| kind=live_truth |") {
        Some("live_truth")
    } else {
        None
    }
}

fn working_eviction_count(working: &memd_schema::WorkingMemoryResponse) -> usize {
    let reported_count = working
        .compaction_quality
        .as_ref()
        .map(|quality| quality.evicted)
        .unwrap_or_else(|| working.evicted.len());

    if working.evicted.is_empty() {
        return reported_count;
    }

    let detailed_loss = working
        .evicted
        .iter()
        .filter(|evicted| !is_refresh_status_eviction(&evicted.record, &evicted.reason))
        .count();
    detailed_loss + reported_count.saturating_sub(working.evicted.len())
}

fn working_has_loss(working: &memd_schema::WorkingMemoryResponse) -> bool {
    working.truncated || working_eviction_count(working) > 0
}

fn is_refresh_status_eviction(record: &str, reason: &str) -> bool {
    if !reason.contains("evicted_by_status_cap") || !is_status_record_text(record) {
        return false;
    }
    is_recoverable_status_decision(record) || is_recoverable_status_fact(record)
}

fn resume_refresh_recommended(
    resume_state_age_minutes: Option<i64>,
    working: &memd_schema::WorkingMemoryResponse,
    inbox: &memd_schema::MemoryInboxResponse,
) -> bool {
    let working_loss = working_has_loss(working);
    let admission_limit = working.policy.admission_limit.max(1);
    let admission_loss = working.records.len() >= admission_limit && working_loss;
    let tight_budget_loss = working.remaining_chars <= 200 && working_loss;

    resume_state_age_minutes.is_some_and(|age_minutes| age_minutes >= 15)
        || working.truncated
        || tight_budget_loss
        || admission_loss
        || inbox.items.len() >= 5
        || working.rehydration_queue.len() >= 4
}

fn trim_resume_context_records(
    context: &mut memd_schema::CompactContextResponse,
    working: &memd_schema::WorkingMemoryResponse,
) {
    let working_records = working
        .records
        .iter()
        .map(|record| normalize_resume_record(&record.record))
        .collect::<std::collections::HashSet<_>>();
    let has_non_status_records = context
        .records
        .iter()
        .any(|record| compact_record_kind(&record.record) != Some("status"));
    let mut kept = Vec::with_capacity(context.records.len());
    let mut seen = std::collections::HashSet::<String>::new();
    let mut kept_live_truth = false;

    for record in context.records.drain(..) {
        let normalized = normalize_resume_record(&record.record);
        if normalized.is_empty() || !seen.insert(normalized.clone()) {
            continue;
        }
        if working_records.contains(&normalized) {
            continue;
        }
        match compact_record_kind(&record.record) {
            Some("status") if has_non_status_records => continue,
            Some("live_truth") if kept_live_truth => continue,
            Some("live_truth") => kept_live_truth = true,
            _ => {}
        }
        kept.push(record);
    }

    context.records = kept;
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

pub(crate) async fn read_bundle_resume(
    args: &ResumeArgs,
    base_url: &str,
) -> anyhow::Result<ResumeSnapshot> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let project_root = infer_bundle_project_root(&args.output);
    let (default_project, default_namespace) = infer_resume_bundle_identity_defaults(&args.output);
    let base_agent = args
        .agent
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.agent.clone()));
    let session = runtime.as_ref().and_then(|config| config.session.clone());
    let project = args
        .project
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.project.clone()))
        .or(default_project);
    let namespace = args
        .namespace
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.namespace.clone()))
        .or(default_namespace);
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

    if let Some(mut snapshot) =
        cache::read_resume_snapshot_cache(&args.output, &resume_cache_key, 3)?
    {
        refresh_resume_local_recovery_state(project_root.as_deref(), &mut snapshot);
        if let Some(raw_next) = latest_raw_spine_next_action(&args.output)
            && !snapshot
                .preferences
                .iter()
                .any(|record| record == &raw_next)
        {
            snapshot.preferences.push(raw_next);
        }
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
    let mut context = client
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
            query: None,
        })
        .await?;
    trim_resume_context_records(&mut context, &working);
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
    let refresh_recommended =
        resume_refresh_recommended(resume_state_age_minutes, &working, &inbox);
    let claims = read_bundle_claims(&args.output).unwrap_or_default();

    // E2: Atlas region hints for wake packet
    let atlas_region_hints = client
        .atlas_regions(&memd_schema::AtlasRegionsRequest {
            project: project.clone(),
            namespace: namespace.clone(),
            lane: None,
            limit: Some(5),
        })
        .await
        .ok()
        .map(|response| {
            response
                .regions
                .iter()
                .map(|r| format!("{} ({})", r.name, r.node_count))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut handoff_quality = working
        .compaction_quality
        .as_ref()
        .map(HandoffQualityScore::from_report);

    let un_read_paths = session
        .as_deref()
        .map(|sid| collect_un_read_paths(&args.output, sid))
        .unwrap_or_default();

    // A3 Part 2 Task 11: Fetch preferences as a separate retrieval intent.
    // RetrievalIntent::Preference maps to [MemoryKind::Preference, MemoryKind::Decision],
    // so we may get Decisions leaked in; this is acceptable (they're high-signal durable memory).
    let pref_intent = parse_retrieval_intent(Some("preference".to_string()))
        .unwrap_or(Some(memd_schema::RetrievalIntent::CurrentTask));
    let mut preferences = match client
        .context_compact(&memd_schema::ContextRequest {
            project: project.clone(),
            agent: agent.clone(),
            workspace: workspace.clone(),
            visibility,
            route,
            intent: pref_intent,
            limit: Some(3),
            max_chars_per_item: Some(220),
        })
        .await
    {
        Ok(resp) => resp.records.into_iter().take(3).map(|r| r.record).collect(),
        Err(_) => Vec::new(), // fail-soft: no preferences = empty block
    };
    if let Some(raw_next) = latest_raw_spine_next_action(&args.output)
        && !preferences.iter().any(|record| record == &raw_next)
    {
        preferences.push(raw_next);
    }
    if let Some(score) = &mut handoff_quality {
        let signals = recoverable_signal_counts(&context, &working, &preferences);
        score.include_recoverable_signals(signals.facts, signals.decisions, signals.total);
    }

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
        atlas_region_hints,
        handoff_quality,
        files_touched: collect_files_touched(&args.output),
        un_read_paths,
        preferences,
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

pub(crate) fn render_bundle_memory_markdown(
    output: &Path,
    snapshot: &ResumeSnapshot,
    handoff: Option<&HandoffSnapshot>,
    hive: Option<&BundleHiveMemorySurface>,
) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# memd memory{}\n\n",
        render_memory_page_header_suffix(output)
    ));
    markdown.push_str(&render_bundle_scope_markdown(output, snapshot));
    markdown.push('\n');

    markdown.push_str("\n## Budget\n\n");
    markdown.push_str(&format!(
        "- tok={} | ch={} | p={} | dup={} | use={}/{} | refresh={} | action=\"{}\"\n",
        snapshot.estimated_prompt_tokens(),
        snapshot.estimated_prompt_chars(),
        snapshot.context_pressure(),
        snapshot.redundant_context_items(),
        snapshot.working.used_chars,
        snapshot.working.budget_chars,
        snapshot.refresh_recommended,
        snapshot.memory_action_hint(),
    ));
    let drivers = snapshot.memory_pressure_drivers();
    markdown.push_str(&format!(
        "- drivers={}\n",
        if drivers.is_empty() {
            "none".to_string()
        } else {
            drivers.join(",")
        }
    ));

    markdown.push_str("\n## Durable Truth\n\n");
    if snapshot.context.records.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for record in snapshot.context.records.iter().take(3) {
            markdown.push_str(&format!(
                "- {}\n",
                compact_inline(record.record.trim(), 160)
            ));
        }
        if snapshot.context.records.len() > 3 {
            markdown.push_str(&format!(
                "- (+{} more)\n",
                snapshot.context.records.len() - 3
            ));
        }
    }

    let current_task = render_current_task_bundle_snapshot(snapshot);
    if !current_task.is_empty() {
        markdown.push_str("\n## Read First\n\n");
        markdown.push_str(&current_task);
        if let Some(focus) = snapshot.working.records.first() {
            markdown.push_str(&format!(
                "- focus={}\n",
                compact_inline(focus.record.trim(), 120)
            ));
        }
        if let Some(next) = snapshot.working.rehydration_queue.first() {
            markdown.push_str(&format!(
                "- next={}: {}\n",
                next.label,
                compact_inline(next.summary.trim(), 120)
            ));
        }
        if let Some(blocker) = snapshot.inbox.items.first() {
            markdown.push_str(&format!(
                "- blocker={:?}/{:?}: {}\n",
                blocker.item.kind,
                blocker.item.status,
                compact_inline(blocker.item.content.trim(), 120)
            ));
        }
    }

    markdown.push_str("\n## Voice\n\n");
    markdown.push_str(&render_voice_mode_section(
        &read_bundle_voice_mode(output).unwrap_or_else(default_voice_mode),
    ));
    markdown.push('\n');

    markdown.push_str("\n## Memory Objects\n\n");
    if let Some(record) = snapshot.context.records.first() {
        markdown.push_str(&format!(
            "- context id={} record=\"{}\"\n",
            short_uuid(record.id),
            compact_inline(record.record.trim(), 120)
        ));
        if let Some(slug) = memory_object_item_slug(snapshot, MemoryObjectLane::Context, 0) {
            markdown.push_str(&format!("- [open](items/context/{slug})\n"));
        }
    } else {
        markdown.push_str("- context none\n");
    }
    if let Some(record) = snapshot.working.records.first() {
        markdown.push_str(&format!(
            "- working id={} record=\"{}\"\n",
            short_uuid(record.id),
            compact_inline(record.record.trim(), 120)
        ));
        if let Some(slug) = memory_object_item_slug(snapshot, MemoryObjectLane::Working, 0) {
            markdown.push_str(&format!("- [open](items/working/{slug})\n"));
        }
    } else {
        markdown.push_str("- working none\n");
    }
    if let Some(item) = snapshot.inbox.items.first() {
        markdown.push_str(&format!(
            "- inbox id={} kind={} type={} status={} stage={} cf={:.2} scope={} source={} note=\"{}\"\n",
            short_uuid(item.item.id),
            enum_label_kind(item.item.kind),
            typed_memory_label(item.item.kind, item.item.stage),
            enum_label_status(item.item.status),
            format!("{:?}", item.item.stage).to_ascii_lowercase(),
            item.item.confidence,
            format!("{:?}", item.item.scope).to_ascii_lowercase(),
            ResumeSnapshot::source_label(
                item.item.source_agent.as_deref(),
                item.item.source_system.as_deref(),
                item.item.source_path.as_deref()
            ),
            compact_inline(item.item.content.trim(), 120)
        ));
        if let Some(slug) = memory_object_item_slug(snapshot, MemoryObjectLane::Inbox, 0) {
            markdown.push_str(&format!("- [open](items/inbox/{slug})\n"));
        }
        if !item.reasons.is_empty() {
            markdown.push_str(&format!(
                "- inbox_reasons={}\n",
                item.reasons
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    } else {
        markdown.push_str("- inbox none\n");
    }
    if let Some(artifact) = snapshot.working.rehydration_queue.first() {
        markdown.push_str(&format!(
            "- recovery id={} kind={} label=\"{}\" source={} reason=\"{}\"\n",
            artifact
                .id
                .map(short_uuid)
                .unwrap_or_else(|| "none".to_string()),
            artifact.kind,
            compact_inline(&artifact.label, 64),
            ResumeSnapshot::source_label(
                artifact.source_agent.as_deref(),
                artifact.source_system.as_deref(),
                artifact.source_path.as_deref()
            ),
            artifact
                .reason
                .as_deref()
                .map(|value| compact_inline(value, 120))
                .unwrap_or_else(|| "none".to_string())
        ));
        if let Some(slug) = memory_object_item_slug(snapshot, MemoryObjectLane::Recovery, 0) {
            markdown.push_str(&format!("- [open](items/recovery/{slug})\n"));
        }
    } else {
        markdown.push_str("- recovery none\n");
    }
    if let Some(semantic) = snapshot
        .semantic
        .as_ref()
        .filter(|semantic| !semantic.items.is_empty())
        .and_then(|semantic| semantic.items.first())
    {
        markdown.push_str(&format!(
            "- semantic score={:.2} content=\"{}\"\n",
            semantic.score,
            compact_inline(&semantic.content, 120)
        ));
        if let Some(slug) = memory_object_item_slug(snapshot, MemoryObjectLane::Semantic, 0) {
            markdown.push_str(&format!("- [open](items/semantic/{slug})\n"));
        }
    } else {
        markdown.push_str("- semantic none\n");
    }
    if let Some(first) = snapshot.workspaces.workspaces.first() {
        markdown.push_str(&format!(
            "- workspace project={} namespace={} workspace={} visibility={} items={} active={} contested={} trust={:.2} cf={:.2}\n",
            first.project.as_deref().unwrap_or("none"),
            first.namespace.as_deref().unwrap_or("none"),
            first.workspace.as_deref().unwrap_or("none"),
            memory_visibility_label(first.visibility),
            first.item_count,
            first.active_count,
            first.contested_count,
            first.trust_score,
            first.avg_confidence
        ));
        if let Some(slug) = memory_object_item_slug(snapshot, MemoryObjectLane::Workspace, 0) {
            markdown.push_str(&format!("- [open](items/workspace/{slug})\n"));
        }
    } else {
        markdown.push_str("- workspace none\n");
    }

    let event_spine = snapshot.event_spine();
    if !event_spine.is_empty() || !snapshot.recent_repo_changes.is_empty() {
        markdown.push_str("\n## E+LT\n\n");
        let event_part = if event_spine.is_empty() {
            None
        } else {
            let summary = event_spine
                .iter()
                .take(2)
                .map(|change| change.trim())
                .collect::<Vec<_>>()
                .join(" | ");
            Some(format!("- E={summary}"))
        };
        let lt_part = if snapshot.recent_repo_changes.is_empty() {
            None
        } else {
            let summary = snapshot
                .recent_repo_changes
                .iter()
                .take(2)
                .map(|change| change.trim())
                .collect::<Vec<_>>()
                .join(" | ");
            Some(format!("- LT={summary}"))
        };
        let mut parts = Vec::new();
        if let Some(part) = event_part {
            parts.push(part);
        }
        if let Some(part) = lt_part {
            parts.push(part);
        }
        markdown.push_str(&format!("- {}\n", parts.join(" | ")));
    }

    markdown.push_str("\n## W\n\n");
    if snapshot.working.records.is_empty() {
        markdown.push_str("- none\n");
    } else {
        let records = snapshot
            .working
            .records
            .iter()
            .take(2)
            .map(|record| record.record.trim())
            .collect::<Vec<_>>();
        markdown.push_str(&format!("- w={}", records.join(" | ")));
        if snapshot.working.records.len() > 2 {
            markdown.push_str(&format!(" (+{} more)", snapshot.working.records.len() - 2));
        }
        markdown.push('\n');
    }

    let mut ri_parts = Vec::new();
    if !snapshot.working.rehydration_queue.is_empty() {
        for artifact in snapshot.working.rehydration_queue.iter().take(6) {
            ri_parts.push(format!("r={}:{}", artifact.label, artifact.summary.trim()));
        }
    }
    if !snapshot.inbox.items.is_empty() {
        for item in snapshot.inbox.items.iter().take(6) {
            ri_parts.push(format!(
                "i={:?}/{:?}:{}",
                item.item.kind,
                item.item.status,
                item.item.content.trim()
            ));
            if !item.reasons.is_empty() {
                ri_parts.push(format!("r={}", item.reasons.join(", ")));
            }
        }
    }
    if !ri_parts.is_empty() {
        markdown.push_str("\n## RI\n\n");
        markdown.push_str(&format!("- {}\n", ri_parts.join(" | ")));
    }

    if let Some(first) = snapshot.workspaces.workspaces.first() {
        markdown.push_str("\n## L\n\n");
        let extras = snapshot.workspaces.workspaces.len() - 1;
        markdown.push_str(&format!(
            "- l={}/{}/{} | v={} | it={} | tr={:.2}{} \n",
            first.project.as_deref().unwrap_or("none"),
            first.namespace.as_deref().unwrap_or("none"),
            first.workspace.as_deref().unwrap_or("none"),
            memory_visibility_label(first.visibility),
            first.item_count,
            first.trust_score,
            if extras > 0 {
                format!(" (+{} more)", extras)
            } else {
                "".to_string()
            }
        ));
    }

    let mut sc_parts = Vec::new();
    if let Some(semantic) = snapshot
        .semantic
        .as_ref()
        .filter(|semantic| !semantic.items.is_empty())
    {
        let items = semantic
            .items
            .iter()
            .take(2)
            .map(|item| {
                format!(
                    "{}@{:.2}",
                    compact_resume_rag_text(&item.content, 220),
                    item.score
                )
            })
            .collect::<Vec<_>>();
        sc_parts.push(format!("S={}", items.join(" | ")));
    }

    if let Some(handoff) = handoff {
        if !handoff.sources.sources.is_empty() {
            let sources = handoff
                .sources
                .sources
                .iter()
                .take(3)
                .map(|source| {
                    format!(
                        "{}({})@{:.2}",
                        source.source_agent.as_deref().unwrap_or("none"),
                        source.workspace.as_deref().unwrap_or("none"),
                        source.trust_score
                    )
                })
                .collect::<Vec<_>>();
            sc_parts.push(format!("C={}", sources.join(" | ")));
        }
        markdown.push_str("\n## Handoff Notes\n\n");
        markdown.push_str("- this file was refreshed from a shared handoff bundle\n");
        markdown.push_str("- dream/consolidation output should feed this same file so durable memory and distilled memory stay aligned\n");
    }

    if !sc_parts.is_empty() {
        markdown.push_str("\n## S+C\n\n");
        markdown.push_str(&format!("- {}\n", sc_parts.join(" | ")));
    }

    if let Some(hive) = hive {
        markdown.push_str("\n## Hive\n\n");
        markdown.push_str(&format!(
            "- queen={} roster={} active={} review={} overlap={} stale={}\n",
            hive.board.queen_session.as_deref().unwrap_or("none"),
            hive.roster.bees.len(),
            hive.board.active_bees.len(),
            hive.board.review_queue.len(),
            hive.board.overlap_risks.len(),
            hive.board.stale_bees.len(),
        ));
        if !hive.board.active_bees.is_empty() {
            let active = hive
                .board
                .active_bees
                .iter()
                .take(3)
                .map(|bee| {
                    format!(
                        "{}({})/{}",
                        bee.worker_name
                            .as_deref()
                            .or(bee.agent.as_deref())
                            .unwrap_or("unnamed"),
                        bee.session,
                        bee.task_id.as_deref().unwrap_or("none")
                    )
                })
                .collect::<Vec<_>>();
            markdown.push_str(&format!("- active_bees={}\n", active.join(" | ")));
        }
        if let Some(follow) = hive.follow.as_ref() {
            markdown.push_str(&format!(
                "- focus={} work=\"{}\" touches={} next=\"{}\" action={}\n",
                follow
                    .target
                    .worker_name
                    .as_deref()
                    .or(follow.target.agent.as_deref())
                    .unwrap_or(follow.target.session.as_str()),
                compact_inline(&follow.work_summary, 120),
                if follow.touch_points.is_empty() {
                    "none".to_string()
                } else {
                    compact_inline(&follow.touch_points.join(","), 120)
                },
                follow.next_action.as_deref().unwrap_or("none"),
                follow.recommended_action,
            ));
        }
        if !hive.board.recommended_actions.is_empty() {
            markdown.push_str(&format!(
                "- recommended={}\n",
                hive.board
                    .recommended_actions
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" | ")
            ));
        }
    }

    markdown.push_str("\n## Event Compiler\n\n");
    markdown.push_str("- live event log: [events.md](events.md)\n");
    markdown.push_str(
        "- compiled event pages: [compiled/events/latest.md](compiled/events/latest.md)\n",
    );
    markdown.push_str(
        "- memory updates now flow through the event compiler before the visible pages refresh\n",
    );

    markdown.push_str("\n## Memory Pages\n\n");
    for lane in [
        MemoryObjectLane::Context,
        MemoryObjectLane::Working,
        MemoryObjectLane::Inbox,
        MemoryObjectLane::Recovery,
        MemoryObjectLane::Semantic,
        MemoryObjectLane::Workspace,
    ] {
        markdown.push_str(&format!(
            "- [{}](compiled/memory/{}.md)\n",
            lane.title(),
            lane.slug()
        ));
    }

    markdown
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
        project: args
            .project
            .clone()
            .or_else(|| infer_resume_bundle_identity_defaults(&target_bundle).0),
        namespace: args
            .namespace
            .clone()
            .or_else(|| infer_resume_bundle_identity_defaults(&target_bundle).1),
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
    if let Some(mut handoff) =
        cache::read_handoff_snapshot_cache(&args.output, &handoff_cache_key, 3)?
    {
        refresh_handoff_local_recovery_state(&target_bundle, &mut handoff);
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
        voice_mode: read_bundle_voice_mode(&target_bundle).unwrap_or_else(default_voice_mode),
        target_session: target.and_then(|entry| entry.session),
        target_bundle: Some(target_bundle_key),
    };
    let _ = cache::write_handoff_snapshot_cache(&args.output, &handoff_cache_key, &handoff);
    Ok(handoff)
}

fn refresh_resume_local_recovery_state(project_root: Option<&Path>, snapshot: &mut ResumeSnapshot) {
    if let Some(project_root) = project_root {
        snapshot.recent_repo_changes = collect_recent_repo_changes(project_root);
    }
    if snapshot.handoff_quality.is_none()
        && let Some(report) = snapshot.working.compaction_quality.as_ref()
    {
        let mut score = HandoffQualityScore::from_report(report);
        let signals =
            recoverable_signal_counts(&snapshot.context, &snapshot.working, &snapshot.preferences);
        score.include_recoverable_signals(signals.facts, signals.decisions, signals.total);
        snapshot.handoff_quality = Some(score);
    }
}

fn refresh_handoff_local_recovery_state(target_bundle: &Path, handoff: &mut HandoffSnapshot) {
    let project_root = infer_bundle_project_root(target_bundle);
    refresh_resume_local_recovery_state(project_root.as_deref(), &mut handoff.resume);
    handoff.voice_mode = read_bundle_voice_mode(target_bundle).unwrap_or_else(default_voice_mode);
}

pub(crate) fn read_bundle_resume_state(output: &Path) -> anyhow::Result<Option<BundleResumeState>> {
    let path = bundle_resume_state_path(output);
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let state = serde_json::from_str::<BundleResumeState>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(state))
}

pub(crate) fn write_bundle_resume_state(
    output: &Path,
    snapshot: &ResumeSnapshot,
) -> anyhow::Result<()> {
    let path = bundle_resume_state_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let state = BundleResumeState::from_snapshot(snapshot);
    fs::write(&path, serde_json::to_string_pretty(&state)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BundleResumeState {
    pub(crate) focus: Option<String>,
    pub(crate) pressure: Option<String>,
    pub(crate) next_recovery: Option<String>,
    pub(crate) lane: Option<String>,
    pub(crate) working_records: usize,
    pub(crate) inbox_items: usize,
    pub(crate) rehydration_items: usize,
    #[serde(default = "Utc::now")]
    pub(crate) recorded_at: DateTime<Utc>,
}

impl BundleResumeState {
    pub(crate) fn from_snapshot(snapshot: &ResumeSnapshot) -> Self {
        Self {
            focus: snapshot
                .working
                .records
                .first()
                .map(|record| record.record.clone()),
            pressure: snapshot
                .inbox
                .items
                .first()
                .map(|item| item.item.content.clone()),
            next_recovery: snapshot
                .working
                .rehydration_queue
                .first()
                .map(|item| format!("{}: {}", item.label, item.summary)),
            lane: snapshot.workspaces.workspaces.first().map(|lane| {
                format!(
                    "{} / {} / {}",
                    lane.project.as_deref().unwrap_or("none"),
                    lane.namespace.as_deref().unwrap_or("none"),
                    lane.workspace.as_deref().unwrap_or("none")
                )
            }),
            working_records: snapshot.working.records.len(),
            inbox_items: snapshot.inbox.items.len(),
            rehydration_items: snapshot.working.rehydration_queue.len(),
            recorded_at: Utc::now(),
        }
    }

    pub(crate) fn age_minutes(&self) -> i64 {
        (Utc::now() - self.recorded_at).num_minutes()
    }
}

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HandoffQualityScore {
    /// Fraction of candidates admitted (0.0–1.0).
    pub(crate) fill_rate: f64,
    /// Fraction of budget chars consumed (0.0–1.0).
    pub(crate) budget_utilization: f64,
    /// Kind with the most admitted items.
    pub(crate) dominant_kind: Option<String>,
    /// Fraction of candidates evicted (complement of fill_rate).
    pub(crate) eviction_pressure: f64,
    /// L2.9: admitted-fact count / target (target = 3). Capped at 1.0.
    #[serde(default)]
    pub(crate) fact_coverage: f64,
    /// L2.9: admitted-decision count / target (target = 2). Capped at 1.0.
    #[serde(default)]
    pub(crate) decision_coverage: f64,
    /// L2.9: working-context depth = admitted / target (target = 8). Capped at 1.0.
    #[serde(default)]
    pub(crate) working_depth: f64,
    /// L2.9: composite weighted score across 4 dimensions. Range 0.0–1.0.
    #[serde(default)]
    pub(crate) composite: f64,
}

impl HandoffQualityScore {
    /// L2.9: minimum composite that counts as a "complete" handoff.
    pub(crate) const ACCEPTANCE_THRESHOLD: f64 = 0.8;
    const TARGET_FACTS: f64 = 3.0;
    const TARGET_DECISIONS: f64 = 2.0;
    const TARGET_WORKING_DEPTH: f64 = 8.0;

    pub(crate) fn from_report(report: &memd_schema::CompactionQualityReport) -> Self {
        let total = report.admitted + report.evicted;
        let fill_rate = if total > 0 {
            report.admitted as f64 / total as f64
        } else {
            1.0
        };
        let budget_utilization = if report.budget_chars > 0 {
            (report.used_chars as f64 / report.budget_chars as f64).min(1.0)
        } else {
            0.0
        };
        let dominant_kind = report
            .per_kind_admitted
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(kind, _)| kind.clone());

        // L2.9: new dimensions. per_kind keys are JSON-encoded
        // ("\"fact\"") or bare ("fact") depending on upstream. Match both.
        let kind_count = |kind: &str| -> usize {
            report
                .per_kind_admitted
                .iter()
                .filter(|(k, _)| {
                    let trimmed = k.trim_matches('"');
                    trimmed.eq_ignore_ascii_case(kind)
                })
                .map(|(_, c)| *c)
                .sum()
        };
        let fact_coverage = (kind_count("fact") as f64 / Self::TARGET_FACTS).min(1.0);
        let decision_coverage = (kind_count("decision") as f64 / Self::TARGET_DECISIONS).min(1.0);
        let working_depth = (report.admitted as f64 / Self::TARGET_WORKING_DEPTH).min(1.0);

        // Trust distribution proxy: budget_utilization stays in-band when the
        // outgoing harness filled but didn't truncate. Penalize both
        // starvation (utilization < 0.25) and truncation (utilization ~= 1.0
        // plus high eviction). For handoff purposes, we want the packet
        // *rich* but not *overstuffed*, so anything in [0.5, 0.95] scores 1.0.
        let trust_score = Self::trust_score_for_budget_utilization(budget_utilization);

        // Weighted composite: coverage of the substantive kinds matters
        // most, working-depth second, trust third.
        let composite =
            Self::composite_score(fact_coverage, decision_coverage, working_depth, trust_score);

        HandoffQualityScore {
            fill_rate,
            budget_utilization,
            dominant_kind,
            eviction_pressure: 1.0 - fill_rate,
            fact_coverage,
            decision_coverage,
            working_depth,
            composite,
        }
    }

    fn trust_score_for_budget_utilization(budget_utilization: f64) -> f64 {
        if budget_utilization >= 0.5 && budget_utilization <= 0.95 {
            1.0
        } else if budget_utilization < 0.5 {
            budget_utilization / 0.5
        } else {
            // utilization in (0.95, 1.0]: mild penalty proportional to how
            // close we are to truncation.
            1.0 - (budget_utilization - 0.95) * 4.0
        }
        .clamp(0.0, 1.0)
    }

    fn composite_score(
        fact_coverage: f64,
        decision_coverage: f64,
        working_depth: f64,
        trust_score: f64,
    ) -> f64 {
        // Weighted composite: coverage of the substantive kinds matters
        // most, working-depth second, trust third.
        0.30 * fact_coverage + 0.30 * decision_coverage + 0.25 * working_depth + 0.15 * trust_score
    }

    pub(crate) fn include_recoverable_signals(
        &mut self,
        fact_count: usize,
        decision_count: usize,
        total_count: usize,
    ) {
        let fact_coverage = (fact_count as f64 / Self::TARGET_FACTS).min(1.0);
        if fact_coverage > self.fact_coverage {
            self.fact_coverage = fact_coverage;
        }
        let decision_coverage = (decision_count as f64 / Self::TARGET_DECISIONS).min(1.0);
        if decision_coverage > self.decision_coverage {
            self.decision_coverage = decision_coverage;
        }
        let working_depth = (total_count as f64 / Self::TARGET_WORKING_DEPTH).min(1.0);
        if working_depth > self.working_depth {
            self.working_depth = working_depth;
        }
        let trust_score = Self::trust_score_for_budget_utilization(self.budget_utilization);
        self.composite = Self::composite_score(
            self.fact_coverage,
            self.decision_coverage,
            self.working_depth,
            trust_score,
        );
    }

    pub(crate) fn include_decision_signals(&mut self, decision_count: usize) {
        self.include_recoverable_signals(0, decision_count, 0);
    }

    /// L2.9: true iff the handoff meets the shipping threshold.
    pub(crate) fn is_acceptable(&self) -> bool {
        self.composite >= Self::ACCEPTANCE_THRESHOLD
    }
}

#[cfg(test)]
mod handoff_quality_tests;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ResumeSnapshot {
    pub(crate) project: Option<String>,
    pub(crate) namespace: Option<String>,
    pub(crate) agent: Option<String>,
    pub(crate) workspace: Option<String>,
    pub(crate) visibility: Option<String>,
    pub(crate) route: String,
    pub(crate) intent: String,
    pub(crate) context: memd_schema::CompactContextResponse,
    pub(crate) working: memd_schema::WorkingMemoryResponse,
    pub(crate) inbox: memd_schema::MemoryInboxResponse,
    pub(crate) workspaces: memd_schema::WorkspaceMemoryResponse,
    pub(crate) sources: memd_schema::SourceMemoryResponse,
    pub(crate) semantic: Option<RagRetrieveResponse>,
    pub(crate) claims: SessionClaimsState,
    pub(crate) recent_repo_changes: Vec<String>,
    pub(crate) change_summary: Vec<String>,
    pub(crate) resume_state_age_minutes: Option<i64>,
    pub(crate) refresh_recommended: bool,
    #[serde(default)]
    pub(crate) atlas_region_hints: Vec<String>,
    #[serde(default)]
    pub(crate) handoff_quality: Option<HandoffQualityScore>,
    #[serde(default)]
    pub(crate) files_touched: Vec<String>,
    #[serde(default)]
    pub(crate) un_read_paths: Vec<String>,
    #[serde(default)]
    pub(crate) preferences: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RetrievalTier {
    Hot,
    Working,
    Rehydration,
    Evidence,
    RawFallback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TruthRecordSummary {
    pub(crate) lane: String,
    pub(crate) truth: String,
    pub(crate) epistemic_state: String,
    pub(crate) freshness: String,
    pub(crate) retrieval_tier: RetrievalTier,
    pub(crate) confidence: f32,
    pub(crate) provenance: String,
    pub(crate) preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TruthSummary {
    pub(crate) retrieval_tier: RetrievalTier,
    pub(crate) truth: String,
    pub(crate) epistemic_state: String,
    pub(crate) freshness: String,
    pub(crate) confidence: f32,
    pub(crate) action_hint: String,
    pub(crate) source_count: usize,
    pub(crate) contested_sources: usize,
    pub(crate) compact_records: usize,
    pub(crate) records: Vec<TruthRecordSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ContinuityCapsule {
    pub(crate) current_task: Option<String>,
    pub(crate) resume_point: Option<String>,
    pub(crate) changed: Option<String>,
    pub(crate) next_action: Option<String>,
    pub(crate) blocker: Option<String>,
}

impl ResumeSnapshot {
    #[cfg(test)]
    pub(crate) fn empty() -> Self {
        use memd_schema::{
            CompactContextResponse, MemoryInboxResponse, RetrievalIntent, RetrievalRoute,
            SourceMemoryResponse, WorkingMemoryPolicyState, WorkingMemoryResponse,
            WorkspaceMemoryResponse,
        };
        Self {
            project: None,
            namespace: None,
            agent: None,
            workspace: None,
            visibility: None,
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            context: CompactContextResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                retrieval_order: Vec::new(),
                records: Vec::new(),
            },
            working: WorkingMemoryResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                retrieval_order: Vec::new(),
                budget_chars: 0,
                used_chars: 0,
                remaining_chars: 0,
                truncated: false,
                policy: WorkingMemoryPolicyState {
                    admission_limit: 0,
                    max_chars_per_item: 0,
                    budget_chars: 0,
                    rehydration_limit: 0,
                },
                records: Vec::new(),
                evicted: Vec::new(),
                rehydration_queue: Vec::new(),
                traces: Vec::new(),
                semantic_consolidation: None,
                procedures: Vec::new(),
                compaction_quality: None,
            },
            inbox: MemoryInboxResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                items: Vec::new(),
            },
            workspaces: WorkspaceMemoryResponse {
                workspaces: Vec::new(),
            },
            sources: SourceMemoryResponse {
                sources: Vec::new(),
            },
            semantic: None,
            claims: SessionClaimsState::default(),
            recent_repo_changes: Vec::new(),
            change_summary: Vec::new(),
            resume_state_age_minutes: None,
            refresh_recommended: false,
            atlas_region_hints: Vec::new(),
            handoff_quality: None,
            files_touched: Vec::new(),
            un_read_paths: Vec::new(),
            preferences: Vec::new(),
        }
    }

    pub(crate) fn continuity_doing(&self) -> Option<String> {
        self.compact_working_records()
            .first()
            .cloned()
            .or_else(|| self.compact_context_records().first().cloned())
    }

    pub(crate) fn continuity_left_off(&self) -> Option<String> {
        self.compact_inbox_items()
            .first()
            .cloned()
            .or_else(|| self.compact_rehydration_summaries().first().cloned())
            .or_else(|| {
                self.workspaces.workspaces.first().map(|lane| {
                    format!(
                        "{} / {} / {}",
                        lane.project.as_deref().unwrap_or("none"),
                        lane.namespace.as_deref().unwrap_or("none"),
                        lane.workspace.as_deref().unwrap_or("none")
                    )
                })
            })
            .or_else(|| self.compact_working_records().first().cloned())
    }

    pub(crate) fn continuity_changed(&self) -> Option<String> {
        self.change_summary
            .first()
            .cloned()
            .or_else(|| self.event_spine().first().cloned())
            .or_else(|| self.recent_repo_changes.first().cloned())
    }

    pub(crate) fn continuity_next(&self) -> Option<String> {
        let rehydration = self.compact_rehydration_summaries();
        let mut next_candidates = Vec::new();
        next_candidates.extend(self.preferences.iter().cloned());
        next_candidates.extend(rehydration.iter().cloned());
        next_candidates.extend(self.compact_context_records());
        next_candidates.extend(self.compact_working_records());
        if let Some(next) = best_next_action_record(next_candidates) {
            return Some(next);
        }
        if self
            .handoff_quality
            .as_ref()
            .is_some_and(|score| !score.is_acceptable())
        {
            return Some(
                "fix partial handoff quality before claiming native recovery ready".to_string(),
            );
        }
        rehydration
            .first()
            .cloned()
            .or_else(|| self.compact_inbox_items().first().cloned())
            .or_else(|| self.compact_working_records().first().cloned())
    }

    pub(crate) fn memory_pressure_drivers(&self) -> Vec<&'static str> {
        let mut drivers = Vec::new();
        let tokens = self.estimated_prompt_tokens();
        if tokens >= 1_800 {
            drivers.push("tokens");
        } else if tokens >= 1_000 {
            drivers.push("tokens");
        }
        if self.redundant_context_items() > 0 {
            drivers.push("duplicates");
        }
        if self.inbox.items.len() >= 3 {
            drivers.push("inbox");
        }
        if self.working.rehydration_queue.len() >= 3 {
            drivers.push("rehydration");
        }
        if self
            .semantic
            .as_ref()
            .is_some_and(|semantic| semantic.items.len() >= 4)
        {
            drivers.push("semantic");
        }
        if self.refresh_recommended {
            drivers.push("refresh");
        }
        drivers.sort_unstable();
        drivers.dedup();
        drivers
    }

    pub(crate) fn memory_action_hint(&self) -> &'static str {
        let pressure = self.context_pressure();
        let drivers = self.memory_pressure_drivers();
        if pressure == "high" {
            if drivers.contains(&"inbox") {
                "drain inbox before the next prompt"
            } else if drivers.contains(&"rehydration") {
                "resolve rehydration backlog before the next prompt"
            } else if drivers.contains(&"duplicates") {
                "collapse repeated context before the next prompt"
            } else {
                "trim context before the next prompt"
            }
        } else if pressure == "medium" {
            if drivers.contains(&"inbox") {
                "handle inbox items before pulling more context"
            } else if drivers.contains(&"rehydration") {
                "resolve rehydration before the prompt grows"
            } else {
                "watch prompt growth"
            }
        } else {
            "none"
        }
    }

    pub(crate) fn source_label(
        agent: Option<&str>,
        system: Option<&str>,
        path: Option<&str>,
    ) -> String {
        let mut parts = Vec::new();
        if let Some(agent) = agent {
            parts.push(agent.to_string());
        }
        if let Some(system) = system {
            parts.push(system.to_string());
        }
        if let Some(path) = path {
            parts.push(path.to_string());
        }
        if parts.is_empty() {
            "none".to_string()
        } else {
            parts.join(" / ")
        }
    }

    pub(crate) fn compact_memory_values<'a, I>(values: I) -> Vec<String>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut seen = std::collections::HashSet::<String>::new();
        let mut compacted = Vec::new();
        for value in values {
            let normalized = Self::normalized_memory_text(value);
            if normalized.is_empty() || !seen.insert(normalized) {
                continue;
            }
            compacted.push(value.trim().to_string());
        }
        compacted
    }

    pub(crate) fn normalized_memory_text(value: &str) -> String {
        normalize_resume_record(value)
    }

    pub(crate) fn compact_context_records(&self) -> Vec<String> {
        Self::compact_memory_values(
            self.context
                .records
                .iter()
                .map(|record| record.record.as_str()),
        )
    }

    pub(crate) fn compact_working_records(&self) -> Vec<String> {
        Self::compact_memory_values(
            self.working
                .records
                .iter()
                .map(|record| record.record.as_str()),
        )
    }

    pub(crate) fn compact_rehydration_summaries(&self) -> Vec<String> {
        Self::compact_memory_values(
            self.working
                .rehydration_queue
                .iter()
                .filter(|item| {
                    // Skip rehydration items whose source_path is a dead file ref.
                    if let Some(source_path) = &item.source_path {
                        let path = std::path::Path::new(source_path.trim());
                        if !source_path.is_empty() && path.is_absolute() && !path.exists() {
                            return false;
                        }
                    }
                    true
                })
                .map(|item| item.summary.as_str()),
        )
    }

    pub(crate) fn compact_inbox_items(&self) -> Vec<String> {
        Self::compact_memory_values(self.inbox.items.iter().filter_map(|item| {
            // Skip expired/superseded items — they are ghost refs.
            if matches!(
                item.item.status,
                MemoryStatus::Expired | MemoryStatus::Superseded
            ) {
                return None;
            }
            let content = item.item.content.as_str();
            // Skip items referencing deleted files (ghost refs from expired entries).
            if let Some(path) = content.strip_prefix("file_edited: ") {
                if !std::path::Path::new(path.trim()).exists() {
                    return None;
                }
            }
            // Skip items whose source_path points to a nonexistent file.
            if let Some(source_path) = &item.item.source_path {
                let path = std::path::Path::new(source_path.trim());
                if !source_path.is_empty() && path.is_absolute() && !path.exists() {
                    return None;
                }
            }
            Some(content)
        }))
    }

    pub(crate) fn compact_semantic_items(&self) -> Vec<String> {
        Self::compact_memory_values(
            self.semantic
                .iter()
                .flat_map(|semantic| semantic.items.iter())
                .map(|item| item.content.as_str()),
        )
    }

    pub(crate) fn active_claims(&self) -> Vec<&SessionClaim> {
        let mut claims = self
            .claims
            .claims
            .iter()
            .filter(|claim| claim.expires_at > Utc::now())
            .collect::<Vec<_>>();
        claims.sort_by(|left, right| right.acquired_at.cmp(&left.acquired_at));
        claims.truncate(6);
        claims
    }

    pub(crate) fn event_spine(&self) -> Vec<String> {
        build_event_spine(
            &self.change_summary,
            &self.recent_repo_changes,
            self.refresh_recommended,
        )
    }

    pub(crate) fn continuity_capsule(&self) -> ContinuityCapsule {
        let current_task = self
            .continuity_doing()
            .map(|value| compact_inline(value.trim(), 180));
        let resume_point = self
            .continuity_left_off()
            .map(|value| compact_inline(value.trim(), 180))
            .or_else(|| {
                Some(format!(
                    "{} / {} / {}",
                    self.project.as_deref().unwrap_or("none"),
                    self.namespace.as_deref().unwrap_or("none"),
                    self.workspace.as_deref().unwrap_or("none")
                ))
            });
        let changed = self
            .continuity_changed()
            .map(|value| compact_inline(value.trim(), 180));
        let raw_next_action = self.continuity_next();
        let next_action = raw_next_action
            .as_deref()
            .map(|value| compact_inline(value.trim(), 1600));
        let blocker = self
            .compact_inbox_items()
            .first()
            .cloned()
            .or_else(|| {
                raw_next_action
                    .as_deref()
                    .and_then(blocker_from_next_action)
                    .map(|value| compact_inline(value.trim(), 512))
            })
            .or_else(|| {
                (self.refresh_recommended && self.refresh_pressure_is_blocking())
                    .then(|| "refresh recommended due to context pressure".to_string())
            });

        ContinuityCapsule {
            current_task,
            resume_point,
            changed,
            next_action,
            blocker,
        }
    }

    pub(crate) fn workflow_capsule(&self) -> Vec<String> {
        let mut lines = Vec::new();

        if let Some(focus) = self.compact_working_records().first() {
            lines.push(format!("rolling_brief: focus {}", focus.trim()));
        }
        if let Some(blocker) = self.compact_inbox_items().first() {
            lines.push(format!("rolling_brief: blocker {}", blocker.trim()));
        }
        if let Some(next) = self.compact_rehydration_summaries().first() {
            lines.push(format!("rolling_brief: next {}", next.trim()));
        }
        if let Some(event) = self.event_spine().first() {
            lines.push(format!("rolling_brief: event {}", event.trim()));
        }

        if let Some(lane) = self.workspaces.workspaces.first() {
            lines.push(format!(
                "entity_sheet: {} / {} / {} | visibility {} | trust {:.2} | claims {}",
                lane.project.as_deref().unwrap_or("none"),
                lane.namespace.as_deref().unwrap_or("none"),
                lane.workspace.as_deref().unwrap_or("none"),
                memory_visibility_label(lane.visibility),
                lane.trust_score,
                self.active_claims().len()
            ));
        }

        let mut claim_groups = std::collections::BTreeMap::<String, Vec<String>>::new();
        for claim in self.active_claims() {
            let owner = claim
                .session
                .as_deref()
                .or(claim.agent.as_deref())
                .unwrap_or("none")
                .to_string();
            claim_groups
                .entry(claim.scope.clone())
                .or_default()
                .push(owner);
        }
        for (scope, owners) in claim_groups.iter().take(4) {
            let unique_owners = owners
                .iter()
                .cloned()
                .collect::<std::collections::BTreeSet<_>>();
            if unique_owners.len() > 1 {
                lines.push(format!(
                    "contradiction_ledger: scope {} contested by {} owners",
                    scope,
                    unique_owners.len()
                ));
            } else if let Some(owner) = unique_owners.iter().next() {
                lines.push(format!("active_claim: {} -> {}", scope, owner));
            }
        }

        let redundant = self.redundant_context_items();
        if redundant > 0 {
            lines.push(format!(
                "contradiction_ledger: {} repeated context item(s)",
                redundant
            ));
        }
        if self.refresh_recommended && self.refresh_pressure_is_blocking() {
            lines.push("blocker: refresh recommended due to context pressure".to_string());
        } else if self.refresh_recommended {
            lines.push("pressure: refresh recommended; action=watch prompt growth".to_string());
        }
        if self.working.rehydration_queue.is_empty() {
            lines.push("blocker: rehydration queue empty".to_string());
        }

        let mut seen = std::collections::HashSet::<String>::new();
        lines.retain(|line| {
            let normalized = line
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase();
            !normalized.is_empty() && seen.insert(normalized)
        });
        lines.truncate(10);
        lines
    }

    pub(crate) fn redundant_context_items(&self) -> usize {
        let mut seen = std::collections::HashSet::<String>::new();
        let mut duplicates = 0usize;

        for value in self.compact_context_records() {
            let normalized = Self::normalized_memory_text(&value);
            if !normalized.is_empty() && !seen.insert(normalized) {
                duplicates += 1;
            }
        }
        for value in self.compact_working_records() {
            let normalized = Self::normalized_memory_text(&value);
            if !normalized.is_empty() && !seen.insert(normalized) {
                duplicates += 1;
            }
        }
        for value in self.compact_rehydration_summaries() {
            let normalized = Self::normalized_memory_text(&value);
            if !normalized.is_empty() && !seen.insert(normalized) {
                duplicates += 1;
            }
        }
        for value in self.compact_inbox_items() {
            let normalized = Self::normalized_memory_text(&value);
            if !normalized.is_empty() && !seen.insert(normalized) {
                duplicates += 1;
            }
        }
        for value in self.compact_semantic_items() {
            let normalized = Self::normalized_memory_text(&value);
            if !normalized.is_empty() && !seen.insert(normalized) {
                duplicates += 1;
            }
        }

        duplicates
    }

    pub(crate) fn estimated_prompt_chars(&self) -> usize {
        let header_chars = self.project.as_deref().map_or(0, str::len)
            + self.namespace.as_deref().map_or(0, str::len)
            + self.agent.as_deref().map_or(0, str::len)
            + self.workspace.as_deref().map_or(0, str::len)
            + self.visibility.as_deref().map_or(0, str::len)
            + self.route.len()
            + self.intent.len();
        let context_chars: usize = self.compact_context_records().iter().map(String::len).sum();
        let working_chars: usize = self.compact_working_records().iter().map(String::len).sum();
        let rehydration_chars: usize = self
            .compact_rehydration_summaries()
            .iter()
            .map(String::len)
            .sum();
        let inbox_chars: usize = self.compact_inbox_items().iter().map(String::len).sum();
        let workspace_chars: usize = self
            .workspaces
            .workspaces
            .iter()
            .map(|lane| {
                lane.project.as_deref().map_or(0, str::len)
                    + lane.namespace.as_deref().map_or(0, str::len)
                    + lane.workspace.as_deref().map_or(0, str::len)
                    + lane.tags.iter().map(|tag| tag.len()).sum::<usize>()
            })
            .sum();
        let semantic_chars: usize = self.compact_semantic_items().iter().map(String::len).sum();
        let event_spine_chars: usize = self.event_spine().iter().map(String::len).sum();
        let workflow_capsule_chars: usize = self.workflow_capsule().iter().map(String::len).sum();
        header_chars
            + context_chars
            + working_chars
            + rehydration_chars
            + inbox_chars
            + workspace_chars
            + semantic_chars
            + event_spine_chars
            + workflow_capsule_chars
    }

    pub(crate) fn core_prompt_chars(&self) -> usize {
        let header_chars = self.project.as_deref().map_or(0, str::len)
            + self.namespace.as_deref().map_or(0, str::len)
            + self.agent.as_deref().map_or(0, str::len)
            + self.workspace.as_deref().map_or(0, str::len)
            + self.visibility.as_deref().map_or(0, str::len)
            + self.route.len()
            + self.intent.len();
        let context_chars: usize = self.compact_context_records().iter().map(String::len).sum();
        let working_chars: usize = self.compact_working_records().iter().map(String::len).sum();
        let rehydration_chars: usize = self
            .compact_rehydration_summaries()
            .iter()
            .map(String::len)
            .sum();
        header_chars + context_chars + working_chars + rehydration_chars
    }

    pub(crate) fn estimated_prompt_tokens(&self) -> usize {
        self.estimated_prompt_chars().div_ceil(4)
    }

    pub(crate) fn core_prompt_tokens(&self) -> usize {
        self.core_prompt_chars().div_ceil(4)
    }

    pub(crate) fn context_pressure(&self) -> &'static str {
        let core_tokens = self.core_prompt_tokens();
        let estimated_tokens = self.estimated_prompt_tokens();
        if self.working.truncated
            || core_tokens >= 1_800
            || self.inbox.items.len() >= 5
            || self.redundant_context_items() >= 3
            || self
                .semantic
                .as_ref()
                .is_some_and(|semantic| semantic.items.len() >= 4)
        {
            "high"
        } else if core_tokens >= 1_000
            || estimated_tokens >= 1_800
            || (self.working.remaining_chars <= 200 && working_has_loss(&self.working))
            || self.inbox.items.len() >= 3
            || self.working.rehydration_queue.len() >= 4
            || self.redundant_context_items() >= 1
        {
            "medium"
        } else {
            "low"
        }
    }

    fn refresh_pressure_is_blocking(&self) -> bool {
        self.working.truncated
            || self.core_prompt_tokens() >= 1_800
            || self.inbox.items.len() >= 5
            || self.redundant_context_items() >= 3
            || self
                .semantic
                .as_ref()
                .is_some_and(|semantic| semantic.items.len() >= 4)
    }

    pub(crate) fn optimization_hints(&self) -> Vec<String> {
        let mut hints = Vec::new();
        if self.refresh_recommended {
            hints.push(
                "prefer a fresh session resumed from the bundle instead of carrying a stale long transcript"
                    .to_string(),
            );
        }
        if !self.recent_repo_changes.is_empty() || !self.change_summary.is_empty() {
            hints.push(
                "prefer live truth and the compact event spine over reopening raw files or replaying old delta logs"
                    .to_string(),
            );
        }
        if self.inbox.items.len() >= 3 {
            hints.push("triage inbox pressure before pulling in more context".to_string());
        }
        let redundant = self.redundant_context_items();
        if redundant > 0 {
            hints.push(format!(
                "collapse {} repeated context item(s) before continuing the session",
                redundant
            ));
        }
        if self
            .semantic
            .as_ref()
            .is_some_and(|semantic| !semantic.items.is_empty())
        {
            hints.push(
                "keep semantic recall off unless deep context is actually required".to_string(),
            );
        }
        if !self.event_spine().is_empty() {
            hints.push(
                "use the event spine first; it is the compact working delta, not a raw reread trail"
                    .to_string(),
            );
        }
        if self.estimated_prompt_tokens() >= 1_200 || self.context.records.len() >= 6 {
            hints.push(
                "promote stable facts into compiled or typed artifacts before rereading raw files"
                    .to_string(),
            );
        }
        if self.working.rehydration_queue.len() >= 3 {
            hints.push(
                "resolve the top rehydration items instead of loading every deferred artifact"
                    .to_string(),
            );
        }
        hints
    }
}

pub(crate) fn truth_freshness_label(snapshot: &ResumeSnapshot) -> String {
    match snapshot.resume_state_age_minutes {
        Some(minutes) if minutes >= 90 || snapshot.refresh_recommended => "stale".to_string(),
        Some(minutes) if minutes >= 30 => "aging".to_string(),
        _ if snapshot.refresh_recommended => "stale".to_string(),
        _ => "fresh".to_string(),
    }
}

fn count_recoverable_decision_records(
    context: &memd_schema::CompactContextResponse,
    working: &memd_schema::WorkingMemoryResponse,
    preferences: &[String],
) -> usize {
    recoverable_signal_counts(context, working, preferences).decisions
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct RecoverableSignalCounts {
    facts: usize,
    decisions: usize,
    total: usize,
}

fn recoverable_signal_counts(
    context: &memd_schema::CompactContextResponse,
    working: &memd_schema::WorkingMemoryResponse,
    preferences: &[String],
) -> RecoverableSignalCounts {
    let mut seen = std::collections::HashSet::new();
    let mut counts = RecoverableSignalCounts::default();
    for record in preferences
        .iter()
        .map(|record| record.as_str())
        .chain(context.records.iter().map(|record| record.record.as_str()))
        .chain(working.records.iter().map(|record| record.record.as_str()))
        .chain(
            working
                .rehydration_queue
                .iter()
                .map(|record| record.summary.as_str()),
        )
    {
        let normalized = normalize_resume_record(record);
        if normalized.is_empty() || !seen.insert(normalized) {
            continue;
        }
        let is_fact = is_fact_record_text(record) || is_recoverable_status_fact(record);
        let is_decision = is_decision_record_text(record) || is_recoverable_status_decision(record);
        let is_procedural = is_procedural_record_text(record);
        if is_fact {
            counts.facts += 1;
        }
        if is_decision {
            counts.decisions += 1;
        }
        if is_fact || is_decision || is_procedural {
            counts.total += 1;
        }
    }
    counts
}

pub(crate) fn count_decision_records(records: &[String]) -> usize {
    count_decision_record_texts(records.iter().map(|record| record.as_str()))
}

fn count_decision_record_texts<'a, I>(records: I) -> usize
where
    I: IntoIterator<Item = &'a str>,
{
    let mut seen = std::collections::HashSet::new();
    records
        .into_iter()
        .filter(|record| is_decision_record_text(record))
        .filter(|record| seen.insert(normalize_resume_record(record)))
        .count()
}

fn is_decision_record_text(record: &str) -> bool {
    let normalized = record.to_ascii_lowercase();
    normalized.contains("| kind=decision |")
        || normalized.contains(" kind=decision ")
        || normalized.starts_with("decision:")
        || normalized.contains("decision: ")
}

fn is_fact_record_text(record: &str) -> bool {
    let normalized = record.to_ascii_lowercase();
    normalized.contains("| kind=fact |")
        || normalized.contains(" kind=fact ")
        || normalized.starts_with("fact:")
        || normalized.contains("fact: ")
}

fn is_status_record_text(record: &str) -> bool {
    let normalized = record.to_ascii_lowercase();
    normalized.contains("| kind=status |") || normalized.contains(" kind=status ")
}

fn is_recoverable_status_decision(record: &str) -> bool {
    if !is_status_record_text(record) {
        return false;
    }
    let normalized = record.to_ascii_lowercase();
    normalized.contains("current next action")
        || normalized.contains("next action")
        || normalized.contains("next=")
        || normalized.contains("fix partial handoff quality")
        || normalized.contains("current-task")
        || normalized.contains("resume_state")
        || normalized.contains("session_state")
}

fn is_recoverable_status_fact(record: &str) -> bool {
    if !is_status_record_text(record) {
        return false;
    }
    let normalized = record.to_ascii_lowercase();
    normalized.contains("proof_blockers=")
        || normalized.contains("live_state_blockers=")
        || normalized.contains("clawcontrol")
        || normalized.contains("auth_required")
        || normalized.contains("git_commit")
        || normalized.contains("tree clean")
        || normalized.contains("status: wake")
        || normalized.contains("bundle-refresh")
        || normalized.contains("resume_state")
        || normalized.contains("session_state")
}

fn is_procedural_record_text(record: &str) -> bool {
    let normalized = record.to_ascii_lowercase();
    normalized.contains("| kind=procedural |")
        || normalized.contains(" kind=procedural ")
        || normalized.starts_with("procedure:")
        || normalized.contains("procedure: ")
}

fn is_next_action_record(record: &str) -> bool {
    let normalized = record.to_ascii_lowercase();
    if is_auto_bundle_refresh_status_record(&normalized) {
        return false;
    }
    is_decision_record_text(record)
        || normalized.contains("next-agent")
        || normalized.contains("next action")
        || normalized.contains("next_action")
}

fn is_auto_bundle_refresh_status_record(normalized: &str) -> bool {
    (normalized.contains("status: wake project=")
        || normalized.contains("source_path=wake")
        || normalized.contains("source_path=checkpoint")
        || normalized.contains("source_path=handoff"))
        && (normalized.contains("auto-short-term") || normalized.contains("bundle-refresh"))
}

fn best_next_action_record(records: Vec<String>) -> Option<String> {
    records
        .into_iter()
        .filter(|record| is_next_action_record(record))
        .max_by_key(|record| record_updated_at(record).unwrap_or(0))
}

fn blocker_from_next_action(next_action: &str) -> Option<String> {
    let markers = [
        "Remaining blockers are ",
        "Remaining blocker pair: ",
        "Remaining blockers: ",
        "blockers are ",
        "blockers: ",
    ];
    markers.iter().find_map(|marker| {
        let (_, tail) = next_action.split_once(marker)?;
        let blocker = tail
            .split(['.', '\n'])
            .next()
            .unwrap_or(tail)
            .trim()
            .trim_end_matches(';')
            .trim();
        (!blocker.is_empty()).then(|| blocker.to_string())
    })
}

fn latest_raw_spine_next_action(output: &Path) -> Option<String> {
    let path = output.join("state").join("raw-spine.jsonl");
    let raw = std::fs::read_to_string(path).ok()?;
    let candidates = raw
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter_map(|value| {
            let content = value
                .get("content_preview")
                .and_then(|value| value.as_str())
                .or_else(|| value.get("content").and_then(|value| value.as_str()))?;
            let tags = value
                .get("tags")
                .and_then(|value| value.as_array())
                .map(|tags| {
                    tags.iter()
                        .filter_map(|tag| tag.as_str())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if tags
                .iter()
                .any(|tag| tag.starts_with("security:") || tag.starts_with("quarantine:"))
            {
                return None;
            }
            let content_lower = content.to_ascii_lowercase();
            let looks_like_current_checkpoint =
                content_lower.contains("current checkpoint")
                    && tags.iter().any(|tag| *tag == "current-task")
                    && tags.iter().any(|tag| *tag == "checkpoint")
                    && !tags.iter().any(|tag| *tag == "auto-short-term");
            let looks_like_next = content_lower.contains("current next action")
                || looks_like_current_checkpoint
                || tags.iter().any(|tag| *tag == "next-agent");
            if !looks_like_next {
                return None;
            }
            let stage = value
                .get("stage")
                .and_then(|value| value.as_str())
                .unwrap_or("raw");
            let status = value
                .get("status")
                .and_then(|value| value.as_str())
                .unwrap_or("active");
            if matches!(status, "archived" | "superseded" | "rejected") {
                return None;
            }
            let id = value
                .get("id")
                .and_then(|value| value.as_str())
                .unwrap_or("raw-next-action");
            let upd = value
                .get("recorded_at")
                .and_then(|value| value.as_str())
                .and_then(parse_record_timestamp)
                .unwrap_or(0);
            let tag_text = if tags.is_empty() {
                "next-agent".to_string()
            } else {
                tags.join(",")
            };
            Some((
                stage == "canonical",
                upd,
                format!(
                    "id={id} | stage={stage} | kind=decision | status={status} | tags={tag_text} | upd={upd} | c={content}"
                ),
            ))
        })
        .collect::<Vec<_>>();
    candidates
        .iter()
        .filter(|(canonical, _, _)| *canonical)
        .max_by_key(|(_, upd, _)| *upd)
        .or_else(|| candidates.iter().max_by_key(|(_, upd, _)| *upd))
        .map(|(_, _, record)| record.clone())
}

fn record_updated_at(record: &str) -> Option<i64> {
    let (_, tail) = record.split_once("| upd=")?;
    let value = tail
        .split(|ch: char| !ch.is_ascii_digit())
        .next()
        .unwrap_or_default();
    value.parse::<i64>().ok()
}

fn parse_record_timestamp(value: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|timestamp| timestamp.timestamp())
}

pub(crate) fn truth_status_label(snapshot: &ResumeSnapshot) -> String {
    if snapshot.refresh_recommended {
        "aging".to_string()
    } else if snapshot.redundant_context_items() > 0 {
        "contested".to_string()
    } else if !snapshot.event_spine().is_empty() {
        "current".to_string()
    } else if !snapshot.compact_working_records().is_empty() {
        "working".to_string()
    } else {
        "fallback".to_string()
    }
}

pub(crate) fn truth_epistemic_state_label(snapshot: &ResumeSnapshot) -> String {
    if !snapshot.event_spine().is_empty() {
        "verified".to_string()
    } else if !snapshot.compact_working_records().is_empty() {
        "claimed".to_string()
    } else if !snapshot.compact_rehydration_summaries().is_empty()
        || !snapshot.compact_context_records().is_empty()
        || !snapshot.compact_semantic_items().is_empty()
        || !snapshot.compact_inbox_items().is_empty()
    {
        "inferred".to_string()
    } else {
        "unknown".to_string()
    }
}

pub(crate) fn choose_retrieval_tier(snapshot: &ResumeSnapshot) -> RetrievalTier {
    if !snapshot.event_spine().is_empty() {
        RetrievalTier::Hot
    } else if !snapshot.compact_working_records().is_empty()
        || !snapshot.compact_context_records().is_empty()
    {
        RetrievalTier::Working
    } else if !snapshot.compact_rehydration_summaries().is_empty() {
        RetrievalTier::Rehydration
    } else if !snapshot.compact_semantic_items().is_empty() {
        RetrievalTier::Evidence
    } else {
        RetrievalTier::RawFallback
    }
}

pub(crate) fn top_source_provenance(snapshot: &ResumeSnapshot) -> String {
    snapshot
        .sources
        .sources
        .iter()
        .max_by(|left, right| left.trust_score.total_cmp(&right.trust_score))
        .map(|source| {
            ResumeSnapshot::source_label(
                source.source_agent.as_deref(),
                source.source_system.as_deref(),
                None,
            )
        })
        .unwrap_or_else(|| "bundle / compact".to_string())
}

pub(crate) fn top_source_confidence(snapshot: &ResumeSnapshot) -> f32 {
    snapshot
        .sources
        .sources
        .iter()
        .map(|source| source.avg_confidence)
        .max_by(|left, right| left.total_cmp(right))
        .unwrap_or(0.92)
}

pub(crate) fn build_truth_record_summary(
    lane: &str,
    truth: &str,
    epistemic_state: &str,
    freshness: &str,
    retrieval_tier: RetrievalTier,
    confidence: f32,
    provenance: &str,
    preview: &str,
) -> TruthRecordSummary {
    TruthRecordSummary {
        lane: lane.to_string(),
        truth: truth.to_string(),
        epistemic_state: epistemic_state.to_string(),
        freshness: freshness.to_string(),
        retrieval_tier,
        confidence,
        provenance: provenance.to_string(),
        preview: compact_inline(preview, 120),
    }
}

pub(crate) fn build_truth_summary(snapshot: &ResumeSnapshot) -> TruthSummary {
    let freshness = truth_freshness_label(snapshot);
    let truth = truth_status_label(snapshot);
    let epistemic_state = truth_epistemic_state_label(snapshot);
    let retrieval_tier = choose_retrieval_tier(snapshot);
    let provenance = top_source_provenance(snapshot);
    let confidence = top_source_confidence(snapshot);
    let mut records = Vec::new();

    if let Some(event) = snapshot.event_spine().first() {
        records.push(build_truth_record_summary(
            "live_truth",
            "current",
            "verified",
            &freshness,
            RetrievalTier::Hot,
            confidence.max(0.95),
            "event_spine / compact",
            event,
        ));
    }
    if let Some(record) = snapshot.compact_working_records().first() {
        records.push(build_truth_record_summary(
            "working_set",
            if snapshot.redundant_context_items() > 0 {
                "contested"
            } else {
                "working"
            },
            if snapshot.redundant_context_items() > 0 {
                "inferred"
            } else {
                "claimed"
            },
            &freshness,
            RetrievalTier::Working,
            confidence,
            &provenance,
            record,
        ));
    }
    if let Some(item) = snapshot.compact_rehydration_summaries().first() {
        records.push(build_truth_record_summary(
            "rehydration",
            "pending",
            "inferred",
            &freshness,
            RetrievalTier::Rehydration,
            (confidence - 0.08).max(0.5),
            "rehydration / deferred",
            item,
        ));
    }
    if let Some(item) = snapshot.compact_semantic_items().first() {
        records.push(build_truth_record_summary(
            "evidence",
            "evidence",
            "verified",
            &freshness,
            RetrievalTier::Evidence,
            confidence,
            &provenance,
            item,
        ));
    }
    if let Some(item) = snapshot.compact_inbox_items().first() {
        records.push(build_truth_record_summary(
            "inbox",
            "candidate",
            "inferred",
            &freshness,
            RetrievalTier::Working,
            (confidence - 0.12).max(0.45),
            "inbox / unmerged",
            item,
        ));
    }

    records.truncate(4);

    TruthSummary {
        retrieval_tier,
        truth,
        epistemic_state,
        freshness,
        confidence,
        action_hint: snapshot.memory_action_hint().to_string(),
        source_count: snapshot.sources.sources.len(),
        contested_sources: snapshot
            .sources
            .sources
            .iter()
            .filter(|source| source.contested_count > 0)
            .count(),
        compact_records: snapshot.compact_context_records().len()
            + snapshot.compact_working_records().len()
            + snapshot.compact_rehydration_summaries().len()
            + snapshot.compact_inbox_items().len(),
        records,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HandoffSnapshot {
    pub(crate) generated_at: DateTime<Utc>,
    pub(crate) resume: ResumeSnapshot,
    pub(crate) sources: memd_schema::SourceMemoryResponse,
    #[serde(default = "default_voice_mode")]
    pub(crate) voice_mode: String,
    pub(crate) target_session: Option<String>,
    pub(crate) target_bundle: Option<String>,
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
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                return Err(err).with_context(|| format!("remove {}", path.display()));
            }
        }
    }
    Ok(())
}
