use super::*;

pub fn open_uri(uri: &str) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg(uri);
        command
    };

    #[cfg(target_os = "linux")]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(uri);
        command
    };

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("cmd");
        command.args(["/C", "start", "", uri]);
        command
    };

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        anyhow::bail!("opening Obsidian URIs is not supported on this platform");
    }

    let status = command
        .status()
        .with_context(|| format!("launch Obsidian URI {uri}"))?;
    if !status.success() {
        anyhow::bail!("Obsidian URI launcher exited with status {status}");
    }
    Ok(())
}

fn compact_markdown_text(value: &str, max_chars: usize) -> String {
    let collapsed = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= max_chars {
        return collapsed;
    }

    let mut output = String::new();
    for ch in collapsed.chars() {
        if output.chars().count() >= max_chars.saturating_sub(1) {
            break;
        }
        output.push(ch);
    }
    output.push('…');
    output
}

pub fn build_writeback_markdown(
    vault: &Path,
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
    if let Some(workspace) = explain.item.workspace.as_deref() {
        markdown.push_str(&format!("workspace: {}\n", workspace));
    }
    markdown.push_str(&format!(
        "visibility: {}\n",
        format_visibility(explain.item.visibility)
    ));
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
    if let Some(source_link) = explain
        .item
        .source_path
        .as_deref()
        .and_then(|path| source_wikilink_for_path(vault, Path::new(path)))
    {
        markdown.push_str("\n\n## Source Note\n\n");
        markdown.push_str(&format!("- {}\n", source_link));
    }
    markdown.push_str("\n\n## Why This Exists\n\n");
    for reason in &explain.reasons {
        markdown.push_str(&format!("- {}\n", reason));
    }
    markdown.push_str(&format!(
        "- visibility: {}\n",
        format_visibility(explain.item.visibility)
    ));
    if let Some(workspace) = explain.item.workspace.as_deref() {
        markdown.push_str(&format!("- workspace: {}\n", workspace));
    }
    if !explain.policy_hooks.is_empty() {
        markdown.push_str("\n## Policy Hooks\n\n");
        for hook in &explain.policy_hooks {
            markdown.push_str(&format!("- {}\n", hook));
        }
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
    if !explain.sources.is_empty() {
        markdown.push_str("\n## Source Lanes\n\n");
        for source in explain.sources.iter().take(5) {
            markdown.push_str(&format!(
                "- {} / {} | workspace {} | visibility {} | trust {:.2} | avg confidence {:.2} | items {}\n",
                source.source_agent.as_deref().unwrap_or("none"),
                source.source_system.as_deref().unwrap_or("none"),
                source.workspace.as_deref().unwrap_or("none"),
                format_visibility(source.visibility),
                source.trust_score,
                source.avg_confidence,
                source.item_count
            ));
        }
    }
    if !explain.branch_siblings.is_empty() {
        markdown.push_str("\n## Sibling Branches\n\n");
        for sibling in explain.branch_siblings.iter().take(5) {
            markdown.push_str(&format!(
                "- {} | {} | confidence {:.2} | status {:?} | {}\n",
                sibling.belief_branch.as_deref().unwrap_or("none"),
                sibling.id,
                sibling.confidence,
                sibling.status,
                if sibling.preferred {
                    "preferred"
                } else {
                    "candidate"
                }
            ));
        }
    }
    if !explain.rehydration.is_empty() {
        markdown.push_str("\n## Rehydration Lane\n\n");
        for artifact in explain.rehydration.iter().take(8) {
            markdown.push_str(&format!(
                "- **{}** {}: {}\n",
                artifact.kind, artifact.label, artifact.summary
            ));
            if let Some(reason) = artifact.reason.as_deref() {
                markdown.push_str(&format!("  - reason: {}\n", reason));
            }
            if artifact.source_path.is_some()
                || artifact.source_agent.is_some()
                || artifact.source_system.is_some()
            {
                markdown.push_str("  - source: ");
                markdown.push_str(artifact.source_agent.as_deref().unwrap_or("none"));
                markdown.push_str(" / ");
                markdown.push_str(artifact.source_system.as_deref().unwrap_or("none"));
                if let Some(path) = artifact.source_path.as_deref() {
                    markdown.push_str(" / ");
                    markdown.push_str(path);
                }
                markdown.push('\n');
            }
            if let Some(path) = artifact.source_path.as_deref()
                && let Some(link) = source_wikilink_for_path(vault, Path::new(path))
            {
                markdown.push_str(&format!("  - wiki: {}\n", link));
            }
        }
    }
    (title, markdown)
}

pub fn build_compiled_note_markdown(
    vault: &Path,
    query: &str,
    response: &SearchMemoryResponse,
    semantic: Option<&memd_rag::RagRetrieveResponse>,
) -> (String, String) {
    let title = query.trim().to_string();
    let mut markdown = String::new();
    markdown.push_str("---\n");
    markdown.push_str(&format!("title: {}\n", title));
    markdown.push_str("source_system: memd\n");
    markdown.push_str("source_agent: memd\n");
    markdown.push_str(&format!("route: {:?}\n", response.route).to_lowercase());
    markdown.push_str(&format!("intent: {:?}\n", response.intent).to_lowercase());
    markdown.push_str(&format!("items: {}\n", response.items.len()));
    markdown.push_str("---\n\n");
    markdown.push_str(&format!("# {}\n\n", title));
    markdown.push_str("## Query\n\n");
    markdown.push_str(query);
    markdown.push_str("\n\n## Matching Memory\n\n");
    for item in response.items.iter().take(16) {
        markdown.push_str(&format!(
            "### {} `{}`\n\n",
            item.tags
                .first()
                .cloned()
                .unwrap_or_else(|| format!("{:?}", item.kind).to_lowercase()),
            item.id
        ));
        markdown.push_str(
            &format!(
                "- kind: {:?}\n- scope: {:?}\n- status: {:?}\n- confidence: {:.2}\n",
                item.kind, item.scope, item.status, item.confidence
            )
            .to_lowercase(),
        );
        if let Some(project) = item.project.as_deref() {
            markdown.push_str(&format!("- project: {}\n", project));
        }
        if let Some(namespace) = item.namespace.as_deref() {
            markdown.push_str(&format!("- namespace: {}\n", namespace));
        }
        if let Some(workspace) = item.workspace.as_deref() {
            markdown.push_str(&format!("- workspace: {}\n", workspace));
        }
        markdown.push_str(&format!(
            "- visibility: {}\n",
            format_visibility(item.visibility)
        ));
        if let Some(branch) = item.belief_branch.as_deref() {
            markdown.push_str(&format!("- belief_branch: {}\n", branch));
        }
        if let Some(source_path) = item.source_path.as_deref() {
            markdown.push_str(&format!("- source_path: {}\n", source_path));
            if let Some(link) = source_wikilink_for_path(vault, Path::new(source_path)) {
                markdown.push_str(&format!("- source_note: {}\n", link));
            }
        }
        markdown.push('\n');
        markdown.push_str(&item.content);
        markdown.push_str("\n\n");
    }
    if let Some(semantic) = semantic.filter(|semantic| !semantic.items.is_empty()) {
        markdown.push_str("## Semantic Recall\n\n");
        for item in semantic.items.iter().take(8) {
            markdown.push_str(&format!(
                "- {}{}\n",
                compact_markdown_text(&item.content, 220),
                item.source
                    .as_deref()
                    .map(|source| format!(" | source {}", compact_markdown_text(source, 64)))
                    .unwrap_or_default()
            ));
            markdown.push_str(&format!("  - score: {:.2}\n", item.score));
        }
        markdown.push('\n');
    }
    (title, markdown)
}

pub fn build_compiled_memory_markdown(
    vault: &Path,
    explain: &ExplainMemoryResponse,
) -> (String, String) {
    let title = format!(
        "{} {}",
        explain
            .item
            .tags
            .first()
            .cloned()
            .unwrap_or_else(|| format!("{:?}", explain.item.kind).to_lowercase()),
        short_uuid(explain.item.id)
    );
    let (_, body) = build_writeback_markdown(vault, explain, explain.entity.as_ref());
    let mut markdown = String::new();
    markdown.push_str("---\n");
    markdown.push_str(&format!("title: {}\n", title));
    markdown.push_str("source_system: memd\n");
    markdown.push_str("source_agent: memd\n");
    markdown.push_str("compiled_from: explain\n");
    markdown.push_str(&format!("memory_id: {}\n", explain.item.id));
    markdown.push_str("---\n\n");
    markdown.push_str("# Compiled Memory Page\n\n");
    markdown.push_str(&format!(
        "- memory: `{}`\n- branch: {}\n- visibility: {}\n- workspace: {}\n- confidence: {:.2}\n- rehydration: {}\n\n",
        explain.item.id,
        explain.item.belief_branch.as_deref().unwrap_or("none"),
        format_visibility(explain.item.visibility),
        explain.item.workspace.as_deref().unwrap_or("none"),
        explain.item.confidence,
        explain.rehydration.len()
    ));
    markdown.push_str(&body);
    (title, markdown)
}

pub fn parse_compiled_artifact_metadata(markdown: &str) -> (Option<String>, Option<usize>) {
    let mut title = None;
    let mut item_count = None;
    let mut in_frontmatter = false;

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            if in_frontmatter {
                break;
            }
            in_frontmatter = true;
            continue;
        }
        if !in_frontmatter {
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("title:") {
            let value = rest.trim().trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                title = Some(value.to_string());
            }
        } else if let Some(rest) = trimmed.strip_prefix("items:") {
            item_count = rest.trim().parse::<usize>().ok();
        }
    }

    (title, item_count)
}

pub fn read_compiled_artifact_metadata(
    path: &Path,
) -> anyhow::Result<(Option<String>, Option<usize>)> {
    let markdown = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(parse_compiled_artifact_metadata(&markdown))
}

pub fn build_compiled_index_markdown(
    existing: Option<&str>,
    entry_kind: &str,
    title: &str,
    note_path: &Path,
    item_count: usize,
) -> String {
    let note_title = note_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(title);
    let entry = format!(
        "- [[{}]] | {}: {} | items: {}",
        note_title, entry_kind, title, item_count
    );
    let mut entries = existing
        .map(|content| {
            content
                .lines()
                .filter(|line| line.starts_with("- [["))
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    entries.retain(|line| !line.contains(&format!("[[{note_title}]]")));
    entries.push(entry);
    entries.sort();

    let mut markdown = String::new();
    markdown.push_str("---\n");
    markdown.push_str("title: Compiled Wiki Index\n");
    markdown.push_str("source_system: memd\n");
    markdown.push_str("source_agent: memd\n");
    markdown.push_str("---\n\n");
    markdown.push_str("# Compiled Wiki Index\n\n");
    markdown.push_str("Generated pages built from `memd` search and explain flows.\n\n");
    for entry in entries {
        markdown.push_str(&entry);
        markdown.push('\n');
    }
    markdown
}

pub fn build_handoff_markdown(
    _vault: &Path,
    snapshot: &crate::ResumeSnapshot,
    sources: &SourceMemoryResponse,
) -> (String, String) {
    let title = format!(
        "Handoff {} {}",
        snapshot.workspace.as_deref().unwrap_or("shared"),
        Utc::now().format("%Y-%m-%d %H:%M")
    );
    let mut markdown = String::new();
    markdown.push_str("---\n");
    markdown.push_str(&format!("title: {}\n", title));
    markdown.push_str("source_system: memd\n");
    markdown.push_str("source_agent: memd\n");
    if let Some(project) = snapshot.project.as_deref() {
        markdown.push_str(&format!("project: {}\n", project));
    }
    if let Some(namespace) = snapshot.namespace.as_deref() {
        markdown.push_str(&format!("namespace: {}\n", namespace));
    }
    if let Some(workspace) = snapshot.workspace.as_deref() {
        markdown.push_str(&format!("workspace: {}\n", workspace));
    }
    if let Some(visibility) = snapshot.visibility.as_deref() {
        markdown.push_str(&format!("visibility: {}\n", visibility));
    }
    markdown.push_str(&format!("route: {}\n", snapshot.route));
    markdown.push_str(&format!("intent: {}\n", snapshot.intent));
    markdown.push_str(&format!(
        "working_items: {}\n",
        snapshot.working.records.len()
    ));
    markdown.push_str(&format!(
        "rehydration_items: {}\n",
        snapshot.working.rehydration_queue.len()
    ));
    markdown.push_str(&format!("inbox_items: {}\n", snapshot.inbox.items.len()));
    markdown.push_str(&format!(
        "workspace_lanes: {}\n",
        snapshot.workspaces.workspaces.len()
    ));
    markdown.push_str(&format!(
        "semantic_hits: {}\n",
        snapshot
            .semantic
            .as_ref()
            .map(|semantic| semantic.items.len())
            .unwrap_or(0)
    ));
    markdown.push_str(&format!("source_lanes: {}\n", sources.sources.len()));
    markdown.push_str("---\n\n");
    markdown.push_str(&format!("# {}\n\n", title));
    markdown.push_str("## Resume Frame\n\n");
    markdown.push_str(&format!(
        "- project: {}\n- namespace: {}\n- agent: {}\n- workspace: {}\n- visibility: {}\n- route: {}\n- intent: {}\n",
        snapshot.project.as_deref().unwrap_or("none"),
        snapshot.namespace.as_deref().unwrap_or("none"),
        snapshot.agent.as_deref().unwrap_or("none"),
        snapshot.workspace.as_deref().unwrap_or("none"),
        snapshot.visibility.as_deref().unwrap_or("all"),
        snapshot.route,
        snapshot.intent
    ));

    markdown.push_str("\n## W\n\n");
    if snapshot.working.records.is_empty() {
        markdown.push_str("- none\n");
    } else {
        let records = snapshot
            .working
            .records
            .iter()
            .take(2)
            .map(|record| record.record.clone())
            .collect::<Vec<_>>();
        markdown.push_str(&format!("- w={}", records.join(" | ")));
        if snapshot.working.records.len() > 2 {
            markdown.push_str(&format!(" (+{} more)", snapshot.working.records.len() - 2));
        }
        markdown.push('\n');
    }

    let mut ri_parts = Vec::new();
    if !snapshot.working.rehydration_queue.is_empty() {
        for artifact in snapshot.working.rehydration_queue.iter().take(8) {
            ri_parts.push(format!("r={}:{}", artifact.label, artifact.summary));
            if let Some(path) = artifact.source_path.as_deref() {
                ri_parts.push(format!("src={}", path));
            }
            if let Some(reason) = artifact.reason.as_deref() {
                ri_parts.push(format!("r={}", reason));
            }
        }
    }
    if !snapshot.inbox.items.is_empty() {
        for item in snapshot.inbox.items.iter().take(8) {
            ri_parts.push(format!(
                "i={:?}/{:?}:cf{:.2}",
                item.item.kind, item.item.status, item.item.confidence
            ));
            if !item.reasons.is_empty() {
                ri_parts.push(format!("r={}", item.reasons.join(", ")));
            }
        }
    }
    if !ri_parts.is_empty() {
        markdown.push_str("\n## RI\n\n");
        markdown.push_str(&format!("- {}\n", ri_parts.join(" | ")));
    }

    if let Some(first) = snapshot.workspaces.workspaces.first() {
        markdown.push_str("\n## L\n\n");
        let extras = snapshot.workspaces.workspaces.len() - 1;
        markdown.push_str(&format!(
            "- l={}/{}/{} | v={} | it={} | tr={:.2} | cf={:.2}{} \n",
            first.project.as_deref().unwrap_or("none"),
            first.namespace.as_deref().unwrap_or("none"),
            first.workspace.as_deref().unwrap_or("none"),
            format_visibility(first.visibility),
            first.item_count,
            first.trust_score,
            first.avg_confidence,
            if extras > 0 {
                format!(" (+{} more)", extras)
            } else {
                "".to_string()
            }
        ));
    }

    if let Some(semantic) = snapshot
        .semantic
        .as_ref()
        .filter(|semantic| !semantic.items.is_empty())
    {
        markdown.push_str("\n## S\n\n");
        for item in semantic.items.iter().take(6) {
            markdown.push_str(&format!(
                "- {}{}\n",
                compact_markdown_text(&item.content, 220),
                item.source
                    .as_deref()
                    .map(|source| format!(" | source {}", compact_markdown_text(source, 64)))
                    .unwrap_or_default()
            ));
            markdown.push_str(&format!("  - score: {:.2}\n", item.score));
        }
    }

    if !sources.sources.is_empty() {
        markdown.push_str("\n## C\n\n");
        for source in sources.sources.iter().take(8) {
            markdown.push_str(&format!(
                "- {} / {} | workspace {} | visibility {} | items {} | trust {:.2} | confidence {:.2}\n",
                source.source_agent.as_deref().unwrap_or("none"),
                source.source_system.as_deref().unwrap_or("none"),
                source.workspace.as_deref().unwrap_or("none"),
                format_visibility(source.visibility),
                source.item_count,
                source.trust_score,
                source.avg_confidence
            ));
        }
    }

    (title, markdown)
}
pub(crate) fn source_wikilink_for_path(vault: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(vault).ok()?;
    let title = relative
        .file_stem()
        .and_then(|value| value.to_str())
        .map(|value| value.replace(['[', ']'], ""))?;
    Some(format!("[[{}]]", title))
}

fn format_visibility(value: MemoryVisibility) -> &'static str {
    match value {
        MemoryVisibility::Private => "private",
        MemoryVisibility::Workspace => "workspace",
        MemoryVisibility::Public => "public",
    }
}
