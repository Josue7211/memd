use super::*;
use crate::runtime::*;

mod fixtures;
#[allow(unused_imports)]
pub(crate) use fixtures::*;

mod verifier_runtime;
#[allow(unused_imports)]
pub(crate) use verifier_runtime::*;

mod scorecard;
#[allow(unused_imports)]
pub(crate) use scorecard::*;

mod reports;
#[allow(unused_imports)]
pub(crate) use reports::*;

mod verify_runtime;
#[allow(unused_imports)]
pub(crate) use verify_runtime::*;

mod docs;
#[allow(unused_imports)]
pub(crate) use docs::*;

pub(crate) fn write_feature_benchmark_artifacts(
    output: &Path,
    response: &FeatureBenchmarkReport,
) -> anyhow::Result<()> {
    let benchmark_dir = feature_benchmark_reports_dir(output);
    fs::create_dir_all(&benchmark_dir)
        .with_context(|| format!("create {}", benchmark_dir.display()))?;

    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let latest_json = benchmark_dir.join("latest.json");
    let latest_md = benchmark_dir.join("latest.md");
    let timestamp_json = benchmark_dir.join(format!("{timestamp}.json"));
    let timestamp_md = benchmark_dir.join(format!("{timestamp}.md"));
    let json = serde_json::to_string_pretty(response)? + "\n";
    let markdown = render_feature_benchmark_markdown(response);

    fs::write(&latest_json, &json).with_context(|| format!("write {}", latest_json.display()))?;
    fs::write(&latest_md, &markdown).with_context(|| format!("write {}", latest_md.display()))?;
    fs::write(&timestamp_json, &json)
        .with_context(|| format!("write {}", timestamp_json.display()))?;
    fs::write(&timestamp_md, &markdown)
        .with_context(|| format!("write {}", timestamp_md.display()))?;
    Ok(())
}
