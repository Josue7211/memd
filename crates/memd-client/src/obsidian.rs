use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use anyhow::Context;
use chrono::{DateTime, Utc};
use memd_schema::{
    CandidateMemoryRequest, EntityLinkRequest, EntityRelationKind, ExplainMemoryResponse,
    MemoryContextFrame, MemoryKind, MemoryScope, SourceQuality,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use walkdir::WalkDir;

const MEMD_ROUNDTRIP_BEGIN: &str = "<!-- memd:begin -->";
const MEMD_ROUNDTRIP_END: &str = "<!-- memd:end -->";

#[derive(Debug, Clone, Serialize)]
pub struct ObsidianVaultScan {
    pub vault: PathBuf,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub note_count: usize,
    pub sensitive_count: usize,
    pub skipped_count: usize,
    pub unchanged_count: usize,
    pub backlink_count: usize,
    pub attachment_count: usize,
    pub attachment_sensitive_count: usize,
    pub attachment_unchanged_count: usize,
    pub sensitive_notes: Vec<ObsidianSensitiveNote>,
    pub attachments: Vec<ObsidianAttachment>,
    pub notes: Vec<ObsidianNote>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObsidianSensitivity {
    pub sensitive: bool,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObsidianSensitiveNote {
    pub path: PathBuf,
    pub relative_path: String,
    pub title: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct ObsidianImportPreview {
    pub scan: ObsidianVaultScan,
    pub candidates: Vec<CandidateMemoryRequest>,
    pub note_index: HashMap<String, usize>,
    pub unchanged_count: usize,
    pub sync_state_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ObsidianAttachmentMatch {
    pub note_index: usize,
    pub relation_kind: EntityRelationKind,
    pub reason: String,
}

pub fn scan_vault(
    vault: impl AsRef<Path>,
    project: Option<String>,
    namespace: Option<String>,
    max_notes: Option<usize>,
    include_attachments: bool,
    max_attachments: Option<usize>,
    include_folders: &[String],
    exclude_folders: &[String],
    include_tags: &[String],
    exclude_tags: &[String],
) -> anyhow::Result<ObsidianVaultScan> {
    let vault = vault.as_ref();
    let mut notes = Vec::new();
    let mut sensitive_notes = Vec::new();
    let mut attachments = Vec::new();
    let mut skipped_count = 0usize;
    let mut sensitive_count = 0usize;
    let mut attachment_sensitive_count = 0usize;
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
            if notes.len() >= max_notes {
                skipped_count += 1;
                continue;
            }

            if let Some(note) = parse_markdown_note(vault, path)? {
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
            }
            continue;
        }

        if !include_attachments {
            skipped_count += 1;
            continue;
        }
        if attachments.len() >= max_attachments {
            skipped_count += 1;
            continue;
        }
        if let Some(attachment) = parse_attachment(vault, path)? {
            if !attachment_matches_scope(&attachment, include_folders, exclude_folders) {
                skipped_count += 1;
                continue;
            }
            if attachment.sensitivity.sensitive {
                attachment_sensitive_count += 1;
                continue;
            }
            attachments.push(attachment);
        }
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

    Ok(ObsidianVaultScan {
        vault: vault.to_path_buf(),
        project,
        namespace,
        note_count: notes.len(),
        sensitive_count,
        skipped_count,
        unchanged_count: 0,
        backlink_count,
        attachment_count: attachments.len(),
        attachment_sensitive_count,
        attachment_unchanged_count: 0,
        sensitive_notes,
        attachments,
        notes,
    })
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

    for (idx, note) in scan.notes.iter().enumerate() {
        note_index.insert(note.normalized_title.clone(), idx);
        for alias in &note.aliases {
            note_index.entry(normalized_title(alias)).or_insert(idx);
        }
        let entry = sync_state.entries.get(&note.relative_path);
        if entry.is_some_and(|entry| entry.content_hash == note.content_hash) {
            unchanged_count += 1;
            continue;
        }
        candidates.push(build_note_request(
            note,
            scan.project.clone(),
            scan.namespace.clone(),
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
            sync_state_path,
        },
        candidates,
        changed_notes,
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

pub fn build_attachment_request(
    attachment: &ObsidianAttachment,
    project: Option<String>,
    namespace: Option<String>,
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

pub fn build_writeback_markdown(
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
    markdown.push_str("\n\n## Why This Exists\n\n");
    for reason in &explain.reasons {
        markdown.push_str(&format!("- {}\n", reason));
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
    (title, markdown)
}

pub fn build_note_mirror_markdown(
    note: &ObsidianNote,
    item_id: Option<Uuid>,
    entity_id: Option<Uuid>,
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

    for attachment in attachments {
        let entry = sync_state.entries.get(&attachment.relative_path);
        if entry.is_some_and(|entry| entry.content_hash == attachment.content_hash) {
            unchanged_count += 1;
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

fn parse_markdown_note(vault: &Path, path: &Path) -> anyhow::Result<Option<ObsidianNote>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let raw = strip_roundtrip_annotation(&raw);
    let metadata = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
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
    let content_hash = hash_content(&raw);
    let modified_at = metadata.modified().ok().map(system_time_to_utc);

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
        bytes: metadata.len(),
        modified_at,
        content_hash,
    }))
}

fn parse_attachment(vault: &Path, path: &Path) -> anyhow::Result<Option<ObsidianAttachment>> {
    let metadata = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
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
    let bytes = metadata.len();
    let modified_at = metadata.modified().ok().map(system_time_to_utc);
    let raw = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let content_hash = hash_bytes(&raw);
    let sensitivity = if is_text_like_attachment(path) {
        let text = String::from_utf8_lossy(&raw);
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
        if let Some(&resolved_idx) = note_index.get(&note.normalized_title) {
            if resolved_idx == idx {
                score += 1;
            }
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
        let block = build_roundtrip_annotation(&note, Some(Uuid::nil()), Some(Uuid::nil()));
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
}
