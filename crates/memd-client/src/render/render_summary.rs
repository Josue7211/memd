use super::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::bundle::infer_bundle_project_root;

#[path = "render_memory_summary.rs"]
mod render_memory_summary;
pub(crate) use render_memory_summary::*;

fn compact_next_action(next_action: &str) -> String {
    let body = next_action
        .split_once(" | c=")
        .map(|(_, content)| content.trim())
        .unwrap_or_else(|| next_action.trim());
    if let Some(id) = extract_compact_record_id(next_action) {
        format!("{id}: {}", compact_inline(body, 160))
    } else {
        compact_inline(body, 180)
    }
}

fn extract_compact_record_id(line: &str) -> Option<String> {
    let start = line.find("id=")?;
    let rest = &line[start + 3..];
    let end = rest
        .find(|ch: char| ch.is_whitespace() || ch == '|')
        .unwrap_or(rest.len());
    let id = rest[..end].trim();
    (!id.is_empty()).then(|| id.to_string())
}

fn handoff_proof_blocker_detail(snapshot: &crate::HandoffSnapshot) -> Option<String> {
    let target_bundle = Path::new(snapshot.target_bundle.as_deref()?);
    let project_root = infer_bundle_project_root(target_bundle)?;
    let report_dir = project_root
        .join("docs")
        .join("verification")
        .join("25-5-memory-os-runs");
    let mut details = Vec::new();
    if let Some(report) =
        latest_handoff_report_with_suffix(&report_dir, "supermemory-head-to-head.json")
    {
        if let Some(value) = read_handoff_json_report(&report) {
            if handoff_report_status_is_blocked(&value) {
                if let Some(items) = handoff_json_string_array(&value, "missing_requirements") {
                    if !items.is_empty() {
                        details.push(format!(
                            "supermemory:missing_requirements={}",
                            items.join(",")
                        ));
                    }
                }
            }
        }
    }
    if let Some(report) =
        latest_handoff_report_with_suffix(&report_dir, "external-public-full.json")
    {
        if let Some(value) = read_handoff_json_report(&report) {
            if handoff_report_status_is_blocked(&value) {
                if let Some(items) = handoff_json_string_array(&value, "missing_explicit_env") {
                    if !items.is_empty() {
                        details.push(format!(
                            "full_public:missing_explicit_env={}",
                            items.join(",")
                        ));
                    }
                }
            }
        }
    }
    (!details.is_empty()).then(|| details.join(";"))
}

fn latest_handoff_report_with_suffix(report_dir: &Path, suffix: &str) -> Option<PathBuf> {
    fs::read_dir(report_dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(suffix))
        })
        .filter_map(|path| {
            let modified = path.metadata().and_then(|meta| meta.modified()).ok()?;
            Some((modified, path))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

fn read_handoff_json_report(path: &Path) -> Option<serde_json::Value> {
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

fn handoff_report_status_is_blocked(value: &serde_json::Value) -> bool {
    value
        .get("status")
        .and_then(|status| status.as_str())
        .is_some_and(|status| status == "blocked")
}

fn handoff_json_string_array(value: &serde_json::Value, key: &str) -> Option<Vec<String>> {
    Some(
        value
            .get(key)?
            .as_array()?
            .iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
    )
}

fn render_handoff_user_prompt(
    snapshot: &crate::HandoffSnapshot,
    proof_blockers: Option<&str>,
) -> String {
    let continuity = snapshot.resume.continuity_capsule();
    let project = snapshot.resume.project.as_deref().unwrap_or("this repo");
    let namespace = snapshot.resume.namespace.as_deref().unwrap_or("main");
    let visibility = snapshot.resume.visibility.as_deref().unwrap_or("all");
    let next = continuity
        .next_action
        .as_deref()
        .map(compact_next_action)
        .unwrap_or_else(|| "inspect wake, dirty tree, and durable handoff continuity".to_string());
    let blocker = continuity
        .blocker
        .as_deref()
        .map(|value| compact_inline(value, 180))
        .unwrap_or_else(|| "none recorded".to_string());
    let dirty_count =
        crate::workflow::repo_dirty_count_from_changes(&snapshot.resume.recent_repo_changes);
    let proof = proof_blockers.unwrap_or("none recorded");

    format!(
        "Pick up `{project}` / `{namespace}` from memd handoff. First inspect `git status --short --untracked-files=all`, `.memd/wake.md`, and `memd lookup --output .memd --query \"handoff continuity next action\"`. Continue: {next}. Current blocker: {blocker}. Proof blockers: {proof}. Visibility: {visibility}. Dirty count at handoff: {dirty_count}. Keep commits atomic and leave tree clean."
    )
}

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
            output.push_str(&format!("- next={}\n", compact_next_action(next_action)));
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
            output.push_str(&format!("- next={}\n", compact_next_action(next_action)));
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
    let proof_blockers = handoff_proof_blocker_detail(snapshot);
    output.push_str("# h\n\n");
    output.push_str(&format!(
        "- at={} | p={} | n={} | a={} | voice={} | w={} | v={} | r={} | i={}\n",
        snapshot.generated_at.to_rfc3339(),
        snapshot.resume.project.as_deref().unwrap_or("none"),
        snapshot.resume.namespace.as_deref().unwrap_or("none"),
        snapshot.resume.agent.as_deref().unwrap_or("none"),
        snapshot.voice_mode,
        snapshot.resume.workspace.as_deref().unwrap_or("none"),
        snapshot.resume.visibility.as_deref().unwrap_or("all"),
        snapshot.resume.route,
        snapshot.resume.intent,
    ));
    let dirty_count =
        crate::workflow::repo_dirty_count_from_changes(&snapshot.resume.recent_repo_changes);
    let quality = snapshot
        .resume
        .handoff_quality
        .as_ref()
        .map(|score| {
            if score.is_acceptable() {
                format!("ready:{:.2}", score.composite)
            } else {
                format!("partial:{:.2}", score.composite)
            }
        })
        .unwrap_or_else(|| "unknown".to_string());
    output.push_str(&format!(
        "- native_handoff=true | target_session={} | target_bundle={} | dirty={} | quality={}\n",
        snapshot.target_session.as_deref().unwrap_or("none"),
        snapshot.target_bundle.as_deref().unwrap_or("none"),
        dirty_count,
        quality
    ));
    output.push_str("\n## User Prompt\n\n");
    output.push_str("- give next agent:\n");
    output.push_str("```text\n");
    output.push_str(&render_handoff_user_prompt(
        snapshot,
        proof_blockers.as_deref(),
    ));
    output.push_str("\n```\n");

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
        || proof_blockers.is_some()
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
            output.push_str(&format!("- next={}\n", compact_next_action(next_action)));
        }
        if let Some(blocker) = continuity.blocker.as_deref() {
            output.push_str(&format!("- blocker={}\n", compact_inline(blocker, 180)));
        }
        if let Some(detail) = proof_blockers.as_deref() {
            output.push_str(&format!(
                "- proof_blockers={}\n",
                compact_inline(detail, 260)
            ));
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

                compaction_quality: None,
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
                        lane: None,
                        version: 1,
                        correction_meta: None,
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
            atlas_region_hints: Vec::new(),
            handoff_quality: None,
            files_touched: Vec::new(),
            un_read_paths: Vec::new(),
            preferences: vec![
                "id=57c5a501-001c-49ec-9934-8a97826bf462 | kind=decision | tags=next-agent,materializer | upd=1778869065 | c=CURRENT NEXT ACTION: implement server-backed fresh-machine materializer before claiming capability sync works".to_string(),
            ],
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
            voice_mode: "caveman-ultra".to_string(),
            target_session: Some("session-noether".to_string()),
            target_bundle: Some(".memd".to_string()),
        };

        let prompt = render_handoff_prompt(&handoff);
        assert!(prompt.contains("voice=caveman-ultra"));
        assert!(prompt.contains("native_handoff=true"));
        assert!(prompt.contains("target_session=session-noether"));
        assert!(prompt.contains("## User Prompt"));
        assert!(prompt.contains("give next agent:"));
        assert!(prompt.contains("Pick up `demo` / `main` from memd handoff"));
        assert!(prompt.contains("Keep commits atomic and leave tree clean"));
        assert!(prompt.contains("## C"));
        assert!(prompt.contains("- doing="));
        assert!(prompt.contains("- left_off="));
        assert!(prompt.contains("- changed="));
        assert!(prompt.contains("- next="));
        assert!(
            prompt.contains("server-backed fresh-machine materializer"),
            "{prompt}"
        );
        assert!(prompt.contains("- blocker=One review item is still open"));
    }

    #[test]
    fn render_handoff_prompt_surfaces_proof_blocker_details() {
        let project = std::env::temp_dir().join(format!(
            "memd-handoff-proof-blockers-{}",
            uuid::Uuid::new_v4()
        ));
        let output = project.join(".memd");
        let report_dir = project
            .join("docs")
            .join("verification")
            .join("25-5-memory-os-runs");
        fs::create_dir_all(&output).expect("create temp bundle");
        fs::create_dir_all(&report_dir).expect("create report dir");
        fs::write(
            report_dir.join("2026-05-16-supermemory-head-to-head.json"),
            serde_json::json!({
                "status": "blocked",
                "missing_requirements": [
                    "approved_supermemory_access_route_or_process_credential",
                    "supermemory_same_fixture_replay_artifact"
                ]
            })
            .to_string(),
        )
        .expect("write Supermemory report");
        fs::write(
            report_dir.join("2026-05-16-external-public-full.json"),
            serde_json::json!({
                "status": "blocked",
                "missing_explicit_env": [
                    "ALLOW_FULL_PUBLIC_PROOF=1",
                    "PUBLIC_BENCH_LIMIT",
                    "PUBLIC_BENCH_TIMEOUT",
                    "RUN_LABEL"
                ]
            })
            .to_string(),
        )
        .expect("write full public report");

        let handoff = crate::HandoffSnapshot {
            generated_at: chrono::Utc::now(),
            resume: sample_resume_snapshot(),
            sources: memd_schema::SourceMemoryResponse {
                sources: Vec::new(),
            },
            voice_mode: "caveman-ultra".to_string(),
            target_session: Some("session-noether".to_string()),
            target_bundle: Some(output.display().to_string()),
        };

        let prompt = render_handoff_prompt(&handoff);

        assert!(prompt.contains("proof_blockers=supermemory:missing_requirements=approved_supermemory_access_route_or_process_credential,supermemory_same_fixture_replay_artifact;full_public:missing_explicit_env=ALLOW_FULL_PUBLIC_PROOF=1,PUBLIC_BENCH_LIMIT,PUBLIC_BENCH_TIMEOUT,RUN_LABEL"), "{prompt}");
        assert!(!prompt.contains("SUPERMEMORY_API_KEY"));

        fs::remove_dir_all(project).expect("cleanup proof blocker temp bundle");
    }
}
