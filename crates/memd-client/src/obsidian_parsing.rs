use super::*;

pub(super) fn parse_markdown_note_from_raw(
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
pub(super) fn parse_markdown_note(vault: &Path, path: &Path) -> anyhow::Result<Option<ObsidianNote>> {
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

pub(super) fn parse_attachment_from_raw(
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

pub(super) fn fallback_attachment_match(
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
