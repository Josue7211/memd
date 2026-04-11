use super::*;

pub(crate) fn render_bundle_eval_markdown(response: &BundleEvalResponse) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd bundle evaluation\n\n");
    markdown.push_str(&format!(
        "- bundle: {}\n- status: {}\n- score: {}\n- baseline_score: {}\n- score_delta: {}\n- agent: {}\n- workspace: {}\n- visibility: {}\n",
        response.bundle_root,
        response.status,
        response.score,
        response
            .baseline_score
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        response
            .score_delta
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        response.agent.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("none"),
    ));
    markdown.push_str(&format!(
        "- working_records: {}\n- context_records: {}\n- rehydration_items: {}\n- inbox_items: {}\n- workspace_lanes: {}\n- semantic_hits: {}\n",
        response.working_records,
        response.context_records,
        response.rehydration_items,
        response.inbox_items,
        response.workspace_lanes,
        response.semantic_hits,
    ));

    markdown.push_str("\n## Findings\n\n");
    if response.findings.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for finding in &response.findings {
            markdown.push_str(&format!("- {}\n", finding));
        }
    }

    markdown.push_str("\n## Changes\n\n");
    if response.changes.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for change in &response.changes {
            markdown.push_str(&format!("- {}\n", change));
        }
    }

    markdown.push_str("\n## Recommendations\n\n");
    if response.recommendations.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for recommendation in &response.recommendations {
            markdown.push_str(&format!("- {}\n", recommendation));
        }
    }

    markdown
}

pub(crate) fn render_agent_shell_profile(output: &Path, env_agent: Option<&str>) -> String {
    let project_hive_enabled = read_bundle_runtime_config(output)
        .ok()
        .flatten()
        .map(|runtime| runtime.hive_project_enabled)
        .unwrap_or(false);
    let authority_warning = read_bundle_runtime_config(output)
        .ok()
        .flatten()
        .map(|runtime| authority_warning_lines(Some(&runtime)))
        .unwrap_or_default();
    let mut script = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\n\nexport MEMD_BUNDLE_ROOT=\"{}\"\nsource \"$MEMD_BUNDLE_ROOT/backend.env\" 2>/dev/null || true\nsource \"$MEMD_BUNDLE_ROOT/env\"\n",
        compact_bundle_value(output.to_string_lossy().as_ref()),
    );
    let bundle_config = read_bundle_config_file(output).ok().map(|(_, config)| config);
    let bundle_session = bundle_config.as_ref().and_then(|config| config.session.clone());
    let bundle_project = bundle_config.as_ref().and_then(|config| config.project.as_deref());
    if project_hive_enabled {
        script.push_str(&format!(
            "if [[ -z \"${{MEMD_BASE_URL:-}}\" || \"${{MEMD_BASE_URL}}\" =~ ^https?://(localhost|127\\.0\\.0\\.1|0\\.0\\.0\\.0)(:[0-9]+)?(/|$) ]]; then\n  export MEMD_BASE_URL=\"{}\"\nfi\n",
            SHARED_MEMD_BASE_URL
        ));
    }
    script.push_str(
        "if [[ -z \"${MEMD_TAB_ID:-}\" ]]; then\n  if [[ -n \"${WT_SESSION:-}\" ]]; then\n    export MEMD_TAB_ID=\"tab-${WT_SESSION:0:8}\"\n  elif [[ -n \"${TERM_SESSION_ID:-}\" ]]; then\n    export MEMD_TAB_ID=\"tab-${TERM_SESSION_ID:0:8}\"\n  else\n    tty_id=\"$(tty 2>/dev/null || true)\"\n    if [[ -n \"$tty_id\" && \"$tty_id\" != \"not a tty\" ]]; then\n      export MEMD_TAB_ID=\"tab-${tty_id//\\//-}\"\n    else\n      export MEMD_TAB_ID=\"tab-$$\"\n    fi\n  fi\nfi\n",
    );
    if !authority_warning.is_empty() {
        script.push_str("printf '%s\\n' 'memd authority warning:' >&2\n");
        for line in authority_warning {
            script.push_str(&format!(
                "printf '%s\\n' {} >&2\n",
                shell_single_quote(&compact_bundle_value(&line))
            ));
        }
    }
    if let Some(env_agent) = env_agent {
        script.push_str(&format!(
            "export MEMD_AGENT=\"{}\"\n",
            compact_bundle_value(env_agent)
        ));
        script.push_str(&format!(
            "export MEMD_WORKER_NAME=\"{}\"\n",
            compact_bundle_value(&default_bundle_worker_name_for_project(
                bundle_project,
                env_agent,
                bundle_session.as_deref()
            ))
        ));
        if env_agent == "codex" {
            script.push_str(
                "if [[ -x \"$MEMD_BUNDLE_ROOT/agents/watch.sh\" ]]; then\n  nohup \"$MEMD_BUNDLE_ROOT/agents/watch.sh\" >/tmp/memd-watch.log 2>&1 &\nfi\n",
            );
        }
    }
    script.push_str(
        "memd wake --output \"$MEMD_BUNDLE_ROOT\" --intent current_task --write >/dev/null 2>&1 || true\n",
    );
    script.push_str(
        "nohup memd heartbeat --output \"$MEMD_BUNDLE_ROOT\" --watch --interval-secs 30 --probe-base-url >/tmp/memd-heartbeat.log 2>&1 &\n",
    );
    script.push_str(
        "memd hive --output \"$MEMD_BUNDLE_ROOT\" --publish-heartbeat --summary >/dev/null 2>&1 || true\n",
    );
    script.push_str(
        "exec memd wake --output \"$MEMD_BUNDLE_ROOT\" --intent current_task --write \"$@\"\n",
    );
    script
}

pub(crate) fn render_agent_ps1_profile(output: &Path, env_agent: Option<&str>) -> String {
    let project_hive_enabled = read_bundle_runtime_config(output)
        .ok()
        .flatten()
        .map(|runtime| runtime.hive_project_enabled)
        .unwrap_or(false);
    let authority_warning = read_bundle_runtime_config(output)
        .ok()
        .flatten()
        .map(|runtime| authority_warning_lines(Some(&runtime)))
        .unwrap_or_default();
    let mut script = format!(
        "$env:MEMD_BUNDLE_ROOT = \"{}\"\n$bundleBackendEnv = Join-Path $env:MEMD_BUNDLE_ROOT \"backend.env.ps1\"\nif (Test-Path $bundleBackendEnv) {{ . $bundleBackendEnv }}\n. (Join-Path $env:MEMD_BUNDLE_ROOT \"env.ps1\")\n",
        escape_ps1(output.to_string_lossy().as_ref()),
    );
    let bundle_config = read_bundle_config_file(output).ok().map(|(_, config)| config);
    let bundle_session = bundle_config.as_ref().and_then(|config| config.session.clone());
    let bundle_project = bundle_config.as_ref().and_then(|config| config.project.as_deref());
    if project_hive_enabled {
        script.push_str(&format!(
            "if ([string]::IsNullOrWhiteSpace($env:MEMD_BASE_URL) -or $env:MEMD_BASE_URL -match '^(https?://)?(localhost|127\\.0\\.0\\.1|0\\.0\\.0\\.0)(:[0-9]+)?(/|$)') {{ $env:MEMD_BASE_URL = \"{}\" }}\n",
            escape_ps1(SHARED_MEMD_BASE_URL)
        ));
    }
    script.push_str(
        "if (-not $env:MEMD_TAB_ID) {\n  if ($env:WT_SESSION) {\n    $env:MEMD_TAB_ID = \"tab-{0}\" -f $env:WT_SESSION.Substring(0, [Math]::Min(8, $env:WT_SESSION.Length))\n  } elseif ($env:TERM_SESSION_ID) {\n    $env:MEMD_TAB_ID = \"tab-{0}\" -f $env:TERM_SESSION_ID.Substring(0, [Math]::Min(8, $env:TERM_SESSION_ID.Length))\n  } else {\n    $env:MEMD_TAB_ID = \"tab-{0}\" -f $PID\n  }\n}\n",
    );
    if !authority_warning.is_empty() {
        script.push_str("Write-Host \"memd authority warning:\" -ForegroundColor Yellow\n");
        for line in authority_warning {
            script.push_str(&format!(
                "Write-Host \"{}\" -ForegroundColor Yellow\n",
                escape_ps1(&line)
            ));
        }
    }
    if let Some(env_agent) = env_agent {
        script.push_str(&format!(
            "$env:MEMD_AGENT = \"{}\"\n",
            escape_ps1(env_agent)
        ));
        script.push_str(&format!(
            "$env:MEMD_WORKER_NAME = \"{}\"\n",
            escape_ps1(&default_bundle_worker_name_for_project(
                bundle_project,
                env_agent,
                bundle_session.as_deref()
            ))
        ));
        if env_agent == "codex" {
            script.push_str(
                "if (Test-Path (Join-Path $env:MEMD_BUNDLE_ROOT \"agents/watch.sh\")) { Start-Process -WindowStyle Hidden -FilePath (Join-Path $env:MEMD_BUNDLE_ROOT \"agents/watch.sh\") -RedirectStandardOutput \"$env:TEMP\\memd-watch.log\" -RedirectStandardError \"$env:TEMP\\memd-watch.err\" }\n",
            );
        }
    }
    script.push_str(
        "try { memd wake --output $env:MEMD_BUNDLE_ROOT --intent current_task --write | Out-Null } catch { }\n",
    );
    script.push_str(
        "Start-Process -WindowStyle Hidden -FilePath memd -ArgumentList @('heartbeat','--output',$env:MEMD_BUNDLE_ROOT,'--watch','--interval-secs','30','--probe-base-url') -RedirectStandardOutput \"$env:TEMP\\memd-heartbeat.log\" -RedirectStandardError \"$env:TEMP\\memd-heartbeat.err\"\n",
    );
    script.push_str(
        "try { memd hive --output $env:MEMD_BUNDLE_ROOT --publish-heartbeat --summary | Out-Null } catch { }\n",
    );
    script.push_str("memd wake --output $env:MEMD_BUNDLE_ROOT --intent current_task --write\n");
    script
}

pub(crate) fn render_lookup_shell_profile(output: &Path, kinds: &[&str], tags: &[&str]) -> String {
    let mut script = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\n\nexport MEMD_BUNDLE_ROOT=\"{}\"\nsource \"$MEMD_BUNDLE_ROOT/backend.env\" 2>/dev/null || true\nsource \"$MEMD_BUNDLE_ROOT/env\"\n\nargs=(lookup --output \"$MEMD_BUNDLE_ROOT\" --route project_first --intent general)\n",
        compact_bundle_value(output.to_string_lossy().as_ref()),
    );
    for kind in kinds {
        script.push_str(&format!(
            "args+=(--kind \"{}\")\n",
            compact_bundle_value(kind)
        ));
    }
    for tag in tags {
        script.push_str(&format!(
            "args+=(--tag \"{}\")\n",
            compact_bundle_value(tag)
        ));
    }
    script.push_str("exec memd \"${args[@]}\" \"$@\"\n");
    script
}

pub(crate) fn render_lookup_ps1_profile(output: &Path, kinds: &[&str], tags: &[&str]) -> String {
    let mut script = format!(
        "$env:MEMD_BUNDLE_ROOT = \"{}\"\n$bundleBackendEnv = Join-Path $env:MEMD_BUNDLE_ROOT \"backend.env.ps1\"\nif (Test-Path $bundleBackendEnv) {{ . $bundleBackendEnv }}\n. (Join-Path $env:MEMD_BUNDLE_ROOT \"env.ps1\")\n$args = @(\"lookup\", \"--output\", $env:MEMD_BUNDLE_ROOT, \"--route\", \"project_first\", \"--intent\", \"general\")\n",
        escape_ps1(output.to_string_lossy().as_ref()),
    );
    for kind in kinds {
        script.push_str(&format!(
            "$args += @(\"--kind\", \"{}\")\n",
            escape_ps1(kind)
        ));
    }
    for tag in tags {
        script.push_str(&format!("$args += @(\"--tag\", \"{}\")\n", escape_ps1(tag)));
    }
    script.push_str("memd @args @Args\n");
    script
}

pub(crate) fn render_remember_shell_profile(output: &Path, kind: &str, tags: &[&str]) -> String {
    let mut script = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\n\nexport MEMD_BUNDLE_ROOT=\"{}\"\nsource \"$MEMD_BUNDLE_ROOT/backend.env\" 2>/dev/null || true\nsource \"$MEMD_BUNDLE_ROOT/env\"\n\nargs=(remember --output \"$MEMD_BUNDLE_ROOT\" --kind \"{}\" --scope project)\n",
        compact_bundle_value(output.to_string_lossy().as_ref()),
        compact_bundle_value(kind),
    );
    for tag in tags {
        script.push_str(&format!(
            "args+=(--tag \"{}\")\n",
            compact_bundle_value(tag)
        ));
    }
    script.push_str("exec memd \"${args[@]}\" \"$@\"\n");
    script
}

pub(crate) fn render_remember_ps1_profile(output: &Path, kind: &str, tags: &[&str]) -> String {
    let mut script = format!(
        "$env:MEMD_BUNDLE_ROOT = \"{}\"\n$bundleBackendEnv = Join-Path $env:MEMD_BUNDLE_ROOT \"backend.env.ps1\"\nif (Test-Path $bundleBackendEnv) {{ . $bundleBackendEnv }}\n. (Join-Path $env:MEMD_BUNDLE_ROOT \"env.ps1\")\n$args = @(\"remember\", \"--output\", $env:MEMD_BUNDLE_ROOT, \"--kind\", \"{}\", \"--scope\", \"project\")\n",
        escape_ps1(output.to_string_lossy().as_ref()),
        escape_ps1(kind),
    );
    for tag in tags {
        script.push_str(&format!("$args += @(\"--tag\", \"{}\")\n", escape_ps1(tag)));
    }
    script.push_str("memd @args @Args\n");
    script
}

pub(crate) fn render_capture_shell_profile(output: &Path, mode: &str) -> String {
    let mut script = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\n\nexport MEMD_BUNDLE_ROOT=\"{}\"\nsource \"$MEMD_BUNDLE_ROOT/backend.env\" 2>/dev/null || true\nsource \"$MEMD_BUNDLE_ROOT/env\"\n\nargs=(hook capture --output \"$MEMD_BUNDLE_ROOT\" --summary)\n",
        compact_bundle_value(output.to_string_lossy().as_ref()),
    );
    if mode == "capture-live" {
        script.push_str("args+=(--tag basic-memory --tag live-capture)\n");
    } else {
        script.push_str("args+=(--tag basic-memory --tag correction)\n");
    }
    script.push_str("exec memd \"${args[@]}\" \"$@\"\n");
    script
}

pub(crate) fn render_capture_ps1_profile(output: &Path, mode: &str) -> String {
    let mut script = format!(
        "$env:MEMD_BUNDLE_ROOT = \"{}\"\n$bundleBackendEnv = Join-Path $env:MEMD_BUNDLE_ROOT \"backend.env.ps1\"\nif (Test-Path $bundleBackendEnv) {{ . $bundleBackendEnv }}\n. (Join-Path $env:MEMD_BUNDLE_ROOT \"env.ps1\")\n$args = @(\"hook\", \"capture\", \"--output\", $env:MEMD_BUNDLE_ROOT, \"--summary\")\n",
        escape_ps1(output.to_string_lossy().as_ref()),
    );
    if mode == "capture-live" {
        script.push_str("$args += @(\"--tag\", \"basic-memory\", \"--tag\", \"live-capture\")\n");
    } else {
        script.push_str("$args += @(\"--tag\", \"basic-memory\", \"--tag\", \"correction\")\n");
    }
    script.push_str("memd @args @Args\n");
    script
}

pub(crate) fn render_checkpoint_shell_profile(output: &Path) -> String {
    format!(
        "#!/usr/bin/env bash\nset -euo pipefail\n\nexport MEMD_BUNDLE_ROOT=\"{}\"\nsource \"$MEMD_BUNDLE_ROOT/backend.env\" 2>/dev/null || true\nsource \"$MEMD_BUNDLE_ROOT/env\"\n\nargs=(checkpoint --output \"$MEMD_BUNDLE_ROOT\" --tag basic-memory --tag short-term)\nexec memd \"${{args[@]}}\" \"$@\"\n",
        compact_bundle_value(output.to_string_lossy().as_ref()),
    )
}

pub(crate) fn render_checkpoint_ps1_profile(output: &Path) -> String {
    format!(
        "$env:MEMD_BUNDLE_ROOT = \"{}\"\n$bundleBackendEnv = Join-Path $env:MEMD_BUNDLE_ROOT \"backend.env.ps1\"\nif (Test-Path $bundleBackendEnv) {{ . $bundleBackendEnv }}\n. (Join-Path $env:MEMD_BUNDLE_ROOT \"env.ps1\")\n$args = @(\"checkpoint\", \"--output\", $env:MEMD_BUNDLE_ROOT, \"--tag\", \"basic-memory\", \"--tag\", \"short-term\")\nmemd @args @Args\n",
        escape_ps1(output.to_string_lossy().as_ref()),
    )
}

pub(crate) fn render_rag_sync_shell_profile(output: &Path) -> String {
    format!(
        "#!/usr/bin/env bash\nset -euo pipefail\n\nexport MEMD_BUNDLE_ROOT=\"{}\"\nsource \"$MEMD_BUNDLE_ROOT/backend.env\" 2>/dev/null || true\nsource \"$MEMD_BUNDLE_ROOT/env\"\n\nargs=(rag sync)\n[[ -n \"${{MEMD_PROJECT:-}}\" ]] && args+=(--project \"$MEMD_PROJECT\")\n[[ -n \"${{MEMD_NAMESPACE:-}}\" ]] && args+=(--namespace \"$MEMD_NAMESPACE\")\nexec memd \"${{args[@]}}\" \"$@\"\n",
        compact_bundle_value(output.to_string_lossy().as_ref()),
    )
}

pub(crate) fn render_rag_sync_ps1_profile(output: &Path) -> String {
    format!(
        "$env:MEMD_BUNDLE_ROOT = \"{}\"\n$bundleBackendEnv = Join-Path $env:MEMD_BUNDLE_ROOT \"backend.env.ps1\"\nif (Test-Path $bundleBackendEnv) {{ . $bundleBackendEnv }}\n. (Join-Path $env:MEMD_BUNDLE_ROOT \"env.ps1\")\n$args = @(\"rag\", \"sync\")\nif ($env:MEMD_PROJECT) {{ $args += @(\"--project\", $env:MEMD_PROJECT) }}\nif ($env:MEMD_NAMESPACE) {{ $args += @(\"--namespace\", $env:MEMD_NAMESPACE) }}\nmemd @args @Args\n",
        escape_ps1(output.to_string_lossy().as_ref()),
    )
}

pub(crate) fn render_watch_shell_profile(output: &Path) -> String {
    format!(
        "#!/usr/bin/env bash\nset -euo pipefail\n\nexport MEMD_BUNDLE_ROOT=\"{}\"\nsource \"$MEMD_BUNDLE_ROOT/backend.env\" 2>/dev/null || true\nsource \"$MEMD_BUNDLE_ROOT/env\"\nproject_root=\"$(cd \"$MEMD_BUNDLE_ROOT/..\" && pwd)\"\nexec memd watch --root \"$project_root\" --output \"$MEMD_BUNDLE_ROOT\" \"$@\"\n",
        compact_bundle_value(output.to_string_lossy().as_ref()),
    )
}

pub(crate) fn render_watch_ps1_profile(output: &Path) -> String {
    format!(
        "$env:MEMD_BUNDLE_ROOT = \"{}\"\n$bundleBackendEnv = Join-Path $env:MEMD_BUNDLE_ROOT \"backend.env.ps1\"\nif (Test-Path $bundleBackendEnv) {{ . $bundleBackendEnv }}\n. (Join-Path $env:MEMD_BUNDLE_ROOT \"env.ps1\")\n$projectRoot = Split-Path -Parent $env:MEMD_BUNDLE_ROOT\nmemd watch --root $projectRoot --output $env:MEMD_BUNDLE_ROOT @Args\n",
        escape_ps1(output.to_string_lossy().as_ref()),
    )
}

pub(crate) fn collect_wakeup_instruction_sources(output: &Path) -> Vec<(String, String)> {
    let mut sources = Vec::new();
    let Some(project_root) = infer_bundle_project_root(output) else {
        return sources;
    };
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

pub(crate) fn render_bundle_memory_markdown(
    output: &Path,
    snapshot: &ResumeSnapshot,
    handoff: Option<&HandoffSnapshot>,
    hive: Option<&BundleHiveMemorySurface>,
) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# memd memory{}\n\n",
        render_memory_page_header_suffix(output)
    ));
    markdown.push_str(&render_bundle_scope_markdown(output, snapshot));
    markdown.push('\n');

    markdown.push_str("\n## Budget\n\n");
    markdown.push_str(&format!(
        "- tok={} | ch={} | p={} | dup={} | use={}/{} | refresh={} | action=\"{}\"\n",
        snapshot.estimated_prompt_tokens(),
        snapshot.estimated_prompt_chars(),
        snapshot.context_pressure(),
        snapshot.redundant_context_items(),
        snapshot.working.used_chars,
        snapshot.working.budget_chars,
        snapshot.refresh_recommended,
        snapshot.memory_action_hint(),
    ));
    let drivers = snapshot.memory_pressure_drivers();
    markdown.push_str(&format!(
        "- drivers={}\n",
        if drivers.is_empty() {
            "none".to_string()
        } else {
            drivers.join(",")
        }
    ));

    let current_task = render_current_task_bundle_snapshot(snapshot);
    if !current_task.is_empty() {
        markdown.push_str("\n## Read First\n\n");
        markdown.push_str(&current_task);
        if let Some(focus) = snapshot.working.records.first() {
            markdown.push_str(&format!(
                "- focus={}\n",
                compact_inline(focus.record.trim(), 120)
            ));
        }
        if let Some(next) = snapshot.working.rehydration_queue.first() {
            markdown.push_str(&format!(
                "- next={}: {}\n",
                next.label,
                compact_inline(next.summary.trim(), 120)
            ));
        }
        if let Some(blocker) = snapshot.inbox.items.first() {
            markdown.push_str(&format!(
                "- blocker={:?}/{:?}: {}\n",
                blocker.item.kind,
                blocker.item.status,
                compact_inline(blocker.item.content.trim(), 120)
            ));
        }
    }

    markdown.push_str("\n## Voice\n\n");
    markdown.push_str(&render_voice_mode_section(
        &read_bundle_voice_mode(output).unwrap_or_else(default_voice_mode),
    ));
    markdown.push('\n');

    markdown.push_str("\n## Memory Objects\n\n");
    if let Some(record) = snapshot.context.records.first() {
        markdown.push_str(&format!(
            "- context id={} record=\"{}\"\n",
            short_uuid(record.id),
            compact_inline(record.record.trim(), 120)
        ));
        if let Some(slug) = memory_object_item_slug(snapshot, MemoryObjectLane::Context, 0) {
            markdown.push_str(&format!("- [open](items/context/{slug})\n"));
        }
    } else {
        markdown.push_str("- context none\n");
    }
    if let Some(record) = snapshot.working.records.first() {
        markdown.push_str(&format!(
            "- working id={} record=\"{}\"\n",
            short_uuid(record.id),
            compact_inline(record.record.trim(), 120)
        ));
        if let Some(slug) = memory_object_item_slug(snapshot, MemoryObjectLane::Working, 0) {
            markdown.push_str(&format!("- [open](items/working/{slug})\n"));
        }
    } else {
        markdown.push_str("- working none\n");
    }
    if let Some(item) = snapshot.inbox.items.first() {
        markdown.push_str(&format!(
            "- inbox id={} kind={} status={} stage={} cf={:.2} scope={} source={} note=\"{}\"\n",
            short_uuid(item.item.id),
            enum_label_kind(item.item.kind),
            enum_label_status(item.item.status),
            format!("{:?}", item.item.stage).to_ascii_lowercase(),
            item.item.confidence,
            format!("{:?}", item.item.scope).to_ascii_lowercase(),
            ResumeSnapshot::source_label(
                item.item.source_agent.as_deref(),
                item.item.source_system.as_deref(),
                item.item.source_path.as_deref()
            ),
            compact_inline(item.item.content.trim(), 120)
        ));
        if let Some(slug) = memory_object_item_slug(snapshot, MemoryObjectLane::Inbox, 0) {
            markdown.push_str(&format!("- [open](items/inbox/{slug})\n"));
        }
        if !item.reasons.is_empty() {
            markdown.push_str(&format!(
                "- inbox_reasons={}\n",
                item.reasons
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    } else {
        markdown.push_str("- inbox none\n");
    }
    if let Some(artifact) = snapshot.working.rehydration_queue.first() {
        markdown.push_str(&format!(
            "- recovery id={} kind={} label=\"{}\" source={} reason=\"{}\"\n",
            artifact
                .id
                .map(short_uuid)
                .unwrap_or_else(|| "none".to_string()),
            artifact.kind,
            compact_inline(&artifact.label, 64),
            ResumeSnapshot::source_label(
                artifact.source_agent.as_deref(),
                artifact.source_system.as_deref(),
                artifact.source_path.as_deref()
            ),
            artifact
                .reason
                .as_deref()
                .map(|value| compact_inline(value, 120))
                .unwrap_or_else(|| "none".to_string())
        ));
        if let Some(slug) = memory_object_item_slug(snapshot, MemoryObjectLane::Recovery, 0) {
            markdown.push_str(&format!("- [open](items/recovery/{slug})\n"));
        }
    } else {
        markdown.push_str("- recovery none\n");
    }
    if let Some(semantic) = snapshot
        .semantic
        .as_ref()
        .filter(|semantic| !semantic.items.is_empty())
        .and_then(|semantic| semantic.items.first())
    {
        markdown.push_str(&format!(
            "- semantic score={:.2} content=\"{}\"\n",
            semantic.score,
            compact_inline(&semantic.content, 120)
        ));
        if let Some(slug) = memory_object_item_slug(snapshot, MemoryObjectLane::Semantic, 0) {
            markdown.push_str(&format!("- [open](items/semantic/{slug})\n"));
        }
    } else {
        markdown.push_str("- semantic none\n");
    }
    if let Some(first) = snapshot.workspaces.workspaces.first() {
        markdown.push_str(&format!(
            "- workspace project={} namespace={} workspace={} visibility={} items={} active={} contested={} trust={:.2} cf={:.2}\n",
            first.project.as_deref().unwrap_or("none"),
            first.namespace.as_deref().unwrap_or("none"),
            first.workspace.as_deref().unwrap_or("none"),
            memory_visibility_label(first.visibility),
            first.item_count,
            first.active_count,
            first.contested_count,
            first.trust_score,
            first.avg_confidence
        ));
        if let Some(slug) = memory_object_item_slug(snapshot, MemoryObjectLane::Workspace, 0) {
            markdown.push_str(&format!("- [open](items/workspace/{slug})\n"));
        }
    } else {
        markdown.push_str("- workspace none\n");
    }

    let event_spine = snapshot.event_spine();
    if !event_spine.is_empty() || !snapshot.recent_repo_changes.is_empty() {
        markdown.push_str("\n## E+LT\n\n");
        let event_part = if event_spine.is_empty() {
            None
        } else {
            let summary = event_spine
                .iter()
                .take(2)
                .map(|change| change.trim())
                .collect::<Vec<_>>()
                .join(" | ");
            Some(format!("- E={summary}"))
        };
        let lt_part = if snapshot.recent_repo_changes.is_empty() {
            None
        } else {
            let summary = snapshot
                .recent_repo_changes
                .iter()
                .take(2)
                .map(|change| change.trim())
                .collect::<Vec<_>>()
                .join(" | ");
            Some(format!("- LT={summary}"))
        };
        let mut parts = Vec::new();
        if let Some(part) = event_part {
            parts.push(part);
        }
        if let Some(part) = lt_part {
            parts.push(part);
        }
        markdown.push_str(&format!("- {}\n", parts.join(" | ")));
    }

    markdown.push_str("\n## W\n\n");
    if snapshot.working.records.is_empty() {
        markdown.push_str("- none\n");
    } else {
        let records = snapshot
            .working
            .records
            .iter()
            .take(2)
            .map(|record| record.record.trim())
            .collect::<Vec<_>>();
        markdown.push_str(&format!("- w={}", records.join(" | ")));
        if snapshot.working.records.len() > 2 {
            markdown.push_str(&format!(" (+{} more)", snapshot.working.records.len() - 2));
        }
        markdown.push('\n');
    }

    let mut ri_parts = Vec::new();
    if !snapshot.working.rehydration_queue.is_empty() {
        for artifact in snapshot.working.rehydration_queue.iter().take(6) {
            ri_parts.push(format!("r={}:{}", artifact.label, artifact.summary.trim()));
        }
    }
    if !snapshot.inbox.items.is_empty() {
        for item in snapshot.inbox.items.iter().take(6) {
            ri_parts.push(format!(
                "i={:?}/{:?}:{}",
                item.item.kind,
                item.item.status,
                item.item.content.trim()
            ));
            if !item.reasons.is_empty() {
                ri_parts.push(format!("r={}", item.reasons.join(", ")));
            }
        }
    }
    if !ri_parts.is_empty() {
        markdown.push_str("\n## RI\n\n");
        markdown.push_str(&format!("- {}\n", ri_parts.join(" | ")));
    }

    if let Some(first) = snapshot.workspaces.workspaces.first() {
        markdown.push_str("\n## L\n\n");
        let extras = snapshot.workspaces.workspaces.len() - 1;
        markdown.push_str(&format!(
            "- l={}/{}/{} | v={} | it={} | tr={:.2}{} \n",
            first.project.as_deref().unwrap_or("none"),
            first.namespace.as_deref().unwrap_or("none"),
            first.workspace.as_deref().unwrap_or("none"),
            memory_visibility_label(first.visibility),
            first.item_count,
            first.trust_score,
            if extras > 0 {
                format!(" (+{} more)", extras)
            } else {
                "".to_string()
            }
        ));
    }

    let mut sc_parts = Vec::new();
    if let Some(semantic) = snapshot
        .semantic
        .as_ref()
        .filter(|semantic| !semantic.items.is_empty())
    {
        let items = semantic
            .items
            .iter()
            .take(2)
            .map(|item| {
                format!(
                    "{}@{:.2}",
                    compact_resume_rag_text(&item.content, 220),
                    item.score
                )
            })
            .collect::<Vec<_>>();
        sc_parts.push(format!("S={}", items.join(" | ")));
    }

    if let Some(handoff) = handoff {
        if !handoff.sources.sources.is_empty() {
            let sources = handoff
                .sources
                .sources
                .iter()
                .take(3)
                .map(|source| {
                    format!(
                        "{}({})@{:.2}",
                        source.source_agent.as_deref().unwrap_or("none"),
                        source.workspace.as_deref().unwrap_or("none"),
                        source.trust_score
                    )
                })
                .collect::<Vec<_>>();
            sc_parts.push(format!("C={}", sources.join(" | ")));
        }
        markdown.push_str("\n## Handoff Notes\n\n");
        markdown.push_str("- this file was refreshed from a shared handoff bundle\n");
        markdown.push_str("- dream/consolidation output should feed this same file so durable memory and distilled memory stay aligned\n");
    }

    if !sc_parts.is_empty() {
        markdown.push_str("\n## S+C\n\n");
        markdown.push_str(&format!("- {}\n", sc_parts.join(" | ")));
    }

    if let Some(hive) = hive {
        markdown.push_str("\n## Hive\n\n");
        markdown.push_str(&format!(
            "- queen={} roster={} active={} review={} overlap={} stale={}\n",
            hive.board.queen_session.as_deref().unwrap_or("none"),
            hive.roster.bees.len(),
            hive.board.active_bees.len(),
            hive.board.review_queue.len(),
            hive.board.overlap_risks.len(),
            hive.board.stale_bees.len(),
        ));
        if !hive.board.active_bees.is_empty() {
            let active = hive
                .board
                .active_bees
                .iter()
                .take(3)
                .map(|bee| {
                    format!(
                        "{}({})/{}",
                        bee.worker_name
                            .as_deref()
                            .or(bee.agent.as_deref())
                            .unwrap_or("unnamed"),
                        bee.session,
                        bee.task_id.as_deref().unwrap_or("none")
                    )
                })
                .collect::<Vec<_>>();
            markdown.push_str(&format!("- active_bees={}\n", active.join(" | ")));
        }
        if let Some(follow) = hive.follow.as_ref() {
            markdown.push_str(&format!(
                "- focus={} work=\"{}\" touches={} next=\"{}\" action={}\n",
                follow
                    .target
                    .worker_name
                    .as_deref()
                    .or(follow.target.agent.as_deref())
                    .unwrap_or(follow.target.session.as_str()),
                compact_inline(&follow.work_summary, 120),
                if follow.touch_points.is_empty() {
                    "none".to_string()
                } else {
                    compact_inline(&follow.touch_points.join(","), 120)
                },
                follow.next_action.as_deref().unwrap_or("none"),
                follow.recommended_action,
            ));
        }
        if !hive.board.recommended_actions.is_empty() {
            markdown.push_str(&format!(
                "- recommended={}\n",
                hive.board
                    .recommended_actions
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" | ")
            ));
        }
    }

    markdown.push_str("\n## Event Compiler\n\n");
    markdown.push_str("- live event log: [MEMD_EVENTS.md](MEMD_EVENTS.md)\n");
    markdown.push_str(
        "- compiled event pages: [compiled/events/latest.md](compiled/events/latest.md)\n",
    );
    markdown.push_str(
        "- memory updates now flow through the event compiler before the visible pages refresh\n",
    );

    markdown.push_str("\n## Memory Pages\n\n");
    for lane in [
        MemoryObjectLane::Context,
        MemoryObjectLane::Working,
        MemoryObjectLane::Inbox,
        MemoryObjectLane::Recovery,
        MemoryObjectLane::Semantic,
        MemoryObjectLane::Workspace,
    ] {
        markdown.push_str(&format!(
            "- [{}](compiled/memory/{}.md)\n",
            lane.title(),
            lane.slug()
        ));
    }

    markdown
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum MemoryObjectLane {
    Context,
    Working,
    Inbox,
    Recovery,
    Semantic,
    Workspace,
}

impl MemoryObjectLane {
    fn slug(self) -> &'static str {
        match self {
            MemoryObjectLane::Context => "context",
            MemoryObjectLane::Working => "working",
            MemoryObjectLane::Inbox => "inbox",
            MemoryObjectLane::Recovery => "recovery",
            MemoryObjectLane::Semantic => "semantic",
            MemoryObjectLane::Workspace => "workspace",
        }
    }

    fn title(self) -> &'static str {
        match self {
            MemoryObjectLane::Context => "Context",
            MemoryObjectLane::Working => "Working",
            MemoryObjectLane::Inbox => "Inbox",
            MemoryObjectLane::Recovery => "Recovery",
            MemoryObjectLane::Semantic => "Semantic",
            MemoryObjectLane::Workspace => "Workspace",
        }
    }
}

pub(crate) fn bundle_compiled_memory_dir(output: &Path) -> PathBuf {
    output.join("compiled").join("memory")
}

pub(crate) fn bundle_compiled_memory_path(output: &Path, lane: MemoryObjectLane) -> PathBuf {
    bundle_compiled_memory_dir(output).join(format!("{}.md", lane.slug()))
}

pub(crate) fn bundle_compiled_memory_item_path(
    output: &Path,
    lane: MemoryObjectLane,
    index: usize,
    key: &str,
) -> PathBuf {
    bundle_compiled_memory_dir(output)
        .join("items")
        .join(lane.slug())
        .join(format!(
            "{}-{:02}-{}.md",
            lane.slug(),
            index + 1,
            short_hash_text(key)
        ))
}

pub(crate) fn short_hash_text(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
        .chars()
        .take(8)
        .collect()
}

pub(crate) fn memory_object_lane_item_key(
    snapshot: &ResumeSnapshot,
    lane: MemoryObjectLane,
    index: usize,
) -> Option<String> {
    match lane {
        MemoryObjectLane::Context => snapshot
            .context
            .records
            .get(index)
            .map(|record| format!("{}|{}", record.id, record.record)),
        MemoryObjectLane::Working => snapshot
            .working
            .records
            .get(index)
            .map(|record| format!("{}|{}", record.id, record.record)),
        MemoryObjectLane::Inbox => snapshot.inbox.items.get(index).map(|item| {
            format!(
                "{}|{}|{}|{}|{}|{:?}|{:?}",
                item.item.id,
                item.item.content,
                format!("{:?}", item.item.kind),
                format!("{:?}", item.item.scope),
                format!("{:?}", item.item.status),
                item.item.stage,
                item.item.confidence
            )
        }),
        MemoryObjectLane::Recovery => snapshot.working.rehydration_queue.get(index).map(|item| {
            format!(
                "{}|{}|{}|{}",
                item.id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                item.kind,
                item.label,
                item.summary
            )
        }),
        MemoryObjectLane::Semantic => snapshot
            .semantic
            .as_ref()
            .and_then(|semantic| semantic.items.get(index))
            .map(|item| format!("{:.4}|{}", item.score, item.content)),
        MemoryObjectLane::Workspace => snapshot.workspaces.workspaces.get(index).map(|lane| {
            format!(
                "{}|{}|{}|{:?}|{}|{}|{}|{}",
                lane.project.as_deref().unwrap_or("none"),
                lane.namespace.as_deref().unwrap_or("none"),
                lane.workspace.as_deref().unwrap_or("none"),
                lane.visibility,
                lane.item_count,
                lane.active_count,
                lane.contested_count,
                lane.trust_score
            )
        }),
    }
}

pub(crate) fn memory_object_item_slug(
    snapshot: &ResumeSnapshot,
    lane: MemoryObjectLane,
    index: usize,
) -> Option<String> {
    let key = memory_object_lane_item_key(snapshot, lane, index)?;
    Some(format!(
        "{}-{:02}-{}",
        lane.slug(),
        index + 1,
        short_hash_text(&key)
    ))
}

pub(crate) fn memory_object_lane_item_count(snapshot: &ResumeSnapshot, lane: MemoryObjectLane) -> usize {
    match lane {
        MemoryObjectLane::Context => snapshot.context.records.len(),
        MemoryObjectLane::Working => snapshot.working.records.len(),
        MemoryObjectLane::Inbox => snapshot.inbox.items.len(),
        MemoryObjectLane::Recovery => snapshot.working.rehydration_queue.len(),
        MemoryObjectLane::Semantic => snapshot
            .semantic
            .as_ref()
            .map(|semantic| semantic.items.len())
            .unwrap_or(0),
        MemoryObjectLane::Workspace => snapshot.workspaces.workspaces.len(),
    }
}

pub(crate) fn render_bundle_memory_object_markdown(
    output: &Path,
    snapshot: &ResumeSnapshot,
    handoff: Option<&HandoffSnapshot>,
    hive: Option<&BundleHiveMemorySurface>,
    lane: MemoryObjectLane,
) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# memd memory object: {}{}\n\n",
        lane.title(),
        render_memory_page_header_suffix(output)
    ));
    markdown.push_str(&render_bundle_scope_markdown(output, snapshot));
    markdown.push('\n');

    match lane {
        MemoryObjectLane::Context => {
            markdown.push_str("\n## Context\n\n");
            if snapshot.context.records.is_empty() {
                markdown.push_str("- none\n");
            } else {
                for record in snapshot.context.records.iter().take(6) {
                    markdown.push_str(&format!(
                        "- id={} record=\"{}\"\n",
                        short_uuid(record.id),
                        compact_inline(record.record.trim(), 160)
                    ));
                }
            }
        }
        MemoryObjectLane::Working => {
            markdown.push_str("\n## Working\n\n");
            if snapshot.working.records.is_empty() {
                markdown.push_str("- none\n");
            } else {
                for record in snapshot.working.records.iter().take(6) {
                    markdown.push_str(&format!(
                        "- id={} record=\"{}\"\n",
                        short_uuid(record.id),
                        compact_inline(record.record.trim(), 160)
                    ));
                }
            }
            markdown.push_str(&format!(
                "\n- budget={}/{} | pressure={} | refresh={}\n",
                snapshot.working.used_chars,
                snapshot.working.budget_chars,
                snapshot.context_pressure(),
                snapshot.refresh_recommended
            ));
        }
        MemoryObjectLane::Inbox => {
            markdown.push_str("\n## Inbox\n\n");
            if snapshot.inbox.items.is_empty() {
                markdown.push_str("- none\n");
            } else {
                for item in snapshot.inbox.items.iter().take(6) {
                    markdown.push_str(&format!(
                        "- id={} kind={} status={} stage={} cf={:.2} scope={} source={} note=\"{}\"\n",
                        short_uuid(item.item.id),
                        enum_label_kind(item.item.kind),
                        enum_label_status(item.item.status),
                        format!("{:?}", item.item.stage).to_ascii_lowercase(),
                        item.item.confidence,
                        format!("{:?}", item.item.scope).to_ascii_lowercase(),
                        ResumeSnapshot::source_label(
                            item.item.source_agent.as_deref(),
                            item.item.source_system.as_deref(),
                            item.item.source_path.as_deref()
                        ),
                        compact_inline(item.item.content.trim(), 160)
                    ));
                    if !item.reasons.is_empty() {
                        markdown.push_str(&format!(
                            "  - reasons={}\n",
                            item.reasons
                                .iter()
                                .take(3)
                                .cloned()
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                }
            }
        }
        MemoryObjectLane::Recovery => {
            markdown.push_str("\n## Recovery\n\n");
            if snapshot.working.rehydration_queue.is_empty() {
                markdown.push_str("- none\n");
            } else {
                for artifact in snapshot.working.rehydration_queue.iter().take(6) {
                    markdown.push_str(&format!(
                        "- id={} kind={} label=\"{}\" source={} reason=\"{}\"\n",
                        artifact
                            .id
                            .map(short_uuid)
                            .unwrap_or_else(|| "none".to_string()),
                        artifact.kind,
                        compact_inline(&artifact.label, 96),
                        ResumeSnapshot::source_label(
                            artifact.source_agent.as_deref(),
                            artifact.source_system.as_deref(),
                            artifact.source_path.as_deref()
                        ),
                        artifact
                            .reason
                            .as_deref()
                            .map(|value| compact_inline(value, 160))
                            .unwrap_or_else(|| "none".to_string())
                    ));
                }
            }
        }
        MemoryObjectLane::Semantic => {
            markdown.push_str("\n## Semantic\n\n");
            if let Some(semantic) = snapshot
                .semantic
                .as_ref()
                .filter(|semantic| !semantic.items.is_empty())
            {
                for item in semantic.items.iter().take(6) {
                    markdown.push_str(&format!(
                        "- score={:.2} content=\"{}\"\n",
                        item.score,
                        compact_inline(&item.content, 160)
                    ));
                }
            } else {
                markdown.push_str("- none\n");
            }
        }
        MemoryObjectLane::Workspace => {
            markdown.push_str("\n## Workspace\n\n");
            if snapshot.workspaces.workspaces.is_empty() {
                markdown.push_str("- none\n");
            } else {
                for lane in snapshot.workspaces.workspaces.iter().take(6) {
                    markdown.push_str(&format!(
                        "- project={} namespace={} workspace={} visibility={} items={} active={} contested={} trust={:.2} cf={:.2}\n",
                        lane.project.as_deref().unwrap_or("none"),
                        lane.namespace.as_deref().unwrap_or("none"),
                        lane.workspace.as_deref().unwrap_or("none"),
                        memory_visibility_label(lane.visibility),
                        lane.item_count,
                        lane.active_count,
                        lane.contested_count,
                        lane.trust_score,
                        lane.avg_confidence
                    ));
                }
            }
        }
    }

    if let Some(handoff) = handoff {
        markdown.push_str("\n## Handoff\n\n");
        markdown.push_str(&format!(
            "- target_session={} target_bundle={}\n",
            handoff.target_session.as_deref().unwrap_or("none"),
            handoff.target_bundle.as_deref().unwrap_or("none")
        ));
        markdown.push_str(&format!("- sources={}\n", handoff.sources.sources.len()));
    }

    if matches!(lane, MemoryObjectLane::Workspace) {
        if let Some(hive) = hive {
            markdown.push_str("\n## Hive\n\n");
            markdown.push_str(&format!(
                "- queen={} active={} stale={} review={} overlap={}\n",
                hive.board.queen_session.as_deref().unwrap_or("none"),
                hive.board.active_bees.len(),
                hive.board.stale_bees.len(),
                hive.board.review_queue.len(),
                hive.board.overlap_risks.len(),
            ));
            for bee in hive.board.active_bees.iter().take(6) {
                markdown.push_str(&format!(
                    "- bee {} ({}) lane={} task={}\n",
                    bee.worker_name
                        .as_deref()
                        .or(bee.agent.as_deref())
                        .unwrap_or("unnamed"),
                    bee.session,
                    bee.lane_id
                        .as_deref()
                        .or(bee.branch.as_deref())
                        .unwrap_or("none"),
                    bee.task_id.as_deref().unwrap_or("none"),
                ));
            }
        }
    }

    markdown.push_str("\n## Items\n\n");
    let item_count = memory_object_lane_item_count(snapshot, lane);
    if item_count == 0 {
        markdown.push_str("- none\n");
    } else {
        for index in 0..item_count {
            if let Some(slug) = memory_object_item_slug(snapshot, lane, index) {
                markdown.push_str(&format!(
                    "- [{}](items/{}/{})\n",
                    lane.title(),
                    lane.slug(),
                    slug
                ));
            }
        }
    }

    markdown
}

pub(crate) fn render_bundle_memory_object_item_markdown(
    output: &Path,
    snapshot: &ResumeSnapshot,
    handoff: Option<&HandoffSnapshot>,
    hive: Option<&BundleHiveMemorySurface>,
    lane: MemoryObjectLane,
    index: usize,
) -> Option<String> {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# memd memory item: {}{}\n\n",
        lane.title(),
        render_memory_page_header_suffix(output)
    ));
    markdown.push_str(&render_bundle_scope_markdown(output, snapshot));
    markdown.push('\n');
    markdown.push_str(&format!("- lane={} | index={}\n", lane.slug(), index + 1));

    match lane {
        MemoryObjectLane::Context => {
            let record = snapshot.context.records.get(index)?;
            markdown.push_str(&format!(
                "- id={} record=\"{}\"\n",
                short_uuid(record.id),
                compact_inline(record.record.trim(), 240)
            ));
        }
        MemoryObjectLane::Working => {
            let record = snapshot.working.records.get(index)?;
            markdown.push_str(&format!(
                "- id={} record=\"{}\"\n",
                short_uuid(record.id),
                compact_inline(record.record.trim(), 240)
            ));
            markdown.push_str(&format!(
                "- budget={}/{} | pressure={} | refresh={}\n",
                snapshot.working.used_chars,
                snapshot.working.budget_chars,
                snapshot.context_pressure(),
                snapshot.refresh_recommended
            ));
        }
        MemoryObjectLane::Inbox => {
            let item = snapshot.inbox.items.get(index)?;
            markdown.push_str(&format!(
                "- id={} kind={} status={} stage={} cf={:.2} scope={} source={} note=\"{}\"\n",
                short_uuid(item.item.id),
                enum_label_kind(item.item.kind),
                enum_label_status(item.item.status),
                format!("{:?}", item.item.stage).to_ascii_lowercase(),
                item.item.confidence,
                format!("{:?}", item.item.scope).to_ascii_lowercase(),
                ResumeSnapshot::source_label(
                    item.item.source_agent.as_deref(),
                    item.item.source_system.as_deref(),
                    item.item.source_path.as_deref()
                ),
                compact_inline(item.item.content.trim(), 240)
            ));
            if !item.reasons.is_empty() {
                markdown.push_str(&format!(
                    "- reasons={}\n",
                    item.reasons
                        .iter()
                        .take(6)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
        MemoryObjectLane::Recovery => {
            let item = snapshot.working.rehydration_queue.get(index)?;
            markdown.push_str(&format!(
                "- id={} kind={} label=\"{}\" source={} reason=\"{}\"\n",
                item.id
                    .map(short_uuid)
                    .unwrap_or_else(|| "none".to_string()),
                item.kind,
                compact_inline(&item.label, 120),
                ResumeSnapshot::source_label(
                    item.source_agent.as_deref(),
                    item.source_system.as_deref(),
                    item.source_path.as_deref()
                ),
                item.reason
                    .as_deref()
                    .map(|value| compact_inline(value, 240))
                    .unwrap_or_else(|| "none".to_string())
            ));
        }
        MemoryObjectLane::Semantic => {
            let semantic = snapshot.semantic.as_ref()?.items.get(index)?;
            markdown.push_str(&format!(
                "- score={:.2} content=\"{}\"\n",
                semantic.score,
                compact_inline(&semantic.content, 240)
            ));
        }
        MemoryObjectLane::Workspace => {
            let lane = snapshot.workspaces.workspaces.get(index)?;
            markdown.push_str(&format!(
                "- project={} namespace={} workspace={} visibility={} items={} active={} contested={} trust={:.2} cf={:.2}\n",
                lane.project.as_deref().unwrap_or("none"),
                lane.namespace.as_deref().unwrap_or("none"),
                lane.workspace.as_deref().unwrap_or("none"),
                memory_visibility_label(lane.visibility),
                lane.item_count,
                lane.active_count,
                lane.contested_count,
                lane.trust_score,
                lane.avg_confidence
            ));
        }
    }

    if let Some(handoff) = handoff {
        markdown.push_str("\n## Handoff\n\n");
        markdown.push_str(&format!(
            "- target_session={} target_bundle={}\n",
            handoff.target_session.as_deref().unwrap_or("none"),
            handoff.target_bundle.as_deref().unwrap_or("none")
        ));
        markdown.push_str(&format!("- sources={}\n", handoff.sources.sources.len()));
    }

    if matches!(lane, MemoryObjectLane::Workspace) {
        if let Some(hive) = hive {
            markdown.push_str("\n## Hive\n\n");
            markdown.push_str(&format!(
                "- queen={} active={} overlap={} stale={}\n",
                hive.board.queen_session.as_deref().unwrap_or("none"),
                hive.board.active_bees.len(),
                hive.board.overlap_risks.len(),
                hive.board.stale_bees.len(),
            ));
            if let Some(follow) = hive.follow.as_ref() {
                markdown.push_str(&format!(
                    "- focus={} work=\"{}\" next=\"{}\"\n",
                    follow
                        .target
                        .worker_name
                        .as_deref()
                        .or(follow.target.agent.as_deref())
                        .unwrap_or(follow.target.session.as_str()),
                    compact_inline(&follow.work_summary, 160),
                    follow.next_action.as_deref().unwrap_or("none"),
                ));
            }
        }
    }

    Some(markdown)
}

pub(crate) fn render_current_task_bundle_snapshot(snapshot: &ResumeSnapshot) -> String {
    let mut markdown = String::new();

    let capsule = snapshot.workflow_capsule();
    if !capsule.is_empty() {
        let summary = capsule
            .iter()
            .take(4)
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join(" | ");
        markdown.push_str(&format!("- t={summary}\n"));
    }

    markdown
}

pub(crate) fn write_bundle_backend_env(output: &Path, config: &BundleConfig) -> anyhow::Result<()> {
    let backend_env = output.join("backend.env");
    let backend_env_ps1 = output.join("backend.env.ps1");
    let rag = &config.backend.rag;

    let mut shell = String::new();
    shell.push_str(&format!(
        "MEMD_BUNDLE_SCHEMA_VERSION={}\n",
        config.schema_version
    ));
    shell.push_str(&format!("MEMD_BUNDLE_BACKEND_PROVIDER={}\n", rag.provider));
    shell.push_str(&format!(
        "MEMD_BUNDLE_BACKEND_ENABLED={}\n",
        if rag.enabled { "true" } else { "false" }
    ));
    if let Some(url) = rag.url.as_deref() {
        shell.push_str(&format!("MEMD_RAG_URL={url}\n"));
    }
    fs::write(&backend_env, shell).with_context(|| format!("write {}", backend_env.display()))?;

    let mut ps1 = String::new();
    ps1.push_str(&format!(
        "$env:MEMD_BUNDLE_SCHEMA_VERSION = \"{}\"\n",
        config.schema_version
    ));
    ps1.push_str(&format!(
        "$env:MEMD_BUNDLE_BACKEND_PROVIDER = \"{}\"\n",
        escape_ps1(&rag.provider)
    ));
    ps1.push_str(&format!(
        "$env:MEMD_BUNDLE_BACKEND_ENABLED = \"{}\"\n",
        if rag.enabled { "true" } else { "false" }
    ));
    if let Some(url) = rag.url.as_deref() {
        ps1.push_str(&format!("$env:MEMD_RAG_URL = \"{}\"\n", escape_ps1(url)));
    }
    fs::write(&backend_env_ps1, ps1)
        .with_context(|| format!("write {}", backend_env_ps1.display()))?;

    Ok(())
}

pub(crate) async fn read_bundle_status(output: &Path, base_url: &str) -> anyhow::Result<serde_json::Value> {
    let runtime_before_overlay = read_bundle_runtime_config_raw(output)?;
    let runtime = read_bundle_runtime_config(output)?;
    let bundle_session = runtime_before_overlay
        .as_ref()
        .and_then(|config| config.session.clone());
    let live_session = runtime.as_ref().and_then(|config| config.session.clone());
    let rebased_from = match (bundle_session.as_deref(), live_session.as_deref()) {
        (Some(bundle), Some(live)) if bundle != live => Some(bundle.to_string()),
        _ => None,
    };
    let resolved_base_url = resolve_bundle_command_base_url(
        base_url,
        runtime
            .as_ref()
            .and_then(|config| config.base_url.as_deref()),
    );
    if runtime
        .as_ref()
        .and_then(|config| config.session.as_deref())
        .is_some()
    {
        let _ = timeout_ok(refresh_bundle_heartbeat(output, None, false)).await;
    }
    let client = MemdClient::new(&resolved_base_url)?;
    let health = timeout_ok(client.healthz()).await;
    let heartbeat = read_bundle_heartbeat(output)?.map(|mut state| {
        if state.project.is_none() {
            state.project = runtime.as_ref().and_then(|config| config.project.clone());
        }
        if state.namespace.is_none() {
            state.namespace = runtime.as_ref().and_then(|config| config.namespace.clone());
        }
        if state.workspace.is_none() {
            state.workspace = runtime.as_ref().and_then(|config| config.workspace.clone());
        }
        if state.visibility.is_none() {
            state.visibility = runtime
                .as_ref()
                .and_then(|config| config.visibility.clone());
        }
        if state.session.is_none() {
            state.session = runtime.as_ref().and_then(|config| config.session.clone());
        }
        if state.agent.is_none() {
            state.agent = runtime.as_ref().and_then(|config| config.agent.clone());
        }
        if state.effective_agent.is_none() {
            state.effective_agent = runtime.as_ref().and_then(|config| {
                config
                    .agent
                    .as_deref()
                    .map(|agent| compose_agent_identity(agent, config.session.as_deref()))
            });
        }
        if state.tab_id.is_none() {
            state.tab_id = runtime.as_ref().and_then(|config| config.tab_id.clone());
        }
        state
    });
    let runtimes = read_memd_runtime_wiring();
    let harness_bridge =
        read_bundle_harness_bridge_registry(output)?.unwrap_or_else(build_harness_bridge_registry);
    let config_exists = output.join("config.json").exists();
    let env_exists = output.join("env").exists();
    let env_ps1_exists = output.join("env.ps1").exists();
    let hooks_exists = output.join("hooks").exists();
    let agents_exists = output.join("agents").exists();
    let worker_name_env_ready = read_bundle_config_file(output)
        .ok()
        .map(|(_, config)| bundle_worker_name_env_ready(output, &config))
        .unwrap_or(false);
    let mut missing = Vec::<&str>::new();
    if !config_exists {
        missing.push("config.json");
    }
    if !env_exists {
        missing.push("env");
    }
    if !env_ps1_exists {
        missing.push("env.ps1");
    }
    if env_exists && env_ps1_exists && !worker_name_env_ready {
        missing.push("worker_name_env");
    }
    if !hooks_exists {
        missing.push("hooks/");
    }
    if !agents_exists {
        missing.push("agents/");
    }
    let resume_preview = if output.join("config.json").exists() && health.is_some() {
        let preview = timeout_ok(read_bundle_resume(
            &ResumeArgs {
                output: output.to_path_buf(),
                project: None,
                namespace: None,
                agent: None,
                workspace: None,
                visibility: None,
                route: None,
                intent: Some("current_task".to_string()),
                limit: Some(4),
                rehydration_limit: Some(2),
                semantic: false,
                prompt: false,
                summary: false,
            },
            &resolved_base_url,
        ))
        .await;
        preview.map(|snapshot| {
            serde_json::json!({
                "project": snapshot.project,
                "namespace": snapshot.namespace,
                "agent": snapshot.agent,
                "session": runtime.as_ref().and_then(|config| config.session.clone()),
                "tab_id": runtime.as_ref().and_then(|config| config.tab_id.clone()),
                "workspace": snapshot.workspace,
                "visibility": snapshot.visibility,
                "route": snapshot.route,
                "intent": snapshot.intent,
                "context_records": snapshot.context.records.len(),
                "working_records": snapshot.working.records.len(),
                "inbox_items": snapshot.inbox.items.len(),
                "workspace_lanes": snapshot.workspaces.workspaces.len(),
                "rehydration_queue": snapshot.working.rehydration_queue.len(),
                "semantic_hits": snapshot.semantic.as_ref().map(|semantic| semantic.items.len()).unwrap_or(0),
                "change_summary": snapshot.change_summary,
                "event_spine": snapshot.event_spine(),
                "focus": snapshot.working.records.first().map(|record| record.record.clone()),
                "pressure": snapshot.inbox.items.first().map(|item| item.item.content.clone()),
                "next_recovery": snapshot.working.rehydration_queue.first().map(|item| format!("{}: {}", item.label, item.summary)),
                "estimated_prompt_chars": snapshot.estimated_prompt_chars(),
                "estimated_prompt_tokens": snapshot.estimated_prompt_tokens(),
                "context_pressure": snapshot.context_pressure(),
                "redundant_context_items": snapshot.redundant_context_items(),
                "refresh_recommended": snapshot.refresh_recommended,
            })
        })
    } else {
        None
    };
    let truth_summary = if output.join("config.json").exists() && health.is_some() {
        let snapshot = timeout_ok(read_bundle_resume(
            &ResumeArgs {
                output: output.to_path_buf(),
                project: None,
                namespace: None,
                agent: None,
                workspace: None,
                visibility: None,
                route: None,
                intent: Some("current_task".to_string()),
                limit: Some(4),
                rehydration_limit: Some(2),
                semantic: true,
                prompt: false,
                summary: false,
            },
            &resolved_base_url,
        ))
        .await;
        snapshot.map(|snapshot| {
            serde_json::to_value(build_truth_summary(&snapshot)).unwrap_or(JsonValue::Null)
        })
    } else {
        None
    };
    let current_project = runtime.as_ref().and_then(|config| config.project.clone());
    let current_namespace = runtime.as_ref().and_then(|config| config.namespace.clone());
    let current_workspace = runtime.as_ref().and_then(|config| config.workspace.clone());
    let current_session = runtime.as_ref().and_then(|config| config.session.clone());
    let cowork_surface = if health.is_some() {
        let inbox_request = HiveCoordinationInboxRequest {
            session: current_session.clone().unwrap_or_default(),
            project: current_project.clone(),
            namespace: current_namespace.clone(),
            workspace: current_workspace.clone(),
            limit: Some(128),
        };
        let inbox = timeout_ok(client.hive_coordination_inbox(&inbox_request)).await;
        let tasks_request = HiveTasksRequest {
            session: None,
            project: current_project.clone(),
            namespace: current_namespace.clone(),
            workspace: current_workspace.clone(),
            active_only: Some(false),
            limit: Some(256),
        };
        let tasks = timeout_ok(client.hive_tasks(&tasks_request)).await;
        match (inbox, tasks) {
            (Some(inbox), Some(tasks)) => {
                let exclusive = tasks
                    .tasks
                    .iter()
                    .filter(|task| task.coordination_mode == "exclusive_write")
                    .count();
                let open = tasks
                    .tasks
                    .iter()
                    .filter(|task| task.status != "done" && task.status != "closed")
                    .count();
                Some(serde_json::json!({
                    "tasks": tasks.tasks.len(),
                    "open_tasks": open,
                    "help_tasks": tasks.tasks.iter().filter(|task| task.help_requested).count(),
                    "review_tasks": tasks.tasks.iter().filter(|task| task.review_requested).count(),
                    "exclusive_tasks": exclusive,
                    "shared_tasks": tasks.tasks.len().saturating_sub(exclusive),
                    "inbox_messages": inbox.messages.len(),
                    "owned_tasks": inbox.owned_tasks.len(),
                    "owned_exclusive_tasks": inbox
                        .owned_tasks
                        .iter()
                        .filter(|task| task.coordination_mode == "exclusive_write")
                        .count(),
                    "owned_shared_tasks": inbox
                        .owned_tasks
                        .iter()
                        .filter(|task| task.coordination_mode != "exclusive_write")
                        .count(),
                    "help_inbox": inbox.help_tasks.len(),
                    "review_inbox": inbox.review_tasks.len(),
                    "views": build_task_view_counts(&tasks.tasks, current_session.as_deref()),
                }))
            }
            _ => None,
        }
    } else {
        None
    };
    let lane_receipts = if health.is_some() {
        let receipts_request = HiveCoordinationReceiptsRequest {
            session: None,
            project: current_project.clone(),
            namespace: current_namespace.clone(),
            workspace: current_workspace.clone(),
            limit: Some(64),
        };
        timeout_ok(client.hive_coordination_receipts(&receipts_request))
            .await
            .map(|response| {
                let receipts = response
                    .receipts
                    .into_iter()
                    .filter(|receipt| receipt.kind.starts_with("lane_"))
                    .collect::<Vec<_>>();
                serde_json::json!({
                    "count": receipts.len(),
                    "latest_kind": receipts.first().map(|receipt| receipt.kind.clone()),
                    "latest_summary": receipts.first().map(|receipt| receipt.summary.clone()),
                    "recent": receipts
                        .into_iter()
                        .take(8)
                        .map(|receipt| serde_json::json!({
                            "kind": receipt.kind,
                            "actor_session": receipt.actor_session,
                            "target_session": receipt.target_session,
                            "scope": receipt.scope,
                            "summary": receipt.summary,
                            "created_at": receipt.created_at,
                        }))
                        .collect::<Vec<_>>(),
                })
            })
    } else {
        None
    };
    let maintenance_surface = match (
        read_latest_maintain_report(output)?,
        read_previous_maintain_report(output)?,
        read_recent_maintain_reports(output, 5)?,
    ) {
        (Some(report), previous, history) => {
            let total = report.compacted_items + report.refreshed_items + report.repaired_items;
            let previous_total = previous
                .as_ref()
                .map(|value| value.compacted_items + value.refreshed_items + value.repaired_items)
                .unwrap_or(0);
            let delta_total = total as i64 - previous_total as i64;
            let auto_mode = report.mode == "auto";
            let auto_reason = if auto_mode {
                "none".to_string()
            } else if delta_total < 0 {
                "trend_down".to_string()
            } else if delta_total == 0 {
                "trend_flat".to_string()
            } else if !report.findings.is_empty() {
                "findings_present".to_string()
            } else {
                "none".to_string()
            };
            let auto_recommended = auto_reason != "none";
            let history_modes = history
                .iter()
                .map(|value| value.mode.clone())
                .collect::<Vec<_>>();
            let history_receipts = history
                .iter()
                .map(|value| {
                    value
                        .receipt_id
                        .clone()
                        .unwrap_or_else(|| "none".to_string())
                })
                .collect::<Vec<_>>();
            let history_totals = history
                .iter()
                .map(|value| value.compacted_items + value.refreshed_items + value.repaired_items)
                .collect::<Vec<_>>();
            Some(serde_json::json!({
                "mode": report.mode,
                "auto_mode": auto_mode,
                "auto_recommended": auto_recommended,
                "auto_reason": auto_reason,
                "receipt": report.receipt_id,
                "compacted": report.compacted_items,
                "refreshed": report.refreshed_items,
                "repaired": report.repaired_items,
                "findings": report.findings.len(),
                "total_actions": total,
                "delta_total_actions": delta_total,
                "trend": if delta_total > 0 { "up" } else if delta_total < 0 { "down" } else { "flat" },
                "previous_mode": previous.as_ref().map(|value| value.mode.clone()),
                "history_modes": history_modes,
                "history_receipts": history_receipts,
                "history_totals": history_totals,
                "history_count": history.len(),
                "generated_at": report.generated_at,
            }))
        }
        _ => None,
    };
    let rag_config = read_bundle_rag_config(output)?;
    let rag = match rag_config {
        Some(config) if config.enabled => {
            let source = config.source;
            let Some(url) = config.url.clone() else {
                return Ok(serde_json::json!({
                    "bundle": output,
                    "exists": output.exists(),
                    "config": output.join("config.json").exists(),
                    "env": output.join("env").exists(),
                    "env_ps1": output.join("env.ps1").exists(),
                    "hooks": output.join("hooks").exists(),
                    "agents": output.join("agents").exists(),
                    "server": health,
                    "rag": {
                        "configured": false,
                        "enabled": true,
                        "healthy": false,
                        "error": "rag backend enabled but no url configured",
                        "source": source,
                    },
                }));
            };
            let rag_result = RagClient::new(url.as_str())?.healthz().await;
            Some(match rag_result {
                Ok(health) => serde_json::json!({
                    "configured": true,
                    "enabled": true,
                    "url": url,
                    "healthy": true,
                    "health": health,
                    "source": source,
                }),
                Err(error) => serde_json::json!({
                    "configured": true,
                    "enabled": true,
                    "url": url,
                    "healthy": false,
                    "error": error.to_string(),
                    "source": source,
                }),
            })
        }
        Some(config) => Some(serde_json::json!({
            "configured": config.configured,
            "enabled": false,
            "url": config.url,
            "healthy": null,
            "source": config.source,
        })),
        None => None,
    };
    let rag_ready = rag
        .as_ref()
        .map(|value| {
            !value
                .get("enabled")
                .and_then(|enabled| enabled.as_bool())
                .unwrap_or(false)
                || value
                    .get("healthy")
                    .and_then(|healthy| healthy.as_bool())
                    .unwrap_or(false)
        })
        .unwrap_or(true);
    let evolution = summarize_evolution_status(output)?;
    let capability_registry =
        build_bundle_capability_registry(std::env::current_dir().ok().as_deref());
    let capability_surface = serde_json::json!({
        "discovered": capability_registry.capabilities.len(),
        "universal": capability_registry
            .capabilities
            .iter()
            .filter(|record| is_universal_class(&record.portability_class))
            .count(),
        "bridgeable": capability_registry
            .capabilities
            .iter()
            .filter(|record| is_bridgeable_class(&record.portability_class))
            .count(),
        "harness_native": capability_registry
            .capabilities
            .iter()
            .filter(|record| is_harness_native_class(&record.portability_class))
            .count(),
    });
    let lane_surface = read_bundle_lane_surface(output)?
        .map(|surface| serde_json::to_value(surface).unwrap_or(JsonValue::Null));
    let lane_fault = detect_bundle_lane_collision(output, current_session.as_deref())
        .await?
        .and_then(|conflict| {
            build_lane_fault_surface(output, current_session.as_deref(), &conflict)
        });
    let bridge_ready = harness_bridge.all_wired;
    let setup_ready = output.exists()
        && missing.is_empty()
        && health.is_some()
        && runtime.is_some()
        && rag_ready
        && bridge_ready;
    Ok(serde_json::json!({
        "bundle": output,
        "exists": output.exists(),
        "config": config_exists,
        "env": env_exists,
        "env_ps1": env_ps1_exists,
        "worker_name_env_ready": worker_name_env_ready,
        "hooks": hooks_exists,
        "agents": agents_exists,
        "setup_ready": setup_ready,
        "missing": missing,
        "runtimes": runtimes,
        "harness_bridge": {
            "ready": bridge_ready,
            "portable": harness_bridge.all_wired,
            "portability_class": harness_bridge.overall_portability_class,
            "generated_at": harness_bridge.generated_at,
            "harnesses": harness_bridge.harnesses,
            "missing_harnesses": harness_bridge
                .harnesses
                .iter()
                .filter(|record| !record.wired)
                .map(|record| record.harness.clone())
                .collect::<Vec<_>>(),
        },
        "active_agent": runtime.as_ref().and_then(|config| config.agent.clone()),
        "defaults": runtime
            .as_ref()
            .and_then(|config| serde_json::to_value(config).ok()),
        "authority": runtime
            .as_ref()
            .map(|config| config.authority_state.mode.clone()),
        "shared_primary": runtime_prefers_shared_authority(runtime.as_ref()),
        "localhost_read_only_allowed": runtime_allows_localhost_read_only(runtime.as_ref()),
        "degraded": runtime
            .as_ref()
            .map(|config| config.authority_state.degraded)
            .unwrap_or(false),
        "shared_base_url": runtime
            .as_ref()
            .and_then(|config| config.authority_state.shared_base_url.clone()),
        "fallback_base_url": runtime
            .as_ref()
            .and_then(|config| config.authority_state.fallback_base_url.clone()),
        "authority_warning": authority_warning_lines(runtime.as_ref()),
        "session_overlay": {
            "bundle_session": bundle_session,
            "live_session": live_session,
            "rebased_from": rebased_from,
        },
        "heartbeat": heartbeat
            .as_ref()
            .and_then(|value| serde_json::to_value(value).ok()),
        "resume_preview": resume_preview,
        "truth_summary": truth_summary,
        "evolution": evolution,
        "cowork_surface": cowork_surface,
        "lane_surface": lane_surface,
        "lane_fault": lane_fault,
        "lane_receipts": lane_receipts,
        "maintenance_surface": maintenance_surface,
        "capability_surface": capability_surface,
        "server": health,
        "rag": rag.unwrap_or_else(|| serde_json::json!({
            "configured": false,
            "enabled": false,
            "healthy": null,
        })),
    }))
}
