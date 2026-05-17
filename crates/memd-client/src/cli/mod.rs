use super::*;
use crate::runtime::*;
use std::net::ToSocketAddrs;

pub(crate) mod args;
pub(crate) use args::*;

pub(crate) mod command_catalog;

mod commands;
pub(crate) use commands::*;

mod cli_memory_runtime;
pub(crate) use cli_memory_runtime::*;

mod cli_memory_os_runtime;
pub(crate) use cli_memory_os_runtime::*;

mod cli_live_state_runtime;
pub(crate) use cli_live_state_runtime::*;

mod cli_awareness_runtime;
pub(crate) use cli_awareness_runtime::*;

mod cli_dev_server_runtime;
pub(crate) use cli_dev_server_runtime::*;

mod cli_analysis_runtime;
pub(crate) use cli_analysis_runtime::*;

mod cli_obsidian_runtime;
pub(crate) use cli_obsidian_runtime::*;

mod cli_hook_runtime;
pub(crate) use cli_hook_runtime::*;

mod cli_hook_enforce;
pub(crate) use cli_hook_enforce::*;

pub(crate) mod cli_gate_runtime;
pub(crate) use cli_gate_runtime::*;

mod cli_rag_runtime;
pub(crate) use cli_rag_runtime::*;

mod cli_embed_runtime;
pub(crate) use cli_embed_runtime::*;

mod cli_utility_runtime;
pub(crate) use cli_utility_runtime::*;

mod cli_inspection_runtime;
pub(crate) use cli_inspection_runtime::*;

mod cli_lifecycle_probe_runtime;
pub(crate) use cli_lifecycle_probe_runtime::*;

mod cli_contract_runtime;
pub(crate) use cli_contract_runtime::*;

mod cli_correction_runtime;
pub(crate) use cli_correction_runtime::*;

mod cli_preference_runtime;
pub(crate) use cli_preference_runtime::*;

mod skill_runtime;
pub(crate) use skill_runtime::*;

pub(crate) mod skill_catalog;

fn bundle_auto_commit_enabled_for(output: &Path) -> bool {
    if let Ok(value) = std::env::var("MEMD_AUTO_COMMIT_ENABLED") {
        return matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on" | "enabled"
        );
    }

    read_bundle_runtime_config(output)
        .ok()
        .flatten()
        .map(|config| config.auto_commit.enabled)
        .unwrap_or(true)
}

pub(crate) fn teach_args_as_remember_args(args: &TeachArgs) -> RememberArgs {
    let mut tags = args.tag.clone();
    if !tags.iter().any(|tag| tag == "user-taught") {
        tags.push("user-taught".to_string());
    }
    if !tags.iter().any(|tag| tag == "teach") {
        tags.push("teach".to_string());
    }
    RememberArgs {
        output: args.output.clone(),
        project: args.project.clone(),
        namespace: args.namespace.clone(),
        workspace: args.workspace.clone(),
        visibility: args.visibility.clone(),
        kind: Some(args.kind.clone()),
        scope: None,
        source_agent: args.source_agent.clone(),
        source_system: Some("user-teach".to_string()),
        source_path: None,
        source_quality: Some("canonical".to_string()),
        confidence: Some(args.confidence.unwrap_or(0.8)),
        ttl_seconds: None,
        tag: tags,
        supersede: args.supersede.clone(),
        content: args.content.clone(),
        input: args.input.clone(),
        stdin: args.stdin,
    }
}

fn maybe_auto_commit_before_write(
    output: Option<&Path>,
    label: &str,
    detail: Option<&str>,
) -> anyhow::Result<()> {
    let enabled = output
        .map(bundle_auto_commit_enabled_for)
        .unwrap_or_else(|| {
            resolve_default_bundle_root()
                .ok()
                .flatten()
                .as_deref()
                .map(bundle_auto_commit_enabled_for)
                .unwrap_or(true)
        });
    if !enabled {
        return Ok(());
    }

    let suffix = detail
        .map(|value| value.chars().take(72).collect::<String>())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| label.to_string());
    let commit_msg = format!("memd auto-commit: {suffix}");
    let repo_root = output.and_then(infer_bundle_project_root);
    let hash = if let Some(repo_root) = repo_root.as_deref() {
        crate::runtime::git_auto_commit_if_dirty_in(&commit_msg, Some(repo_root))?
    } else {
        crate::runtime::git_auto_commit_if_dirty(&commit_msg)?
    };
    if let Some(hash) = hash {
        eprintln!("memd: auto-committed dirty tree before {label} ({hash})");
    }
    Ok(())
}

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
        Commands::State(args) => {
            let state = read_bundle_state(&args.output, &base_url).await?;
            if args.json {
                print_json(&state)?;
            } else {
                println!("{}", render_bundle_state_summary(&state));
            }
        }
        Commands::Claim(args) => {
            let args = match args.command {
                ClaimSubcommand::Create(args) => ClaimsArgs::from(args),
                ClaimSubcommand::List(args) => ClaimsArgs::from(args),
                ClaimSubcommand::Close(args) => ClaimsArgs::from(args),
            };
            let response = run_claims_command(&args, &base_url).await?;
            if args.summary {
                println!("{}", render_claims_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Capabilities(args) => {
            let json = match &args.command {
                Some(CapabilitiesSubcommand::Pull(args)) => args.json,
                Some(CapabilitiesSubcommand::Status(args)) => args.json,
                Some(CapabilitiesSubcommand::Sync(args)) => args.json,
                None => args.json,
            };
            let sync_output = match &args.command {
                Some(CapabilitiesSubcommand::Sync(args)) => Some(args.output.clone()),
                _ => None,
            };
            let response = if matches!(&args.command, Some(CapabilitiesSubcommand::Pull(_))) {
                pull_capabilities_from_server(&client, &base_url, &args).await?
            } else {
                let response = run_capabilities_command(&args)?;
                if let Some(output) = sync_output.as_deref()
                    && let Err(error) =
                        push_capabilities_to_server(&client, &base_url, output, &response).await
                {
                    eprintln!("memd: capability server sync skipped ({error})");
                }
                response
            };
            if json {
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
        Commands::Features(args) => {
            let response = run_features_command(&args)?;
            if args.json {
                print_json(&response)?;
            } else {
                println!("{}", render_feature_summary(&response));
            }
        }
        Commands::Health(args) => {
            let mut response = run_health_command(&args)?;
            if base_url_reachable(&base_url, Duration::from_millis(250))
                && let Ok(server) = fetch_server_token_savings(&client, &args.output, None).await
            {
                response = merge_health_server_token_savings(response, server);
            }
            if args.json {
                print_json(&response)?;
            } else {
                println!("{}", render_health_summary(&response));
            }
        }
        Commands::Access(args) => {
            let json = match &args.command {
                AccessSubcommand::Status(args) => args.json,
                AccessSubcommand::Route(args) => args.json,
                AccessSubcommand::Sync(args) => args.json,
            };
            let sync_output = match &args.command {
                AccessSubcommand::Sync(args) => Some(args.output.clone()),
                _ => None,
            };
            let response = run_access_command(&args)?;
            if let Some(output) = sync_output.as_deref()
                && let Err(error) =
                    push_access_routes_to_server(&client, &base_url, output, &response).await
            {
                eprintln!("memd: access-route server sync skipped ({error})");
            }
            if json {
                print_json(&response)?;
            } else {
                println!("{}", render_access_summary(&response));
            }
        }
        Commands::LiveState(args) => {
            let json = match &args.command {
                LiveStateSubcommand::Ingest(args) => args.json,
                LiveStateSubcommand::Status(args) => args.json,
            };
            let tasks = matches!(&args.command, LiveStateSubcommand::Status(args) if args.tasks);
            let check = matches!(&args.command, LiveStateSubcommand::Status(args) if args.check);
            let response = run_live_state_command(&args)?;
            if json {
                print_json(&response)?;
            } else if tasks {
                println!("{}", render_live_state_task_lines(&response));
            } else {
                println!("{}", render_live_state_summary(&response));
            }
            if check {
                let due_within_secs = match &args.command {
                    LiveStateSubcommand::Status(args) => args.due_within_secs,
                    LiveStateSubcommand::Ingest(_) => 0,
                };
                if live_state_check_required(&response, due_within_secs) {
                    return Err(anyhow::Error::new(LiveStateCheckExitCode(2)));
                }
            }
        }
        Commands::Secrets(args) => {
            let json = match &args.command {
                SecretsSubcommand::Status(args) | SecretsSubcommand::Providers(args) => args.json,
            };
            let response = run_secrets_command(&args)?;
            if json {
                print_json(&response)?;
            } else {
                println!("{}", render_secrets_summary(&response));
            }
        }
        Commands::Tokens(args) => {
            let json = match &args.command {
                TokensSubcommand::Saved(args) => args.json,
                TokensSubcommand::Sync(args) => args.json,
            };
            let sync_output = match &args.command {
                TokensSubcommand::Sync(args) => Some(args.output.clone()),
                _ => None,
            };
            let mut response = run_tokens_command(&args)?;
            let report_scope = tokens_report_scope(&args);
            if sync_output.is_none()
                && let Some((output, since)) = report_scope
                && base_url_reachable(&base_url, Duration::from_millis(250))
                && let Ok(server) = fetch_server_token_savings(&client, output, since).await
            {
                response = merge_server_token_savings_report(response, server);
            }
            if let Some(output) = sync_output.as_deref() {
                match push_token_savings_to_server(&client, &base_url, output).await {
                    Ok(()) => {
                        if let Some((output, since)) = report_scope
                            && base_url_reachable(&base_url, Duration::from_millis(250))
                            && let Ok(server) =
                                fetch_server_token_savings(&client, output, since).await
                        {
                            response = merge_server_token_savings_report(response, server);
                        }
                    }
                    Err(error) => eprintln!("memd: token-savings server sync skipped ({error})"),
                }
            }
            if json {
                print_json(&response)?;
            } else {
                println!("{}", render_tokens_summary(&response));
            }
        }
        Commands::DevServer(args) => {
            let response = run_dev_server_command(&args, &base_url).await?;
            if response.summary_mode {
                println!("{}", render_dev_server_summary(&response));
            } else {
                print_json(&response)?;
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
                    if let Some(markdown) = read_codex_pack_local_markdown(&args.output, "mem.md")?
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
                let voice_mode =
                    read_bundle_voice_mode(&args.output).unwrap_or_else(default_voice_mode);
                let dirty =
                    crate::workflow::repo_dirty_count_from_changes(&snapshot.recent_repo_changes);
                let handoff_quality = snapshot
                    .handoff_quality
                    .as_ref()
                    .map(|score| {
                        if score.is_acceptable() {
                            format!("ready:{:.2}", score.composite)
                        } else {
                            format!("partial:{:.2}", score.composite)
                        }
                    })
                    .unwrap_or_else(|| "unknown".to_string());
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
                    "resume project={} namespace={} agent={} voice={} handoff_quality={} dirty={} workspace={} visibility={} context={} working={} inbox={} workspaces={} changes={} est_tokens={} context_pressure={} redundant_items={} refresh_recommended={} focus=\"{}\" pressure=\"{}\"",
                    snapshot.project.as_deref().unwrap_or("none"),
                    snapshot.namespace.as_deref().unwrap_or("none"),
                    snapshot.agent.as_deref().unwrap_or("none"),
                    voice_mode,
                    handoff_quality,
                    dirty,
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
                let voice_mode =
                    read_bundle_voice_mode(&args.output).unwrap_or_else(default_voice_mode);
                let mut value =
                    serde_json::to_value(&snapshot).context("serialize resume snapshot")?;
                if let serde_json::Value::Object(map) = &mut value {
                    map.insert(
                        "voice_mode".to_string(),
                        serde_json::Value::String(voice_mode),
                    );
                }
                print_json(&value)?;
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
            maybe_auto_commit_before_write(Some(&args.output), "handoff", Some("handoff"))?;
            let snapshot = crate::runtime::read_bundle_handoff(&args, &base_url).await?;
            write_bundle_memory_files(&args.output, &snapshot.resume, Some(&snapshot), false)
                .await?;
            auto_checkpoint_live_snapshot(&args.output, &base_url, &snapshot.resume, "handoff")
                .await?;
            if args.prompt {
                println!("{}", render_handoff_prompt(&snapshot));
            } else if args.summary {
                let dirty = crate::workflow::repo_dirty_count_from_changes(
                    &snapshot.resume.recent_repo_changes,
                );
                let quality = snapshot
                    .resume
                    .handoff_quality
                    .as_ref()
                    .map(|score| {
                        if score.is_acceptable() {
                            "ready"
                        } else {
                            "partial"
                        }
                    })
                    .unwrap_or("unknown");
                println!(
                    "handoff project={} namespace={} agent={} voice={} quality={} dirty={} workspace={} visibility={} working={} inbox={} workspaces={} sources={} rehydration={} target_session={} target_bundle={}",
                    snapshot.resume.project.as_deref().unwrap_or("none"),
                    snapshot.resume.namespace.as_deref().unwrap_or("none"),
                    snapshot.resume.agent.as_deref().unwrap_or("none"),
                    snapshot.voice_mode,
                    quality,
                    dirty,
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
            // Update ROADMAP_STATE before auto-commit so changes are included
            if !args.roadmap_set.is_empty() {
                let updates: Vec<(String, String)> = args
                    .roadmap_set
                    .iter()
                    .filter_map(|pair| {
                        let (key, value) = pair.split_once('=')?;
                        Some((key.trim().to_string(), value.trim().to_string()))
                    })
                    .collect();
                match crate::runtime::update_roadmap_state(&updates) {
                    Ok(true) => {
                        eprintln!("memd: updated ROADMAP_STATE ({} keys)", updates.len());
                    }
                    Ok(false) => {} // no changes needed
                    Err(err) => {
                        eprintln!("memd: roadmap-set failed (non-fatal): {}", err);
                    }
                }
            }
            if args.auto_commit || bundle_auto_commit_enabled_for(&args.output) {
                maybe_auto_commit_before_write(
                    Some(&args.output),
                    "checkpoint",
                    args.content.as_deref(),
                )?;
            }
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
            if is_offline_queued_response(&response) {
                print_json(&response)?;
                return Ok(());
            }
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
            maybe_auto_commit_before_write(
                Some(&args.output),
                "remember",
                args.content.as_deref(),
            )?;
            let (default_project, default_namespace) = infer_bundle_identity_defaults(&args.output);
            let response = remember_with_bundle_defaults(&args, &base_url).await?;
            if is_offline_queued_response(&response) {
                print_json(&response)?;
                return Ok(());
            }
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
        Commands::Teach(args) => {
            let remember_args = teach_args_as_remember_args(&args);
            maybe_auto_commit_before_write(
                Some(&remember_args.output),
                "teach",
                remember_args.content.as_deref(),
            )?;
            let (default_project, default_namespace) =
                infer_bundle_identity_defaults(&remember_args.output);
            let response = remember_with_bundle_defaults(&remember_args, &base_url).await?;
            if is_offline_queued_response(&response) {
                print_json(&response)?;
                return Ok(());
            }
            let snapshot = crate::runtime::read_bundle_resume(
                &ResumeArgs {
                    output: remember_args.output.clone(),
                    project: remember_args.project.clone().or(default_project),
                    namespace: remember_args.namespace.clone().or(default_namespace),
                    agent: None,
                    workspace: remember_args.workspace.clone(),
                    visibility: remember_args.visibility.clone(),
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
            write_bundle_memory_files(&remember_args.output, &snapshot, None, false).await?;
            auto_checkpoint_live_snapshot(&remember_args.output, &base_url, &snapshot, "teach")
                .await?;
            print_json(&response)?;
        }
        Commands::Embed(args) => {
            run_embed_mode(args).await?;
        }
        Commands::Rag(args) => {
            run_rag_mode(&client, args).await?;
        }
        Commands::Offline(args) | Commands::Sync(args) => match args.command {
            OfflineSubcommand::Status(queue_args) => {
                print_json(&offline_queue_status(&queue_args.output)?)?;
            }
            OfflineSubcommand::Replay(queue_args) => {
                let client = MemdClient::new(&base_url)?;
                let report = replay_offline_queue(&queue_args.output, &client).await?;
                print_json(&report)?;
            }
        },
        Commands::Multimodal(args) => {
            run_multimodal_mode(args).await?;
        }
        Commands::Inspiration(args) => {
            run_inspiration_command(args, &base_url).await?;
        }
        Commands::Skill(args) => {
            run_skill_command(&client, &base_url, args).await?;
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
        Commands::Device(args) => {
            crate::run_device_command(&args)?;
        }
        Commands::Dogfood(args) => {
            crate::run_dogfood_command(&args)?;
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
        Commands::IngestSources(args) => {
            run_ingest_sources_command(&client, &args).await?;
        }
        Commands::Store(input) => {
            maybe_auto_commit_before_write(None, "memory store", None)?;
            run_store_command(&client, &input).await?;
        }
        Commands::Candidate(input) => {
            maybe_auto_commit_before_write(None, "memory candidate", None)?;
            run_candidate_command(&client, &input).await?;
        }
        Commands::Promote(input) => {
            maybe_auto_commit_before_write(None, "memory promote", None)?;
            run_promote_command(&client, &input).await?;
        }
        Commands::Expire(input) => {
            maybe_auto_commit_before_write(None, "memory expire", None)?;
            run_expire_command(&client, &input).await?;
        }
        Commands::MemoryVerify(input) => {
            maybe_auto_commit_before_write(None, "memory verify", None)?;
            run_memory_verify_command(&client, &input).await?;
        }
        Commands::Repair(args) => {
            maybe_auto_commit_before_write(None, "memory repair", args.content.as_deref())?;
            run_repair_command(&client, args).await?;
        }
        Commands::Correct(args) => {
            maybe_auto_commit_before_write(None, "memory correct", Some(&args.content))?;
            run_correct_command(&client, args).await?;
        }
        Commands::Search(args) => {
            run_search_command(&client, args).await?;
        }
        Commands::Lookup(args) => {
            run_lookup_command(&client, &base_url, args).await?;
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
        Commands::Atlas(args) => {
            run_atlas_command(&client, args).await?;
        }
        Commands::Procedure(args) => {
            run_procedure_command(&client, args).await?;
        }
        Commands::Routines(args) => {
            run_routines_command(args)?;
        }
        Commands::Audit(args) => {
            run_audit_command(args)?;
        }
        Commands::Events(args) => {
            run_events_command(args)?;
        }
        Commands::Consolidate(args) => {
            run_consolidate_command(&client, args).await?;
        }
        Commands::Dedup(args) => {
            run_dedup_command(&client, args).await?;
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
        Commands::Compiler(args) => {
            run_compiler_command(args)?;
        }
        Commands::Autoresearch(args) => {
            run_autoresearch_command(args, &base_url).await?;
        }
        Commands::Diagnostics(args) => {
            run_diagnostics_command(args).await?;
        }
        Commands::PrimeReads(args) => {
            run_prime_reads(&args)?;
        }
        Commands::Contract(args) => match args.command {
            ContractCommand::Verify(v) => run_contract_verify(&v)?,
            ContractCommand::Generate(g) => run_contract_generate(&g)?,
        },
        Commands::Correction(args) => match args.command {
            CorrectionSubcommand::Detect(a) => run_correction_detect(&a)?,
            CorrectionSubcommand::Capture(a) => run_correction_capture(&a)?,
            CorrectionSubcommand::List(a) => run_correction_list(&a)?,
        },
        Commands::Preference(args) => match args.command {
            PreferenceSubcommand::List(a) => run_preference_list(&a)?,
            PreferenceSubcommand::Drift(a) => run_preference_drift(&a)?,
            PreferenceSubcommand::Confirm(a) => run_preference_confirm(&a)?,
            PreferenceSubcommand::Promote(a) => run_preference_promote(&a)?,
            PreferenceSubcommand::Tick(a) => run_preference_tick(&a)?,
        },
    }

    Ok(())
}

pub(crate) fn run_prime_reads(args: &PrimeReadsArgs) -> anyhow::Result<()> {
    use memd_core::file_ledger::FileInteractionLedger;
    let paths = if let Some(session) = &args.since_session {
        let p = args
            .output
            .join("state")
            .join(format!("session-{session}"))
            .join("file_interactions.json");
        FileInteractionLedger::load_from_path(&p)
            .map(|l| l.distinct_paths())
            .unwrap_or_default()
    } else {
        crate::runtime::collect_files_touched(&args.output)
    };
    for p in paths {
        println!("{p}");
    }
    Ok(())
}

async fn push_capabilities_to_server(
    client: &MemdClient,
    base_url: &str,
    output: &Path,
    response: &CapabilitiesResponse,
) -> anyhow::Result<()> {
    if response.records.is_empty() {
        return Ok(());
    }
    let config = read_memory_os_bundle_config(output).ok();
    let records = response
        .records
        .iter()
        .map(|record| memd_schema::CapabilityRecord {
            harness: record.harness.clone(),
            kind: record.kind.clone(),
            name: record.name.clone(),
            status: record.status.clone(),
            portability_class: record.portability_class.clone(),
            source_path: record.source_path.clone(),
            bridge_hint: record.bridge_hint.clone(),
            hash: record.hash.clone(),
            notes: record.notes.clone(),
            project: config.as_ref().and_then(|config| config.project.clone()),
            namespace: config.as_ref().and_then(|config| config.namespace.clone()),
            workspace: config.as_ref().and_then(|config| config.workspace.clone()),
            user_id: None,
            agent: config.as_ref().and_then(|config| config.agent.clone()),
            updated_at: None,
        })
        .collect::<Vec<_>>();
    let req = memd_schema::CapabilitySyncRequest {
        project: config.as_ref().and_then(|config| config.project.clone()),
        namespace: config.as_ref().and_then(|config| config.namespace.clone()),
        workspace: config.as_ref().and_then(|config| config.workspace.clone()),
        user_id: None,
        agent: config.as_ref().and_then(|config| config.agent.clone()),
        records,
    };
    if !base_url_reachable(base_url, Duration::from_millis(250)) {
        let error = format!("server not reachable at {base_url}");
        queue_offline_sync_payload(output, OfflineSyncPayload::Capabilities(req), &error)?;
        anyhow::bail!(error);
    }
    let chunk_size = capability_sync_chunk_size();
    let max_payload_bytes = capability_sync_max_payload_bytes();
    for chunk in capability_sync_request_chunks(&req, chunk_size, max_payload_bytes) {
        match tokio::time::timeout(capability_sync_timeout(), client.capabilities_sync(&chunk))
            .await
        {
            Ok(Ok(_)) => {}
            Ok(Err(error)) => {
                let error = format!("{error:#}");
                queue_offline_sync_payload(
                    output,
                    OfflineSyncPayload::Capabilities(chunk),
                    &error,
                )?;
                anyhow::bail!(error);
            }
            Err(error) => {
                let error = format!("capability sync timed out: {error}");
                queue_offline_sync_payload(
                    output,
                    OfflineSyncPayload::Capabilities(chunk),
                    &error,
                )?;
                anyhow::bail!(error);
            }
        }
    }
    Ok(())
}

fn capability_sync_timeout() -> Duration {
    std::env::var("MEMD_CAPABILITY_SYNC_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|seconds| *seconds >= 2)
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_secs(10))
}

fn capability_sync_chunk_size() -> usize {
    std::env::var("MEMD_CAPABILITY_SYNC_CHUNK_RECORDS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|records| *records >= 1)
        .unwrap_or(100)
}

fn capability_sync_max_payload_bytes() -> usize {
    std::env::var("MEMD_CAPABILITY_SYNC_MAX_PAYLOAD_BYTES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|bytes| *bytes >= 4096)
        .unwrap_or(512 * 1024)
}

fn capability_sync_request_chunks(
    req: &memd_schema::CapabilitySyncRequest,
    chunk_size: usize,
    max_payload_bytes: usize,
) -> Vec<memd_schema::CapabilitySyncRequest> {
    let chunk_size = chunk_size.max(1);
    let max_payload_bytes = max_payload_bytes.max(4096);
    if req.records.is_empty() {
        return Vec::new();
    }
    let mut chunks = Vec::new();
    let mut current = Vec::new();
    for record in &req.records {
        if current.len() >= chunk_size {
            chunks.push(capability_sync_chunk_request(
                req,
                std::mem::take(&mut current),
            ));
        }
        current.push(record.clone());
        if current.len() > 1 && capability_sync_payload_len(req, &current) > max_payload_bytes {
            let last = current.pop().expect("current chunk has last record");
            chunks.push(capability_sync_chunk_request(
                req,
                std::mem::take(&mut current),
            ));
            current.push(last);
        }
    }
    if !current.is_empty() {
        chunks.push(capability_sync_chunk_request(req, current));
    }
    chunks
}

fn capability_sync_chunk_request(
    req: &memd_schema::CapabilitySyncRequest,
    records: Vec<memd_schema::CapabilityRecord>,
) -> memd_schema::CapabilitySyncRequest {
    memd_schema::CapabilitySyncRequest {
        project: req.project.clone(),
        namespace: req.namespace.clone(),
        workspace: req.workspace.clone(),
        user_id: req.user_id.clone(),
        agent: req.agent.clone(),
        records,
    }
}

fn capability_sync_payload_len(
    req: &memd_schema::CapabilitySyncRequest,
    records: &[memd_schema::CapabilityRecord],
) -> usize {
    serde_json::to_vec(&capability_sync_chunk_request(req, records.to_vec()))
        .map(|bytes| bytes.len())
        .unwrap_or(usize::MAX)
}

#[cfg(test)]
mod capability_sync_chunk_tests {
    use super::*;

    fn record(name: &str) -> memd_schema::CapabilityRecord {
        memd_schema::CapabilityRecord {
            harness: "codex".to_string(),
            kind: "skill".to_string(),
            name: name.to_string(),
            status: "available".to_string(),
            portability_class: "harness-native".to_string(),
            source_path: format!("/tmp/{name}"),
            bridge_hint: None,
            hash: None,
            notes: Vec::new(),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            user_id: None,
            agent: Some("codex".to_string()),
            updated_at: None,
        }
    }

    #[test]
    fn capability_sync_request_chunks_keep_scope_and_bound_payload_size() {
        let req = memd_schema::CapabilitySyncRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            user_id: None,
            agent: Some("codex".to_string()),
            records: (0..205)
                .map(|index| record(&format!("cap-{index}")))
                .collect(),
        };

        let chunks = capability_sync_request_chunks(&req, 100, 512 * 1024);

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].records.len(), 100);
        assert_eq!(chunks[1].records.len(), 100);
        assert_eq!(chunks[2].records.len(), 5);
        assert!(chunks.iter().all(|chunk| chunk.project == req.project));
        assert!(chunks.iter().all(|chunk| chunk.namespace == req.namespace));
        assert!(chunks.iter().all(|chunk| chunk.workspace == req.workspace));
        assert!(chunks.iter().all(|chunk| chunk.agent == req.agent));
    }

    #[test]
    fn capability_sync_request_chunks_respect_payload_byte_limit() {
        let mut req = memd_schema::CapabilitySyncRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("shared".to_string()),
            user_id: None,
            agent: Some("codex".to_string()),
            records: (0..10)
                .map(|index| record(&format!("cap-{index}")))
                .collect(),
        };
        for record in &mut req.records {
            record.notes = vec!["x".repeat(2048)];
        }

        let chunks = capability_sync_request_chunks(&req, 100, 10 * 1024);

        assert!(chunks.len() > 1);
        assert_eq!(
            chunks
                .iter()
                .map(|chunk| chunk.records.len())
                .sum::<usize>(),
            req.records.len()
        );
        assert!(chunks.iter().all(|chunk| {
            serde_json::to_vec(chunk).expect("serialize chunk").len() <= 10 * 1024
        }));
    }

    #[test]
    fn capability_pull_record_limit_defaults_to_full_inventory() {
        let old_limit = std::env::var_os("MEMD_CAPABILITY_PULL_LIMIT");
        unsafe {
            std::env::remove_var("MEMD_CAPABILITY_PULL_LIMIT");
        }

        assert_eq!(capability_pull_record_limit(), 5_000);

        unsafe {
            match old_limit {
                Some(value) => std::env::set_var("MEMD_CAPABILITY_PULL_LIMIT", value),
                None => std::env::remove_var("MEMD_CAPABILITY_PULL_LIMIT"),
            }
        }
    }

    #[test]
    fn capability_pull_timeout_allows_full_live_inventory() {
        let old_timeout = std::env::var_os("MEMD_CAPABILITY_PULL_TIMEOUT_SECS");
        unsafe {
            std::env::remove_var("MEMD_CAPABILITY_PULL_TIMEOUT_SECS");
        }

        assert_eq!(capability_pull_timeout(), Duration::from_secs(30));

        unsafe {
            std::env::set_var("MEMD_CAPABILITY_PULL_TIMEOUT_SECS", "7");
        }
        assert_eq!(capability_pull_timeout(), Duration::from_secs(7));

        unsafe {
            match old_timeout {
                Some(value) => std::env::set_var("MEMD_CAPABILITY_PULL_TIMEOUT_SECS", value),
                None => std::env::remove_var("MEMD_CAPABILITY_PULL_TIMEOUT_SECS"),
            }
        }
    }
}

async fn pull_capabilities_from_server(
    client: &MemdClient,
    base_url: &str,
    args: &CapabilitiesArgs,
) -> anyhow::Result<CapabilitiesResponse> {
    let output = match &args.command {
        Some(CapabilitiesSubcommand::Pull(args)) => &args.output,
        _ => &args.output,
    };
    let config = read_memory_os_bundle_config(output).ok();
    let req = memd_schema::CapabilityListRequest {
        project: config.as_ref().and_then(|config| config.project.clone()),
        namespace: config.as_ref().and_then(|config| config.namespace.clone()),
        workspace: config.as_ref().and_then(|config| config.workspace.clone()),
        user_id: None,
        harness: args.harness.clone(),
        kind: args.kind.clone(),
        query: args.query.clone(),
        limit: Some(capability_pull_record_limit()),
    };
    if !base_url_reachable(base_url, Duration::from_millis(250)) {
        anyhow::bail!("server not reachable at {base_url}");
    }
    let pulled =
        match tokio::time::timeout(capability_pull_timeout(), client.capabilities_list(&req)).await
        {
            Ok(Ok(response)) => response,
            Ok(Err(error)) => anyhow::bail!("{error:#}"),
            Err(error) => anyhow::bail!("capability pull timed out: {error}"),
        };
    let project_root = infer_bundle_project_root(output);
    let local_registry = build_bundle_capability_registry(project_root.as_deref());
    let local_keys = local_registry
        .capabilities
        .iter()
        .map(capability_identity)
        .collect::<std::collections::BTreeSet<_>>();
    let mut registry = CapabilityRegistry {
        generated_at: Utc::now(),
        project_root: project_root.as_ref().map(|path| path.display().to_string()),
        capabilities: pulled
            .records
            .into_iter()
            .map(|record| localize_pulled_capability(record, &local_keys))
            .collect(),
    };
    annotate_capability_registry_host_cli_auth_notes(&mut registry);
    let bridges = detect_capability_bridges();
    write_bundle_capability_registry(output, &registry)?;
    write_bundle_capability_bridges(output, &bridges)?;
    build_capabilities_response_from_registry(
        args, output, &registry, &bridges, "pull", true, args.limit,
    )
}

fn capability_pull_record_limit() -> usize {
    std::env::var("MEMD_CAPABILITY_PULL_LIMIT")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| *value >= 100)
        .unwrap_or(5_000)
}

fn capability_pull_timeout() -> Duration {
    std::env::var("MEMD_CAPABILITY_PULL_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|seconds| *seconds >= 2)
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_secs(30))
}

fn capability_identity(record: &CapabilityRecord) -> String {
    format!("{}\0{}\0{}", record.harness, record.kind, record.name)
}

fn localize_pulled_capability(
    record: memd_schema::CapabilityRecord,
    local_keys: &std::collections::BTreeSet<String>,
) -> CapabilityRecord {
    let candidate = CapabilityRecord {
        harness: record.harness,
        kind: record.kind,
        name: record.name,
        status: record.status,
        portability_class: record.portability_class,
        source_path: record.source_path,
        bridge_hint: record.bridge_hint,
        hash: record.hash,
        notes: record.notes,
    };
    if local_keys.contains(&capability_identity(&candidate)) {
        return candidate;
    }
    let mut notes = candidate.notes.clone();
    if !notes.iter().any(|note| note == "synced_from_server") {
        notes.push("synced_from_server".to_string());
    }
    CapabilityRecord {
        status: "available-server".to_string(),
        notes,
        ..candidate
    }
}

async fn push_access_routes_to_server(
    client: &MemdClient,
    base_url: &str,
    output: &Path,
    response: &AccessReport,
) -> anyhow::Result<()> {
    if response.routes.is_empty() {
        return Ok(());
    }
    let config = read_memory_os_bundle_config(output).ok();
    let routes = response
        .routes
        .iter()
        .map(|route| memd_schema::AccessRouteRecord {
            id: route.id.clone(),
            provider: route.provider.clone(),
            status: route.status.clone(),
            scope: route.scope.clone(),
            secret_values_stored: route.secret_values_stored,
            guidance: route.guidance.clone(),
            source: route.source.clone(),
            project: config.as_ref().and_then(|config| config.project.clone()),
            namespace: config.as_ref().and_then(|config| config.namespace.clone()),
            workspace: config.as_ref().and_then(|config| config.workspace.clone()),
            user_id: None,
            agent: config.as_ref().and_then(|config| config.agent.clone()),
            updated_at: None,
        })
        .collect::<Vec<_>>();
    let req = memd_schema::AccessRouteSyncRequest {
        project: config.as_ref().and_then(|config| config.project.clone()),
        namespace: config.as_ref().and_then(|config| config.namespace.clone()),
        workspace: config.as_ref().and_then(|config| config.workspace.clone()),
        user_id: None,
        agent: config.as_ref().and_then(|config| config.agent.clone()),
        routes,
    };
    if !base_url_reachable(base_url, Duration::from_millis(250)) {
        let error = format!("server not reachable at {base_url}");
        queue_offline_sync_payload(output, OfflineSyncPayload::AccessRoutes(req), &error)?;
        anyhow::bail!(error);
    }
    match tokio::time::timeout(Duration::from_secs(2), client.access_routes_sync(&req)).await {
        Ok(Ok(_)) => return Ok(()),
        Ok(Err(error)) => {
            let error = format!("{error:#}");
            queue_offline_sync_payload(output, OfflineSyncPayload::AccessRoutes(req), &error)?;
            anyhow::bail!(error);
        }
        Err(error) => {
            let error = format!("access-route sync timed out: {error}");
            queue_offline_sync_payload(output, OfflineSyncPayload::AccessRoutes(req), &error)?;
            anyhow::bail!(error);
        }
    }
}

async fn push_token_savings_to_server(
    client: &MemdClient,
    base_url: &str,
    output: &Path,
) -> anyhow::Result<()> {
    let records = build_token_savings_sync_records(output)?;
    if records.is_empty() {
        return Ok(());
    }
    let config = read_memory_os_bundle_config(output).ok();
    let req = memd_schema::TokenSavingsSyncRequest {
        project: config.as_ref().and_then(|config| config.project.clone()),
        namespace: config.as_ref().and_then(|config| config.namespace.clone()),
        workspace: config.as_ref().and_then(|config| config.workspace.clone()),
        user_id: None,
        agent: config.as_ref().and_then(|config| config.agent.clone()),
        records,
    };
    if !base_url_reachable(base_url, Duration::from_millis(250)) {
        let error = format!("server not reachable at {base_url}");
        queue_offline_sync_payload(output, OfflineSyncPayload::TokenSavings(req), &error)?;
        anyhow::bail!(error);
    }
    match tokio::time::timeout(Duration::from_secs(2), client.token_savings_sync(&req)).await {
        Ok(Ok(_)) => return Ok(()),
        Ok(Err(error)) => {
            let error = format!("{error:#}");
            queue_offline_sync_payload(output, OfflineSyncPayload::TokenSavings(req), &error)?;
            anyhow::bail!(error);
        }
        Err(error) => {
            let error = format!("token-savings sync timed out: {error}");
            queue_offline_sync_payload(output, OfflineSyncPayload::TokenSavings(req), &error)?;
            anyhow::bail!(error);
        }
    }
}

fn tokens_report_scope(args: &TokensArgs) -> Option<(&Path, Option<&str>)> {
    match &args.command {
        TokensSubcommand::Saved(args) => Some((args.output.as_path(), args.since.as_deref())),
        TokensSubcommand::Sync(args) => Some((args.output.as_path(), args.since.as_deref())),
    }
}

async fn fetch_server_token_savings(
    client: &MemdClient,
    output: &Path,
    since: Option<&str>,
) -> anyhow::Result<memd_schema::TokenSavingsListResponse> {
    let config = read_memory_os_bundle_config(output).ok();
    let since = since
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.with_timezone(&chrono::Utc));
    let req = memd_schema::TokenSavingsListRequest {
        project: config.as_ref().and_then(|config| config.project.clone()),
        namespace: config.as_ref().and_then(|config| config.namespace.clone()),
        workspace: config.as_ref().and_then(|config| config.workspace.clone()),
        user_id: None,
        agent: None,
        since,
        limit: Some(1000),
    };
    tokio::time::timeout(Duration::from_secs(2), client.token_savings_list(&req))
        .await
        .context("token-savings list timed out")?
}

fn base_url_reachable(base_url: &str, timeout: Duration) -> bool {
    let Some((host, port)) = parse_base_url_host_port(base_url) else {
        return true;
    };
    let Ok(addrs) = (host.as_str(), port).to_socket_addrs() else {
        return false;
    };
    addrs
        .into_iter()
        .any(|addr| std::net::TcpStream::connect_timeout(&addr, timeout).is_ok())
}

fn parse_base_url_host_port(base_url: &str) -> Option<(String, u16)> {
    let trimmed = base_url.trim();
    let (scheme, rest) = trimmed.split_once("://")?;
    let authority = rest.split('/').next().unwrap_or(rest);
    if authority.starts_with('[') {
        return None;
    }
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (
            host.to_string(),
            port.parse::<u16>()
                .ok()
                .unwrap_or_else(|| default_port(scheme)),
        ),
        None => (authority.to_string(), default_port(scheme)),
    };
    if host.trim().is_empty() {
        None
    } else {
        Some((host, port))
    }
}

fn default_port(scheme: &str) -> u16 {
    if scheme.eq_ignore_ascii_case("https") {
        443
    } else {
        80
    }
}

async fn run_diagnostics_command(args: DiagnosticsArgs) -> anyhow::Result<()> {
    match args.command {
        DiagnosticsCommand::Report(report_args) => {
            run_diagnostics_report(&args.base_url, &report_args).await
        }
        DiagnosticsCommand::TokenEfficiency(te_args) => {
            run_diagnostics_token_efficiency(&args.base_url, &te_args).await
        }
        DiagnosticsCommand::LifecycleProbe(probe_args) => {
            run_diagnostics_lifecycle_probe(&args.base_url, &probe_args).await
        }
    }
}

async fn run_diagnostics_lifecycle_probe(
    base_url: &str,
    args: &DiagnosticsLifecycleProbeArgs,
) -> anyhow::Result<()> {
    let client = MemdClient::new(base_url)?;
    let report = run_lifecycle_probe(&client).await;
    if args.summary {
        println!(
            "lifecycle-probe {} probe_id={} steps={}",
            report.status,
            report.probe_id,
            report.steps.len()
        );
        for step in &report.steps {
            let mark = if step.ok { "ok" } else { "FAIL" };
            let detail = step.detail.as_deref().unwrap_or("");
            println!("  - {mark} {} {detail}", step.name);
        }
    } else {
        print_json(&report)?;
    }
    if report.is_green() {
        Ok(())
    } else {
        anyhow::bail!("lifecycle probe red: {:?}", report.steps);
    }
}

async fn run_diagnostics_token_efficiency(
    base_url: &str,
    args: &DiagnosticsTokenEfficiencyArgs,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let req = serde_json::json!({
        "project": args.project.as_deref().unwrap_or("default"),
        "namespace": args.namespace.as_deref().unwrap_or("main"),
        "agent": args.agent,
        "route": "auto",
        "intent": "current_task",
    });

    let resp = client
        .post(format!("{base_url}/api/diagnostics/token-efficiency"))
        .json(&req)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("token-efficiency endpoint returned {status}: {body}");
    }

    let report: memd_schema::OperationTokenReport = resp.json().await?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("## Token Efficiency: {}", report.operation);
        println!();
        println!(
            "Budget: {} chars | Used: {} chars | Utilization: {:.1}%",
            report.budget_chars, report.used_chars, report.utilization_pct
        );
        println!();
        println!("### Per-Kind Breakdown");
        println!();
        println!("{:<20} {:>8} {:>8}", "Kind", "Items", "Chars");
        println!("{}", "-".repeat(40));
        for (kind, items) in &report.per_kind.items_per_kind {
            let chars = report.per_kind.chars_per_kind.get(kind).unwrap_or(&0);
            println!("{:<20} {:>8} {:>8}", kind, items, chars);
        }
        println!("{}", "-".repeat(40));
        println!(
            "{:<20} {:>8} {:>8}",
            "TOTAL", report.per_kind.total_items, report.per_kind.total_chars
        );
    }

    Ok(())
}

async fn run_diagnostics_report(
    base_url: &str,
    args: &DiagnosticsReportArgs,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    // 1. Fetch token efficiency (working memory)
    let te_req = serde_json::json!({
        "project": args.project.as_deref().unwrap_or("default"),
        "namespace": args.namespace.as_deref().unwrap_or("main"),
        "agent": args.agent,
        "route": "auto",
        "intent": "current_task",
    });

    let te_resp = client
        .post(format!("{base_url}/api/diagnostics/token-efficiency"))
        .json(&te_req)
        .send()
        .await;

    let token_report: Option<memd_schema::OperationTokenReport> = match te_resp {
        Ok(r) if r.status().is_success() => r.json().await.ok(),
        _ => None,
    };

    // 2. Fetch decay diagnostics
    let decay_req = serde_json::json!({
        "project": args.project.as_deref().unwrap_or("default"),
        "namespace": args.namespace.as_deref().unwrap_or("main"),
    });

    let decay_resp = client
        .post(format!("{base_url}/api/diagnostics/decay"))
        .json(&decay_req)
        .send()
        .await;

    let decay_report: Option<serde_json::Value> = match decay_resp {
        Ok(r) if r.status().is_success() => r.json().await.ok(),
        _ => None,
    };

    // 3. Fetch health for compaction/consolidation info
    let health_resp = client.get(format!("{base_url}/healthz")).send().await;

    let health: Option<serde_json::Value> = match health_resp {
        Ok(r) if r.status().is_success() => r.json().await.ok(),
        _ => None,
    };

    // P2: Read cached wake token metrics from bundle output if available.
    let wake_token_report: Option<memd_schema::OperationTokenReport> = args
        .output
        .as_ref()
        .and_then(|dir| std::fs::read_to_string(dir.join("wake-token-metrics.json")).ok())
        .and_then(|json| serde_json::from_str(&json).ok());

    if args.json {
        let combined = serde_json::json!({
            "token_efficiency_working_memory": token_report,
            "token_efficiency_wake": wake_token_report,
            "decay": decay_report,
            "health": health,
            "timestamp": chrono::Utc::now().timestamp(),
        });
        println!("{}", serde_json::to_string_pretty(&combined)?);
    } else {
        println!("# memd Diagnostics Report");
        println!();
        println!(
            "Timestamp: {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!();

        // Token efficiency section
        println!("## 1. Token Efficiency");

        // 1a. Working memory operation (from server)
        if let Some(ref te) = token_report {
            println!(
                "- Operation: {} | Budget: {} | Used: {} | Utilization: {:.1}%",
                te.operation, te.budget_chars, te.used_chars, te.utilization_pct
            );
            for (kind, items) in &te.per_kind.items_per_kind {
                let chars = te.per_kind.chars_per_kind.get(kind).unwrap_or(&0);
                println!("  - {}: {} items, {} chars", kind, items, chars);
            }
        } else {
            println!("- working_memory: N/A (server unreachable or no data)");
        }

        // 1b. Wake operation (from cached bundle metrics)
        if let Some(ref wte) = wake_token_report {
            println!(
                "- Operation: {} | Budget: {} | Used: {} | Utilization: {:.1}%",
                wte.operation, wte.budget_chars, wte.used_chars, wte.utilization_pct
            );
            for (kind, items) in &wte.per_kind.items_per_kind {
                let chars = wte.per_kind.chars_per_kind.get(kind).unwrap_or(&0);
                println!("  - {}: {} items, {} chars", kind, items, chars);
            }
        } else {
            println!("- wake: N/A (run `memd wake --write` first to generate metrics)");
        }
        println!();

        // Decay section
        println!("## 2. Decay Diagnostics");
        if let Some(ref decay) = decay_report {
            if let Some(metrics) = decay.get("metrics") {
                println!(
                    "- Items inspected: {}",
                    metrics
                        .get("items_inspected")
                        .unwrap_or(&serde_json::Value::Null)
                );
                println!(
                    "- Items decayed: {}",
                    metrics
                        .get("items_decayed")
                        .unwrap_or(&serde_json::Value::Null)
                );
                println!(
                    "- Items expired: {}",
                    metrics
                        .get("items_expired")
                        .unwrap_or(&serde_json::Value::Null)
                );
            }
            println!(
                "- Inactive days: {}",
                decay
                    .get("inactive_days")
                    .unwrap_or(&serde_json::Value::Null)
            );
            println!(
                "- Max decay: {}",
                decay.get("max_decay").unwrap_or(&serde_json::Value::Null)
            );
            println!(
                "- Decay divisor: {}",
                decay
                    .get("decay_divisor")
                    .unwrap_or(&serde_json::Value::Null)
            );
        } else {
            println!("- N/A (server unreachable or no data)");
        }
        println!();

        // Health section
        println!("## 3. System Health");
        if let Some(ref h) = health {
            println!(
                "- Status: {}",
                h.get("status").unwrap_or(&serde_json::Value::Null)
            );
            if let Some(items) = h.get("item_count") {
                println!("- Total items: {}", items);
            }
            if let Some(entities) = h.get("entity_count") {
                println!("- Total entities: {}", entities);
            }
        } else {
            println!("- N/A (server unreachable)");
        }
        println!();

        // Measurement dimensions summary
        println!("## 4. Measurement Completeness");
        let te_wm_ok = token_report.is_some();
        let te_wake_ok = wake_token_report.is_some();
        let decay_ok = decay_report.is_some();
        let health_ok = health.is_some();
        println!(
            "- [{}] Token efficiency: working_memory (per-kind, server)",
            if te_wm_ok { "✓" } else { " " }
        );
        println!(
            "- [{}] Token efficiency: wake (per-kind, bundle)",
            if te_wake_ok { "✓" } else { " " }
        );
        println!(
            "- [{}] Decay diagnostics (calibrated parameters)",
            if decay_ok { "✓" } else { " " }
        );
        println!(
            "- [{}] System health (item/entity counts)",
            if health_ok { "✓" } else { " " }
        );
        println!("- [✓] Compaction quality (per working memory build — CompactionQualityReport)");
        println!(
            "- [✓] Handoff quality (per resume — HandoffQualityScore from CompactionQualityReport)"
        );
        println!("- [ ] Benchmark results (run `memd benchmark public --ci --record`)");
    }

    Ok(())
}
