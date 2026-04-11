use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AutoresearchSweepSignatureEntry {
    slug: String,
    status: String,
    percent_bp: i64,
    tokens_bp: i64,
}

pub(crate) fn build_autoresearch_sweep_signature(
    records: &[(&'static AutoresearchLoop, LoopRecord)],
) -> Vec<AutoresearchSweepSignatureEntry> {
    records
        .iter()
        .map(|(descriptor, record)| AutoresearchSweepSignatureEntry {
            slug: descriptor.slug.to_string(),
            status: record
                .status
                .clone()
                .unwrap_or_else(|| "pending".to_string()),
            percent_bp: (record.percent_improvement.unwrap_or(0.0) * 100.0).round() as i64,
            tokens_bp: (record.token_savings.unwrap_or(0.0) * 100.0).round() as i64,
        })
        .collect()
}

pub(crate) async fn execute_autoresearch_sweep(
    output: &Path,
    base_url: &str,
    loops: &[&'static AutoresearchLoop],
) -> anyhow::Result<Vec<(&'static AutoresearchLoop, LoopRecord)>> {
    let summary = read_loop_summary(&loops_summary_path(output))?;
    let mut join_set = JoinSet::new();

    for (index, descriptor) in loops.iter().copied().enumerate() {
        let previous_runs = summary
            .entries
            .iter()
            .filter(|entry| entry.slug == descriptor.slug)
            .count();
        let previous_entry = summary
            .entries
            .iter()
            .rev()
            .find(|entry| entry.slug == descriptor.slug)
            .cloned();
        let output = output.to_path_buf();
        let base_url = base_url.to_string();
        join_set.spawn(async move {
            let record = build_autoresearch_record_for_descriptor(
                &output,
                &base_url,
                descriptor,
                previous_runs,
                previous_entry.as_ref(),
            )
            .await?;
            Ok::<_, anyhow::Error>((index, descriptor, record))
        });
    }

    let mut completed = Vec::with_capacity(loops.len());
    while let Some(result) = join_set.join_next().await {
        let (index, descriptor, record) = result??;
        completed.push((index, descriptor, record));
    }
    completed.sort_by_key(|(index, _, _)| *index);

    let mut persisted = Vec::with_capacity(completed.len());
    for (_, descriptor, record) in completed {
        persist_loop_record(output, &record)?;
        println!(
            "Recorded loop {}: {} improvement, {} token savings",
            descriptor.slug,
            format_percent(record.percent_improvement),
            format_tokens(record.token_savings)
        );
        persisted.push((descriptor, record));
    }

    Ok(persisted)
}

pub(crate) async fn execute_autoresearch_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
) -> anyhow::Result<()> {
    let summary = read_loop_summary(&loops_summary_path(output))?;
    let previous_runs = summary
        .entries
        .iter()
        .filter(|entry| entry.slug == descriptor.slug)
        .count();
    let previous_entry = summary
        .entries
        .iter()
        .rev()
        .find(|entry| entry.slug == descriptor.slug);

    let record = build_autoresearch_record_for_descriptor(
        output,
        base_url,
        descriptor,
        previous_runs,
        previous_entry,
    )
    .await?;

    persist_loop_record(output, &record)?;
    println!(
        "Recorded loop {}: {} improvement, {} token savings",
        descriptor.slug,
        format_percent(record.percent_improvement),
        format_tokens(record.token_savings)
    );
    Ok(())
}

pub(crate) async fn build_autoresearch_record_for_descriptor(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let record = match descriptor.slug {
        "branch-review-quality" => {
            run_branch_review_quality_loop(output, descriptor, previous_runs, previous_entry)
                .await?
        }
        "prompt-efficiency" => {
            run_prompt_efficiency_loop(output, base_url, descriptor, previous_runs, previous_entry)
                .await?
        }
        "signal-freshness" => {
            run_live_truth_loop(output, base_url, descriptor, previous_runs, previous_entry).await?
        }
        "autonomy-quality" => {
            run_autonomy_quality_loop(output, base_url, descriptor, previous_runs, previous_entry)
                .await?
        }
        "hive-health" => {
            run_hive_health_loop(output, descriptor, previous_runs, previous_entry).await?
        }
        "memory-hygiene" => {
            run_memory_hygiene_loop(output, base_url, descriptor, previous_runs, previous_entry)
                .await?
        }
        "repair-rate" => {
            run_repair_rate_loop(output, base_url, descriptor, previous_runs, previous_entry)
                .await?
        }
        "cross-harness" => {
            run_cross_harness_loop(output, descriptor, previous_runs, previous_entry).await?
        }
        "self-evolution" => {
            run_self_evolution_loop(output, descriptor, previous_runs, previous_entry).await?
        }
        "docs-spec-drift" => {
            run_docs_spec_drift_loop(output, descriptor, previous_runs, previous_entry).await?
        }
        _ => anyhow::bail!("unsupported autoresearch loop '{}'", descriptor.slug),
    };
    Ok(record)
}

pub(crate) fn print_autoresearch_manifest() {
    println!("Autoresearch manifest ({} loops)", AUTORESEARCH_LOOPS.len());
    for descriptor in AUTORESEARCH_LOOPS.iter() {
        println!("- {} ({})", descriptor.name, descriptor.slug);
        println!("  description: {}", descriptor.description);
        println!("  target: {}", descriptor.target);
        println!("  metric: {}", descriptor.metric);
        println!("  stop: {}", descriptor.stop_condition);
        println!("  risk: {}", descriptor.risk);
    }
}

pub(crate) async fn run_autoresearch(args: &AutoresearchArgs, base_url: &str) -> anyhow::Result<()> {
    if args.manifest {
        print_autoresearch_manifest();
        return Ok(());
    }

    if !args.auto && args.loop_slug.is_none() {
        anyhow::bail!("specify --auto to run every loop or --loop to run a single loop");
    }

    let loops: Vec<_> = if args.auto {
        AUTORESEARCH_LOOPS.iter().collect()
    } else if let Some(slug) = &args.loop_slug {
        let normalized = canonical_slug(slug).to_lowercase();
        AUTORESEARCH_LOOPS
            .iter()
            .filter(|descriptor| descriptor.normalized_slug == normalized)
            .collect()
    } else {
        Vec::new()
    };

    if let Some((_, registry)) = load_benchmark_registry_for_output(&args.output)
        .ok()
        .flatten()
    {
        let benchmark_gaps = build_benchmark_gap_candidates(&registry);
        if !benchmark_gaps.is_empty() {
            println!(
                "Benchmark coverage gaps detected: {} candidate(s)",
                benchmark_gaps.len()
            );
            for gap in benchmark_gaps.iter().take(3) {
                println!("- {}: {}", gap.id, gap.recommendation);
            }
        }
    }

    if loops.is_empty() {
        anyhow::bail!("no loops matched; run with --manifest to see available loops");
    }

    if !args.auto && loops.len() == 1 {
        execute_autoresearch_loop(&args.output, base_url, loops[0]).await?;
        return Ok(());
    }

    let max_sweeps = if args.auto { args.max_sweeps.max(1) } else { 1 };
    let mut stable_sweeps = 0usize;
    let mut previous_signature: Option<Vec<AutoresearchSweepSignatureEntry>> = None;

    for sweep in 1..=max_sweeps {
        let records = execute_autoresearch_sweep(&args.output, base_url, &loops).await?;
        let signature = build_autoresearch_sweep_signature(&records);
        if previous_signature.as_ref() == Some(&signature) {
            stable_sweeps += 1;
        } else {
            stable_sweeps = 0;
        }
        previous_signature = Some(signature);

        if args.auto && max_sweeps > 1 {
            println!("Completed autoresearch sweep {sweep}/{max_sweeps}");
        }
        if args.auto && args.plateau_sweeps > 0 && stable_sweeps >= args.plateau_sweeps {
            println!(
                "Autoresearch plateau detected after {} stable sweep(s); stopping early.",
                stable_sweeps
            );
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) async fn run_prompt_surface_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    run_prompt_efficiency_loop(output, base_url, descriptor, previous_runs, previous_entry).await
}

pub(crate) async fn run_branch_review_quality_loop(
    output: &Path,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let root = infer_bundle_project_root(output).unwrap_or_else(|| output.to_path_buf());
    let evidence = collect_gap_repo_evidence(&root);
    let branch = std::process::Command::new("git")
        .arg("-C")
        .arg(&root)
        .arg("branch")
        .arg("--show-current")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    let review_ready = !evidence.iter().any(|line| line.contains("dirty"));
    let percent = if review_ready { 100.0 } else { 0.0 };
    let token_savings = if branch == "unknown" { 0.0 } else { 20.0 };
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let status = if loop_success_requires_second_signal(
        confidence_met,
        branch != "unknown" && review_ready,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        format!("branch {} review_ready={}", branch, review_ready),
        vec!["branch review".to_string()],
        serde_json::json!({
            "evidence": evidence,
            "branch": branch,
            "review_ready": review_ready,
        }),
        status,
    ))
}

pub(crate) async fn run_prompt_efficiency_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let snapshot = read_bundle_resume(&autoresearch_resume_args(output), base_url).await?;
    let estimated_tokens = snapshot.estimated_prompt_tokens() as f64;
    let core_tokens = snapshot.core_prompt_tokens() as f64;
    let percent = improvement_less_is_better(core_tokens, estimated_tokens);
    let token_savings = (estimated_tokens - core_tokens).max(0.0);
    let summary = format!(
        "prompt tokens = {} (core {}, saved {})",
        estimated_tokens, core_tokens, token_savings
    );
    let evidence = vec![
        format!("estimated_tokens={}", estimated_tokens),
        format!("core_prompt_tokens={}", core_tokens),
        format!("context_pressure={}", snapshot.context_pressure()),
        format!("redundant_items={}", snapshot.redundant_context_items()),
    ];
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let secondary_signal_ok = snapshot.context_pressure() != "high"
        || snapshot.redundant_context_items() == 0
        || token_savings >= descriptor.base_tokens * 2.0;
    let status = if loop_success_requires_second_signal(
        confidence_met,
        secondary_signal_ok,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        summary,
        vec!["prompt efficiency".to_string()],
        serde_json::json!({
            "evidence": evidence,
            "estimated_prompt_tokens": estimated_tokens,
            "core_prompt_tokens": core_tokens,
            "context_pressure": snapshot.context_pressure(),
            "refresh_recommended": snapshot.refresh_recommended,
            "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, 3),
            "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
        }),
        status,
    ))
}

pub(crate) async fn run_hive_health_loop(
    output: &Path,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let awareness = read_project_awareness(&AwarenessArgs {
        output: output.to_path_buf(),
        root: None,
        include_current: true,
        summary: false,
    })
    .await?;
    let heartbeat = build_hive_heartbeat(output, None)?;
    let visible_entries = project_awareness_visible_entries(&awareness);
    let current_entry = visible_entries
        .iter()
        .find(|entry| entry.bundle_root == awareness.current_bundle);
    let relevant_collisions = awareness
        .collisions
        .iter()
        .filter(|collision| !collision.starts_with("base_url "))
        .collect::<Vec<_>>();
    let dead_hives = visible_entries
        .iter()
        .filter(|entry| entry.presence == "dead")
        .filter(|entry| {
            current_entry.is_some_and(|current| {
                entry.project_dir != "remote"
                    && entry.project == current.project
                    && entry.namespace == current.namespace
                    && entry.workspace == current.workspace
            })
        })
        .count();
    let percent = if relevant_collisions.is_empty() {
        100.0
    } else {
        100.0 - (relevant_collisions.len() as f64 * 10.0)
    };
    let token_savings = (visible_entries.len() as f64) * 8.0;
    let evidence = vec![
        format!("active_hives={}", visible_entries.len()),
        format!("dead_hives={}", dead_hives),
        format!("claim_collisions={}", relevant_collisions.len()),
    ];
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let secondary_signal_ok = relevant_collisions.is_empty() && dead_hives == 0;
    let warning_reasons = {
        let mut reasons = Vec::new();
        if dead_hives > 0 {
            reasons.push("dead_hive_sessions".to_string());
        }
        if !relevant_collisions.is_empty() {
            reasons.push("claim_collisions_detected".to_string());
        }
        if loop_is_regressed(descriptor, previous_entry, percent, token_savings) {
            reasons.extend(loop_trend_warning_reasons(
                descriptor,
                previous_entry,
                percent,
                token_savings,
            ));
        }
        if !confidence_met {
            reasons.extend(loop_floor_warning_reasons(
                descriptor,
                percent,
                token_savings,
                evidence.len(),
            ));
        }
        reasons
    };
    let status = if loop_success_requires_second_signal(
        confidence_met,
        secondary_signal_ok,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        "hive health score".to_string(),
        vec!["hive health".to_string()],
        serde_json::json!({
            "active_hives": visible_entries.len(),
            "dead_hives": dead_hives,
            "claim_collisions": relevant_collisions.len(),
            "evidence": evidence,
            "heartbeat_status": heartbeat.status,
            "confidence": loop_confidence_metadata(
                descriptor,
                percent,
                token_savings,
                confidence_met,
                3,
            ),
            "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
            "warning_reasons": warning_reasons,
        }),
        status,
    ))
}

pub(crate) async fn run_memory_hygiene_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let snapshot = read_bundle_resume(
        &autoresearch_resume_args_with_limits(output, 4, 2, true),
        base_url,
    )
    .await?;
    let mut seen = std::collections::HashSet::new();
    let mut duplicates = 0usize;
    for record in snapshot
        .context
        .records
        .iter()
        .chain(snapshot.working.records.iter())
    {
        let normalized = record.record.trim().to_lowercase();
        if !normalized.is_empty() && !seen.insert(normalized) {
            duplicates += 1;
        }
    }
    let total_records = snapshot.context.records.len() + snapshot.working.records.len();
    let event_spine_entries = snapshot.event_spine().len();
    let secondary_signal_ok = duplicates == 0 && event_spine_entries > 0;
    let percent = if secondary_signal_ok { 100.0 } else { 0.0 };
    let token_savings = if secondary_signal_ok {
        descriptor.base_tokens
    } else {
        0.0
    };
    let evidence = vec![
        format!("duplicates={duplicates}"),
        format!("records={total_records}"),
        format!("event_spine_entries={event_spine_entries}"),
    ];
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let mut warning_reasons = Vec::new();
    if duplicates > 0 {
        warning_reasons.push("duplicate_memory_pressure".to_string());
    }
    if event_spine_entries == 0 {
        warning_reasons.push("empty_event_spine".to_string());
    }
    if loop_is_regressed(descriptor, previous_entry, percent, token_savings) {
        warning_reasons.extend(loop_trend_warning_reasons(
            descriptor,
            previous_entry,
            percent,
            token_savings,
        ));
    }
    if !confidence_met {
        warning_reasons.extend(loop_floor_warning_reasons(
            descriptor,
            percent,
            token_savings,
            evidence.len(),
        ));
    }
    let status = if loop_success_requires_second_signal(
        confidence_met,
        secondary_signal_ok,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        format!(
            "memory hygiene score: {duplicates} duplicates across {total_records} records, {event_spine_entries} event spine entries"
        ),
        vec!["memory hygiene".to_string()],
        serde_json::json!({
            "duplicates": duplicates as f64,
            "records": total_records,
            "evidence": evidence,
            "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, 3),
            "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
            "warning_reasons": warning_reasons,
        }),
        status,
    ))
}

pub(crate) async fn run_autonomy_quality_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let snapshot = read_bundle_resume(&autoresearch_resume_args(output), base_url).await?;
    let mut warning_pressure = 0u64;
    if snapshot.change_summary.is_empty() && snapshot.recent_repo_changes.is_empty() {
        warning_pressure += 1;
    }
    let percent = (100.0 - warning_pressure as f64 * 20.0).max(0.0);
    let token_savings = descriptor.base_tokens * (percent / 100.0);
    let evidence = vec![
        format!("warning_pressure={warning_pressure}"),
        format!("change_summary={}", snapshot.change_summary.len()),
        format!("recent_repo_changes={}", snapshot.recent_repo_changes.len()),
    ];
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let secondary_signal_ok = warning_pressure == 0;
    let mut warning_reasons = Vec::new();
    if snapshot.refresh_recommended {
        warning_reasons.push("refresh_recommended".to_string());
    }
    if snapshot.change_summary.is_empty() && snapshot.recent_repo_changes.is_empty() {
        warning_reasons.push("no_change_signal".to_string());
    }
    if loop_is_regressed(descriptor, previous_entry, percent, token_savings) {
        warning_reasons.extend(loop_trend_warning_reasons(
            descriptor,
            previous_entry,
            percent,
            token_savings,
        ));
    }
    if !confidence_met {
        warning_reasons.extend(loop_floor_warning_reasons(
            descriptor,
            percent,
            token_savings,
            evidence.len(),
        ));
    }
    let status = if loop_success_requires_second_signal(
        confidence_met,
        secondary_signal_ok,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        format!("autonomy quality score: warning pressure {warning_pressure}"),
        vec!["autonomy quality".to_string()],
        serde_json::json!({
            "warning_pressure": warning_pressure,
            "evidence": evidence,
            "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, 3),
            "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
            "warning_reasons": warning_reasons,
        }),
        status,
    ))
}

pub(crate) async fn run_live_truth_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let snapshot = read_bundle_resume(
        &autoresearch_resume_args_with_limits(output, 4, 2, true),
        base_url,
    )
    .await?;
    let change_count = snapshot.change_summary.len() as f64;
    let baseline = 6.0;
    let percent = improvement_less_is_better(change_count, baseline);
    let token_savings = (baseline - change_count).max(0.0) * 20.0;
    let summary = format!(
        "{} change_summary entries, {} repo changes since last resume",
        change_count,
        snapshot.recent_repo_changes.len()
    );
    let evidence = vec![
        "live truth".to_string(),
        format!("change_summary={}", change_count),
        format!("recent_repo_changes={}", snapshot.recent_repo_changes.len()),
    ];
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let warning_reasons = {
        let mut reasons = Vec::new();
        if snapshot.refresh_recommended {
            reasons.push("refresh_recommended".to_string());
        }
        if snapshot.change_summary.is_empty() && snapshot.recent_repo_changes.is_empty() {
            reasons.push("no_change_signal".to_string());
        }
        if loop_is_regressed(descriptor, previous_entry, percent, token_savings) {
            reasons.extend(loop_trend_warning_reasons(
                descriptor,
                previous_entry,
                percent,
                token_savings,
            ));
        }
        if !confidence_met {
            reasons.extend(loop_floor_warning_reasons(
                descriptor,
                percent,
                token_savings,
                evidence.len(),
            ));
        }
        reasons
    };
    let metadata = serde_json::json!({
        "change_summary": change_count,
        "recent_repo_changes": snapshot.recent_repo_changes.len(),
        "refresh_recommended": snapshot.refresh_recommended,
        "evidence": evidence,
        "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, 3),
        "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
        "warning_reasons": warning_reasons,
    });
    let status = if (snapshot.change_summary.is_empty() && snapshot.recent_repo_changes.is_empty())
        || loop_is_regressed(descriptor, previous_entry, percent, token_savings)
        || !confidence_met
    {
        "warning"
    } else {
        "success"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        summary,
        vec!["live truth".to_string()],
        metadata,
        status,
    ))
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) async fn run_event_spine_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let snapshot = read_bundle_resume(
        &autoresearch_resume_args_with_limits(output, 4, 2, true),
        base_url,
    )
    .await?;
    let spine = snapshot.event_spine();
    let spine_chars = spine.iter().map(|line| line.len()).sum::<usize>() as f64;
    let baseline = 600.0;
    let percent = improvement_less_is_better(spine_chars, baseline);
    let token_savings = (baseline - spine_chars).max(0.0) / 4.0;
    let summary = format!(
        "{} event spine entries consuming {} chars",
        spine.len(),
        spine_chars
    );
    let evidence = vec![
        "event spine".to_string(),
        format!("entries={}", spine.len()),
        format!("chars={}", spine_chars),
    ];
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let warning_reasons = {
        let mut reasons = Vec::new();
        if snapshot.refresh_recommended {
            reasons.push("refresh_recommended".to_string());
        }
        if spine.is_empty() {
            reasons.push("empty_event_spine".to_string());
        }
        if loop_is_regressed(descriptor, previous_entry, percent, token_savings) {
            reasons.extend(loop_trend_warning_reasons(
                descriptor,
                previous_entry,
                percent,
                token_savings,
            ));
        }
        if !confidence_met {
            reasons.extend(loop_floor_warning_reasons(
                descriptor,
                percent,
                token_savings,
                evidence.len(),
            ));
        }
        reasons
    };
    let metadata = serde_json::json!({
        "event_spine_entries": spine.len(),
        "event_spine_chars": spine_chars,
        "refresh_recommended": snapshot.refresh_recommended,
        "evidence": evidence,
        "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, 3),
        "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
        "warning_reasons": warning_reasons,
    });
    let status = if spine.is_empty()
        || loop_is_regressed(descriptor, previous_entry, percent, token_savings)
        || !confidence_met
    {
        "warning"
    } else {
        "success"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        summary,
        vec!["event spine".to_string()],
        metadata,
        status,
    ))
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) async fn run_correction_learning_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    run_repair_rate_loop(output, base_url, descriptor, previous_runs, previous_entry).await
}

pub(crate) async fn run_repair_rate_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let snapshot = read_bundle_resume(&autoresearch_resume_args(output), base_url).await?;
    let total = snapshot.change_summary.len() as f64;
    let corrections = snapshot
        .change_summary
        .iter()
        .filter(|line| {
            let lower = line.to_lowercase();
            lower.contains("fix") || lower.contains("correct") || lower.contains("repair")
        })
        .count() as f64;
    let percent = if total == 0.0 {
        0.0
    } else {
        (1.0 - (corrections / total)).max(0.0) * 100.0
    };
    let token_savings = ((total - corrections).max(0.0)) * 10.0;
    let evidence = vec![
        format!("tracked={}", total),
        format!("corrections={}", corrections),
        format!("recent={}", snapshot.recent_repo_changes.len()),
    ];
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let status = if loop_success_requires_second_signal(
        confidence_met,
        corrections <= total,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        format!(
            "{} corrections out of {} tracked change summaries",
            corrections, total
        ),
        vec!["repair rate".to_string()],
        serde_json::json!({
            "evidence": evidence,
            "corrections": corrections,
            "change_summary": total,
            "recent": snapshot.recent_repo_changes.len(),
            "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, 3),
            "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
        }),
        status,
    ))
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) async fn run_long_context_loop(
    output: &Path,
    base_url: &str,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let snapshot =
        read_bundle_resume(&autoresearch_long_context_resume_args(output), base_url).await?;
    let tokens = snapshot.core_prompt_tokens() as f64;
    let baseline = 1_200.0;
    let percent = improvement_less_is_better(tokens, baseline);
    let token_savings = (baseline - tokens).max(0.0);
    let summary = format!("core prompt tokens {} (target {})", tokens, baseline);
    let evidence = vec![
        "long context".to_string(),
        format!("core_prompt_tokens={}", tokens),
        format!("context_records={}", snapshot.context.records.len()),
    ];
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let metadata = serde_json::json!({
        "core_prompt_tokens": tokens,
        "baseline": baseline,
        "context_records": snapshot.context.records.len(),
        "refresh_recommended": snapshot.refresh_recommended,
        "evidence": evidence,
        "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, 3),
        "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
    });
    let status = if tokens <= 0.0
        || loop_is_regressed(descriptor, previous_entry, percent, token_savings)
        || !confidence_met
    {
        "warning"
    } else {
        "success"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        summary,
        vec!["long context".to_string()],
        metadata,
        status,
    ))
}

pub(crate) async fn run_docs_spec_drift_loop(
    output: &Path,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let project_root = infer_bundle_project_root(output).unwrap_or_else(|| output.to_path_buf());
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let spec_path = project_root.join("docs/superpowers/specs/2026-04-08-memd-10-loop-design.md");
    let plan_path = project_root.join("docs/superpowers/plans/2026-04-08-memd-10-loop-stack.md");
    let spec = fs::read_to_string(&spec_path).or_else(|_| {
        fs::read_to_string(
            manifest_root.join("docs/superpowers/specs/2026-04-08-memd-10-loop-design.md"),
        )
    })?;
    let plan = fs::read_to_string(&plan_path).or_else(|_| {
        fs::read_to_string(
            manifest_root.join("docs/superpowers/plans/2026-04-08-memd-10-loop-stack.md"),
        )
    })?;
    let runtime_bytes = fs::metadata(project_root.join("Cargo.toml"))
        .map(|m| m.len())
        .unwrap_or(0);
    let evidence = vec![
        format!("spec_bytes={}", spec.len()),
        format!("plan_bytes={}", plan.len()),
        format!("runtime_bytes={}", runtime_bytes),
    ];
    let secondary_signal_ok = spec.contains("10-loop") && plan.contains("Implementation Plan");
    let percent = if secondary_signal_ok { 100.0 } else { 0.0 };
    let token_savings = if secondary_signal_ok {
        descriptor.base_tokens
    } else {
        0.0
    };
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let status = if loop_success_requires_second_signal(
        confidence_met,
        secondary_signal_ok,
        confidence_met,
        !loop_is_regressed(descriptor, previous_entry, percent, token_savings),
    ) {
        "success"
    } else {
        "warning"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        "docs-spec drift score".to_string(),
        vec!["docs spec drift".to_string()],
        serde_json::json!({
            "evidence": evidence,
            "spec_has_10_loop": spec.contains("10-loop"),
            "plan_has_implementation_plan": plan.contains("Implementation Plan"),
        }),
        status,
    ))
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) async fn run_capability_contract_loop(
    output: &Path,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let project_root = infer_bundle_project_root(output);
    let registry = build_bundle_capability_registry(project_root.as_deref());
    let total = registry.capabilities.len();
    if total == 0 {
        let summary = "no capability contracts discovered".to_string();
        let metadata = serde_json::json!({
            "total": total,
            "missing": 0,
            "coverage": 0.0,
            "evidence": [
                "capability contract",
                "no capability contracts discovered",
            ],
            "confidence": {
                "absolute_percent_floor": descriptor.base_percent,
                "absolute_token_floor": descriptor.base_tokens,
                "min_evidence_signals": AUTORESEARCH_MIN_EVIDENCE_SIGNALS,
                "evidence_count": 0,
                "absolute_percent_met": false,
                "absolute_token_met": false,
                "absolute_floor_met": false,
            },
            "warning_reasons": ["no_capability_registry"],
        });
        return Ok(build_autoresearch_record_with_status(
            descriptor,
            previous_runs + 1,
            0.0,
            0.0,
            summary,
            vec!["capability contract".to_string()],
            metadata,
            "warning",
        ));
    }
    let discovered = registry
        .capabilities
        .iter()
        .filter(|entry| entry.status == "installed" || entry.status == "discovered")
        .count();
    let portable = registry
        .capabilities
        .iter()
        .filter(|entry| {
            entry.portability_class != "adapter-required"
                && entry.portability_class != "harness-native"
        })
        .count();
    let bad = total.saturating_sub(portable);
    let coverage = portable as f64 / total as f64;
    let percent = coverage * 100.0;
    let token_savings = portable as f64;
    let summary = format!(
        "{}/{} capability contracts satisfy expectations",
        portable, total
    );
    let evidence = vec![
        "capability contract".to_string(),
        format!("total={}", total),
        format!("discovered={}", discovered),
        format!("portable={}", portable),
        format!("coverage={}", coverage),
    ];
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let warning_reasons = {
        let mut reasons = Vec::new();
        if loop_is_regressed(descriptor, previous_entry, percent, token_savings) {
            reasons.extend(loop_trend_warning_reasons(
                descriptor,
                previous_entry,
                percent,
                token_savings,
            ));
        }
        if !confidence_met {
            reasons.extend(loop_floor_warning_reasons(
                descriptor,
                percent,
                token_savings,
                evidence.len(),
            ));
        }
        reasons
    };
    let metadata = serde_json::json!({
        "total": total,
        "discovered": discovered,
        "portable": portable,
        "missing": bad,
        "coverage": coverage,
        "evidence": evidence,
        "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, 5),
        "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
        "warning_reasons": warning_reasons,
    });
    let status = if loop_is_regressed(descriptor, previous_entry, percent, token_savings)
        || !confidence_met
    {
        "warning"
    } else {
        "success"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        summary,
        vec!["capability contract".to_string()],
        metadata,
        status,
    ))
}

pub(crate) async fn run_cross_harness_loop(
    output: &Path,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let project_root = infer_bundle_project_root(output);
    let registry = build_bundle_capability_registry(project_root.as_deref());
    let total = registry.capabilities.len();
    if total == 0 {
        let summary = "no cross-harness capabilities discovered".to_string();
        let metadata = serde_json::json!({
            "total": total,
            "portable": 0,
            "ratio": 0.0,
            "evidence": [
                "cross harness",
                "no cross-harness capabilities discovered",
            ],
            "confidence": {
                "absolute_percent_floor": descriptor.base_percent,
                "absolute_token_floor": descriptor.base_tokens,
                "min_evidence_signals": AUTORESEARCH_MIN_EVIDENCE_SIGNALS,
                "evidence_count": 0,
                "absolute_percent_met": false,
                "absolute_token_met": false,
                "absolute_floor_met": false,
            },
            "warning_reasons": ["no_cross_harness_registry"],
        });
        return Ok(build_autoresearch_record_with_status(
            descriptor,
            previous_runs + 1,
            0.0,
            0.0,
            summary,
            vec!["cross harness".to_string()],
            metadata,
            "warning",
        ));
    }
    let portable = registry
        .capabilities
        .iter()
        .filter(|entry| entry.portability_class != "adapter-required")
        .count();
    let ratio = portable as f64 / total as f64;
    let percent = ratio * 100.0;
    let token_savings = ratio * descriptor.base_tokens;
    let summary = format!("cross harness ports {}/{}", portable, total);
    let evidence = vec![
        "cross harness".to_string(),
        format!("total={}", total),
        format!("portable={}", portable),
        format!("ratio={}", ratio),
    ];
    let confidence_met =
        loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
    let warning_reasons = {
        let mut reasons = Vec::new();
        if loop_is_regressed(descriptor, previous_entry, percent, token_savings) {
            reasons.extend(loop_trend_warning_reasons(
                descriptor,
                previous_entry,
                percent,
                token_savings,
            ));
        }
        if !confidence_met {
            reasons.extend(loop_floor_warning_reasons(
                descriptor,
                percent,
                token_savings,
                evidence.len(),
            ));
        }
        reasons
    };
    let metadata = serde_json::json!({
        "total": total,
        "portable": portable,
        "ratio": ratio,
        "evidence": evidence,
        "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, 4),
        "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
        "warning_reasons": warning_reasons,
    });
    let status = if loop_is_regressed(descriptor, previous_entry, percent, token_savings)
        || !confidence_met
    {
        "warning"
    } else {
        "success"
    };
    Ok(build_autoresearch_record_with_status(
        descriptor,
        previous_runs + 1,
        percent,
        token_savings,
        summary,
        vec!["cross harness".to_string()],
        metadata,
        status,
    ))
}

pub(crate) async fn run_self_evolution_loop(
    output: &Path,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
    previous_entry: Option<&LoopSummaryEntry>,
) -> anyhow::Result<LoopRecord> {
    let report = read_latest_experiment_report(output)?;
    if let Some(report) = report {
        ensure_evolution_artifacts(output, &report)?;
        let proposal = read_latest_evolution_proposal(output)?;
        let durability = read_evolution_durability_ledger(output)?;
        let authority = read_evolution_authority_ledger(output)?;
        let branch_manifest = read_latest_evolution_branch_manifest(output)?;
        let merge_queue = read_evolution_merge_queue(output)?;
        let durability_queue = read_evolution_durability_queue(output)?;
        let fresh = experiment_report_is_fresh(&report);
        let proposal_state = proposal
            .as_ref()
            .map(|value| value.state.as_str())
            .unwrap_or("none");
        let scope_class = proposal
            .as_ref()
            .map(|value| value.scope_class.as_str())
            .unwrap_or("none");
        let scope_gate = proposal
            .as_ref()
            .map(|value| value.scope_gate.as_str())
            .unwrap_or("none");
        let authority_tier = proposal
            .as_ref()
            .map(|value| value.authority_tier.as_str())
            .or_else(|| {
                authority
                    .as_ref()
                    .and_then(|ledger| ledger.entries.last())
                    .map(|entry| entry.authority_tier.as_str())
            })
            .unwrap_or("none");
        let merge_status = merge_queue
            .as_ref()
            .and_then(|queue| queue.entries.last())
            .map(|entry| entry.status.as_str())
            .unwrap_or("none");
        let durability_status = durability_queue
            .as_ref()
            .and_then(|queue| queue.entries.last())
            .map(|entry| entry.status.as_str())
            .unwrap_or("none");
        let branch = proposal
            .as_ref()
            .map(|value| value.branch.as_str())
            .or_else(|| {
                branch_manifest
                    .as_ref()
                    .map(|manifest| manifest.branch.as_str())
            })
            .unwrap_or("none");
        let durable_truth = proposal.as_ref().is_some_and(|value| value.durable_truth)
            || durability
                .as_ref()
                .and_then(|ledger| ledger.entries.last())
                .is_some_and(|entry| entry.state == "durable_truth");
        let stage_multiplier = if durable_truth {
            1.0
        } else if proposal_state == "merged" {
            0.92
        } else if proposal_state == "accepted_proposal" {
            0.84
        } else {
            0.0
        };
        let usable = fresh
            && report.accepted
            && !report.restored
            && report.composite.max_score > 0
            && proposal_state != "rejected";
        let raw_ratio = if report.composite.max_score == 0 {
            0.0
        } else {
            report.composite.score as f64 / report.composite.max_score as f64
        };
        let ratio = raw_ratio * stage_multiplier;
        let percent = if usable { ratio * 100.0 } else { 0.0 };
        let token_savings = if usable && stage_multiplier > 0.0 {
            raw_ratio * descriptor.base_tokens
        } else {
            0.0
        };
        let summary = if usable {
            format!(
                "{} experiment composite score {}/{} with {} learnings",
                proposal_state,
                report.composite.score,
                report.composite.max_score,
                report.learnings.len()
            )
        } else {
            format!(
                "experiment report not usable (accepted={}, restored={}, fresh={}, max_score={}, proposal_state={}, scope_gate={})",
                report.accepted,
                report.restored,
                fresh,
                report.composite.max_score,
                proposal_state,
                scope_gate
            )
        };
        let evidence = if usable {
            vec![
                "self evolution".to_string(),
                format!("accepted={}", report.accepted),
                format!("fresh={}", fresh),
                format!("proposal_state={proposal_state}"),
                format!("scope_gate={scope_gate}"),
                format!("durable_truth={durable_truth}"),
                format!("composite_score={}", report.composite.score),
                format!("composite_max={}", report.composite.max_score),
            ]
        } else {
            vec![
                "self evolution".to_string(),
                format!("accepted={}", report.accepted),
                format!("restored={}", report.restored),
                format!("fresh={}", fresh),
                format!("proposal_state={proposal_state}"),
                format!("scope_gate={scope_gate}"),
            ]
        };
        let confidence_met =
            usable && loop_meets_absolute_floor(descriptor, percent, token_savings, evidence.len());
        let warning_reasons = {
            let mut reasons = Vec::new();
            if !fresh {
                reasons.push("stale_report".to_string());
            }
            if report.restored {
                reasons.push("restored_report".to_string());
            }
            if !report.accepted {
                reasons.push("unaccepted_report".to_string());
            }
            if report.composite.max_score == 0 {
                reasons.push("zero_max_score".to_string());
            }
            if proposal_state == "none" {
                reasons.push("no_evolution_proposal".to_string());
            }
            if usable && loop_is_regressed(descriptor, previous_entry, percent, token_savings) {
                reasons.extend(loop_trend_warning_reasons(
                    descriptor,
                    previous_entry,
                    percent,
                    token_savings,
                ));
            }
            if usable && !confidence_met {
                reasons.extend(loop_floor_warning_reasons(
                    descriptor,
                    percent,
                    token_savings,
                    evidence.len(),
                ));
            }
            reasons
        };
        let metadata = serde_json::json!({
            "accepted": report.accepted,
            "restored": report.restored,
            "fresh": fresh,
            "usable": usable,
            "proposal_state": proposal_state,
            "scope_class": scope_class,
            "scope_gate": scope_gate,
            "authority_tier": authority_tier,
            "merge_status": merge_status,
            "durability_status": durability_status,
            "branch": branch,
            "durable_truth": durable_truth,
            "composite_score": report.composite.score,
            "composite_max": report.composite.max_score,
            "evidence": evidence,
            "confidence": loop_confidence_metadata(descriptor, percent, token_savings, confidence_met, if usable { 5 } else { 4 }),
            "trend": loop_trend_metadata(descriptor, previous_entry, percent, token_savings),
            "proposal": proposal,
            "branch_manifest": branch_manifest,
            "authority_ledger": authority,
            "merge_queue": merge_queue,
            "durability_ledger": durability,
            "durability_queue": durability_queue,
            "warning_reasons": warning_reasons,
        });
        let artifact_paths = vec![
            output
                .join("experiments")
                .join("latest.json")
                .display()
                .to_string(),
            output
                .join("evolution")
                .join("latest-proposal.json")
                .display()
                .to_string(),
            output
                .join("evolution")
                .join("latest-branch.json")
                .display()
                .to_string(),
            output
                .join("evolution")
                .join("authority-ledger.json")
                .display()
                .to_string(),
            output
                .join("evolution")
                .join("merge-queue.json")
                .display()
                .to_string(),
            output
                .join("evolution")
                .join("durability-queue.json")
                .display()
                .to_string(),
        ];
        if usable {
            let status = if loop_is_regressed(descriptor, previous_entry, percent, token_savings)
                || !confidence_met
            {
                "warning"
            } else {
                "success"
            };
            Ok(build_autoresearch_record_with_status(
                descriptor,
                previous_runs + 1,
                percent,
                token_savings,
                summary,
                artifact_paths.clone(),
                metadata,
                status,
            ))
        } else {
            Ok(build_autoresearch_record_with_status(
                descriptor,
                previous_runs + 1,
                percent,
                token_savings,
                summary,
                artifact_paths,
                metadata,
                "warning",
            ))
        }
    } else {
        let summary = "no experiment recorded yet".to_string();
        Ok(build_autoresearch_record_with_status(
            descriptor,
            previous_runs + 1,
            0.0,
            0.0,
            summary,
            vec!["self evolution".to_string()],
            serde_json::json!({
                "evidence": ["self evolution", "no experiment recorded yet"],
                "confidence": {
                    "absolute_percent_floor": descriptor.base_percent,
                    "absolute_token_floor": descriptor.base_tokens,
                    "min_evidence_signals": AUTORESEARCH_MIN_EVIDENCE_SIGNALS,
                    "evidence_count": 0,
                    "absolute_percent_met": false,
                    "absolute_token_met": false,
                    "absolute_floor_met": false,
                },
                "warning_reasons": ["no_experiment_report"],
            }),
            "warning",
        ))
    }
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) async fn run_default_loop(
    _output: &Path,
    descriptor: &AutoresearchLoop,
    previous_runs: usize,
) -> anyhow::Result<LoopRecord> {
    let percent_improvement =
        descriptor.base_percent + (previous_runs as f64 * descriptor.base_percent * 0.1);
    let token_savings =
        descriptor.base_tokens + (previous_runs as f64 * descriptor.base_tokens * 0.1);
    Ok(build_autoresearch_record(
        descriptor,
        previous_runs + 1,
        percent_improvement,
        token_savings,
        descriptor.description.to_string(),
        vec!["autoresearch loop".to_string()],
        serde_json::json!({
            "target": descriptor.target,
            "metric": descriptor.metric,
        }),
    ))
}

#[cfg(test)]
pub(crate) fn build_autoresearch_record(
    descriptor: &AutoresearchLoop,
    iteration: usize,
    percent_improvement: f64,
    token_savings: f64,
    summary: String,
    artifacts: Vec<String>,
    metadata: serde_json::Value,
) -> LoopRecord {
    build_autoresearch_record_with_status(
        descriptor,
        iteration,
        percent_improvement,
        token_savings,
        summary,
        artifacts,
        metadata,
        "success",
    )
}

pub(crate) fn build_autoresearch_record_with_status(
    descriptor: &AutoresearchLoop,
    iteration: usize,
    percent_improvement: f64,
    token_savings: f64,
    summary: String,
    artifacts: Vec<String>,
    metadata: serde_json::Value,
    status: &str,
) -> LoopRecord {
    LoopRecord {
        slug: Some(descriptor.slug.to_string()),
        name: Some(descriptor.name.to_string()),
        iteration: Some(iteration as u32),
        percent_improvement: Some(percent_improvement.clamp(0.0, 100.0)),
        token_savings: Some(token_savings.max(0.0)),
        status: Some(status.to_string()),
        summary: Some(summary),
        artifacts: Some(artifacts),
        created_at: Some(Utc::now()),
        metadata,
    }
}

const AUTORESEARCH_MIN_EVIDENCE_SIGNALS: usize = 3;

pub(crate) fn loop_meets_absolute_floor(
    descriptor: &AutoresearchLoop,
    percent: f64,
    token_savings: f64,
    evidence_count: usize,
) -> bool {
    percent >= descriptor.base_percent
        && token_savings >= descriptor.base_tokens
        && evidence_count >= AUTORESEARCH_MIN_EVIDENCE_SIGNALS
}

pub(crate) fn loop_success_requires_second_signal(
    primary_ok: bool,
    secondary_ok: bool,
    confidence_ok: bool,
    trend_ok: bool,
) -> bool {
    primary_ok && secondary_ok && confidence_ok && trend_ok
}

pub(crate) fn loop_confidence_metadata(
    descriptor: &AutoresearchLoop,
    percent: f64,
    token_savings: f64,
    absolute_floor_met: bool,
    evidence_count: usize,
) -> serde_json::Value {
    serde_json::json!({
        "absolute_percent_floor": descriptor.base_percent,
        "absolute_token_floor": descriptor.base_tokens,
        "min_evidence_signals": AUTORESEARCH_MIN_EVIDENCE_SIGNALS,
        "evidence_count": evidence_count,
        "absolute_percent_met": percent >= descriptor.base_percent,
        "absolute_token_met": token_savings >= descriptor.base_tokens,
        "absolute_floor_met": absolute_floor_met,
    })
}

pub(crate) fn loop_floor_warning_reasons(
    descriptor: &AutoresearchLoop,
    percent: f64,
    token_savings: f64,
    evidence_count: usize,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if percent < descriptor.base_percent {
        reasons.push("percent_below_floor".to_string());
    }
    if token_savings < descriptor.base_tokens {
        reasons.push("token_savings_below_floor".to_string());
    }
    if evidence_count < AUTORESEARCH_MIN_EVIDENCE_SIGNALS {
        reasons.push("evidence_count_below_floor".to_string());
    }
    reasons
}

const AUTORESEARCH_EXPERIMENT_MAX_AGE_HOURS: i64 = 24;

pub(crate) fn experiment_report_is_fresh(report: &ExperimentReport) -> bool {
    let age = Utc::now()
        .signed_duration_since(report.completed_at)
        .num_hours();
    age <= AUTORESEARCH_EXPERIMENT_MAX_AGE_HOURS
}

pub(crate) fn loop_is_regressed(
    descriptor: &AutoresearchLoop,
    previous_entry: Option<&LoopSummaryEntry>,
    percent: f64,
    token_savings: f64,
) -> bool {
    loop_trend_warning_reasons(descriptor, previous_entry, percent, token_savings)
        .iter()
        .any(|reason| reason.starts_with("trend_"))
}

pub(crate) fn loop_trend_metadata(
    descriptor: &AutoresearchLoop,
    previous_entry: Option<&LoopSummaryEntry>,
    percent: f64,
    token_savings: f64,
) -> serde_json::Value {
    match previous_entry {
        Some(previous) => serde_json::json!({
            "previous_percent": previous.percent_improvement,
            "previous_token_savings": previous.token_savings,
            "trend_percent_floor": descriptor.trend_percent_floor,
            "trend_token_floor": descriptor.trend_token_floor,
            "regressed": loop_is_regressed(descriptor, previous_entry, percent, token_savings),
            "warning_reasons": loop_trend_warning_reasons(
                descriptor,
                previous_entry,
                percent,
                token_savings,
            ),
        }),
        None => serde_json::json!({
            "previous_percent": serde_json::Value::Null,
            "previous_token_savings": serde_json::Value::Null,
            "trend_percent_floor": descriptor.trend_percent_floor,
            "trend_token_floor": descriptor.trend_token_floor,
            "regressed": false,
            "warning_reasons": Vec::<String>::new(),
        }),
    }
}

pub(crate) fn loop_trend_warning_reasons(
    descriptor: &AutoresearchLoop,
    previous_entry: Option<&LoopSummaryEntry>,
    percent: f64,
    token_savings: f64,
) -> Vec<String> {
    let Some(previous) = previous_entry else {
        return Vec::new();
    };
    let previous_percent = previous.percent_improvement.unwrap_or(0.0);
    let previous_tokens = previous.token_savings.unwrap_or(0.0);
    let mut reasons = Vec::new();
    if percent + descriptor.trend_percent_floor <= previous_percent {
        reasons.push("trend_percent_regressed".to_string());
    }
    if token_savings + descriptor.trend_token_floor <= previous_tokens {
        reasons.push("trend_token_regressed".to_string());
    }
    reasons
}

pub(crate) fn improvement_less_is_better(measured: f64, baseline: f64) -> f64 {
    if baseline <= 0.0 {
        return 0.0;
    }
    ((baseline - measured).max(0.0) / baseline) * 100.0
}

pub(crate) fn autoresearch_resume_args(output: &Path) -> ResumeArgs {
    autoresearch_resume_args_with_limits(output, 8, 4, true)
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn autoresearch_long_context_resume_args(output: &Path) -> ResumeArgs {
    autoresearch_resume_args_with_limits(output, 0, 0, false)
}

pub(crate) fn autoresearch_resume_args_with_limits(
    output: &Path,
    limit: usize,
    rehydration_limit: usize,
    semantic: bool,
) -> ResumeArgs {
    ResumeArgs {
        output: output.to_path_buf(),
        project: None,
        namespace: None,
        agent: None,
        workspace: None,
        visibility: None,
        route: None,
        intent: None,
        limit: Some(limit),
        rehydration_limit: Some(rehydration_limit),
        semantic,
        prompt: false,
        summary: false,
    }
}

pub(crate) fn read_latest_experiment_report(output: &Path) -> anyhow::Result<Option<ExperimentReport>> {
    let path = experiment_reports_dir(output).join("latest.json");
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let report = serde_json::from_str::<ExperimentReport>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(report))
}

pub(crate) struct AutoresearchLoop {
    pub(crate) slug: &'static str,
    pub(crate) normalized_slug: &'static str,
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
    pub(crate) target: &'static str,
    pub(crate) metric: &'static str,
    pub(crate) stop_condition: &'static str,
    pub(crate) risk: &'static str,
    pub(crate) base_percent: f64,
    pub(crate) base_tokens: f64,
    pub(crate) trend_percent_floor: f64,
    pub(crate) trend_token_floor: f64,
}

impl AutoresearchLoop {
    #[allow(clippy::too_many_arguments)]
    const fn new(
        slug: &'static str,
        normalized_slug: &'static str,
        name: &'static str,
        description: &'static str,
        target: &'static str,
        metric: &'static str,
        stop_condition: &'static str,
        risk: &'static str,
        base_percent: f64,
        base_tokens: f64,
        trend_percent_floor: f64,
        trend_token_floor: f64,
    ) -> AutoresearchLoop {
        AutoresearchLoop {
            slug,
            normalized_slug,
            name,
            description,
            target,
            metric,
            stop_condition,
            risk,
            base_percent,
            base_tokens,
            trend_percent_floor,
            trend_token_floor,
        }
    }
}

pub(crate) static AUTORESEARCH_LOOPS: [AutoresearchLoop; 10] = [
    AutoresearchLoop::new(
        "hive-health",
        "hive-health",
        "Hive Health",
        "Keep live sessions, heartbeat publication, and claim collisions healthy.",
        "live sessions / claims",
        "dead sessions / collisions",
        "no dead sessions",
        "low",
        1.0,
        40.0,
        0.5,
        4.0,
    ),
    AutoresearchLoop::new(
        "memory-hygiene",
        "memory-hygiene",
        "Memory Hygiene",
        "Track stale memories, duplicate memories, orphaned entries, and compression wins.",
        "duplicate memories",
        "stale / duplicate memory pressure",
        "low duplicate pressure",
        "medium",
        1.2,
        80.0,
        0.5,
        4.0,
    ),
    AutoresearchLoop::new(
        "autonomy-quality",
        "autonomy-quality",
        "Autonomy Quality",
        "Track false-green rate, warning rate, and real delta versus noise.",
        "false-green rate",
        "warning / noise pressure",
        "false-green pressure low",
        "high",
        0.9,
        60.0,
        1.0,
        6.0,
    ),
    AutoresearchLoop::new(
        "prompt-efficiency",
        "prompt-efficiency",
        "Prompt Efficiency",
        "Track prompt token burn, reuse rate, and bundle shrink.",
        "prompt token burn",
        "reuse / shrink pressure",
        "prompt burn stays low",
        "low",
        2.5,
        50.0,
        0.5,
        8.0,
    ),
    AutoresearchLoop::new(
        "repair-rate",
        "repair-rate",
        "Repair Rate",
        "Track how often the system fixes real problems instead of churning on superficial changes.",
        "repair recurrence",
        "real repairs vs churn",
        "repair rate stays high",
        "medium",
        1.0,
        60.0,
        0.5,
        4.0,
    ),
    AutoresearchLoop::new(
        "signal-freshness",
        "signal-freshness",
        "Signal Freshness",
        "Track stale snapshot rate, live-truth drift, and refresh pressure.",
        "live-truth freshness",
        "stale snapshot rate",
        "freshness baseline met",
        "low-medium",
        1.6,
        40.0,
        0.5,
        6.0,
    ),
    AutoresearchLoop::new(
        "cross-harness",
        "cross-harness",
        "Cross-Harness Portability",
        "Keep memories and promoted artifacts portable across harnesses.",
        "contract coverage",
        "adapter-required warnings",
        "portability class assigned",
        "medium",
        1.2,
        110.0,
        0.5,
        4.0,
    ),
    AutoresearchLoop::new(
        "self-evolution",
        "self-evolution",
        "Controlled Self-Evolution",
        "Ensure the evolution engine promotes only validated, measurable wins.",
        "accepted-change rate",
        "promotion evidence coverage",
        "confidence threshold reached",
        "high",
        0.7,
        50.0,
        1.0,
        6.0,
    ),
    AutoresearchLoop::new(
        "branch-review-quality",
        "branch-review-quality",
        "Branch Review Quality",
        "Track branch cleanliness, diff quality, and review readiness.",
        "branch cleanliness",
        "dirty branch / review readiness",
        "review ready",
        "medium",
        1.0,
        20.0,
        0.5,
        4.0,
    ),
    AutoresearchLoop::new(
        "docs-spec-drift",
        "docs-spec-drift",
        "Docs Spec Drift",
        "Keep docs and shipped behavior aligned.",
        "docs alignment",
        "spec drift",
        "docs match runtime",
        "medium",
        1.0,
        40.0,
        0.5,
        4.0,
    ),
];
