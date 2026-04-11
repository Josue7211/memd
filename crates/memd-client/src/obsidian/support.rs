use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::SystemTime,
};

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

use super::{
    ObsidianAttachment, ObsidianNote, ObsidianScanCache, ObsidianScanCacheAttachmentEntry,
    ObsidianScanCacheNoteEntry, normalized_title,
};

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

pub(crate) fn find_cached_note_by_content_hash(
    cache: &ObsidianScanCache,
    content_hash: &str,
) -> Option<(String, ObsidianScanCacheNoteEntry)> {
    cache
        .notes
        .iter()
        .find(|(_, entry)| entry.note.content_hash == content_hash)
        .map(|(path, entry)| (path.clone(), entry.clone()))
}

pub(crate) fn find_cached_attachment_by_content_hash(
    cache: &ObsidianScanCache,
    content_hash: &str,
) -> Option<(String, ObsidianScanCacheAttachmentEntry)> {
    cache
        .attachments
        .iter()
        .find(|(_, entry)| entry.attachment.content_hash == content_hash)
        .map(|(path, entry)| (path.clone(), entry.clone()))
}

pub(crate) fn find_cached_note_by_stat(
    cache: &ObsidianScanCache,
    bytes: u64,
    modified_at: Option<DateTime<Utc>>,
) -> Option<(String, ObsidianScanCacheNoteEntry)> {
    cache
        .notes
        .iter()
        .find(|(_, entry)| entry.bytes == bytes && entry.modified_at == modified_at)
        .map(|(path, entry)| (path.clone(), entry.clone()))
}

pub(crate) fn find_cached_attachment_by_stat(
    cache: &ObsidianScanCache,
    bytes: u64,
    modified_at: Option<DateTime<Utc>>,
) -> Option<(String, ObsidianScanCacheAttachmentEntry)> {
    cache
        .attachments
        .iter()
        .find(|(_, entry)| entry.bytes == bytes && entry.modified_at == modified_at)
        .map(|(path, entry)| (path.clone(), entry.clone()))
}

pub(crate) fn find_cached_note_by_path_migration(
    cache: &ObsidianScanCache,
    current_path: &str,
    bytes: u64,
    modified_at: Option<DateTime<Utc>>,
    migrations: &[(String, String)],
) -> Option<(String, ObsidianScanCacheNoteEntry)> {
    find_cached_by_path_migration(&cache.notes, current_path, bytes, modified_at, migrations)
}

pub(crate) fn find_cached_attachment_by_path_migration(
    cache: &ObsidianScanCache,
    current_path: &str,
    bytes: u64,
    modified_at: Option<DateTime<Utc>>,
    migrations: &[(String, String)],
) -> Option<(String, ObsidianScanCacheAttachmentEntry)> {
    find_cached_by_path_migration(
        &cache.attachments,
        current_path,
        bytes,
        modified_at,
        migrations,
    )
}

fn find_cached_by_path_migration<T>(
    cache: &HashMap<String, T>,
    current_path: &str,
    bytes: u64,
    modified_at: Option<DateTime<Utc>>,
    migrations: &[(String, String)],
) -> Option<(String, T)>
where
    T: Clone + HasScanCacheMetadata,
{
    for (old_prefix, new_prefix) in migrations.iter().rev() {
        if let Some(candidate_path) = translate_migrated_path(current_path, old_prefix, new_prefix)
            && let Some(entry) = cache.get(&candidate_path)
            && entry.cache_bytes() == bytes
            && entry.cache_modified_at() == modified_at
        {
            return Some((candidate_path, entry.clone()));
        }
    }
    None
}

fn translate_migrated_path(
    current_path: &str,
    old_prefix: &str,
    new_prefix: &str,
) -> Option<String> {
    let current = Path::new(current_path);
    let new_prefix = Path::new(new_prefix);
    let remainder = current.strip_prefix(new_prefix).ok()?;
    let mut candidate = PathBuf::from(old_prefix);
    candidate.push(remainder);
    Some(candidate.display().to_string())
}

fn split_path_parts(path: &str) -> Vec<String> {
    Path::new(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect()
}

fn join_path_parts(parts: &[String]) -> String {
    let mut path = PathBuf::new();
    for part in parts {
        path.push(part);
    }
    path.display().to_string()
}

pub(crate) trait HasScanCacheMetadata {
    fn cache_bytes(&self) -> u64;
    fn cache_modified_at(&self) -> Option<DateTime<Utc>>;
}

impl HasScanCacheMetadata for ObsidianScanCacheNoteEntry {
    fn cache_bytes(&self) -> u64 {
        self.bytes
    }

    fn cache_modified_at(&self) -> Option<DateTime<Utc>> {
        self.modified_at
    }
}

impl HasScanCacheMetadata for ObsidianScanCacheAttachmentEntry {
    fn cache_bytes(&self) -> u64 {
        self.bytes
    }

    fn cache_modified_at(&self) -> Option<DateTime<Utc>> {
        self.modified_at
    }
}

pub(crate) fn register_path_migration(
    migrations: &mut Vec<(String, String)>,
    old_prefix: String,
    new_prefix: String,
) {
    if old_prefix.is_empty() || new_prefix.is_empty() || old_prefix == new_prefix {
        return;
    }
    if migrations.iter().any(|(existing_old, existing_new)| {
        existing_old == &old_prefix && existing_new == &new_prefix
    }) {
        return;
    }
    migrations.push((old_prefix, new_prefix));
}

pub(crate) fn infer_path_migration(old_path: &str, new_path: &str) -> Option<(String, String)> {
    let old_parts = split_path_parts(old_path);
    let new_parts = split_path_parts(new_path);
    if old_parts.len() < 2 || new_parts.len() < 2 {
        return None;
    }
    let mut suffix_len = 0usize;
    while suffix_len < old_parts.len()
        && suffix_len < new_parts.len()
        && old_parts[old_parts.len() - 1 - suffix_len]
            == new_parts[new_parts.len() - 1 - suffix_len]
    {
        suffix_len += 1;
    }
    if suffix_len < 2 {
        return None;
    }
    let old_prefix = join_path_parts(&old_parts[..old_parts.len() - suffix_len]);
    let new_prefix = join_path_parts(&new_parts[..new_parts.len() - suffix_len]);
    if old_prefix.is_empty() || new_prefix.is_empty() || old_prefix == new_prefix {
        return None;
    }
    Some((old_prefix, new_prefix))
}
