use super::*;
use crate::cli::command_catalog::{build_command_catalog, filter_command_catalog};
use crate::cli::skill_catalog::{
    build_skill_catalog, find_skill_catalog_matches, resolve_skill_catalog_root,
};

pub(crate) fn run_inspiration_command(args: InspirationArgs) -> anyhow::Result<()> {
    let root = resolve_inspiration_root(args.root.as_deref())?;
    let matches = search_inspiration_lane(&root, &args.query, args.limit)?;
    if args.summary {
        println!(
            "{}",
            render_inspiration_search_summary(&root, &args.query, &matches)
        );
    } else {
        println!(
            "{}",
            render_inspiration_search_markdown(&root, &args.query, &matches)
        );
    }
    Ok(())
}

pub(crate) fn run_skill_catalog_command(args: SkillsArgs) -> anyhow::Result<()> {
    let root = resolve_skill_catalog_root(args.root.as_deref())?;
    let catalog = build_skill_catalog(&root)?;
    if let Some(query) = args.query.as_deref() {
        let matches = find_skill_catalog_matches(&catalog, query);
        if args.summary {
            println!(
                "{}",
                render_skill_catalog_match_summary(&catalog, query, &matches)
            );
        } else {
            println!(
                "{}",
                render_skill_catalog_match_markdown(&catalog, query, &matches)
            );
        }
    } else if args.summary {
        println!("{}", render_skill_catalog_summary(&catalog));
    } else {
        println!("{}", render_skill_catalog_markdown(&catalog));
    }
    Ok(())
}

pub(crate) fn run_pack_catalog_command(args: PacksArgs) -> anyhow::Result<()> {
    let bundle_root = resolve_pack_bundle_root(args.root.as_deref())?;
    let runtime = read_bundle_runtime_config(&bundle_root)?;
    let index = crate::harness::index::build_harness_pack_index(
        &bundle_root,
        runtime
            .as_ref()
            .and_then(|config| config.project.as_deref()),
        runtime
            .as_ref()
            .and_then(|config| config.namespace.as_deref()),
    );
    let index = crate::harness::index::filter_harness_pack_index(index, args.query.as_deref());
    if args.json {
        print_json(&render_harness_pack_index_json(&index))?;
    } else if args.summary {
        println!(
            "{}",
            render_harness_pack_index_summary(&bundle_root, &index, args.query.as_deref())
        );
    } else {
        println!(
            "{}",
            render_harness_pack_index_markdown(&bundle_root, &index)
        );
    }
    Ok(())
}

pub(crate) fn run_command_catalog_command(args: CommandCatalogArgs) -> anyhow::Result<()> {
    let bundle_root = resolve_pack_bundle_root(args.root.as_deref())?;
    let catalog = build_command_catalog(&bundle_root);
    let catalog = filter_command_catalog(catalog, args.query.as_deref());
    if args.json {
        print_json(&render_command_catalog_json(&catalog))?;
    } else if args.summary {
        println!(
            "{}",
            render_command_catalog_summary(&catalog, args.query.as_deref())
        );
    } else {
        println!("{}", render_command_catalog_markdown(&catalog));
    }
    Ok(())
}

pub(crate) fn run_loops_command(args: LoopsArgs) -> anyhow::Result<()> {
    let entries = read_loop_entries(&args.output)?;
    if let Some(slug) = args.loop_slug.as_deref() {
        print_loop_detail(&entries, slug)?;
    } else if args.summary {
        print_loop_summary(&entries);
    } else {
        print_loop_list(&entries, &args.output);
    }
    Ok(())
}

pub(crate) fn run_telemetry_command(args: TelemetryArgs) -> anyhow::Result<()> {
    run_telemetry(&args)
}

pub(crate) async fn run_autoresearch_command(
    args: AutoresearchArgs,
    base_url: &str,
) -> anyhow::Result<()> {
    run_autoresearch(&args, base_url).await
}
