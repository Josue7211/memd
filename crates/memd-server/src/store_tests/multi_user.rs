use super::*;
use rusqlite::params;

fn identity_columns(store: &SqliteStore) -> Vec<String> {
    let conn = store.connect().expect("connect store");
    let mut stmt = conn
        .prepare("PRAGMA table_info(memory_items)")
        .expect("prepare table info");
    stmt.query_map([], |row| row.get::<_, String>(1))
        .expect("query columns")
        .collect::<Result<Vec<_>, _>>()
        .expect("read columns")
}

#[test]
fn a9_migration_adds_identity_columns_idempotently() {
    let (dir, store) = open_temp_store("a9-identity-columns");
    for column in ["user_id", "harness_preset", "user_id_session_seq"] {
        assert!(
            identity_columns(&store).iter().any(|found| found == column),
            "missing identity column {column}"
        );
    }

    let reopened = SqliteStore::open(dir.join("state.sqlite")).expect("reopen sqlite store");
    for column in ["user_id", "harness_preset", "user_id_session_seq"] {
        assert!(
            identity_columns(&reopened)
                .iter()
                .any(|found| found == column),
            "missing identity column after reopen {column}"
        );
    }
    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[test]
fn a9_store_persists_user_identity_columns_on_insert() {
    let (dir, store) = open_temp_store("a9-insert-identity");
    let mut item = sample_memory_item();
    item.source_agent = Some("user-a-agent".to_string());
    item.source_system = Some("codex".to_string());
    let canonical_key = canonical_key(&item);
    let redundancy_key = redundancy_key(&item);

    store
        .insert_or_get_duplicate(&item, &canonical_key, &redundancy_key)
        .expect("insert item");

    let conn = store.connect().expect("connect store");
    let row = conn
        .query_row(
            "SELECT user_id, harness_preset, user_id_session_seq FROM memory_items WHERE id = ?1",
            params![item.id.to_string()],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .expect("read identity columns");
    assert_eq!(row.0.as_deref(), Some("user-a-agent"));
    assert_eq!(row.1.as_deref(), Some("codex"));
    assert_eq!(row.2, 1);
    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}

#[test]
fn a9_store_backfills_legacy_user_identity() {
    let dir = std::env::temp_dir().join(format!("a9-legacy-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let db = dir.join("state.sqlite");
    let legacy = rusqlite::Connection::open(&db).expect("open legacy db");
    legacy
        .execute_batch(
            r#"
            CREATE TABLE memory_items (
              id TEXT PRIMARY KEY,
              kind TEXT NOT NULL,
              scope TEXT NOT NULL,
              stage TEXT NOT NULL,
              project TEXT,
              namespace TEXT,
              source_agent TEXT,
              redundancy_key TEXT,
              status TEXT NOT NULL,
              confidence REAL NOT NULL,
              canonical_key TEXT NOT NULL,
              embedding_model TEXT,
              updated_at TEXT NOT NULL,
              payload_json TEXT NOT NULL,
              version INTEGER NOT NULL DEFAULT 1,
              lane TEXT,
              visibility TEXT
            );
            "#,
        )
        .expect("create legacy table");

    let mut item = sample_memory_item();
    item.source_agent = Some("legacy-user-agent".to_string());
    item.source_system = Some("claude-code".to_string());
    legacy
        .execute(
            r#"
            INSERT INTO memory_items (
              id, kind, scope, stage, project, namespace, source_agent, redundancy_key, status,
              confidence, canonical_key, updated_at, payload_json, version, lane, visibility
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
            "#,
            params![
                item.id.to_string(),
                serde_json::to_string(&item.kind).expect("kind"),
                serde_json::to_string(&item.scope).expect("scope"),
                serde_json::to_string(&item.stage).expect("stage"),
                item.project.clone(),
                item.namespace.clone(),
                item.source_agent.clone(),
                redundancy_key(&item),
                serde_json::to_string(&item.status).expect("status"),
                item.confidence,
                canonical_key(&item),
                item.updated_at.to_rfc3339(),
                serde_json::to_string(&item).expect("payload"),
                item.version as i64,
                item.lane.clone(),
                serde_json::to_string(&item.visibility).expect("visibility"),
            ],
        )
        .expect("insert legacy item");
    drop(legacy);

    let store = SqliteStore::open(&db).expect("migrate legacy db");
    let conn = store.connect().expect("connect migrated db");
    let row = conn
        .query_row(
            "SELECT user_id, harness_preset, user_id_session_seq FROM memory_items WHERE id = ?1",
            params![item.id.to_string()],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .expect("read migrated identity");
    assert_eq!(row.0.as_deref(), Some("legacy-user-agent"));
    assert_eq!(row.1.as_deref(), Some("claude-code"));
    assert_eq!(row.2, 0);
    std::fs::remove_dir_all(dir).expect("cleanup temp dir");
}
