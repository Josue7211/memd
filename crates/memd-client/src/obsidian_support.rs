use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

use crate::obsidian::{normalized_title, ObsidianAttachment, ObsidianNote};

pub(crate) fn default_sync_state_path(vault: &Path) -> PathBuf {
    vault.join(".memd").join("obsidian-sync.json")
}

pub(crate) fn default_scan_cache_path(vault: &Path) -> PathBuf {
    vault.join(".memd").join("obsidian-scan-cache.json")
}

pub(crate) fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(crate) fn hash_bytes(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

pub(crate) fn system_time_to_utc(value: SystemTime) -> DateTime<Utc> {
    value.into()
}

pub(crate) fn classify_asset_kind(path: &Path, mime: Option<&str>) -> &'static str {
    let ext = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.trim().to_ascii_lowercase());

    match (ext.as_deref(), mime) {
        (Some("pdf"), _) | (_, Some("application/pdf")) => "pdf",
        (Some("png"), _)
        | (Some("jpg"), _)
        | (Some("jpeg"), _)
        | (Some("webp"), _)
        | (Some("heic"), _) => "image",
        (Some("mp4"), _)
        | (Some("mov"), _)
        | (Some("mkv"), _)
        | (Some("webm"), _)
        | (_, Some("video/mp4"))
        | (_, Some("video/webm")) => "video",
        (Some("csv"), _) | (Some("tsv"), _) | (Some("xlsx"), _) | (_, Some("text/csv")) => "table",
        (Some("tex"), _) | (Some("mml"), _) | (_, Some("application/mathml+xml")) => "equation",
        (Some("txt"), _)
        | (Some("json"), _)
        | (Some("yaml"), _)
        | (Some("yml"), _)
        | (Some("log"), _)
        | (Some("toml"), _)
        | (_, Some("text/plain")) => "text",
        _ => "unknown",
    }
}

pub(crate) fn is_text_like_attachment(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "txt" | "json" | "yaml" | "yml" | "csv" | "tsv" | "log" | "toml" | "ini"
            )
        })
        .unwrap_or(false)
}

pub(crate) fn should_skip_vault_path(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component,
            std::path::Component::Normal(name)
                if name == ".memd" || name == ".obsidian" || name == ".git"
        )
    })
}

pub(crate) fn note_matches_scope(
    note: &ObsidianNote,
    include_folders: &[String],
    exclude_folders: &[String],
    include_tags: &[String],
    exclude_tags: &[String],
) -> bool {
    let folder = note.folder_path.as_deref().unwrap_or_default();
    let folder_ok = if include_folders.is_empty() {
        true
    } else {
        include_folders
            .iter()
            .any(|candidate| folder_matches(folder, candidate))
    };
    let folder_blocked = exclude_folders
        .iter()
        .any(|candidate| folder_matches(folder, candidate));

    let tag_ok = if include_tags.is_empty() {
        true
    } else {
        note.tags.iter().any(|tag| include_tags.contains(tag))
    };
    let tag_blocked = note.tags.iter().any(|tag| exclude_tags.contains(tag));

    folder_ok && !folder_blocked && tag_ok && !tag_blocked
}

pub(crate) fn attachment_matches_scope(
    attachment: &ObsidianAttachment,
    include_folders: &[String],
    exclude_folders: &[String],
) -> bool {
    let folder = attachment.folder_path.as_deref().unwrap_or_default();
    let folder_ok = if include_folders.is_empty() {
        true
    } else {
        include_folders
            .iter()
            .any(|candidate| folder_matches(folder, candidate))
    };
    let folder_blocked = exclude_folders
        .iter()
        .any(|candidate| folder_matches(folder, candidate));

    folder_ok && !folder_blocked
}

pub(crate) fn folder_matches(folder: &str, candidate: &str) -> bool {
    let folder = folder.trim_matches('/');
    let candidate = candidate.trim_matches('/');
    folder == candidate
        || folder.starts_with(&format!("{candidate}/"))
        || folder.ends_with(&format!("/{candidate}"))
        || normalized_title(folder) == normalized_title(candidate)
}
