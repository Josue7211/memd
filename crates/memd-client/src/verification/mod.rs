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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_feature_benchmark_artifacts_surfaces_typed_retrieval_probe() {
        let dir = std::env::temp_dir().join(format!(
            "memd-feature-benchmark-artifacts-{}",
            uuid::Uuid::new_v4()
        ));
        let output = dir.join(".memd");
        let report = FeatureBenchmarkReport {
            bundle_root: output.display().to_string(),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            session: Some("codex-a".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            score: 91,
            max_score: 100,
            command_count: 5,
            skill_count: 3,
            pack_count: 4,
            memory_pages: 2,
            event_count: 6,
            areas: vec![FeatureBenchmarkArea {
                slug: "retrieval_context".to_string(),
                name: "Retrieval And Context".to_string(),
                score: 94,
                max_score: 100,
                status: "pass".to_string(),
                implemented_commands: 8,
                expected_commands: 8,
                evidence: vec!["typed_retrieval_probe=session_continuity+candidate".to_string()],
                recommendations: Vec::new(),
            }],
            evidence: vec![
                "memory_quality score=92/100 latency_ms=18".to_string(),
                "typed_retrieval_probe=session_continuity+candidate best_score=97 best_path=compiled/memory/working.md best_section=working".to_string(),
                "typed_retrieval_probe=procedural best_score=96 best_path=MEM.md best_section=workflows".to_string(),
                "typed_retrieval_probe=canonical best_score=98 best_path=POLICY.md best_section=decision-history".to_string(),
            ],
            recommendations: Vec::new(),
            generated_at: Utc::now(),
            completed_at: Utc::now(),
        };

        write_feature_benchmark_artifacts(&output, &report).expect("write benchmark artifacts");
        let latest_md =
            fs::read_to_string(feature_benchmark_reports_dir(&output).join("latest.md"))
                .expect("read latest benchmark markdown");
        assert!(latest_md.contains("typed_retrieval_probe=session_continuity+candidate"));
        assert!(latest_md.contains("typed_retrieval_probe=procedural"));
        assert!(latest_md.contains("typed_retrieval_probe=canonical"));
        assert!(latest_md.contains("best_path=compiled/memory/working.md"));

        fs::remove_dir_all(dir).expect("cleanup benchmark artifact dir");
    }
}
