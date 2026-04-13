use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Context;
use chrono::{DateTime, Utc};
use memd_schema::{
    CandidateMemoryRequest, EntityLinkRequest, EntityRelationKind, ExplainMemoryResponse,
    MemoryContextFrame, MemoryKind, MemoryScope, MemoryVisibility, SearchMemoryResponse,
    SourceMemoryResponse, SourceQuality,
};
use serde::{Deserialize, Serialize};
use url::form_urlencoded::byte_serialize;
use uuid::Uuid;
use memdrive::WalkDir;

pub(crate) mod commands;
mod compiled;
mod mirror;
mod parsing;
pub(crate) mod runtime;
pub(crate) mod support;

#[allow(unused_imports)]
pub(crate) use compiled::*;
#[allow(unused_imports)]
pub(crate) use mirror::*;

use self::support::{
    attachment_matches_scope, classify_asset_kind, default_scan_cache_path,
    default_sync_state_path, find_cached_attachment_by_content_hash,
    find_cached_attachment_by_path_migration, find_cached_attachment_by_stat,
    find_cached_note_by_content_hash, find_cached_note_by_path_migration, find_cached_note_by_stat,
    hash_bytes, hash_content, infer_path_migration, is_text_like_attachment, note_matches_scope,
    register_path_migration, should_skip_vault_path, system_time_to_utc,
};
#[cfg(test)]
use parsing::parse_markdown_note;
use parsing::{fallback_attachment_match, parse_attachment_from_raw, parse_markdown_note_from_raw};

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

fn format_visibility(value: MemoryVisibility) -> &'static str {
    match value {
        MemoryVisibility::Private => "private",
        MemoryVisibility::Workspace => "workspace",
        MemoryVisibility::Public => "public",
    }
}
