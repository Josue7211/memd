use super::*;

#[allow(dead_code)]
mod eval_runtime;
#[allow(unused_imports)]
pub(crate) use eval_runtime::simplify_awareness_work_text;

#[allow(dead_code)]
mod eval_report_runtime;
#[allow(unused_imports)]
pub(crate) use eval_report_runtime::{
    build_eval_recommendations, describe_eval_changes, eval_bundle_memory, eval_failure_reason,
    read_latest_bundle_eval, read_latest_scenario_report,
};

#[allow(dead_code)]
mod views_runtime;
pub(crate) use views_runtime::*;

pub(crate) fn public_benchmark_runs_dir(output: &Path) -> PathBuf {
    output.join("benchmarks").join("public")
}

pub(crate) fn public_benchmark_run_artifacts_dir(output: &Path, benchmark_id: &str) -> PathBuf {
    public_benchmark_runs_dir(output)
        .join(benchmark_id)
        .join("latest")
}

pub(crate) fn gap_reports_dir(output: &Path) -> PathBuf {
    output.join("gaps")
}

pub(crate) fn copy_dir_contents(source: &Path, destination: &Path) -> anyhow::Result<()> {
    if !source.exists() {
        return Ok(());
    }
    fs::create_dir_all(destination).with_context(|| format!("create {}", destination.display()))?;
    for entry in fs::read_dir(source).with_context(|| format!("read {}", source.display()))? {
        let entry = entry.with_context(|| format!("read {}", source.display()))?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry
            .file_type()
            .with_context(|| format!("inspect {}", source_path.display()))?;
        if file_type.is_dir() {
            copy_dir_contents(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            fs::copy(&source_path, &destination_path).with_context(|| {
                format!(
                    "copy {} -> {}",
                    source_path.display(),
                    destination_path.display()
                )
            })?;
        }
    }
    Ok(())
}

pub(crate) fn public_benchmark_manifest_json_path(output: &Path, benchmark_id: &str) -> PathBuf {
    public_benchmark_run_artifacts_dir(output, benchmark_id).join("manifest.json")
}

pub(crate) fn public_benchmark_results_json_path(output: &Path, benchmark_id: &str) -> PathBuf {
    public_benchmark_run_artifacts_dir(output, benchmark_id).join("results.json")
}

pub(crate) fn public_benchmark_results_jsonl_path(output: &Path, benchmark_id: &str) -> PathBuf {
    public_benchmark_run_artifacts_dir(output, benchmark_id).join("results.jsonl")
}

pub(crate) fn public_benchmark_report_md_path(output: &Path, benchmark_id: &str) -> PathBuf {
    public_benchmark_run_artifacts_dir(output, benchmark_id).join("report.md")
}

pub(crate) fn write_public_benchmark_manifest(
    output: &Path,
    manifest: &PublicBenchmarkManifest,
) -> anyhow::Result<PathBuf> {
    let benchmark_dir = public_benchmark_run_artifacts_dir(output, &manifest.benchmark_id);
    fs::create_dir_all(&benchmark_dir)
        .with_context(|| format!("create {}", benchmark_dir.display()))?;

    let latest_json = public_benchmark_manifest_json_path(output, &manifest.benchmark_id);
    let json = serde_json::to_string_pretty(manifest)? + "\n";

    fs::write(&latest_json, &json).with_context(|| format!("write {}", latest_json.display()))?;
    Ok(latest_json)
}

pub(crate) fn write_public_benchmark_run_report(
    output: &Path,
    report: &PublicBenchmarkRunReport,
) -> anyhow::Result<PathBuf> {
    let benchmark_dir = public_benchmark_run_artifacts_dir(output, &report.manifest.benchmark_id);
    fs::create_dir_all(&benchmark_dir)
        .with_context(|| format!("create {}", benchmark_dir.display()))?;

    let results_path = public_benchmark_results_json_path(output, &report.manifest.benchmark_id);
    let results_jsonl_path =
        public_benchmark_results_jsonl_path(output, &report.manifest.benchmark_id);
    let report_path = public_benchmark_report_md_path(output, &report.manifest.benchmark_id);
    let json = serde_json::to_string_pretty(report)? + "\n";
    let jsonl = report
        .items
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()?
        .join("\n")
        + if report.items.is_empty() { "" } else { "\n" };
    let markdown = render_public_benchmark_markdown(std::slice::from_ref(report));

    fs::write(&results_path, &json).with_context(|| format!("write {}", results_path.display()))?;
    fs::write(&results_jsonl_path, &jsonl)
        .with_context(|| format!("write {}", results_jsonl_path.display()))?;
    fs::write(&report_path, &markdown)
        .with_context(|| format!("write {}", report_path.display()))?;
    Ok(report_path)
}

pub(crate) fn write_public_benchmark_run_artifacts(
    output: &Path,
    report: &PublicBenchmarkRunReport,
) -> anyhow::Result<PublicBenchmarkRunArtifactReceipt> {
    let run_dir = public_benchmark_run_artifacts_dir(output, &report.manifest.benchmark_id);
    let manifest_path = write_public_benchmark_manifest(output, &report.manifest)?;
    let results_path = public_benchmark_results_json_path(output, &report.manifest.benchmark_id);
    let results_jsonl_path =
        public_benchmark_results_jsonl_path(output, &report.manifest.benchmark_id);
    let report_path = write_public_benchmark_run_report(output, report)?;
    Ok(PublicBenchmarkRunArtifactReceipt {
        run_dir,
        manifest_path,
        results_path,
        results_jsonl_path,
        report_path,
    })
}

pub(crate) fn clear_dir_contents(path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(path).with_context(|| format!("read {}", path.display()))? {
        let entry = entry.with_context(|| format!("read {}", path.display()))?;
        let entry_path = entry.path();
        let file_type = entry
            .file_type()
            .with_context(|| format!("inspect {}", entry_path.display()))?;
        if file_type.is_dir() {
            fs::remove_dir_all(&entry_path)
                .with_context(|| format!("remove {}", entry_path.display()))?;
        } else {
            fs::remove_file(&entry_path)
                .with_context(|| format!("remove {}", entry_path.display()))?;
        }
    }
    Ok(())
}

pub(crate) fn restore_bundle_snapshot(snapshot_root: &Path, output: &Path) -> anyhow::Result<()> {
    clear_dir_contents(output)?;
    copy_dir_contents(snapshot_root, output)?;
    Ok(())
}

pub(crate) fn derive_experiment_learnings(
    improvement: &ImprovementReport,
    composite: &CompositeReport,
) -> Vec<String> {
    let mut learnings = Vec::new();
    learnings.push(format!(
        "accepted composite score {}/{} after {} improvement iteration(s)",
        composite.score,
        composite.max_score,
        improvement.iterations.len()
    ));
    if let Some(gate) = composite
        .gates
        .iter()
        .find(|gate| gate.name == "acceptance")
    {
        learnings.push(format!("acceptance gate {}", gate.status));
    }
    if let Some(change) = improvement.final_changes.first() {
        learnings.push(format!("final change: {change}"));
    } else if let Some(dimension) = composite.dimensions.first() {
        learnings.push(format!(
            "top composite dimension: {}={}",
            dimension.name, dimension.score
        ));
    }
    learnings.truncate(3);
    learnings
}

pub(crate) fn append_experiment_learning_notes(
    output: &Path,
    learnings: &[String],
    composite: &CompositeReport,
) -> anyhow::Result<()> {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    let note = {
        let mut markdown = String::new();
        markdown.push_str(&format!("\n## Accepted Experiment {}\n\n", timestamp));
        markdown.push_str(&format!(
            "- composite: {}/{}\n",
            composite.score, composite.max_score
        ));
        if let Some(scenario) = composite.scenario.as_deref() {
            markdown.push_str(&format!("- scenario: {scenario}\n"));
        }
        markdown.push_str("- learnings:\n");
        for item in learnings {
            markdown.push_str(&format!("  - {item}\n"));
        }
        markdown.push_str("- note: only accepted experiments should remain in durable memory\n");
        markdown
    };

    append_text_to_memory_surface(&output.join("mem.md"), &note)?;
    Ok(())
}

pub(crate) fn append_text_to_memory_surface(path: &Path, note: &str) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut current = fs::read_to_string(path).unwrap_or_default();
    if current.is_empty() {
        current.push_str("# memd memory\n");
    }
    if !current.ends_with('\n') {
        current.push('\n');
    }
    current.push_str(note.trim_start_matches('\n'));
    if !current.ends_with('\n') {
        current.push('\n');
    }
    fs::write(path, current).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

mod gap_runtime;
pub(crate) use gap_runtime::*;

mod gap_artifacts_runtime;
pub(crate) use gap_artifacts_runtime::*;
