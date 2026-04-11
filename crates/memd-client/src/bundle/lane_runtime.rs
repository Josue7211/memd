use super::*;

#[derive(Debug, Clone)]
pub(crate) struct BundleLaneIdentity {
    pub(crate) project_root: PathBuf,
    pub(crate) repo_root: String,
    pub(crate) worktree_root: String,
    pub(crate) branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BundleLaneSurface {
    pub(crate) action: String,
    pub(crate) previous_bundle: String,
    pub(crate) current_bundle: String,
    pub(crate) previous_branch: Option<String>,
    pub(crate) current_branch: Option<String>,
    pub(crate) previous_worktree: Option<String>,
    pub(crate) current_worktree: Option<String>,
    pub(crate) conflict_session: Option<String>,
    pub(crate) conflict_branch: Option<String>,
    pub(crate) conflict_worktree: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub(crate) struct EnsuredHiveLane {
    pub(crate) output: PathBuf,
    pub(crate) lane_surface: Option<BundleLaneSurface>,
}

pub(crate) fn detect_project_lane_identity(project_root: &Path) -> Option<BundleLaneIdentity> {
    let repo_root = detect_git_repo_root(project_root)
        .as_deref()
        .map(display_path_nonempty)?;
    let worktree_root = detect_git_worktree_root(project_root)
        .as_deref()
        .map(display_path_nonempty)?;
    let branch = git_stdout(project_root, &["branch", "--show-current"])?;
    Some(BundleLaneIdentity {
        project_root: project_root.to_path_buf(),
        repo_root,
        worktree_root,
        branch,
    })
}

pub(crate) fn detect_bundle_lane_identity(output: &Path) -> Option<BundleLaneIdentity> {
    let project_root = infer_bundle_project_root(output)?;
    detect_project_lane_identity(&project_root)
}

pub(crate) fn awareness_entry_has_same_lane(
    entry: &ProjectAwarenessEntry,
    lane: &BundleLaneIdentity,
    current_bundle: &Path,
    current_session: Option<&str>,
) -> bool {
    if entry.presence != "active" {
        return false;
    }

    let entry_bundle = PathBuf::from(&entry.bundle_root);
    let canonical_entry_bundle = fs::canonicalize(&entry_bundle).unwrap_or(entry_bundle);
    if canonical_entry_bundle == current_bundle {
        return false;
    }

    if current_session.is_some() && entry.session.as_deref() == current_session {
        return false;
    }

    let same_worktree = entry
        .worktree_root
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| value == lane.worktree_root);
    if same_worktree {
        return true;
    }

    entry
        .repo_root
        .as_deref()
        .map(str::trim)
        .zip(entry.branch.as_deref().map(str::trim))
        .is_some_and(|(repo_root, branch)| repo_root == lane.repo_root && branch == lane.branch)
}

pub(crate) fn detect_lane_collision_from_awareness_entries(
    output: &Path,
    current_session: Option<&str>,
    awareness_entries: &[ProjectAwarenessEntry],
) -> Option<ProjectAwarenessEntry> {
    let lane = detect_bundle_lane_identity(output)?;
    let current_bundle = fs::canonicalize(output).unwrap_or_else(|_| output.to_path_buf());
    awareness_entries
        .iter()
        .find(|entry| awareness_entry_has_same_lane(entry, &lane, &current_bundle, current_session))
        .cloned()
}

pub(crate) async fn detect_bundle_lane_collision(
    output: &Path,
    current_session: Option<&str>,
) -> anyhow::Result<Option<ProjectAwarenessEntry>> {
    let awareness = read_project_awareness(&AwarenessArgs {
        output: output.to_path_buf(),
        root: None,
        include_current: true,
        summary: false,
    })
    .await?;
    Ok(detect_lane_collision_from_awareness_entries(
        output,
        current_session,
        &awareness.entries,
    ))
}

pub(crate) fn render_hive_lane_collision(entry: &ProjectAwarenessEntry) -> String {
    let lane = entry
        .session
        .as_deref()
        .or(entry.effective_agent.as_deref())
        .or(entry.agent.as_deref())
        .unwrap_or("unknown");
    format!(
        "session={} branch={} worktree={}",
        lane,
        entry.branch.as_deref().unwrap_or("none"),
        entry.worktree_root.as_deref().unwrap_or("none")
    )
}

pub(crate) fn build_lane_fault_surface(
    output: &Path,
    current_session: Option<&str>,
    conflict: &ProjectAwarenessEntry,
) -> Option<JsonValue> {
    let lane = detect_bundle_lane_identity(output)?;
    let fault_kind = if conflict
        .worktree_root
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| value == lane.worktree_root)
    {
        "unsafe_same_worktree"
    } else {
        "unsafe_same_branch"
    };
    Some(serde_json::json!({
        "kind": fault_kind,
        "session": conflict.session,
        "branch": conflict.branch,
        "worktree_root": conflict.worktree_root,
        "repo_root": conflict.repo_root,
        "current_session": current_session,
        "current_branch": lane.branch,
        "current_worktree": lane.worktree_root,
    }))
}

pub(crate) fn write_bundle_lane_surface(
    output: &Path,
    surface: &BundleLaneSurface,
) -> anyhow::Result<()> {
    let path = bundle_lane_surface_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(surface)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn read_bundle_lane_surface(output: &Path) -> anyhow::Result<Option<BundleLaneSurface>> {
    let path = bundle_lane_surface_path(output);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let surface =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(surface))
}

pub(crate) fn sanitize_lane_segment(value: &str) -> String {
    let mut rendered = String::new();
    let mut last_dash = false;
    for ch in value.chars() {
        let normalized = ch.to_ascii_lowercase();
        if normalized.is_ascii_alphanumeric() {
            rendered.push(normalized);
            last_dash = false;
        } else if !last_dash {
            rendered.push('-');
            last_dash = true;
        }
    }
    let rendered = rendered.trim_matches('-');
    if rendered.is_empty() {
        "worker".to_string()
    } else {
        rendered.to_string()
    }
}

pub(crate) fn create_isolated_hive_worktree(
    lane: &BundleLaneIdentity,
    session_seed: &str,
) -> anyhow::Result<(PathBuf, String)> {
    let worktree_parent = lane
        .project_root
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| lane.project_root.clone());
    let project_name = lane
        .project_root
        .file_name()
        .and_then(|value| value.to_str())
        .map(sanitize_lane_segment)
        .unwrap_or_else(|| "worker".to_string());
    let branch_seed = sanitize_lane_segment(&lane.branch);
    let branch_prefix = format!("{branch_seed}-bee-{session_seed}");

    for attempt in 0..16 {
        let suffix = &uuid::Uuid::new_v4().simple().to_string()[..6];
        let branch_name = if attempt == 0 {
            format!("{branch_prefix}-{suffix}")
        } else {
            format!("{branch_prefix}-{attempt}-{suffix}")
        };
        if git_branch_exists(Path::new(&lane.repo_root), &branch_name) {
            continue;
        }

        let worktree_name = format!("{project_name}-{branch_name}");
        let worktree_root = worktree_parent.join(worktree_name);
        if worktree_root.exists() {
            continue;
        }

        let status = Command::new("git")
            .arg("-C")
            .arg(&lane.repo_root)
            .arg("worktree")
            .arg("add")
            .arg("-b")
            .arg(&branch_name)
            .arg(&worktree_root)
            .arg(&lane.branch)
            .status()
            .with_context(|| format!("create hive worktree {}", worktree_root.display()))?;
        if !status.success() {
            continue;
        }

        return Ok((worktree_root, branch_name));
    }

    anyhow::bail!("failed to allocate an isolated hive lane after multiple attempts");
}

pub(crate) fn hive_role_requires_isolated_lane(
    hive_role: Option<&str>,
    authority: Option<&str>,
) -> bool {
    let coordinator_role = matches!(
        hive_role.map(str::trim),
        Some("orchestrator" | "memory-control-plane")
    );
    let coordinator_authority =
        matches!(authority.map(str::trim), Some("coordinator" | "canonical"));
    !(coordinator_role || coordinator_authority)
}

pub(crate) fn allocate_isolated_hive_lane(
    output: &Path,
    runtime: &BundleRuntimeConfig,
) -> anyhow::Result<PathBuf> {
    let lane = detect_bundle_lane_identity(output)
        .context("hive cowork isolation requires a git worktree and branch")?;
    let session_seed = runtime
        .session
        .as_deref()
        .map(sanitize_lane_segment)
        .unwrap_or_else(|| "worker".to_string());
    let (worktree_root, _) = create_isolated_hive_worktree(&lane, &session_seed)?;

    let new_output = worktree_root.join(".memd");
    let new_session = runtime.session.as_deref().map(|value| {
        format!(
            "{}-{}",
            sanitize_lane_segment(value),
            &uuid::Uuid::new_v4().simple().to_string()[..4]
        )
    });
    write_init_bundle(&InitArgs {
        project: runtime.project.clone(),
        namespace: runtime.namespace.clone(),
        global: false,
        project_root: Some(worktree_root.clone()),
        seed_existing: false,
        agent: runtime.agent.clone().unwrap_or_else(|| "codex".to_string()),
        session: new_session,
        tab_id: runtime.tab_id.clone(),
        hive_system: runtime.hive_system.clone(),
        hive_role: runtime.hive_role.clone(),
        capability: runtime.capabilities.clone(),
        hive_group: runtime.hive_groups.clone(),
        hive_group_goal: runtime.hive_group_goal.clone(),
        authority: runtime.authority.clone(),
        output: new_output.clone(),
        base_url: runtime.base_url.clone().unwrap_or_else(default_base_url),
        rag_url: None,
        route: runtime.route.clone().unwrap_or_else(|| "auto".to_string()),
        intent: runtime
            .intent
            .clone()
            .unwrap_or_else(|| "current_task".to_string()),
        workspace: runtime.workspace.clone(),
        visibility: runtime.visibility.clone(),
        voice_mode: Some(default_voice_mode()),
        allow_localhost_read_only_fallback: false,
        force: true,
    })?;
    Ok(new_output)
}

pub(crate) fn auto_create_worker_hive_lane(
    project_root: &Path,
    requested_output: &Path,
    init_args: &InitArgs,
) -> anyhow::Result<Option<EnsuredHiveLane>> {
    if !hive_role_requires_isolated_lane(
        init_args.hive_role.as_deref(),
        init_args.authority.as_deref(),
    ) {
        return Ok(None);
    }
    let Some(lane) = detect_project_lane_identity(project_root) else {
        return Ok(None);
    };
    let session_seed = init_args
        .session
        .as_deref()
        .map(sanitize_lane_segment)
        .unwrap_or_else(|| sanitize_lane_segment(&init_args.agent));
    let (worktree_root, branch_name) = create_isolated_hive_worktree(&lane, &session_seed)?;
    let new_output = worktree_root.join(".memd");
    let new_session = init_args.session.as_deref().map(|value| {
        format!(
            "{}-{}",
            sanitize_lane_segment(value),
            &uuid::Uuid::new_v4().simple().to_string()[..4]
        )
    });
    write_init_bundle(&InitArgs {
        project_root: Some(worktree_root.clone()),
        output: new_output.clone(),
        session: new_session,
        voice_mode: Some(default_voice_mode()),
        force: true,
        ..init_args.clone()
    })?;
    let surface = BundleLaneSurface {
        action: "auto_create".to_string(),
        previous_bundle: requested_output.display().to_string(),
        current_bundle: new_output.display().to_string(),
        previous_branch: Some(lane.branch),
        current_branch: Some(branch_name),
        previous_worktree: Some(lane.worktree_root),
        current_worktree: Some(display_path_nonempty(&worktree_root)),
        conflict_session: None,
        conflict_branch: None,
        conflict_worktree: None,
        created_at: Utc::now(),
    };
    write_bundle_lane_surface(&new_output, &surface)?;
    Ok(Some(EnsuredHiveLane {
        output: new_output,
        lane_surface: Some(surface),
    }))
}

pub(crate) async fn ensure_isolated_hive_bundle_lane(
    output: &Path,
    runtime: &BundleRuntimeConfig,
) -> anyhow::Result<EnsuredHiveLane> {
    let current_session = runtime.session.as_deref();
    let Some(conflict) = detect_bundle_lane_collision(output, current_session).await? else {
        return Ok(EnsuredHiveLane {
            output: output.to_path_buf(),
            lane_surface: None,
        });
    };
    let source_lane = detect_bundle_lane_identity(output)
        .context("hive cowork isolation requires a git worktree and branch")?;
    let rerouted_output = allocate_isolated_hive_lane(output, runtime).with_context(|| {
        format!(
            "unsafe hive cowork lane collision detected: {}",
            render_hive_lane_collision(&conflict)
        )
    })?;
    let rerouted_lane = detect_bundle_lane_identity(&rerouted_output)
        .context("reload rerouted hive lane identity after worktree creation")?;
    let surface = BundleLaneSurface {
        action: "auto_reroute".to_string(),
        previous_bundle: output.display().to_string(),
        current_bundle: rerouted_output.display().to_string(),
        previous_branch: Some(source_lane.branch),
        current_branch: Some(rerouted_lane.branch),
        previous_worktree: Some(source_lane.worktree_root),
        current_worktree: Some(rerouted_lane.worktree_root),
        conflict_session: conflict.session.clone(),
        conflict_branch: conflict.branch.clone(),
        conflict_worktree: conflict.worktree_root.clone(),
        created_at: Utc::now(),
    };
    write_bundle_lane_surface(&rerouted_output, &surface)?;
    Ok(EnsuredHiveLane {
        output: rerouted_output,
        lane_surface: Some(surface),
    })
}

pub(crate) fn ensure_target_session_lane_is_safe(
    current_output: &Path,
    current_session: Option<&str>,
    target: &ProjectAwarenessEntry,
) -> anyhow::Result<()> {
    let Some(lane) = detect_bundle_lane_identity(current_output) else {
        return Ok(());
    };
    let current_bundle =
        fs::canonicalize(current_output).unwrap_or_else(|_| current_output.to_path_buf());
    if awareness_entry_has_same_lane(target, &lane, &current_bundle, current_session) {
        anyhow::bail!(
            "unsafe hive cowork target collision: {}",
            render_hive_lane_collision(target)
        );
    }
    Ok(())
}
