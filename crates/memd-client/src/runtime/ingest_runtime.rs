use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};

use anyhow::Context;
use serde::Serialize;

use crate::append_raw_spine_record;
use crate::runtime::retrieval_runtime::{resolve_default_bundle_root, resolve_rag_url};
use crate::{
    IngestArgs, IngestSourcesArgs, RagSyncArgs, parse_memory_kind_value, parse_memory_scope_value,
    parse_memory_visibility_value, parse_source_quality_value, parse_uuid_list,
};
use memd_client::MemdClient;
use memd_multimodal::{
    MultimodalChunk, MultimodalIngestPlan, build_ingest_plan, extract_chunks, to_sidecar_requests,
};
use memd_rag::{RagClient, RagIngestRequest, RagRetrieveMode};
use memd_schema::{
    CandidateMemoryRequest, MemoryKind, MemoryScope, MemoryStage, MemoryStatus, RetrievalIntent,
    RetrievalRoute, SearchMemoryRequest, StoreMemoryRequest,
};
use memd_sidecar::{SidecarClient, SidecarIngestRequest, SidecarIngestResponse};

#[derive(Debug, Serialize)]
pub(crate) struct RagSyncSummary {
    pub(crate) fetched: usize,
    pub(crate) pushed: usize,
    pub(crate) dry_run: bool,
    pub(crate) project: Option<String>,
    pub(crate) namespace: Option<String>,
}

pub(crate) async fn sync_to_rag(
    memd: &MemdClient,
    rag: &RagClient,
    args: RagSyncArgs,
) -> anyhow::Result<RagSyncSummary> {
    let fetched = memd
        .search(&SearchMemoryRequest {
            query: None,
            route: Some(RetrievalRoute::All),
            intent: Some(RetrievalIntent::General),
            scopes: vec![MemoryScope::Project, MemoryScope::Global],
            kinds: vec![
                MemoryKind::Fact,
                MemoryKind::Decision,
                MemoryKind::Preference,
                MemoryKind::Runbook,
                MemoryKind::Procedural,
                MemoryKind::SelfModel,
                MemoryKind::Topology,
                MemoryKind::Status,
                MemoryKind::LiveTruth,
                MemoryKind::Pattern,
                MemoryKind::Constraint,
            ],
            statuses: vec![MemoryStatus::Active],
            project: args.project.clone(),
            namespace: args.namespace.clone(),
            workspace: None,
            visibility: None,
            belief_branch: None,
            source_agent: None,
            tags: Vec::new(),
            stages: vec![MemoryStage::Canonical],
            limit: args.limit,
            max_chars_per_item: Some(1000),
        })
        .await
        .context("load canonical memory for rag sync")?;

    let mut pushed = 0usize;
    for item in &fetched.items {
        if !args.dry_run {
            rag.ingest(&RagIngestRequest::from(item))
                .await
                .context("ingest rag record")?;
        }
        pushed += 1;
    }

    Ok(RagSyncSummary {
        fetched: fetched.items.len(),
        pushed,
        dry_run: args.dry_run,
        project: args.project,
        namespace: args.namespace,
    })
}

pub(crate) async fn sync_candidate_responses_to_rag(
    rag: &RagClient,
    responses: &[memd_schema::CandidateMemoryResponse],
) -> anyhow::Result<usize> {
    let mut pushed = 0usize;
    for response in responses {
        rag.ingest(&RagIngestRequest::from(&response.item))
            .await
            .context("ingest rag record from spill")?;
        pushed += 1;
    }
    Ok(pushed)
}

pub(crate) async fn sync_memory_items_to_rag(
    rag: &RagClient,
    items: &[memd_schema::MemoryItem],
) -> anyhow::Result<usize> {
    let mut pushed = 0usize;
    for item in items {
        rag.ingest(&RagIngestRequest::from(item))
            .await
            .context("ingest rag record from compiled knowledge page")?;
        pushed += 1;
    }
    Ok(pushed)
}

pub(crate) fn parse_rag_retrieve_mode(value: &str) -> anyhow::Result<RagRetrieveMode> {
    let normalized = value.trim().to_ascii_lowercase().replace('-', "_");
    match normalized.as_str() {
        "auto" => Ok(RagRetrieveMode::Auto),
        "text" => Ok(RagRetrieveMode::Text),
        "multimodal" => Ok(RagRetrieveMode::Multimodal),
        "graph" => Ok(RagRetrieveMode::Graph),
        _ => anyhow::bail!(
            "invalid rag retrieve mode '{value}'; expected auto, text, multimodal, or graph"
        ),
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct IngestAutoRouteResult {
    pub(crate) route: String,
    pub(crate) request: Option<CandidateMemoryRequest>,
    pub(crate) candidate: Option<memd_schema::CandidateMemoryResponse>,
    pub(crate) multimodal: Option<MultimodalIngestOutput>,
}

pub(crate) async fn ingest_auto_route(
    client: &MemdClient,
    args: &IngestArgs,
) -> anyhow::Result<IngestAutoRouteResult> {
    let force_text = args
        .route
        .as_deref()
        .is_some_and(|r| matches!(r, "memory" | "text"));

    if let Some(content) = &args.content {
        return ingest_text_memory(client, args, content.clone()).await;
    }

    if args.input.is_some() || args.stdin {
        let raw = read_ingest_payload(args)?;
        if !force_text && looks_like_multimodal(&raw) {
            return ingest_multimodal_payload(args, &raw).await;
        }
        return ingest_text_memory(client, args, raw).await;
    }

    anyhow::bail!("provide --content, --input, or --stdin");
}

fn read_ingest_payload(args: &IngestArgs) -> anyhow::Result<String> {
    if let Some(json) = &args.json {
        return Ok(json.clone());
    }
    if let Some(path) = &args.input {
        return fs::read_to_string(path)
            .with_context(|| format!("read ingest input file {}", path.display()));
    }
    if args.stdin {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("read ingest payload from stdin")?;
        return Ok(buffer);
    }
    anyhow::bail!("no ingest payload");
}

fn looks_like_multimodal(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return false;
    }
    trimmed.split_whitespace().any(|token| {
        let lowered = token
            .trim_matches(|c: char| c == ',' || c == ';')
            .to_ascii_lowercase();
        lowered.ends_with(".pdf")
            || lowered.ends_with(".png")
            || lowered.ends_with(".jpg")
            || lowered.ends_with(".jpeg")
            || lowered.ends_with(".webp")
            || lowered.ends_with(".heic")
            || lowered.ends_with(".mp4")
            || lowered.ends_with(".mov")
            || lowered.ends_with(".mkv")
            || lowered.ends_with(".webm")
            || lowered.ends_with(".csv")
            || lowered.ends_with(".tsv")
            || lowered.ends_with(".xlsx")
            || lowered.ends_with(".txt")
            || lowered.ends_with(".md")
    })
}

async fn ingest_text_memory(
    client: &MemdClient,
    args: &IngestArgs,
    content: String,
) -> anyhow::Result<IngestAutoRouteResult> {
    let kind = args
        .kind
        .as_deref()
        .map(parse_memory_kind_value)
        .transpose()?
        .unwrap_or(MemoryKind::Fact);
    let scope = args
        .scope
        .as_deref()
        .map(parse_memory_scope_value)
        .transpose()?
        .unwrap_or(MemoryScope::Project);
    let source_quality = args
        .source_quality
        .as_deref()
        .map(parse_source_quality_value)
        .transpose()?;
    let supersedes = parse_uuid_list(&args.supersede)?;
    let tags = args.tag.clone();

    let req = CandidateMemoryRequest {
        content,
        kind,
        scope,
        project: args.project.clone(),
        namespace: args.namespace.clone(),
        workspace: args.workspace.clone(),
        visibility: args
            .visibility
            .as_deref()
            .map(parse_memory_visibility_value)
            .transpose()?,
        belief_branch: None,
        source_agent: args.source_agent.clone(),
        source_system: args.source_system.clone(),
        source_path: args.source_path.clone(),
        source_quality,
        confidence: args.confidence,
        ttl_seconds: args.ttl_seconds,
        last_verified_at: None,
        supersedes,
        tags,
        lane: None,
    };

    if args.apply {
        let candidate = client.candidate(&req).await?;
        let output =
            resolve_default_bundle_root()?.unwrap_or_else(crate::bundle::default_bundle_root_path);
        append_raw_spine_record(
            &output,
            "ingest",
            "candidate",
            req.project.as_deref(),
            req.namespace.as_deref(),
            req.workspace.as_deref(),
            req.source_system.as_deref().or(Some("ingest")),
            req.source_path.as_deref(),
            req.confidence,
            &req.tags,
            &req.content,
        )?;
        Ok(IngestAutoRouteResult {
            route: "memory".to_string(),
            request: Some(req),
            candidate: Some(candidate),
            multimodal: None,
        })
    } else {
        Ok(IngestAutoRouteResult {
            route: "memory".to_string(),
            request: Some(req),
            candidate: None,
            multimodal: None,
        })
    }
}

async fn ingest_multimodal_payload(
    args: &IngestArgs,
    payload: &str,
) -> anyhow::Result<IngestAutoRouteResult> {
    let paths = payload
        .lines()
        .flat_map(|line| line.split_whitespace())
        .map(|token| token.trim_matches(|c: char| c == ',' || c == ';'))
        .filter(|token| {
            token.ends_with(".pdf")
                || token.ends_with(".png")
                || token.ends_with(".jpg")
                || token.ends_with(".jpeg")
                || token.ends_with(".webp")
                || token.ends_with(".heic")
                || token.ends_with(".mp4")
                || token.ends_with(".mov")
                || token.ends_with(".mkv")
                || token.ends_with(".webm")
                || token.ends_with(".csv")
                || token.ends_with(".tsv")
                || token.ends_with(".xlsx")
                || token.ends_with(".txt")
                || token.ends_with(".md")
        })
        .map(PathBuf::from)
        .collect::<Vec<_>>();

    let preview = build_multimodal_preview(args.project.clone(), args.namespace.clone(), &paths)?;
    let multimodal = if args.apply {
        let rag_url = resolve_rag_url(None, resolve_default_bundle_root()?.as_deref())?;
        let sidecar = SidecarClient::new(&rag_url)?;
        let responses = ingest_multimodal_preview(&sidecar, &preview.requests).await?;
        let submitted = responses.len();
        MultimodalIngestOutput {
            preview,
            responses,
            submitted,
            dry_run: false,
        }
    } else {
        MultimodalIngestOutput {
            preview,
            responses: Vec::new(),
            submitted: 0,
            dry_run: true,
        }
    };

    Ok(IngestAutoRouteResult {
        route: "multimodal".to_string(),
        request: None,
        candidate: None,
        multimodal: Some(multimodal),
    })
}

pub(crate) fn parse_context_time(
    value: Option<String>,
) -> anyhow::Result<Option<chrono::DateTime<chrono::Utc>>> {
    match value {
        Some(value) => Ok(Some(
            chrono::DateTime::parse_from_rfc3339(&value)
                .context("parse context time as RFC3339")?
                .with_timezone(&chrono::Utc),
        )),
        None => Ok(None),
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct MultimodalPreview {
    pub(crate) plan: MultimodalIngestPlan,
    pub(crate) chunks: Vec<MultimodalChunk>,
    pub(crate) requests: Vec<SidecarIngestRequest>,
}

#[derive(Debug, Serialize)]
pub(crate) struct MultimodalIngestOutput {
    pub(crate) preview: MultimodalPreview,
    pub(crate) responses: Vec<SidecarIngestResponse>,
    pub(crate) submitted: usize,
    pub(crate) dry_run: bool,
}

pub(crate) fn build_multimodal_preview(
    project: Option<String>,
    namespace: Option<String>,
    paths: &[PathBuf],
) -> anyhow::Result<MultimodalPreview> {
    if paths.is_empty() {
        anyhow::bail!("provide at least one --path");
    }

    let plan = build_ingest_plan(paths.iter(), project, namespace)?;
    let chunks = extract_chunks(&plan)?;
    let requests = to_sidecar_requests(&plan, &chunks);

    Ok(MultimodalPreview {
        plan,
        chunks,
        requests,
    })
}

pub(crate) async fn ingest_multimodal_preview(
    sidecar: &SidecarClient,
    requests: &[SidecarIngestRequest],
) -> anyhow::Result<Vec<SidecarIngestResponse>> {
    let mut responses = Vec::with_capacity(requests.len());
    for request in requests {
        responses.push(sidecar.ingest(request).await?);
    }
    Ok(responses)
}

#[derive(Debug, Serialize)]
pub(crate) struct IngestSourcesResult {
    pub(crate) dir: String,
    pub(crate) lane: String,
    pub(crate) files_found: usize,
    pub(crate) ingested: usize,
    pub(crate) skipped: usize,
    pub(crate) dry_run: bool,
    pub(crate) items: Vec<IngestSourcesFileResult>,
}

#[derive(Debug, Serialize)]
pub(crate) struct IngestSourcesFileResult {
    pub(crate) path: String,
    pub(crate) chars: usize,
    pub(crate) status: String,
    pub(crate) id: Option<String>,
}

pub(crate) async fn ingest_sources(
    client: &MemdClient,
    args: &IngestSourcesArgs,
) -> anyhow::Result<IngestSourcesResult> {
    let dir = &args.dir;
    anyhow::ensure!(dir.is_dir(), "{} is not a directory", dir.display());

    let kind = parse_memory_kind_value(&args.kind)?;
    let scope = parse_memory_scope_value(&args.scope)?;

    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .with_context(|| format!("read directory {}", dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|ext| ext == "md" || ext == "txt")
        })
        .collect();
    entries.sort();

    let mut tags = vec![
        format!("lane:{}", args.lane),
        "research".to_string(),
        "ingested-source".to_string(),
    ];
    tags.extend(args.tag.iter().cloned());

    let mut items = Vec::with_capacity(entries.len());
    let mut ingested = 0usize;
    let mut skipped = 0usize;

    for path in &entries {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("warn: skip {}: {e}", path.display());
                skipped += 1;
                items.push(IngestSourcesFileResult {
                    path: path.display().to_string(),
                    chars: 0,
                    status: format!("error: {e}"),
                    id: None,
                });
                continue;
            }
        };

        let trimmed = content.trim();
        if trimmed.is_empty() {
            skipped += 1;
            items.push(IngestSourcesFileResult {
                path: path.display().to_string(),
                chars: 0,
                status: "skipped: empty".to_string(),
                id: None,
            });
            continue;
        }

        let source_path = path.display().to_string();

        if args.apply {
            let req = StoreMemoryRequest {
                content: trimmed.to_string(),
                kind,
                scope,
                project: args.project.clone(),
                namespace: args.namespace.clone(),
                workspace: None,
                visibility: None,
                belief_branch: None,
                source_agent: Some("ingest-sources".to_string()),
                source_system: Some("lane-ingest".to_string()),
                source_path: Some(source_path.clone()),
                source_quality: None,
                confidence: Some(0.85),
                ttl_seconds: None,
                last_verified_at: None,
                supersedes: Vec::new(),
                tags: tags.clone(),
                status: Some(MemoryStatus::Active),
                lane: None,
            };
            let resp = client
                .store(&req)
                .await
                .with_context(|| format!("store canonical item for {}", path.display()))?;
            items.push(IngestSourcesFileResult {
                path: source_path,
                chars: trimmed.len(),
                status: "ingested".to_string(),
                id: Some(resp.item.id.to_string()),
            });
            ingested += 1;
        } else {
            items.push(IngestSourcesFileResult {
                path: source_path,
                chars: trimmed.len(),
                status: "dry-run".to_string(),
                id: None,
            });
            ingested += 1;
        }
    }

    Ok(IngestSourcesResult {
        dir: dir.display().to_string(),
        lane: args.lane.clone(),
        files_found: entries.len(),
        ingested,
        skipped,
        dry_run: !args.apply,
        items,
    })
}
