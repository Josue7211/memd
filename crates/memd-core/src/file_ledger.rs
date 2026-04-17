use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileOp {
    Read,
    Edit,
    Write,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileInteractionEntry {
    pub path: String,
    pub op: FileOp,
    pub count: u32,
    pub last_ts_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInteractionLedger {
    pub session_id: String,
    pub entries: Vec<FileInteractionEntry>,
}

impl FileInteractionLedger {
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            entries: Vec::new(),
        }
    }

    pub fn record(&mut self, path: impl AsRef<str>, op: FileOp, ts_ms: i64) {
        let path = path.as_ref();
        if let Some(e) = self
            .entries
            .iter_mut()
            .find(|e| e.path == path && e.op == op)
        {
            e.count += 1;
            e.last_ts_ms = ts_ms;
            return;
        }
        self.entries.push(FileInteractionEntry {
            path: path.to_string(),
            op,
            count: 1,
            last_ts_ms: ts_ms,
        });
    }

    pub fn find(&self, path: &str, op: FileOp) -> Option<&FileInteractionEntry> {
        self.entries.iter().find(|e| e.path == path && e.op == op)
    }

    pub fn distinct_paths(&self) -> Vec<String> {
        let mut v: Vec<String> = self.entries.iter().map(|e| e.path.clone()).collect();
        v.sort();
        v.dedup();
        v
    }

    pub fn save_to_path(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, bytes)
    }

    pub fn load_from_path(path: &Path) -> io::Result<Self> {
        let bytes = fs::read(path)?;
        serde_json::from_slice(&bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

pub fn session_dir(output: &Path, session_id: &str) -> PathBuf {
    output
        .join("state")
        .join(format!("session-{session_id}"))
}

pub fn ledger_path(output: &Path, session_id: &str) -> PathBuf {
    session_dir(output, session_id).join("file_interactions.json")
}

pub fn seal_session_ledger(session_id: &str, output: &Path) -> io::Result<PathBuf> {
    let src = ledger_path(output, session_id);
    if !src.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "no ledger"));
    }
    let dst_dir = session_dir(output, session_id).join("sealed");
    fs::create_dir_all(&dst_dir)?;
    let ts = chrono::Utc::now().timestamp_millis();
    let dst = dst_dir.join(format!("{ts}.json"));
    fs::copy(&src, &dst)?;
    Ok(dst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_round_trips_through_json() {
        let entry = FileInteractionEntry {
            path: "crates/memd-core/src/lib.rs".into(),
            op: FileOp::Read,
            count: 3,
            last_ts_ms: 1_700_000_000_000,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: FileInteractionEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, entry);
    }

    #[test]
    fn upsert_increments_existing_entry_and_updates_ts() {
        let mut ledger = FileInteractionLedger::new("session-x");
        ledger.record("a.rs", FileOp::Read, 1_000);
        ledger.record("a.rs", FileOp::Read, 2_000);
        ledger.record("a.rs", FileOp::Edit, 3_000);
        let read = ledger.find("a.rs", FileOp::Read).unwrap();
        assert_eq!(read.count, 2);
        assert_eq!(read.last_ts_ms, 2_000);
        let edit = ledger.find("a.rs", FileOp::Edit).unwrap();
        assert_eq!(edit.count, 1);
        assert_eq!(edit.last_ts_ms, 3_000);
    }

    #[test]
    fn distinct_paths_is_deduped_and_sorted() {
        let mut ledger = FileInteractionLedger::new("s");
        ledger.record("b.rs", FileOp::Read, 1);
        ledger.record("a.rs", FileOp::Read, 2);
        ledger.record("a.rs", FileOp::Edit, 3);
        ledger.record("b.rs", FileOp::Write, 4);
        assert_eq!(ledger.distinct_paths(), vec!["a.rs", "b.rs"]);
    }

    #[test]
    fn ledger_round_trips_through_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("file_interactions.json");
        let mut ledger = FileInteractionLedger::new("session-1");
        ledger.record("x.rs", FileOp::Read, 10);
        ledger.save_to_path(&path).unwrap();
        let loaded = FileInteractionLedger::load_from_path(&path).unwrap();
        assert_eq!(loaded.session_id, "session-1");
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].path, "x.rs");
    }

    #[test]
    fn seal_copies_ledger_into_timestamped_sealed_dir() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path();
        let mut ledger = FileInteractionLedger::new("sess-1");
        ledger.record("a.rs", FileOp::Read, 1);
        ledger.save_to_path(&ledger_path(output, "sess-1")).unwrap();
        let sealed = seal_session_ledger("sess-1", output).unwrap();
        assert!(sealed.exists());
        assert!(sealed.starts_with(session_dir(output, "sess-1").join("sealed")));
        let loaded = FileInteractionLedger::load_from_path(&sealed).unwrap();
        assert_eq!(loaded.entries.len(), 1);
    }
}
