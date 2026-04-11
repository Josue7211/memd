use super::*;

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

pub(crate) fn render_visible_memory_home(
    response: &VisibleMemorySnapshotResponse,
    follow: bool,
) -> String {
    let focus = &response.home.focus_artifact;
    let mut output = format!(
        "memory_home focus={} status={} freshness={} visibility={} workspace={} inbox={} repair={} awareness={} nodes={} edges={}",
        compact_inline(&focus.title, 48),
        visible_memory_status_label(focus.status),
        compact_inline(&focus.freshness, 24),
        focus.visibility.map(format_visibility).unwrap_or("none"),
        focus.workspace.as_deref().unwrap_or("none"),
        response.home.inbox_count,
        response.home.repair_count,
        response.home.awareness_count,
        response.knowledge_map.nodes.len(),
        response.knowledge_map.edges.len()
    );

    if follow {
        output.push_str(&format!(
            " source_system={} source_path={} producer={} trust={} actions={}",
            focus.provenance.source_system.as_deref().unwrap_or("none"),
            focus.provenance.source_path.as_deref().unwrap_or("none"),
            focus.provenance.producer.as_deref().unwrap_or("none"),
            compact_inline(&focus.provenance.trust_reason, 64),
            if focus.actions.is_empty() {
                "none".to_string()
            } else {
                focus.actions.join("|")
            }
        ));

        let trail = response
            .knowledge_map
            .nodes
            .iter()
            .take(3)
            .map(|node| {
                format!(
                    "{}:{}:{}",
                    compact_inline(&node.title, 24),
                    node.artifact_kind,
                    visible_memory_status_label(node.status)
                )
            })
            .collect::<Vec<_>>();
        if !trail.is_empty() {
            output.push_str(&format!(" trail={}", trail.join(" | ")));
        }
    }

    output
}

pub(crate) fn render_visible_memory_artifact_detail(
    response: &VisibleMemoryArtifactDetailResponse,
    follow: bool,
) -> String {
    let artifact = &response.artifact;
    let mut output = format!(
        "memory_artifact id={} title={} status={} kind={} visibility={} workspace={} explain={} timeline={} sources={} workspaces={} sessions={} tasks={} claims={} related={} map_nodes={} map_edges={} actions={}",
        short_uuid(artifact.id),
        compact_inline(&artifact.title, 48),
        visible_memory_status_label(artifact.status),
        compact_inline(&artifact.artifact_kind, 24),
        artifact.visibility.map(format_visibility).unwrap_or("none"),
        artifact.workspace.as_deref().unwrap_or("none"),
        bool_label(response.explain.is_some()),
        bool_label(response.timeline.is_some()),
        response.sources.sources.len(),
        response.workspaces.workspaces.len(),
        response.sessions.sessions.len(),
        response.tasks.tasks.len(),
        response.claims.claims.len(),
        response.related_artifacts.len(),
        response.related_map.nodes.len(),
        response.related_map.edges.len(),
        visible_memory_actions_label(&response.actions)
    );

    if follow {
        output.push_str(&format!(
            " source_system={} source_path={} producer={} trust={} repair={} confidence={:.2}",
            artifact
                .provenance
                .source_system
                .as_deref()
                .unwrap_or("none"),
            artifact.provenance.source_path.as_deref().unwrap_or("none"),
            artifact.provenance.producer.as_deref().unwrap_or("none"),
            compact_inline(&artifact.provenance.trust_reason, 64),
            compact_inline(&artifact.repair_state, 24),
            artifact.confidence
        ));

        if let Some(explain) = response.explain.as_ref() {
            output.push_str(&format!(
                " explain_route={} explain_intent={} explain_sources={} explain_events={} explain_rehydrate={}",
                route_label(explain.route),
                intent_label(explain.intent),
                explain.sources.len(),
                explain.events.len(),
                explain.rehydration.len()
            ));
            if let Some(first_reason) = explain.reasons.first() {
                output.push_str(&format!(
                    " explain_reason={}",
                    compact_inline(first_reason, 48)
                ));
            }
        }

        if let Some(timeline) = response.timeline.as_ref() {
            output.push_str(&format!(
                " timeline_events={} timeline_entity={}",
                timeline.events.len(),
                timeline
                    .entity
                    .as_ref()
                    .map(|entity| format!("{}:{}", short_uuid(entity.id), entity.entity_type))
                    .unwrap_or_else(|| "none".to_string())
            ));
            let trail = timeline
                .events
                .iter()
                .take(3)
                .map(|event| {
                    format!(
                        "{}:{}",
                        event.event_type,
                        compact_inline(&event.summary, 36)
                    )
                })
                .collect::<Vec<_>>();
            if !trail.is_empty() {
                output.push_str(&format!(" timeline_trail={}", trail.join(" | ")));
            }
        }

        let related_trail = response
            .related_artifacts
            .iter()
            .take(3)
            .map(|related| {
                format!(
                    "{}:{}:{}",
                    short_uuid(related.id),
                    compact_inline(&related.title, 24),
                    visible_memory_status_label(related.status)
                )
            })
            .collect::<Vec<_>>();
        if !related_trail.is_empty() {
            output.push_str(&format!(" related_trail={}", related_trail.join(" | ")));
        }

        let map_trail = response
            .related_map
            .nodes
            .iter()
            .take(3)
            .map(|node| {
                format!(
                    "{}:{}:{}",
                    compact_inline(&node.title, 24),
                    compact_inline(&node.artifact_kind, 18),
                    visible_memory_status_label(node.status)
                )
            })
            .collect::<Vec<_>>();
        if !map_trail.is_empty() {
            output.push_str(&format!(" map_trail={}", map_trail.join(" | ")));
        }
    }

    output
}

pub(crate) fn render_visible_memory_knowledge_map(
    response: &VisibleMemorySnapshotResponse,
    follow: bool,
) -> String {
    let map = &response.knowledge_map;
    let mut output = format!(
        "memory_map focus={} nodes={} edges={} current={} candidate={} stale={} superseded={} conflicted={} archived={} timeline_nodes={} workspace_nodes={}",
        compact_inline(&response.home.focus_artifact.title, 48),
        map.nodes.len(),
        map.edges.len(),
        knowledge_map_status_count(map, VisibleMemoryStatus::Current),
        knowledge_map_status_count(map, VisibleMemoryStatus::Candidate),
        knowledge_map_status_count(map, VisibleMemoryStatus::Stale),
        knowledge_map_status_count(map, VisibleMemoryStatus::Superseded),
        knowledge_map_status_count(map, VisibleMemoryStatus::Conflicted),
        knowledge_map_status_count(map, VisibleMemoryStatus::Archived),
        knowledge_map_kind_count(map, "timeline_event"),
        knowledge_map_kind_count(map, "workspace_lane")
    );

    if follow {
        let node_trail = map
            .nodes
            .iter()
            .take(4)
            .map(|node| {
                format!(
                    "{}:{}:{}",
                    compact_inline(&node.title, 24),
                    compact_inline(&node.artifact_kind, 18),
                    visible_memory_status_label(node.status)
                )
            })
            .collect::<Vec<_>>();
        if !node_trail.is_empty() {
            output.push_str(&format!(" trail={}", node_trail.join(" | ")));
        }

        let edge_trail = map
            .edges
            .iter()
            .take(3)
            .map(|edge| {
                format!(
                    "{}:{}->{}",
                    compact_inline(&edge.relation, 18),
                    short_uuid(edge.from),
                    short_uuid(edge.to)
                )
            })
            .collect::<Vec<_>>();
        if !edge_trail.is_empty() {
            output.push_str(&format!(" edge_trail={}", edge_trail.join(" | ")));
        }

        output.push_str(&format!(
            " focus_workspace={} focus_visibility={} focus_confidence={:.2}",
            response
                .home
                .focus_artifact
                .workspace
                .as_deref()
                .unwrap_or("none"),
            response
                .home
                .focus_artifact
                .visibility
                .map(format_visibility)
                .unwrap_or("none"),
            response.home.focus_artifact.confidence
        ));
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
                    .map(|event| format!(
                        "{}:{}",
                        event.event_type,
                        compact_inline(&event.summary, 36)
                    ))
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
                        if sibling.preferred {
                            "preferred"
                        } else {
                            "candidate"
                        }
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

pub(super) fn compact_inline(value: &str, max_chars: usize) -> String {
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

pub(super) fn route_label(route: RetrievalRoute) -> &'static str {
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

pub(super) fn intent_label(intent: RetrievalIntent) -> &'static str {
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

pub(super) fn format_visibility(value: memd_schema::MemoryVisibility) -> &'static str {
    match value {
        memd_schema::MemoryVisibility::Private => "private",
        memd_schema::MemoryVisibility::Workspace => "workspace",
        memd_schema::MemoryVisibility::Public => "public",
    }
}

pub(super) fn visible_memory_status_label(status: VisibleMemoryStatus) -> &'static str {
    match status {
        VisibleMemoryStatus::Current => "current",
        VisibleMemoryStatus::Candidate => "candidate",
        VisibleMemoryStatus::Stale => "stale",
        VisibleMemoryStatus::Superseded => "superseded",
        VisibleMemoryStatus::Conflicted => "conflicted",
        VisibleMemoryStatus::Archived => "archived",
    }
}

pub(super) fn visible_memory_action_label(action: VisibleMemoryUiActionKind) -> &'static str {
    match action {
        VisibleMemoryUiActionKind::Inspect => "inspect",
        VisibleMemoryUiActionKind::Explain => "explain",
        VisibleMemoryUiActionKind::VerifyCurrent => "verify_current",
        VisibleMemoryUiActionKind::MarkStale => "mark_stale",
        VisibleMemoryUiActionKind::Promote => "promote",
        VisibleMemoryUiActionKind::OpenSource => "open_source",
        VisibleMemoryUiActionKind::OpenInObsidian => "open_in_obsidian",
    }
}

pub(super) fn visible_memory_actions_label(actions: &[VisibleMemoryUiActionKind]) -> String {
    if actions.is_empty() {
        "none".to_string()
    } else {
        actions
            .iter()
            .map(|action| visible_memory_action_label(*action))
            .collect::<Vec<_>>()
            .join("|")
    }
}

pub(super) fn knowledge_map_status_count(
    map: &VisibleMemoryKnowledgeMap,
    status: VisibleMemoryStatus,
) -> usize {
    map.nodes
        .iter()
        .filter(|node| node.status == status)
        .count()
}

pub(super) fn knowledge_map_kind_count(map: &VisibleMemoryKnowledgeMap, kind: &str) -> usize {
    map.nodes
        .iter()
        .filter(|node| node.artifact_kind == kind)
        .count()
}

pub(super) fn bool_label(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

pub(crate) fn is_default_runtime(runtime: &memd_schema::MemoryPolicyRuntime) -> bool {
    !runtime.live_truth.read_once_sources
        && !runtime.live_truth.raw_reopen_requires_change_or_doubt
        && !runtime.live_truth.visible_memory_objects
        && !runtime.live_truth.compile_from_events
        && !runtime.memory_compilation.event_driven_updates
        && !runtime.memory_compilation.patch_not_rewrite
        && !runtime.memory_compilation.preserve_provenance
        && !runtime.memory_compilation.source_on_demand
        && !runtime.semantic_fallback.enabled
        && !runtime.semantic_fallback.source_of_truth
        && runtime.semantic_fallback.max_items_per_query == 0
        && !runtime.semantic_fallback.rerank_with_visible_memory
        && !runtime.skill_gating.propose_from_repeated_patterns
        && !runtime.skill_gating.sandboxed_evaluation
        && !runtime.skill_gating.auto_activate_low_risk_only
        && !runtime.skill_gating.gated_activation
        && !runtime.skill_gating.require_evaluation
        && !runtime.skill_gating.require_policy_approval
}

#[cfg(test)]
mod tests {
    use super::{
        render_bundle_status_summary, render_harness_preset_markdown, render_policy_summary,
        render_skill_policy_summary, render_visible_memory_artifact_detail,
        render_visible_memory_home, render_visible_memory_knowledge_map,
    };
    use crate::harness::preset::HarnessPresetRegistry;
    use memd_schema::{
        HiveClaimsResponse, HiveSessionsResponse, HiveTasksResponse, MemoryKind,
        MemoryPolicyConsolidation, MemoryPolicyDecay, MemoryPolicyFeedback, MemoryPolicyLiveTruth,
        MemoryPolicyMemoryCompilation, MemoryPolicyPromotion, MemoryPolicyResponse,
        MemoryPolicyRouteDefault, MemoryPolicyRuntime, MemoryPolicySemanticFallback,
        MemoryPolicySkillGating, MemoryPolicyWorkingMemory, MemoryScope, MemoryVisibility,
        RetrievalIntent, RetrievalRoute, SourceMemoryResponse, VisibleMemoryArtifact,
        VisibleMemoryArtifactDetailResponse, VisibleMemoryGraphEdge, VisibleMemoryGraphNode,
        VisibleMemoryHome, VisibleMemoryKnowledgeMap, VisibleMemoryProvenance,
        VisibleMemorySnapshotResponse, VisibleMemoryStatus, VisibleMemoryUiActionKind,
        WorkspaceMemoryResponse,
    };
    use serde_json::json;

    #[test]
    fn policy_summary_includes_skill_gates() {
        let response = MemoryPolicyResponse {
            retrieval_order: vec![MemoryScope::Local, MemoryScope::Project],
            route_defaults: vec![MemoryPolicyRouteDefault {
                intent: RetrievalIntent::CurrentTask,
                route: RetrievalRoute::LocalFirst,
            }],
            working_memory: MemoryPolicyWorkingMemory {
                budget_chars: 1600,
                max_chars_per_item: 220,
                default_limit: 8,
                rehydration_limit: 3,
            },
            retrieval_feedback: MemoryPolicyFeedback {
                enabled: true,
                tracked_surfaces: vec!["search".to_string(), "working".to_string()],
                max_items_per_request: 3,
            },
            source_trust_floor: 0.6,
            runtime: MemoryPolicyRuntime {
                live_truth: MemoryPolicyLiveTruth {
                    read_once_sources: true,
                    raw_reopen_requires_change_or_doubt: true,
                    visible_memory_objects: true,
                    compile_from_events: true,
                },
                memory_compilation: MemoryPolicyMemoryCompilation {
                    event_driven_updates: true,
                    patch_not_rewrite: true,
                    preserve_provenance: true,
                    source_on_demand: true,
                },
                semantic_fallback: MemoryPolicySemanticFallback {
                    enabled: true,
                    source_of_truth: false,
                    max_items_per_query: 3,
                    rerank_with_visible_memory: true,
                },
                skill_gating: MemoryPolicySkillGating {
                    propose_from_repeated_patterns: true,
                    sandboxed_evaluation: true,
                    auto_activate_low_risk_only: true,
                    gated_activation: true,
                    require_evaluation: true,
                    require_policy_approval: true,
                },
            },
            promotion: MemoryPolicyPromotion {
                min_salience: 0.22,
                min_events: 3,
                lookback_days: 14,
                default_ttl_days: 90,
            },
            decay: MemoryPolicyDecay {
                max_items: 128,
                inactive_days: 21,
                max_decay: 0.12,
                record_events: true,
            },
            consolidation: MemoryPolicyConsolidation {
                max_groups: 24,
                min_events: 3,
                lookback_days: 14,
                min_salience: 0.22,
                record_events: true,
            },
        };

        let summary = render_policy_summary(&response, true);
        assert!(summary.contains("skill_gating=on"));
        assert!(summary.contains("sandbox_eval=on"));
        assert!(summary.contains("low_risk_auto=on"));
        assert!(summary.contains("approval=on"));
        assert!(summary.contains("read_once=on"));
    }

    #[test]
    fn skill_policy_summary_includes_lifecycle_flow() {
        let response = MemoryPolicyResponse {
            retrieval_order: vec![MemoryScope::Local, MemoryScope::Project],
            route_defaults: vec![MemoryPolicyRouteDefault {
                intent: RetrievalIntent::CurrentTask,
                route: RetrievalRoute::LocalFirst,
            }],
            working_memory: MemoryPolicyWorkingMemory {
                budget_chars: 1600,
                max_chars_per_item: 220,
                default_limit: 8,
                rehydration_limit: 3,
            },
            retrieval_feedback: MemoryPolicyFeedback {
                enabled: true,
                tracked_surfaces: vec!["search".to_string(), "working".to_string()],
                max_items_per_request: 3,
            },
            source_trust_floor: 0.6,
            runtime: MemoryPolicyRuntime {
                live_truth: MemoryPolicyLiveTruth {
                    read_once_sources: true,
                    raw_reopen_requires_change_or_doubt: true,
                    visible_memory_objects: true,
                    compile_from_events: true,
                },
                memory_compilation: MemoryPolicyMemoryCompilation {
                    event_driven_updates: true,
                    patch_not_rewrite: true,
                    preserve_provenance: true,
                    source_on_demand: true,
                },
                semantic_fallback: MemoryPolicySemanticFallback {
                    enabled: true,
                    source_of_truth: false,
                    max_items_per_query: 3,
                    rerank_with_visible_memory: true,
                },
                skill_gating: MemoryPolicySkillGating {
                    propose_from_repeated_patterns: true,
                    sandboxed_evaluation: true,
                    auto_activate_low_risk_only: true,
                    gated_activation: true,
                    require_evaluation: true,
                    require_policy_approval: true,
                },
            },
            promotion: MemoryPolicyPromotion {
                min_salience: 0.22,
                min_events: 3,
                lookback_days: 14,
                default_ttl_days: 90,
            },
            decay: MemoryPolicyDecay {
                max_items: 128,
                inactive_days: 21,
                max_decay: 0.12,
                record_events: true,
            },
            consolidation: MemoryPolicyConsolidation {
                max_groups: 24,
                min_events: 3,
                lookback_days: 14,
                min_salience: 0.22,
                record_events: true,
            },
        };

        let summary = render_skill_policy_summary(&response, true);
        assert!(summary.contains("skill-policy"));
        assert!(summary.contains("propose=on"));
        assert!(summary.contains("sandbox=on"));
        assert!(summary.contains("activate=on"));
        assert!(summary.contains("flow=pattern->proposal->sandbox->eval->policy->activate"));
    }

    #[test]
    fn harness_preset_markdown_includes_registry_metadata() {
        let registry = HarnessPresetRegistry::default_registry();
        let preset = registry.get("codex").expect("codex preset");

        let markdown = render_harness_preset_markdown(preset);
        assert!(markdown.contains("# Codex Harness Pack"));
        assert!(markdown.contains("- pack id: `codex`"));
        assert!(markdown.contains("## Surface Set"));
        assert!(markdown.contains("## Default Verbs"));
        assert!(markdown.contains("## Shared Core"));
    }

    #[test]
    fn status_summary_surfaces_prompt_pressure_warning() {
        let status = json!({
            "bundle": "/tmp/memd",
            "setup_ready": true,
            "server": { "status": "ok" },
            "rag": { "healthy": true },
            "missing": [],
            "defaults": {
                "project": "demo",
                "namespace": "main",
                "session": "codex-a",
                "tab_id": "tab-a",
                "agent": "codex",
                "voice_mode": "normal"
            },
            "resume_preview": {
                "project": "demo",
                "namespace": "main",
                "session": "codex-a",
                "tab_id": "tab-a",
                "agent": "codex",
                "context_pressure": "high",
                "estimated_prompt_tokens": 1842,
                "refresh_recommended": true,
                "inbox_items": 4,
                "rehydration_queue": 3,
                "redundant_context_items": 2,
                "semantic_hits": 4,
                "focus": "Trim the live bundle before loading more context",
                "next_recovery": "reopen only the changed files"
            }
        });

        let summary = render_bundle_status_summary(&status);
        assert!(summary.contains("status bundle=/tmp/memd"));
        assert!(summary.contains("project=demo"));
        assert!(summary.contains("namespace=main"));
        assert!(summary.contains("session=codex-a"));
        assert!(summary.contains("tab=tab-a"));
        assert!(summary.contains("agent=codex"));
        assert!(summary.contains("voice=normal"));
        assert!(summary.contains("server=ok"));
        assert!(summary.contains("rag=ready"));
        assert!(summary.contains("prompt_pressure=high"));
        assert!(summary.contains("tok=1842"));
        assert!(summary.contains("drivers=duplicates,inbox,refresh,rehydration,semantic,tokens"));
        assert!(summary.contains("action=\"drain inbox before the next prompt\""));
        assert!(summary.contains("warning=\"prompt pressure high\""));
    }

    #[test]
    fn status_summary_surfaces_lane_reroute_context() {
        let status = json!({
            "bundle": "/tmp/memd",
            "setup_ready": true,
            "server": { "status": "ok" },
            "rag": { "healthy": true },
            "missing": [],
            "defaults": {
                "project": "demo",
                "namespace": "main",
                "session": "codex-a",
                "tab_id": "tab-a",
                "agent": "codex"
            },
            "lane_surface": {
                "action": "auto_reroute",
                "previous_branch": "feature/hive-shared",
                "current_branch": "workerbee/codex-a",
                "conflict_session": "claude-b"
            },
            "lane_fault": {
                "kind": "unsafe_same_branch",
                "session": "claude-b"
            },
            "lane_receipts": {
                "count": 3,
                "latest_kind": "lane_fault"
            }
        });

        let summary = render_bundle_status_summary(&status);
        assert!(summary.contains("lane_action=auto_reroute"));
        assert!(summary.contains("lane_previous_branch=feature/hive-shared"));
        assert!(summary.contains("lane_current_branch=workerbee/codex-a"));
        assert!(summary.contains("lane_conflict_session=claude-b"));
        assert!(summary.contains("lane_fault=unsafe_same_branch"));
        assert!(summary.contains("lane_fault_session=claude-b"));
        assert!(summary.contains("lane_receipts=3"));
        assert!(summary.contains("lane_latest_receipt=lane_fault"));
    }

    #[test]
    fn status_summary_ignores_null_resume_preview_and_keeps_defaults_scope() {
        let status = json!({
            "bundle": "/tmp/memd",
            "setup_ready": true,
            "server": { "status": "ok" },
            "rag": { "healthy": true },
            "missing": [],
            "defaults": {
                "project": "demo",
                "namespace": "main",
                "session": "codex-a",
                "tab_id": "tab-a",
                "agent": "codex"
            },
            "resume_preview": null
        });

        let summary = render_bundle_status_summary(&status);
        assert!(summary.contains("project=demo"));
        assert!(summary.contains("namespace=main"));
        assert!(summary.contains("session=codex-a"));
        assert!(summary.contains("tab=tab-a"));
        assert!(summary.contains("agent=codex"));
        assert!(summary.contains("voice=caveman-ultra"));
    }

    #[test]
    fn status_summary_surfaces_live_session_rebind_when_present() {
        let status = json!({
            "bundle": "/tmp/memd",
            "setup_ready": true,
            "server": { "status": "ok" },
            "rag": { "healthy": true },
            "missing": [],
            "defaults": {
                "project": "demo",
                "namespace": "main",
                "session": "codex-fresh",
                "tab_id": "tab-a",
                "agent": "codex"
            },
            "session_overlay": {
                "bundle_session": "codex-stale",
                "live_session": "codex-fresh",
                "rebased_from": "codex-stale"
            },
            "resume_preview": null
        });

        let summary = render_bundle_status_summary(&status);
        assert!(summary.contains("session=codex-fresh"));
        assert!(summary.contains("bundle_session=codex-stale"));
        assert!(summary.contains("live_session=codex-fresh"));
        assert!(summary.contains("rebased_from=codex-stale"));
    }

    #[test]
    fn status_summary_surfaces_truth_first_retrieval_state() {
        let status = json!({
            "bundle": "/tmp/memd",
            "setup_ready": true,
            "server": { "status": "ok" },
            "rag": { "healthy": true },
            "missing": [],
            "defaults": {
                "project": "demo",
                "namespace": "main",
                "session": "codex-a",
                "tab_id": "tab-a",
                "agent": "codex"
            },
            "resume_preview": null,
            "truth_summary": {
                "truth": "current",
                "freshness": "fresh",
                "retrieval_tier": "hot",
                "confidence": 0.97,
                "action_hint": "use the event spine first",
                "source_count": 3,
                "contested_sources": 1,
                "records": [
                    {
                        "lane": "live_truth",
                        "preview": "event spine compact summary"
                    }
                ]
            }
        });

        let summary = render_bundle_status_summary(&status);
        assert!(summary.contains("truth=current"));
        assert!(summary.contains("freshness=fresh"));
        assert!(summary.contains("retrieval=hot"));
        assert!(summary.contains("conf=0.97"));
        assert!(summary.contains("sources=3"));
        assert!(summary.contains("contested=1"));
        assert!(summary.contains("truth_action=\"use the event spine first\""));
        assert!(summary.contains("truth_head=live_truth"));
        assert!(summary.contains("truth_preview=\"event spine compact summary\""));
    }

    #[test]
    fn status_summary_surfaces_capability_surface_counts() {
        let status = json!({
            "bundle": "/tmp/memd",
            "setup_ready": true,
            "server": { "status": "ok" },
            "rag": { "healthy": true },
            "missing": [],
            "defaults": {
                "project": "demo",
                "namespace": "main",
                "session": "codex-a",
                "tab_id": "tab-a",
                "agent": "codex"
            },
            "resume_preview": null,
            "capability_surface": {
                "discovered": 12,
                "universal": 4,
                "bridgeable": 3,
                "harness_native": 5
            }
        });

        let summary = render_bundle_status_summary(&status);
        assert!(summary.contains("capabilities=12"));
        assert!(summary.contains("universal=4"));
        assert!(summary.contains("bridgeable=3"));
        assert!(summary.contains("harness_native=5"));
    }

    #[test]
    fn status_summary_surfaces_cowork_counts() {
        let status = json!({
            "bundle": "/tmp/memd",
            "setup_ready": true,
            "server": { "status": "ok" },
            "rag": { "healthy": true },
            "missing": [],
            "defaults": {
                "project": "demo",
                "namespace": "main",
                "session": "codex-a",
                "tab_id": "tab-a",
                "agent": "codex"
            },
            "resume_preview": null,
            "cowork_surface": {
                "tasks": 4,
                "open_tasks": 3,
                "help_tasks": 1,
                "review_tasks": 2,
                "exclusive_tasks": 2,
                "shared_tasks": 2,
                "inbox_messages": 3,
                "owned_tasks": 1,
                "views": {
                    "owned": 1,
                    "open": 3,
                    "help": 1,
                    "review": 2,
                    "exclusive": 2,
                    "shared": 2
                }
            }
        });

        let summary = render_bundle_status_summary(&status);
        assert!(summary.contains("cowork_tasks=4"));
        assert!(summary.contains("open=3"));
        assert!(summary.contains("help=1"));
        assert!(summary.contains("review=2"));
        assert!(summary.contains("exclusive=2"));
        assert!(summary.contains("shared=2"));
        assert!(summary.contains("inbox_messages=3"));
        assert!(summary.contains("owned=1"));
        assert!(
            summary.contains("cowork_views=owned:1|open:3|help:1|review:2|exclusive:2|shared:2")
        );
    }

    #[test]
    fn status_summary_surfaces_maintenance_state() {
        let status = json!({
            "bundle": "/tmp/memd",
            "setup_ready": true,
            "server": { "status": "ok" },
            "rag": { "healthy": true },
            "missing": [],
            "defaults": {
                "project": "demo",
                "namespace": "main",
                "session": "codex-a",
                "tab_id": "tab-a",
                "agent": "codex"
            },
            "resume_preview": null,
            "maintenance_surface": {
                "mode": "auto",
                "auto_mode": true,
                "auto_recommended": false,
                "auto_reason": "none",
                "receipt": "r-123",
                "compacted": 3,
                "refreshed": 1,
                "repaired": 0,
                "findings": 2,
                "total_actions": 4,
                "delta_total_actions": 2,
                "trend": "up",
                "history_count": 2
            }
        });

        let summary = render_bundle_status_summary(&status);
        assert!(summary.contains("maintain_mode=auto"));
        assert!(summary.contains("auto=yes"));
        assert!(summary.contains("auto_recommended=no"));
        assert!(summary.contains("auto_reason=none"));
        assert!(summary.contains("maintain_receipt=r-123"));
        assert!(summary.contains("compacted=3"));
        assert!(summary.contains("refreshed=1"));
        assert!(summary.contains("repaired=0"));
        assert!(summary.contains("findings=2"));
        assert!(summary.contains("maintain_total=4"));
        assert!(summary.contains("maintain_delta=2"));
        assert!(summary.contains("maintain_trend=up"));
        assert!(summary.contains("history=2"));
    }

    #[test]
    fn visible_memory_home_summary_surfaces_artifact_and_pressure() {
        let snapshot = VisibleMemorySnapshotResponse {
            generated_at: chrono::Utc::now(),
            home: VisibleMemoryHome {
                focus_artifact: VisibleMemoryArtifact {
                    id: uuid::Uuid::new_v4(),
                    title: "runtime spine".to_string(),
                    body: "runtime spine is the canonical memory contract".to_string(),
                    artifact_kind: "compiled_page".to_string(),
                    memory_kind: Some(MemoryKind::Decision),
                    scope: Some(MemoryScope::Project),
                    visibility: Some(MemoryVisibility::Workspace),
                    workspace: Some("team-alpha".to_string()),
                    status: VisibleMemoryStatus::Current,
                    freshness: "verified".to_string(),
                    confidence: 0.94,
                    provenance: VisibleMemoryProvenance {
                        source_system: Some("obsidian".to_string()),
                        source_path: Some("wiki/runtime-spine.md".to_string()),
                        producer: Some("obsidian compile".to_string()),
                        trust_reason: "verified workspace page".to_string(),
                        last_verified_at: None,
                    },
                    sources: vec!["wiki/runtime-spine.md".to_string()],
                    linked_artifact_ids: vec![uuid::Uuid::new_v4()],
                    linked_sessions: vec!["codex-01".to_string()],
                    linked_agents: vec!["codex".to_string()],
                    repair_state: "healthy".to_string(),
                    actions: vec![
                        "inspect".to_string(),
                        "explain".to_string(),
                        "verify_current".to_string(),
                    ],
                },
                inbox_count: 3,
                repair_count: 1,
                awareness_count: 2,
            },
            knowledge_map: VisibleMemoryKnowledgeMap {
                nodes: vec![
                    VisibleMemoryGraphNode {
                        artifact_id: uuid::Uuid::new_v4(),
                        title: "runtime spine".to_string(),
                        artifact_kind: "compiled_page".to_string(),
                        status: VisibleMemoryStatus::Current,
                    },
                    VisibleMemoryGraphNode {
                        artifact_id: uuid::Uuid::new_v4(),
                        title: "workspace lane".to_string(),
                        artifact_kind: "workspace_lane".to_string(),
                        status: VisibleMemoryStatus::Candidate,
                    },
                ],
                edges: vec![VisibleMemoryGraphEdge {
                    from: uuid::Uuid::new_v4(),
                    to: uuid::Uuid::new_v4(),
                    relation: "related".to_string(),
                }],
            },
        };

        let summary = render_visible_memory_home(&snapshot, true);
        assert!(summary.contains("memory_home focus=runtime spine"));
        assert!(summary.contains("status=current"));
        assert!(summary.contains("freshness=verified"));
        assert!(summary.contains("visibility=workspace"));
        assert!(summary.contains("workspace=team-alpha"));
        assert!(summary.contains("inbox=3"));
        assert!(summary.contains("repair=1"));
        assert!(summary.contains("awareness=2"));
        assert!(summary.contains("nodes=2"));
        assert!(summary.contains("edges=1"));
        assert!(summary.contains("source_path=wiki/runtime-spine.md"));
        assert!(summary.contains("actions=inspect|explain|verify_current"));
        assert!(summary.contains("trail=runtime spine:compiled_page:current"));
    }

    #[test]
    fn visible_memory_artifact_detail_summary_surfaces_related_artifacts_and_actions() {
        let related_id = uuid::Uuid::new_v4();
        let snapshot = VisibleMemoryArtifactDetailResponse {
            generated_at: chrono::Utc::now(),
            artifact: VisibleMemoryArtifact {
                id: uuid::Uuid::new_v4(),
                title: "runtime spine".to_string(),
                body: "runtime spine is the canonical memory contract".to_string(),
                artifact_kind: "compiled_page".to_string(),
                memory_kind: Some(MemoryKind::Decision),
                scope: Some(MemoryScope::Project),
                visibility: Some(MemoryVisibility::Workspace),
                workspace: Some("team-alpha".to_string()),
                status: VisibleMemoryStatus::Current,
                freshness: "verified".to_string(),
                confidence: 0.97,
                provenance: VisibleMemoryProvenance {
                    source_system: Some("obsidian".to_string()),
                    source_path: Some("wiki/runtime-spine.md".to_string()),
                    producer: Some("obsidian compile".to_string()),
                    trust_reason: "verified workspace page".to_string(),
                    last_verified_at: None,
                },
                sources: vec!["wiki/runtime-spine.md".to_string()],
                linked_artifact_ids: vec![related_id],
                linked_sessions: vec!["codex-01".to_string()],
                linked_agents: vec!["codex".to_string()],
                repair_state: "healthy".to_string(),
                actions: vec!["inspect".to_string(), "open_in_obsidian".to_string()],
            },
            explain: None,
            timeline: None,
            sources: SourceMemoryResponse { sources: vec![] },
            workspaces: WorkspaceMemoryResponse { workspaces: vec![] },
            sessions: HiveSessionsResponse { sessions: vec![] },
            tasks: HiveTasksResponse { tasks: vec![] },
            claims: HiveClaimsResponse { claims: vec![] },
            related_artifacts: vec![VisibleMemoryArtifact {
                id: related_id,
                title: "workspace lane".to_string(),
                body: "lane supports shared memory inspection".to_string(),
                artifact_kind: "workspace_lane".to_string(),
                memory_kind: Some(MemoryKind::Status),
                scope: Some(MemoryScope::Project),
                visibility: Some(MemoryVisibility::Workspace),
                workspace: Some("team-alpha".to_string()),
                status: VisibleMemoryStatus::Candidate,
                freshness: "fresh".to_string(),
                confidence: 0.75,
                provenance: VisibleMemoryProvenance {
                    source_system: Some("server".to_string()),
                    source_path: Some("ui/home".to_string()),
                    producer: Some("visible memory snapshot".to_string()),
                    trust_reason: "snapshot neighbor".to_string(),
                    last_verified_at: None,
                },
                sources: vec!["ui/home".to_string()],
                linked_artifact_ids: vec![],
                linked_sessions: vec![],
                linked_agents: vec![],
                repair_state: "candidate".to_string(),
                actions: vec!["inspect".to_string()],
            }],
            related_map: VisibleMemoryKnowledgeMap {
                nodes: vec![
                    VisibleMemoryGraphNode {
                        artifact_id: related_id,
                        title: "runtime spine".to_string(),
                        artifact_kind: "compiled_page".to_string(),
                        status: VisibleMemoryStatus::Current,
                    },
                    VisibleMemoryGraphNode {
                        artifact_id: uuid::Uuid::new_v4(),
                        title: "workspace lane".to_string(),
                        artifact_kind: "workspace_lane".to_string(),
                        status: VisibleMemoryStatus::Candidate,
                    },
                ],
                edges: vec![VisibleMemoryGraphEdge {
                    from: related_id,
                    to: uuid::Uuid::new_v4(),
                    relation: "related".to_string(),
                }],
            },
            actions: vec![
                VisibleMemoryUiActionKind::Inspect,
                VisibleMemoryUiActionKind::OpenInObsidian,
            ],
        };

        let summary = render_visible_memory_artifact_detail(&snapshot, true);
        assert!(summary.contains("memory_artifact id="));
        assert!(summary.contains("title=runtime spine"));
        assert!(summary.contains("status=current"));
        assert!(summary.contains("kind=compiled_page"));
        assert!(summary.contains("explain=off"));
        assert!(summary.contains("timeline=off"));
        assert!(summary.contains("sources=0"));
        assert!(summary.contains("workspaces=0"));
        assert!(summary.contains("sessions=0"));
        assert!(summary.contains("tasks=0"));
        assert!(summary.contains("claims=0"));
        assert!(summary.contains("related=1"));
        assert!(summary.contains("map_nodes=2"));
        assert!(summary.contains("map_edges=1"));
        assert!(summary.contains("actions=inspect|open_in_obsidian"));
        assert!(summary.contains("source_system=obsidian"));
        assert!(summary.contains("source_path=wiki/runtime-spine.md"));
        assert!(summary.contains("related_trail="));
        assert!(summary.contains("workspace lane:workspace_lane:candidate"));
        assert!(summary.contains("map_trail=runtime spine:compiled_page:current"));
    }

    #[test]
    fn visible_memory_knowledge_map_summary_surfaces_graph_counts() {
        let artifact_id = uuid::Uuid::new_v4();
        let snapshot = VisibleMemorySnapshotResponse {
            generated_at: chrono::Utc::now(),
            home: VisibleMemoryHome {
                focus_artifact: VisibleMemoryArtifact {
                    id: artifact_id,
                    title: "runtime spine".to_string(),
                    body: "runtime spine is the canonical memory contract".to_string(),
                    artifact_kind: "compiled_page".to_string(),
                    memory_kind: Some(MemoryKind::Decision),
                    scope: Some(MemoryScope::Project),
                    visibility: Some(MemoryVisibility::Workspace),
                    workspace: Some("team-alpha".to_string()),
                    status: VisibleMemoryStatus::Current,
                    freshness: "verified".to_string(),
                    confidence: 0.97,
                    provenance: VisibleMemoryProvenance {
                        source_system: Some("obsidian".to_string()),
                        source_path: Some("wiki/runtime-spine.md".to_string()),
                        producer: Some("obsidian compile".to_string()),
                        trust_reason: "verified workspace page".to_string(),
                        last_verified_at: None,
                    },
                    sources: vec!["wiki/runtime-spine.md".to_string()],
                    linked_artifact_ids: vec![],
                    linked_sessions: vec![],
                    linked_agents: vec![],
                    repair_state: "healthy".to_string(),
                    actions: vec!["inspect".to_string()],
                },
                inbox_count: 1,
                repair_count: 2,
                awareness_count: 3,
            },
            knowledge_map: VisibleMemoryKnowledgeMap {
                nodes: vec![
                    VisibleMemoryGraphNode {
                        artifact_id,
                        title: "runtime spine".to_string(),
                        artifact_kind: "compiled_page".to_string(),
                        status: VisibleMemoryStatus::Current,
                    },
                    VisibleMemoryGraphNode {
                        artifact_id: uuid::Uuid::new_v4(),
                        title: "workspace lane".to_string(),
                        artifact_kind: "workspace_lane".to_string(),
                        status: VisibleMemoryStatus::Candidate,
                    },
                    VisibleMemoryGraphNode {
                        artifact_id: uuid::Uuid::new_v4(),
                        title: "stale note".to_string(),
                        artifact_kind: "note".to_string(),
                        status: VisibleMemoryStatus::Stale,
                    },
                    VisibleMemoryGraphNode {
                        artifact_id: uuid::Uuid::new_v4(),
                        title: "superseded note".to_string(),
                        artifact_kind: "note".to_string(),
                        status: VisibleMemoryStatus::Superseded,
                    },
                    VisibleMemoryGraphNode {
                        artifact_id: uuid::Uuid::new_v4(),
                        title: "conflicted note".to_string(),
                        artifact_kind: "note".to_string(),
                        status: VisibleMemoryStatus::Conflicted,
                    },
                    VisibleMemoryGraphNode {
                        artifact_id: uuid::Uuid::new_v4(),
                        title: "archived note".to_string(),
                        artifact_kind: "note".to_string(),
                        status: VisibleMemoryStatus::Archived,
                    },
                ],
                edges: vec![
                    VisibleMemoryGraphEdge {
                        from: artifact_id,
                        to: uuid::Uuid::new_v4(),
                        relation: "related".to_string(),
                    },
                    VisibleMemoryGraphEdge {
                        from: uuid::Uuid::new_v4(),
                        to: uuid::Uuid::new_v4(),
                        relation: "links".to_string(),
                    },
                ],
            },
        };

        let summary = render_visible_memory_knowledge_map(&snapshot, true);
        assert!(summary.contains("memory_map focus=runtime spine"));
        assert!(summary.contains("nodes=6"));
        assert!(summary.contains("edges=2"));
        assert!(summary.contains("current=1"));
        assert!(summary.contains("candidate=1"));
        assert!(summary.contains("stale=1"));
        assert!(summary.contains("superseded=1"));
        assert!(summary.contains("conflicted=1"));
        assert!(summary.contains("archived=1"));
        assert!(summary.contains("timeline_nodes=0"));
        assert!(summary.contains("workspace_nodes=1"));
        assert!(summary.contains("trail=runtime spine:compiled_page:current"));
        assert!(summary.contains("edge_trail=related:"));
        assert!(summary.contains("focus_workspace=team-alpha"));
        assert!(summary.contains("focus_visibility=workspace"));
    }
}
