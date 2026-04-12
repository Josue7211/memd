use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BundleEventRecord {
    pub(crate) id: String,
    pub(crate) event_type: String,
    pub(crate) summary: String,
    pub(crate) source: String,
    pub(crate) recorded_at: DateTime<Utc>,
    pub(crate) project: Option<String>,
    pub(crate) namespace: Option<String>,
    pub(crate) workspace: Option<String>,
    pub(crate) focus: Option<String>,
    pub(crate) pressure: Option<String>,
    pub(crate) next_recovery: Option<String>,
    pub(crate) context_pressure: String,
    pub(crate) estimated_prompt_tokens: usize,
    pub(crate) working_records: usize,
    pub(crate) inbox_items: usize,
    pub(crate) rehydration_items: usize,
    pub(crate) refresh_recommended: bool,
    pub(crate) event_spine: Vec<String>,
    pub(crate) change_summary: Vec<String>,
    pub(crate) recent_repo_changes: Vec<String>,
    pub(crate) handoff_sources: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CompiledEventHit {
    pub(crate) path: PathBuf,
    pub(crate) line: usize,
    pub(crate) section: String,
    pub(crate) text: String,
}

#[derive(Debug, Clone)]
pub(crate) struct CompiledEventIndex {
    pub(crate) kind_count: usize,
    pub(crate) item_count: usize,
    pub(crate) pages: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CompiledEventIndexJson {
    pub(crate) root: String,
    pub(crate) kind_count: usize,
    pub(crate) item_count: usize,
    pub(crate) pages: Vec<String>,
}

fn bundle_event_log_path(output: &Path) -> PathBuf {
    output.join("state").join("live-events.jsonl")
}

pub(crate) fn compiled_event_dir(output: &Path) -> PathBuf {
    output.join("compiled").join("events")
}

fn compiled_event_kind_path(output: &Path, kind: &str) -> PathBuf {
    compiled_event_dir(output).join(format!("{kind}.md"))
}

fn compiled_event_item_path(output: &Path, kind: &str, id: &str) -> PathBuf {
    compiled_event_dir(output)
        .join("items")
        .join(kind)
        .join(format!("{id}.md"))
}

pub(crate) fn read_bundle_event_log(output: &Path) -> anyhow::Result<Vec<BundleEventRecord>> {
    let path = bundle_event_log_path(output);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let mut records = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let record = serde_json::from_str::<BundleEventRecord>(trimmed)
            .with_context(|| format!("parse {}", path.display()))?;
        records.push(record);
    }
    Ok(records)
}

pub(crate) fn write_bundle_event_log(
    output: &Path,
    records: &[BundleEventRecord],
) -> anyhow::Result<()> {
    let path = bundle_event_log_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut merged = std::collections::BTreeMap::<String, BundleEventRecord>::new();
    for record in read_bundle_event_log(output)? {
        merged.insert(record.id.clone(), record);
    }
    for record in records {
        merged.insert(record.id.clone(), record.clone());
    }

    let mut merged = merged.into_values().collect::<Vec<_>>();
    merged.sort_by(|left, right| {
        right
            .recorded_at
            .cmp(&left.recorded_at)
            .then_with(|| left.event_type.cmp(&right.event_type))
            .then_with(|| left.id.cmp(&right.id))
    });
    merged.truncate(256);

    let mut content = String::new();
    for record in merged {
        content.push_str(&serde_json::to_string(&record)?);
        content.push('\n');
    }
    fs::write(&path, content).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn derive_bundle_event_record(
    snapshot: &ResumeSnapshot,
    handoff: Option<&HandoffSnapshot>,
) -> BundleEventRecord {
    let event_type = if handoff.is_some() {
        "handoff_snapshot"
    } else if snapshot.refresh_recommended {
        "refresh_snapshot"
    } else if !snapshot.change_summary.is_empty() || !snapshot.recent_repo_changes.is_empty() {
        "live_snapshot"
    } else {
        "resume_snapshot"
    };

    let focus = snapshot
        .working
        .records
        .first()
        .map(|record| record.record.clone());
    let pressure = snapshot
        .inbox
        .items
        .first()
        .map(|item| item.item.content.clone());
    let next_recovery = snapshot
        .working
        .rehydration_queue
        .first()
        .map(|item| format!("{}: {}", item.label, item.summary));
    let event_spine = snapshot.event_spine();
    let mut summary = format!(
        "{event_type} project={} namespace={} agent={} working={} inbox={} rehydrate={} pressure={} refresh={} tokens={}",
        snapshot.project.as_deref().unwrap_or("none"),
        snapshot.namespace.as_deref().unwrap_or("none"),
        snapshot.agent.as_deref().unwrap_or("none"),
        snapshot.working.records.len(),
        snapshot.inbox.items.len(),
        snapshot.working.rehydration_queue.len(),
        snapshot.context_pressure(),
        snapshot.refresh_recommended,
        snapshot.estimated_prompt_tokens(),
    );
    if let Some(focus) = focus.as_deref() {
        summary.push_str(&format!(" focus=\"{}\"", compact_inline(focus, 72)));
    }
    if let Some(pressure) = pressure.as_deref() {
        summary.push_str(&format!(" pressure=\"{}\"", compact_inline(pressure, 72)));
    }
    if let Some(next_recovery) = next_recovery.as_deref() {
        summary.push_str(&format!(" next=\"{}\"", compact_inline(next_recovery, 72)));
    }
    if let Some(handoff) = handoff {
        summary.push_str(&format!(
            " handoff_sources={} target_session={} target_bundle={}",
            handoff.sources.sources.len(),
            handoff.target_session.as_deref().unwrap_or("none"),
            handoff.target_bundle.as_deref().unwrap_or("none")
        ));
    }

    let signature = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{:?}",
        event_type,
        snapshot.project.as_deref().unwrap_or("none"),
        snapshot.namespace.as_deref().unwrap_or("none"),
        snapshot.workspace.as_deref().unwrap_or("none"),
        summary,
        focus.as_deref().unwrap_or("none"),
        pressure.as_deref().unwrap_or("none"),
        next_recovery.as_deref().unwrap_or("none"),
        event_spine.join(" | "),
        handoff.as_ref().map(|value| value.sources.sources.len())
    );
    let id = format!("{}-{}", event_type, short_hash_text(&signature));

    BundleEventRecord {
        id,
        event_type: event_type.to_string(),
        summary,
        source: if handoff.is_some() {
            "handoff".to_string()
        } else if snapshot.refresh_recommended {
            "refresh".to_string()
        } else {
            "snapshot".to_string()
        },
        recorded_at: Utc::now(),
        project: snapshot.project.clone(),
        namespace: snapshot.namespace.clone(),
        workspace: snapshot.workspace.clone(),
        focus,
        pressure,
        next_recovery,
        context_pressure: snapshot.context_pressure().to_string(),
        estimated_prompt_tokens: snapshot.estimated_prompt_tokens(),
        working_records: snapshot.working.records.len(),
        inbox_items: snapshot.inbox.items.len(),
        rehydration_items: snapshot.working.rehydration_queue.len(),
        refresh_recommended: snapshot.refresh_recommended,
        event_spine,
        change_summary: snapshot.change_summary.clone(),
        recent_repo_changes: snapshot.recent_repo_changes.clone(),
        handoff_sources: handoff.as_ref().map(|value| value.sources.sources.len()),
    }
}

pub(crate) fn write_bundle_event_files(
    output: &Path,
    snapshot: &ResumeSnapshot,
    handoff: Option<&HandoffSnapshot>,
) -> anyhow::Result<()> {
    let record = derive_bundle_event_record(snapshot, handoff);
    write_bundle_event_log(output, &[record])?;

    let log = read_bundle_event_log(output)?;
    write_bundle_event_markdown_files(output, &log)?;
    write_bundle_event_object_pages(output, &log)?;
    let index = render_compiled_event_index(output)?;
    let compiled_latest = compiled_event_dir(output).join("latest.md");
    if let Some(parent) = compiled_latest.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(
        &compiled_latest,
        render_compiled_event_index_markdown(output, &index),
    )
    .with_context(|| format!("write {}", compiled_latest.display()))?;
    Ok(())
}

pub(crate) fn refresh_live_bundle_event_pages(
    output: &Path,
    snapshot: &ResumeSnapshot,
    handoff: Option<&HandoffSnapshot>,
) -> anyhow::Result<()> {
    write_bundle_event_files(output, snapshot, handoff)
}

pub(crate) fn write_bundle_event_markdown_files(
    output: &Path,
    records: &[BundleEventRecord],
) -> anyhow::Result<()> {
    let markdown = render_bundle_event_log_markdown(output, records);
    let root_events = output.join("MEMD_EVENTS.md");
    fs::write(&root_events, &markdown)
        .with_context(|| format!("write {}", root_events.display()))?;

    let agents_dir = output.join("agents");
    fs::create_dir_all(&agents_dir).with_context(|| format!("create {}", agents_dir.display()))?;
    for file_name in [
        "CODEX_EVENTS.md",
        "CLAUDE_CODE_EVENTS.md",
        "OPENCLAW_EVENTS.md",
        "OPENCODE_EVENTS.md",
    ] {
        let path = agents_dir.join(file_name);
        fs::write(&path, &markdown).with_context(|| format!("write {}", path.display()))?;
    }

    Ok(())
}

pub(crate) fn write_bundle_event_object_pages(
    output: &Path,
    records: &[BundleEventRecord],
) -> anyhow::Result<()> {
    let dir = compiled_event_dir(output);
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let mut grouped = BTreeMap::<String, Vec<&BundleEventRecord>>::new();
    for record in records {
        grouped
            .entry(record.event_type.clone())
            .or_default()
            .push(record);
    }
    for (kind, kind_records) in grouped {
        let kind_path = compiled_event_kind_path(output, &kind);
        let kind_markdown = render_bundle_event_kind_markdown(&kind, &kind_records);
        if let Some(parent) = kind_path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        fs::write(&kind_path, kind_markdown)
            .with_context(|| format!("write {}", kind_path.display()))?;
        for record in kind_records {
            let item_path = compiled_event_item_path(output, &kind, &record.id);
            if let Some(parent) = item_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            fs::write(&item_path, render_bundle_event_item_markdown(record))
                .with_context(|| format!("write {}", item_path.display()))?;
        }
    }
    Ok(())
}

pub(crate) fn resolve_compiled_event_bundle_root(
    explicit: Option<&Path>,
) -> anyhow::Result<PathBuf> {
    resolve_compiled_memory_bundle_root(explicit)
}

pub(crate) fn search_compiled_event_pages(
    bundle_root: &Path,
    query: &str,
    limit: usize,
) -> anyhow::Result<Vec<CompiledEventHit>> {
    let root = compiled_event_dir(bundle_root);
    if !root.exists() {
        return Ok(Vec::new());
    }

    let query_lower = query.to_lowercase();
    let mut hits = Vec::new();
    for entry in walkdir::WalkDir::new(&root)
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
                hits.push(CompiledEventHit {
                    path: entry.path().to_path_buf(),
                    line: idx + 1,
                    section,
                    text: trimmed.to_string(),
                });
                if hits.len() >= limit {
                    return Ok(hits);
                }
            }
        }
    }

    Ok(hits)
}

pub(crate) fn render_compiled_event_index(
    bundle_root: &Path,
) -> anyhow::Result<CompiledEventIndex> {
    let root = compiled_event_dir(bundle_root);
    if !root.exists() {
        return Ok(CompiledEventIndex {
            kind_count: 0,
            item_count: 0,
            pages: Vec::new(),
        });
    }

    let mut kinds = Vec::new();
    let mut pages = Vec::new();
    let mut item_count = 0usize;
    for entry in fs::read_dir(&root).with_context(|| format!("read {}", root.display()))? {
        let entry = entry.with_context(|| format!("read {}", root.display()))?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            if path.file_name().and_then(|name| name.to_str()) != Some("latest.md") {
                kinds.push(
                    path.file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or("")
                        .to_string(),
                );
            }
            pages.push(path.display().to_string());
        } else if path.is_dir() && path.file_name().and_then(|name| name.to_str()) == Some("items")
        {
            for kind_dir in
                fs::read_dir(&path).with_context(|| format!("read {}", path.display()))?
            {
                let kind_dir = kind_dir.with_context(|| format!("read {}", path.display()))?;
                let kind_path = kind_dir.path();
                if !kind_path.is_dir() {
                    continue;
                }
                for item in fs::read_dir(&kind_path)
                    .with_context(|| format!("read {}", kind_path.display()))?
                {
                    let item = item.with_context(|| format!("read {}", kind_path.display()))?;
                    let item_path = item.path();
                    if item_path.extension().and_then(|ext| ext.to_str()) == Some("md") {
                        item_count += 1;
                        pages.push(item_path.display().to_string());
                    }
                }
            }
        }
    }

    kinds.sort();
    kinds.dedup();
    Ok(CompiledEventIndex {
        kind_count: kinds.len(),
        item_count,
        pages,
    })
}

pub(crate) fn render_compiled_event_index_json(
    bundle_root: &Path,
    index: &CompiledEventIndex,
) -> CompiledEventIndexJson {
    CompiledEventIndexJson {
        root: bundle_root.display().to_string(),
        kind_count: index.kind_count,
        item_count: index.item_count,
        pages: index.pages.clone(),
    }
}

pub(crate) fn render_compiled_event_index_summary(
    bundle_root: &Path,
    index: &CompiledEventIndex,
) -> String {
    let mut output = format!(
        "event index root={} kinds={} items={} pages={}",
        bundle_root.display(),
        index.kind_count,
        index.item_count,
        index.pages.len(),
    );
    if let Some(first) = index.pages.first() {
        output.push_str(&format!(" first={}", first));
    }
    output
}

pub(crate) fn render_compiled_event_index_markdown(
    bundle_root: &Path,
    index: &CompiledEventIndex,
) -> String {
    let mut output = String::new();
    output.push_str("# memd event index\n\n");
    output.push_str(&format!("- Root: `{}`\n", bundle_root.display()));
    output.push_str(&format!("- Kinds: `{}`\n", index.kind_count));
    output.push_str(&format!("- Items: `{}`\n\n", index.item_count));
    if index.pages.is_empty() {
        output.push_str("No compiled event pages found.\n\n");
    } else {
        output.push_str("## Pages\n\n");
        for page in &index.pages {
            output.push_str(&format!("- `{}`\n", page));
        }
        output.push('\n');
    }

    if let Ok(raw_spine) = render_raw_spine_markdown(bundle_root) {
        if !raw_spine.is_empty() {
            output.push_str(&raw_spine);
        }
    }
    output
}

fn render_raw_spine_markdown(bundle_root: &Path) -> anyhow::Result<String> {
    let records = crate::runtime::read_raw_spine_records(bundle_root)?;
    if records.is_empty() {
        return Ok(String::new());
    }

    let mut markdown = String::from("## Raw Spine\n\n");
    for record in records.iter().take(24) {
        markdown.push_str(&format!(
            "- `{}` stage=`{}` source=`{}` path=`{}` preview=`{}`\n",
            record.event_type,
            record.stage,
            record.source_system.as_deref().unwrap_or("none"),
            record.source_path.as_deref().unwrap_or("none"),
            record.content_preview
        ));
    }
    markdown.push('\n');
    Ok(markdown)
}

pub(crate) fn render_compiled_event_search_summary(
    bundle_root: &Path,
    query: &str,
    hits: &[CompiledEventHit],
) -> String {
    let mut output = format!(
        "event query=\"{}\" root={} hits={}",
        compact_inline(query, 48),
        bundle_root.display(),
        hits.len(),
    );
    if let Some(first) = hits.first() {
        output.push_str(&format!(
            " best={} path={} line={} section={} text={}",
            first
                .path
                .file_stem()
                .map(|stem| stem.to_string_lossy().to_string())
                .unwrap_or_else(|| first.path.display().to_string()),
            first.path.display(),
            first.line,
            first.section,
            compact_inline(&first.text, 120)
        ));
    }
    output
}

pub(crate) fn render_compiled_event_page_summary(path: &Path, content: &str) -> String {
    let preview = content
        .lines()
        .take(6)
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" | ");
    format!(
        "event page={} preview={}",
        path.display(),
        compact_inline(&preview, 160)
    )
}

pub(crate) fn render_compiled_event_page_markdown(path: &Path, content: &str) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "# memd event page\n\n- path: `{}`\n\n",
        path.display()
    ));
    output.push_str(content);
    if !content.ends_with('\n') {
        output.push('\n');
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_snapshot() -> ResumeSnapshot {
        ResumeSnapshot {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("core".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            context: memd_schema::CompactContextResponse {
                route: memd_schema::RetrievalRoute::ProjectFirst,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                retrieval_order: vec![
                    memd_schema::MemoryScope::Project,
                    memd_schema::MemoryScope::Synced,
                ],
                records: Vec::new(),
            },
            working: memd_schema::WorkingMemoryResponse {
                route: memd_schema::RetrievalRoute::ProjectFirst,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                retrieval_order: vec![
                    memd_schema::MemoryScope::Project,
                    memd_schema::MemoryScope::Synced,
                ],
                budget_chars: 1600,
                used_chars: 120,
                remaining_chars: 1480,
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
                route: memd_schema::RetrievalRoute::ProjectFirst,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                items: Vec::new(),
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: Vec::new(),
            },
            sources: memd_schema::SourceMemoryResponse {
                sources: Vec::new(),
            },
            semantic: None,
            claims: SessionClaimsState::default(),
            recent_repo_changes: Vec::new(),
            change_summary: Vec::new(),
            resume_state_age_minutes: None,
            refresh_recommended: false,
        }
    }

    #[test]
    fn write_bundle_event_files_includes_raw_spine_section() {
        let dir =
            std::env::temp_dir().join(format!("memd-raw-spine-pages-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create bundle dir");

        crate::runtime::write_raw_spine_records(
            &dir,
            &[crate::runtime::derive_raw_spine_record(
                "hook_capture",
                "candidate",
                Some("hook-capture"),
                Some(".memd/agents/CODEX_WAKEUP.md"),
                Some("memd"),
                Some("main"),
                Some("core"),
                Some(0.9),
                &["raw-spine", "correction"],
                "decision: preserve raw truth before promotion",
            )],
        )
        .expect("write raw spine");

        let snapshot = test_snapshot();
        write_bundle_event_files(&dir, &snapshot, None).expect("write bundle event files");

        let latest =
            std::fs::read_to_string(dir.join("compiled/events/latest.md")).expect("read latest");
        assert!(latest.contains("## Raw Spine"));
        assert!(latest.contains("hook_capture"));
        assert!(latest.contains(".memd/agents/CODEX_WAKEUP.md"));

        std::fs::remove_dir_all(&dir).expect("cleanup bundle dir");
    }
}

fn event_kind_title(kind: &str) -> String {
    let mut words = Vec::new();
    for word in kind.split(|ch| ch == '-' || ch == '_' || ch == ' ') {
        let word = word.trim();
        if word.is_empty() {
            continue;
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            words.push(format!("{}{}", first.to_ascii_uppercase(), chars.as_str()));
        }
    }
    if words.is_empty() {
        kind.to_string()
    } else {
        words.join(" ")
    }
}

fn render_bundle_event_log_markdown(bundle_root: &Path, records: &[BundleEventRecord]) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd event log\n\n");
    markdown.push_str(&format!("- root: `{}`\n", bundle_root.display()));
    markdown.push_str(&format!("- records: `{}`\n", records.len()));
    markdown.push_str("- compiler: live snapshot -> visible event objects\n");
    markdown.push_str("- event compiler: live snapshot -> visible event objects\n");
    markdown.push_str("- source: wake / resume / refresh / checkpoint / handoff surface writes\n");
    markdown.push_str("- read: agents should inspect this before rereading raw context\n\n");
    if records.is_empty() {
        markdown.push_str("No events recorded yet.\n");
        return markdown;
    }

    markdown.push_str("## Latest\n\n");
    for record in records.iter().take(8) {
        markdown.push_str(&format!(
            "- [{}](compiled/events/items/{}/{}) {}\n",
            event_kind_title(&record.event_type),
            record.event_type,
            record.id,
            compact_inline(&record.summary, 120)
        ));
    }

    markdown.push_str("\n## Kinds\n\n");
    let mut grouped = BTreeMap::<String, usize>::new();
    for record in records {
        *grouped.entry(record.event_type.clone()).or_insert(0) += 1;
    }
    for (kind, count) in grouped {
        markdown.push_str(&format!(
            "- [{}](compiled/events/{}.md) (`{}`)\n",
            event_kind_title(&kind),
            kind,
            count
        ));
    }

    markdown.push_str("\n## Pointer\n\n");
    markdown.push_str("- live event pages are under `compiled/events/`\n");
    markdown.push_str("- item drilldowns are under `compiled/events/items/<kind>/`\n");
    markdown
}

fn render_bundle_event_kind_markdown(kind: &str, records: &[&BundleEventRecord]) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# memd event lane: {}\n\n",
        event_kind_title(kind)
    ));
    markdown.push_str(&format!("- kind: `{}`\n", kind));
    markdown.push_str(&format!("- records: `{}`\n\n", records.len()));
    if let Some(latest) = records.first() {
        markdown.push_str("## Latest\n\n");
        markdown.push_str(&format!("- {}\n", compact_inline(&latest.summary, 160)));
        if let Some(focus) = latest.focus.as_deref() {
            markdown.push_str(&format!("- focus: {}\n", compact_inline(focus, 160)));
        }
        if let Some(pressure) = latest.pressure.as_deref() {
            markdown.push_str(&format!("- pressure: {}\n", compact_inline(pressure, 160)));
        }
        if let Some(next) = latest.next_recovery.as_deref() {
            markdown.push_str(&format!("- next: {}\n", compact_inline(next, 160)));
        }
    }
    markdown.push_str("\n## Items\n\n");
    if records.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for record in records {
            markdown.push_str(&format!(
                "- [{}](items/{}/{})\n",
                record.id, kind, record.id
            ));
        }
    }
    markdown
}

fn render_bundle_event_item_markdown(record: &BundleEventRecord) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# memd event item: {}\n\n",
        event_kind_title(&record.event_type)
    ));
    markdown.push_str(&format!("- id: `{}`\n", record.id));
    markdown.push_str(&format!("- kind: `{}`\n", record.event_type));
    markdown.push_str(&format!("- source: `{}`\n", record.source));
    markdown.push_str(&format!("- recorded_at: `{}`\n", record.recorded_at));
    markdown.push_str(&format!(
        "- summary: {}\n",
        compact_inline(&record.summary, 200)
    ));
    markdown.push_str(&format!(
        "- project: `{}`\n",
        record.project.as_deref().unwrap_or("none")
    ));
    markdown.push_str(&format!(
        "- namespace: `{}`\n",
        record.namespace.as_deref().unwrap_or("none")
    ));
    markdown.push_str(&format!(
        "- workspace: `{}`\n",
        record.workspace.as_deref().unwrap_or("none")
    ));
    markdown.push_str(&format!(
        "- context_pressure: `{}`\n",
        record.context_pressure
    ));
    markdown.push_str(&format!(
        "- estimated_prompt_tokens: `{}`\n",
        record.estimated_prompt_tokens
    ));
    markdown.push_str(&format!(
        "- working_records: `{}`\n",
        record.working_records
    ));
    markdown.push_str(&format!("- inbox_items: `{}`\n", record.inbox_items));
    markdown.push_str(&format!(
        "- rehydration_items: `{}`\n",
        record.rehydration_items
    ));
    markdown.push_str(&format!(
        "- refresh_recommended: `{}`\n",
        record.refresh_recommended
    ));
    if let Some(focus) = record.focus.as_deref() {
        markdown.push_str(&format!("- focus: {}\n", compact_inline(focus, 200)));
    }
    if let Some(pressure) = record.pressure.as_deref() {
        markdown.push_str(&format!("- pressure: {}\n", compact_inline(pressure, 200)));
    }
    if let Some(next) = record.next_recovery.as_deref() {
        markdown.push_str(&format!("- next_recovery: {}\n", compact_inline(next, 200)));
    }
    if let Some(handoff_sources) = record.handoff_sources {
        markdown.push_str(&format!("- handoff_sources: `{}`\n", handoff_sources));
    }
    if !record.change_summary.is_empty() {
        markdown.push_str("\n## Changes\n\n");
        for change in record.change_summary.iter().take(6) {
            markdown.push_str(&format!("- {}\n", compact_inline(change, 180)));
        }
    }
    if !record.recent_repo_changes.is_empty() {
        markdown.push_str("\n## Repo\n\n");
        for change in record.recent_repo_changes.iter().take(6) {
            markdown.push_str(&format!("- {}\n", compact_inline(change, 180)));
        }
    }
    if !record.event_spine.is_empty() {
        markdown.push_str("\n## Spine\n\n");
        for item in record.event_spine.iter().take(6) {
            markdown.push_str(&format!("- {}\n", compact_inline(item, 180)));
        }
    }
    markdown
}

pub(crate) fn resolve_compiled_event_page(
    bundle_root: &Path,
    target: &str,
) -> anyhow::Result<PathBuf> {
    let root = compiled_event_dir(bundle_root);
    let normalized = target.trim().to_lowercase();
    if normalized.is_empty() {
        anyhow::bail!("event page target cannot be empty")
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
    for entry in walkdir::WalkDir::new(&root)
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

    if let Some(hit) = search_compiled_event_pages(bundle_root, target, 1)?
        .into_iter()
        .next()
    {
        return Ok(hit.path);
    }

    anyhow::bail!("compiled event page '{}' not found", target)
}
