use anyhow::Context;
use memd_schema::{HiveSessionRecord, MemoryEntityRecord, MemoryItem};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use crate::redundancy_key;

/// B3-part2: memory_vectors was introduced with PK(memory_id); per-turn
/// chunking needs a composite PK(memory_id, chunk_idx). Rather than an
/// ALTER-heavy rewrite, detect the missing column and rebuild the table
/// empty (vectors are a derived cache — a full re-embed repopulates).
pub(crate) fn migrate_memory_vectors_chunk_idx(conn: &Connection) -> anyhow::Result<()> {
    let has_table = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'memory_vectors'",
            [],
            |_| Ok(()),
        )
        .optional()?
        .is_some();
    if !has_table {
        return Ok(());
    }
    let mut stmt = conn.prepare("PRAGMA table_info(memory_vectors)")?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;
    if columns.iter().any(|c| c == "chunk_idx") {
        return Ok(());
    }
    conn.execute_batch(
        r#"
        DROP TABLE memory_vectors;
        CREATE TABLE memory_vectors (
          memory_id TEXT NOT NULL,
          chunk_idx INTEGER NOT NULL,
          project TEXT,
          namespace TEXT,
          embedding_model TEXT NOT NULL DEFAULT 'all-minilm-l6-v2',
          dim INTEGER NOT NULL,
          vec BLOB NOT NULL,
          updated_at TEXT NOT NULL,
          PRIMARY KEY (memory_id, chunk_idx)
        );
        CREATE INDEX IF NOT EXISTS idx_memory_vectors_scope
          ON memory_vectors(project, namespace);
        "#,
    )?;
    Ok(())
}

pub(crate) fn migrate_memory_vectors_embedding_model(conn: &Connection) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(memory_vectors)")?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;
    if columns.iter().any(|column| column == "embedding_model") {
        return Ok(());
    }
    conn.execute_batch(
        "ALTER TABLE memory_vectors ADD COLUMN embedding_model TEXT NOT NULL DEFAULT 'all-minilm-l6-v2';",
    )?;
    Ok(())
}

pub(crate) fn migrate_memory_items_embedding_model(conn: &Connection) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(memory_items)")?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;
    if !columns.iter().any(|column| column == "embedding_model") {
        conn.execute_batch("ALTER TABLE memory_items ADD COLUMN embedding_model TEXT;")?;
    }
    conn.execute(
        r#"
        UPDATE memory_items
        SET embedding_model = 'all-minilm-l6-v2'
        WHERE embedding_model IS NULL
          AND id IN (SELECT DISTINCT memory_id FROM memory_vectors)
        "#,
        [],
    )?;
    Ok(())
}

pub(crate) fn migrate_memory_items_user_identity(conn: &Connection) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(memory_items)")?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;

    if !columns.iter().any(|column| column == "user_id") {
        conn.execute_batch("ALTER TABLE memory_items ADD COLUMN user_id TEXT;")?;
    }
    if !columns.iter().any(|column| column == "harness_preset") {
        conn.execute_batch("ALTER TABLE memory_items ADD COLUMN harness_preset TEXT;")?;
    }
    if !columns.iter().any(|column| column == "user_id_session_seq") {
        conn.execute_batch(
            "ALTER TABLE memory_items ADD COLUMN user_id_session_seq INTEGER NOT NULL DEFAULT 0;",
        )?;
    }

    conn.execute_batch(
        r#"
        UPDATE memory_items
        SET user_id = source_agent
        WHERE user_id IS NULL
          AND source_agent IS NOT NULL;

        CREATE INDEX IF NOT EXISTS idx_memory_user_session
          ON memory_items(user_id, source_agent, user_id_session_seq);
        CREATE INDEX IF NOT EXISTS idx_memory_harness_preset
          ON memory_items(harness_preset);
        "#,
    )?;

    let mut stmt = conn.prepare(
        r#"
        SELECT id, payload_json
        FROM memory_items
        WHERE harness_preset IS NULL
        "#,
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    for (id, payload) in rows {
        let harness = serde_json::from_str::<MemoryItem>(&payload)
            .ok()
            .and_then(|item| item.source_system)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "legacy".to_string());
        conn.execute(
            "UPDATE memory_items SET harness_preset = ?1 WHERE id = ?2",
            params![harness, id],
        )?;
    }
    Ok(())
}

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
            .transaction_with_behavior(TransactionBehavior::Immediate)
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

pub(crate) fn migrate_hive_sessions_last_wake_at(conn: &Connection) -> anyhow::Result<()> {
    let columns = {
        let mut stmt = conn.prepare("PRAGMA table_info(hive_sessions)")?;
        stmt.query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?
    };

    if !columns.iter().any(|value| value == "last_wake_at") {
        conn.execute_batch("ALTER TABLE hive_sessions ADD COLUMN last_wake_at TEXT;")?;
    }

    Ok(())
}

pub(crate) fn migrate_visibility_column(conn: &Connection) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(memory_items)")?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;

    if !columns.iter().any(|column| column == "visibility") {
        // Default 'workspace' for existing items — they predate enforcement
        // and should remain visible to all project agents.
        conn.execute_batch(
            "ALTER TABLE memory_items ADD COLUMN visibility TEXT NOT NULL DEFAULT '\"workspace\"';",
        )?;
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_memory_visibility ON memory_items(visibility);",
        )?;

        // Backfill from payload_json where visibility was explicitly set.
        // Items without the key in JSON keep the 'workspace' default.
        let mut read_stmt = conn.prepare("SELECT id, payload_json FROM memory_items")?;
        let rows = read_stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (id, payload) = row?;
            let parsed: serde_json::Value = serde_json::from_str(&payload)
                .context("parse payload_json for visibility backfill")?;
            if let Some(vis) = parsed.get("visibility").and_then(|v| v.as_str()) {
                // Only update if explicitly set and different from default.
                // Store as JSON-quoted string to match INSERT/UPDATE format.
                if vis != "workspace" {
                    let quoted = format!("\"{}\"", vis);
                    conn.execute(
                        "UPDATE memory_items SET visibility = ?1 WHERE id = ?2",
                        params![quoted, id],
                    )?;
                }
            }
        }
    }

    Ok(())
}

/// L2.1: Lamport versioning on memory_items.
/// Adds a monotonic `version` column persisted alongside `payload_json` so
/// conflict resolution between harnesses is timestamp-independent.
pub(crate) fn migrate_version_column(conn: &Connection) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(memory_items)")?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;

    if !columns.iter().any(|column| column == "version") {
        conn.execute_batch(
            "ALTER TABLE memory_items ADD COLUMN version INTEGER NOT NULL DEFAULT 1;",
        )?;
        // Backfill payload_json so existing items expose their version via the
        // JSON path too. Default is 1 — they are pre-Lamport but not
        // pre-existent; any cross-harness import should dominate via version 2+.
        conn.execute(
            "UPDATE memory_items \
             SET payload_json = json_set(payload_json, '$.version', 1) \
             WHERE json_extract(payload_json, '$.version') IS NULL",
            [],
        )?;
    }

    Ok(())
}

pub(crate) fn migrate_lane_column(conn: &Connection) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(memory_items)")?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;

    if !columns.iter().any(|column| column == "lane") {
        conn.execute_batch("ALTER TABLE memory_items ADD COLUMN lane TEXT;")?;
        conn.execute_batch("CREATE INDEX IF NOT EXISTS idx_memory_lane ON memory_items(lane);")?;
    }

    Ok(())
}

pub(crate) fn migrate_fts5_index(conn: &Connection) -> anyhow::Result<()> {
    let has_fts = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'memory_items_fts'",
            [],
            |_| Ok(()),
        )
        .optional()?
        .is_some();

    if !has_fts {
        conn.execute_batch(
            r#"
            CREATE VIRTUAL TABLE memory_items_fts USING fts5(
                content,
                tags,
                item_id UNINDEXED
            );

            CREATE TRIGGER IF NOT EXISTS memory_items_fts_ai
            AFTER INSERT ON memory_items BEGIN
                INSERT INTO memory_items_fts(item_id, content, tags)
                VALUES (
                    new.id,
                    COALESCE(json_extract(new.payload_json, '$.content'), ''),
                    COALESCE(json_extract(new.payload_json, '$.tags'), '[]')
                );
            END;

            CREATE TRIGGER IF NOT EXISTS memory_items_fts_au
            AFTER UPDATE ON memory_items BEGIN
                DELETE FROM memory_items_fts WHERE item_id = old.id;
                INSERT INTO memory_items_fts(item_id, content, tags)
                VALUES (
                    new.id,
                    COALESCE(json_extract(new.payload_json, '$.content'), ''),
                    COALESCE(json_extract(new.payload_json, '$.tags'), '[]')
                );
            END;

            CREATE TRIGGER IF NOT EXISTS memory_items_fts_ad
            AFTER DELETE ON memory_items BEGIN
                DELETE FROM memory_items_fts WHERE item_id = old.id;
            END;
            "#,
        )?;

        // Backfill existing items into the FTS index
        conn.execute_batch(
            r#"
            INSERT INTO memory_items_fts(item_id, content, tags)
            SELECT
                id,
                COALESCE(json_extract(payload_json, '$.content'), ''),
                COALESCE(json_extract(payload_json, '$.tags'), '[]')
            FROM memory_items;
            "#,
        )?;
    }

    Ok(())
}

/// E3-D2: episodes + episode_facts + FTS5 narrative index.
/// Idempotent — re-run is a no-op if tables exist.
pub(crate) fn migrate_episodes_tables(conn: &Connection) -> anyhow::Result<()> {
    let has_episodes = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'episodes'",
            [],
            |_| Ok(()),
        )
        .optional()?
        .is_some();
    if !has_episodes {
        conn.execute_batch(
            r#"
            CREATE TABLE episodes (
              id TEXT PRIMARY KEY,
              session_id TEXT NOT NULL,
              mind TEXT,
              title TEXT NOT NULL,
              narrative TEXT NOT NULL,
              project TEXT,
              namespace TEXT,
              started_at TEXT NOT NULL,
              ended_at TEXT NOT NULL,
              fact_count INTEGER NOT NULL DEFAULT 0,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );

            CREATE UNIQUE INDEX IF NOT EXISTS idx_episodes_session_id
              ON episodes(session_id);
            CREATE INDEX IF NOT EXISTS idx_episodes_project_namespace
              ON episodes(project, namespace);
            CREATE INDEX IF NOT EXISTS idx_episodes_ended_at
              ON episodes(ended_at DESC);

            CREATE TABLE episode_facts (
              episode_id TEXT NOT NULL,
              fact_id TEXT NOT NULL,
              relation TEXT NOT NULL,
              PRIMARY KEY (episode_id, fact_id),
              FOREIGN KEY (episode_id) REFERENCES episodes(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_episode_facts_fact_id
              ON episode_facts(fact_id);
            "#,
        )
        .context("create episodes tables")?;
    }

    let has_fts = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'episodes_fts'",
            [],
            |_| Ok(()),
        )
        .optional()?
        .is_some();
    if !has_fts {
        conn.execute_batch(
            r#"
            CREATE VIRTUAL TABLE episodes_fts USING fts5(
                title,
                narrative,
                episode_id UNINDEXED
            );

            CREATE TRIGGER IF NOT EXISTS episodes_fts_ai
            AFTER INSERT ON episodes BEGIN
                INSERT INTO episodes_fts(episode_id, title, narrative)
                VALUES (new.id, new.title, new.narrative);
            END;

            CREATE TRIGGER IF NOT EXISTS episodes_fts_au
            AFTER UPDATE ON episodes BEGIN
                DELETE FROM episodes_fts WHERE episode_id = old.id;
                INSERT INTO episodes_fts(episode_id, title, narrative)
                VALUES (new.id, new.title, new.narrative);
            END;

            CREATE TRIGGER IF NOT EXISTS episodes_fts_ad
            AFTER DELETE ON episodes BEGIN
                DELETE FROM episodes_fts WHERE episode_id = old.id;
            END;
            "#,
        )
        .context("create episodes_fts")?;

        conn.execute_batch(
            r#"
            INSERT INTO episodes_fts(episode_id, title, narrative)
            SELECT id, title, narrative FROM episodes;
            "#,
        )
        .context("backfill episodes_fts")?;
    }

    Ok(())
}

/// V3/B3: kill the O(N) `list_entities()` scan on `/memory/store`.
///
/// `auto_link_entity`, `create_wiki_links`, and `create_named_entity_links`
/// each scanned every row in `memory_entities` per store. At N=100 the LME
/// ingest stalled. Fix:
///   - Generated column `project_id` from `json_extract(payload_json, '$.context.project')`
///     with an index on `(project_id, updated_at DESC)`.
///   - Companion table `memory_entity_aliases(entity_id, alias, project)` with
///     index on `(project, alias)` (NOCASE collation for case-insensitive
///     exact match + LIKE substring scope).
///   - Backfill both from existing `payload_json` rows on first run.
///
/// Idempotent: checks for existing column/table before creating.
pub(crate) fn migrate_memory_entities_indexed_lookups(conn: &mut Connection) -> anyhow::Result<()> {
    // `PRAGMA table_info` does NOT list generated columns; `table_xinfo` does.
    // See https://www.sqlite.org/pragma.html#pragma_table_xinfo.
    let existing_cols: Vec<String> = {
        let mut stmt = conn.prepare("PRAGMA table_xinfo(memory_entities)")?;
        stmt.query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?
    };
    let has_project_id = existing_cols.iter().any(|c| c == "project_id");
    if !has_project_id {
        conn.execute_batch(
            "ALTER TABLE memory_entities ADD COLUMN project_id TEXT \
             GENERATED ALWAYS AS (json_extract(payload_json, '$.context.project')) VIRTUAL;",
        )
        .context("add memory_entities.project_id generated column")?;
    }
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_memory_entities_project_updated \
         ON memory_entities(project_id, updated_at DESC);",
    )
    .context("create idx_memory_entities_project_updated")?;

    let has_aliases_table = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'memory_entity_aliases'",
            [],
            |_| Ok(()),
        )
        .optional()?
        .is_some();
    if !has_aliases_table {
        conn.execute_batch(
            r#"
            CREATE TABLE memory_entity_aliases (
              entity_id TEXT NOT NULL,
              alias TEXT NOT NULL COLLATE NOCASE,
              project TEXT,
              PRIMARY KEY (entity_id, alias)
            );
            CREATE INDEX idx_memory_entity_aliases_project_alias
              ON memory_entity_aliases(project, alias);
            CREATE INDEX idx_memory_entity_aliases_alias
              ON memory_entity_aliases(alias);
            "#,
        )
        .context("create memory_entity_aliases table")?;
    }

    let aliases_empty = conn
        .query_row(
            "SELECT 1 FROM memory_entity_aliases LIMIT 1",
            [],
            |_| Ok(()),
        )
        .optional()?
        .is_none();
    let entities_present = conn
        .query_row("SELECT 1 FROM memory_entities LIMIT 1", [], |_| Ok(()))
        .optional()?
        .is_some();
    if aliases_empty && entities_present {
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .context("begin memory_entity_aliases backfill")?;
        {
            let mut rows = tx.prepare("SELECT id, payload_json FROM memory_entities")?;
            let mut insert = tx.prepare(
                "INSERT OR IGNORE INTO memory_entity_aliases (entity_id, alias, project) \
                 VALUES (?1, ?2, ?3)",
            )?;
            let mut iter = rows.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            for row in iter.by_ref() {
                let (entity_id, payload) = row?;
                let record: MemoryEntityRecord = match serde_json::from_str(&payload) {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                let project = record.context.as_ref().and_then(|ctx| ctx.project.clone());
                for alias in record.aliases.iter() {
                    if alias.trim().is_empty() {
                        continue;
                    }
                    insert.execute(params![entity_id, alias, project])?;
                }
            }
        }
        tx.commit()
            .context("commit memory_entity_aliases backfill")?;
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
