use super::*;
use clap::{Args, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum DevServerSubcommand {
    Guard(DevServerGuardArgs),
    List(DevServerListArgs),
    Release(DevServerReleaseArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct DevServerGuardArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value = "127.0.0.1")]
    pub(crate) host: String,

    #[arg(long)]
    pub(crate) port: u16,

    #[arg(long, default_value_t = 21600)]
    pub(crate) ttl_secs: u64,

    #[arg(long, default_value_t = 120)]
    pub(crate) stale_after_secs: u64,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(trailing_var_arg = true)]
    pub(crate) command: Vec<String>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct DevServerListArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct DevServerReleaseArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value = "127.0.0.1")]
    pub(crate) host: String,

    #[arg(long)]
    pub(crate) port: u16,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ClaimArgs {
    #[command(subcommand)]
    pub(crate) command: ClaimSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum ClaimSubcommand {
    Create(ClaimCreateArgs),
    List(ClaimListArgs),
    Close(ClaimCloseArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ClaimCreateArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) scope: String,

    #[arg(long, default_value_t = 900)]
    pub(crate) ttl_secs: u64,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ClaimListArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ClaimCloseArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) scope: String,

    #[arg(long)]
    pub(crate) summary: bool,
}

impl From<ClaimCreateArgs> for ClaimsArgs {
    fn from(value: ClaimCreateArgs) -> Self {
        Self {
            output: value.output,
            acquire: true,
            release: false,
            transfer_to_session: None,
            scope: Some(value.scope),
            ttl_secs: value.ttl_secs,
            summary: value.summary,
        }
    }
}

impl From<ClaimListArgs> for ClaimsArgs {
    fn from(value: ClaimListArgs) -> Self {
        Self {
            output: value.output,
            acquire: false,
            release: false,
            transfer_to_session: None,
            scope: None,
            ttl_secs: 900,
            summary: value.summary,
        }
    }
}

impl From<ClaimCloseArgs> for ClaimsArgs {
    fn from(value: ClaimCloseArgs) -> Self {
        Self {
            output: value.output,
            acquire: false,
            release: true,
            transfer_to_session: None,
            scope: Some(value.scope),
            ttl_secs: 900,
            summary: value.summary,
        }
    }
}

#[derive(Debug, Clone, Args)]
pub(crate) struct MessagesArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) send: bool,

    #[arg(long)]
    pub(crate) inbox: bool,

    #[arg(long)]
    pub(crate) ack: Option<String>,

    #[arg(long)]
    pub(crate) target_session: Option<String>,

    #[arg(long)]
    pub(crate) kind: Option<String>,

    #[arg(long)]
    pub(crate) request_help: bool,

    #[arg(long)]
    pub(crate) request_review: bool,

    #[arg(long)]
    pub(crate) assign_scope: Option<String>,

    #[arg(long)]
    pub(crate) scope: Option<String>,

    #[arg(long)]
    pub(crate) content: Option<String>,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct TasksArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) upsert: bool,

    #[arg(long)]
    pub(crate) assign_to_session: Option<String>,

    #[arg(long)]
    pub(crate) target_session: Option<String>,

    #[arg(long)]
    pub(crate) task_id: Option<String>,

    #[arg(long)]
    pub(crate) title: Option<String>,

    #[arg(long)]
    pub(crate) description: Option<String>,

    #[arg(long)]
    pub(crate) status: Option<String>,

    #[arg(long)]
    pub(crate) mode: Option<String>,

    #[arg(long, value_name = "SCOPE")]
    pub(crate) scope: Vec<String>,

    #[arg(long)]
    pub(crate) request_help: bool,

    #[arg(long)]
    pub(crate) request_review: bool,

    #[arg(long)]
    pub(crate) all: bool,

    #[arg(long)]
    pub(crate) view: Option<String>,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct CoordinationArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) view: Option<String>,

    #[arg(long)]
    pub(crate) changes_only: bool,

    #[arg(long)]
    pub(crate) watch: bool,

    #[arg(long, default_value_t = 30)]
    pub(crate) interval_secs: u64,

    #[arg(long)]
    pub(crate) recover_session: Option<String>,

    #[arg(long)]
    pub(crate) retire_session: Option<String>,

    #[arg(long)]
    pub(crate) to_session: Option<String>,

    #[arg(long)]
    pub(crate) deny_session: Option<String>,

    #[arg(long)]
    pub(crate) reroute_session: Option<String>,

    #[arg(long)]
    pub(crate) handoff_scope: Option<String>,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct BundleArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) hive_system: Option<String>,

    #[arg(long)]
    pub(crate) hive_role: Option<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) capability: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) hive_group: Vec<String>,

    #[arg(long)]
    pub(crate) hive_group_goal: Option<String>,

    #[arg(long)]
    pub(crate) authority: Option<String>,

    #[arg(long)]
    pub(crate) base_url: Option<String>,

    #[arg(long)]
    pub(crate) route: Option<String>,

    #[arg(long)]
    pub(crate) intent: Option<String>,

    #[arg(long)]
    pub(crate) tab_id: Option<String>,

    #[arg(long)]
    pub(crate) auto_short_term_capture: Option<bool>,

    #[arg(long)]
    pub(crate) voice_mode: Option<String>,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HiveArgs {
    #[command(subcommand)]
    pub(crate) command: Option<HiveSubcommand>,

    #[arg(long)]
    pub(crate) agent: Option<String>,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long, default_value_t = false)]
    pub(crate) global: bool,

    #[arg(long)]
    pub(crate) project_root: Option<PathBuf>,

    #[arg(long, default_value_t = true)]
    pub(crate) seed_existing: bool,

    #[arg(long)]
    pub(crate) session: Option<String>,

    #[arg(long)]
    pub(crate) tab_id: Option<String>,

    #[arg(long)]
    pub(crate) hive_system: Option<String>,

    #[arg(long)]
    pub(crate) hive_role: Option<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) capability: Vec<String>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) hive_group: Vec<String>,

    #[arg(long)]
    pub(crate) hive_group_goal: Option<String>,

    #[arg(long)]
    pub(crate) authority: Option<String>,

    #[arg(long, default_value_os_t = default_init_output_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = default_base_url())]
    pub(crate) base_url: String,

    #[arg(long)]
    pub(crate) rag_url: Option<String>,

    #[arg(long, default_value = "auto")]
    pub(crate) route: String,

    #[arg(long, default_value = "current_task")]
    pub(crate) intent: String,

    #[arg(long)]
    pub(crate) workspace: Option<String>,

    #[arg(long)]
    pub(crate) visibility: Option<String>,

    #[arg(long, default_value_t = true)]
    pub(crate) publish_heartbeat: bool,

    #[arg(long)]
    pub(crate) force: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum HiveSubcommand {
    Roster(HiveRosterArgs),
    Follow(HiveFollowArgs),
    Handoff(HiveHandoffArgs),
    Cowork {
        #[command(subcommand)]
        command: HiveCoworkSubcommand,
    },
    Queen(HiveQueenArgs),
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum HiveCoworkSubcommand {
    Request(HiveCoworkArgs),
    Ack(HiveCoworkArgs),
    Decline(HiveCoworkArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HiveRosterArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) json: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HiveFollowArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) session: Option<String>,

    #[arg(long)]
    pub(crate) worker: Option<String>,

    #[arg(long)]
    pub(crate) watch: bool,

    #[arg(long, default_value_t = 5)]
    pub(crate) interval_secs: u64,

    #[arg(long)]
    pub(crate) json: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HiveHandoffArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) to_session: Option<String>,

    #[arg(long)]
    pub(crate) to_worker: Option<String>,

    #[arg(long)]
    pub(crate) task_id: Option<String>,

    #[arg(long)]
    pub(crate) topic: Option<String>,

    #[arg(long, value_delimiter = ',')]
    pub(crate) scope: Vec<String>,

    #[arg(long)]
    pub(crate) next_action: Option<String>,

    #[arg(long)]
    pub(crate) blocker: Option<String>,

    #[arg(long)]
    pub(crate) note: Option<String>,

    #[arg(long)]
    pub(crate) allow_ephemeral: bool,

    #[arg(long)]
    pub(crate) json: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HiveCoworkArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) to_session: Option<String>,

    #[arg(long)]
    pub(crate) to_worker: Option<String>,

    #[arg(long)]
    pub(crate) task_id: Option<String>,

    #[arg(long, value_delimiter = ',')]
    pub(crate) scope: Vec<String>,

    #[arg(long)]
    pub(crate) reason: Option<String>,

    #[arg(long)]
    pub(crate) note: Option<String>,

    #[arg(long)]
    pub(crate) json: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HiveQueenArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) view: Option<String>,

    #[arg(long)]
    pub(crate) recover_session: Option<String>,

    #[arg(long)]
    pub(crate) retire_session: Option<String>,

    #[arg(long)]
    pub(crate) to_session: Option<String>,

    #[arg(long)]
    pub(crate) deny_session: Option<String>,

    #[arg(long)]
    pub(crate) reroute_session: Option<String>,

    #[arg(long)]
    pub(crate) handoff_scope: Option<String>,

    #[arg(long)]
    pub(crate) json: bool,

    #[arg(long)]
    pub(crate) summary: bool,

    #[arg(long)]
    pub(crate) cowork_auto_send: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HiveJoinArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = default_hive_join_base_url())]
    pub(crate) base_url: String,

    #[arg(long)]
    pub(crate) all_active: bool,

    #[arg(long)]
    pub(crate) all_local: bool,

    #[arg(long, default_value_t = true)]
    pub(crate) publish_heartbeat: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct HiveProjectArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) enable: bool,

    #[arg(long)]
    pub(crate) disable: bool,

    #[arg(long)]
    pub(crate) status: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct EvalArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) rehydration_limit: Option<usize>,

    #[arg(long)]
    pub(crate) write: bool,

    #[arg(long)]
    pub(crate) fail_below: Option<u8>,

    #[arg(long)]
    pub(crate) fail_on_regression: bool,

    #[arg(long)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct GapArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) recent_commits: Option<usize>,

    #[arg(long, default_value_t = false)]
    pub(crate) write: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ImproveArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long, default_value_t = 3)]
    pub(crate) max_iterations: usize,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) recent_commits: Option<usize>,

    #[arg(long, default_value_t = false)]
    pub(crate) write: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) apply: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ScenarioArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) scenario: Option<String>,

    #[arg(long, default_value_t = false)]
    pub(crate) write: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct CompositeArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) scenario: Option<String>,

    #[arg(long, default_value_t = false)]
    pub(crate) write: bool,

    #[arg(long, default_value_t = false)]
    pub(crate) summary: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct TeachArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) workspace: Option<String>,

    #[arg(long)]
    pub(crate) visibility: Option<String>,

    #[arg(long, default_value = "fact")]
    pub(crate) kind: String,

    #[arg(long)]
    pub(crate) source_agent: Option<String>,

    #[arg(long)]
    pub(crate) confidence: Option<f32>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) tag: Vec<String>,

    #[arg(long, value_name = "UUID")]
    pub(crate) supersede: Vec<String>,

    #[arg(long)]
    pub(crate) content: Option<String>,

    #[arg(long)]
    pub(crate) input: Option<PathBuf>,

    #[arg(long)]
    pub(crate) stdin: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct CheckpointArgs {
    #[arg(long, default_value_os_t = default_bundle_root_path())]
    pub(crate) output: PathBuf,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) workspace: Option<String>,

    #[arg(long)]
    pub(crate) visibility: Option<String>,

    #[arg(long)]
    pub(crate) source_path: Option<String>,

    #[arg(long)]
    pub(crate) confidence: Option<f32>,

    #[arg(long)]
    pub(crate) ttl_seconds: Option<u64>,

    #[arg(long, value_name = "TEXT")]
    pub(crate) tag: Vec<String>,

    #[arg(long)]
    pub(crate) content: Option<String>,

    #[arg(long)]
    pub(crate) input: Option<PathBuf>,

    #[arg(long)]
    pub(crate) stdin: bool,

    /// Auto-commit a small tracked dirty set before checkpointing.
    /// Refuses broad dirty trees so handoffs do not sweep unrelated work.
    #[arg(long)]
    pub(crate) auto_commit: bool,

    /// Update ROADMAP_STATE key-value pairs before checkpointing.
    /// Format: KEY=VALUE (e.g. --roadmap-set current_phase=P2 --roadmap-set phase_status=in_progress).
    /// Applied before auto-commit so changes are included in the commit.
    #[arg(long, value_name = "KEY=VALUE")]
    pub(crate) roadmap_set: Vec<String>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RagArgs {
    #[arg(long)]
    pub(crate) rag_url: Option<String>,

    #[command(subcommand)]
    pub(crate) mode: RagMode,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct EmbedArgs {
    #[command(subcommand)]
    pub(crate) mode: EmbedMode,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum EmbedMode {
    Models(EmbedModelsArgs),
    Bench(EmbedBenchArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct EmbedModelsArgs {
    #[arg(long)]
    pub(crate) target: Option<String>,

    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct EmbedBenchArgs {
    /// JSON bench file with qrels/model scores.
    #[arg(long)]
    pub(crate) input: PathBuf,

    /// Optional target bucket: cloud, local, or hybrid.
    #[arg(long)]
    pub(crate) target: Option<String>,

    /// Optional sidecar URL for live retrieve/rerank scoring.
    #[arg(long)]
    pub(crate) rag_url: Option<String>,

    /// Default project for live sidecar retrieve qrels.
    #[arg(long)]
    pub(crate) project: Option<String>,

    /// Default namespace for live sidecar retrieve qrels.
    #[arg(long)]
    pub(crate) namespace: Option<String>,

    /// Live retrieve limit when --rag-url is set.
    #[arg(long, default_value_t = 10)]
    pub(crate) limit: usize,

    /// Print full JSON report.
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum RagMode {
    Healthz,
    Status,
    Sync(RagSyncArgs),
    Search(RagSearchArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct MultimodalArgs {
    #[arg(long)]
    pub(crate) rag_url: Option<String>,

    #[command(subcommand)]
    pub(crate) mode: MultimodalMode,
}

#[derive(Debug, Clone, Subcommand)]
pub(crate) enum MultimodalMode {
    Healthz,
    Plan(MultimodalPlanArgs),
    Ingest(MultimodalIngestArgs),
    Retrieve(MultimodalRetrieveArgs),
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RagSyncArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) limit: Option<usize>,

    #[arg(long)]
    pub(crate) dry_run: bool,

    #[arg(long)]
    pub(crate) prove: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RagSearchArgs {
    #[arg(long)]
    pub(crate) query: String,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) mode: Option<String>,

    #[arg(long)]
    pub(crate) include_cross_modal: bool,

    #[arg(long)]
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct MultimodalPlanArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long, value_name = "PATH")]
    pub(crate) path: Vec<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct MultimodalIngestArgs {
    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long, value_name = "PATH")]
    pub(crate) path: Vec<PathBuf>,

    #[arg(long)]
    pub(crate) apply: bool,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct MultimodalRetrieveArgs {
    #[arg(long)]
    pub(crate) query: String,

    #[arg(long)]
    pub(crate) project: Option<String>,

    #[arg(long)]
    pub(crate) namespace: Option<String>,

    #[arg(long)]
    pub(crate) mode: Option<String>,

    #[arg(long)]
    pub(crate) include_cross_modal: bool,

    #[arg(long)]
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct RequestInput {
    #[arg(long)]
    pub(crate) json: Option<String>,

    #[arg(long)]
    pub(crate) input: Option<PathBuf>,

    #[arg(long)]
    pub(crate) stdin: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct ObsidianImportOutput {
    pub(crate) preview: ObsidianImportPreview,
    pub(crate) submitted: usize,
    pub(crate) attachment_submitted: usize,
    pub(crate) duplicates: usize,
    pub(crate) attachment_duplicates: usize,
    pub(crate) note_failures: usize,
    pub(crate) attachment_failures: usize,
    pub(crate) links_created: usize,
    pub(crate) attachment_links_created: usize,
    pub(crate) mirrored_notes: usize,
    pub(crate) mirrored_attachments: usize,
    pub(crate) attachments: Option<MultimodalIngestOutput>,
    pub(crate) attachment_unchanged_count: usize,
    pub(crate) dry_run: bool,
}
