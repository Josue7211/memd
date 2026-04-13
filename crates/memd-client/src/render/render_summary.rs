use super::*;

#[path = "render_memory_summary.rs"]
mod render_memory_summary;
pub(crate) use render_memory_summary::*;

pub(crate) fn render_resume_prompt(snapshot: &crate::ResumeSnapshot) -> String {
    let mut output = String::new();
    let continuity = snapshot.continuity_capsule();
    output.push_str("# r\n\n");
    output.push_str(&format!(
        "- p={} | n={} | a={} | w={} | v={} | r={} | i={}\n",
        snapshot.project.as_deref().unwrap_or("none"),
        snapshot.namespace.as_deref().unwrap_or("none"),
        snapshot.agent.as_deref().unwrap_or("none"),
        snapshot.workspace.as_deref().unwrap_or("none"),
        snapshot.visibility.as_deref().unwrap_or("all"),
        snapshot.route,
        snapshot.intent,
    ));
    output.push_str("\n## Context Budget\n\n");
    output.push_str(&format!(
        "- ch={} | tok={} | p={} | dup={} | use={}/{}",
        snapshot.estimated_prompt_chars(),
        snapshot.estimated_prompt_tokens(),
        snapshot.context_pressure(),
        snapshot.redundant_context_items(),
        snapshot.working.used_chars,
        snapshot.working.budget_chars,
    ));
    if let Some(age_minutes) = snapshot.resume_state_age_minutes {
        output.push_str(&format!(" | age={}", age_minutes));
    }
    output.push_str(&format!(" | ref={}\n", snapshot.refresh_recommended));
    let hints = snapshot.optimization_hints();
    if !hints.is_empty() {
        output.push_str(&format!(
            "- h={}\n",
            hints
                .iter()
                .take(4)
                .map(|hint| compact_inline(hint, 180))
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }

    let current_task = render_current_task_snapshot(snapshot);
    if !current_task.is_empty() {
        output.push_str("\n## T\n\n");
        output.push_str(&current_task);
    }

    if continuity.current_task.is_some()
        || continuity.resume_point.is_some()
        || continuity.changed.is_some()
        || continuity.next_action.is_some()
        || continuity.blocker.is_some()
    {
        output.push_str("\n## C\n\n");
        if let Some(current_task) = continuity.current_task.as_deref() {
            output.push_str(&format!("- doing={}\n", compact_inline(current_task, 180)));
        }
        if let Some(resume_point) = continuity.resume_point.as_deref() {
            output.push_str(&format!(
                "- left_off={}\n",
                compact_inline(resume_point, 180)
            ));
        }
        if let Some(changed) = continuity.changed.as_deref() {
            output.push_str(&format!("- changed={}\n", compact_inline(changed, 180)));
        }
        if let Some(next_action) = continuity.next_action.as_deref() {
            output.push_str(&format!("- next={}\n", compact_inline(next_action, 180)));
        }
        if let Some(blocker) = continuity.blocker.as_deref() {
            output.push_str(&format!("- blocker={}\n", compact_inline(blocker, 180)));
        }
    }

    let event_spine = snapshot.event_spine();
    if !event_spine.is_empty() || !snapshot.recent_repo_changes.is_empty() {
        output.push_str("\n## E+LT\n\n");
        let event_part = if event_spine.is_empty() {
            None
        } else {
            let compacted = event_spine
                .iter()
                .take(2)
                .map(|change| compact_inline(change, 180))
                .collect::<Vec<_>>()
                .join(" | ");
            Some(format!("E={}", compacted))
        };
        let lt_part = if snapshot.recent_repo_changes.is_empty() {
            None
        } else {
            let compacted = snapshot
                .recent_repo_changes
                .iter()
                .take(2)
                .map(|change| compact_inline(change, 180))
                .collect::<Vec<_>>()
                .join(" | ");
            Some(format!("LT={}", compacted))
        };
        let mut parts = Vec::new();
        if let Some(part) = event_part {
            parts.push(part);
        }
        if let Some(part) = lt_part {
            parts.push(part);
        }
        output.push_str(&format!("- {}\n", parts.join(" | ")));
    }

    output.push_str("\n## W\n\n");
    if snapshot.working.records.is_empty() {
        output.push_str("- none\n");
    } else {
        let records = snapshot
            .working
            .records
            .iter()
            .take(2)
            .map(|record| compact_inline(&record.record, 220))
            .collect::<Vec<_>>();
        output.push_str(&format!("- w={}", records.join(" | ")));
        if snapshot.working.records.len() > 2 {
            output.push_str(&format!(" (+{} more)", snapshot.working.records.len() - 2));
        }
        output.push('\n');
    }

    let mut ri_parts = Vec::new();
    if !snapshot.working.rehydration_queue.is_empty() {
        for artifact in snapshot.working.rehydration_queue.iter().take(4) {
            ri_parts.push(format!(
                "r={}:{}",
                artifact.label,
                compact_inline(&artifact.summary, 180)
            ));
        }
    }
    if !snapshot.inbox.items.is_empty() {
        for item in snapshot.inbox.items.iter().take(5) {
            let reasons = if item.reasons.is_empty() {
                "none".to_string()
            } else {
                compact_inline(&item.reasons.join(", "), 100)
            };
            ri_parts.push(format!(
                "i={:?}/{:?}:{}|r={}",
                item.item.kind,
                item.item.status,
                compact_inline(&item.item.content, 160),
                reasons
            ));
        }
    }
    if !ri_parts.is_empty() {
        output.push_str("\n## RI\n\n");
        output.push_str(&format!("- {}\n", ri_parts.join(" | ")));
    }

    if let Some(first) = snapshot.workspaces.workspaces.first() {
        output.push_str("\n## L\n\n");
        let extras = snapshot.workspaces.workspaces.len() - 1;
        output.push_str(&format!(
            "- l={}/{}/{} | v={} | it={} | src={} | tr={:.2}{} \n",
            first.project.as_deref().unwrap_or("none"),
            first.namespace.as_deref().unwrap_or("none"),
            first.workspace.as_deref().unwrap_or("none"),
            format_visibility(first.visibility),
            first.item_count,
            first.source_lane_count,
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
            .map(|item| format!("{}@{:.2}", compact_inline(&item.content, 180), item.score))
            .collect::<Vec<_>>();
        sc_parts.push(format!("S={}", items.join(" | ")));
    }

    if !sc_parts.is_empty() {
        output.push_str("\n## S\n\n");
        output.push_str(&format!("- {}\n", sc_parts.join(" | ")));
    }

    output
}

fn render_current_task_snapshot(snapshot: &crate::ResumeSnapshot) -> String {
    let mut output = String::new();
    let continuity = snapshot.continuity_capsule();

    if continuity.current_task.is_some()
        || continuity.resume_point.is_some()
        || continuity.changed.is_some()
        || continuity.next_action.is_some()
        || continuity.blocker.is_some()
    {
        if let Some(current_task) = continuity.current_task.as_deref() {
            output.push_str(&format!("- doing={}\n", compact_inline(current_task, 180)));
        }
        if let Some(resume_point) = continuity.resume_point.as_deref() {
            output.push_str(&format!(
                "- left_off={}\n",
                compact_inline(resume_point, 180)
            ));
        }
        if let Some(changed) = continuity.changed.as_deref() {
            output.push_str(&format!("- changed={}\n", compact_inline(changed, 180)));
        }
        if let Some(next_action) = continuity.next_action.as_deref() {
            output.push_str(&format!("- next={}\n", compact_inline(next_action, 180)));
        }
        if let Some(blocker) = continuity.blocker.as_deref() {
            output.push_str(&format!("- blocker={}\n", compact_inline(blocker, 180)));
        }
    }

    let capsule = snapshot.workflow_capsule();
    if !capsule.is_empty() {
        let summary = capsule
            .iter()
            .take(4)
            .map(|line| compact_inline(line, 180))
            .collect::<Vec<_>>()
            .join(" | ");
        output.push_str(&format!("- t={}\n", summary));
    }

    output
}

pub(crate) fn render_handoff_prompt(snapshot: &crate::HandoffSnapshot) -> String {
    let mut output = String::new();
    let continuity = snapshot.resume.continuity_capsule();
    output.push_str("# h\n\n");
    output.push_str(&format!(
        "- at={} | p={} | n={} | a={} | w={} | v={} | r={} | i={}\n",
        snapshot.generated_at.to_rfc3339(),
        snapshot.resume.project.as_deref().unwrap_or("none"),
        snapshot.resume.namespace.as_deref().unwrap_or("none"),
        snapshot.resume.agent.as_deref().unwrap_or("none"),
        snapshot.resume.workspace.as_deref().unwrap_or("none"),
        snapshot.resume.visibility.as_deref().unwrap_or("all"),
        snapshot.resume.route,
        snapshot.resume.intent,
    ));

    output.push_str("\n## W\n\n");
    if snapshot.resume.working.records.is_empty() {
        output.push_str("- none\n");
    } else {
        let records = snapshot
            .resume
            .working
            .records
            .iter()
            .take(2)
            .map(|record| compact_inline(&record.record, 220))
            .collect::<Vec<_>>();
        output.push_str(&format!("- w={}", records.join(" | ")));
        if snapshot.resume.working.records.len() > 2 {
            output.push_str(&format!(
                " (+{} more)",
                snapshot.resume.working.records.len() - 2
            ));
        }
        output.push('\n');
    }

    if continuity.current_task.is_some()
        || continuity.resume_point.is_some()
        || continuity.changed.is_some()
        || continuity.next_action.is_some()
        || continuity.blocker.is_some()
    {
        output.push_str("\n## C\n\n");
        if let Some(current_task) = continuity.current_task.as_deref() {
            output.push_str(&format!("- doing={}\n", compact_inline(current_task, 180)));
        }
        if let Some(resume_point) = continuity.resume_point.as_deref() {
            output.push_str(&format!(
                "- left_off={}\n",
                compact_inline(resume_point, 180)
            ));
        }
        if let Some(changed) = continuity.changed.as_deref() {
            output.push_str(&format!("- changed={}\n", compact_inline(changed, 180)));
        }
        if let Some(next_action) = continuity.next_action.as_deref() {
            output.push_str(&format!("- next={}\n", compact_inline(next_action, 180)));
        }
        if let Some(blocker) = continuity.blocker.as_deref() {
            output.push_str(&format!("- blocker={}\n", compact_inline(blocker, 180)));
        }
    }

    let mut ri_parts_resume = Vec::new();
    if !snapshot.resume.working.rehydration_queue.is_empty() {
        for artifact in snapshot.resume.working.rehydration_queue.iter().take(5) {
            ri_parts_resume.push(format!(
                "r={}:{}",
                artifact.label,
                compact_inline(&artifact.summary, 180)
            ));
        }
    }
    if !snapshot.resume.inbox.items.is_empty() {
        for item in snapshot.resume.inbox.items.iter().take(5) {
            let reasons = if item.reasons.is_empty() {
                "none".to_string()
            } else {
                compact_inline(&item.reasons.join(", "), 100)
            };
            ri_parts_resume.push(format!(
                "i={:?}/{:?}:{}|r={}",
                item.item.kind,
                item.item.status,
                compact_inline(&item.item.content, 160),
                reasons
            ));
        }
    }
    if !ri_parts_resume.is_empty() {
        output.push_str("\n## RI\n\n");
        output.push_str(&format!("- {}\n", ri_parts_resume.join(" | ")));
    }

    if !snapshot.resume.workspaces.workspaces.is_empty() {
        output.push_str("\n## L\n\n");
        for workspace in snapshot.resume.workspaces.workspaces.iter().take(5) {
            output.push_str(&format!(
                "- {}/{}/{} | v={} | it={} | src={} | tr={:.2}\n",
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

    let mut sc_resume_parts = Vec::new();
    if let Some(semantic) = snapshot
        .resume
        .semantic
        .as_ref()
        .filter(|semantic| !semantic.items.is_empty())
    {
        let items = semantic
            .items
            .iter()
            .take(2)
            .map(|item| format!("{}@{:.2}", compact_inline(&item.content, 180), item.score))
            .collect::<Vec<_>>();
        sc_resume_parts.push(format!("S={}", items.join(" | ")));
    }

    if !sc_resume_parts.is_empty() {
        output.push_str("\n## S\n\n");
        output.push_str(&format!("- {}\n", sc_resume_parts.join(" | ")));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_resume_snapshot() -> crate::ResumeSnapshot {
        crate::ResumeSnapshot {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            context: memd_schema::CompactContextResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                records: Vec::new(),
            },
            working: memd_schema::WorkingMemoryResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                budget_chars: 1600,
                used_chars: 60,
                remaining_chars: 1540,
                truncated: false,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 8,
                    max_chars_per_item: 220,
                    budget_chars: 1600,
                    rehydration_limit: 4,
                },
                records: vec![memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "Follow the active current-task lane".to_string(),
                }],
                evicted: Vec::new(),
                rehydration_queue: vec![memd_schema::MemoryRehydrationRecord {
                    id: None,
                    kind: "source".to_string(),
                    label: "handoff".to_string(),
                    summary: "Reload the shared workspace handoff".to_string(),
                    reason: None,
                    source_agent: None,
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    recorded_at: None,
                }],
                traces: Vec::new(),
                semantic_consolidation: None,
            procedures: vec![],
            },
            inbox: memd_schema::MemoryInboxResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                items: vec![memd_schema::InboxMemoryItem {
                    item: memd_schema::MemoryItem {
                        id: uuid::Uuid::new_v4(),
                        content: "One review item is still open".to_string(),
                        redundancy_key: None,
                        belief_branch: None,
                        preferred: true,
                        kind: memd_schema::MemoryKind::Status,
                        scope: memd_schema::MemoryScope::Project,
                        project: Some("demo".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: Some("team-alpha".to_string()),
                        visibility: memd_schema::MemoryVisibility::Workspace,
                        source_agent: None,
                        source_system: None,
                        source_path: None,
                        source_quality: None,
                        confidence: 0.8,
                        ttl_seconds: Some(86_400),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        last_verified_at: None,
                        supersedes: Vec::new(),
                        tags: vec!["checkpoint".to_string()],
                        status: memd_schema::MemoryStatus::Active,
                        stage: memd_schema::MemoryStage::Candidate,
                    },
                    reasons: vec!["stale".to_string()],
                }],
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: vec![memd_schema::WorkspaceMemoryRecord {
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: memd_schema::MemoryVisibility::Workspace,
                    item_count: 4,
                    active_count: 3,
                    candidate_count: 1,
                    contested_count: 0,
                    source_lane_count: 1,
                    avg_confidence: 0.84,
                    trust_score: 0.91,
                    last_seen_at: None,
                    tags: Vec::new(),
                }],
            },
            sources: memd_schema::SourceMemoryResponse {
                sources: Vec::new(),
            },
            semantic: None,
            claims: crate::SessionClaimsState::default(),
            recent_repo_changes: vec!["status M crates/memd-client/src/render.rs".to_string()],
            change_summary: vec!["focus -> Follow the active current-task lane".to_string()],
            resume_state_age_minutes: None,
            refresh_recommended: false,
        }
    }

    #[test]
    fn render_resume_prompt_surfaces_explicit_continuity_answers() {
        let prompt = render_resume_prompt(&sample_resume_snapshot());
        assert!(prompt.contains("## T"));
        assert!(prompt.contains("## C"));
        assert!(prompt.contains("- doing="));
        assert!(prompt.contains("- left_off="));
        assert!(prompt.contains("- changed="));
        assert!(prompt.contains("- next="));
        assert!(prompt.contains("- blocker="));
    }

    #[test]
    fn render_handoff_prompt_surfaces_explicit_continuity_answers() {
        let handoff = crate::HandoffSnapshot {
            generated_at: chrono::Utc::now(),
            resume: sample_resume_snapshot(),
            sources: memd_schema::SourceMemoryResponse {
                sources: Vec::new(),
            },
            target_session: Some("session-noether".to_string()),
            target_bundle: Some(".memd".to_string()),
        };

        let prompt = render_handoff_prompt(&handoff);
        assert!(prompt.contains("## C"));
        assert!(prompt.contains("- doing="));
        assert!(prompt.contains("- left_off="));
        assert!(prompt.contains("- changed="));
        assert!(prompt.contains("- next="));
        assert!(prompt.contains("- blocker=One review item is still open"));
    }
}
