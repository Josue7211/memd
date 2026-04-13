use super::*;

pub(crate) async fn run_bundle_setup_command(args: &SetupArgs) -> anyhow::Result<()> {
    let init_args = normalize_init_args(setup_args_to_init_args(args))?;
    let decision = resolve_bootstrap_authority(init_args).await?;
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
    Ok(())
}

pub(crate) async fn run_bundle_doctor_command(
    args: &DoctorArgs,
    base_url: &str,
) -> anyhow::Result<()> {
    let bundle_root = resolve_doctor_bundle_root(args.output.as_deref())?;
    let mut status = read_bundle_status(&bundle_root, base_url).await?;
    let setup_ready = status
        .get("setup_ready")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    if args.repair && !setup_ready {
        let project_root = args.project_root.clone().or(detect_current_project_root()?);
        let setup_args = doctor_args_to_setup_args(args, bundle_root.clone(), project_root);
        let decision = resolve_bootstrap_authority(setup_args_to_init_args(&setup_args)).await?;
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
        status = read_bundle_status(&bundle_root, base_url).await?;
    } else if args.repair {
        let repaired_worker_env = repair_bundle_worker_name_env(&bundle_root)?;
        if repaired_worker_env {
            write_agent_profiles(&bundle_root)?;
        }
        status = read_bundle_status(&bundle_root, base_url).await?;
    }
    if args.json {
        print_json(&status)?;
    } else if args.summary {
        println!("{}", render_bundle_status_summary(&status));
    } else {
        println!("{}", render_doctor_status_markdown(&bundle_root, &status));
    }
    Ok(())
}

pub(crate) async fn run_bundle_config_command(
    args: &ConfigArgs,
    base_url: &str,
) -> anyhow::Result<()> {
    let bundle_root = resolve_setup_bundle_root(args.output.as_deref())?;
    let project_root = args.project_root.clone().or(detect_current_project_root()?);
    let runtime = read_bundle_runtime_config(&bundle_root)?;
    let status = read_bundle_status(&bundle_root, base_url).await.ok();
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
    Ok(())
}

pub(crate) async fn run_bundle_init_command(args: InitArgs) -> anyhow::Result<()> {
    let args = normalize_init_args(args)?;
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
    Ok(())
}
