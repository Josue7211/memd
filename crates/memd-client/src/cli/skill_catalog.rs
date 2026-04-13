use super::*;

#[derive(Debug, Clone)]
pub(crate) struct SkillCatalog {
    pub(crate) root: PathBuf,
    pub(crate) builtins: Vec<SkillCatalogEntry>,
    pub(crate) custom: Vec<SkillCatalogEntry>,
    pub(crate) cache_hits: usize,
    pub(crate) cache_scanned: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct SkillCatalogEntry {
    pub(crate) name: String,
    pub(crate) path: Option<PathBuf>,
    pub(crate) summary: String,
    pub(crate) source: String,
    pub(crate) status: String,
    pub(crate) usage: String,
    pub(crate) decision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillCatalogCacheFile {
    entries: Vec<SkillCatalogCacheEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillCatalogCacheEntry {
    path: String,
    len: u64,
    modified: Option<i64>,
    name: String,
    summary: String,
    source: String,
    status: String,
    usage: String,
    decision: String,
}

pub(crate) fn resolve_skill_catalog_root(explicit: Option<&Path>) -> anyhow::Result<PathBuf> {
    if let Some(explicit) = explicit {
        return Ok(explicit.to_path_buf());
    }

    if let Some(bundle_root) = resolve_default_bundle_root()? {
        let skill_root = bundle_root.join("skills");
        if skill_root.exists() {
            return Ok(skill_root);
        }
    }

    Ok(default_global_bundle_root().join("skills"))
}

pub(crate) fn build_skill_catalog(root: &Path) -> anyhow::Result<SkillCatalog> {
    let (custom, cache_hits, cache_scanned) = discover_custom_skill_entries(root)?;
    Ok(SkillCatalog {
        root: root.to_path_buf(),
        builtins: builtin_skill_entries(),
        custom,
        cache_hits,
        cache_scanned,
    })
}

fn builtin_skill_entries() -> Vec<SkillCatalogEntry> {
    vec![
        skill_entry(
            "memd",
            "front door router",
            "built-in",
            "read-only",
            "use `memd`",
            "route to init when bundle is missing, reload when it exists, status when the user asks for readiness",
        ),
        skill_entry(
            "memd-init",
            "bundle bootstrap",
            "built-in",
            "read-only",
            "run `memd setup --agent codex` or `memd setup --global --agent codex`",
            "prefer project bundle when inside a repo and the user did not ask for global; otherwise use global",
        ),
        skill_entry(
            "memd-reload",
            "session refresh",
            "built-in",
            "read-only",
            "run `memd refresh` and then `memd status`",
            "refresh the global bundle first, then layer the repo bundle if present",
        ),
        skill_entry(
            "memd-status",
            "readiness check",
            "built-in",
            "read-only",
            "run `memd status` or `memd status --output .memd`",
            "prefer project bundle status when inside a repo, otherwise global status",
        ),
        skill_entry(
            "memd-hive",
            "hive and session wiring",
            "built-in",
            "read-only",
            "run `memd hive ...` to configure a bundle hive",
            "prefer this for repo/workspace hive setup because it initializes, applies metadata, and publishes a heartbeat",
        ),
        skill_entry(
            "memd-group-link",
            "persistent group anchor",
            "built-in",
            "read-only",
            "run `memd group-link ...` when you want a long-lived hive anchor",
            "use when you want a persistent group trust anchor across sessions",
        ),
        skill_entry(
            "memd-inspiration",
            "repo inspiration lane",
            "built-in",
            "read-only",
            "run `memd inspiration --query <term>`",
            "use for inspiration memory, repo study, and design/architecture recall",
        ),
        skill_entry(
            "memd-policy",
            "runtime policy view",
            "built-in",
            "read-only",
            "run `memd policy --summary`",
            "use for the runtime memory doctrine and policy snapshot",
        ),
        skill_entry(
            "memd-skill-policy",
            "skill lifecycle gate",
            "built-in",
            "read-only",
            "run `memd skill-policy --summary` or `--query`",
            "use for proposal, sandbox, activation, and audit of skills",
        ),
        skill_entry(
            "memd-obsidian",
            "vault bridge",
            "built-in",
            "read-only",
            "run `memd obsidian ...` to sync or compile vault content",
            "use when Obsidian is the human-facing knowledge workspace",
        ),
    ]
}

fn skill_entry(
    name: &str,
    summary: &str,
    source: &str,
    status: &str,
    usage: &str,
    decision: &str,
) -> SkillCatalogEntry {
    SkillCatalogEntry {
        name: name.to_string(),
        path: None,
        summary: summary.to_string(),
        source: source.to_string(),
        status: status.to_string(),
        usage: usage.to_string(),
        decision: decision.to_string(),
    }
}

fn skill_catalog_cache_path(root: &Path) -> PathBuf {
    root.join(".skill-catalog-cache.json")
}

fn read_skill_catalog_cache(
    root: &Path,
) -> anyhow::Result<BTreeMap<String, SkillCatalogCacheEntry>> {
    let path = skill_catalog_cache_path(root);
    if !path.exists() {
        return Ok(BTreeMap::new());
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let cache = serde_json::from_str::<SkillCatalogCacheFile>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(cache
        .entries
        .into_iter()
        .map(|entry| (entry.path.clone(), entry))
        .collect())
}

fn write_skill_catalog_cache(
    root: &Path,
    entries: &[SkillCatalogCacheEntry],
) -> anyhow::Result<()> {
    let path = skill_catalog_cache_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let payload = SkillCatalogCacheFile {
        entries: entries.to_vec(),
    };
    fs::write(&path, serde_json::to_string_pretty(&payload)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn discover_custom_skill_entries(
    root: &Path,
) -> anyhow::Result<(Vec<SkillCatalogEntry>, usize, usize)> {
    if !root.exists() {
        return Ok((Vec::new(), 0, 0));
    }

    let cache = read_skill_catalog_cache(root)?;
    let mut entries = Vec::new();
    let mut cache_hits = 0usize;
    let mut cache_scanned = 0usize;
    let mut cache_to_write = Vec::new();
    for entry in memdrive::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if !entry
            .file_name()
            .to_string_lossy()
            .eq_ignore_ascii_case("SKILL.md")
        {
            continue;
        }
        let path = entry.path().to_path_buf();
        cache_scanned += 1;
        let metadata = fs::metadata(&path).with_context(|| format!("stat {}", path.display()))?;
        let len = metadata.len();
        let modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs() as i64);
        let key = path.to_string_lossy().to_string();
        let cached = cache.get(&key);
        let reuse = cached.is_some_and(|cached| cached.len == len && cached.modified == modified);
        let record = if reuse {
            cache_hits += 1;
            let cached = cached.expect("cached skill entry");
            SkillCatalogEntry {
                name: cached.name.clone(),
                path: Some(path),
                summary: cached.summary.clone(),
                source: cached.source.clone(),
                status: cached.status.clone(),
                usage: cached.usage.clone(),
                decision: cached.decision.clone(),
            }
        } else {
            let raw = fs::read_to_string(&path)
                .with_context(|| format!("read skill {}", path.display()))?;
            let (name, summary) = parse_skill_metadata(&path, &raw);
            SkillCatalogEntry {
                name,
                path: Some(path),
                summary,
                source: "project".to_string(),
                status: "custom".to_string(),
                usage: "edit the file, then propose via skill-policy".to_string(),
                decision: "custom skills stay project-local until promoted by policy".to_string(),
            }
        };
        cache_to_write.push(SkillCatalogCacheEntry {
            path: key,
            len,
            modified,
            name: record.name.clone(),
            summary: record.summary.clone(),
            source: record.source.clone(),
            status: record.status.clone(),
            usage: record.usage.clone(),
            decision: record.decision.clone(),
        });
        entries.push(record);
    }

    entries.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then(left.summary.cmp(&right.summary))
    });
    cache_to_write.sort_by(|left, right| left.path.cmp(&right.path));
    write_skill_catalog_cache(root, &cache_to_write)?;
    Ok((entries, cache_hits, cache_scanned))
}

fn parse_skill_metadata(path: &Path, raw: &str) -> (String, String) {
    let fallback_name = path
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("skill")
        .to_string();

    let mut name = None;
    let mut summary = None;
    let mut lines = raw.lines();
    if lines.next().is_some_and(|line| line.trim() == "---") {
        for line in lines {
            let trimmed = line.trim();
            if trimmed == "---" {
                break;
            }
            if let Some(value) = trimmed.strip_prefix("name:") {
                let value = value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                if !value.is_empty() {
                    name = Some(value);
                }
                continue;
            }
            if let Some(value) = trimmed.strip_prefix("description:") {
                let value = value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                if !value.is_empty() {
                    summary = Some(value);
                }
            }
        }
    }

    let summary = summary
        .or_else(|| {
            raw.lines()
                .skip_while(|line| line.trim().is_empty())
                .find(|line| {
                    let trimmed = line.trim();
                    !trimmed.is_empty() && !trimmed.starts_with("---") && !trimmed.starts_with('#')
                })
                .map(|line| {
                    compact_bundle_value(line)
                        .chars()
                        .take(120)
                        .collect::<String>()
                })
        })
        .unwrap_or_else(|| "custom skill".to_string());

    (name.unwrap_or(fallback_name), summary)
}

pub(crate) fn find_skill_catalog_matches<'a>(
    catalog: &'a SkillCatalog,
    query: &str,
) -> Vec<&'a SkillCatalogEntry> {
    let normalized = normalize_skill_search_text(query);
    if normalized.is_empty() {
        return Vec::new();
    }

    let mut matches = catalog
        .builtins
        .iter()
        .chain(catalog.custom.iter())
        .filter(|entry| {
            let name = normalize_skill_search_text(&entry.name);
            let summary = normalize_skill_search_text(&entry.summary);
            let source = normalize_skill_search_text(&entry.source);
            name.contains(&normalized)
                || summary.contains(&normalized)
                || source.contains(&normalized)
        })
        .collect::<Vec<_>>();

    matches.sort_by(|left, right| left.name.cmp(&right.name));
    matches
}

fn normalize_skill_search_text(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|ch| match ch {
            '-' | '_' | '/' => ' ',
            other => other,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
