use super::*;

pub(crate) mod depth;
pub(crate) mod escalation;
pub(crate) mod telemetry;

pub(crate) use depth::{RecallDepth, depth_flag_enabled, escalation_hint_enabled};

use std::time::Instant;

/// Hard cap on records returned at `--depth lookup`, per
/// `docs/contracts/recall-depth.md` ("1–3 records").
pub(crate) const LOOKUP_DEPTH_RECORD_CAP: usize = 3;

pub(crate) async fn dispatch_lookup_with_depth(
    client: &MemdClient,
    base_url: &str,
    args: LookupArgs,
) -> anyhow::Result<()> {
    if !depth_flag_enabled() && args.depth != RecallDepth::Lookup {
        anyhow::bail!("--depth flag is disabled (set MEMD_E4_DEPTH_FLAG=1 to enable)");
    }

    let bundle_root = args.output.clone();
    let session_id = read_bundle_runtime_config(&bundle_root)
        .ok()
        .flatten()
        .and_then(|c| c.session);
    let query = args.query.clone();
    let depth = args.depth;
    if args.explain_depth {
        eprintln!("{}", depth::explain_line(depth));
    }
    let started = Instant::now();

    let (result, records, tokens, hint) = match depth {
        RecallDepth::Wake => {
            let res = run_wake_arm(&args, base_url).await;
            (res, 0_usize, 0_usize, None)
        }
        RecallDepth::Lookup => {
            let outcome = run_lookup_arm_inner(client, base_url, args).await;
            match outcome {
                Ok(out) => {
                    let hint = out.escalation_hint.clone();
                    if let Some(h) = hint.as_deref() {
                        eprintln!("{h}");
                    }
                    let render_result: anyhow::Result<()> = if out.json {
                        crate::print_json(&out.response)
                    } else {
                        println!("{}", out.markdown);
                        Ok(())
                    };
                    let records = out.response.items.len();
                    let tokens = telemetry::approx_tokens(out.markdown.len());
                    (render_result, records, tokens, hint)
                }
                Err(err) => (Err(err), 0, 0, None),
            }
        }
        RecallDepth::Resume => {
            let res = run_resume_arm(&args, base_url).await;
            (res, 0, 0, None)
        }
    };

    // Wake-arm telemetry is emitted by `run_bundle_wake_command` itself so
    // every wake call (CLI or dispatched) appears exactly once in
    // `recall-depth.ndjson`, per docs/contracts/recall-depth.md.
    if !matches!(depth, RecallDepth::Wake) {
        let _ = telemetry::record(telemetry::RecordOpts {
            bundle_root: &bundle_root,
            session_id: session_id.as_deref(),
            query: &query,
            depth,
            records_returned: records,
            tokens_returned: tokens,
            latency_ms: started.elapsed().as_millis() as u64,
            escalation_hint: hint.as_deref(),
        });
    }

    result
}

async fn run_wake_arm(args: &LookupArgs, base_url: &str) -> anyhow::Result<()> {
    let wake_args = synth_wake_args(args);
    crate::run_bundle_wake_command(&wake_args, base_url).await
}

async fn run_resume_arm(args: &LookupArgs, base_url: &str) -> anyhow::Result<()> {
    let resume_args = synth_resume_args(args);
    let snapshot = read_bundle_resume(&resume_args, base_url).await?;
    crate::print_json(&snapshot)
}

pub(crate) struct LookupArmOutcome {
    pub(crate) response: memd_schema::SearchMemoryResponse,
    pub(crate) markdown: String,
    pub(crate) json: bool,
    pub(crate) escalation_hint: Option<String>,
}

pub(crate) async fn run_lookup_arm_inner(
    client: &MemdClient,
    base_url: &str,
    args: LookupArgs,
) -> anyhow::Result<LookupArmOutcome> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let mut args = crate::cli::apply_lookup_bundle_defaults(args, runtime.as_ref());
    args.limit = Some(clamp_lookup_limit(args.limit));
    let req = build_lookup_request(&args, runtime.as_ref())?;
    let mut response = match tokio::time::timeout(
        lookup_remote_timeout(),
        lookup_with_fallbacks(client, &req, &args.query),
    )
    .await
    {
        Ok(Ok(response)) => response,
        Ok(Err(err)) => {
            let local = lookup_local_continuity_fallback(&args.output, &args.query, &req);
            if local.items.is_empty() {
                return Err(err);
            }
            local
        }
        Err(_) => lookup_local_continuity_fallback(&args.output, &args.query, &req),
    };
    if response.items.is_empty() {
        response = match tokio::time::timeout(
            lookup_remote_timeout(),
            lookup_resume_snapshot_fallback(base_url, &args, &req),
        )
        .await
        {
            Ok(Ok(response)) => response,
            Ok(Err(err)) => {
                let local = lookup_local_continuity_fallback(&args.output, &args.query, &req);
                if local.items.is_empty() {
                    return Err(err);
                }
                local
            }
            Err(_) => lookup_local_continuity_fallback(&args.output, &args.query, &req),
        };
    }
    response = overlay_wake_current_handoff(&args.output, &args.query, &req, response);
    let selective_expansion_mode = escalation::selective_expansion_mode(&args.query);
    let escalation_hint = if escalation_hint_enabled() {
        escalation::ceo_mode_hint_line(&args.query, selective_expansion_mode).or_else(|| {
            (response.items.is_empty() && escalation::detect(&args.query))
                .then(|| escalation::hint_line(&args.query))
        })
    } else {
        None
    };
    let mut markdown = render_lookup_markdown(&args.query, &req, &response, args.verbose);
    if let Some(guidance) = escalation::ceo_mode_guidance_markdown(selective_expansion_mode) {
        markdown.push('\n');
        markdown.push_str(&guidance);
    }
    Ok(LookupArmOutcome {
        response,
        markdown,
        json: args.json,
        escalation_hint,
    })
}

fn lookup_remote_timeout() -> std::time::Duration {
    let millis = std::env::var("MEMD_LOOKUP_REMOTE_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value >= 100)
        .unwrap_or(5_000);
    std::time::Duration::from_millis(millis)
}

fn lookup_live_map_ttl_secs() -> i64 {
    std::env::var("MEMD_CODEBASE_LIVE_MAP_TTL_SECS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value >= 0)
        .unwrap_or(15)
}

fn lookup_host_io_report_ttl_secs() -> i64 {
    std::env::var("MEMD_HOST_IO_REPORT_TTL_SECS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value >= 0)
        .unwrap_or(120)
}

fn lookup_timestamp_freshness(ts: Option<&str>, ttl_secs: i64) -> (String, i64, String) {
    let Some(ts) = ts else {
        return ("unknown".to_string(), ttl_secs, "unknown".to_string());
    };
    let Some(ts) = chrono::DateTime::parse_from_rfc3339(ts)
        .ok()
        .map(|value| value.with_timezone(&chrono::Utc))
    else {
        return ("unknown".to_string(), ttl_secs, "unknown".to_string());
    };
    let age_secs = chrono::Utc::now()
        .signed_duration_since(ts)
        .num_seconds()
        .max(0);
    let fresh = if age_secs <= ttl_secs {
        "true"
    } else {
        "false"
    };
    (age_secs.to_string(), ttl_secs, fresh.to_string())
}

fn lookup_live_map_freshness(updated_at: Option<&str>) -> (String, i64, String) {
    lookup_timestamp_freshness(updated_at, lookup_live_map_ttl_secs())
}

fn lookup_host_io_report_freshness(ts: Option<&str>) -> (String, i64, String) {
    lookup_timestamp_freshness(ts, lookup_host_io_report_ttl_secs())
}

fn lookup_live_map_action(status: &str, needs_reread: bool, fresh: &str) -> &'static str {
    if fresh == "false" {
        "refresh_host_guard_before_trusting_live_map"
    } else if status == "blocked" || needs_reread {
        "wait_or_coordinate_before_broad_repo_work"
    } else if status == "unknown" {
        "missing_live_map_run_host_guard_or_awareness"
    } else {
        "live_map_current"
    }
}

fn lookup_host_io_report_action(status: &str, fresh: &str) -> &'static str {
    if fresh == "false" {
        "refresh_host_guard_before_trusting_host_report"
    } else if status == "blocked" {
        "wait_or_coordinate_before_broad_repo_work"
    } else if status == "clear" {
        "refresh_codebase_live_map_before_broad_repo_work"
    } else {
        "refresh_host_guard_before_trusting_host_report"
    }
}

fn host_io_report_timestamp(raw: &str) -> Option<&str> {
    raw.lines().find_map(|line| line.strip_prefix("ts="))
}

fn host_io_report_status(raw: &str) -> &str {
    raw.lines()
        .find_map(|line| line.strip_prefix("status="))
        .unwrap_or("unknown")
}

fn host_io_report_blocker_sample(raw: &str) -> String {
    raw.lines()
        .filter(|line| {
            !line.starts_with("ts=")
                && !line.starts_with("repo=")
                && !line.starts_with("pid=")
                && !line.starts_with("status=")
                && !line.trim().is_empty()
        })
        .take(3)
        .collect::<Vec<_>>()
        .join(" | ")
}

fn lookup_local_continuity_fallback(
    output: &Path,
    query: &str,
    req: &memd_schema::SearchMemoryRequest,
) -> memd_schema::SearchMemoryResponse {
    let mut items = Vec::new();
    if let Some(item) = wake_current_handoff_item(output, req) {
        items.push(item);
    }
    if lookup_query_requests_local_continuity(query) {
        if let Some(item) = codebase_live_map_status_item(output, req) {
            items.push(item);
        } else if let Some(item) = host_io_guard_status_item(output, req) {
            items.push(item);
        }
    }
    items.truncate(req.limit.unwrap_or(LOOKUP_DEPTH_RECORD_CAP).max(1));
    memd_schema::SearchMemoryResponse {
        route: req
            .route
            .unwrap_or(memd_schema::RetrievalRoute::ProjectFirst),
        intent: req.intent.unwrap_or(memd_schema::RetrievalIntent::General),
        items,
        trace: None,
    }
}

fn lookup_query_requests_local_continuity(query: &str) -> bool {
    let terms = crate::runtime::lookup_query_terms(query);
    [
        "blocker",
        "blocked",
        "codebase",
        "collision",
        "continuity",
        "dirty",
        "handoff",
        "hive",
        "host",
        "live",
        "map",
        "reread",
        "state",
        "sync",
    ]
    .iter()
    .any(|needle| terms.iter().any(|term| term == needle))
        || query.to_ascii_lowercase().contains("t7")
}

fn codebase_live_map_status_item(
    output: &Path,
    req: &memd_schema::SearchMemoryRequest,
) -> Option<memd_schema::MemoryItem> {
    let path = output.join("state").join("codebase-live-map.json");
    let raw = fs::read_to_string(&path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let status = value
        .get("status")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    let autosync = value
        .get("autosync")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    let needs_reread = value
        .get("needs_reread")
        .and_then(|value| value.as_bool())
        .unwrap_or(true);
    let file_count = value
        .get("file_count")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    let updated_at = value.get("updated_at").and_then(|value| value.as_str());
    let (age_secs, ttl_secs, fresh) = lookup_live_map_freshness(updated_at);
    let action = lookup_live_map_action(status, needs_reread, &fresh);
    let blockers = value
        .get("blockers")
        .and_then(|value| value.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str())
                .take(3)
                .collect::<Vec<_>>()
                .join(" | ")
        })
        .unwrap_or_default();
    let changes = value
        .get("last_changes")
        .map(live_map_changes_summary)
        .unwrap_or_else(|| "unknown".to_string());
    let content = format!(
        "Status: codebase live map. status={status} autosync={autosync} reread_required={needs_reread} fresh={fresh} age_secs={age_secs} ttl_secs={ttl_secs} action={action} files={file_count} changes={changes} blockers={}",
        if blockers.is_empty() {
            "none"
        } else {
            blockers.as_str()
        }
    );
    Some(local_status_item(
        req,
        content,
        "codebase-live-map.json",
        path.display().to_string(),
        vec![
            "continuity".to_string(),
            "codebase-live-map".to_string(),
            "lookup-local-fallback".to_string(),
        ],
    ))
}

fn live_map_changes_summary(value: &serde_json::Value) -> String {
    let added = value
        .get("added_count")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    let modified = value
        .get("modified_count")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    let deleted = value
        .get("deleted_count")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    let baseline = value
        .get("baseline_available")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let sample = ["added", "modified", "deleted"]
        .iter()
        .filter_map(|key| {
            let values = value.get(key)?.as_array()?;
            if values.is_empty() {
                return None;
            }
            let sample = values
                .iter()
                .filter_map(|value| value.as_str())
                .take(3)
                .collect::<Vec<_>>()
                .join(",");
            Some(format!("{key}:[{sample}]"))
        })
        .collect::<Vec<_>>()
        .join(" ");
    if sample.is_empty() {
        format!("added:{added} modified:{modified} deleted:{deleted} baseline:{baseline}")
    } else {
        format!(
            "added:{added} modified:{modified} deleted:{deleted} baseline:{baseline} sample:{sample}"
        )
    }
}

fn host_io_guard_status_item(
    output: &Path,
    req: &memd_schema::SearchMemoryRequest,
) -> Option<memd_schema::MemoryItem> {
    let path = output.join("state").join("host-io-guard.txt");
    let raw = fs::read_to_string(&path).ok()?;
    let status = host_io_report_status(&raw);
    let (age_secs, ttl_secs, fresh) =
        lookup_host_io_report_freshness(host_io_report_timestamp(&raw));
    let action = lookup_host_io_report_action(status, &fresh);
    let blockers = host_io_report_blocker_sample(&raw);
    let content = format!(
        "Status: host I/O guard. status={status} fresh={fresh} age_secs={age_secs} ttl_secs={ttl_secs} action={action} blockers={}",
        if blockers.is_empty() {
            "none"
        } else {
            blockers.as_str()
        }
    );
    Some(local_status_item(
        req,
        content,
        "host-io-guard.txt",
        path.display().to_string(),
        vec![
            "continuity".to_string(),
            "host-io-guard".to_string(),
            "lookup-local-fallback".to_string(),
        ],
    ))
}

fn local_status_item(
    req: &memd_schema::SearchMemoryRequest,
    content: String,
    source_system: &str,
    source_path: String,
    tags: Vec<String>,
) -> memd_schema::MemoryItem {
    memd_schema::MemoryItem {
        id: uuid::Uuid::new_v4(),
        content,
        redundancy_key: Some(format!("local:{source_system}:status")),
        belief_branch: None,
        preferred: true,
        kind: memd_schema::MemoryKind::Status,
        scope: memd_schema::MemoryScope::Project,
        project: req.project.clone(),
        namespace: req.namespace.clone(),
        workspace: req.workspace.clone(),
        visibility: req
            .visibility
            .unwrap_or(memd_schema::MemoryVisibility::Private),
        source_agent: None,
        source_system: Some(source_system.to_string()),
        source_path: Some(source_path),
        source_quality: Some(memd_schema::SourceQuality::Canonical),
        confidence: 1.0,
        ttl_seconds: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_verified_at: Some(chrono::Utc::now()),
        supersedes: Vec::new(),
        tags,
        status: memd_schema::MemoryStatus::Active,
        stage: memd_schema::MemoryStage::Canonical,
        lane: Some("continuity".to_string()),
        version: 1,
        correction_meta: None,
    }
}

fn overlay_wake_current_handoff(
    output: &Path,
    query: &str,
    req: &memd_schema::SearchMemoryRequest,
    mut response: memd_schema::SearchMemoryResponse,
) -> memd_schema::SearchMemoryResponse {
    if !lookup_query_requests_handoff_state(query) {
        return response;
    }

    let Some(item) = wake_current_handoff_item(output, req) else {
        return response;
    };
    response.items.retain(|existing| existing.id != item.id);
    response.items.insert(0, item);
    let limit = req.limit.unwrap_or(LOOKUP_DEPTH_RECORD_CAP).max(1);
    response.items.truncate(limit);
    response
}

fn lookup_query_requests_handoff_state(query: &str) -> bool {
    let terms = crate::runtime::lookup_query_terms(query);
    terms.iter().any(|term| term == "handoff")
        && (terms.iter().any(|term| term == "continuity")
            || terms.iter().any(|term| term == "next")
            || terms.iter().any(|term| term == "action")
            || terms.iter().any(|term| term == "current"))
}

fn wake_current_handoff_item(
    output: &Path,
    req: &memd_schema::SearchMemoryRequest,
) -> Option<memd_schema::MemoryItem> {
    let wake = fs::read_to_string(output.join("wake.md")).ok()?;
    let recovery_line = wake
        .lines()
        .find(|line| line.trim_start().starts_with("- recovery voice="))?;
    let next = wake_recovery_field(recovery_line, "next")
        .filter(|value| !value.eq_ignore_ascii_case("none"))?;
    let blocker = wake_recovery_field(recovery_line, "blocker").unwrap_or("none");
    let proof_blockers = wake_recovery_field(recovery_line, "proof_blockers").unwrap_or("none");
    let server_authority_blockers =
        wake_recovery_field(recovery_line, "server_authority_blockers").unwrap_or("none");
    let live_state_blockers =
        wake_recovery_field(recovery_line, "live_state_blockers").unwrap_or("none");
    let id = next
        .split_once(':')
        .and_then(|(candidate, _)| uuid::Uuid::parse_str(candidate.trim()).ok())
        .unwrap_or_else(uuid::Uuid::new_v4);
    let content = format!(
        "Status: current handoff next action from wake.md. next={next} | blocker={blocker} | proof_blockers={proof_blockers} | server_authority_blockers={server_authority_blockers} | live_state_blockers={live_state_blockers}"
    );

    Some(memd_schema::MemoryItem {
        id,
        content,
        redundancy_key: Some("local:wake:current-handoff-next-action".to_string()),
        belief_branch: None,
        preferred: true,
        kind: memd_schema::MemoryKind::Status,
        scope: memd_schema::MemoryScope::Project,
        project: req.project.clone(),
        namespace: req.namespace.clone(),
        workspace: req.workspace.clone(),
        visibility: req
            .visibility
            .unwrap_or(memd_schema::MemoryVisibility::Private),
        source_agent: None,
        source_system: Some("wake.md".to_string()),
        source_path: Some(output.join("wake.md").display().to_string()),
        source_quality: Some(memd_schema::SourceQuality::Canonical),
        confidence: 1.0,
        ttl_seconds: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_verified_at: Some(chrono::Utc::now()),
        supersedes: Vec::new(),
        tags: vec![
            "current-task".to_string(),
            "handoff".to_string(),
            "wake".to_string(),
            "lookup-overlay".to_string(),
        ],
        status: memd_schema::MemoryStatus::Active,
        stage: memd_schema::MemoryStage::Canonical,
        lane: Some("continuity".to_string()),
        version: 1,
        correction_meta: None,
    })
}

fn wake_recovery_field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let marker = format!("{key}=");
    let start = line.find(&marker)? + marker.len();
    let rest = &line[start..];
    let end = rest.find(" | ").unwrap_or(rest.len());
    let value = rest[..end].trim();
    (!value.is_empty()).then_some(value)
}

async fn lookup_resume_snapshot_fallback(
    base_url: &str,
    args: &LookupArgs,
    req: &memd_schema::SearchMemoryRequest,
) -> anyhow::Result<memd_schema::SearchMemoryResponse> {
    let resume_args = synth_resume_args(args);
    let snapshot = read_bundle_resume(&resume_args, base_url).await?;
    let terms = crate::runtime::lookup_query_terms(&args.query);
    let limit = req.limit.unwrap_or(LOOKUP_DEPTH_RECORD_CAP);
    let mut items = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for content in snapshot
        .compact_context_records()
        .into_iter()
        .chain(snapshot.compact_working_records())
        .chain(snapshot.compact_rehydration_summaries())
        .chain(snapshot.preferences.into_iter())
    {
        let normalized = crate::runtime::ResumeSnapshot::normalized_memory_text(&content);
        if normalized.is_empty() || !seen.insert(normalized.clone()) {
            continue;
        }
        if !resume_fallback_matches_terms(&normalized, &terms) {
            continue;
        }
        items.push(resume_fallback_item(content, req));
        if items.len() >= limit {
            break;
        }
    }

    Ok(memd_schema::SearchMemoryResponse {
        route: req
            .route
            .unwrap_or(memd_schema::RetrievalRoute::ProjectFirst),
        intent: req.intent.unwrap_or(memd_schema::RetrievalIntent::General),
        items,
        trace: None,
    })
}

fn resume_fallback_matches_terms(normalized: &str, terms: &[String]) -> bool {
    if terms.is_empty() {
        return true;
    }
    let matches = terms
        .iter()
        .filter(|term| normalized.contains(term.as_str()))
        .count();
    if terms.len() <= 3 {
        matches == terms.len()
    } else {
        matches >= 3
    }
}

fn resume_fallback_item(
    content: String,
    req: &memd_schema::SearchMemoryRequest,
) -> memd_schema::MemoryItem {
    let normalized = content.to_ascii_lowercase();
    let kind = compact_field(&content, "kind")
        .and_then(|value| crate::cli::parse_memory_kind_value(value).ok())
        .unwrap_or_else(|| infer_fallback_kind(&normalized));
    let stage = compact_field(&content, "stage")
        .and_then(parse_compact_stage)
        .unwrap_or(memd_schema::MemoryStage::Canonical);
    let status = compact_field(&content, "status")
        .and_then(|value| crate::cli::parse_memory_status_value(value).ok())
        .unwrap_or(memd_schema::MemoryStatus::Active);
    let confidence = compact_field(&content, "cf")
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(0.6);
    memd_schema::MemoryItem {
        id: compact_field(&content, "id")
            .and_then(|value| value.parse::<uuid::Uuid>().ok())
            .unwrap_or_else(uuid::Uuid::new_v4),
        content,
        redundancy_key: None,
        belief_branch: None,
        preferred: false,
        kind,
        scope: memd_schema::MemoryScope::Project,
        project: req.project.clone(),
        namespace: req.namespace.clone(),
        workspace: req.workspace.clone(),
        visibility: req
            .visibility
            .unwrap_or(memd_schema::MemoryVisibility::Private),
        source_agent: None,
        source_system: Some("resume-snapshot-fallback".to_string()),
        source_path: None,
        source_quality: Some(memd_schema::SourceQuality::Canonical),
        confidence,
        ttl_seconds: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_verified_at: None,
        supersedes: Vec::new(),
        tags: compact_field(&normalized, "tags")
            .map(|value| {
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|tag| !tag.is_empty())
                    .map(ToString::to_string)
                    .chain(std::iter::once("lookup-fallback".to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec!["lookup-fallback".to_string()]),
        status,
        stage,
        lane: None,
        version: 1,
        correction_meta: None,
    }
}

fn infer_fallback_kind(normalized: &str) -> memd_schema::MemoryKind {
    if normalized.starts_with("decision:") {
        memd_schema::MemoryKind::Decision
    } else if normalized.starts_with("preference:") {
        memd_schema::MemoryKind::Preference
    } else if normalized.starts_with("status ") {
        memd_schema::MemoryKind::Status
    } else if normalized.starts_with("file_edited:") {
        memd_schema::MemoryKind::LiveTruth
    } else {
        memd_schema::MemoryKind::Fact
    }
}

fn parse_compact_stage(value: &str) -> Option<memd_schema::MemoryStage> {
    match value.trim().to_ascii_lowercase().replace('-', "_").as_str() {
        "canonical" => Some(memd_schema::MemoryStage::Canonical),
        "candidate" => Some(memd_schema::MemoryStage::Candidate),
        _ => None,
    }
}

fn compact_field<'a>(record: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("{key}=");
    let start = record.find(&needle)? + needle.len();
    let rest = &record[start..];
    let end = rest.find(" | ").unwrap_or(rest.len());
    let value = rest[..end].trim();
    (!value.is_empty()).then_some(value)
}

pub(crate) fn clamp_lookup_limit(limit: Option<usize>) -> usize {
    let raw = limit.unwrap_or(LOOKUP_DEPTH_RECORD_CAP);
    raw.min(LOOKUP_DEPTH_RECORD_CAP).max(1)
}

pub(crate) fn synth_wake_args(args: &LookupArgs) -> WakeArgs {
    WakeArgs {
        output: args.output.clone(),
        project: args.project.clone(),
        namespace: args.namespace.clone(),
        agent: None,
        workspace: args.workspace.clone(),
        visibility: args.visibility.clone(),
        route: args.route.clone(),
        intent: args.intent.clone(),
        limit: args.limit,
        rehydration_limit: None,
        semantic: false,
        verbose: args.verbose,
        write: false,
        summary: false,
        raw: false,
        budget_tokens: 0,
        include_bucket: Vec::new(),
        exclude_bucket: Vec::new(),
    }
}

pub(crate) fn synth_resume_args(args: &LookupArgs) -> ResumeArgs {
    ResumeArgs {
        output: args.output.clone(),
        project: args.project.clone(),
        namespace: args.namespace.clone(),
        agent: None,
        workspace: args.workspace.clone(),
        visibility: args.visibility.clone(),
        route: args.route.clone(),
        intent: args.intent.clone(),
        limit: args.limit,
        rehydration_limit: None,
        semantic: false,
        prompt: false,
        summary: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resume_fallback_matching_allows_strong_overlap_for_long_queries() {
        let terms = crate::runtime::lookup_query_terms(
            "next-agent P0 handoff voice atomic commits repo hygiene cache bloat",
        );
        let record = crate::runtime::ResumeSnapshot::normalized_memory_text(
            "next-agent P0 handoff voice mode repo hygiene cache moved out of repo",
        );
        assert!(resume_fallback_matches_terms(&record, &terms));

        let unrelated = crate::runtime::ResumeSnapshot::normalized_memory_text(
            "configuration files and editor setup",
        );
        assert!(!resume_fallback_matches_terms(&unrelated, &terms));
    }

    #[test]
    fn resume_fallback_matching_allows_recovery_query_partial_records() {
        let terms = crate::runtime::lookup_query_terms(
            "next-agent P0 handoff voice atomic commits repo hygiene cache bloat",
        );
        let atomic_commit_record = crate::runtime::ResumeSnapshot::normalized_memory_text(
            "id=438c | kind=fact | tags=next-agent,atomic-commits,repo-hygiene,p0 | c=atomic commits are not working; keep changes small and scoped",
        );
        let cache_record = crate::runtime::ResumeSnapshot::normalized_memory_text(
            "id=ff8a | kind=fact | tags=token-efficiency,repo-hygiene | c=cache bloat is a P0 failure because raw caches burn tokens",
        );

        assert!(resume_fallback_matches_terms(&atomic_commit_record, &terms));
        assert!(resume_fallback_matches_terms(&cache_record, &terms));
    }

    #[test]
    fn resume_fallback_item_preserves_compact_record_identity_and_labels() {
        let id = uuid::Uuid::parse_str("07ab23c6-7653-4228-a6b9-6281eaf3a726").expect("parse uuid");
        let req = memd_schema::SearchMemoryRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            visibility: Some(memd_schema::MemoryVisibility::Private),
            ..memd_schema::SearchMemoryRequest::default()
        };
        let item = resume_fallback_item(
            "id=07ab23c6-7653-4228-a6b9-6281eaf3a726 | stage=canonical | scope=project | kind=decision | status=active | tags=next-agent,p0-handoff | cf=0.90 | c=next-agent handoff".to_string(),
            &req,
        );

        assert_eq!(item.id, id);
        assert_eq!(item.kind, memd_schema::MemoryKind::Decision);
        assert_eq!(item.stage, memd_schema::MemoryStage::Canonical);
        assert_eq!(item.status, memd_schema::MemoryStatus::Active);
        assert_eq!(item.confidence, 0.90);
        assert!(item.tags.contains(&"next-agent".to_string()));
        assert!(item.tags.contains(&"lookup-fallback".to_string()));
    }

    #[test]
    fn handoff_lookup_overlays_current_wake_next_action() {
        let dir = std::env::temp_dir().join(format!("memd-wake-overlay-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp bundle");
        std::fs::write(
            dir.join("wake.md"),
            "# wake\n\n- recovery voice=caveman-ultra | quality=ready:0.96 | dirty=0 | next=c79d1cb5-920f-4f76-8366-81c02daf4d09: Decision: current next action is live-state coverage | blocker=refresh recommended | proof_blockers=full_public:missing_explicit_env=RUN_LABEL | server_authority_blockers=server git_commit=d819af89 does not match local HEAD 82d65556 | live_state_blockers=memd:status=auth_required missing=calendar producer_route=\"scripts/live-state-sync-memd.sh\"\n",
        )
        .expect("write wake");

        let req = memd_schema::SearchMemoryRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            visibility: Some(memd_schema::MemoryVisibility::Private),
            limit: Some(3),
            ..memd_schema::SearchMemoryRequest::default()
        };
        let stale = resume_fallback_item(
            "id=07ab23c6-7653-4228-a6b9-6281eaf3a726 | stage=canonical | scope=project | kind=decision | status=active | c=old bridge done memory".to_string(),
            &req,
        );
        let response = memd_schema::SearchMemoryResponse {
            route: memd_schema::RetrievalRoute::ProjectFirst,
            intent: memd_schema::RetrievalIntent::General,
            items: vec![stale],
            trace: None,
        };

        let overlaid =
            overlay_wake_current_handoff(&dir, "handoff continuity next action", &req, response);

        assert_eq!(overlaid.items.len(), 2);
        assert_eq!(
            overlaid.items[0].id,
            uuid::Uuid::parse_str("c79d1cb5-920f-4f76-8366-81c02daf4d09").expect("uuid")
        );
        assert_eq!(overlaid.items[0].kind, memd_schema::MemoryKind::Status);
        assert!(overlaid.items[0].preferred);
        assert!(overlaid.items[0].content.contains("current next action"));
        assert!(
            overlaid.items[0]
                .content
                .contains("proof_blockers=full_public")
        );
        assert!(
            overlaid.items[0]
                .content
                .contains("server_authority_blockers=server git_commit=d819af89")
        );
        assert!(
            overlaid.items[0]
                .content
                .contains("live_state_blockers=memd:status=auth_required")
        );
        assert!(
            overlaid.items[0]
                .content
                .contains("producer_route=\"scripts/live-state-sync-memd.sh\"")
        );
        assert!(
            overlaid.items[0]
                .tags
                .contains(&"lookup-overlay".to_string())
        );

        std::fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn local_continuity_fallback_returns_live_map_status() {
        let dir =
            std::env::temp_dir().join(format!("memd-local-continuity-{}", uuid::Uuid::new_v4()));
        let state_dir = dir.join("state");
        std::fs::create_dir_all(&state_dir).expect("create state dir");
        std::fs::write(
            state_dir.join("codebase-live-map.json"),
            r#"{
  "status": "blocked",
  "autosync": "blocked_no_scan",
  "needs_reread": true,
  "updated_at": "2000-01-01T00:00:00Z",
  "file_count": 42,
  "blockers": [
    "host_io_guard_report status=blocked age_secs=1 ttl_secs=120 state=.memd/state/host-io-guard.txt",
    "volume:/Volumes/T7 project_hint=app-git pid=99 state=U command=git status"
  ]
}"#,
        )
        .expect("write live map");
        let req = memd_schema::SearchMemoryRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            visibility: Some(memd_schema::MemoryVisibility::Private),
            limit: Some(3),
            ..memd_schema::SearchMemoryRequest::default()
        };

        let response =
            lookup_local_continuity_fallback(&dir, "what is T7 live map blocker state", &req);

        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].kind, memd_schema::MemoryKind::Status);
        assert_eq!(
            response.items[0].source_system.as_deref(),
            Some("codebase-live-map.json")
        );
        assert!(response.items[0].content.contains("status=blocked"));
        assert!(response.items[0].content.contains("fresh=false"));
        assert!(
            response.items[0]
                .content
                .contains("autosync=blocked_no_scan")
        );
        assert!(response.items[0].content.contains("reread_required=true"));
        assert!(response.items[0].content.contains("fresh=false"));
        assert!(response.items[0].content.contains("ttl_secs=15"));
        assert!(
            response.items[0]
                .content
                .contains("action=refresh_host_guard_before_trusting_live_map")
        );
        assert!(response.items[0].content.contains("project_hint=app-git"));

        std::fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn local_continuity_fallback_uses_host_report_without_live_map() {
        let dir =
            std::env::temp_dir().join(format!("memd-local-host-report-{}", uuid::Uuid::new_v4()));
        let state_dir = dir.join("state");
        std::fs::create_dir_all(&state_dir).expect("create state dir");
        std::fs::write(
            state_dir.join("host-io-guard.txt"),
            "ts=2026-05-18T04:47:09Z\nrepo=/Volumes/T7/projects/memd\npid=1\nstatus=blocked\nvolume:/Volumes/T7 project_hint=filesystem pid=358 state=Us command=mds\n",
        )
        .expect("write host report");
        let req = memd_schema::SearchMemoryRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            visibility: Some(memd_schema::MemoryVisibility::Private),
            limit: Some(3),
            ..memd_schema::SearchMemoryRequest::default()
        };

        let response =
            lookup_local_continuity_fallback(&dir, "continuity host blocker state", &req);

        assert_eq!(response.items.len(), 1);
        assert_eq!(
            response.items[0].source_system.as_deref(),
            Some("host-io-guard.txt")
        );
        assert!(response.items[0].content.contains("status=blocked"));
        assert!(response.items[0].content.contains("fresh=false"));
        assert!(response.items[0].content.contains("ttl_secs=120"));
        assert!(
            response.items[0]
                .content
                .contains("action=refresh_host_guard_before_trusting_host_report")
        );
        assert!(
            response.items[0]
                .content
                .contains("project_hint=filesystem")
        );

        std::fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }
}
