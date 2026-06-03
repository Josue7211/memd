use super::*;
use crate::bundle::BundleConfigFile;
use crate::cli::terminal_ux::{PanelRow, PanelSection, ready_mark, render_panel};

pub(crate) fn read_bundle_voice_mode(output: &Path) -> Option<String> {
    read_bundle_config_file(output)
        .ok()
        .and_then(|(_, config)| {
            config
                .voice_mode
                .and_then(|value| normalize_voice_mode_value(&value).ok())
        })
}

pub(crate) fn set_bundle_agent(output: &Path, agent: &str) -> anyhow::Result<()> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        anyhow::bail!(
            "{} does not exist; initialize the bundle first",
            config_path.display()
        );
    }

    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let mut config: BundleConfigFile =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    config.agent = Some(agent.to_string());
    fs::write(&config_path, serde_json::to_string_pretty(&config)? + "\n")
        .with_context(|| format!("write {}", config_path.display()))?;

    let session = config.session.clone();
    let effective_agent = compose_agent_identity(agent, session.as_deref());
    let worker_name = default_bundle_worker_name_for_project(
        config.project.as_deref(),
        agent,
        session.as_deref(),
    );
    rewrite_shell_env(&output.join("env"), "MEMD_AGENT", &effective_agent)?;
    rewrite_shell_env(&output.join("env"), "MEMD_WORKER_NAME", &worker_name)?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_AGENT = ",
        &format!("$env:MEMD_AGENT = \"{}\"\n", escape_ps1(&effective_agent)),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_WORKER_NAME = ",
        &format!("$env:MEMD_WORKER_NAME = \"{}\"\n", escape_ps1(&worker_name)),
    )?;

    Ok(())
}

pub(crate) fn set_bundle_session(output: &Path, session: &str) -> anyhow::Result<()> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        anyhow::bail!(
            "{} does not exist; initialize the bundle first",
            config_path.display()
        );
    }

    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let mut config: BundleConfigFile =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    config.session = Some(session.to_string());
    fs::write(&config_path, serde_json::to_string_pretty(&config)? + "\n")
        .with_context(|| format!("write {}", config_path.display()))?;

    let agent = config.agent.as_deref().unwrap_or("unknown");
    let effective_agent = compose_agent_identity(agent, Some(session));
    let worker_name =
        default_bundle_worker_name_for_project(config.project.as_deref(), agent, Some(session));
    rewrite_shell_env(&output.join("env"), "MEMD_SESSION", session)?;
    rewrite_shell_env(&output.join("env"), "MEMD_AGENT", &effective_agent)?;
    rewrite_shell_env(&output.join("env"), "MEMD_WORKER_NAME", &worker_name)?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_SESSION = ",
        &format!("$env:MEMD_SESSION = \"{}\"\n", escape_ps1(session)),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_AGENT = ",
        &format!("$env:MEMD_AGENT = \"{}\"\n", escape_ps1(&effective_agent)),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_WORKER_NAME = ",
        &format!("$env:MEMD_WORKER_NAME = \"{}\"\n", escape_ps1(&worker_name)),
    )?;

    Ok(())
}

pub(crate) fn set_bundle_tab_id(output: &Path, tab_id: &str) -> anyhow::Result<()> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        anyhow::bail!(
            "{} does not exist; initialize the bundle first",
            config_path.display()
        );
    }

    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let mut config: BundleConfigFile =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    config.tab_id = Some(tab_id.to_string());
    fs::write(&config_path, serde_json::to_string_pretty(&config)? + "\n")
        .with_context(|| format!("write {}", config_path.display()))?;

    rewrite_shell_env(&output.join("env"), "MEMD_TAB_ID", tab_id)?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_TAB_ID = ",
        &format!("$env:MEMD_TAB_ID = \"{}\"\n", escape_ps1(tab_id)),
    )?;

    Ok(())
}

pub(crate) fn set_bundle_base_url(output: &Path, base_url: &str) -> anyhow::Result<()> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        anyhow::bail!(
            "{} does not exist; initialize the bundle first",
            config_path.display()
        );
    }

    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let mut config: BundleConfigFile =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    config.base_url = Some(base_url.to_string());
    if config.authority_state.mode == "shared" {
        config.authority_state.shared_base_url = Some(base_url.to_string());
        config.authority_state.fallback_base_url = None;
        config.authority_state.degraded = false;
        config.authority_state.blocked_capabilities.clear();
    }
    fs::write(&config_path, serde_json::to_string_pretty(&config)? + "\n")
        .with_context(|| format!("write {}", config_path.display()))?;

    rewrite_shell_env(&output.join("env"), "MEMD_BASE_URL", base_url)?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_BASE_URL = ",
        &format!("$env:MEMD_BASE_URL = \"{}\"\n", escape_ps1(base_url)),
    )?;
    if config.authority_state.mode == "shared" {
        write_bundle_authority_env(output, &config.authority_policy, &config.authority_state)?;
    }

    Ok(())
}

pub(crate) fn set_bundle_project(output: &Path, project: &str) -> anyhow::Result<()> {
    let (config_path, mut config) = read_bundle_config_file(output)?;
    config.project = Some(project.trim().to_string());
    write_bundle_config_file(&config_path, &config)?;
    rewrite_shell_env(&output.join("env"), "MEMD_PROJECT", project.trim())?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_PROJECT = ",
        &format!("$env:MEMD_PROJECT = \"{}\"\n", escape_ps1(project.trim())),
    )?;
    Ok(())
}

pub(crate) fn set_bundle_namespace(output: &Path, namespace: &str) -> anyhow::Result<()> {
    let (config_path, mut config) = read_bundle_config_file(output)?;
    config.namespace = Some(namespace.trim().to_string());
    write_bundle_config_file(&config_path, &config)?;
    rewrite_shell_env(&output.join("env"), "MEMD_NAMESPACE", namespace.trim())?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_NAMESPACE = ",
        &format!(
            "$env:MEMD_NAMESPACE = \"{}\"\n",
            escape_ps1(namespace.trim())
        ),
    )?;
    Ok(())
}

pub(crate) fn set_bundle_route(output: &Path, route: &str) -> anyhow::Result<()> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        anyhow::bail!(
            "{} does not exist; initialize the bundle first",
            config_path.display()
        );
    }

    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let mut config: BundleConfigFile =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    config.route = Some(route.to_string());
    fs::write(&config_path, serde_json::to_string_pretty(&config)? + "\n")
        .with_context(|| format!("write {}", config_path.display()))?;

    rewrite_shell_env(&output.join("env"), "MEMD_ROUTE", route)?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_ROUTE = ",
        &format!("$env:MEMD_ROUTE = \"{}\"\n", escape_ps1(route)),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::terminal_ux::{PanelRow, PanelSection, ready_mark, render_panel};

    #[test]
    fn settings_summary_uses_settings_prefix_and_config_prefix_stays_config() {
        let snapshot = BundleConfigSnapshot {
            bundle_root: "/tmp/memd".to_string(),
            project_root: None,
            setup_ready: true,
            runtime_present: true,
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            session: Some("session-a".to_string()),
            tab_id: None,
            base_url: Some("http://127.0.0.1:8787".to_string()),
            route: Some("auto".to_string()),
            intent: Some("current_task".to_string()),
            workspace: None,
            visibility: None,
            hive_system: None,
            hive_role: None,
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            authority_mode: Some("shared".to_string()),
            authority_degraded: false,
            shared_base_url: None,
            fallback_base_url: None,
            localhost_fallback_policy: Some("deny".to_string()),
            voice_mode: "caveman-ultra".to_string(),
            auto_commit_enabled: false,
        };

        assert!(render_bundle_settings_summary(&snapshot).starts_with("settings "));
        assert!(render_bundle_config_summary(&snapshot).starts_with("config "));
        let human = render_bundle_settings_human(&snapshot);
        assert!(human.contains("memd Settings"));
        assert!(human.contains("╔"));
        assert!(human.contains("Auto commit"));
        assert!(human.contains("disabled"));
    }

    #[test]
    fn set_bundle_base_url_updates_shared_authority_state_when_shared() {
        let dir = std::env::temp_dir().join(format!(
            "memd-bundle-shared-base-url-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "general",
  "authority_policy": {
    "shared_primary": true,
    "localhost_fallback_policy": "deny"
  },
  "authority_state": {
    "mode": "shared",
    "degraded": false,
    "shared_base_url": "http://127.0.0.1:8787",
    "fallback_base_url": null,
    "blocked_capabilities": []
  }
}
"#,
        )
        .expect("write config");
        fs::write(
            dir.join("env"),
            "MEMD_BASE_URL=http://127.0.0.1:8787\nMEMD_SHARED_BASE_URL=http://127.0.0.1:8787\n",
        )
        .expect("write env");
        fs::write(
            dir.join("env.ps1"),
            "$env:MEMD_BASE_URL = \"http://127.0.0.1:8787\"\n$env:MEMD_SHARED_BASE_URL = \"http://127.0.0.1:8787\"\n",
        )
        .expect("write env.ps1");

        set_bundle_base_url(&dir, "http://127.0.0.1:9797").expect("set bundle base url");

        let config = fs::read_to_string(dir.join("config.json")).expect("read config");
        let env = fs::read_to_string(dir.join("env")).expect("read env");
        let env_ps1 = fs::read_to_string(dir.join("env.ps1")).expect("read env.ps1");
        assert!(config.contains(r#""base_url": "http://127.0.0.1:9797""#));
        assert!(config.contains(r#""shared_base_url": "http://127.0.0.1:9797""#));
        assert!(env.contains("MEMD_BASE_URL='http://127.0.0.1:9797'"));
        assert!(env.contains("MEMD_SHARED_BASE_URL='http://127.0.0.1:9797'"));
        assert!(env_ps1.contains("$env:MEMD_BASE_URL = \"http://127.0.0.1:9797\""));
        assert!(env_ps1.contains("$env:MEMD_SHARED_BASE_URL = \"http://127.0.0.1:9797\""));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }
}

pub(crate) fn set_bundle_intent(output: &Path, intent: &str) -> anyhow::Result<()> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        anyhow::bail!(
            "{} does not exist; initialize the bundle first",
            config_path.display()
        );
    }

    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let mut config: BundleConfigFile =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    config.intent = Some(intent.to_string());
    fs::write(&config_path, serde_json::to_string_pretty(&config)? + "\n")
        .with_context(|| format!("write {}", config_path.display()))?;

    rewrite_shell_env(&output.join("env"), "MEMD_INTENT", intent)?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_INTENT = ",
        &format!("$env:MEMD_INTENT = \"{}\"\n", escape_ps1(intent)),
    )?;

    Ok(())
}

pub(crate) fn resolve_setup_bundle_root(explicit: Option<&Path>) -> anyhow::Result<PathBuf> {
    if let Some(explicit) = explicit {
        return Ok(explicit.to_path_buf());
    }

    Ok(resolve_default_bundle_root()?.unwrap_or_else(default_init_output_path))
}

pub(crate) fn resolve_doctor_bundle_root(explicit: Option<&Path>) -> anyhow::Result<PathBuf> {
    resolve_setup_bundle_root(explicit)
}

pub(crate) fn setup_args_to_init_args(args: &SetupArgs) -> InitArgs {
    let project_root = args.project_root.clone();
    let output = args.output.clone().unwrap_or_else(default_init_output_path);
    let project_root_ref = project_root.as_deref();
    let agent = args
        .agent
        .clone()
        .unwrap_or_else(|| detect_setup_agent(project_root_ref, &output));

    InitArgs {
        project: args.project.clone(),
        namespace: args.namespace.clone(),
        global: args.global,
        project_root,
        seed_existing: args.seed_existing,
        agent,
        session: args.session.clone(),
        tab_id: args.tab_id.clone(),
        hive_system: args.hive_system.clone(),
        hive_role: args.hive_role.clone(),
        capability: args.capability.clone(),
        hive_group: args.hive_group.clone(),
        hive_group_goal: args.hive_group_goal.clone(),
        authority: args.authority.clone(),
        output,
        base_url: args.base_url.clone().unwrap_or_else(default_base_url),
        rag_url: args.rag_url.clone(),
        route: args.route.clone().unwrap_or_else(|| "auto".to_string()),
        intent: args
            .intent
            .clone()
            .unwrap_or_else(|| "current_task".to_string()),
        workspace: args.workspace.clone(),
        visibility: args.visibility.clone(),
        voice_mode: args
            .voice_mode
            .as_deref()
            .map(|value| normalize_voice_mode_value(value).unwrap_or_else(|_| default_voice_mode()))
            .or_else(|| Some(default_voice_mode())),
        force: args.force,
        allow_localhost_read_only_fallback: args.allow_localhost_read_only_fallback,
    }
}

pub(crate) fn doctor_args_to_setup_args(
    args: &DoctorArgs,
    output: PathBuf,
    project_root: Option<PathBuf>,
) -> SetupArgs {
    SetupArgs {
        section: None,
        project: None,
        namespace: None,
        global: output == default_global_bundle_root(),
        project_root,
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
        output: Some(output),
        base_url: None,
        rag_url: None,
        route: None,
        intent: None,
        workspace: None,
        visibility: None,
        voice_mode: Some(default_voice_mode()),
        force: args.repair,
        guided: false,
        non_interactive: true,
        allow_localhost_read_only_fallback: false,
        summary: false,
        json: false,
    }
}

pub(crate) fn detect_setup_agent(project_root: Option<&Path>, output: &Path) -> String {
    if let Some(project_root) = project_root {
        if project_root.join("CLAUDE.md").exists() || project_root.join(".claude").exists() {
            return "claude-code".to_string();
        }
        if project_root.join("AGENTS.md").exists() {
            return "codex".to_string();
        }
        if project_root.join(".agents").exists() {
            return "opencode".to_string();
        }
    }

    if output.join("agents").join("CLAUDE_IMPORTS.md").exists() {
        return "claude-code".to_string();
    }

    if let Some(home) = home_dir() {
        if home.join(".claude").is_dir() {
            return "claude-code".to_string();
        }
        if home.join(".codex").is_dir() {
            return "codex".to_string();
        }
        if home.join(".config").join("opencode").is_dir() || home.join(".opencode").is_dir() {
            return "opencode".to_string();
        }
        if home.join(".openclaw").join("workspace").is_dir() {
            return "openclaw".to_string();
        }
        if home.join(".config").join("claw").is_dir() || home.join(".claw").is_dir() {
            return "openclaw".to_string();
        }
    }

    "codex".to_string()
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BundleConfigSnapshot {
    bundle_root: String,
    project_root: Option<String>,
    setup_ready: bool,
    runtime_present: bool,
    project: Option<String>,
    namespace: Option<String>,
    agent: Option<String>,
    session: Option<String>,
    tab_id: Option<String>,
    base_url: Option<String>,
    route: Option<String>,
    intent: Option<String>,
    workspace: Option<String>,
    visibility: Option<String>,
    hive_system: Option<String>,
    hive_role: Option<String>,
    hive_group_goal: Option<String>,
    authority: Option<String>,
    authority_mode: Option<String>,
    authority_degraded: bool,
    shared_base_url: Option<String>,
    fallback_base_url: Option<String>,
    localhost_fallback_policy: Option<String>,
    voice_mode: String,
    auto_commit_enabled: bool,
}

pub(crate) fn render_bundle_config_snapshot(
    bundle_root: &Path,
    project_root: Option<&Path>,
    runtime: Option<&BundleRuntimeConfig>,
    status: Option<&serde_json::Value>,
) -> BundleConfigSnapshot {
    let setup_ready = status
        .and_then(|value| value.get("setup_ready"))
        .and_then(|value| value.as_bool())
        .unwrap_or(runtime.is_some() && bundle_root.exists());
    let voice_mode = status
        .and_then(|value| value.get("defaults"))
        .and_then(|value| value.get("voice_mode"))
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .or_else(|| read_bundle_voice_mode(bundle_root))
        .unwrap_or_else(default_voice_mode);

    BundleConfigSnapshot {
        bundle_root: bundle_root.display().to_string(),
        project_root: project_root.map(|path| path.display().to_string()),
        setup_ready,
        runtime_present: runtime.is_some(),
        project: runtime.as_ref().and_then(|value| value.project.clone()),
        namespace: runtime.as_ref().and_then(|value| value.namespace.clone()),
        agent: runtime.as_ref().and_then(|value| value.agent.clone()),
        session: runtime.as_ref().and_then(|value| value.session.clone()),
        tab_id: runtime.as_ref().and_then(|value| value.tab_id.clone()),
        base_url: runtime.as_ref().and_then(|value| value.base_url.clone()),
        route: runtime.as_ref().and_then(|value| value.route.clone()),
        intent: runtime.as_ref().and_then(|value| value.intent.clone()),
        workspace: runtime.as_ref().and_then(|value| value.workspace.clone()),
        visibility: runtime.as_ref().and_then(|value| value.visibility.clone()),
        hive_system: runtime.as_ref().and_then(|value| value.hive_system.clone()),
        hive_role: runtime.as_ref().and_then(|value| value.hive_role.clone()),
        hive_group_goal: runtime
            .as_ref()
            .and_then(|value| value.hive_group_goal.clone()),
        authority: runtime.as_ref().and_then(|value| value.authority.clone()),
        authority_mode: runtime
            .as_ref()
            .map(|value| value.authority_state.mode.clone()),
        authority_degraded: runtime
            .as_ref()
            .map(|value| value.authority_state.degraded)
            .unwrap_or(false),
        shared_base_url: runtime
            .as_ref()
            .and_then(|value| value.authority_state.shared_base_url.clone()),
        fallback_base_url: runtime
            .as_ref()
            .and_then(|value| value.authority_state.fallback_base_url.clone()),
        localhost_fallback_policy: runtime.as_ref().map(|value| {
            value
                .authority_policy
                .localhost_fallback_policy
                .as_str()
                .to_string()
        }),
        voice_mode,
        auto_commit_enabled: runtime
            .as_ref()
            .map(|value| value.auto_commit.enabled)
            .unwrap_or_else(default_true),
    }
}

pub(crate) fn render_bundle_config_summary(config: &BundleConfigSnapshot) -> String {
    render_bundle_config_summary_with_label(config, "config")
}

pub(crate) fn render_bundle_settings_summary(config: &BundleConfigSnapshot) -> String {
    render_bundle_config_summary_with_label(config, "settings")
}

fn render_bundle_config_summary_with_label(config: &BundleConfigSnapshot, label: &str) -> String {
    format!(
        "{} bundle={} ready={} project={} namespace={} agent={} session={} base_url={} route={} intent={} voice={} auto_commit={} authority={} degraded={}",
        label,
        config.bundle_root,
        config.setup_ready,
        config.project.as_deref().unwrap_or("none"),
        config.namespace.as_deref().unwrap_or("none"),
        config.agent.as_deref().unwrap_or("none"),
        config.session.as_deref().unwrap_or("none"),
        config.base_url.as_deref().unwrap_or("none"),
        config.route.as_deref().unwrap_or("none"),
        config.intent.as_deref().unwrap_or("none"),
        config.voice_mode.as_str(),
        if config.auto_commit_enabled {
            "on"
        } else {
            "off"
        },
        config.authority_mode.as_deref().unwrap_or("shared"),
        if config.authority_degraded {
            "yes"
        } else {
            "no"
        }
    )
}

pub(crate) fn render_bundle_settings_human(config: &BundleConfigSnapshot) -> String {
    let ready = format!("{} {}", ready_mark(config.setup_ready), config.setup_ready);
    let runtime = if config.runtime_present {
        "✓ present"
    } else {
        "✗ missing"
    }
    .to_string();
    let auto_commit = if config.auto_commit_enabled {
        "enabled"
    } else {
        "disabled"
    }
    .to_string();
    let degraded = if config.authority_degraded {
        "yes"
    } else {
        "no"
    }
    .to_string();

    let bundle_values = vec![
        redact_display(config.bundle_root.as_str()),
        config
            .project_root
            .as_deref()
            .map(redact_display)
            .unwrap_or_else(|| "none".to_string()),
        ready,
        runtime,
    ];
    let identity_values = vec![
        value_or_none(config.project.as_deref()),
        value_or_none(config.namespace.as_deref()),
        value_or_none(config.agent.as_deref()),
        value_or_none(config.session.as_deref()),
        value_or_none(config.tab_id.as_deref()),
    ];
    let routing_values = vec![
        value_or_none(config.base_url.as_deref()),
        value_or_none(config.route.as_deref()),
        value_or_none(config.intent.as_deref()),
        config.voice_mode.clone(),
        auto_commit,
    ];
    let hive_values = vec![
        value_or_none(config.workspace.as_deref()),
        value_or_none(config.visibility.as_deref()),
        value_or_none(config.hive_system.as_deref()),
        value_or_none(config.hive_role.as_deref()),
        value_or_none(config.hive_group_goal.as_deref()),
    ];
    let authority_values = vec![
        value_or_none(config.authority.as_deref()),
        value_or_default(config.authority_mode.as_deref(), "shared"),
        degraded,
        value_or_none(config.shared_base_url.as_deref()),
        value_or_none(config.fallback_base_url.as_deref()),
        value_or_default(config.localhost_fallback_policy.as_deref(), "deny"),
    ];

    let bundle_rows = [
        PanelRow {
            label: "Bundle",
            value: bundle_values[0].as_str(),
        },
        PanelRow {
            label: "Project root",
            value: bundle_values[1].as_str(),
        },
        PanelRow {
            label: "Ready",
            value: bundle_values[2].as_str(),
        },
        PanelRow {
            label: "Runtime",
            value: bundle_values[3].as_str(),
        },
    ];
    let identity_rows = [
        PanelRow {
            label: "Project",
            value: identity_values[0].as_str(),
        },
        PanelRow {
            label: "Namespace",
            value: identity_values[1].as_str(),
        },
        PanelRow {
            label: "Agent",
            value: identity_values[2].as_str(),
        },
        PanelRow {
            label: "Session",
            value: identity_values[3].as_str(),
        },
        PanelRow {
            label: "Tab",
            value: identity_values[4].as_str(),
        },
    ];
    let routing_rows = [
        PanelRow {
            label: "Base URL",
            value: routing_values[0].as_str(),
        },
        PanelRow {
            label: "Route",
            value: routing_values[1].as_str(),
        },
        PanelRow {
            label: "Intent",
            value: routing_values[2].as_str(),
        },
        PanelRow {
            label: "Voice",
            value: routing_values[3].as_str(),
        },
        PanelRow {
            label: "Auto commit",
            value: routing_values[4].as_str(),
        },
    ];
    let hive_rows = [
        PanelRow {
            label: "Workspace",
            value: hive_values[0].as_str(),
        },
        PanelRow {
            label: "Visibility",
            value: hive_values[1].as_str(),
        },
        PanelRow {
            label: "Hive system",
            value: hive_values[2].as_str(),
        },
        PanelRow {
            label: "Hive role",
            value: hive_values[3].as_str(),
        },
        PanelRow {
            label: "Hive goal",
            value: hive_values[4].as_str(),
        },
    ];
    let authority_rows = [
        PanelRow {
            label: "Authority",
            value: authority_values[0].as_str(),
        },
        PanelRow {
            label: "Mode",
            value: authority_values[1].as_str(),
        },
        PanelRow {
            label: "Degraded",
            value: authority_values[2].as_str(),
        },
        PanelRow {
            label: "Shared URL",
            value: authority_values[3].as_str(),
        },
        PanelRow {
            label: "Fallback URL",
            value: authority_values[4].as_str(),
        },
        PanelRow {
            label: "Localhost",
            value: authority_values[5].as_str(),
        },
    ];
    let sections = [
        PanelSection {
            title: "Bundle",
            body: Some("Where this workspace stores memory and runtime config."),
            rows: &bundle_rows,
        },
        PanelSection {
            title: "Identity",
            body: Some("Current project, namespace, agent, and session."),
            rows: &identity_rows,
        },
        PanelSection {
            title: "Routing",
            body: Some("How memd connects and what voice agents should use."),
            rows: &routing_rows,
        },
        PanelSection {
            title: "Hive",
            body: Some("Coordination metadata for shared agent work."),
            rows: &hive_rows,
        },
        PanelSection {
            title: "Authority",
            body: Some("Trust and fallback policy for shared memory writes."),
            rows: &authority_rows,
        },
    ];
    render_panel("memd Settings", "memory control plane", &sections)
}

fn value_or_none(value: Option<&str>) -> String {
    value_or_default(value, "none")
}

fn value_or_default(value: Option<&str>, default: &str) -> String {
    value
        .map(redact_display)
        .unwrap_or_else(|| default.to_string())
}

fn redact_display(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return "none".to_string();
    }
    if value.contains('@') && !value.starts_with("http://") && !value.starts_with("https://") {
        return "[redacted]".to_string();
    }
    let lower = value.to_ascii_lowercase();
    if lower.contains("token")
        || lower.contains("secret")
        || lower.contains("password")
        || lower.contains("apikey")
        || lower.contains("api_key")
    {
        return "[redacted]".to_string();
    }
    value.to_string()
}

fn truncate_for_box(value: &str, width: usize) -> String {
    let count = value.chars().count();
    if count <= width {
        return value.to_string();
    }
    if width <= 1 {
        return "…".to_string();
    }
    value.chars().take(width - 1).collect::<String>() + "…"
}

pub(crate) fn render_bundle_config_markdown(config: &BundleConfigSnapshot) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd config\n\n");
    markdown.push_str(&format!("- bundle: `{}`\n", config.bundle_root));
    if let Some(project_root) = config.project_root.as_deref() {
        markdown.push_str(&format!("- project root: `{}`\n", project_root));
    }
    markdown.push_str(&format!("- ready: `{}`\n", config.setup_ready));
    markdown.push_str(&format!(
        "- runtime: `{}`\n",
        if config.runtime_present {
            "present"
        } else {
            "missing"
        }
    ));
    markdown.push_str(&format!(
        "- project: `{}`\n",
        config.project.as_deref().unwrap_or("none")
    ));
    markdown.push_str(&format!(
        "- namespace: `{}`\n",
        config.namespace.as_deref().unwrap_or("none")
    ));
    markdown.push_str(&format!(
        "- agent: `{}`\n",
        config.agent.as_deref().unwrap_or("none")
    ));
    markdown.push_str(&format!(
        "- base url: `{}`\n",
        config.base_url.as_deref().unwrap_or("none")
    ));
    markdown.push_str(&format!(
        "- route: `{}`\n",
        config.route.as_deref().unwrap_or("none")
    ));
    markdown.push_str(&format!(
        "- intent: `{}`\n",
        config.intent.as_deref().unwrap_or("none")
    ));
    markdown.push_str(&format!("- voice mode: `{}`\n", config.voice_mode.as_str()));
    markdown.push_str(&format!(
        "- auto commit: `{}`\n",
        if config.auto_commit_enabled {
            "enabled"
        } else {
            "disabled"
        }
    ));
    markdown.push_str(&format!(
        "- workspace: `{}`\n",
        config.workspace.as_deref().unwrap_or("none")
    ));
    markdown.push_str(&format!(
        "- visibility: `{}`\n",
        config.visibility.as_deref().unwrap_or("none")
    ));
    markdown.push_str(&format!(
        "- hive system: `{}`\n",
        config.hive_system.as_deref().unwrap_or("none")
    ));
    markdown.push_str(&format!(
        "- hive role: `{}`\n",
        config.hive_role.as_deref().unwrap_or("none")
    ));
    markdown.push_str(&format!(
        "- hive group goal: `{}`\n",
        config.hive_group_goal.as_deref().unwrap_or("none")
    ));
    markdown.push_str(&format!(
        "- authority: `{}`\n",
        config.authority.as_deref().unwrap_or("none")
    ));
    markdown.push_str(&format!(
        "- authority mode: `{}`\n",
        config.authority_mode.as_deref().unwrap_or("shared")
    ));
    markdown.push_str(&format!(
        "- authority degraded: `{}`\n",
        if config.authority_degraded {
            "yes"
        } else {
            "no"
        }
    ));
    markdown.push_str(&format!(
        "- shared base url: `{}`\n",
        config.shared_base_url.as_deref().unwrap_or("none")
    ));
    markdown.push_str(&format!(
        "- fallback base url: `{}`\n",
        config.fallback_base_url.as_deref().unwrap_or("none")
    ));
    markdown.push_str(&format!(
        "- localhost fallback policy: `{}`\n",
        config
            .localhost_fallback_policy
            .as_deref()
            .unwrap_or("deny")
    ));
    markdown
}

pub(crate) fn render_doctor_status_markdown(
    bundle_root: &Path,
    status: &serde_json::Value,
) -> String {
    let setup_ready = status
        .get("setup_ready")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let server = status
        .get("server")
        .and_then(|value| value.get("status"))
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    let missing_values = status
        .get("missing")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "none".to_string());
    let ready_value = format!("{} {}", ready_mark(setup_ready), setup_ready);
    let server_value = format!("{} {}", ready_mark(server == "ok"), server);
    let core_values = vec![
        bundle_root.display().to_string(),
        ready_value,
        server_value,
        missing_values,
    ];
    let core_rows = [
        PanelRow {
            label: "Bundle",
            value: core_values[0].as_str(),
        },
        PanelRow {
            label: "Ready",
            value: core_values[1].as_str(),
        },
        PanelRow {
            label: "Server",
            value: core_values[2].as_str(),
        },
        PanelRow {
            label: "Missing",
            value: core_values[3].as_str(),
        },
    ];
    if let Some(evolution) = status
        .get("evolution")
        .and_then(|value| if value.is_null() { None } else { Some(value) })
    {
        let evolution_values = vec![
            evolution
                .get("proposal_state")
                .and_then(|value| value.as_str())
                .unwrap_or("none")
                .to_string(),
            format!(
                "{} / {}",
                evolution
                    .get("scope_class")
                    .and_then(|value| value.as_str())
                    .unwrap_or("none"),
                evolution
                    .get("scope_gate")
                    .and_then(|value| value.as_str())
                    .unwrap_or("none")
            ),
            evolution
                .get("authority_tier")
                .and_then(|value| value.as_str())
                .unwrap_or("none")
                .to_string(),
            format!(
                "merge={} durability={}",
                evolution
                    .get("merge_status")
                    .and_then(|value| value.as_str())
                    .unwrap_or("none"),
                evolution
                    .get("durability_status")
                    .and_then(|value| value.as_str())
                    .unwrap_or("none")
            ),
            evolution
                .get("branch")
                .and_then(|value| value.as_str())
                .unwrap_or("none")
                .to_string(),
        ];
        let evolution_rows = [
            PanelRow {
                label: "Proposal",
                value: evolution_values[0].as_str(),
            },
            PanelRow {
                label: "Scope",
                value: evolution_values[1].as_str(),
            },
            PanelRow {
                label: "Authority",
                value: evolution_values[2].as_str(),
            },
            PanelRow {
                label: "Queues",
                value: evolution_values[3].as_str(),
            },
            PanelRow {
                label: "Branch",
                value: evolution_values[4].as_str(),
            },
        ];
        let sections = [
            PanelSection {
                title: "Readiness",
                body: Some("Doctor checks the local bundle and repair state."),
                rows: &core_rows,
            },
            PanelSection {
                title: "Evolution",
                body: Some("Pending memory evolution and durability gates."),
                rows: &evolution_rows,
            },
        ];
        return render_panel("memd Doctor", "memory control plane", &sections);
    }
    let sections = [PanelSection {
        title: "Readiness",
        body: Some("Doctor checks the local bundle and repair state."),
        rows: &core_rows,
    }];
    render_panel("memd Doctor", "memory control plane", &sections)
}

pub(crate) fn set_bundle_workspace(output: &Path, workspace: &str) -> anyhow::Result<()> {
    let (config_path, mut config) = read_bundle_config_file(output)?;
    config.workspace = Some(workspace.trim().to_string());
    write_bundle_config_file(&config_path, &config)?;
    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_WORKSPACE=",
        &format!("MEMD_WORKSPACE={}\n", workspace.trim()),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_WORKSPACE = ",
        &format!(
            "$env:MEMD_WORKSPACE = \"{}\"\n",
            escape_ps1(workspace.trim())
        ),
    )?;
    Ok(())
}

pub(crate) fn set_bundle_visibility(output: &Path, visibility: &str) -> anyhow::Result<()> {
    let (config_path, mut config) = read_bundle_config_file(output)?;
    config.visibility = Some(visibility.trim().to_string());
    write_bundle_config_file(&config_path, &config)?;
    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_VISIBILITY=",
        &format!("MEMD_VISIBILITY={}\n", visibility.trim()),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_VISIBILITY = ",
        &format!(
            "$env:MEMD_VISIBILITY = \"{}\"\n",
            escape_ps1(visibility.trim())
        ),
    )?;
    Ok(())
}

pub(crate) fn set_bundle_auto_short_term_capture(
    output: &Path,
    enabled: bool,
) -> anyhow::Result<()> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        anyhow::bail!(
            "{} does not exist; initialize the bundle first",
            config_path.display()
        );
    }

    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let mut config: BundleConfigFile =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    config.auto_short_term_capture = enabled;
    fs::write(&config_path, serde_json::to_string_pretty(&config)? + "\n")
        .with_context(|| format!("write {}", config_path.display()))?;

    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_AUTO_SHORT_TERM_CAPTURE=",
        &format!(
            "MEMD_AUTO_SHORT_TERM_CAPTURE={}\n",
            if enabled { "true" } else { "false" }
        ),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_AUTO_SHORT_TERM_CAPTURE = ",
        &format!(
            "$env:MEMD_AUTO_SHORT_TERM_CAPTURE = \"{}\"\n",
            if enabled { "true" } else { "false" }
        ),
    )?;

    Ok(())
}

pub(crate) fn set_bundle_auto_commit_enabled(output: &Path, enabled: bool) -> anyhow::Result<()> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        anyhow::bail!(
            "{} does not exist; initialize the bundle first",
            config_path.display()
        );
    }

    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let mut config: BundleConfigFile =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    config.auto_commit.enabled = enabled;
    fs::write(&config_path, serde_json::to_string_pretty(&config)? + "\n")
        .with_context(|| format!("write {}", config_path.display()))?;

    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_AUTO_COMMIT_ENABLED=",
        &format!(
            "MEMD_AUTO_COMMIT_ENABLED={}\n",
            if enabled { "true" } else { "false" }
        ),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_AUTO_COMMIT_ENABLED = ",
        &format!(
            "$env:MEMD_AUTO_COMMIT_ENABLED = \"{}\"\n",
            if enabled { "true" } else { "false" }
        ),
    )?;

    Ok(())
}

pub(crate) fn set_bundle_voice_mode(output: &Path, voice_mode: &str) -> anyhow::Result<()> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        anyhow::bail!(
            "{} does not exist; initialize the bundle first",
            config_path.display()
        );
    }

    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let mut config: BundleConfigFile =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    let voice_mode = normalize_voice_mode_value(voice_mode)?;
    config.voice_mode = Some(voice_mode.clone());
    fs::write(&config_path, serde_json::to_string_pretty(&config)? + "\n")
        .with_context(|| format!("write {}", config_path.display()))?;

    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_VOICE_MODE=",
        &format!("MEMD_VOICE_MODE={voice_mode}\n"),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_VOICE_MODE = ",
        &format!("$env:MEMD_VOICE_MODE = \"{}\"\n", escape_ps1(&voice_mode)),
    )?;

    Ok(())
}

pub(crate) fn read_bundle_config_file(
    output: &Path,
) -> anyhow::Result<(PathBuf, BundleConfigFile)> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        anyhow::bail!(
            "{} does not exist; initialize the bundle first",
            config_path.display()
        );
    }
    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let config: BundleConfigFile =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    Ok((config_path, config))
}

pub(crate) fn write_bundle_config_file(
    config_path: &Path,
    config: &BundleConfigFile,
) -> anyhow::Result<()> {
    fs::write(config_path, serde_json::to_string_pretty(config)? + "\n")
        .with_context(|| format!("write {}", config_path.display()))?;
    Ok(())
}

pub(crate) fn expected_bundle_worker_name(config: &BundleConfigFile) -> Option<String> {
    config.agent.as_deref().map(|agent| {
        default_bundle_worker_name_for_project(
            config.project.as_deref(),
            agent,
            config.session.as_deref(),
        )
    })
}

pub(crate) fn parse_shell_env_value(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if !(trimmed.starts_with('\'') && trimmed.ends_with('\'')) {
        return None;
    }
    let inner = &trimmed[1..trimmed.len().saturating_sub(1)];
    Some(inner.replace("'\\''", "'"))
}

pub(crate) fn bundle_env_assignment_matches(
    path: &Path,
    prefix: &str,
    expected_value: &str,
) -> bool {
    fs::read_to_string(path)
        .ok()
        .map(|content| {
            content.lines().any(|line| {
                line.strip_prefix(prefix)
                    .map(str::trim)
                    .is_some_and(|value| {
                        value == expected_value
                            || parse_shell_env_value(value)
                                .as_deref()
                                .is_some_and(|parsed| parsed == expected_value)
                    })
            })
        })
        .unwrap_or(false)
}

pub(crate) fn bundle_worker_name_env_ready(output: &Path, config: &BundleConfigFile) -> bool {
    let Some(expected_worker_name) = expected_bundle_worker_name(config) else {
        return true;
    };
    bundle_env_assignment_matches(
        &output.join("env"),
        "MEMD_WORKER_NAME=",
        expected_worker_name.as_str(),
    ) && bundle_env_assignment_matches(
        &output.join("env.ps1"),
        "$env:MEMD_WORKER_NAME = ",
        format!("\"{}\"", escape_ps1(&expected_worker_name)).as_str(),
    )
}

pub(crate) fn repair_bundle_worker_name_env(output: &Path) -> anyhow::Result<bool> {
    let Ok((_, config)) = read_bundle_config_file(output) else {
        return Ok(false);
    };
    let Some(expected_worker_name) = expected_bundle_worker_name(&config) else {
        return Ok(false);
    };
    let shell_ready = bundle_env_assignment_matches(
        &output.join("env"),
        "MEMD_WORKER_NAME=",
        &expected_worker_name,
    );
    let ps1_ready = bundle_env_assignment_matches(
        &output.join("env.ps1"),
        "$env:MEMD_WORKER_NAME = ",
        &format!("\"{}\"", escape_ps1(&expected_worker_name)),
    );
    if shell_ready && ps1_ready {
        return Ok(false);
    }
    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_WORKER_NAME=",
        &format!(
            "MEMD_WORKER_NAME={}\n",
            shell_single_quote(&expected_worker_name)
        ),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_WORKER_NAME = ",
        &format!(
            "$env:MEMD_WORKER_NAME = \"{}\"\n",
            escape_ps1(&expected_worker_name)
        ),
    )?;
    Ok(true)
}

pub(crate) fn set_bundle_hive_system(output: &Path, hive_system: &str) -> anyhow::Result<()> {
    let (config_path, mut config) = read_bundle_config_file(output)?;
    config.hive_system = Some(hive_system.trim().to_string());
    write_bundle_config_file(&config_path, &config)?;
    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_PEER_SYSTEM=",
        &format!("MEMD_PEER_SYSTEM={}\n", hive_system.trim()),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_PEER_SYSTEM = ",
        &format!(
            "$env:MEMD_PEER_SYSTEM = \"{}\"\n",
            escape_ps1(hive_system.trim())
        ),
    )?;
    Ok(())
}

pub(crate) fn set_bundle_hive_role(output: &Path, hive_role: &str) -> anyhow::Result<()> {
    let (config_path, mut config) = read_bundle_config_file(output)?;
    config.hive_role = Some(hive_role.trim().to_string());
    write_bundle_config_file(&config_path, &config)?;
    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_PEER_ROLE=",
        &format!("MEMD_PEER_ROLE={}\n", hive_role.trim()),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_PEER_ROLE = ",
        &format!(
            "$env:MEMD_PEER_ROLE = \"{}\"\n",
            escape_ps1(hive_role.trim())
        ),
    )?;
    Ok(())
}

pub(crate) fn set_bundle_capabilities(
    output: &Path,
    capabilities: &[String],
) -> anyhow::Result<()> {
    let (config_path, mut config) = read_bundle_config_file(output)?;
    let mut normalized = capabilities
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    config.capabilities = normalized.clone();
    write_bundle_config_file(&config_path, &config)?;
    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_PEER_CAPABILITIES=",
        &format!("MEMD_PEER_CAPABILITIES={}\n", normalized.join(",")),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_PEER_CAPABILITIES = ",
        &format!(
            "$env:MEMD_PEER_CAPABILITIES = \"{}\"\n",
            escape_ps1(&normalized.join(","))
        ),
    )?;
    Ok(())
}

pub(crate) fn set_bundle_hive_groups(output: &Path, hive_groups: &[String]) -> anyhow::Result<()> {
    let (config_path, mut config) = read_bundle_config_file(output)?;
    let mut normalized = hive_groups
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    config.hive_groups = normalized.clone();
    write_bundle_config_file(&config_path, &config)?;
    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_PEER_GROUPS=",
        &format!("MEMD_PEER_GROUPS={}\n", normalized.join(",")),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_PEER_GROUPS = ",
        &format!(
            "$env:MEMD_PEER_GROUPS = \"{}\"\n",
            escape_ps1(&normalized.join(","))
        ),
    )?;
    Ok(())
}

pub(crate) fn set_bundle_hive_group_goal(output: &Path, goal: &str) -> anyhow::Result<()> {
    let (config_path, mut config) = read_bundle_config_file(output)?;
    config.hive_group_goal = Some(goal.trim().to_string());
    write_bundle_config_file(&config_path, &config)?;
    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_PEER_GROUP_GOAL=",
        &format!("MEMD_PEER_GROUP_GOAL={}\n", goal.trim()),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_PEER_GROUP_GOAL = ",
        &format!(
            "$env:MEMD_PEER_GROUP_GOAL = \"{}\"\n",
            escape_ps1(goal.trim())
        ),
    )?;
    Ok(())
}

pub(crate) fn set_bundle_authority(output: &Path, authority: &str) -> anyhow::Result<()> {
    let (config_path, mut config) = read_bundle_config_file(output)?;
    config.authority = Some(authority.trim().to_string());
    write_bundle_config_file(&config_path, &config)?;
    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_PEER_AUTHORITY=",
        &format!("MEMD_PEER_AUTHORITY={}\n", authority.trim()),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_PEER_AUTHORITY = ",
        &format!(
            "$env:MEMD_PEER_AUTHORITY = \"{}\"\n",
            escape_ps1(authority.trim())
        ),
    )?;
    Ok(())
}

pub(crate) fn set_bundle_hive_project_state(
    output: &Path,
    enabled: bool,
    anchor: Option<&str>,
    joined_at: Option<DateTime<Utc>>,
) -> anyhow::Result<()> {
    let (config_path, mut config) = read_bundle_config_file(output)?;
    config.hive_project_enabled = enabled;
    config.hive_project_anchor = anchor
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    config.hive_project_joined_at = joined_at;
    write_bundle_config_file(&config_path, &config)?;

    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_HIVE_PROJECT_ENABLED=",
        &format!(
            "MEMD_HIVE_PROJECT_ENABLED={}\n",
            if enabled { "true" } else { "false" }
        ),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_HIVE_PROJECT_ENABLED = ",
        &format!(
            "$env:MEMD_HIVE_PROJECT_ENABLED = \"{}\"\n",
            if enabled { "true" } else { "false" }
        ),
    )?;

    if let Some(anchor) = config.hive_project_anchor.as_deref() {
        rewrite_env_assignment(
            &output.join("env"),
            "MEMD_HIVE_PROJECT_ANCHOR=",
            &format!("MEMD_HIVE_PROJECT_ANCHOR={anchor}\n"),
        )?;
        rewrite_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_HIVE_PROJECT_ANCHOR = ",
            &format!(
                "$env:MEMD_HIVE_PROJECT_ANCHOR = \"{}\"\n",
                escape_ps1(anchor)
            ),
        )?;
    } else {
        remove_env_assignment(&output.join("env"), "MEMD_HIVE_PROJECT_ANCHOR=")?;
        remove_env_assignment(&output.join("env.ps1"), "$env:MEMD_HIVE_PROJECT_ANCHOR = ")?;
    }

    if let Some(joined_at) = config.hive_project_joined_at {
        let joined_at = joined_at.to_rfc3339();
        rewrite_env_assignment(
            &output.join("env"),
            "MEMD_HIVE_PROJECT_JOINED_AT=",
            &format!("MEMD_HIVE_PROJECT_JOINED_AT={joined_at}\n"),
        )?;
        rewrite_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_HIVE_PROJECT_JOINED_AT = ",
            &format!(
                "$env:MEMD_HIVE_PROJECT_JOINED_AT = \"{}\"\n",
                escape_ps1(&joined_at)
            ),
        )?;
    } else {
        remove_env_assignment(&output.join("env"), "MEMD_HIVE_PROJECT_JOINED_AT=")?;
        remove_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_HIVE_PROJECT_JOINED_AT = ",
        )?;
    }

    Ok(())
}

pub(crate) fn clear_bundle_hive_project_state(output: &Path) -> anyhow::Result<()> {
    set_bundle_hive_project_state(output, false, None, None)
}

pub(crate) fn rewrite_env_assignment(
    path: &Path,
    prefix: &str,
    replacement: &str,
) -> anyhow::Result<()> {
    let mut lines = if path.exists() {
        fs::read_to_string(path)
            .with_context(|| format!("read {}", path.display()))?
            .lines()
            .map(|line| format!("{line}\n"))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let mut replaced = false;
    for line in &mut lines {
        if line.starts_with(prefix) {
            *line = replacement.to_string();
            replaced = true;
        }
    }
    if !replaced {
        lines.push(replacement.to_string());
    }

    let mut output = String::new();
    for line in lines {
        output.push_str(&line);
    }
    fs::write(path, output).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

/// Shell-safe variant: quotes the value with single quotes before writing.
pub(crate) fn rewrite_shell_env(path: &Path, key: &str, value: &str) -> anyhow::Result<()> {
    rewrite_env_assignment(
        path,
        &format!("{key}="),
        &format!("{key}={}\n", shell_single_quote(value)),
    )
}

pub(crate) fn remove_env_assignment(path: &Path, prefix: &str) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut output = String::new();
    for line in content.lines() {
        if !line.starts_with(prefix) {
            output.push_str(line);
            output.push('\n');
        }
    }

    fs::write(path, output).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}
