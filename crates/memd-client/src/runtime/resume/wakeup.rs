use super::*;

pub(crate) fn collect_wakeup_instruction_sources(output: &Path) -> Vec<(String, String)> {
    let Some(project_root) = infer_bundle_project_root(output) else {
        return Vec::new();
    };

    let mut sources = Vec::new();
    for relative in [
        "AGENTS.md",
        "CLAUDE.md",
        ".claude/CLAUDE.md",
        ".agents/CLAUDE.md",
        "TEAMS.md",
    ] {
        let path = project_root.join(relative);
        if let Some((snippet, _)) = read_bootstrap_source(&path, 18) {
            sources.push((relative.to_string(), snippet));
        }
    }
    sources
}

pub(crate) fn render_bundle_wakeup_markdown(
    output: &Path,
    snapshot: &ResumeSnapshot,
    verbose: bool,
) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd wake-up\n\n");
    markdown.push_str(&format!(
        "- {} / {} / {} / {} / {} / {} / {}\n\n",
        snapshot.project.as_deref().unwrap_or("none"),
        snapshot.namespace.as_deref().unwrap_or("none"),
        snapshot.agent.as_deref().unwrap_or("none"),
        snapshot.workspace.as_deref().unwrap_or("none"),
        snapshot.visibility.as_deref().unwrap_or("all"),
        snapshot.route,
        snapshot.intent,
    ));

    let instructions = collect_wakeup_instruction_sources(output);
    if verbose && !instructions.is_empty() {
        markdown.push_str("## Instructions\n\n");
        let limit = if verbose { 2 } else { 1 };
        for (source, snippet) in instructions.into_iter().take(limit) {
            markdown.push_str(&format!("- {source}: {}\n", compact_inline(&snippet, 240)));
        }
        markdown.push('\n');
    }

    let event_spine = snapshot.event_spine();
    if !event_spine.is_empty() {
        markdown.push_str("## Live\n\n");
        let limit = if verbose { 4 } else { 1 };
        for item in event_spine.iter().take(limit) {
            markdown.push_str(&format!("- {}\n", compact_inline(item, 120)));
        }
        markdown.push('\n');
    }

    markdown.push_str("## Focus\n\n");
    if snapshot.working.records.is_empty() {
        markdown.push_str("- none\n");
    } else {
        let limit = 1;
        for item in snapshot.working.records.iter().take(limit) {
            markdown.push_str(&format!("- {}\n", compact_inline(item.record.trim(), 140)));
        }
    }
    markdown.push('\n');

    if verbose
        && (!snapshot.inbox.items.is_empty() || !snapshot.working.rehydration_queue.is_empty())
    {
        markdown.push_str("## Recovery\n\n");
        let recovery_limit = if verbose { 1 } else { 1 };
        for item in snapshot
            .working
            .rehydration_queue
            .iter()
            .take(recovery_limit)
        {
            markdown.push_str(&format!(
                "- {}: {}\n",
                item.label,
                compact_inline(item.summary.trim(), 120)
            ));
        }
        let inbox_limit = if verbose { 1 } else { 1 };
        for item in snapshot.inbox.items.iter().take(inbox_limit) {
            markdown.push_str(&format!(
                "- {:?}/{:?}: {}\n",
                item.item.kind,
                item.item.status,
                compact_inline(item.item.content.trim(), 120)
            ));
        }
        markdown.push('\n');
    }

    markdown.push_str("## Protocol\n\n");
    markdown.push_str("- Read first.\n");
    markdown.push_str("- Lookup before answers on decisions, preferences, or history.\n");
    markdown.push_str("- Recall: `memd lookup --output .memd --query \"...\"`.\n");
    markdown.push_str("- Writes: `remember-short`, `remember-decision`, `remember-preference`, `remember-long`, `capture-live`, `correct-memory`, `sync-semantic`, `watch`.\n");
    if verbose {
        markdown
            .push_str("- Wake/resume/refresh/handoff/hook capture auto-write short-term status.\n");
    }
    markdown.push_str("- Promote stable truths; do not rely on transcript recall.\n");
    markdown.push_str(&format!(
        "- Default voice: {}\n",
        read_bundle_voice_mode(output).unwrap_or_else(default_voice_mode)
    ));

    markdown
}

pub(crate) fn render_bundle_wakeup_summary(snapshot: &ResumeSnapshot) -> String {
    format!(
        "wake project={} namespace={} agent={} working={} inbox={} spine={} focus=\"{}\"",
        snapshot.project.as_deref().unwrap_or("none"),
        snapshot.namespace.as_deref().unwrap_or("none"),
        snapshot.agent.as_deref().unwrap_or("none"),
        snapshot.working.records.len(),
        snapshot.inbox.items.len(),
        snapshot.event_spine().len(),
        snapshot
            .working
            .records
            .first()
            .map(|item| compact_inline(item.record.trim(), 96))
            .unwrap_or_else(|| "none".to_string())
    )
}

pub(crate) fn render_bundle_scope_markdown(output: &Path, snapshot: &ResumeSnapshot) -> String {
    let runtime = read_bundle_runtime_config(output).ok().flatten();
    let heartbeat_tab_id = read_bundle_heartbeat(output)
        .ok()
        .flatten()
        .and_then(|state| state.tab_id)
        .filter(|value| !value.trim().is_empty());
    let session = runtime
        .as_ref()
        .and_then(|config| config.session.as_deref())
        .filter(|value| !value.trim().is_empty());
    let tab_id = runtime
        .as_ref()
        .and_then(|config| config.tab_id.as_deref())
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .or(heartbeat_tab_id)
        .or_else(default_bundle_tab_id);
    let effective_agent = runtime
        .as_ref()
        .and_then(|config| config.agent.as_deref())
        .map(|agent| compose_agent_identity(agent, session));

    format!(
        "## Scope\n\n- project: `{}`\n- namespace: `{}`\n- agent: `{}`\n- session: `{}`\n- tab: `{}`\n- effective agent: `{}`\n- workspace: `{}`\n- visibility: `{}`\n- route: `{}`\n- intent: `{}`\n- bundle: `{}`\n",
        snapshot.project.as_deref().unwrap_or("none"),
        snapshot.namespace.as_deref().unwrap_or("none"),
        snapshot.agent.as_deref().unwrap_or("none"),
        session.unwrap_or("none"),
        tab_id.as_deref().unwrap_or("none"),
        effective_agent.as_deref().unwrap_or("none"),
        snapshot.workspace.as_deref().unwrap_or("none"),
        snapshot.visibility.as_deref().unwrap_or("all"),
        snapshot.route,
        snapshot.intent,
        output.display(),
    )
}

pub(crate) fn render_memory_page_header_suffix(output: &Path) -> String {
    let heartbeat_tab_id = read_bundle_heartbeat(output)
        .ok()
        .flatten()
        .and_then(|state| state.tab_id)
        .filter(|value| !value.trim().is_empty());
    let tab_id = read_bundle_runtime_config(output)
        .ok()
        .flatten()
        .and_then(|config| config.tab_id)
        .filter(|value| !value.trim().is_empty())
        .or(heartbeat_tab_id)
        .or_else(default_bundle_tab_id)
        .unwrap_or_else(|| "none".to_string());
    format!(" [tab={}]", tab_id)
}
