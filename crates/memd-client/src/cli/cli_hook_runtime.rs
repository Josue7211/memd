use super::*;
use crate::append_raw_spine_record;
use memd_core::file_ledger::{
    FileInteractionLedger, append_file_interaction, ledger_path as file_ledger_path,
    restore as file_ledger_restore, seal_session_ledger,
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
            // C4.5: when --kind correction, append to corrections.ndjson with
            // provenance before continuing the standard checkpoint/promote flow.
            if matches!(args.kind.as_deref(), Some("correction")) {
                let cap = correction_capture_args_from_hook(&args, content.clone());
                run_correction_capture(&cap)
                    .context("hook capture --kind correction: append corrections.ndjson")?;
            }
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
                    maybe_emit_hook_trace(
                        &args.output,
                        memd_core::hook_runtime::HookEvent::LedgerSeal,
                        &args.session_id,
                        |record| {
                            record.sealed_path = Some(sealed.display().to_string());
                            record.ok = Some(true);
                        },
                    );
                    println!("{}", sealed.display());
                }
                Err(err) if err.kind() == io::ErrorKind::NotFound => {
                    // No ledger to seal — treat as no-op for idempotent precompact hook.
                    maybe_emit_hook_trace(
                        &args.output,
                        memd_core::hook_runtime::HookEvent::LedgerSeal,
                        &args.session_id,
                        |record| {
                            record.ok = Some(false);
                            record.failure_class = memd_core::hook_runtime::FailureClass::None;
                        },
                    );
                }
                Err(err) => return Err(anyhow::Error::from(err)),
            }
        }
        HookMode::Restore(args) => {
            let output = args.output.clone();
            let session_id = args.session_id.clone();
            let report = run_hook_restore(&args)?;
            maybe_emit_hook_trace(
                &output,
                memd_core::hook_runtime::HookEvent::LedgerRestore,
                &session_id,
                |record| {
                    record.restored_path = Some(report.restored_path.display().to_string());
                    record.entries = Some(report.entries as u64);
                    record.ok = Some(report.ok);
                    if !report.ok {
                        record.failure_class = memd_core::hook_runtime::FailureClass::InnerNonzero;
                    }
                },
            );
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
        HookMode::Doctor(doctor_args) => match doctor_args.check {
            Some(HookDoctorCheck::Ordering) => {
                run_hook_doctor_ordering(&doctor_args)?;
            }
            Some(HookDoctorCheck::Contract) => {
                run_hook_doctor_contract(&doctor_args)?;
            }
            None => {
                run_hook_doctor(&doctor_args)?;
            }
        },
        HookMode::Enforce(enforce_args) => {
            let code = super::cli_hook_enforce::run_hook_enforce(&enforce_args)?;
            if code != 0 {
                return Err(anyhow::Error::new(
                    super::cli_hook_enforce::HookEnforceExitCode(code),
                ));
            }
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
        LedgerRestoreReport, RestoreSource, locate_latest_sealed, restore_ledger,
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

/// A4.5: audit a hook trace against `docs/contracts/hook-handoff.md` ordering
/// contract. Currently inspects three rules:
/// 1. PostCompact must be followed by a PostCompact restore event (the CLI
///    emits it via `memd hook restore`) before any file-op tool fires.
/// 2. No PreToolUse event for Read/Edit/Write may precede a pending
///    PostCompact restore (tool-before-restore).
/// 3. A trace with a PostCompact event but no matching restore event is a
///    missing-restore breach.
///
/// Trace events: `{"event":"PostCompact"|"LedgerRestore"|"PreToolUse","ts_ms":N,"tool":"Read"}`.
/// B4.6: append a universal trace line for inner-command emissions
/// (LedgerSeal, LedgerRestore) when `MEMD_HOOK_ENFORCE=1`. Silent no-op
/// otherwise — matches contract §5 flag semantics.
pub(crate) fn maybe_emit_hook_trace(
    bundle_root: &std::path::Path,
    event: memd_core::hook_runtime::HookEvent,
    session_id: &str,
    customize: impl FnOnce(&mut memd_core::hook_runtime::HookRecord),
) {
    if std::env::var("MEMD_HOOK_ENFORCE").unwrap_or_else(|_| "0".to_string()) == "0" {
        return;
    }
    let trace_path = std::env::var("MEMD_HOOK_TRACE_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| bundle_root.join("logs").join("hook-trace.ndjson"));
    let trace = memd_core::hook_runtime::HookTrace::new(trace_path);
    let mut record = memd_core::hook_runtime::HookRecord::new(event, session_id);
    customize(&mut record);
    let _ = trace.append(&record);
}

/// B4.8 — contract check. Audits the hook trace against
/// `docs/contracts/hook-order.md`:
///
/// - Every trace line must parse as a `HookRecord` with a contract-valid
///   event token.
/// - Any line with `failure_class=timeout` is surfaced as a halt-class
///   violation (test 19).
/// - Any line with `failure_class=inner-nonzero` on a log-class default
///   event is flagged as a silent swallow (test 20).
/// - MANIFEST.json must cover every contract-required event (test 21).
pub(crate) fn run_hook_doctor_contract(args: &HookDoctorArgs) -> anyhow::Result<()> {
    use memd_core::hook_runtime::{FailureClass, HookEvent, HookRecord};

    let root = args
        .project_root
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let manifest_path = root.join(".memd").join("hooks").join("MANIFEST.json");

    let trace_path = args
        .trace
        .clone()
        .unwrap_or_else(|| args.output.join("logs").join("hook-trace.ndjson"));

    let lines: Vec<String> = match &args.trace_inline {
        Some(inline) => inline.lines().map(str::to_string).collect(),
        None => fs::read_to_string(&trace_path)
            .with_context(|| format!("read {}", trace_path.display()))?
            .lines()
            .map(str::to_string)
            .collect(),
    };

    let mut timeouts: Vec<HookRecord> = Vec::new();
    let mut silent_swallows: Vec<HookRecord> = Vec::new();
    let mut parse_errors: Vec<String> = Vec::new();

    for line in &lines {
        if line.trim().is_empty() {
            continue;
        }
        let record: HookRecord = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                parse_errors.push(format!(
                    "{e}: {}",
                    line.chars().take(120).collect::<String>()
                ));
                continue;
            }
        };
        // HookEvent parsing is enforced by serde; unknown tokens already
        // surfaced as parse_errors above.
        match record.failure_class {
            FailureClass::Timeout => timeouts.push(record.clone()),
            FailureClass::InnerNonzero
                if matches!(record.event.failure_class_default(), FailureClass::Log) =>
            {
                silent_swallows.push(record.clone());
            }
            _ => {}
        }
    }

    // Manifest coverage of required events (contract §1 + §2).
    let required_events: &[HookEvent] = &[
        HookEvent::SessionStart,
        HookEvent::UserPromptSubmit,
        HookEvent::PreCompact,
        HookEvent::PostCompact,
        HookEvent::Stop,
    ];
    let mut manifest_missing: Vec<&'static str> = Vec::new();
    let manifest_value: Option<serde_json::Value> = fs::read_to_string(&manifest_path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok());
    let manifest_events: std::collections::HashSet<String> = manifest_value
        .as_ref()
        .and_then(|v| v.get("hooks"))
        .and_then(|v| v.as_array())
        .map(|entries| {
            entries
                .iter()
                .filter_map(|e| e.get("event").and_then(|s| s.as_str()))
                .filter(|s| *s != "none")
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    for req in required_events {
        if !manifest_events.contains(req.as_str()) {
            manifest_missing.push(req.as_str());
        }
    }

    let ok = timeouts.is_empty()
        && silent_swallows.is_empty()
        && parse_errors.is_empty()
        && manifest_missing.is_empty();

    if args.json {
        print_json(&serde_json::json!({
            "check": "contract",
            "trace_path": trace_path,
            "manifest_path": manifest_path,
            "lines_scanned": lines.len(),
            "timeouts": timeouts.iter().map(|r| json!({
                "event": r.event.as_str(),
                "session_id": r.session_id,
                "elapsed_ms": r.elapsed_ms,
                "budget_ms": r.budget_ms,
            })).collect::<Vec<_>>(),
            "silent_swallows": silent_swallows.iter().map(|r| json!({
                "event": r.event.as_str(),
                "session_id": r.session_id,
                "exit_code": r.exit_code,
            })).collect::<Vec<_>>(),
            "parse_errors": parse_errors,
            "manifest_missing": manifest_missing,
            "ok": ok,
        }))?;
    } else if ok {
        println!(
            "hook doctor contract: green — {} trace lines, manifest covers all required events",
            lines.len()
        );
    } else {
        println!("hook doctor contract: RED");
        for r in &timeouts {
            println!(
                "  timeout: event={} session={} elapsed_ms={:?} budget_ms={:?}",
                r.event.as_str(),
                r.session_id,
                r.elapsed_ms,
                r.budget_ms,
            );
        }
        for r in &silent_swallows {
            println!(
                "  silent swallow: event={} session={} exit_code={:?} (log-class default — inner failure was hidden)",
                r.event.as_str(),
                r.session_id,
                r.exit_code,
            );
        }
        for e in &parse_errors {
            println!("  parse error: {e}");
        }
        if !manifest_missing.is_empty() {
            println!(
                "  MANIFEST.json at {} missing required events:",
                manifest_path.display()
            );
            for ev in &manifest_missing {
                println!("    - {ev}");
            }
        }
    }

    if !ok {
        anyhow::bail!("hook doctor contract: violations detected");
    }
    Ok(())
}

pub(crate) fn run_hook_doctor_ordering(args: &HookDoctorArgs) -> anyhow::Result<()> {
    use memd_core::file_ledger::restore::{BreachKind, append_breach_line};

    let events = load_trace_events(args)?;
    let breaches = detect_ordering_breaches(&events);

    // Mirror any detected breach into the continuity-breach log so the
    // observability channel sees doctor-detected breaches too.
    for breach in &breaches {
        let _ = append_breach_line(
            &args.output,
            breach.session_id.as_deref().unwrap_or("unknown"),
            breach.kind,
            &breach.extras(),
        );
    }

    if args.json {
        print_json(&serde_json::json!({
            "check": "ordering",
            "events": events.len(),
            "breaches": breaches.iter().map(|b| b.to_json()).collect::<Vec<_>>(),
            "ok": breaches.is_empty(),
        }))?;
    } else if breaches.is_empty() {
        println!(
            "hook doctor ordering: green — {} events, zero breaches",
            events.len()
        );
    } else {
        println!(
            "hook doctor ordering: RED — {} breach(es) across {} events",
            breaches.len(),
            events.len()
        );
        for breach in &breaches {
            println!("  - {}: {}", breach.kind.as_str(), breach.explain());
        }
    }

    if !breaches.is_empty() {
        anyhow::bail!("hook doctor ordering: {} breach(es)", breaches.len());
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct TraceEvent {
    pub(crate) event: String,
    pub(crate) ts_ms: Option<i64>,
    pub(crate) tool: Option<String>,
    pub(crate) path: Option<String>,
    pub(crate) session_id: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct OrderingBreach {
    pub(crate) kind: memd_core::file_ledger::restore::BreachKind,
    pub(crate) session_id: Option<String>,
    pub(crate) tool: Option<String>,
    pub(crate) path: Option<String>,
}

impl OrderingBreach {
    fn extras(&self) -> Vec<(&str, &str)> {
        let mut out: Vec<(&str, &str)> = Vec::new();
        if let Some(t) = self.tool.as_deref() {
            out.push(("tool", t));
        }
        if let Some(p) = self.path.as_deref() {
            out.push(("path", p));
        }
        out
    }

    fn explain(&self) -> String {
        let tool = self.tool.as_deref().unwrap_or("?");
        let path = self.path.as_deref().unwrap_or("?");
        match self.kind {
            memd_core::file_ledger::restore::BreachKind::ToolBeforeRestore => {
                format!("tool {tool} on {path} fired before PostCompact restore")
            }
            memd_core::file_ledger::restore::BreachKind::MissingRestore => {
                "PostCompact event without matching LedgerRestore".to_string()
            }
            memd_core::file_ledger::restore::BreachKind::NoSealedLedger => {
                "no sealed ledger recorded in trace".to_string()
            }
        }
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "kind": self.kind.as_str(),
            "session_id": self.session_id,
            "tool": self.tool,
            "path": self.path,
        })
    }
}

pub(crate) fn detect_ordering_breaches(events: &[TraceEvent]) -> Vec<OrderingBreach> {
    let mut breaches = Vec::new();
    // Track "we saw PostCompact and haven't seen a restore yet for this session".
    let mut pending_restore: Option<Option<String>> = None;
    // Session ID from the most recent PostCompact, for breach attribution.
    let mut last_session: Option<String> = None;

    for ev in events {
        match ev.event.as_str() {
            "PostCompact" => {
                pending_restore = Some(ev.session_id.clone());
                last_session = ev.session_id.clone();
            }
            "LedgerRestore" | "HookRestore" => {
                pending_restore = None;
            }
            "PreToolUse" => {
                if pending_restore.is_some() {
                    let tool_name = ev.tool.as_deref().unwrap_or("");
                    if matches!(tool_name, "Read" | "Edit" | "Write" | "NotebookEdit") {
                        breaches.push(OrderingBreach {
                            kind: memd_core::file_ledger::restore::BreachKind::ToolBeforeRestore,
                            session_id: ev.session_id.clone().or_else(|| last_session.clone()),
                            tool: ev.tool.clone(),
                            path: ev.path.clone(),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(sid) = pending_restore {
        breaches.push(OrderingBreach {
            kind: memd_core::file_ledger::restore::BreachKind::MissingRestore,
            session_id: sid,
            tool: None,
            path: None,
        });
    }

    breaches
}

fn load_trace_events(args: &HookDoctorArgs) -> anyhow::Result<Vec<TraceEvent>> {
    if let Some(raw) = args.trace_inline.as_deref() {
        return parse_trace_payload(raw);
    }
    let path = args
        .trace
        .clone()
        .unwrap_or_else(|| args.output.join("logs").join("hook-trace.ndjson"));
    if !path.exists() {
        anyhow::bail!("trace-unavailable: no trace at {}", path.display());
    }
    let bytes = std::fs::read_to_string(&path)
        .with_context(|| format!("read hook trace {}", path.display()))?;
    parse_trace_payload(&bytes)
}

fn parse_trace_payload(raw: &str) -> anyhow::Result<Vec<TraceEvent>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    // Array form: `[{...}, {...}]`.
    if trimmed.starts_with('[') {
        let values: Vec<serde_json::Value> =
            serde_json::from_str(trimmed).context("parse inline trace array as JSON")?;
        return Ok(values.into_iter().map(value_to_trace_event).collect());
    }
    // NDJSON form: one event per line.
    let mut out = Vec::new();
    for (idx, line) in trimmed.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(line)
            .with_context(|| format!("parse trace line {} as JSON", idx + 1))?;
        out.push(value_to_trace_event(value));
    }
    Ok(out)
}

fn value_to_trace_event(value: serde_json::Value) -> TraceEvent {
    let get_str = |k: &str| value.get(k).and_then(|v| v.as_str()).map(str::to_string);
    TraceEvent {
        event: get_str("event").unwrap_or_default(),
        ts_ms: value.get("ts_ms").and_then(|v| v.as_i64()),
        tool: get_str("tool"),
        path: get_str("path"),
        session_id: get_str("session_id"),
    }
}

/// C4.5: shape a CorrectionCaptureArgs from a HookCaptureArgs payload.
/// captured_by depends on the auto-detect flag so dogfood captures look
/// distinct from CLI-driven manual captures in the audit log.
pub(crate) fn correction_capture_args_from_hook(
    args: &HookCaptureArgs,
    content: String,
) -> CorrectionCaptureArgs {
    let captured_by = if std::env::var("MEMD_C4_CORRECTION_DETECT").ok().as_deref() == Some("1") {
        "hook_auto"
    } else {
        "manual"
    };
    CorrectionCaptureArgs {
        content,
        corrects_id: args.corrects_id.clone(),
        source_turn: args.source_turn.clone(),
        confidence: args.confidence.unwrap_or(0.85),
        captured_by: captured_by.to_string(),
        session_id: None,
        output: args.output.clone(),
    }
}

#[cfg(test)]
mod c4_hook_tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn hook_args(tmp: &PathBuf) -> HookCaptureArgs {
        HookCaptureArgs {
            output: tmp.clone(),
            project: None,
            namespace: None,
            workspace: None,
            visibility: None,
            source_path: None,
            confidence: Some(0.92),
            ttl_seconds: None,
            content: None,
            input: None,
            stdin: false,
            tag: vec![],
            promote_kind: None,
            promote_scope: None,
            promote_supersede: vec![],
            promote_supersede_query: None,
            promote_tag: vec![],
            promote_confidence: None,
            summary: false,
            kind: Some("correction".into()),
            corrects_id: Some("rec-prior".into()),
            source_turn: Some("t-12".into()),
        }
    }

    use std::sync::Mutex;
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn hook_capture_with_kind_correction_routes_through_detector() {
        let _g = ENV_LOCK.lock().unwrap();
        // SAFETY: serialized via ENV_LOCK.
        unsafe { std::env::remove_var("MEMD_C4_CORRECTION_DETECT") };
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();
        let args = hook_args(&path);
        let cap = correction_capture_args_from_hook(&args, "no, beta is the host".into());
        assert_eq!(cap.captured_by, "manual");
        assert_eq!(cap.confidence, 0.92);
        assert_eq!(cap.corrects_id.as_deref(), Some("rec-prior"));
        assert_eq!(cap.source_turn.as_deref(), Some("t-12"));

        run_correction_capture(&cap).unwrap();
        let log = path.join("logs").join("corrections.ndjson");
        let contents = std::fs::read_to_string(log).unwrap();
        assert!(contents.contains("\"corrects_id\":\"rec-prior\""));
        assert!(contents.contains("\"captured_by\":\"manual\""));
    }

    #[test]
    fn hook_capture_kind_correction_marks_hook_auto_when_flag_on() {
        let _g = ENV_LOCK.lock().unwrap();
        // SAFETY: serialized via ENV_LOCK.
        unsafe { std::env::set_var("MEMD_C4_CORRECTION_DETECT", "1") };
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();
        let args = hook_args(&path);
        let cap = correction_capture_args_from_hook(&args, "scratch that, host is beta".into());
        assert_eq!(cap.captured_by, "hook_auto");
        unsafe { std::env::remove_var("MEMD_C4_CORRECTION_DETECT") };
    }
}
