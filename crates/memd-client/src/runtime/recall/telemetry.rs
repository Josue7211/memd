use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use chrono::Utc;
use serde::Serialize;

use super::{RecallDepth, expansion::ExpansionPlan};

const RELPATH: &str = "logs/recall-depth.ndjson";

/// One line per recall call, regardless of depth. Schema is frozen for
/// V4 per `docs/contracts/recall-depth.md`. New fields must be additive.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct DepthLine<'a> {
    pub(crate) ts_ms: i64,
    pub(crate) session_id: Option<&'a str>,
    pub(crate) query: &'a str,
    pub(crate) depth: &'a str,
    pub(crate) records_returned: usize,
    pub(crate) tokens_returned: usize,
    pub(crate) latency_ms: u64,
    pub(crate) escalation_hint: Option<&'a str>,
    pub(crate) selective_expansion: Option<SelectiveExpansionLine<'a>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SelectiveExpansionLine<'a> {
    pub(crate) stages: Vec<&'static str>,
    pub(crate) ceo_mode: bool,
    pub(crate) forensics: bool,
    pub(crate) rationale: &'a str,
}

pub(crate) struct RecordOpts<'a> {
    pub(crate) bundle_root: &'a Path,
    pub(crate) session_id: Option<&'a str>,
    pub(crate) query: &'a str,
    pub(crate) depth: RecallDepth,
    pub(crate) records_returned: usize,
    pub(crate) tokens_returned: usize,
    pub(crate) latency_ms: u64,
    pub(crate) escalation_hint: Option<&'a str>,
    pub(crate) expansion_plan: Option<&'a ExpansionPlan>,
}

pub(crate) fn record(opts: RecordOpts<'_>) -> std::io::Result<()> {
    let line = DepthLine {
        ts_ms: Utc::now().timestamp_millis(),
        session_id: opts.session_id,
        query: opts.query,
        depth: opts.depth.as_str(),
        records_returned: opts.records_returned,
        tokens_returned: opts.tokens_returned,
        latency_ms: opts.latency_ms,
        escalation_hint: opts.escalation_hint,
        selective_expansion: opts.expansion_plan.map(|plan| SelectiveExpansionLine {
            stages: plan.stage_names(),
            ceo_mode: plan.ceo_mode,
            forensics: plan.forensics,
            rationale: plan.rationale,
        }),
    };
    append_ndjson(opts.bundle_root, &line)
}

pub(crate) fn log_path(bundle_root: &Path) -> std::path::PathBuf {
    bundle_root.join(RELPATH)
}

/// Approximate token count from raw output characters, matching the D4
/// ledger estimator (chars / 4) so depth-tokens stay comparable.
pub(crate) fn approx_tokens(chars: usize) -> usize {
    chars / 4
}

fn append_ndjson<T: Serialize>(bundle_root: &Path, payload: &T) -> std::io::Result<()> {
    let target = bundle_root.join(RELPATH);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(&target)?;
    let serialized = serde_json::to_string(payload)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
    writeln!(file, "{serialized}")?;
    Ok(())
}
