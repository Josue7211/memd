use serde::Serialize;
use serde_json::Value;

use memd_schema::{
    AgentProfileResponse, AssociativeRecallResponse, EntitySearchResponse, ExplainMemoryResponse,
    MemoryPolicyResponse, RepairMemoryResponse, RetrievalIntent, RetrievalRoute,
    SourceMemoryResponse, VisibleMemorySnapshotResponse, VisibleMemoryStatus,
    WorkingMemoryResponse, WorkspaceMemoryResponse,
};

use crate::{
    SkillCatalog, SkillCatalogEntry,
    command_catalog::{CommandCatalog, CommandCatalogEntry},
    harness::{
        agent_zero::AgentZeroHarnessPack,
        claude_code::ClaudeCodeHarnessPack,
        codex::CodexHarnessPack,
        hermes::HermesHarnessPack,
        index::{HarnessPackIndex, HarnessPackIndexEntry},
        openclaw::OpenClawHarnessPack,
        opencode::OpenCodeHarnessPack,
        preset::HarnessPreset,
        shared::render_harness_pack_markdown,
    },
    obsidian::ObsidianVaultScan,
};

pub(crate) fn render_obsidian_scan_summary(scan: &ObsidianVaultScan, follow: bool) -> String {
    let mut summary = format!(
        "obsidian_scan vault={} project={} namespace={} workspace={} visibility={} notes={} sensitive={} skipped={} unchanged={} backlinks={} attachments={} attachment_sensitive={} attachment_unchanged={} cache_hits={} attachment_cache_hits={} cache_pruned={} attachment_cache_pruned={}",
        scan.vault.display(),
        scan.project.as_deref().unwrap_or("none"),
        scan.namespace.as_deref().unwrap_or("none"),
        scan.workspace.as_deref().unwrap_or("none"),
        format_visibility(scan.visibility),
        scan.note_count,
        scan.sensitive_count,
        scan.skipped_count,
        scan.unchanged_count,
        scan.backlink_count,
        scan.attachment_count,
        scan.attachment_sensitive_count,
        scan.attachment_unchanged_count,
        scan.cache_hits,
        scan.attachment_cache_hits,
        scan.cache_pruned,
        scan.attachment_cache_pruned
    );

    if follow {
        let trail = scan
            .notes
            .iter()
            .take(3)
            .map(|note| note.title.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        if !trail.is_empty() {
            summary.push_str(&format!(" trail={trail}"));
        }
    }

    summary
}

pub(crate) fn render_obsidian_import_summary(
    output: &crate::ObsidianImportOutput,
    follow: bool,
) -> String {
    let attachment_submitted = output
        .attachments
        .as_ref()
        .map(|attachments| attachments.submitted)
        .unwrap_or(0);
    let mut summary = format!(
        "obsidian_import vault={} project={} namespace={} workspace={} visibility={} notes={} sensitive={} unchanged={} backlinks={} attachments={} attachment_sensitive={} attachment_unchanged={} duplicate_suppressed={} submitted={} attachment_submitted={} duplicates={} attachment_duplicates={} note_failures={} attachment_failures={} links={} attachment_links={} mirrored={} mirrored_attachments={} dry_run={}",
        output.preview.scan.vault.display(),
        output.preview.scan.project.as_deref().unwrap_or("none"),
        output.preview.scan.namespace.as_deref().unwrap_or("none"),
        output.preview.scan.workspace.as_deref().unwrap_or("none"),
        format_visibility(output.preview.scan.visibility),
        output.preview.scan.note_count,
        output.preview.scan.sensitive_count,
        output.preview.scan.unchanged_count,
        output.preview.scan.backlink_count,
        output.preview.scan.attachment_count,
        output.preview.scan.attachment_sensitive_count,
        output.attachment_unchanged_count,
        output.preview.duplicate_count,
        output.submitted,
        attachment_submitted,
        output.duplicates,
        output.attachment_duplicates,
        output.note_failures,
        output.attachment_failures,
        output.links_created,
        output.attachment_links_created,
        output.mirrored_notes,
        output.mirrored_attachments,
        output.dry_run
    );
    if follow {
        let trail = output
            .preview
            .scan
            .notes
            .iter()
            .take(3)
            .map(|note| note.title.as_str())
            .collect::<Vec<_>>()
            .join(" | ");
        if !trail.is_empty() {
            summary.push_str(&format!(" trail={trail}"));
        }
    }
    if let Some(attachments) = output.attachments.as_ref() {
        summary.push_str(&format!(
            " attachments_submitted={} attachments_dry_run={}",
            attachments.submitted, attachments.dry_run
        ));
    }
    summary
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn render_codex_harness_pack_markdown(pack: &CodexHarnessPack) -> String {
    render_harness_pack_markdown(pack)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn render_claude_code_harness_pack_markdown(pack: &ClaudeCodeHarnessPack) -> String {
    render_harness_pack_markdown(pack)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn render_agent_zero_harness_pack_markdown(pack: &AgentZeroHarnessPack) -> String {
    render_harness_pack_markdown(pack)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn render_openclaw_harness_pack_markdown(pack: &OpenClawHarnessPack) -> String {
    render_harness_pack_markdown(pack)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn render_hermes_harness_pack_markdown(pack: &HermesHarnessPack) -> String {
    render_harness_pack_markdown(pack)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn render_opencode_harness_pack_markdown(pack: &OpenCodeHarnessPack) -> String {
    render_harness_pack_markdown(pack)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn render_harness_preset_markdown(preset: &HarnessPreset) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!("# {} Harness Pack\n\n", preset.display_name));
    markdown.push_str(&format!("- pack id: `{}`\n", preset.pack_id));
    markdown.push_str(&format!("- entrypoint: `{}`\n", preset.entrypoint));
    markdown.push_str(&format!("- cache policy: {}\n", preset.cache_policy));
    markdown.push_str(&format!("- tone: {}\n\n", preset.copy_tone));

    markdown.push_str("## Surface Set\n");
    for surface in preset.surface_set {
        markdown.push_str(&format!("- `{}`\n", surface));
    }
    markdown.push('\n');

    markdown.push_str("## Default Verbs\n");
    for verb in preset.default_verbs {
        markdown.push_str(&format!("- `{}`\n", verb));
    }
    markdown.push('\n');

    markdown.push_str("## Shared Core\n");
    markdown.push_str(
        "memd owns the same memory control plane, compiled pages, and turn-scoped cache.\n",
    );
    markdown
}

pub(crate) fn render_harness_pack_index_summary(
    bundle_root: &std::path::Path,
    index: &HarnessPackIndex,
    query: Option<&str>,
) -> String {
    let mut summary = format!(
        "pack index root={} project={} namespace={} packs={}",
        bundle_root.display(),
        index.project,
        index.namespace,
        index.pack_count
    );
    if let Some(query) = query {
        summary.push_str(&format!(" query={}", compact_inline(query, 48)));
    }
    if !index.packs.is_empty() {
        summary.push_str(&format!(
            " names={}",
            index
                .packs
                .iter()
                .map(|pack| pack.name.as_str())
                .collect::<Vec<_>>()
                .join("|")
        ));
    }
    if !index.preset_names.is_empty() {
        summary.push_str(&format!(" presets={}", index.preset_names.join("|")));
    }
    summary
}

pub(crate) fn render_harness_pack_index_markdown(
    bundle_root: &std::path::Path,
    index: &HarnessPackIndex,
) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd harness packs\n\n");
    markdown.push_str(&format!("- Root: `{}`\n", bundle_root.display()));
    markdown.push_str(&format!("- Project: `{}`\n", index.project));
    markdown.push_str(&format!("- Namespace: `{}`\n", index.namespace));
    markdown.push_str(&format!("- Packs: `{}`\n\n", index.pack_count));
    if !index.preset_names.is_empty() {
        markdown.push_str("## Harness Presets\n");
        for preset in &index.preset_names {
            markdown.push_str(&format!("- `{}`\n", preset));
        }
        markdown.push('\n');
    }

    if index.packs.is_empty() {
        markdown.push_str("No visible harness packs found.\n");
        return markdown;
    }

    for pack in &index.packs {
        render_harness_pack_section(&mut markdown, pack);
    }

    markdown
}

pub(crate) fn render_harness_pack_index_json(index: &HarnessPackIndex) -> HarnessPackIndexJson {
    HarnessPackIndexJson {
        root: index.root.clone(),
        project: index.project.clone(),
        namespace: index.namespace.clone(),
        pack_count: index.pack_count,
        preset_names: index.preset_names.clone(),
        packs: index
            .packs
            .iter()
            .map(HarnessPackIndexEntryJson::from)
            .collect(),
    }
}

pub(crate) fn render_command_catalog_summary(
    catalog: &CommandCatalog,
    query: Option<&str>,
) -> String {
    let mut summary = format!(
        "commands root={} commands={}",
        catalog.root, catalog.command_count
    );
    let native_count = catalog
        .commands
        .iter()
        .filter(|command| command.role == "native-cli")
        .count();
    let external_count = catalog
        .commands
        .iter()
        .filter(|command| command.role == "bridge-surface")
        .count();
    let helper_count = catalog
        .commands
        .iter()
        .filter(|command| command.role == "bundle-helper")
        .count();
    summary.push_str(&format!(
        " native={} external={} helpers={}",
        native_count, external_count, helper_count
    ));
    if let Some(query) = query {
        summary.push_str(&format!(" query={}", compact_inline(query, 48)));
    }
    if !catalog.commands.is_empty() {
        summary.push_str(&format!(
            " names={}",
            catalog
                .commands
                .iter()
                .map(|command| command.name.as_str())
                .collect::<Vec<_>>()
                .join("|")
        ));
    }
    summary
}

pub(crate) fn render_command_catalog_markdown(catalog: &CommandCatalog) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd commands\n\n");
    markdown.push_str(&format!("- Root: `{}`\n", catalog.root));
    markdown.push_str(&format!("- Commands: `{}`\n\n", catalog.command_count));
    if catalog.commands.is_empty() {
        markdown.push_str("No commands found.\n");
        return markdown;
    }
    markdown.push_str("memd owns the native CLI surfaces. External bridge surfaces stay listed so they can be migrated, swapped, or reimplemented on other harnesses without pretending memd owns them.\n\n");
    render_command_catalog_section(
        &mut markdown,
        "Native memd CLI",
        &catalog.commands,
        |command| command.role == "native-cli",
    );
    render_command_catalog_section(
        &mut markdown,
        "Bridge surfaces",
        &catalog.commands,
        |command| command.role == "bridge-surface",
    );
    render_command_catalog_section(
        &mut markdown,
        "Bundle helpers",
        &catalog.commands,
        |command| command.role == "bundle-helper",
    );
    markdown
}

pub(crate) fn render_command_catalog_json(catalog: &CommandCatalog) -> CommandCatalogJson {
    CommandCatalogJson {
        root: catalog.root.clone(),
        command_count: catalog.command_count,
        commands: catalog
            .commands
            .iter()
            .map(CommandCatalogEntryJson::from)
            .collect(),
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CommandCatalogJson {
    pub(crate) root: String,
    pub(crate) command_count: usize,
    pub(crate) commands: Vec<CommandCatalogEntryJson>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CommandCatalogEntryJson {
    pub(crate) name: String,
    pub(crate) surface: String,
    pub(crate) kind: String,
    pub(crate) ownership: String,
    pub(crate) role: String,
    pub(crate) compatibility_flags: Vec<String>,
    pub(crate) command: String,
    pub(crate) purpose: String,
    pub(crate) path: Option<String>,
}

impl From<&CommandCatalogEntry> for CommandCatalogEntryJson {
    fn from(value: &CommandCatalogEntry) -> Self {
        Self {
            name: value.name.clone(),
            surface: value.surface.clone(),
            kind: value.kind.clone(),
            ownership: value.ownership.clone(),
            role: value.role.clone(),
            compatibility_flags: value.compatibility_flags.clone(),
            command: value.command.clone(),
            purpose: value.purpose.clone(),
            path: value.path.clone(),
        }
    }
}

fn render_command_catalog_entry(markdown: &mut String, command: &CommandCatalogEntry) {
    markdown.push_str(&format!("## {}\n\n", command.name));
    markdown.push_str(&format!("- surface: `{}`\n", command.surface));
    markdown.push_str(&format!("- kind: `{}`\n", command.kind));
    markdown.push_str(&format!("- ownership: `{}`\n", command.ownership));
    markdown.push_str(&format!("- role: `{}`\n", command.role));
    if !command.compatibility_flags.is_empty() {
        markdown.push_str(&format!(
            "- compatibility: `{}`\n",
            command.compatibility_flags.join("`, `")
        ));
    }
    markdown.push_str(&format!("- command: `{}`\n", command.command));
    markdown.push_str(&format!("- purpose: {}\n", command.purpose));
    if let Some(path) = command.path.as_deref() {
        markdown.push_str(&format!("- path: `{}`\n", path));
    }
    markdown.push('\n');
}

fn render_command_catalog_section<F>(
    markdown: &mut String,
    title: &str,
    commands: &[CommandCatalogEntry],
    matches: F,
) where
    F: Fn(&CommandCatalogEntry) -> bool,
{
    let filtered = commands
        .iter()
        .filter(|command| matches(command))
        .collect::<Vec<_>>();
    if filtered.is_empty() {
        return;
    }
    markdown.push_str(&format!("## {}\n\n", title));
    for command in filtered {
        render_command_catalog_entry(markdown, command);
    }
}

fn render_harness_pack_section(markdown: &mut String, pack: &HarnessPackIndexEntry) {
    markdown.push_str(&format!("## {}\n\n", pack.name));
    markdown.push_str(&format!("- role: `{}`\n", pack.role));
    markdown.push_str(&format!("- project: `{}`\n", pack.project));
    markdown.push_str(&format!("- namespace: `{}`\n", pack.namespace));
    markdown.push_str(&format!("- bundle root: `{}`\n\n", pack.bundle_root));

    markdown.push_str("### Files\n");
    for file in &pack.files {
        markdown.push_str(&format!("- `{}`\n", file));
    }
    markdown.push('\n');

    markdown.push_str("### Commands\n");
    for command in &pack.commands {
        markdown.push_str(&format!("- `{}`\n", command));
    }
    markdown.push('\n');

    markdown.push_str("### Behaviors\n");
    for behavior in &pack.behaviors {
        markdown.push_str(&format!("- {}\n", behavior));
    }
    markdown.push('\n');
}

#[derive(Debug, Serialize)]
pub(crate) struct HarnessPackIndexJson {
    pub(crate) root: String,
    pub(crate) project: String,
    pub(crate) namespace: String,
    pub(crate) pack_count: usize,
    pub(crate) preset_names: Vec<String>,
    pub(crate) packs: Vec<HarnessPackIndexEntryJson>,
}

#[derive(Debug, Serialize)]
pub(crate) struct HarnessPackIndexEntryJson {
    pub(crate) name: String,
    pub(crate) role: String,
    pub(crate) project: String,
    pub(crate) namespace: String,
    pub(crate) bundle_root: String,
    pub(crate) files: Vec<String>,
    pub(crate) commands: Vec<String>,
    pub(crate) behaviors: Vec<String>,
}

impl From<&HarnessPackIndexEntry> for HarnessPackIndexEntryJson {
    fn from(value: &HarnessPackIndexEntry) -> Self {
        Self {
            name: value.name.clone(),
            role: value.role.clone(),
            project: value.project.clone(),
            namespace: value.namespace.clone(),
            bundle_root: value.bundle_root.clone(),
            files: value.files.clone(),
            commands: value.commands.clone(),
            behaviors: value.behaviors.clone(),
        }
    }
}

pub(crate) fn render_bundle_status_summary(status: &Value) -> String {
    let bundle = status
        .get("bundle")
        .and_then(Value::as_str)
        .unwrap_or("none");
    let setup_ready = status
        .get("setup_ready")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let server = status
        .get("server")
        .and_then(|value| value.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let rag = status
        .get("rag")
        .and_then(|value| value.get("healthy"))
        .and_then(Value::as_bool)
        .map(|healthy| if healthy { "ready" } else { "degraded" })
        .unwrap_or("off");

    let mut output = format!(
        "status bundle={} ready={} setup={} server={} rag={}",
        bundle, setup_ready, setup_ready, server, rag
    );

    let resume = status
        .get("resume_preview")
        .and_then(|value| if value.is_null() { None } else { Some(value) });
    let defaults = status
        .get("defaults")
        .and_then(|value| if value.is_null() { None } else { Some(value) });
    let heartbeat = status
        .get("heartbeat")
        .and_then(|value| if value.is_null() { None } else { Some(value) });
    let project = resume
        .and_then(|value| value.get("project").and_then(Value::as_str))
        .or_else(|| defaults.and_then(|value| value.get("project").and_then(Value::as_str)))
        .or_else(|| heartbeat.and_then(|value| value.get("project").and_then(Value::as_str)))
        .unwrap_or("none");
    let namespace = resume
        .and_then(|value| value.get("namespace").and_then(Value::as_str))
        .or_else(|| defaults.and_then(|value| value.get("namespace").and_then(Value::as_str)))
        .or_else(|| heartbeat.and_then(|value| value.get("namespace").and_then(Value::as_str)))
        .unwrap_or("none");
    let session = resume
        .and_then(|value| value.get("session").and_then(Value::as_str))
        .or_else(|| defaults.and_then(|value| value.get("session").and_then(Value::as_str)))
        .or_else(|| heartbeat.and_then(|value| value.get("session").and_then(Value::as_str)))
        .unwrap_or("none");
    let tab_id = resume
        .and_then(|value| value.get("tab_id").and_then(Value::as_str))
        .or_else(|| defaults.and_then(|value| value.get("tab_id").and_then(Value::as_str)))
        .or_else(|| heartbeat.and_then(|value| value.get("tab_id").and_then(Value::as_str)))
        .unwrap_or("none");
    let agent = resume
        .and_then(|value| value.get("agent").and_then(Value::as_str))
        .or_else(|| defaults.and_then(|value| value.get("agent").and_then(Value::as_str)))
        .or_else(|| {
            heartbeat.and_then(|value| {
                value
                    .get("effective_agent")
                    .and_then(Value::as_str)
                    .or_else(|| value.get("agent").and_then(Value::as_str))
            })
        })
        .unwrap_or("none");
    output.push_str(&format!(
        " project={} namespace={} session={} tab={} agent={} voice=caveman-ultra",
        project, namespace, session, tab_id, agent
    ));
    let session_overlay = status
        .get("session_overlay")
        .and_then(|value| if value.is_null() { None } else { Some(value) });
    if let Some(overlay) = session_overlay {
        let rebased_from = overlay.get("rebased_from").and_then(Value::as_str);
        if let Some(rebased_from) = rebased_from {
            let bundle_session = overlay
                .get("bundle_session")
                .and_then(Value::as_str)
                .unwrap_or("none");
            let live_session = overlay
                .get("live_session")
                .and_then(Value::as_str)
                .unwrap_or("none");
            output.push_str(&format!(
                " bundle_session={} live_session={} rebased_from={}",
                bundle_session, live_session, rebased_from
            ));
        }
    }

    if let Some(resume) = resume {
        let pressure = resume
            .get("context_pressure")
            .and_then(Value::as_str)
            .unwrap_or("none");
        let tokens = resume
            .get("estimated_prompt_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let refresh = resume
            .get("refresh_recommended")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let mut drivers = Vec::new();
        if tokens >= 1_800 {
            drivers.push("tokens");
        } else if tokens >= 1_000 {
            drivers.push("tokens");
        }
        if resume
            .get("redundant_context_items")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            > 0
        {
            drivers.push("duplicates");
        }
        if resume
            .get("inbox_items")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            >= 3
        {
            drivers.push("inbox");
        }
        if resume
            .get("rehydration_queue")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            >= 3
        {
            drivers.push("rehydration");
        }
        if resume
            .get("semantic_hits")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            >= 4
        {
            drivers.push("semantic");
        }
        if refresh {
            drivers.push("refresh");
        }
        drivers.sort();
        drivers.dedup();

        let action = if refresh || pressure == "high" {
            if drivers.iter().any(|value| *value == "inbox") {
                "drain inbox before the next prompt"
            } else if drivers.iter().any(|value| *value == "rehydration") {
                "resolve rehydration backlog before the next prompt"
            } else if drivers.iter().any(|value| *value == "duplicates") {
                "collapse repeated context before the next prompt"
            } else {
                "trim context before the next prompt"
            }
        } else if pressure == "medium" {
            if drivers.iter().any(|value| *value == "inbox") {
                "handle inbox items before pulling more context"
            } else if drivers.iter().any(|value| *value == "rehydration") {
                "resolve rehydration before the prompt grows"
            } else {
                "watch prompt growth"
            }
        } else {
            "none"
        };

        output.push_str(&format!(" prompt_pressure={} tok={}", pressure, tokens));
        if !drivers.is_empty() {
            output.push_str(&format!(" drivers={}", drivers.join(",")));
        }
        if action != "none" {
            output.push_str(&format!(" action=\"{}\"", action));
        }
        if let Some(focus) = resume.get("focus").and_then(Value::as_str) {
            output.push_str(&format!(" focus=\"{}\"", compact_inline(focus, 72)));
        }
        if let Some(next) = resume.get("next_recovery").and_then(Value::as_str) {
            output.push_str(&format!(" next=\"{}\"", compact_inline(next, 72)));
        }
        if refresh || pressure == "high" {
            output.push_str(" warning=\"prompt pressure high\"");
        }
    }

    if let Some(truth) = status
        .get("truth_summary")
        .and_then(|value| if value.is_null() { None } else { Some(value) })
    {
        let truth_state = truth
            .get("truth")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let freshness = truth
            .get("freshness")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let tier = truth
            .get("retrieval_tier")
            .and_then(Value::as_str)
            .unwrap_or("raw_fallback");
        let confidence = truth
            .get("confidence")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        let sources = truth
            .get("source_count")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let contested = truth
            .get("contested_sources")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        output.push_str(&format!(
            " truth={} freshness={} retrieval={} conf={:.2} sources={} contested={}",
            truth_state, freshness, tier, confidence, sources, contested
        ));
        if let Some(action_hint) = truth.get("action_hint").and_then(Value::as_str)
            && action_hint != "none"
        {
            output.push_str(&format!(
                " truth_action=\"{}\"",
                compact_inline(action_hint, 64)
            ));
        }
        if let Some(record) = truth
            .get("records")
            .and_then(Value::as_array)
            .and_then(|records| records.first())
        {
            let lane = record.get("lane").and_then(Value::as_str).unwrap_or("none");
            let preview = record
                .get("preview")
                .and_then(Value::as_str)
                .unwrap_or("none");
            output.push_str(&format!(
                " truth_head={} truth_preview=\"{}\"",
                lane,
                compact_inline(preview, 72)
            ));
        }
    }

    if let Some(evolution) = status
        .get("evolution")
        .and_then(|value| if value.is_null() { None } else { Some(value) })
    {
        output.push_str(&format!(
            " evolution={} scope={}/{} authority={} merge={} durability={}",
            evolution
                .get("proposal_state")
                .and_then(Value::as_str)
                .unwrap_or("none"),
            evolution
                .get("scope_class")
                .and_then(Value::as_str)
                .unwrap_or("none"),
            evolution
                .get("scope_gate")
                .and_then(Value::as_str)
                .unwrap_or("none"),
            evolution
                .get("authority_tier")
                .and_then(Value::as_str)
                .unwrap_or("none"),
            evolution
                .get("merge_status")
                .and_then(Value::as_str)
                .unwrap_or("none"),
            evolution
                .get("durability_status")
                .and_then(Value::as_str)
                .unwrap_or("none"),
        ));
        if let Some(branch) = evolution.get("branch").and_then(Value::as_str)
            && branch != "none"
        {
            output.push_str(&format!(" evo_branch={}", compact_inline(branch, 64)));
        }
    }

    if let Some(capability) = status
        .get("capability_surface")
        .and_then(|value| if value.is_null() { None } else { Some(value) })
    {
        output.push_str(&format!(
            " capabilities={} universal={} bridgeable={} harness_native={}",
            capability
                .get("discovered")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            capability
                .get("universal")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            capability
                .get("bridgeable")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            capability
                .get("harness_native")
                .and_then(Value::as_u64)
                .unwrap_or(0),
        ));
    }

    if let Some(cowork) = status
        .get("cowork_surface")
        .and_then(|value| if value.is_null() { None } else { Some(value) })
    {
        output.push_str(&format!(
            " cowork_tasks={} open={} help={} review={} exclusive={} shared={} inbox_messages={} owned={}",
            cowork.get("tasks").and_then(Value::as_u64).unwrap_or(0),
            cowork.get("open_tasks").and_then(Value::as_u64).unwrap_or(0),
            cowork.get("help_tasks").and_then(Value::as_u64).unwrap_or(0),
            cowork.get("review_tasks").and_then(Value::as_u64).unwrap_or(0),
            cowork
                .get("exclusive_tasks")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            cowork.get("shared_tasks").and_then(Value::as_u64).unwrap_or(0),
            cowork
                .get("inbox_messages")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            cowork.get("owned_tasks").and_then(Value::as_u64).unwrap_or(0),
        ));
    }

    if let Some(maintenance) = status
        .get("maintenance_surface")
        .and_then(|value| if value.is_null() { None } else { Some(value) })
    {
        output.push_str(&format!(
            " maintain_mode={} auto={} maintain_receipt={} compacted={} refreshed={} repaired={} findings={} maintain_total={} maintain_delta={} maintain_trend={}",
            maintenance
                .get("mode")
                .and_then(Value::as_str)
                .unwrap_or("none"),
            if maintenance
                .get("auto_mode")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                "yes"
            } else {
                "no"
            },
            maintenance
                .get("receipt")
                .and_then(Value::as_str)
                .unwrap_or("none"),
            maintenance
                .get("compacted")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            maintenance
                .get("refreshed")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            maintenance
                .get("repaired")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            maintenance
                .get("findings")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            maintenance
                .get("total_actions")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            maintenance
                .get("delta_total_actions")
                .and_then(Value::as_i64)
                .unwrap_or(0),
            maintenance
                .get("trend")
                .and_then(Value::as_str)
                .unwrap_or("none"),
        ));
    }

    if let Some(missing) = status.get("missing").and_then(Value::as_array) {
        let missing = missing
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(",");
        if !missing.is_empty() {
            output.push_str(&format!(" missing={missing}"));
        }
    }

    output
}

pub(crate) fn render_entity_summary(
    response: &memd_schema::EntityMemoryResponse,
    follow: bool,
) -> String {
    let Some(entity) = response.entity.as_ref() else {
        return format!(
            "entity=none route={} intent={}",
            route_label(response.route),
            intent_label(response.intent)
        );
    };

    let state = entity
        .current_state
        .as_deref()
        .map(|value| compact_inline(value, 72))
        .unwrap_or_else(|| "no-state".to_string());
    let last_seen = entity
        .last_seen_at
        .map(|value| value.to_rfc3339())
        .unwrap_or_else(|| "unknown".to_string());

    let mut output = format!(
        "entity={} type={} salience={:.2} rehearsal={} state_v={} last_seen={} state=\"{}\" events={}",
        short_uuid(entity.id),
        entity.entity_type,
        entity.salience_score,
        entity.rehearsal_count,
        entity.state_version,
        last_seen,
        state,
        response.events.len()
    );

    if follow && let Some(event) = response.events.first() {
        output.push_str(&format!(
            " latest={}::{}",
            event.event_type,
            compact_inline(&event.summary, 48)
        ));
    }

    output
}

pub(crate) fn render_entity_search_summary(
    response: &EntitySearchResponse,
    follow: bool,
) -> String {
    let mut output = format!(
        "entity-search query=\"{}\" candidates={} ambiguous={}",
        compact_inline(&response.query, 48),
        response.candidates.len(),
        response.ambiguous
    );

    if let Some(best) = response.best_match.as_ref() {
        output.push_str(&format!(
            " best={} type={} score={:.2} reasons={}",
            short_uuid(best.entity.id),
            best.entity.entity_type,
            best.score,
            compact_inline(&best.reasons.join(","), 64)
        ));
    }

    if follow {
        let trail = response
            .candidates
            .iter()
            .take(3)
            .map(|candidate| {
                format!(
                    "{}:{}:{:.2}",
                    short_uuid(candidate.entity.id),
                    candidate.entity.entity_type,
                    candidate.score
                )
            })
            .collect::<Vec<_>>();
        if !trail.is_empty() {
            output.push_str(&format!(" trail={}", trail.join(" | ")));
        }
    }

    output
}

pub(crate) fn render_recall_summary(response: &AssociativeRecallResponse, follow: bool) -> String {
    let root = response
        .root_entity
        .as_ref()
        .map(|entity| format!("root={} type={}", short_uuid(entity.id), entity.entity_type))
        .unwrap_or_else(|| "root=none".to_string());

    let mut output = format!(
        "recall {} hits={} links={} truncated={}",
        root,
        response.hits.len(),
        response.links.len(),
        response.truncated
    );

    if follow {
        let hit_trail = response
            .hits
            .iter()
            .take(3)
            .map(|hit| {
                format!(
                    "d{}:{}:{:.2}:{}",
                    hit.depth,
                    short_uuid(hit.entity.id),
                    hit.score,
                    compact_inline(
                        hit.entity
                            .current_state
                            .as_deref()
                            .unwrap_or(&hit.entity.entity_type),
                        28
                    )
                )
            })
            .collect::<Vec<_>>();
        if !hit_trail.is_empty() {
            output.push_str(&format!(" trail={}", hit_trail.join(" | ")));
        }

        let link_trail = response
            .links
            .iter()
            .take(3)
            .map(|link| {
                format!(
                    "{}:{}->{}",
                    format!("{:?}", link.relation_kind).to_ascii_lowercase(),
                    short_uuid(link.from_entity_id),
                    short_uuid(link.to_entity_id)
                )
            })
            .collect::<Vec<_>>();
        if !link_trail.is_empty() {
            output.push_str(&format!(" links={}", link_trail.join(" | ")));
        }

        if let Some(best) = response.hits.first() {
            output.push_str(&format!(
                " best_score={:.2} best_reasons={}",
                best.score,
                compact_inline(&best.reasons.join(","), 72)
            ));
        }
    }

    output
}

pub(crate) fn render_timeline_summary(
    response: &memd_schema::TimelineMemoryResponse,
    follow: bool,
) -> String {
    let entity = response
        .entity
        .as_ref()
        .map(|entity| {
            format!(
                "entity={} type={}",
                short_uuid(entity.id),
                entity.entity_type
            )
        })
        .unwrap_or_else(|| "entity=none".to_string());
    let latest = response
        .events
        .first()
        .map(|event| {
            format!(
                "{}:{}",
                event.event_type,
                compact_inline(&event.summary, 56)
            )
        })
        .unwrap_or_else(|| "no-events".to_string());

    let mut output = format!(
        "timeline {} route={} intent={} events={} latest={}",
        entity,
        route_label(response.route),
        intent_label(response.intent),
        response.events.len(),
        latest
    );

    if follow {
        let trail = response
            .events
            .iter()
            .take(3)
            .map(|event| {
                format!(
                    "{}:{}",
                    event.event_type,
                    compact_inline(&event.summary, 40)
                )
            })
            .collect::<Vec<_>>();
        if !trail.is_empty() {
            output.push_str(&format!(" trail={}", trail.join(" | ")));
        }
    }

    output
}

pub(crate) fn render_working_summary(response: &WorkingMemoryResponse, follow: bool) -> String {
    let mut output = format!(
        "working route={} intent={} budget={} used={} remaining={} truncated={} records={} evicted={} rehydrate={} traces={} semantic={}",
        route_label(response.route),
        intent_label(response.intent),
        response.budget_chars,
        response.used_chars,
        response.remaining_chars,
        response.truncated,
        response.records.len(),
        response.evicted.len(),
        response.rehydration_queue.len(),
        response.traces.len(),
        response
            .semantic_consolidation
            .as_ref()
            .map(|value| value.consolidated.to_string())
            .unwrap_or_else(|| "off".to_string())
    );

    if follow {
        let trail = response
            .records
            .iter()
            .take(3)
            .map(|record| compact_inline(&record.record, 48))
            .collect::<Vec<_>>();
        if !trail.is_empty() {
            output.push_str(&format!(" trail={}", trail.join(" | ")));
        }

        let trace_trail = response
            .traces
            .iter()
            .take(3)
            .map(|trace| {
                format!(
                    "{}:{}",
                    trace.event_type,
                    compact_inline(&trace.summary, 40)
                )
            })
            .collect::<Vec<_>>();
        if !trace_trail.is_empty() {
            output.push_str(&format!(" trace_trail={}", trace_trail.join(" | ")));
        }

        let rehydrate_trail = response
            .rehydration_queue
            .iter()
            .take(3)
            .map(|entry| {
                let reason = entry.reason.as_deref().unwrap_or("rehydrate");
                format!("{reason}:{}", compact_inline(&entry.summary, 32))
            })
            .collect::<Vec<_>>();
        if !rehydrate_trail.is_empty() {
            output.push_str(&format!(" rehydrate_trail={}", rehydrate_trail.join(" | ")));
        }

        if let Some(semantic) = response.semantic_consolidation.as_ref() {
            let trail = semantic
                .highlights
                .iter()
                .take(3)
                .map(|value| compact_inline(value, 40))
                .collect::<Vec<_>>();
            if !trail.is_empty() {
                output.push_str(&format!(" semantic_trail={}", trail.join(" | ")));
            }
        }
    }

    output
}

pub(crate) fn render_policy_summary(response: &MemoryPolicyResponse, follow: bool) -> String {
    let runtime = &response.runtime;
    let runtime_state = if is_default_runtime(runtime) {
        "defaulted"
    } else {
        "live"
    };
    let mut output = format!(
        "policy trust_floor={:.2} working_limit={} rehydrate_limit={} feedback={} semantic_fallback={} skill_gating={} runtime={} read_once={} event_driven={} sandbox_eval={} low_risk_auto={} approval={}",
        response.source_trust_floor,
        response.working_memory.default_limit,
        response.working_memory.rehydration_limit,
        bool_label(response.retrieval_feedback.enabled),
        bool_label(runtime.semantic_fallback.enabled),
        bool_label(runtime.skill_gating.gated_activation),
        runtime_state,
        bool_label(runtime.live_truth.read_once_sources),
        bool_label(runtime.memory_compilation.event_driven_updates),
        bool_label(runtime.skill_gating.sandboxed_evaluation),
        bool_label(runtime.skill_gating.auto_activate_low_risk_only),
        bool_label(runtime.skill_gating.require_policy_approval),
    );

    if follow {
        output.push_str(&format!(
            " route_defaults={} surfaces={} promote_min_salience={:.2} consolidate_min_salience={:.2}",
            response.route_defaults.len(),
            response.retrieval_feedback.tracked_surfaces.join("|"),
            response.promotion.min_salience,
            response.consolidation.min_salience
        ));
    }

    output
}

pub(crate) fn render_skill_policy_summary(response: &MemoryPolicyResponse, follow: bool) -> String {
    let runtime = &response.runtime;
    let skill = &runtime.skill_gating;
    let runtime_state = if is_default_runtime(runtime) {
        "defaulted"
    } else {
        "live"
    };
    let mut output = format!(
        "skill-policy propose={} sandbox={} activate={} low_risk_auto={} eval={} approval={} runtime={} read_once={} event_driven={} visible_memory={} semantic_fallback={}",
        bool_label(skill.propose_from_repeated_patterns),
        bool_label(skill.sandboxed_evaluation),
        bool_label(skill.gated_activation),
        bool_label(skill.auto_activate_low_risk_only),
        bool_label(skill.require_evaluation),
        bool_label(skill.require_policy_approval),
        runtime_state,
        bool_label(runtime.live_truth.read_once_sources),
        bool_label(runtime.memory_compilation.event_driven_updates),
        bool_label(runtime.live_truth.visible_memory_objects),
        bool_label(runtime.semantic_fallback.enabled),
    );

    if follow {
        output.push_str(&format!(
            " flow=pattern->proposal->sandbox->eval->policy->activate runtime={} trust_floor={:.2} semantic_max={}",
            if skill.gated_activation { "gated" } else { "open" },
            response.source_trust_floor,
            runtime.semantic_fallback.max_items_per_query
        ));
    }

    output
}

pub(crate) fn render_skill_catalog_summary(catalog: &SkillCatalog) -> String {
    let custom_visible = catalog.custom.len();
    let builtin_visible = catalog.builtins.len();
    let mut output = format!(
        "skills root={} builtin={} custom={} cache_hits={} scanned={} mode=hybrid",
        catalog.root.display(),
        builtin_visible,
        custom_visible,
        catalog.cache_hits,
        catalog.cache_scanned,
    );
    if let Some(first) = catalog.custom.first() {
        output.push_str(&format!(" first_custom={}::{}", first.name, first.status));
    }
    output
}

pub(crate) fn render_skill_catalog_markdown(catalog: &SkillCatalog) -> String {
    let mut output = String::new();
    output.push_str("# memd skills\n\n");
    output.push_str(&format!("- Root: `{}`\n", catalog.root.display()));
    output.push_str(&format!("- Built-in: `{}`\n", catalog.builtins.len()));
    output.push_str(&format!("- Custom: `{}`\n\n", catalog.custom.len()));
    output.push_str("## Built-in\n\n");
    for entry in &catalog.builtins {
        push_skill_entry_markdown(&mut output, entry);
    }
    output.push_str("\n## Custom\n\n");
    if catalog.custom.is_empty() {
        output.push_str("No custom skills found.\n");
    } else {
        for entry in &catalog.custom {
            push_skill_entry_markdown(&mut output, entry);
        }
    }
    output
}

pub(crate) fn render_skill_catalog_match_summary(
    catalog: &SkillCatalog,
    query: &str,
    matches: &[&SkillCatalogEntry],
) -> String {
    let mut output = format!(
        "skills query=\"{}\" root={} matches={} builtins={} custom={}",
        compact_inline(query, 48),
        catalog.root.display(),
        matches.len(),
        catalog.builtins.len(),
        catalog.custom.len(),
    );
    if let Some(first) = matches.first() {
        output.push_str(&format!(
            " best={} source={} status={} path={} next={} usage={} decision={}",
            first.name,
            first.source,
            first.status,
            first
                .path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "builtin".to_string()),
            skill_activation_path(first),
            first.usage,
            first.decision
        ));
    }
    output
}

pub(crate) fn render_skill_catalog_match_markdown(
    catalog: &SkillCatalog,
    query: &str,
    matches: &[&SkillCatalogEntry],
) -> String {
    let mut output = String::new();
    output.push_str("# memd skill drilldown\n\n");
    output.push_str(&format!("- Query: `{}`\n", query));
    output.push_str(&format!("- Root: `{}`\n", catalog.root.display()));
    output.push_str(&format!("- Matches: `{}`\n\n", matches.len()));
    if matches.is_empty() {
        output.push_str("No skill matched.\n");
        return output;
    }
    for entry in matches {
        output.push_str(&format!(
            "## {}\n\n- source: `{}`\n- status: `{}`\n- next: `{}`\n- usage: `{}`\n- decision: `{}`\n- summary: {}\n",
            entry.name,
            entry.source,
            entry.status,
            skill_activation_path(entry),
            entry.usage,
            entry.decision,
            entry.summary
        ));
        if let Some(path) = entry.path.as_ref() {
            output.push_str(&format!("- path: `{}`\n", path.display()));
        }
        output.push('\n');
    }
    output
}

fn skill_activation_path(entry: &SkillCatalogEntry) -> &'static str {
    if entry.source == "built-in" {
        "use memd subcommand"
    } else {
        "edit the file, then propose via skill-policy"
    }
}

fn push_skill_entry_markdown(output: &mut String, entry: &SkillCatalogEntry) {
    output.push_str(&format!(
        "- `{}` [{}] - {}\n",
        entry.name, entry.status, entry.summary
    ));
    output.push_str(&format!("  - usage: `{}`\n", entry.usage));
    output.push_str(&format!("  - decision: `{}`\n", entry.decision));
    if let Some(path) = entry.path.as_ref() {
        output.push_str(&format!("  - path: `{}`\n", path.display()));
    }
    output.push_str(&format!("  - source: `{}`\n", entry.source));
}

pub(crate) fn render_profile_summary(response: &AgentProfileResponse, follow: bool) -> String {
    let Some(profile) = response.profile.as_ref() else {
        return "profile=none".to_string();
    };

    let mut output = format!(
        "profile agent={} project={} namespace={} route={} intent={} summary_chars={} max_total_chars={} recall_depth={} trust_floor={} styles={}",
        profile.agent,
        profile.project.as_deref().unwrap_or("none"),
        profile.namespace.as_deref().unwrap_or("none"),
        profile.preferred_route.map(route_label).unwrap_or("none"),
        profile.preferred_intent.map(intent_label).unwrap_or("none"),
        profile
            .summary_chars
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        profile
            .max_total_chars
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        profile
            .recall_depth
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        profile
            .source_trust_floor
            .map(|value| format!("{value:.2}"))
            .unwrap_or_else(|| "none".to_string()),
        if profile.style_tags.is_empty() {
            "none".to_string()
        } else {
            profile.style_tags.join("|")
        }
    );

    if follow {
        if let Some(notes) = profile.notes.as_ref() {
            output.push_str(&format!(" notes={}", compact_inline(notes, 72)));
        }
        output.push_str(&format!(
            " created={} updated={}",
            profile.created_at.to_rfc3339(),
            profile.updated_at.to_rfc3339()
        ));
    }

    output
}

pub(crate) fn render_source_summary(response: &SourceMemoryResponse, follow: bool) -> String {
    let mut output = format!("source_memory sources={}", response.sources.len());

    if let Some(best) = response.sources.first() {
        output.push_str(&format!(
            " top={} system={} project={} namespace={} workspace={} visibility={} items={} trust={:.2} avg_confidence={:.2}",
            best.source_agent.as_deref().unwrap_or("none"),
            best.source_system.as_deref().unwrap_or("none"),
            best.project.as_deref().unwrap_or("none"),
            best.namespace.as_deref().unwrap_or("none"),
            best.workspace.as_deref().unwrap_or("none"),
            format_visibility(best.visibility),
            best.item_count,
            best.trust_score,
            best.avg_confidence
        ));
    }

    if follow {
        let trail = response
            .sources
            .iter()
            .take(3)
            .map(|source| {
                format!(
                    "{}:{}:{}:{}:{:.2}",
                    source.source_agent.as_deref().unwrap_or("none"),
                    source.source_system.as_deref().unwrap_or("none"),
                    source.workspace.as_deref().unwrap_or("none"),
                    source.item_count,
                    source.trust_score
                )
            })
            .collect::<Vec<_>>();
        if !trail.is_empty() {
            output.push_str(&format!(" trail={}", trail.join(" | ")));
        }
        if let Some(best) = response.sources.first()
            && !best.tags.is_empty()
        {
            output.push_str(&format!(" tags={}", best.tags.join("|")));
        }
    }

    output
}

pub(crate) fn render_workspace_summary(response: &WorkspaceMemoryResponse, follow: bool) -> String {
    let mut output = format!("workspace_memory workspaces={}", response.workspaces.len());

    if let Some(best) = response.workspaces.first() {
        output.push_str(&format!(
            " top={} visibility={} project={} namespace={} items={} sources={} trust={:.2} avg_confidence={:.2}",
            best.workspace.as_deref().unwrap_or("none"),
            format_visibility(best.visibility),
            best.project.as_deref().unwrap_or("none"),
            best.namespace.as_deref().unwrap_or("none"),
            best.item_count,
            best.source_lane_count,
            best.trust_score,
            best.avg_confidence
        ));
    }

    if follow {
        let trail = response
            .workspaces
            .iter()
            .take(3)
            .map(|workspace| {
                format!(
                    "{}:{}:{}:{:.2}",
                    workspace.workspace.as_deref().unwrap_or("none"),
                    format_visibility(workspace.visibility),
                    workspace.item_count,
                    workspace.trust_score
                )
            })
            .collect::<Vec<_>>();
        if !trail.is_empty() {
            output.push_str(&format!(" trail={}", trail.join(" | ")));
        }
        if let Some(best) = response.workspaces.first()
            && !best.tags.is_empty()
        {
            output.push_str(&format!(" tags={}", best.tags.join("|")));
        }
    }

    output
}

pub(crate) fn render_resume_prompt(snapshot: &crate::ResumeSnapshot) -> String {
    let mut output = String::new();
    output.push_str("# r\n\n");
    output.push_str(&format!(
        "- p={} | n={} | a={} | w={} | v={} | r={} | i={}\n",
        snapshot.project.as_deref().unwrap_or("none"),
        snapshot.namespace.as_deref().unwrap_or("none"),
        snapshot.agent.as_deref().unwrap_or("none"),
        snapshot.workspace.as_deref().unwrap_or("none"),
        snapshot.visibility.as_deref().unwrap_or("all"),
        snapshot.route,
        snapshot.intent,
    ));
    output.push_str("\n## Context Budget\n\n");
    output.push_str(&format!(
        "- ch={} | tok={} | p={} | dup={} | use={}/{}",
        snapshot.estimated_prompt_chars(),
        snapshot.estimated_prompt_tokens(),
        snapshot.context_pressure(),
        snapshot.redundant_context_items(),
        snapshot.working.used_chars,
        snapshot.working.budget_chars,
    ));
    if let Some(age_minutes) = snapshot.resume_state_age_minutes {
        output.push_str(&format!(" | age={}", age_minutes));
    }
    output.push_str(&format!(" | ref={}\n", snapshot.refresh_recommended));
    let hints = snapshot.optimization_hints();
    if !hints.is_empty() {
        output.push_str(&format!(
            "- h={}\n",
            hints
                .iter()
                .take(4)
                .map(|hint| compact_inline(hint, 180))
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }

    let current_task = render_current_task_snapshot(snapshot);
    if !current_task.is_empty() {
        output.push_str("\n## T\n\n");
        output.push_str(&current_task);
    }

    let event_spine = snapshot.event_spine();
    if !event_spine.is_empty() || !snapshot.recent_repo_changes.is_empty() {
        output.push_str("\n## E+LT\n\n");
        let event_part = if event_spine.is_empty() {
            None
        } else {
            let compacted = event_spine
                .iter()
                .take(2)
                .map(|change| compact_inline(change, 180))
                .collect::<Vec<_>>()
                .join(" | ");
            Some(format!("E={}", compacted))
        };
        let lt_part = if snapshot.recent_repo_changes.is_empty() {
            None
        } else {
            let compacted = snapshot
                .recent_repo_changes
                .iter()
                .take(2)
                .map(|change| compact_inline(change, 180))
                .collect::<Vec<_>>()
                .join(" | ");
            Some(format!("LT={}", compacted))
        };
        let mut parts = Vec::new();
        if let Some(part) = event_part {
            parts.push(part);
        }
        if let Some(part) = lt_part {
            parts.push(part);
        }
        output.push_str(&format!("- {}\n", parts.join(" | ")));
    }

    output.push_str("\n## W\n\n");
    if snapshot.working.records.is_empty() {
        output.push_str("- none\n");
    } else {
        let records = snapshot
            .working
            .records
            .iter()
            .take(2)
            .map(|record| compact_inline(&record.record, 220))
            .collect::<Vec<_>>();
        output.push_str(&format!("- w={}", records.join(" | ")));
        if snapshot.working.records.len() > 2 {
            output.push_str(&format!(" (+{} more)", snapshot.working.records.len() - 2));
        }
        output.push('\n');
    }

    let mut ri_parts = Vec::new();
    if !snapshot.working.rehydration_queue.is_empty() {
        for artifact in snapshot.working.rehydration_queue.iter().take(4) {
            ri_parts.push(format!(
                "r={}:{}",
                artifact.label,
                compact_inline(&artifact.summary, 180)
            ));
        }
    }
    if !snapshot.inbox.items.is_empty() {
        for item in snapshot.inbox.items.iter().take(5) {
            let reasons = if item.reasons.is_empty() {
                "none".to_string()
            } else {
                compact_inline(&item.reasons.join(", "), 100)
            };
            ri_parts.push(format!(
                "i={:?}/{:?}:{}|r={}",
                item.item.kind,
                item.item.status,
                compact_inline(&item.item.content, 160),
                reasons
            ));
        }
    }
    if !ri_parts.is_empty() {
        output.push_str("\n## RI\n\n");
        output.push_str(&format!("- {}\n", ri_parts.join(" | ")));
    }

    if let Some(first) = snapshot.workspaces.workspaces.first() {
        output.push_str("\n## L\n\n");
        let extras = snapshot.workspaces.workspaces.len() - 1;
        output.push_str(&format!(
            "- l={}/{}/{} | v={} | it={} | src={} | tr={:.2}{} \n",
            first.project.as_deref().unwrap_or("none"),
            first.namespace.as_deref().unwrap_or("none"),
            first.workspace.as_deref().unwrap_or("none"),
            format_visibility(first.visibility),
            first.item_count,
            first.source_lane_count,
            first.trust_score,
            if extras > 0 {
                format!(" (+{} more)", extras)
            } else {
                "".to_string()
            }
        ));
    }

    let mut sc_parts = Vec::new();
    if let Some(semantic) = snapshot
        .semantic
        .as_ref()
        .filter(|semantic| !semantic.items.is_empty())
    {
        let items = semantic
            .items
            .iter()
            .take(2)
            .map(|item| format!("{}@{:.2}", compact_inline(&item.content, 180), item.score))
            .collect::<Vec<_>>();
        sc_parts.push(format!("S={}", items.join(" | ")));
    }

    if !sc_parts.is_empty() {
        output.push_str("\n## S\n\n");
        output.push_str(&format!("- {}\n", sc_parts.join(" | ")));
    }

    output
}

fn render_current_task_snapshot(snapshot: &crate::ResumeSnapshot) -> String {
    let mut output = String::new();

    let capsule = snapshot.workflow_capsule();
    if !capsule.is_empty() {
        let summary = capsule
            .iter()
            .take(4)
            .map(|line| compact_inline(line, 180))
            .collect::<Vec<_>>()
            .join(" | ");
        output.push_str(&format!("- t={}\n", summary));
    }

    output
}

pub(crate) fn render_handoff_prompt(snapshot: &crate::HandoffSnapshot) -> String {
    let mut output = String::new();
    output.push_str("# h\n\n");
    output.push_str(&format!(
        "- at={} | p={} | n={} | a={} | w={} | v={} | r={} | i={}\n",
        snapshot.generated_at.to_rfc3339(),
        snapshot.resume.project.as_deref().unwrap_or("none"),
        snapshot.resume.namespace.as_deref().unwrap_or("none"),
        snapshot.resume.agent.as_deref().unwrap_or("none"),
        snapshot.resume.workspace.as_deref().unwrap_or("none"),
        snapshot.resume.visibility.as_deref().unwrap_or("all"),
        snapshot.resume.route,
        snapshot.resume.intent,
    ));

    output.push_str("\n## W\n\n");
    if snapshot.resume.working.records.is_empty() {
        output.push_str("- none\n");
    } else {
        let records = snapshot
            .resume
            .working
            .records
            .iter()
            .take(2)
            .map(|record| compact_inline(&record.record, 220))
            .collect::<Vec<_>>();
        output.push_str(&format!("- w={}", records.join(" | ")));
        if snapshot.resume.working.records.len() > 2 {
            output.push_str(&format!(
                " (+{} more)",
                snapshot.resume.working.records.len() - 2
            ));
        }
        output.push('\n');
    }

    let mut ri_parts_resume = Vec::new();
    if !snapshot.resume.working.rehydration_queue.is_empty() {
        for artifact in snapshot.resume.working.rehydration_queue.iter().take(5) {
            ri_parts_resume.push(format!(
                "r={}:{}",
                artifact.label,
                compact_inline(&artifact.summary, 180)
            ));
        }
    }
    if !snapshot.resume.inbox.items.is_empty() {
        for item in snapshot.resume.inbox.items.iter().take(5) {
            let reasons = if item.reasons.is_empty() {
                "none".to_string()
            } else {
                compact_inline(&item.reasons.join(", "), 100)
            };
            ri_parts_resume.push(format!(
                "i={:?}/{:?}:{}|r={}",
                item.item.kind,
                item.item.status,
                compact_inline(&item.item.content, 160),
                reasons
            ));
        }
    }
    if !ri_parts_resume.is_empty() {
        output.push_str("\n## RI\n\n");
        output.push_str(&format!("- {}\n", ri_parts_resume.join(" | ")));
    }

    if !snapshot.resume.workspaces.workspaces.is_empty() {
        output.push_str("\n## L\n\n");
        for workspace in snapshot.resume.workspaces.workspaces.iter().take(5) {
            output.push_str(&format!(
                "- {}/{}/{} | v={} | it={} | src={} | tr={:.2}\n",
                workspace.project.as_deref().unwrap_or("none"),
                workspace.namespace.as_deref().unwrap_or("none"),
                workspace.workspace.as_deref().unwrap_or("none"),
                format_visibility(workspace.visibility),
                workspace.item_count,
                workspace.source_lane_count,
                workspace.trust_score
            ));
        }
    }

    let mut sc_resume_parts = Vec::new();
    if let Some(semantic) = snapshot
        .resume
        .semantic
        .as_ref()
        .filter(|semantic| !semantic.items.is_empty())
    {
        let items = semantic
            .items
            .iter()
            .take(2)
            .map(|item| format!("{}@{:.2}", compact_inline(&item.content, 180), item.score))
            .collect::<Vec<_>>();
        sc_resume_parts.push(format!("S={}", items.join(" | ")));
    }

    if !sc_resume_parts.is_empty() {
        output.push_str("\n## S\n\n");
        output.push_str(&format!("- {}\n", sc_resume_parts.join(" | ")));
    }

    output
}

pub(crate) fn render_consolidate_summary(
    response: &memd_schema::MemoryConsolidationResponse,
    follow: bool,
) -> String {
    let mut output = format!(
        "consolidate scanned={} groups={} consolidated={} duplicates={} events={}",
        response.scanned,
        response.groups,
        response.consolidated,
        response.duplicates,
        response.events
    );

    if follow && !response.highlights.is_empty() {
        let highlights = response
            .highlights
            .iter()
            .take(3)
            .map(|value| compact_inline(value, 40))
            .collect::<Vec<_>>();
        output.push_str(&format!(" trail={}", highlights.join(" | ")));
    }

    output
}

pub(crate) fn render_maintenance_report_summary(
    response: &memd_schema::MemoryMaintenanceReportResponse,
    follow: bool,
) -> String {
    let mut output = format!(
        "maintenance reinforced={} cooled={} consolidated={} stale={} skipped={}",
        response.reinforced_candidates,
        response.cooled_candidates,
        response.consolidated_candidates,
        response.stale_items,
        response.skipped
    );

    if follow && !response.highlights.is_empty() {
        let highlights = response
            .highlights
            .iter()
            .take(3)
            .map(|value| compact_inline(value, 40))
            .collect::<Vec<_>>();
        output.push_str(&format!(" trail={}", highlights.join(" | ")));
    }

    output
}

pub(crate) fn render_eval_summary(response: &crate::BundleEvalResponse) -> String {
    let mut output = format!(
        "eval status={} score={} baseline={} delta={} agent={} workspace={} working={} context={} rehydration={} inbox={} lanes={} semantic={}",
        response.status,
        response.score,
        response
            .baseline_score
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        response
            .score_delta
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
        response.agent.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.working_records,
        response.context_records,
        response.rehydration_items,
        response.inbox_items,
        response.workspace_lanes,
        response.semantic_hits
    );

    if !response.findings.is_empty() {
        let findings = response
            .findings
            .iter()
            .take(3)
            .map(|value| compact_inline(value, 56))
            .collect::<Vec<_>>();
        output.push_str(&format!(" findings={}", findings.join(" | ")));
    }

    if !response.changes.is_empty() {
        let changes = response
            .changes
            .iter()
            .take(3)
            .map(|value| compact_inline(value, 40))
            .collect::<Vec<_>>();
        output.push_str(&format!(" changes={}", changes.join(" | ")));
    }

    if !response.recommendations.is_empty() {
        let recommendations = response
            .recommendations
            .iter()
            .take(2)
            .map(|value| compact_inline(value, 44))
            .collect::<Vec<_>>();
        output.push_str(&format!(" next={}", recommendations.join(" | ")));
    }

    output
}

pub(crate) fn render_gap_summary(response: &crate::GapReport) -> String {
    let mut output = format!(
        "gap bundle={} project={} namespace={} agent={} session={} workspace={} visibility={} candidates={} high_priority={} eval_score={}",
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.candidate_count,
        response.high_priority_count,
        response
            .eval_score
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string())
    );

    if let Some(status) = response.eval_status.as_deref() {
        output.push_str(&format!(" eval_status={status}"));
    } else {
        output.push_str(" eval_status=none");
    }

    if response.eval_score_delta.is_some() || response.previous_candidate_count.is_some() {
        output.push_str(&format!(
            " eval_score_delta={} prev_candidates={}",
            response
                .eval_score_delta
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
            response
                .previous_candidate_count
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string()),
        ));
    }

    if let Some(changes) = response.changes.first() {
        output.push_str(&format!(" next=top:{}", changes));
    }

    output.push_str(&format!(
        " commit_window={} recent={}",
        response.limit, response.commits_checked
    ));

    output
}

pub(crate) fn render_scenario_summary(response: &crate::ScenarioReport) -> String {
    let mut output = format!(
        "scenario name={} bundle={} project={} namespace={} agent={} session={} workspace={} visibility={} score={}/{} passed={} failed={}",
        response.scenario,
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.score,
        response.max_score,
        response.passed_checks,
        response.failed_checks
    );

    if let Some(top_check) = response.checks.first() {
        output.push_str(&format!(
            " first_check={}:{}",
            top_check.name, top_check.status
        ));
    }

    if let Some(next) = response.next_actions.first() {
        output.push_str(&format!(" next_action={next}"));
    }

    output
}

pub(crate) fn render_improvement_summary(response: &crate::ImprovementReport) -> String {
    let mut output = format!(
        "improve bundle={} project={} namespace={} agent={} session={} workspace={} visibility={} apply={} iterations={} converged={} initial_candidates={} final_candidates={} final_score={}",
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.apply,
        response.iterations.len(),
        response.converged,
        response
            .initial_gap
            .as_ref()
            .map(|value| value.candidate_count)
            .unwrap_or(0),
        response
            .final_gap
            .as_ref()
            .map(|value| value.candidate_count)
            .unwrap_or(0),
        response
            .final_gap
            .as_ref()
            .and_then(|value| value.eval_score)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string()),
    );
    output.push_str(&format!(
        " max_iterations={} started_at={}",
        response.max_iterations,
        response.started_at.format("%Y-%m-%d %H:%M:%S UTC")
    ));
    if response.final_gap.is_some()
        && let Some(changes) = response.final_changes.first()
    {
        output.push_str(&format!(" next=top:{}", changes));
    }
    if !response.iterations.is_empty() {
        let iteration_overview = response
            .iterations
            .iter()
            .map(|iteration| {
                format!(
                    "iter{} pre={}->{}, actions={}",
                    iteration.iteration,
                    iteration.pre_gap.candidate_count,
                    iteration.post_gap.as_ref().map_or_else(
                        || "none".to_string(),
                        |summary| summary.candidate_count.to_string(),
                    ),
                    iteration.planned_actions.len()
                )
            })
            .collect::<Vec<_>>()
            .join(" | ");
        output.push_str(&format!(" iterations=[{}]", iteration_overview));
    }
    output
}

pub(crate) fn render_scenario_markdown(response: &crate::ScenarioReport) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# memd scenario report: {}\n\n",
        response.scenario
    ));
    markdown.push_str(&format!(
        "- bundle: {}\n- project: {}\n- namespace: {}\n- agent: {}\n- session: {}\n- workspace: {}\n- visibility: {}\n- score: {}/{}\n- passed_checks: {}\n- failed_checks: {}\n- generated_at: {}\n- completed_at: {}\n",
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.score,
        response.max_score,
        response.passed_checks,
        response.failed_checks,
        response.generated_at,
        response.completed_at
    ));

    markdown.push_str("\n## Checks\n\n");
    if response.checks.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for check in &response.checks {
            markdown.push_str(&format!(
                "- [{}] {} ({}pts): {}\n",
                check.status, check.name, check.points, check.details
            ));
        }
    }

    markdown.push_str("\n## Evidence\n\n");
    if response.evidence.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.evidence {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown.push_str("\n## Findings\n\n");
    if response.findings.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.findings {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown.push_str("\n## Next Actions\n\n");
    if response.next_actions.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.next_actions {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown
}

pub(crate) fn render_composite_summary(response: &crate::CompositeReport) -> String {
    let mut output = format!(
        "composite bundle={} project={} namespace={} agent={} session={} workspace={} visibility={} score={}/{}",
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.score,
        response.max_score
    );
    if let Some(scenario) = response.scenario.as_deref() {
        output.push_str(&format!(" scenario={scenario}"));
    }
    if let Some(gate) = response.gates.first() {
        output.push_str(&format!(" first_gate={}:{}", gate.name, gate.status));
    }
    if let Some(dim) = response.dimensions.first() {
        output.push_str(&format!(" first_dimension={}:{}", dim.name, dim.score));
    }
    output
}

pub(crate) fn render_composite_markdown(response: &crate::CompositeReport) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd composite report\n\n");
    markdown.push_str(&format!(
        "- bundle: {}\n- project: {}\n- namespace: {}\n- agent: {}\n- session: {}\n- workspace: {}\n- visibility: {}\n- scenario: {}\n- score: {}/{}\n- generated_at: {}\n- completed_at: {}\n",
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.scenario.as_deref().unwrap_or("none"),
        response.score,
        response.max_score,
        response.generated_at,
        response.completed_at
    ));

    markdown.push_str("\n## Dimensions\n\n");
    if response.dimensions.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for dimension in &response.dimensions {
            markdown.push_str(&format!(
                "- [{}] {} (weight {}): {}\n",
                dimension.score, dimension.name, dimension.weight, dimension.details
            ));
        }
    }

    markdown.push_str("\n## Gates\n\n");
    if response.gates.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for gate in &response.gates {
            markdown.push_str(&format!(
                "- [{}] {}: {}\n",
                gate.status, gate.name, gate.details
            ));
        }
    }

    markdown.push_str("\n## Evidence\n\n");
    if response.evidence.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.evidence {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown.push_str("\n## Findings\n\n");
    if response.findings.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.findings {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown.push_str("\n## Recommendations\n\n");
    if response.recommendations.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.recommendations {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown
}

pub(crate) fn render_experiment_summary(response: &crate::ExperimentReport) -> String {
    let mut output = format!(
        "experiment bundle={} project={} namespace={} agent={} session={} workspace={} visibility={} accepted={} restored={} score={}/{} iterations={}",
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.accepted,
        response.restored,
        response.composite.score,
        response.composite.max_score,
        response.improvement.iterations.len()
    );
    output.push_str(&format!(
        " max_iterations={} accept_below={} apply={} consolidate={}",
        response.max_iterations, response.accept_below, response.apply, response.consolidate
    ));
    if let Some(entry) = response.trail.first() {
        output.push_str(&format!(" first_trail={entry}"));
    }
    if let Some(evolution) = &response.evolution {
        output.push_str(&format!(
            " evolution={} scope={}/{} authority={} merge={} durability={}",
            evolution.proposal_state,
            evolution.scope_class,
            evolution.scope_gate,
            evolution.authority_tier,
            evolution.merge_status,
            evolution.durability_status
        ));
        if evolution.branch != "none" {
            output.push_str(&format!(
                " evo_branch={}",
                compact_inline(&evolution.branch, 64)
            ));
        }
    }
    output
}

pub(crate) fn render_experiment_markdown(response: &crate::ExperimentReport) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd experiment report\n\n");
    markdown.push_str(&format!(
        "- bundle: {}\n- project: {}\n- namespace: {}\n- agent: {}\n- session: {}\n- workspace: {}\n- visibility: {}\n- max_iterations: {}\n- accept_below: {}\n- apply: {}\n- consolidate: {}\n- accepted: {}\n- restored: {}\n- started_at: {}\n- completed_at: {}\n",
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.max_iterations,
        response.accept_below,
        response.apply,
        response.consolidate,
        response.accepted,
        response.restored,
        response.started_at,
        response.completed_at
    ));

    if let Some(evolution) = &response.evolution {
        markdown.push_str("\n## Evolution\n\n");
        markdown.push_str(&format!(
            "- proposal_state: {}\n- scope: {}/{}\n- authority_tier: {}\n- merge_status: {}\n- durability_status: {}\n- branch: {}\n- durable_truth: {}\n",
            evolution.proposal_state,
            evolution.scope_class,
            evolution.scope_gate,
            evolution.authority_tier,
            evolution.merge_status,
            evolution.durability_status,
            evolution.branch,
            evolution.durable_truth
        ));
    }

    markdown.push_str("\n## Trail\n\n");
    if response.trail.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.trail {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown.push_str("\n## Learnings\n\n");
    if response.learnings.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.learnings {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown.push_str("\n## Findings\n\n");
    if response.findings.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.findings {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown.push_str("\n## Recommendations\n\n");
    if response.recommendations.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.recommendations {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown.push_str("\n## Evidence\n\n");
    if response.evidence.is_empty() {
        markdown.push_str("- none\n");
    } else {
        for item in &response.evidence {
            markdown.push_str(&format!("- {item}\n"));
        }
    }

    markdown.push_str("\n## Improvement\n\n");
    markdown.push_str(&format!(
        "- iterations: {}\n- converged: {}\n- final_candidates: {}\n- final_score: {}\n",
        response.improvement.iterations.len(),
        response.improvement.converged,
        response
            .improvement
            .final_gap
            .as_ref()
            .map(|value| value.candidate_count)
            .unwrap_or(0),
        response
            .improvement
            .final_gap
            .as_ref()
            .and_then(|value| value.eval_score)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string())
    ));

    markdown.push_str("\n## Composite\n\n");
    markdown.push_str(&format!(
        "- score: {}/{}\n- scenario: {}\n",
        response.composite.score,
        response.composite.max_score,
        response.composite.scenario.as_deref().unwrap_or("none"),
    ));
    for gate in &response.composite.gates {
        markdown.push_str(&format!(
            "- gate [{}] {}: {}\n",
            gate.status, gate.name, gate.details
        ));
    }

    markdown
}

pub(crate) fn render_improvement_markdown(response: &crate::ImprovementReport) -> String {
    let mut markdown = String::new();
    markdown.push_str("# memd improvement report\n\n");
    markdown.push_str(&format!(
        "- bundle: {}\n- project: {}\n- namespace: {}\n- agent: {}\n- session: {}\n- workspace: {}\n- visibility: {}\n- apply: {}\n- max_iterations: {}\n- converged: {}\n- started_at: {}\n- completed_at: {}\n- initial_candidates: {}\n- final_candidates: {}\n- final_score: {}\n",
        response.bundle_root,
        response.project.as_deref().unwrap_or("none"),
        response.namespace.as_deref().unwrap_or("none"),
        response.agent.as_deref().unwrap_or("none"),
        response.session.as_deref().unwrap_or("none"),
        response.workspace.as_deref().unwrap_or("none"),
        response.visibility.as_deref().unwrap_or("all"),
        response.apply,
        response.max_iterations,
        response.converged,
        response.started_at,
        response.completed_at,
        response
            .initial_gap
            .as_ref()
            .map(|value| value.candidate_count)
            .unwrap_or(0),
        response
            .final_gap
            .as_ref()
            .map(|value| value.candidate_count)
            .unwrap_or(0),
        response
            .final_gap
            .as_ref()
            .and_then(|value| value.eval_score)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string())
    ));

    if !response.final_changes.is_empty() {
        markdown.push_str("\n## Final Changes\n\n");
        for change in &response.final_changes {
            markdown.push_str(&format!("- {change}\n"));
        }
    }

    markdown.push_str("\n## Iterations\n\n");
    if response.iterations.is_empty() {
        markdown.push_str("- no iterations executed\n");
        return markdown;
    }

    for iteration in &response.iterations {
        markdown.push_str(&format!("### Iteration {}\n\n", iteration.iteration));
        markdown.push_str(&format!(
            "- pre_gap: candidates={} high_priority={} eval_score={}\n",
            iteration.pre_gap.candidate_count,
            iteration.pre_gap.high_priority_count,
            iteration
                .pre_gap
                .eval_score
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        ));

        if let Some(post_gap) = &iteration.post_gap {
            markdown.push_str(&format!(
                "- post_gap: candidates={} high_priority={} eval_score={}\n",
                post_gap.candidate_count,
                post_gap.high_priority_count,
                post_gap
                    .eval_score
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string())
            ));
        } else {
            markdown.push_str("- post_gap: none\n");
        }

        markdown.push_str("- planned actions:\n");
        if iteration.planned_actions.is_empty() {
            markdown.push_str("  - none\n");
        } else {
            for action in &iteration.planned_actions {
                let extras = format!(
                    "{}{}{}{}",
                    action
                        .task_id
                        .as_ref()
                        .map(|value| format!(" task={value}"))
                        .unwrap_or_default(),
                    action
                        .scope
                        .as_ref()
                        .map(|value| format!(" scope={value}"))
                        .unwrap_or_default(),
                    action
                        .target_session
                        .as_ref()
                        .map(|value| format!(" target_session={value}"))
                        .unwrap_or_default(),
                    action
                        .message_id
                        .as_ref()
                        .map(|value| format!(" message={value}"))
                        .unwrap_or_default(),
                );
                markdown.push_str(&format!(
                    "  - {} [{}] {}{}\n",
                    action.action, action.priority, action.reason, extras,
                ));
            }
        }

        markdown.push_str("- execution:\n");
        if iteration.executed_actions.is_empty() {
            markdown.push_str("  - none\n");
        } else {
            for result in &iteration.executed_actions {
                markdown.push_str(&format!(
                    "  - {} {}: {}\n",
                    result.status, result.action, result.detail
                ));
            }
        }
        markdown.push('\n');
    }

    markdown
}

pub(crate) fn render_repair_summary(response: &RepairMemoryResponse, follow: bool) -> String {
    let mut output = format!(
        "repair mode={} item={} status={} confidence={:.2} reasons={}",
        format!("{:?}", response.mode).to_ascii_lowercase(),
        short_uuid(response.item.id),
        format!("{:?}", response.item.status).to_ascii_lowercase(),
        response.item.confidence,
        response.reasons.join("|")
    );

    if follow {
        output.push_str(&format!(
            " source_agent={} source_system={} source_path={}",
            response.item.source_agent.as_deref().unwrap_or("none"),
            response.item.source_system.as_deref().unwrap_or("none"),
            response.item.source_path.as_deref().unwrap_or("none")
        ));
        if !response.item.tags.is_empty() {
            output.push_str(&format!(" tags={}", response.item.tags.join("|")));
        }
    }

    output
}

pub(crate) fn render_visible_memory_home(
    response: &VisibleMemorySnapshotResponse,
    follow: bool,
) -> String {
    let focus = &response.home.focus_artifact;
    let mut output = format!(
        "memory_home focus={} status={} freshness={} visibility={} workspace={} inbox={} repair={} awareness={} nodes={} edges={}",
        compact_inline(&focus.title, 48),
        visible_memory_status_label(focus.status),
        compact_inline(&focus.freshness, 24),
        focus.visibility.map(format_visibility).unwrap_or("none"),
        focus.workspace.as_deref().unwrap_or("none"),
        response.home.inbox_count,
        response.home.repair_count,
        response.home.awareness_count,
        response.knowledge_map.nodes.len(),
        response.knowledge_map.edges.len()
    );

    if follow {
        output.push_str(&format!(
            " source_system={} source_path={} producer={} trust={} actions={}",
            focus.provenance.source_system.as_deref().unwrap_or("none"),
            focus.provenance.source_path.as_deref().unwrap_or("none"),
            focus.provenance.producer.as_deref().unwrap_or("none"),
            compact_inline(&focus.provenance.trust_reason, 64),
            if focus.actions.is_empty() {
                "none".to_string()
            } else {
                focus.actions.join("|")
            }
        ));

        let trail = response
            .knowledge_map
            .nodes
            .iter()
            .take(3)
            .map(|node| {
                format!(
                    "{}:{}:{}",
                    compact_inline(&node.title, 24),
                    node.artifact_kind,
                    visible_memory_status_label(node.status)
                )
            })
            .collect::<Vec<_>>();
        if !trail.is_empty() {
            output.push_str(&format!(" trail={}", trail.join(" | ")));
        }
    }

    output
}

pub(crate) fn render_explain_summary(response: &ExplainMemoryResponse, follow: bool) -> String {
    let mut output = format!(
        "explain item={} route={} intent={} status={} confidence={:.2} preferred={} branch={} siblings={} retrievals={} entity={} events={} sources={} rehydrate={} hooks={} reasons={}",
        short_uuid(response.item.id),
        route_label(response.route),
        intent_label(response.intent),
        format!("{:?}", response.item.status).to_ascii_lowercase(),
        response.item.confidence,
        response.item.preferred,
        response.item.belief_branch.as_deref().unwrap_or("none"),
        response.branch_siblings.len(),
        response.retrieval_feedback.total_retrievals,
        response
            .entity
            .as_ref()
            .map(|entity| format!("{}:{}", short_uuid(entity.id), entity.entity_type))
            .unwrap_or_else(|| "none".to_string()),
        response.events.len(),
        response.sources.len(),
        response.rehydration.len(),
        response.policy_hooks.len(),
        compact_inline(&response.reasons.join("|"), 96)
    );

    if follow {
        if let Some(first_event) = response.events.first() {
            output.push_str(&format!(
                " latest_event={} trail={}",
                first_event.event_type,
                response
                    .events
                    .iter()
                    .take(3)
                    .map(|event| format!(
                        "{}:{}",
                        event.event_type,
                        compact_inline(&event.summary, 36)
                    ))
                    .collect::<Vec<_>>()
                    .join(" | ")
            ));
        }
        if let Some(best_source) = response.sources.first() {
            output.push_str(&format!(
                " top_source={} system={} trust={:.2} avg_confidence={:.2}",
                best_source.source_agent.as_deref().unwrap_or("none"),
                best_source.source_system.as_deref().unwrap_or("none"),
                best_source.trust_score,
                best_source.avg_confidence
            ));
            if !best_source.tags.is_empty() {
                output.push_str(&format!(" tags={}", best_source.tags.join("|")));
            }
        }
        if !response.branch_siblings.is_empty() {
            let siblings = response
                .branch_siblings
                .iter()
                .take(3)
                .map(|sibling| {
                    format!(
                        "{}:{}:{:.2}:{}",
                        sibling.belief_branch.as_deref().unwrap_or("none"),
                        short_uuid(sibling.id),
                        sibling.confidence,
                        if sibling.preferred {
                            "preferred"
                        } else {
                            "candidate"
                        }
                    )
                })
                .collect::<Vec<_>>();
            output.push_str(&format!(" sibling_branches={}", siblings.join(" | ")));
        }
        if !response.retrieval_feedback.by_surface.is_empty() {
            let surfaces = response
                .retrieval_feedback
                .by_surface
                .iter()
                .take(4)
                .map(|surface| format!("{}:{}", surface.surface, surface.count))
                .collect::<Vec<_>>();
            output.push_str(&format!(" retrieval_surfaces={}", surfaces.join("|")));
        }
        let hooks = response
            .policy_hooks
            .iter()
            .take(4)
            .cloned()
            .collect::<Vec<_>>();
        if !hooks.is_empty() {
            output.push_str(&format!(" hooks={}", hooks.join("|")));
        }
        let trail = response
            .rehydration
            .iter()
            .take(3)
            .map(|artifact| {
                format!(
                    "{}:{}:{}",
                    artifact.kind,
                    artifact.label,
                    compact_inline(&artifact.summary, 32)
                )
            })
            .collect::<Vec<_>>();
        if !trail.is_empty() {
            output.push_str(&format!(" rehydration={}", trail.join(" | ")));
        }
    }

    output
}

fn compact_inline(value: &str, max_chars: usize) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() <= max_chars {
        return normalized;
    }

    let mut compact = String::with_capacity(max_chars + 3);
    for ch in normalized.chars().take(max_chars.saturating_sub(3)) {
        compact.push(ch);
    }
    compact.push_str("...");
    compact
}

pub(crate) fn short_uuid(id: uuid::Uuid) -> String {
    id.to_string().chars().take(8).collect()
}

fn route_label(route: RetrievalRoute) -> &'static str {
    match route {
        RetrievalRoute::Auto => "auto",
        RetrievalRoute::LocalOnly => "local_only",
        RetrievalRoute::SyncedOnly => "synced_only",
        RetrievalRoute::ProjectOnly => "project_only",
        RetrievalRoute::GlobalOnly => "global_only",
        RetrievalRoute::LocalFirst => "local_first",
        RetrievalRoute::SyncedFirst => "synced_first",
        RetrievalRoute::ProjectFirst => "project_first",
        RetrievalRoute::GlobalFirst => "global_first",
        RetrievalRoute::All => "all",
    }
}

fn intent_label(intent: RetrievalIntent) -> &'static str {
    match intent {
        RetrievalIntent::General => "general",
        RetrievalIntent::CurrentTask => "current_task",
        RetrievalIntent::Decision => "decision",
        RetrievalIntent::Runbook => "runbook",
        RetrievalIntent::Procedural => "procedural",
        RetrievalIntent::SelfModel => "self_model",
        RetrievalIntent::Topology => "topology",
        RetrievalIntent::Preference => "preference",
        RetrievalIntent::Fact => "fact",
        RetrievalIntent::Pattern => "pattern",
    }
}

fn format_visibility(value: memd_schema::MemoryVisibility) -> &'static str {
    match value {
        memd_schema::MemoryVisibility::Private => "private",
        memd_schema::MemoryVisibility::Workspace => "workspace",
        memd_schema::MemoryVisibility::Public => "public",
    }
}

fn visible_memory_status_label(status: VisibleMemoryStatus) -> &'static str {
    match status {
        VisibleMemoryStatus::Current => "current",
        VisibleMemoryStatus::Candidate => "candidate",
        VisibleMemoryStatus::Stale => "stale",
        VisibleMemoryStatus::Superseded => "superseded",
        VisibleMemoryStatus::Conflicted => "conflicted",
        VisibleMemoryStatus::Archived => "archived",
    }
}

fn bool_label(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

pub(crate) fn is_default_runtime(runtime: &memd_schema::MemoryPolicyRuntime) -> bool {
    !runtime.live_truth.read_once_sources
        && !runtime.live_truth.raw_reopen_requires_change_or_doubt
        && !runtime.live_truth.visible_memory_objects
        && !runtime.live_truth.compile_from_events
        && !runtime.memory_compilation.event_driven_updates
        && !runtime.memory_compilation.patch_not_rewrite
        && !runtime.memory_compilation.preserve_provenance
        && !runtime.memory_compilation.source_on_demand
        && !runtime.semantic_fallback.enabled
        && !runtime.semantic_fallback.source_of_truth
        && runtime.semantic_fallback.max_items_per_query == 0
        && !runtime.semantic_fallback.rerank_with_visible_memory
        && !runtime.skill_gating.propose_from_repeated_patterns
        && !runtime.skill_gating.sandboxed_evaluation
        && !runtime.skill_gating.auto_activate_low_risk_only
        && !runtime.skill_gating.gated_activation
        && !runtime.skill_gating.require_evaluation
        && !runtime.skill_gating.require_policy_approval
}

#[cfg(test)]
mod tests {
    use super::{
        render_bundle_status_summary, render_harness_preset_markdown, render_policy_summary,
        render_skill_policy_summary, render_visible_memory_home,
    };
    use crate::harness::preset::HarnessPresetRegistry;
    use memd_schema::{
        MemoryKind, MemoryPolicyConsolidation, MemoryPolicyDecay, MemoryPolicyFeedback,
        MemoryPolicyLiveTruth, MemoryPolicyMemoryCompilation, MemoryPolicyPromotion,
        MemoryPolicyResponse, MemoryPolicyRouteDefault, MemoryPolicyRuntime,
        MemoryPolicySemanticFallback, MemoryPolicySkillGating, MemoryPolicyWorkingMemory,
        MemoryScope, MemoryStage, MemoryStatus, MemoryVisibility, RetrievalIntent, RetrievalRoute,
        SourceQuality, VisibleMemoryArtifact, VisibleMemoryGraphEdge, VisibleMemoryGraphNode,
        VisibleMemoryHome, VisibleMemoryKnowledgeMap, VisibleMemoryProvenance,
        VisibleMemorySnapshotResponse, VisibleMemoryStatus,
    };
    use serde_json::json;

    #[test]
    fn policy_summary_includes_skill_gates() {
        let response = MemoryPolicyResponse {
            retrieval_order: vec![MemoryScope::Local, MemoryScope::Project],
            route_defaults: vec![MemoryPolicyRouteDefault {
                intent: RetrievalIntent::CurrentTask,
                route: RetrievalRoute::LocalFirst,
            }],
            working_memory: MemoryPolicyWorkingMemory {
                budget_chars: 1600,
                max_chars_per_item: 220,
                default_limit: 8,
                rehydration_limit: 3,
            },
            retrieval_feedback: MemoryPolicyFeedback {
                enabled: true,
                tracked_surfaces: vec!["search".to_string(), "working".to_string()],
                max_items_per_request: 3,
            },
            source_trust_floor: 0.6,
            runtime: MemoryPolicyRuntime {
                live_truth: MemoryPolicyLiveTruth {
                    read_once_sources: true,
                    raw_reopen_requires_change_or_doubt: true,
                    visible_memory_objects: true,
                    compile_from_events: true,
                },
                memory_compilation: MemoryPolicyMemoryCompilation {
                    event_driven_updates: true,
                    patch_not_rewrite: true,
                    preserve_provenance: true,
                    source_on_demand: true,
                },
                semantic_fallback: MemoryPolicySemanticFallback {
                    enabled: true,
                    source_of_truth: false,
                    max_items_per_query: 3,
                    rerank_with_visible_memory: true,
                },
                skill_gating: MemoryPolicySkillGating {
                    propose_from_repeated_patterns: true,
                    sandboxed_evaluation: true,
                    auto_activate_low_risk_only: true,
                    gated_activation: true,
                    require_evaluation: true,
                    require_policy_approval: true,
                },
            },
            promotion: MemoryPolicyPromotion {
                min_salience: 0.22,
                min_events: 3,
                lookback_days: 14,
                default_ttl_days: 90,
            },
            decay: MemoryPolicyDecay {
                max_items: 128,
                inactive_days: 21,
                max_decay: 0.12,
                record_events: true,
            },
            consolidation: MemoryPolicyConsolidation {
                max_groups: 24,
                min_events: 3,
                lookback_days: 14,
                min_salience: 0.22,
                record_events: true,
            },
        };

        let summary = render_policy_summary(&response, true);
        assert!(summary.contains("skill_gating=on"));
        assert!(summary.contains("sandbox_eval=on"));
        assert!(summary.contains("low_risk_auto=on"));
        assert!(summary.contains("approval=on"));
        assert!(summary.contains("read_once=on"));
    }

    #[test]
    fn skill_policy_summary_includes_lifecycle_flow() {
        let response = MemoryPolicyResponse {
            retrieval_order: vec![MemoryScope::Local, MemoryScope::Project],
            route_defaults: vec![MemoryPolicyRouteDefault {
                intent: RetrievalIntent::CurrentTask,
                route: RetrievalRoute::LocalFirst,
            }],
            working_memory: MemoryPolicyWorkingMemory {
                budget_chars: 1600,
                max_chars_per_item: 220,
                default_limit: 8,
                rehydration_limit: 3,
            },
            retrieval_feedback: MemoryPolicyFeedback {
                enabled: true,
                tracked_surfaces: vec!["search".to_string(), "working".to_string()],
                max_items_per_request: 3,
            },
            source_trust_floor: 0.6,
            runtime: MemoryPolicyRuntime {
                live_truth: MemoryPolicyLiveTruth {
                    read_once_sources: true,
                    raw_reopen_requires_change_or_doubt: true,
                    visible_memory_objects: true,
                    compile_from_events: true,
                },
                memory_compilation: MemoryPolicyMemoryCompilation {
                    event_driven_updates: true,
                    patch_not_rewrite: true,
                    preserve_provenance: true,
                    source_on_demand: true,
                },
                semantic_fallback: MemoryPolicySemanticFallback {
                    enabled: true,
                    source_of_truth: false,
                    max_items_per_query: 3,
                    rerank_with_visible_memory: true,
                },
                skill_gating: MemoryPolicySkillGating {
                    propose_from_repeated_patterns: true,
                    sandboxed_evaluation: true,
                    auto_activate_low_risk_only: true,
                    gated_activation: true,
                    require_evaluation: true,
                    require_policy_approval: true,
                },
            },
            promotion: MemoryPolicyPromotion {
                min_salience: 0.22,
                min_events: 3,
                lookback_days: 14,
                default_ttl_days: 90,
            },
            decay: MemoryPolicyDecay {
                max_items: 128,
                inactive_days: 21,
                max_decay: 0.12,
                record_events: true,
            },
            consolidation: MemoryPolicyConsolidation {
                max_groups: 24,
                min_events: 3,
                lookback_days: 14,
                min_salience: 0.22,
                record_events: true,
            },
        };

        let summary = render_skill_policy_summary(&response, true);
        assert!(summary.contains("skill-policy"));
        assert!(summary.contains("propose=on"));
        assert!(summary.contains("sandbox=on"));
        assert!(summary.contains("activate=on"));
        assert!(summary.contains("flow=pattern->proposal->sandbox->eval->policy->activate"));
    }

    #[test]
    fn harness_preset_markdown_includes_registry_metadata() {
        let registry = HarnessPresetRegistry::default_registry();
        let preset = registry.get("codex").expect("codex preset");

        let markdown = render_harness_preset_markdown(preset);
        assert!(markdown.contains("# Codex Harness Pack"));
        assert!(markdown.contains("- pack id: `codex`"));
        assert!(markdown.contains("## Surface Set"));
        assert!(markdown.contains("## Default Verbs"));
        assert!(markdown.contains("## Shared Core"));
    }

    #[test]
    fn status_summary_surfaces_prompt_pressure_warning() {
        let status = json!({
            "bundle": "/tmp/memd",
            "setup_ready": true,
            "server": { "status": "ok" },
            "rag": { "healthy": true },
            "missing": [],
            "defaults": {
                "project": "demo",
                "namespace": "main",
                "session": "codex-a",
                "tab_id": "tab-a",
                "agent": "codex"
            },
            "resume_preview": {
                "project": "demo",
                "namespace": "main",
                "session": "codex-a",
                "tab_id": "tab-a",
                "agent": "codex",
                "context_pressure": "high",
                "estimated_prompt_tokens": 1842,
                "refresh_recommended": true,
                "inbox_items": 4,
                "rehydration_queue": 3,
                "redundant_context_items": 2,
                "semantic_hits": 4,
                "focus": "Trim the live bundle before loading more context",
                "next_recovery": "reopen only the changed files"
            }
        });

        let summary = render_bundle_status_summary(&status);
        assert!(summary.contains("status bundle=/tmp/memd"));
        assert!(summary.contains("project=demo"));
        assert!(summary.contains("namespace=main"));
        assert!(summary.contains("session=codex-a"));
        assert!(summary.contains("tab=tab-a"));
        assert!(summary.contains("agent=codex"));
        assert!(summary.contains("voice=caveman-ultra"));
        assert!(summary.contains("server=ok"));
        assert!(summary.contains("rag=ready"));
        assert!(summary.contains("prompt_pressure=high"));
        assert!(summary.contains("tok=1842"));
        assert!(summary.contains("drivers=duplicates,inbox,refresh,rehydration,semantic,tokens"));
        assert!(summary.contains("action=\"drain inbox before the next prompt\""));
        assert!(summary.contains("warning=\"prompt pressure high\""));
    }

    #[test]
    fn status_summary_ignores_null_resume_preview_and_keeps_defaults_scope() {
        let status = json!({
            "bundle": "/tmp/memd",
            "setup_ready": true,
            "server": { "status": "ok" },
            "rag": { "healthy": true },
            "missing": [],
            "defaults": {
                "project": "demo",
                "namespace": "main",
                "session": "codex-a",
                "tab_id": "tab-a",
                "agent": "codex"
            },
            "resume_preview": null
        });

        let summary = render_bundle_status_summary(&status);
        assert!(summary.contains("project=demo"));
        assert!(summary.contains("namespace=main"));
        assert!(summary.contains("session=codex-a"));
        assert!(summary.contains("tab=tab-a"));
        assert!(summary.contains("agent=codex"));
        assert!(summary.contains("voice=caveman-ultra"));
    }

    #[test]
    fn status_summary_surfaces_live_session_rebind_when_present() {
        let status = json!({
            "bundle": "/tmp/memd",
            "setup_ready": true,
            "server": { "status": "ok" },
            "rag": { "healthy": true },
            "missing": [],
            "defaults": {
                "project": "demo",
                "namespace": "main",
                "session": "codex-fresh",
                "tab_id": "tab-a",
                "agent": "codex"
            },
            "session_overlay": {
                "bundle_session": "codex-stale",
                "live_session": "codex-fresh",
                "rebased_from": "codex-stale"
            },
            "resume_preview": null
        });

        let summary = render_bundle_status_summary(&status);
        assert!(summary.contains("session=codex-fresh"));
        assert!(summary.contains("bundle_session=codex-stale"));
        assert!(summary.contains("live_session=codex-fresh"));
        assert!(summary.contains("rebased_from=codex-stale"));
    }

    #[test]
    fn status_summary_surfaces_truth_first_retrieval_state() {
        let status = json!({
            "bundle": "/tmp/memd",
            "setup_ready": true,
            "server": { "status": "ok" },
            "rag": { "healthy": true },
            "missing": [],
            "defaults": {
                "project": "demo",
                "namespace": "main",
                "session": "codex-a",
                "tab_id": "tab-a",
                "agent": "codex"
            },
            "resume_preview": null,
            "truth_summary": {
                "truth": "current",
                "freshness": "fresh",
                "retrieval_tier": "hot",
                "confidence": 0.97,
                "action_hint": "use the event spine first",
                "source_count": 3,
                "contested_sources": 1,
                "records": [
                    {
                        "lane": "live_truth",
                        "preview": "event spine compact summary"
                    }
                ]
            }
        });

        let summary = render_bundle_status_summary(&status);
        assert!(summary.contains("truth=current"));
        assert!(summary.contains("freshness=fresh"));
        assert!(summary.contains("retrieval=hot"));
        assert!(summary.contains("conf=0.97"));
        assert!(summary.contains("sources=3"));
        assert!(summary.contains("contested=1"));
        assert!(summary.contains("truth_action=\"use the event spine first\""));
        assert!(summary.contains("truth_head=live_truth"));
        assert!(summary.contains("truth_preview=\"event spine compact summary\""));
    }

    #[test]
    fn status_summary_surfaces_capability_surface_counts() {
        let status = json!({
            "bundle": "/tmp/memd",
            "setup_ready": true,
            "server": { "status": "ok" },
            "rag": { "healthy": true },
            "missing": [],
            "defaults": {
                "project": "demo",
                "namespace": "main",
                "session": "codex-a",
                "tab_id": "tab-a",
                "agent": "codex"
            },
            "resume_preview": null,
            "capability_surface": {
                "discovered": 12,
                "universal": 4,
                "bridgeable": 3,
                "harness_native": 5
            }
        });

        let summary = render_bundle_status_summary(&status);
        assert!(summary.contains("capabilities=12"));
        assert!(summary.contains("universal=4"));
        assert!(summary.contains("bridgeable=3"));
        assert!(summary.contains("harness_native=5"));
    }

    #[test]
    fn status_summary_surfaces_cowork_counts() {
        let status = json!({
            "bundle": "/tmp/memd",
            "setup_ready": true,
            "server": { "status": "ok" },
            "rag": { "healthy": true },
            "missing": [],
            "defaults": {
                "project": "demo",
                "namespace": "main",
                "session": "codex-a",
                "tab_id": "tab-a",
                "agent": "codex"
            },
            "resume_preview": null,
            "cowork_surface": {
                "tasks": 4,
                "open_tasks": 3,
                "help_tasks": 1,
                "review_tasks": 2,
                "exclusive_tasks": 2,
                "shared_tasks": 2,
                "inbox_messages": 3,
                "owned_tasks": 1
            }
        });

        let summary = render_bundle_status_summary(&status);
        assert!(summary.contains("cowork_tasks=4"));
        assert!(summary.contains("open=3"));
        assert!(summary.contains("help=1"));
        assert!(summary.contains("review=2"));
        assert!(summary.contains("exclusive=2"));
        assert!(summary.contains("shared=2"));
        assert!(summary.contains("inbox_messages=3"));
        assert!(summary.contains("owned=1"));
    }

    #[test]
    fn status_summary_surfaces_maintenance_state() {
        let status = json!({
            "bundle": "/tmp/memd",
            "setup_ready": true,
            "server": { "status": "ok" },
            "rag": { "healthy": true },
            "missing": [],
            "defaults": {
                "project": "demo",
                "namespace": "main",
                "session": "codex-a",
                "tab_id": "tab-a",
                "agent": "codex"
            },
            "resume_preview": null,
            "maintenance_surface": {
                "mode": "auto",
                "auto_mode": true,
                "receipt": "r-123",
                "compacted": 3,
                "refreshed": 1,
                "repaired": 0,
                "findings": 2,
                "total_actions": 4,
                "delta_total_actions": 2,
                "trend": "up"
            }
        });

        let summary = render_bundle_status_summary(&status);
        assert!(summary.contains("maintain_mode=auto"));
        assert!(summary.contains("auto=yes"));
        assert!(summary.contains("maintain_receipt=r-123"));
        assert!(summary.contains("compacted=3"));
        assert!(summary.contains("refreshed=1"));
        assert!(summary.contains("repaired=0"));
        assert!(summary.contains("findings=2"));
        assert!(summary.contains("maintain_total=4"));
        assert!(summary.contains("maintain_delta=2"));
        assert!(summary.contains("maintain_trend=up"));
    }

    #[test]
    fn visible_memory_home_summary_surfaces_artifact_and_pressure() {
        let snapshot = VisibleMemorySnapshotResponse {
            generated_at: chrono::Utc::now(),
            home: VisibleMemoryHome {
                focus_artifact: VisibleMemoryArtifact {
                    id: uuid::Uuid::new_v4(),
                    title: "runtime spine".to_string(),
                    body: "runtime spine is the canonical memory contract".to_string(),
                    artifact_kind: "compiled_page".to_string(),
                    memory_kind: Some(MemoryKind::Decision),
                    scope: Some(MemoryScope::Project),
                    visibility: Some(MemoryVisibility::Workspace),
                    workspace: Some("team-alpha".to_string()),
                    status: VisibleMemoryStatus::Current,
                    freshness: "verified".to_string(),
                    confidence: 0.94,
                    provenance: VisibleMemoryProvenance {
                        source_system: Some("obsidian".to_string()),
                        source_path: Some("wiki/runtime-spine.md".to_string()),
                        producer: Some("obsidian compile".to_string()),
                        trust_reason: "verified workspace page".to_string(),
                        last_verified_at: None,
                    },
                    sources: vec!["wiki/runtime-spine.md".to_string()],
                    linked_artifact_ids: vec![uuid::Uuid::new_v4()],
                    linked_sessions: vec!["codex-01".to_string()],
                    linked_agents: vec!["codex".to_string()],
                    repair_state: "healthy".to_string(),
                    actions: vec![
                        "inspect".to_string(),
                        "explain".to_string(),
                        "verify_current".to_string(),
                    ],
                },
                inbox_count: 3,
                repair_count: 1,
                awareness_count: 2,
            },
            knowledge_map: VisibleMemoryKnowledgeMap {
                nodes: vec![
                    VisibleMemoryGraphNode {
                        artifact_id: uuid::Uuid::new_v4(),
                        title: "runtime spine".to_string(),
                        artifact_kind: "compiled_page".to_string(),
                        status: VisibleMemoryStatus::Current,
                    },
                    VisibleMemoryGraphNode {
                        artifact_id: uuid::Uuid::new_v4(),
                        title: "workspace lane".to_string(),
                        artifact_kind: "workspace_lane".to_string(),
                        status: VisibleMemoryStatus::Candidate,
                    },
                ],
                edges: vec![VisibleMemoryGraphEdge {
                    from: uuid::Uuid::new_v4(),
                    to: uuid::Uuid::new_v4(),
                    relation: "related".to_string(),
                }],
            },
        };

        let summary = render_visible_memory_home(&snapshot, true);
        assert!(summary.contains("memory_home focus=runtime spine"));
        assert!(summary.contains("status=current"));
        assert!(summary.contains("freshness=verified"));
        assert!(summary.contains("visibility=workspace"));
        assert!(summary.contains("workspace=team-alpha"));
        assert!(summary.contains("inbox=3"));
        assert!(summary.contains("repair=1"));
        assert!(summary.contains("awareness=2"));
        assert!(summary.contains("nodes=2"));
        assert!(summary.contains("edges=1"));
        assert!(summary.contains("source_path=wiki/runtime-spine.md"));
        assert!(summary.contains("actions=inspect|explain|verify_current"));
        assert!(summary.contains("trail=runtime spine:compiled_page:current"));
    }
}
