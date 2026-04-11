use super::*;
use crate::bundle_config_runtime::{
    bundle_worker_name_env_ready, read_bundle_config_file, remove_env_assignment,
    rewrite_env_assignment,
};
use crate::bundle_lane_runtime::*;
use crate::coordination_views::build_task_view_counts;

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
pub(crate) enum LocalhostFallbackPolicy {
    Deny,
    AllowReadOnly,
}

impl Default for LocalhostFallbackPolicy {
    fn default() -> Self {
        Self::Deny
    }
}

impl LocalhostFallbackPolicy {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Deny => "deny",
            Self::AllowReadOnly => "allow_read_only",
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub(crate) async fn read_bundle_status(
    output: &Path,
    base_url: &str,
) -> anyhow::Result<serde_json::Value> {
    let runtime_before_overlay = read_bundle_runtime_config_raw(output)?;
    let runtime = read_bundle_runtime_config(output)?;
    let bundle_session = runtime_before_overlay
        .as_ref()
        .and_then(|config| config.session.clone());
    let live_session = runtime.as_ref().and_then(|config| config.session.clone());
    let rebased_from = match (bundle_session.as_deref(), live_session.as_deref()) {
        (Some(bundle), Some(live)) if bundle != live => Some(bundle.to_string()),
        _ => None,
    };
    let resolved_base_url = resolve_bundle_command_base_url(
        base_url,
        runtime
            .as_ref()
            .and_then(|config| config.base_url.as_deref()),
    );
    if runtime
        .as_ref()
        .and_then(|config| config.session.as_deref())
        .is_some()
    {
        let _ = timeout_ok(refresh_bundle_heartbeat(output, None, false)).await;
    }
    let client = MemdClient::new(&resolved_base_url)?;
    let health = timeout_ok(client.healthz()).await;
    let heartbeat = read_bundle_heartbeat(output)?.map(|mut state| {
        if state.project.is_none() {
            state.project = runtime.as_ref().and_then(|config| config.project.clone());
        }
        if state.namespace.is_none() {
            state.namespace = runtime.as_ref().and_then(|config| config.namespace.clone());
        }
        if state.workspace.is_none() {
            state.workspace = runtime.as_ref().and_then(|config| config.workspace.clone());
        }
        if state.visibility.is_none() {
            state.visibility = runtime
                .as_ref()
                .and_then(|config| config.visibility.clone());
        }
        if state.session.is_none() {
            state.session = runtime.as_ref().and_then(|config| config.session.clone());
        }
        if state.agent.is_none() {
            state.agent = runtime.as_ref().and_then(|config| config.agent.clone());
        }
        if state.effective_agent.is_none() {
            state.effective_agent = runtime.as_ref().and_then(|config| {
                config
                    .agent
                    .as_deref()
                    .map(|agent| compose_agent_identity(agent, config.session.as_deref()))
            });
        }
        if state.tab_id.is_none() {
            state.tab_id = runtime.as_ref().and_then(|config| config.tab_id.clone());
        }
        state
    });
    let runtimes = read_memd_runtime_wiring();
    let harness_bridge =
        read_bundle_harness_bridge_registry(output)?.unwrap_or_else(build_harness_bridge_registry);
    let config_exists = output.join("config.json").exists();
    let env_exists = output.join("env").exists();
    let env_ps1_exists = output.join("env.ps1").exists();
    let hooks_exists = output.join("hooks").exists();
    let agents_exists = output.join("agents").exists();
    let worker_name_env_ready = read_bundle_config_file(output)
        .ok()
        .map(|(_, config)| bundle_worker_name_env_ready(output, &config))
        .unwrap_or(false);
    let mut missing = Vec::<&str>::new();
    if !config_exists {
        missing.push("config.json");
    }
    if !env_exists {
        missing.push("env");
    }
    if !env_ps1_exists {
        missing.push("env.ps1");
    }
    if env_exists && env_ps1_exists && !worker_name_env_ready {
        missing.push("worker_name_env");
    }
    if !hooks_exists {
        missing.push("hooks/");
    }
    if !agents_exists {
        missing.push("agents/");
    }
    let resume_preview = if output.join("config.json").exists() && health.is_some() {
        let preview = timeout_ok(read_bundle_resume(
            &ResumeArgs {
                output: output.to_path_buf(),
                project: None,
                namespace: None,
                agent: None,
                workspace: None,
                visibility: None,
                route: None,
                intent: Some("current_task".to_string()),
                limit: Some(4),
                rehydration_limit: Some(2),
                semantic: false,
                prompt: false,
                summary: false,
            },
            &resolved_base_url,
        ))
        .await;
        preview.map(|snapshot| {
            serde_json::json!({
                "project": snapshot.project,
                "namespace": snapshot.namespace,
                "agent": snapshot.agent,
                "session": runtime.as_ref().and_then(|config| config.session.clone()),
                "tab_id": runtime.as_ref().and_then(|config| config.tab_id.clone()),
                "workspace": snapshot.workspace,
                "visibility": snapshot.visibility,
                "route": snapshot.route,
                "intent": snapshot.intent,
                "context_records": snapshot.context.records.len(),
                "working_records": snapshot.working.records.len(),
                "inbox_items": snapshot.inbox.items.len(),
                "workspace_lanes": snapshot.workspaces.workspaces.len(),
                "rehydration_queue": snapshot.working.rehydration_queue.len(),
                "semantic_hits": snapshot.semantic.as_ref().map(|semantic| semantic.items.len()).unwrap_or(0),
                "change_summary": snapshot.change_summary,
                "event_spine": snapshot.event_spine(),
                "focus": snapshot.working.records.first().map(|record| record.record.clone()),
                "pressure": snapshot.inbox.items.first().map(|item| item.item.content.clone()),
                "next_recovery": snapshot.working.rehydration_queue.first().map(|item| format!("{}: {}", item.label, item.summary)),
                "estimated_prompt_chars": snapshot.estimated_prompt_chars(),
                "estimated_prompt_tokens": snapshot.estimated_prompt_tokens(),
                "context_pressure": snapshot.context_pressure(),
                "redundant_context_items": snapshot.redundant_context_items(),
                "refresh_recommended": snapshot.refresh_recommended,
            })
        })
    } else {
        None
    };
    let truth_summary = if output.join("config.json").exists() && health.is_some() {
        let snapshot = timeout_ok(read_bundle_resume(
            &ResumeArgs {
                output: output.to_path_buf(),
                project: None,
                namespace: None,
                agent: None,
                workspace: None,
                visibility: None,
                route: None,
                intent: Some("current_task".to_string()),
                limit: Some(4),
                rehydration_limit: Some(2),
                semantic: true,
                prompt: false,
                summary: false,
            },
            &resolved_base_url,
        ))
        .await;
        snapshot.map(|snapshot| {
            serde_json::to_value(build_truth_summary(&snapshot)).unwrap_or(JsonValue::Null)
        })
    } else {
        None
    };
    let current_project = runtime.as_ref().and_then(|config| config.project.clone());
    let current_namespace = runtime.as_ref().and_then(|config| config.namespace.clone());
    let current_workspace = runtime.as_ref().and_then(|config| config.workspace.clone());
    let current_session = runtime.as_ref().and_then(|config| config.session.clone());
    let cowork_surface = if health.is_some() {
        let inbox_request = HiveCoordinationInboxRequest {
            session: current_session.clone().unwrap_or_default(),
            project: current_project.clone(),
            namespace: current_namespace.clone(),
            workspace: current_workspace.clone(),
            limit: Some(128),
        };
        let inbox = timeout_ok(client.hive_coordination_inbox(&inbox_request)).await;
        let tasks_request = HiveTasksRequest {
            session: None,
            project: current_project.clone(),
            namespace: current_namespace.clone(),
            workspace: current_workspace.clone(),
            active_only: Some(false),
            limit: Some(256),
        };
        let tasks = timeout_ok(client.hive_tasks(&tasks_request)).await;
        match (inbox, tasks) {
            (Some(inbox), Some(tasks)) => {
                let exclusive = tasks
                    .tasks
                    .iter()
                    .filter(|task| task.coordination_mode == "exclusive_write")
                    .count();
                let open = tasks
                    .tasks
                    .iter()
                    .filter(|task| task.status != "done" && task.status != "closed")
                    .count();
                Some(serde_json::json!({
                    "tasks": tasks.tasks.len(),
                    "open_tasks": open,
                    "help_tasks": tasks.tasks.iter().filter(|task| task.help_requested).count(),
                    "review_tasks": tasks.tasks.iter().filter(|task| task.review_requested).count(),
                    "exclusive_tasks": exclusive,
                    "shared_tasks": tasks.tasks.len().saturating_sub(exclusive),
                    "inbox_messages": inbox.messages.len(),
                    "owned_tasks": inbox.owned_tasks.len(),
                    "owned_exclusive_tasks": inbox
                        .owned_tasks
                        .iter()
                        .filter(|task| task.coordination_mode == "exclusive_write")
                        .count(),
                    "owned_shared_tasks": inbox
                        .owned_tasks
                        .iter()
                        .filter(|task| task.coordination_mode != "exclusive_write")
                        .count(),
                    "help_inbox": inbox.help_tasks.len(),
                    "review_inbox": inbox.review_tasks.len(),
                    "views": build_task_view_counts(&tasks.tasks, current_session.as_deref()),
                }))
            }
            _ => None,
        }
    } else {
        None
    };
    let lane_receipts = if health.is_some() {
        let receipts_request = HiveCoordinationReceiptsRequest {
            session: None,
            project: current_project.clone(),
            namespace: current_namespace.clone(),
            workspace: current_workspace.clone(),
            limit: Some(64),
        };
        timeout_ok(client.hive_coordination_receipts(&receipts_request))
            .await
            .map(|response| {
                let receipts = response
                    .receipts
                    .into_iter()
                    .filter(|receipt| receipt.kind.starts_with("lane_"))
                    .collect::<Vec<_>>();
                serde_json::json!({
                    "count": receipts.len(),
                    "latest_kind": receipts.first().map(|receipt| receipt.kind.clone()),
                    "latest_summary": receipts.first().map(|receipt| receipt.summary.clone()),
                    "recent": receipts
                        .into_iter()
                        .take(8)
                        .map(|receipt| serde_json::json!({
                            "kind": receipt.kind,
                            "actor_session": receipt.actor_session,
                            "target_session": receipt.target_session,
                            "scope": receipt.scope,
                            "summary": receipt.summary,
                            "created_at": receipt.created_at,
                        }))
                        .collect::<Vec<_>>(),
                })
            })
    } else {
        None
    };
    let maintenance_surface = match (
        read_latest_maintain_report(output)?,
        read_previous_maintain_report(output)?,
        read_recent_maintain_reports(output, 5)?,
    ) {
        (Some(report), previous, history) => {
            let total = report.compacted_items + report.refreshed_items + report.repaired_items;
            let previous_total = previous
                .as_ref()
                .map(|value| value.compacted_items + value.refreshed_items + value.repaired_items)
                .unwrap_or(0);
            let delta_total = total as i64 - previous_total as i64;
            let auto_mode = report.mode == "auto";
            let auto_reason = if auto_mode {
                "none".to_string()
            } else if delta_total < 0 {
                "trend_down".to_string()
            } else if delta_total == 0 {
                "trend_flat".to_string()
            } else if !report.findings.is_empty() {
                "findings_present".to_string()
            } else {
                "none".to_string()
            };
            let auto_recommended = auto_reason != "none";
            let history_modes = history
                .iter()
                .map(|value| value.mode.clone())
                .collect::<Vec<_>>();
            let history_receipts = history
                .iter()
                .map(|value| {
                    value
                        .receipt_id
                        .clone()
                        .unwrap_or_else(|| "none".to_string())
                })
                .collect::<Vec<_>>();
            let history_totals = history
                .iter()
                .map(|value| value.compacted_items + value.refreshed_items + value.repaired_items)
                .collect::<Vec<_>>();
            Some(serde_json::json!({
                "mode": report.mode,
                "auto_mode": auto_mode,
                "auto_recommended": auto_recommended,
                "auto_reason": auto_reason,
                "receipt": report.receipt_id,
                "compacted": report.compacted_items,
                "refreshed": report.refreshed_items,
                "repaired": report.repaired_items,
                "findings": report.findings.len(),
                "total_actions": total,
                "delta_total_actions": delta_total,
                "trend": if delta_total > 0 { "up" } else if delta_total < 0 { "down" } else { "flat" },
                "previous_mode": previous.as_ref().map(|value| value.mode.clone()),
                "history_modes": history_modes,
                "history_receipts": history_receipts,
                "history_totals": history_totals,
                "history_count": history.len(),
                "generated_at": report.generated_at,
            }))
        }
        _ => None,
    };
    let rag_config = read_bundle_rag_config(output)?;
    let rag = match rag_config {
        Some(config) if config.enabled => {
            let source = config.source;
            let Some(url) = config.url.clone() else {
                return Ok(serde_json::json!({
                    "bundle": output,
                    "exists": output.exists(),
                    "config": output.join("config.json").exists(),
                    "env": output.join("env").exists(),
                    "env_ps1": output.join("env.ps1").exists(),
                    "hooks": output.join("hooks").exists(),
                    "agents": output.join("agents").exists(),
                    "server": health,
                    "rag": {
                        "configured": false,
                        "enabled": true,
                        "healthy": false,
                        "error": "rag backend enabled but no url configured",
                        "source": source,
                    },
                }));
            };
            let rag_result = RagClient::new(url.as_str())?.healthz().await;
            Some(match rag_result {
                Ok(health) => serde_json::json!({
                    "configured": true,
                    "enabled": true,
                    "url": url,
                    "healthy": true,
                    "health": health,
                    "source": source,
                }),
                Err(error) => serde_json::json!({
                    "configured": true,
                    "enabled": true,
                    "url": url,
                    "healthy": false,
                    "error": error.to_string(),
                    "source": source,
                }),
            })
        }
        Some(config) => Some(serde_json::json!({
            "configured": config.configured,
            "enabled": false,
            "url": config.url,
            "healthy": null,
            "source": config.source,
        })),
        None => None,
    };
    let rag_ready = rag
        .as_ref()
        .map(|value| {
            !value
                .get("enabled")
                .and_then(|enabled| enabled.as_bool())
                .unwrap_or(false)
                || value
                    .get("healthy")
                    .and_then(|healthy| healthy.as_bool())
                    .unwrap_or(false)
        })
        .unwrap_or(true);
    let evolution = summarize_evolution_status(output)?;
    let capability_registry =
        build_bundle_capability_registry(std::env::current_dir().ok().as_deref());
    let capability_surface = serde_json::json!({
        "discovered": capability_registry.capabilities.len(),
        "universal": capability_registry
            .capabilities
            .iter()
            .filter(|record| is_universal_class(&record.portability_class))
            .count(),
        "bridgeable": capability_registry
            .capabilities
            .iter()
            .filter(|record| is_bridgeable_class(&record.portability_class))
            .count(),
        "harness_native": capability_registry
            .capabilities
            .iter()
            .filter(|record| is_harness_native_class(&record.portability_class))
            .count(),
    });
    let lane_surface = read_bundle_lane_surface(output)?
        .map(|surface| serde_json::to_value(surface).unwrap_or(JsonValue::Null));
    let lane_fault = detect_bundle_lane_collision(output, current_session.as_deref())
        .await?
        .and_then(|conflict| {
            build_lane_fault_surface(output, current_session.as_deref(), &conflict)
        });
    let bridge_ready = harness_bridge.all_wired;
    let setup_ready = output.exists()
        && missing.is_empty()
        && health.is_some()
        && runtime.is_some()
        && rag_ready
        && bridge_ready;
    Ok(serde_json::json!({
        "bundle": output,
        "exists": output.exists(),
        "config": config_exists,
        "env": env_exists,
        "env_ps1": env_ps1_exists,
        "worker_name_env_ready": worker_name_env_ready,
        "hooks": hooks_exists,
        "agents": agents_exists,
        "setup_ready": setup_ready,
        "missing": missing,
        "runtimes": runtimes,
        "harness_bridge": {
            "ready": bridge_ready,
            "portable": harness_bridge.all_wired,
            "portability_class": harness_bridge.overall_portability_class,
            "generated_at": harness_bridge.generated_at,
            "harnesses": harness_bridge.harnesses,
            "missing_harnesses": harness_bridge
                .harnesses
                .iter()
                .filter(|record| !record.wired)
                .map(|record| record.harness.clone())
                .collect::<Vec<_>>(),
        },
        "active_agent": runtime.as_ref().and_then(|config| config.agent.clone()),
        "defaults": runtime.as_ref().and_then(|config| {
            let mut defaults = serde_json::to_value(config).ok()?;
            if let JsonValue::Object(ref mut map) = defaults {
                map.insert(
                    "voice_mode".to_string(),
                    JsonValue::String(
                        read_bundle_voice_mode(output).unwrap_or_else(default_voice_mode),
                    ),
                );
            }
            Some(defaults)
        }),
        "authority": runtime
            .as_ref()
            .map(|config| config.authority_state.mode.clone()),
        "shared_primary": runtime_prefers_shared_authority(runtime.as_ref()),
        "localhost_read_only_allowed": runtime_allows_localhost_read_only(runtime.as_ref()),
        "degraded": runtime
            .as_ref()
            .map(|config| config.authority_state.degraded)
            .unwrap_or(false),
        "shared_base_url": runtime
            .as_ref()
            .and_then(|config| config.authority_state.shared_base_url.clone()),
        "fallback_base_url": runtime
            .as_ref()
            .and_then(|config| config.authority_state.fallback_base_url.clone()),
        "authority_warning": authority_warning_lines(runtime.as_ref()),
        "session_overlay": {
            "bundle_session": bundle_session,
            "live_session": live_session,
            "rebased_from": rebased_from,
        },
        "heartbeat": heartbeat
            .as_ref()
            .and_then(|value| serde_json::to_value(value).ok()),
        "resume_preview": resume_preview,
        "truth_summary": truth_summary,
        "evolution": evolution,
        "cowork_surface": cowork_surface,
        "lane_surface": lane_surface,
        "lane_fault": lane_fault,
        "lane_receipts": lane_receipts,
        "maintenance_surface": maintenance_surface,
        "capability_surface": capability_surface,
        "server": health,
        "rag": rag.unwrap_or_else(|| serde_json::json!({
            "configured": false,
            "enabled": false,
            "healthy": null,
        })),
    }))
}

pub(crate) fn summarize_evolution_status(
    output: &Path,
) -> anyhow::Result<Option<serde_json::Value>> {
    let proposal = read_latest_evolution_proposal(output)?;
    let branch_manifest = read_latest_evolution_branch_manifest(output)?;
    let authority = read_evolution_authority_ledger(output)?;
    let merge_queue = read_evolution_merge_queue(output)?;
    let durability_queue = read_evolution_durability_queue(output)?;

    if proposal.is_none()
        && branch_manifest.is_none()
        && authority.is_none()
        && merge_queue.is_none()
        && durability_queue.is_none()
    {
        return Ok(None);
    }

    Ok(Some(serde_json::json!({
        "proposal_state": proposal.as_ref().map(|value| value.state.clone()).unwrap_or_else(|| "none".to_string()),
        "scope_class": proposal.as_ref().map(|value| value.scope_class.clone()).unwrap_or_else(|| "none".to_string()),
        "scope_gate": proposal.as_ref().map(|value| value.scope_gate.clone()).unwrap_or_else(|| "none".to_string()),
        "authority_tier": proposal
            .as_ref()
            .map(|value| value.authority_tier.clone())
            .or_else(|| authority.as_ref().and_then(|ledger| ledger.entries.last()).map(|entry| entry.authority_tier.clone()))
            .unwrap_or_else(|| "none".to_string()),
        "merge_status": merge_queue
            .as_ref()
            .and_then(|queue| queue.entries.last())
            .map(|entry| entry.status.clone())
            .unwrap_or_else(|| "none".to_string()),
        "durability_status": durability_queue
            .as_ref()
            .and_then(|queue| queue.entries.last())
            .map(|entry| entry.status.clone())
            .unwrap_or_else(|| "none".to_string()),
        "branch": proposal
            .as_ref()
            .map(|value| value.branch.clone())
            .or_else(|| branch_manifest.as_ref().map(|value| value.branch.clone()))
            .unwrap_or_else(|| "none".to_string()),
        "durable_truth": proposal.as_ref().is_some_and(|value| value.durable_truth),
    })))
}

pub(crate) fn experiment_evolution_summary(
    output: &Path,
) -> anyhow::Result<Option<ExperimentEvolutionSummary>> {
    let proposal = read_latest_evolution_proposal(output)?;
    let branch_manifest = read_latest_evolution_branch_manifest(output)?;
    let authority = read_evolution_authority_ledger(output)?;
    let merge_queue = read_evolution_merge_queue(output)?;
    let durability_queue = read_evolution_durability_queue(output)?;

    if proposal.is_none()
        && branch_manifest.is_none()
        && authority.is_none()
        && merge_queue.is_none()
        && durability_queue.is_none()
    {
        return Ok(None);
    }

    Ok(Some(ExperimentEvolutionSummary {
        proposal_state: proposal
            .as_ref()
            .map(|value| value.state.clone())
            .unwrap_or_else(|| "none".to_string()),
        scope_class: proposal
            .as_ref()
            .map(|value| value.scope_class.clone())
            .unwrap_or_else(|| "none".to_string()),
        scope_gate: proposal
            .as_ref()
            .map(|value| value.scope_gate.clone())
            .unwrap_or_else(|| "none".to_string()),
        authority_tier: proposal
            .as_ref()
            .map(|value| value.authority_tier.clone())
            .or_else(|| {
                authority
                    .as_ref()
                    .and_then(|ledger| ledger.entries.last())
                    .map(|entry| entry.authority_tier.clone())
            })
            .unwrap_or_else(|| "none".to_string()),
        merge_status: merge_queue
            .as_ref()
            .and_then(|queue| queue.entries.last())
            .map(|entry| entry.status.clone())
            .unwrap_or_else(|| "none".to_string()),
        durability_status: durability_queue
            .as_ref()
            .and_then(|queue| queue.entries.last())
            .map(|entry| entry.status.clone())
            .unwrap_or_else(|| "none".to_string()),
        branch: proposal
            .as_ref()
            .map(|value| value.branch.clone())
            .or_else(|| branch_manifest.as_ref().map(|value| value.branch.clone()))
            .unwrap_or_else(|| "none".to_string()),
        durable_truth: proposal.as_ref().is_some_and(|value| value.durable_truth),
    }))
}

pub(crate) fn read_memd_runtime_wiring() -> serde_json::Value {
    let codex = detect_codex_memd_wiring();
    let claude = detect_claude_memd_wiring();
    let openclaw = detect_openclaw_memd_wiring();
    let opencode = detect_opencode_memd_wiring();
    let claw = detect_claw_memd_wiring();
    let claude_family = detect_claude_family_memd_wiring();
    serde_json::json!({
        "codex": codex,
        "claude": claude,
        "claw": claw,
        "claude_family": claude_family,
        "openclaw": openclaw,
        "opencode": opencode,
        "all_wired": codex.get("wired").and_then(|value| value.as_bool()).unwrap_or(false)
            && claude.get("wired").and_then(|value| value.as_bool()).unwrap_or(false)
            && openclaw.get("wired").and_then(|value| value.as_bool()).unwrap_or(false)
            && opencode.get("wired").and_then(|value| value.as_bool()).unwrap_or(false),
    })
}

pub(crate) fn detect_codex_memd_wiring() -> serde_json::Value {
    let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
    let config = home.join(".codex").join("config.toml");
    let hook = home
        .join(".codex")
        .join("hooks")
        .join("memd-session-context.js");
    let skill = home
        .join(".codex")
        .join("skills")
        .join("memd")
        .join("SKILL.md");
    let hook_wired = hook.exists()
        && fs::read_to_string(&hook)
            .ok()
            .map(|content| content.contains("memd"))
            .unwrap_or(false);
    let skill_wired = skill.exists();
    serde_json::json!({
        "wired": config.exists() && hook_wired && skill_wired,
        "config": config.exists(),
        "hook": hook_wired,
        "skill": skill_wired,
    })
}

pub(crate) fn detect_claude_memd_wiring() -> serde_json::Value {
    let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
    let settings = home.join(".claude").join("settings.json");
    let hook = home
        .join(".claude")
        .join("hooks")
        .join("gsd-session-context.js");
    let hook_wired = hook.exists()
        && fs::read_to_string(&hook)
            .ok()
            .map(|content| content.contains("memd"))
            .unwrap_or(false);
    serde_json::json!({
        "wired": settings.exists() && hook_wired,
        "settings": settings.exists(),
        "hook": hook_wired,
    })
}

pub(crate) fn detect_claude_family_memd_wiring() -> serde_json::Value {
    let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
    let harnesses = detect_claude_family_harness_roots(&home)
        .into_iter()
        .filter(|root| root.harness != "claude")
        .map(|root| {
            let settings = root.root.join("settings.json");
            let hook_candidates = [
                root.root.join("hooks").join("gsd-session-context.js"),
                root.root.join("hooks").join("memd-session-context.js"),
            ];
            let hook_wired = hook_candidates.iter().any(|path| {
                path.exists()
                    && fs::read_to_string(path)
                        .ok()
                        .map(|content| content.contains("memd"))
                        .unwrap_or(false)
            });
            serde_json::json!({
                "harness": root.harness,
                "root": root.root.display().to_string(),
                "wired": settings.exists() && hook_wired,
                "settings": settings.exists(),
                "hook": hook_wired,
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "count": harnesses.len(),
        "harnesses": harnesses,
    })
}

pub(crate) fn detect_openclaw_memd_wiring() -> serde_json::Value {
    let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
    let ag = home.join(".openclaw").join("workspace").join("AGENTS.md");
    let bootstrap = home
        .join(".openclaw")
        .join("workspace")
        .join("BOOTSTRAP.md");
    let ag_wired = ag.exists()
        && fs::read_to_string(&ag)
            .ok()
            .map(|content| content.contains("memd"))
            .unwrap_or(false);
    let bootstrap_wired = bootstrap.exists()
        && fs::read_to_string(&bootstrap)
            .ok()
            .map(|content| content.contains("memd"))
            .unwrap_or(false);
    serde_json::json!({
        "wired": ag_wired && bootstrap_wired,
        "agents": ag_wired,
        "bootstrap": bootstrap_wired,
    })
}

pub(crate) fn detect_opencode_memd_wiring() -> serde_json::Value {
    let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
    let config_dir = home.join(".config").join("opencode");
    let legacy_dir = home.join(".opencode");
    let config_files = [
        config_dir.join("opencode.json"),
        config_dir.join("settings.json"),
        legacy_dir.join("opencode.json"),
        legacy_dir.join("settings.json"),
    ];
    let config_exists = config_files.iter().any(|path| path.is_file());
    let config_wired = config_files.iter().any(|path| {
        path.is_file()
            && fs::read_to_string(path)
                .ok()
                .map(|content| content.contains("memd-plugin") || content.contains("\"plugin\""))
                .unwrap_or(false)
    });
    let plugin_files = [
        config_dir.join("plugins").join("memd-plugin.mjs"),
        legacy_dir.join("plugins").join("memd-plugin.mjs"),
    ];
    let plugin_wired = plugin_files.iter().any(|path| {
        path.is_file()
            && fs::read_to_string(path)
                .ok()
                .map(|content| content.contains("MEMD_MEMORY.md"))
                .unwrap_or(false)
    });
    let command_files = [
        config_dir.join("command").join("memd.md"),
        legacy_dir.join("command").join("memd.md"),
    ];
    let command_wired = command_files.iter().any(|path| {
        path.is_file()
            && fs::read_to_string(path)
                .ok()
                .map(|content| content.contains("memd refresh") || content.contains("memd init"))
                .unwrap_or(false)
    });
    serde_json::json!({
        "wired": config_wired && plugin_wired && command_wired,
        "config": config_wired || config_exists,
        "plugin": plugin_wired,
        "command": command_wired,
    })
}

pub(crate) fn detect_claw_memd_wiring() -> serde_json::Value {
    let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
    let config_candidates = [
        home.join(".config").join("claw").join("settings.json"),
        home.join(".claw").join("settings.json"),
        home.join(".claw.json"),
    ];
    let config_exists = config_candidates.iter().any(|path| path.is_file());
    let binary_exists = Command::new("sh")
        .arg("-lc")
        .arg("command -v claw >/dev/null 2>&1")
        .status()
        .map(|status| status.success())
        .unwrap_or(false);
    let memd_skill_visible = [
        home.join(".claw")
            .join("skills")
            .join("memd")
            .join("SKILL.md"),
        home.join(".agents")
            .join("skills")
            .join("memd")
            .join("SKILL.md"),
        home.join(".codex")
            .join("skills")
            .join("memd")
            .join("SKILL.md"),
    ]
    .iter()
    .any(|path| path.is_file());
    serde_json::json!({
        "wired": binary_exists && config_exists && memd_skill_visible,
        "binary": binary_exists,
        "config": config_exists,
        "skill": memd_skill_visible,
    })
}

pub(crate) fn read_bundle_rag_config(
    output: &Path,
) -> anyhow::Result<Option<BundleRagConfigState>> {
    let config_path = output.join("config.json");
    let resolved = if config_path.exists() {
        let raw = fs::read_to_string(&config_path)
            .with_context(|| format!("read {}", config_path.display()))?;
        let config: BundleConfigFile = serde_json::from_str(&raw)
            .with_context(|| format!("parse {}", config_path.display()))?;
        resolve_bundle_rag_config(config)
    } else {
        None
    };

    if let Some(state) = resolved.as_ref() {
        if state.url.is_some() {
            return Ok(Some(state.clone()));
        }
    }

    if let Ok(value) = std::env::var("MEMD_RAG_URL") {
        let value = value.trim().to_string();
        if !value.is_empty() {
            return Ok(Some(BundleRagConfigState {
                configured: true,
                enabled: true,
                url: Some(value),
                source: "env.MEMD_RAG_URL".to_string(),
            }));
        }
    }

    Ok(resolved)
}

pub(crate) fn read_bundle_runtime_config_raw(
    output: &Path,
) -> anyhow::Result<Option<BundleRuntimeConfig>> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let config: BundleConfigFile =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    Ok(Some(BundleRuntimeConfig {
        project: config.project,
        namespace: config.namespace,
        agent: config.agent,
        session: config.session,
        tab_id: config.tab_id.or_else(default_bundle_tab_id),
        hive_system: config.hive_system,
        hive_role: config.hive_role,
        capabilities: config.capabilities,
        hive_groups: config.hive_groups,
        hive_group_goal: config.hive_group_goal,
        authority: config.authority,
        hive_project_enabled: config.hive_project_enabled,
        hive_project_anchor: config.hive_project_anchor,
        hive_project_joined_at: config.hive_project_joined_at,
        base_url: config.base_url,
        route: config.route,
        intent: config.intent,
        workspace: config.workspace,
        visibility: config.visibility,
        heartbeat_model: config.heartbeat_model,
        voice_mode: Some(config.voice_mode.unwrap_or_else(default_voice_mode)),
        auto_short_term_capture: config.auto_short_term_capture,
        authority_policy: config.authority_policy,
        authority_state: config.authority_state,
    }))
}

pub(crate) fn resolve_project_bundle_overlay(
    output: &Path,
    current_dir: &Path,
    global_root: &Path,
) -> anyhow::Result<Option<BundleRuntimeConfig>> {
    let output = fs::canonicalize(output).unwrap_or_else(|_| output.to_path_buf());
    let global_root = fs::canonicalize(global_root).unwrap_or_else(|_| global_root.to_path_buf());
    if output != global_root {
        return Ok(None);
    }

    let local_bundle = current_dir.join(".memd");
    let local_bundle = fs::canonicalize(&local_bundle).unwrap_or(local_bundle);
    if local_bundle == output {
        return Ok(None);
    }

    read_bundle_runtime_config_raw(&local_bundle)
}

pub(crate) fn resolve_live_session_overlay(
    output: &Path,
    current_dir: &Path,
    global_root: &Path,
) -> anyhow::Result<Option<BundleRuntimeConfig>> {
    let output = fs::canonicalize(output).unwrap_or_else(|_| output.to_path_buf());
    let global_root = fs::canonicalize(global_root).unwrap_or_else(|_| global_root.to_path_buf());
    if output == global_root {
        return Ok(None);
    }

    let local_bundle = current_dir.join(".memd");
    let local_bundle = fs::canonicalize(&local_bundle).unwrap_or(local_bundle);
    if local_bundle != output {
        return Ok(None);
    }

    let Some(local_runtime) = read_bundle_runtime_config_raw(&local_bundle)? else {
        return Ok(None);
    };

    let local_workspace_scoped = local_runtime
        .workspace
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());
    let local_visibility_scoped = matches!(
        local_runtime
            .visibility
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
        Some("workspace" | "project")
    );
    if !local_runtime.hive_project_enabled && !local_workspace_scoped && !local_visibility_scoped {
        return Ok(None);
    }

    let Some(global_runtime) = read_bundle_runtime_config_raw(&global_root)? else {
        return Ok(None);
    };

    if global_runtime.session.is_none() && global_runtime.tab_id.is_none() {
        return Ok(None);
    }

    Ok(Some(BundleRuntimeConfig {
        project: None,
        namespace: None,
        agent: None,
        session: global_runtime.session,
        tab_id: global_runtime.tab_id,
        hive_system: None,
        hive_role: None,
        capabilities: Vec::new(),
        hive_groups: Vec::new(),
        hive_group_goal: None,
        authority: None,
        hive_project_enabled: false,
        hive_project_anchor: None,
        hive_project_joined_at: None,
        base_url: None,
        route: None,
        intent: None,
        workspace: None,
        visibility: None,
        heartbeat_model: None,
        voice_mode: Some(default_voice_mode()),
        auto_short_term_capture: false,
        authority_policy: BundleAuthorityPolicy::default(),
        authority_state: BundleAuthorityState::default(),
    }))
}

pub(crate) fn merge_bundle_runtime_config(
    mut runtime: BundleRuntimeConfig,
    overlay: BundleRuntimeConfig,
) -> BundleRuntimeConfig {
    if overlay.project.is_some() {
        runtime.project = overlay.project;
    }
    if overlay.namespace.is_some() {
        runtime.namespace = overlay.namespace;
    }
    if overlay.workspace.is_some() {
        runtime.workspace = overlay.workspace;
    }
    if overlay.visibility.is_some() {
        runtime.visibility = overlay.visibility;
    }
    if overlay.session.is_some() {
        runtime.session = overlay.session;
    }
    if overlay.route.is_some() {
        runtime.route = overlay.route;
    }
    if overlay.intent.is_some() {
        runtime.intent = overlay.intent;
    }
    if overlay.tab_id.is_some() {
        runtime.tab_id = overlay.tab_id;
    }
    if overlay.hive_system.is_some() {
        runtime.hive_system = overlay.hive_system;
    }
    if overlay.hive_role.is_some() {
        runtime.hive_role = overlay.hive_role;
    }
    if !overlay.capabilities.is_empty() {
        runtime.capabilities = overlay.capabilities;
    }
    if !overlay.hive_groups.is_empty() {
        runtime.hive_groups = overlay.hive_groups;
    }
    if overlay.hive_group_goal.is_some() {
        runtime.hive_group_goal = overlay.hive_group_goal;
    }
    if overlay.authority.is_some() {
        runtime.authority = overlay.authority;
    }
    runtime.hive_project_enabled = overlay.hive_project_enabled;
    if overlay.hive_project_anchor.is_some() {
        runtime.hive_project_anchor = overlay.hive_project_anchor;
    }
    if overlay.hive_project_joined_at.is_some() {
        runtime.hive_project_joined_at = overlay.hive_project_joined_at;
    }
    runtime
}

pub(crate) fn read_bundle_runtime_config(
    output: &Path,
) -> anyhow::Result<Option<BundleRuntimeConfig>> {
    let Some(mut runtime) = read_bundle_runtime_config_raw(output)? else {
        return Ok(None);
    };

    let current_dir = std::env::current_dir().context("read current directory")?;
    if let Some(overlay) =
        resolve_project_bundle_overlay(output, &current_dir, &default_global_bundle_root())?
    {
        runtime = merge_bundle_runtime_config(runtime, overlay);
    }
    if let Some(overlay) =
        resolve_live_session_overlay(output, &current_dir, &default_global_bundle_root())?
    {
        runtime = merge_bundle_runtime_config(runtime, overlay);
    }

    Ok(Some(runtime))
}

pub(crate) fn resolve_bundle_command_base_url(
    requested: &str,
    runtime_base_url: Option<&str>,
) -> String {
    let requested = requested.trim();
    if std::env::var("MEMD_BASE_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .as_deref()
        == Some(requested)
    {
        return requested.to_string();
    }

    if requested != default_base_url() {
        return requested.to_string();
    }

    runtime_base_url
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| requested.to_string())
}

pub(crate) fn runtime_prefers_shared_authority(runtime: Option<&BundleRuntimeConfig>) -> bool {
    runtime
        .map(|value| value.authority_policy.shared_primary)
        .unwrap_or(true)
}

pub(crate) fn runtime_allows_localhost_read_only(runtime: Option<&BundleRuntimeConfig>) -> bool {
    runtime
        .map(|value| {
            value.authority_policy.localhost_fallback_policy
                == LocalhostFallbackPolicy::AllowReadOnly
        })
        .unwrap_or(false)
}

pub(crate) fn authority_warning_lines(runtime: Option<&BundleRuntimeConfig>) -> Vec<String> {
    let Some(runtime) = runtime else {
        return Vec::new();
    };
    if runtime.authority_state.mode != "localhost_read_only" {
        return Vec::new();
    }

    let mut lines = vec![
        "shared authority unavailable".to_string(),
        "localhost fallback is lower trust".to_string(),
        "prompt-injection and split-brain risk increased".to_string(),
        "coordination writes blocked".to_string(),
    ];
    if let Some(reason) = runtime.authority_state.reason.as_deref() {
        lines.push(format!("reason={reason}"));
    }
    if let Some(expires_at) = runtime.authority_state.expires_at.as_ref() {
        lines.push(format!("expires_at={}", expires_at.to_rfc3339()));
    }
    lines
}

pub(crate) fn write_bundle_authority_env(
    output: &Path,
    policy: &BundleAuthorityPolicy,
    state: &BundleAuthorityState,
) -> anyhow::Result<()> {
    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_AUTHORITY_MODE=",
        &format!("MEMD_AUTHORITY_MODE={}\n", state.mode),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_AUTHORITY_MODE = ",
        &format!(
            "$env:MEMD_AUTHORITY_MODE = \"{}\"\n",
            escape_ps1(&state.mode)
        ),
    )?;
    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_LOCALHOST_FALLBACK_POLICY=",
        &format!(
            "MEMD_LOCALHOST_FALLBACK_POLICY={}\n",
            policy.localhost_fallback_policy.as_str()
        ),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_LOCALHOST_FALLBACK_POLICY = ",
        &format!(
            "$env:MEMD_LOCALHOST_FALLBACK_POLICY = \"{}\"\n",
            escape_ps1(policy.localhost_fallback_policy.as_str())
        ),
    )?;
    rewrite_env_assignment(
        &output.join("env"),
        "MEMD_AUTHORITY_DEGRADED=",
        &format!(
            "MEMD_AUTHORITY_DEGRADED={}\n",
            if state.degraded { "true" } else { "false" }
        ),
    )?;
    rewrite_env_assignment(
        &output.join("env.ps1"),
        "$env:MEMD_AUTHORITY_DEGRADED = ",
        &format!(
            "$env:MEMD_AUTHORITY_DEGRADED = \"{}\"\n",
            if state.degraded { "true" } else { "false" }
        ),
    )?;
    if let Some(shared_base_url) = state.shared_base_url.as_deref() {
        rewrite_env_assignment(
            &output.join("env"),
            "MEMD_SHARED_BASE_URL=",
            &format!("MEMD_SHARED_BASE_URL={shared_base_url}\n"),
        )?;
        rewrite_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_SHARED_BASE_URL = ",
            &format!(
                "$env:MEMD_SHARED_BASE_URL = \"{}\"\n",
                escape_ps1(shared_base_url)
            ),
        )?;
    } else {
        remove_env_assignment(&output.join("env"), "MEMD_SHARED_BASE_URL=")?;
        remove_env_assignment(&output.join("env.ps1"), "$env:MEMD_SHARED_BASE_URL = ")?;
    }
    if let Some(fallback_base_url) = state.fallback_base_url.as_deref() {
        rewrite_env_assignment(
            &output.join("env"),
            "MEMD_LOCALHOST_FALLBACK_BASE_URL=",
            &format!("MEMD_LOCALHOST_FALLBACK_BASE_URL={fallback_base_url}\n"),
        )?;
        rewrite_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_LOCALHOST_FALLBACK_BASE_URL = ",
            &format!(
                "$env:MEMD_LOCALHOST_FALLBACK_BASE_URL = \"{}\"\n",
                escape_ps1(fallback_base_url)
            ),
        )?;
    } else {
        remove_env_assignment(&output.join("env"), "MEMD_LOCALHOST_FALLBACK_BASE_URL=")?;
        remove_env_assignment(
            &output.join("env.ps1"),
            "$env:MEMD_LOCALHOST_FALLBACK_BASE_URL = ",
        )?;
    }
    Ok(())
}

pub(crate) fn set_bundle_shared_authority_state(
    output: &Path,
    shared_base_url: &str,
    activated_by: &str,
    reason: &str,
) -> anyhow::Result<()> {
    let (config_path, mut config) = read_bundle_config_file(output)?;
    config.authority_state.mode = "shared".to_string();
    config.authority_state.degraded = false;
    config.authority_state.shared_base_url = Some(shared_base_url.to_string());
    config.authority_state.fallback_base_url = None;
    config.authority_state.activated_at = Some(Utc::now());
    config.authority_state.activated_by = Some(activated_by.to_string());
    config.authority_state.reason = Some(reason.to_string());
    config.authority_state.warning_acknowledged_at = None;
    config.authority_state.expires_at = None;
    config.authority_state.blocked_capabilities.clear();
    write_bundle_config_file(&config_path, &config)?;
    write_bundle_authority_env(output, &config.authority_policy, &config.authority_state)?;
    Ok(())
}

pub(crate) fn set_bundle_localhost_read_only_authority_state(
    output: &Path,
    shared_base_url: &str,
    activated_by: &str,
    reason: &str,
) -> anyhow::Result<()> {
    let (config_path, mut config) = read_bundle_config_file(output)?;
    config.authority_policy.shared_primary = true;
    config.authority_policy.localhost_fallback_policy = LocalhostFallbackPolicy::AllowReadOnly;
    config.authority_state.mode = "localhost_read_only".to_string();
    config.authority_state.degraded = true;
    config.authority_state.shared_base_url = Some(shared_base_url.to_string());
    config.authority_state.fallback_base_url = Some(localhost_memd_base_url());
    config.authority_state.activated_at = Some(Utc::now());
    config.authority_state.activated_by = Some(activated_by.to_string());
    config.authority_state.reason = Some(reason.to_string());
    config.authority_state.warning_acknowledged_at = None;
    config.authority_state.expires_at = None;
    config.authority_state.blocked_capabilities = vec![
        "coordination_writes".to_string(),
        "queen_actions".to_string(),
        "shared_claim_mutations".to_string(),
        "shared_task_mutations".to_string(),
        "shared_message_mutations".to_string(),
    ];
    write_bundle_config_file(&config_path, &config)?;
    write_bundle_authority_env(output, &config.authority_policy, &config.authority_state)?;
    Ok(())
}

pub(crate) fn ensure_shared_authority_write_allowed(
    runtime: Option<&BundleRuntimeConfig>,
    operation: &str,
) -> anyhow::Result<()> {
    if runtime
        .map(|value| value.authority_state.mode.as_str() == "localhost_read_only")
        .unwrap_or(false)
    {
        anyhow::bail!(
            "localhost read-only fallback active; {} requires trusted shared authority",
            operation
        );
    }
    Ok(())
}

const LIVE_RPC_TIMEOUT: Duration = Duration::from_secs(2);

pub(crate) async fn timeout_ok<T, E, F>(future: F) -> Option<T>
where
    F: Future<Output = Result<T, E>>,
{
    tokio::time::timeout(LIVE_RPC_TIMEOUT, future)
        .await
        .ok()
        .and_then(Result::ok)
}

pub(crate) fn bundle_auto_short_term_capture_enabled(output: &Path) -> anyhow::Result<bool> {
    if let Ok(value) = std::env::var("MEMD_AUTO_SHORT_TERM_CAPTURE") {
        let value = value.trim().to_ascii_lowercase();
        return Ok(matches!(value.as_str(), "1" | "true" | "yes" | "on"));
    }

    Ok(read_bundle_runtime_config(output)?
        .map(|config| config.auto_short_term_capture)
        .unwrap_or(true))
}

pub(crate) fn resolve_awareness_paths(
    args: &AwarenessArgs,
) -> anyhow::Result<(PathBuf, PathBuf, PathBuf)> {
    let current_bundle = if args.output.is_absolute() {
        args.output.clone()
    } else {
        std::env::current_dir()?.join(&args.output)
    };
    let current_bundle = fs::canonicalize(&current_bundle).unwrap_or(current_bundle);
    let current_project = current_bundle
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let scan_root = if let Some(root) = args.root.as_ref() {
        if root.is_absolute() {
            root.clone()
        } else {
            std::env::current_dir()?.join(root)
        }
    } else {
        current_project
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| current_project.clone())
    };
    let scan_root = fs::canonicalize(&scan_root).unwrap_or(scan_root);

    Ok((current_bundle, current_project, scan_root))
}

pub(crate) async fn read_project_awareness(
    args: &AwarenessArgs,
) -> anyhow::Result<ProjectAwarenessResponse> {
    let (current_bundle, _, _) = resolve_awareness_paths(args)?;
    let runtime = read_bundle_runtime_config(&current_bundle)?;
    if runtime
        .as_ref()
        .and_then(|config| config.session.as_deref())
        .is_some()
    {
        let _ = timeout_ok(refresh_bundle_heartbeat(&current_bundle, None, false)).await;
    }
    let include_current = args.include_current
        || runtime
            .as_ref()
            .and_then(|config| config.session.as_deref())
            .is_some();
    let local = read_project_awareness_local(&AwarenessArgs {
        include_current,
        ..args.clone()
    })?;
    if let Some(shared) = read_project_awareness_shared(
        &AwarenessArgs {
            include_current: args.include_current,
            ..args.clone()
        },
        &local,
    )
    .await?
    {
        let mut entries = merge_project_awareness_entries(local.entries, shared.entries);
        entries = suppress_superseded_awareness_entries(entries, &local.current_bundle);
        entries.sort_by(|left, right| left.bundle_root.cmp(&right.bundle_root));

        let mut collisions = local.collisions;
        for collision in shared.collisions {
            if !collisions.contains(&collision) {
                collisions.push(collision);
            }
        }

        return Ok(ProjectAwarenessResponse {
            root: shared.root,
            current_bundle: local.current_bundle,
            collisions,
            entries,
        });
    }
    Ok(local)
}

pub(crate) fn merge_project_awareness_entries(
    local_entries: Vec<ProjectAwarenessEntry>,
    shared_entries: Vec<ProjectAwarenessEntry>,
) -> Vec<ProjectAwarenessEntry> {
    let mut entries = local_entries;
    for entry in shared_entries {
        if let Some(index) = entries
            .iter()
            .position(|candidate| awareness_entries_overlap(candidate, &entry))
        {
            let preferred = prefer_project_awareness_entry(entries[index].clone(), entry);
            entries[index] = preferred;
        } else {
            entries.push(entry);
        }
    }
    entries
}

pub(crate) fn suppress_superseded_awareness_entries(
    entries: Vec<ProjectAwarenessEntry>,
    current_bundle: &str,
) -> Vec<ProjectAwarenessEntry> {
    let Some(current) = entries
        .iter()
        .find(|entry| entry.bundle_root == current_bundle && entry.presence == "active")
        .cloned()
    else {
        return entries;
    };

    entries
        .into_iter()
        .filter(|entry| !is_superseded_stale_remote_session(entry, &current))
        .collect()
}

pub(crate) fn is_superseded_stale_remote_session(
    entry: &ProjectAwarenessEntry,
    current: &ProjectAwarenessEntry,
) -> bool {
    entry.project_dir == "remote"
        && entry.presence == "stale"
        && current.presence == "active"
        && entry.session != current.session
        && entry.project == current.project
        && entry.namespace == current.namespace
        && entry.workspace == current.workspace
        && entry.agent == current.agent
        && entry.base_url == current.base_url
}

pub(crate) fn project_awareness_visible_entries<'a>(
    response: &'a ProjectAwarenessResponse,
) -> Vec<&'a ProjectAwarenessEntry> {
    let current_entry = response
        .entries
        .iter()
        .find(|candidate| candidate.bundle_root == response.current_bundle);
    response
        .entries
        .iter()
        .filter(|entry| !(entry.project_dir == "remote" && entry.presence == "dead"))
        .filter(|entry| {
            !current_entry
                .map(|current| is_superseded_stale_remote_session(entry, current))
                .unwrap_or(false)
        })
        .filter(|entry| {
            !current_entry
                .map(|current| is_shadowed_local_seen_session(entry, current))
                .unwrap_or(false)
        })
        .collect()
}

pub(crate) fn is_shadowed_local_seen_session(
    entry: &ProjectAwarenessEntry,
    current: &ProjectAwarenessEntry,
) -> bool {
    let entry_is_shadow_candidate = matches!(entry.presence.as_str(), "unknown" | "active");
    if !entry_is_shadow_candidate {
        return false;
    }
    let current_is_newer = match (entry.last_updated, current.last_updated) {
        (Some(entry_updated), Some(current_updated)) => current_updated > entry_updated,
        _ => false,
    };
    entry.bundle_root != current.bundle_root
        && entry.project_dir != "remote"
        && current.presence == "active"
        && current_is_newer
        && entry.project == current.project
        && entry.namespace == current.namespace
        && entry.workspace == current.workspace
        && entry.agent == current.agent
        && entry.base_url == current.base_url
        && entry.active_claims == 0
        && entry.hive_system.is_none()
        && entry.hive_role.is_none()
        && entry.hive_groups.is_empty()
        && entry.branch.is_none()
}

pub(crate) fn awareness_entries_overlap(
    left: &ProjectAwarenessEntry,
    right: &ProjectAwarenessEntry,
) -> bool {
    let same_session = left.session.is_some()
        && left.session == right.session
        && left.project == right.project
        && left.namespace == right.namespace
        && left.workspace == right.workspace
        && left.branch == right.branch
        && left.worktree_root == right.worktree_root;
    if same_session {
        return left.tab_id == right.tab_id || left.tab_id.is_none() || right.tab_id.is_none();
    }

    left.bundle_root == right.bundle_root
        || (left.base_url.is_some()
            && left.base_url == right.base_url
            && left.project == right.project
            && left.namespace == right.namespace
            && left.session == right.session)
}

pub(crate) fn prefer_project_awareness_entry(
    left: ProjectAwarenessEntry,
    right: ProjectAwarenessEntry,
) -> ProjectAwarenessEntry {
    if project_awareness_entry_rank(&right) > project_awareness_entry_rank(&left) {
        right
    } else {
        left
    }
}

pub(crate) fn project_awareness_entry_rank(
    entry: &ProjectAwarenessEntry,
) -> (u8, u8, u8, u8, i64, usize, usize) {
    (
        if entry.project_dir == "remote" { 0 } else { 1 },
        awareness_presence_rank(&entry.presence),
        if entry.hive_system.is_some() || entry.hive_role.is_some() {
            1
        } else {
            0
        },
        if entry.hive_groups.is_empty() { 0 } else { 1 },
        entry
            .last_updated
            .map(|value| value.timestamp())
            .unwrap_or_default(),
        entry.active_claims,
        entry.capabilities.len(),
    )
}

pub(crate) fn awareness_presence_rank(presence: &str) -> u8 {
    match presence {
        "active" => 3,
        "stale" => 2,
        "dead" => 1,
        _ => 0,
    }
}

pub(crate) fn prune_dead_local_bundle_heartbeat(
    bundle_root: &Path,
    heartbeat: Option<&BundleHeartbeatState>,
    active_claims: usize,
    is_current_bundle: bool,
    current_runtime: Option<&BundleRuntimeConfig>,
) -> anyhow::Result<bool> {
    if is_current_bundle || active_claims > 0 {
        return Ok(false);
    }
    let Some(heartbeat) = heartbeat else {
        return Ok(false);
    };
    if heartbeat_presence_label(heartbeat.last_seen) != "dead" {
        return Ok(false);
    }
    let shares_current_session = current_runtime
        .and_then(|runtime| runtime.session.as_deref())
        .zip(heartbeat.session.as_deref())
        .is_some_and(|(current_session, heartbeat_session)| current_session == heartbeat_session);
    let shares_current_tab = current_runtime
        .and_then(|runtime| runtime.tab_id.as_deref())
        .zip(heartbeat.tab_id.as_deref())
        .is_some_and(|(current_tab, heartbeat_tab)| current_tab == heartbeat_tab);
    if shares_current_session || shares_current_tab {
        return Ok(false);
    }

    let heartbeat_path = bundle_heartbeat_state_path(bundle_root);
    if heartbeat_path.exists() {
        fs::remove_file(&heartbeat_path)
            .with_context(|| format!("remove {}", heartbeat_path.display()))?;
    }
    Ok(true)
}

pub(crate) fn skip_inactive_local_bundle_entry(
    runtime: &BundleRuntimeConfig,
    heartbeat: Option<&BundleHeartbeatState>,
    state: Option<&BundleResumeState>,
    active_claims: usize,
    is_current_bundle: bool,
) -> bool {
    !is_current_bundle
        && active_claims == 0
        && heartbeat.is_none()
        && state.is_none()
        && runtime
            .session
            .as_deref()
            .map(str::trim)
            .is_none_or(|value| value.is_empty())
}

pub(crate) async fn read_project_awareness_shared(
    args: &AwarenessArgs,
    fallback: &ProjectAwarenessResponse,
) -> anyhow::Result<Option<ProjectAwarenessResponse>> {
    let (current_bundle, _, _) = resolve_awareness_paths(args)?;
    let runtime = read_bundle_runtime_config(&current_bundle)?;
    let Some(base_url) = resolve_project_hive_base_url(
        runtime.as_ref(),
        runtime
            .as_ref()
            .and_then(|config| config.base_url.as_deref()),
    ) else {
        return Ok(None);
    };

    let client = match MemdClient::new(&base_url) {
        Ok(client) => client,
        Err(_) => return Ok(None),
    };
    let workspace = runtime.as_ref().and_then(|config| config.workspace.clone());
    let (shared_project, shared_namespace) = shared_awareness_scope(runtime.as_ref());

    let sessions_request = memd_schema::HiveSessionsRequest {
        session: None,
        project: shared_project.clone(),
        namespace: shared_namespace.clone(),
        repo_root: None,
        worktree_root: None,
        branch: None,
        workspace: workspace.clone(),
        hive_system: None,
        hive_role: None,
        host: None,
        hive_group: None,
        active_only: Some(false),
        limit: Some(512),
    };
    let sessions = match timeout_ok(client.hive_sessions(&sessions_request)).await {
        Some(response) => response.sessions,
        None => return Ok(None),
    };
    if sessions.is_empty() {
        return Ok(None);
    }

    let claims_request = HiveClaimsRequest {
        session: None,
        project: shared_project,
        namespace: shared_namespace,
        workspace,
        active_only: Some(true),
        limit: Some(512),
    };
    let claims = timeout_ok(client.hive_claims(&claims_request))
        .await
        .map(|response| response.claims)
        .unwrap_or_default();
    let current_session = runtime
        .as_ref()
        .and_then(|config| config.session.as_deref());

    let mut entries = Vec::new();
    let mut base_url_counts = std::collections::BTreeMap::<String, usize>::new();
    for session in sessions {
        if !args.include_current && current_session == Some(session.session.as_str()) {
            continue;
        }

        let active_claims = claims
            .iter()
            .filter(|claim| claim.session == session.session && claim.expires_at > Utc::now())
            .count();
        let entry = ProjectAwarenessEntry {
            project_dir: "remote".to_string(),
            bundle_root: format!("remote:{}:{}", base_url, session.session),
            project: session.project.clone(),
            namespace: session.namespace.clone(),
            repo_root: session.repo_root.clone(),
            worktree_root: session.worktree_root.clone(),
            branch: session.branch.clone(),
            base_branch: session.base_branch.clone(),
            agent: session.agent.clone(),
            session: Some(session.session.clone()),
            tab_id: session.tab_id.clone(),
            effective_agent: session.effective_agent.clone(),
            hive_system: session.hive_system.clone(),
            hive_role: session.hive_role.clone(),
            capabilities: session.capabilities.clone(),
            hive_groups: session.hive_groups.clone(),
            hive_group_goal: session.hive_group_goal.clone(),
            authority: session.authority.clone(),
            base_url: session.base_url.clone(),
            presence: heartbeat_presence_label(session.last_seen).to_string(),
            host: session.host.clone(),
            pid: session.pid,
            active_claims,
            workspace: session.workspace.clone(),
            visibility: session.visibility.clone(),
            topic_claim: session.topic_claim.clone(),
            scope_claims: session.scope_claims.clone(),
            task_id: session.task_id.clone(),
            focus: session.focus.clone(),
            pressure: session.pressure.clone(),
            next_recovery: session.next_recovery.clone(),
            last_updated: Some(session.last_seen),
        };
        if let Some(url) = entry
            .base_url
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        {
            *base_url_counts.entry(url).or_insert(0) += 1;
        }
        entries.push(entry);
    }

    entries.sort_by(|left, right| left.bundle_root.cmp(&right.bundle_root));
    let mut collisions = base_url_counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(url, count)| format!("base_url {} used by {} bundles", url, count))
        .collect::<Vec<_>>();
    collisions.extend(session_collision_warnings(&entries));
    let root = if let Some(workspace) = runtime.and_then(|config| config.workspace.clone()) {
        format!("server:{base_url} workspace:{workspace}")
    } else {
        format!("server:{base_url}")
    };

    Ok(Some(ProjectAwarenessResponse {
        root,
        current_bundle: fallback.current_bundle.clone(),
        collisions,
        entries,
    }))
}

pub(crate) fn shared_awareness_scope(
    runtime: Option<&BundleRuntimeConfig>,
) -> (Option<String>, Option<String>) {
    let project = runtime.and_then(|config| config.project.clone());
    let namespace = runtime.and_then(|config| config.namespace.clone());
    let workspace = runtime.and_then(|config| config.workspace.clone());
    if workspace.is_some() {
        (None, None)
    } else {
        (project, namespace)
    }
}

pub(crate) fn session_collision_warnings(entries: &[ProjectAwarenessEntry]) -> Vec<String> {
    let mut groups = std::collections::BTreeMap::<
        (Option<String>, Option<String>, Option<String>),
        Vec<&ProjectAwarenessEntry>,
    >::new();
    for entry in entries {
        groups
            .entry((
                entry.workspace.clone(),
                entry.session.clone(),
                entry.tab_id.clone(),
            ))
            .or_default()
            .push(entry);
    }

    groups
        .into_iter()
        .filter_map(|((workspace, session, tab_id), group)| {
            if group.len() <= 1 {
                return None;
            }

            let mut agents = std::collections::BTreeSet::new();
            let mut urls = std::collections::BTreeSet::new();
            for entry in &group {
                agents.insert(
                    entry
                        .effective_agent
                        .as_deref()
                        .or(entry.agent.as_deref())
                        .unwrap_or("none")
                        .to_string(),
                );
                urls.insert(entry.base_url.as_deref().unwrap_or("none").to_string());
            }

            if agents.len() <= 1 && urls.len() <= 1 {
                return None;
            }

            Some(format!(
                "session {} tab {} in workspace {} seen across {} bundles / {} agents / {} endpoints",
                session.as_deref().unwrap_or("none"),
                tab_id.as_deref().unwrap_or("none"),
                workspace.as_deref().unwrap_or("none"),
                group.len(),
                agents.len(),
                urls.len()
            ))
        })
        .collect()
}

pub(crate) fn read_project_awareness_local(
    args: &AwarenessArgs,
) -> anyhow::Result<ProjectAwarenessResponse> {
    let (current_bundle, _current_project, scan_root) = resolve_awareness_paths(args)?;
    let current_runtime = read_bundle_runtime_config(&current_bundle)?;

    let mut entries = Vec::new();
    let mut base_url_counts = std::collections::BTreeMap::<String, usize>::new();
    for child in fs::read_dir(&scan_root)
        .with_context(|| format!("read awareness root {}", scan_root.display()))?
    {
        let child = child?;
        if !child.file_type()?.is_dir() {
            continue;
        }

        let project_dir = child.path();
        let bundle_root = project_dir.join(".memd");
        let config_path = bundle_root.join("config.json");
        if !config_path.exists() {
            continue;
        }

        let canonical_bundle = fs::canonicalize(&bundle_root).unwrap_or(bundle_root.clone());
        if !args.include_current && canonical_bundle == current_bundle {
            continue;
        }

        let runtime = read_bundle_runtime_config(&bundle_root)?.unwrap_or(BundleRuntimeConfig {
            project: None,
            namespace: None,
            agent: None,
            session: None,
            tab_id: None,
            hive_system: None,
            hive_role: None,
            capabilities: Vec::new(),
            hive_groups: Vec::new(),
            hive_group_goal: None,
            authority: None,
            hive_project_enabled: false,
            hive_project_anchor: None,
            hive_project_joined_at: None,
            base_url: None,
            route: None,
            intent: None,
            workspace: None,
            visibility: None,
            heartbeat_model: Some(default_heartbeat_model()),
            voice_mode: Some(default_voice_mode()),
            auto_short_term_capture: true,
            authority_policy: BundleAuthorityPolicy::default(),
            authority_state: BundleAuthorityState::default(),
        });
        let state = read_bundle_resume_state(&bundle_root)?;
        let heartbeat = read_bundle_heartbeat(&bundle_root)?;
        let claims = read_bundle_claims(&bundle_root)?;
        let active_claims = claims
            .claims
            .iter()
            .filter(|claim| claim.expires_at > Utc::now())
            .count();
        if prune_dead_local_bundle_heartbeat(
            &bundle_root,
            heartbeat.as_ref(),
            active_claims,
            canonical_bundle == current_bundle,
            current_runtime.as_ref(),
        )? {
            continue;
        }
        if skip_inactive_local_bundle_entry(
            &runtime,
            heartbeat.as_ref(),
            state.as_ref(),
            active_claims,
            canonical_bundle == current_bundle,
        ) {
            continue;
        }
        let state_path = bundle_resume_state_path(&bundle_root);
        let heartbeat_path = bundle_heartbeat_state_path(&bundle_root);
        let last_updated = if heartbeat_path.exists() {
            fs::metadata(&heartbeat_path)
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .map(DateTime::<Utc>::from)
        } else if state_path.exists() {
            fs::metadata(&state_path)
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .map(DateTime::<Utc>::from)
        } else {
            fs::metadata(&config_path)
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .map(DateTime::<Utc>::from)
        };

        entries.push(ProjectAwarenessEntry {
            project_dir: project_dir.display().to_string(),
            bundle_root: bundle_root.display().to_string(),
            project: runtime.project,
            namespace: runtime.namespace,
            repo_root: heartbeat.as_ref().and_then(|value| value.repo_root.clone()),
            worktree_root: heartbeat
                .as_ref()
                .and_then(|value| value.worktree_root.clone())
                .or_else(|| Some(project_dir.display().to_string())),
            branch: heartbeat.as_ref().and_then(|value| value.branch.clone()),
            base_branch: heartbeat
                .as_ref()
                .and_then(|value| value.base_branch.clone()),
            tab_id: heartbeat
                .as_ref()
                .and_then(|value| value.tab_id.clone())
                .or(runtime.tab_id),
            effective_agent: runtime
                .agent
                .as_deref()
                .map(|agent| compose_agent_identity(agent, runtime.session.as_deref())),
            agent: runtime.agent,
            session: runtime.session,
            hive_system: heartbeat
                .as_ref()
                .and_then(|value| value.hive_system.clone())
                .or(runtime.hive_system),
            hive_role: heartbeat
                .as_ref()
                .and_then(|value| value.hive_role.clone())
                .or(runtime.hive_role),
            capabilities: heartbeat
                .as_ref()
                .map(|value| value.capabilities.clone())
                .filter(|value| !value.is_empty())
                .unwrap_or(runtime.capabilities),
            hive_groups: heartbeat
                .as_ref()
                .map(|value| value.hive_groups.clone())
                .filter(|value| !value.is_empty())
                .unwrap_or(runtime.hive_groups),
            hive_group_goal: heartbeat
                .as_ref()
                .and_then(|value| value.hive_group_goal.clone())
                .or(runtime.hive_group_goal),
            authority: heartbeat
                .as_ref()
                .and_then(|value| value.authority.clone())
                .or(runtime.authority),
            base_url: runtime.base_url.clone(),
            presence: heartbeat
                .as_ref()
                .map(|value| heartbeat_presence_label(value.last_seen).to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            host: heartbeat.as_ref().and_then(|value| value.host.clone()),
            pid: heartbeat.as_ref().and_then(|value| value.pid),
            active_claims,
            workspace: heartbeat
                .as_ref()
                .and_then(|value| value.workspace.clone())
                .or(runtime.workspace),
            visibility: heartbeat
                .as_ref()
                .and_then(|value| value.visibility.clone())
                .or(runtime.visibility),
            topic_claim: heartbeat
                .as_ref()
                .and_then(|value| value.topic_claim.clone()),
            scope_claims: heartbeat
                .as_ref()
                .map(|value| value.scope_claims.clone())
                .unwrap_or_default(),
            task_id: heartbeat.as_ref().and_then(|value| value.task_id.clone()),
            focus: heartbeat
                .as_ref()
                .and_then(|value| value.focus.clone())
                .or_else(|| state.as_ref().and_then(|value| value.focus.clone())),
            pressure: heartbeat
                .as_ref()
                .and_then(|value| value.pressure.clone())
                .or_else(|| state.as_ref().and_then(|value| value.pressure.clone())),
            next_recovery: heartbeat
                .as_ref()
                .and_then(|value| value.next_recovery.clone())
                .or_else(|| state.as_ref().and_then(|value| value.next_recovery.clone())),
            last_updated,
        });
        if let Some(url) = entries
            .last()
            .and_then(|entry| entry.base_url.as_ref())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        {
            *base_url_counts.entry(url).or_insert(0) += 1;
        }
    }

    entries.sort_by(|left, right| left.project_dir.cmp(&right.project_dir));
    let mut collisions = base_url_counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(url, count)| format!("base_url {} used by {} bundles", url, count))
        .collect::<Vec<_>>();
    collisions.extend(session_collision_warnings(&entries));

    Ok(ProjectAwarenessResponse {
        root: scan_root.display().to_string(),
        current_bundle: current_bundle.display().to_string(),
        collisions,
        entries,
    })
}

pub(crate) fn render_project_awareness_summary(response: &ProjectAwarenessResponse) -> String {
    let current_entry = response
        .entries
        .iter()
        .find(|candidate| candidate.bundle_root == response.current_bundle);
    let visible_entries = project_awareness_visible_entries(response);
    let hidden_remote_dead = response
        .entries
        .iter()
        .filter(|entry| {
            entry.project_dir == "remote"
                && entry.presence == "dead"
                && current_entry
                    .map(|current| {
                        entry.project == current.project
                            && entry.namespace == current.namespace
                            && entry.workspace == current.workspace
                            && entry.base_url == current.base_url
                    })
                    .unwrap_or(true)
        })
        .count();
    let superseded_stale_sessions = response
        .entries
        .iter()
        .filter(|entry| {
            current_entry
                .map(|current| is_superseded_stale_remote_session(entry, current))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    let superseded_stale_count = superseded_stale_sessions.len();
    let superseded_stale_session_ids = superseded_stale_sessions
        .iter()
        .filter_map(|entry| entry.session.as_deref())
        .take(3)
        .collect::<Vec<_>>();
    let superseded_stale_suffix = if superseded_stale_count > superseded_stale_session_ids.len() {
        format!(
            " +{}",
            superseded_stale_count - superseded_stale_session_ids.len()
        )
    } else {
        String::new()
    };
    let current_session = visible_entries
        .iter()
        .find(|entry| entry.bundle_root == response.current_bundle)
        .and_then(|entry| entry.session.as_deref());
    let stale_remote_sessions = visible_entries
        .iter()
        .filter(|entry| entry.project_dir == "remote" && entry.presence == "stale")
        .collect::<Vec<_>>();
    let active_hive_sessions = current_session
        .map(|current| {
            visible_entries
                .iter()
                .filter(|entry| entry.presence == "active")
                .filter(|entry| !entry.hive_groups.is_empty())
                .filter_map(|entry| entry.session.as_deref())
                .filter(|session| *session != current)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let rendered_diagnostics = awareness_summary_diagnostics(&visible_entries);
    let mut lines = vec![format!(
        "awareness root={} bundles={} diagnostics={} hidden_remote_dead={} hidden_superseded_stale={}",
        response.root,
        visible_entries.len(),
        rendered_diagnostics.len(),
        hidden_remote_dead,
        superseded_stale_count,
    )];
    if !active_hive_sessions.is_empty() {
        lines.push(format!(
            "! active_hive_sessions={} sessions={}",
            active_hive_sessions.len(),
            active_hive_sessions.join(",")
        ));
    }
    if !stale_remote_sessions.is_empty() {
        let sessions = stale_remote_sessions
            .iter()
            .take(3)
            .filter_map(|entry| entry.session.as_deref())
            .collect::<Vec<_>>();
        let suffix = if stale_remote_sessions.len() > sessions.len() {
            format!(" +{}", stale_remote_sessions.len() - sessions.len())
        } else {
            String::new()
        };
        lines.push(format!(
            "! stale_remote_sessions={} sessions={}{}",
            stale_remote_sessions.len(),
            if sessions.is_empty() {
                "unknown".to_string()
            } else {
                sessions.join(",")
            },
            suffix,
        ));
    }
    if superseded_stale_count > 0 {
        lines.push(format!(
            "! superseded_stale_sessions={} sessions={}{}",
            superseded_stale_count,
            if superseded_stale_session_ids.is_empty() {
                "unknown".to_string()
            } else {
                superseded_stale_session_ids.join(",")
            },
            superseded_stale_suffix,
        ));
    }
    for diagnostic in &rendered_diagnostics {
        lines.push(format!("! {}", diagnostic));
    }
    let current_entries = visible_entries
        .iter()
        .copied()
        .filter(|entry| entry.bundle_root == response.current_bundle)
        .collect::<Vec<_>>();
    let active_hive_entries = visible_entries
        .iter()
        .copied()
        .filter(|entry| entry.bundle_root != response.current_bundle)
        .filter(|entry| entry.presence == "active" && !entry.hive_groups.is_empty())
        .collect::<Vec<_>>();
    let stale_entries = visible_entries
        .iter()
        .copied()
        .filter(|entry| entry.presence == "stale")
        .collect::<Vec<_>>();
    let seen_entries = visible_entries
        .iter()
        .copied()
        .filter(|entry| entry.bundle_root != response.current_bundle)
        .filter(|entry| !(entry.presence == "active" && !entry.hive_groups.is_empty()))
        .filter(|entry| entry.presence != "stale")
        .filter(|entry| entry.presence != "dead")
        .collect::<Vec<_>>();
    let dead_entries = visible_entries
        .iter()
        .copied()
        .filter(|entry| entry.presence == "dead")
        .collect::<Vec<_>>();

    push_awareness_section(
        &mut lines,
        "current_session",
        &current_entries,
        "current",
        &response.current_bundle,
    );
    push_awareness_section(
        &mut lines,
        "active_hive_sessions",
        &active_hive_entries,
        "hive-session",
        &response.current_bundle,
    );
    push_awareness_section(
        &mut lines,
        "stale_sessions",
        &stale_entries,
        "stale-session",
        &response.current_bundle,
    );
    push_awareness_section(
        &mut lines,
        "dead_sessions",
        &dead_entries,
        "dead-session",
        &response.current_bundle,
    );
    push_awareness_section(
        &mut lines,
        "seen_sessions",
        &seen_entries,
        "seen",
        &response.current_bundle,
    );
    lines.join("\n")
}

pub(crate) fn awareness_summary_diagnostics(entries: &[&ProjectAwarenessEntry]) -> Vec<String> {
    let owned = entries
        .iter()
        .map(|entry| (*entry).clone())
        .collect::<Vec<_>>();
    let mut diagnostics = shared_endpoint_diagnostics(entries);
    diagnostics.extend(session_collision_warnings(&owned));
    diagnostics.extend(branch_collision_warnings(&owned));
    diagnostics.extend(work_overlap_warnings(entries));
    diagnostics
}

pub(crate) fn shared_endpoint_diagnostics(entries: &[&ProjectAwarenessEntry]) -> Vec<String> {
    let mut counts = std::collections::BTreeMap::<String, usize>::new();
    for entry in entries {
        if let Some(url) = entry
            .base_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            *counts.entry(url.to_string()).or_insert(0) += 1;
        }
    }

    counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(url, count)| format!("shared_hive_endpoint {} sessions={}", url, count))
        .collect()
}

pub(crate) fn branch_collision_warnings(entries: &[ProjectAwarenessEntry]) -> Vec<String> {
    let mut same_branch = std::collections::BTreeMap::<(String, String), Vec<String>>::new();
    let mut same_worktree = std::collections::BTreeMap::<String, Vec<String>>::new();

    for entry in entries {
        let lane = entry
            .session
            .clone()
            .or_else(|| entry.effective_agent.clone())
            .or_else(|| entry.agent.clone())
            .unwrap_or_else(|| entry.bundle_root.clone());

        if let (Some(repo_root), Some(branch)) = (
            entry
                .repo_root
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
            entry
                .branch
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        ) {
            same_branch
                .entry((repo_root.to_string(), branch.to_string()))
                .or_default()
                .push(lane.clone());
        }

        if let Some(worktree_root) = entry
            .worktree_root
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            same_worktree
                .entry(worktree_root.to_string())
                .or_default()
                .push(lane);
        }
    }

    let mut warnings = Vec::new();
    warnings.extend(
        same_branch
            .into_iter()
            .filter(|(_, lanes)| lanes.len() > 1)
            .map(|((repo_root, branch), lanes)| {
                format!(
                    "unsafe_same_branch repo={} branch={} sessions={}",
                    repo_root,
                    branch,
                    lanes.join(",")
                )
            }),
    );
    warnings.extend(
        same_worktree
            .into_iter()
            .filter(|(_, lanes)| lanes.len() > 1)
            .map(|(worktree_root, lanes)| {
                format!(
                    "unsafe_same_worktree worktree={} sessions={}",
                    worktree_root,
                    lanes.join(",")
                )
            }),
    );
    warnings
}

pub(crate) fn work_overlap_warnings(entries: &[&ProjectAwarenessEntry]) -> Vec<String> {
    let mut warnings = Vec::new();
    let active_entries = entries
        .iter()
        .copied()
        .filter(|entry| entry.presence == "active")
        .collect::<Vec<_>>();

    for (idx, left) in active_entries.iter().enumerate() {
        let left_touches = awareness_overlap_touch_points(left);
        if left_touches.is_empty() {
            continue;
        }
        for right in active_entries.iter().skip(idx + 1) {
            let right_touches = awareness_overlap_touch_points(right);
            if right_touches.is_empty() {
                continue;
            }
            let shared = left_touches
                .iter()
                .filter(|touch| right_touches.iter().any(|other| other == *touch))
                .cloned()
                .collect::<Vec<_>>();
            if shared.is_empty() {
                continue;
            }
            warnings.push(format!(
                "possible_work_overlap touches={} sessions={},{}",
                shared.join(","),
                left.session.as_deref().unwrap_or("none"),
                right.session.as_deref().unwrap_or("none"),
            ));
        }
    }

    warnings
}

#[derive(Debug, Clone)]
pub(crate) struct BundleHiveMemorySurface {
    pub(crate) board: HiveBoardResponse,
    pub(crate) roster: HiveRosterResponse,
    pub(crate) follow: Option<HiveFollowResponse>,
}

pub(crate) fn push_awareness_section(
    lines: &mut Vec<String>,
    label: &str,
    entries: &[&ProjectAwarenessEntry],
    role: &str,
    current_bundle: &str,
) {
    if entries.is_empty() {
        return;
    }
    lines.push(format!("{label}:"));
    for entry in entries {
        lines.push(render_awareness_entry_line(entry, role, current_bundle));
    }
}

pub(crate) fn awareness_truth_label(
    entry: &ProjectAwarenessEntry,
    current_bundle: &str,
) -> &'static str {
    if entry.bundle_root == current_bundle && entry.presence == "active" {
        return "current";
    }
    match entry.presence.as_str() {
        "active" => match entry.last_updated {
            Some(last_updated) => {
                let age = Utc::now() - last_updated;
                if age.num_seconds() <= 120 {
                    "fresh"
                } else if age.num_minutes() <= 15 {
                    "aging"
                } else {
                    "stale-truth"
                }
            }
            None => "active",
        },
        "stale" => "stale",
        "dead" => "dead",
        _ => "seen",
    }
}

pub(crate) fn render_awareness_entry_line(
    entry: &ProjectAwarenessEntry,
    role: &str,
    current_bundle: &str,
) -> String {
    let focus = entry
        .focus
        .as_deref()
        .map(|value| compact_inline(value, 56))
        .unwrap_or_else(|| "none".to_string());
    let pressure = entry
        .pressure
        .as_deref()
        .map(|value| compact_inline(value, 56))
        .unwrap_or_else(|| "none".to_string());
    let truth = awareness_truth_label(entry, current_bundle);
    let updated = entry
        .last_updated
        .map(|value| value.to_rfc3339())
        .unwrap_or_else(|| "unknown".to_string());
    let work = entry
        .topic_claim
        .as_deref()
        .map(|value| compact_inline(value, 56))
        .unwrap_or_else(|| awareness_work_quickview(entry));
    let next = entry
        .next_recovery
        .as_deref()
        .and_then(simplify_awareness_work_text)
        .map(|value| compact_inline(&value, 56))
        .unwrap_or_else(|| "none".to_string());
    let touches = if entry.scope_claims.is_empty() {
        awareness_touch_quickview(entry)
    } else {
        compact_inline(&entry.scope_claims.join(","), 56)
    };
    format!(
        "- {} [{}] | presence={} truth={} updated={} claims={} ns={} hive={} role={} groups={} goal=\"{}\" authority={} agent={} session={} tab={} branch={} worktree={} base_url={} workspace={} visibility={} task={} work=\"{}\" touches={} next=\"{}\" focus=\"{}\" pressure=\"{}\"",
        entry.project.as_deref().unwrap_or("unknown"),
        role,
        entry.presence,
        truth,
        updated,
        entry.active_claims,
        entry.namespace.as_deref().unwrap_or("none"),
        entry.hive_system.as_deref().unwrap_or("none"),
        entry.hive_role.as_deref().unwrap_or("none"),
        if entry.hive_groups.is_empty() {
            "none".to_string()
        } else {
            entry.hive_groups.join(",")
        },
        entry.hive_group_goal.as_deref().unwrap_or("none"),
        entry.authority.as_deref().unwrap_or("none"),
        entry
            .effective_agent
            .as_deref()
            .or(entry.agent.as_deref())
            .unwrap_or("none"),
        entry.session.as_deref().unwrap_or("none"),
        entry.tab_id.as_deref().unwrap_or("none"),
        entry.branch.as_deref().unwrap_or("none"),
        entry.worktree_root.as_deref().unwrap_or("none"),
        entry.base_url.as_deref().unwrap_or("none"),
        entry.workspace.as_deref().unwrap_or("none"),
        entry.visibility.as_deref().unwrap_or("all"),
        entry.task_id.as_deref().unwrap_or("none"),
        work,
        touches,
        next,
        focus,
        pressure,
    )
}

pub(crate) fn awareness_work_quickview(entry: &ProjectAwarenessEntry) -> String {
    if let Some(value) = entry
        .topic_claim
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return compact_inline(value, 56);
    }
    for candidate in [entry.focus.as_deref(), entry.next_recovery.as_deref()] {
        if let Some(value) = candidate.and_then(simplify_awareness_work_text) {
            return compact_inline(&value, 56);
        }
    }
    let touches = awareness_touch_points(entry);
    if let Some(first) = touches.first() {
        if touches.len() == 1 {
            return compact_inline(&format!("editing {first}"), 56);
        }
        return compact_inline(&format!("editing {first} +{}", touches.len() - 1), 56);
    }
    if let Some(value) = entry
        .pressure
        .as_deref()
        .and_then(simplify_awareness_work_text)
    {
        return compact_inline(&value, 56);
    }
    "none".to_string()
}

pub(crate) fn derive_awareness_worker_name(entry: &ProjectAwarenessEntry) -> Option<String> {
    entry
        .effective_agent
        .as_deref()
        .and_then(|value| value.split('@').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            entry
                .agent
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
}

pub(crate) fn derive_awareness_lane_id(entry: &ProjectAwarenessEntry) -> Option<String> {
    entry
        .worktree_root
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            entry
                .branch
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
}

pub(crate) fn project_awareness_entry_to_hive_session(
    entry: &ProjectAwarenessEntry,
) -> memd_schema::HiveSessionRecord {
    let working = entry
        .topic_claim
        .clone()
        .or_else(|| Some(awareness_work_quickview(entry)));
    let touches = awareness_touch_points(entry)
        .into_iter()
        .filter_map(|value| normalize_hive_touch(&value))
        .collect::<Vec<_>>();
    memd_schema::HiveSessionRecord {
        session: entry
            .session
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        tab_id: entry.tab_id.clone(),
        agent: entry.agent.clone(),
        effective_agent: entry.effective_agent.clone(),
        hive_system: entry.hive_system.clone(),
        hive_role: entry.hive_role.clone(),
        worker_name: derive_awareness_worker_name(entry),
        display_name: None,
        role: entry.hive_role.clone(),
        capabilities: entry.capabilities.clone(),
        hive_groups: entry.hive_groups.clone(),
        lane_id: derive_awareness_lane_id(entry),
        hive_group_goal: entry.hive_group_goal.clone(),
        authority: entry.authority.clone(),
        heartbeat_model: None,
        project: entry.project.clone(),
        namespace: entry.namespace.clone(),
        workspace: entry.workspace.clone(),
        repo_root: entry.repo_root.clone(),
        worktree_root: entry.worktree_root.clone(),
        branch: entry.branch.clone(),
        base_branch: entry.base_branch.clone(),
        visibility: entry.visibility.clone(),
        base_url: entry.base_url.clone(),
        base_url_healthy: None,
        host: entry.host.clone(),
        pid: entry.pid,
        topic_claim: entry.topic_claim.clone(),
        scope_claims: entry.scope_claims.clone(),
        task_id: entry.task_id.clone(),
        focus: entry.focus.clone(),
        pressure: entry.pressure.clone(),
        next_recovery: entry.next_recovery.clone(),
        next_action: None,
        working,
        touches,
        relationship_state: None,
        relationship_peer: None,
        relationship_reason: None,
        suggested_action: None,
        blocked_by: Vec::new(),
        cowork_with: Vec::new(),
        handoff_target: None,
        offered_to: Vec::new(),
        needs_help: false,
        needs_review: false,
        handoff_state: None,
        confidence: None,
        risk: None,
        status: entry.presence.clone(),
        last_seen: entry.last_updated.unwrap_or_else(Utc::now),
    }
}

pub(crate) fn awareness_touch_quickview(entry: &ProjectAwarenessEntry) -> String {
    let touches = awareness_touch_points(entry);
    if touches.is_empty() {
        "none".to_string()
    } else {
        compact_inline(&touches.join(","), 56)
    }
}

pub(crate) fn awareness_touch_points(entry: &ProjectAwarenessEntry) -> Vec<String> {
    let mut touches = Vec::new();
    for scope in &entry.scope_claims {
        push_unique_touch_point(&mut touches, scope);
    }
    for candidate in [
        entry.pressure.as_deref(),
        entry.focus.as_deref(),
        entry.next_recovery.as_deref(),
    ] {
        let Some(value) = candidate else {
            continue;
        };
        append_awareness_touch_points(value, &mut touches);
    }
    touches.truncate(4);
    touches
}

pub(crate) fn normalize_hive_touch(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

pub(crate) fn awareness_overlap_touch_points(entry: &ProjectAwarenessEntry) -> Vec<String> {
    awareness_touch_points(entry)
        .into_iter()
        .filter(|touch| !is_generic_overlap_touch(touch))
        .collect()
}

pub(crate) fn is_generic_overlap_touch(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.is_empty()
        || matches!(
            trimmed,
            "project" | "workspace" | "shared" | "none" | "unknown"
        )
}

pub(crate) fn append_awareness_touch_points(value: &str, touches: &mut Vec<String>) {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return;
    }

    for part in trimmed
        .split('\n')
        .flat_map(|line| line.split(" | "))
        .map(str::trim)
    {
        if let Some(path) = part.strip_prefix("file_edited:") {
            push_unique_touch_point(touches, path.trim());
            continue;
        }
        if let Some(scope) = part.strip_prefix("scope=") {
            push_unique_touch_point(touches, scope.trim());
            continue;
        }
        if let Some(location) = part.strip_prefix("location=") {
            push_unique_touch_point(touches, location.trim());
        }
    }
}

pub(crate) fn push_unique_touch_point(touches: &mut Vec<String>, value: &str) {
    let trimmed = value.trim();
    if trimmed.is_empty() || touches.iter().any(|existing| existing == trimmed) {
        return;
    }
    touches.push(trimmed.to_string());
}

pub(crate) fn derive_hive_topic_claim(
    focus: Option<&str>,
    next_recovery: Option<&str>,
    pressure: Option<&str>,
) -> Option<String> {
    for candidate in [focus, next_recovery] {
        if let Some(value) = candidate.and_then(simplify_awareness_work_text) {
            return Some(compact_inline(&value, 120));
        }
    }
    let touches = [pressure]
        .into_iter()
        .flatten()
        .flat_map(|value| {
            let mut out = Vec::new();
            append_awareness_touch_points(value, &mut out);
            out
        })
        .collect::<Vec<_>>();
    if let Some(first) = touches.first() {
        return Some(if touches.len() == 1 {
            format!("editing {first}")
        } else {
            format!("editing {first} +{}", touches.len() - 1)
        });
    }
    None
}

pub(crate) fn derive_hive_worker_name(
    agent: Option<&str>,
    _session: Option<&str>,
) -> Option<String> {
    agent
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub(crate) fn humanize_worker_label(value: &str) -> String {
    let parts = value
        .split(|ch: char| ch == '-' || ch == '_' || ch.is_whitespace())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            format!(
                "{}{}",
                first.to_uppercase(),
                chars.as_str().to_ascii_lowercase()
            )
        })
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        value.trim().to_string()
    } else {
        parts.join(" ")
    }
}

pub(crate) fn hive_worker_name_is_generic(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "codex" | "claude" | "claude-code"
    )
}

pub(crate) fn derive_hive_display_name(
    agent: Option<&str>,
    session: Option<&str>,
) -> Option<String> {
    let agent = agent.map(str::trim).filter(|value| !value.is_empty())?;
    if !hive_worker_name_is_generic(agent) {
        return None;
    }
    let session = session.map(str::trim).filter(|value| !value.is_empty())?;
    let session_suffix = session
        .strip_prefix("session-")
        .or_else(|| session.strip_prefix("codex-"))
        .or_else(|| session.strip_prefix("sender-"))
        .unwrap_or(session)
        .trim();
    if session_suffix.is_empty() {
        return None;
    }
    let base = match agent.to_ascii_lowercase().as_str() {
        "claude" | "claude-code" => "Claude",
        _ => "Codex",
    };
    Some(format!("{base} {}", session_suffix))
}

pub(crate) fn derive_project_scoped_worker_name(
    project: Option<&str>,
    agent: &str,
    session: Option<&str>,
) -> Option<String> {
    if !hive_worker_name_is_generic(agent) {
        return None;
    }
    let project = project
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(humanize_worker_label)?;
    let generic = derive_hive_display_name(Some(agent), session)?;
    Some(format!("{project} {generic}"))
}

pub(crate) fn default_bundle_worker_name(agent: &str, session: Option<&str>) -> String {
    derive_hive_display_name(Some(agent), session)
        .or_else(|| {
            derive_hive_worker_name(Some(agent), session).map(|value| humanize_worker_label(&value))
        })
        .unwrap_or_else(|| humanize_worker_label(agent))
}

pub(crate) fn default_bundle_worker_name_for_project(
    project: Option<&str>,
    agent: &str,
    session: Option<&str>,
) -> String {
    derive_project_scoped_worker_name(project, agent, session)
        .unwrap_or_else(|| default_bundle_worker_name(agent, session))
}

pub(crate) fn hive_actor_label(
    display_name: Option<&str>,
    worker_name: Option<&str>,
    agent: Option<&str>,
    session: Option<&str>,
) -> String {
    display_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| derive_hive_display_name(worker_name.or(agent), session))
        .or_else(|| {
            worker_name
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .or_else(|| {
            agent
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .or_else(|| {
            session
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "unnamed".to_string())
}

pub(crate) fn derive_hive_next_action(
    focus: Option<&str>,
    next_recovery: Option<&str>,
    pressure: Option<&str>,
) -> Option<String> {
    for candidate in [focus, next_recovery, pressure] {
        if let Some(value) = candidate.and_then(simplify_awareness_work_text) {
            return Some(compact_inline(&value, 120));
        }
    }
    None
}

pub(crate) fn derive_hive_lane_id(
    branch: Option<&str>,
    worktree_root: Option<&str>,
) -> Option<String> {
    worktree_root
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            branch
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
}

pub(crate) fn hive_topic_claim_needs_runtime_upgrade(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.is_empty()
        || trimmed.starts_with("editing ")
        || trimmed.starts_with("ws=")
        || trimmed.starts_with("workspace=")
        || trimmed == "project"
}

pub(crate) fn derive_hive_scope_claims(
    claims_state: Option<&SessionClaimsState>,
    focus: Option<&str>,
    pressure: Option<&str>,
    next_recovery: Option<&str>,
) -> Vec<String> {
    let mut scopes = claims_state
        .map(|state| {
            state
                .claims
                .iter()
                .filter(|claim| claim.expires_at > Utc::now())
                .map(|claim| claim.scope.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for candidate in [pressure, focus, next_recovery] {
        let Some(value) = candidate else {
            continue;
        };
        append_awareness_touch_points(value, &mut scopes);
    }
    scopes.truncate(8);
    scopes
}

pub(crate) fn derive_hive_task_id(
    scope_claims: &[String],
    topic_claim: Option<&str>,
) -> Option<String> {
    for scope in scope_claims {
        if let Some(task_id) = scope.strip_prefix("task:") {
            let task_id = task_id.trim();
            if !task_id.is_empty() {
                return Some(task_id.to_string());
            }
        }
    }
    if let Some(topic) = topic_claim {
        if let Some(task_id) = topic.strip_prefix("task:") {
            let task_id = task_id.trim();
            if !task_id.is_empty() {
                return Some(task_id.to_string());
            }
        }
    }
    None
}

pub(crate) fn confirmed_hive_overlap_reason(
    target: &ProjectAwarenessEntry,
    task_id: Option<&str>,
    topic_claim: Option<&str>,
    scope_claims: &[String],
) -> Option<String> {
    let current_scopes = scope_claims
        .iter()
        .map(|scope| scope.trim())
        .filter(|scope| !scope.is_empty())
        .filter(|scope| !is_generic_overlap_touch(scope))
        .collect::<Vec<_>>();
    let target_scopes = target
        .scope_claims
        .iter()
        .map(|scope| scope.trim())
        .filter(|scope| !scope.is_empty())
        .filter(|scope| !is_generic_overlap_touch(scope))
        .collect::<Vec<_>>();
    let task_id = task_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let topic_claim = topic_claim
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase());

    if let (Some(current_task), Some(target_task)) = (
        task_id.as_deref(),
        target
            .task_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    ) && current_task != target_task
    {
        if !target_scopes.is_empty()
            && current_scopes.iter().any(|scope| {
                target_scopes
                    .iter()
                    .any(|target_scope| target_scope == scope)
            })
        {
            return Some(format!(
                "confirmed hive overlap: target session {} already owns scope(s) for task {}",
                target.session.as_deref().unwrap_or("none"),
                target_task
            ));
        }
    }

    let shared_scopes = current_scopes
        .iter()
        .filter(|scope| {
            target_scopes
                .iter()
                .any(|target_scope| target_scope == *scope)
        })
        .map(|scope| (*scope).to_string())
        .collect::<Vec<_>>();
    if !shared_scopes.is_empty() {
        return Some(format!(
            "confirmed hive overlap: target session {} already claims {}",
            target.session.as_deref().unwrap_or("none"),
            shared_scopes.join(",")
        ));
    }

    if let (Some(current_topic), Some(target_topic)) = (
        topic_claim.as_deref(),
        target
            .topic_claim
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_ascii_lowercase()),
    ) && current_topic == target_topic
    {
        return Some(format!(
            "confirmed hive overlap: target session {} already owns topic {}",
            target.session.as_deref().unwrap_or("none"),
            target.topic_claim.as_deref().unwrap_or("none")
        ));
    }

    None
}

pub(crate) async fn existing_task_scopes_for_assignment(
    client: &MemdClient,
    project: &Option<String>,
    namespace: &Option<String>,
    workspace: &Option<String>,
    task_id: &str,
) -> anyhow::Result<Vec<String>> {
    let response = client
        .hive_tasks(&HiveTasksRequest {
            session: None,
            project: project.clone(),
            namespace: namespace.clone(),
            workspace: workspace.clone(),
            active_only: Some(false),
            limit: Some(256),
        })
        .await?;
    Ok(response
        .tasks
        .into_iter()
        .find(|task| task.task_id == task_id)
        .map(|task| task.claim_scopes)
        .unwrap_or_default())
}

pub(crate) fn render_attach_snippet(shell: &str, bundle_path: &Path) -> anyhow::Result<String> {
    let shell = shell.trim().to_ascii_lowercase();
    let project_hive_enabled = read_bundle_runtime_config(bundle_path)
        .ok()
        .flatten()
        .map(|runtime| runtime.hive_project_enabled)
        .unwrap_or(false);
    match shell.as_str() {
        "bash" | "zsh" | "sh" => Ok(format!(
            r#"export MEMD_BUNDLE_ROOT="{bundle_path}"
source "$MEMD_BUNDLE_ROOT/env"
{base_url_block}nohup memd heartbeat --output "$MEMD_BUNDLE_ROOT" --watch --interval-secs 30 --probe-base-url >/tmp/memd-heartbeat.log 2>&1 &
memd wake --output "$MEMD_BUNDLE_ROOT" --intent current_task --write
# pre-answer durable recall:
# .memd/agents/lookup.sh --query "what did we already decide?"
"#,
            bundle_path = bundle_path.display(),
            base_url_block = if project_hive_enabled {
                format!(
                    "if [[ -z \"${{MEMD_BASE_URL:-}}\" || \"${{MEMD_BASE_URL}}\" =~ ^https?://(localhost|127\\.0\\.0\\.1|0\\.0\\.0\\.0)(:[0-9]+)?(/|$) ]]; then\n  export MEMD_BASE_URL=\"{}\"\nfi\n",
                    SHARED_MEMD_BASE_URL
                )
            } else {
                String::new()
            },
        )),
        "powershell" | "pwsh" => Ok(format!(
            r#"$env:MEMD_BUNDLE_ROOT = "{bundle_path}"
. (Join-Path $env:MEMD_BUNDLE_ROOT "env.ps1")
{base_url_block}Start-Process -WindowStyle Hidden -FilePath memd -ArgumentList @('heartbeat','--output',$env:MEMD_BUNDLE_ROOT,'--watch','--interval-secs','30','--probe-base-url') -RedirectStandardOutput "$env:TEMP\memd-heartbeat.log" -RedirectStandardError "$env:TEMP\memd-heartbeat.err"
memd wake --output $env:MEMD_BUNDLE_ROOT --intent current_task --write
# pre-answer durable recall:
# .memd/agents/lookup.ps1 --query "what did we already decide?"
"#,
            bundle_path = escape_ps1(&bundle_path.display().to_string()),
            base_url_block = if project_hive_enabled {
                format!(
                    "if ([string]::IsNullOrWhiteSpace($env:MEMD_BASE_URL) -or $env:MEMD_BASE_URL -match '^(https?://)?(localhost|127\\.0\\.0\\.1|0\\.0\\.0\\.0)(:[0-9]+)?(/|$)') {{ $env:MEMD_BASE_URL = \"{}\" }}\n",
                    escape_ps1(SHARED_MEMD_BASE_URL)
                )
            } else {
                String::new()
            },
        )),
        other => anyhow::bail!(
            "unsupported shell '{other}'; expected bash, zsh, sh, powershell, or pwsh"
        ),
    }
}
