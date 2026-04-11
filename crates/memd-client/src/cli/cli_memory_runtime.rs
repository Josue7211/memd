use super::*;

pub(crate) async fn run_memory_command(
    _client: &MemdClient,
    base_url: &str,
    args: &MemoryArgs,
) -> anyhow::Result<()> {
    let bundle_root = resolve_compiled_memory_bundle_root(args.root.as_deref())?;
    let use_runtime_summary = !args.quality
        && !args.list
        && compiled_memory_target(&args).is_none()
        && args.query.is_none();
    if use_runtime_summary {
        match read_memory_surface(&bundle_root, &base_url).await {
            Ok(response) if args.json => print_json(&response)?,
            Ok(response) => println!("{}", render_memory_surface_summary(&response)),
            Err(_) if !args.json => {
                let page = bundle_compiled_memory_path(&bundle_root, MemoryObjectLane::Working);
                let content = fs::read_to_string(&page)
                    .with_context(|| format!("read {}", page.display()))?;
                println!("{}", render_compiled_memory_page_summary(&page, &content));
            }
            Err(err) => return Err(err),
        }
    } else if args.quality {
        let report = build_compiled_memory_quality_report(&bundle_root)?;
        if args.json {
            print_json(&render_compiled_memory_quality_json(&bundle_root, &report))?;
        } else if args.summary {
            println!(
                "{}",
                render_compiled_memory_quality_summary(&bundle_root, &report)
            );
        } else {
            println!(
                "{}",
                render_compiled_memory_quality_markdown(&bundle_root, &report)
            );
        }
    } else if args.list {
        let index = render_compiled_memory_index(&bundle_root)?;
        let index = filter_compiled_memory_index(
            index,
            args.lanes_only,
            args.items_only,
            args.filter.as_deref(),
        );
        if args.json {
            print_json(&render_compiled_memory_index_json(&bundle_root, &index))?;
        } else if args.summary {
            println!(
                "{}",
                render_compiled_memory_index_summary(&bundle_root, &index)
            );
        } else if args.grouped {
            println!(
                "{}",
                render_compiled_memory_index_grouped_markdown(
                    &bundle_root,
                    &index,
                    args.expand_items,
                )
            );
        } else {
            println!(
                "{}",
                render_compiled_memory_index_markdown(&bundle_root, &index)
            );
        }
    } else if let Some(target) = compiled_memory_target(&args) {
        let path = resolve_compiled_memory_page(&bundle_root, target)?;
        let content =
            fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        if args.summary {
            println!("{}", render_compiled_memory_page_summary(&path, &content));
        } else {
            println!("{}", render_compiled_memory_page_markdown(&path, &content));
        }
    } else if let Some(query) = args.query.as_deref() {
        let matches = search_compiled_memory_pages(&bundle_root, query, args.limit)?;
        if args.summary {
            println!(
                "{}",
                render_compiled_memory_search_summary(&bundle_root, query, &matches)
            );
        } else {
            println!(
                "{}",
                render_compiled_memory_search_markdown(&bundle_root, query, &matches)
            );
        }
    } else {
        let page = bundle_compiled_memory_path(&bundle_root, MemoryObjectLane::Working);
        let content =
            fs::read_to_string(&page).with_context(|| format!("read {}", page.display()))?;
        if args.summary {
            println!("{}", render_compiled_memory_page_summary(&page, &content));
        } else {
            println!("{}", render_compiled_memory_page_markdown(&page, &content));
        }
    }

    Ok(())
}

pub(crate) async fn run_ingest_command(
    client: &MemdClient,
    args: &IngestArgs,
) -> anyhow::Result<()> {
    let result = ingest_auto_route(client, args).await?;
    print_json(&result)?;
    Ok(())
}

pub(crate) async fn run_store_command(
    client: &MemdClient,
    input: &RequestInput,
) -> anyhow::Result<()> {
    let req = read_request::<StoreMemoryRequest>(input)?;
    print_json(&client.store(&req).await?)?;
    Ok(())
}

pub(crate) async fn run_candidate_command(
    client: &MemdClient,
    input: &RequestInput,
) -> anyhow::Result<()> {
    let req = read_request::<CandidateMemoryRequest>(input)?;
    print_json(&client.candidate(&req).await?)?;
    Ok(())
}

pub(crate) async fn run_promote_command(
    client: &MemdClient,
    input: &RequestInput,
) -> anyhow::Result<()> {
    let req = read_request::<PromoteMemoryRequest>(input)?;
    print_json(&client.promote(&req).await?)?;
    Ok(())
}

pub(crate) async fn run_expire_command(
    client: &MemdClient,
    input: &RequestInput,
) -> anyhow::Result<()> {
    let req = read_request::<ExpireMemoryRequest>(input)?;
    print_json(&client.expire(&req).await?)?;
    Ok(())
}

pub(crate) async fn run_memory_verify_command(
    client: &MemdClient,
    input: &RequestInput,
) -> anyhow::Result<()> {
    let req = read_request::<VerifyMemoryRequest>(input)?;
    print_json(&client.verify(&req).await?)?;
    Ok(())
}

pub(crate) async fn run_repair_command(
    client: &MemdClient,
    args: RepairArgs,
) -> anyhow::Result<()> {
    let mode = commands::parse_memory_repair_mode_value(&args.mode)?;
    let status = match args.status.as_deref() {
        Some(value) => Some(commands::parse_memory_status_value(value)?),
        None => None,
    };
    let source_quality = match args.source_quality.as_deref() {
        Some(value) => Some(parse_source_quality_value(value)?),
        None => None,
    };
    let supersedes = parse_uuid_list(&args.supersede)?;
    let response = client
        .repair(&RepairMemoryRequest {
            id: args.id.parse()?,
            mode,
            confidence: args.confidence,
            status,
            workspace: args.workspace.clone(),
            visibility: args
                .visibility
                .as_deref()
                .map(parse_memory_visibility_value)
                .transpose()?,
            source_agent: args.source_agent.clone(),
            source_system: args.source_system.clone(),
            source_path: args.source_path.clone(),
            source_quality,
            content: args.content.clone(),
            tags: if args.tag.is_empty() {
                None
            } else {
                Some(args.tag.clone())
            },
            supersedes,
        })
        .await?;
    if args.summary {
        println!("{}", render_repair_summary(&response, args.follow));
    } else {
        print_json(&response)?;
    }
    Ok(())
}

pub(crate) async fn run_search_command(
    client: &MemdClient,
    args: SearchArgs,
) -> anyhow::Result<()> {
    let mut req = read_request::<SearchMemoryRequest>(&args.input)?;
    if args.route.is_some() || args.intent.is_some() {
        req.route = parse_retrieval_route(args.route)?;
        req.intent = parse_retrieval_intent(args.intent)?;
    }
    if args.belief_branch.is_some() {
        req.belief_branch = args.belief_branch.clone();
    }
    if args.workspace.is_some() {
        req.workspace = args.workspace.clone();
    }
    if let Some(visibility) = args.visibility.as_deref() {
        req.visibility = Some(parse_memory_visibility_value(visibility)?);
    }
    print_json(&client.search(&req).await?)?;
    Ok(())
}

pub(crate) async fn run_lookup_command(
    client: &MemdClient,
    args: LookupArgs,
) -> anyhow::Result<()> {
    let runtime = read_bundle_runtime_config(&args.output)?;
    let req = build_lookup_request(&args, runtime.as_ref())?;
    let response = lookup_with_fallbacks(client, &req, &args.query).await?;
    if args.json {
        print_json(&response)?;
    } else {
        println!(
            "{}",
            render_lookup_markdown(&args.query, &response, args.verbose)
        );
    }
    Ok(())
}

pub(crate) async fn run_context_command(
    client: &MemdClient,
    args: ContextArgs,
) -> anyhow::Result<()> {
    let req = if args.json.is_some() || args.input.is_some() || args.stdin {
        read_request::<ContextRequest>(&RequestInput {
            json: args.json.clone(),
            input: args.input.clone(),
            stdin: args.stdin,
        })?
    } else {
        ContextRequest {
            project: args.project.clone(),
            agent: args.agent.clone(),
            workspace: args.workspace.clone(),
            visibility: args
                .visibility
                .as_deref()
                .map(parse_memory_visibility_value)
                .transpose()?,
            route: parse_retrieval_route(args.route.clone())?,
            intent: parse_retrieval_intent(args.intent.clone())?,
            limit: args.limit,
            max_chars_per_item: args.max_chars_per_item,
        }
    };

    if args.compact {
        print_json(&client.context_compact(&req).await?)?;
    } else {
        print_json(&client.context(&req).await?)?;
    }
    Ok(())
}

pub(crate) async fn run_working_command(
    client: &MemdClient,
    args: WorkingArgs,
) -> anyhow::Result<()> {
    let response = client
        .working(&WorkingMemoryRequest {
            project: args.project.clone(),
            agent: args.agent.clone(),
            workspace: args.workspace.clone(),
            visibility: args
                .visibility
                .as_deref()
                .map(parse_memory_visibility_value)
                .transpose()?,
            route: parse_retrieval_route(args.route.clone())?,
            intent: parse_retrieval_intent(args.intent.clone())?,
            limit: args.limit,
            max_chars_per_item: args.max_chars_per_item,
            max_total_chars: args.max_total_chars,
            rehydration_limit: args.rehydration_limit,
            auto_consolidate: Some(args.auto_consolidate),
        })
        .await?;
    if args.summary {
        println!("{}", render_working_summary(&response, args.follow));
    } else {
        print_json(&response)?;
    }
    Ok(())
}
