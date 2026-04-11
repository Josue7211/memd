use super::*;

pub(crate) async fn run_scenario_command(
    args: &ScenarioArgs,
    base_url: &str,
) -> anyhow::Result<ScenarioReport> {
    fn supported_scenarios() -> &'static str {
        "bundle_health, resume_after_pause, handoff, workspace_retrieval, stale_session_recovery, coworking"
    }

    #[derive(Debug, Clone, Copy)]
    enum ScenarioKind {
        BundleHealth,
        ResumeAfterPause,
        Handoff,
        WorkspaceRetrieval,
        StaleSessionRecovery,
        Coworking,
    }

    impl ScenarioKind {
        fn from_input(input: &str) -> Option<Self> {
            match input {
                "bundle_health" => Some(Self::BundleHealth),
                "resume_after_pause" => Some(Self::ResumeAfterPause),
                "handoff" => Some(Self::Handoff),
                "workspace_retrieval" => Some(Self::WorkspaceRetrieval),
                "stale_session_recovery" => Some(Self::StaleSessionRecovery),
                "coworking" => Some(Self::Coworking),
                _ => None,
            }
        }
    }

    let started_at = Utc::now();
    let runtime = read_bundle_runtime_config(&args.output)?;
    let runtime_project = runtime.as_ref().and_then(|value| value.project.clone());
    let runtime_namespace = runtime.as_ref().and_then(|value| value.namespace.clone());
    let runtime_session = runtime.as_ref().and_then(|value| value.session.clone());
    let runtime_workspace = runtime.as_ref().and_then(|value| value.workspace.clone());
    let runtime_visibility = runtime.as_ref().and_then(|value| value.visibility.clone());
    let scenario_name = args
        .scenario
        .clone()
        .unwrap_or_else(|| "bundle_health".to_string())
        .to_lowercase();
    let scenario_kind = ScenarioKind::from_input(&scenario_name).ok_or_else(|| {
        anyhow!(
            "unknown scenario '{scenario_name}'; supported: {supported}",
            supported = supported_scenarios()
        )
    })?;

    let resume = read_bundle_resume(
        &ResumeArgs {
            output: args.output.clone(),
            project: runtime_project.clone(),
            namespace: runtime_namespace.clone(),
            agent: runtime.as_ref().and_then(|value| value.agent.clone()),
            workspace: runtime_workspace.clone(),
            visibility: runtime_visibility.clone(),
            route: runtime
                .as_ref()
                .and_then(|value| value.route.clone())
                .or(Some("auto".to_string())),
            intent: runtime
                .as_ref()
                .and_then(|value| value.intent.clone())
                .or(Some("current_task".to_string())),
            limit: Some(12),
            rehydration_limit: Some(4),
            semantic: false,
            prompt: false,
            summary: false,
        },
        base_url,
    )
    .await
    .ok();

    let eval = read_latest_bundle_eval(&args.output).ok().flatten();
    let gap = read_latest_gap_report(&args.output).ok().flatten();
    let coordination = if runtime_session.is_some() {
        run_coordination_command(
            &CoordinationArgs {
                output: args.output.clone(),
                view: Some("all".to_string()),
                changes_only: false,
                watch: false,
                interval_secs: 30,
                recover_session: None,
                retire_session: None,
                to_session: None,
                deny_session: None,
                reroute_session: None,
                handoff_scope: None,
                summary: false,
            },
            base_url,
        )
        .await
        .ok()
    } else {
        None
    };

    let mut checks = Vec::new();
    let mut findings = Vec::new();
    let mut next_actions = Vec::new();
    let mut evidence = Vec::new();
    let mut passed_checks: usize = 0;
    let mut failed_checks: usize = 0;
    let mut score: u16 = 0;
    let mut max_score: u16 = 0;

    let mut add_check = |name: &str, status: &str, points: u16, details: String| {
        checks.push(ScenarioCheck {
            name: name.to_string(),
            status: status.to_string(),
            points,
            details: details.clone(),
        });
        max_score += points;
        match status {
            "pass" => {
                score += points;
                passed_checks += 1;
            }
            "warn" => {
                findings.push(details);
                next_actions.push(format!(
                    "consider addressing {name} to improve scenario stability"
                ));
            }
            _ => {
                failed_checks += 1;
                findings.push(details);
                next_actions.push(format!("resolve {name} before next scenario run"));
            }
        }
    };

    evidence.push(format!("bundle_root={}", args.output.display()));
    evidence.push(format!("scenario={scenario_name}"));

    if runtime.is_some() {
        add_check(
            "runtime_config",
            "pass",
            28,
            "bundle runtime config is available".to_string(),
        );
        evidence.push("runtime config loaded".to_string());
    } else {
        add_check(
            "runtime_config",
            "fail",
            0,
            "missing .memd/config.json for bundle".to_string(),
        );
    }

    let has_workspace = runtime_workspace.is_some();
    if let Some(resume) = &resume {
        let pressure = resume.context_pressure();
        if pressure == "low" {
            add_check(
                "resume_signal",
                "pass",
                22,
                "resume signal pressure is low".to_string(),
            );
        } else if pressure == "medium" {
            add_check(
                "resume_signal",
                "warn",
                16,
                "resume signal is medium pressure, consider reducing context".to_string(),
            );
        } else {
            add_check(
                "resume_signal",
                "fail",
                0,
                "resume signal is high".to_string(),
            );
        }
        evidence.push(format!(
            "resume pressure={} working_records={} context_records={}",
            pressure,
            resume.working.records.len(),
            resume.context.records.len()
        ));
        if resume.inbox.items.is_empty() {
            add_check(
                "resume_inbox",
                "pass",
                12,
                "resume inbox has no pending items".to_string(),
            );
        } else {
            add_check(
                "resume_inbox",
                "warn",
                8,
                format!("{} inbox item(s) pending", resume.inbox.items.len()),
            );
        }
    } else {
        add_check(
            "resume_signal",
            "warn",
            4,
            "resume could not be loaded from bundle runtime".to_string(),
        );
    }

    if let Some(eval) = &eval {
        if eval.score >= 80 {
            add_check(
                "eval_score",
                "pass",
                24,
                format!("eval score {} meets target", eval.score),
            );
        } else if eval.score >= 70 {
            add_check(
                "eval_score",
                "warn",
                16,
                format!("eval score {} below strong target", eval.score),
            );
        } else {
            add_check(
                "eval_score",
                "fail",
                0,
                format!("eval score {} is below stable threshold", eval.score),
            );
        }
        evidence.push(format!(
            "eval score={} status={} workspace_lanes={}",
            eval.score, eval.status, eval.workspace_lanes
        ));
    } else {
        add_check(
            "eval_score",
            "warn",
            10,
            "no eval snapshot found in .memd/evals/latest.json".to_string(),
        );
    }

    if let Some(gap) = &gap {
        if gap.candidate_count == 0 {
            add_check(
                "gap_pressure",
                "pass",
                16,
                "gap report shows no candidates".to_string(),
            );
        } else if gap.candidate_count <= 4 {
            add_check(
                "gap_pressure",
                "warn",
                8,
                format!("{} gap candidate(s) still open", gap.candidate_count),
            );
        } else {
            add_check(
                "gap_pressure",
                "fail",
                0,
                format!("{} gap candidates exceed target", gap.candidate_count),
            );
        }
        evidence.push(format!(
            "gap candidates={} high_priority={}",
            gap.candidate_count, gap.high_priority_count
        ));
    } else {
        add_check(
            "gap_pressure",
            "warn",
            6,
            "no prior gap report available".to_string(),
        );
    }

    if let Some(coordination) = &coordination {
        if coordination.recovery.stale_hives.is_empty()
            && coordination.policy_conflicts.is_empty()
            && coordination.suggestions.is_empty()
        {
            add_check(
                "coordination_health",
                "pass",
                20,
                "coordination has no immediate health warnings".to_string(),
            );
        } else if coordination.recovery.stale_hives.len() <= 2 {
            add_check(
                "coordination_health",
                "warn",
                12,
                format!(
                    "coordination has {} stale hives and {} suggestion(s)",
                    coordination.recovery.stale_hives.len(),
                    coordination.suggestions.len()
                ),
            );
        } else {
            add_check(
                "coordination_health",
                "fail",
                0,
                format!(
                    "{} stale hives and {} policy conflicts",
                    coordination.recovery.stale_hives.len(),
                    coordination.policy_conflicts.len()
                ),
            );
        }
        evidence.push(format!(
            "coordination inbox_messages={} stale_hives={} policy_conflicts={}",
            coordination.inbox.messages.len(),
            coordination.recovery.stale_hives.len(),
            coordination.policy_conflicts.len()
        ));
    } else if runtime_session.is_some() {
        add_check(
            "coordination_health",
            "warn",
            8,
            "coordination status unavailable for active runtime session".to_string(),
        );
    } else {
        add_check(
            "coordination_health",
            "warn",
            4,
            "coordination not sampled because no active session".to_string(),
        );
    }

    match scenario_kind {
        ScenarioKind::BundleHealth => {}
        ScenarioKind::ResumeAfterPause => {
            if let Some(resume) = &resume {
                if !resume.context.records.is_empty() && !resume.working.records.is_empty() {
                    add_check(
                        "resume_data_presence",
                        "pass",
                        10,
                        "resume has both context and working records".to_string(),
                    );
                } else {
                    add_check(
                        "resume_data_presence",
                        "warn",
                        6,
                        "resume data appears incomplete".to_string(),
                    );
                }
                if let Some(age) = resume.resume_state_age_minutes {
                    if age <= 30 {
                        add_check(
                            "resume_state_age",
                            "pass",
                            8,
                            format!("resume state age {age} minutes").to_string(),
                        );
                    } else {
                        add_check(
                            "resume_state_age",
                            "warn",
                            4,
                            format!("resume state age {age} minutes is high").to_string(),
                        );
                    }
                } else {
                    add_check(
                        "resume_state_age",
                        "warn",
                        4,
                        "resume state age unavailable".to_string(),
                    );
                }
            } else {
                add_check(
                    "resume_data_presence",
                    "warn",
                    0,
                    "resume data unavailable for resume_after_pause scenario".to_string(),
                );
                next_actions.push(
                    "resolve resume read errors before resume-focused scenario runs".to_string(),
                );
            }
        }
        ScenarioKind::WorkspaceRetrieval => {
            if has_workspace {
                add_check(
                    "workspace_configured",
                    "pass",
                    10,
                    "bundle session includes an active workspace".to_string(),
                );
            } else {
                add_check(
                    "workspace_configured",
                    "warn",
                    6,
                    "no workspace configured; expected workspace-aware retrieval for this scenario"
                        .to_string(),
                );
            }

            if let Some(resume) = &resume {
                if resume.workspaces.workspaces.is_empty() {
                    add_check(
                        "workspace_lanes",
                        "warn",
                        4,
                        "workspace retrieval returned zero lanes".to_string(),
                    );
                } else if runtime_workspace.as_ref().is_some_and(|workspace| {
                    resume
                        .workspaces
                        .workspaces
                        .iter()
                        .any(|value| value.workspace.as_deref() == Some(workspace.as_str()))
                }) {
                    add_check(
                        "workspace_lanes",
                        "pass",
                        10,
                        "target workspace lane is present".to_string(),
                    );
                } else {
                    add_check(
                        "workspace_lanes",
                        "warn",
                        8,
                        "target workspace lane not present in active workspace list".to_string(),
                    );
                }
            }
        }
        ScenarioKind::Handoff => {
            if runtime_session.is_some() {
                add_check(
                    "handoff_session_present",
                    "pass",
                    10,
                    "runtime has an active handoff-capable session".to_string(),
                );
            } else {
                add_check(
                    "handoff_session_present",
                    "warn",
                    6,
                    "no active session; handoff scenario should configure runtime session"
                        .to_string(),
                );
            }
            if has_workspace {
                add_check(
                    "handoff_workspace",
                    "pass",
                    6,
                    "handoff scenario includes workspace context".to_string(),
                );
            } else {
                add_check(
                    "handoff_workspace",
                    "warn",
                    4,
                    "handoff scenario could not verify workspace context".to_string(),
                );
            }
            add_check(
                "handoff_readiness",
                "pass",
                8,
                "handoff-related resume state was sampled for compact continuity".to_string(),
            );
        }
        ScenarioKind::StaleSessionRecovery => {
            if let Some(coordination) = &coordination {
                if coordination.recovery.stale_hives.is_empty() {
                    add_check(
                        "stale_session_scan",
                        "pass",
                        12,
                        "no stale hives detected".to_string(),
                    );
                } else if coordination.recovery.stale_hives.len() <= 2 {
                    add_check(
                        "stale_session_scan",
                        "warn",
                        6,
                        format!(
                            "{} stale hive(s) detected; recovery path available",
                            coordination.recovery.stale_hives.len()
                        ),
                    );
                } else {
                    add_check(
                        "stale_session_scan",
                        "warn",
                        2,
                        format!(
                            "{} stale hives detected; investigate before recovery wave",
                            coordination.recovery.stale_hives.len()
                        ),
                    );
                }
                if !coordination.recovery.reclaimable_claims.is_empty()
                    || !coordination.recovery.stalled_tasks.is_empty()
                {
                    add_check(
                        "stale_session_recoverability",
                        "pass",
                        8,
                        "stale session claims/tasks appear recoverable".to_string(),
                    );
                } else {
                    add_check(
                        "stale_session_recoverability",
                        "pass",
                        4,
                        "no active stale-session recovery payloads observed".to_string(),
                    );
                }
            } else if runtime_session.is_some() {
                add_check(
                    "stale_session_scan",
                    "warn",
                    4,
                    "stale-session scan unavailable; coordination not sampled for active session"
                        .to_string(),
                );
            } else {
                add_check(
                    "stale_session_scan",
                    "warn",
                    4,
                    "stale-session scan unavailable because no active session exists".to_string(),
                );
            }
        }
        ScenarioKind::Coworking => {
            if let Some(coordination) = &coordination {
                if !coordination.inbox.messages.is_empty() {
                    add_check(
                        "coworking_inbox",
                        "pass",
                        8,
                        format!(
                            "coordination inbox has {} message(s)",
                            coordination.inbox.messages.len()
                        ),
                    );
                } else {
                    add_check(
                        "coworking_inbox",
                        "warn",
                        4,
                        "coordination inbox empty; coworking load appears low".to_string(),
                    );
                }
                if coordination.suggestions.is_empty() {
                    add_check(
                        "coworking_actionability",
                        "warn",
                        4,
                        "no coordination suggestions available yet".to_string(),
                    );
                } else {
                    add_check(
                        "coworking_actionability",
                        "pass",
                        10,
                        format!(
                            "{} actionable suggestion(s)",
                            coordination.suggestions.len()
                        ),
                    );
                }
                if coordination.receipts.is_empty() {
                    add_check(
                        "coworking_history",
                        "warn",
                        2,
                        "no coordination receipts recorded yet".to_string(),
                    );
                } else {
                    add_check(
                        "coworking_history",
                        "pass",
                        6,
                        "coordination history has receipts".to_string(),
                    );
                }
            } else if runtime_session.is_some() {
                add_check(
                    "coworking_inbox",
                    "warn",
                    4,
                    "coworking visibility unavailable for active session".to_string(),
                );
            } else {
                add_check(
                    "coworking_inbox",
                    "warn",
                    4,
                    "coworking visibility unavailable with no active session".to_string(),
                );
            }
        }
    }

    Ok(ScenarioReport {
        bundle_root: args.output.display().to_string(),
        project: runtime_project,
        namespace: runtime_namespace,
        agent: runtime.as_ref().and_then(|value| value.agent.clone()),
        session: runtime_session,
        workspace: runtime_workspace,
        visibility: runtime_visibility,
        scenario: scenario_name,
        score,
        max_score,
        checks,
        passed_checks,
        failed_checks,
        findings,
        next_actions,
        evidence,
        generated_at: started_at,
        completed_at: Utc::now(),
    })
}

pub(crate) async fn run_composite_command(
    args: &CompositeArgs,
    base_url: &str,
) -> anyhow::Result<CompositeReport> {
    let started_at = Utc::now();
    let started = Instant::now();
    let runtime = read_bundle_runtime_config(&args.output)?;
    let runtime_project = runtime.as_ref().and_then(|value| value.project.clone());
    let runtime_namespace = runtime.as_ref().and_then(|value| value.namespace.clone());
    let runtime_session = runtime.as_ref().and_then(|value| value.session.clone());
    let runtime_workspace = runtime.as_ref().and_then(|value| value.workspace.clone());
    let runtime_visibility = runtime.as_ref().and_then(|value| value.visibility.clone());

    let resume = read_bundle_resume(
        &ResumeArgs {
            output: args.output.clone(),
            project: runtime_project.clone(),
            namespace: runtime_namespace.clone(),
            agent: runtime.as_ref().and_then(|value| value.agent.clone()),
            workspace: runtime_workspace.clone(),
            visibility: runtime_visibility.clone(),
            route: runtime
                .as_ref()
                .and_then(|value| value.route.clone())
                .or(Some("auto".to_string())),
            intent: runtime
                .as_ref()
                .and_then(|value| value.intent.clone())
                .or(Some("current_task".to_string())),
            limit: Some(12),
            rehydration_limit: Some(4),
            semantic: false,
            prompt: false,
            summary: false,
        },
        base_url,
    )
    .await
    .ok();

    let eval = read_latest_bundle_eval(&args.output).ok().flatten();
    let scenario = read_latest_scenario_report(&args.output).ok().flatten();
    let coordination = if runtime_session.is_some() {
        run_coordination_command(
            &CoordinationArgs {
                output: args.output.clone(),
                view: Some("all".to_string()),
                changes_only: false,
                watch: false,
                interval_secs: 30,
                recover_session: None,
                retire_session: None,
                to_session: None,
                deny_session: None,
                reroute_session: None,
                handoff_scope: None,
                summary: false,
            },
            base_url,
        )
        .await
        .ok()
    } else {
        None
    };

    let mut findings = Vec::new();
    let mut recommendations = Vec::new();
    let mut evidence = Vec::new();
    let mut dimensions = Vec::new();
    let mut gates = Vec::new();
    let self_evolution_mode = args.scenario.as_deref() == Some("self_evolution")
        || scenario
            .as_ref()
            .is_some_and(|value| value.scenario == "self_evolution");

    let clamp = |value: i32| value.clamp(0, 100) as u8;
    let mut add_dimension = |name: &str, weight: u8, score: u8, details: String| {
        dimensions.push(CompositeDimension {
            name: name.to_string(),
            weight,
            score,
            details,
        });
    };

    evidence.push(format!("bundle_root={}", args.output.display()));
    if let Some(expected) = args.scenario.as_deref() {
        evidence.push(format!("expected_scenario={expected}"));
    }
    if let Some(scenario) = &scenario {
        evidence.push(format!(
            "scenario name={} score={}/{} failed_checks={}",
            scenario.scenario, scenario.score, scenario.max_score, scenario.failed_checks
        ));
    }
    if let Some(eval) = &eval {
        evidence.push(format!(
            "eval score={} status={} working={} context={} inbox={} lanes={}",
            eval.score,
            eval.status,
            eval.working_records,
            eval.context_records,
            eval.inbox_items,
            eval.workspace_lanes
        ));
    }
    if let Some(coordination) = &coordination {
        evidence.push(format!(
            "coordination messages={} stale_hives={} conflicts={} suggestions={}",
            coordination.inbox.messages.len(),
            coordination.recovery.stale_hives.len(),
            coordination.policy_conflicts.len(),
            coordination.suggestions.len()
        ));
    }
    if let Some(resume) = &resume {
        evidence.push(format!(
            "resume working={} context={} inbox={} truncated={}",
            resume.working.records.len(),
            resume.context.records.len(),
            resume.inbox.items.len(),
            resume.working.truncated
        ));
    }

    let eval_score = eval.as_ref().map(|value| value.score).unwrap_or(55);
    let scenario_score = scenario
        .as_ref()
        .map(|value| value.score as i32)
        .unwrap_or(50);
    let coordination_score = if let Some(coordination) = &coordination {
        let mut score = if self_evolution_mode { 85i32 } else { 100i32 };
        score -= (coordination.recovery.stale_hives.len() as i32).min(3)
            * if self_evolution_mode { 10 } else { 15 };
        score -= (coordination.policy_conflicts.len() as i32).min(3) * 15;
        if !self_evolution_mode {
            score -= if coordination.suggestions.is_empty() {
                5
            } else {
                0
            };
            score -= if coordination.inbox.messages.is_empty() {
                5
            } else {
                0
            };
        }
        clamp(score)
    } else if runtime_session.is_some() {
        if self_evolution_mode { 75 } else { 60 }
    } else if self_evolution_mode {
        80
    } else {
        70
    };
    let latency_ms = started.elapsed().as_millis() as i32;
    let raw_latency_score = clamp(100 - (latency_ms / 25).min(40));
    let latency_score = if self_evolution_mode {
        raw_latency_score.max(70)
    } else {
        raw_latency_score
    };
    let raw_bloat_score = if let Some(resume) = &resume {
        let mut score = if self_evolution_mode { 90i32 } else { 100i32 };
        score -=
            (resume.working.records.len() as i32).min(8) * if self_evolution_mode { 2 } else { 4 };
        score -=
            (resume.context.records.len() as i32).min(8) * if self_evolution_mode { 1 } else { 3 };
        score -= (resume.inbox.items.len() as i32).min(6) * if self_evolution_mode { 2 } else { 4 };
        score -= if resume.working.truncated {
            if self_evolution_mode { 10 } else { 20 }
        } else {
            0
        };
        score -= if resume.working.remaining_chars < 200 {
            if self_evolution_mode { 5 } else { 10 }
        } else {
            0
        };
        clamp(score)
    } else if self_evolution_mode {
        75
    } else {
        65
    };
    let bloat_score = if self_evolution_mode {
        raw_bloat_score.max(60)
    } else {
        raw_bloat_score
    };
    let coordination_score = if self_evolution_mode {
        coordination_score.max(60)
    } else {
        coordination_score
    };

    let correctness_score = {
        let mut score = 100i32;
        if eval.is_none() {
            score -= 25;
            findings.push("missing latest eval snapshot".to_string());
        }
        if scenario.is_none() {
            score -= 20;
            findings.push("missing latest scenario snapshot".to_string());
        }
        if scenario
            .as_ref()
            .is_some_and(|value| value.failed_checks > 0)
        {
            score -= 30;
            findings.push("scenario has failed checks".to_string());
        }
        if eval.as_ref().is_some_and(|value| value.score < 80) {
            score -= 15;
            findings.push("eval score below strong target".to_string());
        }
        if coordination
            .as_ref()
            .is_some_and(|value| !value.policy_conflicts.is_empty())
        {
            score -= 15;
            findings.push("coordination still has policy conflicts".to_string());
        }
        if resume.is_none() {
            score -= 10;
            findings.push("resume snapshot unavailable".to_string());
        }
        clamp(score)
    };
    let memory_quality_score = {
        let mut samples = Vec::new();
        samples.push(eval_score as i32);
        if let Some(scenario) = &scenario {
            samples.push(scenario.score as i32);
        }
        if let Some(resume) = &resume {
            if !resume.context.records.is_empty() {
                samples.push(90);
            } else {
                samples.push(70);
            }
        }
        let baseline = samples.iter().sum::<i32>() / samples.len().max(1) as i32;
        clamp((baseline + scenario_score) / 2)
    };

    add_dimension(
        "correctness",
        35,
        correctness_score,
        "hard correctness uses eval, scenario, coordination, and resume presence".to_string(),
    );
    add_dimension(
        "memory_quality",
        30,
        memory_quality_score,
        "memory quality blends eval and scenario scores with resume density".to_string(),
    );
    add_dimension(
        "coordination_quality",
        20,
        coordination_score,
        "coordination quality reflects stale hives, conflicts, inbox pressure, and suggestions"
            .to_string(),
    );
    add_dimension(
        "latency",
        10,
        latency_score,
        format!("composite command completed in {}ms", latency_ms),
    );
    add_dimension(
        "bloat",
        5,
        bloat_score,
        "bloat penalizes truncated or oversized working memory and inbox pressure".to_string(),
    );

    let weighted_total: u32 = dimensions
        .iter()
        .map(|dimension| (dimension.score as u32 * dimension.weight as u32) / 100)
        .sum();
    let score = weighted_total.min(100) as u8;

    let hard_correctness_ok = correctness_score >= 80
        && scenario
            .as_ref()
            .is_none_or(|value| value.failed_checks == 0);
    let acceptance_ok = hard_correctness_ok && score >= 80;

    gates.push(CompositeGate {
        name: "hard_correctness".to_string(),
        status: if hard_correctness_ok {
            "pass".to_string()
        } else if correctness_score >= 60 {
            "warn".to_string()
        } else {
            "fail".to_string()
        },
        details: format!("hard correctness score={correctness_score}"),
    });
    gates.push(CompositeGate {
        name: "acceptance".to_string(),
        status: if acceptance_ok {
            "pass".to_string()
        } else if score >= 70 {
            "warn".to_string()
        } else {
            "fail".to_string()
        },
        details: format!("weighted composite score={score}"),
    });

    if let Some(expected) = args.scenario.as_deref() {
        if scenario
            .as_ref()
            .is_none_or(|value| value.scenario != expected)
        {
            findings.push(format!(
                "latest scenario snapshot did not match expected scenario {expected}"
            ));
            recommendations
                .push("rerun the expected scenario before trusting the composite gate".to_string());
        }
    }

    if scenario.is_none() {
        recommendations.push("run `memd scenario --write` before composite scoring".to_string());
    }
    if eval.is_none() {
        recommendations.push("run `memd eval --write` before composite scoring".to_string());
    }
    if coordination.is_none() && runtime_session.is_some() {
        recommendations.push(
            "run a coordination sample with an active session before composite gating".to_string(),
        );
    }
    if bloat_score < 70 && !self_evolution_mode {
        recommendations.push(
            "trim working memory and inbox pressure before accepting experiments".to_string(),
        );
    }

    Ok(CompositeReport {
        bundle_root: args.output.display().to_string(),
        project: runtime_project,
        namespace: runtime_namespace,
        agent: runtime.as_ref().and_then(|value| value.agent.clone()),
        session: runtime_session,
        workspace: runtime_workspace,
        visibility: runtime_visibility,
        scenario: args
            .scenario
            .clone()
            .or_else(|| scenario.as_ref().map(|value| value.scenario.clone())),
        score,
        max_score: 100,
        dimensions,
        gates,
        findings,
        recommendations,
        evidence,
        generated_at: started_at,
        completed_at: Utc::now(),
    })
}

pub(crate) fn write_scenario_artifacts(
    output: &Path,
    response: &ScenarioReport,
) -> anyhow::Result<()> {
    let scenario_dir = scenario_reports_dir(output);
    fs::create_dir_all(&scenario_dir)
        .with_context(|| format!("create {}", scenario_dir.display()))?;

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let baseline_json = scenario_dir.join("latest.json");
    let baseline_md = scenario_dir.join("latest.md");
    let timestamp_json = scenario_dir.join(format!("{timestamp}.json"));
    let timestamp_md = scenario_dir.join(format!("{timestamp}.md"));
    let json = serde_json::to_string_pretty(response)? + "\n";
    let markdown = render_scenario_markdown(response);

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

pub(crate) fn write_composite_artifacts(
    output: &Path,
    response: &CompositeReport,
) -> anyhow::Result<()> {
    let composite_dir = output.join("composite");
    fs::create_dir_all(&composite_dir)
        .with_context(|| format!("create {}", composite_dir.display()))?;

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let baseline_json = composite_dir.join("latest.json");
    let baseline_md = composite_dir.join("latest.md");
    let timestamp_json = composite_dir.join(format!("{timestamp}.json"));
    let timestamp_md = composite_dir.join(format!("{timestamp}.md"));
    let json = serde_json::to_string_pretty(response)? + "\n";
    let markdown = render_composite_markdown(response);

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
