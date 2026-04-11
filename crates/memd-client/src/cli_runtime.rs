use super::*;

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
            invalidate_bundle_runtime_caches(&args.output)?;
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
            let snapshot = match read_bundle_resume(&resume_args, &base_url).await {
                Ok(snapshot) => snapshot,
                Err(err)
                    if codex_pack
                        || agent_zero_pack
                        || hermes_pack
                        || opencode_pack
                        || openclaw_pack =>
                {
                    if let Some(markdown) =
                        read_codex_pack_local_markdown(&args.output, "MEMD_WAKEUP.md")?
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
                auto_checkpoint_live_snapshot(&args.output, &base_url, &snapshot, "wake").await?;
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
        }
        Commands::Awareness(args) => {
            let response = read_project_awareness(&args).await?;
            if args.summary {
                println!("{}", render_project_awareness_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
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
            let response = eval_bundle_memory(&args, &base_url).await?;
            if args.write {
                write_bundle_eval_artifacts(&args.output, &response)?;
            }
            if args.summary {
                println!("{}", render_eval_summary(&response));
            } else {
                print_json(&response)?;
            }
            if let Some(reason) =
                eval_failure_reason(&response, args.fail_below, args.fail_on_regression)
            {
                anyhow::bail!(reason);
            }
        }
        Commands::Gap(args) => {
            let response = gap_report(&args).await?;
            if args.write {
                write_gap_artifacts(&args.output, &response)?;
            }
            if args.summary {
                println!("{}", render_gap_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Improve(args) => {
            let response = run_improvement_loop(&args, &base_url).await?;
            if args.write {
                write_improvement_artifacts(&args.output, &response)?;
            }
            if args.summary {
                println!("{}", render_improvement_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Scenario(args) => {
            let response = run_scenario_command(&args, &base_url).await?;
            if args.write {
                write_scenario_artifacts(&args.output, &response)?;
            }
            if args.summary {
                println!("{}", render_scenario_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Composite(args) => {
            let response = run_composite_command(&args, &base_url).await?;
            if args.write {
                write_composite_artifacts(&args.output, &response)?;
            }
            if args.summary {
                println!("{}", render_composite_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Benchmark(args) => match &args.subcommand {
            Some(BenchmarkSubcommand::Public(public_args)) => {
                let response = run_public_benchmark_command(public_args).await?;
                if public_args.write {
                    let receipt =
                        write_public_benchmark_run_artifacts(&public_args.out, &response)?;
                    let _ = (
                        &receipt.run_dir,
                        &receipt.manifest_path,
                        &receipt.results_path,
                        &receipt.results_jsonl_path,
                        &receipt.report_path,
                    );
                    if let Some(repo_root) = infer_bundle_project_root(&public_args.out) {
                        write_public_benchmark_docs(&repo_root, &public_args.out, &response)?;
                    }
                }
                if public_args.json {
                    print_json(&response)?;
                } else if args.summary {
                    println!("{}", render_public_benchmark_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
            None => {
                let response = run_feature_benchmark_command(&args, &base_url).await?;
                if args.write {
                    write_feature_benchmark_artifacts(&args.output, &response)?;
                    if let Some((repo_root, registry)) =
                        load_benchmark_registry_for_output(&args.output)?
                    {
                        write_benchmark_registry_docs(&repo_root, &registry, &response)?;
                    }
                }
                if args.summary {
                    println!("{}", render_feature_benchmark_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
        },
        Commands::Verify(args) => match &args.command {
            VerifyCommand::Feature(verify_args) => {
                let response = run_verify_feature_command(verify_args).await?;
                if verify_args.summary {
                    println!("{}", render_verify_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
            VerifyCommand::Journey(verify_args) => {
                let response = run_verify_journey_command(verify_args).await?;
                if verify_args.summary {
                    println!("{}", render_verify_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
            VerifyCommand::Adversarial(verify_args) => {
                let response = run_verify_adversarial_command(verify_args).await?;
                if verify_args.summary {
                    println!("{}", render_verify_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
            VerifyCommand::Compare(verify_args) => {
                let response = run_verify_compare_command(verify_args).await?;
                if verify_args.summary {
                    println!("{}", render_verify_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
            VerifyCommand::Sweep(verify_args) => {
                let response = run_verify_sweep_command(verify_args).await?;
                if verify_args.summary {
                    println!("{}", render_verify_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
            VerifyCommand::Doctor(verify_args) => {
                let response = run_verify_doctor_command(verify_args)?;
                if verify_args.summary {
                    println!("{}", render_verify_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
            VerifyCommand::List(verify_args) => {
                let response = run_verify_list_command(verify_args)?;
                if verify_args.summary {
                    println!("{}", render_verify_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
            VerifyCommand::Show(verify_args) => {
                let response = run_verify_show_command(verify_args)?;
                if verify_args.summary {
                    println!("{}", render_verify_summary(&response));
                } else {
                    print_json(&response)?;
                }
            }
        },
        Commands::Experiment(args) => {
            let mut response = run_experiment_command(&args, &base_url).await?;
            if args.write {
                write_experiment_artifacts(&args.output, &response)?;
                hydrate_experiment_evolution_summary(&mut response, &args.output)?;
            }
            if args.summary {
                println!("{}", render_experiment_summary(&response));
            } else {
                print_json(&response)?;
            }
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
                let snapshot = read_bundle_resume(
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
            let response = bundle_agent_profiles::build_bundle_agent_profiles(
                &args.output,
                args.name.as_deref(),
                args.shell.as_deref(),
            )?;
            if args.summary {
                println!(
                    "{}",
                    bundle_agent_profiles::render_bundle_agent_profiles_summary(&response)
                );
            } else {
                print_json(&response)?;
            }
        }
        Commands::Attach(args) => {
            let shell = args
                .shell
                .or_else(bundle_agent_profiles::detect_shell)
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
            let snapshot = match read_bundle_resume(&args, &base_url).await {
                Ok(snapshot) => snapshot,
                Err(err)
                    if codex_pack
                        || agent_zero_pack
                        || hermes_pack
                        || opencode_pack
                        || openclaw_pack =>
                {
                    if let Some(markdown) =
                        read_codex_pack_local_markdown(&args.output, "MEMD_MEMORY.md")?
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
            invalidate_bundle_runtime_caches(&args.output)?;
            let snapshot = read_bundle_resume(&args, &base_url).await?;
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
            let snapshot = read_bundle_handoff(&args, &base_url).await?;
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
            let snapshot = read_bundle_resume(
                &ResumeArgs {
                    output: args.output.clone(),
                    project: args.project.clone(),
                    namespace: args.namespace.clone(),
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
            let response = remember_with_bundle_defaults(&args, &base_url).await?;
            let snapshot = read_bundle_resume(
                &ResumeArgs {
                    output: args.output.clone(),
                    project: args.project.clone(),
                    namespace: args.namespace.clone(),
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
                    let summary = sync_to_rag(&client, &rag, args).await?;
                    print_json(&summary)?;
                }
            }
        }
        Commands::Multimodal(args) => {
            let rag_url = resolve_rag_url(args.rag_url, resolve_default_bundle_root()?.as_deref())?;
            let sidecar = SidecarClient::new(&rag_url)?;
            match args.mode {
                MultimodalMode::Healthz => print_json(&sidecar.healthz().await?)?,
                MultimodalMode::Plan(args) => {
                    let preview =
                        build_multimodal_preview(args.project, args.namespace, &args.path)?;
                    print_json(&preview)?;
                }
                MultimodalMode::Ingest(args) => {
                    let preview =
                        build_multimodal_preview(args.project, args.namespace, &args.path)?;
                    if args.apply {
                        let responses =
                            ingest_multimodal_preview(&sidecar, &preview.requests).await?;
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
        }
        Commands::Inspiration(args) => {
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
        }
        Commands::Skills(args) => {
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
        }
        Commands::Packs(args) => {
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
            let index =
                crate::harness::index::filter_harness_pack_index(index, args.query.as_deref());
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
        }
        Commands::Commands(args) => {
            let bundle_root = resolve_pack_bundle_root(args.root.as_deref())?;
            let catalog = command_catalog::build_command_catalog(&bundle_root);
            let catalog = command_catalog::filter_command_catalog(catalog, args.query.as_deref());
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
        }
        Commands::Setup(args) => {
            let decision = resolve_bootstrap_authority(setup_args_to_init_args(&args)).await?;
            let init_args = decision.init_args;
            write_init_bundle(&init_args)?;
            if decision.fallback_activated {
                set_bundle_localhost_read_only_authority_state(
                    &init_args.output,
                    &decision.shared_base_url,
                    "setup",
                    "shared authority unavailable during bootstrap",
                )?;
                write_agent_profiles(&init_args.output)?;
                write_native_agent_bridge_files(&init_args.output)?;
            }
            if args.json {
                print_json(&json!({
                    "bundle": init_args.output,
                    "project": init_args.project,
                    "namespace": init_args.namespace,
                    "agent": init_args.agent,
                    "base_url": init_args.base_url,
                    "shared_base_url": decision.shared_base_url,
                    "authority_mode": if decision.fallback_activated { "localhost_read_only" } else { "shared" },
                    "route": init_args.route,
                    "intent": init_args.intent,
                    "voice_mode": init_args.voice_mode,
                    "workspace": init_args.workspace,
                    "visibility": init_args.visibility,
                    "setup_ready": true,
                }))?;
            } else if args.summary {
                println!(
                    "setup bundle={} project={} namespace={} agent={} voice={} authority={} ready=true",
                    init_args.output.display(),
                    init_args.project.as_deref().unwrap_or("none"),
                    init_args.namespace.as_deref().unwrap_or("none"),
                    init_args.agent,
                    init_args.voice_mode.as_deref().unwrap_or("caveman-ultra"),
                    if decision.fallback_activated {
                        "localhost_read_only"
                    } else {
                        "shared"
                    },
                );
            } else {
                println!("Initialized memd bundle at {}", init_args.output.display());
                if decision.fallback_activated {
                    eprintln!("memd authority warning:");
                    eprintln!("- shared authority unavailable");
                    eprintln!("- localhost fallback is lower trust");
                    eprintln!("- prompt-injection and split-brain risk increased");
                    eprintln!("- coordination writes blocked");
                }
            }
        }
        Commands::Doctor(args) => {
            let bundle_root = resolve_doctor_bundle_root(args.output.as_deref())?;
            let mut status = read_bundle_status(&bundle_root, &base_url).await?;
            let setup_ready = status
                .get("setup_ready")
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            if args.repair && !setup_ready {
                let project_root = args.project_root.clone().or(detect_current_project_root()?);
                let setup_args =
                    doctor_args_to_setup_args(&args, bundle_root.clone(), project_root);
                let decision =
                    resolve_bootstrap_authority(setup_args_to_init_args(&setup_args)).await?;
                write_init_bundle(&decision.init_args)?;
                if decision.fallback_activated {
                    set_bundle_localhost_read_only_authority_state(
                        &decision.init_args.output,
                        &decision.shared_base_url,
                        "doctor",
                        "shared authority unavailable during repair bootstrap",
                    )?;
                    write_agent_profiles(&decision.init_args.output)?;
                    write_native_agent_bridge_files(&decision.init_args.output)?;
                }
                status = read_bundle_status(&bundle_root, &base_url).await?;
            } else if args.repair {
                let repaired_worker_env = repair_bundle_worker_name_env(&bundle_root)?;
                if repaired_worker_env {
                    write_agent_profiles(&bundle_root)?;
                }
                status = read_bundle_status(&bundle_root, &base_url).await?;
            }
            if args.json {
                print_json(&status)?;
            } else if args.summary {
                println!("{}", render_bundle_status_summary(&status));
            } else {
                println!("{}", render_doctor_status_markdown(&bundle_root, &status));
            }
        }
        Commands::Config(args) => {
            let bundle_root = resolve_setup_bundle_root(args.output.as_deref())?;
            let project_root = args.project_root.clone().or(detect_current_project_root()?);
            let runtime = read_bundle_runtime_config(&bundle_root)?;
            let status = read_bundle_status(&bundle_root, &base_url).await.ok();
            let config = render_bundle_config_snapshot(
                &bundle_root,
                project_root.as_deref(),
                runtime.as_ref(),
                status.as_ref(),
            );
            if args.json {
                print_json(&config)?;
            } else if args.summary {
                println!("{}", render_bundle_config_summary(&config));
            } else {
                println!("{}", render_bundle_config_markdown(&config));
            }
        }
        Commands::Memory(args) => {
            let bundle_root = resolve_compiled_memory_bundle_root(args.root.as_deref())?;
            let use_runtime_summary = !args.quality
                && !args.list
                && compiled_memory_target(&args).is_none()
                && args.query.is_none();
            if use_runtime_summary {
                match read_memory_surface(&bundle_root, &base_url).await {
                    Ok(response) if args.json => print_json(&response)?,
                    Ok(response) => println!("{}", render_memory_surface_summary(&response)),
                    Err(_) if !args.json => {
                        let page =
                            bundle_compiled_memory_path(&bundle_root, MemoryObjectLane::Working);
                        let content = fs::read_to_string(&page)
                            .with_context(|| format!("read {}", page.display()))?;
                        println!("{}", render_compiled_memory_page_summary(&page, &content));
                    }
                    Err(err) => return Err(err),
                }
            } else if args.quality {
                let report = build_compiled_memory_quality_report(&bundle_root)?;
                if args.json {
                    print_json(&render_compiled_memory_quality_json(&bundle_root, &report))?;
                } else if args.summary {
                    println!(
                        "{}",
                        render_compiled_memory_quality_summary(&bundle_root, &report)
                    );
                } else {
                    println!(
                        "{}",
                        render_compiled_memory_quality_markdown(&bundle_root, &report)
                    );
                }
            } else if args.list {
                let index = render_compiled_memory_index(&bundle_root)?;
                let index = filter_compiled_memory_index(
                    index,
                    args.lanes_only,
                    args.items_only,
                    args.filter.as_deref(),
                );
                if args.json {
                    print_json(&render_compiled_memory_index_json(&bundle_root, &index))?;
                } else if args.summary {
                    println!(
                        "{}",
                        render_compiled_memory_index_summary(&bundle_root, &index)
                    );
                } else if args.grouped {
                    println!(
                        "{}",
                        render_compiled_memory_index_grouped_markdown(
                            &bundle_root,
                            &index,
                            args.expand_items
                        )
                    );
                } else {
                    println!(
                        "{}",
                        render_compiled_memory_index_markdown(&bundle_root, &index)
                    );
                }
            } else if let Some(target) = compiled_memory_target(&args) {
                let path = resolve_compiled_memory_page(&bundle_root, target)?;
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("read {}", path.display()))?;
                if args.summary {
                    println!("{}", render_compiled_memory_page_summary(&path, &content));
                } else {
                    println!("{}", render_compiled_memory_page_markdown(&path, &content));
                }
            } else if let Some(query) = args.query.as_deref() {
                let matches = search_compiled_memory_pages(&bundle_root, query, args.limit)?;
                if args.summary {
                    println!(
                        "{}",
                        render_compiled_memory_search_summary(&bundle_root, query, &matches)
                    );
                } else {
                    println!(
                        "{}",
                        render_compiled_memory_search_markdown(&bundle_root, query, &matches)
                    );
                }
            } else {
                let page = bundle_compiled_memory_path(&bundle_root, MemoryObjectLane::Working);
                let content = fs::read_to_string(&page)
                    .with_context(|| format!("read {}", page.display()))?;
                if args.summary {
                    println!("{}", render_compiled_memory_page_summary(&page, &content));
                } else {
                    println!("{}", render_compiled_memory_page_markdown(&page, &content));
                }
            }
        }
        Commands::Ingest(args) => {
            let result = ingest_auto_route(&client, &args).await?;
            print_json(&result)?;
        }
        Commands::Store(input) => {
            let req = read_request::<StoreMemoryRequest>(&input)?;
            print_json(&client.store(&req).await?)?;
        }
        Commands::Candidate(input) => {
            let req = read_request::<CandidateMemoryRequest>(&input)?;
            print_json(&client.candidate(&req).await?)?;
        }
        Commands::Promote(input) => {
            let req = read_request::<PromoteMemoryRequest>(&input)?;
            print_json(&client.promote(&req).await?)?;
        }
        Commands::Expire(input) => {
            let req = read_request::<ExpireMemoryRequest>(&input)?;
            print_json(&client.expire(&req).await?)?;
        }
        Commands::MemoryVerify(input) => {
            let req = read_request::<VerifyMemoryRequest>(&input)?;
            print_json(&client.verify(&req).await?)?;
        }
        Commands::Repair(args) => {
            let mode = commands::parse_memory_repair_mode_value(&args.mode)?;
            let status = match args.status.as_deref() {
                Some(value) => Some(commands::parse_memory_status_value(value)?),
                None => None,
            };
            let source_quality = match args.source_quality.as_deref() {
                Some(value) => Some(parse_source_quality_value(value)?),
                None => None,
            };
            let supersedes = parse_uuid_list(&args.supersede)?;
            let response = client
                .repair(&RepairMemoryRequest {
                    id: args.id.parse()?,
                    mode,
                    confidence: args.confidence,
                    status,
                    workspace: args.workspace.clone(),
                    visibility: args
                        .visibility
                        .as_deref()
                        .map(parse_memory_visibility_value)
                        .transpose()?,
                    source_agent: args.source_agent.clone(),
                    source_system: args.source_system.clone(),
                    source_path: args.source_path.clone(),
                    source_quality,
                    content: args.content.clone(),
                    tags: if args.tag.is_empty() {
                        None
                    } else {
                        Some(args.tag.clone())
                    },
                    supersedes,
                })
                .await?;
            if args.summary {
                println!("{}", render_repair_summary(&response, args.follow));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Search(args) => {
            let mut req = read_request::<SearchMemoryRequest>(&args.input)?;
            if args.route.is_some() || args.intent.is_some() {
                req.route = parse_retrieval_route(args.route)?;
                req.intent = parse_retrieval_intent(args.intent)?;
            }
            if args.belief_branch.is_some() {
                req.belief_branch = args.belief_branch.clone();
            }
            if args.workspace.is_some() {
                req.workspace = args.workspace.clone();
            }
            if let Some(visibility) = args.visibility.as_deref() {
                req.visibility = Some(parse_memory_visibility_value(visibility)?);
            }
            print_json(&client.search(&req).await?)?;
        }
        Commands::Lookup(args) => {
            let runtime = read_bundle_runtime_config(&args.output)?;
            let req = build_lookup_request(&args, runtime.as_ref())?;
            let response = lookup_with_fallbacks(&client, &req, &args.query).await?;
            if args.json {
                print_json(&response)?;
            } else {
                println!(
                    "{}",
                    render_lookup_markdown(&args.query, &response, args.verbose)
                );
            }
        }
        Commands::Context(args) => {
            let req = if args.json.is_some() || args.input.is_some() || args.stdin {
                read_request::<ContextRequest>(&RequestInput {
                    json: args.json.clone(),
                    input: args.input.clone(),
                    stdin: args.stdin,
                })?
            } else {
                ContextRequest {
                    project: args.project.clone(),
                    agent: args.agent.clone(),
                    workspace: args.workspace.clone(),
                    visibility: args
                        .visibility
                        .as_deref()
                        .map(parse_memory_visibility_value)
                        .transpose()?,
                    route: parse_retrieval_route(args.route.clone())?,
                    intent: parse_retrieval_intent(args.intent.clone())?,
                    limit: args.limit,
                    max_chars_per_item: args.max_chars_per_item,
                }
            };

            if args.compact {
                print_json(&client.context_compact(&req).await?)?;
            } else {
                print_json(&client.context(&req).await?)?;
            }
        }
        Commands::Working(args) => {
            let response = client
                .working(&WorkingMemoryRequest {
                    project: args.project.clone(),
                    agent: args.agent.clone(),
                    workspace: args.workspace.clone(),
                    visibility: args
                        .visibility
                        .as_deref()
                        .map(parse_memory_visibility_value)
                        .transpose()?,
                    route: parse_retrieval_route(args.route.clone())?,
                    intent: parse_retrieval_intent(args.intent.clone())?,
                    limit: args.limit,
                    max_chars_per_item: args.max_chars_per_item,
                    max_total_chars: args.max_total_chars,
                    rehydration_limit: args.rehydration_limit,
                    auto_consolidate: Some(args.auto_consolidate),
                })
                .await?;
            if args.summary {
                println!("{}", render_working_summary(&response, args.follow));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Profile(args) => {
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
                        project: args.project.clone(),
                        namespace: args.namespace.clone(),
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
                        project: args.project.clone(),
                        namespace: args.namespace.clone(),
                    })
                    .await?;
                if args.summary {
                    println!("{}", render_profile_summary(&response, args.follow));
                } else {
                    print_json(&response)?;
                }
            }
        }
        Commands::Source(args) => {
            let response = client
                .source_memory(&SourceMemoryRequest {
                    project: args.project.clone(),
                    namespace: args.namespace.clone(),
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
        }
        Commands::Workspaces(args) => {
            let response = client
                .workspace_memory(&memd_schema::WorkspaceMemoryRequest {
                    project: args.project.clone(),
                    namespace: args.namespace.clone(),
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
        }
        Commands::Inbox(args) => {
            let req = MemoryInboxRequest {
                project: args.project.clone(),
                namespace: args.namespace.clone(),
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
        }
        Commands::Explain(args) => {
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
        }
        Commands::Entity(args) => {
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
        }
        Commands::EntitySearch(args) => {
            let response = client
                .entity_search(&EntitySearchRequest {
                    query: args.query.clone(),
                    project: args.project.clone(),
                    namespace: args.namespace.clone(),
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
        }
        Commands::EntityLink(args) => {
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
        }
        Commands::EntityLinks(args) => {
            let response = client
                .entity_links(&EntityLinksRequest {
                    entity_id: args.entity_id.parse().context("parse entity_id as uuid")?,
                })
                .await?;
            print_json(&response)?;
        }
        Commands::Recall(args) => {
            let req = resolve_recall_request(&client, &args).await?;
            let response = client.associative_recall(&req).await?;
            if args.summary {
                println!("{}", render_recall_summary(&response, args.follow));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Timeline(args) => {
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
        }
        Commands::Events(args) => {
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
                let content = fs::read_to_string(&path)
                    .with_context(|| format!("read {}", path.display()))?;
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
        }
        Commands::Consolidate(args) => {
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
        }
        Commands::MaintenanceReport(args) => {
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
        }
        Commands::Maintain(args) => {
            let response = run_maintain_command(&args, &cli.base_url).await?;
            if args.summary {
                println!("{}", render_maintain_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        Commands::Policy(args) => {
            let response = client.policy().await?;
            if args.summary {
                println!("{}", render_policy_summary(&response, args.follow));
            } else {
                print_json(&response)?;
            }
        }
        Commands::SkillPolicy(args) => {
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
                let receipt =
                    write_skill_policy_artifacts(&args.output, &response, &report, args.apply)?;
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
        }
        Commands::Compact(args) => {
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
                            .map(|(kind, value)| {
                                (kind.trim().to_string(), value.trim().to_string())
                            })
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
                    auto_checkpoint_compaction_packet(&packet, &base_url).await?;
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
        }
        Commands::Obsidian(args) => match args.mode {
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
                run_obsidian_import(&client, &args, false, false).await?;
            }
            ObsidianMode::Sync => {
                run_obsidian_import(&client, &args, true, false).await?;
            }
            ObsidianMode::Compile => {
                run_obsidian_compile(&client, &args).await?;
            }
            ObsidianMode::Handoff => {
                run_obsidian_handoff(&args, &base_url).await?;
            }
            ObsidianMode::Writeback => {
                run_obsidian_writeback(&client, &args).await?;
            }
            ObsidianMode::Open => {
                run_obsidian_open(&client, &args).await?;
            }
            ObsidianMode::Roundtrip => {
                run_obsidian_import(&client, &args, true, true).await?;
            }
            ObsidianMode::Watch => {
                run_obsidian_watch(&client, &args).await?;
            }
            ObsidianMode::Status => {
                run_obsidian_status(&client, &args).await?;
            }
        },
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
        Commands::Hook(args) => match args.mode {
            HookMode::Context(args) => {
                let req = ContextRequest {
                    project: args.project,
                    agent: args.agent,
                    workspace: None,
                    visibility: None,
                    route: parse_retrieval_route(args.route)?,
                    intent: parse_retrieval_intent(
                        args.intent.or(Some("current_task".to_string())),
                    )?,
                    limit: args.limit,
                    max_chars_per_item: args.max_chars_per_item,
                };
                print_json(&client.context_compact(&req).await?)?;
            }
            HookMode::Capture(args) => {
                let content = if let Some(content) = &args.content {
                    content.clone()
                } else if let Some(path) = &args.input {
                    fs::read_to_string(path).with_context(|| {
                        format!("read hook capture input file {}", path.display())
                    })?
                } else if args.stdin {
                    let mut content = String::new();
                    io::stdin()
                        .read_to_string(&mut content)
                        .context("read hook capture payload from stdin")?;
                    content
                } else {
                    "hook capture: active task state changed".to_string()
                };
                let effective_promote_kind = effective_hook_capture_promote_kind(&args, &content);
                let (supersede_targets, supersede_diagnostics) =
                    find_hook_capture_supersede_targets(&base_url, &args, &content).await?;
                let promote_response = if let Some(promote_kind) = effective_promote_kind {
                    Some(
                        remember_with_bundle_defaults(
                            &remember_args_from_effective_hook_capture(
                                &args,
                                content.clone(),
                                promote_kind,
                                supersede_targets.clone(),
                            ),
                            &base_url,
                        )
                        .await?,
                    )
                } else {
                    None
                };
                let supersede_responses = if let Some(response) = promote_response.as_ref() {
                    mark_hook_capture_supersede_targets(
                        &base_url,
                        &args,
                        &supersede_targets,
                        response.item.id,
                    )
                    .await?
                } else {
                    Vec::new()
                };
                let checkpoint = checkpoint_with_bundle_defaults(
                    &CheckpointArgs {
                        output: args.output.clone(),
                        project: args.project.clone(),
                        namespace: args.namespace.clone(),
                        workspace: args.workspace.clone(),
                        visibility: args.visibility.clone(),
                        source_path: args
                            .source_path
                            .clone()
                            .or(Some("hook-capture".to_string())),
                        confidence: args.confidence,
                        ttl_seconds: args.ttl_seconds.or(Some(86_400)),
                        tag: if args.tag.is_empty() {
                            vec![
                                "hook-capture".to_string(),
                                "episodic".to_string(),
                                "live-memory".to_string(),
                            ]
                        } else {
                            args.tag.clone()
                        },
                        content: Some(content.clone()),
                        input: None,
                        stdin: false,
                    },
                    &base_url,
                )
                .await;
                let checkpoint_id = checkpoint
                    .as_ref()
                    .map(|response| response.item.id.to_string())
                    .unwrap_or_else(|_| "none".to_string());
                let checkpoint_json = checkpoint
                    .as_ref()
                    .map(|response| json!(response))
                    .unwrap_or_else(|err| json!({ "error": err.to_string() }));
                let snapshot = match checkpoint {
                    Ok(_) => match read_bundle_resume(
                        &ResumeArgs {
                            output: args.output.clone(),
                            project: args.project.clone(),
                            namespace: args.namespace.clone(),
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
                    .await
                    {
                        Ok(snapshot) => Some(snapshot),
                        Err(_) => {
                            preserve_codex_capture_locally(&args.output, &content)?;
                            None
                        }
                    },
                    Err(_) => {
                        preserve_codex_capture_locally(&args.output, &content)?;
                        None
                    }
                };
                if let Some(snapshot) = snapshot.as_ref() {
                    write_bundle_memory_files(&args.output, snapshot, None, false).await?;
                    refresh_live_bundle_event_pages(&args.output, snapshot, None)?;
                    auto_checkpoint_live_snapshot(
                        &args.output,
                        &base_url,
                        snapshot,
                        "hook-capture",
                    )
                    .await?;
                    let _ = refresh_harness_pack_files_for_snapshot(
                        &args.output,
                        snapshot,
                        "hook-capture",
                        &["codex", "agent-zero", "openclaw"],
                    )
                    .await?;
                }
                if args.summary {
                    let (supersede_query, supersede_tried, supersede_hits) =
                        summarize_hook_capture_supersede_diagnostics(&supersede_diagnostics);
                    println!(
                        "hook_capture stored={} promoted={} superseded={} query={} tried={} hits={} working={} inbox={}",
                        checkpoint_id,
                        promote_response
                            .as_ref()
                            .map(|response| response.item.id.to_string())
                            .unwrap_or_else(|| "none".to_string()),
                        supersede_responses.len(),
                        supersede_query,
                        supersede_tried,
                        supersede_hits,
                        snapshot
                            .as_ref()
                            .map(|value| value.working.records.len())
                            .unwrap_or(0),
                        snapshot
                            .as_ref()
                            .map(|value| value.inbox.items.len())
                            .unwrap_or(0)
                    );
                } else {
                    print_json(&json!({
                        "live": checkpoint_json,
                        "promoted": promote_response,
                        "superseded": supersede_responses,
                        "supersede_search": supersede_diagnostics,
                    }))?;
                }
            }
            HookMode::Spill(args) => {
                let packet = read_request::<CompactionPacket>(&args.input)?;
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
                    auto_checkpoint_compaction_packet(&packet, &base_url).await?;
                    let submitted = responses.len();
                    print_json(&CompactionSpillResult {
                        submitted,
                        duplicates,
                        responses,
                        batch: spill,
                    })?;
                } else {
                    print_json(&spill)?;
                }
            }
        },
        Commands::Init(args) => {
            let decision = resolve_bootstrap_authority(args).await?;
            write_init_bundle(&decision.init_args)?;
            if decision.fallback_activated {
                set_bundle_localhost_read_only_authority_state(
                    &decision.init_args.output,
                    &decision.shared_base_url,
                    "init",
                    "shared authority unavailable during bootstrap",
                )?;
                write_agent_profiles(&decision.init_args.output)?;
                write_native_agent_bridge_files(&decision.init_args.output)?;
            }
            println!(
                "Initialized memd bundle at {}",
                decision.init_args.output.display()
            );
            if decision.fallback_activated {
                eprintln!("memd authority warning:");
                eprintln!("- shared authority unavailable");
                eprintln!("- localhost fallback is lower trust");
                eprintln!("- prompt-injection and split-brain risk increased");
                eprintln!("- coordination writes blocked");
            }
        }
        Commands::Loops(args) => {
            let entries = read_loop_entries(&args.output)?;
            if let Some(slug) = args.loop_slug.as_deref() {
                print_loop_detail(&entries, slug)?;
            } else if args.summary {
                print_loop_summary(&entries);
            } else {
                print_loop_list(&entries, &args.output);
            }
        }
        Commands::Telemetry(args) => {
            run_telemetry(&args)?;
        }
        Commands::Autoresearch(args) => {
            run_autoresearch(&args, &base_url).await?;
        }
    }

    Ok(())
}
