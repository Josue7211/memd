use super::*;

pub(crate) async fn run_obsidian_import(
    client: &MemdClient,
    args: &ObsidianArgs,
    sync_mode: bool,
    mirror_mode: bool,
) -> anyhow::Result<()> {
    let include_attachments = args.include_attachments || sync_mode;
    let link_notes = args.link_notes || sync_mode;
    let apply = args.apply || sync_mode;

    let scan = obsidian::scan_vault(
        &args.vault,
        args.project.clone(),
        args.namespace.clone(),
        args.workspace.clone(),
        args.visibility
            .as_deref()
            .map(parse_memory_visibility_value)
            .transpose()?,
        args.max_notes,
        include_attachments,
        args.max_attachments,
        &args.include_folder,
        &args.exclude_folder,
        &args.include_tag,
        &args.exclude_tag,
    )?;
    if args.review_sensitive {
        println!("{}", obsidian::render_sensitive_review(&scan));
        return Ok(());
    }
    let (state_path, sync_state) = obsidian::load_sync_state(&args.vault, args.state_file.clone())?;
    let (preview, _candidates, changed_notes) =
        obsidian::build_import_preview(scan, &sync_state, state_path.clone());
    let (attachment_assets, attachment_unchanged_count) = if include_attachments {
        obsidian::partition_changed_attachments(&preview.scan.attachments, &sync_state)
    } else {
        (Vec::new(), 0)
    };

    if apply {
        let mut next_state = sync_state.clone();
        let mut submitted = 0usize;
        let mut duplicates = 0usize;
        let mut note_failures = 0usize;
        for note in &changed_notes {
            let request = obsidian::build_note_request(
                note,
                args.project.clone(),
                args.namespace.clone(),
                args.workspace.clone(),
                args.visibility
                    .as_deref()
                    .map(parse_memory_visibility_value)
                    .transpose()?,
                preview.scan.vault.clone(),
                next_state
                    .entries
                    .get(&note.relative_path)
                    .and_then(|entry| entry.item_id),
            );
            let response = match client.candidate(&request).await {
                Ok(response) => response,
                Err(err) => {
                    note_failures += 1;
                    eprintln!(
                        "obsidian note import failed for {}: {err:#}",
                        note.relative_path
                    );
                    continue;
                }
            };
            let stored_id = response.duplicate_of.unwrap_or(response.item.id);
            let entity_id = match client
                .entity(&memd_schema::EntityMemoryRequest {
                    id: stored_id,
                    route: None,
                    intent: None,
                    limit: Some(4),
                })
                .await
            {
                Ok(entity) => entity.entity.as_ref().map(|entity| entity.id),
                Err(err) => {
                    note_failures += 1;
                    eprintln!(
                        "obsidian entity lookup failed for {}: {err:#}",
                        note.relative_path
                    );
                    None
                }
            };
            next_state.entries.insert(
                note.relative_path.clone(),
                ObsidianSyncEntry {
                    content_hash: note.content_hash.clone(),
                    bytes: note.bytes,
                    modified_at: note.modified_at,
                    item_id: Some(stored_id),
                    entity_id,
                },
            );
            submitted += 1;
            if response.duplicate_of.is_some() {
                duplicates += 1;
            }
            obsidian::save_sync_state(&state_path, &next_state)?;
        }

        let mut attachment_multimodal = None;
        let mut attachment_submitted = 0usize;
        let mut attachment_duplicates = 0usize;
        let mut attachment_failures = 0usize;
        if include_attachments && !attachment_assets.is_empty() {
            let attachment_paths = attachment_assets
                .iter()
                .map(|asset| asset.path.clone())
                .collect::<Vec<_>>();
            let multimodal_preview = build_multimodal_preview(
                args.project.clone(),
                args.namespace.clone(),
                &attachment_paths,
            )?;
            let rag_url = resolve_rag_url(None, resolve_default_bundle_root()?.as_deref())?;
            let sidecar = SidecarClient::new(&rag_url)?;
            let mut multimodal_responses = Vec::with_capacity(attachment_assets.len());
            let mut ingested_attachment_pairs = Vec::with_capacity(attachment_assets.len());
            for (asset, request) in attachment_assets
                .iter()
                .zip(multimodal_preview.requests.iter())
            {
                let response = match sidecar.ingest(request).await {
                    Ok(response) => response,
                    Err(err) => {
                        attachment_failures += 1;
                        eprintln!(
                            "obsidian attachment ingest failed for {}: {err:#}",
                            asset.relative_path
                        );
                        continue;
                    }
                };
                multimodal_responses.push(response.clone());
                ingested_attachment_pairs.push((asset, response));
            }
            attachment_multimodal = Some(MultimodalIngestOutput {
                preview: multimodal_preview,
                responses: multimodal_responses,
                submitted: ingested_attachment_pairs.len(),
                dry_run: false,
            });

            for (asset, response) in ingested_attachment_pairs {
                let match_ = obsidian::resolve_attachment_match(
                    asset,
                    &preview.scan.notes,
                    &preview.note_index,
                );
                let linked_note = match_
                    .as_ref()
                    .and_then(|association| preview.scan.notes.get(association.note_index));
                let attachment_candidate = obsidian::build_attachment_request(
                    asset,
                    args.project.clone(),
                    args.namespace.clone(),
                    args.workspace.clone(),
                    args.visibility
                        .as_deref()
                        .map(parse_memory_visibility_value)
                        .transpose()?,
                    preview.scan.vault.clone(),
                    linked_note,
                    Some(response.track_id),
                );
                let attachment_response = match client.candidate(&attachment_candidate).await {
                    Ok(response) => response,
                    Err(err) => {
                        attachment_failures += 1;
                        eprintln!(
                            "obsidian attachment import failed for {}: {err:#}",
                            asset.relative_path
                        );
                        continue;
                    }
                };
                attachment_submitted += 1;
                if attachment_response.duplicate_of.is_some() {
                    attachment_duplicates += 1;
                }
                let stored_id = attachment_response
                    .duplicate_of
                    .unwrap_or(attachment_response.item.id);
                let entity_id = match client
                    .entity(&memd_schema::EntityMemoryRequest {
                        id: stored_id,
                        route: None,
                        intent: None,
                        limit: Some(4),
                    })
                    .await
                {
                    Ok(entity) => entity.entity.as_ref().map(|entity| entity.id),
                    Err(err) => {
                        attachment_failures += 1;
                        eprintln!(
                            "obsidian attachment entity lookup failed for {}: {err:#}",
                            asset.relative_path
                        );
                        None
                    }
                };
                next_state.entries.insert(
                    asset.relative_path.clone(),
                    ObsidianSyncEntry {
                        content_hash: asset.content_hash.clone(),
                        bytes: asset.bytes,
                        modified_at: asset.modified_at,
                        item_id: Some(stored_id),
                        entity_id,
                    },
                );
                obsidian::save_sync_state(&state_path, &next_state)?;
            }
        }

        let mut entity_ids_by_item_id = std::collections::HashMap::new();
        for entry in next_state.entries.values() {
            let Some(item_id) = entry.item_id else {
                continue;
            };
            if entity_ids_by_item_id.contains_key(&item_id) {
                continue;
            }
            if let Some(entity_id) = entry.entity_id {
                entity_ids_by_item_id.insert(item_id, entity_id);
                continue;
            }
            let entity = client
                .entity(&memd_schema::EntityMemoryRequest {
                    id: item_id,
                    route: None,
                    intent: None,
                    limit: Some(4),
                })
                .await?;
            if let Some(entity) = entity.entity {
                entity_ids_by_item_id.insert(item_id, entity.id);
            }
        }

        obsidian::save_sync_state(&state_path, &next_state)?;

        let mut links_created = 0usize;
        if link_notes {
            for note in &preview.scan.notes {
                let Some(from_entity_id) = next_state
                    .entries
                    .get(&note.relative_path)
                    .and_then(|entry| entry.item_id)
                    .and_then(|item_id| entity_ids_by_item_id.get(&item_id).copied())
                else {
                    continue;
                };
                for target in &note.links {
                    let target_key = obsidian::normalized_title(target);
                    let Some(target_idx) = preview.note_index.get(&target_key) else {
                        continue;
                    };
                    let target_note = &preview.scan.notes[*target_idx];
                    let Some(to_entity_id) = next_state
                        .entries
                        .get(&target_note.relative_path)
                        .and_then(|entry| entry.item_id)
                        .and_then(|item_id| entity_ids_by_item_id.get(&item_id).copied())
                    else {
                        continue;
                    };
                    if from_entity_id == to_entity_id {
                        continue;
                    }
                    let request =
                        obsidian::build_entity_link_request(from_entity_id, to_entity_id, note);
                    let _ = client.link_entity(&request).await?;
                    links_created += 1;
                }
            }
        }

        let mut attachment_links_created = 0usize;
        if include_attachments && !attachment_assets.is_empty() {
            for asset in &attachment_assets {
                let Some(match_) = obsidian::resolve_attachment_match(
                    asset,
                    &preview.scan.notes,
                    &preview.note_index,
                ) else {
                    continue;
                };
                let Some(attachment_entry) = next_state.entries.get(&asset.relative_path) else {
                    continue;
                };
                let Some(attachment_item_id) = attachment_entry.item_id else {
                    continue;
                };
                let Some(attachment_entity_id) =
                    entity_ids_by_item_id.get(&attachment_item_id).copied()
                else {
                    continue;
                };
                let Some(note) = preview.scan.notes.get(match_.note_index) else {
                    continue;
                };
                let Some(note_entry) = next_state.entries.get(&note.relative_path) else {
                    continue;
                };
                let Some(note_item_id) = note_entry.item_id else {
                    continue;
                };
                let Some(note_entity_id) = entity_ids_by_item_id.get(&note_item_id).copied() else {
                    continue;
                };
                if attachment_entity_id == note_entity_id {
                    continue;
                }
                let request = memd_schema::EntityLinkRequest {
                    from_entity_id: attachment_entity_id,
                    to_entity_id: note_entity_id,
                    relation_kind: match_.relation_kind,
                    confidence: Some(0.78),
                    note: Some(format!("obsidian attachment from {}", asset.relative_path)),
                    context: Some(memd_schema::MemoryContextFrame {
                        at: Some(chrono::Utc::now()),
                        project: args.project.clone(),
                        namespace: args.namespace.clone(),
                        workspace: args.workspace.clone(),
                        repo: Some("obsidian".to_string()),
                        host: None,
                        branch: None,
                        agent: Some("obsidian".to_string()),
                        location: Some(asset.relative_path.clone()),
                    }),
                    tags: vec![
                        "obsidian".to_string(),
                        "vault_attachment".to_string(),
                        format!("linked_note={}", note.normalized_title),
                        format!("reason={}", match_.reason),
                    ],
                };
                let _ = client.link_entity(&request).await?;
                attachment_links_created += 1;
            }
        }

        let mut mirrored_notes = 0usize;
        let mut mirrored_attachments = 0usize;
        if mirror_mode {
            for note in &preview.scan.notes {
                let Some(entry) = next_state.entries.get(&note.relative_path) else {
                    continue;
                };
                let Some(item_id) = entry.item_id else {
                    continue;
                };
                let entity_id = if let Some(entity_id) = entry.entity_id {
                    Some(entity_id)
                } else {
                    let entity = client
                        .entity(&memd_schema::EntityMemoryRequest {
                            id: item_id,
                            route: None,
                            intent: None,
                            limit: Some(4),
                        })
                        .await?;
                    entity.entity.as_ref().map(|entity| entity.id)
                };
                let block = obsidian::build_roundtrip_annotation(
                    note,
                    Some(item_id),
                    entity_id,
                    args.workspace.as_deref(),
                    args.visibility
                        .as_deref()
                        .map(parse_memory_visibility_value)
                        .transpose()?,
                );
                obsidian::annotate_note(&note.path, &block)?;
                let (_, mirror_markdown) = obsidian::build_note_mirror_markdown(
                    note,
                    Some(item_id),
                    entity_id,
                    args.workspace.as_deref(),
                    args.visibility
                        .as_deref()
                        .map(parse_memory_visibility_value)
                        .transpose()?,
                );
                let mirror_path = obsidian::note_mirror_path(&preview.scan.vault, note);
                obsidian::write_markdown(&mirror_path, &mirror_markdown)?;
                mirrored_notes += 1;
            }

            if include_attachments {
                for asset in &attachment_assets {
                    let Some(entry) = next_state.entries.get(&asset.relative_path) else {
                        continue;
                    };
                    let Some(item_id) = entry.item_id else {
                        continue;
                    };
                    let entity_id = if let Some(entity_id) = entry.entity_id {
                        Some(entity_id)
                    } else {
                        let entity = client
                            .entity(&memd_schema::EntityMemoryRequest {
                                id: item_id,
                                route: None,
                                intent: None,
                                limit: Some(4),
                            })
                            .await?;
                        entity.entity.as_ref().map(|entity| entity.id)
                    };
                    let linked_note = obsidian::resolve_attachment_match(
                        asset,
                        &preview.scan.notes,
                        &preview.note_index,
                    )
                    .and_then(|association| preview.scan.notes.get(association.note_index));
                    let (_, mirror_markdown) = obsidian::build_attachment_mirror_markdown(
                        asset,
                        Some(item_id),
                        entity_id,
                        linked_note,
                        None,
                        args.workspace.as_deref(),
                        args.visibility
                            .as_deref()
                            .map(parse_memory_visibility_value)
                            .transpose()?,
                    );
                    let mirror_path = obsidian::attachment_mirror_path(&preview.scan.vault, asset);
                    obsidian::write_markdown(&mirror_path, &mirror_markdown)?;
                    mirrored_attachments += 1;
                }
            }
        }

        let output = ObsidianImportOutput {
            preview,
            submitted,
            attachment_submitted,
            duplicates,
            attachment_duplicates,
            note_failures,
            attachment_failures,
            links_created,
            attachment_links_created,
            mirrored_notes,
            mirrored_attachments,
            attachments: attachment_multimodal,
            attachment_unchanged_count,
            dry_run: false,
        };
        if args.summary {
            println!("{}", render_obsidian_import_summary(&output, args.follow));
        } else {
            print_json(&output)?;
        }
    } else {
        let output = ObsidianImportOutput {
            preview,
            submitted: 0,
            attachment_submitted: 0,
            duplicates: 0,
            attachment_duplicates: 0,
            note_failures: 0,
            attachment_failures: 0,
            links_created: 0,
            attachment_links_created: 0,
            mirrored_notes: 0,
            mirrored_attachments: 0,
            attachments: None,
            attachment_unchanged_count,
            dry_run: true,
        };
        if args.summary {
            println!("{}", render_obsidian_import_summary(&output, args.follow));
        } else {
            print_json(&output)?;
        }
    }

    Ok(())
}

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

pub(crate) async fn run_obsidian_open(
    client: &MemdClient,
    args: &ObsidianArgs,
) -> anyhow::Result<()> {
    let target_path = if let Some(note) = args.note.as_ref() {
        obsidian::resolve_open_path(&args.vault, note)
    } else if let Some(id) = args.id.as_ref() {
        let id = id.parse::<uuid::Uuid>().context("parse obsidian open id")?;
        let explain = client
            .explain(&ExplainMemoryRequest {
                id,
                belief_branch: None,
                route: None,
                intent: None,
            })
            .await?;
        args.output
            .clone()
            .unwrap_or_else(|| obsidian::default_writeback_path(&args.vault, &explain))
    } else if let Some(output) = args.output.as_ref() {
        obsidian::resolve_open_path(&args.vault, output)
    } else {
        args.vault.clone()
    };

    let uri = obsidian::build_open_uri(&target_path, args.pane_type.as_deref())?;
    let preview = serde_json::json!({
        "vault": args.vault.display().to_string(),
        "target_path": target_path.display().to_string(),
        "open_uri": uri,
        "apply": args.apply,
    });

    if !args.apply {
        print_json(&preview)?;
        return Ok(());
    }

    obsidian::open_uri(preview["open_uri"].as_str().unwrap_or_default())?;
    print_json(&preview)?;
    Ok(())
}

pub(crate) async fn run_obsidian_status(
    _client: &MemdClient,
    args: &ObsidianArgs,
) -> anyhow::Result<()> {
    let scan = obsidian::scan_vault(
        &args.vault,
        args.project.clone(),
        args.namespace.clone(),
        args.workspace.clone(),
        args.visibility
            .as_deref()
            .map(parse_memory_visibility_value)
            .transpose()?,
        args.max_notes,
        args.include_attachments,
        args.max_attachments,
        &args.include_folder,
        &args.exclude_folder,
        &args.include_tag,
        &args.exclude_tag,
    )?;
    let (state_path, sync_state) = obsidian::load_sync_state(&args.vault, args.state_file.clone())?;
    let (preview, _, _) = obsidian::build_import_preview(scan, &sync_state, state_path.clone());
    let attachment_assets = if args.include_attachments {
        obsidian::partition_changed_attachments(&preview.scan.attachments, &sync_state).0
    } else {
        Vec::new()
    };
    let cache_requests = preview.scan.note_count + preview.scan.attachment_count;
    let cache_hits = preview.scan.cache_hits + preview.scan.attachment_cache_hits;
    let cache_health = if cache_requests == 0 {
        "empty"
    } else if cache_hits == 0 {
        "cold"
    } else if cache_hits * 2 >= cache_requests {
        "hot"
    } else {
        "warm"
    };
    let mirror_notes = count_obsidian_mirrors(&args.vault, "notes")?;
    let mirror_attachments = count_obsidian_mirrors(&args.vault, "attachments")?;
    let sync_state_entries = sync_state.entries.len();
    let changed_notes = preview.candidates.len();
    let unchanged_notes = preview.unchanged_count;
    let changed_attachments = attachment_assets.len();
    let unchanged_attachments = preview.scan.attachment_unchanged_count;
    let roundtrip_live = sync_state_entries > 0 || mirror_notes > 0 || mirror_attachments > 0;
    let mut summary = format!(
        "obsidian_status vault={} notes={} changed_notes={} unchanged_notes={} attachments={} changed_attachments={} unchanged_attachments={} cache_health={} cache_hits={} attachment_cache_hits={} cache_pruned={} attachment_cache_pruned={} sync_entries={} mirrors_notes={} mirrors_attachments={} roundtrip_live={} state={}",
        args.vault.display(),
        preview.scan.note_count,
        changed_notes,
        unchanged_notes,
        preview.scan.attachment_count,
        changed_attachments,
        unchanged_attachments,
        cache_health,
        preview.scan.cache_hits,
        preview.scan.attachment_cache_hits,
        preview.scan.cache_pruned,
        preview.scan.attachment_cache_pruned,
        sync_state_entries,
        mirror_notes,
        mirror_attachments,
        roundtrip_live,
        state_path.display()
    );
    if args.follow {
        let trail = preview
            .scan
            .notes
            .iter()
            .take(3)
            .map(|note| note.title.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        if !trail.is_empty() {
            summary.push_str(&format!(" trail={trail}"));
        }
    }
    if args.summary {
        println!("{summary}");
    } else {
        print_json(&serde_json::json!({
            "vault": preview.scan.vault,
            "project": preview.scan.project,
            "namespace": preview.scan.namespace,
            "notes": preview.scan.note_count,
            "changed_notes": changed_notes,
            "unchanged_notes": unchanged_notes,
            "attachments": preview.scan.attachment_count,
            "changed_attachments": changed_attachments,
            "unchanged_attachments": unchanged_attachments,
            "cache_health": cache_health,
            "cache_hits": preview.scan.cache_hits,
            "attachment_cache_hits": preview.scan.attachment_cache_hits,
            "cache_pruned": preview.scan.cache_pruned,
            "attachment_cache_pruned": preview.scan.attachment_cache_pruned,
            "sync_state_entries": sync_state_entries,
            "mirror_notes": mirror_notes,
            "mirror_attachments": mirror_attachments,
            "roundtrip_live": roundtrip_live,
            "state_path": state_path,
        }))?;
    }
    Ok(())
}

pub(crate) async fn run_obsidian_watch(
    client: &MemdClient,
    args: &ObsidianArgs,
) -> anyhow::Result<()> {
    println!(
        "obsidian_watch vault={} debounce_ms={}",
        args.vault.display(),
        args.debounce_ms
    );

    run_obsidian_import(client, args, true, true).await?;

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();
    let mut watcher = RecommendedWatcher::new(
        move |result: notify::Result<notify::Event>| {
            if let Ok(event) = result {
                let should_trigger = matches!(
                    event.kind,
                    EventKind::Create(_)
                        | EventKind::Modify(_)
                        | EventKind::Remove(_)
                        | EventKind::Any
                ) && event
                    .paths
                    .iter()
                    .any(|path| !obsidian_path_is_internal(path));
                if should_trigger {
                    let _ = tx.send(());
                }
            }
        },
        NotifyConfig::default(),
    )
    .context("create obsidian watcher")?;
    watcher
        .watch(&args.vault, RecursiveMode::Recursive)
        .with_context(|| format!("watch {}", args.vault.display()))?;

    let debounce = Duration::from_millis(args.debounce_ms.max(100));
    loop {
        if rx.recv().await.is_none() {
            break;
        }

        let mut dirty = true;
        while dirty {
            dirty = false;
            tokio::time::sleep(debounce).await;
            while rx.try_recv().is_ok() {
                dirty = true;
            }
        }

        if let Err(err) = run_obsidian_import(client, args, true, true).await {
            eprintln!("obsidian watch sync failed: {err:#}");
        }
    }

    Ok(())
}
