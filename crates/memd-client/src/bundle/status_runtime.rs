use super::*;

fn memd_status_full_mode() -> bool {
    std::env::var("MEMD_STATUS_FULL")
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

pub(crate) async fn read_bundle_status(
    output: &Path,
    base_url: &str,
) -> anyhow::Result<serde_json::Value> {
    let runtime_before_overlay = read_bundle_runtime_config_raw(output)?;
    let runtime = read_bundle_runtime_config(output)?;
    let bundle_session = runtime_before_overlay
        .as_ref()
        .and_then(|config| config.session.clone());
    let live_session = runtime.as_ref().and_then(|config| config.session.clone());
    let rebased_from = match (bundle_session.as_deref(), live_session.as_deref()) {
        (Some(bundle), Some(live)) if bundle != live => Some(bundle.to_string()),
        _ => None,
    };
    let resolved_base_url = resolve_bundle_command_base_url(
        base_url,
        runtime
            .as_ref()
            .and_then(|config| config.base_url.as_deref()),
    );
    let full_mode = memd_status_full_mode();
    if runtime
        .as_ref()
        .and_then(|config| config.session.as_deref())
        .is_some()
        && full_mode
    {
        let _ = timeout_ok(refresh_bundle_heartbeat(output, None, false)).await;
    }
    let client = MemdClient::new(&resolved_base_url)?;
    let health = timeout_ok(client.healthz()).await;
    let heartbeat = read_bundle_heartbeat(output)?.map(|mut state| {
        if state.project.is_none() {
            state.project = runtime.as_ref().and_then(|config| config.project.clone());
        }
        if state.namespace.is_none() {
            state.namespace = runtime.as_ref().and_then(|config| config.namespace.clone());
        }
        if state.workspace.is_none() {
            state.workspace = runtime.as_ref().and_then(|config| config.workspace.clone());
        }
        if state.visibility.is_none() {
            state.visibility = runtime
                .as_ref()
                .and_then(|config| config.visibility.clone());
        }
        if state.session.is_none() {
            state.session = runtime.as_ref().and_then(|config| config.session.clone());
        }
        if state.agent.is_none() {
            state.agent = runtime.as_ref().and_then(|config| config.agent.clone());
        }
        if state.effective_agent.is_none() {
            state.effective_agent = runtime.as_ref().and_then(|config| {
                config
                    .agent
                    .as_deref()
                    .map(|agent| compose_agent_identity(agent, config.session.as_deref()))
            });
        }
        if state.tab_id.is_none() {
            state.tab_id = runtime.as_ref().and_then(|config| config.tab_id.clone());
        }
        state
    });
    let runtimes = read_memd_runtime_wiring();
    let harness_bridge =
        read_bundle_harness_bridge_registry(output)?.unwrap_or_else(build_harness_bridge_registry);
    let config_exists = output.join("config.json").exists();
    let env_exists = output.join("env").exists();
    let env_ps1_exists = output.join("env.ps1").exists();
    let hooks_exists = output.join("hooks").exists();
    let agents_exists = output.join("agents").exists();
    let worker_name_env_ready = read_bundle_config_file(output)
        .ok()
        .map(|(_, config)| bundle_worker_name_env_ready(output, &config))
        .unwrap_or(false);
    let mut missing = Vec::<&str>::new();
    if !config_exists {
        missing.push("config.json");
    }
    if !env_exists {
        missing.push("env");
    }
    if !env_ps1_exists {
        missing.push("env.ps1");
    }
    if env_exists && env_ps1_exists && !worker_name_env_ready {
        missing.push("worker_name_env");
    }
    if !hooks_exists {
        missing.push("hooks/");
    }
    if !agents_exists {
        missing.push("agents/");
    }
    let mut memory_quality_degraded = false;
    let resume_preview = if full_mode && output.join("config.json").exists() && health.is_some() {
        let preview = timeout_ok(crate::runtime::read_bundle_resume(
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
                semantic: false,
                prompt: false,
                summary: false,
            },
            &resolved_base_url,
        ))
        .await;
        preview.map(|snapshot| {
            // Lightweight memory quality check (mirrors eval assertions).
            let total_working = snapshot.working.records.len();
            let status_count = snapshot
                .working
                .records
                .iter()
                .filter(|r| r.record.contains("kind=status"))
                .count();
            let has_non_status = total_working > status_count;
            let context_has_facts = snapshot.context.records.iter().any(|r| {
                r.record.contains("kind=fact")
                    || r.record.contains("kind=decision")
                    || r.record.contains("kind=procedural")
            });
            if (total_working > 0 && !has_non_status) || (!snapshot.context.records.is_empty() && !context_has_facts) {
                memory_quality_degraded = true;
            }

            serde_json::json!({
                "project": snapshot.project,
                "namespace": snapshot.namespace,
                "agent": snapshot.agent,
                "session": runtime.as_ref().and_then(|config| config.session.clone()),
                "tab_id": runtime.as_ref().and_then(|config| config.tab_id.clone()),
                "workspace": snapshot.workspace,
                "visibility": snapshot.visibility,
                "route": snapshot.route,
                "intent": snapshot.intent,
                "context_records": snapshot.context.records.len(),
                "working_records": snapshot.working.records.len(),
                "working_status_records": status_count,
                "working_has_non_status": has_non_status,
                "context_has_facts": context_has_facts,
                "inbox_items": snapshot.inbox.items.len(),
                "workspace_lanes": snapshot.workspaces.workspaces.len(),
                "rehydration_queue": snapshot.working.rehydration_queue.len(),
                "semantic_hits": snapshot.semantic.as_ref().map(|semantic| semantic.items.len()).unwrap_or(0),
                "change_summary": snapshot.change_summary,
                "event_spine": snapshot.event_spine(),
                "focus": snapshot.working.records.first().map(|record| record.record.clone()),
                "pressure": snapshot.inbox.items.first().map(|item| item.item.content.clone()),
                "next_recovery": snapshot.working.rehydration_queue.first().map(|item| format!("{}: {}", item.label, item.summary)),
                "estimated_prompt_chars": snapshot.estimated_prompt_chars(),
                "estimated_prompt_tokens": snapshot.estimated_prompt_tokens(),
                "context_pressure": snapshot.context_pressure(),
                "redundant_context_items": snapshot.redundant_context_items(),
                "refresh_recommended": snapshot.refresh_recommended,
            })
        })
    } else {
        None
    };
    let truth_summary = if full_mode && output.join("config.json").exists() && health.is_some() {
        let snapshot = timeout_ok(crate::runtime::read_bundle_resume(
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
            &resolved_base_url,
        ))
        .await;
        snapshot.map(|snapshot| {
            serde_json::to_value(build_truth_summary(&snapshot)).unwrap_or(JsonValue::Null)
        })
    } else {
        None
    };
    let current_project = runtime.as_ref().and_then(|config| config.project.clone());
    let current_namespace = runtime.as_ref().and_then(|config| config.namespace.clone());
    let current_workspace = runtime.as_ref().and_then(|config| config.workspace.clone());
    let current_session = runtime.as_ref().and_then(|config| config.session.clone());
    let cowork_surface = if full_mode && health.is_some() {
        let inbox_request = HiveCoordinationInboxRequest {
            session: current_session.clone().unwrap_or_default(),
            project: current_project.clone(),
            namespace: current_namespace.clone(),
            workspace: current_workspace.clone(),
            limit: Some(128),
        };
        let inbox = timeout_ok(client.hive_coordination_inbox(&inbox_request)).await;
        let tasks_request = HiveTasksRequest {
            session: None,
            project: current_project.clone(),
            namespace: current_namespace.clone(),
            workspace: current_workspace.clone(),
            active_only: Some(false),
            limit: Some(256),
        };
        let tasks = timeout_ok(client.hive_tasks(&tasks_request)).await;
        match (inbox, tasks) {
            (Some(inbox), Some(tasks)) => {
                let exclusive = tasks
                    .tasks
                    .iter()
                    .filter(|task| task.coordination_mode == "exclusive_write")
                    .count();
                let open = tasks
                    .tasks
                    .iter()
                    .filter(|task| task.status != "done" && task.status != "closed")
                    .count();
                Some(serde_json::json!({
                    "tasks": tasks.tasks.len(),
                    "open_tasks": open,
                    "help_tasks": tasks.tasks.iter().filter(|task| task.help_requested).count(),
                    "review_tasks": tasks.tasks.iter().filter(|task| task.review_requested).count(),
                    "exclusive_tasks": exclusive,
                    "shared_tasks": tasks.tasks.len().saturating_sub(exclusive),
                    "inbox_messages": inbox.messages.len(),
                    "owned_tasks": inbox.owned_tasks.len(),
                    "owned_exclusive_tasks": inbox
                        .owned_tasks
                        .iter()
                        .filter(|task| task.coordination_mode == "exclusive_write")
                        .count(),
                    "owned_shared_tasks": inbox
                        .owned_tasks
                        .iter()
                        .filter(|task| task.coordination_mode != "exclusive_write")
                        .count(),
                    "help_inbox": inbox.help_tasks.len(),
                    "review_inbox": inbox.review_tasks.len(),
                    "views": build_task_view_counts(&tasks.tasks, current_session.as_deref()),
                }))
            }
            _ => None,
        }
    } else {
        None
    };
    let lane_receipts = if full_mode && health.is_some() {
        let receipts_request = HiveCoordinationReceiptsRequest {
            session: None,
            project: current_project.clone(),
            namespace: current_namespace.clone(),
            workspace: current_workspace.clone(),
            limit: Some(64),
        };
        timeout_ok(client.hive_coordination_receipts(&receipts_request))
            .await
            .map(|response| {
                let receipts = response
                    .receipts
                    .into_iter()
                    .filter(|receipt| receipt.kind.starts_with("lane_"))
                    .collect::<Vec<_>>();
                serde_json::json!({
                    "count": receipts.len(),
                    "latest_kind": receipts.first().map(|receipt| receipt.kind.clone()),
                    "latest_summary": receipts.first().map(|receipt| receipt.summary.clone()),
                    "recent": receipts
                        .into_iter()
                        .take(8)
                        .map(|receipt| serde_json::json!({
                            "kind": receipt.kind,
                            "actor_session": receipt.actor_session,
                            "target_session": receipt.target_session,
                            "scope": receipt.scope,
                            "summary": receipt.summary,
                            "created_at": receipt.created_at,
                        }))
                        .collect::<Vec<_>>(),
                })
            })
    } else {
        None
    };
    let maintenance_surface = match (
        read_latest_maintain_report(output)?,
        read_previous_maintain_report(output)?,
        read_recent_maintain_reports(output, 5)?,
    ) {
        (Some(report), previous, history) => {
            let total = report.compacted_items + report.refreshed_items + report.repaired_items;
            let previous_total = previous
                .as_ref()
                .map(|value| value.compacted_items + value.refreshed_items + value.repaired_items)
                .unwrap_or(0);
            let delta_total = total as i64 - previous_total as i64;
            let auto_mode = report.mode == "auto";
            let auto_reason = if auto_mode {
                "none".to_string()
            } else if delta_total < 0 {
                "trend_down".to_string()
            } else if delta_total == 0 {
                "trend_flat".to_string()
            } else if !report.findings.is_empty() {
                "findings_present".to_string()
            } else {
                "none".to_string()
            };
            let auto_recommended = auto_reason != "none";
            let history_modes = history
                .iter()
                .map(|value| value.mode.clone())
                .collect::<Vec<_>>();
            let history_receipts = history
                .iter()
                .map(|value| {
                    value
                        .receipt_id
                        .clone()
                        .unwrap_or_else(|| "none".to_string())
                })
                .collect::<Vec<_>>();
            let history_totals = history
                .iter()
                .map(|value| value.compacted_items + value.refreshed_items + value.repaired_items)
                .collect::<Vec<_>>();
            Some(serde_json::json!({
                "mode": report.mode,
                "auto_mode": auto_mode,
                "auto_recommended": auto_recommended,
                "auto_reason": auto_reason,
                "receipt": report.receipt_id,
                "compacted": report.compacted_items,
                "refreshed": report.refreshed_items,
                "repaired": report.repaired_items,
                "findings": report.findings.len(),
                "total_actions": total,
                "delta_total_actions": delta_total,
                "trend": if delta_total > 0 { "up" } else if delta_total < 0 { "down" } else { "flat" },
                "previous_mode": previous.as_ref().map(|value| value.mode.clone()),
                "history_modes": history_modes,
                "history_receipts": history_receipts,
                "history_totals": history_totals,
                "history_count": history.len(),
                "generated_at": report.generated_at,
            }))
        }
        _ => None,
    };
    let rag_config = read_bundle_rag_config(output)?;
    let rag = match rag_config {
        Some(config) if config.enabled => {
            let source = config.source;
            let Some(url) = config.url.clone() else {
                return Ok(serde_json::json!({
                    "bundle": output,
                    "exists": output.exists(),
                    "config": output.join("config.json").exists(),
                    "env": output.join("env").exists(),
                    "env_ps1": output.join("env.ps1").exists(),
                    "hooks": output.join("hooks").exists(),
                    "agents": output.join("agents").exists(),
                    "server": health,
                    "rag": {
                        "configured": false,
                        "enabled": true,
                        "healthy": false,
                        "error": "rag backend enabled but no url configured",
                        "source": source,
                    },
                }));
            };
            if full_mode {
                let rag_result = RagClient::new(url.as_str())?.healthz().await;
                Some(match rag_result {
                    Ok(health) => serde_json::json!({
                        "configured": true,
                        "enabled": true,
                        "url": url,
                        "healthy": true,
                        "health": health,
                        "source": source,
                    }),
                    Err(error) => serde_json::json!({
                        "configured": true,
                        "enabled": true,
                        "url": url,
                        "healthy": false,
                        "error": error.to_string(),
                        "source": source,
                    }),
                })
            } else {
                Some(serde_json::json!({
                    "configured": true,
                    "enabled": true,
                    "url": url,
                    "healthy": null,
                    "source": source,
                }))
            }
        }
        Some(config) => Some(serde_json::json!({
            "configured": config.configured,
            "enabled": false,
            "url": config.url,
            "healthy": null,
            "source": config.source,
        })),
        None => None,
    };
    let rag_ready = rag
        .as_ref()
        .map(|value| {
            !value
                .get("enabled")
                .and_then(|enabled| enabled.as_bool())
                .unwrap_or(false)
                || value
                    .get("healthy")
                    .and_then(|healthy| healthy.as_bool())
                    .unwrap_or(false)
        })
        .unwrap_or(true);
    let evolution = summarize_evolution_status(output)?;
    let capability_registry =
        build_bundle_capability_registry(std::env::current_dir().ok().as_deref());
    let capability_surface = serde_json::json!({
        "discovered": capability_registry.capabilities.len(),
        "universal": capability_registry
            .capabilities
            .iter()
            .filter(|record| is_universal_class(&record.portability_class))
            .count(),
        "bridgeable": capability_registry
            .capabilities
            .iter()
            .filter(|record| is_bridgeable_class(&record.portability_class))
            .count(),
        "harness_native": capability_registry
            .capabilities
            .iter()
            .filter(|record| is_harness_native_class(&record.portability_class))
            .count(),
    });
    let lane_surface = read_bundle_lane_surface(output)?
        .map(|surface| serde_json::to_value(surface).unwrap_or(JsonValue::Null));
    let lane_fault = detect_bundle_lane_collision(output, current_session.as_deref())
        .await?
        .and_then(|conflict| {
            build_lane_fault_surface(output, current_session.as_deref(), &conflict)
        });
    let bridge_ready = harness_bridge.all_wired;
    let setup_ready = output.exists()
        && missing.is_empty()
        && health.is_some()
        && runtime.is_some()
        && rag_ready
        && bridge_ready;
    Ok(serde_json::json!({
        "bundle": output,
        "exists": output.exists(),
        "config": config_exists,
        "env": env_exists,
        "env_ps1": env_ps1_exists,
        "worker_name_env_ready": worker_name_env_ready,
        "hooks": hooks_exists,
        "agents": agents_exists,
        "setup_ready": setup_ready,
        "missing": missing,
        "runtimes": runtimes,
        "harness_bridge": {
            "ready": bridge_ready,
            "portable": harness_bridge.all_wired,
            "portability_class": harness_bridge.overall_portability_class,
            "generated_at": harness_bridge.generated_at,
            "harnesses": harness_bridge.harnesses,
            "missing_harnesses": harness_bridge
                .harnesses
                .iter()
                .filter(|record| !record.wired)
                .map(|record| record.harness.clone())
                .collect::<Vec<_>>(),
        },
        "active_agent": runtime.as_ref().and_then(|config| config.agent.clone()),
        "defaults": runtime.as_ref().and_then(|config| {
            let mut defaults = serde_json::to_value(config).ok()?;
            if let JsonValue::Object(ref mut map) = defaults {
                map.insert(
                    "voice_mode".to_string(),
                    JsonValue::String(
                        read_bundle_voice_mode(output).unwrap_or_else(default_voice_mode),
                    ),
                );
            }
            Some(defaults)
        }),
        "authority": runtime
            .as_ref()
            .map(|config| config.authority_state.mode.clone()),
        "shared_primary": runtime_prefers_shared_authority(runtime.as_ref()),
        "localhost_read_only_allowed": runtime_allows_localhost_read_only(runtime.as_ref()),
        "degraded": runtime
            .as_ref()
            .map(|config| config.authority_state.degraded)
            .unwrap_or(false)
            || memory_quality_degraded,
        "memory_quality_degraded": memory_quality_degraded,
        "shared_base_url": runtime
            .as_ref()
            .and_then(|config| config.authority_state.shared_base_url.clone()),
        "fallback_base_url": runtime
            .as_ref()
            .and_then(|config| config.authority_state.fallback_base_url.clone()),
        "authority_warning": authority_warning_lines(runtime.as_ref()),
        "session_overlay": {
            "bundle_session": bundle_session,
            "live_session": live_session,
            "rebased_from": rebased_from,
        },
        "heartbeat": heartbeat
            .as_ref()
            .and_then(|value| serde_json::to_value(value).ok()),
        "resume_preview": resume_preview,
        "truth_summary": truth_summary,
        "evolution": evolution,
        "cowork_surface": cowork_surface,
        "lane_surface": lane_surface,
        "lane_fault": lane_fault,
        "lane_receipts": lane_receipts,
        "maintenance_surface": maintenance_surface,
        "capability_surface": capability_surface,
        "server": health,
        "rag": rag.unwrap_or_else(|| serde_json::json!({
            "configured": false,
            "enabled": false,
            "healthy": null,
        })),
    }))
}
