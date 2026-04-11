use super::*;

pub(crate) async fn run_workspace_watch(
    _client: &MemdClient,
    base_url: &str,
    args: &WatchArgs,
) -> anyhow::Result<()> {
    println!(
        "workspace_watch root={} output={} debounce_ms={}",
        args.root.display(),
        args.output.display(),
        args.debounce_ms
    );

    let initial = read_bundle_resume(
        &ResumeArgs {
            output: args.output.clone(),
            project: args.project.clone(),
            namespace: args.namespace.clone(),
            agent: args.agent.clone(),
            workspace: args.workspace.clone(),
            visibility: args.visibility.clone(),
            route: args.route.clone(),
            intent: args.intent.clone().or(Some("current_task".to_string())),
            limit: args.limit,
            rehydration_limit: args.rehydration_limit,
            semantic: args.semantic,
            prompt: false,
            summary: false,
        },
        base_url,
    )
    .await?;
    write_bundle_memory_files(&args.output, &initial, None, false).await?;
    auto_checkpoint_live_snapshot(&args.output, base_url, &initial, "watch-start").await?;

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
                    .any(|path| workspace_path_should_trigger(path));
                if should_trigger {
                    let _ = tx.send(());
                }
            }
        },
        NotifyConfig::default(),
    )
    .context("create workspace watcher")?;
    watcher
        .watch(&args.root, RecursiveMode::Recursive)
        .with_context(|| format!("watch {}", args.root.display()))?;

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

        match read_bundle_resume(
            &ResumeArgs {
                output: args.output.clone(),
                project: args.project.clone(),
                namespace: args.namespace.clone(),
                agent: args.agent.clone(),
                workspace: args.workspace.clone(),
                visibility: args.visibility.clone(),
                route: args.route.clone(),
                intent: args.intent.clone().or(Some("current_task".to_string())),
                limit: args.limit,
                rehydration_limit: args.rehydration_limit,
                semantic: args.semantic,
                prompt: false,
                summary: false,
            },
            base_url,
        )
        .await
        {
            Ok(snapshot) => {
                if let Err(err) =
                    write_bundle_memory_files(&args.output, &snapshot, None, false).await
                {
                    eprintln!("workspace watch write failed: {err:#}");
                    continue;
                }
                if let Err(err) =
                    auto_checkpoint_live_snapshot(&args.output, base_url, &snapshot, "watch").await
                {
                    eprintln!("workspace watch auto-checkpoint failed: {err:#}");
                }
                println!(
                    "workspace_watch update root={} working={} inbox={} focus=\"{}\"",
                    args.root.display(),
                    snapshot.working.records.len(),
                    snapshot.inbox.items.len(),
                    snapshot
                        .working
                        .records
                        .first()
                        .map(|record| compact_inline(&record.record, 72))
                        .unwrap_or_else(|| "none".to_string())
                );
            }
            Err(err) => eprintln!("workspace watch refresh failed: {err:#}"),
        }
    }

    Ok(())
}

pub(crate) async fn resolve_recall_request(
    client: &MemdClient,
    args: &RecallArgs,
) -> anyhow::Result<AssociativeRecallRequest> {
    if let Some(entity_id) = &args.entity_id {
        return Ok(AssociativeRecallRequest {
            entity_id: entity_id.parse().context("parse entity id as uuid")?,
            depth: args.depth,
            limit: args.limit,
        });
    }

    let query = args
        .query
        .clone()
        .context("provide either --entity-id or --query")?;
    let response = client
        .entity_search(&EntitySearchRequest {
            query,
            project: args.project.clone(),
            namespace: args.namespace.clone(),
            at: parse_context_time(args.at.clone())?,
            host: args.host.clone(),
            branch: args.branch.clone(),
            location: args.location.clone(),
            route: None,
            intent: None,
            limit: Some(5),
        })
        .await
        .context("resolve recall target")?;

    let Some(best_match) = response.best_match else {
        anyhow::bail!("no entity matched the recall query");
    };
    if response.ambiguous {
        anyhow::bail!(
            "recall query was ambiguous; use --entity-id instead (best match {}::{})",
            short_uuid(best_match.entity.id),
            best_match.entity.entity_type,
        );
    }

    Ok(AssociativeRecallRequest {
        entity_id: best_match.entity.id,
        depth: args.depth,
        limit: args.limit,
    })
}
