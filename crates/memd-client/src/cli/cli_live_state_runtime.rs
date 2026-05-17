use super::*;
use anyhow::{Context, bail};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fmt;

const LIVE_STATE_VERSION: u32 = 1;
const LIVE_STATE_PRODUCER_CONTRACT_VERSION: u32 = 1;
const LIVE_STATE_DEFAULT_REFRESH_SECS: i64 = 86_400;

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
        source_app: "clawcontrol",
        module: "visible_page",
        canonical_scope: "current",
        accepted_scopes: &["current"],
        privacy_route: "private metadata",
        action: "capture visible app/page state before answering present-tense UI questions",
    },
    LiveAppStateRequirement {
        source_app: "clawcontrol",
        module: "calendar",
        canonical_scope: "primary",
        accepted_scopes: &["primary", "current"],
        privacy_route: "private approved or metadata",
        action: "ingest current/next calendar events before answering calendar questions",
    },
    LiveAppStateRequirement {
        source_app: "clawcontrol",
        module: "reminders",
        canonical_scope: "default",
        accepted_scopes: &["default", "current"],
        privacy_route: "private approved or metadata",
        action: "ingest active reminders before answering reminder questions",
    },
    LiveAppStateRequirement {
        source_app: "clawcontrol",
        module: "todos",
        canonical_scope: "default",
        accepted_scopes: &["default", "current"],
        privacy_route: "private approved or metadata",
        action: "ingest active todos before answering task questions",
    },
    LiveAppStateRequirement {
        source_app: "clawcontrol",
        module: "messages",
        canonical_scope: "approved",
        accepted_scopes: &["approved", "current"],
        privacy_route: "private metadata/redacted; no unrestricted chat access",
        action: "ingest only approved text-message metadata or redacted snippets",
    },
    LiveAppStateRequirement {
        source_app: "clawcontrol",
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
    pub(crate) records: Vec<LiveAppStateRecord>,
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

pub(crate) fn live_app_state_path(output: &Path) -> PathBuf {
    output.join("state").join("live-app-state.json")
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
    lines.extend(render_live_state_requirement_lines(&records, now));
    lines.join("\n")
}

pub(crate) fn run_live_state_command(args: &LiveStateArgs) -> anyhow::Result<LiveAppStateReport> {
    match &args.command {
        LiveStateSubcommand::Ingest(ingest) => ingest_live_state(ingest),
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

pub(crate) fn live_state_report(output: &Path) -> anyhow::Result<LiveAppStateReport> {
    let path = live_app_state_path(output);
    let store = read_live_app_state(output)?;
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
        records: store.records,
    })
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
            "- sync_task:{}:{} scope={} status={} privacy={} visibility={} media_agentsecrets={} action=\"{}\"",
            task.source_app,
            task.module,
            task.required_scope,
            task.status,
            task.privacy,
            task.visibility,
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
            let matching = records.iter().find(|record| {
                record.source_app == requirement.source_app
                    && record.module == requirement.module
                    && requirement
                        .accepted_scopes
                        .iter()
                        .any(|scope| record.scope == *scope)
            });
            let status = match matching {
                Some(record) if record.expires_at > now => "fresh",
                Some(_) => "stale",
                None => "missing",
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
                matched_scope: matching.map(|record| record.scope.clone()),
                privacy_route: requirement.privacy_route.to_string(),
                action: requirement.action.to_string(),
            }
        })
        .collect()
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
    let approved_required = false;
    let agentsecrets_required_for_media = matches!(
        requirement.module.as_str(),
        "messages" | "email" | "text_messages" | "texts" | "imessage" | "mail"
    );
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
            r#"{"mode":"metadata-only","threads":[],"idea_context":null,"raw_media_stored":false}"#
        }
        "email" => r#"{"mode":"approved-metadata","messages":[],"raw_body_stored":false}"#,
        _ => "{}",
    }
}

pub(crate) fn render_live_state_summary(report: &LiveAppStateReport) -> String {
    let mut lines = vec![format!(
        "live_state status={} total={} fresh={} stale={} requirement_fresh={} requirement_stale={} requirement_missing={} sync_required={} sync_actions={} sync_tasks={} next_refresh_at={} refresh_reason=\"{}\" contract={} path={}",
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
        report.next_refresh_at.to_rfc3339(),
        compact_live_state_text(&report.refresh_reason, 160),
        report.producer_contract_version,
        report.path
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
    lines.extend(
        report
            .sync_actions
            .iter()
            .map(|action| format!("sync_action:{action}")),
    );
    lines.extend(report.sync_tasks.iter().map(|task| {
        format!(
            "sync_task:{}:{} scope={} status={} privacy={} visibility={} agentsecrets_required_for_media={}",
            task.source_app,
            task.module,
            task.required_scope,
            task.status,
            task.privacy,
            task.visibility,
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
        format!(
            "task source={} module={} scope={} status={} privacy={} visibility={} freshness_secs={} media_agentsecrets={} labels={} action=\"{}\"",
            task.source_app,
            task.module,
            task.required_scope,
            task.status,
            task.privacy,
            task.visibility,
            task.freshness_secs,
            task.agentsecrets_required_for_media,
            task.labels.join(","),
            compact_live_state_text(&task.action, 220)
        )
    }));
    lines.join("\n")
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
mod tests {
    use super::*;

    #[test]
    fn live_state_rejects_unapproved_private_messages_payload() {
        let root = tempfile::tempdir().expect("tempdir");
        let args = LiveStateIngestArgs {
            output: root.path().join(".memd"),
            source: "clawcontrol".to_string(),
            module: "messages".to_string(),
            scope: "inbox".to_string(),
            visibility: "private".to_string(),
            privacy: "redacted".to_string(),
            approved: false,
            agentsecrets_approved: false,
            freshness_secs: 300,
            label: vec![],
            summary: "latest message from approved contact".to_string(),
            payload_json: Some(r#"{"latest":"redacted"}"#.to_string()),
            payload_file: None,
            json: false,
        };
        let err = ingest_live_state(&args).expect_err("must reject");
        assert!(err.to_string().contains("requires --approved"));
    }

    #[test]
    fn live_state_allows_messages_metadata_without_full_chat_access() {
        let root = tempfile::tempdir().expect("tempdir");
        let args = LiveStateIngestArgs {
            output: root.path().join(".memd"),
            source: "clawcontrol".to_string(),
            module: "messages".to_string(),
            scope: "idea-context".to_string(),
            visibility: "private".to_string(),
            privacy: "metadata".to_string(),
            approved: false,
            agentsecrets_approved: false,
            freshness_secs: 300,
            label: vec!["messages".to_string()],
            summary: "approved text metadata says the user is discussing a launch idea".to_string(),
            payload_json: Some(r#"{"topic":"launch idea","contact":"approved"}"#.to_string()),
            payload_file: None,
            json: false,
        };
        let report = ingest_live_state(&args).expect("metadata ingest");
        assert_eq!(report.total, 1);
        assert_eq!(report.records[0].privacy, "metadata");
    }

    #[test]
    fn live_state_rejects_public_personal_calendar_state() {
        let root = tempfile::tempdir().expect("tempdir");
        let args = LiveStateIngestArgs {
            output: root.path().join(".memd"),
            source: "clawcontrol".to_string(),
            module: "calendar".to_string(),
            scope: "primary".to_string(),
            visibility: "public".to_string(),
            privacy: "public".to_string(),
            approved: true,
            agentsecrets_approved: false,
            freshness_secs: 300,
            label: vec!["calendar".to_string()],
            summary: "next event Dentist".to_string(),
            payload_json: Some(r#"{"events":[{"title":"Dentist"}]}"#.to_string()),
            payload_file: None,
            json: false,
        };
        let err = ingest_live_state(&args).expect_err("must reject");
        assert!(err.to_string().contains("must use --visibility private"));
    }

    #[test]
    fn live_state_rejects_message_media_without_agentsecrets_approval() {
        let root = tempfile::tempdir().expect("tempdir");
        let args = LiveStateIngestArgs {
            output: root.path().join(".memd"),
            source: "clawcontrol".to_string(),
            module: "messages".to_string(),
            scope: "attachments".to_string(),
            visibility: "private".to_string(),
            privacy: "metadata".to_string(),
            approved: false,
            agentsecrets_approved: false,
            freshness_secs: 300,
            label: vec!["image".to_string()],
            summary: "thread contains an image attachment".to_string(),
            payload_json: Some(r#"{"attachments":[{"type":"image"}]}"#.to_string()),
            payload_file: None,
            json: false,
        };
        let err = ingest_live_state(&args).expect_err("must reject");
        assert!(err.to_string().contains("--agentsecrets-approved"));
    }

    #[test]
    fn live_state_rejects_raw_message_media_even_with_agentsecrets_approval() {
        let root = tempfile::tempdir().expect("tempdir");
        let args = LiveStateIngestArgs {
            output: root.path().join(".memd"),
            source: "clawcontrol".to_string(),
            module: "messages".to_string(),
            scope: "attachments".to_string(),
            visibility: "private".to_string(),
            privacy: "metadata".to_string(),
            approved: false,
            agentsecrets_approved: true,
            freshness_secs: 300,
            label: vec!["image".to_string()],
            summary: "thread contains an image attachment".to_string(),
            payload_json: Some(r#"{"data_url":"data:image/png;base64,AAAA"}"#.to_string()),
            payload_file: None,
            json: false,
        };
        let err = ingest_live_state(&args).expect_err("must reject");
        assert!(err.to_string().contains("must stay behind AgentSecrets"));
    }

    #[test]
    fn live_state_ingests_calendar_and_renders_context_section() {
        let root = tempfile::tempdir().expect("tempdir");
        let output = root.path().join(".memd");
        let args = LiveStateIngestArgs {
            output: output.clone(),
            source: "ClawControl".to_string(),
            module: "Calendar".to_string(),
            scope: "primary".to_string(),
            visibility: "private".to_string(),
            privacy: "approved".to_string(),
            approved: true,
            agentsecrets_approved: false,
            freshness_secs: 300,
            label: vec!["calendar".to_string()],
            summary: "next event Dentist at 2026-05-17T14:00:00Z".to_string(),
            payload_json: Some(r#"{"events":[{"title":"Dentist"}]}"#.to_string()),
            payload_file: None,
            json: false,
        };
        let report = ingest_live_state(&args).expect("ingest");
        assert_eq!(report.total, 1);
        assert_eq!(report.fresh, 1);

        let section = render_live_app_state_section(&output, 4);
        assert!(section.contains("clawcontrol:calendar"));
        assert!(section.contains("required:clawcontrol:calendar"));
        assert!(section.contains("status=fresh"));
        assert!(section.contains("required:clawcontrol:messages"));
        assert!(section.contains("no unrestricted chat access"));
        assert!(section.contains("privacy=approved"));
        assert!(section.contains("Dentist"));
    }

    #[test]
    fn live_state_requirement_report_accepts_clawcontrol_current_scope() {
        let root = tempfile::tempdir().expect("tempdir");
        let output = root.path().join(".memd");
        let args = LiveStateIngestArgs {
            output: output.clone(),
            source: "clawcontrol".to_string(),
            module: "calendar".to_string(),
            scope: "current".to_string(),
            visibility: "private".to_string(),
            privacy: "approved".to_string(),
            approved: true,
            agentsecrets_approved: false,
            freshness_secs: 300,
            label: vec!["calendar".to_string()],
            summary: "calendar: loaded; upcoming_events=1".to_string(),
            payload_json: Some(r#"{"events":[{"title":"Dentist"}]}"#.to_string()),
            payload_file: None,
            json: false,
        };
        let report = ingest_live_state(&args).expect("ingest current-scope calendar");
        let calendar_requirement = report
            .requirements
            .iter()
            .find(|requirement| requirement.module == "calendar")
            .expect("calendar requirement");
        assert_eq!(calendar_requirement.status, "fresh");
        assert_eq!(
            calendar_requirement.matched_scope.as_deref(),
            Some("current")
        );
        assert_eq!(calendar_requirement.canonical_scope, "primary");
        assert_eq!(report.requirement_fresh, 1);

        let summary = render_live_state_summary(&report);
        assert!(summary.contains("requirement_fresh=1"));
        assert!(summary.contains("required:clawcontrol:calendar"));
        assert!(summary.contains("matched_scope=current"));
    }

    #[test]
    fn live_state_check_can_warn_before_next_refresh() {
        let root = tempfile::tempdir().expect("tempdir");
        let output = root.path().join(".memd");
        let mut report = live_state_report(&output).expect("empty status");
        for (module, scope, payload_json) in [
            (
                "visible_page",
                "current",
                r#"{"route":"/calendar","title":"Calendar","facts":[]}"#,
            ),
            ("calendar", "primary", r#"{"events":[]}"#),
            ("reminders", "default", r#"{"reminders":[]}"#),
            ("todos", "default", r#"{"todos":[]}"#),
            (
                "messages",
                "approved",
                r#"{"mode":"metadata-only","threads":[],"raw_media_stored":false}"#,
            ),
            (
                "email",
                "approved",
                r#"{"mode":"approved-metadata","messages":[],"raw_body_stored":false}"#,
            ),
        ] {
            let args = LiveStateIngestArgs {
                output: output.clone(),
                source: "clawcontrol".to_string(),
                module: module.to_string(),
                scope: scope.to_string(),
                visibility: "private".to_string(),
                privacy: "metadata".to_string(),
                approved: false,
                agentsecrets_approved: false,
                freshness_secs: 60,
                label: vec![module.to_string()],
                summary: format!("{module} fixture fresh"),
                payload_json: Some(payload_json.to_string()),
                payload_file: None,
                json: false,
            };
            report = ingest_live_state(&args).expect("ingest fixture");
        }

        assert_eq!(report.status, "fresh");
        assert_eq!(report.requirement_missing, 0);
        assert_eq!(report.requirement_stale, 0);
        assert!(!report.sync_required);
        assert!(report.next_refresh_at > report.checked_at);
        assert!(!live_state_check_required(&report, 5));
        assert!(live_state_check_required(&report, 120));
    }

    #[test]
    fn live_state_empty_map_renders_required_sync_surface() {
        let root = tempfile::tempdir().expect("tempdir");
        let output = root.path().join(".memd");
        let section = render_live_app_state_section(&output, 8);
        assert!(section.contains("no fresh live app state"));
        assert!(section.contains("refresh_policy contract=1"));
        assert!(section.contains("reason=\"missing required live-state surface"));
        assert!(section.contains("default_ttl=86400s"));
        assert!(section.contains("required:clawcontrol:visible_page"));
        assert!(section.contains("required:clawcontrol:calendar"));
        assert!(section.contains("required:clawcontrol:reminders"));
        assert!(section.contains("required:clawcontrol:messages"));
        assert!(section.contains("sync_required=true sync_tasks=6"));
        assert!(section.contains("sync_task:clawcontrol:messages"));
        assert!(section.contains("media_agentsecrets=true"));
        assert!(section.contains("status=missing"));
        assert!(section.contains("private metadata/redacted; no unrestricted chat access"));

        let report = live_state_report(&output).expect("status report");
        assert_eq!(report.status, "missing_requirements");
        assert_eq!(
            report.requirement_missing,
            LIVE_APP_STATE_REQUIREMENTS.len()
        );
        assert_eq!(report.requirement_fresh, 0);
        assert!(report.sync_required);
        assert_eq!(report.checked_at, report.next_refresh_at);
        assert_eq!(report.producer_contract_version, 1);
        assert!(report.refresh_reason.contains("missing required"));
        assert!(
            report
                .refresh_policy
                .contains("default producer ttl=86400s")
        );
        assert_eq!(report.sync_actions.len(), LIVE_APP_STATE_REQUIREMENTS.len());
        assert_eq!(report.sync_tasks.len(), LIVE_APP_STATE_REQUIREMENTS.len());
        assert!(
            report
                .sync_actions
                .iter()
                .any(|action| action.contains("clawcontrol:visible_page status=missing"))
        );
        let messages_task = report
            .sync_tasks
            .iter()
            .find(|task| task.module == "messages")
            .expect("messages sync task");
        assert_eq!(messages_task.required_scope, "approved");
        assert_eq!(messages_task.privacy, "metadata");
        assert!(messages_task.agentsecrets_required_for_media);
        assert!(
            messages_task
                .ingest_argv
                .windows(2)
                .any(|items| items == ["--privacy", "metadata"])
        );
        let summary = render_live_state_summary(&report);
        assert!(summary.contains("sync_required=true"));
        assert!(summary.contains("next_refresh_at="));
        assert!(summary.contains("contract=1"));
        assert!(summary.contains("sync_action:clawcontrol:calendar status=missing"));
        assert!(summary.contains("sync_task:clawcontrol:messages"));

        let tasks = render_live_state_task_lines(&report);
        assert!(tasks.contains("live_state_tasks sync_required=true count=6"));
        assert!(tasks.contains("task source=clawcontrol module=visible_page"));
        assert!(tasks.contains("task source=clawcontrol module=messages"));
        assert!(tasks.contains("media_agentsecrets=true"));

        let commands = render_live_state_command_lines(&report);
        assert!(commands.contains("# live_state_commands sync_required=true count=6"));
        assert!(commands.contains("memd live-state ingest"));
        assert!(commands.contains("--module messages"));
        assert!(commands.contains("--scope approved"));
        assert!(commands.contains("'approved text-message metadata"));
    }
}
