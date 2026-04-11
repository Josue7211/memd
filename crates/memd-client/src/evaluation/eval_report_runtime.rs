use super::*;

pub(crate) async fn eval_bundle_memory(
    args: &EvalArgs,
    base_url: &str,
) -> anyhow::Result<BundleEvalResponse> {
    let baseline = read_latest_bundle_eval(&args.output)?;
    let snapshot = read_bundle_resume(
        &ResumeArgs {
            output: args.output.clone(),
            project: None,
            namespace: None,
            agent: None,
            workspace: None,
            visibility: None,
            route: None,
            intent: None,
            limit: args.limit.or(Some(8)),
            rehydration_limit: args.rehydration_limit.or(Some(4)),
            semantic: true,
            prompt: false,
            summary: false,
        },
        base_url,
    )
    .await?;

    let runtime = read_bundle_runtime_config(&args.output)?;
    let mut score = 100i32;
    let mut findings = Vec::new();

    if snapshot.working.records.is_empty() {
        score -= 30;
        findings.push("no working memory records returned from bundle resume".to_string());
    }
    if snapshot.context.records.is_empty() {
        score -= 15;
        findings.push("no compact context records returned from bundle resume".to_string());
    }
    if snapshot.working.rehydration_queue.is_empty() {
        score -= 10;
        findings.push("rehydration queue is empty; deeper evidence recovery is weak".to_string());
    }
    if snapshot.workspace.is_some() && snapshot.workspaces.workspaces.is_empty() {
        score -= 15;
        findings.push("active workspace is set but no workspace lanes were returned".to_string());
    }
    if snapshot
        .semantic
        .as_ref()
        .is_some_and(|semantic| semantic.items.is_empty())
    {
        score -= 5;
        findings.push("semantic recall is configured but returned no items".to_string());
    }
    if snapshot.inbox.items.len() >= 6 {
        score -= 10;
        findings.push("inbox pressure is high; resume lane may need maintenance".to_string());
    }

    let score = score.clamp(0, 100) as u8;
    let status = if score >= 85 {
        "strong"
    } else if score >= 65 {
        "usable"
    } else {
        "weak"
    };

    let baseline_score = baseline.as_ref().map(|value| value.score);
    let score_delta = baseline_score.map(|baseline| score as i32 - baseline as i32);
    let changes = baseline
        .as_ref()
        .map(|baseline| describe_eval_changes(baseline, score, &snapshot))
        .unwrap_or_default();
    let recommendations = build_eval_recommendations(&snapshot, score);

    Ok(BundleEvalResponse {
        bundle_root: args.output.display().to_string(),
        project: snapshot.project.clone(),
        namespace: snapshot.namespace.clone(),
        agent: snapshot
            .agent
            .clone()
            .or_else(|| runtime.as_ref().and_then(|config| config.agent.clone())),
        workspace: snapshot.workspace.clone(),
        visibility: snapshot.visibility.clone(),
        status: status.to_string(),
        score,
        working_records: snapshot.working.records.len(),
        context_records: snapshot.context.records.len(),
        rehydration_items: snapshot.working.rehydration_queue.len(),
        inbox_items: snapshot.inbox.items.len(),
        workspace_lanes: snapshot.workspaces.workspaces.len(),
        semantic_hits: snapshot
            .semantic
            .as_ref()
            .map(|semantic| semantic.items.len())
            .unwrap_or(0),
        findings,
        baseline_score,
        score_delta,
        changes,
        recommendations,
    })
}

pub(crate) fn read_latest_bundle_eval(output: &Path) -> anyhow::Result<Option<BundleEvalResponse>> {
    let path = output.join("evals").join("latest.json");
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let eval = serde_json::from_str::<BundleEvalResponse>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(eval))
}

pub(crate) fn read_latest_scenario_report(output: &Path) -> anyhow::Result<Option<ScenarioReport>> {
    let path = output.join("scenarios").join("latest.json");
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let report = serde_json::from_str::<ScenarioReport>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(report))
}

pub(crate) fn describe_eval_changes(
    baseline: &BundleEvalResponse,
    score: u8,
    snapshot: &ResumeSnapshot,
) -> Vec<String> {
    let mut changes = Vec::new();

    if baseline.score != score {
        changes.push(format!("score {} -> {}", baseline.score, score));
    }

    let working_records = snapshot.working.records.len();
    if baseline.working_records != working_records {
        changes.push(format!(
            "working {} -> {}",
            baseline.working_records, working_records
        ));
    }

    let context_records = snapshot.context.records.len();
    if baseline.context_records != context_records {
        changes.push(format!(
            "context {} -> {}",
            baseline.context_records, context_records
        ));
    }

    let rehydration_items = snapshot.working.rehydration_queue.len();
    if baseline.rehydration_items != rehydration_items {
        changes.push(format!(
            "rehydration {} -> {}",
            baseline.rehydration_items, rehydration_items
        ));
    }

    let inbox_items = snapshot.inbox.items.len();
    if baseline.inbox_items != inbox_items {
        changes.push(format!("inbox {} -> {}", baseline.inbox_items, inbox_items));
    }

    let workspace_lanes = snapshot.workspaces.workspaces.len();
    if baseline.workspace_lanes != workspace_lanes {
        changes.push(format!(
            "lanes {} -> {}",
            baseline.workspace_lanes, workspace_lanes
        ));
    }

    let semantic_hits = snapshot
        .semantic
        .as_ref()
        .map(|semantic| semantic.items.len())
        .unwrap_or(0);
    if baseline.semantic_hits != semantic_hits {
        changes.push(format!(
            "semantic {} -> {}",
            baseline.semantic_hits, semantic_hits
        ));
    }

    changes
}

pub(crate) fn eval_failure_reason(
    response: &BundleEvalResponse,
    fail_below: Option<u8>,
    fail_on_regression: bool,
) -> Option<String> {
    if let Some(threshold) = fail_below {
        if response.score < threshold {
            return Some(format!(
                "bundle evaluation score {} fell below required threshold {}",
                response.score, threshold
            ));
        }
    }

    if fail_on_regression && response.score_delta.is_some_and(|delta| delta < 0) {
        let baseline = response
            .baseline_score
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let delta = response.score_delta.unwrap_or_default();
        return Some(format!(
            "bundle evaluation regressed from baseline {} to {} (delta {})",
            baseline, response.score, delta
        ));
    }

    None
}

pub(crate) fn build_eval_recommendations(snapshot: &ResumeSnapshot, score: u8) -> Vec<String> {
    let mut recommendations = Vec::new();

    if snapshot.working.records.is_empty() {
        recommendations.push(
            "capture durable memory with `memd remember --output .memd ...` before relying on resume"
                .to_string(),
        );
    }
    if snapshot.context.records.is_empty() {
        recommendations.push(
            "review bundle route/intent defaults and verify compact context retrieval for the active lane"
                .to_string(),
        );
    }
    if snapshot.working.rehydration_queue.is_empty() {
        recommendations.push(
            "promote richer evidence or inspect key items with `memd explain --follow` so resume can rehydrate deeper context"
                .to_string(),
        );
    }
    if snapshot.workspace.is_some() && snapshot.workspaces.workspaces.is_empty() {
        recommendations.push(
            "repair workspace or visibility lanes so shared memory is visible to the active bundle"
                .to_string(),
        );
    }
    if snapshot.inbox.items.len() >= 6 {
        recommendations.push(
            "drain inbox pressure with repair or promotion passes before the next handoff or resume"
                .to_string(),
        );
    }
    if snapshot
        .semantic
        .as_ref()
        .is_some_and(|semantic| semantic.items.is_empty())
    {
        recommendations.push(
            "check the LightRAG index or sync path before depending on semantic fallback"
                .to_string(),
        );
    }
    if score < 85 {
        recommendations.push(
            "write a fresh baseline with `memd eval --output .memd --write --summary` after corrective changes"
                .to_string(),
        );
    }

    recommendations
}
