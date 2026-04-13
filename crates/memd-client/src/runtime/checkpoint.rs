use super::*;

pub(crate) fn append_raw_spine_record(
    output: &Path,
    event_type: &str,
    stage: &str,
    project: Option<&str>,
    namespace: Option<&str>,
    workspace: Option<&str>,
    source_system: Option<&str>,
    source_path: Option<&str>,
    confidence: Option<f32>,
    tags: &[String],
    content: &str,
) -> anyhow::Result<()> {
    let tag_refs = tags.iter().map(String::as_str).collect::<Vec<_>>();
    let record = derive_raw_spine_record(
        event_type,
        stage,
        source_system,
        source_path,
        project,
        namespace,
        workspace,
        confidence,
        &tag_refs,
        content,
    );
    write_raw_spine_records(output, &[record])
}

pub(crate) fn infer_bundle_identity_defaults(output: &Path) -> (Option<String>, Option<String>) {
    let Some(project_root) = infer_bundle_project_root(output) else {
        return (None, None);
    };

    let project = project_root
        .file_name()
        .and_then(|value| value.to_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let namespace = project.as_ref().map(|_| "main".to_string());
    (project, namespace)
}

pub(crate) async fn remember_with_bundle_defaults(
    args: &RememberArgs,
    base_url: &str,
) -> anyhow::Result<memd_schema::StoreMemoryResponse> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let session = runtime.as_ref().and_then(|config| config.session.clone());
    let (inferred_project, inferred_namespace) = infer_bundle_identity_defaults(&args.output);
    let project = args
        .project
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.project.clone()))
        .or(inferred_project);
    let namespace = args
        .namespace
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.namespace.clone()))
        .or(inferred_namespace);
    let workspace = args
        .workspace
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.workspace.clone()));
    let visibility_raw = args.visibility.clone().or_else(|| {
        runtime
            .as_ref()
            .and_then(|config| config.visibility.clone())
    });
    let base_url = resolve_bundle_command_base_url(
        base_url,
        runtime
            .as_ref()
            .and_then(|config| config.base_url.as_deref()),
    );
    let source_agent = args
        .source_agent
        .clone()
        .or_else(|| runtime.as_ref().and_then(|config| config.agent.clone()))
        .as_deref()
        .map(|value| compose_agent_identity(value, session.as_deref()));

    let content = if let Some(content) = &args.content {
        content.clone()
    } else if let Some(path) = &args.input {
        fs::read_to_string(path)
            .with_context(|| format!("read remember input file {}", path.display()))?
    } else if args.stdin {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("read remember payload from stdin")?;
        buffer
    } else {
        anyhow::bail!("provide --content, --input, or --stdin");
    };

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
        .unwrap_or_else(|| {
            if project.is_some() {
                MemoryScope::Project
            } else {
                MemoryScope::Synced
            }
        });
    let source_quality = args
        .source_quality
        .as_deref()
        .map(parse_source_quality_value)
        .transpose()?;
    let visibility = visibility_raw
        .as_deref()
        .map(parse_memory_visibility_value)
        .transpose()?;
    let supersedes = parse_uuid_list(&args.supersede)?;

    let client = MemdClient::new(&base_url)?;
    let response = client
        .store(&memd_schema::StoreMemoryRequest {
            content,
            kind,
            scope,
            project,
            namespace,
            workspace,
            visibility,
            belief_branch: None,
            source_agent,
            source_system: args.source_system.clone().or(Some("memd".to_string())),
            source_path: args.source_path.clone(),
            source_quality,
            confidence: args.confidence,
            ttl_seconds: args.ttl_seconds,
            last_verified_at: None,
            supersedes,
            tags: args.tag.clone(),
            status: Some(MemoryStatus::Active),
        })
        .await?;

    append_raw_spine_record(
        &args.output,
        "remember",
        "canonical",
        response.item.project.as_deref(),
        response.item.namespace.as_deref(),
        response.item.workspace.as_deref(),
        response.item.source_system.as_deref(),
        response.item.source_path.as_deref(),
        Some(response.item.confidence),
        &response.item.tags,
        &response.item.content,
    )?;

    Ok(response)
}

pub(crate) async fn checkpoint_with_bundle_defaults(
    args: &CheckpointArgs,
    base_url: &str,
) -> anyhow::Result<memd_schema::StoreMemoryResponse> {
    let translated = checkpoint_as_remember_args(args);
    let response = remember_with_bundle_defaults(&translated, base_url).await?;
    append_raw_spine_record(
        &args.output,
        "checkpoint",
        "candidate",
        translated.project.as_deref(),
        translated.namespace.as_deref(),
        translated.workspace.as_deref(),
        translated.source_system.as_deref(),
        translated.source_path.as_deref(),
        translated.confidence,
        &translated.tag,
        translated.content.as_deref().unwrap_or_default(),
    )?;
    Ok(response)
}

pub(crate) fn remember_args_from_hook_capture(
    args: &HookCaptureArgs,
    content: String,
) -> RememberArgs {
    let tags = if args.promote_tag.is_empty() {
        vec![
            "promoted".to_string(),
            "durable-memory".to_string(),
            "from-hook-capture".to_string(),
        ]
    } else {
        args.promote_tag.clone()
    };

    RememberArgs {
        output: args.output.clone(),
        project: args.project.clone(),
        namespace: args.namespace.clone(),
        workspace: args.workspace.clone(),
        visibility: args.visibility.clone(),
        kind: args.promote_kind.clone(),
        scope: args.promote_scope.clone(),
        source_agent: None,
        source_system: Some("memd".to_string()),
        source_path: args
            .source_path
            .clone()
            .or(Some("hook-capture-promotion".to_string())),
        source_quality: Some("canonical".to_string()),
        confidence: args.promote_confidence.or(args.confidence),
        ttl_seconds: None,
        tag: tags,
        supersede: args.promote_supersede.clone(),
        content: Some(content),
        input: None,
        stdin: false,
    }
}

pub(crate) fn infer_promote_kind_from_capture(content: &str) -> Option<&'static str> {
    let trimmed = content.trim_start();
    let normalized = trimmed.to_ascii_lowercase();
    for kind in [
        "decision",
        "preference",
        "constraint",
        "fact",
        "runbook",
        "procedural",
        "status",
    ] {
        let prefix = format!("{kind}:");
        if normalized.starts_with(&prefix) {
            return Some(kind);
        }
    }
    None
}

pub(crate) fn effective_hook_capture_promote_kind(
    args: &HookCaptureArgs,
    content: &str,
) -> Option<String> {
    args.promote_kind
        .clone()
        .or_else(|| infer_promote_kind_from_capture(content).map(str::to_string))
}

pub(crate) fn infer_supersede_query_from_capture(content: &str) -> Option<String> {
    let trimmed = content.trim();
    let normalized = trimmed.to_ascii_lowercase();
    let prefixes = [
        "corrected fact:",
        "corrected decision:",
        "corrected preference:",
        "corrected constraint:",
        "correction:",
    ];
    for prefix in prefixes {
        if normalized.starts_with(prefix) {
            let query = trimmed[prefix.len()..].trim();
            if !query.is_empty() {
                return Some(query.to_string());
            }
        }
    }
    None
}

pub(crate) fn effective_hook_capture_supersede_query(
    args: &HookCaptureArgs,
    content: &str,
) -> Option<String> {
    args.promote_supersede_query.clone().or_else(|| {
        if args.promote_supersede.is_empty() {
            infer_supersede_query_from_capture(content)
        } else {
            None
        }
    })
}

pub(crate) fn condensed_supersede_query(query: &str) -> Option<String> {
    let tokens = supersede_query_keywords(query)
        .into_iter()
        .take(5)
        .collect::<Vec<_>>();
    if tokens.len() >= 2 {
        Some(tokens.join(" "))
    } else {
        None
    }
}

pub(crate) fn supersede_query_keywords(query: &str) -> Vec<String> {
    let stopwords = [
        "does", "do", "did", "not", "prove", "should", "would", "could", "must", "keep", "that",
        "this", "from", "with", "into", "about", "memory", "agent", "usable",
    ];
    query
        .split(|c: char| !c.is_ascii_alphanumeric())
        .map(|token| token.trim().to_ascii_lowercase())
        .filter(|token| token.len() >= 4)
        .filter(|token| !stopwords.iter().any(|stop| stop == token))
        .collect::<Vec<_>>()
}

pub(crate) fn supersede_query_candidates(query: &str) -> Vec<String> {
    let mut candidates = vec![query.trim().to_string()];
    if let Some(condensed) = condensed_supersede_query(query)
        && !candidates.iter().any(|existing| existing == &condensed) {
            candidates.push(condensed);
        }
    let keywords = supersede_query_keywords(query);
    for window in keywords.windows(2).take(3) {
        let candidate = window.join(" ");
        if !candidates.iter().any(|existing| existing == &candidate) {
            candidates.push(candidate);
        }
    }
    for window in keywords.windows(3).take(2) {
        let candidate = window.join(" ");
        if !candidates.iter().any(|existing| existing == &candidate) {
            candidates.push(candidate);
        }
    }
    candidates
}

pub(crate) fn infer_promote_tags_from_capture(
    promote_kind: &str,
    content: &str,
    has_supersedes: bool,
) -> Vec<String> {
    let normalized = content.to_ascii_lowercase();
    let mut tags = vec![
        "promoted".to_string(),
        "durable-memory".to_string(),
        "from-hook-capture".to_string(),
        "auto-promoted".to_string(),
        promote_kind.to_string(),
    ];

    if has_supersedes
        || normalized.starts_with("corrected ")
        || normalized.starts_with("correction:")
    {
        tags.push("correction".to_string());
    }
    if normalized.contains("design")
        || normalized.contains(" ux")
        || normalized.contains("ux/")
        || normalized.contains(" ui")
        || normalized.contains("ui/")
    {
        tags.push("design-memory".to_string());
    }
    if normalized.contains("product direction")
        || normalized.contains("startup surface")
        || normalized.contains("memory loop")
        || normalized.contains("live memory")
    {
        tags.push("product-direction".to_string());
    }

    tags.sort();
    tags.dedup();
    tags
}

pub(crate) fn remember_args_from_effective_hook_capture(
    args: &HookCaptureArgs,
    content: String,
    promote_kind: String,
    supersedes: Vec<uuid::Uuid>,
) -> RememberArgs {
    let mut remember = remember_args_from_hook_capture(args, content);
    remember.kind = Some(promote_kind);
    remember.supersede = supersedes.into_iter().map(|id| id.to_string()).collect();
    if args.promote_tag.is_empty() {
        remember.tag = infer_promote_tags_from_capture(
            remember.kind.as_deref().unwrap_or("fact"),
            remember.content.as_deref().unwrap_or(""),
            !remember.supersede.is_empty(),
        );
    }
    remember
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SupersedeSearchDiagnostics {
    pub(crate) inferred_query: Option<String>,
    pub(crate) tried_queries: Vec<String>,
    pub(crate) matched_ids: Vec<String>,
    pub(crate) candidate_hits: Vec<SupersedeCandidateHit>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SupersedeCandidateHit {
    pub(crate) query: String,
    pub(crate) ids: Vec<String>,
    pub(crate) statuses: Vec<String>,
    pub(crate) kinds: Vec<String>,
    pub(crate) previews: Vec<String>,
}

pub(crate) async fn find_hook_capture_supersede_targets(
    base_url: &str,
    args: &HookCaptureArgs,
    content: &str,
) -> anyhow::Result<(Vec<uuid::Uuid>, SupersedeSearchDiagnostics)> {
    let mut superseded_ids = parse_uuid_list(&args.promote_supersede)?;
    let inferred_query = effective_hook_capture_supersede_query(args, content);
    let mut tried_queries = Vec::new();
    let mut candidate_hits = Vec::new();
    if let Some(query) = inferred_query.as_deref() {
        let result = search_supersede_candidates(base_url, args, query).await?;
        tried_queries = result.tried_queries;
        candidate_hits = result.candidate_hits;
        superseded_ids.extend(
            result
                .items
                .into_iter()
                .filter(|item| matches!(item.status, MemoryStatus::Stale | MemoryStatus::Contested))
                .map(|item| item.id),
        );
    }
    superseded_ids.sort();
    superseded_ids.dedup();
    Ok((
        superseded_ids,
        SupersedeSearchDiagnostics {
            inferred_query,
            tried_queries,
            matched_ids: Vec::new(),
            candidate_hits,
        },
    ))
}

pub(crate) async fn mark_hook_capture_supersede_targets(
    base_url: &str,
    args: &HookCaptureArgs,
    superseded_ids: &[uuid::Uuid],
    promoted_id: uuid::Uuid,
) -> anyhow::Result<Vec<memd_schema::RepairMemoryResponse>> {
    if superseded_ids.is_empty() {
        return Ok(Vec::new());
    }
    let visibility = args
        .visibility
        .as_deref()
        .map(parse_memory_visibility_value)
        .transpose()?;
    let client = MemdClient::new(base_url)?;
    let mut responses = Vec::with_capacity(superseded_ids.len());
    for id in superseded_ids {
        let response = client
            .repair(&RepairMemoryRequest {
                id: *id,
                mode: MemoryRepairMode::Supersede,
                confidence: Some(0.25),
                status: Some(MemoryStatus::Superseded),
                workspace: args.workspace.clone(),
                visibility,
                source_agent: None,
                source_system: Some("memd-correction".to_string()),
                source_path: args
                    .source_path
                    .clone()
                    .or(Some("hook-capture-promotion".to_string())),
                source_quality: Some(memd_schema::SourceQuality::Canonical),
                content: None,
                tags: Some(vec![
                    "superseded".to_string(),
                    "from-hook-capture".to_string(),
                    promoted_id.to_string(),
                ]),
                supersedes: vec![],
            })
            .await?;
        responses.push(response);
    }
    Ok(responses)
}

pub(crate) struct SupersedeSearchResult {
    items: Vec<memd_schema::MemoryItem>,
    pub(crate) tried_queries: Vec<String>,
    pub(crate) candidate_hits: Vec<SupersedeCandidateHit>,
}

pub(crate) async fn search_supersede_candidates(
    base_url: &str,
    args: &HookCaptureArgs,
    query: &str,
) -> anyhow::Result<SupersedeSearchResult> {
    let visibility = args
        .visibility
        .as_deref()
        .map(parse_memory_visibility_value)
        .transpose()?;
    let client = MemdClient::new(base_url)?;
    let kinds = if let Some(kind) = effective_hook_capture_promote_kind(args, query) {
        vec![parse_memory_kind_value(&kind)?]
    } else {
        default_supersede_search_kinds()
    };
    let mut tried_queries = Vec::new();
    let mut candidate_hits = Vec::new();
    for candidate_query in supersede_query_candidates(query) {
        tried_queries.push(candidate_query.clone());
        let response = client
            .search(&SearchMemoryRequest {
                query: Some(candidate_query.clone()),
                route: Some(RetrievalRoute::ProjectFirst),
                intent: Some(RetrievalIntent::General),
                scopes: vec![MemoryScope::Project, MemoryScope::Synced],
                kinds: kinds.clone(),
                statuses: vec![
                    MemoryStatus::Active,
                    MemoryStatus::Stale,
                    MemoryStatus::Contested,
                ],
                project: args.project.clone(),
                namespace: args.namespace.clone(),
                workspace: args.workspace.clone(),
                visibility,
                belief_branch: None,
                source_agent: None,
                tags: Vec::new(),
                stages: vec![MemoryStage::Canonical],
                limit: Some(3),
                max_chars_per_item: Some(220),
            })
            .await?;
        let mut items = response.items;
        items.sort_by_key(|item| match item.status {
            MemoryStatus::Stale => 0,
            MemoryStatus::Contested => 1,
            MemoryStatus::Active => 2,
            MemoryStatus::Superseded => 3,
            MemoryStatus::Expired => 4,
        });
        candidate_hits.push(summarize_supersede_candidate_hit(&candidate_query, &items));
        if !items.is_empty() {
            return Ok(SupersedeSearchResult {
                items,
                tried_queries,
                candidate_hits,
            });
        }
    }
    let recent_fallback =
        search_recent_supersede_candidates(&client, args, visibility, &kinds, query).await?;
    candidate_hits.push(summarize_supersede_candidate_hit(
        "recent-scan",
        &recent_fallback,
    ));
    Ok(SupersedeSearchResult {
        items: recent_fallback,
        tried_queries,
        candidate_hits,
    })
}

pub(crate) async fn search_recent_supersede_candidates(
    client: &MemdClient,
    args: &HookCaptureArgs,
    visibility: Option<memd_schema::MemoryVisibility>,
    kinds: &[MemoryKind],
    query: &str,
) -> anyhow::Result<Vec<memd_schema::MemoryItem>> {
    let response = client
        .search(&SearchMemoryRequest {
            query: None,
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::General),
            scopes: vec![MemoryScope::Project, MemoryScope::Synced],
            kinds: kinds.to_vec(),
            statuses: vec![
                MemoryStatus::Active,
                MemoryStatus::Stale,
                MemoryStatus::Contested,
            ],
            project: args.project.clone(),
            namespace: args.namespace.clone(),
            workspace: args.workspace.clone(),
            visibility,
            belief_branch: None,
            source_agent: None,
            tags: Vec::new(),
            stages: vec![MemoryStage::Canonical],
            limit: Some(24),
            max_chars_per_item: Some(220),
        })
        .await?;
    let ranked = rank_recent_supersede_candidates(query, response.items);
    if !ranked.is_empty() || kinds.len() != 1 {
        return Ok(ranked);
    }

    let broad_response = client
        .search(&SearchMemoryRequest {
            query: None,
            route: Some(RetrievalRoute::ProjectFirst),
            intent: Some(RetrievalIntent::General),
            scopes: vec![MemoryScope::Project, MemoryScope::Synced],
            kinds: default_supersede_search_kinds(),
            statuses: vec![
                MemoryStatus::Active,
                MemoryStatus::Stale,
                MemoryStatus::Contested,
            ],
            project: args.project.clone(),
            namespace: args.namespace.clone(),
            workspace: args.workspace.clone(),
            visibility,
            belief_branch: None,
            source_agent: None,
            tags: Vec::new(),
            stages: vec![MemoryStage::Canonical],
            limit: Some(32),
            max_chars_per_item: Some(220),
        })
        .await?;
    Ok(rank_recent_supersede_candidates(
        query,
        broad_response.items,
    ))
}

pub(crate) fn rank_recent_supersede_candidates(
    query: &str,
    items: Vec<memd_schema::MemoryItem>,
) -> Vec<memd_schema::MemoryItem> {
    let query_terms = lexical_terms(query);
    let mut ranked = items
        .into_iter()
        .filter_map(|item| {
            let score = lexical_overlap_score(&query_terms, &item.content);
            if score == 0 {
                None
            } else {
                Some((score, supersede_status_rank(item.status), item))
            }
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    ranked
        .into_iter()
        .map(|(_, _, item)| item)
        .take(3)
        .collect()
}

pub(crate) fn lexical_overlap_score(
    query_terms: &std::collections::HashSet<String>,
    content: &str,
) -> usize {
    let content_terms = lexical_terms(content);
    query_terms.intersection(&content_terms).count()
}

pub(crate) fn lexical_terms(value: &str) -> std::collections::HashSet<String> {
    supersede_query_keywords(value).into_iter().collect()
}

pub(crate) fn supersede_status_rank(status: MemoryStatus) -> usize {
    match status {
        MemoryStatus::Stale => 0,
        MemoryStatus::Contested => 1,
        MemoryStatus::Active => 2,
        MemoryStatus::Superseded => 3,
        MemoryStatus::Expired => 4,
    }
}

pub(crate) fn default_supersede_search_kinds() -> Vec<MemoryKind> {
    vec![
        MemoryKind::Fact,
        MemoryKind::Decision,
        MemoryKind::Preference,
        MemoryKind::Constraint,
        MemoryKind::Status,
    ]
}

pub(crate) fn summarize_supersede_candidate_hit(
    query: &str,
    items: &[memd_schema::MemoryItem],
) -> SupersedeCandidateHit {
    SupersedeCandidateHit {
        query: query.to_string(),
        ids: items.iter().map(|item| item.id.to_string()).collect(),
        statuses: items
            .iter()
            .map(|item| format!("{:?}", item.status).to_ascii_lowercase())
            .collect(),
        kinds: items
            .iter()
            .map(|item| format!("{:?}", item.kind).to_ascii_lowercase())
            .collect(),
        previews: items
            .iter()
            .map(|item| summarize_supersede_content_preview(&item.content))
            .collect(),
    }
}

pub(crate) fn summarize_supersede_content_preview(content: &str) -> String {
    const MAX_LEN: usize = 48;
    let compact = content.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= MAX_LEN {
        compact
    } else {
        let trimmed = compact.chars().take(MAX_LEN).collect::<String>();
        format!("{trimmed}...")
    }
}

pub(crate) fn format_supersede_candidate_hit(hit: &SupersedeCandidateHit) -> String {
    let entries = hit
        .ids
        .iter()
        .zip(hit.statuses.iter())
        .zip(hit.kinds.iter())
        .zip(hit.previews.iter())
        .map(|(((id, status), kind), preview)| {
            format!("{}:{}:{}:{}", short_uuid_label(id), status, kind, preview)
        })
        .collect::<Vec<_>>();
    if entries.is_empty() {
        format!("{}=>none", hit.query)
    } else {
        format!("{}=>{}", hit.query, entries.join(","))
    }
}

pub(crate) fn summarize_hook_capture_supersede_diagnostics(
    diagnostics: &SupersedeSearchDiagnostics,
) -> (String, String, String) {
    let query = diagnostics
        .inferred_query
        .as_deref()
        .unwrap_or("none")
        .to_string();
    let tried = if diagnostics.tried_queries.is_empty() {
        "none".to_string()
    } else {
        diagnostics.tried_queries.join("|")
    };
    let hits = if diagnostics.candidate_hits.is_empty() {
        "none".to_string()
    } else {
        diagnostics
            .candidate_hits
            .iter()
            .map(format_supersede_candidate_hit)
            .collect::<Vec<_>>()
            .join("|")
    };
    (query, tried, hits)
}

pub(crate) fn short_uuid_label(value: &str) -> String {
    value.chars().take(8).collect()
}

pub(crate) async fn auto_checkpoint_bundle_event(
    output: &Path,
    base_url: &str,
    source_path: &str,
    content: String,
    tags: Vec<String>,
    confidence: f32,
) -> anyhow::Result<()> {
    if read_bundle_runtime_config(output)?.is_none() {
        return Ok(());
    }
    if !bundle_auto_short_term_capture_enabled(output)? {
        return Ok(());
    }
    if content.trim().is_empty() {
        return Ok(());
    }

    checkpoint_with_bundle_defaults(
        &CheckpointArgs {
            output: output.to_path_buf(),
            project: None,
            namespace: None,
            workspace: None,
            visibility: None,
            source_path: Some(source_path.to_string()),
            confidence: Some(confidence),
            ttl_seconds: Some(3_600),
            tag: tags,
            content: Some(content),
            input: None,
            stdin: false,
        },
        base_url,
    )
    .await?;

    let snapshot = read_bundle_resume(
        &ResumeArgs {
            output: output.to_path_buf(),
            project: None,
            namespace: None,
            agent: None,
            workspace: None,
            visibility: None,
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
    .await?;
    write_bundle_memory_files(output, &snapshot, None, false).await?;
    Ok(())
}

pub(crate) async fn auto_checkpoint_live_snapshot(
    output: &Path,
    base_url: &str,
    snapshot: &ResumeSnapshot,
    source_path: &str,
) -> anyhow::Result<()> {
    if read_bundle_runtime_config(output)?.is_none() {
        return Ok(());
    }
    if !bundle_auto_short_term_capture_enabled(output)? {
        return Ok(());
    }

    let content = format!(
        "status: {} source_path={source_path}",
        render_bundle_wakeup_summary(snapshot)
    );
    checkpoint_with_bundle_defaults(
        &CheckpointArgs {
            output: output.to_path_buf(),
            project: snapshot.project.clone(),
            namespace: snapshot.namespace.clone(),
            workspace: snapshot.workspace.clone(),
            visibility: snapshot.visibility.clone(),
            source_path: Some(source_path.to_string()),
            confidence: Some(0.72),
            ttl_seconds: Some(3_600),
            tag: vec![
                "auto-short-term".to_string(),
                "bundle-refresh".to_string(),
                source_path.to_string(),
            ],
            content: Some(content),
            input: None,
            stdin: false,
        },
        base_url,
    )
    .await?;
    Ok(())
}

pub(crate) async fn auto_checkpoint_compaction_packet(
    packet: &CompactionPacket,
    base_url: &str,
) -> anyhow::Result<()> {
    let Some(output) = resolve_default_bundle_root()? else {
        return Ok(());
    };
    if read_bundle_runtime_config(&output)?.is_none() {
        return Ok(());
    }
    if !bundle_auto_short_term_capture_enabled(&output)? {
        return Ok(());
    }

    let Some(content) = render_compaction_checkpoint_content(packet) else {
        return Ok(());
    };

    let response = checkpoint_with_bundle_defaults(
        &CheckpointArgs {
            output: output.clone(),
            project: packet.session.project.clone(),
            namespace: None,
            workspace: None,
            visibility: None,
            source_path: Some("compaction".to_string()),
            confidence: Some(0.85),
            ttl_seconds: Some(3_600),
            tag: vec!["compaction".to_string(), "auto-checkpoint".to_string()],
            content: Some(content),
            input: None,
            stdin: false,
        },
        base_url,
    )
    .await?;

    let snapshot = read_bundle_resume(
        &ResumeArgs {
            output,
            project: packet.session.project.clone(),
            namespace: None,
            agent: packet.session.agent.clone(),
            workspace: None,
            visibility: None,
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
    .await?;
    write_bundle_memory_files(
        &snapshot_bundle_root(&response, &snapshot),
        &snapshot,
        None,
        false,
    )
    .await?;
    Ok(())
}

pub(crate) fn snapshot_bundle_root(
    _response: &memd_schema::StoreMemoryResponse,
    _snapshot: &ResumeSnapshot,
) -> PathBuf {
    resolve_default_bundle_root()
        .ok()
        .flatten()
        .unwrap_or_else(|| PathBuf::from(".memd"))
}

pub(crate) fn render_compaction_checkpoint_content(packet: &CompactionPacket) -> Option<String> {
    let mut lines = Vec::new();

    if !packet.session.task.trim().is_empty() {
        lines.push(format!("task: {}", packet.session.task.trim()));
    }
    if !packet.goal.trim().is_empty() {
        lines.push(format!("goal: {}", packet.goal.trim()));
    }
    if !packet.active_work.is_empty() {
        lines.push(format!(
            "active: {}",
            packet
                .active_work
                .iter()
                .take(3)
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    if !packet.next_actions.is_empty() {
        lines.push(format!(
            "next: {}",
            packet
                .next_actions
                .iter()
                .take(3)
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    if !packet.do_not_drop.is_empty() {
        lines.push(format!(
            "keep: {}",
            packet
                .do_not_drop
                .iter()
                .take(2)
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }

    let content = lines.join("\n");
    if content.trim().is_empty() {
        None
    } else {
        Some(content)
    }
}

pub(crate) fn checkpoint_as_remember_args(args: &CheckpointArgs) -> RememberArgs {
    let mut tags = vec!["checkpoint".to_string(), "current-task".to_string()];
    for tag in &args.tag {
        if !tags.iter().any(|existing| existing == tag) {
            tags.push(tag.clone());
        }
    }

    RememberArgs {
        output: args.output.clone(),
        project: args.project.clone(),
        namespace: args.namespace.clone(),
        workspace: args.workspace.clone(),
        visibility: args.visibility.clone(),
        kind: Some("status".to_string()),
        scope: Some("project".to_string()),
        source_agent: None,
        source_system: Some("memd-short-term".to_string()),
        source_path: args.source_path.clone(),
        source_quality: Some("derived".to_string()),
        confidence: args.confidence.or(Some(0.8)),
        ttl_seconds: args.ttl_seconds.or(Some(86_400)),
        tag: tags,
        supersede: Vec::new(),
        content: args.content.clone(),
        input: args.input.clone(),
        stdin: args.stdin,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_bundle_identity_defaults_bind_repo_without_runtime_config() {
        let temp_root =
            std::env::temp_dir().join(format!("memd-checkpoint-defaults-{}", uuid::Uuid::new_v4()));
        let repo_root = temp_root.join("repo-b");
        let bundle_root = repo_root.join(".memd");

        fs::create_dir_all(repo_root.join(".git")).expect("create repo git dir");

        let (project, namespace) = infer_bundle_identity_defaults(&bundle_root);
        assert_eq!(project.as_deref(), Some("repo-b"));
        assert_eq!(namespace.as_deref(), Some("main"));

        fs::remove_dir_all(temp_root).expect("cleanup temp root");
    }
}
