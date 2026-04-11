use super::*;

pub(crate) async fn lookup_with_fallbacks(
    client: &MemdClient,
    req: &SearchMemoryRequest,
    query: &str,
) -> anyhow::Result<memd_schema::SearchMemoryResponse> {
    let response = client.search(req).await?;
    if !response.items.is_empty() {
        return Ok(response);
    }

    for candidate_query in supersede_query_candidates(query).into_iter().skip(1) {
        let fallback = client
            .search(&SearchMemoryRequest {
                query: Some(candidate_query.clone()),
                ..req.clone()
            })
            .await?;
        if !fallback.items.is_empty() {
            return Ok(fallback);
        }
    }

    let broad_fallback = client
        .search(&SearchMemoryRequest {
            query: None,
            ..req.clone()
        })
        .await?;
    if !broad_fallback.items.is_empty() {
        return Ok(broad_fallback);
    }

    Ok(response)
}

pub(crate) fn build_lookup_request(
    args: &LookupArgs,
    runtime: Option<&BundleRuntimeConfig>,
) -> anyhow::Result<SearchMemoryRequest> {
    let project = args
        .project
        .clone()
        .or_else(|| runtime.and_then(|config| config.project.clone()));
    let namespace = args
        .namespace
        .clone()
        .or_else(|| runtime.and_then(|config| config.namespace.clone()));
    let workspace = args
        .workspace
        .clone()
        .or_else(|| runtime.and_then(|config| config.workspace.clone()));
    let visibility = args
        .visibility
        .clone()
        .or_else(|| runtime.and_then(|config| config.visibility.clone()))
        .as_deref()
        .map(parse_memory_visibility_value)
        .transpose()?;
    let route = parse_retrieval_route(args.route.clone().or(Some("project_first".to_string())))?;
    let intent = parse_retrieval_intent(args.intent.clone().or(Some("general".to_string())))?;
    let kinds = if args.kind.is_empty() {
        vec![
            MemoryKind::Decision,
            MemoryKind::Preference,
            MemoryKind::Fact,
            MemoryKind::Constraint,
            MemoryKind::Runbook,
            MemoryKind::Procedural,
            MemoryKind::Status,
        ]
    } else {
        args.kind
            .iter()
            .map(|value| parse_memory_kind_value(value))
            .collect::<anyhow::Result<Vec<_>>>()?
    };
    let mut statuses = vec![MemoryStatus::Active];
    if args.include_stale {
        statuses.push(MemoryStatus::Stale);
        statuses.push(MemoryStatus::Contested);
    }

    Ok(SearchMemoryRequest {
        query: Some(args.query.clone()),
        route,
        intent,
        scopes: vec![
            MemoryScope::Project,
            MemoryScope::Synced,
            MemoryScope::Global,
        ],
        kinds,
        statuses,
        project,
        namespace,
        workspace,
        visibility,
        belief_branch: None,
        source_agent: None,
        tags: args.tag.clone(),
        stages: vec![MemoryStage::Canonical],
        limit: args.limit.or(Some(6)),
        max_chars_per_item: Some(280),
    })
}

pub(crate) fn render_lookup_markdown(
    query: &str,
    response: &memd_schema::SearchMemoryResponse,
    verbose: bool,
) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd lookup\n\n");
    markdown.push_str(&format!("- query: {}\n", query));
    markdown.push_str(&format!("- matches: {}\n\n", response.items.len()));

    if response.items.is_empty() {
        markdown.push_str("No durable memory matched.\n");
        return markdown;
    }

    markdown.push_str("## Matches\n\n");
    let limit = if verbose { 6 } else { 3 };
    for item in response.items.iter().take(limit) {
        markdown.push_str(&format!(
            "- [{}] {} ({}, {}, {:.2})\n",
            short_uuid(item.id),
            item.content.replace('\n', " "),
            enum_label_kind(item.kind),
            enum_label_status(item.status),
            item.confidence
        ));
    }

    markdown
        .push_str("\n- Use recalled items before answering; correct memory if they conflict.\n");
    markdown
}

pub(crate) fn enum_label_kind(kind: MemoryKind) -> &'static str {
    match kind {
        MemoryKind::Fact => "fact",
        MemoryKind::Decision => "decision",
        MemoryKind::Preference => "preference",
        MemoryKind::Runbook => "runbook",
        MemoryKind::Procedural => "procedural",
        MemoryKind::SelfModel => "self_model",
        MemoryKind::Topology => "topology",
        MemoryKind::Status => "status",
        MemoryKind::LiveTruth => "live_truth",
        MemoryKind::Pattern => "pattern",
        MemoryKind::Constraint => "constraint",
    }
}

pub(crate) fn enum_label_status(status: MemoryStatus) -> &'static str {
    match status {
        MemoryStatus::Active => "active",
        MemoryStatus::Stale => "stale",
        MemoryStatus::Superseded => "superseded",
        MemoryStatus::Contested => "contested",
        MemoryStatus::Expired => "expired",
    }
}

pub(crate) fn resolve_default_bundle_root() -> anyhow::Result<Option<PathBuf>> {
    if let Ok(value) = std::env::var("MEMD_BUNDLE_ROOT") {
        let value = value.trim();
        if !value.is_empty() {
            return Ok(Some(PathBuf::from(value)));
        }
    }

    let global_root = default_global_bundle_root();
    if global_root.join("config.json").exists() {
        return Ok(Some(global_root));
    }

    let cwd = std::env::current_dir().context("read current directory")?;
    let bundle_root = cwd.join(".memd");
    if bundle_root.join("config.json").exists() {
        return Ok(Some(bundle_root));
    }

    Ok(None)
}

pub(crate) fn resolve_rag_url(
    explicit: Option<String>,
    bundle_root: Option<&Path>,
) -> anyhow::Result<String> {
    if let Some(value) = explicit
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return Ok(value);
    }

    if let Some(bundle_root) = bundle_root
        && let Some(config) = read_bundle_rag_config(bundle_root)?
        && config.enabled
    {
        if let Some(url) = config.url {
            return Ok(url);
        }
        anyhow::bail!(
            "rag backend is enabled in {} but no url was configured",
            bundle_root.display()
        );
    }

    if let Ok(value) = std::env::var("MEMD_RAG_URL") {
        let value = value.trim().to_string();
        if !value.is_empty() {
            return Ok(value);
        }
    }

    anyhow::bail!("provide --rag-url, configure rag_url in the bundle, or set MEMD_RAG_URL")
}

pub(crate) fn maybe_rag_client_from_bundle_or_env() -> anyhow::Result<Option<RagClient>> {
    if let Some(bundle_root) = resolve_default_bundle_root()?
        && let Some(config) = read_bundle_rag_config(bundle_root.as_path())?
        && config.enabled
    {
        let rag_url = config.url.with_context(|| {
            format!(
                "rag backend is enabled in {} but no url was configured",
                bundle_root.display()
            )
        })?;
        return Ok(Some(RagClient::new(rag_url)?));
    }

    match std::env::var("MEMD_RAG_URL") {
        Ok(value) if !value.trim().is_empty() => Ok(Some(RagClient::new(value)?)),
        _ => Ok(None),
    }
}

pub(crate) fn maybe_rag_client_for_bundle(output: &Path) -> anyhow::Result<Option<RagClient>> {
    if let Some(config) = read_bundle_rag_config(output)?
        && config.enabled
    {
        let rag_url = config.url.with_context(|| {
            format!(
                "rag backend is enabled in {} but no url was configured",
                output.display()
            )
        })?;
        return Ok(Some(RagClient::new(rag_url)?));
    }

    match std::env::var("MEMD_RAG_URL") {
        Ok(value) if !value.trim().is_empty() => Ok(Some(RagClient::new(value)?)),
        _ => Ok(None),
    }
}

pub(crate) fn build_resume_rag_query(
    project: Option<&str>,
    workspace: Option<&str>,
    intent: &str,
    working: &memd_schema::WorkingMemoryResponse,
    context: &memd_schema::CompactContextResponse,
) -> String {
    let mut parts = Vec::new();

    if let Some(project) = project.filter(|value| !value.trim().is_empty()) {
        parts.push(format!("project: {project}"));
    }
    if let Some(workspace) = workspace.filter(|value| !value.trim().is_empty()) {
        parts.push(format!("workspace: {workspace}"));
    }
    if !intent.trim().is_empty() {
        parts.push(format!("intent: {intent}"));
    }

    for record in working.records.iter().take(2) {
        let value = compact_resume_rag_text(&record.record, 180);
        if !value.is_empty() {
            parts.push(format!("working: {value}"));
        }
    }

    for record in context.records.iter().take(2) {
        let value = compact_resume_rag_text(&record.record, 180);
        if !value.is_empty() {
            parts.push(format!("context: {value}"));
        }
    }

    compact_resume_rag_text(&parts.join(" | "), 700)
}

pub(crate) fn compact_resume_rag_text(input: &str, max_chars: usize) -> String {
    let collapsed = input.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.len() <= max_chars {
        return collapsed;
    }

    let mut output = String::new();
    for ch in collapsed.chars() {
        if output.chars().count() >= max_chars.saturating_sub(1) {
            break;
        }
        output.push(ch);
    }
    output.push('…');
    output
}
