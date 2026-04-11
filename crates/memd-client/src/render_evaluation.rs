use super::compact_inline;

pub(crate) fn render_eval_summary(response: &crate::BundleEvalResponse) -> String {
    let mut output = format!(
        "eval status={} score={} baseline={} delta={} agent={} workspace={} working={} context={} rehydration={} inbox={} lanes={} semantic={}",
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
        response.working_records,
        response.context_records,
        response.rehydration_items,
        response.inbox_items,
        response.workspace_lanes,
        response.semantic_hits
    );

    if !response.findings.is_empty() {
        let findings = response
            .findings
            .iter()
            .take(3)
            .map(|value| compact_inline(value, 56))
            .collect::<Vec<_>>();
        output.push_str(&format!(" findings={}", findings.join(" | ")));
    }

    if !response.changes.is_empty() {
        let changes = response
            .changes
            .iter()
            .take(3)
            .map(|value| compact_inline(value, 40))
            .collect::<Vec<_>>();
        output.push_str(&format!(" changes={}", changes.join(" | ")));
    }

    if !response.recommendations.is_empty() {
        let recommendations = response
            .recommendations
            .iter()
            .take(2)
            .map(|value| compact_inline(value, 44))
            .collect::<Vec<_>>();
        output.push_str(&format!(" next={}", recommendations.join(" | ")));
    }

    output
}

pub(crate) fn render_gap_summary(response: &crate::GapReport) -> String {
    let mut output = format!(
        "gap bundle={} project={} namespace={} agent={} session={} workspace={} visibility={} candidates={} high_priority={} eval_score={}",
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.candidate_count,
        response.high_priority_count,
        response
            .eval_score
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string())
    );

    if let Some(status) = response.eval_status.as_deref() {
        output.push_str(&format!(" eval_status={status}"));
    } else {
        output.push_str(" eval_status=none");
    }

    if response.eval_score_delta.is_some() || response.previous_candidate_count.is_some() {
        output.push_str(&format!(
            " eval_score_delta={} prev_candidates={}",
            response
                .eval_score_delta
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
            response
                .previous_candidate_count
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
        ));
    }

    if let Some(changes) = response.changes.first() {
        output.push_str(&format!(" next=top:{}", changes));
    }

    output.push_str(&format!(
        " commit_window={} recent={}",
        response.limit, response.commits_checked
    ));

    output
}

pub(crate) fn render_scenario_summary(response: &crate::ScenarioReport) -> String {
    let mut output = format!(
        "scenario name={} bundle={} project={} namespace={} agent={} session={} workspace={} visibility={} score={}/{} passed={} failed={}",
        response.scenario,
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.score,
        response.max_score,
        response.passed_checks,
        response.failed_checks
    );

    if let Some(top_check) = response.checks.first() {
        output.push_str(&format!(
            " first_check={}:{}",
            top_check.name, top_check.status
        ));
    }

    if let Some(next) = response.next_actions.first() {
        output.push_str(&format!(" next_action={next}"));
    }

    output
}

pub(crate) fn render_improvement_summary(response: &crate::ImprovementReport) -> String {
    let mut output = format!(
        "improve bundle={} project={} namespace={} agent={} session={} workspace={} visibility={} apply={} iterations={} converged={} initial_candidates={} final_candidates={} final_score={}",
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.apply,
        response.iterations.len(),
        response.converged,
        response
            .initial_gap
            .as_ref()
            .map(|value| value.candidate_count)
            .unwrap_or(0),
        response
            .final_gap
            .as_ref()
            .map(|value| value.candidate_count)
            .unwrap_or(0),
        response
            .final_gap
            .as_ref()
            .and_then(|value| value.eval_score)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
    );
    output.push_str(&format!(
        " max_iterations={} started_at={}",
        response.max_iterations,
        response.started_at.format("%Y-%m-%d %H:%M:%S UTC")
    ));
    if response.final_gap.is_some()
        && let Some(changes) = response.final_changes.first()
    {
        output.push_str(&format!(" next=top:{}", changes));
    }
    if !response.iterations.is_empty() {
        let iteration_overview = response
            .iterations
            .iter()
            .map(|iteration| {
                format!(
                    "iter{} pre={}->{}, actions={}",
                    iteration.iteration,
                    iteration.pre_gap.candidate_count,
                    iteration.post_gap.as_ref().map_or_else(
                        || "none".to_string(),
                        |summary| summary.candidate_count.to_string(),
                    ),
                    iteration.planned_actions.len()
                )
            })
            .collect::<Vec<_>>()
            .join(" | ");
        output.push_str(&format!(" iterations=[{}]", iteration_overview));
    }
    output
}

pub(crate) fn render_scenario_markdown(response: &crate::ScenarioReport) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# memd scenario report: {}\n\n",
        response.scenario
    ));
    markdown.push_str(&format!(
        "- bundle: {}\n- project: {}\n- namespace: {}\n- agent: {}\n- session: {}\n- workspace: {}\n- visibility: {}\n- score: {}/{}\n- passed_checks: {}\n- failed_checks: {}\n- generated_at: {}\n- completed_at: {}\n",
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.score,
        response.max_score,
        response.passed_checks,
        response.failed_checks,
        response.generated_at,
        response.completed_at
    ));

    markdown.push_str("\n## Checks\n\n");
    if response.checks.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for check in &response.checks {
            markdown.push_str(&format!(
                "- [{}] {} ({}pts): {}\n",
                check.status, check.name, check.points, check.details
            ));
        }
    }

    markdown.push_str("\n## Evidence\n\n");
    if response.evidence.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.evidence {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown.push_str("\n## Findings\n\n");
    if response.findings.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.findings {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown.push_str("\n## Next Actions\n\n");
    if response.next_actions.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.next_actions {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown
}

pub(crate) fn render_composite_summary(response: &crate::CompositeReport) -> String {
    let mut output = format!(
        "composite bundle={} project={} namespace={} agent={} session={} workspace={} visibility={} score={}/{}",
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.score,
        response.max_score
    );
    if let Some(scenario) = response.scenario.as_deref() {
        output.push_str(&format!(" scenario={scenario}"));
    }
    if let Some(gate) = response.gates.first() {
        output.push_str(&format!(" first_gate={}:{}", gate.name, gate.status));
    }
    if let Some(dim) = response.dimensions.first() {
        output.push_str(&format!(" first_dimension={}:{}", dim.name, dim.score));
    }
    output
}

pub(crate) fn render_composite_markdown(response: &crate::CompositeReport) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd composite report\n\n");
    markdown.push_str(&format!(
        "- bundle: {}\n- project: {}\n- namespace: {}\n- agent: {}\n- session: {}\n- workspace: {}\n- visibility: {}\n- scenario: {}\n- score: {}/{}\n- generated_at: {}\n- completed_at: {}\n",
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.scenario.as_deref().unwrap_or("none"),
        response.score,
        response.max_score,
        response.generated_at,
        response.completed_at
    ));

    markdown.push_str("\n## Dimensions\n\n");
    if response.dimensions.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for dimension in &response.dimensions {
            markdown.push_str(&format!(
                "- [{}] {} (weight {}): {}\n",
                dimension.score, dimension.name, dimension.weight, dimension.details
            ));
        }
    }

    markdown.push_str("\n## Gates\n\n");
    if response.gates.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for gate in &response.gates {
            markdown.push_str(&format!(
                "- [{}] {}: {}\n",
                gate.status, gate.name, gate.details
            ));
        }
    }

    markdown.push_str("\n## Evidence\n\n");
    if response.evidence.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.evidence {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown.push_str("\n## Findings\n\n");
    if response.findings.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.findings {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown.push_str("\n## Recommendations\n\n");
    if response.recommendations.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.recommendations {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown
}

pub(crate) fn render_feature_benchmark_summary(response: &crate::FeatureBenchmarkReport) -> String {
    let pass = response
        .areas
        .iter()
        .filter(|area| area.status == "pass")
        .count();
    let warn = response
        .areas
        .iter()
        .filter(|area| area.status == "warn")
        .count();
    let fail = response
        .areas
        .iter()
        .filter(|area| area.status == "fail")
        .count();
    let lead = response
        .areas
        .first()
        .map(|area| format!(" first_area={}:{}", area.slug, area.score))
        .unwrap_or_default();
    format!(
        "benchmark bundle={} project={} namespace={} agent={} session={} score={}/{} areas={} pass={} warn={} fail={} commands={} skills={} packs={} pages={} events={}{}",
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.score,
        response.max_score,
        response.areas.len(),
        pass,
        warn,
        fail,
        response.command_count,
        response.skill_count,
        response.pack_count,
        response.memory_pages,
        response.event_count,
        lead
    )
}

pub(crate) fn render_feature_benchmark_markdown(
    response: &crate::FeatureBenchmarkReport,
) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd feature benchmark\n\n");
    markdown.push_str(&format!(
        "- bundle: {}\n- project: {}\n- namespace: {}\n- agent: {}\n- session: {}\n- workspace: {}\n- visibility: {}\n- score: {}/{}\n- commands: {}\n- skills: {}\n- packs: {}\n- memory_pages: {}\n- event_count: {}\n- generated_at: {}\n- completed_at: {}\n",
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.score,
        response.max_score,
        response.command_count,
        response.skill_count,
        response.pack_count,
        response.memory_pages,
        response.event_count,
        response.generated_at,
        response.completed_at
    ));

    markdown.push_str("\n## Areas\n\n");
    for area in &response.areas {
        markdown.push_str(&format!(
            "### {} [{}]\n\n- score: {}/{}\n- command coverage: {}/{}\n",
            area.name,
            area.status,
            area.score,
            area.max_score,
            area.implemented_commands,
            area.expected_commands
        ));
        markdown.push_str("\n#### Evidence\n\n");
        if area.evidence.is_empty() {
            markdown.push_str("- none\n");
        } else {
            for item in &area.evidence {
                markdown.push_str(&format!("- {item}\n"));
            }
        }
        markdown.push_str("\n#### Recommendations\n\n");
        if area.recommendations.is_empty() {
            markdown.push_str("- none\n");
        } else {
            for item in &area.recommendations {
                markdown.push_str(&format!("- {item}\n"));
            }
        }
        markdown.push('\n');
    }

    markdown.push_str("## Evidence\n\n");
    if response.evidence.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.evidence {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown.push_str("\n## Recommendations\n\n");
    if response.recommendations.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.recommendations {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown
}

pub(crate) fn render_experiment_summary(response: &crate::ExperimentReport) -> String {
    let mut output = format!(
        "experiment bundle={} project={} namespace={} agent={} session={} workspace={} visibility={} accepted={} restored={} score={}/{} iterations={}",
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.accepted,
        response.restored,
        response.composite.score,
        response.composite.max_score,
        response.improvement.iterations.len()
    );
    output.push_str(&format!(
        " max_iterations={} accept_below={} apply={} consolidate={}",
        response.max_iterations, response.accept_below, response.apply, response.consolidate
    ));
    if let Some(entry) = response.trail.first() {
        output.push_str(&format!(" first_trail={entry}"));
    }
    if let Some(evolution) = &response.evolution {
        output.push_str(&format!(
            " evolution={} scope={}/{} authority={} merge={} durability={}",
            evolution.proposal_state,
            evolution.scope_class,
            evolution.scope_gate,
            evolution.authority_tier,
            evolution.merge_status,
            evolution.durability_status
        ));
        if evolution.branch != "none" {
            output.push_str(&format!(
                " evo_branch={}",
                compact_inline(&evolution.branch, 64)
            ));
        }
    }
    output
}

pub(crate) fn render_experiment_markdown(response: &crate::ExperimentReport) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd experiment report\n\n");
    markdown.push_str(&format!(
        "- bundle: {}\n- project: {}\n- namespace: {}\n- agent: {}\n- session: {}\n- workspace: {}\n- visibility: {}\n- max_iterations: {}\n- accept_below: {}\n- apply: {}\n- consolidate: {}\n- accepted: {}\n- restored: {}\n- started_at: {}\n- completed_at: {}\n",
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.max_iterations,
        response.accept_below,
        response.apply,
        response.consolidate,
        response.accepted,
        response.restored,
        response.started_at,
        response.completed_at
    ));

    if let Some(evolution) = &response.evolution {
        markdown.push_str("\n## Evolution\n\n");
        markdown.push_str(&format!(
            "- proposal_state: {}\n- scope: {}/{}\n- authority_tier: {}\n- merge_status: {}\n- durability_status: {}\n- branch: {}\n- durable_truth: {}\n",
            evolution.proposal_state,
            evolution.scope_class,
            evolution.scope_gate,
            evolution.authority_tier,
            evolution.merge_status,
            evolution.durability_status,
            evolution.branch,
            evolution.durable_truth
        ));
    }

    markdown.push_str("\n## Trail\n\n");
    if response.trail.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.trail {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown.push_str("\n## Learnings\n\n");
    if response.learnings.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.learnings {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown.push_str("\n## Findings\n\n");
    if response.findings.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.findings {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown.push_str("\n## Recommendations\n\n");
    if response.recommendations.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.recommendations {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown.push_str("\n## Evidence\n\n");
    if response.evidence.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.evidence {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown.push_str("\n## Improvement\n\n");
    markdown.push_str(&format!(
        "- iterations: {}\n- converged: {}\n- final_candidates: {}\n- final_score: {}\n",
        response.improvement.iterations.len(),
        response.improvement.converged,
        response
            .improvement
            .final_gap
            .as_ref()
            .map(|value| value.candidate_count)
            .unwrap_or(0),
        response
            .improvement
            .final_gap
            .as_ref()
            .and_then(|value| value.eval_score)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string())
    ));

    markdown.push_str("\n## Composite\n\n");
    markdown.push_str(&format!(
        "- score: {}/{}\n- scenario: {}\n",
        response.composite.score,
        response.composite.max_score,
        response.composite.scenario.as_deref().unwrap_or("none"),
    ));
    for gate in &response.composite.gates {
        markdown.push_str(&format!(
            "- gate [{}] {}: {}\n",
            gate.status, gate.name, gate.details
        ));
    }

    markdown
}

pub(crate) fn render_improvement_markdown(response: &crate::ImprovementReport) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd improvement report\n\n");
    markdown.push_str(&format!(
        "- bundle: {}\n- project: {}\n- namespace: {}\n- agent: {}\n- session: {}\n- workspace: {}\n- visibility: {}\n- apply: {}\n- max_iterations: {}\n- converged: {}\n- started_at: {}\n- completed_at: {}\n- initial_candidates: {}\n- final_candidates: {}\n- final_score: {}\n",
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.apply,
        response.max_iterations,
        response.converged,
        response.started_at,
        response.completed_at,
        response
            .initial_gap
            .as_ref()
            .map(|value| value.candidate_count)
            .unwrap_or(0),
        response
            .final_gap
            .as_ref()
            .map(|value| value.candidate_count)
            .unwrap_or(0),
        response
            .final_gap
            .as_ref()
            .and_then(|value| value.eval_score)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string())
    ));

    if !response.final_changes.is_empty() {
        markdown.push_str("\n## Final Changes\n\n");
        for change in &response.final_changes {
            markdown.push_str(&format!("- {change}\n"));
        }
    }

    markdown.push_str("\n## Iterations\n\n");
    if response.iterations.is_empty() {
        markdown.push_str("- no iterations executed\n");
        return markdown;
    }

    for iteration in &response.iterations {
        markdown.push_str(&format!("### Iteration {}\n\n", iteration.iteration));
        markdown.push_str(&format!(
            "- pre_gap: candidates={} high_priority={} eval_score={}\n",
            iteration.pre_gap.candidate_count,
            iteration.pre_gap.high_priority_count,
            iteration
                .pre_gap
                .eval_score
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ));

        if let Some(post_gap) = &iteration.post_gap {
            markdown.push_str(&format!(
                "- post_gap: candidates={} high_priority={} eval_score={}\n",
                post_gap.candidate_count,
                post_gap.high_priority_count,
                post_gap
                    .eval_score
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string())
            ));
        } else {
            markdown.push_str("- post_gap: none\n");
        }

        markdown.push_str("- planned actions:\n");
        if iteration.planned_actions.is_empty() {
            markdown.push_str("  - none\n");
        } else {
            for action in &iteration.planned_actions {
                let extras = format!(
                    "{}{}{}{}",
                    action
                        .task_id
                        .as_ref()
                        .map(|value| format!(" task={value}"))
                        .unwrap_or_default(),
                    action
                        .scope
                        .as_ref()
                        .map(|value| format!(" scope={value}"))
                        .unwrap_or_default(),
                    action
                        .target_session
                        .as_ref()
                        .map(|value| format!(" target_session={value}"))
                        .unwrap_or_default(),
                    action
                        .message_id
                        .as_ref()
                        .map(|value| format!(" message={value}"))
                        .unwrap_or_default(),
                );
                markdown.push_str(&format!(
                    "  - {} [{}] {}{}\n",
                    action.action, action.priority, action.reason, extras,
                ));
            }
        }

        markdown.push_str("- execution:\n");
        if iteration.executed_actions.is_empty() {
            markdown.push_str("  - none\n");
        } else {
            for result in &iteration.executed_actions {
                markdown.push_str(&format!(
                    "  - {} {}: {}\n",
                    result.status, result.action, result.detail
                ));
            }
        }
        markdown.push('\n');
    }

    markdown
}
