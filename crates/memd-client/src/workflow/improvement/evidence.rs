use super::*;

use chrono::Utc;
use std::{path::Path, process::Command};

pub(crate) fn collect_gap_plan_evidence(project_root: &Path) -> Vec<String> {
    let planning_root = project_root.join(".planning");
    let mut evidence = Vec::new();
    let mut repo_evidence = collect_gap_repo_evidence(project_root);
    let roadmap = read_text_file(&planning_root.join("ROADMAP.md"));
    let state = read_text_file(&planning_root.join("STATE.md"));
    let project = read_text_file(&planning_root.join("PROJECT.md"));

    if let Some(roadmap) = roadmap {
        let lines = roadmap
            .lines()
            .filter(|value| value.contains("Phase") && value.contains("v6"))
            .take(4)
            .collect::<Vec<_>>();
        if !lines.is_empty() {
            evidence.push(format!("roadmap phases: {}", lines.join(" | ")));
        }
    }
    if let Some(state) = state {
        if let Some(open_loops) = state
            .lines()
            .find(|line| line.starts_with("- ") && line.contains("phase"))
        {
            evidence.push(format!("state signal: {open_loops}"));
        }
        if let Some(open_block) = state.split("## Open Loops").nth(1) {
            let next = open_block
                .lines()
                .take(3)
                .filter(|value| value.starts_with("- "))
                .collect::<Vec<_>>();
            if !next.is_empty() {
                evidence.push(format!("state open loops: {}", next.join(" | ")));
            }
        }
    }
    if let Some(project) = project {
        if let Some(core) = project
            .lines()
            .find(|line| line.starts_with("##") && line.contains("Core"))
        {
            evidence.push(format!("project: {core}"));
        }
    }

    evidence.append(&mut repo_evidence);
    evidence
}

pub(crate) fn collect_gap_repo_evidence(project_root: &Path) -> Vec<String> {
    let mut evidence = Vec::new();
    let branch = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("branch")
        .arg("--show-current")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    evidence.push(format!("git branch: {branch}"));

    let status = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("status")
        .arg("--short")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .take(12)
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if status.is_empty() {
        evidence.push("git status: clean".to_string());
    } else {
        evidence.push(format!("git status: {}", status.join(" | ")));
    }

    for (path, label, keywords) in [
        (project_root.join("AGENTS.md"), "AGENTS.md", &["memd", "memory", "bootstrap"][..]),
        (project_root.join("CLAUDE.md"), "CLAUDE.md", &["memd", "memory", "hook"][..]),
        (project_root.join("MEMORY.md"), "MEMORY.md", &["memory", "memd", "decision"][..]),
        (project_root.join("README.md"), "README.md", &["memd", "setup", "memory"][..]),
        (project_root.join("ROADMAP.md"), "ROADMAP.md", &["v5", "v6", "memd"][..]),
        (project_root.join("docs/core/setup.md"), "docs/core/setup.md", &["memd", "bundle", "codex"][..]),
        (
            project_root.join("docs/reference/infra-facts.md"),
            "docs/reference/infra-facts.md",
            &["memd", "openclaw", "tailnet"][..],
        ),
        (
            project_root.join(".planning/STATE.md"),
            ".planning/STATE.md",
            &["memory", "gap", "open loop"][..],
        ),
    ] {
        if let Some(snippet) = read_keyword_snippet(&path, keywords, 4) {
            evidence.push(format!("{label}: {snippet}"));
        }
    }

    let local_bundle = project_root.join(".memd").join("config.json").exists();
    let global_bundle = home_dir()
        .map(|home| home.join(".memd").join("config.json").exists())
        .unwrap_or(false);
    evidence.push(format!("memd bundles: global={} project={}", global_bundle, local_bundle));

    let wiring = read_memd_runtime_wiring();
    let codex_wired = wiring
        .get("codex")
        .and_then(|value| value.get("wired"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let claude_wired = wiring
        .get("claude")
        .and_then(|value| value.get("wired"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let openclaw_wired = wiring
        .get("openclaw")
        .and_then(|value| value.get("wired"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let opencode_wired = wiring
        .get("opencode")
        .and_then(|value| value.get("wired"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    evidence.push(format!(
        "runtime wiring: codex={} claude={} openclaw={} opencode={}",
        codex_wired, claude_wired, openclaw_wired, opencode_wired
    ));

    evidence
}

pub(crate) fn collect_recent_repo_changes(project_root: &Path) -> Vec<String> {
    let mut changes = Vec::new();

    let status_entries = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("status")
        .arg("--short")
        .arg("--untracked-files=normal")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .take(8)
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if status_entries.is_empty() {
        changes.push("repo clean".to_string());
    } else {
        changes.extend(status_entries.into_iter().map(|entry| format!("status {entry}")));
    }

    let diff_stats = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("diff")
        .arg("--stat=72,40")
        .arg("--compact-summary")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .take(4)
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    changes.extend(diff_stats.into_iter().map(|entry| format!("diff {entry}")));
    changes
}

pub(crate) fn summarize_repo_event_line(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.eq_ignore_ascii_case("repo clean") {
        return "repo_state: clean".to_string();
    }

    if let Some(rest) = trimmed.strip_prefix("status ") {
        let mut parts = rest.split_whitespace();
        let code = parts.next().unwrap_or_default();
        let path = parts.collect::<Vec<_>>().join(" ");
        let label = if code.contains('?') {
            "file_created"
        } else if code.contains('D') {
            "file_deleted"
        } else if code.contains('A')
            || code.contains('M')
            || code.contains('R')
            || code.contains('C')
            || code.contains('U')
            || code.contains('T')
        {
            "file_edited"
        } else {
            "repo_change"
        };
        let detail = if path.is_empty() { code } else { path.as_str() };
        return format!("{label}: {detail}");
    }

    if let Some(rest) = trimmed.strip_prefix("diff ") {
        return format!("repo_delta: {}", rest.trim());
    }

    trimmed.to_string()
}

pub(crate) fn build_event_spine(
    change_summary: &[String],
    recent_repo_changes: &[String],
    refresh_recommended: bool,
) -> Vec<String> {
    let mut spine = Vec::new();

    for change in change_summary.iter().take(4) {
        let compact = change.trim();
        if !compact.is_empty() {
            spine.push(format!("resume_delta: {compact}"));
        }
    }

    for change in recent_repo_changes.iter().take(6) {
        let compact = summarize_repo_event_line(change);
        if !compact.is_empty() {
            spine.push(compact);
        }
    }

    if refresh_recommended {
        spine.push("compaction_due: refresh recommended for current resume state".to_string());
    }

    let mut deduped = Vec::new();
    let mut seen = std::collections::HashSet::<String>::new();
    for item in spine {
        let normalized = item
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        if normalized.is_empty() || !seen.insert(normalized) {
            continue;
        }
        deduped.push(item);
    }

    deduped.truncate(8);
    deduped
}

pub(crate) async fn sync_recent_repo_live_truth(
    project_root: Option<&Path>,
    base_url: &str,
    project: Option<&str>,
    namespace: Option<&str>,
    workspace: Option<&str>,
    visibility: Option<memd_schema::MemoryVisibility>,
) -> anyhow::Result<()> {
    let Some(project_root) = project_root else { return Ok(()); };
    let Some(project) = project else { return Ok(()); };

    let changes = collect_recent_repo_changes(project_root);
    let content = {
        let spine = build_event_spine(&[], &changes, false);
        if spine.is_empty() { "repo_state: clean".to_string() } else { spine.join("\n") }
    };

    let client = MemdClient::new(base_url)?;
    let live_truth_tags = vec!["live_truth".to_string(), "repo_changes".to_string()];
    let search = match search_live_truth_record(&client, project, namespace, workspace, visibility, false).await {
        Ok(response) => response,
        Err(err) if is_live_truth_kind_rejection(&err) => {
            search_live_truth_record(&client, project, namespace, workspace, visibility, true).await?
        }
        Err(err) => return Err(err),
    };

    if let Some(existing) = search.items.first() {
        let repair_request = RepairMemoryRequest {
            id: existing.id,
            mode: MemoryRepairMode::CorrectMetadata,
            confidence: Some(0.98),
            status: Some(MemoryStatus::Active),
            workspace: workspace.map(ToOwned::to_owned),
            visibility,
            source_agent: Some("memd".to_string()),
            source_system: Some("memd-live-truth".to_string()),
            source_path: Some(project_root.display().to_string()),
            source_quality: Some(memd_schema::SourceQuality::Derived),
            content: Some(content.clone()),
            tags: Some(live_truth_tags.clone()),
            supersedes: Vec::new(),
        };
        match client.repair(&repair_request).await {
            Ok(_) => {}
            Err(err) if err.to_string().contains("memory item not found") => {
                store_live_truth_record(
                    &client,
                    content,
                    project,
                    namespace,
                    workspace,
                    visibility,
                    project_root,
                    live_truth_tags,
                )
                .await?;
            }
            Err(err) => return Err(err),
        }
    } else {
        store_live_truth_record(
            &client,
            content,
            project,
            namespace,
            workspace,
            visibility,
            project_root,
            live_truth_tags,
        )
        .await?;
    }

    Ok(())
}

pub(crate) fn is_live_truth_kind_rejection(error: &anyhow::Error) -> bool {
    let message = error.to_string();
    message.contains("unknown variant `live_truth`")
        || message.contains("unknown variant 'live_truth'")
        || message.contains("expected one of fact, decision, preference, runbook, procedural, self_model, topology, status, pattern, constraint")
}

pub(crate) async fn search_live_truth_record(
    client: &MemdClient,
    project: &str,
    namespace: Option<&str>,
    workspace: Option<&str>,
    visibility: Option<memd_schema::MemoryVisibility>,
    legacy_compatible: bool,
) -> anyhow::Result<memd_schema::SearchMemoryResponse> {
    let kinds = if legacy_compatible { Vec::new() } else { vec![MemoryKind::LiveTruth] };
    client
        .search(&SearchMemoryRequest {
            query: None,
            route: Some(RetrievalRoute::LocalFirst),
            intent: Some(RetrievalIntent::CurrentTask),
            scopes: vec![MemoryScope::Local],
            kinds,
            statuses: vec![MemoryStatus::Active],
            project: Some(project.to_string()),
            namespace: namespace.map(ToOwned::to_owned),
            workspace: workspace.map(ToOwned::to_owned),
            visibility,
            belief_branch: None,
            source_agent: Some("memd".to_string()),
            tags: vec!["live_truth".to_string()],
            stages: vec![MemoryStage::Canonical],
            limit: Some(1),
            max_chars_per_item: Some(800),
        })
        .await
}

pub(crate) async fn emit_lane_surface_receipt(
    client: &MemdClient,
    surface: &BundleLaneSurface,
    runtime: &BundleRuntimeConfig,
    actor_session: &str,
) -> anyhow::Result<()> {
    let (kind, summary) = if surface.action == "auto_create" {
        (
            "lane_create",
            format!(
                "Auto-created isolated hive lane from {} to {}.",
                surface.previous_branch.as_deref().unwrap_or("none"),
                surface.current_branch.as_deref().unwrap_or("none"),
            ),
        )
    } else {
        (
            "lane_reroute",
            format!(
                "Auto-rerouted hive lane from {} to {} after collision with {}.",
                surface.previous_branch.as_deref().unwrap_or("none"),
                surface.current_branch.as_deref().unwrap_or("none"),
                surface.conflict_session.as_deref().unwrap_or("unknown"),
            ),
        )
    };
    emit_coordination_receipt(
        client,
        kind,
        actor_session,
        runtime.agent.as_deref().map(|agent| compose_agent_identity(agent, runtime.session.as_deref())),
        surface.conflict_session.clone(),
        None,
        surface.current_branch.clone(),
        runtime.project.clone(),
        runtime.namespace.clone(),
        runtime.workspace.clone(),
        summary,
    )
    .await
}

pub(crate) async fn emit_lane_fault_receipt(
    client: &MemdClient,
    actor_session: &str,
    actor_agent: Option<String>,
    target: &ProjectAwarenessEntry,
    task_id: Option<String>,
    scope: Option<String>,
    project: Option<String>,
    namespace: Option<String>,
    workspace: Option<String>,
) {
    let _ = emit_coordination_receipt(
        client,
        "lane_fault",
        actor_session,
        actor_agent,
        target.session.clone(),
        task_id,
        scope,
        project,
        namespace,
        workspace,
        format!("Queen denied unsafe shared lane target: {}.", render_hive_lane_collision(target)),
    )
    .await;
}

pub(crate) async fn store_live_truth_record(
    client: &MemdClient,
    content: String,
    project: &str,
    namespace: Option<&str>,
    workspace: Option<&str>,
    visibility: Option<memd_schema::MemoryVisibility>,
    project_root: &Path,
    tags: Vec<String>,
) -> anyhow::Result<()> {
    let request = StoreMemoryRequest {
        content: content.clone(),
        kind: MemoryKind::LiveTruth,
        scope: MemoryScope::Local,
        project: Some(project.to_string()),
        namespace: namespace.map(ToOwned::to_owned),
        workspace: workspace.map(ToOwned::to_owned),
        visibility,
        belief_branch: None,
        source_agent: Some("memd".to_string()),
        source_system: Some("memd-live-truth".to_string()),
        source_path: Some(project_root.display().to_string()),
        source_quality: Some(memd_schema::SourceQuality::Derived),
        confidence: Some(0.98),
        ttl_seconds: Some(3_600),
        last_verified_at: Some(Utc::now()),
        supersedes: Vec::new(),
        tags: tags.clone(),
        status: Some(MemoryStatus::Active),
    };

    match client.store(&request).await {
        Ok(_) => Ok(()),
        Err(err) if is_live_truth_kind_rejection(&err) => {
            client
                .store(&StoreMemoryRequest {
                    kind: MemoryKind::Status,
                    source_system: Some("memd-live-truth-compat".to_string()),
                    tags,
                    ..request
                })
                .await?;
            Ok(())
        }
        Err(err) => Err(err),
    }
}
