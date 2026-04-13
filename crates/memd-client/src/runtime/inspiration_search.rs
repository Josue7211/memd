use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::harness::cache;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct InspirationHit {
    pub(crate) file: PathBuf,
    pub(crate) line: usize,
    pub(crate) section: String,
    pub(crate) text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InspirationFileFingerprint {
    path: String,
    len: u64,
    modified: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InspirationSearchCacheFile {
    root: String,
    query: String,
    limit: usize,
    files: Vec<InspirationFileFingerprint>,
    hits: Vec<InspirationHit>,
    refreshed_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub(crate) struct InspirationSearchResult {
    pub(crate) hits: Vec<InspirationHit>,
    pub(crate) cache_hits: usize,
    pub(crate) cache_scanned: usize,
}

fn inspiration_search_cache_dir(root: &Path) -> PathBuf {
    root.join(".memd").join("state").join(".inspiration-cache")
}

fn inspiration_search_cache_path(root: &Path, query: &str, limit: usize) -> PathBuf {
    let key = format!(
        "root={}|query={}|limit={}",
        root.display(),
        cache::normalize_query(query),
        limit
    );
    let hash = format!("{:x}", Sha256::digest(key.as_bytes()));
    inspiration_search_cache_dir(root).join(format!("{hash}.json"))
}

fn inspiration_file_fingerprint(path: &Path) -> anyhow::Result<InspirationFileFingerprint> {
    let metadata = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|value| value.as_secs());
    Ok(InspirationFileFingerprint {
        path: path.display().to_string(),
        len: metadata.len(),
        modified,
    })
}

fn read_inspiration_search_cache(
    root: &Path,
    query: &str,
    limit: usize,
) -> anyhow::Result<Option<InspirationSearchCacheFile>> {
    let path = inspiration_search_cache_path(root, query, limit);
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let cache = serde_json::from_str::<InspirationSearchCacheFile>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(cache))
}

fn write_inspiration_search_cache(
    root: &Path,
    query: &str,
    limit: usize,
    cache: &InspirationSearchCacheFile,
) -> anyhow::Result<()> {
    let path = inspiration_search_cache_path(root, query, limit);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(cache)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

const INSPIRATION_FILES: &[&str] = &[
    ".memd/lanes/inspiration/INSPIRATION-LANE.md",
    ".memd/lanes/inspiration/INSPIRATION-ARCHITECTURE.md",
    ".memd/lanes/inspiration/INSPIRATION-BACKLOG.md",
    ".memd/lanes/inspiration/INSPIRATION-MATRIX.md",
];

pub(crate) fn resolve_inspiration_root(explicit: Option<&Path>) -> anyhow::Result<PathBuf> {
    if let Some(explicit) = explicit {
        return Ok(explicit.to_path_buf());
    }

    let mut dir = std::env::current_dir().context("read current directory")?;
    loop {
        if dir.join(".memd/lanes/inspiration/INSPIRATION-LANE.md").exists() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }

    anyhow::bail!("could not find .memd/lanes/inspiration/INSPIRATION-LANE.md from current directory")
}

pub(crate) fn search_inspiration_lane(
    root: &Path,
    query: &str,
    limit: usize,
) -> anyhow::Result<InspirationSearchResult> {
    let query_lower = query.to_lowercase();
    let cache_key_query = cache::normalize_query(query);
    let existing_cache = read_inspiration_search_cache(root, query, limit)?;
    let mut current_fingerprints = Vec::new();
    let mut current_paths = Vec::new();
    for relative in INSPIRATION_FILES {
        let path = root.join(relative);
        if !path.exists() {
            continue;
        }
        current_fingerprints.push(inspiration_file_fingerprint(&path)?);
        current_paths.push(path);
    }

    if let Some(cache) = existing_cache.as_ref()
        && cache.query == cache_key_query
        && cache.limit == limit
        && cache.root == root.display().to_string()
        && cache.files.len() == current_fingerprints.len()
        && cache
            .files
            .iter()
            .zip(current_fingerprints.iter())
            .all(|(left, right)| {
                left.path == right.path && left.len == right.len && left.modified == right.modified
            })
    {
        return Ok(InspirationSearchResult {
            hits: cache.hits.clone(),
            cache_hits: cache.files.len(),
            cache_scanned: 0,
        });
    }

    let mut hits = Vec::new();
    let mut scanned = 0usize;

    for path in current_paths {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("read inspiration file {}", path.display()))?;
        scanned += 1;
        let mut section_stack: Vec<String> = Vec::new();

        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            let level = trimmed.chars().take_while(|c| *c == '#').count();
            if level > 0 {
                let title = trimmed[level..].trim().to_string();
                if !title.is_empty() {
                    section_stack.truncate(level.saturating_sub(1));
                    section_stack.push(title);
                }
            }

            if trimmed.to_lowercase().contains(&query_lower) {
                let section = if section_stack.is_empty() {
                    String::from("(top level)")
                } else {
                    section_stack.join(" > ")
                };
                hits.push(InspirationHit {
                    file: path.clone(),
                    line: idx + 1,
                    section,
                    text: trimmed.to_string(),
                });
                if hits.len() >= limit {
                    let cache = InspirationSearchCacheFile {
                        root: root.display().to_string(),
                        query: cache_key_query,
                        limit,
                        files: current_fingerprints,
                        hits: hits.clone(),
                        refreshed_at: Utc::now(),
                    };
                    let _ = write_inspiration_search_cache(root, query, limit, &cache);
                    return Ok(InspirationSearchResult {
                        hits,
                        cache_hits: 0,
                        cache_scanned: scanned,
                    });
                }
            }
        }
    }

    let cache = InspirationSearchCacheFile {
        root: root.display().to_string(),
        query: cache_key_query,
        limit,
        files: current_fingerprints,
        hits: hits.clone(),
        refreshed_at: Utc::now(),
    };
    let _ = write_inspiration_search_cache(root, query, limit, &cache);

    Ok(InspirationSearchResult {
        hits,
        cache_hits: 0,
        cache_scanned: scanned,
    })
}

pub(crate) fn render_inspiration_search_summary(
    root: &Path,
    query: &str,
    result: &InspirationSearchResult,
) -> String {
    let mut out = String::new();
    out.push_str(&format!("Inspiration search: {query}\n"));
    out.push_str(&format!("Root: {}\n", root.display()));
    out.push_str(&format!(
        "Matches: {} | cache_hits={} | scanned={}\n\n",
        result.hits.len(),
        result.cache_hits,
        result.cache_scanned
    ));
    for (idx, hit) in result.hits.iter().enumerate() {
        out.push_str(&format!(
            "{}. {}:{} [{}]\n   {}\n",
            idx + 1,
            hit.file.display(),
            hit.line,
            hit.section,
            hit.text
        ));
    }
    if result.hits.is_empty() {
        out.push_str("No matches found.\n");
    }
    out
}

pub(crate) fn render_inspiration_search_markdown(
    root: &Path,
    query: &str,
    result: &InspirationSearchResult,
) -> String {
    let mut out = String::new();
    out.push_str("# Inspiration Search\n\n");
    out.push_str(&format!("- Query: `{query}`\n"));
    out.push_str(&format!("- Root: `{}`\n", root.display()));
    out.push_str(&format!(
        "- Matches: `{}`\n- Cache hits: `{}`\n- Files scanned: `{}`\n\n",
        result.hits.len(),
        result.cache_hits,
        result.cache_scanned
    ));
    if result.hits.is_empty() {
        out.push_str("No matches found.\n");
        return out;
    }
    for hit in &result.hits {
        out.push_str(&format!(
            "- `{}`:{} [{}]\n  - {}\n",
            hit.file.display(),
            hit.line,
            hit.section,
            hit.text
        ));
    }
    out
}
