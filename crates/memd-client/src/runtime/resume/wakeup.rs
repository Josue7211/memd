use super::*;

pub(crate) fn collect_wakeup_instruction_sources(output: &Path) -> Vec<(String, String)> {
    let Some(project_root) = infer_bundle_project_root(output) else {
        return Vec::new();
    };

    let mut sources = Vec::new();
    for relative in [
        "AGENTS.md",
        "CLAUDE.md",
        ".claude/CLAUDE.md",
        ".agents/CLAUDE.md",
        "TEAMS.md",
    ] {
        let path = project_root.join(relative);
        if let Some((snippet, _)) = read_bootstrap_source(&path, 18) {
            sources.push((relative.to_string(), snippet));
        }
    }
    sources
}

pub(crate) fn render_bundle_wakeup_markdown(
    output: &Path,
    snapshot: &ResumeSnapshot,
    verbose: bool,
) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd wake-up\n\n");
    markdown.push_str(&format!(
        "- {} / {} / {} / {} / {} / {} / {}\n\n",
        snapshot.project.as_deref().unwrap_or("none"),
        snapshot.namespace.as_deref().unwrap_or("none"),
        snapshot.agent.as_deref().unwrap_or("none"),
        snapshot.workspace.as_deref().unwrap_or("none"),
        snapshot.visibility.as_deref().unwrap_or("all"),
        snapshot.route,
        snapshot.intent,
    ));

    let instructions = collect_wakeup_instruction_sources(output);
    if !instructions.is_empty() {
        markdown.push_str("## Instructions\n\n");
        let limit = if verbose { 3 } else { 2 };
        for (source, snippet) in instructions.into_iter().take(limit) {
            markdown.push_str(&format!("- {source}: {}\n", compact_inline(&snippet, 240)));
        }
        markdown.push('\n');
    }

    let event_spine = snapshot.event_spine();
    if !event_spine.is_empty() {
        markdown.push_str("## Live\n\n");
        let limit = if verbose { 4 } else { 1 };
        for item in event_spine.iter().take(limit) {
            markdown.push_str(&format!("- {}\n", compact_inline(item, 120)));
        }
        markdown.push('\n');
    }

    markdown.push_str("## Durable Truth\n\n");
    if snapshot.context.records.is_empty() {
        markdown.push_str("- none\n\n");
    } else {
        let limit = if verbose { 4 } else { 2 };
        for item in snapshot.context.records.iter().take(limit) {
            markdown.push_str(&format!("- {}\n", compact_inline(item.record.trim(), 160)));
        }
        markdown.push('\n');
    }

    markdown.push_str("## Focus\n\n");
    if snapshot.working.records.is_empty() {
        markdown.push_str("- none\n");
    } else {
        let limit = 1;
        for item in snapshot.working.records.iter().take(limit) {
            markdown.push_str(&format!("- {}\n", compact_inline(item.record.trim(), 140)));
        }
    }
    markdown.push('\n');

    let continuity = snapshot.continuity_capsule();
    if continuity.current_task.is_some()
        || continuity.resume_point.is_some()
        || continuity.changed.is_some()
        || continuity.next_action.is_some()
        || continuity.blocker.is_some()
    {
        markdown.push_str("## Continuity\n\n");
        if let Some(current_task) = continuity.current_task.as_deref() {
            markdown.push_str(&format!("- doing={}\n", compact_inline(current_task, 140)));
        }
        if let Some(resume_point) = continuity.resume_point.as_deref() {
            markdown.push_str(&format!(
                "- left_off={}\n",
                compact_inline(resume_point, 140)
            ));
        }
        if let Some(changed) = continuity.changed.as_deref() {
            markdown.push_str(&format!("- changed={}\n", compact_inline(changed, 140)));
        }
        if let Some(next_action) = continuity.next_action.as_deref() {
            markdown.push_str(&format!("- next={}\n", compact_inline(next_action, 140)));
        }
        if let Some(blocker) = continuity.blocker.as_deref() {
            markdown.push_str(&format!("- blocker={}\n", compact_inline(blocker, 140)));
        }
        markdown.push('\n');
    }

    if verbose
        && (!snapshot.inbox.items.is_empty() || !snapshot.working.rehydration_queue.is_empty())
    {
        markdown.push_str("## Recovery\n\n");
        let recovery_limit = if verbose { 1 } else { 1 };
        for item in snapshot
            .working
            .rehydration_queue
            .iter()
            .take(recovery_limit)
        {
            markdown.push_str(&format!(
                "- {}: {}\n",
                item.label,
                compact_inline(item.summary.trim(), 120)
            ));
        }
        let inbox_limit = if verbose { 1 } else { 1 };
        for item in snapshot.inbox.items.iter().take(inbox_limit) {
            markdown.push_str(&format!(
                "- {:?}/{:?}: {}\n",
                item.item.kind,
                item.item.status,
                compact_inline(item.item.content.trim(), 120)
            ));
        }
        markdown.push('\n');
    }

    markdown.push_str("## Protocol\n\n");
    markdown.push_str("- Read first.\n");
    markdown.push_str("- Durable truth beats transcript recall.\n");
    markdown.push_str(
        "- Lookup before answers on decisions, preferences, history, or prior user corrections.\n",
    );
    markdown.push_str("- Recall: `memd lookup --output .memd --query \"...\"`.\n");
    markdown.push_str("- If the user corrects you, write the correction back instead of trusting the transcript.\n");
    markdown.push_str("- Writes: `remember-short`, `remember-decision`, `remember-preference`, `remember-long`, `capture-live`, `correct-memory`, `sync-semantic`, `watch`.\n");
    if verbose {
        markdown
            .push_str("- Wake/resume/refresh/handoff/hook capture auto-write short-term status.\n");
    }
    markdown.push_str("- Promote stable truths; do not rely on transcript recall.\n");
    let active_voice = read_bundle_voice_mode(output).unwrap_or_else(default_voice_mode);
    markdown.push_str(&format!("- Default voice: {}\n", active_voice));
    markdown.push_str(&format!(
        "- Reply in `{}` unless `.memd/config.json` changes it.\n",
        active_voice
    ));
    markdown.push_str(&format!(
        "- If your draft is not in `{}`, stop and rewrite it before sending.\n",
        active_voice
    ));

    markdown
}

pub(crate) fn render_bundle_wakeup_summary(snapshot: &ResumeSnapshot) -> String {
    format!(
        "wake project={} namespace={} agent={} working={} inbox={} spine={} tokens={} core={} focus=\"{}\"",
        snapshot.project.as_deref().unwrap_or("none"),
        snapshot.namespace.as_deref().unwrap_or("none"),
        snapshot.agent.as_deref().unwrap_or("none"),
        snapshot.working.records.len(),
        snapshot.inbox.items.len(),
        snapshot.event_spine().len(),
        snapshot.estimated_prompt_tokens(),
        snapshot.core_prompt_tokens(),
        snapshot
            .working
            .records
            .first()
            .map(|item| compact_inline(item.record.trim(), 96))
            .unwrap_or_else(|| "none".to_string())
    )
}

pub(crate) fn render_bundle_scope_markdown(output: &Path, snapshot: &ResumeSnapshot) -> String {
    let runtime = read_bundle_runtime_config(output).ok().flatten();
    let heartbeat_tab_id = read_bundle_heartbeat(output)
        .ok()
        .flatten()
        .and_then(|state| state.tab_id)
        .filter(|value| !value.trim().is_empty());
    let session = runtime
        .as_ref()
        .and_then(|config| config.session.as_deref())
        .filter(|value| !value.trim().is_empty());
    let tab_id = runtime
        .as_ref()
        .and_then(|config| config.tab_id.as_deref())
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .or(heartbeat_tab_id)
        .or_else(default_bundle_tab_id);
    let effective_agent = runtime
        .as_ref()
        .and_then(|config| config.agent.as_deref())
        .map(|agent| compose_agent_identity(agent, session));

    format!(
        "## Scope\n\n- project: `{}`\n- namespace: `{}`\n- agent: `{}`\n- session: `{}`\n- tab: `{}`\n- effective agent: `{}`\n- workspace: `{}`\n- visibility: `{}`\n- route: `{}`\n- intent: `{}`\n- bundle: `{}`\n",
        snapshot.project.as_deref().unwrap_or("none"),
        snapshot.namespace.as_deref().unwrap_or("none"),
        snapshot.agent.as_deref().unwrap_or("none"),
        session.unwrap_or("none"),
        tab_id.as_deref().unwrap_or("none"),
        effective_agent.as_deref().unwrap_or("none"),
        snapshot.workspace.as_deref().unwrap_or("none"),
        snapshot.visibility.as_deref().unwrap_or("all"),
        snapshot.route,
        snapshot.intent,
        output.display(),
    )
}

pub(crate) fn render_memory_page_header_suffix(output: &Path) -> String {
    let heartbeat_tab_id = read_bundle_heartbeat(output)
        .ok()
        .flatten()
        .and_then(|state| state.tab_id)
        .filter(|value| !value.trim().is_empty());
    let tab_id = read_bundle_runtime_config(output)
        .ok()
        .flatten()
        .and_then(|config| config.tab_id)
        .filter(|value| !value.trim().is_empty())
        .or(heartbeat_tab_id)
        .or_else(default_bundle_tab_id)
        .unwrap_or_else(|| "none".to_string());
    format!(" [tab={}]", tab_id)
}

#[cfg(test)]
mod tests {
    use super::*;

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
                    record:
                        "remembered project fact: memd must preserve important user corrections"
                            .to_string(),
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
                    record: "follow durable truth before transcript recall".to_string(),
                }],
                evicted: Vec::new(),
                rehydration_queue: Vec::new(),
                traces: Vec::new(),
                semantic_consolidation: None,
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
            recent_repo_changes: Vec::new(),
            change_summary: Vec::new(),
            resume_state_age_minutes: None,
            refresh_recommended: false,
        }
    }

    fn pressure_snapshot() -> ResumeSnapshot {
        ResumeSnapshot {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            context: memd_schema::CompactContextResponse {
                route: RetrievalRoute::ProjectFirst,
                intent: RetrievalIntent::CurrentTask,
                retrieval_order: vec![MemoryScope::Project, MemoryScope::Synced],
                records: (0..8)
                    .map(|index| memd_schema::CompactMemoryRecord {
                        id: uuid::Uuid::new_v4(),
                        record: format!(
                            "durable truth {}: keep the packet compact and preserve next action",
                            index
                        ),
                    })
                    .collect(),
            },
            working: memd_schema::WorkingMemoryResponse {
                route: RetrievalRoute::ProjectFirst,
                intent: RetrievalIntent::CurrentTask,
                retrieval_order: vec![MemoryScope::Project, MemoryScope::Synced],
                budget_chars: 1600,
                used_chars: 1510,
                remaining_chars: 90,
                truncated: true,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 8,
                    max_chars_per_item: 220,
                    budget_chars: 1600,
                    rehydration_limit: 4,
                },
                records: (0..6)
                    .map(|index| memd_schema::CompactMemoryRecord {
                        id: uuid::Uuid::new_v4(),
                        record: format!(
                            "working truth {}: follow durable truth before transcript recall",
                            index
                        ),
                    })
                    .collect(),
                evicted: Vec::new(),
                rehydration_queue: (0..4)
                    .map(|index| memd_schema::MemoryRehydrationRecord {
                        id: None,
                        kind: "source".to_string(),
                        label: format!("artifact-{index}"),
                        summary: format!("rehydrate source artifact {index}"),
                        reason: Some("pressure".to_string()),
                        source_agent: Some("codex@test".to_string()),
                        source_system: Some("hook-capture".to_string()),
                        source_path: Some(format!("notes/rehydrate-{index}.md")),
                        source_quality: None,
                        recorded_at: None,
                    })
                    .collect(),
                traces: Vec::new(),
                semantic_consolidation: None,
            },
            inbox: memd_schema::MemoryInboxResponse {
                route: RetrievalRoute::ProjectFirst,
                intent: RetrievalIntent::CurrentTask,
                items: (0..5)
                    .map(|index| memd_schema::InboxMemoryItem {
                        item: memd_schema::MemoryItem {
                            id: uuid::Uuid::new_v4(),
                            content: format!(
                                "inbox pressure {}: keep the next action visible",
                                index
                            ),
                            redundancy_key: Some("same".to_string()),
                            belief_branch: None,
                            preferred: true,
                            kind: memd_schema::MemoryKind::Status,
                            scope: memd_schema::MemoryScope::Project,
                            project: Some("memd".to_string()),
                            namespace: Some("main".to_string()),
                            workspace: Some("shared".to_string()),
                            visibility: memd_schema::MemoryVisibility::Workspace,
                            source_agent: Some("codex@test".to_string()),
                            source_system: Some("checkpoint".to_string()),
                            source_path: Some(format!("notes/inbox-{index}.md")),
                            source_quality: None,
                            confidence: 0.8,
                            ttl_seconds: Some(86_400),
                            created_at: Utc::now(),
                            status: memd_schema::MemoryStatus::Active,
                            stage: memd_schema::MemoryStage::Candidate,
                            last_verified_at: None,
                            supersedes: Vec::new(),
                            updated_at: Utc::now(),
                            tags: vec!["checkpoint".to_string()],
                        },
                        reasons: vec!["stale".to_string()],
                    })
                    .collect(),
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: vec![memd_schema::WorkspaceMemoryRecord {
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("shared".to_string()),
                    visibility: memd_schema::MemoryVisibility::Workspace,
                    item_count: 9,
                    active_count: 7,
                    candidate_count: 2,
                    contested_count: 0,
                    source_lane_count: 2,
                    avg_confidence: 0.87,
                    trust_score: 0.93,
                    last_seen_at: None,
                    tags: Vec::new(),
                }],
            },
            sources: memd_schema::SourceMemoryResponse {
                sources: Vec::new(),
            },
            semantic: Some(RagRetrieveResponse {
                status: "ok".to_string(),
                mode: RagRetrieveMode::Auto,
                items: (0..4)
                    .map(|index| memd_rag::RagRetrieveItem {
                        content: format!("semantic evidence {index}"),
                        score: 0.7 - (index as f32 * 0.05),
                        source: Some(format!("semantic-{index}")),
                    })
                    .collect(),
            }),
            claims: SessionClaimsState::default(),
            recent_repo_changes: vec![
                "repo change: packet compiler touched".to_string(),
                "repo change: packet compiler touched".to_string(),
                "repo change: packet compiler touched".to_string(),
            ],
            change_summary: vec![
                "changed=preserve next action".to_string(),
                "next=refresh from bundle".to_string(),
            ],
            resume_state_age_minutes: Some(25),
            refresh_recommended: true,
        }
    }

    #[test]
    fn wakeup_markdown_surfaces_durable_truth_without_verbose_mode() {
        let dir = std::env::temp_dir().join(format!("memd-wakeup-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");

        let markdown = render_bundle_wakeup_markdown(&dir, &sample_snapshot(), false);
        assert!(markdown.contains("## Durable Truth"));
        assert!(markdown.contains("## Continuity"));
        assert!(markdown.contains("- doing="));
        assert!(markdown.contains("- left_off="));
        assert!(markdown.contains("memd must preserve important user corrections"));
        assert!(markdown.contains("Durable truth beats transcript recall."));
        assert!(markdown.contains("Reply in `caveman-ultra`"));
        assert!(markdown.contains("If your draft is not in `caveman-ultra`"));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn wakeup_summary_surfaces_packet_efficiency_signal() {
        let snapshot = sample_snapshot();
        let summary = render_bundle_wakeup_summary(&snapshot);

        assert!(summary.contains("wake project=memd"));
        assert!(summary.contains("tokens="));
        assert!(summary.contains("core="));
        assert!(summary.contains("focus=\"follow durable truth before transcript recall\""));
        assert!(snapshot.core_prompt_tokens() <= snapshot.estimated_prompt_tokens());
    }

    #[test]
    fn wakeup_markdown_stays_compact_under_pressure_and_keeps_continuity() {
        let dir =
            std::env::temp_dir().join(format!("memd-wakeup-pressure-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");

        let snapshot = pressure_snapshot();
        let markdown = render_bundle_wakeup_markdown(&dir, &snapshot, false);

        assert!(markdown.contains("## Durable Truth"));
        assert!(markdown.contains("## Focus"));
        assert!(markdown.contains("## Continuity"));
        assert!(markdown.contains("- doing="));
        assert!(markdown.contains("- left_off="));
        assert!(markdown.contains("- changed="));
        assert!(markdown.contains("- next="));
        assert!(!markdown.contains("artifact-3"));
        assert!(markdown.lines().count() < 40);

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }
}
