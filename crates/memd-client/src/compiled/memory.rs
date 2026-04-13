use super::*;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CompiledMemoryHit {
    pub(crate) path: PathBuf,
    pub(crate) line: usize,
    pub(crate) section: String,
    pub(crate) text: String,
    pub(crate) score: i32,
    pub(crate) reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CompiledMemoryQualityDimension {
    pub(crate) name: String,
    pub(crate) weight: u8,
    pub(crate) score: u8,
    pub(crate) details: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CompiledMemoryQualityProbe {
    pub(crate) query: String,
    pub(crate) hit_count: usize,
    pub(crate) best_score: i32,
    pub(crate) best_path: Option<String>,
    pub(crate) best_section: Option<String>,
    pub(crate) best_text: Option<String>,
    pub(crate) reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CompiledMemoryQualityReport {
    pub(crate) root: String,
    pub(crate) benchmark_target: String,
    pub(crate) project: String,
    pub(crate) namespace: String,
    pub(crate) agent: String,
    pub(crate) session: String,
    pub(crate) tab_id: String,
    pub(crate) effective_agent: String,
    pub(crate) page_count: usize,
    pub(crate) lane_count: usize,
    pub(crate) item_count: usize,
    pub(crate) memory_file_bytes: u64,
    pub(crate) latency_ms: u64,
    pub(crate) score: u8,
    pub(crate) max_score: u8,
    pub(crate) dimensions: Vec<CompiledMemoryQualityDimension>,
    pub(crate) probes: Vec<CompiledMemoryQualityProbe>,
    pub(crate) recommendations: Vec<String>,
    pub(crate) generated_at: DateTime<Utc>,
}

pub(crate) fn resolve_compiled_memory_bundle_root(
    explicit: Option<&Path>,
) -> anyhow::Result<PathBuf> {
    if let Some(explicit) = explicit {
        if explicit.join("compiled").join("memory").exists() {
            return Ok(explicit.to_path_buf());
        }
        if explicit.join("mem.md").exists() {
            return Ok(explicit.to_path_buf());
        }
        if explicit.ends_with("compiled/memory") {
            return Ok(explicit
                .parent()
                .and_then(Path::parent)
                .map(Path::to_path_buf)
                .unwrap_or_else(|| explicit.to_path_buf()));
        }
    }

    let mut dir = std::env::current_dir().context("read current directory")?;
    loop {
        if dir.join(".memd").join("compiled").join("memory").exists()
            || dir.join(".memd").join("mem.md").exists()
        {
            return Ok(dir.join(".memd"));
        }
        if dir.join("compiled").join("memory").exists() || dir.join("mem.md").exists() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }

    anyhow::bail!("could not find compiled memory from current directory")
}

fn compiled_memory_dir(bundle_root: &Path) -> PathBuf {
    if bundle_root.join("compiled").join("memory").exists() {
        bundle_root.join("compiled").join("memory")
    } else if bundle_root
        .join(".memd")
        .join("compiled")
        .join("memory")
        .exists()
    {
        bundle_root.join(".memd").join("compiled").join("memory")
    } else {
        bundle_root.join("compiled").join("memory")
    }
}

pub(crate) fn search_compiled_memory_pages(
    bundle_root: &Path,
    query: &str,
    limit: usize,
) -> anyhow::Result<Vec<CompiledMemoryHit>> {
    let root = compiled_memory_dir(bundle_root);
    if !root.exists() {
        return Ok(Vec::new());
    }

    let query_lower = query.to_lowercase();
    let query_terms = tokenize_compiled_memory_query(query);
    let mut hits = Vec::new();
    for entry in memdrive::WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let content = fs::read_to_string(entry.path())
            .with_context(|| format!("read {}", entry.path().display()))?;
        let mut section_stack: Vec<String> = Vec::new();
        let filename = entry.file_name().to_string_lossy().to_lowercase();
        let path_text = entry.path().display().to_string().to_lowercase();
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

            let haystack = format!("{}\n{}\n{}", filename, path_text, trimmed.to_lowercase());
            if haystack.contains(&query_lower) {
                let section = if section_stack.is_empty() {
                    String::from("(top level)")
                } else {
                    section_stack.join(" > ")
                };
                let (score, reasons) = score_compiled_memory_hit(
                    &query_lower,
                    &query_terms,
                    entry.path(),
                    &section,
                    trimmed,
                );
                hits.push(CompiledMemoryHit {
                    path: entry.path().to_path_buf(),
                    line: idx + 1,
                    section,
                    text: trimmed.to_string(),
                    score,
                    reasons,
                });
            }
        }
    }

    hits.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.path.cmp(&b.path))
            .then_with(|| a.line.cmp(&b.line))
    });
    hits.truncate(limit);
    Ok(hits)
}

pub(crate) fn compiled_memory_target(args: &MemoryArgs) -> Option<&str> {
    args.item
        .as_deref()
        .or(args.lane.as_deref())
        .or(args.open.as_deref())
}

#[derive(Debug, Clone)]
pub(crate) struct CompiledMemoryIndex {
    pub(crate) project: String,
    pub(crate) namespace: String,
    pub(crate) agent: String,
    pub(crate) session: String,
    pub(crate) tab_id: String,
    pub(crate) effective_agent: String,
    pub(crate) lane_count: usize,
    pub(crate) item_count: usize,
    pub(crate) pages: Vec<String>,
    pub(crate) entries: Vec<CompiledMemoryIndexEntry>,
}

#[derive(Debug, Clone)]
pub(crate) struct CompiledMemoryIndexEntry {
    pub(crate) kind: String,
    pub(crate) lane: String,
    pub(crate) label: String,
    pub(crate) path: String,
    pub(crate) relative_path: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct CompiledMemoryIndexJson {
    pub(crate) root: String,
    pub(crate) project: String,
    pub(crate) namespace: String,
    pub(crate) agent: String,
    pub(crate) session: String,
    pub(crate) tab_id: String,
    pub(crate) effective_agent: String,
    pub(crate) lane_count: usize,
    pub(crate) item_count: usize,
    pub(crate) pages: Vec<String>,
    pub(crate) entries: Vec<CompiledMemoryIndexEntryJson>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CompiledMemoryIndexEntryJson {
    pub(crate) kind: String,
    pub(crate) lane: String,
    pub(crate) label: String,
    pub(crate) path: String,
    pub(crate) relative_path: String,
}

pub(crate) fn render_compiled_memory_index(
    bundle_root: &Path,
) -> anyhow::Result<CompiledMemoryIndex> {
    let root = compiled_memory_dir(bundle_root);
    let runtime = read_bundle_runtime_config(bundle_root).ok().flatten();
    let project = runtime
        .as_ref()
        .and_then(|config| config.project.as_deref())
        .unwrap_or("none")
        .to_string();
    let namespace = runtime
        .as_ref()
        .and_then(|config| config.namespace.as_deref())
        .unwrap_or("none")
        .to_string();
    let agent = runtime
        .as_ref()
        .and_then(|config| config.agent.as_deref())
        .unwrap_or("none")
        .to_string();
    let session = runtime
        .as_ref()
        .and_then(|config| config.session.as_deref())
        .unwrap_or("none")
        .to_string();
    let tab_id = runtime
        .as_ref()
        .and_then(|config| config.tab_id.as_deref())
        .unwrap_or("none")
        .to_string();
    let effective_agent = runtime
        .as_ref()
        .and_then(|config| config.agent.as_deref())
        .map(|value| {
            compose_agent_identity(
                value,
                runtime
                    .as_ref()
                    .and_then(|config| config.session.as_deref()),
            )
        })
        .unwrap_or_else(|| "none".to_string());
    if !root.exists() {
        return Ok(CompiledMemoryIndex {
            project,
            namespace,
            agent,
            session,
            tab_id,
            effective_agent,
            lane_count: 0,
            item_count: 0,
            pages: Vec::new(),
            entries: Vec::new(),
        });
    }

    let mut lanes = Vec::new();
    let mut pages = Vec::new();
    let mut entries = Vec::new();
    let mut item_count = 0usize;
    for lane in [
        MemoryObjectLane::Context,
        MemoryObjectLane::Working,
        MemoryObjectLane::Inbox,
        MemoryObjectLane::Recovery,
        MemoryObjectLane::Semantic,
        MemoryObjectLane::Workspace,
    ] {
        let lane_path = root.join(format!("{}.md", lane.slug()));
        if lane_path.exists() {
            lanes.push(lane.slug().to_string());
            pages.push(lane_path.display().to_string());
            entries.push(CompiledMemoryIndexEntry {
                kind: "lane".to_string(),
                lane: lane.slug().to_string(),
                label: lane.title().to_string(),
                path: lane_path.display().to_string(),
                relative_path: lane_path
                    .strip_prefix(&root)
                    .unwrap_or(&lane_path)
                    .display()
                    .to_string(),
            });
            let item_dir = root.join("items").join(lane.slug());
            if item_dir.exists() {
                for entry in fs::read_dir(&item_dir)
                    .with_context(|| format!("read {}", item_dir.display()))?
                {
                    let entry = entry.with_context(|| format!("read {}", item_dir.display()))?;
                    let path = entry.path();
                    if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
                        item_count += 1;
                        pages.push(path.display().to_string());
                        let label = path
                            .file_stem()
                            .and_then(|stem| stem.to_str())
                            .unwrap_or("")
                            .to_string();
                        entries.push(CompiledMemoryIndexEntry {
                            kind: "item".to_string(),
                            lane: lane.slug().to_string(),
                            label,
                            path: path.display().to_string(),
                            relative_path: path
                                .strip_prefix(&root)
                                .unwrap_or(&path)
                                .display()
                                .to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(CompiledMemoryIndex {
        project,
        namespace,
        agent,
        session,
        tab_id,
        effective_agent,
        lane_count: lanes.len(),
        item_count,
        pages,
        entries,
    })
}

pub(crate) fn filter_compiled_memory_index(
    index: CompiledMemoryIndex,
    lanes_only: bool,
    items_only: bool,
    filter: Option<&str>,
) -> CompiledMemoryIndex {
    let filter_lower = filter.map(|value| value.to_lowercase());
    let entries = index
        .entries
        .into_iter()
        .filter(|entry| {
            let page_lower = entry.path.to_lowercase();
            let is_item = entry.kind == "item";
            let is_lane = entry.kind == "lane";
            let matches_kind = if lanes_only {
                is_lane
            } else if items_only {
                is_item
            } else {
                true
            };
            let matches_filter = filter_lower
                .as_ref()
                .is_none_or(|needle| page_lower.contains(needle));
            matches_kind && matches_filter
        })
        .collect::<Vec<_>>();
    let pages = entries
        .iter()
        .map(|entry| entry.path.clone())
        .collect::<Vec<_>>();
    let lane_count = entries.iter().filter(|entry| entry.kind == "lane").count();
    let item_count = entries.len().saturating_sub(lane_count);
    CompiledMemoryIndex {
        project: index.project,
        namespace: index.namespace,
        agent: index.agent,
        session: index.session,
        tab_id: index.tab_id,
        effective_agent: index.effective_agent,
        lane_count,
        item_count,
        pages,
        entries,
    }
}

pub(crate) fn render_compiled_memory_index_summary(
    bundle_root: &Path,
    index: &CompiledMemoryIndex,
) -> String {
    let mut output = format!(
        "memory index root={} project={} namespace={} session={} tab_id={} agent={} effective_agent={} lanes={} items={} pages={}",
        bundle_root.display(),
        index.project,
        index.namespace,
        index.session,
        index.tab_id,
        index.agent,
        index.effective_agent,
        index.lane_count,
        index.item_count,
        index.pages.len(),
    );
    if let Some(first) = index.pages.first() {
        output.push_str(&format!(" first={}", first));
    }
    output
}

pub(crate) fn render_compiled_memory_index_json(
    bundle_root: &Path,
    index: &CompiledMemoryIndex,
) -> CompiledMemoryIndexJson {
    CompiledMemoryIndexJson {
        root: bundle_root.display().to_string(),
        project: index.project.clone(),
        namespace: index.namespace.clone(),
        agent: index.agent.clone(),
        session: index.session.clone(),
        tab_id: index.tab_id.clone(),
        effective_agent: index.effective_agent.clone(),
        lane_count: index.lane_count,
        item_count: index.item_count,
        pages: index.pages.clone(),
        entries: index
            .entries
            .iter()
            .map(|entry| CompiledMemoryIndexEntryJson {
                kind: entry.kind.clone(),
                lane: entry.lane.clone(),
                label: entry.label.clone(),
                path: entry.path.clone(),
                relative_path: entry.relative_path.clone(),
            })
            .collect(),
    }
}

pub(crate) fn render_compiled_memory_index_markdown(
    bundle_root: &Path,
    index: &CompiledMemoryIndex,
) -> String {
    let mut output = String::new();
    output.push_str("# memd memory index\n\n");
    output.push_str(&format!("- Root: `{}`\n", bundle_root.display()));
    output.push_str(&format!("- Project: `{}`\n", index.project));
    output.push_str(&format!("- Namespace: `{}`\n", index.namespace));
    output.push_str(&format!("- Session: `{}`\n", index.session));
    output.push_str(&format!("- Tab: `{}`\n", index.tab_id));
    output.push_str(&format!("- Agent: `{}`\n", index.agent));
    output.push_str(&format!("- Effective agent: `{}`\n", index.effective_agent));
    output.push_str(&format!("- Lanes: `{}`\n", index.lane_count));
    output.push_str(&format!("- Items: `{}`\n\n", index.item_count));
    if index.pages.is_empty() {
        output.push_str("No compiled memory pages found.\n");
        return output;
    }
    output.push_str("## Pages\n\n");
    for page in &index.pages {
        output.push_str(&format!("- `{}`\n", page));
    }
    output
}

pub(crate) fn render_compiled_memory_index_grouped_markdown(
    bundle_root: &Path,
    index: &CompiledMemoryIndex,
    expand_items: bool,
) -> String {
    let mut grouped = BTreeMap::<String, Vec<(String, String)>>::new();
    for page in &index.pages {
        if let Some((lane, label)) = compiled_memory_page_group(page) {
            grouped.entry(lane).or_default().push((label, page.clone()));
        }
    }

    let mut output = String::new();
    output.push_str("# memd memory index\n\n");
    output.push_str(&format!("- Root: `{}`\n", bundle_root.display()));
    output.push_str(&format!("- Project: `{}`\n", index.project));
    output.push_str(&format!("- Namespace: `{}`\n", index.namespace));
    output.push_str(&format!("- Session: `{}`\n", index.session));
    output.push_str(&format!("- Tab: `{}`\n", index.tab_id));
    output.push_str(&format!("- Agent: `{}`\n", index.agent));
    output.push_str(&format!("- Effective agent: `{}`\n", index.effective_agent));
    output.push_str(&format!("- Lanes: `{}`\n", index.lane_count));
    output.push_str(&format!("- Items: `{}`\n\n", index.item_count));
    if grouped.is_empty() {
        output.push_str("No compiled memory pages found.\n");
        return output;
    }
    for (lane, mut entries) in grouped {
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        output.push_str(&format!("## {}\n\n", lane_title(&lane)));
        if expand_items {
            for (label, page) in entries {
                output.push_str(&format!("- [{}]({})\n", label, page));
            }
        } else if let Some((label, page)) = entries.into_iter().next() {
            output.push_str(&format!("- [{}]({})\n", label, page));
            let remaining = index
                .pages
                .iter()
                .filter(|candidate| {
                    compiled_memory_page_group(candidate)
                        .map(|(lane_slug, _)| lane_slug == lane)
                        .unwrap_or(false)
                })
                .count()
                .saturating_sub(1);
            if remaining > 0 {
                output.push_str(&format!("  - +{} more item(s)\n", remaining));
            }
        }
        output.push('\n');
    }
    output
}

fn compiled_memory_page_group(page: &str) -> Option<(String, String)> {
    let path = Path::new(page);
    let file_name = path.file_name()?.to_str()?.to_string();
    if let Some(parent) = path.parent() {
        if parent
            .components()
            .any(|component| component.as_os_str() == "items")
        {
            let lane = path.parent()?.file_name()?.to_str()?.to_string();
            let label = path.file_stem()?.to_str()?.to_string();
            return Some((lane, label));
        }
    }

    let stem = Path::new(&file_name).file_stem()?.to_str()?.to_string();
    let title = lane_title(&stem);
    Some((stem, title))
}

fn tokenize_compiled_memory_query(query: &str) -> Vec<String> {
    query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| part.to_ascii_lowercase())
        .collect()
}

fn score_compiled_memory_hit(
    query_lower: &str,
    query_terms: &[String],
    path: &Path,
    section: &str,
    text: &str,
) -> (i32, Vec<String>) {
    let mut score = 0i32;
    let mut reasons = Vec::new();
    let path_lower = path.display().to_string().to_lowercase();
    let stem_lower = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("")
        .to_lowercase();
    let section_lower = section.to_lowercase();
    let text_lower = text.to_lowercase();

    if !query_lower.is_empty() && (stem_lower == query_lower || path_lower.contains(query_lower)) {
        score += 45;
        reasons.push("path_match".to_string());
    }
    if !query_lower.is_empty() && section_lower.contains(query_lower) {
        score += 25;
        reasons.push("section_match".to_string());
    }
    if !query_lower.is_empty() && text_lower.contains(query_lower) {
        score += 30;
        reasons.push("text_match".to_string());
    }

    let overlap = query_terms
        .iter()
        .filter(|term| {
            text_lower.contains(term.as_str())
                || section_lower.contains(term.as_str())
                || path_lower.contains(term.as_str())
        })
        .count();
    if overlap > 0 {
        score += (overlap as i32) * 8;
        reasons.push(format!("term_overlap={overlap}"));
    }
    if section == "(top level)" {
        score += 5;
        reasons.push("top_level".to_string());
    }
    (score, reasons)
}

pub(crate) fn lane_title(slug: &str) -> String {
    let mut chars = slug.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

pub(crate) fn resolve_compiled_memory_page(
    bundle_root: &Path,
    target: &str,
) -> anyhow::Result<PathBuf> {
    let root = compiled_memory_dir(bundle_root);
    let normalized = target.trim().to_lowercase();
    if normalized.is_empty() {
        anyhow::bail!("memory page target cannot be empty")
    }

    let lane_pages = [
        MemoryObjectLane::Context,
        MemoryObjectLane::Working,
        MemoryObjectLane::Inbox,
        MemoryObjectLane::Recovery,
        MemoryObjectLane::Semantic,
        MemoryObjectLane::Workspace,
    ];
    for lane in lane_pages {
        if normalized == lane.slug() || normalized == lane.title().to_lowercase() {
            let path = root.join(format!("{}.md", lane.slug()));
            if path.exists() {
                return Ok(path);
            }
        }
    }

    let direct = if target.ends_with(".md") {
        root.join(target)
    } else {
        root.join(format!("{target}.md"))
    };
    if direct.exists() {
        return Ok(direct);
    }

    let exact_markdown_name = format!("{normalized}.md");
    let mut partial_match: Option<PathBuf> = None;
    for entry in memdrive::WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let path = entry.path();
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("")
            .to_lowercase();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_lowercase();
        if stem == normalized || file_name == exact_markdown_name {
            return Ok(path.to_path_buf());
        }
        if partial_match.is_none()
            && (stem.contains(&normalized)
                || file_name.contains(&normalized)
                || path
                    .display()
                    .to_string()
                    .to_lowercase()
                    .contains(&normalized))
        {
            partial_match = Some(path.to_path_buf());
        }
    }

    if let Some(path) = partial_match {
        return Ok(path);
    }

    if let Some(hit) = search_compiled_memory_pages(bundle_root, target, 1)?
        .into_iter()
        .next()
    {
        return Ok(hit.path);
    }

    anyhow::bail!("compiled memory page '{}' not found", target)
}

pub(crate) fn render_compiled_memory_search_summary(
    bundle_root: &Path,
    query: &str,
    hits: &[CompiledMemoryHit],
) -> String {
    let mut output = format!(
        "memory query=\"{}\" root={} hits={}",
        compact_inline(query, 48),
        bundle_root.display(),
        hits.len(),
    );
    if let Some(first) = hits.first() {
        let reasons = if first.reasons.is_empty() {
            "none".to_string()
        } else {
            first.reasons.join(",")
        };
        output.push_str(&format!(
            " best={} score={} path={} line={} section={} reasons={} text={}",
            first
                .path
                .file_stem()
                .map(|stem| stem.to_string_lossy().to_string())
                .unwrap_or_else(|| first.path.display().to_string()),
            first.score,
            first.path.display(),
            first.line,
            first.section,
            reasons,
            compact_inline(&first.text, 120)
        ));
    }
    output
}

pub(crate) fn render_compiled_memory_search_markdown(
    bundle_root: &Path,
    query: &str,
    hits: &[CompiledMemoryHit],
) -> String {
    let mut output = String::new();
    output.push_str("# memd memory search\n\n");
    output.push_str(&format!("- Query: `{}`\n", query));
    output.push_str(&format!("- Root: `{}`\n", bundle_root.display()));
    output.push_str(&format!("- Hits: `{}`\n\n", hits.len()));
    if hits.is_empty() {
        output.push_str("No matches found.\n");
        return output;
    }
    for hit in hits {
        let reasons = if hit.reasons.is_empty() {
            "none".to_string()
        } else {
            hit.reasons.join(",")
        };
        output.push_str(&format!(
            "- `{}`:{} [{}] score={} reasons={}\n  - {}\n",
            hit.path.display(),
            hit.line,
            hit.section,
            hit.score,
            reasons,
            hit.text
        ));
    }
    output
}

pub(crate) fn render_compiled_memory_page_summary(path: &Path, content: &str) -> String {
    let preview = content
        .lines()
        .take(6)
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" | ");
    format!(
        "memory page={} preview={}",
        path.display(),
        compact_inline(&preview, 160)
    )
}

pub(crate) fn render_compiled_memory_page_markdown(path: &Path, content: &str) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "# memd memory page\n\n- path: `{}`\n\n",
        path.display()
    ));
    output.push_str(content);
    if !content.ends_with('\n') {
        output.push('\n');
    }
    output
}

pub(crate) fn build_compiled_memory_quality_report(
    bundle_root: &Path,
) -> anyhow::Result<CompiledMemoryQualityReport> {
    let started = Instant::now();
    let index = render_compiled_memory_index(bundle_root)?;
    let memory_file = bundle_root.join("mem.md");
    let event_log = read_bundle_event_log(bundle_root).unwrap_or_default();
    let memory_file_bytes = fs::metadata(&memory_file)
        .map(|meta| meta.len())
        .unwrap_or(0);
    let memory_file_text = fs::read_to_string(&memory_file).unwrap_or_default();
    let semantic_page_present = index
        .entries
        .iter()
        .any(|entry| entry.kind == "lane" && entry.lane == MemoryObjectLane::Semantic.slug());

    let mut probes = Vec::new();
    for query in [
        index.project.as_str(),
        index.namespace.as_str(),
        index.session.as_str(),
        index.tab_id.as_str(),
        "working",
        "inbox",
        "semantic",
        "session_continuity",
        "procedural",
        "canonical",
    ] {
        if query == "none" || query.is_empty() {
            continue;
        }
        let hits = search_compiled_memory_pages(bundle_root, query, 1)?;
        let best = hits.first();
        probes.push(CompiledMemoryQualityProbe {
            query: query.to_string(),
            hit_count: hits.len(),
            best_score: best.map(|hit| hit.score).unwrap_or(0),
            best_path: best.map(|hit| hit.path.display().to_string()),
            best_section: best.map(|hit| hit.section.clone()),
            best_text: best.map(|hit| compact_inline(&hit.text, 120)),
            reasons: best.map(|hit| hit.reasons.clone()).unwrap_or_default(),
        });
    }

    let scope_score = if index.project != "none"
        && index.namespace != "none"
        && index.session != "none"
        && index.tab_id != "none"
    {
        100
    } else if index.project != "none" && index.namespace != "none" {
        80
    } else {
        60
    };

    let coverage_score =
        clamp_u8((20 + index.pages.len() as i32 * 4 + index.item_count as i32 * 2).min(100));
    let retrieval_score = if probes.is_empty() {
        0
    } else {
        clamp_u8(
            probes.iter().map(|probe| probe.best_score).sum::<i32>() / probes.len().max(1) as i32,
        )
    };
    let freshness_score = if let Some(latest_event) = event_log.first() {
        let age_minutes = (Utc::now() - latest_event.recorded_at).num_minutes();
        if age_minutes <= 15 {
            100
        } else if age_minutes <= 60 {
            85
        } else if age_minutes <= 240 {
            70
        } else {
            50
        }
    } else if memory_file.exists() {
        70
    } else {
        40
    };
    let contradiction_count = memory_file_text.matches("contested").count()
        + memory_file_text.matches("contradiction").count()
        + memory_file_text.matches("superseded").count()
        + memory_file_text.matches("expired").count()
        + memory_file_text.matches("conflict").count();
    let contradiction_score = if contradiction_count == 0 {
        100
    } else {
        clamp_u8(100 - (contradiction_count as i32 * 15).min(65))
    };
    let token_efficiency_score = if memory_file_bytes == 0 {
        30
    } else if memory_file_bytes <= 12_000 {
        100
    } else if memory_file_bytes <= 24_000 {
        90
    } else if memory_file_bytes <= 40_000 {
        80
    } else if memory_file_bytes <= 64_000 {
        70
    } else {
        55
    };
    let provenance_score = if memory_file.exists()
        && (memory_file_text.contains("source_note") || memory_file_text.contains("source_path"))
    {
        100
    } else if index.pages.len() > 0 {
        80
    } else {
        40
    };
    let latency_ms = started.elapsed().as_millis() as u64;
    let latency_score = if latency_ms <= 50 {
        100
    } else if latency_ms <= 100 {
        90
    } else if latency_ms <= 200 {
        80
    } else if latency_ms <= 400 {
        70
    } else {
        55
    };
    let semantic_alignment_score = if semantic_page_present { 100 } else { 70 };

    let dimensions = vec![
        CompiledMemoryQualityDimension {
            name: "scope".to_string(),
            weight: 15,
            score: scope_score,
            details: "project, namespace, session, and tab are visible in the memory surface"
                .to_string(),
        },
        CompiledMemoryQualityDimension {
            name: "coverage".to_string(),
            weight: 15,
            score: coverage_score,
            details: "compiled pages and items are present as browsable memory objects".to_string(),
        },
        CompiledMemoryQualityDimension {
            name: "retrieval".to_string(),
            weight: 20,
            score: retrieval_score,
            details: "probe queries rank visible pages with score and reasons".to_string(),
        },
        CompiledMemoryQualityDimension {
            name: "freshness".to_string(),
            weight: 15,
            score: freshness_score,
            details: "recent events and live truth keep the memory surface fresh".to_string(),
        },
        CompiledMemoryQualityDimension {
            name: "contradiction".to_string(),
            weight: 10,
            score: contradiction_score,
            details: "unresolved contradiction pressure stays visible and controlled".to_string(),
        },
        CompiledMemoryQualityDimension {
            name: "token_efficiency".to_string(),
            weight: 10,
            score: token_efficiency_score,
            details: "the compiled memory surface stays small enough to read quickly".to_string(),
        },
        CompiledMemoryQualityDimension {
            name: "provenance".to_string(),
            weight: 5,
            score: provenance_score,
            details: "compiled pages keep source anchors visible".to_string(),
        },
        CompiledMemoryQualityDimension {
            name: "latency".to_string(),
            weight: 5,
            score: latency_score,
            details: format!("quality report generated in {latency_ms}ms"),
        },
        CompiledMemoryQualityDimension {
            name: "semantic_alignment".to_string(),
            weight: 5,
            score: semantic_alignment_score,
            details: "semantic lane exists alongside the markdown base".to_string(),
        },
    ];
    let weighted_total: u32 = dimensions
        .iter()
        .map(|dimension| (dimension.score as u32 * dimension.weight as u32) / 100)
        .sum();
    let score = weighted_total.min(100) as u8;

    let mut recommendations = Vec::new();
    if scope_score < 100 {
        recommendations
            .push("stamp project, namespace, session, and tab into runtime scope".to_string());
    }
    if retrieval_score < 80 {
        recommendations
            .push("tighten search ranking and keep the best hit explainable".to_string());
    }
    if provenance_score < 80 {
        recommendations
            .push("surface source links in the compiled pages more aggressively".to_string());
    }
    if freshness_score < 80 {
        recommendations.push("refresh live truth sooner so hot memory stays current".to_string());
    }
    if contradiction_score < 90 {
        recommendations
            .push("resolve contested or superseded facts before they pile up".to_string());
    }
    if token_efficiency_score < 70 {
        recommendations.push("trim the visible surface or split cold pages out".to_string());
    }
    if semantic_alignment_score < 80 {
        recommendations.push("sync compiled pages into the semantic backend lane".to_string());
    }
    if latency_score < 80 {
        recommendations
            .push("cut the quality path cost so the report stays cheap to run".to_string());
    }
    if recommendations.is_empty() {
        recommendations
            .push("memory surface looks healthy; keep the current hybrid base".to_string());
    }

    Ok(CompiledMemoryQualityReport {
        root: bundle_root.display().to_string(),
        benchmark_target: "supermemory".to_string(),
        project: index.project,
        namespace: index.namespace,
        agent: index.agent,
        session: index.session,
        tab_id: index.tab_id,
        effective_agent: index.effective_agent,
        page_count: index.pages.len(),
        lane_count: index.lane_count,
        item_count: index.item_count,
        memory_file_bytes,
        latency_ms,
        score,
        max_score: 100,
        dimensions,
        probes,
        recommendations,
        generated_at: Utc::now(),
    })
}

pub(crate) fn render_compiled_memory_quality_summary(
    bundle_root: &Path,
    report: &CompiledMemoryQualityReport,
) -> String {
    let best_probe = report
        .probes
        .iter()
        .max_by_key(|probe| probe.best_score)
        .map(|probe| {
            format!(
                "{}:{}@{}",
                probe.query,
                probe.best_score,
                probe.best_path.as_deref().unwrap_or("none")
            )
        })
        .unwrap_or_else(|| "none".to_string());
    let dims = report
        .dimensions
        .iter()
        .map(|dim| format!("{}={}", dim.name, dim.score))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "memory_quality benchmark={} root={} project={} namespace={} session={} tab={} agent={} score={}/{} pages={} lanes={} items={} mem_bytes={} latency_ms={} probes={} best_probe={} dims={} recommendations={}",
        report.benchmark_target,
        bundle_root.display(),
        report.project,
        report.namespace,
        report.session,
        report.tab_id,
        report.effective_agent,
        report.score,
        report.max_score,
        report.page_count,
        report.lane_count,
        report.item_count,
        report.memory_file_bytes,
        report.latency_ms,
        report.probes.len(),
        best_probe,
        dims,
        report.recommendations.join(" | ")
    )
}

pub(crate) fn render_compiled_memory_quality_markdown(
    bundle_root: &Path,
    report: &CompiledMemoryQualityReport,
) -> String {
    let mut output = String::new();
    output.push_str("# memd memory quality\n\n");
    output.push_str(&format!("- Root: `{}`\n", bundle_root.display()));
    output.push_str(&format!(
        "- Benchmark target: `{}`\n",
        report.benchmark_target
    ));
    output.push_str(&format!("- Project: `{}`\n", report.project));
    output.push_str(&format!("- Namespace: `{}`\n", report.namespace));
    output.push_str(&format!("- Session: `{}`\n", report.session));
    output.push_str(&format!("- Tab: `{}`\n", report.tab_id));
    output.push_str(&format!("- Agent: `{}`\n", report.effective_agent));
    output.push_str(&format!(
        "- Score: `{}/{}`\n",
        report.score, report.max_score
    ));
    output.push_str(&format!("- Pages: `{}`\n", report.page_count));
    output.push_str(&format!("- Lanes: `{}`\n", report.lane_count));
    output.push_str(&format!("- Items: `{}`\n", report.item_count));
    output.push_str(&format!(
        "- Memory bytes: `{}`\n\n",
        report.memory_file_bytes
    ));
    output.push_str(&format!("- Latency ms: `{}`\n\n", report.latency_ms));
    output.push_str("## Dimensions\n\n");
    for dim in &report.dimensions {
        output.push_str(&format!(
            "- {}: {}/{} — {}\n",
            dim.name, dim.score, dim.weight, dim.details
        ));
    }
    output.push_str("\n## Probes\n\n");
    for probe in &report.probes {
        output.push_str(&format!(
            "- `{}` -> score={} hits={} path={} section={} reasons={} text={}\n",
            probe.query,
            probe.best_score,
            probe.hit_count,
            probe.best_path.as_deref().unwrap_or("none"),
            probe.best_section.as_deref().unwrap_or("none"),
            if probe.reasons.is_empty() {
                "none".to_string()
            } else {
                probe.reasons.join(",")
            },
            probe.best_text.as_deref().unwrap_or("none"),
        ));
    }
    output.push_str("\n## Recommendations\n\n");
    for recommendation in &report.recommendations {
        output.push_str(&format!("- {}\n", recommendation));
    }
    output
}

pub(crate) fn render_compiled_memory_quality_json(
    bundle_root: &Path,
    report: &CompiledMemoryQualityReport,
) -> CompiledMemoryQualityReport {
    let _ = bundle_root;
    report.clone()
}

pub(crate) fn clamp_u8(value: i32) -> u8 {
    value.clamp(0, 100) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_compiled_memory_quality_report_scores_session_continuity_probe() {
        let root = std::env::temp_dir().join(format!(
            "memd-compiled-memory-quality-{}",
            uuid::Uuid::new_v4()
        ));
        let compiled = root.join("compiled").join("memory");
        fs::create_dir_all(&compiled).expect("create compiled memory dir");
        fs::write(
            root.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "session-alpha",
  "tab_id": "tab-alpha"
}
"#,
        )
        .expect("write runtime config");
        fs::write(
            root.join("mem.md"),
            "# memd memory\n\n## Scope\n\n- source_note: [[Working]]\n",
        )
        .expect("write memory surface");
        fs::write(
            compiled.join("working.md"),
            "# Working\n\nsession_continuity stays hot.\n\nprocedural memory stays explicit.\n\ncanonical memory stays explicit.\n",
        )
        .expect("write working page");

        let report = build_compiled_memory_quality_report(&root).expect("build quality report");
        assert!(
            report
                .probes
                .iter()
                .any(|probe| probe.query == "session_continuity" && probe.best_score > 0)
        );
        assert!(
            report
                .probes
                .iter()
                .any(|probe| probe.query == "procedural" && probe.best_score > 0)
        );
        assert!(
            report
                .probes
                .iter()
                .any(|probe| probe.query == "canonical" && probe.best_score > 0)
        );

        fs::remove_dir_all(root).expect("cleanup memory quality temp dir");
    }
}
