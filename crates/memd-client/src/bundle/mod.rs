use self::config_runtime::{
    bundle_worker_name_env_ready, read_bundle_config_file, remove_env_assignment,
    rewrite_env_assignment,
};
use super::*;
use crate::coordination::build_task_view_counts;
use crate::runtime::*;

mod admin_runtime;
pub(crate) mod agent_profiles;
mod bootstrap_runtime;
mod config_runtime;
mod init_runtime;
mod lane_runtime;
mod maintenance_runtime;
mod memory_surface;
mod models;
mod profile_runtime;
mod report_runtime;
mod status_runtime;
mod turn_runtime;

#[allow(unused_imports)]
pub(crate) use admin_runtime::*;
#[allow(unused_imports)]
pub(crate) use agent_profiles::*;
#[allow(unused_imports)]
pub(crate) use bootstrap_runtime::*;
#[allow(unused_imports)]
pub(crate) use config_runtime::*;
#[allow(unused_imports)]
pub(crate) use init_runtime::*;
#[allow(unused_imports)]
pub(crate) use lane_runtime::*;
#[allow(unused_imports)]
pub(crate) use maintenance_runtime::*;
#[allow(unused_imports)]
pub(crate) use memory_surface::*;
#[allow(unused_imports)]
pub(crate) use models::*;
#[allow(unused_imports)]
pub(crate) use profile_runtime::*;
#[allow(unused_imports)]
pub(crate) use report_runtime::*;
#[allow(unused_imports)]
pub(crate) use status_runtime::*;
#[allow(unused_imports)]
pub(crate) use turn_runtime::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct BundleConfigFile {
    #[serde(default)]
    pub(crate) project: Option<String>,
    #[serde(default)]
    pub(crate) namespace: Option<String>,
    #[serde(default)]
    pub(crate) agent: Option<String>,
    #[serde(default)]
    pub(crate) session: Option<String>,
    #[serde(default)]
    pub(crate) tab_id: Option<String>,
    #[serde(default)]
    pub(crate) hive_system: Option<String>,
    #[serde(default)]
    pub(crate) hive_role: Option<String>,
    #[serde(default)]
    pub(crate) capabilities: Vec<String>,
    #[serde(default)]
    pub(crate) hive_groups: Vec<String>,
    #[serde(default)]
    pub(crate) hive_group_goal: Option<String>,
    #[serde(default)]
    pub(crate) authority: Option<String>,
    #[serde(default)]
    pub(crate) hive_project_enabled: bool,
    #[serde(default)]
    pub(crate) hive_project_anchor: Option<String>,
    #[serde(default)]
    pub(crate) hive_project_joined_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub(crate) base_url: Option<String>,
    #[serde(default)]
    pub(crate) route: Option<String>,
    #[serde(default)]
    pub(crate) intent: Option<String>,
    #[serde(default)]
    pub(crate) workspace: Option<String>,
    #[serde(default)]
    pub(crate) visibility: Option<String>,
    #[serde(default)]
    pub(crate) heartbeat_model: Option<String>,
    #[serde(default)]
    pub(crate) voice_mode: Option<String>,
    #[serde(default = "default_auto_short_term_capture")]
    pub(crate) auto_short_term_capture: bool,
    #[serde(default)]
    pub(crate) rag_url: Option<String>,
    #[serde(default)]
    pub(crate) backend: Option<BundleBackendConfigFile>,
    #[serde(default)]
    pub(crate) authority_policy: BundleAuthorityPolicy,
    #[serde(default)]
    pub(crate) authority_state: BundleAuthorityState,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BundleConfig {
    pub(crate) schema_version: u32,
    pub(crate) project: String,
    pub(crate) namespace: Option<String>,
    pub(crate) agent: String,
    pub(crate) session: String,
    pub(crate) tab_id: Option<String>,
    pub(crate) hive_system: Option<String>,
    pub(crate) hive_role: Option<String>,
    pub(crate) capabilities: Vec<String>,
    pub(crate) hive_groups: Vec<String>,
    pub(crate) hive_group_goal: Option<String>,
    pub(crate) hive_project_enabled: bool,
    pub(crate) hive_project_anchor: Option<String>,
    pub(crate) hive_project_joined_at: Option<DateTime<Utc>>,
    pub(crate) authority: Option<String>,
    pub(crate) base_url: String,
    pub(crate) route: String,
    pub(crate) intent: String,
    pub(crate) workspace: Option<String>,
    pub(crate) visibility: Option<String>,
    pub(crate) heartbeat_model: String,
    pub(crate) voice_mode: String,
    pub(crate) auto_short_term_capture: bool,
    pub(crate) authority_policy: BundleAuthorityPolicy,
    pub(crate) authority_state: BundleAuthorityState,
    pub(crate) backend: BundleBackendConfig,
    pub(crate) hooks: BundleHooksConfig,
    pub(crate) rag_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BundleBackendConfig {
    pub(crate) rag: BundleRagConfig,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BundleRagConfig {
    pub(crate) enabled: bool,
    pub(crate) provider: String,
    pub(crate) url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BundleHooksConfig {
    pub(crate) context: String,
    pub(crate) capture: String,
    pub(crate) spill: String,
    pub(crate) context_ps1: String,
    pub(crate) capture_ps1: String,
    pub(crate) spill_ps1: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub(crate) enum LocalhostFallbackPolicy {
    #[default]
    Deny,
    AllowReadOnly,
}


impl LocalhostFallbackPolicy {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Deny => "deny",
            Self::AllowReadOnly => "allow_read_only",
        }
    }
}

pub(crate) fn bundle_startup_route_intent(output: &Path) -> (String, String) {
    let runtime = read_bundle_runtime_config(output).ok().flatten();
    let route = runtime
        .as_ref()
        .and_then(|config| config.route.clone())
        .unwrap_or_else(|| "auto".to_string());
    let intent = runtime
        .as_ref()
        .and_then(|config| config.intent.clone())
        .unwrap_or_else(|| "general".to_string());
    (route, intent)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BundleAuthorityPolicy {
    #[serde(default = "default_true")]
    pub(crate) shared_primary: bool,
    #[serde(default)]
    pub(crate) localhost_fallback_policy: LocalhostFallbackPolicy,
}

impl Default for BundleAuthorityPolicy {
    fn default() -> Self {
        Self {
            shared_primary: true,
            localhost_fallback_policy: LocalhostFallbackPolicy::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct BundleAuthorityState {
    #[serde(default = "default_authority_mode")]
    pub(crate) mode: String,
    #[serde(default)]
    pub(crate) degraded: bool,
    #[serde(default)]
    pub(crate) shared_base_url: Option<String>,
    #[serde(default)]
    pub(crate) fallback_base_url: Option<String>,
    #[serde(default)]
    pub(crate) activated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub(crate) activated_by: Option<String>,
    #[serde(default)]
    pub(crate) reason: Option<String>,
    #[serde(default)]
    pub(crate) warning_acknowledged_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub(crate) expires_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub(crate) blocked_capabilities: Vec<String>,
}

impl Default for BundleAuthorityState {
    fn default() -> Self {
        Self {
            mode: default_authority_mode(),
            degraded: false,
            shared_base_url: None,
            fallback_base_url: None,
            activated_at: None,
            activated_by: None,
            reason: None,
            warning_acknowledged_at: None,
            expires_at: None,
            blocked_capabilities: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct BundleRuntimeConfig {
    #[serde(default)]
    pub(crate) project: Option<String>,
    #[serde(default)]
    pub(crate) namespace: Option<String>,
    #[serde(default)]
    pub(crate) agent: Option<String>,
    #[serde(default)]
    pub(crate) session: Option<String>,
    #[serde(default)]
    pub(crate) tab_id: Option<String>,
    #[serde(default)]
    pub(crate) hive_system: Option<String>,
    #[serde(default)]
    pub(crate) hive_role: Option<String>,
    #[serde(default)]
    pub(crate) capabilities: Vec<String>,
    #[serde(default)]
    pub(crate) hive_groups: Vec<String>,
    #[serde(default)]
    pub(crate) hive_group_goal: Option<String>,
    #[serde(default)]
    pub(crate) authority: Option<String>,
    #[serde(default)]
    pub(crate) hive_project_enabled: bool,
    #[serde(default)]
    pub(crate) hive_project_anchor: Option<String>,
    #[serde(default)]
    pub(crate) hive_project_joined_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub(crate) base_url: Option<String>,
    #[serde(default)]
    pub(crate) route: Option<String>,
    #[serde(default)]
    pub(crate) intent: Option<String>,
    #[serde(default)]
    pub(crate) workspace: Option<String>,
    #[serde(default)]
    pub(crate) visibility: Option<String>,
    #[serde(default)]
    pub(crate) heartbeat_model: Option<String>,
    #[serde(default)]
    pub(crate) voice_mode: Option<String>,
    #[serde(default = "default_auto_short_term_capture")]
    pub(crate) auto_short_term_capture: bool,
    #[serde(default)]
    pub(crate) authority_policy: BundleAuthorityPolicy,
    #[serde(default)]
    pub(crate) authority_state: BundleAuthorityState,
}

pub(crate) fn escape_ps1(value: &str) -> String {
    value.replace('\"', "`\"")
}

pub(crate) fn compact_bundle_value(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub(crate) fn memory_visibility_label(value: memd_schema::MemoryVisibility) -> &'static str {
    match value {
        memd_schema::MemoryVisibility::Private => "private",
        memd_schema::MemoryVisibility::Workspace => "workspace",
        memd_schema::MemoryVisibility::Public => "public",
    }
}

pub(crate) fn set_executable_if_shell_script(path: &Path, file_name: &str) -> anyhow::Result<()> {
    if !file_name.ends_with(".sh") {
        return Ok(());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let metadata = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)
            .with_context(|| format!("chmod +x {}", path.display()))?;
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct BundleHeartbeatState {
    #[serde(default)]
    pub(crate) session: Option<String>,
    #[serde(default)]
    pub(crate) agent: Option<String>,
    #[serde(default)]
    pub(crate) effective_agent: Option<String>,
    #[serde(default)]
    pub(crate) tab_id: Option<String>,
    #[serde(default)]
    pub(crate) hive_system: Option<String>,
    #[serde(default)]
    pub(crate) hive_role: Option<String>,
    #[serde(default)]
    pub(crate) worker_name: Option<String>,
    #[serde(default)]
    pub(crate) display_name: Option<String>,
    #[serde(default)]
    pub(crate) role: Option<String>,
    #[serde(default)]
    pub(crate) capabilities: Vec<String>,
    #[serde(default)]
    pub(crate) hive_groups: Vec<String>,
    #[serde(default)]
    pub(crate) lane_id: Option<String>,
    #[serde(default)]
    pub(crate) hive_group_goal: Option<String>,
    #[serde(default)]
    pub(crate) authority: Option<String>,
    #[serde(default)]
    pub(crate) authority_mode: Option<String>,
    #[serde(default)]
    pub(crate) authority_degraded: bool,
    #[serde(default)]
    pub(crate) heartbeat_model: Option<String>,
    #[serde(default)]
    pub(crate) project: Option<String>,
    #[serde(default)]
    pub(crate) namespace: Option<String>,
    #[serde(default)]
    pub(crate) workspace: Option<String>,
    #[serde(default)]
    pub(crate) repo_root: Option<String>,
    #[serde(default)]
    pub(crate) worktree_root: Option<String>,
    #[serde(default)]
    pub(crate) branch: Option<String>,
    #[serde(default)]
    pub(crate) base_branch: Option<String>,
    #[serde(default)]
    pub(crate) visibility: Option<String>,
    #[serde(default)]
    pub(crate) base_url: Option<String>,
    #[serde(default)]
    pub(crate) base_url_healthy: Option<bool>,
    #[serde(default)]
    pub(crate) host: Option<String>,
    #[serde(default)]
    pub(crate) pid: Option<u32>,
    #[serde(default)]
    pub(crate) topic_claim: Option<String>,
    #[serde(default)]
    pub(crate) scope_claims: Vec<String>,
    #[serde(default)]
    pub(crate) task_id: Option<String>,
    #[serde(default)]
    pub(crate) focus: Option<String>,
    #[serde(default)]
    pub(crate) pressure: Option<String>,
    #[serde(default)]
    pub(crate) next_recovery: Option<String>,
    #[serde(default)]
    pub(crate) next_action: Option<String>,
    #[serde(default)]
    pub(crate) working: Option<String>,
    #[serde(default)]
    pub(crate) touches: Vec<String>,
    #[serde(default)]
    pub(crate) blocked_by: Vec<String>,
    #[serde(default)]
    pub(crate) cowork_with: Vec<String>,
    #[serde(default)]
    pub(crate) handoff_target: Option<String>,
    #[serde(default)]
    pub(crate) offered_to: Vec<String>,
    #[serde(default)]
    pub(crate) needs_help: bool,
    #[serde(default)]
    pub(crate) needs_review: bool,
    #[serde(default)]
    pub(crate) handoff_state: Option<String>,
    #[serde(default)]
    pub(crate) confidence: Option<String>,
    #[serde(default)]
    pub(crate) risk: Option<String>,
    pub(crate) status: String,
    pub(crate) last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SessionClaim {
    pub(crate) scope: String,
    pub(crate) session: Option<String>,
    pub(crate) tab_id: Option<String>,
    pub(crate) agent: Option<String>,
    pub(crate) effective_agent: Option<String>,
    pub(crate) project: Option<String>,
    pub(crate) workspace: Option<String>,
    pub(crate) host: Option<String>,
    pub(crate) pid: Option<u32>,
    pub(crate) acquired_at: DateTime<Utc>,
    pub(crate) expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct SessionClaimsState {
    pub(crate) claims: Vec<SessionClaim>,
}

pub(crate) fn session_claim_from_record(record: memd_schema::HiveClaimRecord) -> SessionClaim {
    SessionClaim {
        scope: record.scope,
        session: Some(record.session),
        tab_id: record.tab_id,
        agent: record.agent,
        effective_agent: record.effective_agent,
        project: record.project,
        workspace: record.workspace,
        host: record.host,
        pid: record.pid,
        acquired_at: record.acquired_at,
        expires_at: record.expires_at,
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ClaimsResponse {
    pub(crate) bundle_root: String,
    pub(crate) bundle_session: Option<String>,
    pub(crate) live_session: Option<String>,
    pub(crate) rebased_from: Option<String>,
    pub(crate) current_session: Option<String>,
    pub(crate) current_tab_id: Option<String>,
    pub(crate) claims: Vec<SessionClaim>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MessagesResponse {
    pub(crate) bundle_root: String,
    pub(crate) bundle_session: Option<String>,
    pub(crate) live_session: Option<String>,
    pub(crate) rebased_from: Option<String>,
    pub(crate) current_session: Option<String>,
    pub(crate) messages: Vec<HiveMessageRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SessionResponse {
    pub(crate) action: String,
    pub(crate) bundle_root: String,
    pub(crate) bundle_session: Option<String>,
    pub(crate) live_session: Option<String>,
    pub(crate) rebased_from: Option<String>,
    pub(crate) tab_id: Option<String>,
    pub(crate) reconciled: bool,
    pub(crate) reconciled_retired_sessions: usize,
    pub(crate) retired_sessions: usize,
    pub(crate) retire_target: Option<String>,
    pub(crate) heartbeat: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CapabilitiesResponse {
    pub(crate) bundle_root: String,
    pub(crate) generated_at: DateTime<Utc>,
    pub(crate) discovered: usize,
    pub(crate) universal: usize,
    pub(crate) bridgeable: usize,
    pub(crate) harness_native: usize,
    pub(crate) bridge_actions: usize,
    pub(crate) wired_harnesses: usize,
    pub(crate) filters: serde_json::Value,
    pub(crate) harnesses: Vec<CapabilityHarnessSummary>,
    pub(crate) records: Vec<CapabilityRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CapabilityHarnessSummary {
    pub(crate) harness: String,
    pub(crate) capabilities: usize,
    pub(crate) installed: usize,
    pub(crate) bridge_actions: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TasksResponse {
    pub(crate) bundle_root: String,
    pub(crate) current_session: Option<String>,
    pub(crate) tasks: Vec<HiveTaskRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MemorySurfaceResponse {
    pub(crate) bundle_root: String,
    pub(crate) truth_summary: TruthSummary,
    pub(crate) context_records: usize,
    pub(crate) working_records: usize,
    pub(crate) inbox_items: usize,
    pub(crate) source_lanes: usize,
    pub(crate) rehydration_queue: usize,
    pub(crate) semantic_hits: usize,
    pub(crate) change_summary: usize,
    pub(crate) estimated_prompt_tokens: usize,
    pub(crate) refresh_recommended: bool,
    pub(crate) contradiction_pressure: usize,
    pub(crate) superseded_pressure: usize,
    pub(crate) contradiction_reasons: Vec<String>,
    pub(crate) superseded_reasons: Vec<String>,
    pub(crate) records: Vec<TruthRecordSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProjectAwarenessEntry {
    pub(crate) project_dir: String,
    pub(crate) bundle_root: String,
    pub(crate) project: Option<String>,
    pub(crate) namespace: Option<String>,
    pub(crate) repo_root: Option<String>,
    pub(crate) worktree_root: Option<String>,
    pub(crate) branch: Option<String>,
    pub(crate) base_branch: Option<String>,
    pub(crate) agent: Option<String>,
    pub(crate) session: Option<String>,
    pub(crate) tab_id: Option<String>,
    pub(crate) effective_agent: Option<String>,
    pub(crate) hive_system: Option<String>,
    pub(crate) hive_role: Option<String>,
    pub(crate) capabilities: Vec<String>,
    pub(crate) hive_groups: Vec<String>,
    pub(crate) hive_group_goal: Option<String>,
    pub(crate) authority: Option<String>,
    pub(crate) base_url: Option<String>,
    pub(crate) presence: String,
    pub(crate) host: Option<String>,
    pub(crate) pid: Option<u32>,
    pub(crate) active_claims: usize,
    pub(crate) workspace: Option<String>,
    pub(crate) visibility: Option<String>,
    pub(crate) topic_claim: Option<String>,
    pub(crate) scope_claims: Vec<String>,
    pub(crate) task_id: Option<String>,
    pub(crate) focus: Option<String>,
    pub(crate) pressure: Option<String>,
    pub(crate) next_recovery: Option<String>,
    pub(crate) last_updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProjectAwarenessResponse {
    pub(crate) root: String,
    pub(crate) current_bundle: String,
    pub(crate) collisions: Vec<String>,
    pub(crate) entries: Vec<ProjectAwarenessEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoordinationResponse {
    pub(crate) bundle_root: String,
    pub(crate) current_session: String,
    pub(crate) inbox: HiveCoordinationInboxResponse,
    pub(crate) active_hives: Vec<ProjectAwarenessEntry>,
    pub(crate) recovery: CoordinationRecoverySummary,
    pub(crate) lane_fault: Option<JsonValue>,
    pub(crate) lane_receipts: Vec<HiveCoordinationReceiptRecord>,
    pub(crate) policy_conflicts: Vec<String>,
    pub(crate) suggestions: Vec<CoordinationSuggestion>,
    pub(crate) boundary_recommendations: Vec<String>,
    pub(crate) receipts: Vec<HiveCoordinationReceiptRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoordinationRecoverySummary {
    pub(crate) stale_hives: Vec<ProjectAwarenessEntry>,
    pub(crate) reclaimable_claims: Vec<SessionClaim>,
    pub(crate) stalled_tasks: Vec<HiveTaskRecord>,
    pub(crate) retireable_sessions: Vec<ProjectAwarenessEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoordinationChangeResponse {
    pub(crate) bundle_root: String,
    pub(crate) current_session: String,
    pub(crate) view: String,
    pub(crate) changed: bool,
    pub(crate) alerts: Vec<String>,
    pub(crate) snapshot: CoordinationAlertSnapshot,
    pub(crate) generated_at: DateTime<Utc>,
    pub(crate) previous_generated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CoordinationSnapshotState {
    pub(crate) generated_at: DateTime<Utc>,
    pub(crate) view: String,
    pub(crate) snapshot: CoordinationAlertSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CoordinationAlertSnapshot {
    pub(crate) message_count: usize,
    pub(crate) owned_count: usize,
    pub(crate) help_count: usize,
    pub(crate) review_count: usize,
    pub(crate) lane_fault_count: usize,
    pub(crate) lane_receipt_count: usize,
    pub(crate) stale_hive_count: usize,
    pub(crate) reclaimable_claim_count: usize,
    pub(crate) stalled_task_count: usize,
    pub(crate) policy_conflict_count: usize,
    pub(crate) recommendation_count: usize,
    pub(crate) suggestion_count: usize,
    pub(crate) latest_receipt_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CoordinationSuggestion {
    pub(crate) action: String,
    pub(crate) priority: String,
    pub(crate) target_session: Option<String>,
    pub(crate) task_id: Option<String>,
    pub(crate) scope: Option<String>,
    pub(crate) message_id: Option<String>,
    pub(crate) reason: String,
    pub(crate) stale_session: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct BundleBackendConfigFile {
    #[serde(default)]
    pub(crate) rag: Option<BundleRagConfigFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct BundleRagConfigFile {
    #[serde(default)]
    pub(crate) enabled: Option<bool>,
    #[serde(default)]
    pub(crate) url: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct BundleRagConfigState {
    pub(crate) configured: bool,
    pub(crate) enabled: bool,
    pub(crate) url: Option<String>,
    pub(crate) source: String,
}

pub(crate) fn resolve_bundle_rag_config(config: BundleConfigFile) -> Option<BundleRagConfigState> {
    if let Some(rag) = config.backend.and_then(|backend| backend.rag) {
        let url = rag
            .url
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let enabled = rag.enabled.unwrap_or(url.is_some());
        let configured = url.is_some();
        return Some(BundleRagConfigState {
            configured,
            enabled,
            url,
            source: "backend.rag".to_string(),
        });
    }

    let url = config
        .rag_url
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if let Some(url) = url {
        return Some(BundleRagConfigState {
            configured: true,
            enabled: true,
            url: Some(url),
            source: "rag_url".to_string(),
        });
    }

    None
}

pub(crate) fn write_bundle_backend_env(output: &Path, config: &BundleConfig) -> anyhow::Result<()> {
    let backend_env = output.join("backend.env");
    let backend_env_ps1 = output.join("backend.env.ps1");
    let rag = &config.backend.rag;

    let mut shell = String::new();
    shell.push_str(&format!(
        "MEMD_BUNDLE_SCHEMA_VERSION={}\n",
        config.schema_version
    ));
    shell.push_str(&format!("MEMD_BUNDLE_BACKEND_PROVIDER={}\n", rag.provider));
    shell.push_str(&format!(
        "MEMD_BUNDLE_BACKEND_ENABLED={}\n",
        if rag.enabled { "true" } else { "false" }
    ));
    if let Some(url) = rag.url.as_deref() {
        shell.push_str(&format!("MEMD_RAG_URL={url}\n"));
    }
    fs::write(&backend_env, shell).with_context(|| format!("write {}", backend_env.display()))?;

    let mut ps1 = String::new();
    ps1.push_str(&format!(
        "$env:MEMD_BUNDLE_SCHEMA_VERSION = \"{}\"\n",
        config.schema_version
    ));
    ps1.push_str(&format!(
        "$env:MEMD_BUNDLE_BACKEND_PROVIDER = \"{}\"\n",
        escape_ps1(&rag.provider)
    ));
    ps1.push_str(&format!(
        "$env:MEMD_BUNDLE_BACKEND_ENABLED = \"{}\"\n",
        if rag.enabled { "true" } else { "false" }
    ));
    if let Some(url) = rag.url.as_deref() {
        ps1.push_str(&format!("$env:MEMD_RAG_URL = \"{}\"\n", escape_ps1(url)));
    }
    fs::write(&backend_env_ps1, ps1)
        .with_context(|| format!("write {}", backend_env_ps1.display()))?;

    Ok(())
}

pub(crate) fn bundle_heartbeat_state_path(output: &Path) -> PathBuf {
    output.join("state").join("heartbeat.json")
}

pub(crate) fn bundle_claims_state_path(output: &Path) -> PathBuf {
    output.join("state").join("claims.json")
}

pub(crate) fn read_bundle_heartbeat(output: &Path) -> anyhow::Result<Option<BundleHeartbeatState>> {
    let path = bundle_heartbeat_state_path(output);
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let state = serde_json::from_str::<BundleHeartbeatState>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(state))
}

pub(crate) fn read_bundle_claims(output: &Path) -> anyhow::Result<SessionClaimsState> {
    let path = bundle_claims_state_path(output);
    if !path.exists() {
        return Ok(SessionClaimsState::default());
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let state = serde_json::from_str::<SessionClaimsState>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(state)
}

pub(crate) fn write_bundle_claims(output: &Path, state: &SessionClaimsState) -> anyhow::Result<()> {
    let path = bundle_claims_state_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&path, serde_json::to_string_pretty(state)? + "\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub(crate) fn detect_host_name() -> Option<String> {
    std::env::var("HOSTNAME")
        .ok()
        .or_else(|| std::env::var("COMPUTERNAME").ok())
        .filter(|value| !value.trim().is_empty())
}

pub(crate) fn heartbeat_presence_label(last_seen: DateTime<Utc>) -> &'static str {
    let age = Utc::now() - last_seen;
    if age.num_seconds() <= 120 {
        "active"
    } else if age.num_minutes() <= 15 {
        "stale"
    } else {
        "dead"
    }
}
