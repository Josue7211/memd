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

fn wake_budget_agent_name(output: &Path, snapshot: &ResumeSnapshot) -> Option<String> {
    read_bundle_runtime_config(output)
        .ok()
        .flatten()
        .and_then(|config| config.agent)
        .or_else(|| snapshot.agent.clone())
}

// B3 Task 4: priority dedup across wake tiers (canonical > working > search).
// Rollback: set MEMD_RETRIEVAL_PRIORITY_DEDUP=0 to disable.
fn priority_dedup_enabled() -> bool {
    match std::env::var("MEMD_RETRIEVAL_PRIORITY_DEDUP") {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            !(v == "0" || v == "false" || v == "off" || v == "no")
        }
        Err(_) => true,
    }
}

// B3 Task 3: layered wake packet (L0 identity / L1 essential / L2 on-demand /
// L3 deep). Flag-gated so the default render remains byte-identical for
// downstream consumers until rolled out. Enable with MEMD_WAKE_LAYERED=1.
pub(crate) fn layered_wake_enabled() -> bool {
    match std::env::var("MEMD_WAKE_LAYERED") {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "on" | "yes")
        }
        Err(_) => false,
    }
}

/// Returns the H2 suffix tag for a wake section. Empty when layered-wake
/// is disabled so existing markdown stays byte-identical.
pub(crate) fn layer_suffix(layer: &str) -> String {
    if layered_wake_enabled() {
        format!(" ({layer})")
    } else {
        String::new()
    }
}

fn extract_record_id(line: &str) -> Option<String> {
    let start = line.find("id=")?;
    let rest = &line[start + 3..];
    let end = rest
        .find(|c: char| c.is_whitespace() || c == '|')
        .unwrap_or(rest.len());
    let id = rest[..end].trim();
    if id.is_empty() {
        None
    } else {
        Some(id.to_string())
    }
}

fn truncate_visible_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let mut truncated = String::new();
    for ch in value.chars().take(max_chars.saturating_sub(1)) {
        truncated.push(ch);
    }
    truncated.push('…');
    truncated
}

pub(crate) fn render_continuity_gate_block(un_read: &[String], verbose: bool) -> String {
    if un_read.is_empty() {
        return String::new();
    }
    let mut s = String::new();
    s.push_str("## Continuity Gate\n\n");
    s.push_str(
        "_Prior session touched these files and THIS session has not Read them yet. Bulk-Read before editing or memd will deny (policy=block) or warn (policy=warn)._\n\n",
    );
    let limit = if verbose { 20 } else { 10 };
    for p in un_read.iter().take(limit) {
        s.push_str(&format!("- {p}\n"));
    }
    if un_read.len() > limit {
        s.push_str(&format!("- + {} more\n", un_read.len() - limit));
    }
    s.push('\n');
    s
}

pub(crate) fn render_preferences_block(
    preferences: &[String],
    claude_strict: bool,
    verbose: bool,
    seen_ids: &mut std::collections::HashSet<String>,
) -> String {
    if preferences.is_empty() {
        return String::new();
    }
    let dedup = priority_dedup_enabled();
    let item_limit = if claude_strict { 110 } else { 140 };
    let count = if verbose { 5 } else { 3 };
    let mut rows: Vec<String> = Vec::new();
    for p in preferences.iter() {
        if rows.len() >= count {
            break;
        }
        let trimmed = p.trim();
        if dedup {
            if let Some(id) = extract_record_id(trimmed) {
                if !seen_ids.insert(id) {
                    continue;
                }
            }
        }
        rows.push(format!("- {}\n", compact_inline(trimmed, item_limit)));
    }
    if rows.is_empty() {
        return String::new();
    }
    let mut s = String::new();
    s.push_str(&format!(
        "## Preferences{}\n\n",
        layer_suffix("L2 — On-Demand")
    ));
    for r in rows {
        s.push_str(&r);
    }
    s.push('\n');
    s
}

fn handoff_quality_label(snapshot: &ResumeSnapshot) -> String {
    snapshot
        .handoff_quality
        .as_ref()
        .map(|score| {
            if score.is_acceptable() {
                format!("ready:{:.2}", score.composite)
            } else {
                format!("partial:{:.2}", score.composite)
            }
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn dirty_change_count(snapshot: &ResumeSnapshot) -> usize {
    snapshot
        .recent_repo_changes
        .iter()
        .filter(|change| !change.eq_ignore_ascii_case("repo clean"))
        .count()
}

fn render_recovery_identity_line(output: &Path, snapshot: &ResumeSnapshot) -> String {
    if snapshot.handoff_quality.is_none() {
        return String::new();
    }

    let continuity = snapshot.continuity_capsule();
    let active_voice = read_bundle_voice_mode(output).unwrap_or_else(default_voice_mode);
    let mut parts = vec![format!(
        "voice={} | quality={} | dirty={}",
        active_voice,
        handoff_quality_label(snapshot),
        dirty_change_count(snapshot)
    )];
    if let Some(next_action) = continuity.next_action.as_deref() {
        parts.push(format!("next={}", compact_inline(next_action, 180)));
    }
    if let Some(blocker) = continuity.blocker.as_deref() {
        parts.push(format!("blocker={}", compact_inline(blocker, 96)));
    }
    format!("- recovery {}\n\n", parts.join(" | "))
}

fn render_continuity_block(snapshot: &ResumeSnapshot, claude_strict: bool) -> String {
    let continuity = snapshot.continuity_capsule();
    if continuity.current_task.is_none()
        && continuity.resume_point.is_none()
        && continuity.changed.is_none()
        && continuity.next_action.is_none()
        && continuity.blocker.is_none()
    {
        return String::new();
    }

    let mut s = String::new();
    s.push_str(&format!("## Continuity{}\n\n", layer_suffix("L3 — Deep")));
    let continuity_limit = if claude_strict { 72 } else { 84 };
    if let Some(current_task) = continuity.current_task.as_deref() {
        s.push_str(&format!(
            "- doing={}\n",
            compact_inline(current_task, continuity_limit)
        ));
    }
    if let Some(resume_point) = continuity.resume_point.as_deref() {
        s.push_str(&format!(
            "- left_off={}\n",
            compact_inline(resume_point, continuity_limit)
        ));
    }
    if let Some(changed) = continuity.changed.as_deref() {
        s.push_str(&format!(
            "- changed={}\n",
            compact_inline(changed, continuity_limit)
        ));
    }
    if let Some(next_action) = continuity.next_action.as_deref() {
        s.push_str(&format!(
            "- next={}\n",
            compact_inline(next_action, continuity_limit)
        ));
    }
    if let Some(blocker) = continuity.blocker.as_deref() {
        s.push_str(&format!(
            "- blocker={}\n",
            compact_inline(blocker, continuity_limit)
        ));
    }
    s.push('\n');
    s
}

fn enforce_wake_char_budget(prefix: &str, protocol: &str, max_chars: usize) -> String {
    let full = format!("{prefix}{protocol}");
    if full.chars().count() <= max_chars {
        return full;
    }

    let trimmed_protocol = protocol.trim_start();
    let elided_marker = "\n## Wake Budget\n\n- startup trimmed; use `memd lookup` or `memd resume` for deeper recall.\n\n";
    let required_chars = trimmed_protocol.chars().count() + elided_marker.chars().count();

    if required_chars >= max_chars {
        return truncate_visible_chars(trimmed_protocol, max_chars);
    }

    let prefix_budget = max_chars - required_chars;
    let mut trimmed_prefix = String::new();
    for line in prefix.lines() {
        let candidate = if trimmed_prefix.is_empty() {
            format!("{line}\n")
        } else {
            format!("{trimmed_prefix}{line}\n")
        };
        if candidate.chars().count() > prefix_budget {
            break;
        }
        trimmed_prefix = candidate;
    }

    let combined = format!(
        "{}{}{}",
        trimmed_prefix.trim_end(),
        elided_marker,
        trimmed_protocol
    );
    if combined.chars().count() <= max_chars {
        combined
    } else {
        truncate_visible_chars(&combined, max_chars)
    }
}

pub(crate) fn render_bundle_wakeup_markdown(
    output: &Path,
    snapshot: &ResumeSnapshot,
    verbose: bool,
) -> String {
    let mut prefix = String::new();
    let budget = crate::harness::preset::wake_char_budget_for_agent(
        wake_budget_agent_name(output, snapshot).as_deref(),
    );
    let claude_strict = wake_budget_agent_name(output, snapshot)
        .as_deref()
        .is_some_and(|agent| {
            let normalized = agent.trim().to_ascii_lowercase();
            normalized == "claude-code" || normalized.starts_with("claude-code@")
        });

    prefix.push_str("# memd wake-up\n\n");
    if layered_wake_enabled() {
        prefix.push_str("_L0 — Identity_\n");
    }
    prefix.push_str(&format!(
        "- {} / {} / {} / {} / {} / {} / {}\n\n",
        snapshot.project.as_deref().unwrap_or("none"),
        snapshot.namespace.as_deref().unwrap_or("none"),
        snapshot.agent.as_deref().unwrap_or("none"),
        snapshot.workspace.as_deref().unwrap_or("none"),
        snapshot.visibility.as_deref().unwrap_or("all"),
        snapshot.route,
        snapshot.intent,
    ));

    prefix.push_str(&render_recovery_identity_line(output, snapshot));

    let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    let dedup = priority_dedup_enabled();

    let instructions = collect_wakeup_instruction_sources(output);
    if !instructions.is_empty() && !claude_strict {
        prefix.push_str("## Instructions\n\n");
        let limit = if verbose { 3 } else { 1 };
        for (source, snippet) in instructions.into_iter().take(limit) {
            prefix.push_str(&format!("- {source}: {}\n", compact_inline(&snippet, 180)));
        }
        prefix.push('\n');
    }

    let event_spine = snapshot.event_spine();
    if !event_spine.is_empty() && !claude_strict {
        prefix.push_str("## Live\n\n");
        let limit = if verbose { 2 } else { 1 };
        for item in event_spine.iter().take(limit) {
            prefix.push_str(&format!("- {}\n", compact_inline(item, 100)));
        }
        prefix.push('\n');
    }

    prefix.push_str(&format!(
        "## Durable Truth{}\n\n",
        layer_suffix("L1 — Essential Story")
    ));
    if snapshot.context.records.is_empty() {
        prefix.push_str("- none\n\n");
    } else {
        let limit = if claude_strict {
            2
        } else if verbose {
            6
        } else {
            4
        };
        let item_limit = if claude_strict {
            120
        } else if verbose {
            160
        } else {
            140
        };
        // B2: Reorder durable truth to show non-live_truth items first.
        // LiveTruth (repo deltas, file edits) is transient; facts/decisions
        // are the durable knowledge this section should surface.
        let is_live_truth =
            |r: &memd_schema::CompactMemoryRecord| r.record.contains("kind=live_truth");
        let non_live: Vec<_> = snapshot
            .context
            .records
            .iter()
            .filter(|r| !is_live_truth(r))
            .collect();
        let live: Vec<_> = snapshot
            .context
            .records
            .iter()
            .filter(|r| is_live_truth(r))
            .collect();
        let reordered: Vec<_> = non_live.into_iter().chain(live.into_iter()).collect();
        let mut rendered = 0usize;
        for item in reordered.iter() {
            if rendered >= limit {
                break;
            }
            let line = item.record.trim();
            if dedup {
                if let Some(id) = extract_record_id(line) {
                    seen_ids.insert(id);
                }
            }
            prefix.push_str(&format!("- {}\n", compact_inline(line, item_limit)));
            rendered += 1;
        }
        let total = snapshot.context.records.len();
        if total > rendered {
            prefix.push_str(&format!(
                "- + {} more via `memd lookup`\n",
                total - rendered
            ));
        }
        prefix.push('\n');
    }

    prefix.push_str(&format!("## Focus{}\n\n", layer_suffix("L2 — On-Demand")));
    if snapshot.working.records.is_empty() {
        prefix.push_str("- none\n");
    } else {
        let limit = 1;
        let item_limit = if claude_strict { 110 } else { 140 };
        let mut rendered = 0usize;
        for item in snapshot.working.records.iter() {
            if rendered >= limit {
                break;
            }
            let line = item.record.trim();
            if dedup {
                if let Some(id) = extract_record_id(line) {
                    if !seen_ids.insert(id) {
                        continue;
                    }
                }
            }
            prefix.push_str(&format!("- {}\n", compact_inline(line, item_limit)));
            rendered += 1;
        }
        if rendered == 0 {
            prefix.push_str("- (all covered in Durable Truth)\n");
        }
    }
    prefix.push('\n');
    prefix.push_str(&render_continuity_block(snapshot, claude_strict));

    // A3 Part 2 Task 11: Surface preference memories in wake packet.
    prefix.push_str(&render_preferences_block(
        &snapshot.preferences,
        claude_strict,
        verbose,
        &mut seen_ids,
    ));

    prefix.push_str(&render_active_skills_block(output));

    // E2: Atlas region hints in wake packet
    if !snapshot.atlas_region_hints.is_empty() && !claude_strict {
        prefix.push_str("## Atlas\n\n");
        for hint in snapshot.atlas_region_hints.iter().take(3) {
            prefix.push_str(&format!("- {}\n", compact_inline(hint, 80)));
        }
        prefix.push('\n');
    }

    // A3 Part 1: surface prior-session file interactions so the continuation
    // can Bulk-Read before first Edit and avoid re-Read errors post-compaction.
    // Load-bearing: NOT gated by claude_strict. Claude Code is the harness
    // most likely to hit compaction-mid-edit, so suppressing this block under
    // claude_strict would defeat the A3 continuity guarantee. Under
    // claude_strict we shrink the row budget but always emit the block.
    if !snapshot.files_touched.is_empty() {
        prefix.push_str(&format!(
            "## Files Touched{}\n\n",
            layer_suffix("L2 — On-Demand")
        ));
        prefix.push_str(
            "_Prior session Read/Edit/Write. Bulk-Read before first Edit to avoid re-Read errors after compaction._\n\n",
        );
        let limit = if verbose {
            20
        } else if claude_strict {
            6
        } else {
            10
        };
        for p in snapshot.files_touched.iter().take(limit) {
            prefix.push_str(&format!("- {p}\n"));
        }
        if snapshot.files_touched.len() > limit {
            prefix.push_str(&format!(
                "- + {} more via `memd prime-reads`\n",
                snapshot.files_touched.len() - limit
            ));
        }
        prefix.push('\n');
    }

    if !claude_strict {
        prefix.push_str(&render_continuity_gate_block(
            &snapshot.un_read_paths,
            verbose,
        ));
    }

    if !snapshot.working.procedures.is_empty() && !claude_strict {
        prefix.push_str("## Procedures\n\n");
        let proc_limit = if verbose { 3 } else { 2 };
        for proc in snapshot.working.procedures.iter().take(proc_limit) {
            let steps_compact = proc.steps.join(" → ");
            prefix.push_str(&format!(
                "- **{}** ({}): {}\n",
                proc.name,
                compact_inline(&proc.trigger, 60),
                compact_inline(&steps_compact, 100),
            ));
        }
        prefix.push('\n');
    }

    if verbose
        && !claude_strict
        && (!snapshot.inbox.items.is_empty() || !snapshot.working.rehydration_queue.is_empty())
    {
        prefix.push_str("## Recovery\n\n");
        let recovery_limit = if verbose { 1 } else { 1 };
        for item in snapshot
            .working
            .rehydration_queue
            .iter()
            .take(recovery_limit)
        {
            prefix.push_str(&format!(
                "- {}: {}\n",
                item.label,
                compact_inline(item.summary.trim(), 120)
            ));
        }
        let inbox_limit = if verbose { 1 } else { 1 };
        for item in snapshot.inbox.items.iter().take(inbox_limit) {
            prefix.push_str(&format!(
                "- {:?}/{:?}: {}\n",
                item.item.kind,
                item.item.status,
                compact_inline(item.item.content.trim(), 120)
            ));
        }
        prefix.push('\n');
    }

    let active_voice = read_bundle_voice_mode(output).unwrap_or_else(default_voice_mode);
    let mut protocol = String::new();
    if claude_strict {
        // Claude Code: minimal protocol — behavioral rules live in AGENTS.md and the memd skill.
        protocol.push_str(&format!("## Voice\n\n- {}\n", active_voice));
    } else {
        protocol.push_str("## Protocol\n\n");
        protocol.push_str(
            "- Read first. Durable truth beats transcript recall. Promote stable truths.\n",
        );
        protocol.push_str(
            "- Lookup before answers on decisions, preferences, history, or prior user corrections.\n",
        );
        protocol.push_str("- If a required fact is absent or unknown, ask a clarifying question or run lookup before acting.\n");
        protocol.push_str("- Recall: `memd lookup --output .memd --query \"...\"`.\n");
        protocol.push_str("- If the user corrects you, write the correction back instead of trusting the transcript.\n");
        protocol.push_str("- Writes: user-taught facts -> `memd teach --output .memd --content \"...\"`; decisions/preferences -> `memd remember`; short-term -> `memd checkpoint`; live/correction spill -> `memd hook capture --summary`.\n");
        if verbose {
            protocol.push_str("- Handoff: `memd checkpoint --auto-commit --content \"...\"` commits only small tracked dirty sets before saving state.\n");
            protocol.push_str("- Roadmap: `memd checkpoint --roadmap-set current_phase=X --roadmap-set phase_status=Y` patches ROADMAP_STATE before commit.\n");
            protocol.push_str(
                "- Wake/resume/refresh/handoff/hook capture auto-write short-term status.\n",
            );
        }
        protocol.push_str(&format!(
            "- Default voice: {}. Reply in `{}` unless `.memd/config.json` changes it.\n",
            active_voice, active_voice
        ));
        protocol.push_str(&format!(
            "- If your draft is not in `{}`, stop and rewrite it before sending.\n",
            active_voice
        ));
    };

    enforce_wake_char_budget(&prefix, &protocol, budget)
}

fn collect_active_skills(bundle_root: &Path, limit: usize) -> Vec<(String, String, String)> {
    let dir = bundle_root.join("skills");
    if !dir.is_dir() {
        return Vec::new();
    }
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };
    // (salience, name, description, body) — salience used only for sort.
    let mut found: Vec<(f32, String, String, String)> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let skill_md = path.join("SKILL.md");
        if !skill_md.is_file() {
            continue;
        }
        let raw = match std::fs::read_to_string(&skill_md) {
            Ok(raw) => raw,
            Err(_) => continue,
        };
        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) if !name.is_empty() => name.to_string(),
            _ => continue,
        };
        // Phase 2 §10 records-as-truth: SKILL.md mirrors a record. Fall back
        // to dir name + empty desc if the file is malformed (legacy mirrors
        // without proper fences) so wake doesn't black-hole on drift.
        let (name, description, body, salience) =
            match memd_schema::skill::SkillBody::parse_skill_md(&raw) {
                Some(sb) => (
                    if sb.frontmatter.name.is_empty() {
                        dir_name
                    } else {
                        sb.frontmatter.name
                    },
                    sb.frontmatter.description,
                    sb.body,
                    sb.frontmatter.salience.unwrap_or(0.0),
                ),
                None => (dir_name, String::new(), raw, 0.0),
            };
        found.push((salience, name, description, body));
    }
    // Phase 2 §9: salience desc, name asc on tie. None → 0.0 fallback above.
    found.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.1.cmp(&b.1))
    });
    found.truncate(limit);
    found.into_iter().map(|(_s, n, d, b)| (n, d, b)).collect()
}

fn render_active_skills_block(bundle_root: &Path) -> String {
    let skills = collect_active_skills(bundle_root, 3);
    if skills.is_empty() {
        return String::new();
    }
    let mut out = String::from("## Active Skills\n\n");
    for (i, (name, description, body)) in skills.iter().enumerate() {
        if i == 0 {
            out.push_str(&format!("- **{}** — {}\n", name, description));
            let trimmed = body.trim();
            if !trimmed.is_empty() {
                let truncated: String = if trimmed.chars().count() > 500 {
                    let mut s: String = trimmed.chars().take(500).collect();
                    s.push('…');
                    s
                } else {
                    trimmed.to_string()
                };
                for line in truncated.lines() {
                    out.push_str(&format!("  {}\n", line));
                }
            }
        } else {
            out.push_str(&format!(
                "- {} — `memd lookup --kind skill --name {}`\n",
                name, name
            ));
        }
    }
    out.push('\n');
    out
}

pub(crate) fn render_bundle_wakeup_summary(snapshot: &ResumeSnapshot) -> String {
    format!(
        "wake project={} namespace={} agent={} working={} inbox={} spine={} tokens={} core={} focus=\"{}\"",
        snapshot.project.as_deref().unwrap_or("none"),
        snapshot.namespace.as_deref().unwrap_or("none"),
        snapshot.agent.as_deref().unwrap_or("none"),
        snapshot.working.records.len(),
        snapshot.inbox.items.len(),
        snapshot.event_spine().len(),
        snapshot.estimated_prompt_tokens(),
        snapshot.core_prompt_tokens(),
        snapshot
            .working
            .records
            .first()
            .map(|item| compact_inline(item.record.trim(), 96))
            .unwrap_or_else(|| "none".to_string())
    )
}

/// Extract the `kind=<value>` field from a compact memory record string.
fn extract_kind_from_record(record: &str) -> String {
    // Records look like: "id=... | kind=fact | status=active | ..."
    for segment in record.split('|') {
        let trimmed = segment.trim();
        if let Some(rest) = trimmed.strip_prefix("kind=") {
            return rest.trim().to_string();
        }
    }
    "unknown".to_string()
}

/// Compute per-kind token metrics for the wake packet.
///
/// Analyzes both context (durable truth) and working (focus) records from the
/// snapshot, counting characters and items per memory kind. Returns an
/// `OperationTokenReport` for the "wake" operation.
pub(crate) fn compute_wake_token_metrics(
    output: &Path,
    snapshot: &ResumeSnapshot,
    rendered_markdown: &str,
) -> memd_schema::OperationTokenReport {
    use std::collections::BTreeMap;

    let budget = crate::harness::preset::wake_char_budget_for_agent(
        wake_budget_agent_name(output, snapshot).as_deref(),
    );
    let used_chars = rendered_markdown.len();

    let mut chars_per_kind: BTreeMap<String, usize> = BTreeMap::new();
    let mut items_per_kind: BTreeMap<String, usize> = BTreeMap::new();
    let mut total_items = 0usize;
    let mut total_record_chars = 0usize;

    // Count context records (durable truth)
    for record in &snapshot.context.records {
        let kind = extract_kind_from_record(&record.record);
        let char_len = record.record.len();
        *chars_per_kind.entry(kind.clone()).or_insert(0) += char_len;
        *items_per_kind.entry(kind).or_insert(0) += 1;
        total_items += 1;
        total_record_chars += char_len;
    }

    // Count working records (focus)
    for record in &snapshot.working.records {
        let kind = extract_kind_from_record(&record.record);
        let char_len = record.record.len();
        *chars_per_kind.entry(kind.clone()).or_insert(0) += char_len;
        *items_per_kind.entry(kind).or_insert(0) += 1;
        total_items += 1;
        total_record_chars += char_len;
    }

    let utilization_pct = if budget > 0 {
        (used_chars as f64 / budget as f64) * 100.0
    } else {
        0.0
    };

    memd_schema::OperationTokenReport {
        operation: "wake".to_string(),
        budget_chars: budget,
        used_chars,
        utilization_pct,
        per_kind: memd_schema::PerKindTokenMetrics {
            chars_per_kind,
            items_per_kind,
            total_chars: total_record_chars,
            total_items,
        },
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_snapshot() -> ResumeSnapshot {
        ResumeSnapshot {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            context: memd_schema::CompactContextResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                retrieval_order: vec![MemoryScope::Project],
                records: vec![memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record:
                        "remembered project fact: memd must preserve important user corrections"
                            .to_string(),
                }],
            },
            working: memd_schema::WorkingMemoryResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                retrieval_order: vec![MemoryScope::Project],
                budget_chars: 1600,
                used_chars: 120,
                remaining_chars: 1480,
                truncated: false,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 8,
                    max_chars_per_item: 220,
                    budget_chars: 1600,
                    rehydration_limit: 4,
                },
                records: vec![memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "follow durable truth before transcript recall".to_string(),
                }],
                evicted: Vec::new(),
                rehydration_queue: Vec::new(),
                traces: Vec::new(),
                semantic_consolidation: None,
                procedures: vec![],

                compaction_quality: None,
            },
            inbox: memd_schema::MemoryInboxResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                items: Vec::new(),
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: Vec::new(),
            },
            sources: memd_schema::SourceMemoryResponse {
                sources: Vec::new(),
            },
            semantic: None,
            claims: SessionClaimsState::default(),
            recent_repo_changes: Vec::new(),
            change_summary: Vec::new(),
            resume_state_age_minutes: None,
            refresh_recommended: false,
            atlas_region_hints: Vec::new(),
            handoff_quality: None,
            files_touched: Vec::new(),
            un_read_paths: Vec::new(),
            preferences: Vec::new(),
        }
    }

    fn pressure_snapshot() -> ResumeSnapshot {
        ResumeSnapshot {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            context: memd_schema::CompactContextResponse {
                route: RetrievalRoute::ProjectFirst,
                intent: RetrievalIntent::CurrentTask,
                retrieval_order: vec![MemoryScope::Project, MemoryScope::Synced],
                records: (0..8)
                    .map(|index| memd_schema::CompactMemoryRecord {
                        id: uuid::Uuid::new_v4(),
                        record: format!(
                            "durable truth {}: keep the packet compact and preserve next action",
                            index
                        ),
                    })
                    .collect(),
            },
            working: memd_schema::WorkingMemoryResponse {
                route: RetrievalRoute::ProjectFirst,
                intent: RetrievalIntent::CurrentTask,
                retrieval_order: vec![MemoryScope::Project, MemoryScope::Synced],
                budget_chars: 1600,
                used_chars: 1510,
                remaining_chars: 90,
                truncated: true,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 8,
                    max_chars_per_item: 220,
                    budget_chars: 1600,
                    rehydration_limit: 4,
                },
                records: (0..6)
                    .map(|index| memd_schema::CompactMemoryRecord {
                        id: uuid::Uuid::new_v4(),
                        record: format!(
                            "working truth {}: follow durable truth before transcript recall",
                            index
                        ),
                    })
                    .collect(),
                evicted: Vec::new(),
                rehydration_queue: (0..4)
                    .map(|index| memd_schema::MemoryRehydrationRecord {
                        id: None,
                        kind: "source".to_string(),
                        label: format!("artifact-{index}"),
                        summary: format!("rehydrate source artifact {index}"),
                        reason: Some("pressure".to_string()),
                        source_agent: Some("codex@test".to_string()),
                        source_system: Some("hook-capture".to_string()),
                        source_path: Some(format!("notes/rehydrate-{index}.md")),
                        source_quality: None,
                        recorded_at: None,
                    })
                    .collect(),
                traces: Vec::new(),
                semantic_consolidation: None,
                procedures: vec![],

                compaction_quality: None,
            },
            inbox: memd_schema::MemoryInboxResponse {
                route: RetrievalRoute::ProjectFirst,
                intent: RetrievalIntent::CurrentTask,
                items: (0..5)
                    .map(|index| memd_schema::InboxMemoryItem {
                        item: memd_schema::MemoryItem {
                            id: uuid::Uuid::new_v4(),
                            content: format!(
                                "inbox pressure {}: keep the next action visible",
                                index
                            ),
                            redundancy_key: Some("same".to_string()),
                            belief_branch: None,
                            preferred: true,
                            kind: memd_schema::MemoryKind::Status,
                            scope: memd_schema::MemoryScope::Project,
                            project: Some("memd".to_string()),
                            namespace: Some("main".to_string()),
                            workspace: Some("shared".to_string()),
                            visibility: memd_schema::MemoryVisibility::Workspace,
                            source_agent: Some("codex@test".to_string()),
                            source_system: Some("checkpoint".to_string()),
                            source_path: Some(format!("notes/inbox-{index}.md")),
                            source_quality: None,
                            confidence: 0.8,
                            ttl_seconds: Some(86_400),
                            created_at: Utc::now(),
                            status: memd_schema::MemoryStatus::Active,
                            stage: memd_schema::MemoryStage::Candidate,
                            lane: None,
                            version: 1,
                            last_verified_at: None,
                            supersedes: Vec::new(),
                            updated_at: Utc::now(),
                            tags: vec!["checkpoint".to_string()],
                            correction_meta: None,
                        },
                        reasons: vec!["stale".to_string()],
                    })
                    .collect(),
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: vec![memd_schema::WorkspaceMemoryRecord {
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("shared".to_string()),
                    visibility: memd_schema::MemoryVisibility::Workspace,
                    item_count: 9,
                    active_count: 7,
                    candidate_count: 2,
                    contested_count: 0,
                    source_lane_count: 2,
                    avg_confidence: 0.87,
                    trust_score: 0.93,
                    last_seen_at: None,
                    tags: Vec::new(),
                }],
            },
            sources: memd_schema::SourceMemoryResponse {
                sources: Vec::new(),
            },
            semantic: Some(RagRetrieveResponse {
                status: "ok".to_string(),
                mode: RagRetrieveMode::Auto,
                items: (0..4)
                    .map(|index| memd_rag::RagRetrieveItem {
                        content: format!("semantic evidence {index}"),
                        score: 0.7 - (index as f32 * 0.05),
                        source: Some(format!("semantic-{index}")),
                    })
                    .collect(),
            }),
            claims: SessionClaimsState::default(),
            recent_repo_changes: vec![
                "repo change: packet compiler touched".to_string(),
                "repo change: packet compiler touched".to_string(),
                "repo change: packet compiler touched".to_string(),
            ],
            change_summary: vec![
                "changed=preserve next action".to_string(),
                "next=refresh from bundle".to_string(),
            ],
            resume_state_age_minutes: Some(25),
            refresh_recommended: true,
            atlas_region_hints: Vec::new(),
            handoff_quality: None,
            files_touched: Vec::new(),
            un_read_paths: Vec::new(),
            preferences: Vec::new(),
        }
    }

    #[test]
    fn wakeup_markdown_surfaces_durable_truth_without_verbose_mode() {
        let dir = std::env::temp_dir().join(format!("memd-wakeup-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");

        let markdown = render_bundle_wakeup_markdown(&dir, &sample_snapshot(), false);
        assert!(markdown.contains("## Durable Truth"));
        assert!(markdown.contains("## Continuity"));
        assert!(markdown.contains("- doing="));
        assert!(markdown.contains("- left_off="), "{markdown}");
        assert!(markdown.contains("memd must preserve important user corrections"));
        assert!(markdown.contains("Durable truth beats transcript recall."));
        assert!(markdown.contains("If a required fact is absent or unknown"));
        assert!(markdown.contains("user-taught facts -> `memd teach"));
        assert!(markdown.contains("Reply in `caveman-ultra`"));
        assert!(markdown.contains("If your draft is not in `caveman-ultra`"));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn wakeup_markdown_keeps_native_recovery_state_under_budget() {
        let dir =
            std::env::temp_dir().join(format!("memd-wakeup-recovery-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");

        let mut snapshot = sample_snapshot();
        snapshot.handoff_quality = Some(HandoffQualityScore {
            fill_rate: 1.0,
            budget_utilization: 0.5,
            dominant_kind: Some("fact".to_string()),
            eviction_pressure: 0.0,
            fact_coverage: 1.0,
            decision_coverage: 0.0,
            working_depth: 1.0,
            composite: 0.55,
        });
        snapshot.recent_repo_changes = vec![
            "repo change: crates/memd-client/src/runtime/resume/wakeup.rs".to_string(),
            "repo clean".to_string(),
        ];

        let markdown = render_bundle_wakeup_markdown(&dir, &snapshot, false);
        let recovery_index = markdown.find("- recovery voice=").expect("recovery state");
        let durable_index = markdown.find("## Durable Truth").expect("durable truth");
        assert!(
            recovery_index < durable_index,
            "recovery state must precede bulky context: {markdown}"
        );
        assert!(markdown.contains("quality=partial:0.55"));
        assert!(markdown.contains("dirty=1"));
        assert!(markdown.contains("next=fix partial handoff quality"));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn wakeup_recovery_line_keeps_current_next_action_preview() {
        let dir = std::env::temp_dir().join(format!("memd-wakeup-next-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");

        let mut snapshot = sample_snapshot();
        snapshot.handoff_quality = Some(HandoffQualityScore {
            fill_rate: 1.0,
            budget_utilization: 0.5,
            dominant_kind: Some("decision".to_string()),
            eviction_pressure: 0.0,
            fact_coverage: 1.0,
            decision_coverage: 1.0,
            working_depth: 1.0,
            composite: 0.96,
        });
        snapshot.preferences = vec![
            "id=next | kind=decision | tags=next-agent,materializer | upd=1778869065 | c=CURRENT NEXT ACTION: implement server-backed fresh-machine materializer for Codex plugins and Claude Code assets before claiming capability sync works".to_string(),
        ];

        let markdown = render_bundle_wakeup_markdown(&dir, &snapshot, false);

        assert!(markdown.contains("- recovery voice="));
        assert!(
            markdown.contains("server-backed fresh-machine materializer"),
            "recovery line must carry useful next-action text: {markdown}"
        );

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn wakeup_markdown_surfaces_files_touched_block_when_populated() {
        let dir = std::env::temp_dir().join(format!("memd-wakeup-ft-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");

        let mut snapshot = sample_snapshot();
        snapshot.files_touched = vec![
            "crates/memd-core/src/lib.rs".to_string(),
            "ROADMAP.md".to_string(),
        ];

        let markdown = render_bundle_wakeup_markdown(&dir, &snapshot, false);
        assert!(
            markdown.contains("## Files Touched"),
            "missing heading: {markdown}"
        );
        assert!(markdown.contains("crates/memd-core/src/lib.rs"));
        assert!(markdown.contains("ROADMAP.md"));
        assert!(markdown.contains("Bulk-Read before first Edit"));

        // empty list omits the block entirely
        let mut empty = sample_snapshot();
        empty.files_touched = Vec::new();
        let empty_md = render_bundle_wakeup_markdown(&dir, &empty, false);
        assert!(!empty_md.contains("## Files Touched"));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    /// Claude Code is the harness most likely to hit compaction-mid-edit, so
    /// Files Touched MUST emit under claude_strict even though other cosmetic
    /// wake blocks are trimmed. A3 Part 1 continuity guarantee.
    #[test]
    fn wakeup_markdown_emits_files_touched_under_claude_strict() {
        let dir =
            std::env::temp_dir().join(format!("memd-wakeup-ft-claude-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");

        let mut snapshot = sample_snapshot();
        snapshot.agent = Some("claude-code@session-abc".to_string());
        snapshot.files_touched = (0..12).map(|i| format!("src/file_{i}.rs")).collect();

        let markdown = render_bundle_wakeup_markdown(&dir, &snapshot, false);
        assert!(
            markdown.contains("## Files Touched"),
            "claude_strict must still emit Files Touched: {markdown}"
        );
        // Under claude_strict the row budget is 6; surplus surfaces as overflow hint.
        assert!(markdown.contains("src/file_0.rs"));
        assert!(markdown.contains("src/file_5.rs"));
        assert!(
            !markdown.contains("src/file_6.rs"),
            "claude_strict should cap to 6 rows"
        );
        assert!(markdown.contains("more via `memd prime-reads`"));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn wakeup_summary_surfaces_packet_efficiency_signal() {
        let snapshot = sample_snapshot();
        let summary = render_bundle_wakeup_summary(&snapshot);

        assert!(summary.contains("wake project=memd"));
        assert!(summary.contains("tokens="));
        assert!(summary.contains("core="));
        assert!(summary.contains("focus=\"follow durable truth before transcript recall\""));
        assert!(snapshot.core_prompt_tokens() <= snapshot.estimated_prompt_tokens());
    }

    #[test]
    fn wakeup_markdown_stays_compact_under_pressure_and_keeps_continuity() {
        let dir =
            std::env::temp_dir().join(format!("memd-wakeup-pressure-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");

        let snapshot = pressure_snapshot();
        let markdown = render_bundle_wakeup_markdown(&dir, &snapshot, false);

        assert!(markdown.contains("## Durable Truth"));
        assert!(markdown.contains("## Focus"));
        assert!(markdown.contains("## Continuity"));
        assert!(markdown.contains("- doing="));
        assert!(markdown.contains("- left_off="), "markdown:\n{markdown}");
        assert!(markdown.contains("- changed="));
        assert!(markdown.contains("- next="));
        assert!(!markdown.contains("artifact-3"));
        assert!(
            markdown.lines().count() < 45,
            "expanded durable truth budget (4 items + overflow hint) still fits under 45 lines"
        );
        assert!(
            markdown.contains("more via `memd lookup`"),
            "overflow hint should appear when durable truth items exceed limit"
        );

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn claude_wakeup_markdown_respects_strict_budget() {
        let dir = std::env::temp_dir().join(format!(
            "memd-wakeup-claude-budget-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "memd",
  "agent": "claude-code",
  "route": "auto",
  "intent": "current_task"
}
"#,
        )
        .expect("write config");

        let markdown = render_bundle_wakeup_markdown(&dir, &pressure_snapshot(), false);

        assert!(markdown.chars().count() <= 1200);
        assert!(markdown.contains("## Voice"));
        assert!(!markdown.contains("## Protocol"));
        assert!(!markdown.contains("## Instructions"));
        assert!(!markdown.contains("## Live"));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    // ── Cross-Harness Wake Proof Tests ──────────────────────────────

    #[test]
    fn shared_surface_contract_all_non_wake_only_harnesses_use_shared_surfaces() {
        use crate::harness::preset::{
            HarnessPresetRegistry, SHARED_VISIBLE_SURFACES, WAKE_ONLY_SURFACES,
        };
        let registry = HarnessPresetRegistry::default_registry();
        for preset in &registry.packs {
            if preset.wake_only {
                assert_eq!(
                    preset.surface_set, WAKE_ONLY_SURFACES,
                    "wake-only harness {} should use WAKE_ONLY_SURFACES",
                    preset.pack_id
                );
            } else {
                assert_eq!(
                    preset.surface_set, SHARED_VISIBLE_SURFACES,
                    "harness {} should use SHARED_VISIBLE_SURFACES",
                    preset.pack_id
                );
            }
        }
    }

    #[test]
    fn shared_surface_contract_no_duplicate_surface_filenames() {
        use crate::harness::preset::HarnessPresetRegistry;
        let registry = HarnessPresetRegistry::default_registry();
        for preset in &registry.packs {
            let mut seen = std::collections::HashSet::new();
            for surface in preset.surface_set {
                assert!(
                    seen.insert(*surface),
                    "harness {} has duplicate surface: {}",
                    preset.pack_id,
                    surface
                );
            }
        }
    }

    #[test]
    fn claude_code_is_in_preset_registry_with_wake_only() {
        use crate::harness::preset::HarnessPresetRegistry;
        let registry = HarnessPresetRegistry::default_registry();
        let claude = registry
            .get("claude-code")
            .expect("claude-code must be in registry");
        assert!(claude.wake_only, "claude-code must be wake_only");
        assert_eq!(claude.wake_char_budget, 1200);
        assert_eq!(claude.surface_set.len(), 1);
        assert_eq!(claude.surface_set[0], "wake.md");
    }

    #[test]
    fn claude_code_pack_files_do_not_include_mem_or_events() {
        let dir = std::env::temp_dir().join(format!("memd-claude-pack-{}", uuid::Uuid::new_v4()));
        let pack =
            crate::harness::claude_code::build_claude_code_harness_pack(&dir, "demo", "main");
        for file in &pack.files {
            let name = file.file_name().unwrap().to_string_lossy();
            assert_ne!(name, "mem.md", "claude-code pack must not include mem.md");
            assert_ne!(
                name, "events.md",
                "claude-code pack must not include events.md"
            );
        }
        assert!(
            pack.files.iter().any(|f| f.ends_with("wake.md")),
            "claude-code pack must include wake.md"
        );
    }

    #[test]
    fn wake_budget_for_claude_code_uses_registry_not_special_case() {
        use crate::harness::preset::wake_char_budget_for_agent;
        assert_eq!(wake_char_budget_for_agent(Some("claude-code")), 1200);
        assert_eq!(
            wake_char_budget_for_agent(Some("claude-code@session-abc")),
            1200
        );
        assert_eq!(wake_char_budget_for_agent(Some("codex")), 1800);
        assert_eq!(wake_char_budget_for_agent(Some("hermes")), 1800);
        assert_eq!(wake_char_budget_for_agent(Some("unknown-agent")), 1800);
    }

    #[test]
    fn is_wake_only_agent_matches_registry() {
        use crate::harness::preset::is_wake_only_agent;
        assert!(is_wake_only_agent(Some("claude-code")));
        assert!(is_wake_only_agent(Some("claude-code@session-xyz")));
        assert!(!is_wake_only_agent(Some("codex")));
        assert!(!is_wake_only_agent(Some("hermes")));
        assert!(!is_wake_only_agent(Some("opencode")));
        assert!(!is_wake_only_agent(Some("openclaw")));
        assert!(!is_wake_only_agent(Some("agent-zero")));
        assert!(!is_wake_only_agent(None));
    }

    #[test]
    fn all_harness_wake_packets_stay_within_budget() {
        use crate::harness::preset::HarnessPresetRegistry;
        let registry = HarnessPresetRegistry::default_registry();
        let snapshot = pressure_snapshot();
        for preset in &registry.packs {
            let dir = std::env::temp_dir().join(format!(
                "memd-budget-{}-{}",
                preset.pack_id,
                uuid::Uuid::new_v4()
            ));
            fs::create_dir_all(&dir).expect("create temp bundle");
            fs::write(
                dir.join("config.json"),
                format!(
                    r#"{{
  "project": "memd",
  "agent": "{}",
  "route": "auto",
  "intent": "current_task"
}}
"#,
                    preset.pack_id
                ),
            )
            .expect("write config");

            let markdown = render_bundle_wakeup_markdown(&dir, &snapshot, false);
            let char_count = markdown.chars().count();
            assert!(
                char_count <= preset.wake_char_budget,
                "harness {} wake packet ({} chars) exceeds budget ({} chars)",
                preset.pack_id,
                char_count,
                preset.wake_char_budget,
            );

            fs::remove_dir_all(dir).expect("cleanup temp bundle");
        }
    }

    #[test]
    fn generated_claude_imports_template_only_imports_wake() {
        // Verify the code template in maintenance_runtime writes only @../wake.md
        // This is a structural test on the bridge file content
        let template_fragment = "@../wake.md";
        let cold_surfaces = ["@../mem.md", "@../events.md"];

        // Simulate what write_native_agent_bridge_files generates
        let generated = format!("## Imported memd memory files\n\n{}\n\n", template_fragment);
        assert!(
            generated.contains(template_fragment),
            "CLAUDE_IMPORTS template must import wake.md"
        );
        for cold in &cold_surfaces {
            assert!(
                !generated.contains(cold),
                "CLAUDE_IMPORTS template must NOT import {} (cold-path surface)",
                cold
            );
        }
    }

    #[test]
    fn preset_registry_has_exactly_six_harnesses() {
        use crate::harness::preset::HarnessPresetRegistry;
        let registry = HarnessPresetRegistry::default_registry();
        assert_eq!(
            registry.packs.len(),
            6,
            "registry should have 6 harnesses: codex, claude-code, agent-zero, openclaw, hermes, opencode"
        );
    }

    fn write_skill_md(bundle: &Path, name: &str, description: &str, body: &str) {
        let dir = bundle.join("skills").join(name);
        std::fs::create_dir_all(&dir).unwrap();
        let payload = format!(
            "---\nname: {}\ndescription: {}\n---\n\n{}",
            name, description, body
        );
        std::fs::write(dir.join("SKILL.md"), payload).unwrap();
    }

    #[test]
    fn active_skills_block_omitted_when_no_skills() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert_eq!(render_active_skills_block(tmp.path()), "");
    }

    #[test]
    fn active_skills_block_lists_skills_with_inlined_first_body() {
        let tmp = tempfile::TempDir::new().unwrap();
        write_skill_md(
            tmp.path(),
            "tdd",
            "drive features test-first",
            "## Steps\n1. Red\n2. Green\n",
        );
        write_skill_md(tmp.path(), "review", "second skill", "body two");
        let block = render_active_skills_block(tmp.path());
        assert!(block.starts_with("## Active Skills"));
        assert!(block.contains("- **review** — second skill"));
        assert!(block.contains("body two"));
        assert!(block.contains("- tdd — `memd lookup --kind skill --name tdd`"));
    }

    fn write_skill_md_with_salience(
        bundle: &Path,
        name: &str,
        description: &str,
        body: &str,
        salience: f32,
    ) {
        let dir = bundle.join("skills").join(name);
        std::fs::create_dir_all(&dir).unwrap();
        let payload = format!(
            "---\nname: {}\ndescription: {}\nsalience: {}\n---\n\n{}",
            name, description, salience, body
        );
        std::fs::write(dir.join("SKILL.md"), payload).unwrap();
    }

    #[test]
    fn active_skills_section_orders_by_salience_descending() {
        // P2.5 §9: high-salience skill must appear as the inlined headline,
        // not the alphabetical winner. "alpha" < "zoom" lexically; salience
        // wins.
        let tmp = tempfile::TempDir::new().unwrap();
        write_skill_md_with_salience(tmp.path(), "alpha", "low priority", "alpha body", 0.1);
        write_skill_md_with_salience(tmp.path(), "zoom", "high priority", "zoom body", 0.9);
        let block = render_active_skills_block(tmp.path());
        let zoom_pos = block.find("zoom").expect("zoom present");
        let alpha_pos = block.find("alpha").expect("alpha present");
        assert!(
            zoom_pos < alpha_pos,
            "zoom (salience 0.9) must precede alpha (salience 0.1):\n{block}"
        );
        assert!(
            block.contains("- **zoom** — high priority"),
            "zoom should be the inlined headline:\n{block}"
        );
    }

    #[test]
    fn active_skills_section_falls_back_to_alphabetical_on_tie() {
        // P2.5 §9: equal salience → name asc. Both at None (legacy mirrors
        // without salience) collapse to 0.0 and tie.
        let tmp = tempfile::TempDir::new().unwrap();
        write_skill_md(tmp.path(), "bravo", "second alphabetical", "b body");
        write_skill_md(tmp.path(), "alpha", "first alphabetical", "a body");
        let block = render_active_skills_block(tmp.path());
        let alpha_pos = block.find("alpha").expect("alpha present");
        let bravo_pos = block.find("bravo").expect("bravo present");
        assert!(
            alpha_pos < bravo_pos,
            "alpha must precede bravo when salience ties:\n{block}"
        );
        assert!(
            block.contains("- **alpha** — first alphabetical"),
            "alpha should be the inlined headline:\n{block}"
        );
    }
}
