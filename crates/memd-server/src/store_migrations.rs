use anyhow::Context;
use memd_schema::{HiveSessionRecord, MemoryItem};
use rusqlite::{Connection, OptionalExtension, params};

use crate::redundancy_key;

pub(crate) fn migrate_redundancy_key(conn: &Connection) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(memory_items)")?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;

    if !columns.iter().any(|column| column == "redundancy_key") {
        conn.execute_batch("ALTER TABLE memory_items ADD COLUMN redundancy_key TEXT;")?;
        let mut stmt = conn.prepare("SELECT id, payload_json FROM memory_items")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (id, payload) = row?;
            let item: MemoryItem = serde_json::from_str(&payload)?;
            let key = redundancy_key(&item);
            conn.execute(
                "UPDATE memory_items SET redundancy_key = ?1 WHERE id = ?2",
                params![key, id],
            )?;
        }
    }

    Ok(())
}

pub(crate) fn migrate_hive_sessions_identity_columns(conn: &mut Connection) -> anyhow::Result<()> {
    let columns = {
        let mut stmt = conn.prepare("PRAGMA table_info(hive_sessions)")?;
        stmt.query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?
    };

    let has_hive_system = columns.iter().any(|value| value == "hive_system");
    let has_hive_role = columns.iter().any(|value| value == "hive_role");
    let has_host = columns.iter().any(|value| value == "host");
    let has_repo_root = columns.iter().any(|value| value == "repo_root");
    let has_worktree_root = columns.iter().any(|value| value == "worktree_root");
    let has_branch = columns.iter().any(|value| value == "branch");

    if !has_hive_system {
        conn.execute_batch("ALTER TABLE hive_sessions ADD COLUMN hive_system TEXT;")?;
    }
    if !has_hive_role {
        conn.execute_batch("ALTER TABLE hive_sessions ADD COLUMN hive_role TEXT;")?;
    }
    if !has_host {
        conn.execute_batch("ALTER TABLE hive_sessions ADD COLUMN host TEXT;")?;
    }
    if !has_repo_root {
        conn.execute_batch("ALTER TABLE hive_sessions ADD COLUMN repo_root TEXT;")?;
    }
    if !has_worktree_root {
        conn.execute_batch("ALTER TABLE hive_sessions ADD COLUMN worktree_root TEXT;")?;
    }
    if !has_branch {
        conn.execute_batch("ALTER TABLE hive_sessions ADD COLUMN branch TEXT;")?;
    }

    let has_hive_session_groups = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'hive_session_groups'",
            [],
            |_| Ok(()),
        )
        .optional()?
        .is_some();
    if !has_hive_session_groups {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS hive_session_groups (
              session_key TEXT NOT NULL,
              hive_group TEXT NOT NULL,
              PRIMARY KEY (session_key, hive_group),
              FOREIGN KEY (session_key) REFERENCES hive_sessions(session_key) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_hive_session_groups_session
              ON hive_session_groups(session_key);
            CREATE INDEX IF NOT EXISTS idx_hive_session_groups_hive_group
              ON hive_session_groups(hive_group, session_key);
            "#,
        )?;
    }

    let hive_session_groups_empty = conn
        .query_row("SELECT 1 FROM hive_session_groups LIMIT 1", [], |_| Ok(()))
        .optional()?
        .is_none();

    if !has_hive_system
        || !has_hive_role
        || !has_host
        || !has_repo_root
        || !has_worktree_root
        || !has_branch
        || hive_session_groups_empty
    {
        let tx = conn
            .transaction()
            .context("begin hive session migration backfill")?;

        {
            let mut statement =
                tx.prepare("SELECT session_key, payload_json FROM hive_sessions")?;
            let mut rows = statement.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;

            let mut update = if !has_hive_system
                || !has_hive_role
                || !has_host
                || !has_repo_root
                || !has_worktree_root
                || !has_branch
            {
                Some(tx.prepare(
                    "UPDATE hive_sessions SET hive_system = ?1, hive_role = ?2, host = ?3, repo_root = ?4, worktree_root = ?5, branch = ?6 WHERE session_key = ?7",
                )?)
            } else {
                None
            };
            let mut insert_group = if hive_session_groups_empty {
                Some(tx.prepare(
                    "INSERT OR IGNORE INTO hive_session_groups (session_key, hive_group) VALUES (?1, ?2)",
                )?)
            } else {
                None
            };

            if hive_session_groups_empty {
                tx.execute("DELETE FROM hive_session_groups", [])?;
            }

            for row in rows.by_ref() {
                let (session_key, payload) = row?;
                let record: HiveSessionRecord =
                    serde_json::from_str(&payload).context("deserialize hive session record")?;
                if let Some(update) = update.as_mut() {
                    update.execute(params![
                        record.hive_system,
                        record.hive_role,
                        record.host,
                        record.repo_root,
                        record.worktree_root,
                        record.branch,
                        session_key
                    ])?;
                }
                if let Some(insert_group) = insert_group.as_mut() {
                    for hive_group in record.hive_groups.iter() {
                        insert_group.execute(params![session_key, hive_group])?;
                    }
                }
            }
        }
        tx.commit()
            .context("commit hive session migration backfill")?;
    }

    Ok(())
}

pub(crate) fn create_hive_session_identity_indexes(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        r#"
        CREATE INDEX IF NOT EXISTS idx_hive_sessions_hive_system
          ON hive_sessions(hive_system);
        CREATE INDEX IF NOT EXISTS idx_hive_sessions_hive_role
          ON hive_sessions(hive_role);
        CREATE INDEX IF NOT EXISTS idx_hive_sessions_host
          ON hive_sessions(host);
        CREATE INDEX IF NOT EXISTS idx_hive_sessions_repo_root
          ON hive_sessions(repo_root);
        CREATE INDEX IF NOT EXISTS idx_hive_sessions_worktree_root_identity
          ON hive_sessions(worktree_root);
        CREATE INDEX IF NOT EXISTS idx_hive_sessions_branch_identity
          ON hive_sessions(branch);
        "#,
    )
    .context("create hive session identity indexes")?;

    Ok(())
}
