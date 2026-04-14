use super::*;

fn canonical_bundle_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn same_bundle_path(entry_bundle_root: &str, current_bundle: &Path) -> bool {
    canonical_bundle_path(Path::new(entry_bundle_root)) == canonical_bundle_path(current_bundle)
}

fn claim_holder_label(claim: &SessionClaim) -> String {
    claim
        .effective_agent
        .as_deref()
        .or(claim.session.as_deref())
        .unwrap_or("none")
        .to_string()
}

fn claim_json(claim: &SessionClaim) -> JsonValue {
    serde_json::json!({
        "scope": claim.scope,
        "holder": claim_holder_label(claim),
        "session": claim.session,
        "tab_id": claim.tab_id,
        "workspace": claim.workspace,
        "acquired_at": claim.acquired_at,
        "expires_at": claim.expires_at,
    })
}

fn claim_identity(claim: &SessionClaim) -> (String, Option<String>, Option<String>) {
    (
        claim.scope.clone(),
        claim.session.clone(),
        claim.tab_id.clone(),
    )
}

fn build_claim_conflicts(claims: &[SessionClaim]) -> Vec<JsonValue> {
    let mut grouped = BTreeMap::<String, BTreeSet<String>>::new();
    for claim in claims {
        grouped
            .entry(claim.scope.clone())
            .or_default()
            .insert(claim_holder_label(claim));
    }

    grouped
        .into_iter()
        .filter(|(_, holders)| holders.len() > 1)
        .map(|(scope, holders)| {
            let count = holders.len();
            serde_json::json!({
                "scope": scope,
                "holders": holders.into_iter().collect::<Vec<_>>(),
                "count": count,
            })
        })
        .collect()
}

fn divergence_kind(summary: &str) -> &'static str {
    if summary.starts_with("unsafe_same_branch") {
        "branch_collision"
    } else if summary.starts_with("unsafe_same_worktree") {
        "worktree_collision"
    } else if summary.starts_with("possible_work_overlap") {
        "work_overlap"
    } else if summary.starts_with("session ") {
        "session_collision"
    } else if summary.starts_with("base_url ") || summary.starts_with("shared_hive_endpoint") {
        "endpoint_overlap"
    } else if summary.starts_with("lane_fault ") {
        "lane_fault"
    } else {
        "diagnostic"
    }
}

fn divergence_detail(summary: &str, key: &str) -> Option<String> {
    if key == "base_url" && summary.starts_with("base_url ") {
        return summary
            .strip_prefix("base_url ")
            .and_then(|rest| rest.split_whitespace().next())
            .map(str::to_string);
    }
    if key == "shared_hive_endpoint" && summary.starts_with("shared_hive_endpoint ") {
        return summary
            .strip_prefix("shared_hive_endpoint ")
            .and_then(|rest| rest.split_whitespace().next())
            .map(str::to_string);
    }
    summary
        .split_whitespace()
        .find_map(|part| part.strip_prefix(&format!("{key}=")))
        .map(str::to_string)
}

fn divergence_sessions(summary: &str) -> Vec<String> {
    divergence_detail(summary, "sessions")
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn build_divergence_items(
    awareness: &ProjectAwarenessResponse,
    status: &JsonValue,
) -> Vec<JsonValue> {
    let mut summaries = awareness.collisions.clone();
    summaries.extend(crate::awareness::branch_collision_warnings(
        &awareness.entries,
    ));
    let entry_refs = awareness.entries.iter().collect::<Vec<_>>();
    summaries.extend(crate::awareness::work_overlap_warnings(&entry_refs));
    if let Some(kind) = status
        .get("lane_fault")
        .and_then(|value| value.get("kind"))
        .and_then(JsonValue::as_str)
    {
        let session = status
            .get("lane_fault")
            .and_then(|value| value.get("session"))
            .and_then(JsonValue::as_str)
            .unwrap_or("none");
        summaries.push(format!("lane_fault {kind} session={session}"));
    }

    let mut deduped = BTreeSet::new();
    summaries
        .into_iter()
        .filter(|summary| !summary.trim().is_empty() && deduped.insert(summary.clone()))
        .map(|summary| {
            let kind = divergence_kind(&summary);
            serde_json::json!({
                "kind": kind,
                "severity": match kind {
                    "branch_collision" | "worktree_collision" | "lane_fault" => "high",
                    "work_overlap" | "endpoint_overlap" | "session_collision" => "medium",
                    _ => "low",
                },
                "scope": divergence_detail(&summary, "touches"),
                "branch": divergence_detail(&summary, "branch"),
                "worktree": divergence_detail(&summary, "worktree"),
                "repo": divergence_detail(&summary, "repo"),
                "endpoint": divergence_detail(&summary, "base_url")
                    .or_else(|| divergence_detail(&summary, "shared_hive_endpoint")),
                "session": divergence_detail(&summary, "session"),
                "sessions": divergence_sessions(&summary),
                "summary": summary,
            })
        })
        .collect()
}

fn default_resume_args(output: &Path) -> ResumeArgs {
    ResumeArgs {
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
    }
}

fn default_claims_args(output: &Path) -> ClaimsArgs {
    ClaimsArgs {
        output: output.to_path_buf(),
        acquire: false,
        release: false,
        transfer_to_session: None,
        scope: None,
        ttl_secs: 900,
        summary: false,
    }
}

fn effective_repo_changes(snapshot: &ResumeSnapshot) -> Vec<String> {
    snapshot
        .recent_repo_changes
        .iter()
        .filter_map(|line| {
            let trimmed = line.trim();
            let is_generated_bundle_artifact =
                [".memd/env", ".memd/env.ps1", ".memd/state/heartbeat.json"]
                    .iter()
                    .any(|needle| trimmed.contains(needle));
            if trimmed.is_empty()
                || trimmed.eq_ignore_ascii_case("repo clean")
                || is_generated_bundle_artifact
            {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect()
}

fn operator_freshness_status(snapshot: &ResumeSnapshot) -> &'static str {
    if snapshot.refresh_recommended
        || snapshot
            .resume_state_age_minutes
            .is_some_and(|age| age >= 30)
    {
        "stale"
    } else if !snapshot.change_summary.is_empty() || !effective_repo_changes(snapshot).is_empty() {
        "changed"
    } else {
        "unchanged"
    }
}

pub(crate) async fn read_bundle_state(output: &Path, base_url: &str) -> anyhow::Result<JsonValue> {
    let status = read_bundle_status(output, base_url).await?;
    let snapshot = read_bundle_resume(&default_resume_args(output), base_url).await?;
    let claims_response = run_claims_command(&default_claims_args(output), base_url).await?;
    let awareness = read_project_awareness(&AwarenessArgs {
        output: output.to_path_buf(),
        root: None,
        include_current: true,
        summary: false,
    })
    .await?;
    let current_entry = awareness
        .entries
        .iter()
        .find(|entry| same_bundle_path(&entry.bundle_root, output));

    let now = Utc::now();
    let active_claims = claims_response
        .claims
        .iter()
        .filter(|claim| claim.expires_at > now)
        .cloned()
        .collect::<Vec<_>>();
    let mut expired_claims = snapshot
        .claims
        .claims
        .iter()
        .filter(|claim| claim.expires_at <= now)
        .cloned()
        .collect::<Vec<_>>();
    let active_keys = active_claims
        .iter()
        .map(claim_identity)
        .collect::<BTreeSet<_>>();
    expired_claims.retain(|claim| !active_keys.contains(&claim_identity(claim)));
    let claim_conflicts = build_claim_conflicts(&active_claims);
    let divergence_items = build_divergence_items(&awareness, &status);
    let effective_repo_changes = effective_repo_changes(&snapshot);

    let mut warnings = Vec::<String>::new();
    if snapshot.refresh_recommended {
        warnings.push("memory refresh recommended due to context pressure".to_string());
    }
    warnings.extend(claim_conflicts.iter().filter_map(|conflict| {
        let scope = conflict.get("scope").and_then(JsonValue::as_str)?;
        let count = conflict
            .get("count")
            .and_then(JsonValue::as_u64)
            .unwrap_or(0);
        Some(format!("claim conflict on {scope} across {count} holders"))
    }));
    warnings.extend(
        status
            .get("authority_warning")
            .and_then(JsonValue::as_array)
            .into_iter()
            .flatten()
            .filter_map(JsonValue::as_str)
            .map(str::to_string),
    );
    warnings.extend(
        divergence_items
            .iter()
            .filter_map(|item| item.get("summary").and_then(JsonValue::as_str))
            .take(4)
            .map(str::to_string),
    );

    let truth_summary = serde_json::to_value(build_truth_summary(&snapshot))?;
    Ok(serde_json::json!({
        "bundle": output.display().to_string(),
        "live_truth": truth_summary,
        "focus": snapshot.continuity_doing(),
        "session": {
            "project": snapshot.project,
            "namespace": snapshot.namespace,
            "agent": snapshot.agent,
            "workspace": snapshot.workspace,
            "visibility": snapshot.visibility,
            "route": snapshot.route,
            "intent": snapshot.intent,
            "session": status
                .get("defaults")
                .and_then(|value| value.get("session"))
                .and_then(JsonValue::as_str),
            "tab_id": status
                .get("defaults")
                .and_then(|value| value.get("tab_id"))
                .and_then(JsonValue::as_str),
            "branch": current_entry.and_then(|entry| entry.branch.as_deref()),
            "worktree_root": current_entry.and_then(|entry| entry.worktree_root.as_deref()),
            "repo_root": current_entry.and_then(|entry| entry.repo_root.as_deref()),
        },
        "claims": {
            "active_count": active_claims.len(),
            "expired_count": expired_claims.len(),
            "active": active_claims.iter().map(claim_json).collect::<Vec<_>>(),
            "expired": expired_claims.iter().map(claim_json).collect::<Vec<_>>(),
            "conflicts": claim_conflicts,
        },
        "freshness": {
            "status": operator_freshness_status(&snapshot),
            "since_checkpoint_minutes": snapshot.resume_state_age_minutes,
            "refresh_recommended": snapshot.refresh_recommended,
            "change_count": snapshot.change_summary.len(),
            "repo_change_count": effective_repo_changes.len(),
            "changed": !snapshot.change_summary.is_empty() || !effective_repo_changes.is_empty(),
            "event_spine": snapshot.event_spine(),
        },
        "divergence": {
            "status": if divergence_items.is_empty() { "none" } else { "warning" },
            "items": divergence_items,
        },
        "memory_health": {
            "pressure": snapshot.context_pressure(),
            "estimated_prompt_tokens": snapshot.estimated_prompt_tokens(),
            "redundant_context_items": snapshot.redundant_context_items(),
            "inbox_items": snapshot.inbox.items.len(),
            "rehydration_queue": snapshot.working.rehydration_queue.len(),
            "degraded": status
                .get("degraded")
                .and_then(JsonValue::as_bool)
                .unwrap_or(false),
        },
        "warnings": warnings,
        "runtime": {
            "server": status
                .get("server")
                .and_then(|value| value.get("status"))
                .and_then(JsonValue::as_str),
            "rag_healthy": status
                .get("rag")
                .and_then(|value| value.get("healthy"))
                .and_then(JsonValue::as_bool),
            "setup_ready": status.get("setup_ready").and_then(JsonValue::as_bool),
            "rebased_from": status
                .get("session_overlay")
                .and_then(|value| value.get("rebased_from"))
                .and_then(JsonValue::as_str),
        }
    }))
}
