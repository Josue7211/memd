use super::*;

pub(crate) fn build_gap_candidates(
    output: &Path,
    runtime: &Option<BundleRuntimeConfig>,
    resume: &Option<ResumeSnapshot>,
    state: Option<&BundleResumeState>,
    eval: Option<&BundleEvalResponse>,
    coordination: Option<&CoordinationResponse>,
    awareness: Option<&ProjectAwarenessResponse>,
    research_loops_doc_count: Option<usize>,
    recent_commits: &[String],
    evidence: &mut Vec<String>,
    benchmark_registry: Option<&BenchmarkRegistry>,
) -> Vec<GapCandidate> {
    let mut candidates = Vec::new();
    let add = |candidates: &mut Vec<GapCandidate>,
               area: &str,
               signal: &str,
               priority: u8,
               evidence: Vec<String>,
               recommendation: &str| {
        candidates.push(GapCandidate {
            id: format!("{}:{signal}", area),
            area: area.to_string(),
            priority,
            severity: if priority >= 85 {
                "high".to_string()
            } else if priority >= 65 {
                "medium".to_string()
            } else {
                "low".to_string()
            },
            signal: signal.to_string(),
            evidence,
            recommendation: recommendation.to_string(),
        });
    };

    if let Some(eval) = eval {
        if eval.score < 70 {
            add(
                &mut candidates,
                "memory",
                "low_eval_score",
                95,
                vec![format!(
                    "memd eval score {} with status {}",
                    eval.score, eval.status
                )],
                "run `memd eval --write --summary` and address top recommendations before the next context switch",
            );
        } else if eval.score < 82 {
            add(
                &mut candidates,
                "memory",
                "below_target_eval_score",
                76,
                vec![format!(
                    "eval score {} indicates medium risk, status {}",
                    eval.score, eval.status
                )],
                "close immediate resume-pressure gaps (context, rehydration, inbox pressure) and rerun `memd eval`",
            );
        }
        if eval.inbox_items >= 6 {
            add(
                &mut candidates,
                "memory",
                "inbox_pressure",
                72,
                vec![format!(
                    "eval inbox_items={} indicates pressure",
                    eval.inbox_items
                )],
                "triage/ack backlog with `memd coordination` then rerun resume",
            );
        }
    } else {
        add(
            &mut candidates,
            "memory",
            "no_eval_snapshot",
            82,
            vec!["no .memd/evals/latest.json was available".to_string()],
            "run `memd eval --write --summary` to establish a baseline before gap scoring",
        );
    }

    if let Some(snapshot) = resume {
        if snapshot.working.records.len() <= 1 {
            add(
                &mut candidates,
                "memory",
                "weak_working_lane",
                86,
                vec![format!(
                    "working.records={}",
                    snapshot.working.records.len()
                )],
                "capture durable and short-term lane before resuming high-cost tasks",
            );
        }
        if snapshot.context.records.is_empty() {
            add(
                &mut candidates,
                "memory",
                "empty_context_lane",
                84,
                vec!["compact context returned no records for current route/intent".to_string()],
                "verify active project/namespace and reset route/intent defaults",
            );
        }
        if snapshot.working.rehydration_queue.is_empty() {
            add(
                &mut candidates,
                "memory",
                "empty_rehydration_queue",
                66,
                vec!["working.rehydration_queue empty".to_string()],
                "write a checkpointable deep-context item and rerun handoff/resume",
            );
        }
        if snapshot.workspace.is_some() && snapshot.workspaces.workspaces.is_empty() {
            add(
                &mut candidates,
                "memory",
                "missing_active_workspace_lane",
                70,
                vec!["active workspace had no workspace lane visibility".to_string()],
                "repair workspace visibility and rehydrate shared lane state",
            );
        }
        if snapshot.inbox.items.len() >= 7 {
            add(
                &mut candidates,
                "memory",
                "inbox_growth",
                68,
                vec![format!("inbox items={}", snapshot.inbox.items.len())],
                "drain high-urgency items and clear stale messages before the next decision",
            );
        }
    } else if let Some(state) = state {
        if state.working_records <= 1 {
            add(
                &mut candidates,
                "memory",
                "resume_state_weak",
                80,
                vec![
                    "resume snapshot unavailable; using last saved state".to_string(),
                    format!("working_records={}", state.working_records),
                ],
                "resume the bundle and immediately run `memd eval --write --summary`",
            );
        }
        if state.inbox_items >= 7 {
            add(
                &mut candidates,
                "memory",
                "resume_state_inbox_backlog",
                64,
                vec![format!("saved inbox_items={}", state.inbox_items)],
                "refresh resume and inspect backlog with `memd coordination --summary`",
            );
        }
    } else {
        add(
            &mut candidates,
            "memory",
            "missing_resume_state",
            74,
            vec!["resume state was not available locally".to_string()],
            "run `memd resume` once so gap reports get live lane evidence",
        );
    }

    if let Some(coordination) = coordination {
        if !coordination.recovery.stale_hives.is_empty()
            && (!coordination.recovery.reclaimable_claims.is_empty()
                || !coordination.recovery.stalled_tasks.is_empty())
        {
            add(
                &mut candidates,
                "coordination",
                "stale_hives_recovery",
                90,
                vec![format!(
                    "stale hives={}",
                    coordination.recovery.stale_hives.len()
                )],
                "recover stale sessions before assigning new claims",
            );
        }
        if !coordination.policy_conflicts.is_empty() {
            add(
                &mut candidates,
                "coordination",
                "policy_conflicts",
                84,
                vec![format!(
                    "policy_conflicts={}",
                    coordination.policy_conflicts.len()
                )],
                "resolve conflicts by explicit assign/recover actions",
            );
        }
        if coordination.inbox.messages.len() >= 6 {
            add(
                &mut candidates,
                "coordination",
                "message_backlog",
                76,
                vec![format!(
                    "inbox messages={}",
                    coordination.inbox.messages.len()
                )],
                "ack now and reduce queue churn before adding new tasks",
            );
        }
        if coordination.suggestions.len() >= 3 {
            add(
                &mut candidates,
                "coordination",
                "stale_action_pressure",
                62,
                vec![format!(
                    "coordination suggestions={} pending",
                    coordination.suggestions.len()
                )],
                "execute highest-priority coordination suggestion via bounded actions",
            );
        }
    } else if !coordination_exists(output) {
        add(
            &mut candidates,
            "coordination",
            "coordination_unreachable",
            60,
            vec!["coordination snapshot was unavailable".to_string()],
            "configure bundle session/base_url and rerun `memd gap`",
        );
    }

    if let Some(awareness) = awareness {
        let visible_entries = project_awareness_visible_entries(awareness);
        let visible_diagnostics = awareness_summary_diagnostics(&visible_entries);
        let current_entry = awareness
            .entries
            .iter()
            .find(|entry| entry.bundle_root == awareness.current_bundle);
        let active_sessions = visible_entries
            .iter()
            .copied()
            .filter(|entry| entry.presence == "active")
            .collect::<Vec<_>>();
        let unhived_active_sessions = active_sessions
            .iter()
            .filter(|entry| entry.hive_system.as_deref().is_none() || entry.hive_groups.is_empty())
            .count();
        if unhived_active_sessions > 0 {
            add(
                &mut candidates,
                "coordination",
                "unhived_active_sessions",
                88,
                vec![
                    format!("active sessions={}", active_sessions.len()),
                    format!("unhived active sessions={}", unhived_active_sessions),
                    format!("awareness root={}", awareness.root),
                ],
                "publish hive metadata for active sessions before assigning new claims",
            );
        }
        if !visible_diagnostics.is_empty() {
            add(
                &mut candidates,
                "coordination",
                "awareness_collisions",
                74,
                visible_diagnostics.clone(),
                "resolve session collisions so awareness reflects one live owner per session",
            );
        }

        let stale_remote_sessions = visible_entries
            .iter()
            .copied()
            .filter(|entry| entry.project_dir == "remote")
            .filter(|entry| entry.presence == "stale" || entry.presence == "dead")
            .collect::<Vec<_>>();
        if current_entry.is_some_and(|entry| entry.presence == "active")
            && !stale_remote_sessions.is_empty()
        {
            let recoverable_sessions = stale_remote_sessions
                .iter()
                .copied()
                .filter(|entry| {
                    let session = entry.session.as_deref();
                    coordination.is_some_and(|value| {
                        value
                            .recovery
                            .reclaimable_claims
                            .iter()
                            .any(|claim| claim.session.as_deref() == session)
                            || value
                                .recovery
                                .stalled_tasks
                                .iter()
                                .any(|task| task.session.as_deref() == session)
                    })
                })
                .collect::<Vec<_>>();
            let retireable_sessions = stale_remote_sessions
                .iter()
                .copied()
                .filter(|entry| {
                    let session = entry.session.as_deref();
                    !coordination.is_some_and(|value| {
                        value
                            .recovery
                            .reclaimable_claims
                            .iter()
                            .any(|claim| claim.session.as_deref() == session)
                            || value
                                .recovery
                                .stalled_tasks
                                .iter()
                                .any(|task| task.session.as_deref() == session)
                    })
                })
                .collect::<Vec<_>>();
            let sessions = stale_remote_sessions
                .iter()
                .take(3)
                .filter_map(|entry| entry.session.as_deref())
                .collect::<Vec<_>>();
            let recovery_hint = recoverable_sessions
                .iter()
                .filter_map(|entry| entry.session.as_deref())
                .next()
                .map(|session| format!("memd coordination --recover-session {session}"))
                .unwrap_or_else(|| "none".to_string());
            let retirement_hint = retireable_sessions
                .iter()
                .filter_map(|entry| entry.session.as_deref())
                .next()
                .map(|session| format!("memd coordination --retire-session {session}"))
                .unwrap_or_else(|| "none".to_string());
            add(
                &mut candidates,
                "coordination",
                "stale_remote_sessions",
                87,
                vec![
                    format!("stale remote sessions={}", stale_remote_sessions.len()),
                    format!("recoverable sessions={}", recoverable_sessions.len()),
                    format!("retireable sessions={}", retireable_sessions.len()),
                    format!(
                        "sessions={}",
                        if sessions.is_empty() {
                            "unknown".to_string()
                        } else {
                            sessions.join(",")
                        }
                    ),
                    format!("recovery={recovery_hint}"),
                    format!("retirement={retirement_hint}"),
                ],
                "recover stale sessions with owned work and retire the rest before adding new claims",
            );
        }
    }

    if let Some(doc_count) = research_loops_doc_count {
        let manifest_count = AUTORESEARCH_LOOPS.len();
        if doc_count != manifest_count {
            add(
                &mut candidates,
                "docs",
                "loop_manifest_drift",
                83,
                vec![
                    format!("docs/research-loops.md lists {} loop entries", doc_count),
                    format!("runtime autoresearch manifest has {} loops", manifest_count),
                ],
                "update docs/research-loops.md to match `memd autoresearch --manifest` and rerun gap scoring",
            );
        }
    }

    if let Some(registry) = benchmark_registry {
        let benchmark_gaps = build_benchmark_gap_candidates(registry);
        evidence.push(format!("benchmark coverage gaps={}", benchmark_gaps.len()));
        candidates.extend(benchmark_gaps);
    } else {
        evidence.push("benchmark coverage gaps=unavailable".to_string());
    }

    if let Some(runtime) = runtime {
        if let Some(agent) = runtime.agent.as_ref() {
            let mut session_hint = String::new();
            if let Some(session) = runtime.session.as_ref() {
                session_hint.push_str(session);
            }
            if !recent_commits.is_empty() {
                evidence.push(format!("agent={agent} session={session_hint}"));
            }
        }
    }

    if recent_commits.is_empty() {
        add(
            &mut candidates,
            "research_loop",
            "no_recent_commits",
            58,
            vec!["no local commits discovered for configured limit".to_string()],
            "run `git log` with commits available and compare gap deltas across windows",
        );
    } else {
        evidence.push(format!(
            "recent commits: {}",
            recent_commits.first().map_or("none", |value| value)
        ));
    }

    candidates
}

pub(crate) fn research_loops_doc_loop_count(project_root: &Path) -> Option<usize> {
    let path = project_root.join("docs/research-loops.md");
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

pub(crate) fn evaluate_gap_changes(current: &GapReport, baseline: Option<&GapReport>) -> Vec<String> {
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

pub(crate) fn prioritize_gap_candidates(mut candidates: Vec<GapCandidate>, limit: usize) -> Vec<GapCandidate> {
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
