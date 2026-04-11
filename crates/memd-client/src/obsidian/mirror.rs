use super::*;

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

pub(crate) fn short_uuid(id: Uuid) -> String {
    id.to_string().chars().take(8).collect()
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
