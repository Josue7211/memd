//! Wake-cost telemetry — D4.6.
//!
//! Two NDJSON ledgers under `<bundle>/.memd/logs/`:
//!
//! - `wake-budget.ndjson` — per-wake bucket fill ratios + demotions.
//!   Consumed by phase-doc revision §10 kinds-coverage histograms.
//! - `wake-cost.ndjson` — per-wake token utilization + estimated cost
//!   per model family. Required for token_efficiency 2 → 4 axis credit
//!   (without cost numbers, the claim is phantom).

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use chrono::Utc;
use serde::Serialize;

use super::{BucketKind, CompiledWake};

const BUDGET_LOG_RELPATH: &str = "logs/wake-budget.ndjson";
const COST_LOG_RELPATH: &str = "logs/wake-cost.ndjson";

#[derive(Debug, Clone, Serialize)]
struct BudgetLine {
    ts_ms: i64,
    session_id: Option<String>,
    raw_tokens: usize,
    compiled_tokens: usize,
    bucket_sizes: HashMap<String, usize>,
    bucket_fill_ratio: HashMap<String, f64>,
    demoted: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize)]
struct CostLine {
    ts: String,
    session_id: Option<String>,
    wake_token_count: usize,
    budget_target: usize,
    budget_utilization: f64,
    model_family: String,
    estimated_cost_usd: f64,
}

pub fn write_budget_line(
    bundle_root: &Path,
    session_id: Option<&str>,
    raw_tokens: usize,
    compiled: &CompiledWake,
) -> std::io::Result<()> {
    let bucket_sizes: HashMap<String, usize> = compiled
        .bucket_report
        .iter()
        .map(|(k, r)| (label(*k), r.admitted))
        .collect();
    let bucket_fill_ratio: HashMap<String, f64> = compiled
        .bucket_report
        .iter()
        .map(|(k, r)| (label(*k), r.fill_ratio))
        .collect();
    let demoted: HashMap<String, usize> = compiled
        .demotion_hints
        .iter()
        .map(|h| (label(h.bucket), h.count))
        .collect();
    let line = BudgetLine {
        ts_ms: Utc::now().timestamp_millis(),
        session_id: session_id.map(str::to_string),
        raw_tokens,
        compiled_tokens: compiled.tokens,
        bucket_sizes,
        bucket_fill_ratio,
        demoted,
    };
    append_ndjson(bundle_root, BUDGET_LOG_RELPATH, &line)
}

pub fn write_cost_line(
    bundle_root: &Path,
    session_id: Option<&str>,
    wake_token_count: usize,
    budget_target: usize,
    model_family: &str,
) -> std::io::Result<()> {
    let utilization = if budget_target == 0 {
        0.0
    } else {
        wake_token_count as f64 / budget_target as f64
    };
    let line = CostLine {
        ts: Utc::now().to_rfc3339(),
        session_id: session_id.map(str::to_string),
        wake_token_count,
        budget_target,
        budget_utilization: utilization,
        model_family: model_family.to_string(),
        estimated_cost_usd: estimate_cost(wake_token_count, model_family),
    };
    append_ndjson(bundle_root, COST_LOG_RELPATH, &line)
}

/// Conservative input-token cost per 1k tokens. Numbers are deliberately
/// approximate — exact pricing belongs in a config file, but this is
/// good enough for the TE axis ledger contract (D4.4 revision §10).
fn estimate_cost(chars: usize, model_family: &str) -> f64 {
    let usd_per_1k_input_tokens = match model_family {
        f if f.contains("opus") => 0.015,
        f if f.contains("sonnet") => 0.003,
        f if f.contains("haiku") => 0.00025,
        f if f.contains("gpt-5") => 0.005,
        _ => 0.001,
    };
    let approx_tokens = chars as f64 / 4.0;
    (approx_tokens / 1000.0) * usd_per_1k_input_tokens
}

fn label(kind: BucketKind) -> String {
    kind.label().to_string()
}

fn append_ndjson<T: Serialize>(
    bundle_root: &Path,
    relpath: &str,
    payload: &T,
) -> std::io::Result<()> {
    let target = bundle_root.join(relpath);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&target)?;
    let serialized = serde_json::to_string(payload)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
    writeln!(file, "{serialized}")?;
    Ok(())
}
