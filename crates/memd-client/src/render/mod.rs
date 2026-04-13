use serde::Serialize;
use serde_json::Value;

use memd_schema::{
    AgentProfileResponse, AssociativeRecallResponse, EntitySearchResponse, ExplainMemoryResponse,
    MemoryPolicyResponse, RepairMemoryResponse, RetrievalIntent, RetrievalRoute,
    SourceMemoryResponse, VisibleMemoryArtifactDetailResponse, VisibleMemoryKnowledgeMap,
    VisibleMemorySnapshotResponse, VisibleMemoryStatus, VisibleMemoryUiActionKind,
    WorkingMemoryResponse, WorkspaceMemoryResponse,
};

use crate::cli::command_catalog::{CommandCatalog, CommandCatalogEntry};
use crate::cli::skill_catalog::{SkillCatalog, SkillCatalogEntry};
use crate::{
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

mod render_catalog;
pub(crate) use render_catalog::*;

mod render_summary;
pub(crate) use render_summary::*;

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
            if drivers.contains(&"inbox") {
                "drain inbox before the next prompt"
            } else if drivers.contains(&"rehydration") {
                "resolve rehydration backlog before the next prompt"
            } else if drivers.contains(&"duplicates") {
                "collapse repeated context before the next prompt"
            } else {
                "trim context before the next prompt"
            }
        } else if pressure == "medium" {
            if drivers.contains(&"inbox") {
                "handle inbox items before pulling more context"
            } else if drivers.contains(&"rehydration") {
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
        let epistemic_state = truth
            .get("epistemic_state")
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
            " truth={} epistemic={} freshness={} retrieval={} conf={:.2} sources={} contested={}",
            truth_state, epistemic_state, freshness, tier, confidence, sources, contested
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

mod render_memory;
pub(crate) use render_memory::*;
