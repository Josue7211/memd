use memd_schema::{
    AgentProfileResponse, AssociativeRecallResponse, EntitySearchResponse, ExplainMemoryResponse,
    RepairMemoryResponse, RetrievalIntent, RetrievalRoute, SourceMemoryResponse,
    WorkingMemoryResponse, WorkspaceMemoryResponse,
};

use crate::obsidian::ObsidianVaultScan;

pub(crate) fn render_obsidian_scan_summary(scan: &ObsidianVaultScan, follow: bool) -> String {
    let mut summary = format!(
        "obsidian_scan vault={} project={} namespace={} workspace={} visibility={} notes={} sensitive={} skipped={} unchanged={} backlinks={} attachments={} attachment_sensitive={} attachment_unchanged={}",
        scan.vault.display(),
        scan.project.as_deref().unwrap_or("none"),
        scan.namespace.as_deref().unwrap_or("none"),
        scan.workspace.as_deref().unwrap_or("none"),
        format_visibility(scan.visibility),
        scan.note_count,
        scan.sensitive_count,
        scan.skipped_count,
        scan.unchanged_count,
        scan.backlink_count,
        scan.attachment_count,
        scan.attachment_sensitive_count,
        scan.attachment_unchanged_count
    );

    if follow {
        let trail = scan
            .notes
            .iter()
            .take(3)
            .map(|note| note.title.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        if !trail.is_empty() {
            summary.push_str(&format!(" trail={trail}"));
        }
    }

    summary
}

pub(crate) fn render_obsidian_import_summary(
    output: &crate::ObsidianImportOutput,
    follow: bool,
) -> String {
    let attachment_submitted = output
        .attachments
        .as_ref()
        .map(|attachments| attachments.submitted)
        .unwrap_or(0);
    let mut summary = format!(
        "obsidian_import vault={} project={} namespace={} workspace={} visibility={} notes={} sensitive={} unchanged={} backlinks={} attachments={} attachment_sensitive={} attachment_unchanged={} submitted={} attachment_submitted={} duplicates={} attachment_duplicates={} note_failures={} attachment_failures={} links={} attachment_links={} mirrored={} mirrored_attachments={} dry_run={}",
        output.preview.scan.vault.display(),
        output.preview.scan.project.as_deref().unwrap_or("none"),
        output.preview.scan.namespace.as_deref().unwrap_or("none"),
        output.preview.scan.workspace.as_deref().unwrap_or("none"),
        format_visibility(output.preview.scan.visibility),
        output.preview.scan.note_count,
        output.preview.scan.sensitive_count,
        output.preview.scan.unchanged_count,
        output.preview.scan.backlink_count,
        output.preview.scan.attachment_count,
        output.preview.scan.attachment_sensitive_count,
        output.attachment_unchanged_count,
        output.submitted,
        attachment_submitted,
        output.duplicates,
        output.attachment_duplicates,
        output.note_failures,
        output.attachment_failures,
        output.links_created,
        output.attachment_links_created,
        output.mirrored_notes,
        output.mirrored_attachments,
        output.dry_run
    );
    if follow {
        let trail = output
            .preview
            .scan
            .notes
            .iter()
            .take(3)
            .map(|note| note.title.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        if !trail.is_empty() {
            summary.push_str(&format!(" trail={trail}"));
        }
    }
    if let Some(attachments) = output.attachments.as_ref() {
        summary.push_str(&format!(
            " attachments_submitted={} attachments_dry_run={}",
            attachments.submitted, attachments.dry_run
        ));
    }
    summary
}

pub(crate) fn render_entity_summary(
    response: &memd_schema::EntityMemoryResponse,
    follow: bool,
) -> String {
    let Some(entity) = response.entity.as_ref() else {
        return format!(
            "entity=none route={} intent={}",
            route_label(response.route),
            intent_label(response.intent)
        );
    };

    let state = entity
        .current_state
        .as_deref()
        .map(|value| compact_inline(value, 72))
        .unwrap_or_else(|| "no-state".to_string());
    let last_seen = entity
        .last_seen_at
        .map(|value| value.to_rfc3339())
        .unwrap_or_else(|| "unknown".to_string());

    let mut output = format!(
        "entity={} type={} salience={:.2} rehearsal={} state_v={} last_seen={} state=\"{}\" events={}",
        short_uuid(entity.id),
        entity.entity_type,
        entity.salience_score,
        entity.rehearsal_count,
        entity.state_version,
        last_seen,
        state,
        response.events.len()
    );

    if follow && let Some(event) = response.events.first() {
        output.push_str(&format!(
            " latest={}::{}",
            event.event_type,
            compact_inline(&event.summary, 48)
        ));
    }

    output
}

pub(crate) fn render_entity_search_summary(response: &EntitySearchResponse, follow: bool) -> String {
    let mut output = format!(
        "entity-search query=\"{}\" candidates={} ambiguous={}",
        compact_inline(&response.query, 48),
        response.candidates.len(),
        response.ambiguous
    );

    if let Some(best) = response.best_match.as_ref() {
        output.push_str(&format!(
            " best={} type={} score={:.2} reasons={}",
            short_uuid(best.entity.id),
            best.entity.entity_type,
            best.score,
            compact_inline(&best.reasons.join(","), 64)
        ));
    }

    if follow {
        let trail = response
            .candidates
            .iter()
            .take(3)
            .map(|candidate| {
                format!(
                    "{}:{}:{:.2}",
                    short_uuid(candidate.entity.id),
                    candidate.entity.entity_type,
                    candidate.score
                )
            })
            .collect::<Vec<_>>();
        if !trail.is_empty() {
            output.push_str(&format!(" trail={}", trail.join(" | ")));
        }
    }

    output
}

pub(crate) fn render_recall_summary(response: &AssociativeRecallResponse, follow: bool) -> String {
    let root = response
        .root_entity
        .as_ref()
        .map(|entity| format!("root={} type={}", short_uuid(entity.id), entity.entity_type))
        .unwrap_or_else(|| "root=none".to_string());

    let mut output = format!(
        "recall {} hits={} links={} truncated={}",
        root,
        response.hits.len(),
        response.links.len(),
        response.truncated
    );

    if follow {
        let hit_trail = response
            .hits
            .iter()
            .take(3)
            .map(|hit| {
                format!(
                    "d{}:{}:{:.2}:{}",
                    hit.depth,
                    short_uuid(hit.entity.id),
                    hit.score,
                    compact_inline(
                        hit.entity
                            .current_state
                            .as_deref()
                            .unwrap_or(&hit.entity.entity_type),
                        28
                    )
                )
            })
            .collect::<Vec<_>>();
        if !hit_trail.is_empty() {
            output.push_str(&format!(" trail={}", hit_trail.join(" | ")));
        }

        let link_trail = response
            .links
            .iter()
            .take(3)
            .map(|link| {
                format!(
                    "{}:{}->{}",
                    format!("{:?}", link.relation_kind).to_ascii_lowercase(),
                    short_uuid(link.from_entity_id),
                    short_uuid(link.to_entity_id)
                )
            })
            .collect::<Vec<_>>();
        if !link_trail.is_empty() {
            output.push_str(&format!(" links={}", link_trail.join(" | ")));
        }

        if let Some(best) = response.hits.first() {
            output.push_str(&format!(
                " best_score={:.2} best_reasons={}",
                best.score,
                compact_inline(&best.reasons.join(","), 72)
            ));
        }
    }

    output
}

pub(crate) fn render_timeline_summary(
    response: &memd_schema::TimelineMemoryResponse,
    follow: bool,
) -> String {
    let entity = response
        .entity
        .as_ref()
        .map(|entity| {
            format!(
                "entity={} type={}",
                short_uuid(entity.id),
                entity.entity_type
            )
        })
        .unwrap_or_else(|| "entity=none".to_string());
    let latest = response
        .events
        .first()
        .map(|event| {
            format!(
                "{}:{}",
                event.event_type,
                compact_inline(&event.summary, 56)
            )
        })
        .unwrap_or_else(|| "no-events".to_string());

    let mut output = format!(
        "timeline {} route={} intent={} events={} latest={}",
        entity,
        route_label(response.route),
        intent_label(response.intent),
        response.events.len(),
        latest
    );

    if follow {
        let trail = response
            .events
            .iter()
            .take(3)
            .map(|event| {
                format!(
                    "{}:{}",
                    event.event_type,
                    compact_inline(&event.summary, 40)
                )
            })
            .collect::<Vec<_>>();
        if !trail.is_empty() {
            output.push_str(&format!(" trail={}", trail.join(" | ")));
        }
    }

    output
}

pub(crate) fn render_working_summary(response: &WorkingMemoryResponse, follow: bool) -> String {
    let mut output = format!(
        "working route={} intent={} budget={} used={} remaining={} truncated={} records={} evicted={} rehydrate={} traces={} semantic={}",
        route_label(response.route),
        intent_label(response.intent),
        response.budget_chars,
        response.used_chars,
        response.remaining_chars,
        response.truncated,
        response.records.len(),
        response.evicted.len(),
        response.rehydration_queue.len(),
        response.traces.len(),
        response
            .semantic_consolidation
            .as_ref()
            .map(|value| value.consolidated.to_string())
            .unwrap_or_else(|| "off".to_string())
    );

    if follow {
        let trail = response
            .records
            .iter()
            .take(3)
            .map(|record| compact_inline(&record.record, 48))
            .collect::<Vec<_>>();
        if !trail.is_empty() {
            output.push_str(&format!(" trail={}", trail.join(" | ")));
        }

        let trace_trail = response
            .traces
            .iter()
            .take(3)
            .map(|trace| {
                format!(
                    "{}:{}",
                    trace.event_type,
                    compact_inline(&trace.summary, 40)
                )
            })
            .collect::<Vec<_>>();
        if !trace_trail.is_empty() {
            output.push_str(&format!(" trace_trail={}", trace_trail.join(" | ")));
        }

        let rehydrate_trail = response
            .rehydration_queue
            .iter()
            .take(3)
            .map(|entry| {
                let reason = entry.reason.as_deref().unwrap_or("rehydrate");
                format!("{reason}:{}", compact_inline(&entry.summary, 32))
            })
            .collect::<Vec<_>>();
        if !rehydrate_trail.is_empty() {
            output.push_str(&format!(" rehydrate_trail={}", rehydrate_trail.join(" | ")));
        }

        if let Some(semantic) = response.semantic_consolidation.as_ref() {
            let trail = semantic
                .highlights
                .iter()
                .take(3)
                .map(|value| compact_inline(value, 40))
                .collect::<Vec<_>>();
            if !trail.is_empty() {
                output.push_str(&format!(" semantic_trail={}", trail.join(" | ")));
            }
        }
    }

    output
}

pub(crate) fn render_profile_summary(response: &AgentProfileResponse, follow: bool) -> String {
    let Some(profile) = response.profile.as_ref() else {
        return "profile=none".to_string();
    };

    let mut output = format!(
        "profile agent={} project={} namespace={} route={} intent={} summary_chars={} max_total_chars={} recall_depth={} trust_floor={} styles={}",
        profile.agent,
        profile.project.as_deref().unwrap_or("none"),
        profile.namespace.as_deref().unwrap_or("none"),
        profile.preferred_route.map(route_label).unwrap_or("none"),
        profile.preferred_intent.map(intent_label).unwrap_or("none"),
        profile
            .summary_chars
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        profile
            .max_total_chars
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        profile
            .recall_depth
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        profile
            .source_trust_floor
            .map(|value| format!("{value:.2}"))
            .unwrap_or_else(|| "none".to_string()),
        if profile.style_tags.is_empty() {
            "none".to_string()
        } else {
            profile.style_tags.join("|")
        }
    );

    if follow {
        if let Some(notes) = profile.notes.as_ref() {
            output.push_str(&format!(" notes={}", compact_inline(notes, 72)));
        }
        output.push_str(&format!(
            " created={} updated={}",
            profile.created_at.to_rfc3339(),
            profile.updated_at.to_rfc3339()
        ));
    }

    output
}

pub(crate) fn render_source_summary(response: &SourceMemoryResponse, follow: bool) -> String {
    let mut output = format!("source_memory sources={}", response.sources.len());

    if let Some(best) = response.sources.first() {
        output.push_str(&format!(
            " top={} system={} project={} namespace={} workspace={} visibility={} items={} trust={:.2} avg_confidence={:.2}",
            best.source_agent.as_deref().unwrap_or("none"),
            best.source_system.as_deref().unwrap_or("none"),
            best.project.as_deref().unwrap_or("none"),
            best.namespace.as_deref().unwrap_or("none"),
            best.workspace.as_deref().unwrap_or("none"),
            format_visibility(best.visibility),
            best.item_count,
            best.trust_score,
            best.avg_confidence
        ));
    }

    if follow {
        let trail = response
            .sources
            .iter()
            .take(3)
            .map(|source| {
                format!(
                    "{}:{}:{}:{}:{:.2}",
                    source.source_agent.as_deref().unwrap_or("none"),
                    source.source_system.as_deref().unwrap_or("none"),
                    source.workspace.as_deref().unwrap_or("none"),
                    source.item_count,
                    source.trust_score
                )
            })
            .collect::<Vec<_>>();
        if !trail.is_empty() {
            output.push_str(&format!(" trail={}", trail.join(" | ")));
        }
        if let Some(best) = response.sources.first()
            && !best.tags.is_empty()
        {
            output.push_str(&format!(" tags={}", best.tags.join("|")));
        }
    }

    output
}

pub(crate) fn render_workspace_summary(
    response: &WorkspaceMemoryResponse,
    follow: bool,
) -> String {
    let mut output = format!("workspace_memory workspaces={}", response.workspaces.len());

    if let Some(best) = response.workspaces.first() {
        output.push_str(&format!(
            " top={} visibility={} project={} namespace={} items={} sources={} trust={:.2} avg_confidence={:.2}",
            best.workspace.as_deref().unwrap_or("none"),
            format_visibility(best.visibility),
            best.project.as_deref().unwrap_or("none"),
            best.namespace.as_deref().unwrap_or("none"),
            best.item_count,
            best.source_lane_count,
            best.trust_score,
            best.avg_confidence
        ));
    }

    if follow {
        let trail = response
            .workspaces
            .iter()
            .take(3)
            .map(|workspace| {
                format!(
                    "{}:{}:{}:{:.2}",
                    workspace.workspace.as_deref().unwrap_or("none"),
                    format_visibility(workspace.visibility),
                    workspace.item_count,
                    workspace.trust_score
                )
            })
            .collect::<Vec<_>>();
        if !trail.is_empty() {
            output.push_str(&format!(" trail={}", trail.join(" | ")));
        }
        if let Some(best) = response.workspaces.first()
            && !best.tags.is_empty()
        {
            output.push_str(&format!(" tags={}", best.tags.join("|")));
        }
    }

    output
}

pub(crate) fn render_resume_prompt(snapshot: &crate::ResumeSnapshot) -> String {
    let mut output = String::new();
    output.push_str("# memd resume\n\n");
    output.push_str(&format!(
        "- project: {}\n- namespace: {}\n- agent: {}\n- workspace: {}\n- visibility: {}\n- route: {}\n- intent: {}\n",
        snapshot.project.as_deref().unwrap_or("none"),
        snapshot.namespace.as_deref().unwrap_or("none"),
        snapshot.agent.as_deref().unwrap_or("none"),
        snapshot.workspace.as_deref().unwrap_or("none"),
        snapshot.visibility.as_deref().unwrap_or("all"),
        snapshot.route,
        snapshot.intent,
    ));

    let current_task = render_current_task_snapshot(snapshot);
    if !current_task.is_empty() {
        output.push_str("\n## Current Task Snapshot\n\n");
        output.push_str(&current_task);
    }

    if !snapshot.change_summary.is_empty() {
        output.push_str("\n## Since Last Resume\n\n");
        for change in snapshot.change_summary.iter().take(6) {
            output.push_str(&format!("- {}\n", compact_inline(change, 180)));
        }
    }

    output.push_str("\n## Working Memory\n\n");
    if snapshot.working.records.is_empty() {
        output.push_str("- none\n");
    } else {
        for record in snapshot.working.records.iter().take(8) {
            output.push_str(&format!("- {}\n", compact_inline(&record.record, 220)));
        }
    }

    if !snapshot.working.rehydration_queue.is_empty() {
        output.push_str("\n## Rehydration Queue\n\n");
        for artifact in snapshot.working.rehydration_queue.iter().take(4) {
            output.push_str(&format!(
                "- {}: {}\n",
                artifact.label,
                compact_inline(&artifact.summary, 180)
            ));
        }
    }

    if !snapshot.inbox.items.is_empty() {
        output.push_str("\n## Inbox\n\n");
        for item in snapshot.inbox.items.iter().take(5) {
            let reasons = if item.reasons.is_empty() {
                "none".to_string()
            } else {
                compact_inline(&item.reasons.join(", "), 100)
            };
            output.push_str(&format!(
                "- {:?} {:?}: {} | reasons: {}\n",
                item.item.kind,
                item.item.status,
                compact_inline(&item.item.content, 160),
                reasons
            ));
        }
    }

    if !snapshot.workspaces.workspaces.is_empty() {
        output.push_str("\n## Workspace Lanes\n\n");
        for workspace in snapshot.workspaces.workspaces.iter().take(4) {
            output.push_str(&format!(
                "- {} / {} / {} | items {} | sources {} | trust {:.2}\n",
                workspace.project.as_deref().unwrap_or("none"),
                workspace.namespace.as_deref().unwrap_or("none"),
                workspace.workspace.as_deref().unwrap_or("none"),
                workspace.item_count,
                workspace.source_lane_count,
                workspace.trust_score
            ));
        }
    }

    if let Some(semantic) = snapshot.semantic.as_ref().filter(|semantic| !semantic.items.is_empty()) {
        output.push_str("\n## Semantic Recall\n\n");
        for item in semantic.items.iter().take(4) {
            output.push_str(&format!(
                "- {}{} | score {:.2}\n",
                compact_inline(&item.content, 180),
                item.source
                    .as_deref()
                    .map(|source| format!(" | source {}", compact_inline(source, 48)))
                    .unwrap_or_default(),
                item.score
            ));
        }
    }

    output
}

fn render_current_task_snapshot(snapshot: &crate::ResumeSnapshot) -> String {
    let mut output = String::new();

    if let Some(focus) = snapshot.working.records.first() {
        output.push_str(&format!(
            "- focus: {}\n",
            compact_inline(&focus.record, 180)
        ));
    }

    if let Some(blocker) = snapshot.inbox.items.first() {
        output.push_str(&format!(
            "- pressure: {:?} {:?}: {}\n",
            blocker.item.kind,
            blocker.item.status,
            compact_inline(&blocker.item.content, 160)
        ));
    }

    if let Some(next) = snapshot.working.rehydration_queue.first() {
        output.push_str(&format!(
            "- next_recovery: {}: {}\n",
            next.label,
            compact_inline(&next.summary, 160)
        ));
    }

    if !snapshot.workspaces.workspaces.is_empty() {
        let lane = &snapshot.workspaces.workspaces[0];
        output.push_str(&format!(
            "- lane: {} / {} / {} | visibility {} | trust {:.2}\n",
            lane.project.as_deref().unwrap_or("none"),
            lane.namespace.as_deref().unwrap_or("none"),
            lane.workspace.as_deref().unwrap_or("none"),
            format_visibility(lane.visibility),
            lane.trust_score
        ));
    }

    output
}

pub(crate) fn render_handoff_prompt(snapshot: &crate::HandoffSnapshot) -> String {
    let mut output = String::new();
    output.push_str("# memd handoff\n\n");
    output.push_str(&format!(
        "- generated_at: {}\n- project: {}\n- namespace: {}\n- agent: {}\n- workspace: {}\n- visibility: {}\n- route: {}\n- intent: {}\n",
        snapshot.generated_at.to_rfc3339(),
        snapshot.resume.project.as_deref().unwrap_or("none"),
        snapshot.resume.namespace.as_deref().unwrap_or("none"),
        snapshot.resume.agent.as_deref().unwrap_or("none"),
        snapshot.resume.workspace.as_deref().unwrap_or("none"),
        snapshot.resume.visibility.as_deref().unwrap_or("all"),
        snapshot.resume.route,
        snapshot.resume.intent,
    ));

    output.push_str("\n## Working Memory\n\n");
    if snapshot.resume.working.records.is_empty() {
        output.push_str("- none\n");
    } else {
        for record in snapshot.resume.working.records.iter().take(8) {
            output.push_str(&format!("- {}\n", compact_inline(&record.record, 220)));
        }
    }

    if !snapshot.resume.working.rehydration_queue.is_empty() {
        output.push_str("\n## Rehydration Queue\n\n");
        for artifact in snapshot.resume.working.rehydration_queue.iter().take(5) {
            output.push_str(&format!(
                "- {}: {}\n",
                artifact.label,
                compact_inline(&artifact.summary, 180)
            ));
        }
    }

    if !snapshot.resume.inbox.items.is_empty() {
        output.push_str("\n## Inbox Pressure\n\n");
        for item in snapshot.resume.inbox.items.iter().take(5) {
            let reasons = if item.reasons.is_empty() {
                "none".to_string()
            } else {
                compact_inline(&item.reasons.join(", "), 100)
            };
            output.push_str(&format!(
                "- {:?} {:?}: {} | reasons: {}\n",
                item.item.kind,
                item.item.status,
                compact_inline(&item.item.content, 160),
                reasons
            ));
        }
    }

    if !snapshot.resume.workspaces.workspaces.is_empty() {
        output.push_str("\n## Workspace Lanes\n\n");
        for workspace in snapshot.resume.workspaces.workspaces.iter().take(5) {
            output.push_str(&format!(
                "- {} / {} / {} | visibility {} | items {} | sources {} | trust {:.2}\n",
                workspace.project.as_deref().unwrap_or("none"),
                workspace.namespace.as_deref().unwrap_or("none"),
                workspace.workspace.as_deref().unwrap_or("none"),
                format_visibility(workspace.visibility),
                workspace.item_count,
                workspace.source_lane_count,
                workspace.trust_score
            ));
        }
    }

    if let Some(semantic) = snapshot
        .resume
        .semantic
        .as_ref()
        .filter(|semantic| !semantic.items.is_empty())
    {
        output.push_str("\n## Semantic Recall\n\n");
        for item in semantic.items.iter().take(5) {
            output.push_str(&format!(
                "- {}{} | score {:.2}\n",
                compact_inline(&item.content, 180),
                item.source
                    .as_deref()
                    .map(|source| format!(" | source {}", compact_inline(source, 48)))
                    .unwrap_or_default(),
                item.score
            ));
        }
    }

    if !snapshot.sources.sources.is_empty() {
        output.push_str("\n## Source Lanes\n\n");
        for source in snapshot.sources.sources.iter().take(5) {
            output.push_str(&format!(
                "- {} / {} | workspace {} | visibility {} | items {} | trust {:.2} | confidence {:.2}\n",
                source.source_agent.as_deref().unwrap_or("none"),
                source.source_system.as_deref().unwrap_or("none"),
                source.workspace.as_deref().unwrap_or("none"),
                format_visibility(source.visibility),
                source.item_count,
                source.trust_score,
                source.avg_confidence
            ));
        }
    }

    output
}

pub(crate) fn render_consolidate_summary(
    response: &memd_schema::MemoryConsolidationResponse,
    follow: bool,
) -> String {
    let mut output = format!(
        "consolidate scanned={} groups={} consolidated={} duplicates={} events={}",
        response.scanned,
        response.groups,
        response.consolidated,
        response.duplicates,
        response.events
    );

    if follow && !response.highlights.is_empty() {
        let highlights = response
            .highlights
            .iter()
            .take(3)
            .map(|value| compact_inline(value, 40))
            .collect::<Vec<_>>();
        output.push_str(&format!(" trail={}", highlights.join(" | ")));
    }

    output
}

pub(crate) fn render_maintenance_report_summary(
    response: &memd_schema::MemoryMaintenanceReportResponse,
    follow: bool,
) -> String {
    let mut output = format!(
        "maintenance reinforced={} cooled={} consolidated={} stale={} skipped={}",
        response.reinforced_candidates,
        response.cooled_candidates,
        response.consolidated_candidates,
        response.stale_items,
        response.skipped
    );

    if follow && !response.highlights.is_empty() {
        let highlights = response
            .highlights
            .iter()
            .take(3)
            .map(|value| compact_inline(value, 40))
            .collect::<Vec<_>>();
        output.push_str(&format!(" trail={}", highlights.join(" | ")));
    }

    output
}

pub(crate) fn render_eval_summary(response: &crate::BundleEvalResponse) -> String {
    let mut output = format!(
        "eval status={} score={} baseline={} delta={} agent={} workspace={} working={} context={} rehydration={} inbox={} lanes={} semantic={}",
        response.status,
        response.score,
        response
            .baseline_score
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        response
            .score_delta
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        response.agent.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.working_records,
        response.context_records,
        response.rehydration_items,
        response.inbox_items,
        response.workspace_lanes,
        response.semantic_hits
    );

    if !response.findings.is_empty() {
        let findings = response
            .findings
            .iter()
            .take(3)
            .map(|value| compact_inline(value, 56))
            .collect::<Vec<_>>();
        output.push_str(&format!(" findings={}", findings.join(" | ")));
    }

    if !response.changes.is_empty() {
        let changes = response
            .changes
            .iter()
            .take(3)
            .map(|value| compact_inline(value, 40))
            .collect::<Vec<_>>();
        output.push_str(&format!(" changes={}", changes.join(" | ")));
    }

    if !response.recommendations.is_empty() {
        let recommendations = response
            .recommendations
            .iter()
            .take(2)
            .map(|value| compact_inline(value, 44))
            .collect::<Vec<_>>();
        output.push_str(&format!(" next={}", recommendations.join(" | ")));
    }

    output
}

pub(crate) fn render_repair_summary(response: &RepairMemoryResponse, follow: bool) -> String {
    let mut output = format!(
        "repair mode={} item={} status={} confidence={:.2} reasons={}",
        format!("{:?}", response.mode).to_ascii_lowercase(),
        short_uuid(response.item.id),
        format!("{:?}", response.item.status).to_ascii_lowercase(),
        response.item.confidence,
        response.reasons.join("|")
    );

    if follow {
        output.push_str(&format!(
            " source_agent={} source_system={} source_path={}",
            response.item.source_agent.as_deref().unwrap_or("none"),
            response.item.source_system.as_deref().unwrap_or("none"),
            response.item.source_path.as_deref().unwrap_or("none")
        ));
        if !response.item.tags.is_empty() {
            output.push_str(&format!(" tags={}", response.item.tags.join("|")));
        }
    }

    output
}

pub(crate) fn render_explain_summary(response: &ExplainMemoryResponse, follow: bool) -> String {
    let mut output = format!(
        "explain item={} route={} intent={} status={} confidence={:.2} preferred={} branch={} siblings={} retrievals={} entity={} events={} sources={} rehydrate={} hooks={} reasons={}",
        short_uuid(response.item.id),
        route_label(response.route),
        intent_label(response.intent),
        format!("{:?}", response.item.status).to_ascii_lowercase(),
        response.item.confidence,
        response.item.preferred,
        response.item.belief_branch.as_deref().unwrap_or("none"),
        response.branch_siblings.len(),
        response.retrieval_feedback.total_retrievals,
        response
            .entity
            .as_ref()
            .map(|entity| format!("{}:{}", short_uuid(entity.id), entity.entity_type))
            .unwrap_or_else(|| "none".to_string()),
        response.events.len(),
        response.sources.len(),
        response.rehydration.len(),
        response.policy_hooks.len(),
        compact_inline(&response.reasons.join("|"), 96)
    );

    if follow {
        if let Some(first_event) = response.events.first() {
            output.push_str(&format!(
                " latest_event={} trail={}",
                first_event.event_type,
                response
                    .events
                    .iter()
                    .take(3)
                    .map(|event| format!("{}:{}", event.event_type, compact_inline(&event.summary, 36)))
                    .collect::<Vec<_>>()
                    .join(" | ")
            ));
        }
        if let Some(best_source) = response.sources.first() {
            output.push_str(&format!(
                " top_source={} system={} trust={:.2} avg_confidence={:.2}",
                best_source.source_agent.as_deref().unwrap_or("none"),
                best_source.source_system.as_deref().unwrap_or("none"),
                best_source.trust_score,
                best_source.avg_confidence
            ));
            if !best_source.tags.is_empty() {
                output.push_str(&format!(" tags={}", best_source.tags.join("|")));
            }
        }
        if !response.branch_siblings.is_empty() {
            let siblings = response
                .branch_siblings
                .iter()
                .take(3)
                .map(|sibling| {
                    format!(
                        "{}:{}:{:.2}:{}",
                        sibling.belief_branch.as_deref().unwrap_or("none"),
                        short_uuid(sibling.id),
                        sibling.confidence,
                        if sibling.preferred { "preferred" } else { "candidate" }
                    )
                })
                .collect::<Vec<_>>();
            output.push_str(&format!(" sibling_branches={}", siblings.join(" | ")));
        }
        if !response.retrieval_feedback.by_surface.is_empty() {
            let surfaces = response
                .retrieval_feedback
                .by_surface
                .iter()
                .take(4)
                .map(|surface| format!("{}:{}", surface.surface, surface.count))
                .collect::<Vec<_>>();
            output.push_str(&format!(" retrieval_surfaces={}", surfaces.join("|")));
        }
        let hooks = response
            .policy_hooks
            .iter()
            .take(4)
            .cloned()
            .collect::<Vec<_>>();
        if !hooks.is_empty() {
            output.push_str(&format!(" hooks={}", hooks.join("|")));
        }
        let trail = response
            .rehydration
            .iter()
            .take(3)
            .map(|artifact| {
                format!(
                    "{}:{}:{}",
                    artifact.kind,
                    artifact.label,
                    compact_inline(&artifact.summary, 32)
                )
            })
            .collect::<Vec<_>>();
        if !trail.is_empty() {
            output.push_str(&format!(" rehydration={}", trail.join(" | ")));
        }
    }

    output
}

fn compact_inline(value: &str, max_chars: usize) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() <= max_chars {
        return normalized;
    }

    let mut compact = String::with_capacity(max_chars + 3);
    for ch in normalized.chars().take(max_chars.saturating_sub(3)) {
        compact.push(ch);
    }
    compact.push_str("...");
    compact
}

pub(crate) fn short_uuid(id: uuid::Uuid) -> String {
    id.to_string().chars().take(8).collect()
}

fn route_label(route: RetrievalRoute) -> &'static str {
    match route {
        RetrievalRoute::Auto => "auto",
        RetrievalRoute::LocalOnly => "local_only",
        RetrievalRoute::SyncedOnly => "synced_only",
        RetrievalRoute::ProjectOnly => "project_only",
        RetrievalRoute::GlobalOnly => "global_only",
        RetrievalRoute::LocalFirst => "local_first",
        RetrievalRoute::SyncedFirst => "synced_first",
        RetrievalRoute::ProjectFirst => "project_first",
        RetrievalRoute::GlobalFirst => "global_first",
        RetrievalRoute::All => "all",
    }
}

fn intent_label(intent: RetrievalIntent) -> &'static str {
    match intent {
        RetrievalIntent::General => "general",
        RetrievalIntent::CurrentTask => "current_task",
        RetrievalIntent::Decision => "decision",
        RetrievalIntent::Runbook => "runbook",
        RetrievalIntent::Procedural => "procedural",
        RetrievalIntent::SelfModel => "self_model",
        RetrievalIntent::Topology => "topology",
        RetrievalIntent::Preference => "preference",
        RetrievalIntent::Fact => "fact",
        RetrievalIntent::Pattern => "pattern",
    }
}

fn format_visibility(value: memd_schema::MemoryVisibility) -> &'static str {
    match value {
        memd_schema::MemoryVisibility::Private => "private",
        memd_schema::MemoryVisibility::Workspace => "workspace",
        memd_schema::MemoryVisibility::Public => "public",
    }
}
