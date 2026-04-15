use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use anyhow::Context;
use chrono::Utc;
use memd_schema::{
    MemoryItem, MemoryKind, MemoryScope, MemoryStage, MemoryStatus, MemoryVisibility,
};
use uuid::Uuid;

use crate::keys::content_hash;
use crate::store::IngestionManifestEntry;
use crate::{AppState, canonical_key, redundancy_key};

/// Summary of a lane ingestion run.
#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct LaneIngestionSummary {
    pub(crate) files_scanned: usize,
    pub(crate) files_ingested: usize,
    pub(crate) files_skipped: usize,
    pub(crate) files_stale: usize,
}

/// Walk `.memd/lanes/*/` under `root`, ingest changed files into DB.
pub(crate) fn ingest_lane_files(
    state: &AppState,
    root: &Path,
    project: Option<&str>,
    namespace: Option<&str>,
) -> anyhow::Result<LaneIngestionSummary> {
    let lanes_dir = root.join(".memd").join("lanes");
    if !lanes_dir.is_dir() {
        return Ok(LaneIngestionSummary {
            files_scanned: 0,
            files_ingested: 0,
            files_skipped: 0,
            files_stale: 0,
        });
    }

    let mut scanned = 0usize;
    let mut ingested = 0usize;
    let mut skipped = 0usize;
    let mut seen_paths = std::collections::HashSet::new();

    let lane_entries = fs::read_dir(&lanes_dir)
        .with_context(|| format!("read lanes dir {}", lanes_dir.display()))?;

    for lane_entry in lane_entries {
        let lane_entry = lane_entry?;
        let lane_path = lane_entry.path();
        if !lane_path.is_dir() {
            continue;
        }
        let lane_name = lane_entry
            .file_name()
            .to_string_lossy()
            .to_string();

        let files = fs::read_dir(&lane_path)
            .with_context(|| format!("read lane dir {}", lane_path.display()))?;

        for file_entry in files {
            let file_entry = file_entry?;
            let file_path = file_entry.path();
            if !file_path.is_file() {
                continue;
            }
            // Only ingest markdown and text files.
            let ext = file_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            if !matches!(ext, "md" | "txt" | "toml" | "json" | "yaml" | "yml") {
                continue;
            }

            scanned += 1;
            let source_path_str = file_path.display().to_string();
            seen_paths.insert(source_path_str.clone());

            let content = fs::read_to_string(&file_path)
                .with_context(|| format!("read {}", file_path.display()))?;
            let hash = content_hash(&content);
            let mtime = file_entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            // Check manifest — skip if hash unchanged.
            if let Some(existing) = state.store.ingestion_manifest_get(&source_path_str)? {
                if existing.content_hash == hash {
                    skipped += 1;
                    continue;
                }
            }

            // Classify content by filename hints.
            let kind = classify_lane_file_kind(&file_path, &lane_name);

            // Build memory item from file content.
            let now = Utc::now();
            let item = MemoryItem {
                id: Uuid::new_v4(),
                kind,
                content: content.clone(),
                scope: MemoryScope::Project,
                visibility: MemoryVisibility::Private,
                project: project.map(String::from),
                namespace: namespace.map(String::from),
                workspace: None,
                source_agent: Some("ingestion-pipeline".to_string()),
                source_system: Some("memd".to_string()),
                source_path: Some(source_path_str.clone()),
                source_quality: Some(memd_schema::SourceQuality::Canonical),
                belief_branch: None,
                preferred: false,
                confidence: 0.8,
                ttl_seconds: if kind == MemoryKind::Status {
                    Some(86400)
                } else {
                    None
                },
                created_at: now,
                updated_at: now,
                last_verified_at: Some(now),
                supersedes: vec![],
                tags: vec![format!("lane:{lane_name}")],
                status: MemoryStatus::Active,
                stage: MemoryStage::Canonical,
                redundancy_key: None,
                lane: Some(lane_name.clone()),
            };

            let ck = canonical_key(&item);
            let rk = redundancy_key(&item);
            let item = MemoryItem {
                redundancy_key: Some(rk.clone()),
                ..item
            };

            // Insert or reinforce existing.
            state.store.insert_or_get_duplicate(&item, &ck, &rk)?;

            // Update manifest.
            state.store.ingestion_manifest_upsert(&IngestionManifestEntry {
                source_path: source_path_str,
                content_hash: hash,
                mtime_epoch: mtime,
                lane: Some(lane_name.clone()),
                project: project.map(String::from),
                namespace: namespace.map(String::from),
                last_ingested_at: now.to_rfc3339(),
                memory_item_id: Some(item.id.to_string()),
            })?;

            ingested += 1;
        }
    }

    // Mark stale: manifest entries whose files no longer exist.
    let all_manifest = state
        .store
        .ingestion_manifest_list(project, namespace)?;
    let mut stale = 0usize;
    for entry in &all_manifest {
        if !seen_paths.contains(&entry.source_path) && !Path::new(&entry.source_path).exists() {
            // File was deleted — expire its memory item if present.
            if let Some(item_id) = &entry.memory_item_id {
                if let Ok(Some(mut item)) =
                    state.store.get(Uuid::parse_str(item_id).unwrap_or(Uuid::nil()))
                {
                    if item.status == MemoryStatus::Active {
                        item.status = MemoryStatus::Expired;
                        item.updated_at = Utc::now();
                        let ck = canonical_key(&item);
                        let rk = redundancy_key(&item);
                        let _ = state.store.update(&item, &ck, &rk);
                        stale += 1;
                    }
                }
            }
        }
    }

    Ok(LaneIngestionSummary {
        files_scanned: scanned,
        files_ingested: ingested,
        files_skipped: skipped,
        files_stale: stale,
    })
}

/// Classify a lane file into a MemoryKind based on filename and lane name.
fn classify_lane_file_kind(path: &Path, lane_name: &str) -> MemoryKind {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    if stem.contains("decision") || stem.contains("adr") {
        return MemoryKind::Decision;
    }
    if stem.contains("procedure") || stem.contains("runbook") || stem.contains("howto") {
        return MemoryKind::Procedural;
    }
    if stem.contains("status") || stem.contains("changelog") {
        return MemoryKind::Status;
    }
    if stem.contains("constraint") || stem.contains("rule") {
        return MemoryKind::Constraint;
    }
    if stem.contains("pattern") || stem.contains("convention") {
        return MemoryKind::Pattern;
    }
    if stem.contains("topology") || stem.contains("architecture") || stem.contains("infra") {
        return MemoryKind::Topology;
    }

    // Fall back based on lane name.
    match lane_name {
        "decisions" => MemoryKind::Decision,
        "procedures" | "runbooks" => MemoryKind::Procedural,
        "architecture" | "topology" => MemoryKind::Topology,
        "constraints" | "rules" => MemoryKind::Constraint,
        "patterns" | "conventions" => MemoryKind::Pattern,
        _ => MemoryKind::Fact,
    }
}
