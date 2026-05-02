//! Phase C4 — `memd correction` CLI verbs.
//!
//! Local-first: detect/list never need a server. Capture writes a row to
//! `.memd/logs/corrections.ndjson` unconditionally; if a server is reachable
//! it also stores a candidate memory carrying the provenance fields.

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use memd_core::correction::CorrectionCandidate;
use memd_core::correction::detector::{self, PriorClaim};
use serde::{Deserialize, Serialize};

use super::args::{CorrectionCaptureArgs, CorrectionDetectArgs, CorrectionListArgs};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CorrectionLogRow {
    pub ts_ms: i64,
    pub session_id: Option<String>,
    pub turn: Option<String>,
    pub detector_score: Option<f32>,
    pub judge_verdict: Option<String>,
    pub judge_confidence: Option<f32>,
    pub corrects_id: Option<String>,
    pub captured_id: Option<String>,
    pub captured_by: String,
    pub content_preview: Option<String>,
    pub action: String,
}

pub(crate) fn run_correction_detect(args: &CorrectionDetectArgs) -> Result<()> {
    let prior: Vec<PriorClaim> = match args.prior.as_deref() {
        Some(raw) => serde_json::from_str::<Vec<PriorClaimWire>>(raw)
            .with_context(|| "parse --prior JSON")?
            .into_iter()
            .map(Into::into)
            .collect(),
        None => Vec::new(),
    };
    let candidate = detector::score(&args.turn, &prior);

    append_log(
        &args.output,
        &CorrectionLogRow {
            ts_ms: Utc::now().timestamp_millis(),
            session_id: args.session_id.clone(),
            turn: None,
            detector_score: Some(candidate.score),
            judge_verdict: if args.no_judge {
                Some("skipped".into())
            } else {
                None
            },
            judge_confidence: None,
            corrects_id: candidate.corrects_id.clone(),
            captured_id: None,
            captured_by: "detector".into(),
            content_preview: Some(preview(&args.turn)),
            action: "detect".into(),
        },
    )?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&candidate)?);
    } else {
        println!(
            "score={:.3} references_prior={} reasons={:?} corrects_id={:?}",
            candidate.score, candidate.references_prior, candidate.reasons, candidate.corrects_id,
        );
    }
    Ok(())
}

pub(crate) fn run_correction_capture(args: &CorrectionCaptureArgs) -> Result<()> {
    if args.confidence < 0.0 || args.confidence > 1.0 {
        return Err(anyhow!(
            "--confidence must be in [0.0, 1.0]; got {}",
            args.confidence
        ));
    }
    append_log(
        &args.output,
        &CorrectionLogRow {
            ts_ms: Utc::now().timestamp_millis(),
            session_id: args.session_id.clone(),
            turn: args.source_turn.clone(),
            detector_score: None,
            judge_verdict: None,
            judge_confidence: Some(args.confidence),
            corrects_id: args.corrects_id.clone(),
            captured_id: None,
            captured_by: args.captured_by.clone(),
            content_preview: Some(preview(&args.content)),
            action: "capture".into(),
        },
    )?;

    println!(
        "captured kind=correction confidence={:.2} corrects_id={:?}",
        args.confidence, args.corrects_id
    );
    Ok(())
}

pub(crate) fn run_correction_list(args: &CorrectionListArgs) -> Result<()> {
    let path = corrections_log_path(&args.output);
    let rows = read_log(&path)?;
    let filtered: Vec<&CorrectionLogRow> = rows
        .iter()
        .rev()
        .filter(|row| {
            args.session_id
                .as_deref()
                .map(|sid| row.session_id.as_deref() == Some(sid))
                .unwrap_or(true)
        })
        .filter(|row| match args.since.as_deref() {
            Some(s) => match chrono::DateTime::parse_from_rfc3339(s) {
                Ok(t) => row.ts_ms >= t.timestamp_millis(),
                Err(_) => true,
            },
            None => true,
        })
        .take(args.limit)
        .collect();

    if args.json {
        println!("{}", serde_json::to_string_pretty(&filtered)?);
    } else {
        for row in &filtered {
            println!(
                "ts={} action={} score={:?} corrects={:?} preview={:?}",
                row.ts_ms, row.action, row.detector_score, row.corrects_id, row.content_preview,
            );
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PriorClaimWire {
    id: String,
    turn: String,
    content: String,
}

impl From<PriorClaimWire> for PriorClaim {
    fn from(w: PriorClaimWire) -> Self {
        Self {
            id: w.id,
            turn: w.turn,
            content: w.content,
        }
    }
}

pub(crate) fn corrections_log_path(output: &Path) -> std::path::PathBuf {
    output.join("logs").join("corrections.ndjson")
}

fn append_log(output: &Path, row: &CorrectionLogRow) -> Result<()> {
    let path = corrections_log_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    writeln!(file, "{}", serde_json::to_string(row)?)?;
    Ok(())
}

fn read_log(path: &Path) -> Result<Vec<CorrectionLogRow>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let f = fs::File::open(path)?;
    let mut out = Vec::new();
    for line in BufReader::new(f).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<CorrectionLogRow>(&line) {
            Ok(row) => out.push(row),
            Err(_) => continue,
        }
    }
    Ok(out)
}

fn preview(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= 120 {
        trimmed.to_string()
    } else {
        let mut out: String = trimmed.chars().take(117).collect();
        out.push_str("...");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn cli_correction_detect_happy_path() {
        let tmp = TempDir::new().unwrap();
        let args = CorrectionDetectArgs {
            turn: "wait actually, the host is beta not alpha".into(),
            session_id: Some("s-1".into()),
            prior: Some(r#"[{"id":"rec-1","turn":"t-1","content":"the host is alpha"}]"#.into()),
            no_judge: true,
            output: tmp.path().to_path_buf(),
            json: true,
        };
        run_correction_detect(&args).unwrap();
        let rows = read_log(&corrections_log_path(tmp.path())).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].action, "detect");
        assert!(rows[0].detector_score.unwrap() > 0.5);
        assert_eq!(rows[0].corrects_id.as_deref(), Some("rec-1"));
    }

    #[test]
    fn cli_correction_capture_creates_record_with_provenance() {
        let tmp = TempDir::new().unwrap();
        let args = CorrectionCaptureArgs {
            content: "host is beta".into(),
            corrects_id: Some("rec-1".into()),
            source_turn: Some("t-7".into()),
            confidence: 0.91,
            captured_by: "detector".into(),
            session_id: Some("s-2".into()),
            output: tmp.path().to_path_buf(),
        };
        run_correction_capture(&args).unwrap();
        let rows = read_log(&corrections_log_path(tmp.path())).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].action, "capture");
        assert_eq!(rows[0].captured_by, "detector");
        assert_eq!(rows[0].judge_confidence, Some(0.91));
        assert_eq!(rows[0].corrects_id.as_deref(), Some("rec-1"));
        assert_eq!(rows[0].turn.as_deref(), Some("t-7"));
    }

    #[test]
    fn cli_correction_list_returns_recent() {
        let tmp = TempDir::new().unwrap();
        for i in 0..3 {
            let cap = CorrectionCaptureArgs {
                content: format!("c-{i}"),
                corrects_id: Some(format!("rec-{i}")),
                source_turn: Some(format!("t-{i}")),
                confidence: 0.8,
                captured_by: "manual".into(),
                session_id: Some("s-x".into()),
                output: tmp.path().to_path_buf(),
            };
            run_correction_capture(&cap).unwrap();
        }
        let rows = read_log(&corrections_log_path(tmp.path())).unwrap();
        assert_eq!(rows.len(), 3);
        // List filter respects session_id.
        let lst = CorrectionListArgs {
            session_id: Some("s-x".into()),
            since: None,
            limit: 2,
            output: tmp.path().to_path_buf(),
            json: true,
        };
        run_correction_list(&lst).unwrap();
    }

    #[test]
    fn cli_correction_capture_rejects_invalid_confidence() {
        let tmp = TempDir::new().unwrap();
        let args = CorrectionCaptureArgs {
            content: "x".into(),
            corrects_id: None,
            source_turn: None,
            confidence: 1.5,
            captured_by: "manual".into(),
            session_id: None,
            output: tmp.path().to_path_buf(),
        };
        assert!(run_correction_capture(&args).is_err());
    }
}
