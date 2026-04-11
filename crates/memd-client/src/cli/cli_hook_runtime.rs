use super::*;

pub(crate) async fn run_hook_mode(
    client: &MemdClient,
    base_url: &str,
    args: HookArgs,
) -> anyhow::Result<()> {
    match args.mode {
        HookMode::Context(args) => {
            let req = ContextRequest {
                project: args.project,
                agent: args.agent,
                workspace: None,
                visibility: None,
                route: parse_retrieval_route(args.route)?,
                intent: parse_retrieval_intent(args.intent.or(Some("current_task".to_string())))?,
                limit: args.limit,
                max_chars_per_item: args.max_chars_per_item,
            };
            print_json(&client.context_compact(&req).await?)?;
        }
        HookMode::Capture(args) => {
            let content = if let Some(content) = &args.content {
                content.clone()
            } else if let Some(path) = &args.input {
                fs::read_to_string(path)
                    .with_context(|| format!("read hook capture input file {}", path.display()))?
            } else if args.stdin {
                let mut content = String::new();
                io::stdin()
                    .read_to_string(&mut content)
                    .context("read hook capture payload from stdin")?;
                content
            } else {
                "hook capture: active task state changed".to_string()
            };
            let effective_promote_kind = effective_hook_capture_promote_kind(&args, &content);
            let (supersede_targets, supersede_diagnostics) =
                find_hook_capture_supersede_targets(base_url, &args, &content).await?;
            let promote_response = if let Some(promote_kind) = effective_promote_kind {
                Some(
                    remember_with_bundle_defaults(
                        &remember_args_from_effective_hook_capture(
                            &args,
                            content.clone(),
                            promote_kind,
                            supersede_targets.clone(),
                        ),
                        base_url,
                    )
                    .await?,
                )
            } else {
                None
            };
            let supersede_responses = if let Some(response) = promote_response.as_ref() {
                mark_hook_capture_supersede_targets(
                    base_url,
                    &args,
                    &supersede_targets,
                    response.item.id,
                )
                .await?
            } else {
                Vec::new()
            };
            let checkpoint = checkpoint_with_bundle_defaults(
                &CheckpointArgs {
                    output: args.output.clone(),
                    project: args.project.clone(),
                    namespace: args.namespace.clone(),
                    workspace: args.workspace.clone(),
                    visibility: args.visibility.clone(),
                    source_path: args
                        .source_path
                        .clone()
                        .or(Some("hook-capture".to_string())),
                    confidence: args.confidence,
                    ttl_seconds: args.ttl_seconds.or(Some(86_400)),
                    tag: if args.tag.is_empty() {
                        vec![
                            "hook-capture".to_string(),
                            "episodic".to_string(),
                            "live-memory".to_string(),
                        ]
                    } else {
                        args.tag.clone()
                    },
                    content: Some(content.clone()),
                    input: None,
                    stdin: false,
                },
                base_url,
            )
            .await;
            let checkpoint_id = checkpoint
                .as_ref()
                .map(|response| response.item.id.to_string())
                .unwrap_or_else(|_| "none".to_string());
            let checkpoint_json = checkpoint
                .as_ref()
                .map(|response| json!(response))
                .unwrap_or_else(|err| json!({ "error": err.to_string() }));
            let snapshot = match checkpoint {
                Ok(_) => match read_bundle_resume(
                    &ResumeArgs {
                        output: args.output.clone(),
                        project: args.project.clone(),
                        namespace: args.namespace.clone(),
                        agent: None,
                        workspace: args.workspace.clone(),
                        visibility: args.visibility.clone(),
                        route: None,
                        intent: Some("current_task".to_string()),
                        limit: Some(8),
                        rehydration_limit: Some(4),
                        semantic: false,
                        prompt: false,
                        summary: false,
                    },
                    base_url,
                )
                .await
                {
                    Ok(snapshot) => Some(snapshot),
                    Err(_) => {
                        preserve_codex_capture_locally(&args.output, &content)?;
                        None
                    }
                },
                Err(_) => {
                    preserve_codex_capture_locally(&args.output, &content)?;
                    None
                }
            };
            if let Some(snapshot) = snapshot.as_ref() {
                write_bundle_memory_files(&args.output, snapshot, None, false).await?;
                refresh_live_bundle_event_pages(&args.output, snapshot, None)?;
                auto_checkpoint_live_snapshot(&args.output, base_url, snapshot, "hook-capture")
                    .await?;
                let _ = refresh_harness_pack_files_for_snapshot(
                    &args.output,
                    snapshot,
                    "hook-capture",
                    &["codex", "agent-zero", "openclaw"],
                )
                .await?;
            }
            if args.summary {
                let (supersede_query, supersede_tried, supersede_hits) =
                    summarize_hook_capture_supersede_diagnostics(&supersede_diagnostics);
                println!(
                    "hook_capture stored={} promoted={} superseded={} query={} tried={} hits={} working={} inbox={}",
                    checkpoint_id,
                    promote_response
                        .as_ref()
                        .map(|response| response.item.id.to_string())
                        .unwrap_or_else(|| "none".to_string()),
                    supersede_responses.len(),
                    supersede_query,
                    supersede_tried,
                    supersede_hits,
                    snapshot
                        .as_ref()
                        .map(|value| value.working.records.len())
                        .unwrap_or(0),
                    snapshot
                        .as_ref()
                        .map(|value| value.inbox.items.len())
                        .unwrap_or(0)
                );
            } else {
                print_json(&json!({
                    "live": checkpoint_json,
                    "promoted": promote_response,
                    "superseded": supersede_responses,
                    "supersede_search": supersede_diagnostics,
                }))?;
            }
        }
        HookMode::Spill(args) => {
            let packet = read_request::<CompactionPacket>(&args.input)?;
            let spill = if args.spill_transient {
                derive_compaction_spill_with_options(
                    &packet,
                    CompactionSpillOptions {
                        include_transient_state: true,
                    },
                )
            } else {
                derive_compaction_spill(&packet)
            };

            if args.apply {
                let responses = client.candidate_batch(&spill.items).await?;
                let duplicates = responses
                    .iter()
                    .filter(|response| response.duplicate_of.is_some())
                    .count();
                if let Some(rag) = maybe_rag_client_from_bundle_or_env()? {
                    sync_candidate_responses_to_rag(&rag, &responses).await?;
                }
                auto_checkpoint_compaction_packet(&packet, base_url).await?;
                let submitted = responses.len();
                print_json(&CompactionSpillResult {
                    submitted,
                    duplicates,
                    responses,
                    batch: spill,
                })?;
            } else {
                print_json(&spill)?;
            }
        }
    }

    Ok(())
}
