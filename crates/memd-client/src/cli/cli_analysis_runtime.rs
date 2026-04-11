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
        Some(BenchmarkSubcommand::Public(public_args)) => {
            let response = run_public_benchmark_command(public_args).await?;
            if public_args.write {
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
