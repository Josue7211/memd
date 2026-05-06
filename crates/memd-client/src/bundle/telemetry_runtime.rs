use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::Context;
use chrono::{DateTime, Utc};
use memd_core::telemetry::{
    TelemetryEvent, append_telemetry_event, deterministic_noise, read_telemetry_events, scrub_json,
    telemetry_events_path,
};
use serde::Serialize;
use serde_json::{Value as JsonValue, json};

use crate::{TelemetryCommand, TelemetryExportArgs, TelemetryRecordArgs, TelemetryReportArgs};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TelemetryStatus {
    pub(crate) enabled: bool,
    pub(crate) retention_days: u64,
    pub(crate) export_scope: String,
    pub(crate) events_path: PathBuf,
    pub(crate) event_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TelemetryReport {
    pub(crate) window: String,
    pub(crate) event_count: usize,
    pub(crate) total_tokens: u64,
    pub(crate) total_cost_usd: f64,
    pub(crate) users: BTreeMap<String, TelemetryUserReport>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct TelemetryUserReport {
    pub(crate) token_count: u64,
    pub(crate) estimated_cost_usd: f64,
    pub(crate) harnesses: BTreeMap<String, TelemetryHarnessReport>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct TelemetryHarnessReport {
    pub(crate) event_count: usize,
    pub(crate) token_count: u64,
    pub(crate) estimated_cost_usd: f64,
}

pub(crate) fn run_v14_telemetry_command(
    output: &Path,
    global_json: bool,
    command: &TelemetryCommand,
) -> anyhow::Result<()> {
    match command {
        TelemetryCommand::Enable => {
            set_telemetry_enabled(output, true)?;
            fs::create_dir_all(output.join("telemetry"))?;
            println!("telemetry enabled");
        }
        TelemetryCommand::Disable => {
            set_telemetry_enabled(output, false)?;
            println!("telemetry disabled");
        }
        TelemetryCommand::Status => {
            let status = telemetry_status(output)?;
            if global_json {
                crate::print_json(&status)?;
            } else {
                println!(
                    "telemetry enabled={} retention_days={} export_scope={} events={}",
                    status.enabled, status.retention_days, status.export_scope, status.event_count
                );
            }
        }
        TelemetryCommand::Record(args) => {
            record_telemetry_event(output, args)?;
            println!("telemetry event recorded");
        }
        TelemetryCommand::Report(args) => {
            let report = build_telemetry_usage_report(output, args)?;
            if args.json || global_json {
                crate::print_json(&report)?;
            } else {
                print_telemetry_usage_report(&report);
            }
        }
        TelemetryCommand::Export(args) => {
            let exported = export_telemetry(output, args)?;
            println!("telemetry export wrote {}", exported.display());
        }
    }
    Ok(())
}

pub(crate) fn telemetry_enabled(output: &Path) -> bool {
    read_telemetry_config(output)
        .map(|config| config.enabled)
        .unwrap_or(false)
}

pub(crate) fn telemetry_status(output: &Path) -> anyhow::Result<TelemetryStatus> {
    let config = read_telemetry_config(output)?;
    let event_count = read_telemetry_events(output)?.len();
    Ok(TelemetryStatus {
        enabled: config.enabled,
        retention_days: config.retention_days,
        export_scope: config.export_scope,
        events_path: telemetry_events_path(output),
        event_count,
    })
}

pub(crate) fn record_telemetry_event(
    output: &Path,
    args: &TelemetryRecordArgs,
) -> anyhow::Result<()> {
    if !args.force && !telemetry_enabled(output) {
        anyhow::bail!("telemetry disabled; run `memd telemetry enable` or pass --force");
    }
    let user = args
        .user
        .clone()
        .or_else(|| std::env::var("MEMD_USER").ok())
        .or_else(|| std::env::var("USER").ok())
        .unwrap_or_else(|| "local-user".to_string());
    let harness = args
        .harness
        .clone()
        .or_else(|| std::env::var("MEMD_HIVE_SYSTEM").ok())
        .or_else(|| std::env::var("MEMD_AGENT").ok())
        .unwrap_or_else(|| "unknown".to_string());
    let mut event = TelemetryEvent::new(
        &user,
        &harness,
        &args.event_kind,
        &args.source,
        args.tokens,
        args.cost_usd,
    );
    event.session_id = args.session_id.clone();
    event.model_family = args.model_family.clone();
    if let Some(raw) = args.metadata_json.as_deref() {
        let value: JsonValue = serde_json::from_str(raw).context("parse --metadata-json")?;
        match scrub_json(value) {
            JsonValue::Object(map) => event.metadata = map,
            scrubbed => {
                event.metadata.insert("value".to_string(), scrubbed);
            }
        }
    }
    append_telemetry_event(output, &event).context("append telemetry event")
}

pub(crate) fn append_cost_telemetry_event(
    output: &Path,
    session_id: Option<&str>,
    wake_token_count: usize,
    model_family: &str,
    estimated_cost_usd: f64,
) -> std::io::Result<()> {
    if !telemetry_enabled(output) {
        return Ok(());
    }
    let user = std::env::var("MEMD_USER")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "local-user".to_string());
    let harness = std::env::var("MEMD_HIVE_SYSTEM")
        .or_else(|_| std::env::var("MEMD_AGENT"))
        .unwrap_or_else(|_| "unknown".to_string());
    let mut event = TelemetryEvent::new(
        &user,
        &harness,
        "wake_cost",
        "wake-cost-ledger",
        wake_token_count as u64,
        estimated_cost_usd,
    );
    event.session_id = session_id.map(str::to_string);
    event.model_family = Some(model_family.to_string());
    append_telemetry_event(output, &event)
}

pub(crate) fn build_telemetry_usage_report(
    output: &Path,
    args: &TelemetryReportArgs,
) -> anyhow::Result<TelemetryReport> {
    let cutoff = Utc::now() - parse_window(&args.window)?;
    let mut report = TelemetryReport {
        window: args.window.clone(),
        event_count: 0,
        total_tokens: 0,
        total_cost_usd: 0.0,
        users: BTreeMap::new(),
    };
    for event in read_telemetry_events(output)? {
        if event.ts < cutoff {
            continue;
        }
        report.event_count += 1;
        report.total_tokens += event.token_count;
        report.total_cost_usd += event.estimated_cost_usd;
        let user = report.users.entry(event.user_hash.clone()).or_default();
        user.token_count += event.token_count;
        user.estimated_cost_usd += event.estimated_cost_usd;
        let harness = user.harnesses.entry(event.harness.clone()).or_default();
        harness.event_count += 1;
        harness.token_count += event.token_count;
        harness.estimated_cost_usd += event.estimated_cost_usd;
    }
    Ok(report)
}

pub(crate) fn export_telemetry(
    output: &Path,
    args: &TelemetryExportArgs,
) -> anyhow::Result<PathBuf> {
    let cutoff = Utc::now() - parse_window(&args.window)?;
    let scope = args.scope.trim().to_ascii_lowercase();
    let target = args
        .output_file
        .clone()
        .unwrap_or_else(|| output.join("telemetry").join("export.ndjson"));
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut lines = Vec::new();
    for mut event in read_telemetry_events(output)? {
        if event.ts < cutoff {
            continue;
        }
        event = event.scrubbed();
        if scope == "bench" {
            let seed = format!("{}:{}:{}", event.user_hash, event.harness, event.ts);
            let noise = deterministic_noise(&seed, 3);
            event.token_count = (event.token_count as i64 + noise).max(0) as u64;
            event.estimated_cost_usd =
                (event.estimated_cost_usd + (noise as f64 / 1_000_000.0)).max(0.0);
            event.session_id = None;
        }
        lines.push(serde_json::to_string(&event)?);
    }
    fs::write(
        &target,
        lines.join("\n") + if lines.is_empty() { "" } else { "\n" },
    )
    .with_context(|| format!("write {}", target.display()))?;
    Ok(target)
}

fn print_telemetry_usage_report(report: &TelemetryReport) {
    println!(
        "Telemetry report window={} events={} tokens={} cost_usd={:.6}",
        report.window, report.event_count, report.total_tokens, report.total_cost_usd
    );
    for (user_hash, user) in &report.users {
        println!(
            "user={} tokens={} cost_usd={:.6}",
            user_hash, user.token_count, user.estimated_cost_usd
        );
        for (harness, row) in &user.harnesses {
            println!(
                "  harness={} events={} tokens={} cost_usd={:.6}",
                harness, row.event_count, row.token_count, row.estimated_cost_usd
            );
        }
    }
}

#[derive(Debug, Clone)]
struct TelemetryConfig {
    enabled: bool,
    retention_days: u64,
    export_scope: String,
}

fn read_telemetry_config(output: &Path) -> anyhow::Result<TelemetryConfig> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        return Ok(TelemetryConfig {
            enabled: false,
            retention_days: crate::default_telemetry_retention_days(),
            export_scope: crate::default_telemetry_export_scope(),
        });
    }
    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let doc: JsonValue =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    let telemetry = doc.get("telemetry").and_then(JsonValue::as_object);
    Ok(TelemetryConfig {
        enabled: telemetry
            .and_then(|obj| obj.get("enabled"))
            .and_then(JsonValue::as_bool)
            .unwrap_or(false),
        retention_days: telemetry
            .and_then(|obj| obj.get("retention_days"))
            .and_then(JsonValue::as_u64)
            .unwrap_or_else(crate::default_telemetry_retention_days),
        export_scope: telemetry
            .and_then(|obj| obj.get("export_scope"))
            .and_then(JsonValue::as_str)
            .unwrap_or("local")
            .to_string(),
    })
}

fn set_telemetry_enabled(output: &Path, enabled: bool) -> anyhow::Result<()> {
    let config_path = output.join("config.json");
    if !config_path.exists() {
        anyhow::bail!(
            "{} does not exist; initialize the bundle first",
            config_path.display()
        );
    }
    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let mut doc: JsonValue =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    if !doc.is_object() {
        doc = json!({});
    }
    let root = doc.as_object_mut().expect("object checked");
    let entry = root.entry("telemetry").or_insert_with(|| json!({}));
    if !entry.is_object() {
        *entry = json!({});
    }
    let telemetry = entry.as_object_mut().expect("object checked");
    telemetry.insert("enabled".to_string(), json!(enabled));
    telemetry
        .entry("retention_days".to_string())
        .or_insert_with(|| json!(crate::default_telemetry_retention_days()));
    telemetry
        .entry("export_scope".to_string())
        .or_insert_with(|| json!(crate::default_telemetry_export_scope()));
    let tmp = config_path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_string_pretty(&doc)? + "\n")?;
    fs::rename(&tmp, &config_path)?;
    Ok(())
}

fn parse_window(value: &str) -> anyhow::Result<chrono::Duration> {
    let trimmed = value.trim();
    let split_at = trimmed
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(trimmed.len());
    let (digits, unit) = trimmed.split_at(split_at);
    let amount: i64 = digits
        .parse()
        .with_context(|| format!("parse telemetry window '{value}'"))?;
    match unit {
        "m" | "min" | "mins" => Ok(chrono::Duration::minutes(amount)),
        "h" | "hr" | "hrs" => Ok(chrono::Duration::hours(amount)),
        "" | "d" | "day" | "days" => Ok(chrono::Duration::days(amount)),
        other => anyhow::bail!("unsupported telemetry window unit '{other}'"),
    }
}

#[allow(dead_code)]
fn _std_duration(duration: chrono::Duration) -> Option<Duration> {
    duration.to_std().ok()
}
