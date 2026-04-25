use super::*;

pub(crate) async fn run_eval_command(args: &EvalArgs, base_url: &str) -> anyhow::Result<()> {
    let response = eval_bundle_memory(args, base_url).await?;
    if args.write {
        write_bundle_eval_artifacts(&args.output, &response)?;
    }
    if args.summary {
        println!("{}", render_eval_summary(&response));
    } else {
        print_json(&response)?;
    }
    if let Some(reason) = eval_failure_reason(&response, args.fail_below, args.fail_on_regression) {
        anyhow::bail!(reason);
    }
    Ok(())
}

pub(crate) async fn run_gap_command(args: &GapArgs) -> anyhow::Result<()> {
    let response = gap_report(args).await?;
    if args.write {
        write_gap_artifacts(&args.output, &response)?;
    }
    if args.summary {
        println!("{}", render_gap_summary(&response));
    } else {
        print_json(&response)?;
    }
    Ok(())
}

pub(crate) async fn run_improve_command(args: &ImproveArgs, base_url: &str) -> anyhow::Result<()> {
    let response = run_improvement_loop(args, base_url).await?;
    if args.write {
        write_improvement_artifacts(&args.output, &response)?;
    }
    if args.summary {
        println!("{}", render_improvement_summary(&response));
    } else {
        print_json(&response)?;
    }
    Ok(())
}

pub(crate) async fn run_scenario_command_cli(
    args: &ScenarioArgs,
    base_url: &str,
) -> anyhow::Result<()> {
    let response = run_scenario_command(args, base_url).await?;
    if args.write {
        write_scenario_artifacts(&args.output, &response)?;
    }
    if args.summary {
        println!("{}", render_scenario_summary(&response));
    } else {
        print_json(&response)?;
    }
    Ok(())
}

pub(crate) async fn run_composite_command_cli(
    args: &CompositeArgs,
    base_url: &str,
) -> anyhow::Result<()> {
    let response = run_composite_command(args, base_url).await?;
    if args.write {
        write_composite_artifacts(&args.output, &response)?;
    }
    if args.summary {
        println!("{}", render_composite_summary(&response));
    } else {
        print_json(&response)?;
    }
    Ok(())
}

pub(crate) async fn run_benchmark_command(
    args: &BenchmarkArgs,
    base_url: &str,
) -> anyhow::Result<()> {
    match &args.subcommand {
        Some(BenchmarkSubcommand::Substrate(substrate_args)) => {
            return crate::benchmark::run_substrate_command(substrate_args).await;
        }
        Some(BenchmarkSubcommand::Public(public_args)) => {
            // CI mode: run all benchmarks, hard-fail on threshold breach
            if public_args.ci {
                return run_benchmark_ci_gate(public_args).await;
            }

            let response = run_public_benchmark_command(public_args).await?;
            if public_args.write && !public_args.all {
                let receipt = write_public_benchmark_run_artifacts(&public_args.out, &response)?;
                let _ = (
                    &receipt.run_dir,
                    &receipt.manifest_path,
                    &receipt.results_path,
                    &receipt.results_jsonl_path,
                    &receipt.report_path,
                );
                if let Some(repo_root) = infer_bundle_project_root(&public_args.out) {
                    write_public_benchmark_docs(&repo_root, &public_args.out, &response)?;
                }
            }
            // Record results if --record
            if public_args.record && !public_args.all {
                record_benchmark_run(&public_args.out, &response)?;
            }
            // Threshold gate
            let thresholds_path = public_args.out.join("benchmarks").join("thresholds.json");
            let mode = if response
                .manifest
                .runtime_settings
                .get("full_eval")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
            {
                "full_eval"
            } else {
                "retrieval"
            };
            if !check_benchmark_threshold(
                &response.manifest.benchmark_id,
                mode,
                &response.metrics,
                &thresholds_path,
            )? {
                eprintln!(
                    "WARNING: {} {} below threshold (see {})",
                    response.manifest.benchmark_id,
                    mode,
                    thresholds_path.display()
                );
            }
            // Comparison table (full-eval only)
            if public_args.full_eval {
                let baselines_path = default_baselines_path(&public_args.out);
                if baselines_path.exists() {
                    if let Ok(baselines) = load_published_baselines(&baselines_path) {
                        let (primary_label, primary_value) =
                            public_benchmark_primary_metric(&response);
                        let mut memd_scores = std::collections::BTreeMap::new();
                        memd_scores.insert(response.manifest.benchmark_id.clone(), primary_value);
                        let table = render_comparison_table(&memd_scores, &baselines);
                        eprintln!("\n{table}");
                        let _ = primary_label;
                    }
                }
            }
            if public_args.json {
                print_json(&response)?;
            } else if args.summary {
                println!("{}", render_public_benchmark_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        None => {
            let response = run_feature_benchmark_command(args, base_url).await?;
            if args.write {
                write_feature_benchmark_artifacts(&args.output, &response)?;
                if let Some((repo_root, registry)) =
                    load_benchmark_registry_for_output(&args.output)?
                {
                    write_benchmark_registry_docs(&repo_root, &registry, &response)?;
                }
            }
            if args.summary {
                println!("{}", render_feature_benchmark_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
    }
    Ok(())
}

/// CI gate: hardcoded regression thresholds per benchmark.
/// Returns (benchmark_id, threshold).
fn ci_gate_thresholds() -> Vec<(&'static str, &'static str, f64)> {
    // Metric names must match what the benchmark runtime actually records.
    // longmemeval primary: session_recall_any@5 (also stored as accuracy)
    // locomo primary: accuracy (evidence_hit_rate@5)
    // membench primary: accuracy (target_hit_rate@5)
    vec![
        ("longmemeval", "accuracy", 0.80),
        ("locomo", "accuracy", 0.414),
        ("membench", "accuracy", 0.30),
    ]
}

/// Run all public benchmarks in CI mode. Exit 1 if any drops below threshold.
async fn run_benchmark_ci_gate(base_args: &PublicBenchmarkArgs) -> anyhow::Result<()> {
    let benchmark_ids = implemented_public_benchmark_ids();
    let mut results: Vec<(String, std::collections::BTreeMap<String, f64>, bool)> = Vec::new();
    let thresholds = ci_gate_thresholds();

    for dataset_id in benchmark_ids {
        let mut sub_args = base_args.clone();
        sub_args.dataset = dataset_id.to_string();
        sub_args.all = false;
        sub_args.ci = false;

        eprintln!("[ci-gate] running {dataset_id}…");
        match Box::pin(run_public_benchmark_command(&sub_args)).await {
            Ok(report) => {
                // Record if requested
                if base_args.record {
                    let _ = record_benchmark_run(&base_args.out, &report);
                }

                // Check against hardcoded thresholds
                let mut passed = true;
                for (tid, metric, threshold) in &thresholds {
                    if *tid == *dataset_id {
                        let actual = report.metrics.get(*metric).copied().unwrap_or(0.0);
                        if actual < *threshold {
                            eprintln!(
                                "[ci-gate] FAIL: {dataset_id} {metric} = {actual:.3} < {threshold:.3}"
                            );
                            passed = false;
                        } else {
                            eprintln!(
                                "[ci-gate] PASS: {dataset_id} {metric} = {actual:.3} >= {threshold:.3}"
                            );
                        }
                    }
                }
                results.push((dataset_id.to_string(), report.metrics.clone(), passed));
            }
            Err(err) => {
                eprintln!("[ci-gate] ERROR: {dataset_id} failed: {err}");
                results.push((dataset_id.to_string(), Default::default(), false));
            }
        }
    }

    // Summary table
    eprintln!();
    eprintln!("[ci-gate] Summary:");
    for (id, _metrics, passed) in &results {
        eprintln!("  {} {}", if *passed { "✓" } else { "✗" }, id);
    }

    let any_failed = results.iter().any(|(_, _, passed)| !*passed);
    if any_failed {
        anyhow::bail!("CI gate FAILED: one or more benchmarks below threshold");
    }

    eprintln!("[ci-gate] All benchmarks passed.");
    Ok(())
}

/// Record a benchmark run to the history log (JSON-lines format).
pub(crate) fn record_benchmark_run(
    output: &std::path::Path,
    report: &PublicBenchmarkRunReport,
) -> anyhow::Result<()> {
    let history_dir = output.join("benchmarks").join("history");
    std::fs::create_dir_all(&history_dir)?;
    let history_file = history_dir.join("benchmark-runs.jsonl");

    // Get git SHA
    let git_sha = report
        .manifest
        .git_sha
        .clone()
        .or_else(|| {
            std::process::Command::new("git")
                .args(["rev-parse", "--short", "HEAD"])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    let (primary_label, primary_value) = public_benchmark_primary_metric(report);
    let verification_status = report
        .manifest
        .runtime_settings
        .get("dataset_verification")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(if report.manifest.dataset_source_url.starts_with("http") {
            "recorded-unpinned"
        } else {
            "fixture-only"
        });

    let entry = serde_json::json!({
        "benchmark_id": report.manifest.benchmark_id,
        "git_sha": git_sha,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "primary_metric": primary_label,
        "primary_value": primary_value,
        "verification_status": verification_status,
        "metrics": report.metrics,
        "item_count": report.item_count,
    });

    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&history_file)?;
    writeln!(f, "{}", serde_json::to_string(&entry)?)?;

    eprintln!(
        "[record] appended {} result to {}",
        report.manifest.benchmark_id,
        history_file.display()
    );
    Ok(())
}

pub(crate) async fn run_verify_command(args: &VerifyArgs) -> anyhow::Result<()> {
    match &args.command {
        VerifyCommand::Feature(verify_args) => {
            let response = run_verify_feature_command(verify_args).await?;
            if verify_args.summary {
                println!("{}", render_verify_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        VerifyCommand::Journey(verify_args) => {
            let response = run_verify_journey_command(verify_args).await?;
            if verify_args.summary {
                println!("{}", render_verify_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        VerifyCommand::Adversarial(verify_args) => {
            let response = run_verify_adversarial_command(verify_args).await?;
            if verify_args.summary {
                println!("{}", render_verify_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        VerifyCommand::Compare(verify_args) => {
            let response = run_verify_compare_command(verify_args).await?;
            if verify_args.summary {
                println!("{}", render_verify_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        VerifyCommand::Sweep(verify_args) => {
            let response = run_verify_sweep_command(verify_args).await?;
            if verify_args.summary {
                println!("{}", render_verify_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        VerifyCommand::Doctor(verify_args) => {
            let response = run_verify_doctor_command(verify_args)?;
            if verify_args.summary {
                println!("{}", render_verify_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        VerifyCommand::List(verify_args) => {
            let response = run_verify_list_command(verify_args)?;
            if verify_args.summary {
                println!("{}", render_verify_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
        VerifyCommand::Show(verify_args) => {
            let response = run_verify_show_command(verify_args)?;
            if verify_args.summary {
                println!("{}", render_verify_summary(&response));
            } else {
                print_json(&response)?;
            }
        }
    }
    Ok(())
}

pub(crate) async fn run_experiment_command_cli(
    args: &ExperimentArgs,
    base_url: &str,
) -> anyhow::Result<()> {
    let mut response = run_experiment_command(args, base_url).await?;
    if args.write {
        write_experiment_artifacts(&args.output, &response)?;
        hydrate_experiment_evolution_summary(&mut response, &args.output)?;
    }
    if args.summary {
        println!("{}", render_experiment_summary(&response));
    } else {
        print_json(&response)?;
    }
    Ok(())
}
