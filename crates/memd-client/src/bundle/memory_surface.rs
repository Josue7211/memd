use super::*;

#[derive(Debug, Clone, Copy)]
pub(crate) enum MemoryObjectLane {
    Context,
    Working,
    Inbox,
    Recovery,
    Semantic,
    Workspace,
}

impl MemoryObjectLane {
    pub(crate) fn slug(self) -> &'static str {
        match self {
            MemoryObjectLane::Context => "context",
            MemoryObjectLane::Working => "working",
            MemoryObjectLane::Inbox => "inbox",
            MemoryObjectLane::Recovery => "recovery",
            MemoryObjectLane::Semantic => "semantic",
            MemoryObjectLane::Workspace => "workspace",
        }
    }

    pub(crate) fn title(self) -> &'static str {
        match self {
            MemoryObjectLane::Context => "Context",
            MemoryObjectLane::Working => "Working",
            MemoryObjectLane::Inbox => "Inbox",
            MemoryObjectLane::Recovery => "Recovery",
            MemoryObjectLane::Semantic => "Semantic",
            MemoryObjectLane::Workspace => "Workspace",
        }
    }
}

pub(crate) fn bundle_compiled_memory_dir(output: &Path) -> PathBuf {
    output.join("compiled").join("memory")
}

pub(crate) fn bundle_compiled_memory_path(output: &Path, lane: MemoryObjectLane) -> PathBuf {
    bundle_compiled_memory_dir(output).join(format!("{}.md", lane.slug()))
}

pub(crate) fn bundle_compiled_memory_item_path(
    output: &Path,
    lane: MemoryObjectLane,
    index: usize,
    key: &str,
) -> PathBuf {
    bundle_compiled_memory_dir(output)
        .join("items")
        .join(lane.slug())
        .join(format!(
            "{}-{:02}-{}.md",
            lane.slug(),
            index + 1,
            short_hash_text(key)
        ))
}

pub(crate) fn short_hash_text(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
        .chars()
        .take(8)
        .collect()
}

pub(crate) fn memory_object_lane_item_key(
    snapshot: &ResumeSnapshot,
    lane: MemoryObjectLane,
    index: usize,
) -> Option<String> {
    match lane {
        MemoryObjectLane::Context => snapshot
            .context
            .records
            .get(index)
            .map(|record| format!("{}|{}", record.id, record.record)),
        MemoryObjectLane::Working => snapshot
            .working
            .records
            .get(index)
            .map(|record| format!("{}|{}", record.id, record.record)),
        MemoryObjectLane::Inbox => snapshot.inbox.items.get(index).map(|item| {
            format!(
                "{}|{}|{}|{}|{}|{:?}|{:?}",
                item.item.id,
                item.item.content,
                format!("{:?}", item.item.kind),
                format!("{:?}", item.item.scope),
                format!("{:?}", item.item.status),
                item.item.stage,
                item.item.confidence
            )
        }),
        MemoryObjectLane::Recovery => snapshot.working.rehydration_queue.get(index).map(|item| {
            format!(
                "{}|{}|{}|{}",
                item.id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                item.kind,
                item.label,
                item.summary
            )
        }),
        MemoryObjectLane::Semantic => snapshot
            .semantic
            .as_ref()
            .and_then(|semantic| semantic.items.get(index))
            .map(|item| format!("{:.4}|{}", item.score, item.content)),
        MemoryObjectLane::Workspace => snapshot.workspaces.workspaces.get(index).map(|lane| {
            format!(
                "{}|{}|{}|{:?}|{}|{}|{}|{}",
                lane.project.as_deref().unwrap_or("none"),
                lane.namespace.as_deref().unwrap_or("none"),
                lane.workspace.as_deref().unwrap_or("none"),
                lane.visibility,
                lane.item_count,
                lane.active_count,
                lane.contested_count,
                lane.trust_score
            )
        }),
    }
}

pub(crate) fn memory_object_item_slug(
    snapshot: &ResumeSnapshot,
    lane: MemoryObjectLane,
    index: usize,
) -> Option<String> {
    let key = memory_object_lane_item_key(snapshot, lane, index)?;
    Some(format!(
        "{}-{:02}-{}",
        lane.slug(),
        index + 1,
        short_hash_text(&key)
    ))
}

pub(crate) fn memory_object_lane_item_count(
    snapshot: &ResumeSnapshot,
    lane: MemoryObjectLane,
) -> usize {
    match lane {
        MemoryObjectLane::Context => snapshot.context.records.len(),
        MemoryObjectLane::Working => snapshot.working.records.len(),
        MemoryObjectLane::Inbox => snapshot.inbox.items.len(),
        MemoryObjectLane::Recovery => snapshot.working.rehydration_queue.len(),
        MemoryObjectLane::Semantic => snapshot
            .semantic
            .as_ref()
            .map(|semantic| semantic.items.len())
            .unwrap_or(0),
        MemoryObjectLane::Workspace => snapshot.workspaces.workspaces.len(),
    }
}

pub(crate) fn render_bundle_memory_object_markdown(
    output: &Path,
    snapshot: &ResumeSnapshot,
    _handoff: Option<&HandoffSnapshot>,
    hive: Option<&BundleHiveMemorySurface>,
    lane: MemoryObjectLane,
) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# memd memory object: {}{}\n\n",
        lane.title(),
        render_memory_page_header_suffix(output)
    ));
    markdown.push_str(&render_bundle_scope_markdown(output, snapshot));
    markdown.push('\n');

    match lane {
        MemoryObjectLane::Context => {
            markdown.push_str("\n## Context\n\n");
            if snapshot.context.records.is_empty() {
                markdown.push_str("- none\n");
            } else {
                for record in snapshot.context.records.iter().take(6) {
                    markdown.push_str(&format!(
                        "- id={} record=\"{}\"\n",
                        short_uuid(record.id),
                        compact_inline(record.record.trim(), 160)
                    ));
                }
            }
        }
        MemoryObjectLane::Working => {
            markdown.push_str("\n## Working\n\n");
            if snapshot.working.records.is_empty() {
                markdown.push_str("- none\n");
            } else {
                for record in snapshot.working.records.iter().take(6) {
                    markdown.push_str(&format!(
                        "- id={} record=\"{}\"\n",
                        short_uuid(record.id),
                        compact_inline(record.record.trim(), 160)
                    ));
                }
            }
            markdown.push_str(&format!(
                "\n- budget={}/{} | pressure={} | refresh={}\n",
                snapshot.working.used_chars,
                snapshot.working.budget_chars,
                snapshot.context_pressure(),
                snapshot.refresh_recommended
            ));
        }
        MemoryObjectLane::Inbox => {
            markdown.push_str("\n## Inbox\n\n");
            if snapshot.inbox.items.is_empty() {
                markdown.push_str("- none\n");
            } else {
                for item in snapshot.inbox.items.iter().take(6) {
                    markdown.push_str(&format!(
                        "- id={} kind={} type={} status={} stage={} cf={:.2} scope={} source={} note=\"{}\"\n",
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
                        compact_inline(item.item.content.trim(), 160)
                    ));
                    if !item.reasons.is_empty() {
                        markdown.push_str(&format!(
                            "  - reasons={}\n",
                            item.reasons
                                .iter()
                                .take(3)
                                .cloned()
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                }
            }
        }
        MemoryObjectLane::Recovery => {
            markdown.push_str("\n## Recovery\n\n");
            if snapshot.working.rehydration_queue.is_empty() {
                markdown.push_str("- none\n");
            } else {
                for artifact in snapshot.working.rehydration_queue.iter().take(6) {
                    markdown.push_str(&format!(
                        "- id={} kind={} label=\"{}\" source={} reason=\"{}\"\n",
                        artifact
                            .id
                            .map(short_uuid)
                            .unwrap_or_else(|| "none".to_string()),
                        artifact.kind,
                        compact_inline(&artifact.label, 96),
                        ResumeSnapshot::source_label(
                            artifact.source_agent.as_deref(),
                            artifact.source_system.as_deref(),
                            artifact.source_path.as_deref()
                        ),
                        artifact
                            .reason
                            .as_deref()
                            .map(|value| compact_inline(value, 160))
                            .unwrap_or_else(|| "none".to_string())
                    ));
                }
            }
        }
        MemoryObjectLane::Semantic => {
            markdown.push_str("\n## Semantic\n\n");
            if let Some(semantic) = snapshot
                .semantic
                .as_ref()
                .filter(|semantic| !semantic.items.is_empty())
            {
                for item in semantic.items.iter().take(6) {
                    markdown.push_str(&format!(
                        "- score={:.2} content=\"{}\"\n",
                        item.score,
                        compact_inline(&item.content, 160)
                    ));
                }
            } else {
                markdown.push_str("- none\n");
            }
        }
        MemoryObjectLane::Workspace => {
            markdown.push_str("\n## Workspace\n\n");
            if snapshot.workspaces.workspaces.is_empty() {
                markdown.push_str("- none\n");
            } else {
                for lane in snapshot.workspaces.workspaces.iter().take(6) {
                    markdown.push_str(&format!(
                        "- project={} namespace={} workspace={} visibility={} items={} active={} contested={} trust={:.2} cf={:.2}\n",
                        lane.project.as_deref().unwrap_or("none"),
                        lane.namespace.as_deref().unwrap_or("none"),
                        lane.workspace.as_deref().unwrap_or("none"),
                        memory_visibility_label(lane.visibility),
                        lane.item_count,
                        lane.active_count,
                        lane.contested_count,
                        lane.trust_score,
                        lane.avg_confidence
                    ));
                }
            }
        }
    }

    if matches!(lane, MemoryObjectLane::Workspace) {
        if let Some(hive) = hive {
            markdown.push_str("\n## Hive\n\n");
            markdown.push_str(&format!(
                "- queen={} active={} stale={} review={} overlap={}\n",
                hive.board.queen_session.as_deref().unwrap_or("none"),
                hive.board.active_bees.len(),
                hive.board.stale_bees.len(),
                hive.board.review_queue.len(),
                hive.board.overlap_risks.len(),
            ));
            for bee in hive.board.active_bees.iter().take(6) {
                markdown.push_str(&format!(
                    "- bee {} ({}) lane={} task={}\n",
                    bee.worker_name
                        .as_deref()
                        .or(bee.agent.as_deref())
                        .unwrap_or("unnamed"),
                    bee.session,
                    bee.lane_id
                        .as_deref()
                        .or(bee.branch.as_deref())
                        .unwrap_or("none"),
                    bee.task_id.as_deref().unwrap_or("none"),
                ));
            }
        }
    }

    markdown.push_str("\n## Items\n\n");
    let item_count = memory_object_lane_item_count(snapshot, lane);
    if item_count == 0 {
        markdown.push_str("- none\n");
    } else {
        for index in 0..item_count {
            if let Some(slug) = memory_object_item_slug(snapshot, lane, index) {
                markdown.push_str(&format!(
                    "- [{}](items/{}/{})\n",
                    lane.title(),
                    lane.slug(),
                    slug
                ));
            }
        }
    }

    markdown
}

pub(crate) fn render_bundle_memory_object_item_markdown(
    output: &Path,
    snapshot: &ResumeSnapshot,
    handoff: Option<&HandoffSnapshot>,
    hive: Option<&BundleHiveMemorySurface>,
    lane: MemoryObjectLane,
    index: usize,
) -> Option<String> {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# memd memory item: {}{}\n\n",
        lane.title(),
        render_memory_page_header_suffix(output)
    ));
    markdown.push_str(&render_bundle_scope_markdown(output, snapshot));
    markdown.push('\n');
    markdown.push_str(&format!("- lane={} | index={}\n", lane.slug(), index + 1));

    match lane {
        MemoryObjectLane::Context => {
            let record = snapshot.context.records.get(index)?;
            markdown.push_str(&format!(
                "- id={} record=\"{}\"\n",
                short_uuid(record.id),
                compact_inline(record.record.trim(), 240)
            ));
        }
        MemoryObjectLane::Working => {
            let record = snapshot.working.records.get(index)?;
            markdown.push_str(&format!(
                "- id={} record=\"{}\"\n",
                short_uuid(record.id),
                compact_inline(record.record.trim(), 240)
            ));
            markdown.push_str(&format!(
                "- budget={}/{} | pressure={} | refresh={}\n",
                snapshot.working.used_chars,
                snapshot.working.budget_chars,
                snapshot.context_pressure(),
                snapshot.refresh_recommended
            ));
        }
        MemoryObjectLane::Inbox => {
            let item = snapshot.inbox.items.get(index)?;
            markdown.push_str(&format!(
                "- id={} kind={} type={} status={} stage={} cf={:.2} scope={} source={} note=\"{}\"\n",
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
                compact_inline(item.item.content.trim(), 240)
            ));
            if !item.reasons.is_empty() {
                markdown.push_str(&format!(
                    "- reasons={}\n",
                    item.reasons
                        .iter()
                        .take(6)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
        MemoryObjectLane::Recovery => {
            let item = snapshot.working.rehydration_queue.get(index)?;
            markdown.push_str(&format!(
                "- id={} kind={} label=\"{}\" source={} reason=\"{}\"\n",
                item.id
                    .map(short_uuid)
                    .unwrap_or_else(|| "none".to_string()),
                item.kind,
                compact_inline(&item.label, 120),
                ResumeSnapshot::source_label(
                    item.source_agent.as_deref(),
                    item.source_system.as_deref(),
                    item.source_path.as_deref()
                ),
                item.reason
                    .as_deref()
                    .map(|value| compact_inline(value, 240))
                    .unwrap_or_else(|| "none".to_string())
            ));
        }
        MemoryObjectLane::Semantic => {
            let semantic = snapshot.semantic.as_ref()?.items.get(index)?;
            markdown.push_str(&format!(
                "- score={:.2} content=\"{}\"\n",
                semantic.score,
                compact_inline(&semantic.content, 240)
            ));
        }
        MemoryObjectLane::Workspace => {
            let lane = snapshot.workspaces.workspaces.get(index)?;
            markdown.push_str(&format!(
                "- project={} namespace={} workspace={} visibility={} items={} active={} contested={} trust={:.2} cf={:.2}\n",
                lane.project.as_deref().unwrap_or("none"),
                lane.namespace.as_deref().unwrap_or("none"),
                lane.workspace.as_deref().unwrap_or("none"),
                memory_visibility_label(lane.visibility),
                lane.item_count,
                lane.active_count,
                lane.contested_count,
                lane.trust_score,
                lane.avg_confidence
            ));
        }
    }

    if let Some(handoff) = handoff {
        markdown.push_str("\n## Handoff\n\n");
        markdown.push_str(&format!(
            "- target_session={} target_bundle={}\n",
            handoff.target_session.as_deref().unwrap_or("none"),
            handoff.target_bundle.as_deref().unwrap_or("none")
        ));
        markdown.push_str(&format!("- sources={}\n", handoff.sources.sources.len()));
    }

    if matches!(lane, MemoryObjectLane::Workspace) {
        if let Some(hive) = hive {
            markdown.push_str("\n## Hive\n\n");
            markdown.push_str(&format!(
                "- queen={} active={} overlap={} stale={}\n",
                hive.board.queen_session.as_deref().unwrap_or("none"),
                hive.board.active_bees.len(),
                hive.board.overlap_risks.len(),
                hive.board.stale_bees.len(),
            ));
            if let Some(follow) = hive.follow.as_ref() {
                markdown.push_str(&format!(
                    "- focus={} work=\"{}\" next=\"{}\"\n",
                    follow
                        .target
                        .worker_name
                        .as_deref()
                        .or(follow.target.agent.as_deref())
                        .unwrap_or(follow.target.session.as_str()),
                    compact_inline(&follow.work_summary, 160),
                    follow.next_action.as_deref().unwrap_or("none"),
                ));
            }
        }
    }

    Some(markdown)
}

pub(crate) fn render_current_task_bundle_snapshot(snapshot: &ResumeSnapshot) -> String {
    let mut markdown = String::new();
    let continuity = snapshot.continuity_capsule();

    if let Some(current_task) = continuity.current_task.as_deref() {
        markdown.push_str(&format!("- doing={}\n", compact_inline(current_task, 180)));
    }
    if let Some(resume_point) = continuity.resume_point.as_deref() {
        markdown.push_str(&format!(
            "- left_off={}\n",
            compact_inline(resume_point, 180)
        ));
    }
    if let Some(changed) = continuity.changed.as_deref() {
        markdown.push_str(&format!("- changed={}\n", compact_inline(changed, 180)));
    }
    if let Some(next_action) = continuity.next_action.as_deref() {
        markdown.push_str(&format!("- next={}\n", compact_inline(next_action, 180)));
    }
    if let Some(blocker) = continuity.blocker.as_deref() {
        markdown.push_str(&format!("- blocker={}\n", compact_inline(blocker, 180)));
    }

    let capsule = snapshot.workflow_capsule();
    if !capsule.is_empty() {
        let summary = capsule
            .iter()
            .take(4)
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join(" | ");
        markdown.push_str(&format!("- t={summary}\n"));
    }

    markdown
}

pub(crate) async fn read_memory_surface(
    output: &Path,
    base_url: &str,
) -> anyhow::Result<MemorySurfaceResponse> {
    let snapshot = crate::runtime::read_bundle_resume(
        &ResumeArgs {
            output: output.to_path_buf(),
            project: None,
            namespace: None,
            agent: None,
            workspace: None,
            visibility: None,
            route: None,
            intent: Some("current_task".to_string()),
            limit: Some(4),
            rehydration_limit: Some(2),
            semantic: true,
            prompt: false,
            summary: false,
        },
        base_url,
    )
    .await?;
    let truth_summary = build_truth_summary(&snapshot);
    let records = truth_summary.records.clone();
    let contradiction_pressure = snapshot.redundant_context_items()
        + truth_summary.contested_sources
        + snapshot.active_claims().len();
    let superseded_pressure = snapshot
        .change_summary
        .iter()
        .chain(snapshot.recent_repo_changes.iter())
        .filter(|entry| {
            let lowered = entry.to_ascii_lowercase();
            lowered.contains("supersed") || lowered.contains("stale")
        })
        .count();
    let contradiction_reasons = snapshot
        .workspaces
        .workspaces
        .iter()
        .filter(|lane| lane.contested_count > 0)
        .take(4)
        .map(|lane| {
            format!(
                "{} / {} / {} contested={} trust={:.2} cf={:.2}",
                lane.project.as_deref().unwrap_or("none"),
                lane.namespace.as_deref().unwrap_or("none"),
                lane.workspace.as_deref().unwrap_or("none"),
                lane.contested_count,
                lane.trust_score,
                lane.avg_confidence
            )
        })
        .collect::<Vec<_>>();
    let mut superseded_reasons = Vec::new();
    if snapshot.redundant_context_items() > 0 {
        superseded_reasons.push(format!(
            "redundant_context_items={}",
            snapshot.redundant_context_items()
        ));
    }
    if snapshot.refresh_recommended {
        superseded_reasons.push("refresh_recommended".to_string());
    }
    for driver in snapshot.memory_pressure_drivers().into_iter().take(3) {
        superseded_reasons.push(format!("pressure_driver={driver}"));
    }
    Ok(MemorySurfaceResponse {
        bundle_root: output.display().to_string(),
        truth_summary,
        context_records: snapshot.context.records.len(),
        working_records: snapshot.working.records.len(),
        inbox_items: snapshot.inbox.items.len(),
        source_lanes: snapshot.sources.sources.len(),
        rehydration_queue: snapshot.working.rehydration_queue.len(),
        semantic_hits: snapshot
            .semantic
            .as_ref()
            .map(|semantic| semantic.items.len())
            .unwrap_or(0),
        change_summary: snapshot.change_summary.len(),
        estimated_prompt_tokens: snapshot.estimated_prompt_tokens(),
        refresh_recommended: snapshot.refresh_recommended,
        contradiction_pressure,
        superseded_pressure,
        contradiction_reasons,
        superseded_reasons,
        records,
    })
}

pub(crate) fn render_memory_surface_summary(response: &MemorySurfaceResponse) -> String {
    let truth = &response.truth_summary;
    let head = response.records.first();
    format!(
        "memory bundle={} truth={} epistemic={} freshness={} retrieval={} conf={:.2} tiers=context:{} working:{} inbox:{} sources:{} rehydrate:{} semantic:{} changes:{} tok={} refresh={} contradictions={} superseded={} action=\"{}\" head={} preview=\"{}\"",
        response.bundle_root,
        truth.truth,
        truth.epistemic_state,
        truth.freshness,
        serde_json::to_string(&truth.retrieval_tier)
            .unwrap_or_else(|_| "\"raw_fallback\"".to_string())
            .trim_matches('"'),
        truth.confidence,
        response.context_records,
        response.working_records,
        response.inbox_items,
        response.source_lanes,
        response.rehydration_queue,
        response.semantic_hits,
        response.change_summary,
        response.estimated_prompt_tokens,
        if response.refresh_recommended {
            "yes"
        } else {
            "no"
        },
        response.contradiction_pressure,
        response.superseded_pressure,
        compact_inline(&truth.action_hint, 64),
        head.map(|record| record.lane.as_str()).unwrap_or("none"),
        compact_inline(
            head.map(|record| record.preview.as_str()).unwrap_or("none"),
            72
        )
    )
}
