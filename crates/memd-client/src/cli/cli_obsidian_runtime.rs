use super::*;

pub(crate) async fn run_obsidian_mode(
    client: &MemdClient,
    base_url: &str,
    args: ObsidianArgs,
) -> anyhow::Result<()> {
    match args.mode {
        ObsidianMode::Scan => {
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
            if args.review_sensitive {
                println!("{}", obsidian::render_sensitive_review(&scan));
                return Ok(());
            }
            if args.summary {
                println!("{}", render_obsidian_scan_summary(&scan, args.follow));
            } else {
                print_json(&scan)?;
            }
        }
        ObsidianMode::Import => {
            run_obsidian_import(client, &args, false, false).await?;
        }
        ObsidianMode::Sync => {
            run_obsidian_import(client, &args, true, false).await?;
        }
        ObsidianMode::Compile => {
            run_obsidian_compile(client, &args).await?;
        }
        ObsidianMode::Handoff => {
            run_obsidian_handoff(&args, base_url).await?;
        }
        ObsidianMode::Writeback => {
            run_obsidian_writeback(client, &args).await?;
        }
        ObsidianMode::Open => {
            run_obsidian_open(client, &args).await?;
        }
        ObsidianMode::Roundtrip => {
            run_obsidian_import(client, &args, true, true).await?;
        }
        ObsidianMode::Watch => {
            run_obsidian_watch(client, &args).await?;
        }
        ObsidianMode::Status => {
            run_obsidian_status(client, &args).await?;
        }
    }

    Ok(())
}
