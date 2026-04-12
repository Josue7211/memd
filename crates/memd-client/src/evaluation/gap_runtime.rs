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
                    format!(
                        "docs/strategy/research-loops.md lists {} loop entries",
                        doc_count
                    ),
                    format!("runtime autoresearch manifest has {} loops", manifest_count),
                ],
                "update docs/strategy/research-loops.md to match `memd autoresearch --manifest` and rerun gap scoring",
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
