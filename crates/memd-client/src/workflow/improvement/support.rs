use super::*;

pub(crate) fn improvement_reports_dir(output: &Path) -> PathBuf {
    output.join("improvements")
}

pub(crate) fn scenario_reports_dir(output: &Path) -> PathBuf {
    output.join("scenarios")
}

pub(crate) fn project_root_from_bundle(output: &Path) -> &Path {
    output.parent().unwrap_or_else(|| Path::new("."))
}

pub(crate) fn read_text_file(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(crate) fn read_recent_commits(root: &Path, limit: usize) -> Vec<String> {
    let limit = limit.clamp(1, 64);
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("log")
        .arg(format!("-n{limit}"))
        .arg("--oneline")
        .output();

    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    raw.lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .take(limit)
        .collect()
}

pub(crate) fn gap_to_improvement_snapshot(response: &GapReport) -> ImprovementGapSnapshot {
    ImprovementGapSnapshot {
        candidate_count: response.candidate_count,
        high_priority_count: response.high_priority_count,
        eval_status: response.eval_status.clone(),
        eval_score: response.eval_score,
        eval_score_delta: response.eval_score_delta,
        top_priorities: response.top_priorities.clone(),
        generated_at: response.generated_at,
    }
}

pub(crate) fn improvement_progress(previous: &GapReport, current: &GapReport) -> bool {
    if current.candidate_count < previous.candidate_count {
        return true;
    }
    if current.high_priority_count < previous.high_priority_count {
        return true;
    }
    if let (Some(previous_score), Some(current_score)) = (previous.eval_score, current.eval_score) {
        if current_score > previous_score {
            return true;
        }
    } else if current.eval_score.is_some() && previous.eval_score.is_none() {
        return true;
    }
    previous.top_priorities != current.top_priorities
}
