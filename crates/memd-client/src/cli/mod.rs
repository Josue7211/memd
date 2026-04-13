use super::*;
use crate::runtime::*;

pub(crate) mod args;
pub(crate) use args::*;

pub(crate) mod command_catalog;

mod commands;
pub(crate) use commands::*;

mod cli_memory_runtime;
pub(crate) use cli_memory_runtime::*;

mod cli_awareness_runtime;
pub(crate) use cli_awareness_runtime::*;

mod cli_analysis_runtime;
pub(crate) use cli_analysis_runtime::*;

mod cli_obsidian_runtime;
pub(crate) use cli_obsidian_runtime::*;

mod cli_hook_runtime;
pub(crate) use cli_hook_runtime::*;

mod cli_rag_runtime;
pub(crate) use cli_rag_runtime::*;

mod cli_utility_runtime;
pub(crate) use cli_utility_runtime::*;

mod cli_inspection_runtime;
pub(crate) use cli_inspection_runtime::*;

pub(crate) mod skill_catalog;

pub(crate) async fn run_cli(cli: Cli) -> anyhow::Result<()> {
    let client = MemdClient::new(&cli.base_url)?;
    let base_url = cli.base_url.clone();

    #[allow(unreachable_patterns)]
    match cli.command {
        Commands::Healthz => print_json(&client.healthz().await?)?,
        Commands::Status(args) => {
            let status = read_bundle_status(&args.output, &base_url).await?;
            if args.summary {
                println!("{}", render_bundle_status_summary(&status));
            } else {
                print_json(&status)?;
            }
        }
        Commands::Capabilities(args) => {
            let response = run_capabilities_command(&args)?;
            if args.json {
                print_json(&response)?;
            } else {
                println!("{}", render_capabilities_runtime_summary(&response));
            }
        }
        Commands::Session(args) => {
            let response = run_session_command(&args, &base_url).await?;
            if args.summary {
                println!("{}", render_session_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Wake(args) => {
            crate::run_bundle_wake_command(&args, &base_url).await?;
        }
        Commands::Awareness(args) => run_awareness_command(&args).await?,
        Commands::Heartbeat(args) => {
            if args.watch {
                let interval = Duration::from_secs(args.interval_secs.max(1));
                loop {
                    let response =
                        refresh_bundle_heartbeat(&args.output, None, args.probe_base_url).await?;
                    if args.summary {
                        println!("{}", render_bundle_heartbeat_summary(&response));
                    } else {
                        print_json(&response)?;
                    }
                    tokio::time::sleep(interval).await;
                }
            } else {
                let response =
                    refresh_bundle_heartbeat(&args.output, None, args.probe_base_url).await?;
                if args.summary {
                    println!("{}", render_bundle_heartbeat_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
        }
        Commands::Claims(args) => {
            let response = run_claims_command(&args, &base_url).await?;
            if args.summary {
                println!("{}", render_claims_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Messages(args) => {
            let response = run_messages_command(&args, &base_url).await?;
            if args.summary {
                println!("{}", render_messages_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Tasks(args) => {
            let response = run_tasks_command(&args, &base_url).await?;
            if args.json {
                print_json(&response)?;
            } else {
                println!("{}", render_tasks_summary(&response));
            }
        }
        Commands::Coordination(args) => {
            if args.watch {
                let interval = Duration::from_secs(args.interval_secs.max(1));
                let mut previous: Option<CoordinationResponse> = None;
                loop {
                    let response = run_coordination_command(&args, &base_url).await?;
                    if args.summary {
                        let alerts = render_coordination_alerts(
                            previous.as_ref(),
                            &response,
                            args.view.as_deref(),
                        );
                        if previous.is_none() || !alerts.is_empty() {
                            println!("[{}]", Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
                            for line in alerts {
                                println!("{line}");
                            }
                            println!(
                                "{}",
                                render_coordination_summary(&response, args.view.as_deref())
                            );
                            println!();
                        }
                    } else {
                        print_json(&response)?;
                    }
                    previous = Some(response);
                    tokio::time::sleep(interval).await;
                }
            } else if args.changes_only {
                let response = run_coordination_command(&args, &base_url).await?;
                let changes = build_coordination_change_response(
                    &args.output,
                    &response,
                    args.view.as_deref(),
                )?;
                if args.summary {
                    println!("{}", render_coordination_change_summary(&changes));
                } else {
                    print_json(&changes)?;
                }
            } else {
                let response = run_coordination_command(&args, &base_url).await?;
                if args.summary {
                    println!(
                        "{}",
                        render_coordination_summary(&response, args.view.as_deref())
                    );
                } else {
                    print_json(&response)?;
                }
            }
        }
        Commands::Bundle(args) => {
            if let Some(value) = args.hive_system.as_deref() {
                set_bundle_hive_system(&args.output, value)?;
            }
            if let Some(value) = args.hive_role.as_deref() {
                set_bundle_hive_role(&args.output, value)?;
            }
            if !args.capability.is_empty() {
                set_bundle_capabilities(&args.output, &args.capability)?;
            }
            if !args.hive_group.is_empty() {
                set_bundle_hive_groups(&args.output, &args.hive_group)?;
            }
            if let Some(value) = args.hive_group_goal.as_deref() {
                set_bundle_hive_group_goal(&args.output, value)?;
            }
            if let Some(value) = args.authority.as_deref() {
                set_bundle_authority(&args.output, value)?;
            }
            if let Some(value) = args.base_url.as_deref() {
                set_bundle_base_url(&args.output, value)?;
            }
            if let Some(value) = args.route.as_deref() {
                set_bundle_route(&args.output, value)?;
            }
            if let Some(value) = args.intent.as_deref() {
                set_bundle_intent(&args.output, value)?;
            }
            if let Some(value) = args.voice_mode.as_deref() {
                set_bundle_voice_mode(&args.output, value)?;
            }
            if let Some(value) = args.tab_id.as_deref() {
                set_bundle_tab_id(&args.output, value)?;
            }
            if let Some(value) = args.auto_short_term_capture {
                set_bundle_auto_short_term_capture(&args.output, value)?;
            }
            let status = read_bundle_status(&args.output, &base_url).await?;
            if args.summary {
                let base_url = status
                    .get("defaults")
                    .and_then(|value| value.get("base_url"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("none");
                let enabled = status
                    .get("defaults")
                    .and_then(|value| value.get("auto_short_term_capture"))
                    .and_then(|value| value.as_bool())
                    .unwrap_or(true);
                let route = status
                    .get("defaults")
                    .and_then(|value| value.get("route"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("auto");
                let intent = status
                    .get("defaults")
                    .and_then(|value| value.get("intent"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("general");
                let hive_system = status
                    .get("defaults")
                    .and_then(|value| value.get("hive_system"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("none");
                let hive_role = status
                    .get("defaults")
                    .and_then(|value| value.get("hive_role"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("none");
                let authority = status
                    .get("defaults")
                    .and_then(|value| value.get("authority"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("none");
                let hive_groups = status
                    .get("defaults")
                    .and_then(|value| value.get("hive_groups"))
                    .and_then(|value| value.as_array())
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(|value| value.as_str())
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "none".to_string());
                let hive_group_goal = status
                    .get("defaults")
                    .and_then(|value| value.get("hive_group_goal"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("none");
                println!(
                    "bundle={} hive={} role={} groups={} goal=\"{}\" authority={} base_url={} route={} intent={} auto_short_term_capture={}",
                    args.output.display(),
                    hive_system,
                    hive_role,
                    hive_groups,
                    hive_group_goal,
                    authority,
                    base_url,
                    route,
                    intent,
                    if enabled { "true" } else { "false" }
                );
            } else {
                print_json(&status)?;
            }
        }
        Commands::Hive(args) => match &args.command {
            Some(HiveSubcommand::Roster(roster_args)) => {
                let response = run_hive_roster_command(roster_args).await?;
                if roster_args.json {
                    print_json(&response)?;
                } else {
                    println!("{}", render_hive_roster_summary(&response));
                }
            }
            Some(HiveSubcommand::Follow(follow_args)) => {
                if follow_args.watch {
                    run_hive_follow_watch(follow_args).await?;
                } else {
                    let response = run_hive_follow_command(follow_args).await?;
                    if follow_args.json {
                        print_json(&response)?;
                    } else {
                        println!("{}", render_hive_follow_summary(&response));
                    }
                }
            }
            Some(HiveSubcommand::Handoff(handoff_args)) => {
                let response = run_hive_handoff_command(handoff_args, &default_base_url()).await?;
                if handoff_args.json {
                    print_json(&response)?;
                } else {
                    println!("{}", render_hive_handoff_summary(&response));
                }
            }
            Some(HiveSubcommand::Cowork {
                command: cowork_args,
            }) => match cowork_args {
                HiveCoworkSubcommand::Request(cowork_args) => {
                    let response =
                        run_hive_cowork_command(cowork_args, &default_base_url(), "request")
                            .await?;
                    if cowork_args.json {
                        print_json(&response)?;
                    } else {
                        println!("{}", render_hive_cowork_summary(&response));
                    }
                }
                HiveCoworkSubcommand::Ack(cowork_args) => {
                    let response =
                        run_hive_cowork_command(cowork_args, &default_base_url(), "ack").await?;
                    if cowork_args.json {
                        print_json(&response)?;
                    } else {
                        println!("{}", render_hive_cowork_summary(&response));
                    }
                }
                HiveCoworkSubcommand::Decline(cowork_args) => {
                    let response =
                        run_hive_cowork_command(cowork_args, &default_base_url(), "decline")
                            .await?;
                    if cowork_args.json {
                        print_json(&response)?;
                    } else {
                        println!("{}", render_hive_cowork_summary(&response));
                    }
                }
            },
            Some(HiveSubcommand::Queen(queen_args)) => {
                let response = run_hive_queen_command(queen_args, &default_base_url()).await?;
                if queen_args.json {
                    print_json(&response)?;
                } else {
                    println!("{}", render_hive_queen_summary(&response));
                }
            }
            None => {
                if args.summary {
                    let response = run_hive_board_command(&args, &default_base_url()).await?;
                    println!("{}", render_hive_board_summary(&response));
                } else {
                    let response = run_hive_command(&args).await?;
                    print_json(&response)?;
                }
            }
        },
        Commands::HiveProject(args) => {
            let response = run_hive_project_command(&args).await?;
            if args.summary {
                println!("{}", render_hive_project_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::HiveJoin(args) => {
            let response = run_hive_join_command(&args).await?;
            if args.summary {
                println!("{}", render_hive_join_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Eval(args) => {
            run_eval_command(&args, &base_url).await?;
        }
        Commands::Gap(args) => {
            run_gap_command(&args).await?;
        }
        Commands::Improve(args) => {
            run_improve_command(&args, &base_url).await?;
        }
        Commands::Scenario(args) => {
            run_scenario_command_cli(&args, &base_url).await?;
        }
        Commands::Composite(args) => {
            run_composite_command_cli(&args, &base_url).await?;
        }
        Commands::Benchmark(args) => {
            run_benchmark_command(&args, &base_url).await?;
        }
        Commands::Verify(args) => {
            run_verify_command(&args).await?;
        }
        Commands::Experiment(args) => {
            run_experiment_command_cli(&args, &base_url).await?;
        }
        Commands::Agent(args) => {
            if args.apply {
                let Some(name) = args.name.as_deref() else {
                    anyhow::bail!("memd agent --apply requires --name <agent>");
                };
                set_bundle_agent(&args.output, name)?;
                if let Some(session) = args.session.as_deref() {
                    set_bundle_session(&args.output, session)?;
                }
                let snapshot = crate::runtime::read_bundle_resume(
                    &ResumeArgs {
                        output: args.output.clone(),
                        project: None,
                        namespace: None,
                        agent: Some(name.to_string()),
                        workspace: None,
                        visibility: None,
                        route: None,
                        intent: None,
                        limit: Some(8),
                        rehydration_limit: Some(4),
                        semantic: false,
                        prompt: false,
                        summary: false,
                    },
                    &base_url,
                )
                .await?;
                write_bundle_memory_files(&args.output, &snapshot, None, false).await?;
                auto_checkpoint_live_snapshot(&args.output, &base_url, &snapshot, "agent").await?;
            } else if let Some(session) = args.session.as_deref() {
                set_bundle_session(&args.output, session)?;
            }
            let response = crate::build_bundle_agent_profiles(
                &args.output,
                args.name.as_deref(),
                args.shell.as_deref(),
            )?;
            if args.summary {
                println!("{}", crate::render_bundle_agent_profiles_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Attach(args) => {
            let shell = args
                .shell
                .or_else(crate::detect_shell)
                .unwrap_or_else(|| "bash".to_string());
            println!("{}", render_attach_snippet(&shell, &args.output)?);
        }
        Commands::Resume(args) => {
            let codex_pack =
                harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "codex");
            let agent_zero_pack =
                harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "agent-zero");
            let hermes_pack =
                harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "hermes");
            let opencode_pack =
                harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "opencode");
            let openclaw_pack =
                harness_pack_enabled_for_bundle(&args.output, args.agent.as_deref(), "openclaw");
            let snapshot = match crate::runtime::read_bundle_resume(&args, &base_url).await {
                Ok(snapshot) => snapshot,
                Err(err)
                    if codex_pack
                        || agent_zero_pack
                        || hermes_pack
                        || opencode_pack
                        || openclaw_pack =>
                {
                    if let Some(markdown) =
                        read_codex_pack_local_markdown(&args.output, "mem.md")?
                    {
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
                        println!("{markdown}");
                        return Ok(());
                    }
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
                    return Err(err);
                }
                Err(err) => return Err(err),
            };
            write_bundle_memory_files(&args.output, &snapshot, None, false).await?;
            auto_checkpoint_live_snapshot(&args.output, &base_url, &snapshot, "resume").await?;
            if codex_pack || agent_zero_pack || openclaw_pack || hermes_pack || opencode_pack {
                let _ = refresh_harness_pack_files_for_snapshot(
                    &args.output,
                    &snapshot,
                    "resume",
                    &["codex", "agent-zero", "openclaw", "hermes", "opencode"],
                )
                .await?;
            }
            if args.prompt {
                println!("{}", render_resume_prompt(&snapshot));
            } else if args.summary {
                let focus = snapshot
                    .working
                    .records
                    .first()
                    .map(|record| compact_inline(&record.record, 72))
                    .unwrap_or_else(|| "none".to_string());
                let pressure = snapshot
                    .inbox
                    .items
                    .first()
                    .map(|item| compact_inline(&item.item.content, 72))
                    .unwrap_or_else(|| "none".to_string());
                println!(
                    "resume project={} namespace={} agent={} workspace={} visibility={} context={} working={} inbox={} workspaces={} changes={} est_tokens={} context_pressure={} redundant_items={} refresh_recommended={} focus=\"{}\" pressure=\"{}\"",
                    snapshot.project.as_deref().unwrap_or("none"),
                    snapshot.namespace.as_deref().unwrap_or("none"),
                    snapshot.agent.as_deref().unwrap_or("none"),
                    snapshot.workspace.as_deref().unwrap_or("none"),
                    snapshot.visibility.as_deref().unwrap_or("all"),
                    snapshot.context.records.len(),
                    snapshot.working.records.len(),
                    snapshot.inbox.items.len(),
                    snapshot.workspaces.workspaces.len(),
                    snapshot.change_summary.len(),
                    snapshot.estimated_prompt_tokens(),
                    snapshot.context_pressure(),
                    snapshot.redundant_context_items(),
                    snapshot.refresh_recommended,
                    focus,
                    pressure,
                );
            } else {
                print_json(&snapshot)?;
            }
        }
        Commands::Refresh(args) => {
            crate::runtime::invalidate_bundle_runtime_caches(&args.output)?;
            let snapshot = crate::runtime::read_bundle_resume(&args, &base_url).await?;
            write_bundle_memory_files(&args.output, &snapshot, None, false).await?;
            auto_checkpoint_live_snapshot(&args.output, &base_url, &snapshot, "refresh").await?;
            let _ = refresh_harness_pack_files_for_snapshot(
                &args.output,
                &snapshot,
                "refresh",
                &["codex", "agent-zero", "openclaw", "hermes", "opencode"],
            )
            .await?;
            if args.prompt {
                println!("{}", render_resume_prompt(&snapshot));
            } else {
                let focus = snapshot
                    .working
                    .records
                    .first()
                    .map(|record| compact_inline(&record.record, 72))
                    .unwrap_or_else(|| "none".to_string());
                let pressure = snapshot
                    .inbox
                    .items
                    .first()
                    .map(|item| compact_inline(&item.item.content, 72))
                    .unwrap_or_else(|| "none".to_string());
                println!(
                    "refresh project={} namespace={} agent={} workspace={} visibility={} context={} working={} inbox={} workspaces={} changes={} est_tokens={} context_pressure={} redundant_items={} refresh_recommended={} focus=\"{}\" pressure=\"{}\"",
                    snapshot.project.as_deref().unwrap_or("none"),
                    snapshot.namespace.as_deref().unwrap_or("none"),
                    snapshot.agent.as_deref().unwrap_or("none"),
                    snapshot.workspace.as_deref().unwrap_or("none"),
                    snapshot.visibility.as_deref().unwrap_or("all"),
                    snapshot.context.records.len(),
                    snapshot.working.records.len(),
                    snapshot.inbox.items.len(),
                    snapshot.workspaces.workspaces.len(),
                    snapshot.change_summary.len(),
                    snapshot.estimated_prompt_tokens(),
                    snapshot.context_pressure(),
                    snapshot.redundant_context_items(),
                    snapshot.refresh_recommended,
                    focus,
                    pressure,
                );
            }
        }
        Commands::Watch(args) => {
            run_workspace_watch(&client, &base_url, &args).await?;
        }
        Commands::Handoff(args) => {
            let snapshot = crate::runtime::read_bundle_handoff(&args, &base_url).await?;
            write_bundle_memory_files(&args.output, &snapshot.resume, Some(&snapshot), false)
                .await?;
            auto_checkpoint_live_snapshot(&args.output, &base_url, &snapshot.resume, "handoff")
                .await?;
            if args.prompt {
                println!("{}", render_handoff_prompt(&snapshot));
            } else if args.summary {
                println!(
                    "handoff project={} namespace={} agent={} workspace={} visibility={} working={} inbox={} workspaces={} sources={} rehydration={} target_session={} target_bundle={}",
                    snapshot.resume.project.as_deref().unwrap_or("none"),
                    snapshot.resume.namespace.as_deref().unwrap_or("none"),
                    snapshot.resume.agent.as_deref().unwrap_or("none"),
                    snapshot.resume.workspace.as_deref().unwrap_or("none"),
                    snapshot.resume.visibility.as_deref().unwrap_or("all"),
                    snapshot.resume.working.records.len(),
                    snapshot.resume.inbox.items.len(),
                    snapshot.resume.workspaces.workspaces.len(),
                    snapshot.sources.sources.len(),
                    snapshot.resume.working.rehydration_queue.len(),
                    snapshot.target_session.as_deref().unwrap_or("none"),
                    snapshot.target_bundle.as_deref().unwrap_or("none"),
                );
            } else {
                print_json(&snapshot)?;
            }
        }
        Commands::Checkpoint(args) => {
            let (default_project, default_namespace) = infer_bundle_identity_defaults(&args.output);
            let response = match checkpoint_with_bundle_defaults(&args, &base_url).await {
                Ok(response) => response,
                Err(err) => {
                    write_bundle_turn_placeholder_memory(
                        &args.output,
                        args.project.as_deref(),
                        args.namespace.as_deref(),
                        None,
                        args.workspace.as_deref(),
                        args.visibility.as_deref(),
                        Some("auto"),
                        Some("current_task"),
                    )?;
                    return Err(err);
                }
            };
            let snapshot = crate::runtime::read_bundle_resume(
                &ResumeArgs {
                    output: args.output.clone(),
                    project: args.project.clone().or(default_project),
                    namespace: args.namespace.clone().or(default_namespace),
                    agent: None,
                    workspace: args.workspace.clone(),
                    visibility: args.visibility.clone(),
                    route: None,
                    intent: Some("current_task".to_string()),
                    limit: Some(8),
                    rehydration_limit: Some(4),
                    semantic: false,
                    prompt: false,
                    summary: false,
                },
                &base_url,
            )
            .await?;
            write_bundle_memory_files(&args.output, &snapshot, None, false).await?;
            refresh_live_bundle_event_pages(&args.output, &snapshot, None)?;
            auto_checkpoint_live_snapshot(&args.output, &base_url, &snapshot, "checkpoint").await?;
            let _ = refresh_harness_pack_files_for_snapshot(
                &args.output,
                &snapshot,
                "checkpoint",
                &["codex", "agent-zero", "openclaw", "hermes", "opencode"],
            )
            .await?;
            print_json(&response)?;
        }
        Commands::Remember(args) => {
            let (default_project, default_namespace) = infer_bundle_identity_defaults(&args.output);
            let response = remember_with_bundle_defaults(&args, &base_url).await?;
            let snapshot = crate::runtime::read_bundle_resume(
                &ResumeArgs {
                    output: args.output.clone(),
                    project: args.project.clone().or(default_project),
                    namespace: args.namespace.clone().or(default_namespace),
                    agent: None,
                    workspace: args.workspace.clone(),
                    visibility: args.visibility.clone(),
                    route: None,
                    intent: Some("current_task".to_string()),
                    limit: Some(8),
                    rehydration_limit: Some(4),
                    semantic: false,
                    prompt: false,
                    summary: false,
                },
                &base_url,
            )
            .await?;
            write_bundle_memory_files(&args.output, &snapshot, None, false).await?;
            auto_checkpoint_live_snapshot(&args.output, &base_url, &snapshot, "remember").await?;
            print_json(&response)?;
        }
        Commands::Rag(args) => {
            run_rag_mode(&client, args).await?;
        }
        Commands::Multimodal(args) => {
            run_multimodal_mode(args).await?;
        }
        Commands::Inspiration(args) => {
            run_inspiration_command(args)?;
        }
        Commands::Skills(args) => {
            run_skill_catalog_command(args)?;
        }
        Commands::Packs(args) => {
            run_pack_catalog_command(args)?;
        }
        Commands::Commands(args) => {
            run_command_catalog_command(args)?;
        }
        Commands::Setup(args) => {
            crate::run_bundle_setup_command(&args).await?;
        }
        Commands::Doctor(args) => {
            crate::run_bundle_doctor_command(&args, &base_url).await?;
        }
        Commands::Config(args) => {
            crate::run_bundle_config_command(&args, &base_url).await?;
        }
        Commands::Memory(args) => {
            run_memory_command(&client, &base_url, &args).await?;
        }
        Commands::Ingest(args) => {
            run_ingest_command(&client, &args).await?;
        }
        Commands::Store(input) => {
            run_store_command(&client, &input).await?;
        }
        Commands::Candidate(input) => {
            run_candidate_command(&client, &input).await?;
        }
        Commands::Promote(input) => {
            run_promote_command(&client, &input).await?;
        }
        Commands::Expire(input) => {
            run_expire_command(&client, &input).await?;
        }
        Commands::MemoryVerify(input) => {
            run_memory_verify_command(&client, &input).await?;
        }
        Commands::Repair(args) => {
            run_repair_command(&client, args).await?;
        }
        Commands::Search(args) => {
            run_search_command(&client, args).await?;
        }
        Commands::Lookup(args) => {
            run_lookup_command(&client, args).await?;
        }
        Commands::Context(args) => {
            run_context_command(&client, args).await?;
        }
        Commands::Working(args) => {
            run_working_command(&client, args).await?;
        }
        Commands::Profile(args) => {
            run_profile_command(&client, args).await?;
        }
        Commands::Source(args) => {
            run_source_command(&client, args).await?;
        }
        Commands::Workspaces(args) => {
            run_workspaces_command(&client, args).await?;
        }
        Commands::Inbox(args) => {
            run_inbox_command(&client, args).await?;
        }
        Commands::Explain(args) => {
            run_explain_command(&client, args).await?;
        }
        Commands::Entity(args) => {
            run_entity_command(&client, args).await?;
        }
        Commands::EntitySearch(args) => {
            run_entity_search_command(&client, args).await?;
        }
        Commands::EntityLink(args) => {
            run_entity_link_command(&client, args).await?;
        }
        Commands::EntityLinks(args) => {
            run_entity_links_command(&client, args).await?;
        }
        Commands::Recall(args) => {
            run_recall_command(&client, args).await?;
        }
        Commands::Timeline(args) => {
            run_timeline_command(&client, args).await?;
        }
        Commands::Events(args) => {
            run_events_command(args)?;
        }
        Commands::Consolidate(args) => {
            run_consolidate_command(&client, args).await?;
        }
        Commands::MaintenanceReport(args) => {
            run_maintenance_report_command(&client, args).await?;
        }
        Commands::Maintain(args) => {
            run_bundle_maintain_command(args, &cli.base_url).await?;
        }
        Commands::Policy(args) => {
            run_policy_command(&client, args).await?;
        }
        Commands::SkillPolicy(args) => {
            run_skill_policy_command(&client, args).await?;
        }
        Commands::Compact(args) => {
            run_compact_command(&client, &base_url, args).await?;
        }
        Commands::Obsidian(args) => {
            run_obsidian_mode(&client, &base_url, args).await?;
        }
        Commands::Ui(args) => match args.mode {
            UiMode::Home(args) => {
                let snapshot = client.visible_memory_snapshot().await?;
                if args.json {
                    print_json(&snapshot)?;
                } else {
                    println!("{}", render_visible_memory_home(&snapshot, args.follow));
                }
            }
            UiMode::Artifact(args) => {
                let artifact_id = uuid::Uuid::parse_str(&args.id)
                    .with_context(|| format!("parse visible memory artifact id {}", args.id))?;
                let detail = client.visible_memory_artifact_detail(artifact_id).await?;
                if args.json {
                    print_json(&detail)?;
                } else {
                    println!(
                        "{}",
                        render_visible_memory_artifact_detail(&detail, args.follow)
                    );
                }
            }
            UiMode::Map(args) => {
                let snapshot = client.visible_memory_snapshot().await?;
                if args.json {
                    print_json(&snapshot)?;
                } else {
                    println!(
                        "{}",
                        render_visible_memory_knowledge_map(&snapshot, args.follow)
                    );
                }
            }
        },
        Commands::Hook(args) => {
            run_hook_mode(&client, &base_url, args).await?;
        }
        Commands::Init(args) => {
            crate::run_bundle_init_command(args).await?;
        }
        Commands::Loops(args) => {
            run_loops_command(args)?;
        }
        Commands::Telemetry(args) => {
            run_telemetry_command(args)?;
        }
        Commands::Autoresearch(args) => {
            run_autoresearch_command(args, &base_url).await?;
        }
    }

    Ok(())
}
