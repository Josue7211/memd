use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};

use anyhow::Context;
use chrono::{DateTime, Utc};
use memd_schema::{
    CandidateMemoryRequest, EntityLinkRequest, EntityRelationKind, ExplainMemoryResponse,
    MemoryContextFrame, MemoryKind, MemoryScope, MemoryVisibility, SearchMemoryResponse,
    SourceMemoryResponse, SourceQuality,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::form_urlencoded::byte_serialize;
use uuid::Uuid;
use walkdir::WalkDir;

const MEMD_ROUNDTRIP_BEGIN: &str = "<!-- memd:begin -->";
const MEMD_ROUNDTRIP_END: &str = "<!-- memd:end -->";

#[derive(Debug, Clone, Serialize)]
pub struct ObsidianVaultScan {
    pub vault: PathBuf,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub workspace: Option<String>,
    pub visibility: MemoryVisibility,
    pub note_count: usize,
    pub sensitive_count: usize,
    pub skipped_count: usize,
    pub unchanged_count: usize,
    pub backlink_count: usize,
    pub attachment_count: usize,
    pub attachment_sensitive_count: usize,
    pub attachment_unchanged_count: usize,
    pub cache_hits: usize,
    pub attachment_cache_hits: usize,
    pub cache_pruned: usize,
    pub attachment_cache_pruned: usize,
    pub sensitive_notes: Vec<ObsidianSensitiveNote>,
    pub attachments: Vec<ObsidianAttachment>,
    pub notes: Vec<ObsidianNote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianSensitivity {
    pub sensitive: bool,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianSensitiveNote {
    pub path: PathBuf,
    pub relative_path: String,
    pub title: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianAttachment {
    pub path: PathBuf,
    pub relative_path: String,
    pub folder_path: Option<String>,
    pub asset_kind: String,
    pub mime: Option<String>,
    pub bytes: u64,
    pub modified_at: Option<DateTime<Utc>>,
    pub content_hash: String,
    pub sensitivity: ObsidianSensitivity,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize, Default)]
pub struct ObsidianSyncState {
    pub entries: HashMap<String, ObsidianSyncEntry>,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct ObsidianSyncEntry {
    pub content_hash: String,
    pub bytes: u64,
    pub modified_at: Option<DateTime<Utc>>,
    pub item_id: Option<Uuid>,
    #[serde(default)]
    pub entity_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianNote {
    pub path: PathBuf,
    pub relative_path: String,
    pub folder_path: Option<String>,
    pub folder_depth: usize,
    pub title: String,
    pub normalized_title: String,
    pub excerpt: String,
    pub kind: MemoryKind,
    pub tags: Vec<String>,
    pub aliases: Vec<String>,
    pub links: Vec<String>,
    pub backlinks: Vec<String>,
    pub sensitivity: ObsidianSensitivity,
    pub bytes: u64,
    pub modified_at: Option<DateTime<Utc>>,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObsidianScanCache {
    pub notes: HashMap<String, ObsidianScanCacheNoteEntry>,
    pub attachments: HashMap<String, ObsidianScanCacheAttachmentEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianScanCacheNoteEntry {
    pub bytes: u64,
    pub modified_at: Option<DateTime<Utc>>,
    pub note: ObsidianNote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianScanCacheAttachmentEntry {
    pub bytes: u64,
    pub modified_at: Option<DateTime<Utc>>,
    pub attachment: ObsidianAttachment,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObsidianImportPreview {
    pub scan: ObsidianVaultScan,
    pub candidates: Vec<CandidateMemoryRequest>,
    pub note_index: HashMap<String, usize>,
    pub unchanged_count: usize,
    pub duplicate_count: usize,
    pub sync_state_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ObsidianAttachmentMatch {
    pub note_index: usize,
    pub relation_kind: EntityRelationKind,
    pub reason: String,
}

#[allow(clippy::too_many_arguments)]
pub fn scan_vault(
    vault: impl AsRef<Path>,
    project: Option<String>,
    namespace: Option<String>,
    workspace: Option<String>,
    visibility: Option<MemoryVisibility>,
    max_notes: Option<usize>,
    include_attachments: bool,
    max_attachments: Option<usize>,
    include_folders: &[String],
    exclude_folders: &[String],
    include_tags: &[String],
    exclude_tags: &[String],
) -> anyhow::Result<ObsidianVaultScan> {
    let vault = vault.as_ref();
    let cache_path = default_scan_cache_path(vault);
    let mut scan_cache = load_scan_cache(&cache_path).unwrap_or_default();
    let mut notes = Vec::new();
    let mut sensitive_notes = Vec::new();
    let mut attachments = Vec::new();
    let mut skipped_count = 0usize;
    let mut sensitive_count = 0usize;
    let mut attachment_sensitive_count = 0usize;
    let mut cache_hits = 0usize;
    let mut attachment_cache_hits = 0usize;
    let mut path_migrations = Vec::<(String, String)>::new();
    let mut note_paths_seen = HashSet::<String>::new();
    let mut attachment_paths_seen = HashSet::<String>::new();
    let max_notes = max_notes.unwrap_or(usize::MAX);
    let max_attachments = max_attachments.unwrap_or(usize::MAX);

    for entry in WalkDir::new(vault)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if should_skip_vault_path(path) {
            skipped_count += 1;
            continue;
        }

        let is_markdown = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"));
        if is_markdown {
            let relative_path = path
                .strip_prefix(vault)
                .unwrap_or(path)
                .display()
                .to_string();
            note_paths_seen.insert(relative_path.clone());
            if notes.len() >= max_notes {
                skipped_count += 1;
                continue;
            }

            let metadata =
                fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
            let modified_at = metadata.modified().ok().map(system_time_to_utc);
            let cached_note = scan_cache.notes.get(&relative_path).and_then(|entry| {
                (entry.bytes == metadata.len() && entry.modified_at == modified_at)
                    .then(|| entry.note.clone())
            });
            if let Some(mut note) = cached_note {
                cache_hits += 1;
                note.path = path.to_path_buf();
                note.relative_path = relative_path;
                note.modified_at = modified_at;
                notes.push(note);
                continue;
            }
            if let Some((cached_path, cached_entry)) = find_cached_note_by_path_migration(
                &scan_cache,
                &relative_path,
                metadata.len(),
                modified_at,
                &path_migrations,
            ) {
                cache_hits += 1;
                scan_cache.notes.remove(&cached_path);
                let mut note = cached_entry.note;
                note.path = path.to_path_buf();
                note.relative_path = relative_path.clone();
                note.modified_at = modified_at;
                if let Some((old_prefix, new_prefix)) =
                    infer_path_migration(&cached_path, &relative_path)
                {
                    register_path_migration(&mut path_migrations, old_prefix, new_prefix);
                }
                scan_cache.notes.insert(
                    relative_path.clone(),
                    ObsidianScanCacheNoteEntry {
                        bytes: note.bytes,
                        modified_at: note.modified_at,
                        note: note.clone(),
                    },
                );
                notes.push(note);
                continue;
            }
            if let Some((cached_path, cached_entry)) =
                find_cached_note_by_stat(&scan_cache, metadata.len(), modified_at)
            {
                cache_hits += 1;
                scan_cache.notes.remove(&cached_path);
                let mut note = cached_entry.note;
                note.path = path.to_path_buf();
                note.relative_path = relative_path.clone();
                note.modified_at = modified_at;
                if let Some((old_prefix, new_prefix)) =
                    infer_path_migration(&cached_path, &relative_path)
                {
                    register_path_migration(&mut path_migrations, old_prefix, new_prefix);
                }
                scan_cache.notes.insert(
                    relative_path.clone(),
                    ObsidianScanCacheNoteEntry {
                        bytes: note.bytes,
                        modified_at: note.modified_at,
                        note: note.clone(),
                    },
                );
                notes.push(note);
                continue;
            }

            let raw =
                fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
            let raw = strip_roundtrip_annotation(&raw);
            let content_hash = hash_content(&raw);
            if let Some((cached_path, cached_entry)) =
                find_cached_note_by_content_hash(&scan_cache, &content_hash)
            {
                cache_hits += 1;
                scan_cache.notes.remove(&cached_path);
                let mut note = cached_entry.note;
                note.path = path.to_path_buf();
                note.relative_path = relative_path.clone();
                note.bytes = metadata.len();
                note.modified_at = modified_at;
                note.content_hash = content_hash.clone();
                scan_cache.notes.insert(
                    relative_path.clone(),
                    ObsidianScanCacheNoteEntry {
                        bytes: note.bytes,
                        modified_at: note.modified_at,
                        note: note.clone(),
                    },
                );
                if !note_matches_scope(
                    &note,
                    include_folders,
                    exclude_folders,
                    include_tags,
                    exclude_tags,
                ) {
                    skipped_count += 1;
                    continue;
                }
                if note.sensitivity.sensitive {
                    sensitive_notes.push(ObsidianSensitiveNote {
                        path: note.path.clone(),
                        relative_path: note.relative_path.clone(),
                        title: note.title.clone(),
                        reasons: note.sensitivity.reasons.clone(),
                    });
                    sensitive_count += 1;
                    continue;
                }
                notes.push(note);
                continue;
            }

            if let Some(note) = parse_markdown_note_from_raw(
                vault,
                path,
                &raw,
                metadata.len(),
                modified_at,
                content_hash,
            )? {
                if !note_matches_scope(
                    &note,
                    include_folders,
                    exclude_folders,
                    include_tags,
                    exclude_tags,
                ) {
                    skipped_count += 1;
                    continue;
                }
                if note.sensitivity.sensitive {
                    sensitive_notes.push(ObsidianSensitiveNote {
                        path: note.path.clone(),
                        relative_path: note.relative_path.clone(),
                        title: note.title.clone(),
                        reasons: note.sensitivity.reasons.clone(),
                    });
                    sensitive_count += 1;
                    continue;
                }
                scan_cache.notes.insert(
                    note.relative_path.clone(),
                    ObsidianScanCacheNoteEntry {
                        bytes: note.bytes,
                        modified_at: note.modified_at,
                        note: note.clone(),
                    },
                );
                notes.push(note);
            }
            continue;
        }

        if !include_attachments {
            skipped_count += 1;
            continue;
        }
        let relative_path = path
            .strip_prefix(vault)
            .unwrap_or(path)
            .display()
            .to_string();
        attachment_paths_seen.insert(relative_path.clone());
        if attachments.len() >= max_attachments {
            skipped_count += 1;
            continue;
        }
        let metadata = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
        let modified_at = metadata.modified().ok().map(system_time_to_utc);
        let cached_attachment = scan_cache
            .attachments
            .get(&relative_path)
            .and_then(|entry| {
                (entry.bytes == metadata.len() && entry.modified_at == modified_at)
                    .then(|| entry.attachment.clone())
            });
        if let Some(mut attachment) = cached_attachment {
            attachment_cache_hits += 1;
            attachment.path = path.to_path_buf();
            attachment.relative_path = relative_path;
            attachment.modified_at = modified_at;
            if !attachment_matches_scope(&attachment, include_folders, exclude_folders) {
                skipped_count += 1;
                continue;
            }
            if attachment.sensitivity.sensitive {
                attachment_sensitive_count += 1;
                continue;
            }
            attachments.push(attachment);
            continue;
        }
        if let Some((cached_path, cached_entry)) = find_cached_attachment_by_path_migration(
            &scan_cache,
            &relative_path,
            metadata.len(),
            modified_at,
            &path_migrations,
        ) {
            attachment_cache_hits += 1;
            scan_cache.attachments.remove(&cached_path);
            let mut attachment = cached_entry.attachment;
            attachment.path = path.to_path_buf();
            attachment.relative_path = relative_path.clone();
            attachment.modified_at = modified_at;
            if let Some((old_prefix, new_prefix)) =
                infer_path_migration(&cached_path, &relative_path)
            {
                register_path_migration(&mut path_migrations, old_prefix, new_prefix);
            }
            if !attachment_matches_scope(&attachment, include_folders, exclude_folders) {
                skipped_count += 1;
                continue;
            }
            if attachment.sensitivity.sensitive {
                attachment_sensitive_count += 1;
                continue;
            }
            scan_cache.attachments.insert(
                relative_path.clone(),
                ObsidianScanCacheAttachmentEntry {
                    bytes: attachment.bytes,
                    modified_at: attachment.modified_at,
                    attachment: attachment.clone(),
                },
            );
            attachments.push(attachment);
            continue;
        }
        if let Some((cached_path, cached_entry)) =
            find_cached_attachment_by_stat(&scan_cache, metadata.len(), modified_at)
        {
            attachment_cache_hits += 1;
            scan_cache.attachments.remove(&cached_path);
            let mut attachment = cached_entry.attachment;
            attachment.path = path.to_path_buf();
            attachment.relative_path = relative_path.clone();
            attachment.modified_at = modified_at;
            if let Some((old_prefix, new_prefix)) =
                infer_path_migration(&cached_path, &relative_path)
            {
                register_path_migration(&mut path_migrations, old_prefix, new_prefix);
            }
            if !attachment_matches_scope(&attachment, include_folders, exclude_folders) {
                skipped_count += 1;
                continue;
            }
            if attachment.sensitivity.sensitive {
                attachment_sensitive_count += 1;
                continue;
            }
            scan_cache.attachments.insert(
                relative_path.clone(),
                ObsidianScanCacheAttachmentEntry {
                    bytes: attachment.bytes,
                    modified_at: attachment.modified_at,
                    attachment: attachment.clone(),
                },
            );
            attachments.push(attachment);
            continue;
        }

        let raw = fs::read(path).with_context(|| format!("read {}", path.display()))?;
        let content_hash = hash_bytes(&raw);
        if let Some((cached_path, cached_entry)) =
            find_cached_attachment_by_content_hash(&scan_cache, &content_hash)
        {
            attachment_cache_hits += 1;
            scan_cache.attachments.remove(&cached_path);
            let mut attachment = cached_entry.attachment;
            attachment.path = path.to_path_buf();
            attachment.relative_path = relative_path.clone();
            attachment.bytes = metadata.len();
            attachment.modified_at = modified_at;
            attachment.content_hash = content_hash.clone();
            scan_cache.attachments.insert(
                relative_path.clone(),
                ObsidianScanCacheAttachmentEntry {
                    bytes: attachment.bytes,
                    modified_at: attachment.modified_at,
                    attachment: attachment.clone(),
                },
            );
            if !attachment_matches_scope(&attachment, include_folders, exclude_folders) {
                skipped_count += 1;
                continue;
            }
            if attachment.sensitivity.sensitive {
                attachment_sensitive_count += 1;
                continue;
            }
            attachments.push(attachment);
            continue;
        }

        if let Some(attachment) =
            parse_attachment_from_raw(vault, path, &raw, metadata.len(), modified_at, content_hash)?
        {
            if !attachment_matches_scope(&attachment, include_folders, exclude_folders) {
                skipped_count += 1;
                continue;
            }
            if attachment.sensitivity.sensitive {
                attachment_sensitive_count += 1;
                continue;
            }
            scan_cache.attachments.insert(
                attachment.relative_path.clone(),
                ObsidianScanCacheAttachmentEntry {
                    bytes: attachment.bytes,
                    modified_at: attachment.modified_at,
                    attachment: attachment.clone(),
                },
            );
            attachments.push(attachment);
        }
    }

    let cache_pruned = scan_cache
        .notes
        .keys()
        .filter(|path| !note_paths_seen.contains(*path))
        .count();
    let attachment_cache_pruned = if include_attachments {
        scan_cache
            .attachments
            .keys()
            .filter(|path| !attachment_paths_seen.contains(*path))
            .count()
    } else {
        0
    };
    scan_cache
        .notes
        .retain(|path, _| note_paths_seen.contains(path));
    if include_attachments {
        scan_cache
            .attachments
            .retain(|path, _| attachment_paths_seen.contains(path));
    }

    let mut note_index = HashMap::new();
    for (idx, note) in notes.iter().enumerate() {
        note_index.insert(note.normalized_title.clone(), idx);
        for alias in &note.aliases {
            note_index.entry(normalized_title(alias)).or_insert(idx);
        }
    }

    let mut backlink_count = 0usize;
    for idx in 0..notes.len() {
        let mut backlinks = HashSet::new();
        for source in &notes {
            let source_hits_current = source
                .links
                .iter()
                .any(|link| note_index.get(&normalized_title(link)).copied() == Some(idx));
            if source_hits_current {
                backlinks.insert(source.relative_path.clone());
            }
        }
        backlink_count += backlinks.len();
        let mut backlinks = backlinks.into_iter().collect::<Vec<_>>();
        backlinks.sort();
        notes[idx].backlinks = backlinks;
    }

    let scan = ObsidianVaultScan {
        vault: vault.to_path_buf(),
        project,
        namespace,
        workspace,
        visibility: visibility.unwrap_or_default(),
        note_count: notes.len(),
        sensitive_count,
        skipped_count,
        unchanged_count: 0,
        backlink_count,
        attachment_count: attachments.len(),
        attachment_sensitive_count,
        attachment_unchanged_count: 0,
        cache_hits,
        attachment_cache_hits,
        cache_pruned,
        attachment_cache_pruned,
        sensitive_notes,
        attachments,
        notes,
    };

    let _ = save_scan_cache(&cache_path, &scan_cache);
    Ok(scan)
}

pub fn load_sync_state(
    vault: impl AsRef<Path>,
    state_path: Option<PathBuf>,
) -> anyhow::Result<(PathBuf, ObsidianSyncState)> {
    let vault = vault.as_ref();
    let path = state_path.unwrap_or_else(|| default_sync_state_path(vault));
    if !path.exists() {
        return Ok((path, ObsidianSyncState::default()));
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let state: ObsidianSyncState =
        serde_json::from_str(&raw).context("parse obsidian sync state")?;
    Ok((path, state))
}

pub fn save_sync_state(path: impl AsRef<Path>, state: &ObsidianSyncState) -> anyhow::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(state).context("serialize obsidian sync state")?;
    fs::write(path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub fn load_scan_cache(path: impl AsRef<Path>) -> anyhow::Result<ObsidianScanCache> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(ObsidianScanCache::default());
    }
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let state: ObsidianScanCache =
        serde_json::from_str(&raw).context("parse obsidian scan cache")?;
    Ok(state)
}

pub fn save_scan_cache(path: impl AsRef<Path>, state: &ObsidianScanCache) -> anyhow::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(state).context("serialize obsidian scan cache")?;
    fs::write(path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn find_cached_note_by_content_hash(
    cache: &ObsidianScanCache,
    content_hash: &str,
) -> Option<(String, ObsidianScanCacheNoteEntry)> {
    cache
        .notes
        .iter()
        .find(|(_, entry)| entry.note.content_hash == content_hash)
        .map(|(path, entry)| (path.clone(), entry.clone()))
}

fn find_cached_attachment_by_content_hash(
    cache: &ObsidianScanCache,
    content_hash: &str,
) -> Option<(String, ObsidianScanCacheAttachmentEntry)> {
    cache
        .attachments
        .iter()
        .find(|(_, entry)| entry.attachment.content_hash == content_hash)
        .map(|(path, entry)| (path.clone(), entry.clone()))
}

fn find_cached_note_by_stat(
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

fn find_cached_attachment_by_stat(
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

fn find_cached_note_by_path_migration(
    cache: &ObsidianScanCache,
    current_path: &str,
    bytes: u64,
    modified_at: Option<DateTime<Utc>>,
    migrations: &[(String, String)],
) -> Option<(String, ObsidianScanCacheNoteEntry)> {
    find_cached_by_path_migration(&cache.notes, current_path, bytes, modified_at, migrations)
}

fn find_cached_attachment_by_path_migration(
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

fn register_path_migration(
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

fn infer_path_migration(old_path: &str, new_path: &str) -> Option<(String, String)> {
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

trait HasScanCacheMetadata {
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

pub fn build_import_preview(
    scan: ObsidianVaultScan,
    sync_state: &ObsidianSyncState,
    sync_state_path: PathBuf,
) -> (
    ObsidianImportPreview,
    Vec<CandidateMemoryRequest>,
    Vec<ObsidianNote>,
) {
    let mut candidates = Vec::with_capacity(scan.notes.len());
    let mut changed_notes = Vec::new();
    let mut note_index = HashMap::new();
    let mut unchanged_count = 0usize;
    let mut duplicate_count = 0usize;
    let mut seen_fingerprints = HashSet::<String>::new();

    for (idx, note) in scan.notes.iter().enumerate() {
        let entry = sync_state.entries.get(&note.relative_path);
        if entry.is_some_and(|entry| entry.content_hash == note.content_hash) {
            unchanged_count += 1;
            continue;
        }
        let fingerprint = note_import_fingerprint(note);
        if !seen_fingerprints.insert(fingerprint) {
            duplicate_count += 1;
            continue;
        }
        note_index.insert(note.normalized_title.clone(), idx);
        for alias in &note.aliases {
            note_index.entry(normalized_title(alias)).or_insert(idx);
        }
        candidates.push(build_note_request(
            note,
            scan.project.clone(),
            scan.namespace.clone(),
            scan.workspace.clone(),
            Some(scan.visibility),
            scan.vault.clone(),
            entry.and_then(|entry| entry.item_id),
        ));
        changed_notes.push(note.clone());
    }

    (
        ObsidianImportPreview {
            scan: ObsidianVaultScan {
                unchanged_count,
                ..scan
            },
            candidates: candidates.clone(),
            note_index,
            unchanged_count,
            duplicate_count,
            sync_state_path,
        },
        candidates,
        changed_notes,
    )
}

fn note_import_fingerprint(note: &ObsidianNote) -> String {
    let tags = note
        .tags
        .iter()
        .map(|tag| normalized_title(tag))
        .collect::<Vec<_>>()
        .join("|");
    let aliases = note
        .aliases
        .iter()
        .map(|alias| normalized_title(alias))
        .collect::<Vec<_>>()
        .join("|");
    format!(
        "{:?}|{}|{}|{}",
        note.kind,
        normalized_title(&note.excerpt),
        tags,
        aliases
    )
}

pub fn resolve_attachment_match(
    attachment: &ObsidianAttachment,
    notes: &[ObsidianNote],
    note_index: &HashMap<String, usize>,
) -> Option<ObsidianAttachmentMatch> {
    let stem = attachment
        .path
        .file_stem()
        .and_then(|value| value.to_str())
        .map(normalized_title);
    let parent = attachment
        .path
        .parent()
        .and_then(|value| value.file_name())
        .and_then(|value| value.to_str())
        .map(normalized_title);
    let relative = normalized_title(
        attachment
            .relative_path
            .strip_suffix(
                attachment
                    .path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| format!(".{ext}"))
                    .as_deref()
                    .unwrap_or(""),
            )
            .unwrap_or(&attachment.relative_path),
    );

    let mut best: Option<(usize, ObsidianAttachmentMatch)> = None;
    for (candidate, relation_kind, reason) in [
        (
            stem.as_deref(),
            EntityRelationKind::DerivedFrom,
            "attachment stem matches note",
        ),
        (
            parent.as_deref(),
            EntityRelationKind::Related,
            "attachment parent folder matches note",
        ),
        (
            Some(relative.as_str()),
            EntityRelationKind::Related,
            "attachment relative path matches note",
        ),
    ] {
        let Some(key) = candidate else {
            continue;
        };
        if let Some(&idx) = note_index.get(key) {
            let score = match relation_kind {
                EntityRelationKind::DerivedFrom => 3,
                EntityRelationKind::Related => 2,
                _ => 1,
            };
            let should_replace = best
                .as_ref()
                .map(|(best_score, _)| score > *best_score)
                .unwrap_or(true);
            if should_replace {
                best = Some((
                    score,
                    ObsidianAttachmentMatch {
                        note_index: idx,
                        relation_kind,
                        reason: reason.to_string(),
                    },
                ));
            }
        }
    }

    best.map(|(_, match_)| match_)
        .or_else(|| fallback_attachment_match(attachment, notes, note_index))
}

#[allow(clippy::too_many_arguments)]
pub fn build_attachment_request(
    attachment: &ObsidianAttachment,
    project: Option<String>,
    namespace: Option<String>,
    workspace: Option<String>,
    visibility: Option<MemoryVisibility>,
    vault: PathBuf,
    linked_note: Option<&ObsidianNote>,
    sidecar_track_id: Option<Uuid>,
) -> CandidateMemoryRequest {
    let scope = if project.is_some() {
        MemoryScope::Project
    } else {
        MemoryScope::Synced
    };
    let source_path = vault.join(&attachment.path).display().to_string();
    let mut tags = vec![
        "obsidian".to_string(),
        "vault_attachment".to_string(),
        format!("asset_kind={}", attachment.asset_kind),
    ];
    if let Some(note) = linked_note {
        tags.push("linked_note".to_string());
        tags.push(format!("note={}", note.normalized_title));
    }
    if let Some(track_id) = sidecar_track_id {
        tags.push(format!("sidecar_track_id={track_id}"));
    }
    let mut content = String::new();
    content.push_str(&format!(
        "Obsidian attachment: {}\n",
        attachment.relative_path
    ));
    content.push_str(&format!("Vault path: {}\n", attachment.relative_path));
    content.push_str(&format!("Asset kind: {}\n", attachment.asset_kind));
    if let Some(folder_path) = attachment.folder_path.as_deref() {
        content.push_str(&format!("Folder path: {}\n", folder_path));
    }
    if let Some(mime) = attachment.mime.as_deref() {
        content.push_str(&format!("Mime: {mime}\n"));
    }
    content.push_str(&format!("Bytes: {}\n", attachment.bytes));
    if let Some(note) = linked_note {
        content.push_str(&format!("Linked note: {}\n", note.title));
        content.push_str(&format!("Linked note path: {}\n", note.relative_path));
    }
    if let Some(track_id) = sidecar_track_id {
        content.push_str(&format!("Sidecar track id: {track_id}\n"));
    }
    content.push_str("This attachment was imported from an Obsidian vault.\n");

    CandidateMemoryRequest {
        content,
        kind: MemoryKind::Fact,
        scope,
        project,
        namespace,
        workspace,
        visibility,
        belief_branch: None,
        source_agent: Some("obsidian".to_string()),
        source_system: Some("obsidian".to_string()),
        source_path: Some(source_path),
        source_quality: Some(SourceQuality::Derived),
        confidence: Some(0.88),
        ttl_seconds: None,
        last_verified_at: Some(Utc::now()),
        supersedes: Vec::new(),
        tags,
    }
}

pub fn default_writeback_path(vault: &Path, explain: &ExplainMemoryResponse) -> PathBuf {
    let kind = format!("{:?}", explain.item.kind).to_lowercase();
    let short_id = short_uuid(explain.item.id);
    vault
        .join(".memd")
        .join("writeback")
        .join(format!("{kind}-{short_id}.md"))
}

pub fn default_compiled_note_path(vault: &Path, query: &str) -> PathBuf {
    vault
        .join(".memd")
        .join("compiled")
        .join(format!("{}.md", slugify(query)))
}

pub fn default_compiled_memory_path(vault: &Path, explain: &ExplainMemoryResponse) -> PathBuf {
    let base = explain
        .item
        .tags
        .first()
        .cloned()
        .unwrap_or_else(|| format!("{:?}", explain.item.kind).to_lowercase());
    let slug = slugify(&format!("{base}-{}", short_uuid(explain.item.id)));
    vault
        .join(".memd")
        .join("compiled")
        .join("memory")
        .join(format!("{slug}.md"))
}

pub fn find_compiled_memory_path_by_id(vault: &Path, id: Uuid) -> Option<PathBuf> {
    let memory_dir = vault.join(".memd").join("compiled").join("memory");
    let short_id = short_uuid(id);
    let needle = format!("-{short_id}.md");
    let entries = fs::read_dir(&memory_dir).ok()?;
    entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .is_some_and(|value| value.ends_with(&needle))
        })
}

pub fn default_compiled_index_path(vault: &Path) -> PathBuf {
    vault.join(".memd").join("compiled").join("INDEX.md")
}

pub fn default_handoff_path(vault: &Path, snapshot: &crate::ResumeSnapshot) -> PathBuf {
    let base = snapshot
        .workspace
        .clone()
        .or_else(|| snapshot.project.clone())
        .unwrap_or_else(|| "shared-handoff".to_string());
    let slug = slugify(&format!("{base}-{}", Utc::now().format("%Y%m%d-%H%M%S")));
    vault
        .join(".memd")
        .join("handoffs")
        .join(format!("{slug}.md"))
}

pub fn resolve_open_path(vault: &Path, target: &Path) -> PathBuf {
    if target.is_absolute() {
        target.to_path_buf()
    } else {
        vault.join(target)
    }
}

pub fn build_open_uri(path: &Path, pane_type: Option<&str>) -> anyhow::Result<String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .context("read current directory for obsidian uri")?
            .join(path)
    };
    let mut uri = format!(
        "obsidian://open?path={}",
        encode_uri_component(&absolute.to_string_lossy())
    );
    if let Some(pane_type) = pane_type {
        let normalized = pane_type.trim();
        if !normalized.is_empty() {
            uri.push_str("&paneType=");
            uri.push_str(&encode_uri_component(normalized));
        }
    }
    Ok(uri)
}

fn encode_uri_component(value: &str) -> String {
    byte_serialize(value.as_bytes()).collect()
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in value.trim().chars() {
        let normalized = ch.to_ascii_lowercase();
        if normalized.is_ascii_alphanumeric() {
            slug.push(normalized);
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

pub fn open_uri(uri: &str) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg(uri);
        command
    };

    #[cfg(target_os = "linux")]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(uri);
        command
    };

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("cmd");
        command.args(["/C", "start", "", uri]);
        command
    };

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        anyhow::bail!("opening Obsidian URIs is not supported on this platform");
    }

    let status = command
        .status()
        .with_context(|| format!("launch Obsidian URI {uri}"))?;
    if !status.success() {
        anyhow::bail!("Obsidian URI launcher exited with status {status}");
    }
    Ok(())
}

pub fn build_writeback_markdown(
    vault: &Path,
    explain: &ExplainMemoryResponse,
    entity: Option<&memd_schema::MemoryEntityRecord>,
) -> (String, String) {
    let title = explain
        .item
        .tags
        .iter()
        .find(|tag| !tag.starts_with("source_"))
        .cloned()
        .unwrap_or_else(|| format!("{:?}", explain.item.kind));
    let mut markdown = String::new();
    markdown.push_str("---\n");
    markdown.push_str(&format!("id: {}\n", explain.item.id));
    markdown.push_str(&format!("title: {}\n", title));
    markdown.push_str(&format!("kind: {:?}\n", explain.item.kind).to_lowercase());
    markdown.push_str(&format!("scope: {:?}\n", explain.item.scope).to_lowercase());
    if let Some(project) = explain.item.project.as_deref() {
        markdown.push_str(&format!("project: {}\n", project));
    }
    if let Some(namespace) = explain.item.namespace.as_deref() {
        markdown.push_str(&format!("namespace: {}\n", namespace));
    }
    if let Some(workspace) = explain.item.workspace.as_deref() {
        markdown.push_str(&format!("workspace: {}\n", workspace));
    }
    markdown.push_str(&format!(
        "visibility: {}\n",
        format_visibility(explain.item.visibility)
    ));
    if let Some(source_system) = explain.item.source_system.as_deref() {
        markdown.push_str(&format!("source_system: {}\n", source_system));
    }
    if let Some(source_agent) = explain.item.source_agent.as_deref() {
        markdown.push_str(&format!("source_agent: {}\n", source_agent));
    }
    if let Some(source_path) = explain.item.source_path.as_deref() {
        markdown.push_str(&format!("source_path: {}\n", source_path));
    }
    markdown
        .push_str(&format!("source_quality: {:?}\n", explain.item.source_quality).to_lowercase());
    markdown.push_str(&format!("status: {:?}\n", explain.item.status).to_lowercase());
    markdown.push_str(&format!("stage: {:?}\n", explain.item.stage).to_lowercase());
    markdown.push_str(&format!("redundancy_key: {}\n", explain.redundancy_key));
    markdown.push_str(&format!("canonical_key: {}\n", explain.canonical_key));
    markdown.push_str("tags:\n");
    for tag in &explain.item.tags {
        markdown.push_str(&format!("  - {}\n", tag));
    }
    markdown.push_str("---\n\n");
    markdown.push_str(&format!("# {}\n\n", title));
    markdown.push_str("## Summary\n\n");
    markdown.push_str(&explain.item.content);
    if let Some(source_link) = explain
        .item
        .source_path
        .as_deref()
        .and_then(|path| source_wikilink_for_path(vault, Path::new(path)))
    {
        markdown.push_str("\n\n## Source Note\n\n");
        markdown.push_str(&format!("- {}\n", source_link));
    }
    markdown.push_str("\n\n## Why This Exists\n\n");
    for reason in &explain.reasons {
        markdown.push_str(&format!("- {}\n", reason));
    }
    markdown.push_str(&format!(
        "- visibility: {}\n",
        format_visibility(explain.item.visibility)
    ));
    if let Some(workspace) = explain.item.workspace.as_deref() {
        markdown.push_str(&format!("- workspace: {}\n", workspace));
    }
    if !explain.policy_hooks.is_empty() {
        markdown.push_str("\n## Policy Hooks\n\n");
        for hook in &explain.policy_hooks {
            markdown.push_str(&format!("- {}\n", hook));
        }
    }
    if let Some(entity) = entity {
        markdown.push_str("\n## Entity\n\n");
        markdown.push_str(&format!("- entity: {}\n", entity.id));
        markdown.push_str(&format!("- type: {}\n", entity.entity_type));
        markdown.push_str(&format!("- salience: {:.2}\n", entity.salience_score));
        markdown.push_str(&format!("- rehearsal: {}\n", entity.rehearsal_count));
        markdown.push_str(&format!("- state version: {}\n", entity.state_version));
    }
    if !explain.events.is_empty() {
        markdown.push_str("\n## Recent Events\n\n");
        for event in explain.events.iter().take(5) {
            markdown.push_str(&format!(
                "- {} {} {}\n",
                event.occurred_at.to_rfc3339(),
                event.event_type,
                event.summary
            ));
        }
    }
    if !explain.sources.is_empty() {
        markdown.push_str("\n## Source Lanes\n\n");
        for source in explain.sources.iter().take(5) {
            markdown.push_str(&format!(
                "- {} / {} | workspace {} | visibility {} | trust {:.2} | avg confidence {:.2} | items {}\n",
                source.source_agent.as_deref().unwrap_or("none"),
                source.source_system.as_deref().unwrap_or("none"),
                source.workspace.as_deref().unwrap_or("none"),
                format_visibility(source.visibility),
                source.trust_score,
                source.avg_confidence,
                source.item_count
            ));
        }
    }
    if !explain.branch_siblings.is_empty() {
        markdown.push_str("\n## Sibling Branches\n\n");
        for sibling in explain.branch_siblings.iter().take(5) {
            markdown.push_str(&format!(
                "- {} | {} | confidence {:.2} | status {:?} | {}\n",
                sibling.belief_branch.as_deref().unwrap_or("none"),
                sibling.id,
                sibling.confidence,
                sibling.status,
                if sibling.preferred {
                    "preferred"
                } else {
                    "candidate"
                }
            ));
        }
    }
    if !explain.rehydration.is_empty() {
        markdown.push_str("\n## Rehydration Lane\n\n");
        for artifact in explain.rehydration.iter().take(8) {
            markdown.push_str(&format!(
                "- **{}** {}: {}\n",
                artifact.kind, artifact.label, artifact.summary
            ));
            if let Some(reason) = artifact.reason.as_deref() {
                markdown.push_str(&format!("  - reason: {}\n", reason));
            }
            if artifact.source_path.is_some()
                || artifact.source_agent.is_some()
                || artifact.source_system.is_some()
            {
                markdown.push_str("  - source: ");
                markdown.push_str(artifact.source_agent.as_deref().unwrap_or("none"));
                markdown.push_str(" / ");
                markdown.push_str(artifact.source_system.as_deref().unwrap_or("none"));
                if let Some(path) = artifact.source_path.as_deref() {
                    markdown.push_str(" / ");
                    markdown.push_str(path);
                }
                markdown.push('\n');
            }
            if let Some(path) = artifact.source_path.as_deref()
                && let Some(link) = source_wikilink_for_path(vault, Path::new(path))
            {
                markdown.push_str(&format!("  - wiki: {}\n", link));
            }
        }
    }
    (title, markdown)
}

pub fn build_compiled_note_markdown(
    vault: &Path,
    query: &str,
    response: &SearchMemoryResponse,
    semantic: Option<&memd_rag::RagRetrieveResponse>,
) -> (String, String) {
    let title = query.trim().to_string();
    let mut markdown = String::new();
    markdown.push_str("---\n");
    markdown.push_str(&format!("title: {}\n", title));
    markdown.push_str("source_system: memd\n");
    markdown.push_str("source_agent: memd\n");
    markdown.push_str(&format!("route: {:?}\n", response.route).to_lowercase());
    markdown.push_str(&format!("intent: {:?}\n", response.intent).to_lowercase());
    markdown.push_str(&format!("items: {}\n", response.items.len()));
    markdown.push_str("---\n\n");
    markdown.push_str(&format!("# {}\n\n", title));
    markdown.push_str("## Query\n\n");
    markdown.push_str(query);
    markdown.push_str("\n\n## Matching Memory\n\n");
    for item in response.items.iter().take(16) {
        markdown.push_str(&format!(
            "### {} `{}`\n\n",
            item.tags
                .first()
                .cloned()
                .unwrap_or_else(|| format!("{:?}", item.kind).to_lowercase()),
            item.id
        ));
        markdown.push_str(
            &format!(
                "- kind: {:?}\n- scope: {:?}\n- status: {:?}\n- confidence: {:.2}\n",
                item.kind, item.scope, item.status, item.confidence
            )
            .to_lowercase(),
        );
        if let Some(project) = item.project.as_deref() {
            markdown.push_str(&format!("- project: {}\n", project));
        }
        if let Some(namespace) = item.namespace.as_deref() {
            markdown.push_str(&format!("- namespace: {}\n", namespace));
        }
        if let Some(workspace) = item.workspace.as_deref() {
            markdown.push_str(&format!("- workspace: {}\n", workspace));
        }
        markdown.push_str(&format!(
            "- visibility: {}\n",
            format_visibility(item.visibility)
        ));
        if let Some(branch) = item.belief_branch.as_deref() {
            markdown.push_str(&format!("- belief_branch: {}\n", branch));
        }
        if let Some(source_path) = item.source_path.as_deref() {
            markdown.push_str(&format!("- source_path: {}\n", source_path));
            if let Some(link) = source_wikilink_for_path(vault, Path::new(source_path)) {
                markdown.push_str(&format!("- source_note: {}\n", link));
            }
        }
        markdown.push('\n');
        markdown.push_str(&item.content);
        markdown.push_str("\n\n");
    }
    if let Some(semantic) = semantic.filter(|semantic| !semantic.items.is_empty()) {
        markdown.push_str("## Semantic Recall\n\n");
        for item in semantic.items.iter().take(8) {
            markdown.push_str(&format!(
                "- {}{}\n",
                compact_markdown_text(&item.content, 220),
                item.source
                    .as_deref()
                    .map(|source| format!(" | source {}", compact_markdown_text(source, 64)))
                    .unwrap_or_default()
            ));
            markdown.push_str(&format!("  - score: {:.2}\n", item.score));
        }
        markdown.push('\n');
    }
    (title, markdown)
}

pub fn build_compiled_memory_markdown(
    vault: &Path,
    explain: &ExplainMemoryResponse,
) -> (String, String) {
    let title = format!(
        "{} {}",
        explain
            .item
            .tags
            .first()
            .cloned()
            .unwrap_or_else(|| format!("{:?}", explain.item.kind).to_lowercase()),
        short_uuid(explain.item.id)
    );
    let (_, body) = build_writeback_markdown(vault, explain, explain.entity.as_ref());
    let mut markdown = String::new();
    markdown.push_str("---\n");
    markdown.push_str(&format!("title: {}\n", title));
    markdown.push_str("source_system: memd\n");
    markdown.push_str("source_agent: memd\n");
    markdown.push_str("compiled_from: explain\n");
    markdown.push_str(&format!("memory_id: {}\n", explain.item.id));
    markdown.push_str("---\n\n");
    markdown.push_str("# Compiled Memory Page\n\n");
    markdown.push_str(&format!(
        "- memory: `{}`\n- branch: {}\n- visibility: {}\n- workspace: {}\n- confidence: {:.2}\n- rehydration: {}\n\n",
        explain.item.id,
        explain.item.belief_branch.as_deref().unwrap_or("none"),
        format_visibility(explain.item.visibility),
        explain.item.workspace.as_deref().unwrap_or("none"),
        explain.item.confidence,
        explain.rehydration.len()
    ));
    markdown.push_str(&body);
    (title, markdown)
}

pub fn parse_compiled_artifact_metadata(markdown: &str) -> (Option<String>, Option<usize>) {
    let mut title = None;
    let mut item_count = None;
    let mut in_frontmatter = false;

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            if in_frontmatter {
                break;
            }
            in_frontmatter = true;
            continue;
        }
        if !in_frontmatter {
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("title:") {
            let value = rest.trim().trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                title = Some(value.to_string());
            }
        } else if let Some(rest) = trimmed.strip_prefix("items:") {
            item_count = rest.trim().parse::<usize>().ok();
        }
    }

    (title, item_count)
}

pub fn read_compiled_artifact_metadata(
    path: &Path,
) -> anyhow::Result<(Option<String>, Option<usize>)> {
    let markdown = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(parse_compiled_artifact_metadata(&markdown))
}

pub fn build_compiled_index_markdown(
    existing: Option<&str>,
    entry_kind: &str,
    title: &str,
    note_path: &Path,
    item_count: usize,
) -> String {
    let note_title = note_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(title);
    let entry = format!(
        "- [[{}]] | {}: {} | items: {}",
        note_title, entry_kind, title, item_count
    );
    let mut entries = existing
        .map(|content| {
            content
                .lines()
                .filter(|line| line.starts_with("- [["))
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    entries.retain(|line| !line.contains(&format!("[[{note_title}]]")));
    entries.push(entry);
    entries.sort();

    let mut markdown = String::new();
    markdown.push_str("---\n");
    markdown.push_str("title: Compiled Wiki Index\n");
    markdown.push_str("source_system: memd\n");
    markdown.push_str("source_agent: memd\n");
    markdown.push_str("---\n\n");
    markdown.push_str("# Compiled Wiki Index\n\n");
    markdown.push_str("Generated pages built from `memd` search and explain flows.\n\n");
    for entry in entries {
        markdown.push_str(&entry);
        markdown.push('\n');
    }
    markdown
}

pub fn build_handoff_markdown(
    _vault: &Path,
    snapshot: &crate::ResumeSnapshot,
    sources: &SourceMemoryResponse,
) -> (String, String) {
    let title = format!(
        "Handoff {} {}",
        snapshot.workspace.as_deref().unwrap_or("shared"),
        Utc::now().format("%Y-%m-%d %H:%M")
    );
    let mut markdown = String::new();
    markdown.push_str("---\n");
    markdown.push_str(&format!("title: {}\n", title));
    markdown.push_str("source_system: memd\n");
    markdown.push_str("source_agent: memd\n");
    if let Some(project) = snapshot.project.as_deref() {
        markdown.push_str(&format!("project: {}\n", project));
    }
    if let Some(namespace) = snapshot.namespace.as_deref() {
        markdown.push_str(&format!("namespace: {}\n", namespace));
    }
    if let Some(workspace) = snapshot.workspace.as_deref() {
        markdown.push_str(&format!("workspace: {}\n", workspace));
    }
    if let Some(visibility) = snapshot.visibility.as_deref() {
        markdown.push_str(&format!("visibility: {}\n", visibility));
    }
    markdown.push_str(&format!("route: {}\n", snapshot.route));
    markdown.push_str(&format!("intent: {}\n", snapshot.intent));
    markdown.push_str(&format!(
        "working_items: {}\n",
        snapshot.working.records.len()
    ));
    markdown.push_str(&format!(
        "rehydration_items: {}\n",
        snapshot.working.rehydration_queue.len()
    ));
    markdown.push_str(&format!("inbox_items: {}\n", snapshot.inbox.items.len()));
    markdown.push_str(&format!(
        "workspace_lanes: {}\n",
        snapshot.workspaces.workspaces.len()
    ));
    markdown.push_str(&format!(
        "semantic_hits: {}\n",
        snapshot
            .semantic
            .as_ref()
            .map(|semantic| semantic.items.len())
            .unwrap_or(0)
    ));
    markdown.push_str(&format!("source_lanes: {}\n", sources.sources.len()));
    markdown.push_str("---\n\n");
    markdown.push_str(&format!("# {}\n\n", title));
    markdown.push_str("## Resume Frame\n\n");
    markdown.push_str(&format!(
        "- project: {}\n- namespace: {}\n- agent: {}\n- workspace: {}\n- visibility: {}\n- route: {}\n- intent: {}\n",
        snapshot.project.as_deref().unwrap_or("none"),
        snapshot.namespace.as_deref().unwrap_or("none"),
        snapshot.agent.as_deref().unwrap_or("none"),
        snapshot.workspace.as_deref().unwrap_or("none"),
        snapshot.visibility.as_deref().unwrap_or("all"),
        snapshot.route,
        snapshot.intent
    ));

    markdown.push_str("\n## W\n\n");
    if snapshot.working.records.is_empty() {
        markdown.push_str("- none\n");
    } else {
        let records = snapshot
            .working
            .records
            .iter()
            .take(2)
            .map(|record| record.record.clone())
            .collect::<Vec<_>>();
        markdown.push_str(&format!("- w={}", records.join(" | ")));
        if snapshot.working.records.len() > 2 {
            markdown.push_str(&format!(" (+{} more)", snapshot.working.records.len() - 2));
        }
        markdown.push('\n');
    }

    let mut ri_parts = Vec::new();
    if !snapshot.working.rehydration_queue.is_empty() {
        for artifact in snapshot.working.rehydration_queue.iter().take(8) {
            ri_parts.push(format!("r={}:{}", artifact.label, artifact.summary));
            if let Some(path) = artifact.source_path.as_deref() {
                ri_parts.push(format!("src={}", path));
            }
            if let Some(reason) = artifact.reason.as_deref() {
                ri_parts.push(format!("r={}", reason));
            }
        }
    }
    if !snapshot.inbox.items.is_empty() {
        for item in snapshot.inbox.items.iter().take(8) {
            ri_parts.push(format!(
                "i={:?}/{:?}:cf{:.2}",
                item.item.kind, item.item.status, item.item.confidence
            ));
            if !item.reasons.is_empty() {
                ri_parts.push(format!("r={}", item.reasons.join(", ")));
            }
        }
    }
    if !ri_parts.is_empty() {
        markdown.push_str("\n## RI\n\n");
        markdown.push_str(&format!("- {}\n", ri_parts.join(" | ")));
    }

    if let Some(first) = snapshot.workspaces.workspaces.first() {
        markdown.push_str("\n## L\n\n");
        let extras = snapshot.workspaces.workspaces.len() - 1;
        markdown.push_str(&format!(
            "- l={}/{}/{} | v={} | it={} | tr={:.2} | cf={:.2}{} \n",
            first.project.as_deref().unwrap_or("none"),
            first.namespace.as_deref().unwrap_or("none"),
            first.workspace.as_deref().unwrap_or("none"),
            format_visibility(first.visibility),
            first.item_count,
            first.trust_score,
            first.avg_confidence,
            if extras > 0 {
                format!(" (+{} more)", extras)
            } else {
                "".to_string()
            }
        ));
    }

    if let Some(semantic) = snapshot
        .semantic
        .as_ref()
        .filter(|semantic| !semantic.items.is_empty())
    {
        markdown.push_str("\n## S\n\n");
        for item in semantic.items.iter().take(6) {
            markdown.push_str(&format!(
                "- {}{}\n",
                compact_markdown_text(&item.content, 220),
                item.source
                    .as_deref()
                    .map(|source| format!(" | source {}", compact_markdown_text(source, 64)))
                    .unwrap_or_default()
            ));
            markdown.push_str(&format!("  - score: {:.2}\n", item.score));
        }
    }

    if !sources.sources.is_empty() {
        markdown.push_str("\n## C\n\n");
        for source in sources.sources.iter().take(8) {
            markdown.push_str(&format!(
                "- {} / {} | workspace {} | visibility {} | items {} | trust {:.2} | confidence {:.2}\n",
                source.source_agent.as_deref().unwrap_or("none"),
                source.source_system.as_deref().unwrap_or("none"),
                source.workspace.as_deref().unwrap_or("none"),
                format_visibility(source.visibility),
                source.item_count,
                source.trust_score,
                source.avg_confidence
            ));
        }
    }

    (title, markdown)
}

fn compact_markdown_text(value: &str, max_chars: usize) -> String {
    let collapsed = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= max_chars {
        return collapsed;
    }

    let mut output = String::new();
    for ch in collapsed.chars() {
        if output.chars().count() >= max_chars.saturating_sub(1) {
            break;
        }
        output.push(ch);
    }
    output.push('…');
    output
}

fn source_wikilink_for_path(vault: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(vault).ok()?;
    let title = relative
        .file_stem()
        .and_then(|value| value.to_str())
        .map(|value| value.replace(['[', ']'], ""))?;
    Some(format!("[[{}]]", title))
}

fn format_visibility(value: MemoryVisibility) -> &'static str {
    match value {
        MemoryVisibility::Private => "private",
        MemoryVisibility::Workspace => "workspace",
        MemoryVisibility::Public => "public",
    }
}

pub fn build_note_mirror_markdown(
    note: &ObsidianNote,
    item_id: Option<Uuid>,
    entity_id: Option<Uuid>,
    workspace: Option<&str>,
    visibility: Option<MemoryVisibility>,
) -> (String, String) {
    let title = note.title.clone();
    let mut markdown = String::new();
    markdown.push_str("---\n");
    markdown.push_str(&format!("title: {}\n", note.title));
    markdown.push_str(&format!("path: {}\n", note.relative_path));
    markdown.push_str(&format!("kind: {:?}\n", note.kind).to_lowercase());
    if let Some(item_id) = item_id {
        markdown.push_str(&format!("item_id: {}\n", item_id));
    }
    if let Some(entity_id) = entity_id {
        markdown.push_str(&format!("entity_id: {}\n", entity_id));
    }
    if let Some(workspace) = workspace {
        markdown.push_str(&format!("workspace: {}\n", workspace));
    }
    if let Some(visibility) = visibility {
        markdown.push_str(&format!("visibility: {}\n", format_visibility(visibility)));
    }
    if let Some(folder_path) = note.folder_path.as_deref() {
        markdown.push_str(&format!("folder: {}\n", folder_path));
    }
    markdown.push_str(&format!("folder_depth: {}\n", note.folder_depth));
    markdown.push_str(&format!("content_hash: {}\n", note.content_hash));
    markdown.push_str(&format!("bytes: {}\n", note.bytes));
    if let Some(modified_at) = note.modified_at {
        markdown.push_str(&format!("modified_at: {}\n", modified_at.to_rfc3339()));
    }
    markdown.push_str("source_system: obsidian\n");
    markdown.push_str("source_agent: obsidian\n");
    markdown.push_str("---\n\n");
    markdown.push_str(&format!("# {}\n\n", note.title));
    markdown.push_str("## Excerpt\n\n");
    markdown.push_str(&note.excerpt);
    if workspace.is_some() || visibility.is_some() {
        markdown.push_str("\n\n## Shared Lane\n\n");
        if let Some(workspace) = workspace {
            markdown.push_str(&format!("- workspace: {}\n", workspace));
        }
        if let Some(visibility) = visibility {
            markdown.push_str(&format!(
                "- visibility: {}\n",
                format_visibility(visibility)
            ));
        }
    }
    if !note.aliases.is_empty() {
        markdown.push_str("\n\n## Aliases\n\n");
        for alias in &note.aliases {
            markdown.push_str(&format!("- {}\n", alias));
        }
    }
    if !note.links.is_empty() {
        markdown.push_str("\n\n## Links\n\n");
        for link in &note.links {
            markdown.push_str(&format!("- [[{}]]\n", link));
        }
    }
    if !note.backlinks.is_empty() {
        markdown.push_str("\n\n## Backlinks\n\n");
        for backlink in &note.backlinks {
            markdown.push_str(&format!("- {}\n", backlink));
        }
    }
    (title, markdown)
}

pub fn build_attachment_mirror_markdown(
    attachment: &ObsidianAttachment,
    item_id: Option<Uuid>,
    entity_id: Option<Uuid>,
    linked_note: Option<&ObsidianNote>,
    track_id: Option<Uuid>,
    workspace: Option<&str>,
    visibility: Option<MemoryVisibility>,
) -> (String, String) {
    let title = Path::new(&attachment.relative_path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("attachment")
        .to_string();
    let mut markdown = String::new();
    markdown.push_str("---\n");
    markdown.push_str(&format!("title: {}\n", title));
    markdown.push_str(&format!("path: {}\n", attachment.relative_path));
    markdown.push_str(&format!("asset_kind: {}\n", attachment.asset_kind));
    if let Some(mime) = attachment.mime.as_deref() {
        markdown.push_str(&format!("mime: {}\n", mime));
    }
    markdown.push_str(&format!("bytes: {}\n", attachment.bytes));
    markdown.push_str(&format!("content_hash: {}\n", attachment.content_hash));
    if let Some(modified_at) = attachment.modified_at {
        markdown.push_str(&format!("modified_at: {}\n", modified_at.to_rfc3339()));
    }
    if let Some(item_id) = item_id {
        markdown.push_str(&format!("item_id: {}\n", item_id));
    }
    if let Some(entity_id) = entity_id {
        markdown.push_str(&format!("entity_id: {}\n", entity_id));
    }
    if let Some(track_id) = track_id {
        markdown.push_str(&format!("track_id: {}\n", track_id));
    }
    if let Some(workspace) = workspace {
        markdown.push_str(&format!("workspace: {}\n", workspace));
    }
    if let Some(visibility) = visibility {
        markdown.push_str(&format!("visibility: {}\n", format_visibility(visibility)));
    }
    if let Some(note) = linked_note {
        markdown.push_str(&format!("linked_note: {}\n", note.title));
        markdown.push_str(&format!("linked_note_path: {}\n", note.relative_path));
    }
    markdown.push_str("source_system: obsidian\n");
    markdown.push_str("source_agent: obsidian\n");
    markdown.push_str("---\n\n");
    markdown.push_str(&format!("# {}\n\n", title));
    markdown.push_str("## Attachment\n\n");
    markdown.push_str(&format!("- path: {}\n", attachment.relative_path));
    markdown.push_str(&format!("- kind: {}\n", attachment.asset_kind));
    if let Some(workspace) = workspace {
        markdown.push_str(&format!("- workspace: {}\n", workspace));
    }
    if let Some(visibility) = visibility {
        markdown.push_str(&format!(
            "- visibility: {}\n",
            format_visibility(visibility)
        ));
    }
    if let Some(folder_path) = attachment.folder_path.as_deref() {
        markdown.push_str(&format!("- folder: {}\n", folder_path));
    }
    if let Some(note) = linked_note {
        markdown.push_str(&format!("- linked note: {}\n", note.title));
    }
    (title, markdown)
}

pub fn write_markdown(path: impl AsRef<Path>, content: &str) -> anyhow::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub fn build_roundtrip_annotation(
    note: &ObsidianNote,
    item_id: Option<Uuid>,
    entity_id: Option<Uuid>,
    workspace: Option<&str>,
    visibility: Option<MemoryVisibility>,
) -> String {
    let mut block = String::new();
    block.push_str(MEMD_ROUNDTRIP_BEGIN);
    block.push('\n');
    block.push_str("## memd sync\n\n");
    block.push_str(&format!("- note: {}\n", note.title));
    block.push_str(&format!("- path: {}\n", note.relative_path));
    block.push_str(&format!("- kind: {:?}\n", note.kind).to_lowercase());
    if let Some(item_id) = item_id {
        block.push_str(&format!("- item_id: {}\n", item_id));
    }
    if let Some(entity_id) = entity_id {
        block.push_str(&format!("- entity_id: {}\n", entity_id));
    }
    if let Some(workspace) = workspace {
        block.push_str(&format!("- workspace: {}\n", workspace));
    }
    if let Some(visibility) = visibility {
        block.push_str(&format!(
            "- visibility: {}\n",
            format_visibility(visibility)
        ));
    }
    if !note.links.is_empty() {
        block.push_str(&format!("- links: {}\n", note.links.join(", ")));
    }
    if !note.backlinks.is_empty() {
        block.push_str(&format!("- backlinks: {}\n", note.backlinks.join(", ")));
    }
    if let Some(folder_path) = note.folder_path.as_deref() {
        block.push_str(&format!("- folder: {}\n", folder_path));
    }
    block.push_str(&format!("- folder_depth: {}\n", note.folder_depth));
    block.push_str(MEMD_ROUNDTRIP_END);
    block.push('\n');
    block
}

pub fn strip_roundtrip_annotation(content: &str) -> String {
    let mut current = content.to_string();
    loop {
        let Some(begin) = current.find(MEMD_ROUNDTRIP_BEGIN) else {
            break;
        };
        let Some(relative_end) = current[begin..].find(MEMD_ROUNDTRIP_END) else {
            break;
        };
        let mut end = begin + relative_end + MEMD_ROUNDTRIP_END.len();
        if current[end..].starts_with('\n') {
            end += '\n'.len_utf8();
        }
        current.replace_range(begin..end, "");
    }
    let mut compacted = current;
    while compacted.contains("\n\n\n") {
        compacted = compacted.replace("\n\n\n", "\n\n");
    }
    compacted
}

pub fn upsert_markdown_block(
    content: &str,
    begin_marker: &str,
    end_marker: &str,
    block: &str,
) -> String {
    if let (Some(begin), Some(end)) = (content.find(begin_marker), content.find(end_marker)) {
        let mut end = end + end_marker.len();
        if content[end..].starts_with('\n') {
            end += '\n'.len_utf8();
        }
        let mut updated = content.to_string();
        updated.replace_range(begin..end, block);
        return updated;
    }

    let mut updated = content.trim_end().to_string();
    if !updated.is_empty() {
        updated.push_str("\n\n");
    }
    updated.push_str(block.trim_end());
    updated.push('\n');
    updated
}

pub fn annotate_note(path: impl AsRef<Path>, block: &str) -> anyhow::Result<()> {
    let path = path.as_ref();
    let original = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let updated = upsert_markdown_block(&original, MEMD_ROUNDTRIP_BEGIN, MEMD_ROUNDTRIP_END, block);
    if updated == original {
        return Ok(());
    }
    fs::write(path, updated).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub fn note_mirror_path(vault: &Path, note: &ObsidianNote) -> PathBuf {
    vault
        .join(".memd")
        .join("writeback")
        .join("notes")
        .join(&note.relative_path)
}

pub fn attachment_mirror_path(vault: &Path, attachment: &ObsidianAttachment) -> PathBuf {
    let mut mirror = vault
        .join(".memd")
        .join("writeback")
        .join("attachments")
        .join(&attachment.relative_path);
    mirror.set_extension("md");
    mirror
}

pub fn partition_changed_attachments<'a>(
    attachments: &'a [ObsidianAttachment],
    sync_state: &ObsidianSyncState,
) -> (Vec<&'a ObsidianAttachment>, usize) {
    let mut changed = Vec::new();
    let mut unchanged_count = 0usize;
    let mut seen_hashes = HashSet::<String>::new();

    for attachment in attachments {
        let entry = sync_state.entries.get(&attachment.relative_path);
        if entry.is_some_and(|entry| entry.content_hash == attachment.content_hash) {
            unchanged_count += 1;
            continue;
        }
        if !seen_hashes.insert(attachment.content_hash.clone()) {
            continue;
        }
        changed.push(attachment);
    }

    (changed, unchanged_count)
}

pub fn build_note_request(
    note: &ObsidianNote,
    project: Option<String>,
    namespace: Option<String>,
    workspace: Option<String>,
    visibility: Option<MemoryVisibility>,
    vault: PathBuf,
    supersedes_item_id: Option<Uuid>,
) -> CandidateMemoryRequest {
    let scope = if project.is_some() {
        MemoryScope::Project
    } else {
        MemoryScope::Synced
    };
    let source_path = vault.join(&note.path).display().to_string();
    let mut tags = note.tags.clone();
    tags.push("obsidian".to_string());
    tags.push("vault_note".to_string());
    tags.push(format!(
        "kind={}",
        format!("{:?}", note.kind).to_lowercase()
    ));
    if !note.aliases.is_empty() {
        tags.push("has_aliases".to_string());
    }

    let mut content = String::new();
    content.push_str(&format!("Obsidian note: {}\n", note.title));
    content.push_str(&format!("Vault path: {}\n", note.relative_path));
    if let Some(folder_path) = note.folder_path.as_deref() {
        content.push_str(&format!("Folder path: {}\n", folder_path));
    }
    content.push_str(&format!("Folder depth: {}\n", note.folder_depth));
    if !note.aliases.is_empty() {
        content.push_str(&format!("Aliases: {}\n", note.aliases.join(", ")));
    }
    if !note.tags.is_empty() {
        content.push_str(&format!("Tags: {}\n", note.tags.join(", ")));
    }
    if !note.links.is_empty() {
        content.push_str(&format!("Wiki links: {}\n", note.links.join(", ")));
    }
    if !note.backlinks.is_empty() {
        content.push_str(&format!("Backlinks: {}\n", note.backlinks.join(", ")));
    }
    content.push_str("Excerpt:\n");
    content.push_str(&note.excerpt);

    CandidateMemoryRequest {
        content,
        kind: note.kind,
        scope,
        project,
        namespace,
        workspace,
        visibility,
        belief_branch: None,
        source_agent: Some("obsidian".to_string()),
        source_system: Some("obsidian".to_string()),
        source_path: Some(source_path),
        source_quality: Some(SourceQuality::Canonical),
        confidence: Some(0.92),
        ttl_seconds: None,
        last_verified_at: Some(Utc::now()),
        supersedes: supersedes_item_id.into_iter().collect(),
        tags,
    }
}

pub fn build_entity_link_request(
    from_entity_id: Uuid,
    to_entity_id: Uuid,
    note: &ObsidianNote,
) -> EntityLinkRequest {
    EntityLinkRequest {
        from_entity_id,
        to_entity_id,
        relation_kind: EntityRelationKind::Related,
        confidence: Some(0.72),
        note: Some(format!("obsidian wiki link from {}", note.relative_path)),
        context: Some(MemoryContextFrame {
            at: Some(Utc::now()),
            project: None,
            namespace: None,
            workspace: None,
            repo: Some("obsidian".to_string()),
            host: None,
            branch: None,
            agent: Some("obsidian".to_string()),
            location: Some(note.relative_path.clone()),
        }),
        tags: vec!["obsidian".to_string(), "wiki-link".to_string()],
    }
}

pub fn normalized_title(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_markdown_note_from_raw(
    vault: &Path,
    path: &Path,
    raw: &str,
    bytes: u64,
    modified_at: Option<DateTime<Utc>>,
    content_hash: String,
) -> anyhow::Result<Option<ObsidianNote>> {
    let raw = strip_roundtrip_annotation(raw);
    let (frontmatter, body) = split_frontmatter(&raw);
    let title = frontmatter
        .as_ref()
        .and_then(|frontmatter| frontmatter.get("title").cloned())
        .or_else(|| first_heading(&body))
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("untitled")
                .to_string()
        });
    let kind = infer_kind(
        frontmatter
            .as_ref()
            .and_then(|frontmatter| frontmatter.get("kind").cloned())
            .or_else(|| {
                frontmatter
                    .as_ref()
                    .and_then(|frontmatter| frontmatter.get("type").cloned())
            }),
        &title,
        &body,
    );
    let tags = parse_tags(
        frontmatter
            .as_ref()
            .and_then(|frontmatter| frontmatter.get("tags").cloned()),
    );
    let aliases = parse_aliases(
        frontmatter
            .as_ref()
            .and_then(|frontmatter| frontmatter.get("aliases").cloned()),
    );
    let links = extract_wiki_links(&body);
    let excerpt = build_excerpt(&body, 8, 700);
    let relative_path = path
        .strip_prefix(vault)
        .unwrap_or(path)
        .display()
        .to_string();
    let folder_path = path.parent().and_then(|value| {
        value
            .strip_prefix(vault)
            .ok()
            .map(|folder| folder.display().to_string())
            .filter(|folder| !folder.is_empty())
    });
    let folder_depth = folder_path
        .as_deref()
        .map(|folder| folder.split('/').count())
        .unwrap_or(0);
    Ok(Some(ObsidianNote {
        path: path.to_path_buf(),
        relative_path,
        folder_path,
        folder_depth,
        title: title.trim().to_string(),
        normalized_title: normalized_title(&title),
        excerpt,
        kind,
        tags,
        aliases,
        links,
        backlinks: Vec::new(),
        sensitivity: detect_sensitivity(&title, &body, frontmatter.as_ref()),
        bytes,
        modified_at,
        content_hash,
    }))
}

#[cfg(test)]
fn parse_markdown_note(vault: &Path, path: &Path) -> anyhow::Result<Option<ObsidianNote>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let metadata = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
    parse_markdown_note_from_raw(
        vault,
        path,
        &raw,
        metadata.len(),
        metadata.modified().ok().map(system_time_to_utc),
        hash_content(&strip_roundtrip_annotation(&raw)),
    )
}

fn parse_attachment_from_raw(
    vault: &Path,
    path: &Path,
    raw: &[u8],
    bytes: u64,
    modified_at: Option<DateTime<Utc>>,
    content_hash: String,
) -> anyhow::Result<Option<ObsidianAttachment>> {
    let relative_path = path
        .strip_prefix(vault)
        .unwrap_or(path)
        .display()
        .to_string();
    let folder_path = path.parent().and_then(|value| {
        value
            .strip_prefix(vault)
            .ok()
            .map(|folder| folder.display().to_string())
            .filter(|folder| !folder.is_empty())
    });
    let mime = mime_guess::from_path(path)
        .first_raw()
        .map(|value| value.to_string());
    let asset_kind = classify_asset_kind(path, mime.as_deref()).to_string();
    let sensitivity = if is_text_like_attachment(path) {
        let text = String::from_utf8_lossy(raw);
        detect_sensitivity(
            path.file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("attachment"),
            &text,
            None,
        )
    } else {
        ObsidianSensitivity {
            sensitive: false,
            reasons: Vec::new(),
        }
    };

    Ok(Some(ObsidianAttachment {
        path: path.to_path_buf(),
        relative_path,
        folder_path,
        asset_kind,
        mime,
        bytes,
        modified_at,
        content_hash,
        sensitivity,
    }))
}

fn fallback_attachment_match(
    attachment: &ObsidianAttachment,
    notes: &[ObsidianNote],
    note_index: &HashMap<String, usize>,
) -> Option<ObsidianAttachmentMatch> {
    let stem = attachment
        .path
        .file_stem()
        .and_then(|value| value.to_str())
        .map(normalized_title)?;
    let mut best: Option<(usize, usize)> = None;
    for (idx, note) in notes.iter().enumerate() {
        let mut score = 0usize;
        if note.normalized_title == stem {
            score += 5;
        }
        if note
            .aliases
            .iter()
            .any(|alias| normalized_title(alias) == stem)
        {
            score += 4;
        }
        if note.normalized_title.contains(&stem) || stem.contains(&note.normalized_title) {
            score += 2;
        }
        if note
            .path
            .parent()
            .and_then(|value| value.file_name())
            .and_then(|value| value.to_str())
            .map(normalized_title)
            .as_deref()
            == Some(stem.as_str())
        {
            score += 3;
        }
        if let Some(&resolved_idx) = note_index.get(&note.normalized_title)
            && resolved_idx == idx
        {
            score += 1;
        }
        if score == 0 {
            continue;
        }
        if best
            .as_ref()
            .map(|(best_score, _)| score > *best_score)
            .unwrap_or(true)
        {
            best = Some((score, idx));
        }
    }

    best.map(|(_, idx)| ObsidianAttachmentMatch {
        note_index: idx,
        relation_kind: EntityRelationKind::Related,
        reason: "fuzzy attachment stem match".to_string(),
    })
}

fn default_sync_state_path(vault: &Path) -> PathBuf {
    vault.join(".memd").join("obsidian-sync.json")
}

fn default_scan_cache_path(vault: &Path) -> PathBuf {
    vault.join(".memd").join("obsidian-scan-cache.json")
}

fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn hash_bytes(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

fn system_time_to_utc(value: SystemTime) -> DateTime<Utc> {
    value.into()
}

fn classify_asset_kind(path: &Path, mime: Option<&str>) -> &'static str {
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

fn is_text_like_attachment(path: &Path) -> bool {
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

fn should_skip_vault_path(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component,
            std::path::Component::Normal(name)
                if name == ".memd" || name == ".obsidian" || name == ".git"
        )
    })
}

fn note_matches_scope(
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

fn attachment_matches_scope(
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

fn folder_matches(folder: &str, candidate: &str) -> bool {
    let folder = folder.trim_matches('/');
    let candidate = candidate.trim_matches('/');
    folder == candidate
        || folder.starts_with(&format!("{candidate}/"))
        || folder.ends_with(&format!("/{candidate}"))
        || normalized_title(folder) == normalized_title(candidate)
}

fn short_uuid(id: Uuid) -> String {
    id.to_string().chars().take(8).collect()
}

fn detect_sensitivity(
    title: &str,
    body: &str,
    frontmatter: Option<&HashMap<String, String>>,
) -> ObsidianSensitivity {
    let mut reasons = Vec::new();
    let haystack = format!(
        "{}\n{}\n{}",
        title,
        body,
        frontmatter
            .map(|frontmatter| {
                frontmatter
                    .iter()
                    .map(|(key, value)| format!("{key}:{value}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default()
    )
    .to_ascii_lowercase();

    let patterns = [
        ("private_key", "-----begin private key-----"),
        ("ssh_key", "ssh-rsa"),
        ("aws_access_key_id", "aws_access_key_id"),
        ("aws_secret_access_key", "aws_secret_access_key"),
        ("x_api_key", "x-api-key"),
        ("client_secret", "client_secret"),
        ("api_key", "api key"),
        ("api_key", "apikey"),
        ("token", "bearer "),
        ("token", "access token"),
        ("password", "password"),
        ("password", "passwd"),
        ("secret", "secret"),
        ("secret", "sk-"),
        ("secret", "ghp_"),
        ("secret", "github_pat_"),
        ("secret", "xoxb-"),
        ("secret", "xoxp-"),
    ];

    for (reason, needle) in patterns {
        if haystack.contains(needle) {
            reasons.push(reason.to_string());
        }
    }

    ObsidianSensitivity {
        sensitive: !reasons.is_empty(),
        reasons,
    }
}

pub fn render_sensitive_review(scan: &ObsidianVaultScan) -> String {
    let mut output = format!(
        "vault={} sensitive={} notes={} skipped={}",
        scan.vault.display(),
        scan.sensitive_count,
        scan.note_count,
        scan.skipped_count
    );

    if !scan.sensitive_notes.is_empty() {
        let trail = scan
            .sensitive_notes
            .iter()
            .take(5)
            .map(|note| {
                format!(
                    "{} [{}]",
                    note.relative_path,
                    note.reasons
                        .iter()
                        .take(2)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(",")
                )
            })
            .collect::<Vec<_>>()
            .join(" | ");
        output.push_str(&format!(" trail={trail}"));
    }

    output
}

fn split_frontmatter(raw: &str) -> (Option<HashMap<String, String>>, String) {
    let mut lines = raw.lines();
    if !matches!(lines.next(), Some(line) if line.trim() == "---") {
        return (None, raw.to_string());
    }

    let mut frontmatter = HashMap::new();
    let mut body = Vec::new();
    let mut in_frontmatter = true;
    for line in lines {
        if in_frontmatter && line.trim() == "---" {
            in_frontmatter = false;
            continue;
        }
        if in_frontmatter {
            if let Some((key, value)) = line.split_once(':') {
                frontmatter.insert(key.trim().to_string(), value.trim().to_string());
            }
        } else {
            body.push(line);
        }
    }

    (
        if frontmatter.is_empty() {
            None
        } else {
            Some(frontmatter)
        },
        body.join("\n"),
    )
}

fn first_heading(body: &str) -> Option<String> {
    body.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix("# ")
            .map(|heading| heading.trim().to_string())
    })
}

fn infer_kind(kind: Option<String>, title: &str, body: &str) -> MemoryKind {
    let candidates = kind
        .into_iter()
        .chain([title.to_string(), body.to_string()])
        .map(|value| normalized_title(&value))
        .collect::<Vec<_>>();

    for candidate in candidates {
        if candidate.contains("decision") {
            return MemoryKind::Decision;
        }
        if candidate.contains("runbook") || candidate.contains("how to") {
            return MemoryKind::Runbook;
        }
        if candidate.contains("status") || candidate.contains("daily note") {
            return MemoryKind::Status;
        }
        if candidate.contains("live truth") || candidate.contains("truth lane") {
            return MemoryKind::LiveTruth;
        }
        if candidate.contains("preference") || candidate.contains("prefs") {
            return MemoryKind::Preference;
        }
        if candidate.contains("topology") || candidate.contains("diagram") {
            return MemoryKind::Topology;
        }
        if candidate.contains("constraint") {
            return MemoryKind::Constraint;
        }
        if candidate.contains("pattern") {
            return MemoryKind::Pattern;
        }
        if candidate.contains("fact") {
            return MemoryKind::Fact;
        }
    }

    MemoryKind::Pattern
}

fn parse_tags(value: Option<String>) -> Vec<String> {
    parse_listish(value)
}

fn parse_aliases(value: Option<String>) -> Vec<String> {
    parse_listish(value)
}

fn parse_listish(value: Option<String>) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };

    value
        .trim_matches(|ch| ch == '[' || ch == ']')
        .split([',', ';'])
        .map(|entry| {
            entry
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string()
        })
        .filter(|entry| !entry.is_empty())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
}

fn extract_wiki_links(body: &str) -> Vec<String> {
    let mut links = HashSet::new();
    let bytes = body.as_bytes();
    let mut idx = 0usize;
    while idx + 3 < bytes.len() {
        if bytes[idx] == b'[' && bytes[idx + 1] == b'[' {
            let start = idx + 2;
            if let Some(end) = body[start..].find("]]") {
                let raw = &body[start..start + end];
                let target = raw.split_once('|').map(|(left, _)| left).unwrap_or(raw);
                let target = target.trim();
                if !target.is_empty() {
                    links.insert(target.to_string());
                }
                idx = start + end + 2;
                continue;
            }
        }
        idx += 1;
    }

    let mut links = links.into_iter().collect::<Vec<_>>();
    links.sort();
    links
}

fn build_excerpt(body: &str, max_lines: usize, max_chars: usize) -> String {
    let mut excerpt = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("# ") && excerpt.is_empty() {
            continue;
        }
        excerpt.push(trimmed.to_string());
        if excerpt.len() >= max_lines {
            break;
        }
    }

    let mut joined = excerpt.join(" ");
    if joined.chars().count() > max_chars {
        joined = joined.chars().take(max_chars.saturating_sub(3)).collect();
        joined.push_str("...");
    }
    joined
}

#[cfg(test)]
mod tests {
    use super::*;
    use memd_schema::MemoryItem;
    use std::io::Write;

    fn temp_file(name: &str, contents: &str) -> PathBuf {
        let dir = std::env::temp_dir();
        let file_name = match name.rsplit_once('.') {
            Some((stem, ext)) if !stem.is_empty() && !ext.is_empty() => {
                format!("memd-obsidian-{}-{}.{}", stem, Uuid::new_v4(), ext)
            }
            _ => format!("memd-obsidian-{}-{}", name, Uuid::new_v4()),
        };
        let path = dir.join(file_name);
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
        path
    }

    #[test]
    fn detects_sensitive_note_content() {
        let note = parse_markdown_note(
            Path::new("/tmp/vault"),
            &temp_file(
                "secrets.md",
                "---\ntitle: Secrets\n---\n# Secrets\nAWS_SECRET_ACCESS_KEY=shhh\n",
            ),
        )
        .unwrap()
        .unwrap();

        assert!(note.sensitivity.sensitive);
        assert!(!note.sensitivity.reasons.is_empty());
    }

    #[test]
    fn skips_sensitive_notes_from_import() {
        let vault = std::env::temp_dir().join(format!("memd-obsidian-vault-{}", Uuid::new_v4()));
        fs::create_dir_all(&vault).unwrap();

        let public = vault.join("public.md");
        let secret = vault.join("secrets.md");
        fs::write(
            &public,
            "---\ntitle: Public Note\ntags: [notes]\n---\n# Public Note\nHello world.\n",
        )
        .unwrap();
        fs::write(
            &secret,
            "---\ntitle: Secrets Note\ntags: [keys]\n---\n# Secrets Note\nAWS_SECRET_ACCESS_KEY=shhh\n",
        )
        .unwrap();

        let scan = scan_vault(
            &vault,
            Some("notes".to_string()),
            None,
            None,
            None,
            Some(10),
            false,
            None,
            &[],
            &[],
            &[],
            &[],
        )
        .unwrap();
        assert_eq!(scan.note_count, 1);
        assert_eq!(scan.sensitive_count, 1);
        assert_eq!(scan.notes.len(), 1);
        assert_eq!(scan.notes[0].title, "Public Note");
        assert!(!scan.notes[0].sensitivity.sensitive);
    }

    #[test]
    fn build_import_preview_suppresses_duplicate_note_content() {
        let vault = std::env::temp_dir().join(format!("memd-obsidian-vault-{}", Uuid::new_v4()));
        fs::create_dir_all(&vault).unwrap();

        fs::write(
            vault.join("alpha.md"),
            "---\ntitle: Alpha\n---\n# Alpha\nSame body.\n",
        )
        .unwrap();
        fs::write(
            vault.join("beta.md"),
            "---\ntitle: Beta\n---\n# Beta\nSame body.\n",
        )
        .unwrap();

        let scan = scan_vault(
            &vault,
            None,
            None,
            None,
            None,
            Some(10),
            false,
            None,
            &[],
            &[],
            &[],
            &[],
        )
        .unwrap();
        let sync_state = ObsidianSyncState::default();
        let (preview, candidates, changed_notes) =
            build_import_preview(scan, &sync_state, vault.join(".memd/state.json"));

        assert_eq!(preview.duplicate_count, 1);
        assert_eq!(candidates.len(), 1);
        assert_eq!(changed_notes.len(), 1);
        assert_eq!(preview.scan.note_count, 2);
        assert_eq!(preview.scan.unchanged_count, 0);
    }

    #[test]
    fn scan_cache_prunes_deleted_notes() {
        let vault = std::env::temp_dir().join(format!("memd-obsidian-vault-{}", Uuid::new_v4()));
        fs::create_dir_all(&vault).unwrap();
        let note_path = vault.join("prune.md");
        fs::write(
            &note_path,
            "---\ntitle: Prune Me\n---\n# Prune Me\nKeep this once.\n",
        )
        .unwrap();

        let first_scan = scan_vault(
            &vault,
            None,
            None,
            None,
            None,
            Some(10),
            false,
            None,
            &[],
            &[],
            &[],
            &[],
        )
        .unwrap();
        assert_eq!(first_scan.note_count, 1);

        fs::remove_file(&note_path).unwrap();

        let second_scan = scan_vault(
            &vault,
            None,
            None,
            None,
            None,
            Some(10),
            false,
            None,
            &[],
            &[],
            &[],
            &[],
        )
        .unwrap();
        assert_eq!(second_scan.note_count, 0);
        assert!(second_scan.cache_pruned >= 1);

        let cache = load_scan_cache(default_scan_cache_path(&vault)).unwrap();
        assert!(cache.notes.is_empty());
    }

    #[test]
    fn scan_cache_reuses_renamed_notes() {
        let vault = std::env::temp_dir().join(format!("memd-obsidian-vault-{}", Uuid::new_v4()));
        fs::create_dir_all(&vault).unwrap();
        let source_path = vault.join("source.md");
        let renamed_path = vault.join("renamed.md");
        fs::write(
            &source_path,
            "---\ntitle: Rename Me\n---\n# Rename Me\nSame body for rename.\n",
        )
        .unwrap();

        let first_scan = scan_vault(
            &vault,
            None,
            None,
            None,
            None,
            Some(10),
            false,
            None,
            &[],
            &[],
            &[],
            &[],
        )
        .unwrap();
        assert_eq!(first_scan.note_count, 1);

        fs::rename(&source_path, &renamed_path).unwrap();

        let second_scan = scan_vault(
            &vault,
            None,
            None,
            None,
            None,
            Some(10),
            false,
            None,
            &[],
            &[],
            &[],
            &[],
        )
        .unwrap();
        assert_eq!(second_scan.note_count, 1);
        assert!(second_scan.cache_hits >= 1);

        let cache = load_scan_cache(default_scan_cache_path(&vault)).unwrap();
        assert!(cache.notes.contains_key("renamed.md"));
        assert!(!cache.notes.contains_key("source.md"));
    }

    #[test]
    fn scan_cache_reuses_touched_notes_by_content_hash() {
        let vault = std::env::temp_dir().join(format!("memd-obsidian-vault-{}", Uuid::new_v4()));
        fs::create_dir_all(&vault).unwrap();
        let note_path = vault.join("touch.md");
        let body = "---\ntitle: Touch Me\n---\n# Touch Me\nSame body.\n";
        fs::write(&note_path, body).unwrap();

        let first_scan = scan_vault(
            &vault,
            None,
            None,
            None,
            None,
            Some(10),
            false,
            None,
            &[],
            &[],
            &[],
            &[],
        )
        .unwrap();
        assert_eq!(first_scan.note_count, 1);

        std::thread::sleep(std::time::Duration::from_millis(5));
        fs::write(&note_path, body).unwrap();

        let second_scan = scan_vault(
            &vault,
            None,
            None,
            None,
            None,
            Some(10),
            false,
            None,
            &[],
            &[],
            &[],
            &[],
        )
        .unwrap();
        assert_eq!(second_scan.note_count, 1);
        assert!(second_scan.cache_hits >= 1);

        let cache = load_scan_cache(default_scan_cache_path(&vault)).unwrap();
        assert!(cache.notes.contains_key("touch.md"));
    }

    #[test]
    fn scans_attachments_when_enabled() {
        let vault = std::env::temp_dir().join(format!("memd-obsidian-vault-{}", Uuid::new_v4()));
        fs::create_dir_all(&vault).unwrap();

        let pdf = vault.join("diagram.pdf");
        let image = vault.join("screenshot.png");
        let note = vault.join("note.md");
        fs::write(&pdf, b"%PDF-1.7").unwrap();
        fs::write(&image, b"fake").unwrap();
        fs::write(&note, "# heading\nbody").unwrap();

        let scan = scan_vault(
            &vault,
            None,
            None,
            None,
            None,
            Some(10),
            true,
            Some(10),
            &[],
            &[],
            &[],
            &[],
        )
        .unwrap();
        assert_eq!(scan.note_count, 1);
        assert_eq!(scan.attachment_count, 2);
        assert_eq!(scan.attachments.len(), 2);
        assert!(
            scan.attachments
                .iter()
                .any(|asset| asset.relative_path == "diagram.pdf")
        );
        assert!(
            scan.attachments
                .iter()
                .any(|asset| asset.relative_path == "screenshot.png")
        );
    }

    #[test]
    fn partition_changed_attachments_suppresses_duplicate_content_hashes() {
        let attachments = vec![
            ObsidianAttachment {
                path: PathBuf::from("a.png"),
                relative_path: "a.png".to_string(),
                folder_path: None,
                asset_kind: "image".to_string(),
                mime: Some("image/png".to_string()),
                bytes: 4,
                modified_at: None,
                content_hash: "same".to_string(),
                sensitivity: ObsidianSensitivity {
                    sensitive: false,
                    reasons: Vec::new(),
                },
            },
            ObsidianAttachment {
                path: PathBuf::from("b.png"),
                relative_path: "b.png".to_string(),
                folder_path: None,
                asset_kind: "image".to_string(),
                mime: Some("image/png".to_string()),
                bytes: 4,
                modified_at: None,
                content_hash: "same".to_string(),
                sensitivity: ObsidianSensitivity {
                    sensitive: false,
                    reasons: Vec::new(),
                },
            },
        ];

        let sync_state = ObsidianSyncState::default();
        let (changed, unchanged) = partition_changed_attachments(&attachments, &sync_state);

        assert_eq!(changed.len(), 1);
        assert_eq!(unchanged, 0);
    }

    #[test]
    fn filters_notes_by_folder_and_tag_scope() {
        let vault = std::env::temp_dir().join(format!("memd-obsidian-vault-{}", Uuid::new_v4()));
        fs::create_dir_all(vault.join("work")).unwrap();
        fs::create_dir_all(vault.join("personal")).unwrap();

        fs::write(
            vault.join("work").join("project.md"),
            "---\ntitle: Work Note\ntags: [work, project]\n---\n# Work Note\nBody\n",
        )
        .unwrap();
        fs::write(
            vault.join("personal").join("journal.md"),
            "---\ntitle: Personal Note\ntags: [life]\n---\n# Personal Note\nBody\n",
        )
        .unwrap();

        let scan = scan_vault(
            &vault,
            None,
            None,
            None,
            None,
            Some(10),
            false,
            None,
            &[String::from("work")],
            &[String::from("personal")],
            &[String::from("project")],
            &[String::from("life")],
        )
        .unwrap();

        assert_eq!(scan.note_count, 1);
        assert_eq!(scan.notes[0].title, "Work Note");
    }

    #[test]
    fn skips_sensitive_text_attachments() {
        let vault = std::env::temp_dir().join(format!("memd-obsidian-vault-{}", Uuid::new_v4()));
        fs::create_dir_all(&vault).unwrap();

        let secret = vault.join("secrets.txt");
        fs::write(&secret, "AWS_SECRET_ACCESS_KEY=shhh").unwrap();

        let scan = scan_vault(
            &vault,
            None,
            None,
            None,
            None,
            Some(10),
            true,
            Some(10),
            &[],
            &[],
            &[],
            &[],
        )
        .unwrap();
        assert_eq!(scan.attachment_sensitive_count, 1);
        assert_eq!(scan.attachment_count, 0);
        assert!(scan.attachments.is_empty());
    }

    #[test]
    fn upserts_roundtrip_annotation_block() {
        let content = "# Note\n\nBody.\n";
        let note = ObsidianNote {
            path: PathBuf::from("Note.md"),
            relative_path: "Note.md".to_string(),
            folder_path: None,
            folder_depth: 0,
            title: "Note".to_string(),
            normalized_title: normalized_title("Note"),
            excerpt: "Body.".to_string(),
            kind: MemoryKind::Fact,
            tags: Vec::new(),
            aliases: Vec::new(),
            links: vec!["Other".to_string()],
            backlinks: vec!["Back.md".to_string()],
            sensitivity: ObsidianSensitivity {
                sensitive: false,
                reasons: Vec::new(),
            },
            bytes: 8,
            modified_at: None,
            content_hash: "abc123".to_string(),
        };
        let block = build_roundtrip_annotation(
            &note,
            Some(Uuid::nil()),
            Some(Uuid::nil()),
            Some("core"),
            Some(MemoryVisibility::Workspace),
        );
        let updated =
            upsert_markdown_block(content, MEMD_ROUNDTRIP_BEGIN, MEMD_ROUNDTRIP_END, &block);
        assert!(updated.contains("memd sync"));
        assert!(updated.contains("item_id"));
        let second =
            upsert_markdown_block(&updated, MEMD_ROUNDTRIP_BEGIN, MEMD_ROUNDTRIP_END, &block);
        assert_eq!(updated, second);
    }

    #[test]
    fn strips_roundtrip_annotation_from_source_content() {
        let content = "# Note\n\nBody.\n\n<!-- memd:begin -->\n## memd sync\n\n- note: Note\n- path: Note.md\n- kind: fact\n<!-- memd:end -->\n";
        let stripped = strip_roundtrip_annotation(content);
        assert_eq!(stripped.trim_end(), "# Note\n\nBody.");
        assert!(!stripped.contains("memd sync"));
    }

    #[test]
    fn builds_stable_mirror_paths() {
        let vault = PathBuf::from("/tmp/vault");
        let note = ObsidianNote {
            path: PathBuf::from("work/note.md"),
            relative_path: "work/note.md".to_string(),
            folder_path: Some("work".to_string()),
            folder_depth: 1,
            title: "Note".to_string(),
            normalized_title: normalized_title("Note"),
            excerpt: "Body.".to_string(),
            kind: MemoryKind::Fact,
            tags: Vec::new(),
            aliases: Vec::new(),
            links: Vec::new(),
            backlinks: Vec::new(),
            sensitivity: ObsidianSensitivity {
                sensitive: false,
                reasons: Vec::new(),
            },
            bytes: 8,
            modified_at: None,
            content_hash: "abc123".to_string(),
        };
        let attachment = ObsidianAttachment {
            path: PathBuf::from("assets/image.png"),
            relative_path: "assets/image.png".to_string(),
            folder_path: Some("assets".to_string()),
            asset_kind: "image".to_string(),
            mime: Some("image/png".to_string()),
            bytes: 16,
            modified_at: None,
            content_hash: "def456".to_string(),
            sensitivity: ObsidianSensitivity {
                sensitive: false,
                reasons: Vec::new(),
            },
        };

        assert_eq!(
            note_mirror_path(&vault, &note),
            vault
                .join(".memd")
                .join("writeback")
                .join("notes")
                .join("work/note.md")
        );
        assert_eq!(
            attachment_mirror_path(&vault, &attachment),
            vault
                .join(".memd")
                .join("writeback")
                .join("attachments")
                .join("assets/image.md")
        );
    }

    #[test]
    fn builds_obsidian_open_uri() {
        let path = std::env::temp_dir().join(format!(
            "memd-obsidian-uri-{}/writeback/note.md",
            uuid::Uuid::new_v4()
        ));
        let uri = build_open_uri(&path, Some("split")).unwrap();
        let encoded_path = encode_uri_component(&path.to_string_lossy());
        assert!(uri.starts_with(&format!("obsidian://open?path={encoded_path}")));
        assert!(uri.contains("&paneType=split"));
    }

    #[test]
    fn builds_compiled_note_path() {
        let path = default_compiled_note_path(Path::new("/tmp/vault"), "Rust Memory Patterns");
        assert_eq!(
            path,
            Path::new("/tmp/vault/.memd/compiled/rust-memory-patterns.md")
        );
    }

    #[test]
    fn builds_compiled_memory_path() {
        let now = Utc::now();
        let explain = ExplainMemoryResponse {
            route: memd_schema::RetrievalRoute::ProjectFirst,
            intent: memd_schema::RetrievalIntent::Fact,
            item: MemoryItem {
                id: Uuid::parse_str("12345678-1234-5678-1234-567812345678").unwrap(),
                content: "Bundle-first memory config".to_string(),
                redundancy_key: Some("fact:bundle-first".to_string()),
                belief_branch: Some("mainline".to_string()),
                preferred: true,
                kind: MemoryKind::Fact,
                scope: memd_schema::MemoryScope::Project,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("core".to_string()),
                visibility: memd_schema::MemoryVisibility::Workspace,
                source_agent: Some("codex".to_string()),
                source_system: Some("cli".to_string()),
                source_path: Some("/tmp/vault/wiki/bundle.md".to_string()),
                source_quality: Some(SourceQuality::Canonical),
                confidence: 0.9,
                ttl_seconds: None,
                created_at: now,
                updated_at: now,
                last_verified_at: Some(now),
                supersedes: Vec::new(),
                tags: vec!["bundle".to_string()],
                status: memd_schema::MemoryStatus::Active,
                stage: memd_schema::MemoryStage::Canonical,
            },
            canonical_key: "fact:bundle-first".to_string(),
            redundancy_key: "fact:bundle-first".to_string(),
            reasons: vec!["route=project_first".to_string()],
            entity: None,
            events: Vec::new(),
            sources: Vec::new(),
            retrieval_feedback: memd_schema::RetrievalFeedbackSummary {
                total_retrievals: 0,
                last_retrieved_at: None,
                by_surface: Vec::new(),
                recent_policy_hooks: Vec::new(),
            },
            branch_siblings: Vec::new(),
            rehydration: Vec::new(),
            policy_hooks: Vec::new(),
        };

        let path = default_compiled_memory_path(Path::new("/tmp/vault"), &explain);
        assert_eq!(
            path,
            Path::new("/tmp/vault/.memd/compiled/memory/bundle-12345678.md")
        );
    }

    #[test]
    fn parses_compiled_query_metadata() {
        let markdown = r#"---
title: Rust Memory Patterns
source_system: memd
source_agent: memd
route: projectfirst
intent: fact
items: 7
---

# Rust Memory Patterns
"#;
        let (title, items) = parse_compiled_artifact_metadata(markdown);
        assert_eq!(title.as_deref(), Some("Rust Memory Patterns"));
        assert_eq!(items, Some(7));
    }

    #[test]
    fn finds_existing_compiled_memory_path_by_id() {
        let vault = std::env::temp_dir().join(format!("memd-compiled-{}", Uuid::new_v4()));
        let memory_dir = vault.join(".memd").join("compiled").join("memory");
        fs::create_dir_all(&memory_dir).unwrap();
        let expected = memory_dir.join("bundle-12345678.md");
        fs::write(&expected, "# Compiled Memory Page\n").unwrap();

        let found = find_compiled_memory_path_by_id(
            &vault,
            Uuid::parse_str("12345678-1234-5678-1234-567812345678").unwrap(),
        );
        assert_eq!(found.as_deref(), Some(expected.as_path()));

        let _ = fs::remove_dir_all(&vault);
    }

    #[test]
    fn builds_compiled_index_path() {
        let path = default_compiled_index_path(Path::new("/tmp/vault"));
        assert_eq!(path, Path::new("/tmp/vault/.memd/compiled/INDEX.md"));
    }

    #[test]
    fn builds_handoff_path() {
        let snapshot = crate::ResumeSnapshot {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "general".to_string(),
            context: memd_schema::CompactContextResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::General,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                records: Vec::new(),
            },
            working: memd_schema::WorkingMemoryResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::General,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                budget_chars: 0,
                used_chars: 0,
                remaining_chars: 0,
                truncated: false,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 8,
                    max_chars_per_item: 220,
                    budget_chars: 1600,
                    rehydration_limit: 4,
                },
                records: Vec::new(),
                evicted: Vec::new(),
                rehydration_queue: Vec::new(),
                traces: Vec::new(),
                semantic_consolidation: None,
            },
            inbox: memd_schema::MemoryInboxResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::General,
                items: Vec::new(),
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: Vec::new(),
            },
            sources: memd_schema::SourceMemoryResponse {
                sources: Vec::new(),
            },
            semantic: None,
            claims: crate::SessionClaimsState::default(),
            recent_repo_changes: Vec::new(),
            change_summary: Vec::new(),
            resume_state_age_minutes: None,
            refresh_recommended: false,
        };
        let path = default_handoff_path(Path::new("/tmp/vault"), &snapshot);
        assert!(path.starts_with("/tmp/vault/.memd/handoffs"));
        assert_eq!(path.extension().and_then(|ext| ext.to_str()), Some("md"));
    }

    #[test]
    fn builds_handoff_markdown() {
        let snapshot = crate::ResumeSnapshot {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "general".to_string(),
            context: memd_schema::CompactContextResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::General,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                records: Vec::new(),
            },
            working: memd_schema::WorkingMemoryResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::General,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                budget_chars: 1600,
                used_chars: 32,
                remaining_chars: 1568,
                truncated: false,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 8,
                    max_chars_per_item: 220,
                    budget_chars: 1600,
                    rehydration_limit: 4,
                },
                records: vec![memd_schema::CompactMemoryRecord {
                    id: Uuid::new_v4(),
                    record: "Remember the active handoff lane".to_string(),
                }],
                evicted: Vec::new(),
                rehydration_queue: Vec::new(),
                traces: Vec::new(),
                semantic_consolidation: None,
            },
            inbox: memd_schema::MemoryInboxResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::General,
                items: Vec::new(),
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: vec![memd_schema::WorkspaceMemoryRecord {
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: MemoryVisibility::Workspace,
                    item_count: 4,
                    active_count: 3,
                    candidate_count: 1,
                    contested_count: 0,
                    source_lane_count: 2,
                    avg_confidence: 0.86,
                    trust_score: 0.92,
                    last_seen_at: None,
                    tags: vec!["handoff".to_string()],
                }],
            },
            semantic: Some(memd_rag::RagRetrieveResponse {
                status: "ok".to_string(),
                mode: memd_rag::RagRetrieveMode::Auto,
                items: vec![memd_rag::RagRetrieveItem {
                    content:
                        "Shared workspace notes mention the same handoff lane and recovery path."
                            .to_string(),
                    source: Some("vault/wiki/team-alpha.md".to_string()),
                    score: 0.93,
                }],
            }),
            sources: memd_schema::SourceMemoryResponse {
                sources: Vec::new(),
            },
            claims: crate::SessionClaimsState::default(),
            recent_repo_changes: Vec::new(),
            change_summary: Vec::new(),
            resume_state_age_minutes: None,
            refresh_recommended: false,
        };
        let sources = SourceMemoryResponse {
            sources: vec![memd_schema::SourceMemoryRecord {
                source_agent: Some("codex".to_string()),
                source_system: Some("cli".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("team-alpha".to_string()),
                visibility: MemoryVisibility::Workspace,
                item_count: 2,
                active_count: 2,
                candidate_count: 0,
                derived_count: 0,
                synthetic_count: 0,
                contested_count: 0,
                avg_confidence: 0.88,
                trust_score: 0.94,
                last_seen_at: None,
                tags: vec!["handoff".to_string()],
            }],
        };

        let (_, markdown) = build_handoff_markdown(Path::new("/tmp/vault"), &snapshot, &sources);
        assert!(markdown.contains("# Handoff"));
        assert!(markdown.contains("## W"));
        assert!(markdown.contains("## L"));
        assert!(markdown.contains("## S"));
        assert!(markdown.contains("## C"));
        assert!(markdown.contains("team-alpha"));
    }

    #[test]
    fn updates_compiled_index_entries() {
        let existing = "# Compiled Wiki Index\n\n- [[old-note]] | query: old note | items: 3\n";
        let markdown = build_compiled_index_markdown(
            Some(existing),
            "query",
            "Rust Memory Patterns",
            Path::new("/tmp/vault/.memd/compiled/rust-memory-patterns.md"),
            7,
        );
        assert!(markdown.contains("[[old-note]]"));
        assert!(
            markdown.contains("[[rust-memory-patterns]] | query: Rust Memory Patterns | items: 7")
        );
    }

    #[test]
    fn updates_compiled_memory_index_entries() {
        let markdown = build_compiled_index_markdown(
            None,
            "memory",
            "bundle-first fact 12345678",
            Path::new("/tmp/vault/.memd/compiled/memory/bundle-first-fact-12345678.md"),
            1,
        );
        assert!(markdown.contains(
            "[[bundle-first-fact-12345678]] | memory: bundle-first fact 12345678 | items: 1"
        ));
    }

    #[test]
    fn derives_wikilink_for_vault_path() {
        let link = source_wikilink_for_path(
            Path::new("/tmp/vault"),
            Path::new("/tmp/vault/wiki/Topic Note.md"),
        );
        assert_eq!(link.as_deref(), Some("[[Topic Note]]"));
    }

    #[test]
    fn resolves_relative_open_paths_under_vault() {
        let vault = PathBuf::from("/tmp/vault");
        let resolved = resolve_open_path(&vault, Path::new("wiki/topic.md"));
        assert_eq!(resolved, vault.join("wiki/topic.md"));
    }

    #[test]
    fn preserves_absolute_open_paths() {
        let absolute = PathBuf::from("/tmp/elsewhere/topic.md");
        let resolved = resolve_open_path(Path::new("/tmp/vault"), &absolute);
        assert_eq!(resolved, absolute);
    }
}
