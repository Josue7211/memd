//! V6 / B6 — semantic distillation.
//!
//! Episodic turn → semantic candidates (`Fact` / `Decision` /
//! `Preference`). Contract: `docs/contracts/semantic-distillation.md`.
//!
//! B6.1 lands the prompt card constant and the JSON-schema validator
//! used by the codex-lb client (B6.2). The judge call itself reuses the
//! pattern at `public_benchmark.rs:1341` (`call_openai_yes_no_grader_cached`).

use serde::{Deserialize, Serialize};

/// Prompt-card identifier baked into the cache key. Bumping invalidates
/// every cached extraction.
pub(crate) const PROMPT_CARD_VERSION: &str = "semantic-distillation/v1";

/// Frozen system prompt. Mirror of `docs/contracts/semantic-distillation.md` §2.
pub(crate) const PROMPT_CARD_V1: &str = "\
You extract durable facts, decisions, and preferences from a single \
conversation turn. Return ONLY a JSON object {\"candidates\": [...]}.\n\
Each candidate has fields: kind (\"Fact\"|\"Decision\"|\"Preference\"), \
content (one self-contained sentence), confidence (0.0-1.0), \
source_turn_ids (array of strings), rationale (<=120 chars).\n\n\
Rules:\n\
- Skip filler (greetings, acks, \"ok\", \"thanks\", chit-chat). Emit zero candidates.\n\
- Speaker = user → preferences and facts about the user.\n\
- Speaker = assistant → only emit if the assistant states a durable \
decision or fact the user agreed to.\n\
- One candidate = one self-contained claim. Split compound claims.\n\
- confidence < 0.5 means \"skip\" — drop the candidate entirely.\n\
- source_turn_ids MUST contain the provenance.session_id::turn_index pair.\n";

/// Loadable prompt card — currently a single static version. Future
/// revisions add more cards keyed by version string.
#[derive(Debug, Clone)]
pub(crate) struct PromptCard {
    pub version: &'static str,
    pub system_prompt: &'static str,
}

impl PromptCard {
    pub(crate) fn v1() -> Self {
        Self {
            version: PROMPT_CARD_VERSION,
            system_prompt: PROMPT_CARD_V1,
        }
    }
}

/// One candidate emitted by the distiller. Mirrors §3 of the contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct DistillCandidate {
    pub kind: CandidateKind,
    pub content: String,
    pub confidence: f32,
    pub source_turn_ids: Vec<String>,
    pub rationale: String,
}

/// `MemoryKind` subset the distiller emits. Wider taxonomy stays on
/// `memd-schema`; B6 only emits these three.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum CandidateKind {
    Fact,
    Decision,
    Preference,
}

/// Wrapper matching the JSON the model returns: `{"candidates": [...]}`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct DistillOutput {
    pub candidates: Vec<DistillCandidate>,
}

/// Validate raw judge JSON against the B6 schema. Returns the parsed
/// candidates or a human-readable rejection reason.
pub(crate) fn validate_distill_json(raw: &str) -> Result<DistillOutput, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| format!("not valid JSON: {e}"))?;
    let obj = value
        .as_object()
        .ok_or_else(|| "top-level must be object".to_string())?;

    for k in obj.keys() {
        if k != "candidates" {
            return Err(format!("unexpected top-level key `{k}`"));
        }
    }
    let arr = obj
        .get("candidates")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "missing `candidates` array".to_string())?;

    let mut out = Vec::with_capacity(arr.len());
    for (i, c) in arr.iter().enumerate() {
        let c = c
            .as_object()
            .ok_or_else(|| format!("candidate[{i}] not object"))?;

        let kind_str = c
            .get("kind")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("candidate[{i}] missing `kind`"))?;
        let kind = match kind_str {
            "Fact" => CandidateKind::Fact,
            "Decision" => CandidateKind::Decision,
            "Preference" => CandidateKind::Preference,
            other => return Err(format!("candidate[{i}] unknown kind `{other}`")),
        };

        let content = c
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("candidate[{i}] missing `content`"))?
            .to_string();
        if content.trim().is_empty() {
            return Err(format!("candidate[{i}] empty content"));
        }

        let confidence = c
            .get("confidence")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| format!("candidate[{i}] missing/non-numeric `confidence`"))?;
        if !(0.0..=1.0).contains(&confidence) {
            return Err(format!(
                "candidate[{i}] confidence {confidence} outside [0, 1]"
            ));
        }

        let source_turn_ids = c
            .get("source_turn_ids")
            .and_then(|v| v.as_array())
            .ok_or_else(|| format!("candidate[{i}] missing `source_turn_ids`"))?;
        if source_turn_ids.is_empty() {
            return Err(format!("candidate[{i}] empty source_turn_ids"));
        }
        let source_turn_ids = source_turn_ids
            .iter()
            .map(|v| {
                v.as_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| format!("candidate[{i}] source_turn_ids contains non-string"))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let rationale = c
            .get("rationale")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        out.push(DistillCandidate {
            kind,
            content,
            confidence: confidence as f32,
            source_turn_ids,
            rationale,
        });
    }
    Ok(DistillOutput { candidates: out })
}

/// Resolve the effective distill model. `MEMD_V6_DISTILL_MODEL`
/// overrides the CLI default when set non-empty. Mirrors A6.8's env-var
/// read-site pattern so the contract doc and code agree.
pub(crate) fn effective_distill_model(cli_default: &str) -> String {
    match std::env::var("MEMD_V6_DISTILL_MODEL") {
        Ok(s) if !s.trim().is_empty() => s,
        _ => cli_default.to_string(),
    }
}

/// Cache enabled? Default true. `MEMD_V6_DISTILL_CACHE=0` disables. The
/// runtime checks this before calling `cache_get`/`cache_put` (graduates
/// alongside A6.9 when judge calls go live).
pub(crate) fn cache_enabled() -> bool {
    !matches!(std::env::var("MEMD_V6_DISTILL_CACHE").as_deref(), Ok("0"))
}

/// Cache key for one turn × prompt-version. SHA-256 hex of
/// `prompt_version || source_hash`. Stable across runs.
pub(crate) fn cache_key(prompt_version: &str, source_hash: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(prompt_version.as_bytes());
    h.update(b"|");
    h.update(source_hash.as_bytes());
    format!("{:x}", h.finalize())
}

/// Cache record shape. One JSON object per turn under
/// `.memd/benchmarks/public/cache/distill/<key>.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct CacheRecord {
    pub key: String,
    pub model: String,
    pub milli_usd: u64,
    pub candidates: Vec<DistillCandidate>,
    pub ts: String,
}

/// Read a cached extraction by key, if present. Returns `Ok(None)` for
/// a clean miss (file not found); errors for malformed cache entries.
pub(crate) fn cache_get(
    cache_dir: &std::path::Path,
    key: &str,
) -> std::io::Result<Option<CacheRecord>> {
    let path = cache_dir.join(format!("{key}.json"));
    match std::fs::read_to_string(&path) {
        Ok(s) => {
            let rec: CacheRecord = serde_json::from_str(&s)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
            Ok(Some(rec))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

/// Write a cache record, creating the directory if missing.
pub(crate) fn cache_put(cache_dir: &std::path::Path, rec: &CacheRecord) -> std::io::Result<()> {
    std::fs::create_dir_all(cache_dir)?;
    let path = cache_dir.join(format!("{}.json", rec.key));
    let body = serde_json::to_string_pretty(rec)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    std::fs::write(path, body)
}

/// Cache-hit/miss tag for telemetry NDJSON.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CacheOutcome {
    Hit,
    Miss,
}

/// One distill telemetry record. Per-turn NDJSON emitted at
/// `.memd/benchmarks/public/results/distill-<date>.ndjson`. Aggregator
/// reads these alongside the A6 ingest card.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct DistillTelemetry {
    pub ts: String,
    pub bench_id: String,
    pub turn_id: String,
    pub judge_model: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub milli_usd: u64,
    pub candidate_count: u32,
    pub cache: CacheOutcome,
}

/// Format one telemetry record as a single NDJSON line (no trailing newline).
pub(crate) fn format_distill_telemetry_line(t: &DistillTelemetry) -> String {
    serde_json::to_string(t).unwrap_or_else(|_| String::new())
}

/// Path for today's telemetry file under the bundle's
/// `benchmarks/public/results/` directory. `date` = ISO `YYYY-MM-DD`.
pub(crate) fn distill_telemetry_path(
    results_dir: &std::path::Path,
    date: &str,
) -> std::path::PathBuf {
    results_dir.join(format!("distill-{date}.ndjson"))
}

/// Append a telemetry line to the per-day NDJSON file. Creates parent
/// dir on first write.
pub(crate) fn append_distill_telemetry(
    results_dir: &std::path::Path,
    date: &str,
    t: &DistillTelemetry,
) -> std::io::Result<()> {
    use std::io::Write as _;
    std::fs::create_dir_all(results_dir)?;
    let path = distill_telemetry_path(results_dir, date);
    let line = format_distill_telemetry_line(t);
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    f.write_all(line.as_bytes())?;
    f.write_all(b"\n")?;
    Ok(())
}
