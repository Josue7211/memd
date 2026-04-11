use crate::obsidian;
use crate::*;

pub(crate) async fn run_obsidian_compile(
    client: &MemdClient,
    args: &ObsidianArgs,
) -> anyhow::Result<()> {
    let update_compiled_index = |output_path: &Path,
                                 title: &str,
                                 entry_kind: &str,
                                 item_count: usize|
     -> anyhow::Result<()> {
        let index_path = obsidian::default_compiled_index_path(&args.vault);
        let existing_index = fs::read_to_string(&index_path).ok();
        let index_markdown = obsidian::build_compiled_index_markdown(
            existing_index.as_deref(),
            entry_kind,
            title,
            output_path,
            item_count,
        );
        if existing_index.as_deref() != Some(index_markdown.as_str()) {
            obsidian::write_markdown(&index_path, &index_markdown)?;
        }
        Ok(())
    };

    let (title, markdown, output_path, preview, index_items, index_kind, semantic_sync_items) =
        if let Some(id) = args.id.as_ref() {
            let id = id
                .parse::<uuid::Uuid>()
                .context("parse obsidian compile id")?;
            let output_path = args
                .output
                .clone()
                .or_else(|| obsidian::find_compiled_memory_path_by_id(&args.vault, id));
            if let Some(output_path) = output_path
                .as_ref()
                .filter(|path| path.exists() && !args.overwrite)
            {
                let (artifact_title, _) = obsidian::read_compiled_artifact_metadata(output_path)?;
                let title = artifact_title.unwrap_or_else(|| {
                    output_path
                        .file_stem()
                        .and_then(|value| value.to_str())
                        .unwrap_or("compiled-memory")
                        .to_string()
                });
                if args.apply {
                    update_compiled_index(output_path, &title, "memory", 1)?;
                }
                let preview = serde_json::json!({
                    "output_path": output_path.display().to_string(),
                    "open_uri": obsidian::build_open_uri(output_path, args.pane_type.as_deref())?,
                    "title": title,
                    "id": id,
                    "kind": "memory",
                    "rehydration": 0usize,
                    "semantic_synced": 0usize,
                    "apply": args.apply,
                    "reused_existing": true,
                    "compiled_source": "existing_artifact",
                });
                if args.open {
                    let uri = obsidian::build_open_uri(output_path, args.pane_type.as_deref())?;
                    obsidian::open_uri(&uri)?;
                }
                print_json(&preview)?;
                return Ok(());
            }
            let explain = client
                .explain(&ExplainMemoryRequest {
                    id,
                    belief_branch: None,
                    route: parse_retrieval_route(args.route.clone())?,
                    intent: parse_retrieval_intent(args.intent.clone())?,
                })
                .await?;
            let output_path = args
                .output
                .clone()
                .unwrap_or_else(|| obsidian::default_compiled_memory_path(&args.vault, &explain));
            let (title, markdown) = obsidian::build_compiled_memory_markdown(&args.vault, &explain);
            let preview = serde_json::json!({
                "output_path": output_path.display().to_string(),
                "open_uri": obsidian::build_open_uri(&output_path, args.pane_type.as_deref())?,
                "title": title,
                "id": explain.item.id,
                "kind": format!("{:?}", explain.item.kind).to_lowercase(),
                "rehydration": explain.rehydration.len(),
                "apply": args.apply,
                "reused_existing": false,
                "compiled_source": "explained_memory",
            });
            (
                title,
                markdown,
                output_path,
                preview,
                1usize,
                "memory",
                Some(vec![explain.item.clone()]),
            )
        } else {
            let Some(query) = args.query.as_ref() else {
                anyhow::bail!("obsidian compile requires --query <text> or --id <uuid>");
            };
            let output_path = args
                .output
                .clone()
                .unwrap_or_else(|| obsidian::default_compiled_note_path(&args.vault, query));
            if output_path.exists() && !args.overwrite {
                let (artifact_title, item_count) =
                    obsidian::read_compiled_artifact_metadata(&output_path)?;
                let title = artifact_title.unwrap_or_else(|| query.trim().to_string());
                let items = item_count.unwrap_or(0);
                if args.apply {
                    update_compiled_index(&output_path, &title, "query", items)?;
                }
                let preview = serde_json::json!({
                    "output_path": output_path.display().to_string(),
                    "open_uri": obsidian::build_open_uri(&output_path, args.pane_type.as_deref())?,
                    "title": title,
                    "query": query,
                    "items": items,
                    "semantic_hits": 0usize,
                    "semantic_synced": 0usize,
                    "apply": args.apply,
                    "reused_existing": true,
                    "compiled_source": "existing_artifact",
                });
                if args.open {
                    let uri = obsidian::build_open_uri(&output_path, args.pane_type.as_deref())?;
                    obsidian::open_uri(&uri)?;
                }
                print_json(&preview)?;
                return Ok(());
            }
            let route = parse_retrieval_route(args.route.clone())?;
            let intent = parse_retrieval_intent(args.intent.clone())?;
            let response = client
                .search(&SearchMemoryRequest {
                    query: Some(query.clone()),
                    route,
                    intent,
                    scopes: vec![
                        MemoryScope::Project,
                        MemoryScope::Global,
                        MemoryScope::Synced,
                    ],
                    kinds: vec![],
                    statuses: vec![
                        MemoryStatus::Active,
                        MemoryStatus::Stale,
                        MemoryStatus::Contested,
                    ],
                    project: args.project.clone(),
                    namespace: args.namespace.clone(),
                    workspace: args.workspace.clone(),
                    visibility: args
                        .visibility
                        .as_deref()
                        .map(parse_memory_visibility_value)
                        .transpose()?,
                    belief_branch: None,
                    source_agent: None,
                    tags: Vec::new(),
                    stages: vec![MemoryStage::Canonical, MemoryStage::Candidate],
                    limit: Some(args.limit.unwrap_or(12).clamp(1, 48)),
                    max_chars_per_item: Some(800),
                })
                .await?;
            let semantic = if let Some(rag) = maybe_rag_client_from_bundle_or_env()? {
                rag.retrieve(&RagRetrieveRequest {
                    query: query.clone(),
                    project: args.project.clone(),
                    namespace: args.namespace.clone(),
                    mode: RagRetrieveMode::Auto,
                    limit: Some(6),
                    include_cross_modal: false,
                })
                .await
                .ok()
                .filter(|response| !response.items.is_empty())
            } else {
                None
            };
            let (title, markdown) = obsidian::build_compiled_note_markdown(
                &args.vault,
                query,
                &response,
                semantic.as_ref(),
            );
            let preview = serde_json::json!({
                "output_path": output_path.display().to_string(),
                "open_uri": obsidian::build_open_uri(&output_path, args.pane_type.as_deref())?,
                "title": title,
                "query": query,
                "items": response.items.len(),
                "semantic_hits": semantic.as_ref().map(|response| response.items.len()).unwrap_or(0),
                "semantic_synced": 0usize,
                "apply": args.apply,
                "reused_existing": false,
                "compiled_source": "compiled_query",
            });
            (
                title,
                markdown,
                output_path,
                preview,
                response.items.len(),
                "query",
                Some(response.items.clone()),
            )
        };

    let mut preview = preview;
    if !args.apply {
        print_json(&preview)?;
        return Ok(());
    }

    if output_path.exists() && !args.overwrite {
        anyhow::bail!(
            "{} already exists; pass --overwrite to replace it",
            output_path.display()
        );
    }
    obsidian::write_markdown(&output_path, &markdown)?;
    update_compiled_index(&output_path, &title, index_kind, index_items)?;
    let semantic_synced = if let (Some(rag), Some(items)) = (
        maybe_rag_client_from_bundle_or_env()?,
        semantic_sync_items.as_ref(),
    ) {
        sync_memory_items_to_rag(&rag, items).await?
    } else {
        0
    };
    if let Some(map) = preview.as_object_mut() {
        map.insert("semantic_synced".to_string(), json!(semantic_synced));
    }
    if args.open {
        let uri = obsidian::build_open_uri(&output_path, args.pane_type.as_deref())?;
        obsidian::open_uri(&uri)?;
    }
    print_json(&preview)?;
    Ok(())
}
