use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Context;
use serde_json::Value as JsonValue;

/// Load published baselines from the JSON file.
pub(crate) fn load_published_baselines(
    baselines_path: &Path,
) -> anyhow::Result<BTreeMap<String, BTreeMap<String, BaselineEntry>>> {
    let raw = std::fs::read_to_string(baselines_path)
        .context("read published baselines JSON")?;
    let parsed: JsonValue = serde_json::from_str(&raw)
        .context("parse published baselines JSON")?;
    let mut result = BTreeMap::new();
    if let Some(obj) = parsed.as_object() {
        for (benchmark_id, systems) in obj {
            let mut entries = BTreeMap::new();
            if let Some(systems_obj) = systems.as_object() {
                for (system_name, data) in systems_obj {
                    entries.insert(
                        system_name.clone(),
                        BaselineEntry {
                            accuracy: data.get("accuracy").and_then(JsonValue::as_f64),
                            source: data
                                .get("source")
                                .and_then(JsonValue::as_str)
                                .unwrap_or("")
                                .to_string(),
                            date: data
                                .get("date")
                                .and_then(JsonValue::as_str)
                                .unwrap_or("")
                                .to_string(),
                            note: data
                                .get("note")
                                .and_then(JsonValue::as_str)
                                .map(str::to_string),
                        },
                    );
                }
            }
            result.insert(benchmark_id.clone(), entries);
        }
    }
    Ok(result)
}

#[derive(Debug, Clone)]
pub(crate) struct BaselineEntry {
    pub accuracy: Option<f64>,
    pub source: String,
    pub date: String,
    pub note: Option<String>,
}

/// Render a markdown comparison table for a set of benchmark results.
pub(crate) fn render_comparison_table(
    memd_scores: &BTreeMap<String, f64>,
    baselines: &BTreeMap<String, BTreeMap<String, BaselineEntry>>,
) -> String {
    let benchmark_ids: Vec<&str> = ["longmemeval", "locomo", "membench"]
        .iter()
        .copied()
        .collect::<Vec<_>>();
    let mut lines = Vec::new();
    lines.push("## Competitive Comparison".to_string());
    lines.push(String::new());
    lines.push("| System | LongMemEval | LoCoMo | MemBench | Source |".to_string());
    lines.push("|--------|-------------|--------|----------|--------|".to_string());

    // memd row
    let memd_lme = memd_scores
        .get("longmemeval")
        .map(|v| format!("{:.1}%", v * 100.0))
        .unwrap_or_else(|| "-".to_string());
    let memd_loc = memd_scores
        .get("locomo")
        .map(|v| format!("{:.1}%", v * 100.0))
        .unwrap_or_else(|| "-".to_string());
    let memd_mb = memd_scores
        .get("membench")
        .map(|v| format!("{:.1}%", v * 100.0))
        .unwrap_or_else(|| "-".to_string());
    lines.push(format!(
        "| memd | {memd_lme} | {memd_loc} | {memd_mb} | this run |"
    ));

    // Collect all competitor names across benchmarks
    let mut all_systems: BTreeMap<String, [Option<f64>; 3]> = BTreeMap::new();
    let mut system_sources: BTreeMap<String, String> = BTreeMap::new();
    for (i, bid) in benchmark_ids.iter().enumerate() {
        if let Some(entries) = baselines.get(*bid) {
            for (name, entry) in entries {
                let scores = all_systems.entry(name.clone()).or_insert([None; 3]);
                scores[i] = entry.accuracy;
                system_sources
                    .entry(name.clone())
                    .or_insert_with(|| entry.source.clone());
            }
        }
    }

    for (name, scores) in &all_systems {
        let fmt = |v: Option<f64>| -> String {
            v.map(|x| format!("{x:.1}%"))
                .unwrap_or_else(|| "-".to_string())
        };
        let source = system_sources.get(name).map(String::as_str).unwrap_or("-");
        lines.push(format!(
            "| {} | {} | {} | {} | {} |",
            name,
            fmt(scores[0]),
            fmt(scores[1]),
            fmt(scores[2]),
            source
        ));
    }

    lines.join("\n")
}

/// Default path to the published baselines JSON file.
pub(crate) fn default_baselines_path(output: &Path) -> std::path::PathBuf {
    output
        .join("benchmarks")
        .join("baselines")
        .join("published_baselines.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_table_with_memd_only() {
        let mut memd_scores = BTreeMap::new();
        memd_scores.insert("longmemeval".to_string(), 0.75);
        let baselines = BTreeMap::new();
        let table = render_comparison_table(&memd_scores, &baselines);
        assert!(table.contains("memd"));
        assert!(table.contains("75.0%"));
    }

    #[test]
    fn render_table_with_baselines() {
        let mut memd_scores = BTreeMap::new();
        memd_scores.insert("longmemeval".to_string(), 0.80);
        let mut baselines = BTreeMap::new();
        let mut lme = BTreeMap::new();
        lme.insert(
            "SuperMem".to_string(),
            BaselineEntry {
                accuracy: Some(81.6),
                source: "supermemory.ai".to_string(),
                date: "2026".to_string(),
                note: None,
            },
        );
        baselines.insert("longmemeval".to_string(), lme);
        let table = render_comparison_table(&memd_scores, &baselines);
        assert!(table.contains("SuperMem"));
        assert!(table.contains("81.6%"));
    }
}
