use std::path::{Path, PathBuf};

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::ResumeSnapshot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HarnessTurnCache {
    pub(crate) turn_key: String,
    pub(crate) project: Option<String>,
    pub(crate) namespace: Option<String>,
    pub(crate) agent: Option<String>,
    pub(crate) mode: String,
    pub(crate) query: String,
    pub(crate) refreshed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ResumeSnapshotCacheFile {
    pub(crate) turn_key: String,
    pub(crate) snapshot: ResumeSnapshot,
    pub(crate) refreshed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HandoffSnapshotCacheFile {
    pub(crate) turn_key: String,
    pub(crate) target_session: Option<String>,
    pub(crate) target_bundle: Option<String>,
    pub(crate) handoff: crate::HandoffSnapshot,
    pub(crate) refreshed_at: DateTime<Utc>,
}

pub(crate) fn normalize_query(query: &str) -> String {
    query
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

pub(crate) fn build_turn_key(
    project: Option<&str>,
    namespace: Option<&str>,
    agent: Option<&str>,
    mode: &str,
    query: &str,
) -> String {
    let normalized = format!(
        "project={}|namespace={}|agent={}|mode={}|query={}",
        project
            .map(|value| value.trim().to_ascii_lowercase())
            .unwrap_or_else(|| "none".to_string()),
        namespace
            .map(|value| value.trim().to_ascii_lowercase())
            .unwrap_or_else(|| "none".to_string()),
        agent
            .map(|value| value.trim().to_ascii_lowercase())
            .unwrap_or_else(|| "none".to_string()),
        mode.trim().to_ascii_lowercase(),
        normalize_query(query),
    );
    format!("{:x}", Sha256::digest(normalized.as_bytes()))
}

pub(crate) fn build_turn_cache_from_snapshot(
    snapshot: &ResumeSnapshot,
    mode: &str,
    query: &str,
) -> HarnessTurnCache {
    HarnessTurnCache {
        turn_key: build_turn_key(
            snapshot.project.as_deref(),
            snapshot.namespace.as_deref(),
            snapshot.agent.as_deref(),
            mode,
            query,
        ),
        project: snapshot.project.clone(),
        namespace: snapshot.namespace.clone(),
        agent: snapshot.agent.clone(),
        mode: mode.to_string(),
        query: normalize_query(query),
        refreshed_at: Utc::now(),
    }
}

pub(crate) fn turn_cache_path(output: &Path, agent: &str) -> PathBuf {
    output.join("state").join(format!(
        "{}-turn-cache.json",
        agent.trim().to_ascii_lowercase()
    ))
}

pub(crate) fn read_turn_cache(
    output: &Path,
    agent: &str,
) -> anyhow::Result<Option<HarnessTurnCache>> {
    let path = turn_cache_path(output, agent);
    if !path.exists() {
        return Ok(None);
    }

    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let cache = serde_json::from_str::<HarnessTurnCache>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(cache))
}

pub(crate) fn write_turn_cache(
    output: &Path,
    agent: &str,
    cache: &HarnessTurnCache,
) -> anyhow::Result<()> {
    let path = turn_cache_path(output, agent);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(cache)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn resume_snapshot_cache_path(output: &Path) -> PathBuf {
    output.join("state").join("resume-snapshot-cache.json")
}

pub(crate) fn read_resume_snapshot_cache(
    output: &Path,
    turn_key: &str,
    max_age_minutes: i64,
) -> anyhow::Result<Option<ResumeSnapshot>> {
    let path = resume_snapshot_cache_path(output);
    if !path.exists() {
        return Ok(None);
    }

    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let cache = serde_json::from_str::<ResumeSnapshotCacheFile>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    if cache.turn_key != turn_key {
        return Ok(None);
    }
    let age_minutes = (Utc::now() - cache.refreshed_at).num_minutes();
    if age_minutes > max_age_minutes {
        return Ok(None);
    }
    Ok(Some(cache.snapshot))
}

pub(crate) fn write_resume_snapshot_cache(
    output: &Path,
    turn_key: &str,
    snapshot: &ResumeSnapshot,
) -> anyhow::Result<()> {
    let path = resume_snapshot_cache_path(output);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let cache = ResumeSnapshotCacheFile {
        turn_key: turn_key.to_string(),
        snapshot: snapshot.clone(),
        refreshed_at: Utc::now(),
    };
    std::fs::write(&path, serde_json::to_string_pretty(&cache)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn handoff_snapshot_cache_path(output: &Path) -> PathBuf {
    output.join("state").join("handoff-snapshot-cache.json")
}

pub(crate) fn read_handoff_snapshot_cache(
    output: &Path,
    turn_key: &str,
    max_age_minutes: i64,
) -> anyhow::Result<Option<crate::HandoffSnapshot>> {
    let path = handoff_snapshot_cache_path(output);
    if !path.exists() {
        return Ok(None);
    }

    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let cache = serde_json::from_str::<HandoffSnapshotCacheFile>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    if cache.turn_key != turn_key {
        return Ok(None);
    }
    let age_minutes = (Utc::now() - cache.refreshed_at).num_minutes();
    if age_minutes > max_age_minutes {
        return Ok(None);
    }
    Ok(Some(cache.handoff))
}

pub(crate) fn write_handoff_snapshot_cache(
    output: &Path,
    turn_key: &str,
    handoff: &crate::HandoffSnapshot,
) -> anyhow::Result<()> {
    let path = handoff_snapshot_cache_path(output);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let cache = HandoffSnapshotCacheFile {
        turn_key: turn_key.to_string(),
        target_session: handoff.target_session.clone(),
        target_bundle: handoff.target_bundle.clone(),
        handoff: handoff.clone(),
        refreshed_at: Utc::now(),
    };
    std::fs::write(&path, serde_json::to_string_pretty(&cache)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) async fn refresh_turn_cached_pack_files(
    output: &Path,
    snapshot: &ResumeSnapshot,
    manifest_files: &[PathBuf],
    agent: &str,
    mode: &str,
    query: &str,
    write_bundle_memory_files: impl std::future::Future<Output = anyhow::Result<()>>,
) -> anyhow::Result<Vec<PathBuf>> {
    let turn_cache = build_turn_cache_from_snapshot(snapshot, mode, query);
    let existing_cache = read_turn_cache(output, agent)?;
    if existing_cache
        .as_ref()
        .is_some_and(|existing| existing.turn_key == turn_cache.turn_key)
        && manifest_files.iter().all(|path| path.exists())
    {
        return Ok(manifest_files.to_vec());
    }

    write_bundle_memory_files.await?;
    write_turn_cache(output, agent, &turn_cache)?;
    Ok(manifest_files.to_vec())
}
