use std::path::PathBuf;

use anyhow::Context;
use memd_client::MemdClient;
use memd_schema::ExplainMemoryRequest;
use serde_json::json;

use super::obsidian;
use super::{
    HandoffArgs, ObsidianArgs, print_json, resolve_default_bundle_root,
    runtime_resume::read_bundle_handoff,
};

fn build_handoff_args(args: &ObsidianArgs) -> anyhow::Result<HandoffArgs> {
    Ok(HandoffArgs {
        output: resolve_default_bundle_root()?.unwrap_or_else(|| PathBuf::from(".memd")),
        target_session: None,
        project: args.project.clone(),
        namespace: args.namespace.clone(),
        agent: None,
        workspace: args.workspace.clone(),
        visibility: args.visibility.clone(),
        route: args.route.clone(),
        intent: args.intent.clone(),
        limit: args.limit,
        rehydration_limit: Some(4),
        source_limit: Some(6),
        semantic: true,
        prompt: false,
        summary: false,
    })
}

pub(crate) async fn run_obsidian_writeback(
    client: &MemdClient,
    args: &ObsidianArgs,
) -> anyhow::Result<()> {
    let Some(id) = args.id.as_ref() else {
        anyhow::bail!("obsidian writeback requires --id <uuid>");
    };
    let id = id
        .parse::<uuid::Uuid>()
        .context("parse obsidian writeback id")?;
    let explain = client
        .explain(&ExplainMemoryRequest {
            id,
            belief_branch: None,
            route: None,
            intent: None,
        })
        .await?;

    let output_path = args
        .output
        .clone()
        .unwrap_or_else(|| obsidian::default_writeback_path(&args.vault, &explain));
    let (title, markdown) =
        obsidian::build_writeback_markdown(&args.vault, &explain, explain.entity.as_ref());

    let preview = json!({
        "output_path": output_path.display().to_string(),
        "open_uri": obsidian::build_open_uri(&output_path, args.pane_type.as_deref())?,
        "title": title,
        "id": explain.item.id,
        "kind": format!("{:?}", explain.item.kind).to_lowercase(),
        "summary": explain.item.content.clone(),
        "reasons": explain.reasons.clone(),
        "entity": explain.entity.as_ref().map(|entity| entity.id),
        "events": explain.events.len(),
        "apply": args.apply,
    });

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
    if args.open {
        let uri = obsidian::build_open_uri(&output_path, args.pane_type.as_deref())?;
        obsidian::open_uri(&uri)?;
    }
    print_json(&preview)?;
    Ok(())
}

pub(crate) async fn run_obsidian_handoff(
    args: &ObsidianArgs,
    base_url: &str,
) -> anyhow::Result<()> {
    let handoff_args = build_handoff_args(args)?;
    let snapshot = read_bundle_handoff(&handoff_args, base_url).await?;

    let output_path = args
        .output
        .clone()
        .unwrap_or_else(|| obsidian::default_handoff_path(&args.vault, &snapshot.resume));
    let (title, markdown) =
        obsidian::build_handoff_markdown(&args.vault, &snapshot.resume, &snapshot.sources);
    let preview = json!({
        "output_path": output_path.display().to_string(),
        "open_uri": obsidian::build_open_uri(&output_path, args.pane_type.as_deref())?,
        "title": title,
        "project": snapshot.resume.project,
        "namespace": snapshot.resume.namespace,
        "workspace": snapshot.resume.workspace,
        "visibility": snapshot.resume.visibility,
        "working": snapshot.resume.working.records.len(),
        "inbox": snapshot.resume.inbox.items.len(),
        "workspaces": snapshot.resume.workspaces.workspaces.len(),
        "semantic_hits": snapshot.resume.semantic.as_ref().map(|semantic| semantic.items.len()).unwrap_or(0),
        "sources": snapshot.sources.sources.len(),
        "apply": args.apply,
    });

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
    if args.open {
        let uri = obsidian::build_open_uri(&output_path, args.pane_type.as_deref())?;
        obsidian::open_uri(&uri)?;
    }
    print_json(&preview)?;
    Ok(())
}
