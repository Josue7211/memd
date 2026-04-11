use super::*;

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
