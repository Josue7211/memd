use super::*;
use anyhow::{Context, bail};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

const LIVE_STATE_VERSION: u32 = 1;

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
    pub(crate) total: usize,
    pub(crate) fresh: usize,
    pub(crate) stale: usize,
    pub(crate) records: Vec<LiveAppStateRecord>,
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
    let mut records = store
        .records
        .into_iter()
        .filter(|record| record.expires_at > now)
        .collect::<Vec<_>>();
    records.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    if records.is_empty() {
        return "- none; producers should run `memd live-state ingest` with privacy labels"
            .to_string();
    }
    records
        .into_iter()
        .take(limit)
        .map(|record| {
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
        })
        .collect::<Vec<_>>()
        .join("\n")
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
    Ok(LiveAppStateReport {
        status: if store.records.is_empty() {
            "empty".to_string()
        } else if stale > 0 {
            "stale".to_string()
        } else {
            "fresh".to_string()
        },
        path: path.display().to_string(),
        total: store.records.len(),
        fresh,
        stale,
        records: store.records,
    })
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
    if sensitive_module && !args.approved && !matches!(privacy.as_str(), "metadata" | "aggregate") {
        bail!("sensitive live state requires --approved unless privacy is metadata or aggregate");
    }
    if sensitive_module && visibility != "private" {
        bail!("sensitive live state must use --visibility private");
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

pub(crate) fn render_live_state_summary(report: &LiveAppStateReport) -> String {
    let mut lines = vec![format!(
        "live_state status={} total={} fresh={} stale={} path={}",
        report.status, report.total, report.fresh, report.stale, report.path
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
    lines.join("\n")
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
        assert!(section.contains("privacy=approved"));
        assert!(section.contains("Dentist"));
    }
}
