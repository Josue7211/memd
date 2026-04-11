use super::*;

#[derive(Debug, Clone)]
pub(crate) struct BundleHiveMemorySurface {
    pub(crate) board: HiveBoardResponse,
    pub(crate) roster: HiveRosterResponse,
    pub(crate) follow: Option<HiveFollowResponse>,
}

pub(crate) fn render_project_awareness_summary(response: &ProjectAwarenessResponse) -> String {
    let current_entry = response
        .entries
        .iter()
        .find(|candidate| candidate.bundle_root == response.current_bundle);
    let visible_entries = project_awareness_visible_entries(response);
    let hidden_remote_dead = response
        .entries
        .iter()
        .filter(|entry| {
            entry.project_dir == "remote"
                && entry.presence == "dead"
                && current_entry
                    .map(|current| {
                        entry.project == current.project
                            && entry.namespace == current.namespace
                            && entry.workspace == current.workspace
                            && entry.base_url == current.base_url
                    })
                    .unwrap_or(true)
        })
        .count();
    let superseded_stale_sessions = response
        .entries
        .iter()
        .filter(|entry| {
            current_entry
                .map(|current| is_superseded_stale_remote_session(entry, current))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    let superseded_stale_count = superseded_stale_sessions.len();
    let superseded_stale_session_ids = superseded_stale_sessions
        .iter()
        .filter_map(|entry| entry.session.as_deref())
        .take(3)
        .collect::<Vec<_>>();
    let superseded_stale_suffix = if superseded_stale_count > superseded_stale_session_ids.len() {
        format!(
            " +{}",
            superseded_stale_count - superseded_stale_session_ids.len()
        )
    } else {
        String::new()
    };
    let current_session = visible_entries
        .iter()
        .find(|entry| entry.bundle_root == response.current_bundle)
        .and_then(|entry| entry.session.as_deref());
    let stale_remote_sessions = visible_entries
        .iter()
        .filter(|entry| entry.project_dir == "remote" && entry.presence == "stale")
        .collect::<Vec<_>>();
    let active_hive_sessions = current_session
        .map(|current| {
            visible_entries
                .iter()
                .filter(|entry| entry.presence == "active")
                .filter(|entry| !entry.hive_groups.is_empty())
                .filter_map(|entry| entry.session.as_deref())
                .filter(|session| *session != current)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let rendered_diagnostics = awareness_summary_diagnostics(&visible_entries);
    let mut lines = vec![format!(
        "awareness root={} bundles={} diagnostics={} hidden_remote_dead={} hidden_superseded_stale={}",
        response.root,
        visible_entries.len(),
        rendered_diagnostics.len(),
        hidden_remote_dead,
        superseded_stale_count,
    )];
    if !active_hive_sessions.is_empty() {
        lines.push(format!(
            "! active_hive_sessions={} sessions={}",
            active_hive_sessions.len(),
            active_hive_sessions.join(",")
        ));
    }
    if !stale_remote_sessions.is_empty() {
        let sessions = stale_remote_sessions
            .iter()
            .take(3)
            .filter_map(|entry| entry.session.as_deref())
            .collect::<Vec<_>>();
        let suffix = if stale_remote_sessions.len() > sessions.len() {
            format!(" +{}", stale_remote_sessions.len() - sessions.len())
        } else {
            String::new()
        };
        lines.push(format!(
            "! stale_remote_sessions={} sessions={}{}",
            stale_remote_sessions.len(),
            if sessions.is_empty() {
                "unknown".to_string()
            } else {
                sessions.join(",")
            },
            suffix,
        ));
    }
    if superseded_stale_count > 0 {
        lines.push(format!(
            "! superseded_stale_sessions={} sessions={}{}",
            superseded_stale_count,
            if superseded_stale_session_ids.is_empty() {
                "unknown".to_string()
            } else {
                superseded_stale_session_ids.join(",")
            },
            superseded_stale_suffix,
        ));
    }
    for diagnostic in &rendered_diagnostics {
        lines.push(format!("! {}", diagnostic));
    }
    let current_entries = visible_entries
        .iter()
        .copied()
        .filter(|entry| entry.bundle_root == response.current_bundle)
        .collect::<Vec<_>>();
    let active_hive_entries = visible_entries
        .iter()
        .copied()
        .filter(|entry| entry.bundle_root != response.current_bundle)
        .filter(|entry| entry.presence == "active" && !entry.hive_groups.is_empty())
        .collect::<Vec<_>>();
    let stale_entries = visible_entries
        .iter()
        .copied()
        .filter(|entry| entry.presence == "stale")
        .collect::<Vec<_>>();
    let seen_entries = visible_entries
        .iter()
        .copied()
        .filter(|entry| entry.bundle_root != response.current_bundle)
        .filter(|entry| !(entry.presence == "active" && !entry.hive_groups.is_empty()))
        .filter(|entry| entry.presence != "stale")
        .filter(|entry| entry.presence != "dead")
        .collect::<Vec<_>>();
    let dead_entries = visible_entries
        .iter()
        .copied()
        .filter(|entry| entry.presence == "dead")
        .collect::<Vec<_>>();

    push_awareness_section(
        &mut lines,
        "current_session",
        &current_entries,
        "current",
        &response.current_bundle,
    );
    push_awareness_section(
        &mut lines,
        "active_hive_sessions",
        &active_hive_entries,
        "hive-session",
        &response.current_bundle,
    );
    push_awareness_section(
        &mut lines,
        "stale_sessions",
        &stale_entries,
        "stale-session",
        &response.current_bundle,
    );
    push_awareness_section(
        &mut lines,
        "dead_sessions",
        &dead_entries,
        "dead-session",
        &response.current_bundle,
    );
    push_awareness_section(
        &mut lines,
        "seen_sessions",
        &seen_entries,
        "seen",
        &response.current_bundle,
    );
    lines.join("\n")
}

pub(crate) fn push_awareness_section(
    lines: &mut Vec<String>,
    label: &str,
    entries: &[&ProjectAwarenessEntry],
    role: &str,
    current_bundle: &str,
) {
    if entries.is_empty() {
        return;
    }
    lines.push(format!("{label}:"));
    for entry in entries {
        lines.push(render_awareness_entry_line(entry, role, current_bundle));
    }
}

pub(crate) fn awareness_truth_label(
    entry: &ProjectAwarenessEntry,
    current_bundle: &str,
) -> &'static str {
    if entry.bundle_root == current_bundle && entry.presence == "active" {
        return "current";
    }
    match entry.presence.as_str() {
        "active" => match entry.last_updated {
            Some(last_updated) => {
                let age = Utc::now() - last_updated;
                if age.num_seconds() <= 120 {
                    "fresh"
                } else if age.num_minutes() <= 15 {
                    "aging"
                } else {
                    "stale-truth"
                }
            }
            None => "active",
        },
        "stale" => "stale",
        "dead" => "dead",
        _ => "seen",
    }
}

pub(crate) fn render_awareness_entry_line(
    entry: &ProjectAwarenessEntry,
    role: &str,
    current_bundle: &str,
) -> String {
    let focus = entry
        .focus
        .as_deref()
        .map(|value| compact_inline(value, 56))
        .unwrap_or_else(|| "none".to_string());
    let pressure = entry
        .pressure
        .as_deref()
        .map(|value| compact_inline(value, 56))
        .unwrap_or_else(|| "none".to_string());
    let truth = awareness_truth_label(entry, current_bundle);
    let updated = entry
        .last_updated
        .map(|value| value.to_rfc3339())
        .unwrap_or_else(|| "unknown".to_string());
    let work = entry
        .topic_claim
        .as_deref()
        .map(|value| compact_inline(value, 56))
        .unwrap_or_else(|| super::hive::awareness_work_quickview(entry));
    let next = entry
        .next_recovery
        .as_deref()
        .and_then(simplify_awareness_work_text)
        .map(|value| compact_inline(&value, 56))
        .unwrap_or_else(|| "none".to_string());
    let touches = if entry.scope_claims.is_empty() {
        super::hive::awareness_touch_quickview(entry)
    } else {
        compact_inline(&entry.scope_claims.join(","), 56)
    };
    format!(
        "- {} [{}] | presence={} truth={} updated={} claims={} ns={} hive={} role={} groups={} goal=\"{}\" authority={} agent={} session={} tab={} branch={} worktree={} base_url={} workspace={} visibility={} task={} work=\"{}\" touches={} next=\"{}\" focus=\"{}\" pressure=\"{}\"",
        entry.project.as_deref().unwrap_or("unknown"),
        role,
        entry.presence,
        truth,
        updated,
        entry.active_claims,
        entry.namespace.as_deref().unwrap_or("none"),
        entry.hive_system.as_deref().unwrap_or("none"),
        entry.hive_role.as_deref().unwrap_or("none"),
        if entry.hive_groups.is_empty() {
            "none".to_string()
        } else {
            entry.hive_groups.join(",")
        },
        entry.hive_group_goal.as_deref().unwrap_or("none"),
        entry.authority.as_deref().unwrap_or("none"),
        entry
            .effective_agent
            .as_deref()
            .or(entry.agent.as_deref())
            .unwrap_or("none"),
        entry.session.as_deref().unwrap_or("none"),
        entry.tab_id.as_deref().unwrap_or("none"),
        entry.branch.as_deref().unwrap_or("none"),
        entry.worktree_root.as_deref().unwrap_or("none"),
        entry.base_url.as_deref().unwrap_or("none"),
        entry.workspace.as_deref().unwrap_or("none"),
        entry.visibility.as_deref().unwrap_or("all"),
        entry.task_id.as_deref().unwrap_or("none"),
        work,
        touches,
        next,
        focus,
        pressure,
    )
}
