use super::*;
use crate::cli::terminal_ux::{self, CheckState, MenuOption};
use memd_schema::IngestLanesRequest;

/// If cwd is inside a git worktree (not the main worktree) and the main
/// worktree already has a `.memd/` bundle, symlink instead of writing a
/// fresh bundle. Keeps memory shared across worktrees so wake/lookup/etc
/// resolve to the same store regardless of which worktree you invoke
/// from. Returns Ok(true) if the symlink path was taken.
fn maybe_symlink_worktree_bundle(output: &Path) -> anyhow::Result<bool> {
    if output.is_symlink() {
        return Ok(true);
    }
    if output.exists() {
        return Ok(false);
    }
    let git_common = match std::process::Command::new("git")
        .args(["rev-parse", "--git-common-dir"])
        .output()
    {
        Ok(o) if o.status.success() => {
            let raw = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if raw.is_empty() {
                return Ok(false);
            }
            PathBuf::from(raw)
        }
        _ => return Ok(false),
    };
    let cwd = std::env::current_dir()?;
    let git_common_abs = if git_common.is_absolute() {
        git_common
    } else {
        cwd.join(&git_common)
    };
    let main_worktree = match git_common_abs.parent() {
        Some(p) => p.canonicalize().unwrap_or_else(|_| p.to_path_buf()),
        None => return Ok(false),
    };
    let cwd_canonical = cwd.canonicalize().unwrap_or(cwd.clone());
    if cwd_canonical == main_worktree {
        return Ok(false);
    }
    let main_bundle = main_worktree.join(".memd");
    if !main_bundle.is_dir() {
        return Ok(false);
    }
    let target = if output.is_absolute() {
        output.to_path_buf()
    } else {
        cwd.join(output)
    };
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&main_bundle, &target).with_context(|| {
            format!("symlink {} -> {}", target.display(), main_bundle.display())
        })?;
        eprintln!(
            "memd: detected git worktree, linked {} -> {}",
            target.display(),
            main_bundle.display()
        );
        Ok(true)
    }
    #[cfg(not(unix))]
    {
        // Windows: try directory symlink, may need elevated privileges.
        match std::os::windows::fs::symlink_dir(&main_bundle, &target) {
            Ok(()) => {
                eprintln!(
                    "memd: detected git worktree, linked {} -> {}",
                    target.display(),
                    main_bundle.display()
                );
                Ok(true)
            }
            Err(error) => {
                eprintln!(
                    "memd: detected git worktree but could not symlink {} -> {} ({}). Falling back to fresh bundle. To share memory across worktrees, run: mklink /D {} {}",
                    target.display(),
                    main_bundle.display(),
                    error,
                    target.display(),
                    main_bundle.display(),
                );
                Ok(false)
            }
        }
    }
}

const INTERACTIVE_PROVIDERS: &[&str] =
    &["Local only", "Shared memd server", "Custom MEMD_BASE_URL"];
const INTERACTIVE_PROVIDER_OPTIONS: &[MenuOption<'static>] = &[
    MenuOption::with_description(
        "Local only",
        "Use the default localhost bundle/server path.",
    ),
    MenuOption::with_description(
        "Shared memd server",
        "Use team/shared memory authority when available.",
    ),
    MenuOption::with_description(
        "Custom MEMD_BASE_URL",
        "Respect MEMD_BASE_URL or pass --base-url.",
    ),
];
const INTERACTIVE_HARNESSES: &[&str] = &[
    "codex",
    "claude-code",
    "hermes",
    "openclaw",
    "opencode",
    "done",
];
const INTERACTIVE_HARNESS_OPTIONS: &[MenuOption<'static>] = &[
    MenuOption::new("codex"),
    MenuOption::new("claude-code"),
    MenuOption::new("hermes"),
    MenuOption::new("openclaw"),
    MenuOption::new("opencode"),
    MenuOption::with_description("done", "Keep detected/default harness."),
];
const SETUP_BEGINNER_STEPS: &[(&str, &str)] = &[
    ("Install", "scripts/install-memd.sh"),
    ("Configure", "memd setup"),
    ("Doctor", "memd doctor --summary"),
    (
        "First proof",
        "memd status --output .memd --summary && memd resume --output .memd --intent current_task",
    ),
    (
        "Fixes",
        "docs/setup/troubleshooting.md maps symptom -> cause -> fix -> verify",
    ),
];

fn render_setup_guided_markdown() -> String {
    let mut out = String::new();
    out.push_str("# memd guided setup\n\n");
    out.push_str(
        "Goal: Apple-level first run. Clear steps, exact proof, no maintainer handholding.\n\n",
    );
    for (idx, (name, command)) in SETUP_BEGINNER_STEPS.iter().enumerate() {
        out.push_str(&format!("{}. **{}** — `{}`\n", idx + 1, name, command));
    }
    out.push_str("\nRun `memd setup-demo --summary` for an isolated proof that setup can create and read a bundle.\n");
    out
}

fn render_setup_guided_json() -> serde_json::Value {
    json!({
        "goal": "Apple-level first run: understandable setup, thorough docs, reliable proof",
        "steps": SETUP_BEGINNER_STEPS
            .iter()
            .enumerate()
            .map(|(idx, (name, command))| json!({
                "step": idx + 1,
                "name": name,
                "command": command,
            }))
            .collect::<Vec<_>>(),
        "proof_command": "memd setup-demo --summary",
        "troubleshooting": "docs/setup/troubleshooting.md",
    })
}

fn render_interactive_menu(
    title: &str,
    prompt: &str,
    options: &[MenuOption<'_>],
    selected: usize,
) -> String {
    let mut out = terminal_ux::render_brand_box(
        "memd Setup",
        "memory control plane",
        &format!("SETUP / {title}"),
    );
    out.push_str(
        "  Configure shared memory. Ctrl+C exits any prompt.

",
    );
    out.push_str(&terminal_ux::render_selector(prompt, options, selected));
    out
}

fn render_setup_section(section: SetupSection) -> String {
    let mut out =
        terminal_ux::render_brand_box("memd Setup", "memory control plane", "SETUP / Sections");
    match section {
        SetupSection::Provider => {
            out.push_str(&terminal_ux::render_section_header(
                "Provider",
                "Choose local, shared, or MEMD_BASE_URL-backed memory authority.",
            ));
            out.push_str(&terminal_ux::render_selector(
                "Pick where memd should connect first",
                INTERACTIVE_PROVIDER_OPTIONS,
                0,
            ));
        }
        SetupSection::Harness => {
            out.push_str(&terminal_ux::render_section_header(
                "Harness",
                "Configure bridge files for your agent surface.",
            ));
            out.push_str(&terminal_ux::render_selector(
                "Pick the agent surface you want to configure first",
                INTERACTIVE_HARNESS_OPTIONS,
                0,
            ));
        }
        SetupSection::Memory => {
            out.push_str(&terminal_ux::render_section_header(
                "Memory",
                "Initialize or refresh the .memd bundle and runtime config.",
            ));
            out.push_str(&terminal_ux::render_checklist(&[
                ("bundle config", CheckState::Ready),
                ("agent profiles", CheckState::Ready),
                ("lane ingestion", CheckState::Pending),
            ]));
            out.push_str(
                "
  Run: memd setup --summary
",
            );
        }
        SetupSection::Voice => {
            out.push_str(&terminal_ux::render_section_header(
                "Voice",
                "Set memd voice defaults for generated guidance.",
            ));
            out.push_str(
                "  Run: memd setup --voice-mode caveman-ultra --summary
",
            );
        }
        SetupSection::Hive => {
            out.push_str(&terminal_ux::render_section_header(
                "Hive",
                "Attach hive system, role, group, and coordination authority metadata.",
            ));
            out.push_str(
                "  Run: memd setup --hive-system NAME --hive-role ROLE --summary
",
            );
        }
        SetupSection::Proof => {
            out.push_str(&terminal_ux::render_section_header(
                "Proof",
                "Verify setup without changing this repository, then inspect current bundle status.",
            ));
            out.push_str(
                "  1. memd setup-demo --summary
",
            );
            out.push_str(
                "  2. memd status --output .memd --summary
",
            );
            out.push_str(
                "  3. memd resume --output .memd --intent current_task
",
            );
        }
    }
    out
}

fn should_run_setup_picker(args: &SetupArgs) -> bool {
    use std::io::IsTerminal;
    if args.summary || args.json || args.guided || args.non_interactive || args.section.is_some() {
        return false;
    }
    if args.project.is_some()
        || args.namespace.is_some()
        || args.global
        || args.project_root.is_some()
        || args.agent.is_some()
        || args.session.is_some()
        || args.tab_id.is_some()
        || args.hive_system.is_some()
        || args.hive_role.is_some()
        || !args.capability.is_empty()
        || !args.hive_group.is_empty()
        || args.hive_group_goal.is_some()
        || args.authority.is_some()
        || args.output.is_some()
        || args.base_url.is_some()
        || args.rag_url.is_some()
        || args.route.is_some()
        || args.intent.is_some()
        || args.workspace.is_some()
        || args.visibility.is_some()
        || args.voice_mode.is_some()
        || args.force
        || args.allow_localhost_read_only_fallback
    {
        return false;
    }
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

fn run_centered_picker(title: &str, prompt: &str, options: &[&str]) -> anyhow::Result<usize> {
    use console::{Key, Term};
    let term = Term::stdout();
    let mut selected = 0usize;
    loop {
        term.clear_screen()?;
        let menu_options: Vec<MenuOption<'_>> = options
            .iter()
            .map(|option| MenuOption::new(option))
            .collect();
        term.write_line(&render_interactive_menu(
            title,
            prompt,
            &menu_options,
            selected,
        ))?;
        match term.read_key()? {
            Key::ArrowUp => selected = selected.saturating_sub(1),
            Key::ArrowDown => selected = (selected + 1).min(options.len().saturating_sub(1)),
            Key::Enter => return Ok(selected),
            Key::Char('q') | Key::Escape | Key::CtrlC => {
                anyhow::bail!("interactive setup cancelled")
            }
            _ => {}
        }
    }
}

fn run_interactive_setup(args: &SetupArgs) -> anyhow::Result<SetupArgs> {
    let provider_idx = run_centered_picker(
        "Provider",
        "Pick where memd should connect first",
        INTERACTIVE_PROVIDERS,
    )?;
    let harness_idx = run_centered_picker(
        "Harness",
        "Pick the agent surface you want to configure first",
        INTERACTIVE_HARNESSES,
    )?;
    let mut configured = args.clone();
    configured.summary = true;
    configured.agent = match INTERACTIVE_HARNESSES[harness_idx] {
        "done" => configured.agent.or_else(|| Some("codex".to_string())),
        harness => Some(harness.to_string()),
    };
    if provider_idx == 2 && configured.base_url.is_none() {
        println!(
            "custom provider selected; using MEMD_BASE_URL/default unless --base-url is supplied"
        );
    }
    println!(
        "setup selection provider={} harness={}",
        INTERACTIVE_PROVIDERS[provider_idx],
        configured.agent.as_deref().unwrap_or("codex")
    );
    Ok(configured)
}

pub(crate) async fn run_bundle_setup_command(args: &SetupArgs) -> anyhow::Result<()> {
    if let Some(section) = args.section {
        println!("{}", render_setup_section(section));
        return Ok(());
    }
    if args.guided {
        if args.json {
            print_json(&render_setup_guided_json())?;
        } else {
            print!("{}", render_setup_guided_markdown());
        }
        return Ok(());
    }
    let interactive_args;
    let args = if should_run_setup_picker(args) {
        interactive_args = run_interactive_setup(args)?;
        &interactive_args
    } else {
        args
    };
    let init_args = normalize_init_args(setup_args_to_init_args(args))?;
    let decision = resolve_bootstrap_authority(init_args).await?;
    let init_args = decision.init_args;

    // W1: If we're in a git worktree and the main worktree already has a
    // `.memd/` bundle, symlink it instead of forking a fresh per-worktree
    // store. Memory must be continuous across worktrees of the same project.
    let symlinked = maybe_symlink_worktree_bundle(&init_args.output).unwrap_or(false);

    let bundle_preexisted = init_args.output.exists() && !init_args.force;
    if !symlinked && !bundle_preexisted {
        write_init_bundle(&init_args)?;
    } else if !symlinked && bundle_preexisted {
        write_agent_profiles(&init_args.output)?;
        write_native_agent_bridge_files(&init_args.output)?;
    }

    // F2: Ingest lane source files into DB after setup.
    if let Ok(memd) = MemdClient::new(&init_args.base_url) {
        let root = init_args
            .output
            .parent()
            .unwrap_or(&init_args.output)
            .to_path_buf();
        let _ = memd
            .ingest_lanes(&IngestLanesRequest {
                root: root.display().to_string(),
                project: init_args.project.clone(),
                namespace: init_args.namespace.clone(),
            })
            .await;
    }

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
        if bundle_preexisted {
            println!(
                "Updated existing memd bundle at {}",
                init_args.output.display()
            );
        } else {
            println!("Initialized memd bundle at {}", init_args.output.display());
        }
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

#[cfg(test)]
mod setup_interactive_tests {
    use super::*;

    #[test]
    fn setup_menu_mimics_hermes_openclaw_brand_language() {
        let menu = render_interactive_menu("Provider", "Pick one", INTERACTIVE_PROVIDER_OPTIONS, 1);
        assert!(menu.contains("memd"));
        assert!(menu.contains("memory control plane"));
        assert!(menu.contains("SETUP / Provider"));
        assert!(menu.contains("◆  2. Shared memd server"));
        assert!(menu.contains("◇  1. Local only"));
        assert!(!menu.contains("Hermes Agent Setup Wizard"));
        assert!(!menu.contains("Choice [default"));
    }

    #[test]
    fn setup_picker_lists_requested_harnesses() {
        let menu = render_interactive_menu("Harness", "Pick one", INTERACTIVE_HARNESS_OPTIONS, 3);
        assert!(menu.contains("codex"));
        assert!(menu.contains("hermes"));
        assert!(menu.contains("◆  4. openclaw"));
        assert!(menu.contains("opencode"));
    }

    #[test]
    fn guided_setup_renders_hands_on_beginner_path() {
        let guide = render_setup_guided_markdown();
        assert!(guide.contains("Apple-level first run"));
        assert!(guide.contains("memd setup"));
        assert!(!guide.contains("--interactive"));
        assert!(guide.contains("memd setup-demo --summary"));
        assert!(guide.contains("docs/setup/troubleshooting.md"));
    }

    #[test]
    fn guided_setup_json_has_redacted_proof_path() {
        let guide = render_setup_guided_json();
        assert_eq!(guide["proof_command"], "memd setup-demo --summary");
        assert!(guide["steps"].as_array().unwrap().len() >= 5);
    }
    #[test]
    fn setup_provider_section_renders_provider_picker() {
        let rendered = render_setup_section(SetupSection::Provider);
        assert!(rendered.contains("memd Setup"));
        assert!(rendered.contains("Provider"));
        assert!(rendered.contains("Shared memd server"));
        assert!(!rendered.contains("--interactive"));
    }

    #[test]
    fn setup_proof_section_prints_proof_commands() {
        let rendered = render_setup_section(SetupSection::Proof);
        assert!(rendered.contains("memd setup-demo --summary"));
        assert!(rendered.contains("memd status --output .memd --summary"));
        assert!(!rendered.contains("--interactive"));
    }

    #[test]
    fn setup_non_interactive_flag_skips_picker() {
        let mut args = minimal_setup_args();
        args.non_interactive = true;
        assert!(!should_run_setup_picker(&args));
    }

    #[test]
    fn setup_summary_skips_picker() {
        let mut args = minimal_setup_args();
        args.summary = true;
        assert!(!should_run_setup_picker(&args));
    }

    #[test]
    fn setup_section_skips_picker() {
        let mut args = minimal_setup_args();
        args.section = Some(SetupSection::Proof);
        assert!(!should_run_setup_picker(&args));
    }

    fn minimal_setup_args() -> SetupArgs {
        SetupArgs {
            section: None,
            project: None,
            namespace: None,
            global: false,
            project_root: None,
            seed_existing: true,
            agent: None,
            session: None,
            tab_id: None,
            hive_system: None,
            hive_role: None,
            capability: Vec::new(),
            hive_group: Vec::new(),
            hive_group_goal: None,
            authority: None,
            output: None,
            base_url: None,
            rag_url: None,
            route: None,
            intent: None,
            workspace: None,
            visibility: None,
            voice_mode: None,
            force: false,
            guided: false,
            non_interactive: false,
            allow_localhost_read_only_fallback: false,
            summary: false,
            json: false,
        }
    }
}

pub(crate) async fn run_bundle_setup_demo_command(args: &SetupDemoArgs) -> anyhow::Result<()> {
    let temp_root = tempfile::tempdir().context("create setup demo temp root")?;
    let output = temp_root.path().join(".memd");
    let setup_args = SetupArgs {
        section: None,
        project: Some("memd-setup-demo".to_string()),
        namespace: Some("demo".to_string()),
        global: false,
        project_root: Some(temp_root.path().to_path_buf()),
        seed_existing: true,
        agent: Some("codex".to_string()),
        session: None,
        tab_id: None,
        hive_system: None,
        hive_role: None,
        capability: Vec::new(),
        hive_group: Vec::new(),
        hive_group_goal: None,
        authority: None,
        output: Some(output.clone()),
        base_url: None,
        rag_url: None,
        route: None,
        intent: Some("current_task".to_string()),
        workspace: None,
        visibility: Some("private".to_string()),
        voice_mode: Some(default_voice_mode()),
        force: true,
        guided: false,
        non_interactive: true,
        allow_localhost_read_only_fallback: true,
        summary: true,
        json: false,
    };
    let decision = resolve_bootstrap_authority(setup_args_to_init_args(&setup_args)).await?;
    write_init_bundle(&decision.init_args)?;
    let bundle_root = if output.join("config.json").is_file() {
        output.clone()
    } else {
        temp_root.path().join(&output)
    };
    let first_memory_proof =
        bundle_root.join("wake.md").is_file() || bundle_root.join("mem.md").is_file();
    let status_ok = bundle_root.join("config.json").is_file() && first_memory_proof;
    if args.json {
        print_json(&json!({
            "demo_ready": status_ok,
            "bundle": "<temp>/.memd",
            "project": "memd-setup-demo",
            "first_memory_proof": first_memory_proof,
            "result": if status_ok { "setup-demo=pass" } else { "setup-demo=fail" },
        }))?;
    } else {
        println!(
            "setup-demo bundle=<temp>/.memd project=memd-setup-demo first_memory_proof={} ready={}",
            first_memory_proof, status_ok
        );
        println!("setup-demo={}", if status_ok { "pass" } else { "fail" });
    }
    anyhow::ensure!(status_ok, "setup demo did not create readable bundle");
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
        println!("doctor {}", render_bundle_status_summary(&status));
    } else {
        println!("{}", render_doctor_status_markdown(&bundle_root, &status));
    }
    Ok(())
}

pub(crate) async fn run_bundle_config_command(
    args: &ConfigArgs,
    base_url: &str,
) -> anyhow::Result<()> {
    run_bundle_config_or_settings_command(args, base_url, false).await
}

pub(crate) async fn run_bundle_settings_command(
    args: &ConfigArgs,
    base_url: &str,
) -> anyhow::Result<()> {
    run_bundle_config_or_settings_command(args, base_url, true).await
}

async fn run_bundle_config_or_settings_command(
    args: &ConfigArgs,
    base_url: &str,
    settings_invocation: bool,
) -> anyhow::Result<()> {
    let bundle_root = resolve_setup_bundle_root(args.output.as_deref())?;
    if !bundle_root.join("config.json").is_file() {
        let setup_args = SetupArgs {
            section: None,
            project: None,
            namespace: None,
            global: false,
            project_root: args.project_root.clone(),
            seed_existing: true,
            agent: None,
            session: None,
            tab_id: None,
            hive_system: None,
            hive_role: None,
            capability: Vec::new(),
            hive_group: Vec::new(),
            hive_group_goal: None,
            authority: None,
            output: Some(bundle_root.clone()),
            base_url: None,
            rag_url: None,
            route: None,
            intent: None,
            workspace: None,
            visibility: None,
            voice_mode: None,
            force: false,
            guided: false,
            non_interactive: true,
            allow_localhost_read_only_fallback: false,
            summary: true,
            json: false,
        };
        let init_args = normalize_init_args(setup_args_to_init_args(&setup_args))?;
        let decision = resolve_bootstrap_authority(init_args).await?;
        write_init_bundle(&decision.init_args)?;
    }
    if let Some(command) = &args.command {
        return run_bundle_config_subcommand(&bundle_root, command);
    }
    for setting in &args.set {
        apply_bundle_config_setting(&bundle_root, setting)?;
    }
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
        if settings_invocation {
            println!("{}", render_bundle_settings_summary(&config));
        } else {
            println!("{}", render_bundle_config_summary(&config));
        }
    } else if settings_invocation {
        println!("{}", render_bundle_settings_human(&config));
    } else {
        println!("{}", render_bundle_config_markdown(&config));
    }
    Ok(())
}

pub(crate) fn run_device_command(args: &DeviceArgs) -> anyhow::Result<()> {
    match &args.command {
        DeviceCommand::Add(add_args) => run_device_add_command(add_args),
    }
}

pub(crate) fn run_dogfood_command(args: &DogfoodArgs) -> anyhow::Result<()> {
    match &args.command {
        DogfoodCommand::Enroll(enroll_args) => run_dogfood_enroll_command(enroll_args),
        DogfoodCommand::Status(status_args) => run_dogfood_status_command(status_args),
    }
}

fn run_device_add_command(args: &DeviceAddArgs) -> anyhow::Result<()> {
    let bundle_root = resolve_setup_bundle_root(args.output.as_deref())?;
    ensure_bundle_exists_for_dogfood(&bundle_root)?;
    let user = args.user.clone().unwrap_or_else(default_local_user);
    let record = register_device_record(&bundle_root, args.name.as_deref(), &user)?;
    if args.json {
        print_json(&record)?;
    } else {
        println!(
            "device add: id={} name={} user={} bundle={} evidence={}",
            record
                .get("device_id")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown"),
            record
                .get("name")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown"),
            record
                .get("user")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown"),
            bundle_root.display(),
            record
                .get("evidence_path")
                .and_then(JsonValue::as_str)
                .unwrap_or("bundle-only")
        );
    }
    Ok(())
}

fn run_dogfood_enroll_command(args: &DogfoodEnrollArgs) -> anyhow::Result<()> {
    if !args.consent && std::env::var("MEMD_DOGFOOD_CONSENT").ok().as_deref() != Some("1") {
        anyhow::bail!("dogfood enroll requires --consent so real-user evidence is explicit");
    }

    let bundle_root = resolve_setup_bundle_root(args.output.as_deref())?;
    ensure_bundle_exists_for_dogfood(&bundle_root)?;
    let user_id = args
        .user_id
        .clone()
        .unwrap_or_else(|| format!("user-{}", short_uuid()));
    let device_id = match &args.device_id {
        Some(id) => id.clone(),
        None => latest_device_id(&bundle_root)?.unwrap_or_else(|| {
            register_device_record(&bundle_root, None, &user_id)
                .ok()
                .and_then(|record| {
                    record
                        .get("device_id")
                        .and_then(JsonValue::as_str)
                        .map(str::to_string)
                })
                .unwrap_or_else(|| format!("device-{}", short_uuid()))
        }),
    };
    let harnesses = if args.harness.is_empty() {
        vec![active_agent_or_default(&bundle_root)]
    } else {
        args.harness.clone()
    };
    let enrollment_id = format!("dogfood-{}", short_uuid());
    let now = Utc::now();
    let started_at = now.to_rfc3339();
    let started_on = now.format("%Y-%m-%d").to_string();
    let weekly_review_due = (now + chrono::Duration::days(7))
        .format("%Y-%m-%d")
        .to_string();
    let record = json!({
        "enrollment_id": enrollment_id,
        "user_id": user_id,
        "device_id": device_id,
        "harnesses": harnesses,
        "consent": true,
        "started_at": started_at,
        "started_on": started_on,
        "weekly_review_due": weekly_review_due,
        "bundle": bundle_root,
        "git_head": current_git_head().unwrap_or_else(|| "unknown".to_string()),
    });
    append_state_record(&bundle_root, "dogfood.json", "enrollments", record.clone())?;
    let evidence_path = write_dogfood_enrollment_artifact(&record)?;
    ensure_weekly_evidence_note(&weekly_review_due)?;
    let mut response = record;
    if let JsonValue::Object(ref mut map) = response {
        if let Some(path) = evidence_path {
            map.insert(
                "evidence_path".to_string(),
                JsonValue::String(path.display().to_string()),
            );
        }
    }
    if args.json {
        print_json(&response)?;
    } else {
        println!(
            "dogfood enroll: enrollment={} user={} device={} harnesses={} due={}",
            response
                .get("enrollment_id")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown"),
            response
                .get("user_id")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown"),
            response
                .get("device_id")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown"),
            response
                .get("harnesses")
                .and_then(JsonValue::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(JsonValue::as_str)
                        .collect::<Vec<_>>()
                        .join(",")
                })
                .unwrap_or_else(|| "unknown".to_string()),
            response
                .get("weekly_review_due")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown")
        );
    }
    Ok(())
}

fn run_dogfood_status_command(args: &DogfoodStatusArgs) -> anyhow::Result<()> {
    let bundle_root = resolve_setup_bundle_root(args.output.as_deref())?;
    ensure_bundle_exists_for_dogfood(&bundle_root)?;
    let dogfood = read_state_json(&bundle_root, "dogfood.json")?;
    let devices = read_state_json(&bundle_root, "devices.json")?;
    let enrollments = dogfood
        .get("enrollments")
        .and_then(JsonValue::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let device_count = devices
        .get("devices")
        .and_then(JsonValue::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let response = json!({
        "bundle": bundle_root,
        "enrollments": enrollments,
        "devices": device_count,
        "real_user_gate": if enrollments >= 3 { "ready" } else { "needs_3_users" },
        "device_gate": if device_count >= 3 { "ready" } else { "needs_3_devices" },
        "next": if enrollments < 3 {
            "run memd dogfood enroll --user-id <id> --consent"
        } else if device_count < 3 {
            "run memd device add --user <id> on another machine"
        } else {
            "collect weekly evidence notes"
        },
    });
    if args.json {
        print_json(&response)?;
    } else {
        println!(
            "dogfood status: enrollments={} devices={} real_user_gate={} device_gate={} next=\"{}\"",
            enrollments,
            device_count,
            response
                .get("real_user_gate")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown"),
            response
                .get("device_gate")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown"),
            response
                .get("next")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown")
        );
    }
    Ok(())
}

fn ensure_bundle_exists_for_dogfood(bundle_root: &Path) -> anyhow::Result<()> {
    if !bundle_root.join("config.json").exists() {
        anyhow::bail!(
            "memd bundle missing at {}; run `memd setup --summary` first",
            bundle_root.display()
        );
    }
    Ok(())
}

fn register_device_record(
    bundle_root: &Path,
    name: Option<&str>,
    user: &str,
) -> anyhow::Result<JsonValue> {
    let device_id = format!("device-{}", short_uuid());
    let now = Utc::now();
    let record = json!({
        "device_id": device_id,
        "name": name.map(str::to_string).unwrap_or_else(default_device_name),
        "user": user,
        "added_at": now.to_rfc3339(),
        "added_on": now.format("%Y-%m-%d").to_string(),
        "bundle": bundle_root,
        "git_head": current_git_head().unwrap_or_else(|| "unknown".to_string()),
    });
    append_state_record(bundle_root, "devices.json", "devices", record.clone())?;
    let evidence_path = write_device_artifact(&record)?;
    let mut response = record;
    if let JsonValue::Object(ref mut map) = response {
        if let Some(path) = evidence_path {
            map.insert(
                "evidence_path".to_string(),
                JsonValue::String(path.display().to_string()),
            );
        }
    }
    Ok(response)
}

fn append_state_record(
    bundle_root: &Path,
    file_name: &str,
    list_key: &str,
    record: JsonValue,
) -> anyhow::Result<()> {
    let state_dir = bundle_root.join("state");
    fs::create_dir_all(&state_dir)?;
    let path = state_dir.join(file_name);
    let mut doc = if path.exists() {
        serde_json::from_str::<JsonValue>(&fs::read_to_string(&path)?)
            .with_context(|| format!("parse {}", path.display()))?
    } else {
        json!({ list_key: [] })
    };
    let list = doc
        .as_object_mut()
        .ok_or_else(|| anyhow!("{} must contain a JSON object", path.display()))?
        .entry(list_key.to_string())
        .or_insert_with(|| json!([]));
    list.as_array_mut()
        .ok_or_else(|| anyhow!("{}.{} must be an array", path.display(), list_key))?
        .push(record);
    fs::write(&path, serde_json::to_string_pretty(&doc)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn read_state_json(bundle_root: &Path, file_name: &str) -> anyhow::Result<JsonValue> {
    let path = bundle_root.join("state").join(file_name);
    if !path.exists() {
        return Ok(json!({}));
    }
    serde_json::from_str::<JsonValue>(&fs::read_to_string(&path)?)
        .with_context(|| format!("parse {}", path.display()))
}

fn latest_device_id(bundle_root: &Path) -> anyhow::Result<Option<String>> {
    Ok(read_state_json(bundle_root, "devices.json")?
        .get("devices")
        .and_then(JsonValue::as_array)
        .and_then(|devices| devices.last())
        .and_then(|device| device.get("device_id"))
        .and_then(JsonValue::as_str)
        .map(str::to_string))
}

fn active_agent_or_default(bundle_root: &Path) -> String {
    read_config_json(bundle_root)
        .ok()
        .and_then(|doc| {
            doc.get("agent")
                .and_then(JsonValue::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "codex".to_string())
}

fn default_local_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown-user".to_string())
}

fn default_device_name() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("memd-device-{}", short_uuid()))
}

fn short_uuid() -> String {
    uuid::Uuid::new_v4().to_string().chars().take(8).collect()
}

fn current_git_head() -> Option<String> {
    let root = detect_current_project_root().ok().flatten()?;
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn release_evidence_root() -> Option<PathBuf> {
    let root = detect_current_project_root().ok().flatten()?;
    Some(root.join("docs").join("verification").join("release-1-0-0"))
}

fn write_device_artifact(record: &JsonValue) -> anyhow::Result<Option<PathBuf>> {
    let Some(root) = release_evidence_root() else {
        return Ok(None);
    };
    let dir = root.join("devices");
    fs::create_dir_all(&dir)?;
    let date = record
        .get("added_on")
        .and_then(JsonValue::as_str)
        .unwrap_or("unknown-date");
    let device_id = record
        .get("device_id")
        .and_then(JsonValue::as_str)
        .unwrap_or("unknown-device");
    let path = dir.join(format!("{date}-{device_id}.json"));
    fs::write(&path, serde_json::to_string_pretty(record)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(Some(path))
}

fn write_dogfood_enrollment_artifact(record: &JsonValue) -> anyhow::Result<Option<PathBuf>> {
    let Some(root) = release_evidence_root() else {
        return Ok(None);
    };
    let dir = root.join("dogfood");
    fs::create_dir_all(&dir)?;
    let date = record
        .get("started_on")
        .and_then(JsonValue::as_str)
        .unwrap_or("unknown-date");
    let enrollment_id = record
        .get("enrollment_id")
        .and_then(JsonValue::as_str)
        .unwrap_or("unknown-enrollment");
    let path = dir.join(format!("{date}-{enrollment_id}.md"));
    let harnesses = record
        .get("harnesses")
        .and_then(JsonValue::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(JsonValue::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_else(|| "unknown".to_string());
    let markdown = format!(
        "# Dogfood Enrollment - {date}\n\n\
Status: enrolled\n\n\
- enrollment: `{}`\n\
- user: `{}`\n\
- device: `{}`\n\
- harnesses: `{}`\n\
- started: `{}`\n\
- weekly review due: `{}`\n\
- git head: `{}`\n\n\
## Evidence Notes\n\n\
- Install completed.\n\
- Real usage clock started.\n",
        enrollment_id,
        record
            .get("user_id")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown"),
        record
            .get("device_id")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown"),
        harnesses,
        record
            .get("started_at")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown"),
        record
            .get("weekly_review_due")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown"),
        record
            .get("git_head")
            .and_then(JsonValue::as_str)
            .unwrap_or("unknown"),
    );
    fs::write(&path, markdown).with_context(|| format!("write {}", path.display()))?;
    Ok(Some(path))
}

fn ensure_weekly_evidence_note(date: &str) -> anyhow::Result<()> {
    let Some(root) = release_evidence_root() else {
        return Ok(());
    };
    fs::create_dir_all(&root)?;
    let path = root.join(format!("{date}-weekly-review.md"));
    if path.exists() {
        return Ok(());
    }
    let markdown = format!(
        "# 1.0.0 Weekly Evidence Review - {date}\n\n\
Status: pending review\n\n\
## Cohort\n\n\
- real users enrolled:\n\
- harness-user pairs enrolled:\n\
- devices on current main:\n\n\
## Gate Notes\n\n\
- V14 telemetry dogfood:\n\
- V15 self-tuning dogfood:\n\
- V16 sync dogfood:\n\
- V17 marketplace dogfood:\n\
- V18 correction graph dogfood:\n\
- V19 external auditor:\n\
- V20 third-party replay:\n\n\
## Blockers\n\n\
- none recorded yet\n"
    );
    fs::write(&path, markdown).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn run_bundle_config_subcommand(bundle_root: &Path, command: &ConfigCommand) -> anyhow::Result<()> {
    match command {
        ConfigCommand::List(args) => {
            let doc = read_config_json(bundle_root)?;
            let rows = config_setting_rows(&doc);
            if args.json {
                print_json(&rows)?;
            } else {
                for row in rows {
                    println!(
                        "{}={} default={} owner={}",
                        row.key, row.value, row.default, row.owner
                    );
                }
            }
        }
        ConfigCommand::Get(args) => {
            let key = normalize_config_key(&args.key)?;
            let doc = read_config_json(bundle_root)?;
            let row = config_setting_rows(&doc)
                .into_iter()
                .find(|row| row.key == key)
                .expect("validated key exists");
            if args.json {
                print_json(&row)?;
            } else {
                println!("{}", row.value);
            }
        }
        ConfigCommand::Set(args) => {
            let (key, value) = args
                .setting
                .split_once('=')
                .ok_or_else(|| anyhow::anyhow!("config setting must be KEY=VALUE"))?;
            let key = normalize_config_key(key)?;
            let mut doc = read_config_json(bundle_root)?;
            set_config_value(&mut doc, &key, value.trim())?;
            write_config_json(bundle_root, &doc)?;
            let row = config_setting_rows(&doc)
                .into_iter()
                .find(|row| row.key == key)
                .expect("validated key exists");
            if args.json {
                print_json(&row)?;
            } else {
                println!("{}={}", row.key, row.value);
            }
        }
        ConfigCommand::Reset(args) => {
            let mut doc = read_config_json(bundle_root)?;
            if let Some(key) = &args.key {
                let key = normalize_config_key(key)?;
                set_default_config_value(&mut doc, &key)?;
            } else {
                for key in CONFIG_KEYS {
                    set_default_config_value(&mut doc, key)?;
                }
            }
            write_config_json(bundle_root, &doc)?;
            let rows = config_setting_rows(&doc);
            if args.json {
                print_json(&rows)?;
            } else if let Some(key) = &args.key {
                let key = normalize_config_key(key)?;
                let row = rows
                    .into_iter()
                    .find(|row| row.key == key)
                    .expect("validated key exists");
                println!("{}={}", row.key, row.value);
            } else {
                println!("reset {} settings", CONFIG_KEYS.len());
            }
        }
        ConfigCommand::ShowSchema(args) => {
            let schema = config_schema_json();
            if args.json {
                print_json(&schema)?;
            } else {
                println!("{}", serde_json::to_string_pretty(&schema)?);
            }
        }
    }
    Ok(())
}

fn apply_bundle_config_setting(bundle_root: &Path, setting: &str) -> anyhow::Result<()> {
    let (key, value) = setting
        .split_once('=')
        .ok_or_else(|| anyhow::anyhow!("config setting must be KEY=VALUE"))?;
    let key = normalize_config_key(key)?;
    let mut doc = read_config_json(bundle_root)?;
    set_config_value(&mut doc, key, value.trim())?;
    write_config_json(bundle_root, &doc)
}

#[derive(Debug, Clone, Serialize)]
struct ConfigSettingRow {
    key: &'static str,
    value: JsonValue,
    default: JsonValue,
    ty: &'static str,
    owner: &'static str,
}

const CONFIG_KEYS: &[&str] = &[
    "auto_commit.enabled",
    "cost_ledger.budget_tokens",
    "cost_ledger.per_turn_warn",
    "provenance.drilldown_depth_max",
    "telemetry.enabled",
    "telemetry.retention_days",
    "telemetry.export_scope",
    "compiler.mode",
    "compiler.self_tuning.min_samples",
    "compiler.self_tuning.min_quality_score",
    "compiler.self_tuning.max_quality_regression",
    "compiler.self_tuning.max_budget_regression_pct",
    "sync.enabled",
    "sync.relay_url",
    "sync.conflict_policy",
    "voice.mode",
    "visibility.default_scope",
    "feature_flags.v11_compiler",
    "feature_flags.v13_sota_guard",
];

fn read_config_json(bundle_root: &Path) -> anyhow::Result<JsonValue> {
    let config_path = bundle_root.join("config.json");
    if !config_path.exists() {
        anyhow::bail!(
            "{} does not exist; initialize the bundle first",
            config_path.display()
        );
    }
    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))
}

fn write_config_json(bundle_root: &Path, doc: &JsonValue) -> anyhow::Result<()> {
    let config_path = bundle_root.join("config.json");
    let tmp_path = config_path.with_extension("json.tmp");
    fs::write(&tmp_path, serde_json::to_string_pretty(doc)? + "\n")
        .with_context(|| format!("write {}", tmp_path.display()))?;
    fs::rename(&tmp_path, &config_path)
        .with_context(|| format!("replace {}", config_path.display()))?;
    Ok(())
}

fn config_setting_rows(doc: &JsonValue) -> Vec<ConfigSettingRow> {
    CONFIG_KEYS
        .iter()
        .map(|key| ConfigSettingRow {
            key,
            value: config_value(doc, key),
            default: config_default_value(key),
            ty: config_type(key),
            owner: config_owner(key),
        })
        .collect()
}

pub(crate) fn config_budget_tokens_from_bundle(output: &Path) -> Option<usize> {
    read_config_json(output).ok().and_then(|doc| {
        config_value(&doc, "cost_ledger.budget_tokens")
            .as_u64()
            .map(|value| value as usize)
    })
}

fn normalize_config_key(key: &str) -> anyhow::Result<&'static str> {
    let key = key.trim();
    if let Some(found) = CONFIG_KEYS.iter().find(|candidate| **candidate == key) {
        return Ok(found);
    }
    let hint = CONFIG_KEYS
        .iter()
        .min_by_key(|candidate| levenshtein(key, candidate))
        .copied()
        .unwrap_or("auto_commit.enabled");
    eprintln!("unknown config key '{key}'; did you mean '{hint}'?");
    Err(anyhow::Error::new(HookEnforceExitCode(2)))
}

fn config_default_value(key: &str) -> JsonValue {
    match key {
        "auto_commit.enabled" => json!(true),
        "cost_ledger.budget_tokens" => json!(4000),
        "cost_ledger.per_turn_warn" => json!(1200),
        "provenance.drilldown_depth_max" => json!(3),
        "telemetry.enabled" => json!(false),
        "telemetry.retention_days" => json!(default_telemetry_retention_days()),
        "telemetry.export_scope" => json!(default_telemetry_export_scope()),
        "compiler.mode" => json!(memd_core::self_tuning::CompilerMode::Dynamic.as_str()),
        "compiler.self_tuning.min_samples" => {
            json!(memd_core::self_tuning::DEFAULT_MIN_TUNING_SAMPLES)
        }
        "compiler.self_tuning.min_quality_score" => {
            json!(memd_core::self_tuning::DEFAULT_MIN_QUALITY_SCORE)
        }
        "compiler.self_tuning.max_quality_regression" => {
            json!(memd_core::self_tuning::DEFAULT_MAX_QUALITY_REGRESSION)
        }
        "compiler.self_tuning.max_budget_regression_pct" => {
            json!(memd_core::self_tuning::DEFAULT_MAX_BUDGET_REGRESSION_PCT)
        }
        "sync.enabled" => json!(false),
        "sync.relay_url" => JsonValue::Null,
        "sync.conflict_policy" => json!("last_writer_wins"),
        "voice.mode" => json!(default_voice_mode()),
        "visibility.default_scope" => json!("project"),
        "feature_flags.v11_compiler" => json!(false),
        "feature_flags.v13_sota_guard" => json!(false),
        _ => JsonValue::Null,
    }
}

fn config_type(key: &str) -> &'static str {
    match key {
        "auto_commit.enabled"
        | "telemetry.enabled"
        | "sync.enabled"
        | "feature_flags.v11_compiler"
        | "feature_flags.v13_sota_guard" => "bool",
        "cost_ledger.budget_tokens"
        | "cost_ledger.per_turn_warn"
        | "provenance.drilldown_depth_max"
        | "compiler.self_tuning.min_samples"
        | "telemetry.retention_days" => "number",
        "compiler.self_tuning.min_quality_score"
        | "compiler.self_tuning.max_quality_regression"
        | "compiler.self_tuning.max_budget_regression_pct" => "float",
        "voice.mode"
        | "visibility.default_scope"
        | "telemetry.export_scope"
        | "compiler.mode"
        | "sync.relay_url"
        | "sync.conflict_policy" => "string",
        _ => "unknown",
    }
}

fn config_owner(key: &str) -> &'static str {
    match key {
        "auto_commit.enabled" => "V7 H7",
        "cost_ledger.budget_tokens" | "cost_ledger.per_turn_warn" => "V8 E8/G8",
        "provenance.drilldown_depth_max" => "V8 D8/G8",
        "telemetry.enabled" | "telemetry.retention_days" | "telemetry.export_scope" => "V14",
        "compiler.mode"
        | "compiler.self_tuning.min_samples"
        | "compiler.self_tuning.min_quality_score"
        | "compiler.self_tuning.max_quality_regression"
        | "compiler.self_tuning.max_budget_regression_pct" => "V15",
        "sync.enabled" | "sync.relay_url" | "sync.conflict_policy" => "V16",
        "voice.mode" => "memd voice bootstrap",
        "visibility.default_scope" => "V9 reserved",
        "feature_flags.v11_compiler" => "V11 reserved",
        "feature_flags.v13_sota_guard" => "V13 reserved",
        _ => "unknown",
    }
}

fn config_value(doc: &JsonValue, key: &str) -> JsonValue {
    match key {
        "voice.mode" => doc
            .get("voice_mode")
            .cloned()
            .unwrap_or_else(|| config_default_value(key)),
        _ => read_nested(doc, key).unwrap_or_else(|| config_default_value(key)),
    }
}

fn read_nested(doc: &JsonValue, key: &str) -> Option<JsonValue> {
    let mut current = doc;
    for part in key.split('.') {
        current = current.get(part)?;
    }
    Some(current.clone())
}

fn set_default_config_value(doc: &mut JsonValue, key: &str) -> anyhow::Result<()> {
    let value = config_default_value(key);
    write_config_value(doc, key, value)
}

fn set_config_value(doc: &mut JsonValue, key: &str, raw: &str) -> anyhow::Result<()> {
    let value = match config_type(key) {
        "bool" => json!(parse_config_bool(raw)?),
        "number" => {
            let parsed: u64 = raw
                .parse()
                .with_context(|| format!("parse numeric config value for {key}"))?;
            json!(parsed)
        }
        "float" => {
            let parsed: f64 = raw
                .parse()
                .with_context(|| format!("parse float config value for {key}"))?;
            json!(parsed)
        }
        "string" => {
            if key == "compiler.mode" {
                let mode =
                    <memd_core::self_tuning::CompilerMode as std::str::FromStr>::from_str(raw)
                        .map_err(anyhow::Error::msg)?;
                json!(mode.as_str())
            } else {
                json!(raw.trim().to_string())
            }
        }
        _ => anyhow::bail!("unsupported config key '{key}'"),
    };
    write_config_value(doc, key, value)
}

fn write_config_value(doc: &mut JsonValue, key: &str, value: JsonValue) -> anyhow::Result<()> {
    if !doc.is_object() {
        *doc = json!({});
    }
    if key == "voice.mode" {
        doc.as_object_mut()
            .expect("object checked")
            .insert("voice_mode".to_string(), value);
        return Ok(());
    }
    let mut current = doc;
    let mut parts = key.split('.').peekable();
    while let Some(part) = parts.next() {
        if parts.peek().is_none() {
            current
                .as_object_mut()
                .expect("object checked")
                .insert(part.to_string(), value);
            return Ok(());
        }
        let obj = current.as_object_mut().expect("object checked");
        current = obj.entry(part).or_insert_with(|| json!({}));
        if !current.is_object() {
            *current = json!({});
        }
    }
    Ok(())
}

fn config_schema_json() -> JsonValue {
    let properties = config_setting_rows(&json!({}))
        .into_iter()
        .map(|row| {
            (
                row.key.to_string(),
                json!({
                    "type": row.ty,
                    "default": row.default,
                    "owner": row.owner,
                }),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "memd runtime settings",
        "schema_version": "0.3",
        "properties": properties,
        "additionalProperties": true
    })
}

fn levenshtein(a: &str, b: &str) -> usize {
    let mut costs = (0..=b.chars().count()).collect::<Vec<_>>();
    for (i, ca) in a.chars().enumerate() {
        let mut last = i;
        costs[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let old = costs[j + 1];
            let substitution = last + usize::from(ca != cb);
            costs[j + 1] = (costs[j + 1] + 1).min(costs[j] + 1).min(substitution);
            last = old;
        }
    }
    costs[b.chars().count()]
}

fn parse_config_bool(value: &str) -> anyhow::Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "enabled" => Ok(true),
        "0" | "false" | "no" | "off" | "disabled" => Ok(false),
        other => anyhow::bail!("invalid boolean '{other}'"),
    }
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
