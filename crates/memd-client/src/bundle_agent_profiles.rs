use super::*;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BundleAgentProfile {
    pub(crate) name: String,
    pub(crate) env_agent: String,
    pub(crate) session: Option<String>,
    pub(crate) effective_agent: String,
    pub(crate) memory_file: String,
    pub(crate) shell_entrypoint: String,
    pub(crate) powershell_entrypoint: String,
    pub(crate) launch_hint: String,
    pub(crate) native_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BundleAgentProfilesResponse {
    pub(crate) bundle_root: String,
    pub(crate) shell: String,
    pub(crate) current: Option<String>,
    pub(crate) current_session: Option<String>,
    pub(crate) selected: Option<String>,
    pub(crate) agents: Vec<BundleAgentProfile>,
}

pub(crate) fn build_bundle_agent_profiles(
    output: &Path,
    name: Option<&str>,
    shell: Option<&str>,
) -> anyhow::Result<BundleAgentProfilesResponse> {
    let runtime = read_bundle_runtime_config(output)?;
    let current = runtime.as_ref().and_then(|config| config.agent.clone());
    let current_session = runtime.as_ref().and_then(|config| config.session.clone());
    let shell = shell
        .map(|value| value.trim().to_ascii_lowercase())
        .or_else(detect_shell)
        .unwrap_or_else(|| "bash".to_string());
    let mut agents = vec![
        ("codex", "codex", "CODEX_MEMORY.md"),
        ("claude-code", "claude-code", "CLAUDE_CODE_MEMORY.md"),
        ("agent-zero", "agent-zero", "AGENT_ZERO_MEMORY.md"),
        ("hermes", "hermes", "HERMES_MEMORY.md"),
        ("opencode", "opencode", "OPENCODE_MEMORY.md"),
        ("openclaw", "openclaw", "OPENCLAW_MEMORY.md"),
    ]
    .into_iter()
    .map(|(name, env_agent, memory_file)| BundleAgentProfile {
        name: name.to_string(),
        env_agent: env_agent.to_string(),
        session: current_session.clone(),
        effective_agent: compose_agent_identity(env_agent, current_session.as_deref()),
        memory_file: output
            .join("agents")
            .join(memory_file)
            .display()
            .to_string(),
        shell_entrypoint: output
            .join("agents")
            .join(format!("{name}.sh"))
            .display()
            .to_string(),
        powershell_entrypoint: output
            .join("agents")
            .join(format!("{name}.ps1"))
            .display()
            .to_string(),
        launch_hint: String::new(),
        native_hint: None,
    })
    .collect::<Vec<_>>();

    for agent in &mut agents {
        agent.launch_hint = match shell.as_str() {
            "powershell" | "pwsh" => format!(". \"{}\"", agent.powershell_entrypoint),
            _ => format!("\"{}\"", agent.shell_entrypoint),
        };
        if agent.name == "claude-code" {
            agent.native_hint = Some(
                "import @.memd/agents/CLAUDE_IMPORTS.md into CLAUDE.md, then verify with /memory"
                    .to_string(),
            );
        }
    }

    let selected = name.map(|value| value.trim().to_ascii_lowercase());
    if let Some(selected_name) = selected.as_deref() {
        agents.retain(|agent| agent.name == selected_name);
        if agents.is_empty() {
            anyhow::bail!("unknown agent profile '{selected_name}'");
        }
    }

    Ok(BundleAgentProfilesResponse {
        bundle_root: output.display().to_string(),
        shell,
        current,
        current_session,
        selected,
        agents,
    })
}

pub(crate) fn render_bundle_agent_profiles_summary(
    response: &BundleAgentProfilesResponse,
) -> String {
    let mut output = String::new();
    let authority_warning = read_bundle_runtime_config(Path::new(&response.bundle_root))
        .ok()
        .flatten()
        .map(|runtime| authority_warning_lines(Some(&runtime)))
        .unwrap_or_default();
    output.push_str(&format!(
        "bundle={} shell={} current={} session={}\n",
        response.bundle_root,
        response.shell,
        response.current.as_deref().unwrap_or("none"),
        response.current_session.as_deref().unwrap_or("none")
    ));
    if !authority_warning.is_empty() {
        output.push_str("! authority warning:\n");
        for line in authority_warning {
            output.push_str(&format!("  - {line}\n"));
        }
    }
    for agent in &response.agents {
        output.push_str(&format!(
            "- {}{} | effective {} | memory {} | launch {}\n",
            agent.name,
            if response.current.as_deref() == Some(agent.name.as_str()) {
                " [active]"
            } else {
                ""
            },
            agent.effective_agent,
            agent.memory_file,
            agent.launch_hint
        ));
        if let Some(native_hint) = agent.native_hint.as_deref() {
            output.push_str(&format!("  native {}\n", native_hint));
        }
    }
    output.trim_end().to_string()
}

pub(crate) fn detect_shell() -> Option<String> {
    std::env::var("SHELL")
        .ok()
        .and_then(|shell| {
            let shell = shell.rsplit('/').next()?.to_string();
            Some(shell)
        })
        .or_else(|| {
            std::env::var("PSModulePath")
                .ok()
                .map(|_| "powershell".to_string())
        })
}

pub(crate) fn copy_hook_assets(target: &Path) -> anyhow::Result<()> {
    let source_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("integrations")
        .join("hooks");

    for file in [
        "README.md",
        "install.sh",
        "install.ps1",
        "memd-context.sh",
        "memd-context.ps1",
        "memd-capture.sh",
        "memd-capture.ps1",
        "memd-spill.sh",
        "memd-spill.ps1",
    ] {
        let src = source_dir.join(file);
        let dst = target.join(file);
        fs::copy(&src, &dst)
            .with_context(|| format!("copy {} to {}", src.display(), dst.display()))?;
        set_executable_if_shell_script(&dst, file)?;
    }

    Ok(())
}
