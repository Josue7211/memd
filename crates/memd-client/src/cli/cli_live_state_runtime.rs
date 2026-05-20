use super::*;
use anyhow::{Context, bail};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fmt;
use std::io::Read;

const LIVE_STATE_VERSION: u32 = 1;
const LIVE_STATE_PRODUCER_CONTRACT_VERSION: u32 = 1;
const LIVE_STATE_DEFAULT_REFRESH_SECS: i64 = 86_400;
const LIVE_STATE_SOURCE_STATUS_FRESH_SECS: i64 = 900;
const MEMD_LIVE_STATE_SOURCE: &str = "memd";

#[derive(Debug)]
pub(crate) struct LiveStateCheckExitCode(pub(crate) i32);

impl fmt::Display for LiveStateCheckExitCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "live-state sync required")
    }
}

impl std::error::Error for LiveStateCheckExitCode {}

#[derive(Debug, Clone, Copy)]
struct LiveAppStateRequirement {
    source_app: &'static str,
    module: &'static str,
    canonical_scope: &'static str,
    accepted_scopes: &'static [&'static str],
    privacy_route: &'static str,
    action: &'static str,
}

const LIVE_APP_STATE_REQUIREMENTS: &[LiveAppStateRequirement] = &[
    LiveAppStateRequirement {
        source_app: MEMD_LIVE_STATE_SOURCE,
        module: "visible_page",
        canonical_scope: "current",
        accepted_scopes: &["current"],
        privacy_route: "private metadata",
        action: "capture visible app/page state before answering present-tense UI questions",
    },
    LiveAppStateRequirement {
        source_app: MEMD_LIVE_STATE_SOURCE,
        module: "calendar",
        canonical_scope: "primary",
        accepted_scopes: &["primary", "current"],
        privacy_route: "private approved or metadata",
        action: "ingest current/next calendar events before answering calendar questions",
    },
    LiveAppStateRequirement {
        source_app: MEMD_LIVE_STATE_SOURCE,
        module: "reminders",
        canonical_scope: "default",
        accepted_scopes: &["default", "current"],
        privacy_route: "private approved or metadata",
        action: "ingest active reminders before answering reminder questions",
    },
    LiveAppStateRequirement {
        source_app: MEMD_LIVE_STATE_SOURCE,
        module: "todos",
        canonical_scope: "default",
        accepted_scopes: &["default", "current"],
        privacy_route: "private approved or metadata",
        action: "ingest active todos before answering task questions",
    },
    LiveAppStateRequirement {
        source_app: "approved_communications",
        module: "messages",
        canonical_scope: "approved",
        accepted_scopes: &["approved", "current"],
        privacy_route: "private metadata/redacted; no unrestricted chat access",
        action: "ingest only approved text-message metadata or redacted snippets",
    },
    LiveAppStateRequirement {
        source_app: "approved_communications",
        module: "email",
        canonical_scope: "approved",
        accepted_scopes: &["approved", "current"],
        privacy_route: "private metadata/redacted; approved mail only",
        action: "ingest only approved email metadata or redacted snippets",
    },
];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct LiveAppStateStore {
    pub(crate) version: u32,
    pub(crate) updated_at: Option<chrono::DateTime<Utc>>,
    #[serde(default)]
    pub(crate) records: Vec<LiveAppStateRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LiveAppStateRecord {
    pub(crate) id: String,
    pub(crate) source_app: String,
    pub(crate) module: String,
    pub(crate) scope: String,
    pub(crate) visibility: String,
    pub(crate) privacy: String,
    pub(crate) approved: bool,
    #[serde(default)]
    pub(crate) agentsecrets_approved: bool,
    #[serde(default)]
    pub(crate) labels: Vec<String>,
    pub(crate) summary: String,
    pub(crate) payload: Value,
    pub(crate) payload_hash: String,
    pub(crate) captured_at: chrono::DateTime<Utc>,
    pub(crate) updated_at: chrono::DateTime<Utc>,
    pub(crate) expires_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LiveAppStateReport {
    pub(crate) status: String,
    pub(crate) path: String,
    pub(crate) source_status_path: String,
    pub(crate) checked_at: chrono::DateTime<Utc>,
    pub(crate) next_refresh_at: chrono::DateTime<Utc>,
    pub(crate) refresh_reason: String,
    pub(crate) refresh_policy: String,
    pub(crate) producer_contract_version: u32,
    pub(crate) total: usize,
    pub(crate) fresh: usize,
    pub(crate) stale: usize,
    pub(crate) requirement_fresh: usize,
    pub(crate) requirement_stale: usize,
    pub(crate) requirement_missing: usize,
    pub(crate) sync_required: bool,
    pub(crate) sync_actions: Vec<String>,
    pub(crate) sync_tasks: Vec<LiveAppStateSyncTask>,
    pub(crate) requirements: Vec<LiveAppStateRequirementStatus>,
    pub(crate) source_fresh: usize,
    pub(crate) source_stale: usize,
    pub(crate) source_unavailable: usize,
    pub(crate) source_statuses: Vec<LiveAppStateSourceStatus>,
    pub(crate) records: Vec<LiveAppStateRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct LiveAppStateSourceStatusStore {
    pub(crate) version: u32,
    pub(crate) updated_at: Option<chrono::DateTime<Utc>>,
    #[serde(default)]
    pub(crate) sources: Vec<LiveAppStateSourceStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct LiveAppStateSourceStatus {
    pub(crate) source_app: String,
    pub(crate) status: String,
    pub(crate) checked_at: chrono::DateTime<Utc>,
    pub(crate) api_base: Option<String>,
    #[serde(default)]
    pub(crate) api_bases: Vec<String>,
    #[serde(default)]
    pub(crate) auth_configured: bool,
    pub(crate) visible_page: Option<String>,
    #[serde(default)]
    pub(crate) produced: Vec<String>,
    #[serde(default)]
    pub(crate) missing: Vec<String>,
    pub(crate) record_count: usize,
    #[serde(default)]
    pub(crate) endpoints: Vec<LiveAppStateSourceEndpointStatus>,
    pub(crate) last_error: Option<String>,
    #[serde(default)]
    pub(crate) approval_request_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct LiveAppStateSourceEndpointStatus {
    pub(crate) module: String,
    pub(crate) path: String,
    #[serde(default, alias = "apiBase")]
    pub(crate) api_base: Option<String>,
    pub(crate) ok: bool,
    pub(crate) status: i64,
    pub(crate) error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct LiveAppStateRequirementStatus {
    pub(crate) source_app: String,
    pub(crate) module: String,
    pub(crate) canonical_scope: String,
    pub(crate) accepted_scopes: Vec<String>,
    pub(crate) status: String,
    pub(crate) matched_scope: Option<String>,
    pub(crate) privacy_route: String,
    pub(crate) action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct LiveAppStateSyncTask {
    pub(crate) source_app: String,
    pub(crate) module: String,
    pub(crate) required_scope: String,
    pub(crate) accepted_scopes: Vec<String>,
    pub(crate) status: String,
    pub(crate) visibility: String,
    pub(crate) privacy: String,
    pub(crate) approved_required: bool,
    pub(crate) agentsecrets_required_for_media: bool,
    pub(crate) freshness_secs: i64,
    pub(crate) labels: Vec<String>,
    pub(crate) summary_hint: String,
    pub(crate) payload_hint: String,
    pub(crate) ingest_argv: Vec<String>,
    pub(crate) action: String,
}

#[derive(Debug, Clone, Deserialize)]
struct LiveStateIngestBatchBody {
    #[serde(default)]
    records: Vec<LiveStateIngestBatchRecord>,
}

#[derive(Debug, Clone, Deserialize)]
struct LiveStateIngestBatchRecord {
    #[serde(default, alias = "sourceApp")]
    source_app: Option<String>,
    module: String,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    visibility: Option<String>,
    #[serde(default)]
    privacy: Option<String>,
    #[serde(default)]
    approved: Option<bool>,
    #[serde(default, alias = "agentsecretsApproved")]
    agentsecrets_approved: Option<bool>,
    #[serde(default, alias = "freshnessSecs")]
    freshness_secs: Option<i64>,
    #[serde(default)]
    labels: Option<Vec<String>>,
    summary: String,
    #[serde(default)]
    payload: Option<Value>,
}

pub(crate) fn live_app_state_path(output: &Path) -> PathBuf {
    output.join("state").join("live-app-state.json")
}

pub(crate) fn live_app_source_status_path(output: &Path) -> PathBuf {
    output.join("state").join("live-app-source-status.json")
}

pub(crate) fn read_live_app_state(output: &Path) -> anyhow::Result<LiveAppStateStore> {
    let path = live_app_state_path(output);
    if !path.exists() {
        return Ok(LiveAppStateStore {
            version: LIVE_STATE_VERSION,
            updated_at: None,
            records: Vec::new(),
        });
    }
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("read live app state {}", path.display()))?;
    let mut store: LiveAppStateStore = serde_json::from_str(&text)
        .with_context(|| format!("parse live app state {}", path.display()))?;
    store.version = store.version.max(1);
    Ok(store)
}

fn read_live_app_source_status(output: &Path) -> anyhow::Result<LiveAppStateSourceStatusStore> {
    let path = live_app_source_status_path(output);
    if !path.exists() {
        return Ok(LiveAppStateSourceStatusStore {
            version: 1,
            updated_at: None,
            sources: Vec::new(),
        });
    }
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("read live app source status {}", path.display()))?;
    let mut store: LiveAppStateSourceStatusStore = serde_json::from_str(&text)
        .with_context(|| format!("parse live app source status {}", path.display()))?;
    store.version = store.version.max(1);
    Ok(store)
}

pub(crate) fn render_live_app_state_section(output: &Path, limit: usize) -> String {
    let Ok(store) = read_live_app_state(output) else {
        return "- unavailable: live app state map unreadable".to_string();
    };
    let now = Utc::now();
    let mut records = store.records;
    records.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    let fresh_records = records
        .iter()
        .filter(|record| record.expires_at > now)
        .collect::<Vec<_>>();

    let mut lines = Vec::new();
    if fresh_records.is_empty() {
        lines.push(
            "- no fresh live app state; present-tense app facts are unknown until a producer ingests the state map".to_string(),
        );
    } else {
        lines.extend(
            fresh_records
                .into_iter()
                .take(limit)
                .map(format_live_state_record),
        );
    }

    let requirements = live_state_requirement_statuses(&records, now);
    let (next_refresh_at, refresh_reason) = live_state_next_refresh(&records, now, &requirements);
    lines.push(format!(
        "- refresh_policy contract={} next_refresh_at={} reason=\"{}\" policy=\"immediate on missing/stale; otherwise before earliest expiry; default_ttl={}s\"",
        LIVE_STATE_PRODUCER_CONTRACT_VERSION,
        next_refresh_at.to_rfc3339(),
        compact_live_state_text(&refresh_reason, 180),
        LIVE_STATE_DEFAULT_REFRESH_SECS
    ));
    lines.extend(render_live_state_sync_task_lines(&records, now));
    if let Ok(source_status_store) = read_live_app_source_status(output) {
        lines.extend(render_live_state_source_status_lines(
            &source_status_store.sources,
            now,
            &requirements,
        ));
    }
    lines.extend(render_live_state_requirement_lines(&records, now));
    lines.join("\n")
}

pub(crate) fn run_live_state_command(args: &LiveStateArgs) -> anyhow::Result<LiveAppStateReport> {
    match &args.command {
        LiveStateSubcommand::Ingest(ingest) => ingest_live_state(ingest),
        LiveStateSubcommand::IngestBatch(batch) => ingest_live_state_batch(batch),
        LiveStateSubcommand::Import(import) => import_live_state(import),
        LiveStateSubcommand::Sync(sync) => sync_live_state(sync),
        LiveStateSubcommand::Status(status) => live_state_report(&status.output),
    }
}

fn ingest_live_state(args: &LiveStateIngestArgs) -> anyhow::Result<LiveAppStateReport> {
    let payload = read_payload(args)?;
    validate_live_state_privacy(args, &payload)?;
    let now = Utc::now();
    let freshness_secs = args.freshness_secs.max(60);
    let payload_hash = hash_payload(&payload);
    let module = normalize_key(&args.module);
    let source_app = normalize_key(&args.source);
    let scope = args.scope.trim().to_string();
    let record = LiveAppStateRecord {
        id: format!("{source_app}:{module}:{scope}"),
        source_app,
        module,
        scope,
        visibility: args.visibility.trim().to_ascii_lowercase(),
        privacy: args.privacy.trim().to_ascii_lowercase(),
        approved: args.approved,
        agentsecrets_approved: args.agentsecrets_approved,
        labels: args
            .label
            .iter()
            .map(|label| label.trim().to_string())
            .filter(|label| !label.is_empty())
            .collect(),
        summary: args.summary.trim().to_string(),
        payload,
        payload_hash,
        captured_at: now,
        updated_at: now,
        expires_at: now + Duration::seconds(freshness_secs),
    };

    let mut store = read_live_app_state(&args.output)?;
    store.version = LIVE_STATE_VERSION;
    store.updated_at = Some(now);
    store.records.retain(|existing| existing.id != record.id);
    store.records.push(record);
    store
        .records
        .sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    write_live_app_state(&args.output, &store)?;
    live_state_report(&args.output)
}

fn ingest_live_state_batch(args: &LiveStateIngestBatchArgs) -> anyhow::Result<LiveAppStateReport> {
    let body = read_batch_body(args)?;
    if body.records.is_empty() {
        bail!("live-state batch has no records");
    }

    let now = Utc::now();
    let mut store = read_live_app_state(&args.output)?;
    store.version = LIVE_STATE_VERSION;
    store.updated_at = Some(now);

    for input in body.records {
        let payload = input
            .payload
            .clone()
            .unwrap_or_else(|| Value::Object(Default::default()));
        let source = input
            .source_app
            .as_deref()
            .unwrap_or("clawcontrol")
            .trim()
            .to_string();
        let scope = input
            .scope
            .as_deref()
            .unwrap_or("current")
            .trim()
            .to_string();
        let visibility = input
            .visibility
            .as_deref()
            .unwrap_or("private")
            .trim()
            .to_string();
        let privacy = input
            .privacy
            .as_deref()
            .unwrap_or("metadata")
            .trim()
            .to_string();
        let labels = input
            .labels
            .unwrap_or_default()
            .into_iter()
            .map(|label| label.trim().to_string())
            .filter(|label| !label.is_empty())
            .collect::<Vec<_>>();
        let freshness_secs = input
            .freshness_secs
            .unwrap_or(LIVE_STATE_DEFAULT_REFRESH_SECS)
            .max(60);
        let ingest = LiveStateIngestArgs {
            output: args.output.clone(),
            source,
            module: input.module.trim().to_string(),
            scope,
            visibility,
            privacy,
            approved: input.approved.unwrap_or(false),
            agentsecrets_approved: input.agentsecrets_approved.unwrap_or(false),
            freshness_secs,
            label: labels,
            summary: input.summary.trim().to_string(),
            payload_json: None,
            payload_file: None,
            json: args.json,
        };
        if ingest.summary.is_empty() {
            bail!("live state summary is required");
        }
        validate_live_state_privacy(&ingest, &payload)?;

        let payload_hash = hash_payload(&payload);
        let module = normalize_key(&ingest.module);
        let source_app = normalize_key(&ingest.source);
        let scope = ingest.scope.trim().to_string();
        let record = LiveAppStateRecord {
            id: format!("{source_app}:{module}:{scope}"),
            source_app,
            module,
            scope,
            visibility: ingest.visibility.trim().to_ascii_lowercase(),
            privacy: ingest.privacy.trim().to_ascii_lowercase(),
            approved: ingest.approved,
            agentsecrets_approved: ingest.agentsecrets_approved,
            labels: ingest.label,
            summary: ingest.summary,
            payload,
            payload_hash,
            captured_at: now,
            updated_at: now,
            expires_at: now + Duration::seconds(freshness_secs),
        };
        store.records.retain(|existing| existing.id != record.id);
        store.records.push(record);
    }

    store
        .records
        .sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    write_live_app_state(&args.output, &store)?;
    live_state_report(&args.output)
}

fn import_live_state(args: &LiveStateImportArgs) -> anyhow::Result<LiveAppStateReport> {
    let source_filter = args.source.as_deref().map(normalize_key);
    let now = Utc::now();
    let source_store = read_live_app_state(&args.from_output)?;
    let mut imported = 0usize;
    let mut store = read_live_app_state(&args.output)?;
    store.version = LIVE_STATE_VERSION;
    store.updated_at = Some(now);

    for record in source_store.records {
        if source_filter
            .as_deref()
            .is_some_and(|source| record.source_app != source)
        {
            continue;
        }
        if args.fresh_only && record.expires_at <= now {
            continue;
        }
        if record.summary.trim().is_empty() {
            bail!("live state summary is required");
        }
        let validation_args = LiveStateIngestArgs {
            output: args.output.clone(),
            source: record.source_app.clone(),
            module: record.module.clone(),
            scope: record.scope.clone(),
            visibility: record.visibility.clone(),
            privacy: record.privacy.clone(),
            approved: record.approved,
            agentsecrets_approved: record.agentsecrets_approved,
            freshness_secs: (record.expires_at - record.captured_at)
                .num_seconds()
                .max(60),
            label: record.labels.clone(),
            summary: record.summary.clone(),
            payload_json: None,
            payload_file: None,
            json: args.json,
        };
        validate_live_state_privacy(&validation_args, &record.payload)?;
        store.records.retain(|existing| existing.id != record.id);
        store.records.push(record);
        imported += 1;
    }

    if imported == 0 {
        bail!("no live-state records imported");
    }

    store
        .records
        .sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    write_live_app_state(&args.output, &store)?;
    live_state_report(&args.output)
}

fn sync_live_state(args: &LiveStateSyncArgs) -> anyhow::Result<LiveAppStateReport> {
    let report = live_state_report(&args.output)?;
    if !live_state_check_required(&report, args.due_within_secs) {
        return Ok(report);
    }
    let source = normalize_key(&args.source);
    let import = LiveStateImportArgs {
        output: args.output.clone(),
        from_output: args.from_output.clone(),
        source: (!matches!(source.as_str(), "all" | "*")).then_some(source),
        fresh_only: !args.allow_stale,
        json: args.json,
    };
    import_live_state(&import)
}

pub(crate) fn live_state_report(output: &Path) -> anyhow::Result<LiveAppStateReport> {
    let path = live_app_state_path(output);
    let source_status_path = live_app_source_status_path(output);
    let store = read_live_app_state(output)?;
    let source_status_store = read_live_app_source_status(output)?;
    let now = Utc::now();
    let fresh = store
        .records
        .iter()
        .filter(|record| record.expires_at > now)
        .count();
    let stale = store.records.len().saturating_sub(fresh);
    let requirements = live_state_requirement_statuses(&store.records, now);
    let requirement_fresh = requirements
        .iter()
        .filter(|requirement| requirement.status == "fresh")
        .count();
    let requirement_stale = requirements
        .iter()
        .filter(|requirement| requirement.status == "stale")
        .count();
    let requirement_missing = requirements
        .iter()
        .filter(|requirement| requirement.status == "missing")
        .count();
    let sync_required = requirement_missing > 0 || requirement_stale > 0;
    let sync_actions = live_state_sync_actions(&requirements);
    let sync_tasks = live_state_sync_tasks(&requirements);
    let source_has_unmet_requirements = |source: &LiveAppStateSourceStatus| {
        !live_state_unmet_modules_for_source_from(
            &requirements,
            &source_status_store.sources,
            source,
        )
        .is_empty()
    };
    let source_fresh = source_status_store
        .sources
        .iter()
        .filter(|source| source_has_unmet_requirements(source))
        .filter(|source| live_state_source_status_is_fresh(source, now))
        .count();
    let source_stale = source_status_store
        .sources
        .iter()
        .filter(|source| source_has_unmet_requirements(source))
        .filter(|source| !live_state_source_status_is_fresh(source, now))
        .count();
    let source_unavailable = source_status_store
        .sources
        .iter()
        .filter(|source| source_has_unmet_requirements(source))
        .filter(|source| source.status != "ok")
        .count();
    let (next_refresh_at, refresh_reason) =
        live_state_next_refresh(&store.records, now, &requirements);
    Ok(LiveAppStateReport {
        status: if requirement_missing > 0 {
            "missing_requirements".to_string()
        } else if requirement_stale > 0 {
            "stale_requirements".to_string()
        } else if store.records.is_empty() {
            "empty".to_string()
        } else if stale > 0 {
            "stale".to_string()
        } else {
            "fresh".to_string()
        },
        path: path.display().to_string(),
        source_status_path: source_status_path.display().to_string(),
        checked_at: now,
        next_refresh_at,
        refresh_reason,
        refresh_policy: format!(
            "refresh immediately when any required surface is missing/stale; otherwise refresh before earliest expiry; default producer ttl={}s",
            LIVE_STATE_DEFAULT_REFRESH_SECS
        ),
        producer_contract_version: LIVE_STATE_PRODUCER_CONTRACT_VERSION,
        total: store.records.len(),
        fresh,
        stale,
        requirement_fresh,
        requirement_stale,
        requirement_missing,
        sync_required,
        sync_actions,
        sync_tasks,
        requirements,
        source_fresh,
        source_stale,
        source_unavailable,
        source_statuses: source_status_store.sources,
        records: store.records,
    })
}

pub(crate) fn clawcontrol_api_key_access_route_command() -> &'static str {
    "memd access route --output .memd --purpose clawcontrol-api-key --provider process-env --agent codex"
}

pub(crate) fn approved_communications_access_route_command() -> &'static str {
    "memd access route --output .memd --purpose approved-communications-file --provider process-env --agent codex"
}

pub(crate) fn approved_communications_empty_approval_command() -> &'static str {
    "APPROVED_COMMUNICATIONS_EMPTY_APPROVED=1 scripts/live-state-capture-approved-communications.mjs"
}

pub(crate) fn live_state_unmet_modules_for_source(
    report: &LiveAppStateReport,
    source: &LiveAppStateSourceStatus,
) -> Vec<String> {
    live_state_unmet_modules_for_source_from(&report.requirements, &report.source_statuses, source)
}

fn live_state_unmet_modules_for_source_from(
    requirements: &[LiveAppStateRequirementStatus],
    _sources: &[LiveAppStateSourceStatus],
    source: &LiveAppStateSourceStatus,
) -> Vec<String> {
    let source_name = live_state_source_name(source);
    let filter_communications_from_generic_source =
        !live_state_source_is_approved_communications(source);
    let filter_module = |module: &String| {
        !(filter_communications_from_generic_source && sensitive_communication_module(module))
    };
    let source_claims_module_missing =
        |module: &String| source.missing.iter().any(|missing| missing == module);

    let unmet_for_source = requirements
        .iter()
        .filter(|requirement| {
            requirement.source_app == source_name && requirement.status != "fresh"
        })
        .map(|requirement| requirement.module.clone())
        .filter(|module| filter_module(module))
        .collect::<Vec<_>>();

    if source.missing.is_empty() {
        return unmet_for_source;
    }

    unmet_for_source
        .into_iter()
        .filter(|module| source_claims_module_missing(module))
        .collect()
}

pub(crate) fn live_state_source_is_approved_communications(
    source: &LiveAppStateSourceStatus,
) -> bool {
    source.source_app == "approved_communications"
        || source.source_app == "approved-communications"
        || source.api_base.as_deref() == Some("approved-communications")
        || source.api_base.as_deref() == Some("approved_communications")
        || source
            .api_bases
            .iter()
            .any(|base| base == "approved-communications" || base == "approved_communications")
}

pub(crate) fn live_state_source_name<'a>(source: &'a LiveAppStateSourceStatus) -> &'a str {
    if live_state_source_is_approved_communications(source) {
        "approved_communications"
    } else {
        source.source_app.as_str()
    }
}

pub(crate) fn live_state_source_is_clawcontrol_app(source: &LiveAppStateSourceStatus) -> bool {
    source.source_app == "clawcontrol" && !live_state_source_is_approved_communications(source)
}

pub(crate) fn live_state_source_is_memd_app(source: &LiveAppStateSourceStatus) -> bool {
    source.source_app == MEMD_LIVE_STATE_SOURCE
        && !live_state_source_is_approved_communications(source)
}

fn live_state_source_satisfies_requirement(
    source_app: &str,
    requirement: &LiveAppStateRequirement,
) -> bool {
    source_app == requirement.source_app
        || (requirement.source_app == MEMD_LIVE_STATE_SOURCE
            && matches!(source_app, "clawcontrol" | "mac_bridge" | "mac-bridge"))
}

pub(crate) fn live_state_blocker_detail(output: &Path) -> Option<String> {
    live_state_report(output)
        .ok()
        .and_then(|report| live_state_blocker_detail_from_report(&report))
}

pub(crate) fn live_state_recovery_blocker_detail(output: &Path) -> Option<String> {
    live_state_report(output)
        .ok()
        .and_then(|report| live_state_blocker_detail_from_report_with_options(&report, false))
}

pub(crate) fn live_state_blocker_detail_from_report(report: &LiveAppStateReport) -> Option<String> {
    live_state_blocker_detail_from_report_with_options(report, true)
}

fn live_state_blocker_detail_from_report_with_options(
    report: &LiveAppStateReport,
    include_producer_route: bool,
) -> Option<String> {
    let mut details = Vec::new();
    for source in report
        .source_statuses
        .iter()
        .filter(|source| source.status != "ok")
    {
        let missing_modules = live_state_unmet_modules_for_source(report, source);
        if missing_modules.is_empty() {
            continue;
        }
        let missing = if missing_modules.is_empty() {
            "none".to_string()
        } else {
            missing_modules.join(",")
        };
        let source_name = live_state_source_name(source);
        let access_route =
            if live_state_source_is_clawcontrol_app(source) && source.status == "auth_required" {
                format!(
                    " access_route=\"{}\"",
                    clawcontrol_api_key_access_route_command()
                )
            } else if live_state_source_is_approved_communications(source)
                && (source.status == "missing_approval" || source.status == "invalid_approval")
            {
                format!(
                    " access_route=\"{}\"",
                    approved_communications_access_route_command()
                )
            } else {
                String::new()
            };
        let producer_route = if !include_producer_route {
            String::new()
        } else if live_state_source_is_memd_app(source)
            && matches!(source.status.as_str(), "auth_required" | "unavailable")
        {
            " producer_route=\"scripts/live-state-sync-memd.sh\" external_source_note=\"memd-owned producers only; does not launch ClawControl\"".to_string()
        } else if live_state_source_is_clawcontrol_app(source)
            && matches!(source.status.as_str(), "auth_required" | "unavailable")
        {
            let api_bases = if source.api_bases.is_empty() {
                source.api_base.as_deref().unwrap_or("unknown").to_string()
            } else {
                source.api_bases.join(",")
            };
            format!(
                " producer_route=\"scripts/live-state-sync-memd.sh\" external_source_route=\"MEMD_ALLOW_CLAWCONTROL_SYNC=1 CAPTURE_HTTP=1 IMPORT_CLAWCONTROL_BUNDLE=1 scripts/live-state-sync-clawcontrol.sh\" external_source_note=\"reads already-running ClawControl only; does not launch it\" api_bases={}",
                api_bases
            )
        } else if live_state_source_is_approved_communications(source)
            && (source.status == "missing_approval" || source.status == "invalid_approval")
        {
            format!(
                " producer_route=\"scripts/live-state-capture-approved-communications.mjs\" approved_zero_route=\"{}\" approved_zero_note=\"only when the user/process explicitly approves zero message/email metadata\"",
                approved_communications_empty_approval_command()
            )
        } else {
            String::new()
        };
        let approval_request = if live_state_source_is_approved_communications(source)
            && matches!(
                source.status.as_str(),
                "missing_approval" | "invalid_approval"
            ) {
            source
                .approval_request_path
                .as_deref()
                .map(|path| format!(" approval_request=\"{}\"", path))
                .unwrap_or_default()
        } else {
            String::new()
        };
        details.push(format!(
            "{}:status={} missing={}{}{}{}",
            source_name, source.status, missing, access_route, producer_route, approval_request
        ));
    }

    if details.is_empty()
        && (report.sync_required || report.requirement_missing > 0 || report.requirement_stale > 0)
    {
        details.push(format!(
            "requirements_missing={} requirements_stale={} sync_required={}",
            report.requirement_missing, report.requirement_stale, report.sync_required
        ));
    }

    (!details.is_empty()).then(|| details.join(";"))
}

pub(crate) fn live_state_check_required(report: &LiveAppStateReport, due_within_secs: i64) -> bool {
    if report.sync_required {
        return true;
    }
    let due_within_secs = due_within_secs.max(0);
    report.next_refresh_at <= report.checked_at + Duration::seconds(due_within_secs)
}

fn write_live_app_state(output: &Path, store: &LiveAppStateStore) -> anyhow::Result<()> {
    let path = live_app_state_path(output);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create live state dir {}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(store)?;
    std::fs::write(&path, text).with_context(|| format!("write live app state {}", path.display()))
}

fn read_payload(args: &LiveStateIngestArgs) -> anyhow::Result<Value> {
    if let Some(raw) = args.payload_json.as_deref() {
        return serde_json::from_str(raw).context("parse --payload-json");
    }
    if let Some(path) = args.payload_file.as_deref() {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("read --payload-file {}", path.display()))?;
        return serde_json::from_str(&text).context("parse --payload-file JSON");
    }
    Ok(Value::Object(Default::default()))
}

fn read_batch_body(args: &LiveStateIngestBatchArgs) -> anyhow::Result<LiveStateIngestBatchBody> {
    let sources = args.stdin as usize
        + args.input_json.is_some() as usize
        + args.input_file.is_some() as usize;
    if sources != 1 {
        bail!("provide exactly one of --stdin, --input-json, or --input-file");
    }
    let text = if args.stdin {
        let mut text = String::new();
        std::io::stdin()
            .read_to_string(&mut text)
            .context("read live-state batch from stdin")?;
        text
    } else if let Some(raw) = args.input_json.as_deref() {
        raw.to_string()
    } else if let Some(path) = args.input_file.as_deref() {
        std::fs::read_to_string(path)
            .with_context(|| format!("read --input-file {}", path.display()))?
    } else {
        unreachable!("exactly one batch input source checked above");
    };
    serde_json::from_str(&text).context("parse live-state batch JSON")
}

fn validate_live_state_privacy(args: &LiveStateIngestArgs, payload: &Value) -> anyhow::Result<()> {
    let module = args.module.to_ascii_lowercase();
    let privacy = args.privacy.to_ascii_lowercase();
    let visibility = args.visibility.to_ascii_lowercase();
    if !matches!(visibility.as_str(), "private" | "workspace" | "public") {
        bail!("invalid visibility; expected private, workspace, or public");
    }
    if !matches!(
        privacy.as_str(),
        "metadata" | "redacted" | "approved" | "aggregate" | "public"
    ) {
        bail!("invalid privacy; expected metadata, redacted, approved, aggregate, or public");
    }
    let sensitive_module = matches!(
        module.as_str(),
        "messages" | "texts" | "text_messages" | "imessage" | "email" | "mail"
    );
    let personal_state_module = matches!(
        module.as_str(),
        "calendar"
            | "calendars"
            | "email"
            | "mail"
            | "messages"
            | "reminder"
            | "reminders"
            | "tasks"
            | "text_messages"
            | "texts"
            | "todos"
            | "imessage"
    );
    if personal_state_module && visibility != "private" {
        bail!("personal live state must use --visibility private");
    }
    if personal_state_module && privacy == "public" {
        bail!("personal live state must not use public privacy");
    }
    if sensitive_module && !args.approved && !matches!(privacy.as_str(), "metadata" | "aggregate") {
        bail!("sensitive live state requires --approved unless privacy is metadata or aggregate");
    }
    let media_state = labels_contain_media(&args.label) || payload_contains_media_hint(payload);
    if sensitive_module && media_state {
        if !args.agentsecrets_approved {
            bail!("sensitive media live state requires --agentsecrets-approved");
        }
        if !matches!(privacy.as_str(), "metadata" | "redacted") {
            bail!("sensitive media live state must use metadata or redacted privacy");
        }
    }
    if sensitive_module && payload_contains_raw_media(payload) {
        bail!("raw message media must stay behind AgentSecrets; store refs or metadata only");
    }
    Ok(())
}

fn labels_contain_media(labels: &[String]) -> bool {
    labels.iter().any(|label| {
        matches!(
            label.trim().to_ascii_lowercase().as_str(),
            "attachment"
                | "attachments"
                | "audio"
                | "file"
                | "files"
                | "image"
                | "images"
                | "media"
                | "message-file"
                | "message-files"
                | "photo"
                | "photos"
                | "video"
        )
    })
}

fn payload_contains_media_hint(value: &Value) -> bool {
    match value {
        Value::Object(map) => map.iter().any(|(key, value)| {
            is_media_key(key) || payload_contains_media_hint(value) || value_has_media_type(value)
        }),
        Value::Array(items) => items.iter().any(payload_contains_media_hint),
        Value::String(text) => looks_like_media_reference(text),
        _ => false,
    }
}

fn payload_contains_raw_media(value: &Value) -> bool {
    match value {
        Value::Object(map) => map.iter().any(|(key, value)| {
            (is_raw_media_key(key) && value_has_raw_media(value))
                || payload_contains_raw_media(value)
        }),
        Value::Array(items) => items.iter().any(payload_contains_raw_media),
        Value::String(text) => looks_like_raw_media_blob(text),
        _ => false,
    }
}

fn value_has_media_type(value: &Value) -> bool {
    match value {
        Value::String(text) => matches!(
            text.trim().to_ascii_lowercase().as_str(),
            "attachment" | "audio" | "file" | "image" | "media" | "photo" | "video"
        ),
        _ => false,
    }
}

fn value_has_raw_media(value: &Value) -> bool {
    match value {
        Value::String(text) => looks_like_raw_media_blob(text),
        _ => false,
    }
}

fn is_media_key(key: &str) -> bool {
    matches!(
        key.trim().to_ascii_lowercase().as_str(),
        "attachment"
            | "attachments"
            | "audio"
            | "file"
            | "files"
            | "image"
            | "images"
            | "media"
            | "message_file"
            | "message_files"
            | "photo"
            | "photos"
            | "video"
    )
}

fn is_raw_media_key(key: &str) -> bool {
    matches!(
        key.trim().to_ascii_lowercase().as_str(),
        "base64" | "blob" | "bytes" | "content" | "data" | "data_url" | "payload"
    )
}

fn looks_like_media_reference(text: &str) -> bool {
    let lower = text.trim().to_ascii_lowercase();
    lower.starts_with("data:image/")
        || lower.starts_with("data:video/")
        || lower.starts_with("data:audio/")
        || lower.ends_with(".gif")
        || lower.ends_with(".heic")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".jpg")
        || lower.ends_with(".m4a")
        || lower.ends_with(".mov")
        || lower.ends_with(".mp3")
        || lower.ends_with(".mp4")
        || lower.ends_with(".png")
        || lower.ends_with(".wav")
        || lower.ends_with(".webp")
}

fn looks_like_raw_media_blob(text: &str) -> bool {
    let trimmed = text.trim();
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("data:image/")
        || lower.starts_with("data:video/")
        || lower.starts_with("data:audio/")
        || (trimmed.len() > 256
            && trimmed.chars().all(|ch| {
                ch.is_ascii_alphanumeric() || matches!(ch, '+' | '/' | '=' | '\n' | '\r')
            }))
}

fn hash_payload(payload: &Value) -> String {
    let text = serde_json::to_string(payload).unwrap_or_default();
    let digest = Sha256::digest(text.as_bytes());
    format!("{digest:x}")
}

fn normalize_key(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn compact_live_state_text(value: &str, max_chars: usize) -> String {
    let compact = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= max_chars {
        compact
    } else {
        let mut text = compact
            .chars()
            .take(max_chars.saturating_sub(3))
            .collect::<String>();
        text.push_str("...");
        text
    }
}

fn format_live_state_record(record: &LiveAppStateRecord) -> String {
    let labels = if record.labels.is_empty() {
        "none".to_string()
    } else {
        record.labels.join(",")
    };
    format!(
        "- {}:{} scope={} visibility={} privacy={} approved={} agentsecrets_approved={} fresh_until={} labels={} summary={}",
        record.source_app,
        record.module,
        record.scope,
        record.visibility,
        record.privacy,
        record.approved,
        record.agentsecrets_approved,
        record.expires_at.to_rfc3339(),
        labels,
        compact_live_state_text(&record.summary, 180)
    )
}

fn render_live_state_requirement_lines(
    records: &[LiveAppStateRecord],
    now: chrono::DateTime<Utc>,
) -> Vec<String> {
    live_state_requirement_statuses(records, now)
        .into_iter()
        .map(|requirement| {
            let matched_scope = requirement
                .matched_scope
                .as_deref()
                .map(|scope| format!(" matched_scope={scope}"))
                .unwrap_or_default();
            format!(
                "- required:{}:{} scope={} accepted_scopes={} status={}{} privacy_route=\"{}\" action=\"{}\"",
                requirement.source_app,
                requirement.module,
                requirement.canonical_scope,
                requirement.accepted_scopes.join(","),
                requirement.status,
                matched_scope,
                requirement.privacy_route,
                requirement.action,
            )
        })
        .collect()
}

fn live_state_source_status_is_fresh(
    source: &LiveAppStateSourceStatus,
    now: chrono::DateTime<Utc>,
) -> bool {
    source.checked_at + Duration::seconds(LIVE_STATE_SOURCE_STATUS_FRESH_SECS) > now
}

fn render_live_state_source_status_lines(
    sources: &[LiveAppStateSourceStatus],
    now: chrono::DateTime<Utc>,
    requirements: &[LiveAppStateRequirementStatus],
) -> Vec<String> {
    sources
        .iter()
        .map(|source| {
            let source_name = live_state_source_name(source);
            let api_base = source.api_base.as_deref().unwrap_or("unknown");
            let api_bases = if source.api_bases.is_empty() {
                api_base.to_string()
            } else {
                source.api_bases.join(",")
            };
            let visible_page = source.visible_page.as_deref().unwrap_or("unknown");
            let missing = if source.missing.is_empty() {
                "none".to_string()
            } else {
                source.missing.join(",")
            };
            let error = source.last_error.as_deref().unwrap_or("none");
            let approval_request = source.approval_request_path.as_deref().unwrap_or("none");
            let fresh_until =
                source.checked_at + Duration::seconds(LIVE_STATE_SOURCE_STATUS_FRESH_SECS);
            let freshness = if fresh_until > now { "fresh" } else { "stale" };
            let state_map_fresh = source_requirement_modules(requirements, source_name, "fresh");
            let state_map_unmet =
                source_requirement_modules_not_status(requirements, source_name, "fresh");
            format!(
                "source_status:{} status={} freshness={} checked_at={} fresh_until={} api_base={} api_bases={} auth_configured={} visible_page={} produced={} missing={} state_map_fresh={} state_map_unmet={} endpoints={} approval_request={} error=\"{}\"",
                source_name,
                source.status,
                freshness,
                source.checked_at.to_rfc3339(),
                fresh_until.to_rfc3339(),
                shell_quote(api_base),
                api_bases,
                source.auth_configured,
                visible_page,
                source.record_count,
                missing,
                render_module_list(&state_map_fresh),
                render_module_list(&state_map_unmet),
                source.endpoints.len(),
                shell_quote(approval_request),
                compact_live_state_text(error, 160)
            )
        })
        .collect()
}

fn source_requirement_modules(
    requirements: &[LiveAppStateRequirementStatus],
    source_app: &str,
    status: &str,
) -> Vec<String> {
    requirements
        .iter()
        .filter(|requirement| requirement.source_app == source_app && requirement.status == status)
        .map(|requirement| requirement.module.clone())
        .collect()
}

fn source_requirement_modules_not_status(
    requirements: &[LiveAppStateRequirementStatus],
    source_app: &str,
    status: &str,
) -> Vec<String> {
    requirements
        .iter()
        .filter(|requirement| requirement.source_app == source_app && requirement.status != status)
        .map(|requirement| requirement.module.clone())
        .collect()
}

fn render_module_list(modules: &[String]) -> String {
    if modules.is_empty() {
        "none".to_string()
    } else {
        modules.join(",")
    }
}

fn render_live_state_sync_task_lines(
    records: &[LiveAppStateRecord],
    now: chrono::DateTime<Utc>,
) -> Vec<String> {
    let requirements = live_state_requirement_statuses(records, now);
    let tasks = live_state_sync_tasks(&requirements);
    if tasks.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![format!(
        "- sync_required=true sync_tasks={} source=memd-live-state-status",
        tasks.len()
    )];
    lines.extend(tasks.into_iter().map(|task| {
        format!(
            "- sync_task:{}:{} scope={} status={} privacy={} visibility={} approved_required={} media_agentsecrets={} action=\"{}\"",
            task.source_app,
            task.module,
            task.required_scope,
            task.status,
            task.privacy,
            task.visibility,
            task.approved_required,
            task.agentsecrets_required_for_media,
            task.action
        )
    }));
    lines
}

fn live_state_requirement_statuses(
    records: &[LiveAppStateRecord],
    now: chrono::DateTime<Utc>,
) -> Vec<LiveAppStateRequirementStatus> {
    LIVE_APP_STATE_REQUIREMENTS
        .iter()
        .map(|requirement| {
            let mut fresh_scope = None;
            let mut stale_scope = None;
            for record in records.iter().filter(|record| {
                live_state_record_matches_requirement(record, requirement)
                    && live_state_record_eligible_for_requirement(record, requirement)
            }) {
                if record.expires_at > now {
                    fresh_scope = Some(record.scope.clone());
                    break;
                }
                if stale_scope.is_none() {
                    stale_scope = Some(record.scope.clone());
                }
            }
            let (status, matched_scope) = if let Some(scope) = fresh_scope {
                ("fresh", Some(scope))
            } else if let Some(scope) = stale_scope {
                ("stale", Some(scope))
            } else {
                ("missing", None)
            };
            LiveAppStateRequirementStatus {
                source_app: requirement.source_app.to_string(),
                module: requirement.module.to_string(),
                canonical_scope: requirement.canonical_scope.to_string(),
                accepted_scopes: requirement
                    .accepted_scopes
                    .iter()
                    .map(|scope| (*scope).to_string())
                    .collect(),
                status: status.to_string(),
                matched_scope,
                privacy_route: requirement.privacy_route.to_string(),
                action: requirement.action.to_string(),
            }
        })
        .collect()
}

fn live_state_record_matches_requirement(
    record: &LiveAppStateRecord,
    requirement: &LiveAppStateRequirement,
) -> bool {
    live_state_source_satisfies_requirement(&record.source_app, requirement)
        && record.module == requirement.module
        && requirement
            .accepted_scopes
            .iter()
            .any(|scope| record.scope == *scope)
}

fn live_state_record_eligible_for_requirement(
    record: &LiveAppStateRecord,
    requirement: &LiveAppStateRequirement,
) -> bool {
    if !sensitive_communication_module(requirement.module) {
        return true;
    }
    record.approved
        && record.visibility == "private"
        && safe_communication_requirement_privacy(&record.privacy)
        && (!communication_record_has_media(record) || record.agentsecrets_approved)
}

fn safe_communication_requirement_privacy(privacy: &str) -> bool {
    matches!(privacy, "metadata" | "redacted" | "approved")
}

fn communication_record_has_media(record: &LiveAppStateRecord) -> bool {
    labels_contain_media(&record.labels) || payload_contains_media_hint(&record.payload)
}

fn live_state_sync_actions(requirements: &[LiveAppStateRequirementStatus]) -> Vec<String> {
    requirements
        .iter()
        .filter(|requirement| requirement.status != "fresh")
        .map(|requirement| {
            format!(
                "{}:{} status={} action={}",
                requirement.source_app, requirement.module, requirement.status, requirement.action
            )
        })
        .collect()
}

fn live_state_next_refresh(
    records: &[LiveAppStateRecord],
    now: chrono::DateTime<Utc>,
    requirements: &[LiveAppStateRequirementStatus],
) -> (chrono::DateTime<Utc>, String) {
    if let Some(requirement) = requirements
        .iter()
        .find(|requirement| requirement.status == "missing")
    {
        return (
            now,
            format!(
                "missing required live-state surface {}:{}",
                requirement.source_app, requirement.module
            ),
        );
    }
    if let Some(requirement) = requirements
        .iter()
        .find(|requirement| requirement.status == "stale")
    {
        return (
            now,
            format!(
                "stale required live-state surface {}:{}",
                requirement.source_app, requirement.module
            ),
        );
    }
    let Some(next_expiry) = records
        .iter()
        .filter(|record| record.expires_at > now)
        .map(|record| record.expires_at)
        .min()
    else {
        return (now, "no live-state records available".to_string());
    };
    (next_expiry, "earliest live-state record expiry".to_string())
}

fn live_state_sync_tasks(
    requirements: &[LiveAppStateRequirementStatus],
) -> Vec<LiveAppStateSyncTask> {
    requirements
        .iter()
        .filter(|requirement| requirement.status != "fresh")
        .map(live_state_sync_task)
        .collect()
}

fn live_state_sync_task(requirement: &LiveAppStateRequirementStatus) -> LiveAppStateSyncTask {
    let privacy = default_sync_privacy(&requirement.module).to_string();
    let visibility = "private".to_string();
    let approved_required = sensitive_communication_module(&requirement.module);
    let agentsecrets_required_for_media = sensitive_communication_module(&requirement.module);
    let freshness_secs = LIVE_STATE_DEFAULT_REFRESH_SECS;
    let labels = default_sync_labels(&requirement.module);
    let summary_hint = sync_summary_hint(&requirement.module).to_string();
    let payload_hint = sync_payload_hint(&requirement.module).to_string();
    let mut ingest_argv = vec![
        "memd".to_string(),
        "live-state".to_string(),
        "ingest".to_string(),
        "--source".to_string(),
        requirement.source_app.clone(),
        "--module".to_string(),
        requirement.module.clone(),
        "--scope".to_string(),
        requirement.canonical_scope.clone(),
        "--visibility".to_string(),
        visibility.clone(),
        "--privacy".to_string(),
        privacy.clone(),
        "--freshness-secs".to_string(),
        freshness_secs.to_string(),
        "--summary".to_string(),
        summary_hint.clone(),
        "--payload-json".to_string(),
        payload_hint.clone(),
    ];
    if approved_required {
        ingest_argv.push("--approved".to_string());
    }
    for label in &labels {
        ingest_argv.push("--label".to_string());
        ingest_argv.push(label.clone());
    }
    LiveAppStateSyncTask {
        source_app: requirement.source_app.clone(),
        module: requirement.module.clone(),
        required_scope: requirement.canonical_scope.clone(),
        accepted_scopes: requirement.accepted_scopes.clone(),
        status: requirement.status.clone(),
        visibility,
        privacy,
        approved_required,
        agentsecrets_required_for_media,
        freshness_secs,
        labels,
        summary_hint,
        payload_hint,
        ingest_argv,
        action: requirement.action.clone(),
    }
}

fn sensitive_communication_module(module: &str) -> bool {
    matches!(
        module,
        "messages" | "email" | "text_messages" | "texts" | "imessage" | "mail"
    )
}

fn default_sync_privacy(module: &str) -> &'static str {
    match module {
        "messages" | "email" => "metadata",
        _ => "metadata",
    }
}

fn default_sync_labels(module: &str) -> Vec<String> {
    match module {
        "messages" => vec!["messages".to_string(), "metadata".to_string()],
        "email" => vec!["email".to_string(), "metadata".to_string()],
        value => vec![value.to_string()],
    }
}

fn sync_summary_hint(module: &str) -> &'static str {
    match module {
        "visible_page" => "visible page/module, route, selected item, and visible facts",
        "calendar" => {
            "current and next calendar events with times, calendars, and privacy-safe titles"
        }
        "reminders" => "active reminders with due dates, list names, and completion state",
        "todos" => "active todos/tasks with priority, due dates, and completion state",
        "messages" => {
            "approved text-message metadata or redacted idea context; no unrestricted chat content"
        }
        "email" => "approved email metadata or redacted snippets; headers first, no mailbox dump",
        _ => "current module state",
    }
}

fn sync_payload_hint(module: &str) -> &'static str {
    match module {
        "visible_page" => {
            r#"{"route":"/current","title":"visible title","facts":[],"selected_item":null}"#
        }
        "calendar" => r#"{"events":[],"range":"current-and-next"}"#,
        "reminders" => r#"{"reminders":[]}"#,
        "todos" => r#"{"todos":[]}"#,
        "messages" => {
            r#"{"mode":"metadata-only","threads":[],"idea_context":null,"raw_media_stored":false,"approval_policy":"approved=true required; redactedSnippet requires redacted=true or redactionApproved=true; set agentsecretsApproved=true for attachment/media metadata; raw chat/media forbidden"}"#
        }
        "email" => {
            r#"{"mode":"approved-metadata","messages":[],"raw_body_stored":false,"approval_policy":"approved=true required; redactedSnippet requires redacted=true or redactionApproved=true; set agentsecretsApproved=true for attachment/media metadata; raw mail/media forbidden"}"#
        }
        _ => "{}",
    }
}

pub(crate) fn render_live_state_summary(report: &LiveAppStateReport) -> String {
    let mut lines = vec![format!(
        "live_state status={} total={} fresh={} stale={} requirement_fresh={} requirement_stale={} requirement_missing={} sync_required={} sync_actions={} sync_tasks={} source_fresh={} source_stale={} source_unavailable={} next_refresh_at={} refresh_reason=\"{}\" contract={} path={} source_status_path={}",
        report.status,
        report.total,
        report.fresh,
        report.stale,
        report.requirement_fresh,
        report.requirement_stale,
        report.requirement_missing,
        report.sync_required,
        report.sync_actions.len(),
        report.sync_tasks.len(),
        report.source_fresh,
        report.source_stale,
        report.source_unavailable,
        report.next_refresh_at.to_rfc3339(),
        compact_live_state_text(&report.refresh_reason, 160),
        report.producer_contract_version,
        report.path,
        report.source_status_path
    )];
    lines.extend(report.records.iter().take(12).map(|record| {
        format!(
            "{}:{} privacy={} visibility={} approved={} agentsecrets_approved={} expires={} summary={}",
            record.source_app,
            record.module,
            record.privacy,
            record.visibility,
            record.approved,
            record.agentsecrets_approved,
            record.expires_at.to_rfc3339(),
            compact_live_state_text(&record.summary, 120)
        )
    }));
    lines.extend(report.requirements.iter().map(|requirement| {
        let matched_scope = requirement
            .matched_scope
            .as_deref()
            .map(|scope| format!(" matched_scope={scope}"))
            .unwrap_or_default();
        format!(
            "required:{}:{} scope={} accepted_scopes={} status={}{} privacy_route=\"{}\"",
            requirement.source_app,
            requirement.module,
            requirement.canonical_scope,
            requirement.accepted_scopes.join(","),
            requirement.status,
            matched_scope,
            requirement.privacy_route
        )
    }));
    lines.extend(render_live_state_source_status_lines(
        &report.source_statuses,
        report.checked_at,
        &report.requirements,
    ));
    lines.extend(
        report
            .sync_actions
            .iter()
            .map(|action| format!("sync_action:{action}")),
    );
    lines.extend(report.sync_tasks.iter().map(|task| {
        format!(
            "sync_task:{}:{} scope={} status={} privacy={} visibility={} approved_required={} agentsecrets_required_for_media={}",
            task.source_app,
            task.module,
            task.required_scope,
            task.status,
            task.privacy,
            task.visibility,
            task.approved_required,
            task.agentsecrets_required_for_media
        )
    }));
    lines.join("\n")
}

pub(crate) fn render_live_state_task_lines(report: &LiveAppStateReport) -> String {
    if report.sync_tasks.is_empty() {
        return format!(
            "live_state_tasks sync_required=false next_refresh_at={} reason=\"{}\"",
            report.next_refresh_at.to_rfc3339(),
            compact_live_state_text(&report.refresh_reason, 160)
        );
    }
    let mut lines = vec![format!(
        "live_state_tasks sync_required={} count={} next_refresh_at={} reason=\"{}\"",
        report.sync_required,
        report.sync_tasks.len(),
        report.next_refresh_at.to_rfc3339(),
        compact_live_state_text(&report.refresh_reason, 160)
    )];
    lines.extend(report.sync_tasks.iter().map(|task| {
        let approved_route = approved_communications_task_route_detail(report, task);
        format!(
            "task source={} module={} scope={} status={} privacy={} visibility={} approved_required={} freshness_secs={} media_agentsecrets={} labels={}{} action=\"{}\"",
            task.source_app,
            task.module,
            task.required_scope,
            task.status,
            task.privacy,
            task.visibility,
            task.approved_required,
            task.freshness_secs,
            task.agentsecrets_required_for_media,
            task.labels.join(","),
            approved_route,
            compact_live_state_text(&task.action, 220)
        )
    }));
    lines.join("\n")
}

fn approved_communications_task_route_detail(
    report: &LiveAppStateReport,
    task: &LiveAppStateSyncTask,
) -> String {
    if task.source_app != "approved_communications" {
        return String::new();
    }
    let request = report
        .source_statuses
        .iter()
        .find(|source| {
            live_state_source_is_approved_communications(source)
                && matches!(
                    source.status.as_str(),
                    "missing_approval" | "invalid_approval"
                )
        })
        .and_then(|source| source.approval_request_path.as_deref());
    let request = request.unwrap_or(".memd/state/approved-communications-request.json");
    let template = approved_communications_template_path_from_request(request)
        .unwrap_or_else(|| ".memd/state/approved-communications-template.json".to_string());
    format!(
        " access_route=\"{}\" producer_route=\"scripts/live-state-capture-approved-communications.mjs\" approved_zero_route=\"{}\" approval_request={} approved_template={}",
        approved_communications_access_route_command(),
        approved_communications_empty_approval_command(),
        shell_quote(request),
        shell_quote(&template)
    )
}

fn approved_communications_template_path_from_request(request: &str) -> Option<String> {
    let value = std::fs::read_to_string(request)
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())?;
    value
        .get("approval")
        .and_then(|approval| approval.get("approvedFileTemplate"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

pub(crate) fn render_live_state_command_lines(report: &LiveAppStateReport) -> String {
    if report.sync_tasks.is_empty() {
        return format!(
            "# live_state_commands sync_required=false next_refresh_at={} reason={}",
            shell_quote(&report.next_refresh_at.to_rfc3339()),
            shell_quote(&report.refresh_reason)
        );
    }
    let mut lines = vec![format!(
        "# live_state_commands sync_required={} count={} next_refresh_at={} reason={}",
        report.sync_required,
        report.sync_tasks.len(),
        shell_quote(&report.next_refresh_at.to_rfc3339()),
        shell_quote(&report.refresh_reason)
    )];
    lines.extend(report.sync_tasks.iter().map(|task| {
        task.ingest_argv
            .iter()
            .map(|arg| shell_quote(arg))
            .collect::<Vec<_>>()
            .join(" ")
    }));
    lines.push(
        "# replace template payload_json/summary with current producer data before running"
            .to_string(),
    );
    lines.join("\n")
}

pub(crate) fn render_live_state_batch_template(report: &LiveAppStateReport) -> String {
    let records = report
        .sync_tasks
        .iter()
        .map(|task| {
            let payload = serde_json::from_str::<Value>(&task.payload_hint)
                .unwrap_or_else(|_| Value::String(task.payload_hint.clone()));
            serde_json::json!({
                "sourceApp": task.source_app,
                "module": task.module,
                "scope": task.required_scope,
                "visibility": task.visibility,
                "privacy": task.privacy,
                "approved": task.approved_required,
                "agentsecretsApproved": false,
                "freshnessSecs": task.freshness_secs,
                "labels": task.labels,
                "summary": task.summary_hint,
                "payload": payload,
            })
        })
        .collect::<Vec<_>>();
    serde_json::to_string_pretty(&serde_json::json!({ "records": records }))
        .expect("serialize live-state batch template")
}

fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/' | ':' | '='))
    {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
include!("cli_live_state_runtime_tests.rs");
