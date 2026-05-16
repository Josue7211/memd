use super::*;

/// Render a bash snippet that resolves `MEMD_BUNDLE_ROOT` at runtime.
///
/// Resolution order:
/// 1. Existing `MEMD_BUNDLE_ROOT` env var (if it points to a directory).
/// 2. Walk up from `$PWD` looking for a `.memd/` directory (handles
///    invocation from any subdirectory of the project).
/// 3. `git rev-parse --git-common-dir` so worktrees fall back to the
///    main worktree's `.memd/` (the shared memory bundle).
/// 4. The absolute path baked at init time as a final fallback.
fn bash_bundle_root_resolver(output: &Path) -> String {
    let absolute = std::fs::canonicalize(output)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| output.display().to_string());
    let absolute_q = compact_bundle_value(&absolute);
    format!(
        "_memd_default_root=\"{absolute_q}\"\n\
         _memd_resolve_root() {{\n\
           if [[ -n \"${{MEMD_BUNDLE_ROOT:-}}\" && -d \"$MEMD_BUNDLE_ROOT\" ]]; then\n\
             printf '%s\\n' \"$MEMD_BUNDLE_ROOT\"; return 0\n\
           fi\n\
           local d=\"$PWD\"\n\
           while [[ \"$d\" != \"/\" && -n \"$d\" ]]; do\n\
             if [[ -d \"$d/.memd\" ]]; then printf '%s\\n' \"$d/.memd\"; return 0; fi\n\
             d=\"$(dirname \"$d\")\"\n\
           done\n\
           local gcd\n\
           if gcd=\"$(git rev-parse --git-common-dir 2>/dev/null)\"; then\n\
             [[ \"$gcd\" != /* ]] && gcd=\"$PWD/$gcd\"\n\
             local main_root\n\
             if main_root=\"$(cd \"$(dirname \"$gcd\")\" 2>/dev/null && pwd)\" && [[ -n \"$main_root\" && -d \"$main_root/.memd\" ]]; then\n\
               printf '%s\\n' \"$main_root/.memd\"; return 0\n\
             fi\n\
           fi\n\
           printf '%s\\n' \"$_memd_default_root\"\n\
         }}\n\
         export MEMD_BUNDLE_ROOT=\"$(_memd_resolve_root)\"\n",
        absolute_q = absolute_q
    )
}

/// PowerShell equivalent of [`bash_bundle_root_resolver`].
fn ps1_bundle_root_resolver(output: &Path) -> String {
    let absolute = std::fs::canonicalize(output)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| output.display().to_string());
    let absolute_q = escape_ps1(&absolute);
    format!(
        "$_memdDefaultRoot = \"{absolute_q}\"\n\
         function _Memd-ResolveRoot {{\n\
           if ($env:MEMD_BUNDLE_ROOT -and (Test-Path $env:MEMD_BUNDLE_ROOT)) {{ return $env:MEMD_BUNDLE_ROOT }}\n\
           $d = (Get-Location).Path\n\
           while ($d) {{\n\
             $candidate = Join-Path $d '.memd'\n\
             if (Test-Path $candidate) {{ return $candidate }}\n\
             $parent = Split-Path -Parent $d\n\
             if (-not $parent -or $parent -eq $d) {{ break }}\n\
             $d = $parent\n\
           }}\n\
           try {{\n\
             $gcd = (git rev-parse --git-common-dir 2>$null) | Out-String\n\
             $gcd = $gcd.Trim()\n\
             if ($gcd) {{\n\
               if (-not [System.IO.Path]::IsPathRooted($gcd)) {{ $gcd = Join-Path (Get-Location).Path $gcd }}\n\
               $mainRoot = Split-Path -Parent $gcd\n\
               $candidate = Join-Path $mainRoot '.memd'\n\
               if (Test-Path $candidate) {{ return $candidate }}\n\
             }}\n\
           }} catch {{ }}\n\
           return $_memdDefaultRoot\n\
         }}\n\
         $env:MEMD_BUNDLE_ROOT = (_Memd-ResolveRoot)\n",
        absolute_q = absolute_q
    )
}

pub(crate) fn render_agent_shell_profile(output: &Path, env_agent: Option<&str>) -> String {
    let (startup_route, startup_intent) = bundle_startup_route_intent(output);
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
        "#!/usr/bin/env bash\nset -euo pipefail\n\n{}set -a\nsource \"$MEMD_BUNDLE_ROOT/backend.env\" 2>/dev/null || true\nsource \"$MEMD_BUNDLE_ROOT/env\"\nset +a\n",
        bash_bundle_root_resolver(output),
    );
    let bundle_config = read_bundle_config_file(output)
        .ok()
        .map(|(_, config)| config);
    let bundle_session = bundle_config
        .as_ref()
        .and_then(|config| config.session.clone());
    let bundle_project = bundle_config
        .as_ref()
        .and_then(|config| config.project.as_deref());
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
        &format!(
            "memd wake --output \"$MEMD_BUNDLE_ROOT\" --route {} --intent {} --write >/dev/null 2>&1 || true\n",
            compact_bundle_value(&startup_route),
            compact_bundle_value(&startup_intent)
        ),
    );
    script.push_str(
        "nohup memd heartbeat --output \"$MEMD_BUNDLE_ROOT\" --watch --interval-secs 30 --probe-base-url >/tmp/memd-heartbeat.log 2>&1 &\n",
    );
    script.push_str(
        "memd hive --output \"$MEMD_BUNDLE_ROOT\" --publish-heartbeat --summary >/dev/null 2>&1 || true\n",
    );
    if env_agent == Some("codex") {
        script.push_str("printf '%s\\n' \"memd voice: ${MEMD_VOICE_MODE:-unknown}\"\n");
        script.push_str(
            "printf '%s\\n' \"memd rule: if draft not in ${MEMD_VOICE_MODE:-unknown}, rewrite before send.\"\n",
        );
        script.push_str(
            "if [[ -f \"$MEMD_BUNDLE_ROOT/wake.md\" ]]; then\n  cat \"$MEMD_BUNDLE_ROOT/wake.md\"\n  printf '\\n'\nfi\n",
        );
        script.push_str(
            "printf '%s\\n' 'memd reminder: run .memd/agents/lookup.sh \"what did we already decide about this?\" before memory-dependent answers.'\n",
        );
    }
    script.push_str(&format!(
        "exec memd wake --output \"$MEMD_BUNDLE_ROOT\" --route {} --intent {} --write \"$@\"\n",
        compact_bundle_value(&startup_route),
        compact_bundle_value(&startup_intent)
    ));
    script
}

pub(crate) fn render_agent_ps1_profile(output: &Path, env_agent: Option<&str>) -> String {
    let (startup_route, startup_intent) = bundle_startup_route_intent(output);
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
        "{}$bundleBackendEnv = Join-Path $env:MEMD_BUNDLE_ROOT \"backend.env.ps1\"\nif (Test-Path $bundleBackendEnv) {{ . $bundleBackendEnv }}\n. (Join-Path $env:MEMD_BUNDLE_ROOT \"env.ps1\")\n",
        ps1_bundle_root_resolver(output),
    );
    let bundle_config = read_bundle_config_file(output)
        .ok()
        .map(|(_, config)| config);
    let bundle_session = bundle_config
        .as_ref()
        .and_then(|config| config.session.clone());
    let bundle_project = bundle_config
        .as_ref()
        .and_then(|config| config.project.as_deref());
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
    script.push_str(&format!(
        "try {{ memd wake --output $env:MEMD_BUNDLE_ROOT --route {} --intent {} --write | Out-Null }} catch {{ }}\n",
        escape_ps1(&startup_route),
        escape_ps1(&startup_intent)
    ));
    script.push_str(
        "Start-Process -WindowStyle Hidden -FilePath memd -ArgumentList @('heartbeat','--output',$env:MEMD_BUNDLE_ROOT,'--watch','--interval-secs','30','--probe-base-url') -RedirectStandardOutput \"$env:TEMP\\memd-heartbeat.log\" -RedirectStandardError \"$env:TEMP\\memd-heartbeat.err\"\n",
    );
    script.push_str(
        "try { memd hive --output $env:MEMD_BUNDLE_ROOT --publish-heartbeat --summary | Out-Null } catch { }\n",
    );
    if env_agent == Some("codex") {
        script.push_str(
            "Write-Host (\"memd voice: {0}\" -f $(if ($env:MEMD_VOICE_MODE) { $env:MEMD_VOICE_MODE } else { \"unknown\" }))\n",
        );
        script.push_str(
            "Write-Host (\"memd rule: if draft not in {0}, rewrite before send.\" -f $(if ($env:MEMD_VOICE_MODE) { $env:MEMD_VOICE_MODE } else { \"unknown\" }))\n",
        );
        script.push_str(
            "$codexWake = Join-Path $env:MEMD_BUNDLE_ROOT \"wake.md\"\nif (Test-Path $codexWake) { Get-Content $codexWake }\n",
        );
        script.push_str(
            "Write-Host 'memd reminder: run .memd/agents/lookup.ps1 \"what did we already decide about this?\" before memory-dependent answers.'\n",
        );
    }
    script.push_str(&format!(
        "memd wake --output $env:MEMD_BUNDLE_ROOT --route {} --intent {} --write\n",
        escape_ps1(&startup_route),
        escape_ps1(&startup_intent)
    ));
    script
}

pub(crate) fn render_lookup_shell_profile(output: &Path, kinds: &[&str], tags: &[&str]) -> String {
    let mut script = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\n\n{}source \"$MEMD_BUNDLE_ROOT/backend.env\" 2>/dev/null || true\nsource \"$MEMD_BUNDLE_ROOT/env\"\n\nargs=(lookup --output \"$MEMD_BUNDLE_ROOT\" --route project_first --intent general)\n",
        bash_bundle_root_resolver(output),
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
        "{}$bundleBackendEnv = Join-Path $env:MEMD_BUNDLE_ROOT \"backend.env.ps1\"\nif (Test-Path $bundleBackendEnv) {{ . $bundleBackendEnv }}\n. (Join-Path $env:MEMD_BUNDLE_ROOT \"env.ps1\")\n$args = @(\"lookup\", \"--output\", $env:MEMD_BUNDLE_ROOT, \"--route\", \"project_first\", \"--intent\", \"general\")\n",
        ps1_bundle_root_resolver(output),
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
        "#!/usr/bin/env bash\nset -euo pipefail\n\n{}source \"$MEMD_BUNDLE_ROOT/backend.env\" 2>/dev/null || true\nsource \"$MEMD_BUNDLE_ROOT/env\"\n\nargs=(remember --output \"$MEMD_BUNDLE_ROOT\" --kind \"{}\" --scope project)\n",
        bash_bundle_root_resolver(output),
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

pub(crate) fn render_teach_shell_profile(output: &Path) -> String {
    format!(
        "#!/usr/bin/env bash\nset -euo pipefail\n\n{}source \"$MEMD_BUNDLE_ROOT/backend.env\" 2>/dev/null || true\nsource \"$MEMD_BUNDLE_ROOT/env\"\n\nmemd_cmd=\"${{MEMD_BIN:-memd}}\"\nif [[ -z \"${{MEMD_BIN:-}}\" ]]; then\ncandidate=\"$MEMD_BUNDLE_ROOT/../target/debug/memd\"\n[[ -x \"$candidate\" ]] && memd_cmd=\"$candidate\"\nfi\nargs=(teach --output \"$MEMD_BUNDLE_ROOT\")\nexec \"$memd_cmd\" \"${{args[@]}}\" \"$@\"\n",
        bash_bundle_root_resolver(output),
    )
}

pub(crate) fn render_remember_ps1_profile(output: &Path, kind: &str, tags: &[&str]) -> String {
    let mut script = format!(
        "{}$bundleBackendEnv = Join-Path $env:MEMD_BUNDLE_ROOT \"backend.env.ps1\"\nif (Test-Path $bundleBackendEnv) {{ . $bundleBackendEnv }}\n. (Join-Path $env:MEMD_BUNDLE_ROOT \"env.ps1\")\n$args = @(\"remember\", \"--output\", $env:MEMD_BUNDLE_ROOT, \"--kind\", \"{}\", \"--scope\", \"project\")\n",
        ps1_bundle_root_resolver(output),
        escape_ps1(kind),
    );
    for tag in tags {
        script.push_str(&format!("$args += @(\"--tag\", \"{}\")\n", escape_ps1(tag)));
    }
    script.push_str("memd @args @Args\n");
    script
}

pub(crate) fn render_teach_ps1_profile(output: &Path) -> String {
    format!(
        "{}$bundleBackendEnv = Join-Path $env:MEMD_BUNDLE_ROOT \"backend.env.ps1\"\nif (Test-Path $bundleBackendEnv) {{ . $bundleBackendEnv }}\n. (Join-Path $env:MEMD_BUNDLE_ROOT \"env.ps1\")\n$memdCmd = if ($env:MEMD_BIN) {{ $env:MEMD_BIN }} else {{ \"memd\" }}\nif (-not $env:MEMD_BIN) {{\n$candidate = Join-Path (Split-Path -Parent $env:MEMD_BUNDLE_ROOT) \"target/debug/memd\"\nif (Test-Path $candidate) {{ $memdCmd = $candidate }}\n}}\n$args = @(\"teach\", \"--output\", $env:MEMD_BUNDLE_ROOT)\n& $memdCmd @args @Args\n",
        ps1_bundle_root_resolver(output),
    )
}

pub(crate) fn render_capture_shell_profile(output: &Path, mode: &str) -> String {
    let mut script = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\n\n{}source \"$MEMD_BUNDLE_ROOT/backend.env\" 2>/dev/null || true\nsource \"$MEMD_BUNDLE_ROOT/env\"\n\nargs=(hook capture --output \"$MEMD_BUNDLE_ROOT\" --summary)\n",
        bash_bundle_root_resolver(output),
    );
    if mode == "capture-live" {
        script
            .push_str("args+=(--tag basic-memory --tag live-capture --promote-kind live_truth)\n");
    } else {
        script.push_str("args+=(--tag basic-memory --tag correction)\n");
    }
    script.push_str("exec memd \"${args[@]}\" \"$@\"\n");
    script
}

pub(crate) fn render_capture_ps1_profile(output: &Path, mode: &str) -> String {
    let mut script = format!(
        "{}$bundleBackendEnv = Join-Path $env:MEMD_BUNDLE_ROOT \"backend.env.ps1\"\nif (Test-Path $bundleBackendEnv) {{ . $bundleBackendEnv }}\n. (Join-Path $env:MEMD_BUNDLE_ROOT \"env.ps1\")\n$args = @(\"hook\", \"capture\", \"--output\", $env:MEMD_BUNDLE_ROOT, \"--summary\")\n",
        ps1_bundle_root_resolver(output),
    );
    if mode == "capture-live" {
        script.push_str("$args += @(\"--tag\", \"basic-memory\", \"--tag\", \"live-capture\", \"--promote-kind\", \"live_truth\")\n");
    } else {
        script.push_str("$args += @(\"--tag\", \"basic-memory\", \"--tag\", \"correction\")\n");
    }
    script.push_str("memd @args @Args\n");
    script
}

pub(crate) fn render_checkpoint_shell_profile(output: &Path) -> String {
    format!(
        "#!/usr/bin/env bash\nset -euo pipefail\n\n{}source \"$MEMD_BUNDLE_ROOT/backend.env\" 2>/dev/null || true\nsource \"$MEMD_BUNDLE_ROOT/env\"\n\nargs=(checkpoint --output \"$MEMD_BUNDLE_ROOT\" --tag basic-memory --tag short-term)\nexec memd \"${{args[@]}}\" \"$@\"\n",
        bash_bundle_root_resolver(output),
    )
}

pub(crate) fn render_checkpoint_ps1_profile(output: &Path) -> String {
    format!(
        "{}$bundleBackendEnv = Join-Path $env:MEMD_BUNDLE_ROOT \"backend.env.ps1\"\nif (Test-Path $bundleBackendEnv) {{ . $bundleBackendEnv }}\n. (Join-Path $env:MEMD_BUNDLE_ROOT \"env.ps1\")\n$args = @(\"checkpoint\", \"--output\", $env:MEMD_BUNDLE_ROOT, \"--tag\", \"basic-memory\", \"--tag\", \"short-term\")\nmemd @args @Args\n",
        ps1_bundle_root_resolver(output),
    )
}

pub(crate) fn render_rag_sync_shell_profile(output: &Path) -> String {
    format!(
        "#!/usr/bin/env bash\nset -euo pipefail\n\n{}source \"$MEMD_BUNDLE_ROOT/backend.env\" 2>/dev/null || true\nsource \"$MEMD_BUNDLE_ROOT/env\"\n\nargs=(rag sync)\n[[ -n \"${{MEMD_PROJECT:-}}\" ]] && args+=(--project \"$MEMD_PROJECT\")\n[[ -n \"${{MEMD_NAMESPACE:-}}\" ]] && args+=(--namespace \"$MEMD_NAMESPACE\")\nexec memd \"${{args[@]}}\" \"$@\"\n",
        bash_bundle_root_resolver(output),
    )
}

pub(crate) fn render_rag_sync_ps1_profile(output: &Path) -> String {
    format!(
        "{}$bundleBackendEnv = Join-Path $env:MEMD_BUNDLE_ROOT \"backend.env.ps1\"\nif (Test-Path $bundleBackendEnv) {{ . $bundleBackendEnv }}\n. (Join-Path $env:MEMD_BUNDLE_ROOT \"env.ps1\")\n$args = @(\"rag\", \"sync\")\nif ($env:MEMD_PROJECT) {{ $args += @(\"--project\", $env:MEMD_PROJECT) }}\nif ($env:MEMD_NAMESPACE) {{ $args += @(\"--namespace\", $env:MEMD_NAMESPACE) }}\nmemd @args @Args\n",
        ps1_bundle_root_resolver(output),
    )
}

pub(crate) fn render_watch_shell_profile(output: &Path) -> String {
    format!(
        "#!/usr/bin/env bash\nset -euo pipefail\n\n{}source \"$MEMD_BUNDLE_ROOT/backend.env\" 2>/dev/null || true\nsource \"$MEMD_BUNDLE_ROOT/env\"\nproject_root=\"$(cd \"$MEMD_BUNDLE_ROOT/..\" && pwd)\"\nexec memd watch --root \"$project_root\" --output \"$MEMD_BUNDLE_ROOT\" \"$@\"\n",
        bash_bundle_root_resolver(output),
    )
}

pub(crate) fn render_watch_ps1_profile(output: &Path) -> String {
    format!(
        "{}$bundleBackendEnv = Join-Path $env:MEMD_BUNDLE_ROOT \"backend.env.ps1\"\nif (Test-Path $bundleBackendEnv) {{ . $bundleBackendEnv }}\n. (Join-Path $env:MEMD_BUNDLE_ROOT \"env.ps1\")\n$projectRoot = Split-Path -Parent $env:MEMD_BUNDLE_ROOT\nmemd watch --root $projectRoot --output $env:MEMD_BUNDLE_ROOT @Args\n",
        ps1_bundle_root_resolver(output),
    )
}
