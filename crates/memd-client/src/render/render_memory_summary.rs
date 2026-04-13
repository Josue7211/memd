use super::*;

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

pub(crate) fn render_entity_search_summary(
    response: &EntitySearchResponse,
    follow: bool,
) -> String {
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
                    "{}:{}:{}",
                    trace.typed_memory,
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

pub(crate) fn render_policy_summary(response: &MemoryPolicyResponse, follow: bool) -> String {
    let runtime = &response.runtime;
    let runtime_state = if is_default_runtime(runtime) {
        "defaulted"
    } else {
        "live"
    };
    let mut output = format!(
        "policy trust_floor={:.2} working_limit={} rehydrate_limit={} feedback={} semantic_fallback={} skill_gating={} runtime={} read_once={} event_driven={} sandbox_eval={} low_risk_auto={} approval={}",
        response.source_trust_floor,
        response.working_memory.default_limit,
        response.working_memory.rehydration_limit,
        bool_label(response.retrieval_feedback.enabled),
        bool_label(runtime.semantic_fallback.enabled),
        bool_label(runtime.skill_gating.gated_activation),
        runtime_state,
        bool_label(runtime.live_truth.read_once_sources),
        bool_label(runtime.memory_compilation.event_driven_updates),
        bool_label(runtime.skill_gating.sandboxed_evaluation),
        bool_label(runtime.skill_gating.auto_activate_low_risk_only),
        bool_label(runtime.skill_gating.require_policy_approval),
    );

    if follow {
        output.push_str(&format!(
            " route_defaults={} surfaces={} promote_min_salience={:.2} consolidate_min_salience={:.2}",
            response.route_defaults.len(),
            response.retrieval_feedback.tracked_surfaces.join("|"),
            response.promotion.min_salience,
            response.consolidation.min_salience
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use memd_schema::{
        CompactMemoryRecord, MemoryKind, MemoryScope, MemoryStage, MemoryVisibility,
        SourceMemoryRecord, SourceMemoryResponse, WorkingMemoryPolicyState,
        WorkingMemoryTraceRecord,
    };
    use uuid::Uuid;

    #[test]
    fn render_working_summary_surfaces_typed_trace_trail_in_verification_suite() {
        let response = WorkingMemoryResponse {
            route: RetrievalRoute::ProjectFirst,
            intent: RetrievalIntent::CurrentTask,
            retrieval_order: vec![MemoryScope::Project, MemoryScope::Synced],
            budget_chars: 1600,
            used_chars: 240,
            remaining_chars: 1360,
            truncated: false,
            policy: WorkingMemoryPolicyState {
                admission_limit: 8,
                max_chars_per_item: 220,
                budget_chars: 1600,
                rehydration_limit: 4,
            },
            records: vec![CompactMemoryRecord {
                id: Uuid::new_v4(),
                record: "current task: lock typed trace families".to_string(),
            }],
            evicted: Vec::new(),
            rehydration_queue: Vec::new(),
            traces: vec![WorkingMemoryTraceRecord {
                item_id: Uuid::new_v4(),
                entity_id: Some(Uuid::new_v4()),
                memory_kind: MemoryKind::Status,
                memory_stage: MemoryStage::Candidate,
                typed_memory: "session_continuity+candidate".to_string(),
                event_type: "retrieved_context".to_string(),
                summary: "continuity state entered working set".to_string(),
                occurred_at: Utc::now(),
                salience_score: 0.82,
            }],
            semantic_consolidation: None,
            procedures: vec![],
        };

        let summary = render_working_summary(&response, true);
        assert!(summary.contains("trace_trail=session_continuity+candidate:retrieved_context:continuity state entered working set"));
    }

    #[test]
    fn render_source_summary_surfaces_provenance_trail_and_tags() {
        let response = SourceMemoryResponse {
            sources: vec![SourceMemoryRecord {
                source_agent: Some("codex@test".to_string()),
                source_system: Some("hook-capture".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("core".to_string()),
                visibility: MemoryVisibility::Workspace,
                item_count: 3,
                active_count: 2,
                candidate_count: 1,
                derived_count: 0,
                synthetic_count: 0,
                contested_count: 0,
                avg_confidence: 0.88,
                trust_score: 0.93,
                last_seen_at: Some(Utc::now()),
                tags: vec!["raw-spine".to_string(), "correction".to_string()],
            }],
        };

        let summary = render_source_summary(&response, true);
        assert!(summary.contains("source_memory sources=1"));
        assert!(summary.contains("top=codex@test"));
        assert!(summary.contains("system=hook-capture"));
        assert!(summary.contains("workspace=core"));
        assert!(summary.contains("items=3"));
        assert!(summary.contains("trust=0.93"));
        assert!(summary.contains("avg_confidence=0.88"));
        assert!(summary.contains("trail=codex@test:hook-capture:core:3:0.93"));
        assert!(summary.contains("tags=raw-spine|correction"));
    }
}

pub(crate) fn render_skill_policy_summary(response: &MemoryPolicyResponse, follow: bool) -> String {
    let runtime = &response.runtime;
    let skill = &runtime.skill_gating;
    let runtime_state = if is_default_runtime(runtime) {
        "defaulted"
    } else {
        "live"
    };
    let mut output = format!(
        "skill-policy propose={} sandbox={} activate={} low_risk_auto={} eval={} approval={} runtime={} read_once={} event_driven={} visible_memory={} semantic_fallback={}",
        bool_label(skill.propose_from_repeated_patterns),
        bool_label(skill.sandboxed_evaluation),
        bool_label(skill.gated_activation),
        bool_label(skill.auto_activate_low_risk_only),
        bool_label(skill.require_evaluation),
        bool_label(skill.require_policy_approval),
        runtime_state,
        bool_label(runtime.live_truth.read_once_sources),
        bool_label(runtime.memory_compilation.event_driven_updates),
        bool_label(runtime.live_truth.visible_memory_objects),
        bool_label(runtime.semantic_fallback.enabled),
    );

    if follow {
        output.push_str(&format!(
            " flow=pattern->proposal->sandbox->eval->policy->activate runtime={} trust_floor={:.2} semantic_max={}",
            if skill.gated_activation { "gated" } else { "open" },
            response.source_trust_floor,
            runtime.semantic_fallback.max_items_per_query
        ));
    }

    output
}

pub(crate) fn render_skill_catalog_summary(catalog: &SkillCatalog) -> String {
    let custom_visible = catalog.custom.len();
    let builtin_visible = catalog.builtins.len();
    let mut output = format!(
        "skills root={} builtin={} custom={} cache_hits={} scanned={} mode=hybrid",
        catalog.root.display(),
        builtin_visible,
        custom_visible,
        catalog.cache_hits,
        catalog.cache_scanned,
    );
    if let Some(first) = catalog.custom.first() {
        output.push_str(&format!(" first_custom={}::{}", first.name, first.status));
    }
    output
}

pub(crate) fn render_skill_catalog_markdown(catalog: &SkillCatalog) -> String {
    let mut output = String::new();
    output.push_str("# memd skills\n\n");
    output.push_str(&format!("- Root: `{}`\n", catalog.root.display()));
    output.push_str(&format!("- Built-in: `{}`\n", catalog.builtins.len()));
    output.push_str(&format!("- Custom: `{}`\n\n", catalog.custom.len()));
    output.push_str("## Built-in\n\n");
    for entry in &catalog.builtins {
        push_skill_entry_markdown(&mut output, entry);
    }
    output.push_str("\n## Custom\n\n");
    if catalog.custom.is_empty() {
        output.push_str("No custom skills found.\n");
    } else {
        for entry in &catalog.custom {
            push_skill_entry_markdown(&mut output, entry);
        }
    }
    output
}

pub(crate) fn render_skill_catalog_match_summary(
    catalog: &SkillCatalog,
    query: &str,
    matches: &[&SkillCatalogEntry],
) -> String {
    let mut output = format!(
        "skills query=\"{}\" root={} matches={} builtins={} custom={}",
        compact_inline(query, 48),
        catalog.root.display(),
        matches.len(),
        catalog.builtins.len(),
        catalog.custom.len(),
    );
    if let Some(first) = matches.first() {
        output.push_str(&format!(
            " best={} source={} status={} path={} next={} usage={} decision={}",
            first.name,
            first.source,
            first.status,
            first
                .path
                .as_ref()
                .map(|path: &std::path::PathBuf| path.display().to_string())
                .unwrap_or_else(|| "builtin".to_string()),
            skill_activation_path(first),
            first.usage,
            first.decision
        ));
    }
    output
}

pub(crate) fn render_skill_catalog_match_markdown(
    catalog: &SkillCatalog,
    query: &str,
    matches: &[&SkillCatalogEntry],
) -> String {
    let mut output = String::new();
    output.push_str("# memd skill drilldown\n\n");
    output.push_str(&format!("- Query: `{}`\n", query));
    output.push_str(&format!("- Root: `{}`\n", catalog.root.display()));
    output.push_str(&format!("- Matches: `{}`\n\n", matches.len()));
    if matches.is_empty() {
        output.push_str("No skill matched.\n");
        return output;
    }
    for entry in matches {
        output.push_str(&format!(
            "## {}\n\n- source: `{}`\n- status: `{}`\n- next: `{}`\n- usage: `{}`\n- decision: `{}`\n- summary: {}\n",
            entry.name,
            entry.source,
            entry.status,
            skill_activation_path(entry),
            entry.usage,
            entry.decision,
            entry.summary
        ));
        if let Some(path) = entry.path.as_ref() {
            output.push_str(&format!("- path: `{}`\n", path.display()));
        }
        output.push('\n');
    }
    output
}

fn skill_activation_path(entry: &SkillCatalogEntry) -> &'static str {
    if entry.source == "built-in" {
        "use memd subcommand"
    } else {
        "edit the file, then propose via skill-policy"
    }
}

fn push_skill_entry_markdown(output: &mut String, entry: &SkillCatalogEntry) {
    output.push_str(&format!(
        "- `{}` [{}] - {}\n",
        entry.name, entry.status, entry.summary
    ));
    output.push_str(&format!("  - usage: `{}`\n", entry.usage));
    output.push_str(&format!("  - decision: `{}`\n", entry.decision));
    if let Some(path) = entry.path.as_ref() {
        output.push_str(&format!("  - path: `{}`\n", path.display()));
    }
    output.push_str(&format!("  - source: `{}`\n", entry.source));
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

pub(crate) fn render_workspace_summary(response: &WorkspaceMemoryResponse, follow: bool) -> String {
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
