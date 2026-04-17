// K2.7: online backup + WAL-safe restore.
//
// Backup uses sqlite's online backup API (rusqlite::backup) which copies
// pages from a live source to a target file while writers keep working.
// We run it on a fresh read-only connection against the main db_path so
// in-flight handlers are unaffected.
//
// Restore is the WAL trap: leaving stale <db>-wal / <db>-shm next to a
// replaced main file causes sqlite to replay old WAL pages on top of
// the restore -> silent corruption. The sequence below is:
//   1. checkpoint(TRUNCATE) on the live db to flush the WAL
//   2. close the checkpoint connection
//   3. delete main + wal + shm
//   4. copy snapshot atomically over db_path
// Callers must guarantee no concurrent write handlers during restore;
// that's enforced by only running restore from the CLI subcommand path
// (process not bound to an axum listener yet).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{Context, anyhow};
use rusqlite::{Connection, backup::Backup};

pub(crate) fn snapshots_dir(db_path: &Path) -> PathBuf {
    let mut p = db_path.to_path_buf();
    let name = p
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "memd.db".to_string());
    p.pop();
    p.push(format!(".{name}.snapshots"));
    p
}

pub(crate) fn write_snapshot(db_path: &Path, out_path: &Path) -> anyhow::Result<u64> {
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create snapshot parent dir {}", parent.display()))?;
    }
    let tmp = out_path.with_extension("tmp");
    // Online backup: source live, dest is fresh file at tmp path.
    let src = Connection::open(db_path)
        .with_context(|| format!("open source db {}", db_path.display()))?;
    let mut dst =
        Connection::open(&tmp).with_context(|| format!("open snapshot tmp {}", tmp.display()))?;
    {
        let backup = Backup::new(&src, &mut dst).context("init sqlite backup")?;
        backup
            .run_to_completion(64, Duration::from_millis(5), None)
            .context("run sqlite backup to completion")?;
    }
    // Drop connections before rename so OS releases the fd on Windows / NFS.
    drop(dst);
    drop(src);

    fs::rename(&tmp, out_path)
        .with_context(|| format!("atomic rename {} -> {}", tmp.display(), out_path.display()))?;

    let meta =
        fs::metadata(out_path).with_context(|| format!("stat snapshot {}", out_path.display()))?;
    Ok(meta.len())
}

pub(crate) fn rotate_snapshots(dir: &Path, keep: usize) -> anyhow::Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries: Vec<(SystemTime, PathBuf)> = fs::read_dir(dir)
        .with_context(|| format!("read snapshot dir {}", dir.display()))?
        .filter_map(Result::ok)
        .filter_map(|e| {
            let path = e.path();
            if !path.is_file() {
                return None;
            }
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if ext != "db" {
                return None;
            }
            let mtime = e.metadata().ok().and_then(|m| m.modified().ok())?;
            Some((mtime, path))
        })
        .collect();
    entries.sort_by(|a, b| b.0.cmp(&a.0));

    let mut pruned = Vec::new();
    for (_, path) in entries.into_iter().skip(keep) {
        fs::remove_file(&path).with_context(|| format!("prune snapshot {}", path.display()))?;
        pruned.push(path);
    }
    Ok(pruned)
}

pub(crate) fn restore_from(snapshot: &Path, db_path: &Path) -> anyhow::Result<()> {
    if !snapshot.exists() {
        return Err(anyhow!("snapshot not found: {}", snapshot.display()));
    }

    // Step 1+2: checkpoint + close live db so no stale WAL remains.
    if db_path.exists() {
        let conn = Connection::open(db_path)
            .with_context(|| format!("open db for checkpoint {}", db_path.display()))?;
        conn.pragma_update(None, "journal_mode", "WAL").ok();
        let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
        drop(conn);
    }

    // Step 3: remove main + wal + shm. Missing files are fine.
    for ext in ["", "-wal", "-shm"] {
        let p = if ext.is_empty() {
            db_path.to_path_buf()
        } else {
            let mut s = db_path.as_os_str().to_os_string();
            s.push(ext);
            PathBuf::from(s)
        };
        if p.exists() {
            fs::remove_file(&p)
                .with_context(|| format!("remove pre-restore file {}", p.display()))?;
        }
    }

    // Step 4: copy snapshot to db_path atomically via tmp + rename.
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create db parent dir {}", parent.display()))?;
    }
    let tmp = db_path.with_extension("restore-tmp");
    fs::copy(snapshot, &tmp)
        .with_context(|| format!("copy snapshot {} -> {}", snapshot.display(), tmp.display()))?;
    fs::rename(&tmp, db_path)
        .with_context(|| format!("rename {} -> {}", tmp.display(), db_path.display()))?;

    // Sanity: open + integrity_check the restored file.
    let conn = Connection::open(db_path)
        .with_context(|| format!("open restored db {}", db_path.display()))?;
    let result: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .context("run integrity_check on restored db")?;
    if result != "ok" {
        return Err(anyhow!("restored db failed integrity_check: {result}"));
    }
    Ok(())
}

pub(crate) fn snapshot_filename_now() -> String {
    let ts = chrono::Utc::now().format("%Y%m%dT%H%M%S%3fZ");
    format!("memd-{ts}.db")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;
    use std::io::{Seek, SeekFrom, Write};

    fn seed(db: &Path) {
        let conn = Connection::open(db).unwrap();
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            CREATE TABLE IF NOT EXISTS t (id INTEGER PRIMARY KEY, v TEXT);
            "#,
        )
        .unwrap();
        for i in 0..64 {
            conn.execute("INSERT INTO t(v) VALUES (?1)", params![format!("row-{i}")])
                .unwrap();
        }
    }

    #[test]
    fn write_then_restore_roundtrip() {
        let tmp = tempdir();
        let db = tmp.join("memd.db");
        seed(&db);

        let snap_dir = snapshots_dir(&db);
        std::fs::create_dir_all(&snap_dir).unwrap();
        let snap = snap_dir.join("snap.db");
        write_snapshot(&db, &snap).unwrap();
        assert!(snap.exists());

        // mutate after snapshot — restore should roll back
        let conn = Connection::open(&db).unwrap();
        conn.execute("INSERT INTO t(v) VALUES ('after')", [])
            .unwrap();
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM t", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 65);
        drop(conn);

        restore_from(&snap, &db).unwrap();

        let conn = Connection::open(&db).unwrap();
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM t", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 64, "restore must drop post-snapshot rows");
    }

    #[test]
    fn corruption_detected_and_restored() {
        let tmp = tempdir();
        let db = tmp.join("memd.db");
        seed(&db);

        let snap_dir = snapshots_dir(&db);
        std::fs::create_dir_all(&snap_dir).unwrap();
        let snap = snap_dir.join("snap.db");
        write_snapshot(&db, &snap).unwrap();

        // Force any WAL back into the main file so page 2 holds real data.
        let conn = Connection::open(&db).unwrap();
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .unwrap();
        drop(conn);

        // Deterministic corruption: zero page 2 (first b-tree page, offset 4096).
        {
            let mut f = std::fs::OpenOptions::new().write(true).open(&db).unwrap();
            f.seek(SeekFrom::Start(4096)).unwrap();
            let zeros = vec![0u8; 4096];
            f.write_all(&zeros).unwrap();
            f.sync_all().unwrap();
        }

        // Integrity check must notice the damage.
        let conn = Connection::open(&db).unwrap();
        let result: String = conn
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        drop(conn);
        assert_ne!(result, "ok", "corruption must be detected");

        restore_from(&snap, &db).unwrap();

        let conn = Connection::open(&db).unwrap();
        let result: String = conn
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        assert_eq!(result, "ok", "post-restore integrity must be ok");
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM t", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 64);
    }

    #[test]
    fn rotate_keeps_last_n() {
        let tmp = tempdir();
        let dir = tmp.join("snaps");
        std::fs::create_dir_all(&dir).unwrap();
        for i in 0..7u32 {
            let p = dir.join(format!("s{i}.db"));
            std::fs::write(&p, b"x").unwrap();
            // stagger mtimes so newest-wins ordering is stable
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
        let pruned = rotate_snapshots(&dir, 3).unwrap();
        assert_eq!(pruned.len(), 4);
        let kept: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        assert_eq!(kept.len(), 3);
    }

    fn tempdir() -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "memd-backup-test-{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}
