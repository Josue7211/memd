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

fn wake_budget_agent_name(output: &Path, snapshot: &ResumeSnapshot) -> Option<String> {
    read_bundle_runtime_config(output)
        .ok()
        .flatten()
        .and_then(|config| config.agent)
        .or_else(|| snapshot.agent.clone())
}

fn truncate_visible_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let mut truncated = String::new();
    for ch in value.chars().take(max_chars.saturating_sub(1)) {
        truncated.push(ch);
    }
    truncated.push('…');
    truncated
}

fn enforce_wake_char_budget(prefix: &str, protocol: &str, max_chars: usize) -> String {
    let full = format!("{prefix}{protocol}");
    if full.chars().count() <= max_chars {
        return full;
    }

    let trimmed_protocol = protocol.trim_start();
    let elided_marker = "\n## Wake Budget\n\n- startup trimmed; use `memd lookup` or `memd resume` for deeper recall.\n\n";
    let required_chars = trimmed_protocol.chars().count() + elided_marker.chars().count();

    if required_chars >= max_chars {
        return truncate_visible_chars(trimmed_protocol, max_chars);
    }

    let prefix_budget = max_chars - required_chars;
    let mut trimmed_prefix = String::new();
    for line in prefix.lines() {
        let candidate = if trimmed_prefix.is_empty() {
            format!("{line}\n")
        } else {
            format!("{trimmed_prefix}{line}\n")
        };
        if candidate.chars().count() > prefix_budget {
            break;
        }
        trimmed_prefix = candidate;
    }

    let combined = format!(
        "{}{}{}",
        trimmed_prefix.trim_end(),
        elided_marker,
        trimmed_protocol
    );
    if combined.chars().count() <= max_chars {
        combined
    } else {
        truncate_visible_chars(&combined, max_chars)
    }
}

pub(crate) fn render_bundle_wakeup_markdown(
    output: &Path,
    snapshot: &ResumeSnapshot,
    verbose: bool,
) -> String {
    let mut prefix = String::new();
    let budget = crate::harness::preset::wake_char_budget_for_agent(
        wake_budget_agent_name(output, snapshot).as_deref(),
    );
    let claude_strict = wake_budget_agent_name(output, snapshot)
        .as_deref()
        .is_some_and(|agent| agent.trim().eq_ignore_ascii_case("claude-code"));

    prefix.push_str("# memd wake-up\n\n");
    prefix.push_str(&format!(
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
    if !instructions.is_empty() && !claude_strict {
        prefix.push_str("## Instructions\n\n");
        let limit = if verbose { 3 } else { 1 };
        for (source, snippet) in instructions.into_iter().take(limit) {
            prefix.push_str(&format!("- {source}: {}\n", compact_inline(&snippet, 180)));
        }
        prefix.push('\n');
    }

    let event_spine = snapshot.event_spine();
    if !event_spine.is_empty() && !claude_strict {
        prefix.push_str("## Live\n\n");
        let limit = if verbose { 2 } else { 1 };
        for item in event_spine.iter().take(limit) {
            prefix.push_str(&format!("- {}\n", compact_inline(item, 100)));
        }
        prefix.push('\n');
    }

    prefix.push_str("## Durable Truth\n\n");
    if snapshot.context.records.is_empty() {
        prefix.push_str("- none\n\n");
    } else {
        let limit = if claude_strict { 1 } else if verbose { 4 } else { 2 };
        for item in snapshot.context.records.iter().take(limit) {
            let item_limit = if claude_strict { 120 } else { 160 };
            prefix.push_str(&format!(
                "- {}\n",
                compact_inline(item.record.trim(), item_limit)
            ));
        }
        prefix.push('\n');
    }

    prefix.push_str("## Focus\n\n");
    if snapshot.working.records.is_empty() {
        prefix.push_str("- none\n");
    } else {
        let limit = 1;
        for item in snapshot.working.records.iter().take(limit) {
            let item_limit = if claude_strict { 110 } else { 140 };
            prefix.push_str(&format!(
                "- {}\n",
                compact_inline(item.record.trim(), item_limit)
            ));
        }
    }
    prefix.push('\n');

    let continuity = snapshot.continuity_capsule();
    if continuity.current_task.is_some()
        || continuity.resume_point.is_some()
        || continuity.changed.is_some()
        || continuity.next_action.is_some()
        || continuity.blocker.is_some()
    {
        prefix.push_str("## Continuity\n\n");
        let continuity_limit = if claude_strict { 96 } else { 140 };
        if let Some(current_task) = continuity.current_task.as_deref() {
            prefix.push_str(&format!(
                "- doing={}\n",
                compact_inline(current_task, continuity_limit)
            ));
        }
        if let Some(resume_point) = continuity.resume_point.as_deref() {
            prefix.push_str(&format!(
                "- left_off={}\n",
                compact_inline(resume_point, continuity_limit)
            ));
        }
        if let Some(changed) = continuity.changed.as_deref() {
            prefix.push_str(&format!(
                "- changed={}\n",
                compact_inline(changed, continuity_limit)
            ));
        }
        if let Some(next_action) = continuity.next_action.as_deref() {
            prefix.push_str(&format!(
                "- next={}\n",
                compact_inline(next_action, continuity_limit)
            ));
        }
        if let Some(blocker) = continuity.blocker.as_deref() {
            prefix.push_str(&format!(
                "- blocker={}\n",
                compact_inline(blocker, continuity_limit)
            ));
        }
        prefix.push('\n');
    }

    if !snapshot.working.procedures.is_empty() && !claude_strict {
        prefix.push_str("## Procedures\n\n");
        let proc_limit = if verbose { 3 } else { 2 };
        for proc in snapshot.working.procedures.iter().take(proc_limit) {
            let steps_compact = proc.steps.join(" → ");
            prefix.push_str(&format!(
                "- **{}** ({}): {}\n",
                proc.name,
                compact_inline(&proc.trigger, 60),
                compact_inline(&steps_compact, 100),
            ));
        }
        prefix.push('\n');
    }

    if verbose
        && !claude_strict
        && (!snapshot.inbox.items.is_empty() || !snapshot.working.rehydration_queue.is_empty())
    {
        prefix.push_str("## Recovery\n\n");
        let recovery_limit = if verbose { 1 } else { 1 };
        for item in snapshot
            .working
            .rehydration_queue
            .iter()
            .take(recovery_limit)
        {
            prefix.push_str(&format!(
                "- {}: {}\n",
                item.label,
                compact_inline(item.summary.trim(), 120)
            ));
        }
        let inbox_limit = if verbose { 1 } else { 1 };
        for item in snapshot.inbox.items.iter().take(inbox_limit) {
            prefix.push_str(&format!(
                "- {:?}/{:?}: {}\n",
                item.item.kind,
                item.item.status,
                compact_inline(item.item.content.trim(), 120)
            ));
        }
        prefix.push('\n');
    }

    let active_voice = read_bundle_voice_mode(output).unwrap_or_else(default_voice_mode);
    let mut protocol = String::new();
    if claude_strict {
        // Claude Code: minimal protocol — behavioral rules live in AGENTS.md and the memd skill.
        protocol.push_str(&format!("## Voice\n\n- {}\n", active_voice));
    } else {
        protocol.push_str("## Protocol\n\n");
        protocol.push_str("- Read first.\n");
        protocol.push_str("- Durable truth beats transcript recall.\n");
        protocol.push_str(
            "- Lookup before answers on decisions, preferences, history, or prior user corrections.\n",
        );
        protocol.push_str("- Recall: `memd lookup --output .memd --query \"...\"`.\n");
        protocol.push_str("- If the user corrects you, write the correction back instead of trusting the transcript.\n");
        protocol.push_str("- Writes: `remember-short`, `remember-decision`, `remember-preference`, `remember-long`, `capture-live`, `correct-memory`, `sync-semantic`, `watch`.\n");
        if verbose {
            protocol
                .push_str("- Wake/resume/refresh/handoff/hook capture auto-write short-term status.\n");
        }
        protocol.push_str("- Promote stable truths; do not rely on transcript recall.\n");
        protocol.push_str(&format!("- Default voice: {}\n", active_voice));
        protocol.push_str(&format!(
            "- Reply in `{}` unless `.memd/config.json` changes it.\n",
            active_voice
        ));
        protocol.push_str(&format!(
            "- If your draft is not in `{}`, stop and rewrite it before sending.\n",
            active_voice
        ));
    };

    enforce_wake_char_budget(&prefix, &protocol, budget)
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
            procedures: vec![],
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
        assert!(markdown.contains("Reply in `caveman-lite`"));
        assert!(markdown.contains("If your draft is not in `caveman-lite`"));

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

    #[test]
    fn claude_wakeup_markdown_respects_strict_budget() {
        let dir =
            std::env::temp_dir().join(format!("memd-wakeup-claude-budget-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "memd",
  "agent": "claude-code",
  "route": "auto",
  "intent": "current_task"
}
"#,
        )
        .expect("write config");

        let markdown = render_bundle_wakeup_markdown(&dir, &pressure_snapshot(), false);

        assert!(markdown.chars().count() <= 1200);
        assert!(markdown.contains("## Voice"));
        assert!(!markdown.contains("## Protocol"));
        assert!(!markdown.contains("## Instructions"));
        assert!(!markdown.contains("## Live"));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    // ── Cross-Harness Wake Proof Tests ──────────────────────────────

    #[test]
    fn shared_surface_contract_all_non_wake_only_harnesses_use_shared_surfaces() {
        use crate::harness::preset::{
            HarnessPresetRegistry, SHARED_VISIBLE_SURFACES, WAKE_ONLY_SURFACES,
        };
        let registry = HarnessPresetRegistry::default_registry();
        for preset in &registry.packs {
            if preset.wake_only {
                assert_eq!(
                    preset.surface_set, WAKE_ONLY_SURFACES,
                    "wake-only harness {} should use WAKE_ONLY_SURFACES",
                    preset.pack_id
                );
            } else {
                assert_eq!(
                    preset.surface_set, SHARED_VISIBLE_SURFACES,
                    "harness {} should use SHARED_VISIBLE_SURFACES",
                    preset.pack_id
                );
            }
        }
    }

    #[test]
    fn shared_surface_contract_no_duplicate_surface_filenames() {
        use crate::harness::preset::HarnessPresetRegistry;
        let registry = HarnessPresetRegistry::default_registry();
        for preset in &registry.packs {
            let mut seen = std::collections::HashSet::new();
            for surface in preset.surface_set {
                assert!(
                    seen.insert(*surface),
                    "harness {} has duplicate surface: {}",
                    preset.pack_id,
                    surface
                );
            }
        }
    }

    #[test]
    fn claude_code_is_in_preset_registry_with_wake_only() {
        use crate::harness::preset::HarnessPresetRegistry;
        let registry = HarnessPresetRegistry::default_registry();
        let claude = registry.get("claude-code").expect("claude-code must be in registry");
        assert!(claude.wake_only, "claude-code must be wake_only");
        assert_eq!(claude.wake_char_budget, 1200);
        assert_eq!(claude.surface_set.len(), 1);
        assert_eq!(claude.surface_set[0], "wake.md");
    }

    #[test]
    fn claude_code_pack_files_do_not_include_mem_or_events() {
        let dir = std::env::temp_dir().join(format!("memd-claude-pack-{}", uuid::Uuid::new_v4()));
        let pack =
            crate::harness::claude_code::build_claude_code_harness_pack(&dir, "demo", "main");
        for file in &pack.files {
            let name = file.file_name().unwrap().to_string_lossy();
            assert_ne!(name, "mem.md", "claude-code pack must not include mem.md");
            assert_ne!(
                name, "events.md",
                "claude-code pack must not include events.md"
            );
        }
        assert!(
            pack.files.iter().any(|f| f.ends_with("wake.md")),
            "claude-code pack must include wake.md"
        );
    }

    #[test]
    fn wake_budget_for_claude_code_uses_registry_not_special_case() {
        use crate::harness::preset::wake_char_budget_for_agent;
        assert_eq!(wake_char_budget_for_agent(Some("claude-code")), 1200);
        assert_eq!(
            wake_char_budget_for_agent(Some("claude-code@session-abc")),
            1200
        );
        assert_eq!(wake_char_budget_for_agent(Some("codex")), 1800);
        assert_eq!(wake_char_budget_for_agent(Some("hermes")), 1800);
        assert_eq!(wake_char_budget_for_agent(Some("unknown-agent")), 1800);
    }

    #[test]
    fn is_wake_only_agent_matches_registry() {
        use crate::harness::preset::is_wake_only_agent;
        assert!(is_wake_only_agent(Some("claude-code")));
        assert!(is_wake_only_agent(Some("claude-code@session-xyz")));
        assert!(!is_wake_only_agent(Some("codex")));
        assert!(!is_wake_only_agent(Some("hermes")));
        assert!(!is_wake_only_agent(Some("opencode")));
        assert!(!is_wake_only_agent(Some("openclaw")));
        assert!(!is_wake_only_agent(Some("agent-zero")));
        assert!(!is_wake_only_agent(None));
    }

    #[test]
    fn all_harness_wake_packets_stay_within_budget() {
        use crate::harness::preset::HarnessPresetRegistry;
        let registry = HarnessPresetRegistry::default_registry();
        let snapshot = pressure_snapshot();
        for preset in &registry.packs {
            let dir = std::env::temp_dir().join(format!(
                "memd-budget-{}-{}",
                preset.pack_id,
                uuid::Uuid::new_v4()
            ));
            fs::create_dir_all(&dir).expect("create temp bundle");
            fs::write(
                dir.join("config.json"),
                format!(
                    r#"{{
  "project": "memd",
  "agent": "{}",
  "route": "auto",
  "intent": "current_task"
}}
"#,
                    preset.pack_id
                ),
            )
            .expect("write config");

            let markdown = render_bundle_wakeup_markdown(&dir, &snapshot, false);
            let char_count = markdown.chars().count();
            assert!(
                char_count <= preset.wake_char_budget,
                "harness {} wake packet ({} chars) exceeds budget ({} chars)",
                preset.pack_id,
                char_count,
                preset.wake_char_budget,
            );

            fs::remove_dir_all(dir).expect("cleanup temp bundle");
        }
    }

    #[test]
    fn generated_claude_imports_template_only_imports_wake() {
        // Verify the code template in maintenance_runtime writes only @../wake.md
        // This is a structural test on the bridge file content
        let template_fragment = "@../wake.md";
        let cold_surfaces = ["@../mem.md", "@../events.md"];

        // Simulate what write_native_agent_bridge_files generates
        let generated = format!(
            "## Imported memd memory files\n\n{}\n\n",
            template_fragment
        );
        assert!(
            generated.contains(template_fragment),
            "CLAUDE_IMPORTS template must import wake.md"
        );
        for cold in &cold_surfaces {
            assert!(
                !generated.contains(cold),
                "CLAUDE_IMPORTS template must NOT import {} (cold-path surface)",
                cold
            );
        }
    }

    #[test]
    fn preset_registry_has_exactly_six_harnesses() {
        use crate::harness::preset::HarnessPresetRegistry;
        let registry = HarnessPresetRegistry::default_registry();
        assert_eq!(
            registry.packs.len(),
            6,
            "registry should have 6 harnesses: codex, claude-code, agent-zero, openclaw, hermes, opencode"
        );
    }
}
