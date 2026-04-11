use super::*;

pub(crate) async fn run_rag_mode(client: &MemdClient, args: RagArgs) -> anyhow::Result<()> {
    let rag_url = resolve_rag_url(args.rag_url, resolve_default_bundle_root()?.as_deref())?;
    let rag = RagClient::new(&rag_url)?;
    match args.mode {
        RagMode::Healthz => print_json(&rag.healthz().await?)?,
        RagMode::Search(args) => {
            let mode = args
                .mode
                .as_deref()
                .map(parse_rag_retrieve_mode)
                .transpose()?
                .unwrap_or(RagRetrieveMode::Auto);
            let query = RagRetrieveRequest {
                query: args.query,
                project: args.project,
                namespace: args.namespace,
                mode,
                limit: args.limit,
                include_cross_modal: args.include_cross_modal,
            };
            print_json(&rag.retrieve(&query).await?)?;
        }
        RagMode::Sync(args) => {
            let summary = sync_to_rag(client, &rag, args).await?;
            print_json(&summary)?;
        }
    }

    Ok(())
}

pub(crate) async fn run_multimodal_mode(args: MultimodalArgs) -> anyhow::Result<()> {
    let rag_url = resolve_rag_url(args.rag_url, resolve_default_bundle_root()?.as_deref())?;
    let sidecar = SidecarClient::new(&rag_url)?;
    match args.mode {
        MultimodalMode::Healthz => print_json(&sidecar.healthz().await?)?,
        MultimodalMode::Plan(args) => {
            let preview = build_multimodal_preview(args.project, args.namespace, &args.path)?;
            print_json(&preview)?;
        }
        MultimodalMode::Ingest(args) => {
            let preview = build_multimodal_preview(args.project, args.namespace, &args.path)?;
            if args.apply {
                let responses = ingest_multimodal_preview(&sidecar, &preview.requests).await?;
                let submitted = responses.len();
                print_json(&MultimodalIngestOutput {
                    preview,
                    responses,
                    submitted,
                    dry_run: false,
                })?;
            } else {
                print_json(&MultimodalIngestOutput {
                    preview,
                    responses: Vec::new(),
                    submitted: 0,
                    dry_run: true,
                })?;
            }
        }
        MultimodalMode::Retrieve(args) => {
            let mut request = memd_multimodal::build_retrieve_request(
                args.query,
                args.project,
                args.namespace,
                args.limit,
                args.include_cross_modal,
            );
            if let Some(mode) = args
                .mode
                .as_deref()
                .map(parse_rag_retrieve_mode)
                .transpose()?
            {
                request.mode = mode;
            }
            print_json(&sidecar.retrieve(&request).await?)?;
        }
    }

    Ok(())
}
