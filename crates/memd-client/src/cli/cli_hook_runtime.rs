use super::*;
use crate::append_raw_spine_record;
use memd_core::file_ledger::{
    append_file_interaction, ledger_path as file_ledger_path, restore as file_ledger_restore,
    seal_session_ledger, FileInteractionLedger,
};

/// Sentinel error: `memd hook restore` found no sealed ledger. `main.rs`
/// downcasts this and returns exit code 2 — non-fatal for the caller (the
/// PostCompact hook) but observable.
#[derive(Debug)]
pub(crate) struct HookRestoreNoSealed {
    pub(crate) session_id: String,
}

impl std::fmt::Display for HookRestoreNoSealed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "memd hook restore: no sealed ledger for session {}",
            self.session_id
        )
    }
}

impl std::error::Error for HookRestoreNoSealed {}

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
        HookMode::Restore(args) => {
            let report = run_hook_restore(&args)?;
            if !report.ok {
                // Non-fatal for the hook caller, but surface a distinct error
                // so `main.rs` can map to exit code 2.
                return Err(anyhow::Error::new(HookRestoreNoSealed {
                    session_id: args.session_id.clone(),
                }));
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

/// `memd hook restore` entry point.
///
/// Side effects (unless `args.dry_run`): copies newest sealed ledger to the
/// active `ledger_path`, then appends a single ndjson line to
/// `<output>/logs/ledger-restore.ndjson`. When no sealed ledger exists the
/// returned report has `ok=false` and `error=Some("no-sealed-ledger")`;
/// caller is responsible for exit-code mapping. Breach-log emission is owned
/// by `file_ledger::restore` and lands in Task A4.3.
pub(crate) fn run_hook_restore(
    args: &HookRestoreArgs,
) -> anyhow::Result<file_ledger_restore::LedgerRestoreReport> {
    use file_ledger_restore::{
        locate_latest_sealed, restore_ledger, LedgerRestoreReport, RestoreSource,
    };

    let source = RestoreSource::Postcompact;

    if args.dry_run {
        let sealed = locate_latest_sealed(&args.output, &args.session_id);
        let restored_path = file_ledger_path(&args.output, &args.session_id);
        let (entries, ok, error) = match sealed.as_ref() {
            Some(sp) => match FileInteractionLedger::load_from_path(sp) {
                Ok(l) => (l.entries.len(), true, None),
                Err(e) => (0, false, Some(e.to_string())),
            },
            None => (0, false, Some("no-sealed-ledger".to_string())),
        };
        let report = LedgerRestoreReport {
            session_id: args.session_id.clone(),
            sealed_path: sealed,
            restored_path,
            entries,
            source,
            ok,
            error,
        };
        emit_restore_report(&report, args.json)?;
        return Ok(report);
    }

    let report = restore_ledger(&args.session_id, &args.output, source)?;
    append_restore_ndjson(&args.output, &report)?;
    emit_restore_report(&report, args.json)?;
    Ok(report)
}

fn append_restore_ndjson(
    output: &Path,
    report: &file_ledger_restore::LedgerRestoreReport,
) -> io::Result<()> {
    use std::io::Write;

    let logs_dir = output.join("logs");
    fs::create_dir_all(&logs_dir)?;
    let path = logs_dir.join("ledger-restore.ndjson");
    let ts_ms = chrono::Utc::now().timestamp_millis();
    let source_tag = match report.source {
        file_ledger_restore::RestoreSource::Postcompact => "postcompact-hook",
        file_ledger_restore::RestoreSource::Manual => "manual",
        file_ledger_restore::RestoreSource::Test => "test",
    };
    let line = serde_json::json!({
        "ts_ms": ts_ms,
        "session_id": report.session_id,
        "sealed_path": report.sealed_path,
        "restored_path": report.restored_path,
        "entries": report.entries,
        "source": source_tag,
        "ok": report.ok,
        "error": report.error,
    });
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    writeln!(file, "{line}")?;
    Ok(())
}

fn emit_restore_report(
    report: &file_ledger_restore::LedgerRestoreReport,
    json: bool,
) -> anyhow::Result<()> {
    if json {
        print_json(report)?;
    } else if report.ok {
        println!(
            "restored {} entries from {} → {}",
            report.entries,
            report
                .sealed_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<none>".to_string()),
            report.restored_path.display(),
        );
    } else {
        println!(
            "no sealed ledger for session {} ({})",
            report.session_id,
            report.error.as_deref().unwrap_or("unknown"),
        );
    }
    Ok(())
}
