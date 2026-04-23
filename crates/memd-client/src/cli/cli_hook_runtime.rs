use super::*;
use crate::append_raw_spine_record;
use memd_core::file_ledger::{append_file_interaction, seal_session_ledger};

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
            let output = args.output.clone();
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
                    auto_commit: false,
                    roadmap_set: vec![],
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
                Ok(_) => match crate::runtime::read_bundle_resume(
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
            append_raw_spine_record(
                &output,
                "hook_capture",
                "candidate",
                args.project.as_deref(),
                args.namespace.as_deref(),
                args.workspace.as_deref(),
                Some("hook-capture"),
                args.source_path.as_deref().or(Some("hook-capture")),
                args.confidence,
                &args.tag,
                &content,
            )?;
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
        HookMode::FileInteraction(args) => {
            let payload = if let Some(content) = &args.content {
                content.clone()
            } else if args.stdin {
                let mut buf = String::new();
                io::stdin()
                    .read_to_string(&mut buf)
                    .context("read file-interaction payload from stdin")?;
                buf
            } else {
                return Ok(());
            };
            let payload_trim = payload.trim();
            if payload_trim.is_empty() {
                return Ok(());
            }
            let value: serde_json::Value = serde_json::from_str(payload_trim)
                .context("parse file-interaction payload as JSON")?;
            let now_ms = chrono::Utc::now().timestamp_millis();
            append_file_interaction(&value, args.session_id.as_deref(), &args.output, now_ms)
                .context("append file-interaction ledger")?;
        }
        HookMode::SealLedger(args) => {
            match seal_session_ledger(&args.session_id, &args.output) {
                Ok(sealed) => {
                    println!("{}", sealed.display());
                }
                Err(err) if err.kind() == io::ErrorKind::NotFound => {
                    // No ledger to seal — treat as no-op for idempotent precompact hook.
                }
                Err(err) => return Err(anyhow::Error::from(err)),
            }
        }
        HookMode::Spill(args) => {
            let output = resolve_default_bundle_root()?
                .unwrap_or_else(crate::bundle::default_bundle_root_path);
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
                append_raw_spine_record(
                    &output,
                    "hook_spill",
                    "candidate",
                    packet.session.project.as_deref(),
                    None,
                    None,
                    Some("hook-spill"),
                    Some("compaction-packet"),
                    None,
                    &[String::from("hook-spill")],
                    &serde_json::to_string(&spill)
                        .unwrap_or_else(|_| "compaction spill".to_string()),
                )?;
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
        HookMode::Gate(gate_args) => {
            cli_gate_runtime::run_gate_cli(&gate_args).await?;
        }
        HookMode::Doctor(doctor_args) => {
            run_hook_doctor(&doctor_args)?;
        }
    }

    Ok(())
}

pub(crate) fn run_hook_doctor(args: &HookDoctorArgs) -> anyhow::Result<()> {
    use sha2::{Digest, Sha256};
    use std::fs;

    let root = args
        .project_root
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let hooks_dir = root.join(".memd").join("hooks");
    let manifest_path = hooks_dir.join("MANIFEST.json");

    let manifest_raw = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw)
        .with_context(|| format!("parse {}", manifest_path.display()))?;

    let entries = manifest
        .get("hooks")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("MANIFEST.json missing `hooks` array"))?;

    let mut ok: Vec<String> = Vec::new();
    let mut mismatched: Vec<(String, String, String)> = Vec::new();
    let mut missing: Vec<String> = Vec::new();
    let mut unknown: Vec<String> = Vec::new();

    let mut known_paths: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

    for entry in entries {
        let name = entry
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("<unnamed>");
        let path = entry
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("manifest entry {} missing `path`", name))?;
        let expected = entry
            .get("sha256")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("manifest entry {} missing `sha256`", name))?;

        let abs = root.join(path);
        known_paths.insert(abs.clone());
        match fs::read(&abs) {
            Ok(bytes) => {
                let actual = format!("{:x}", Sha256::digest(&bytes));
                if actual == expected {
                    ok.push(name.to_string());
                } else {
                    mismatched.push((name.to_string(), expected.to_string(), actual));
                }
            }
            Err(_) => missing.push(name.to_string()),
        }
    }

    if let Ok(read_dir) = fs::read_dir(&hooks_dir) {
        for entry in read_dir.flatten() {
            let p = entry.path();
            if !p.is_file() {
                continue;
            }
            let is_script = p
                .extension()
                .and_then(|e| e.to_str())
                .map(|ext| matches!(ext, "sh" | "ps1"))
                .unwrap_or(false);
            if is_script && !known_paths.contains(&p) {
                unknown.push(
                    p.strip_prefix(&root)
                        .unwrap_or(&p)
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
    }

    let green = mismatched.is_empty() && missing.is_empty() && unknown.is_empty();

    if args.json {
        print_json(&json!({
            "ok": ok,
            "mismatched": mismatched.iter().map(|(n, e, a)| json!({"name": n, "expected": e, "actual": a})).collect::<Vec<_>>(),
            "missing": missing,
            "unknown": unknown,
            "green": green,
        }))?;
    } else if green {
        println!(
            "hooks doctor: green — {} hooks verified against MANIFEST.json",
            ok.len()
        );
    } else {
        println!("hooks doctor: RED");
        if !mismatched.is_empty() {
            println!("  sha256 mismatch:");
            for (n, expected, actual) in &mismatched {
                println!("    - {n}: expected {expected}, got {actual}");
            }
        }
        if !missing.is_empty() {
            println!("  missing:");
            for n in &missing {
                println!("    - {n}");
            }
        }
        if !unknown.is_empty() {
            println!("  untracked (not in MANIFEST.json):");
            for n in &unknown {
                println!("    - {n}");
            }
        }
    }

    if !green {
        anyhow::bail!("hooks doctor: manifest verification failed");
    }
    Ok(())
}
