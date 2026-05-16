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
    let mut response = lookup_with_fallbacks(client, &req, &args.query).await?;
    if response.items.is_empty() {
        response = lookup_resume_snapshot_fallback(base_url, &args, &req).await?;
    }
    let escalation_hint =
        (response.items.is_empty() && escalation_hint_enabled() && escalation::detect(&args.query))
            .then(|| escalation::hint_line(&args.query));
    let markdown = render_lookup_markdown(&args.query, &req, &response, args.verbose);
    Ok(LookupArmOutcome {
        response,
        markdown,
        json: args.json,
        escalation_hint,
    })
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
}
