use super::*;


pub(crate) fn hydrate_experiment_evolution_summary(
    response: &mut ExperimentReport,
    output: &Path,
) -> anyhow::Result<()> {
    response.evolution = experiment_evolution_summary(output)?;
    Ok(())
}

pub(crate) fn write_experiment_artifacts(
    output: &Path,
    response: &ExperimentReport,
) -> anyhow::Result<()> {
    let experiment_dir = experiment_reports_dir(output);
    fs::create_dir_all(&experiment_dir)
        .with_context(|| format!("create {}", experiment_dir.display()))?;

    let proposal = build_evolution_proposal_report(response);
    write_evolution_proposal_artifacts(output, &proposal)?;
    let branch_manifest = create_or_update_evolution_branch(output, &proposal)?;
    write_evolution_branch_artifacts(output, &branch_manifest)?;
    append_evolution_durability_entry(output, &proposal)?;
    append_evolution_authority_entry(output, &proposal)?;
    append_evolution_merge_queue_entry(output, &proposal)?;
    append_evolution_durability_queue_entry(output, &proposal)?;
    process_evolution_queues(output)?;

    let mut enriched = response.clone();
    hydrate_experiment_evolution_summary(&mut enriched, output)?;

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let baseline_json = experiment_dir.join("latest.json");
    let baseline_md = experiment_dir.join("latest.md");
    let timestamp_json = experiment_dir.join(format!("{timestamp}.json"));
    let timestamp_md = experiment_dir.join(format!("{timestamp}.md"));
    let json = serde_json::to_string_pretty(&enriched)? + "\n";
    let markdown = render_experiment_markdown(&enriched);

    fs::write(&baseline_json, &json)
        .with_context(|| format!("write {}", baseline_json.display()))?;
    fs::write(&baseline_md, &markdown)
        .with_context(|| format!("write {}", baseline_md.display()))?;
    fs::write(&timestamp_json, &json)
        .with_context(|| format!("write {}", timestamp_json.display()))?;
    fs::write(&timestamp_md, &markdown)
        .with_context(|| format!("write {}", timestamp_md.display()))?;
    Ok(())
}

pub(crate) fn experiment_reports_dir(output: &Path) -> PathBuf {
    output.join("experiments")
}

pub(crate) fn evolution_reports_dir(output: &Path) -> PathBuf {
    output.join("evolution")
}

pub(crate) fn evolution_durability_ledger_path(output: &Path) -> PathBuf {
    evolution_reports_dir(output).join("durability-ledger.json")
}

pub(crate) fn evolution_authority_ledger_path(output: &Path) -> PathBuf {
    evolution_reports_dir(output).join("authority-ledger.json")
}

pub(crate) fn evolution_merge_queue_path(output: &Path) -> PathBuf {
    evolution_reports_dir(output).join("merge-queue.json")
}

pub(crate) fn evolution_durability_queue_path(output: &Path) -> PathBuf {
    evolution_reports_dir(output).join("durability-queue.json")
}

pub(crate) fn write_evolution_proposal_artifacts(
    output: &Path,
    response: &EvolutionProposalReport,
) -> anyhow::Result<()> {
    let evolution_dir = evolution_reports_dir(output);
    fs::create_dir_all(&evolution_dir)
        .with_context(|| format!("create {}", evolution_dir.display()))?;

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let baseline_json = evolution_dir.join("latest-proposal.json");
    let timestamp_json = evolution_dir.join(format!("proposal-{timestamp}.json"));
    let json = serde_json::to_string_pretty(response)? + "\n";

    fs::write(&baseline_json, &json)
        .with_context(|| format!("write {}", baseline_json.display()))?;
    fs::write(&timestamp_json, &json)
        .with_context(|| format!("write {}", timestamp_json.display()))?;
    Ok(())
}

pub(crate) fn write_evolution_branch_artifacts(
    output: &Path,
    response: &EvolutionBranchManifest,
) -> anyhow::Result<()> {
    let evolution_dir = evolution_reports_dir(output);
    fs::create_dir_all(&evolution_dir)
        .with_context(|| format!("create {}", evolution_dir.display()))?;

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let baseline_json = evolution_dir.join("latest-branch.json");
    let timestamp_json = evolution_dir.join(format!("branch-{timestamp}.json"));
    let json = serde_json::to_string_pretty(response)? + "\n";

    fs::write(&baseline_json, &json)
        .with_context(|| format!("write {}", baseline_json.display()))?;
    fs::write(&timestamp_json, &json)
        .with_context(|| format!("write {}", timestamp_json.display()))?;
    Ok(())
}

pub(crate) fn read_latest_evolution_proposal(
    output: &Path,
) -> anyhow::Result<Option<EvolutionProposalReport>> {
    let path = evolution_reports_dir(output).join("latest-proposal.json");
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let report = serde_json::from_str::<EvolutionProposalReport>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(report))
}

pub(crate) fn read_latest_evolution_branch_manifest(
    output: &Path,
) -> anyhow::Result<Option<EvolutionBranchManifest>> {
    let path = evolution_reports_dir(output).join("latest-branch.json");
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let manifest = serde_json::from_str::<EvolutionBranchManifest>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(manifest))
}

pub(crate) fn read_evolution_durability_ledger(
    output: &Path,
) -> anyhow::Result<Option<EvolutionDurabilityLedger>> {
    let path = evolution_durability_ledger_path(output);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let ledger = serde_json::from_str::<EvolutionDurabilityLedger>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(ledger))
}

pub(crate) fn read_evolution_authority_ledger(
    output: &Path,
) -> anyhow::Result<Option<EvolutionAuthorityLedger>> {
    let path = evolution_authority_ledger_path(output);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let ledger = serde_json::from_str::<EvolutionAuthorityLedger>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(ledger))
}

pub(crate) fn read_evolution_merge_queue(output: &Path) -> anyhow::Result<Option<EvolutionMergeQueue>> {
    let path = evolution_merge_queue_path(output);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let queue = serde_json::from_str::<EvolutionMergeQueue>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(queue))
}

pub(crate) fn read_evolution_durability_queue(
    output: &Path,
) -> anyhow::Result<Option<EvolutionDurabilityQueue>> {
    let path = evolution_durability_queue_path(output);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let queue = serde_json::from_str::<EvolutionDurabilityQueue>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(queue))
}

pub(crate) fn write_evolution_merge_queue(output: &Path, queue: &EvolutionMergeQueue) -> anyhow::Result<()> {
    let path = evolution_merge_queue_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(queue)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn write_evolution_durability_ledger(
    output: &Path,
    ledger: &EvolutionDurabilityLedger,
) -> anyhow::Result<()> {
    let path = evolution_durability_ledger_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(ledger)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn write_evolution_authority_ledger(
    output: &Path,
    ledger: &EvolutionAuthorityLedger,
) -> anyhow::Result<()> {
    let path = evolution_authority_ledger_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(ledger)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn write_evolution_durability_queue(
    output: &Path,
    queue: &EvolutionDurabilityQueue,
) -> anyhow::Result<()> {
    let path = evolution_durability_queue_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(queue)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn append_evolution_durability_entry(
    output: &Path,
    proposal: &EvolutionProposalReport,
) -> anyhow::Result<()> {
    let mut ledger = read_evolution_durability_ledger(output)?.unwrap_or_default();
    ledger.entries.push(EvolutionDurabilityEntry {
        proposal_id: proposal.proposal_id.clone(),
        branch: proposal.branch.clone(),
        branch_prefix: format!(
            "auto/evolution/{}/{}",
            branch_safe_slug(&proposal.scope_class),
            branch_safe_slug(&proposal.topic)
        ),
        state: proposal.state.clone(),
        scope_class: proposal.scope_class.clone(),
        scope_gate: proposal.scope_gate.clone(),
        merge_eligible: proposal.merge_eligible,
        durable_truth: proposal.durable_truth,
        recorded_at: proposal.generated_at,
    });
    write_evolution_durability_ledger(output, &ledger)
}

pub(crate) fn append_evolution_authority_entry(
    output: &Path,
    proposal: &EvolutionProposalReport,
) -> anyhow::Result<()> {
    let mut ledger = read_evolution_authority_ledger(output)?.unwrap_or_default();
    ledger.entries.push(EvolutionAuthorityEntry {
        scope_class: proposal.scope_class.clone(),
        authority_tier: proposal.authority_tier.clone(),
        accepted: proposal.accepted,
        merged: proposal.state == "merged" || proposal.state == "durable_truth",
        durable_truth: proposal.durable_truth,
        proposal_id: proposal.proposal_id.clone(),
        branch: proposal.branch.clone(),
        recorded_at: proposal.generated_at,
    });
    write_evolution_authority_ledger(output, &ledger)
}

pub(crate) fn append_evolution_merge_queue_entry(
    output: &Path,
    proposal: &EvolutionProposalReport,
) -> anyhow::Result<()> {
    let path = evolution_merge_queue_path(output);
    let mut queue = read_evolution_merge_queue(output)?.unwrap_or_default();
    queue.entries.push(EvolutionMergeQueueEntry {
        proposal_id: proposal.proposal_id.clone(),
        branch: proposal.branch.clone(),
        scope_class: proposal.scope_class.clone(),
        scope_gate: proposal.scope_gate.clone(),
        authority_tier: proposal.authority_tier.clone(),
        status: if proposal.merge_eligible {
            "pending_merge".to_string()
        } else {
            "human_review".to_string()
        },
        merge_eligible: proposal.merge_eligible,
        recorded_at: proposal.generated_at,
    });
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(&queue)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn append_evolution_durability_queue_entry(
    output: &Path,
    proposal: &EvolutionProposalReport,
) -> anyhow::Result<()> {
    let path = evolution_durability_queue_path(output);
    let mut queue = read_evolution_durability_queue(output)?.unwrap_or_default();
    queue.entries.push(EvolutionDurabilityQueueEntry {
        proposal_id: proposal.proposal_id.clone(),
        branch: proposal.branch.clone(),
        state: proposal.state.clone(),
        status: if proposal.state == "merged" || proposal.state == "durable_truth" {
            "scheduled".to_string()
        } else if !proposal.merge_eligible {
            "human_review".to_string()
        } else {
            "waiting_for_merge".to_string()
        },
        due_at: proposal.durability_due_at,
        recorded_at: proposal.generated_at,
    });
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(&queue)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn process_evolution_queues(output: &Path) -> anyhow::Result<()> {
    process_evolution_merge_queue(output)?;
    process_evolution_durability_queue(output)?;
    Ok(())
}

pub(crate) fn process_evolution_merge_queue(output: &Path) -> anyhow::Result<()> {
    let Some(mut queue) = read_evolution_merge_queue(output)? else {
        return Ok(());
    };
    let project_root = infer_bundle_project_root(output);
    for entry in &mut queue.entries {
        if entry.status == "merged"
            || entry.status == "human_review" && entry.authority_tier == "proposal_only"
        {
            continue;
        }
        let Some(root) = project_root.as_ref() else {
            entry.status = "blocked_no_project_root".to_string();
            continue;
        };
        let Some(base_branch) = git_stdout(root, &["branch", "--show-current"]) else {
            entry.status = "blocked_no_base".to_string();
            continue;
        };
        let worktree_dirty = git_worktree_dirty(root);
        if worktree_dirty && git_worktree_conflicts_with_branch(root, &base_branch, &entry.branch) {
            entry.status = "blocked_dirty_worktree".to_string();
            continue;
        }
        if !git_branch_exists(root, &entry.branch) {
            entry.status = "blocked_missing_branch".to_string();
            continue;
        }
        if !git_branch_has_diff(root, &base_branch, &entry.branch) {
            entry.status = "no_diff".to_string();
            continue;
        }
        let evaluated_status = if entry.authority_tier == "proposal_only" {
            "human_review".to_string()
        } else {
            "merge_ready".to_string()
        };
        if evaluated_status == "merge_ready" {
            entry.status = if worktree_dirty {
                execute_evolution_merge_in_isolated_worktree(output, root, entry, &base_branch)?
            } else {
                execute_evolution_merge(output, root, entry, &base_branch)?
            };
        } else {
            entry.status = evaluated_status;
        }
    }
    write_evolution_merge_queue(output, &queue)?;
    Ok(())
}

pub(crate) fn process_evolution_durability_queue(output: &Path) -> anyhow::Result<()> {
    let Some(mut queue) = read_evolution_durability_queue(output)? else {
        return Ok(());
    };
    let merge_queue = read_evolution_merge_queue(output)?.unwrap_or_default();
    for entry in &mut queue.entries {
        if entry.status == "scheduled" {
            entry.status = execute_evolution_durability_check(output, entry)?;
            continue;
        }
        if matches!(entry.status.as_str(), "verified" | "regressed") {
            continue;
        }
        let merge_status = merge_queue
            .entries
            .iter()
            .rev()
            .find(|candidate| candidate.proposal_id == entry.proposal_id)
            .map(|candidate| candidate.status.as_str())
            .unwrap_or("unknown");
        entry.status = match merge_status {
            "merge_ready" => "waiting_for_merge".to_string(),
            "merged" => "scheduled".to_string(),
            "human_review" => "human_review".to_string(),
            "no_diff" => "no_diff".to_string(),
            "blocked_no_base" => "blocked_no_base".to_string(),
            "blocked_missing_branch" => "blocked_missing_branch".to_string(),
            _ => entry.status.clone(),
        };
    }
    write_evolution_durability_queue(output, &queue)?;
    Ok(())
}

pub(crate) fn execute_evolution_durability_check(
    output: &Path,
    entry: &EvolutionDurabilityQueueEntry,
) -> anyhow::Result<String> {
    let Some(due_at) = entry.due_at else {
        return Ok("scheduled".to_string());
    };
    if due_at > Utc::now() {
        return Ok("scheduled".to_string());
    }
    let Some(root) = infer_bundle_project_root(output) else {
        return Ok("blocked_no_project_root".to_string());
    };
    if git_worktree_dirty(&root) {
        return Ok("blocked_dirty_worktree".to_string());
    }
    if !git_branch_exists(&root, &entry.branch) {
        return Ok("blocked_missing_branch".to_string());
    }
    if !git_branch_tip_ancestor_of_head(&root, &entry.branch) {
        transition_evolution_proposal_state(
            output,
            &entry.proposal_id,
            "merged",
            false,
            Some(due_at),
        )?;
        transition_evolution_branch_state(output, &entry.proposal_id, "merged", false)?;
        return Ok("regressed".to_string());
    }
    transition_evolution_proposal_state(
        output,
        &entry.proposal_id,
        "durable_truth",
        true,
        Some(due_at),
    )?;
    transition_evolution_branch_state(output, &entry.proposal_id, "durable_truth", true)?;
    append_evolution_durability_transition_from_queue(output, entry, "durable_truth", true)?;
    append_evolution_authority_transition_from_queue(output, entry, "durable_truth", true)?;
    Ok("verified".to_string())
}

pub(crate) fn execute_evolution_merge(
    output: &Path,
    root: &Path,
    entry: &EvolutionMergeQueueEntry,
    base_branch: &str,
) -> anyhow::Result<String> {
    let current_branch = git_stdout(root, &["branch", "--show-current"]);
    if current_branch.as_deref() != Some(base_branch) {
        return Ok("blocked_wrong_base_branch".to_string());
    }

    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("merge")
        .arg("--ff-only")
        .arg(&entry.branch)
        .status();
    let Ok(status) = status else {
        return Ok("merge_error".to_string());
    };
    if !status.success() {
        return Ok("merge_conflict".to_string());
    }

    let due_at = Some(Utc::now() + chrono::TimeDelta::hours(1));
    transition_evolution_proposal_state(output, &entry.proposal_id, "merged", false, due_at)?;
    transition_evolution_branch_state(output, &entry.proposal_id, "merged", false)?;
    append_evolution_durability_transition(output, entry, "merged", false)?;
    append_evolution_authority_transition(output, entry, "merged", false)?;
    Ok("merged".to_string())
}

pub(crate) fn execute_evolution_merge_in_isolated_worktree(
    output: &Path,
    root: &Path,
    entry: &EvolutionMergeQueueEntry,
    base_branch: &str,
) -> anyhow::Result<String> {
    let base_sha = match git_stdout(root, &["rev-parse", base_branch]) {
        Some(value) => value,
        None => return Ok("blocked_no_base".to_string()),
    };
    let tempdir =
        std::env::temp_dir().join(format!("memd-evolution-merge-{}", uuid::Uuid::new_v4()));
    let add_status = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("worktree")
        .arg("add")
        .arg("--detach")
        .arg(&tempdir)
        .arg(&base_sha)
        .status();
    let Ok(add_status) = add_status else {
        return Ok("merge_error".to_string());
    };
    if !add_status.success() {
        return Ok("merge_error".to_string());
    }

    let result = (|| -> anyhow::Result<String> {
        let merge_status = Command::new("git")
            .arg("-C")
            .arg(&tempdir)
            .arg("merge")
            .arg("--ff-only")
            .arg(&entry.branch)
            .status()
            .context("run isolated ff merge")?;
        if !merge_status.success() {
            return Ok("merge_conflict".to_string());
        }

        let Some(merged_sha) = git_stdout(&tempdir, &["rev-parse", "HEAD"]) else {
            return Ok("merge_error".to_string());
        };
        let update_status = Command::new("git")
            .arg("-C")
            .arg(root)
            .arg("update-ref")
            .arg(format!("refs/heads/{base_branch}"))
            .arg(&merged_sha)
            .arg(&base_sha)
            .status()
            .context("update branch ref after isolated merge")?;
        if !update_status.success() {
            return Ok("merge_error".to_string());
        }

        let due_at = Some(Utc::now() + chrono::TimeDelta::hours(1));
        transition_evolution_proposal_state(output, &entry.proposal_id, "merged", false, due_at)?;
        transition_evolution_branch_state(output, &entry.proposal_id, "merged", false)?;
        append_evolution_durability_transition(output, entry, "merged", false)?;
        append_evolution_authority_transition(output, entry, "merged", false)?;
        Ok("merged".to_string())
    })();

    let _ = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("worktree")
        .arg("remove")
        .arg("--force")
        .arg(&tempdir)
        .status();

    result
}

pub(crate) fn transition_evolution_proposal_state(
    output: &Path,
    proposal_id: &str,
    state: &str,
    durable_truth: bool,
    durability_due_at: Option<DateTime<Utc>>,
) -> anyhow::Result<()> {
    let Some(mut proposal) = read_latest_evolution_proposal(output)? else {
        return Ok(());
    };
    if proposal.proposal_id != proposal_id {
        return Ok(());
    }
    proposal.state = state.to_string();
    proposal.durable_truth = durable_truth;
    proposal.durability_due_at = durability_due_at;
    write_evolution_proposal_artifacts(output, &proposal)?;
    Ok(())
}

pub(crate) fn transition_evolution_branch_state(
    output: &Path,
    proposal_id: &str,
    status: &str,
    durable_truth: bool,
) -> anyhow::Result<()> {
    let Some(mut manifest) = read_latest_evolution_branch_manifest(output)? else {
        return Ok(());
    };
    if manifest.proposal_id != proposal_id {
        return Ok(());
    }
    manifest.status = status.to_string();
    manifest.durable_truth = durable_truth;
    write_evolution_branch_artifacts(output, &manifest)?;
    Ok(())
}

pub(crate) fn append_evolution_durability_transition(
    output: &Path,
    entry: &EvolutionMergeQueueEntry,
    state: &str,
    durable_truth: bool,
) -> anyhow::Result<()> {
    let path = evolution_durability_ledger_path(output);
    let mut ledger = read_evolution_durability_ledger(output)?.unwrap_or_default();
    ledger.entries.push(EvolutionDurabilityEntry {
        proposal_id: entry.proposal_id.clone(),
        branch: entry.branch.clone(),
        branch_prefix: branch_prefix_from_branch_name(&entry.branch),
        state: state.to_string(),
        scope_class: entry.scope_class.clone(),
        scope_gate: entry.scope_gate.clone(),
        merge_eligible: entry.merge_eligible,
        durable_truth,
        recorded_at: Utc::now(),
    });
    let json = serde_json::to_string_pretty(&ledger)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn append_evolution_authority_transition(
    output: &Path,
    entry: &EvolutionMergeQueueEntry,
    state: &str,
    durable_truth: bool,
) -> anyhow::Result<()> {
    let path = evolution_authority_ledger_path(output);
    let mut ledger = read_evolution_authority_ledger(output)?.unwrap_or_default();
    ledger.entries.push(EvolutionAuthorityEntry {
        scope_class: entry.scope_class.clone(),
        authority_tier: entry.authority_tier.clone(),
        accepted: true,
        merged: state == "merged" || state == "durable_truth",
        durable_truth,
        proposal_id: entry.proposal_id.clone(),
        branch: entry.branch.clone(),
        recorded_at: Utc::now(),
    });
    let json = serde_json::to_string_pretty(&ledger)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn append_evolution_durability_transition_from_queue(
    output: &Path,
    entry: &EvolutionDurabilityQueueEntry,
    state: &str,
    durable_truth: bool,
) -> anyhow::Result<()> {
    let path = evolution_durability_ledger_path(output);
    let mut ledger = read_evolution_durability_ledger(output)?.unwrap_or_default();
    let proposal = read_latest_evolution_proposal(output)?
        .filter(|proposal| proposal.proposal_id == entry.proposal_id);
    ledger.entries.push(EvolutionDurabilityEntry {
        proposal_id: entry.proposal_id.clone(),
        branch: entry.branch.clone(),
        branch_prefix: branch_prefix_from_branch_name(&entry.branch),
        state: state.to_string(),
        scope_class: proposal
            .as_ref()
            .map(|value| value.scope_class.clone())
            .unwrap_or_else(|| "unknown".to_string()),
        scope_gate: proposal
            .as_ref()
            .map(|value| value.scope_gate.clone())
            .unwrap_or_else(|| "proposal_only".to_string()),
        merge_eligible: proposal.as_ref().is_some_and(|value| value.merge_eligible),
        durable_truth,
        recorded_at: Utc::now(),
    });
    let json = serde_json::to_string_pretty(&ledger)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn append_evolution_authority_transition_from_queue(
    output: &Path,
    entry: &EvolutionDurabilityQueueEntry,
    state: &str,
    durable_truth: bool,
) -> anyhow::Result<()> {
    let path = evolution_authority_ledger_path(output);
    let mut ledger = read_evolution_authority_ledger(output)?.unwrap_or_default();
    let proposal = read_latest_evolution_proposal(output)?
        .filter(|proposal| proposal.proposal_id == entry.proposal_id);
    ledger.entries.push(EvolutionAuthorityEntry {
        scope_class: proposal
            .as_ref()
            .map(|value| value.scope_class.clone())
            .unwrap_or_else(|| "unknown".to_string()),
        authority_tier: proposal
            .as_ref()
            .map(|value| value.authority_tier.clone())
            .unwrap_or_else(default_evolution_authority_tier),
        accepted: true,
        merged: state == "merged" || state == "durable_truth",
        durable_truth,
        proposal_id: entry.proposal_id.clone(),
        branch: entry.branch.clone(),
        recorded_at: Utc::now(),
    });
    let json = serde_json::to_string_pretty(&ledger)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn create_or_update_evolution_branch(
    output: &Path,
    proposal: &EvolutionProposalReport,
) -> anyhow::Result<EvolutionBranchManifest> {
    let branch_prefix = format!(
        "auto/evolution/{}/{}",
        branch_safe_slug(&proposal.scope_class),
        branch_safe_slug(&proposal.topic)
    );
    let Some(project_root) = infer_bundle_project_root(output) else {
        return Ok(EvolutionBranchManifest {
            proposal_id: proposal.proposal_id.clone(),
            branch: proposal.branch.clone(),
            branch_prefix,
            project_root: None,
            head_sha: None,
            base_branch: None,
            status: "no_project_root".to_string(),
            merge_eligible: proposal.merge_eligible,
            durable_truth: proposal.durable_truth,
            scope_class: proposal.scope_class.clone(),
            scope_gate: proposal.scope_gate.clone(),
            generated_at: proposal.generated_at,
            notes: vec!["bundle is not attached to a detectable project root".to_string()],
        });
    };

    let head_sha = git_stdout(&project_root, &["rev-parse", "HEAD"]);
    let base_branch = git_stdout(&project_root, &["branch", "--show-current"]);

    if !proposal.accepted {
        return Ok(EvolutionBranchManifest {
            proposal_id: proposal.proposal_id.clone(),
            branch: proposal.branch.clone(),
            branch_prefix,
            project_root: Some(display_path_nonempty(&project_root)),
            head_sha,
            base_branch,
            status: "rejected".to_string(),
            merge_eligible: proposal.merge_eligible,
            durable_truth: proposal.durable_truth,
            scope_class: proposal.scope_class.clone(),
            scope_gate: proposal.scope_gate.clone(),
            generated_at: proposal.generated_at,
            notes: vec!["rejected proposals do not create evolution branches".to_string()],
        });
    }

    let exists = Command::new("git")
        .arg("-C")
        .arg(&project_root)
        .arg("show-ref")
        .arg("--verify")
        .arg(format!("refs/heads/{}", proposal.branch))
        .output()
        .ok()
        .is_some_and(|output| output.status.success());

    let status = if exists {
        "existing".to_string()
    } else {
        let mut cmd = Command::new("git");
        cmd.arg("-C")
            .arg(&project_root)
            .arg("branch")
            .arg(&proposal.branch);
        if let Some(head) = head_sha.as_deref() {
            cmd.arg(head);
        }
        match cmd.output() {
            Ok(output) if output.status.success() => "created".to_string(),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                return Ok(EvolutionBranchManifest {
                    proposal_id: proposal.proposal_id.clone(),
                    branch: proposal.branch.clone(),
                    branch_prefix,
                    project_root: Some(display_path_nonempty(&project_root)),
                    head_sha,
                    base_branch,
                    status: "branch_error".to_string(),
                    merge_eligible: proposal.merge_eligible,
                    durable_truth: proposal.durable_truth,
                    scope_class: proposal.scope_class.clone(),
                    scope_gate: proposal.scope_gate.clone(),
                    generated_at: proposal.generated_at,
                    notes: vec![if stderr.is_empty() {
                        "git branch creation failed".to_string()
                    } else {
                        stderr
                    }],
                });
            }
            Err(err) => {
                return Ok(EvolutionBranchManifest {
                    proposal_id: proposal.proposal_id.clone(),
                    branch: proposal.branch.clone(),
                    branch_prefix,
                    project_root: Some(display_path_nonempty(&project_root)),
                    head_sha,
                    base_branch,
                    status: "branch_error".to_string(),
                    merge_eligible: proposal.merge_eligible,
                    durable_truth: proposal.durable_truth,
                    scope_class: proposal.scope_class.clone(),
                    scope_gate: proposal.scope_gate.clone(),
                    generated_at: proposal.generated_at,
                    notes: vec![format!("git branch creation failed: {err}")],
                });
            }
        }
    };

    Ok(EvolutionBranchManifest {
        proposal_id: proposal.proposal_id.clone(),
        branch: proposal.branch.clone(),
        branch_prefix,
        project_root: Some(display_path_nonempty(&project_root)),
        head_sha,
        base_branch,
        status,
        merge_eligible: proposal.merge_eligible,
        durable_truth: proposal.durable_truth,
        scope_class: proposal.scope_class.clone(),
        scope_gate: proposal.scope_gate.clone(),
        generated_at: proposal.generated_at,
        notes: vec!["evolution branch isolated from active working branch".to_string()],
    })
}

pub(crate) fn git_stdout(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(crate) fn detect_git_worktree_root(root: &Path) -> Option<PathBuf> {
    git_stdout(root, &["rev-parse", "--show-toplevel"]).map(PathBuf::from)
}

pub(crate) fn detect_git_repo_root(root: &Path) -> Option<PathBuf> {
    let common_dir = git_stdout(
        root,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )
    .map(PathBuf::from)?;
    if common_dir
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value == ".git")
    {
        common_dir.parent().map(Path::to_path_buf)
    } else {
        detect_git_worktree_root(root)
    }
}

pub(crate) fn git_worktree_dirty(root: &Path) -> bool {
    git_dirty_paths(root).is_some_and(|paths| !paths.is_empty())
}

pub(crate) fn git_dirty_paths(root: &Path) -> Option<BTreeSet<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("status")
        .arg("--porcelain")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(parse_git_status_path)
            .filter(|path| !is_bundle_generated_path(path))
            .collect(),
    )
}

pub(crate) fn git_branch_changed_paths(
    root: &Path,
    base_branch: &str,
    branch: &str,
) -> Option<BTreeSet<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("diff")
        .arg("--name-only")
        .arg(format!("{base_branch}..{branch}"))
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .map(str::to_string)
            .collect(),
    )
}

pub(crate) fn git_worktree_conflicts_with_branch(root: &Path, base_branch: &str, branch: &str) -> bool {
    let Some(dirty_paths) = git_dirty_paths(root) else {
        return git_worktree_dirty(root);
    };
    if dirty_paths.is_empty() {
        return false;
    }
    let Some(branch_paths) = git_branch_changed_paths(root, base_branch, branch) else {
        return true;
    };
    if branch_paths.is_empty() {
        return false;
    }
    dirty_paths.iter().any(|path| branch_paths.contains(path))
}

pub(crate) fn parse_git_status_path(line: &str) -> Option<String> {
    if line.len() < 4 {
        return None;
    }
    let path = line.get(3..)?.trim();
    if path.is_empty() {
        return None;
    }
    if let Some((_, renamed)) = path.split_once(" -> ") {
        return Some(renamed.trim().to_string());
    }
    Some(path.to_string())
}

pub(crate) fn is_bundle_generated_path(path: &str) -> bool {
    let normalized = path.trim_start_matches("./");
    normalized == ".memd" || normalized.starts_with(".memd/") || normalized.contains("/.memd/")
}

pub(crate) fn branch_prefix_from_branch_name(branch: &str) -> String {
    branch
        .rsplit_once('/')
        .map(|(prefix, _)| prefix.to_string())
        .unwrap_or_else(|| branch.to_string())
}

pub(crate) fn git_branch_exists(root: &Path, branch: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("show-ref")
        .arg("--verify")
        .arg(format!("refs/heads/{branch}"))
        .output()
        .ok()
        .is_some_and(|output| output.status.success())
}

pub(crate) fn git_branch_has_diff(root: &Path, base_branch: &str, branch: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("diff")
        .arg("--quiet")
        .arg(format!("{base_branch}..{branch}"))
        .status()
        .ok()
        .is_some_and(|status| !status.success())
}

pub(crate) fn git_branch_tip_ancestor_of_head(root: &Path, branch: &str) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("merge-base")
        .arg("--is-ancestor")
        .arg(branch)
        .arg("HEAD")
        .status()
        .ok()
        .is_some_and(|status| status.success())
}

pub(crate) fn display_path_nonempty(path: &Path) -> String {
    let rendered = path.display().to_string();
    if rendered.is_empty() {
        ".".to_string()
    } else {
        rendered
    }
}

pub(crate) fn compute_evolution_authority_tier(output: &Path, scope_class: &str, scope_gate: &str) -> String {
    if scope_gate != "auto_merge" {
        return "proposal_only".to_string();
    }
    let recent = read_evolution_authority_ledger(output)
        .ok()
        .flatten()
        .map(|ledger| {
            ledger
                .entries
                .into_iter()
                .filter(|entry| entry.scope_class == scope_class)
                .rev()
                .take(3)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if recent.len() >= 2 && recent.iter().take(2).any(|entry| !entry.accepted) {
        return "proposal_only".to_string();
    }
    if recent.len() >= 3 && recent.iter().take(3).all(|entry| entry.durable_truth) {
        return "durable_auto_merge".to_string();
    }
    "phase1_auto_merge".to_string()
}

pub(crate) fn default_evolution_authority_tier() -> String {
    "proposal_only".to_string()
}

pub(crate) fn ensure_evolution_artifacts(output: &Path, report: &ExperimentReport) -> anyhow::Result<()> {
    let built = build_evolution_proposal_report(report);
    let proposal = if let Some(existing) = read_latest_evolution_proposal(output)? {
        if evolution_proposal_needs_refresh(&existing, &built) {
            sync_latest_evolution_artifacts(output, &built)?;
            built
        } else {
            existing
        }
    } else {
        sync_latest_evolution_artifacts(output, &built)?;
        built
    };
    let existing_branch_manifest = read_latest_evolution_branch_manifest(output)?;
    if !existing_branch_manifest
        .as_ref()
        .is_some_and(|manifest| !manifest.project_root.as_deref().unwrap_or("").is_empty())
    {
        let branch_manifest = create_or_update_evolution_branch(output, &proposal)?;
        write_evolution_branch_artifacts(output, &branch_manifest)?;
    }
    if read_evolution_durability_ledger(output)?.is_none() {
        append_evolution_durability_entry(output, &proposal)?;
    }
    if read_evolution_authority_ledger(output)?.is_none() {
        append_evolution_authority_entry(output, &proposal)?;
    }
    if read_evolution_merge_queue(output)?.is_none() {
        append_evolution_merge_queue_entry(output, &proposal)?;
    }
    if read_evolution_durability_queue(output)?.is_none() {
        append_evolution_durability_queue_entry(output, &proposal)?;
    }
    process_evolution_queues(output)?;
    Ok(())
}

pub(crate) fn evolution_proposal_needs_refresh(
    existing: &EvolutionProposalReport,
    built: &EvolutionProposalReport,
) -> bool {
    existing.scope_class != built.scope_class
        || existing.scope_gate != built.scope_gate
        || existing.authority_tier != built.authority_tier
        || existing.branch != built.branch
        || existing.state != built.state
        || existing.merge_eligible != built.merge_eligible
        || existing.durable_truth != built.durable_truth
        || existing.allowed_write_surface != built.allowed_write_surface
        || existing.scope_reasons != built.scope_reasons
}

pub(crate) fn sync_latest_evolution_artifacts(
    output: &Path,
    proposal: &EvolutionProposalReport,
) -> anyhow::Result<()> {
    write_evolution_proposal_artifacts(output, proposal)?;
    let branch_manifest = create_or_update_evolution_branch(output, proposal)?;
    write_evolution_branch_artifacts(output, &branch_manifest)?;
    upsert_evolution_durability_entry(output, proposal)?;
    upsert_evolution_authority_entry(output, proposal)?;
    upsert_evolution_merge_queue_entry(output, proposal)?;
    upsert_evolution_durability_queue_entry(output, proposal)?;
    Ok(())
}

pub(crate) fn upsert_evolution_durability_entry(
    output: &Path,
    proposal: &EvolutionProposalReport,
) -> anyhow::Result<()> {
    let mut ledger = read_evolution_durability_ledger(output)?.unwrap_or_default();
    let next = EvolutionDurabilityEntry {
        proposal_id: proposal.proposal_id.clone(),
        branch: proposal.branch.clone(),
        branch_prefix: format!(
            "auto/evolution/{}/{}",
            branch_safe_slug(&proposal.scope_class),
            branch_safe_slug(&proposal.topic)
        ),
        state: proposal.state.clone(),
        scope_class: proposal.scope_class.clone(),
        scope_gate: proposal.scope_gate.clone(),
        merge_eligible: proposal.merge_eligible,
        durable_truth: proposal.durable_truth,
        recorded_at: proposal.generated_at,
    };
    if let Some(index) = ledger
        .entries
        .iter()
        .rposition(|entry| entry.proposal_id == proposal.proposal_id)
    {
        ledger.entries[index] = next;
    } else {
        ledger.entries.push(next);
    }
    write_evolution_durability_ledger(output, &ledger)
}

pub(crate) fn upsert_evolution_authority_entry(
    output: &Path,
    proposal: &EvolutionProposalReport,
) -> anyhow::Result<()> {
    let mut ledger = read_evolution_authority_ledger(output)?.unwrap_or_default();
    let next = EvolutionAuthorityEntry {
        scope_class: proposal.scope_class.clone(),
        authority_tier: proposal.authority_tier.clone(),
        accepted: proposal.accepted,
        merged: proposal.state == "merged" || proposal.state == "durable_truth",
        durable_truth: proposal.durable_truth,
        proposal_id: proposal.proposal_id.clone(),
        branch: proposal.branch.clone(),
        recorded_at: proposal.generated_at,
    };
    if let Some(index) = ledger
        .entries
        .iter()
        .rposition(|entry| entry.proposal_id == proposal.proposal_id)
    {
        ledger.entries[index] = next;
    } else {
        ledger.entries.push(next);
    }
    write_evolution_authority_ledger(output, &ledger)
}

pub(crate) fn upsert_evolution_merge_queue_entry(
    output: &Path,
    proposal: &EvolutionProposalReport,
) -> anyhow::Result<()> {
    let mut queue = read_evolution_merge_queue(output)?.unwrap_or_default();
    let next = EvolutionMergeQueueEntry {
        proposal_id: proposal.proposal_id.clone(),
        branch: proposal.branch.clone(),
        scope_class: proposal.scope_class.clone(),
        scope_gate: proposal.scope_gate.clone(),
        authority_tier: proposal.authority_tier.clone(),
        status: if proposal.merge_eligible {
            "pending_merge".to_string()
        } else {
            "human_review".to_string()
        },
        merge_eligible: proposal.merge_eligible,
        recorded_at: proposal.generated_at,
    };
    if let Some(index) = queue
        .entries
        .iter()
        .rposition(|entry| entry.proposal_id == proposal.proposal_id)
    {
        queue.entries[index] = next;
    } else {
        queue.entries.push(next);
    }
    write_evolution_merge_queue(output, &queue)
}

pub(crate) fn upsert_evolution_durability_queue_entry(
    output: &Path,
    proposal: &EvolutionProposalReport,
) -> anyhow::Result<()> {
    let mut queue = read_evolution_durability_queue(output)?.unwrap_or_default();
    let next = EvolutionDurabilityQueueEntry {
        proposal_id: proposal.proposal_id.clone(),
        branch: proposal.branch.clone(),
        state: proposal.state.clone(),
        status: if proposal.state == "merged" || proposal.state == "durable_truth" {
            "scheduled".to_string()
        } else if !proposal.merge_eligible {
            "human_review".to_string()
        } else {
            "waiting_for_merge".to_string()
        },
        due_at: proposal.durability_due_at,
        recorded_at: proposal.generated_at,
    };
    if let Some(index) = queue
        .entries
        .iter()
        .rposition(|entry| entry.proposal_id == proposal.proposal_id)
    {
        queue.entries[index] = next;
    } else {
        queue.entries.push(next);
    }
    write_evolution_durability_queue(output, &queue)
}

#[derive(Debug, Clone)]
pub(crate) struct EvolutionScopeAssessment {
    pub(crate) topic: String,
    pub(crate) scope_class: String,
    pub(crate) scope_gate: String,
    pub(crate) allowed_write_surface: Vec<String>,
    pub(crate) scope_reasons: Vec<String>,
}

pub(crate) fn build_evolution_proposal_report(report: &ExperimentReport) -> EvolutionProposalReport {
    let scope = classify_evolution_scope(report);
    let branch = evolution_branch_name(&scope, report.completed_at);
    let authority_tier = compute_evolution_authority_tier(
        Path::new(&report.bundle_root),
        &scope.scope_class,
        &scope.scope_gate,
    );
    let merge_eligible =
        report.accepted && scope.scope_gate == "auto_merge" && authority_tier != "proposal_only";
    let prior_ledger = read_evolution_durability_ledger(Path::new(&report.bundle_root))
        .ok()
        .flatten()
        .unwrap_or_default();
    let prior_merged = prior_ledger
        .entries
        .iter()
        .rev()
        .find(|entry| entry.branch_prefix == evolution_branch_prefix(&scope))
        .is_some_and(|entry| entry.state == "merged" || entry.state == "durable_truth");
    let state = if !report.accepted {
        "rejected".to_string()
    } else if merge_eligible && prior_merged {
        "durable_truth".to_string()
    } else if merge_eligible && report.apply {
        "merged".to_string()
    } else {
        "accepted_proposal".to_string()
    };
    let evidence = vec![
        format!("accepted={}", report.accepted),
        format!("restored={}", report.restored),
        format!("scope_class={}", scope.scope_class),
        format!("scope_gate={}", scope.scope_gate),
        format!(
            "composite_score={}/{}",
            report.composite.score, report.composite.max_score
        ),
    ];
    EvolutionProposalReport {
        bundle_root: report.bundle_root.clone(),
        project: report.project.clone(),
        namespace: report.namespace.clone(),
        agent: report.agent.clone(),
        session: report.session.clone(),
        workspace: report.workspace.clone(),
        visibility: report.visibility.clone(),
        proposal_id: format!(
            "{}-{}",
            canonical_slug(
                report
                    .composite
                    .scenario
                    .as_deref()
                    .unwrap_or("self-evolution")
            ),
            report.completed_at.format("%Y%m%dT%H%M%SZ")
        ),
        scenario: report.composite.scenario.clone(),
        topic: scope.topic,
        branch,
        state: state.clone(),
        scope_class: scope.scope_class,
        scope_gate: scope.scope_gate,
        authority_tier,
        allowed_write_surface: scope.allowed_write_surface,
        merge_eligible,
        durable_truth: state == "durable_truth",
        accepted: report.accepted,
        restored: report.restored,
        composite_score: report.composite.score,
        composite_max: report.composite.max_score,
        evidence,
        scope_reasons: scope.scope_reasons,
        generated_at: report.completed_at,
        durability_due_at: if state == "merged" {
            Some(report.completed_at + chrono::TimeDelta::hours(1))
        } else {
            None
        },
    }
}

pub(crate) fn evolution_branch_name(scope: &EvolutionScopeAssessment, recorded_at: DateTime<Utc>) -> String {
    format!(
        "{}/{}",
        evolution_branch_prefix(scope),
        recorded_at.format("%Y%m%d%H%M%S")
    )
}

pub(crate) fn evolution_branch_prefix(scope: &EvolutionScopeAssessment) -> String {
    format!(
        "auto/evolution/{}/{}",
        branch_safe_slug(&scope.scope_class),
        branch_safe_slug(&scope.topic)
    )
}

pub(crate) fn classify_evolution_scope(report: &ExperimentReport) -> EvolutionScopeAssessment {
    let mut haystack = report.improvement.final_changes.join(" ").to_lowercase();
    if !haystack.is_empty() {
        haystack.push(' ');
    }
    haystack.push_str(&report.findings.join(" ").to_lowercase());
    if !haystack.is_empty() {
        haystack.push(' ');
    }
    haystack.push_str(&report.recommendations.join(" ").to_lowercase());
    let topic_source = report
        .improvement
        .final_changes
        .first()
        .cloned()
        .or_else(|| report.composite.scenario.clone())
        .unwrap_or_else(|| "self-evolution".to_string());
    let scenario = report.composite.scenario.as_deref().unwrap_or_default();
    let docs_score = count_matches(
        &haystack,
        &[
            "docs/", ".md", "spec", "manifest", "readme", "docs", "guide",
        ],
    );
    let runtime_policy_score = count_matches(
        &haystack,
        &[
            "threshold",
            "floor",
            "cutoff",
            "gate",
            "policy",
            "prompt",
            "weight",
            "penalty",
            "bonus",
            "clamp",
            "cap",
            "tune",
            "retune",
            "calibrate",
            "refresh cadence",
        ],
    );
    let evaluation_score = count_matches(
        &haystack,
        &[
            "evaluation",
            "eval",
            "score",
            "scoring",
            "scorer",
            "grader",
            "rubric",
            "composite",
            "dimension",
            "signal",
            "pass/fail",
            "acceptance",
            "readiness",
            "judge",
            "ranking",
            "heuristic",
            "review readiness",
            "loop",
        ],
    );
    let persistence_score = count_matches(
        &haystack,
        &[
            "schema",
            "migration",
            "persist",
            "sqlite",
            "storage",
            "database",
            "ledger format",
            "journal format",
        ],
    );
    let coordination_score = count_matches(
        &haystack,
        &[
            "coordination",
            "claim",
            "claims",
            "task",
            "tasks",
            "hive",
            "heartbeat",
            "protocol",
            "session roster",
        ],
    );
    let api_score = count_matches(&haystack, &["api", "contract", "endpoint", "wire format"]);
    let self_evolution_prior = usize::from(scenario == "self_evolution");

    let (scope_class, scope_gate, allowed_write_surface, scope_reasons) = if persistence_score > 0 {
        (
            "persistence_semantics".to_string(),
            "proposal_only".to_string(),
            vec!["proposal-only".to_string()],
            vec![format!(
                "persistence semantics signal ({persistence_score})"
            )],
        )
    } else if coordination_score > 0 {
        (
            "coordination_semantics".to_string(),
            "proposal_only".to_string(),
            vec!["proposal-only".to_string()],
            vec![format!(
                "coordination semantics signal ({coordination_score})"
            )],
        )
    } else if api_score > 0 {
        (
            "api_contract".to_string(),
            "proposal_only".to_string(),
            vec!["proposal-only".to_string()],
            vec![format!("api contract signal ({api_score})")],
        )
    } else if docs_score > 0 && runtime_policy_score == 0 && evaluation_score == 0 {
        (
            "docs_spec".to_string(),
            "auto_merge".to_string(),
            vec!["docs/**".to_string(), "*.md".to_string()],
            vec![format!("docs/spec signal ({docs_score})")],
        )
    } else if runtime_policy_score > 0 && runtime_policy_score >= evaluation_score {
        let mut reasons = vec![format!("runtime policy score={runtime_policy_score}")];
        if self_evolution_prior > 0 {
            reasons.push("self_evolution scenario prior".to_string());
        }
        (
            "runtime_policy".to_string(),
            "auto_merge".to_string(),
            vec![
                ".memd/**".to_string(),
                "policy/**".to_string(),
                "crates/memd-client/src/main.rs".to_string(),
            ],
            reasons,
        )
    } else if evaluation_score > 0 || self_evolution_prior > 0 {
        let mut reasons = vec![format!(
            "evaluation score={}",
            evaluation_score + self_evolution_prior
        )];
        if self_evolution_prior > 0 {
            reasons.push("self_evolution scenario prior".to_string());
        }
        (
            "low_risk_evaluation_code".to_string(),
            "auto_merge".to_string(),
            vec!["crates/memd-client/src/main.rs".to_string()],
            reasons,
        )
    } else if docs_score > 0 {
        (
            "docs_spec".to_string(),
            "auto_merge".to_string(),
            vec!["docs/**".to_string(), "*.md".to_string()],
            vec![format!("docs/spec signal ({docs_score})")],
        )
    } else {
        (
            "broader_implementation".to_string(),
            "proposal_only".to_string(),
            vec!["proposal-only".to_string()],
            vec!["scope unclear; keep on proposal branch".to_string()],
        )
    };

    EvolutionScopeAssessment {
        topic: canonical_slug(&topic_source),
        scope_class,
        scope_gate,
        allowed_write_surface,
        scope_reasons,
    }
}

pub(crate) fn count_matches(haystack: &str, needles: &[&str]) -> usize {
    needles
        .iter()
        .filter(|needle| haystack.contains(**needle))
        .count()
}

pub(crate) fn branch_safe_slug(value: &str) -> String {
    let mut slug = String::with_capacity(value.len());
    let mut last_dash = false;
    for ch in value.chars() {
        let normalized = if ch.is_ascii_alphanumeric() {
            ch.to_ascii_lowercase()
        } else {
            '-'
        };
        if normalized == '-' {
            if !last_dash {
                slug.push('-');
            }
            last_dash = true;
        } else {
            slug.push(normalized);
            last_dash = false;
        }
    }
    slug.trim_matches('-').to_string()
}

pub(crate) fn snapshot_bundle_for_reversion(output: &Path) -> anyhow::Result<PathBuf> {
    let snapshot_root =
        std::env::temp_dir().join(format!("memd-experiment-backup-{}", uuid::Uuid::new_v4()));
    copy_dir_contents(output, &snapshot_root)?;
    Ok(snapshot_root)
}
