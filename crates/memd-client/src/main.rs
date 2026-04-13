#![allow(unused_imports)]

mod awareness;
mod benchmark;
mod bundle;
mod cli;
mod compiled;
mod coordination;
mod evaluation;
pub(crate) mod harness;
mod hive;
mod obsidian;
mod render;
mod runtime;
mod verification;
mod workflow;

#[cfg(test)]
pub(crate) mod test_support {
    use std::{
        collections::BTreeMap,
        ffi::{OsStr, OsString},
        path::{Path, PathBuf},
        sync::{Mutex, MutexGuard, OnceLock},
    };

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    static CWD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    pub(crate) fn lock_env_mutation() -> MutexGuard<'static, ()> {
        ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

    pub(crate) fn lock_cwd_mutation() -> MutexGuard<'static, ()> {
        CWD_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

    pub(crate) struct EnvScope {
        _lock: MutexGuard<'static, ()>,
        originals: BTreeMap<&'static str, Option<OsString>>,
    }

    impl EnvScope {
        pub(crate) fn new() -> Self {
            Self {
                _lock: lock_env_mutation(),
                originals: BTreeMap::new(),
            }
        }

        fn remember(&mut self, name: &'static str) {
            self.originals
                .entry(name)
                .or_insert_with(|| std::env::var_os(name));
        }

        pub(crate) fn set(&mut self, name: &'static str, value: impl AsRef<OsStr>) {
            self.remember(name);
            unsafe {
                std::env::set_var(name, value);
            }
        }

        pub(crate) fn remove(&mut self, name: &'static str) {
            self.remember(name);
            unsafe {
                std::env::remove_var(name);
            }
        }
    }

    impl Drop for EnvScope {
        fn drop(&mut self) {
            for (name, original) in self.originals.iter().rev() {
                unsafe {
                    match original {
                        Some(value) => std::env::set_var(name, value),
                        None => std::env::remove_var(name),
                    }
                }
            }
        }
    }

    pub(crate) struct CwdGuard {
        _lock: MutexGuard<'static, ()>,
        original: PathBuf,
    }

    impl Drop for CwdGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original);
        }
    }

    pub(crate) fn set_current_dir(path: impl AsRef<Path>) -> CwdGuard {
        let lock = lock_cwd_mutation();
        let original = std::env::current_dir().expect("read current dir");
        std::env::set_current_dir(path).expect("set current dir");
        CwdGuard {
            _lock: lock,
            original,
        }
    }
}

#[cfg(test)]
mod e2e_tests;
#[cfg(test)]
mod main_tests;

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    future::Future,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use crate::harness::cache;
use anyhow::{Context, anyhow};
pub(crate) use awareness::{derive_awareness_worker_name, project_awareness_entry_to_hive_session};
pub(crate) use benchmark::*;
pub(crate) use bundle::agent_profiles::copy_hook_assets;
pub(crate) use bundle::*;
use chrono::{DateTime, Utc};
use clap::{CommandFactory, Parser};
pub(crate) use cli::command_catalog::build_command_catalog;
pub(crate) use cli::skill_catalog::{build_skill_catalog, find_skill_catalog_matches};
#[allow(unused_imports)]
pub(crate) use cli::*;
use cli::{
    normalize_voice_mode_value, parse_memory_kind_value, parse_memory_scope_value,
    parse_memory_visibility_value, parse_retrieval_intent, parse_retrieval_route,
    parse_source_quality_value, parse_uuid_list,
};
pub(crate) use compiled::*;
pub(crate) use coordination::*;
pub(crate) use evaluation::*;
use hive::*;
use memd_client::MemdClient;
use memd_core::{
    BuildCompactionPacketArgs, build_compaction_packet, derive_compaction_spill,
    derive_compaction_spill_with_options, render_compaction_wire,
};
use memd_rag::{
    RagClient, RagIngestRequest, RagIngestSource, RagRetrieveMode, RagRetrieveRequest,
    RagRetrieveResponse,
};
use memd_schema::{
    AgentProfileRequest, AgentProfileUpsertRequest, AssociativeRecallRequest,
    BenchmarkEvidenceSummary, BenchmarkGateDecision, BenchmarkRegistry, BenchmarkSubjectMetrics,
    CandidateMemoryRequest, CompactionDecision, CompactionOpenLoop, CompactionPacket,
    CompactionReference, CompactionSession, CompactionSpillOptions, CompactionSpillResult,
    ContextRequest, ContinuityJourneyReport, EntityLinkRequest, EntityLinksRequest,
    EntitySearchRequest, ExpireMemoryRequest, ExplainMemoryRequest, FixtureRecord,
    HiveBoardRequest, HiveBoardResponse, HiveClaimRecoverRequest, HiveClaimsRequest,
    HiveCoordinationInboxRequest, HiveCoordinationInboxResponse, HiveCoordinationReceiptRecord,
    HiveCoordinationReceiptRequest, HiveCoordinationReceiptsRequest, HiveFollowRequest,
    HiveFollowResponse, HiveHandoffPacket, HiveMessageAckRequest, HiveMessageInboxRequest,
    HiveMessageRecord, HiveMessageSendRequest, HiveRosterRequest, HiveRosterResponse,
    HiveSessionAutoRetireRequest, HiveTaskAssignRequest, HiveTaskRecord, HiveTaskUpsertRequest,
    HiveTasksRequest, MaintainReport, MemoryConsolidationRequest, MemoryInboxRequest, MemoryKind,
    MemoryMaintenanceReportRequest, MemoryPolicyResponse, MemoryRepairMode, MemoryScope,
    MemoryStage, MemoryStatus, PromoteMemoryRequest, RepairMemoryRequest, RetrievalIntent,
    RetrievalRoute, SearchMemoryRequest, SkillPolicyActivationEntriesRequest,
    SkillPolicyActivationEntriesResponse, SkillPolicyActivationRecord,
    SkillPolicyApplyReceiptsRequest, SkillPolicyApplyReceiptsResponse, SkillPolicyApplyRequest,
    SourceMemoryRequest, StoreMemoryRequest, VerifierRecord, VerifyMemoryRequest,
    WorkingMemoryRequest,
};
use memd_sidecar::SidecarClient;
use notify::{Config as NotifyConfig, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use obsidian::ObsidianImportPreview;
use obsidian::commands::{
    run_obsidian_compile, run_obsidian_import, run_obsidian_open, run_obsidian_status,
    run_obsidian_watch,
};
use obsidian::runtime::{run_obsidian_handoff, run_obsidian_writeback};
#[allow(unused_imports)]
pub(crate) use obsidian::support::*;
use render::{
    is_default_runtime, render_bundle_status_summary, render_command_catalog_json,
    render_command_catalog_markdown, render_command_catalog_summary, render_composite_markdown,
    render_composite_summary, render_consolidate_summary, render_entity_search_summary,
    render_entity_summary, render_eval_summary, render_experiment_markdown,
    render_experiment_summary, render_explain_summary, render_feature_benchmark_markdown,
    render_feature_benchmark_summary, render_gap_summary, render_handoff_prompt,
    render_harness_pack_index_json, render_harness_pack_index_markdown,
    render_harness_pack_index_summary, render_improvement_markdown, render_improvement_summary,
    render_maintenance_report_summary, render_obsidian_scan_summary, render_policy_summary,
    render_profile_summary, render_recall_summary, render_repair_summary, render_resume_prompt,
    render_scenario_markdown, render_scenario_summary, render_skill_catalog_markdown,
    render_skill_catalog_match_markdown, render_skill_catalog_match_summary,
    render_skill_catalog_summary, render_skill_policy_summary, render_source_summary,
    render_timeline_summary, render_visible_memory_artifact_detail, render_visible_memory_home,
    render_visible_memory_knowledge_map, render_working_summary, render_workspace_summary,
    short_uuid,
};
pub(crate) use runtime::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_json::json;
use sha2::{Digest, Sha256};
use tempfile::TempDir;
#[cfg(test)]
pub(crate) use test_support::*;
use tokio::task::JoinSet;
pub(crate) use verification::*;
#[allow(unused_imports)]
pub(crate) use workflow::*;

fn read_request<T>(input: &RequestInput) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let json = if let Some(json) = &input.json {
        json.clone()
    } else if let Some(path) = &input.input {
        fs::read_to_string(path).with_context(|| format!("read request file {}", path.display()))?
    } else if input.stdin {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("read request from stdin")?;
        buffer
    } else {
        anyhow::bail!("provide --json, --input, or --stdin");
    };

    serde_json::from_str(&json).context("parse request json")
}

fn print_json<T>(value: &T) -> anyhow::Result<()>
where
    T: serde::Serialize,
{
    let json = serde_json::to_string_pretty(value).context("serialize response json")?;
    println!("{json}");
    Ok(())
}

fn obsidian_path_is_internal(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component,
            std::path::Component::Normal(name)
                if name == ".memd" || name == ".obsidian" || name == ".git"
        )
    })
}

fn workspace_path_is_internal(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component,
            std::path::Component::Normal(name)
                if name == ".memd"
                    || name == ".git"
                    || name == "target"
                    || name == "node_modules"
                    || name == "watch.out"
                    || name == "memd-watch.log"
                    || name == "memd-watch.err"
        )
    })
}

fn workspace_path_should_trigger(path: &Path) -> bool {
    if workspace_path_is_internal(path) {
        return false;
    }

    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if matches!(
        file_name,
        "Cargo.toml"
            | "Cargo.lock"
            | "Makefile"
            | "Dockerfile"
            | "README"
            | "README.md"
            | "AGENTS.md"
            | "CLAUDE.md"
            | "ROADMAP.md"
            | "DESIGN.md"
            | "CONTRIBUTING.md"
            | "CHANGELOG.md"
    ) {
        return true;
    }

    match path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
    {
        Some(ext)
            if matches!(
                ext.as_str(),
                "rs" | "toml"
                    | "md"
                    | "sh"
                    | "ps1"
                    | "json"
                    | "yml"
                    | "yaml"
                    | "js"
                    | "ts"
                    | "tsx"
                    | "py"
                    | "go"
                    | "c"
                    | "h"
                    | "cpp"
                    | "css"
                    | "html"
                    | "txt"
                    | "lock"
            ) =>
        {
            true
        }
        _ => false,
    }
}

fn count_obsidian_mirrors(vault: &Path, kind: &str) -> anyhow::Result<usize> {
    let root = vault.join(".memd").join("writeback").join(kind);
    if !root.exists() {
        return Ok(0);
    }
    let mut count = 0usize;
    for entry in memdrive::WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if entry.file_type().is_file() {
            count += 1;
        }
    }
    Ok(count)
}

fn resolve_pack_bundle_root(explicit: Option<&Path>) -> anyhow::Result<PathBuf> {
    if let Some(explicit) = explicit {
        return Ok(explicit.to_path_buf());
    }

    Ok(resolve_default_bundle_root()?.unwrap_or_else(default_bundle_root_path))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::run_cli(Cli::parse()).await
}
