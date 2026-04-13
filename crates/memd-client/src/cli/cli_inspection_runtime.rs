use super::*;

fn inspect_bundle_identity_defaults() -> (Option<String>, Option<String>) {
    let Some(project_root) = detect_current_project_root().ok().flatten() else {
        return (None, None);
    };

    let bundle_root = project_root.join(".memd");
    infer_bundle_identity_defaults(&bundle_root)
}

pub(crate) async fn run_profile_command(
    client: &MemdClient,
    args: ProfileArgs,
) -> anyhow::Result<()> {
    let (default_project, default_namespace) = inspect_bundle_identity_defaults();
    let should_set = args.set
        || args.preferred_route.is_some()
        || args.preferred_intent.is_some()
        || args.summary_chars.is_some()
        || args.max_total_chars.is_some()
        || args.recall_depth.is_some()
        || args.source_trust_floor.is_some()
        || !args.style_tag.is_empty()
        || args.notes.is_some();

    if should_set {
        let response = client
            .upsert_agent_profile(&AgentProfileUpsertRequest {
                agent: args.agent.clone(),
                project: args.project.clone().or(default_project.clone()),
                namespace: args.namespace.clone().or(default_namespace.clone()),
                preferred_route: parse_retrieval_route(args.preferred_route.clone())?,
                preferred_intent: parse_retrieval_intent(args.preferred_intent.clone())?,
                summary_chars: args.summary_chars,
                max_total_chars: args.max_total_chars,
                recall_depth: args.recall_depth,
                source_trust_floor: args.source_trust_floor,
                style_tags: args.style_tag.clone(),
                notes: args.notes.clone(),
            })
            .await?;
        if args.summary {
            println!("{}", render_profile_summary(&response, args.follow));
        } else {
            print_json(&response)?;
        }
    } else {
        let response = client
            .agent_profile(&AgentProfileRequest {
                agent: args.agent.clone(),
                project: args.project.clone().or(default_project),
                namespace: args.namespace.clone().or(default_namespace),
            })
            .await?;
        if args.summary {
            println!("{}", render_profile_summary(&response, args.follow));
        } else {
            print_json(&response)?;
        }
    }

    Ok(())
}

pub(crate) async fn run_source_command(
    client: &MemdClient,
    args: SourceArgs,
) -> anyhow::Result<()> {
    let (default_project, default_namespace) = inspect_bundle_identity_defaults();
    let response = client
        .source_memory(&SourceMemoryRequest {
            project: args.project.clone().or(default_project),
            namespace: args.namespace.clone().or(default_namespace),
            workspace: args.workspace.clone(),
            visibility: args
                .visibility
                .as_deref()
                .map(parse_memory_visibility_value)
                .transpose()?,
            source_agent: args.source_agent.clone(),
            source_system: args.source_system.clone(),
            limit: args.limit,
        })
        .await?;
    if args.summary {
        println!("{}", render_source_summary(&response, args.follow));
    } else {
        print_json(&response)?;
    }
    Ok(())
}

pub(crate) async fn run_workspaces_command(
    client: &MemdClient,
    args: SourceArgs,
) -> anyhow::Result<()> {
    let (default_project, default_namespace) = inspect_bundle_identity_defaults();
    let response = client
        .workspace_memory(&memd_schema::WorkspaceMemoryRequest {
            project: args.project.clone().or(default_project),
            namespace: args.namespace.clone().or(default_namespace),
            workspace: args.workspace.clone(),
            visibility: args
                .visibility
                .as_deref()
                .map(parse_memory_visibility_value)
                .transpose()?,
            source_agent: args.source_agent.clone(),
            source_system: args.source_system.clone(),
            limit: args.limit,
        })
        .await?;
    if args.summary {
        println!("{}", render_workspace_summary(&response, args.follow));
    } else {
        print_json(&response)?;
    }
    Ok(())
}

pub(crate) async fn run_inbox_command(client: &MemdClient, args: InboxArgs) -> anyhow::Result<()> {
    let (default_project, default_namespace) = inspect_bundle_identity_defaults();
    let req = MemoryInboxRequest {
        project: args.project.clone().or(default_project),
        namespace: args.namespace.clone().or(default_namespace),
        workspace: args.workspace.clone(),
        visibility: args
            .visibility
            .as_deref()
            .map(parse_memory_visibility_value)
            .transpose()?,
        belief_branch: args.belief_branch.clone(),
        route: parse_retrieval_route(args.route.clone())?,
        intent: parse_retrieval_intent(args.intent.clone())?,
        limit: args.limit,
    };
    print_json(&client.inbox(&req).await?)?;
    Ok(())
}

pub(crate) async fn run_explain_command(
    client: &MemdClient,
    args: ExplainArgs,
) -> anyhow::Result<()> {
    let req = ExplainMemoryRequest {
        id: args.id.parse().context("parse memory id as uuid")?,
        belief_branch: args.belief_branch.clone(),
        route: parse_retrieval_route(args.route.clone())?,
        intent: parse_retrieval_intent(args.intent.clone())?,
    };
    let response = client.explain(&req).await?;
    if args.summary {
        println!("{}", render_explain_summary(&response, args.follow));
    } else {
        print_json(&response)?;
    }
    Ok(())
}

pub(crate) async fn run_entity_command(
    client: &MemdClient,
    args: EntityArgs,
) -> anyhow::Result<()> {
    let req = memd_schema::EntityMemoryRequest {
        id: args.id.parse().context("parse memory id as uuid")?,
        route: parse_retrieval_route(args.route.clone())?,
        intent: parse_retrieval_intent(args.intent.clone())?,
        limit: args.limit,
    };
    let response = client.entity(&req).await?;
    if args.summary {
        println!("{}", render_entity_summary(&response, args.follow));
    } else {
        print_json(&response)?;
    }
    Ok(())
}

pub(crate) async fn run_entity_search_command(
    client: &MemdClient,
    args: EntitySearchArgs,
) -> anyhow::Result<()> {
    let (default_project, default_namespace) = inspect_bundle_identity_defaults();
    let response = client
        .entity_search(&EntitySearchRequest {
            query: args.query.clone(),
            project: args.project.clone().or(default_project),
            namespace: args.namespace.clone().or(default_namespace),
            at: parse_context_time(args.at.clone())?,
            host: args.host.clone(),
            branch: args.branch.clone(),
            location: args.location.clone(),
            route: parse_retrieval_route(args.route.clone())?,
            intent: parse_retrieval_intent(args.intent.clone())?,
            limit: args.limit,
        })
        .await?;
    if args.summary {
        println!("{}", render_entity_search_summary(&response, args.follow));
    } else {
        print_json(&response)?;
    }
    Ok(())
}

pub(crate) async fn run_entity_link_command(
    client: &MemdClient,
    args: EntityLinkArgs,
) -> anyhow::Result<()> {
    let response = client
        .link_entity(&EntityLinkRequest {
            from_entity_id: args
                .from_entity_id
                .parse()
                .context("parse from_entity_id as uuid")?,
            to_entity_id: args
                .to_entity_id
                .parse()
                .context("parse to_entity_id as uuid")?,
            relation_kind: parse_entity_relation_kind(&args.relation_kind)?,
            confidence: args.confidence,
            note: args.note,
            context: None,
            tags: Vec::new(),
        })
        .await?;
    print_json(&response)?;
    Ok(())
}

pub(crate) async fn run_entity_links_command(
    client: &MemdClient,
    args: EntityLinksArgs,
) -> anyhow::Result<()> {
    let response = client
        .entity_links(&EntityLinksRequest {
            entity_id: args.entity_id.parse().context("parse entity_id as uuid")?,
        })
        .await?;
    print_json(&response)?;
    Ok(())
}

pub(crate) async fn run_recall_command(
    client: &MemdClient,
    args: RecallArgs,
) -> anyhow::Result<()> {
    let req = resolve_recall_request(client, &args).await?;
    let response = client.associative_recall(&req).await?;
    if args.summary {
        println!("{}", render_recall_summary(&response, args.follow));
    } else {
        print_json(&response)?;
    }
    Ok(())
}

pub(crate) async fn run_timeline_command(
    client: &MemdClient,
    args: TimelineArgs,
) -> anyhow::Result<()> {
    let req = memd_schema::TimelineMemoryRequest {
        id: args.id.parse().context("parse memory id as uuid")?,
        route: parse_retrieval_route(args.route.clone())?,
        intent: parse_retrieval_intent(args.intent.clone())?,
        limit: args.limit,
    };
    let response = client.timeline(&req).await?;
    if args.summary {
        println!("{}", render_timeline_summary(&response, args.follow));
    } else {
        print_json(&response)?;
    }
    Ok(())
}

pub(crate) async fn run_atlas_command(
    client: &MemdClient,
    args: AtlasArgs,
) -> anyhow::Result<()> {
    match args.command {
        AtlasCommand::Regions(args) => {
            let req = memd_schema::AtlasRegionsRequest {
                project: args.project,
                namespace: args.namespace,
                lane: args.lane,
                limit: args.limit,
            };
            let response = client.atlas_regions(&req).await?;
            if args.json {
                print_json(&response)?;
            } else {
                println!("{}", render_atlas_regions(&response));
            }
        }
        AtlasCommand::Explore(args) => {
            let region_id = args
                .region
                .as_deref()
                .map(|s| s.parse::<uuid::Uuid>())
                .transpose()
                .context("parse region id")?;
            let node_id = args
                .node
                .as_deref()
                .map(|s| s.parse::<uuid::Uuid>())
                .transpose()
                .context("parse node id")?;
            let pivot_kind = args
                .kind
                .as_deref()
                .map(parse_memory_kind_value)
                .transpose()?;
            let pivot_scope = args
                .scope
                .as_deref()
                .map(parse_memory_scope_value)
                .transpose()?;
            let req = memd_schema::AtlasExploreRequest {
                region_id,
                node_id,
                project: args.project,
                namespace: args.namespace,
                lane: args.lane,
                depth: args.depth,
                limit: args.limit,
                pivot_time: None,
                pivot_kind,
                pivot_scope,
                pivot_source_agent: args.source_agent,
                pivot_source_system: args.source_system,
                min_trust: args.min_trust,
                include_evidence: args.include_evidence,
            };
            let response = client.atlas_explore(&req).await?;
            if args.json {
                print_json(&response)?;
            } else {
                println!("{}", render_atlas_explore(&response));
            }
        }
        AtlasCommand::Compile(args) => {
            run_atlas_compile(client, args).await?;
        }
        AtlasCommand::Generate(args) => {
            let req = memd_schema::AtlasRegionsRequest {
                project: args.project,
                namespace: args.namespace,
                lane: args.lane,
                limit: args.limit,
            };
            let response = client.atlas_generate(&req).await?;
            if args.json {
                print_json(&response)?;
            } else {
                println!("{}", render_atlas_regions(&response));
            }
        }
    }
    Ok(())
}

fn render_atlas_regions(response: &memd_schema::AtlasRegionsResponse) -> String {
    let mut out = String::from("# Atlas Regions\n\n");
    if response.regions.is_empty() {
        out.push_str("No regions found. Run `memd atlas generate` to create them.\n");
        return out;
    }
    for region in &response.regions {
        out.push_str(&format!(
            "- **{}** [{}] ({} nodes{})\n",
            region.name,
            &region.id.to_string()[..8],
            region.node_count,
            region
                .lane
                .as_deref()
                .map(|l| format!(", lane={l}"))
                .unwrap_or_default(),
        ));
        if let Some(desc) = &region.description {
            out.push_str(&format!("  {desc}\n"));
        }
    }
    out
}

fn render_atlas_explore(response: &memd_schema::AtlasExploreResponse) -> String {
    let mut out = String::new();
    if let Some(region) = &response.region {
        out.push_str(&format!(
            "# Region: {} [{}]\n\n",
            region.name,
            &region.id.to_string()[..8]
        ));
    } else {
        out.push_str("# Atlas Explore\n\n");
    }

    if response.nodes.is_empty() {
        out.push_str("No nodes found.\n");
        return out;
    }

    out.push_str(&format!("## Nodes ({})\n\n", response.nodes.len()));
    for node in &response.nodes {
        let depth_marker = if node.depth > 0 {
            format!(" d={}", node.depth)
        } else {
            String::new()
        };
        let evidence_tag = if node.evidence_count > 0 {
            format!(", ev={}", node.evidence_count)
        } else {
            String::new()
        };
        out.push_str(&format!(
            "- [{}] {} ({:?}, cf={:.2}{}{})\n",
            &node.id.to_string()[..8],
            node.label,
            node.kind,
            node.confidence,
            depth_marker,
            evidence_tag,
        ));
    }

    if !response.links.is_empty() {
        out.push_str(&format!("\n## Links ({})\n\n", response.links.len()));
        for link in &response.links {
            out.push_str(&format!(
                "- [{}] --{:?}--> [{}] (w={:.2})\n",
                &link.from_node_id.to_string()[..8],
                link.link_kind,
                &link.to_node_id.to_string()[..8],
                link.weight,
            ));
        }
    }

    if !response.evidence.is_empty() {
        out.push_str(&format!("\n## Evidence ({} events)\n\n", response.evidence.len()));
        for event in &response.evidence {
            out.push_str(&format!(
                "- [{}] {} — {} ({})\n",
                &event.id.to_string()[..8],
                event.event_type,
                event.summary.lines().next().unwrap_or(""),
                event.occurred_at.format("%Y-%m-%d %H:%M"),
            ));
        }
    }

    if response.truncated {
        out.push_str("\n(truncated — increase --limit for more)\n");
    }

    out
}

async fn run_atlas_compile(client: &MemdClient, args: AtlasCompileArgs) -> anyhow::Result<()> {
    // Generate regions first
    let gen_req = memd_schema::AtlasRegionsRequest {
        project: args.project.clone(),
        namespace: args.namespace.clone(),
        lane: None,
        limit: None,
    };
    let regions = client.atlas_generate(&gen_req).await?;

    if regions.regions.is_empty() {
        println!("No atlas regions to compile.");
        return Ok(());
    }

    // Determine output directory
    let atlas_dir = if let Some(vault) = &args.vault {
        std::path::PathBuf::from(vault).join("atlas")
    } else {
        args.output.join("compiled").join("atlas")
    };
    std::fs::create_dir_all(&atlas_dir)
        .with_context(|| format!("create atlas dir {}", atlas_dir.display()))?;

    // Write index
    let mut index = String::from("# Memory Atlas\n\n");
    index.push_str(&format!(
        "Generated: {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M")
    ));
    index.push_str("## Regions\n\n");

    for region in &regions.regions {
        // Explore each region to get nodes
        let explore_req = memd_schema::AtlasExploreRequest {
            region_id: Some(region.id),
            node_id: None,
            project: args.project.clone(),
            namespace: args.namespace.clone(),
            lane: None,
            depth: Some(1),
            limit: Some(50),
            pivot_time: None,
            pivot_kind: None,
            pivot_scope: None,
            pivot_source_agent: None,
            pivot_source_system: None,
            min_trust: None,
            include_evidence: false,
        };
        let explored = client.atlas_explore(&explore_req).await?;

        // Build region markdown
        let region_md = build_region_obsidian_note(region, &explored);
        let safe_name = region.name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-");
        let region_path = atlas_dir.join(format!("{safe_name}.md"));
        std::fs::write(&region_path, &region_md)
            .with_context(|| format!("write region {}", region_path.display()))?;

        index.push_str(&format!(
            "- [[atlas/{safe_name}|{}]] — {} nodes\n",
            region.name, region.node_count,
        ));
    }

    let index_path = atlas_dir.join("_atlas-index.md");
    std::fs::write(&index_path, &index)
        .with_context(|| format!("write atlas index {}", index_path.display()))?;

    println!(
        "Compiled {} regions to {}",
        regions.regions.len(),
        atlas_dir.display()
    );
    Ok(())
}

fn build_region_obsidian_note(
    region: &memd_schema::AtlasRegion,
    explored: &memd_schema::AtlasExploreResponse,
) -> String {
    let mut md = format!("# {}\n\n", region.name);
    md.push_str(&format!("- id: `{}`\n", region.id));
    if let Some(project) = &region.project {
        md.push_str(&format!("- project: `{project}`\n"));
    }
    if let Some(ns) = &region.namespace {
        md.push_str(&format!("- namespace: `{ns}`\n"));
    }
    if let Some(lane) = &region.lane {
        md.push_str(&format!("- lane: `{lane}`\n"));
    }
    md.push_str(&format!("- nodes: {}\n", explored.nodes.len()));
    md.push_str(&format!(
        "- auto: {}\n\n",
        if region.auto_generated { "yes" } else { "no" }
    ));

    if !explored.nodes.is_empty() {
        md.push_str("## Nodes\n\n");
        for node in &explored.nodes {
            let depth_tag = if node.depth > 0 {
                format!(" (neighbor, depth={})", node.depth)
            } else {
                String::new()
            };
            let ev_tag = if node.evidence_count > 0 {
                format!(", {} evidence events", node.evidence_count)
            } else {
                String::new()
            };
            md.push_str(&format!(
                "- `[{}]` **{}** — {:?}, cf={:.2}{}{}\n",
                &node.id.to_string()[..8],
                node.label,
                node.kind,
                node.confidence,
                depth_tag,
                ev_tag,
            ));
        }
    }

    if !explored.links.is_empty() {
        md.push_str("\n## Links\n\n");
        for link in &explored.links {
            md.push_str(&format!(
                "- `[{}]` → `[{}]` ({:?}, w={:.2})\n",
                &link.from_node_id.to_string()[..8],
                &link.to_node_id.to_string()[..8],
                link.link_kind,
                link.weight,
            ));
        }
    }

    if !explored.trails.is_empty() {
        md.push_str("\n## Trails\n\n");
        for trail in &explored.trails {
            md.push_str(&format!(
                "### {} trail ({} nodes)\n\n",
                trail.name,
                trail.nodes.len()
            ));
            for (i, node_id) in trail.nodes.iter().enumerate() {
                let label = explored
                    .nodes
                    .iter()
                    .find(|n| n.id == *node_id)
                    .map(|n| n.label.as_str())
                    .unwrap_or("?");
                md.push_str(&format!("{}. `[{}]` {}\n", i + 1, &node_id.to_string()[..8], label));
            }
            md.push('\n');
        }
    }

    md
}

pub(crate) fn run_events_command(args: EventsArgs) -> anyhow::Result<()> {
    let bundle_root = resolve_compiled_event_bundle_root(Some(&args.root))?;
    if args.list {
        let index = render_compiled_event_index(&bundle_root)?;
        if args.summary {
            println!(
                "{}",
                render_compiled_event_index_summary(&bundle_root, &index)
            );
        } else {
            print_json(&render_compiled_event_index_json(&bundle_root, &index))?;
        }
    } else if let Some(query) = args.query.as_deref() {
        let hits = search_compiled_event_pages(&bundle_root, query, args.limit)?;
        if args.summary {
            println!(
                "{}",
                render_compiled_event_search_summary(&bundle_root, query, &hits)
            );
        } else {
            print_json(&hits)?;
        }
    } else if let Some(target) = args.open.as_deref() {
        let path = resolve_compiled_event_page(&bundle_root, target)?;
        let content =
            fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        if args.summary {
            println!("{}", render_compiled_event_page_summary(&path, &content));
        } else {
            println!("{}", render_compiled_event_page_markdown(&path, &content));
        }
    } else {
        let index = render_compiled_event_index(&bundle_root)?;
        if args.summary {
            println!(
                "{}",
                render_compiled_event_index_summary(&bundle_root, &index)
            );
        } else {
            println!(
                "{}",
                render_compiled_event_index_markdown(&bundle_root, &index)
            );
        }
    }
    Ok(())
}

pub(crate) async fn run_consolidate_command(
    client: &MemdClient,
    args: ConsolidateArgs,
) -> anyhow::Result<()> {
    let response = client
        .consolidate(&MemoryConsolidationRequest {
            project: args.project.clone(),
            namespace: args.namespace.clone(),
            max_groups: args.max_groups,
            min_events: args.min_events,
            lookback_days: args.lookback_days,
            min_salience: args.min_salience,
            record_events: Some(args.record_events),
        })
        .await?;
    if args.summary {
        println!("{}", render_consolidate_summary(&response, args.follow));
    } else {
        print_json(&response)?;
    }
    Ok(())
}

pub(crate) async fn run_maintenance_report_command(
    client: &MemdClient,
    args: MaintenanceReportArgs,
) -> anyhow::Result<()> {
    let response = client
        .maintenance_report(&MemoryMaintenanceReportRequest {
            project: args.project.clone(),
            namespace: args.namespace.clone(),
            inactive_days: args.inactive_days,
            lookback_days: args.lookback_days,
            min_events: args.min_events,
            max_decay: args.max_decay,
            mode: Some("scan".to_string()),
            apply: Some(false),
        })
        .await?;
    if args.summary {
        println!(
            "{}",
            render_maintenance_report_summary(&response, args.follow)
        );
    } else {
        print_json(&response)?;
    }
    Ok(())
}

pub(crate) async fn run_bundle_maintain_command(
    args: MaintainArgs,
    base_url: &str,
) -> anyhow::Result<()> {
    let response = crate::run_maintain_command(&args, base_url).await?;
    if args.summary {
        println!("{}", render_maintain_summary(&response));
    } else {
        print_json(&response)?;
    }
    Ok(())
}

pub(crate) async fn run_policy_command(
    client: &MemdClient,
    args: PolicyArgs,
) -> anyhow::Result<()> {
    let response = client.policy().await?;
    if args.summary {
        println!("{}", render_policy_summary(&response, args.follow));
    } else {
        print_json(&response)?;
    }
    Ok(())
}

pub(crate) async fn run_skill_policy_command(
    client: &MemdClient,
    args: PolicyArgs,
) -> anyhow::Result<()> {
    let response = client.policy().await?;
    let report = build_skill_lifecycle_report(&response);
    if args.query {
        let query = SkillPolicyApplyReceiptsRequest {
            project: args.project.clone(),
            namespace: args.namespace.clone(),
            workspace: args.workspace.clone(),
            limit: args.limit,
        };
        let receipts = client.skill_policy_apply_receipts(&query).await?;
        let activations = client
            .skill_policy_activations(&SkillPolicyActivationEntriesRequest {
                project: args.project.clone(),
                namespace: args.namespace.clone(),
                workspace: args.workspace.clone(),
                limit: args.limit,
            })
            .await?;
        if args.summary {
            println!(
                "{}",
                render_skill_policy_query_summary(&receipts, &activations, args.follow)
            );
        } else {
            print_json(&serde_json::json!({
                "receipts": receipts,
                "activations": activations,
            }))?;
        }
    } else if args.summary {
        println!("{}", render_skill_policy_summary(&response, args.follow));
        println!();
        print!("{}", render_skill_lifecycle_report(&report, args.follow));
    } else {
        print_json(&response)?;
    }
    if args.write || args.apply {
        let receipt = write_skill_policy_artifacts(&args.output, &response, &report, args.apply)?;
        if let Some(receipt) = receipt {
            let posted = client
                .record_skill_policy_apply(&skill_policy_apply_request(&receipt))
                .await?;
            println!(
                "applied {} via server receipt {}",
                posted.receipt.applied_count, posted.receipt.id
            );
        }
        let mut paths = vec![
            skill_policy_batch_state_path(&args.output)
                .display()
                .to_string(),
            skill_policy_review_state_path(&args.output)
                .display()
                .to_string(),
            skill_policy_activate_state_path(&args.output)
                .display()
                .to_string(),
        ];
        if args.apply {
            paths.push(
                skill_policy_apply_state_path(&args.output)
                    .display()
                    .to_string(),
            );
        }
        println!("wrote {}", paths.join(", "));
    }
    Ok(())
}

pub(crate) async fn run_compact_command(
    client: &MemdClient,
    base_url: &str,
    args: CompactArgs,
) -> anyhow::Result<()> {
    if args.spill && args.wire {
        anyhow::bail!("use either --spill or --wire, not both");
    }

    let memory = client
        .context_compact(&ContextRequest {
            project: args.project.clone(),
            agent: args.agent.clone(),
            workspace: None,
            visibility: None,
            route: parse_retrieval_route(args.route.clone())?,
            intent: parse_retrieval_intent(args.intent.clone())?,
            limit: args.limit,
            max_chars_per_item: args.max_chars_per_item,
        })
        .await?;

    let packet = build_compaction_packet(BuildCompactionPacketArgs {
        session: CompactionSession {
            project: args.project,
            agent: args.agent,
            task: args.task,
        },
        goal: args.goal,
        hard_constraints: args.hard_constraint,
        active_work: args.active_work,
        decisions: args
            .decision
            .into_iter()
            .enumerate()
            .map(|(idx, text)| CompactionDecision {
                id: format!("decision-{}", idx + 1),
                text,
            })
            .collect(),
        open_loops: args
            .open_loop
            .into_iter()
            .enumerate()
            .map(|(idx, text)| CompactionOpenLoop {
                id: format!("loop-{}", idx + 1),
                text,
                status: "open".to_string(),
            })
            .collect(),
        exact_refs: args
            .exact_ref
            .into_iter()
            .map(|value| {
                let (kind, value) = value
                    .split_once('=')
                    .map(|(kind, value)| (kind.trim().to_string(), value.trim().to_string()))
                    .unwrap_or_else(|| ("unknown".to_string(), value.trim().to_string()));
                CompactionReference { kind, value }
            })
            .collect(),
        next_actions: args.next_action,
        do_not_drop: args.do_not_drop,
        memory,
    });

    if args.spill {
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
            let submitted = responses.len();
            let result = CompactionSpillResult {
                submitted,
                duplicates,
                responses,
                batch: spill,
            };
            print_json(&result)?;
        } else {
            print_json(&spill)?;
        }
    } else if args.wire {
        println!("{}", render_compaction_wire(&packet));
    } else {
        print_json(&packet)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::set_current_dir;

    #[test]
    fn inspection_bundle_defaults_bind_repo_identity_without_runtime_config() {
        let temp_root =
            std::env::temp_dir().join(format!("memd-inspection-defaults-{}", uuid::Uuid::new_v4()));
        let repo_root = temp_root.join("repo-b");

        fs::create_dir_all(repo_root.join(".git")).expect("create repo git dir");
        let _cwd = set_current_dir(&repo_root);

        let (project, namespace) = inspect_bundle_identity_defaults();
        assert_eq!(project.as_deref(), Some("repo-b"));
        assert_eq!(namespace.as_deref(), Some("main"));

        drop(_cwd);
        fs::remove_dir_all(temp_root).expect("cleanup temp root");
    }
}
