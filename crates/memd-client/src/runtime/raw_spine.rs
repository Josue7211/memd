use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RawSpineRecord {
    pub(crate) id: String,
    pub(crate) event_type: String,
    pub(crate) stage: String,
    pub(crate) source_system: Option<String>,
    pub(crate) source_path: Option<String>,
    pub(crate) project: Option<String>,
    pub(crate) namespace: Option<String>,
    pub(crate) workspace: Option<String>,
    pub(crate) confidence: Option<f32>,
    pub(crate) tags: Vec<String>,
    pub(crate) content_preview: String,
    pub(crate) recorded_at: DateTime<Utc>,
}

fn raw_spine_path(output: &Path) -> PathBuf {
    output.join("state").join("raw-spine.jsonl")
}

pub(crate) fn derive_raw_spine_record(
    event_type: &str,
    stage: &str,
    source_system: Option<&str>,
    source_path: Option<&str>,
    project: Option<&str>,
    namespace: Option<&str>,
    workspace: Option<&str>,
    confidence: Option<f32>,
    tags: &[&str],
    content: &str,
) -> RawSpineRecord {
    let preview = compact_inline(content.trim(), raw_spine_preview_limit(tags, content));
    let signature = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        event_type,
        stage,
        source_system.unwrap_or("none"),
        source_path.unwrap_or("none"),
        project.unwrap_or("none"),
        namespace.unwrap_or("none"),
        workspace.unwrap_or("none"),
        preview
    );

    RawSpineRecord {
        id: format!("raw-{}", short_hash_text(&signature)),
        event_type: event_type.to_string(),
        stage: stage.to_string(),
        source_system: source_system.map(str::to_string),
        source_path: source_path.map(str::to_string),
        project: project.map(str::to_string),
        namespace: namespace.map(str::to_string),
        workspace: workspace.map(str::to_string),
        confidence,
        tags: tags.iter().map(|value| value.to_string()).collect(),
        content_preview: preview,
        recorded_at: Utc::now(),
    }
}

fn raw_spine_preview_limit(tags: &[&str], content: &str) -> usize {
    let normalized = content.to_ascii_lowercase();
    let continuity_tag = tags.iter().any(|tag| {
        matches!(
            *tag,
            "current-task" | "continuity" | "handoff" | "next-agent"
        )
    });
    if continuity_tag
        || normalized.contains("current next action")
        || normalized.contains("next action")
    {
        1600
    } else {
        180
    }
}

pub(crate) fn read_raw_spine_records(output: &Path) -> anyhow::Result<Vec<RawSpineRecord>> {
    let path = raw_spine_path(output);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let mut records = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let record = serde_json::from_str::<RawSpineRecord>(trimmed)
            .with_context(|| format!("parse {}", path.display()))?;
        records.push(record);
    }
    Ok(records)
}

pub(crate) fn write_raw_spine_records(
    output: &Path,
    records: &[RawSpineRecord],
) -> anyhow::Result<()> {
    let path = raw_spine_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut merged = std::collections::BTreeMap::<String, RawSpineRecord>::new();
    for record in read_raw_spine_records(output)? {
        merged.insert(record.id.clone(), record);
    }
    for record in records {
        merged.insert(record.id.clone(), record.clone());
    }

    let mut merged = merged.into_values().collect::<Vec<_>>();
    merged.sort_by(|left, right| right.recorded_at.cmp(&left.recorded_at));
    merged.truncate(512);

    let mut body = String::new();
    for record in merged {
        body.push_str(&serde_json::to_string(&record)?);
        body.push('\n');
    }
    fs::write(&path, body).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_raw_spine_records_merges_and_sorts_latest_first() {
        let dir = std::env::temp_dir().join(format!("memd-raw-spine-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create raw spine dir");

        let older = RawSpineRecord {
            id: "older".to_string(),
            event_type: "remember".to_string(),
            stage: "canonical".to_string(),
            source_system: Some("remember".to_string()),
            source_path: Some("README.md".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("core".to_string()),
            confidence: Some(0.8),
            tags: vec!["raw-spine".to_string()],
            content_preview: "older record".to_string(),
            recorded_at: chrono::Utc::now() - chrono::Duration::minutes(5),
        };

        let newer = RawSpineRecord {
            id: "newer".to_string(),
            event_type: "checkpoint".to_string(),
            stage: "candidate".to_string(),
            source_system: Some("checkpoint".to_string()),
            source_path: Some("checkpoint".to_string()),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("core".to_string()),
            confidence: Some(0.9),
            tags: vec!["raw-spine".to_string(), "checkpoint".to_string()],
            content_preview: "newer record".to_string(),
            recorded_at: chrono::Utc::now(),
        };

        write_raw_spine_records(&dir, &[older.clone()]).expect("write older");
        write_raw_spine_records(&dir, &[newer.clone()]).expect("write newer");

        let records = read_raw_spine_records(&dir).expect("read raw spine");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, "newer");
        assert_eq!(records[1].id, "older");

        std::fs::remove_dir_all(&dir).expect("cleanup raw spine dir");
    }

    #[test]
    fn derive_raw_spine_record_keeps_source_linkage() {
        let record = derive_raw_spine_record(
            "hook_capture",
            "candidate",
            Some("hook-capture"),
            Some(".memd/wake.md"),
            Some("memd"),
            Some("main"),
            Some("core"),
            Some(0.88),
            &["raw-spine", "correction"],
            "corrected deployment target is local-first",
        );

        assert_eq!(record.event_type, "hook_capture");
        assert_eq!(record.stage, "candidate");
        assert_eq!(record.source_system.as_deref(), Some("hook-capture"));
        assert_eq!(record.source_path.as_deref(), Some(".memd/wake.md"));
        assert!(record.tags.iter().any(|tag| tag == "raw-spine"));
    }

    #[test]
    fn derive_raw_spine_record_preserves_next_action_preview() {
        let content = "CURRENT NEXT ACTION after commit 3dd7ad82: continue remaining blockers only. Done this slice: memd live-state sync is separated from ClawControl by default. Removed stale com.memd.live-state-sync-clawcontrol launchd job that forced CAPTURE_HTTP=1; installed com.memd.live-state-sync pointing at scripts/live-state-sync-memd.sh with CAPTURE_HTTP=0 and IMPORT_CLAWCONTROL_BUNDLE=0. scripts/live-state-sync-clawcontrol.sh now uses memd-owned mac-bridge and approved-communications fallbacks by default and only probes/imports ClawControl when CAPTURE_HTTP=1 IMPORT_CLAWCONTROL_BUNDLE=1 is explicit.";
        let record = derive_raw_spine_record(
            "checkpoint",
            "canonical",
            Some("checkpoint"),
            None,
            Some("memd"),
            Some("main"),
            None,
            Some(0.8),
            &["checkpoint", "current-task", "continuity", "handoff"],
            content,
        );

        assert!(
            record
                .content_preview
                .contains("Removed stale com.memd.live-state-sync-clawcontrol launchd job"),
            "{}",
            record.content_preview
        );
        assert!(
            record
                .content_preview
                .contains("IMPORT_CLAWCONTROL_BUNDLE=1 is explicit"),
            "{}",
            record.content_preview
        );
        assert!(!record.content_preview.ends_with("..."));
    }
}
