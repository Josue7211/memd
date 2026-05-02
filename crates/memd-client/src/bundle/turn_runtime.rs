use super::*;
use memd_schema::IngestLanesRequest;

pub(crate) async fn run_bundle_wake_command(args: &WakeArgs, base_url: &str) -> anyhow::Result<()> {
    let recall_started = std::time::Instant::now();
    // F2: Re-ingest lane source files on every wake so modified files are picked up.
    if let Some(project_root) = infer_bundle_project_root(&args.output) {
        let runtime = read_bundle_runtime_config(&args.output).ok().flatten();
        let resolved_url = resolve_bundle_command_base_url(
            base_url,
            runtime.as_ref().and_then(|c| c.base_url.as_deref()),
        );
        if let Ok(client) = MemdClient::new(&resolved_url) {
            let _ = client
                .ingest_lanes(&IngestLanesRequest {
                    root: project_root.display().to_string(),
                    project: args
                        .project
                        .clone()
                        .or_else(|| runtime.as_ref().and_then(|c| c.project.clone())),
                    namespace: args
                        .namespace
                        .clone()
                        .or_else(|| runtime.as_ref().and_then(|c| c.namespace.clone())),
                })
                .await;
        }
    }

    if let Some(tab_id) = default_bundle_tab_id() {
        let existing_tab_id = read_bundle_runtime_config(&args.output)
            .ok()
            .flatten()
            .and_then(|config| config.tab_id)
            .filter(|value| !value.trim().is_empty());
        if existing_tab_id.is_none() {
            set_bundle_tab_id(&args.output, &tab_id)?;
        }
    }
    crate::runtime::invalidate_bundle_runtime_caches(&args.output)?;
    let codex_pack = harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "codex");
    let agent_zero_pack =
        harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "agent-zero");
    let hermes_pack =
        harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "hermes");
    let opencode_pack =
        harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "opencode");
    let openclaw_pack =
        harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "openclaw");
    let resume_args = ResumeArgs {
        output: args.output.clone(),
        project: args.project.clone(),
        namespace: args.namespace.clone(),
        agent: args.agent.clone(),
        workspace: args.workspace.clone(),
        visibility: args.visibility.clone(),
        route: args.route.clone(),
        intent: args.intent.clone().or(Some("current_task".to_string())),
        limit: args.limit,
        rehydration_limit: args.rehydration_limit,
        semantic: args.semantic,
        prompt: false,
        summary: false,
    };
    let snapshot = match crate::runtime::read_bundle_resume(&resume_args, base_url).await {
        Ok(snapshot) => snapshot,
        Err(err)
            if codex_pack || agent_zero_pack || hermes_pack || opencode_pack || openclaw_pack =>
        {
            if let Some(markdown) = read_codex_pack_local_markdown(&args.output, "wake.md")? {
                if args.write {
                    write_bundle_turn_fallback_artifacts(
                        &args.output,
                        args.project.as_deref(),
                        args.namespace.as_deref(),
                        args.agent.as_deref(),
                        args.workspace.as_deref(),
                        args.visibility.as_deref(),
                        args.route.as_deref(),
                        args.intent.as_deref(),
                        &markdown,
                    )?;
                }
                println!("{markdown}");
                return Ok(());
            }
            if args.write {
                write_bundle_turn_placeholder_memory(
                    &args.output,
                    args.project.as_deref(),
                    args.namespace.as_deref(),
                    args.agent.as_deref(),
                    args.workspace.as_deref(),
                    args.visibility.as_deref(),
                    args.route.as_deref(),
                    args.intent.as_deref(),
                )?;
            }
            return Err(err);
        }
        Err(err) => return Err(err),
    };
    let raw_wakeup = render_bundle_wakeup_markdown(&args.output, &snapshot, args.verbose);
    let raw_tokens = raw_wakeup.len();
    let use_compiler = !args.raw && crate::runtime::resume::compiler::compiler_enabled();
    let session_id = read_bundle_runtime_config(&args.output)
        .ok()
        .flatten()
        .and_then(|c| c.session);
    let model_family = args
        .agent
        .clone()
        .or_else(|| {
            read_bundle_runtime_config(&args.output)
                .ok()
                .flatten()
                .and_then(|c| c.agent)
        })
        .unwrap_or_else(|| "unknown".to_string());

    let wakeup = if use_compiler {
        let env_tokens = crate::runtime::resume::compiler::env_budget_tokens();
        let tokens = if args.budget_tokens > 0 {
            args.budget_tokens
        } else if env_tokens > 0 {
            env_tokens
        } else {
            2000
        };
        let budget = crate::runtime::resume::compiler::WakeBudget::default_2000()
            .with_tokens(tokens)
            .with_includes(&args.include_bucket)
            .with_excludes(&args.exclude_bucket);
        let mut input = crate::runtime::resume::compiler::input_from_snapshot(&snapshot);
        input.drift_notes =
            crate::runtime::resume::compiler::drift_notes_from_outstanding(&args.output);
        let compiled = crate::runtime::resume::compiler::compile_wake(input, budget);
        let _ = crate::runtime::resume::compiler::ledger::write_budget_line(
            &args.output,
            session_id.as_deref(),
            raw_tokens,
            &compiled,
        );
        let _ = crate::runtime::resume::compiler::ledger::write_cost_line(
            &args.output,
            session_id.as_deref(),
            compiled.tokens,
            tokens,
            &model_family,
        );
        compiled.markdown
    } else {
        raw_wakeup.clone()
    };
    let wake_token_metrics =
        crate::runtime::compute_wake_token_metrics(&args.output, &snapshot, &wakeup);
    if args.write {
        write_bundle_memory_files(&args.output, &snapshot, None, false).await?;
        auto_checkpoint_live_snapshot(&args.output, base_url, &snapshot, "wake").await?;
        update_hive_session_wake_timestamp(&args.output, base_url).await?;
        // P2: Persist wake token metrics for diagnostics report consumption.
        if let Ok(json) = serde_json::to_string_pretty(&wake_token_metrics) {
            let metrics_path = args.output.join("wake-token-metrics.json");
            let _ = std::fs::write(&metrics_path, json);
        }
    }
    if args.verbose {
        eprintln!(
            "[memd] wake token efficiency: {}/{} chars ({:.1}%), {} items across {} kinds",
            wake_token_metrics.used_chars,
            wake_token_metrics.budget_chars,
            wake_token_metrics.utilization_pct,
            wake_token_metrics.per_kind.total_items,
            wake_token_metrics.per_kind.chars_per_kind.len(),
        );
    }
    if codex_pack || agent_zero_pack || openclaw_pack || hermes_pack || opencode_pack {
        let _ = refresh_harness_pack_files_for_snapshot(
            &args.output,
            &snapshot,
            "wake",
            &["codex", "agent-zero", "openclaw", "hermes", "opencode"],
        )
        .await?;
    }
    if args.summary {
        println!("{}", render_bundle_wakeup_summary(&snapshot));
    } else {
        println!("{wakeup}");
    }

    // E4.4: every wake invocation logs a depth-telemetry line so the
    // `recall-depth.ndjson` distribution captures wake calls too.
    let _ =
        crate::runtime::recall::telemetry::record(crate::runtime::recall::telemetry::RecordOpts {
            bundle_root: &args.output,
            session_id: session_id.as_deref(),
            query: "wake",
            depth: crate::runtime::recall::RecallDepth::Wake,
            records_returned: snapshot.context.records.len() + snapshot.working.records.len(),
            tokens_returned: crate::runtime::recall::telemetry::approx_tokens(wakeup.len()),
            latency_ms: recall_started.elapsed().as_millis() as u64,
            escalation_hint: None,
        });

    Ok(())
}

async fn update_hive_session_wake_timestamp(output: &Path, base_url: &str) -> anyhow::Result<()> {
    let Some(session) = read_bundle_runtime_config(output)?
        .and_then(|config| config.session)
        .filter(|value| !value.trim().is_empty())
    else {
        return Ok(());
    };

    let url = resolve_bundle_command_base_url(base_url, None);
    if url.is_empty() {
        return Ok(());
    }

    let client = MemdClient::new(&url)?;
    let mut request = memd_schema::HiveSessionUpsertRequest::default();
    request.session = session.trim().to_string();
    request.last_wake_at = Some(chrono::Utc::now());

    let _ = client
        .upsert_hive_session(&request)
        .await
        .context("update hive session wake timestamp");
    Ok(())
}
