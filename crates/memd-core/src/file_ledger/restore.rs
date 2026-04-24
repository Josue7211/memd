use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

use super::{ledger_path, session_dir, FileInteractionLedger};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RestoreSource {
    Postcompact,
    Manual,
    Test,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerRestoreReport {
    pub session_id: String,
    pub sealed_path: Option<PathBuf>,
    pub restored_path: PathBuf,
    pub entries: usize,
    pub source: RestoreSource,
    pub ok: bool,
    pub error: Option<String>,
}

pub fn sealed_dir(output: &Path, session_id: &str) -> PathBuf {
    session_dir(output, session_id).join("sealed")
}

/// Return the newest sealed ledger path by parsing the numeric timestamp stem
/// of every `*.json` file in the sealed dir. Non-numeric stems and non-json
/// extensions are ignored. Returns `None` if the sealed dir is absent or empty.
pub fn locate_latest_sealed(output: &Path, session_id: &str) -> Option<PathBuf> {
    let dir = sealed_dir(output, session_id);
    let rd = fs::read_dir(&dir).ok()?;
    let mut best: Option<(u64, PathBuf)> = None;
    for entry in rd.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Ok(ts) = stem.parse::<u64>() else {
            continue;
        };
        if best.as_ref().map_or(true, |(b, _)| ts > *b) {
            best = Some((ts, path));
        }
    }
    best.map(|(_, p)| p)
}

/// Copy the newest sealed ledger for `session_id` back to the active ledger
/// path. When no sealed ledger exists, returns an `ok:false` report with
/// `error = Some("no-sealed-ledger")` — caller decides whether to treat as
/// breach (hook path) or soft no-op (manual dry-run).
pub fn restore_ledger(
    session_id: &str,
    output: &Path,
    source: RestoreSource,
) -> io::Result<LedgerRestoreReport> {
    let restored_path = ledger_path(output, session_id);
    let sealed = locate_latest_sealed(output, session_id);
    let Some(sealed_path) = sealed else {
        return Ok(LedgerRestoreReport {
            session_id: session_id.to_string(),
            sealed_path: None,
            restored_path,
            entries: 0,
            source,
            ok: false,
            error: Some("no-sealed-ledger".to_string()),
        });
    };
    if let Some(parent) = restored_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&sealed_path, &restored_path)?;
    let ledger = FileInteractionLedger::load_from_path(&restored_path)?;
    Ok(LedgerRestoreReport {
        session_id: session_id.to_string(),
        sealed_path: Some(sealed_path),
        restored_path,
        entries: ledger.entries.len(),
        source,
        ok: true,
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_ledger::{seal_session_ledger, FileOp};

    fn seed_ledger(output: &Path, session_id: &str, paths: &[&str]) {
        let mut ledger = FileInteractionLedger::new(session_id);
        for (i, p) in paths.iter().enumerate() {
            ledger.record(*p, FileOp::Read, 1_000 + i as i64);
        }
        ledger.save_to_path(&ledger_path(output, session_id)).unwrap();
    }

    fn write_sealed(output: &Path, session_id: &str, ts_ms: u64, paths: &[&str]) -> PathBuf {
        let dir = sealed_dir(output, session_id);
        fs::create_dir_all(&dir).unwrap();
        let mut ledger = FileInteractionLedger::new(session_id);
        for (i, p) in paths.iter().enumerate() {
            ledger.record(*p, FileOp::Read, 10 + i as i64);
        }
        let dst = dir.join(format!("{ts_ms}.json"));
        ledger.save_to_path(&dst).unwrap();
        dst
    }

    #[test]
    fn locate_latest_sealed_returns_newest_by_timestamp() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path();
        let sid = "sess-latest";
        write_sealed(output, sid, 100, &["a.rs"]);
        let newest = write_sealed(output, sid, 300, &["c.rs"]);
        write_sealed(output, sid, 200, &["b.rs"]);
        let found = locate_latest_sealed(output, sid).unwrap();
        assert_eq!(found, newest);
    }

    #[test]
    fn locate_latest_sealed_returns_none_when_sealed_dir_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(locate_latest_sealed(dir.path(), "sess-empty").is_none());
    }

    #[test]
    fn locate_latest_sealed_ignores_non_json_files() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path();
        let sid = "sess-noise";
        let good = write_sealed(output, sid, 500, &["a.rs"]);
        // Noise: different extension, junk stem, nested dir.
        let noise = sealed_dir(output, sid);
        fs::write(noise.join("latest.tmp"), b"noise").unwrap();
        fs::write(noise.join("README.txt"), b"noise").unwrap();
        fs::write(noise.join("not-a-number.json"), b"{}").unwrap();
        let found = locate_latest_sealed(output, sid).unwrap();
        assert_eq!(found, good);
    }

    #[test]
    fn restore_ledger_copies_sealed_to_active_path() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path();
        let sid = "sess-copy";
        let sealed = write_sealed(output, sid, 100, &["a.rs", "b.rs", "c.rs"]);
        let report = restore_ledger(sid, output, RestoreSource::Manual).unwrap();
        assert!(report.ok);
        assert_eq!(report.sealed_path.as_deref(), Some(sealed.as_path()));
        assert_eq!(report.entries, 3);
        let active = ledger_path(output, sid);
        assert!(active.exists());
        let loaded = FileInteractionLedger::load_from_path(&active).unwrap();
        let paths = loaded.distinct_paths();
        assert_eq!(paths, vec!["a.rs", "b.rs", "c.rs"]);
    }

    #[test]
    fn restore_ledger_overwrites_existing_active_ledger() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path();
        let sid = "sess-overwrite";
        // Active ledger has stale content.
        seed_ledger(output, sid, &["stale.rs"]);
        // Sealed has fresh content.
        write_sealed(output, sid, 999, &["fresh1.rs", "fresh2.rs"]);
        let report = restore_ledger(sid, output, RestoreSource::Manual).unwrap();
        assert!(report.ok);
        let loaded = FileInteractionLedger::load_from_path(&ledger_path(output, sid)).unwrap();
        assert_eq!(loaded.distinct_paths(), vec!["fresh1.rs", "fresh2.rs"]);
    }

    #[test]
    fn restore_ledger_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path();
        let sid = "sess-idem";
        write_sealed(output, sid, 42, &["a.rs", "b.rs"]);
        let first = restore_ledger(sid, output, RestoreSource::Manual).unwrap();
        let second = restore_ledger(sid, output, RestoreSource::Manual).unwrap();
        assert!(first.ok && second.ok);
        assert_eq!(first.entries, second.entries);
        let loaded = FileInteractionLedger::load_from_path(&ledger_path(output, sid)).unwrap();
        // Idempotent: no duplicate entries after second restore.
        assert_eq!(loaded.entries.len(), 2);
    }

    #[test]
    fn restore_ledger_records_source_postcompact() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path();
        let sid = "sess-src";
        write_sealed(output, sid, 1, &["a.rs"]);
        let report = restore_ledger(sid, output, RestoreSource::Postcompact).unwrap();
        assert_eq!(report.source, RestoreSource::Postcompact);
    }

    #[test]
    fn restore_ledger_reports_no_sealed_when_dir_missing() {
        let dir = tempfile::tempdir().unwrap();
        let report = restore_ledger("nothing", dir.path(), RestoreSource::Manual).unwrap();
        assert!(!report.ok);
        assert_eq!(report.error.as_deref(), Some("no-sealed-ledger"));
        assert_eq!(report.entries, 0);
        assert!(report.sealed_path.is_none());
    }

    #[test]
    fn restore_round_trips_from_seal_session_ledger_helper() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path();
        let sid = "sess-helper";
        let mut ledger = FileInteractionLedger::new(sid);
        ledger.record("x.rs", FileOp::Read, 1);
        ledger.record("y.rs", FileOp::Edit, 2);
        ledger.save_to_path(&ledger_path(output, sid)).unwrap();
        let sealed = seal_session_ledger(sid, output).unwrap();
        // Wipe active ledger to simulate compaction reset.
        fs::remove_file(ledger_path(output, sid)).unwrap();
        let report = restore_ledger(sid, output, RestoreSource::Postcompact).unwrap();
        assert!(report.ok);
        assert_eq!(report.sealed_path.as_deref(), Some(sealed.as_path()));
        let loaded = FileInteractionLedger::load_from_path(&ledger_path(output, sid)).unwrap();
        assert_eq!(loaded.entries.len(), 2);
        assert_eq!(loaded.find("x.rs", FileOp::Read).unwrap().count, 1);
    }
}
