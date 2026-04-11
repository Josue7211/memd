use super::*;

pub(crate) struct EvolutionScopeAssessment {
    pub(crate) topic: String,
    pub(crate) scope_class: String,
    pub(crate) scope_gate: String,
    pub(crate) allowed_write_surface: Vec<String>,
    pub(crate) scope_reasons: Vec<String>,
}

pub(crate) fn build_evolution_proposal_report(
    report: &ExperimentReport,
) -> EvolutionProposalReport {
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

pub(crate) fn evolution_branch_name(
    scope: &EvolutionScopeAssessment,
    recorded_at: DateTime<Utc>,
) -> String {
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
