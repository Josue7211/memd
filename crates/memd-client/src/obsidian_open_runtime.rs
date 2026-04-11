use super::*;

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
