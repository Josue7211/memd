//! NDJSON append-only hook trace writer.
//!
//! Writes one JSON object per hook fire to `logs/hook-trace.ndjson` under
//! the bundle root. Uses `OpenOptions::create(true).append(true)` so
//! crash-safe on POSIX up to the pipe-buffer write size. An internal
//! `Mutex` serialises concurrent writers within the same process.

use super::{FailureClass, HookEvent};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use ulid::Ulid;

/// Trace file size cap — contract §7.
pub const TRACE_SIZE_CAP_BYTES: u64 = 100 * 1024 * 1024;

/// NDJSON trace line shape. Matches contract §3.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookRecord {
    pub ts_ms: u64,
    pub event: HookEvent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    pub failure_class: FailureClass,
    pub trace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sealed_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restored_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ok: Option<bool>,
}

impl HookRecord {
    /// Build a minimal record with the current time and a fresh ULID.
    pub fn new(event: HookEvent, session_id: impl Into<String>) -> Self {
        let ts_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        Self {
            ts_ms,
            event,
            harness: None,
            session_id: session_id.into(),
            budget_ms: None,
            elapsed_ms: None,
            exit_code: None,
            failure_class: FailureClass::None,
            trace_id: Ulid::new().to_string(),
            tool: None,
            path: None,
            sealed_path: None,
            restored_path: None,
            entries: None,
            ok: None,
        }
    }

    pub fn with_harness(mut self, harness: impl Into<String>) -> Self {
        self.harness = Some(harness.into());
        self
    }

    pub fn with_budget_ms(mut self, budget_ms: u64) -> Self {
        self.budget_ms = Some(budget_ms);
        self
    }

    pub fn with_outcome(mut self, elapsed_ms: u64, exit_code: i32, class: FailureClass) -> Self {
        self.elapsed_ms = Some(elapsed_ms);
        self.exit_code = Some(exit_code);
        self.failure_class = class;
        self
    }

    pub fn with_tool(mut self, tool: impl Into<String>) -> Self {
        self.tool = Some(tool.into());
        self
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
}

/// Append-only trace writer. Clone-safe: all clones share the same
/// `Mutex<File>` via the inner `PathBuf` + on-demand `OpenOptions`.
#[derive(Debug)]
pub struct HookTrace {
    path: PathBuf,
    gate: Mutex<()>,
}

impl HookTrace {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            gate: Mutex::new(()),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Append a record as a single NDJSON line.
    pub fn append(&self, record: &HookRecord) -> io::Result<()> {
        let _g = self
            .gate
            .lock()
            .map_err(|_| io::Error::other("hook trace mutex poisoned"))?;
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = self.open_for_append()?;
        // Check cap; if exceeded, emit a single truncation-required line
        // and stop — contract §7.
        if let Ok(meta) = file.metadata()
            && meta.len() >= TRACE_SIZE_CAP_BYTES
        {
            let marker = serde_json::json!({
                "ts_ms": HookRecord::new(HookEvent::TruncationRequired, &record.session_id).ts_ms,
                "event": "truncation-required",
                "session_id": record.session_id,
                "failure_class": "none",
                "trace_id": Ulid::new().to_string(),
            });
            writeln!(file, "{}", marker)?;
            return Ok(());
        }
        let line = serde_json::to_string(record)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        writeln!(file, "{}", line)?;
        Ok(())
    }

    fn open_for_append(&self) -> io::Result<File> {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};
    use std::sync::Arc;
    use std::thread;
    use tempfile::TempDir;

    #[test]
    fn trace_append_is_line_delimited_and_parseable() {
        let dir = TempDir::new().unwrap();
        let trace = HookTrace::new(dir.path().join("hook-trace.ndjson"));
        let rec = HookRecord::new(HookEvent::PreCompact, "sess-1").with_budget_ms(5_000);
        trace.append(&rec).unwrap();
        trace
            .append(&HookRecord::new(HookEvent::PostCompact, "sess-1"))
            .unwrap();

        let contents = std::fs::read_to_string(trace.path()).unwrap();
        let lines: Vec<_> = contents.lines().collect();
        assert_eq!(lines.len(), 2);
        for line in lines {
            let parsed: HookRecord = serde_json::from_str(line).expect("parseable");
            assert!(!parsed.trace_id.is_empty());
        }
    }

    #[test]
    fn trace_survives_concurrent_writers() {
        let dir = TempDir::new().unwrap();
        let trace = Arc::new(HookTrace::new(dir.path().join("hook-trace.ndjson")));

        let mut handles = Vec::new();
        for thread_id in 0..4 {
            let trace = Arc::clone(&trace);
            handles.push(thread::spawn(move || {
                for i in 0..25 {
                    let rec = HookRecord::new(HookEvent::PreRead, format!("sess-{thread_id}-{i}"))
                        .with_tool("Read");
                    trace.append(&rec).unwrap();
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }

        let file = File::open(trace.path()).unwrap();
        let mut count = 0;
        for line in BufReader::new(file).lines() {
            let line = line.unwrap();
            assert!(!line.is_empty());
            let parsed: HookRecord = serde_json::from_str(&line).expect("every line parseable");
            assert_eq!(parsed.event, HookEvent::PreRead);
            count += 1;
        }
        assert_eq!(count, 100);
    }

    #[test]
    fn trace_writes_trace_id_ulid() {
        let dir = TempDir::new().unwrap();
        let trace = HookTrace::new(dir.path().join("hook-trace.ndjson"));
        let rec = HookRecord::new(HookEvent::Stop, "sess-9");
        trace.append(&rec).unwrap();

        let contents = std::fs::read_to_string(trace.path()).unwrap();
        let parsed: HookRecord = serde_json::from_str(contents.trim()).unwrap();
        // ULID canonical length = 26 chars.
        assert_eq!(parsed.trace_id.len(), 26);
        assert!(parsed.trace_id.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn truncation_line_emitted_when_cap_exceeded() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("hook-trace.ndjson");
        // Seed a file larger than the cap with newline-terminated filler
        // so existing lines remain grep-parseable.
        {
            let mut f = File::create(&path).unwrap();
            let blob = {
                let mut s = String::with_capacity(4096);
                while s.len() < 4095 {
                    s.push_str("{\"filler\":\"x\"}\n");
                }
                s
            };
            let mut written: u64 = 0;
            while written < TRACE_SIZE_CAP_BYTES {
                f.write_all(blob.as_bytes()).unwrap();
                written += blob.len() as u64;
            }
        }
        let trace = HookTrace::new(&path);
        let rec = HookRecord::new(HookEvent::Stop, "sess-cap");
        trace.append(&rec).unwrap();

        let contents = std::fs::read_to_string(&path).unwrap();
        let last_line = contents.lines().last().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(last_line).unwrap();
        assert_eq!(parsed["event"], "truncation-required");
    }
}
