use super::*;

pub(crate) fn research_loops_doc_loop_count(project_root: &Path) -> Option<usize> {
    let path = project_root.join("docs/strategy/research-loops.md");
    let raw = fs::read_to_string(&path).ok()?;
    let count = raw
        .lines()
        .map(str::trim_start)
        .filter(|line| {
            let digit_count = line.chars().take_while(|ch| ch.is_ascii_digit()).count();
            digit_count > 0
                && line
                    .get(digit_count..)
                    .is_some_and(|rest| rest.starts_with(". "))
        })
        .count();
    Some(count)
}

pub(crate) fn evaluate_gap_changes(
    current: &GapReport,
    baseline: Option<&GapReport>,
) -> Vec<String> {
    let mut changes = Vec::new();
    let current_top = current.candidate_count;
    if let Some(baseline) = baseline {
        if baseline.candidate_count != current_top {
            changes.push(format!(
                "candidate_count {} -> {}",
                baseline.candidate_count, current_top
            ));
        }
        if baseline.eval_score != current.eval_score {
            changes.push(format!(
                "eval_score {:?} -> {:?}",
                baseline.eval_score, current.eval_score
            ));
        }
    }
    if current.eval_status.is_some() {
        changes.push(format!(
            "eval_status={}",
            current.eval_status.as_deref().unwrap_or("none")
        ));
    }
    changes
}

pub(crate) fn eval_score_delta(previous: u8, current: Option<&BundleEvalResponse>) -> Option<i32> {
    current.map(|value| i32::from(value.score) - i32::from(previous))
}

pub(crate) fn prioritize_gap_candidates(
    mut candidates: Vec<GapCandidate>,
    limit: usize,
) -> Vec<GapCandidate> {
    candidates.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left.area.cmp(&right.area))
    });
    candidates.into_iter().take(limit).collect()
}

pub(crate) fn coordination_exists(output: &Path) -> bool {
    output
        .join("state")
        .join("coordination-snapshot.json")
        .exists()
}

pub(crate) fn gap_artifact_paths(output: &Path, name: &str) -> PathBuf {
    gap_reports_dir(output).join(name)
}

pub(crate) fn write_gap_artifacts(output: &Path, response: &GapReport) -> anyhow::Result<()> {
    let gap_dir = gap_reports_dir(output);
    fs::create_dir_all(&gap_dir).with_context(|| format!("create {}", gap_dir.display()))?;

    let baseline_json = gap_artifact_paths(output, "latest.json");
    let baseline_md = gap_artifact_paths(output, "latest.md");
    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let timestamp_json = gap_artifact_paths(output, &format!("{timestamp}.json"));
    let timestamp_md = gap_artifact_paths(output, &format!("{timestamp}.md"));
    let markdown = render_gap_markdown(response);
    let json = serde_json::to_string_pretty(response)? + "\n";

    fs::write(&baseline_json, &json)
        .with_context(|| format!("write {}", baseline_json.display()))?;
    fs::write(&baseline_md, &markdown)
        .with_context(|| format!("write {}", baseline_md.display()))?;
    fs::write(&timestamp_json, &json)
        .with_context(|| format!("write {}", timestamp_json.display()))?;
    fs::write(&timestamp_md, &markdown)
        .with_context(|| format!("write {}", timestamp_md.display()))?;

    write_gap_loop_record(output, response)?;

    Ok(())
}

pub(crate) fn write_gap_loop_record(output: &Path, response: &GapReport) -> anyhow::Result<()> {
    let slug = format!("gap-{}", response.generated_at.format("%Y%m%dT%H%M%SZ"));
    let percent_improvement = response
        .eval_score_delta
        .map(|delta| (delta as f64).clamp(-100.0, 100.0));
    let token_savings = response.previous_candidate_count.map(|previous| {
        if previous > response.candidate_count {
            ((previous - response.candidate_count) as f64) * 25.0
        } else {
            0.0
        }
    });
    let record = LoopRecord {
        slug: Some(slug.clone()),
        name: Some("gap research loop".to_string()),
        iteration: Some(response.candidate_count as u32),
        percent_improvement,
        token_savings,
        status: Some("gap".to_string()),
        summary: Some(format!(
            "{} candidates ({} high priority) with eval score {}",
            response.candidate_count,
            response.high_priority_count,
            response
                .eval_score
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        )),
        artifacts: Some(vec![
            gap_artifact_paths(output, "latest.json")
                .display()
                .to_string(),
            gap_artifact_paths(output, "latest.md")
                .display()
                .to_string(),
        ]),
        created_at: Some(response.generated_at),
        metadata: serde_json::json!({
            "commits_checked": response.commits_checked,
            "changes": response.changes,
            "evidence": response.evidence,
        }),
    };
    persist_loop_record(output, &record)?;
    Ok(())
}

pub(crate) fn persist_loop_record(output: &Path, record: &LoopRecord) -> anyhow::Result<()> {
    let dir = loops_directory(output);
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;

    let slug = canonical_slug(record.slug.as_deref().unwrap_or("loop"));
    let path = dir.join(format!("loop-{}.json", slug));
    let json = serde_json::to_string_pretty(record)? + "\n";
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    update_loop_summary(&dir.join("loops.summary.json"), record)?;
    Ok(())
}

pub(crate) fn update_loop_summary(path: &Path, record: &LoopRecord) -> anyhow::Result<()> {
    let mut summary = read_loop_summary(path)?;
    summary.entries.push(LoopSummaryEntry {
        slug: canonical_slug(record.slug.as_deref().unwrap_or("loop")),
        percent_improvement: record.percent_improvement,
        token_savings: record.token_savings,
        status: record.status.clone(),
        recorded_at: record.created_at.unwrap_or(Utc::now()),
    });
    let json = serde_json::to_string_pretty(&summary)? + "\n";
    fs::write(path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn read_loop_summary(path: &Path) -> anyhow::Result<LoopSummary> {
    if !path.exists() {
        return Ok(LoopSummary::default());
    }

    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let summary = serde_json::from_str::<LoopSummary>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(summary)
}

pub(crate) fn read_latest_gap_report(output: &Path) -> anyhow::Result<Option<GapReport>> {
    let path = gap_artifact_paths(output, "latest.json");
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let report = serde_json::from_str::<GapReport>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(report))
}

pub(crate) fn render_gap_markdown(response: &GapReport) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd gap report\n\n");
    markdown.push_str(&format!(
        "- bundle: {}\n- project: {}\n- namespace: {}\n- agent: {}\n- session: {}\n- workspace: {}\n- visibility: {}\n- eval_status: {}\n- eval_score: {}\n- eval_score_delta: {}\n- candidate_count: {}\n- high_priority_count: {}\n- previous_candidate_count: {}\n- commits_checked: {}\n- generated_at: {}\n",
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.eval_status.clone().unwrap_or_else(|| "none".to_string()),
        response
            .eval_score
            .map(|value: u8| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        response.eval_score_delta
            .map(|value: i32| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        response.candidate_count,
        response.high_priority_count,
        response.previous_candidate_count.unwrap_or(0),
        response.commits_checked,
        response.generated_at,
    ));

    markdown.push_str("\n## Evidence\n\n");
    if response.evidence.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.evidence {
            markdown.push_str(&format!("- {}\n", item));
        }
    }

    markdown.push_str("\n## Candidates\n\n");
    if response.candidates.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for candidate in &response.candidates {
            markdown.push_str(&format!(
                "- [{}] {} {} (priority={})\n",
                candidate.severity, candidate.area, candidate.signal, candidate.priority
            ));
            markdown.push_str(&format!("  - action: {}\n", candidate.recommendation));
            for entry in &candidate.evidence {
                markdown.push_str(&format!("  - evidence: {}\n", entry));
            }
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

    markdown.push_str("\n## Priorities\n\n");
    if response.top_priorities.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.top_priorities {
            markdown.push_str(&format!("- {}\n", item));
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

    markdown
}
