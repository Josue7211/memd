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
pub(super) fn parse_markdown_note(
    vault: &Path,
    path: &Path,
) -> anyhow::Result<Option<ObsidianNote>> {
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
