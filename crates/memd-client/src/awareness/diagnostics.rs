use super::*;

pub(crate) fn awareness_summary_diagnostics(entries: &[&ProjectAwarenessEntry]) -> Vec<String> {
    let owned = entries
        .iter()
        .map(|entry| (*entry).clone())
        .collect::<Vec<_>>();
    let mut diagnostics = shared_endpoint_diagnostics(entries);
    diagnostics.extend(session_collision_warnings(&owned));
    diagnostics.extend(branch_collision_warnings(&owned));
    diagnostics.extend(hive_goal_mismatch_warnings(entries));
    diagnostics.extend(work_overlap_warnings(entries));
    diagnostics
}

pub(crate) fn shared_endpoint_diagnostics(entries: &[&ProjectAwarenessEntry]) -> Vec<String> {
    let mut counts = std::collections::BTreeMap::<String, usize>::new();
    for entry in entries {
        if let Some(url) = entry
            .base_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            *counts.entry(url.to_string()).or_insert(0) += 1;
        }
    }

    counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(url, count)| format!("shared_hive_endpoint {} sessions={}", url, count))
        .collect()
}

pub(crate) fn branch_collision_warnings(entries: &[ProjectAwarenessEntry]) -> Vec<String> {
    let mut same_branch = std::collections::BTreeMap::<(String, String), Vec<String>>::new();
    let mut same_worktree = std::collections::BTreeMap::<String, Vec<String>>::new();

    for entry in entries {
        let lane = entry
            .session
            .clone()
            .or_else(|| entry.effective_agent.clone())
            .or_else(|| entry.agent.clone())
            .unwrap_or_else(|| entry.bundle_root.clone());

        if let (Some(repo_root), Some(branch)) = (
            entry
                .repo_root
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
            entry
                .branch
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        ) {
            same_branch
                .entry((repo_root.to_string(), branch.to_string()))
                .or_default()
                .push(lane.clone());
        }

        if let Some(worktree_root) = entry
            .worktree_root
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            same_worktree
                .entry(worktree_root.to_string())
                .or_default()
                .push(lane);
        }
    }

    let mut warnings = Vec::new();
    warnings.extend(
        same_branch
            .into_iter()
            .filter(|(_, lanes)| lanes.len() > 1)
            .map(|((repo_root, branch), lanes)| {
                format!(
                    "unsafe_same_branch repo={} branch={} sessions={}",
                    repo_root,
                    branch,
                    lanes.join(",")
                )
            }),
    );
    warnings.extend(
        same_worktree
            .into_iter()
            .filter(|(_, lanes)| lanes.len() > 1)
            .map(|(worktree_root, lanes)| {
                format!(
                    "unsafe_same_worktree worktree={} sessions={}",
                    worktree_root,
                    lanes.join(",")
                )
            }),
    );
    warnings
}

pub(crate) fn hive_goal_mismatch_warnings(entries: &[&ProjectAwarenessEntry]) -> Vec<String> {
    let mut by_group =
        std::collections::BTreeMap::<String, std::collections::BTreeMap<String, Vec<String>>>::new(
        );

    for entry in entries {
        let Some(goal) = entry
            .hive_group_goal
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let session = entry
            .session
            .as_deref()
            .or(entry.effective_agent.as_deref())
            .or(entry.agent.as_deref())
            .unwrap_or("unknown")
            .to_string();
        for group in entry
            .hive_groups
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            by_group
                .entry(group.to_string())
                .or_default()
                .entry(goal.to_string())
                .or_default()
                .push(session.clone());
        }
    }

    by_group
        .into_iter()
        .filter(|(_, goals)| goals.len() > 1)
        .map(|(group, goals)| {
            let goal_labels = goals
                .keys()
                .map(|goal| compact_inline(goal, 48))
                .collect::<Vec<_>>()
                .join("|");
            let sessions = goals
                .values()
                .flatten()
                .cloned()
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "hive_goal_mismatch group={} goals={} sessions={} action=align_hive_group_goal_before_handoff",
                group, goal_labels, sessions
            )
        })
        .collect()
}

pub(crate) fn work_overlap_warnings(entries: &[&ProjectAwarenessEntry]) -> Vec<String> {
    let mut warnings = Vec::new();
    let active_entries = entries
        .iter()
        .copied()
        .filter(|entry| entry.presence == "active")
        .collect::<Vec<_>>();

    for (idx, left) in active_entries.iter().enumerate() {
        let left_touches = super::hive::awareness_overlap_touch_points(left);
        if left_touches.is_empty() {
            continue;
        }
        for right in active_entries.iter().skip(idx + 1) {
            let right_touches = super::hive::awareness_overlap_touch_points(right);
            if right_touches.is_empty() {
                continue;
            }
            let shared = left_touches
                .iter()
                .filter(|touch| right_touches.iter().any(|other| other == *touch))
                .cloned()
                .collect::<Vec<_>>();
            if shared.is_empty() {
                continue;
            }
            warnings.push(format!(
                "possible_work_overlap touches={} sessions={},{}",
                shared.join(","),
                left.session.as_deref().unwrap_or("none"),
                right.session.as_deref().unwrap_or("none"),
            ));
        }
    }

    warnings
}
