use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use ulid::Ulid;

pub const TELEMETRY_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TelemetryEvent {
    pub schema_version: u32,
    pub ts: DateTime<Utc>,
    pub user_hash: String,
    pub harness: String,
    pub event_kind: String,
    pub source: String,
    pub token_count: u64,
    pub estimated_cost_usd: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_family: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub metadata: serde_json::Map<String, JsonValue>,
}

impl TelemetryEvent {
    pub fn new(
        user: &str,
        harness: &str,
        event_kind: &str,
        source: &str,
        token_count: u64,
        estimated_cost_usd: f64,
    ) -> Self {
        Self {
            schema_version: TELEMETRY_SCHEMA_VERSION,
            ts: Utc::now(),
            user_hash: hash_user_to_ulid(user),
            harness: scrub_text(harness),
            event_kind: scrub_text(event_kind),
            source: scrub_text(source),
            token_count,
            estimated_cost_usd,
            session_id: None,
            model_family: None,
            metadata: serde_json::Map::new(),
        }
    }

    pub fn scrubbed(mut self) -> Self {
        self.harness = scrub_text(&self.harness);
        self.event_kind = scrub_text(&self.event_kind);
        self.source = scrub_text(&self.source);
        self.session_id = self.session_id.as_deref().map(scrub_text);
        self.model_family = self.model_family.as_deref().map(scrub_text);
        self.metadata = self
            .metadata
            .into_iter()
            .map(|(key, value)| (scrub_text(&key), scrub_json(value)))
            .collect();
        self
    }
}

pub fn telemetry_dir(bundle_root: &Path) -> PathBuf {
    bundle_root.join("telemetry")
}

pub fn telemetry_events_path(bundle_root: &Path) -> PathBuf {
    telemetry_dir(bundle_root).join("events.ndjson")
}

pub fn append_telemetry_event(bundle_root: &Path, event: &TelemetryEvent) -> std::io::Result<()> {
    let target = telemetry_events_path(bundle_root);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(&target)?;
    let line = serde_json::to_string(&event.clone().scrubbed())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
    writeln!(file, "{line}")?;
    Ok(())
}

pub fn read_telemetry_events(bundle_root: &Path) -> anyhow::Result<Vec<TelemetryEvent>> {
    let path = telemetry_events_path(bundle_root);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(&path)?;
    let mut events = Vec::new();
    for (idx, line) in raw.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let event: TelemetryEvent = serde_json::from_str(line)
            .map_err(|err| anyhow::anyhow!("parse {} line {}: {err}", path.display(), idx + 1))?;
        events.push(event.scrubbed());
    }
    Ok(events)
}

pub fn hash_user_to_ulid(user: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(user.trim().as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    Ulid::from_bytes(bytes).to_string()
}

pub fn scrub_text(value: &str) -> String {
    let mut out = value.to_string();
    for (pattern, replacement) in [
        (
            r"(?i)[A-Z0-9._%+\-]+@[A-Z0-9.\-]+\.[A-Z]{2,}",
            "[redacted-email]",
        ),
        (r"\b(?:\d{1,3}\.){3}\d{1,3}\b", "[redacted-ip]"),
        (
            r"(?i)\b(?:sk|xox[baprs]|gh[pousr])_[A-Za-z0-9_\-]{12,}\b",
            "[redacted-token]",
        ),
        (r"/Users/[^/\s]+", "/Users/[redacted-user]"),
        (r"\\\\Users\\\\[^\\\\\s]+", "\\Users\\[redacted-user]"),
    ] {
        let regex = Regex::new(pattern).expect("valid telemetry scrub regex");
        out = regex.replace_all(&out, replacement).into_owned();
    }
    out
}

pub fn scrub_json(value: JsonValue) -> JsonValue {
    match value {
        JsonValue::String(value) => JsonValue::String(scrub_text(&value)),
        JsonValue::Array(values) => JsonValue::Array(values.into_iter().map(scrub_json).collect()),
        JsonValue::Object(map) => JsonValue::Object(
            map.into_iter()
                .map(|(key, value)| (scrub_text(&key), scrub_json(value)))
                .collect(),
        ),
        other => other,
    }
}

pub fn deterministic_noise(seed: &str, magnitude: i64) -> i64 {
    if magnitude <= 0 {
        return 0;
    }
    let digest = Sha256::digest(seed.as_bytes());
    let span = (magnitude * 2 + 1) as u64;
    (u64::from(digest[0]) % span) as i64 - magnitude
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_hash_is_stable_ulid_shape() {
        let a = hash_user_to_ulid("alice@example.com");
        let b = hash_user_to_ulid("alice@example.com");
        assert_eq!(a, b);
        assert_eq!(a.len(), 26);
        assert_ne!(a, hash_user_to_ulid("bob@example.com"));
    }

    #[test]
    fn scrubber_redacts_common_pii() {
        let text = scrub_text(
            "email a@b.com ip 10.1.2.3 path /Users/alice/proj token ghp_abcdefghijklmnop",
        );
        assert!(text.contains("[redacted-email]"));
        assert!(text.contains("[redacted-ip]"));
        assert!(text.contains("/Users/[redacted-user]/proj"));
        assert!(text.contains("[redacted-token]"));
    }
}
