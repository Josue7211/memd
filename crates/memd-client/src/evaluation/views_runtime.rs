use super::*;

pub(crate) fn render_bundle_eval_markdown(response: &BundleEvalResponse) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd bundle evaluation\n\n");
    markdown.push_str(&format!(
        "- bundle: {}\n- status: {}\n- score: {}\n- baseline_score: {}\n- score_delta: {}\n- agent: {}\n- workspace: {}\n- visibility: {}\n",
        response.bundle_root,
        response.status,
        response.score,
        response
            .baseline_score
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        response
            .score_delta
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        response.agent.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("none"),
    ));
    markdown.push_str(&format!(
        "- working_records: {}\n- context_records: {}\n- rehydration_items: {}\n- inbox_items: {}\n- workspace_lanes: {}\n- semantic_hits: {}\n",
        response.working_records,
        response.context_records,
        response.rehydration_items,
        response.inbox_items,
        response.workspace_lanes,
        response.semantic_hits,
    ));

    markdown.push_str("\n## Findings\n\n");
    if response.findings.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for finding in &response.findings {
            markdown.push_str(&format!("- {}\n", finding));
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

    markdown.push_str("\n## Recommendations\n\n");
    if response.recommendations.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for recommendation in &response.recommendations {
            markdown.push_str(&format!("- {}\n", recommendation));
        }
    }

    markdown
}
