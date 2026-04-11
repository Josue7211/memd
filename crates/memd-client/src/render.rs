use serde::Serialize;
use serde_json::Value;

use memd_schema::{
    AgentProfileResponse, AssociativeRecallResponse, EntitySearchResponse, ExplainMemoryResponse,
    MemoryPolicyResponse, RepairMemoryResponse, RetrievalIntent, RetrievalRoute,
    SourceMemoryResponse, VisibleMemoryArtifactDetailResponse, VisibleMemoryKnowledgeMap,
    VisibleMemorySnapshotResponse, VisibleMemoryStatus, VisibleMemoryUiActionKind,
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

#[path = "render_evaluation.rs"]
mod render_evaluation;
pub(crate) use render_evaluation::*;

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

#[allow(dead_code)]
pub(crate) fn render_codex_harness_pack_markdown(pack: &CodexHarnessPack) -> String {
    render_harness_pack_markdown(pack)
}

#[allow(dead_code)]
pub(crate) fn render_claude_code_harness_pack_markdown(pack: &ClaudeCodeHarnessPack) -> String {
    render_harness_pack_markdown(pack)
}

#[allow(dead_code)]
pub(crate) fn render_agent_zero_harness_pack_markdown(pack: &AgentZeroHarnessPack) -> String {
    render_harness_pack_markdown(pack)
}

#[allow(dead_code)]
pub(crate) fn render_openclaw_harness_pack_markdown(pack: &OpenClawHarnessPack) -> String {
    render_harness_pack_markdown(pack)
}

#[allow(dead_code)]
pub(crate) fn render_hermes_harness_pack_markdown(pack: &HermesHarnessPack) -> String {
    render_harness_pack_markdown(pack)
}

#[allow(dead_code)]
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
    let voice_mode = defaults
        .and_then(|value| value.get("voice_mode").and_then(Value::as_str))
        .unwrap_or("caveman-ultra");
    output.push_str(&format!(
        " project={} namespace={} session={} tab={} agent={} voice={}",
        project, namespace, session, tab_id, agent, voice_mode
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
    if let Some(lane) = status
        .get("lane_surface")
        .and_then(|value| if value.is_null() { None } else { Some(value) })
    {
        output.push_str(&format!(
            " lane_action={} lane_previous_branch={} lane_current_branch={} lane_conflict_session={}",
            lane.get("action").and_then(Value::as_str).unwrap_or("none"),
            lane.get("previous_branch")
                .and_then(Value::as_str)
                .unwrap_or("none"),
            lane.get("current_branch")
                .and_then(Value::as_str)
                .unwrap_or("none"),
            lane.get("conflict_session")
                .and_then(Value::as_str)
                .unwrap_or("none"),
        ));
    }
    if let Some(lane_fault) = status
        .get("lane_fault")
        .and_then(|value| if value.is_null() { None } else { Some(value) })
    {
        output.push_str(&format!(
            " lane_fault={} lane_fault_session={}",
            lane_fault
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("none"),
            lane_fault
                .get("session")
                .and_then(Value::as_str)
                .unwrap_or("none"),
        ));
    }
    if let Some(lane_receipts) = status
        .get("lane_receipts")
        .and_then(|value| if value.is_null() { None } else { Some(value) })
    {
        output.push_str(&format!(
            " lane_receipts={} lane_latest_receipt={}",
            lane_receipts
                .get("count")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            lane_receipts
                .get("latest_kind")
                .and_then(Value::as_str)
                .unwrap_or("none"),
        ));
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
        if let Some(views) = cowork.get("views") {
            output.push_str(&format!(
                " cowork_views=owned:{}|open:{}|help:{}|review:{}|exclusive:{}|shared:{}",
                views.get("owned").and_then(Value::as_u64).unwrap_or(0),
                views.get("open").and_then(Value::as_u64).unwrap_or(0),
                views.get("help").and_then(Value::as_u64).unwrap_or(0),
                views.get("review").and_then(Value::as_u64).unwrap_or(0),
                views.get("exclusive").and_then(Value::as_u64).unwrap_or(0),
                views.get("shared").and_then(Value::as_u64).unwrap_or(0),
            ));
        }
    }

    if let Some(maintenance) = status
        .get("maintenance_surface")
        .and_then(|value| if value.is_null() { None } else { Some(value) })
    {
        output.push_str(&format!(
            " maintain_mode={} auto={} auto_recommended={} auto_reason={} maintain_receipt={} compacted={} refreshed={} repaired={} findings={} maintain_total={} maintain_delta={} maintain_trend={} history={}",
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
            if maintenance
                .get("auto_recommended")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                "yes"
            } else {
                "no"
            },
            maintenance
                .get("auto_reason")
                .and_then(Value::as_str)
                .unwrap_or("none"),
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
            maintenance
                .get("history_count")
                .and_then(Value::as_u64)
                .unwrap_or(0),
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

#[path = "render_memory.rs"]
mod render_memory;
pub(crate) use render_memory::*;

