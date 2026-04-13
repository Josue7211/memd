use super::*;

pub(crate) async fn run_bundle_wake_command(args: &WakeArgs, base_url: &str) -> anyhow::Result<()> {
    if let Some(tab_id) = default_bundle_tab_id() {
        let existing_tab_id = read_bundle_runtime_config(&args.output)
            .ok()
            .flatten()
            .and_then(|config| config.tab_id)
            .filter(|value| !value.trim().is_empty());
        if existing_tab_id.is_none() {
            set_bundle_tab_id(&args.output, &tab_id)?;
        }
    }
    crate::runtime::invalidate_bundle_runtime_caches(&args.output)?;
    let codex_pack = harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "codex");
    let agent_zero_pack =
        harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "agent-zero");
    let hermes_pack =
        harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "hermes");
    let opencode_pack =
        harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "opencode");
    let openclaw_pack =
        harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "openclaw");
    let resume_args = ResumeArgs {
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
    };
    let snapshot = match crate::runtime::read_bundle_resume(&resume_args, base_url).await {
        Ok(snapshot) => snapshot,
        Err(err)
            if codex_pack || agent_zero_pack || hermes_pack || opencode_pack || openclaw_pack =>
        {
            if let Some(markdown) = read_codex_pack_local_markdown(&args.output, "wake.md")?
            {
                if args.write {
                    write_bundle_turn_fallback_artifacts(
                        &args.output,
                        args.project.as_deref(),
                        args.namespace.as_deref(),
                        args.agent.as_deref(),
                        args.workspace.as_deref(),
                        args.visibility.as_deref(),
                        args.route.as_deref(),
                        args.intent.as_deref(),
                        &markdown,
                    )?;
                }
                println!("{markdown}");
                return Ok(());
            }
            if args.write {
                write_bundle_turn_placeholder_memory(
                    &args.output,
                    args.project.as_deref(),
                    args.namespace.as_deref(),
                    args.agent.as_deref(),
                    args.workspace.as_deref(),
                    args.visibility.as_deref(),
                    args.route.as_deref(),
                    args.intent.as_deref(),
                )?;
            }
            return Err(err);
        }
        Err(err) => return Err(err),
    };
    let wakeup = render_bundle_wakeup_markdown(&args.output, &snapshot, args.verbose);
    if args.write {
        write_bundle_memory_files(&args.output, &snapshot, None, false).await?;
        auto_checkpoint_live_snapshot(&args.output, base_url, &snapshot, "wake").await?;
    }
    if codex_pack || agent_zero_pack || openclaw_pack || hermes_pack || opencode_pack {
        let _ = refresh_harness_pack_files_for_snapshot(
            &args.output,
            &snapshot,
            "wake",
            &["codex", "agent-zero", "openclaw", "hermes", "opencode"],
        )
        .await?;
    }
    if args.summary {
        println!("{}", render_bundle_wakeup_summary(&snapshot));
    } else {
        println!("{wakeup}");
    }
    Ok(())
}
