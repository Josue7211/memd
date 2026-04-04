use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use chrono::Utc;
use memd_schema::{
    CandidateMemoryRequest, EntityLinkRequest, EntityRelationKind, MemoryContextFrame, MemoryKind,
    MemoryScope, SourceQuality,
};
use serde::Serialize;
use uuid::Uuid;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
pub struct ObsidianVaultScan {
    pub vault: PathBuf,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub note_count: usize,
    pub sensitive_count: usize,
    pub skipped_count: usize,
    pub sensitive_notes: Vec<ObsidianSensitiveNote>,
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
pub struct ObsidianNote {
    pub path: PathBuf,
    pub relative_path: String,
    pub title: String,
    pub normalized_title: String,
    pub excerpt: String,
    pub kind: MemoryKind,
    pub tags: Vec<String>,
    pub aliases: Vec<String>,
    pub links: Vec<String>,
    pub sensitivity: ObsidianSensitivity,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObsidianImportPreview {
    pub scan: ObsidianVaultScan,
    pub candidates: Vec<CandidateMemoryRequest>,
    pub note_index: HashMap<String, usize>,
}

pub fn scan_vault(
    vault: impl AsRef<Path>,
    project: Option<String>,
    namespace: Option<String>,
    max_notes: Option<usize>,
) -> anyhow::Result<ObsidianVaultScan> {
    let vault = vault.as_ref();
    let mut notes = Vec::new();
    let mut sensitive_notes = Vec::new();
    let mut skipped_count = 0usize;
    let mut sensitive_count = 0usize;
    let max_notes = max_notes.unwrap_or(usize::MAX);

    for entry in WalkDir::new(vault)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry
            .path()
            .extension()
            .and_then(|ext| ext.to_str())
            .is_none_or(|ext| !ext.eq_ignore_ascii_case("md"))
        {
            skipped_count += 1;
            continue;
        }
        if notes.len() >= max_notes {
            skipped_count += 1;
            continue;
        }

        if let Some(note) = parse_markdown_note(vault, entry.path())? {
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
    }

    Ok(ObsidianVaultScan {
        vault: vault.to_path_buf(),
        project,
        namespace,
        note_count: notes.len(),
        sensitive_count,
        skipped_count,
        sensitive_notes,
        notes,
    })
}

pub fn build_import_preview(
    scan: ObsidianVaultScan,
) -> (ObsidianImportPreview, Vec<CandidateMemoryRequest>) {
    let mut candidates = Vec::with_capacity(scan.notes.len());
    let mut note_index = HashMap::new();

    for (idx, note) in scan.notes.iter().enumerate() {
        note_index.insert(note.normalized_title.clone(), idx);
        candidates.push(build_note_request(
            note,
            scan.project.clone(),
            scan.namespace.clone(),
            scan.vault.clone(),
        ));
    }

    (
        ObsidianImportPreview {
            scan,
            candidates: candidates.clone(),
            note_index,
        },
        candidates,
    )
}

pub fn build_note_request(
    note: &ObsidianNote,
    project: Option<String>,
    namespace: Option<String>,
    vault: PathBuf,
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
    if !note.aliases.is_empty() {
        content.push_str(&format!("Aliases: {}\n", note.aliases.join(", ")));
    }
    if !note.tags.is_empty() {
        content.push_str(&format!("Tags: {}\n", note.tags.join(", ")));
    }
    if !note.links.is_empty() {
        content.push_str(&format!("Wiki links: {}\n", note.links.join(", ")));
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
        supersedes: Vec::new(),
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

    Ok(Some(ObsidianNote {
        path: path.to_path_buf(),
        relative_path,
        title: title.trim().to_string(),
        normalized_title: normalized_title(&title),
        excerpt,
        kind,
        tags,
        aliases,
        links,
        sensitivity: detect_sensitivity(&title, &body, frontmatter.as_ref()),
    }))
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

        let scan = scan_vault(&vault, Some("notes".to_string()), None, Some(10)).unwrap();
        assert_eq!(scan.note_count, 1);
        assert_eq!(scan.sensitive_count, 1);
        assert_eq!(scan.notes.len(), 1);
        assert_eq!(scan.notes[0].title, "Public Note");
        assert!(!scan.notes[0].sensitivity.sensitive);
    }
}
