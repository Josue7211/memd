//! E3-D2: store-level CRUD for episodes + episode_facts.
//!
//! Drives the pure-function core in `episodes.rs` against sqlite. The
//! consolidation flow:
//!
//!   1. Pull `memory_items` in the window, ordered by `updated_at`.
//!   2. `detect_sessions` turns that stream into `SessionSpan`s.
//!   3. For each span, `session_id` (deterministic UUIDv5) is looked up in
//!      `episodes(session_id UNIQUE)` — hits become `idempotent_skipped`.
//!   4. Misses get `build_episode` + atomic insert into episodes +
//!      episode_facts (one row per linked memory_item).
//!
//! `list_episodes` supports an optional FTS5 query against the
//! `episodes_fts(title, narrative)` virtual table.

use anyhow::Context;
use chrono::{DateTime, Utc};
use memd_schema::{
    ConsolidateEpisodesRequest, ConsolidateEpisodesResponse, Episode, EpisodeFactRelation,
    ListEpisodesRequest, ListEpisodesResponse, MemoryItem, SessionSpan,
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use uuid::Uuid;

use crate::episodes::{
    DEFAULT_SESSION_GAP_SECONDS, EventPoint, build_episode, classify_relation, detect_sessions,
};
use crate::store::SqliteStore;

impl SqliteStore {
    pub fn consolidate_episodes(
        &self,
        req: &ConsolidateEpisodesRequest,
        now: DateTime<Utc>,
    ) -> anyhow::Result<ConsolidateEpisodesResponse> {
        let gap = req
            .session_gap_seconds
            .unwrap_or(DEFAULT_SESSION_GAP_SECONDS);
        let since = req
            .since
            .unwrap_or_else(|| now - chrono::Duration::hours(24));

        let mut conn = self.connect()?;
        let items = load_items_in_window(
            &conn,
            req.project.as_deref(),
            req.namespace.as_deref(),
            since,
        )?;

        let events: Vec<EventPoint> = items
            .iter()
            .map(|it| EventPoint {
                memory_id: it.id,
                at: it.updated_at,
            })
            .collect();
        let total_events_scanned = events.len();

        let spans = detect_sessions(
            &events,
            gap,
            req.project.as_deref(),
            req.namespace.as_deref(),
        );

        let mut episodes_created = Vec::new();
        let mut idempotent_skipped = 0usize;

        for span in &spans {
            if episode_exists(&conn, span.id)? {
                idempotent_skipped += 1;
                continue;
            }

            let span_items: Vec<MemoryItem> = span
                .memory_ids
                .iter()
                .filter_map(|id| items.iter().find(|it| &it.id == id).cloned())
                .collect();

            let episode = build_episode(span, &span_items, now);

            if !req.dry_run {
                let fact_links: Vec<(Uuid, EpisodeFactRelation)> = span_items
                    .iter()
                    .map(|it| (it.id, classify_relation(it)))
                    .collect();
                insert_episode_tx(&mut conn, &episode, &fact_links)?;
            }

            episodes_created.push(episode);
        }

        Ok(ConsolidateEpisodesResponse {
            sessions_detected: spans.len(),
            episodes_created,
            idempotent_skipped,
            total_events_scanned,
        })
    }

    pub fn list_episodes(&self, req: &ListEpisodesRequest) -> anyhow::Result<ListEpisodesResponse> {
        let conn = self.connect()?;
        let limit = req.limit.unwrap_or(50).min(500);

        let episodes = if let Some(query) = req.query.as_ref().filter(|q| !q.trim().is_empty()) {
            list_episodes_fts(
                &conn,
                query,
                req.project.as_deref(),
                req.namespace.as_deref(),
                limit,
            )?
        } else {
            list_episodes_recent(
                &conn,
                req.project.as_deref(),
                req.namespace.as_deref(),
                limit,
            )?
        };
        Ok(ListEpisodesResponse { episodes })
    }

    #[cfg(test)]
    pub(crate) fn count_episode_facts(&self) -> anyhow::Result<usize> {
        let conn = self.connect()?;
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM episode_facts", [], |row| row.get(0))
            .context("count episode_facts")?;
        Ok(n as usize)
    }
}

fn episode_exists(conn: &Connection, session_id: Uuid) -> anyhow::Result<bool> {
    let hit = conn
        .query_row(
            "SELECT 1 FROM episodes WHERE session_id = ?1",
            params![session_id.to_string()],
            |_| Ok(()),
        )
        .optional()
        .context("probe episodes by session_id")?;
    Ok(hit.is_some())
}

fn load_items_in_window(
    conn: &Connection,
    project: Option<&str>,
    namespace: Option<&str>,
    since: DateTime<Utc>,
) -> anyhow::Result<Vec<MemoryItem>> {
    let mut sql = String::from("SELECT payload_json FROM memory_items WHERE updated_at >= ?1");
    let mut args: Vec<String> = vec![since.to_rfc3339()];
    if project.is_some() {
        sql.push_str(" AND project = ?");
        sql.push_str(&(args.len() + 1).to_string());
        args.push(project.unwrap().to_string());
    }
    if namespace.is_some() {
        sql.push_str(" AND namespace = ?");
        sql.push_str(&(args.len() + 1).to_string());
        args.push(namespace.unwrap().to_string());
    }
    sql.push_str(" ORDER BY updated_at ASC");

    let mut stmt = conn.prepare(&sql).context("prepare window query")?;
    let param_refs: Vec<&dyn rusqlite::ToSql> =
        args.iter().map(|a| a as &dyn rusqlite::ToSql).collect();
    let rows = stmt
        .query_map(param_refs.as_slice(), |row| row.get::<_, String>(0))
        .context("query window items")?;

    let mut items = Vec::new();
    for row in rows {
        let payload = row.context("read window row")?;
        let item: MemoryItem =
            serde_json::from_str(&payload).context("deserialize memory item in window")?;
        items.push(item);
    }
    Ok(items)
}

fn insert_episode_tx(
    conn: &mut Connection,
    ep: &Episode,
    facts: &[(Uuid, EpisodeFactRelation)],
) -> anyhow::Result<()> {
    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .context("begin episode insert tx")?;
    tx.execute(
        r#"
        INSERT INTO episodes (
            id, session_id, mind, title, narrative, project, namespace,
            started_at, ended_at, fact_count, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        "#,
        params![
            ep.id.to_string(),
            ep.session_id.to_string(),
            ep.mind,
            ep.title,
            ep.narrative,
            ep.project,
            ep.namespace,
            ep.started_at.to_rfc3339(),
            ep.ended_at.to_rfc3339(),
            ep.fact_count as i64,
            ep.created_at.to_rfc3339(),
            ep.updated_at.to_rfc3339(),
        ],
    )
    .context("insert episodes row")?;

    {
        let mut stmt = tx
            .prepare(
                "INSERT OR IGNORE INTO episode_facts (episode_id, fact_id, relation) VALUES (?1, ?2, ?3)",
            )
            .context("prepare episode_facts insert")?;
        for (fact_id, relation) in facts {
            stmt.execute(params![
                ep.id.to_string(),
                fact_id.to_string(),
                relation_str(*relation),
            ])
            .context("insert episode_facts row")?;
        }
    }

    tx.commit().context("commit episode insert tx")?;
    Ok(())
}

fn relation_str(r: EpisodeFactRelation) -> &'static str {
    match r {
        EpisodeFactRelation::Origin => "origin",
        EpisodeFactRelation::Evidence => "evidence",
        EpisodeFactRelation::Reference => "reference",
        EpisodeFactRelation::Outcome => "outcome",
    }
}

const EPISODE_COLS: &str = "id, session_id, mind, title, narrative, project, namespace, \
     started_at, ended_at, fact_count, created_at, updated_at";

fn list_episodes_recent(
    conn: &Connection,
    project: Option<&str>,
    namespace: Option<&str>,
    limit: usize,
) -> anyhow::Result<Vec<Episode>> {
    let mut sql = format!("SELECT {EPISODE_COLS} FROM episodes WHERE 1=1");
    let mut args: Vec<String> = Vec::new();
    if let Some(p) = project {
        sql.push_str(" AND project = ?");
        sql.push_str(&(args.len() + 1).to_string());
        args.push(p.to_string());
    }
    if let Some(n) = namespace {
        sql.push_str(" AND namespace = ?");
        sql.push_str(&(args.len() + 1).to_string());
        args.push(n.to_string());
    }
    sql.push_str(&format!(" ORDER BY ended_at DESC LIMIT {limit}"));

    let mut stmt = conn.prepare(&sql).context("prepare list_episodes_recent")?;
    let refs: Vec<&dyn rusqlite::ToSql> = args.iter().map(|a| a as &dyn rusqlite::ToSql).collect();
    let rows = stmt
        .query_map(refs.as_slice(), episode_from_row)
        .context("query episodes")?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.context("read episode row")?);
    }
    Ok(out)
}

fn list_episodes_fts(
    conn: &Connection,
    query: &str,
    project: Option<&str>,
    namespace: Option<&str>,
    limit: usize,
) -> anyhow::Result<Vec<Episode>> {
    let q = sanitize_fts_query(query);
    if q.is_empty() {
        return list_episodes_recent(conn, project, namespace, limit);
    }

    let episode_cols_e: String = EPISODE_COLS
        .split(", ")
        .map(|c| format!("e.{c}"))
        .collect::<Vec<_>>()
        .join(", ");
    let mut sql = format!(
        "SELECT {episode_cols_e} \
         FROM episodes e \
         JOIN episodes_fts f ON f.episode_id = e.id \
         WHERE episodes_fts MATCH ?1"
    );
    let mut args: Vec<String> = vec![q];
    if let Some(p) = project {
        sql.push_str(" AND e.project = ?");
        sql.push_str(&(args.len() + 1).to_string());
        args.push(p.to_string());
    }
    if let Some(n) = namespace {
        sql.push_str(" AND e.namespace = ?");
        sql.push_str(&(args.len() + 1).to_string());
        args.push(n.to_string());
    }
    sql.push_str(&format!(" ORDER BY bm25(episodes_fts) ASC LIMIT {limit}"));

    let mut stmt = conn.prepare(&sql).context("prepare list_episodes_fts")?;
    let refs: Vec<&dyn rusqlite::ToSql> = args.iter().map(|a| a as &dyn rusqlite::ToSql).collect();
    let rows = stmt
        .query_map(refs.as_slice(), episode_from_row)
        .context("query episodes fts")?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.context("read episode fts row")?);
    }
    Ok(out)
}

fn sanitize_fts_query(q: &str) -> String {
    // Strip FTS5 syntax characters, then quote each token for safe MATCH.
    let cleaned: String = q
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c.is_whitespace() {
                c
            } else {
                ' '
            }
        })
        .collect();
    let tokens: Vec<String> = cleaned
        .split_whitespace()
        .filter(|t| t.len() >= 2)
        .map(|t| format!("\"{t}\""))
        .collect();
    tokens.join(" ")
}

fn episode_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Episode> {
    let id_s: String = row.get(0)?;
    let sid_s: String = row.get(1)?;
    let mind: Option<String> = row.get(2)?;
    let title: String = row.get(3)?;
    let narrative: String = row.get(4)?;
    let project: Option<String> = row.get(5)?;
    let namespace: Option<String> = row.get(6)?;
    let started_s: String = row.get(7)?;
    let ended_s: String = row.get(8)?;
    let fact_count: i64 = row.get(9)?;
    let created_s: String = row.get(10)?;
    let updated_s: String = row.get(11)?;

    Ok(Episode {
        id: Uuid::parse_str(&id_s).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?,
        mind,
        title,
        narrative,
        session_id: Uuid::parse_str(&sid_s).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(e))
        })?,
        project,
        namespace,
        started_at: parse_ts(&started_s, 7)?,
        ended_at: parse_ts(&ended_s, 8)?,
        fact_count: fact_count as usize,
        created_at: parse_ts(&created_s, 10)?,
        updated_at: parse_ts(&updated_s, 11)?,
    })
}

fn parse_ts(s: &str, col: usize) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(col, rusqlite::types::Type::Text, Box::new(e))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use memd_schema::{
        MemoryItem, MemoryKind, MemoryScope, MemoryStage, MemoryStatus, MemoryVisibility,
    };

    fn test_store() -> SqliteStore {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.into_path().join("episodes.db");
        SqliteStore::open(path).expect("open store")
    }

    fn item(id: Uuid, content: &str, at: DateTime<Utc>, kind: MemoryKind) -> MemoryItem {
        MemoryItem {
            id,
            content: content.to_string(),
            redundancy_key: None,
            belief_branch: None,
            preferred: false,
            kind,
            scope: MemoryScope::Project,
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            workspace: None,
            visibility: MemoryVisibility::default(),
            source_agent: None,
            source_system: None,
            source_path: None,
            source_quality: None,
            confidence: 0.8,
            ttl_seconds: None,
            created_at: at,
            updated_at: at,
            last_verified_at: None,
            supersedes: Vec::new(),
            tags: Vec::new(),
            status: MemoryStatus::Active,
            stage: MemoryStage::Canonical,
            lane: None,
            version: 1,
            correction_meta: None,
        }
    }

    fn t(h: u32, m: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 4, 20, h, m, 0).unwrap()
    }

    #[test]
    fn consolidate_empty_returns_empty_response() {
        let store = test_store();
        let now = Utc::now();
        let resp = store
            .consolidate_episodes(
                &ConsolidateEpisodesRequest {
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    since: Some(t(0, 0)),
                    session_gap_seconds: None,
                    dry_run: false,
                },
                now,
            )
            .expect("consolidate");
        assert_eq!(resp.sessions_detected, 0);
        assert_eq!(resp.episodes_created.len(), 0);
        assert_eq!(resp.idempotent_skipped, 0);
        assert_eq!(resp.total_events_scanned, 0);
    }

    fn seed_items(store: &SqliteStore, items: &[MemoryItem]) {
        use crate::keys::{canonical_key, redundancy_key};
        for it in items {
            let ck = canonical_key(it);
            let rk = redundancy_key(it);
            store
                .insert_or_get_duplicate(it, &ck, &rk)
                .expect("seed item");
        }
    }

    #[test]
    fn consolidate_detects_sessions_and_creates_episodes() {
        let store = test_store();
        // 2 sessions: 10:00-10:10, 12:00-12:10 (2h gap > 30min)
        let items = vec![
            item(Uuid::new_v4(), "alpha one", t(10, 0), MemoryKind::Fact),
            item(Uuid::new_v4(), "alpha two", t(10, 10), MemoryKind::Decision),
            item(Uuid::new_v4(), "beta one", t(12, 0), MemoryKind::Fact),
            item(Uuid::new_v4(), "beta two", t(12, 10), MemoryKind::Fact),
        ];
        seed_items(&store, &items);

        let resp = store
            .consolidate_episodes(
                &ConsolidateEpisodesRequest {
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    since: Some(t(0, 0)),
                    session_gap_seconds: None,
                    dry_run: false,
                },
                Utc::now(),
            )
            .expect("consolidate");

        assert_eq!(resp.sessions_detected, 2);
        assert_eq!(resp.episodes_created.len(), 2);
        assert_eq!(resp.idempotent_skipped, 0);
        assert_eq!(resp.total_events_scanned, 4);
        assert_eq!(store.count_episode_facts().unwrap(), 4);
    }

    #[test]
    fn consolidate_is_idempotent_on_rerun() {
        let store = test_store();
        let items = vec![
            item(Uuid::new_v4(), "alpha", t(10, 0), MemoryKind::Fact),
            item(Uuid::new_v4(), "beta", t(12, 30), MemoryKind::Fact),
        ];
        seed_items(&store, &items);

        let req = ConsolidateEpisodesRequest {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            since: Some(t(0, 0)),
            session_gap_seconds: None,
            dry_run: false,
        };
        let first = store.consolidate_episodes(&req, Utc::now()).unwrap();
        assert_eq!(first.episodes_created.len(), 2);
        assert_eq!(first.idempotent_skipped, 0);

        let second = store.consolidate_episodes(&req, Utc::now()).unwrap();
        assert_eq!(second.sessions_detected, 2);
        assert_eq!(second.episodes_created.len(), 0);
        assert_eq!(second.idempotent_skipped, 2);
    }

    #[test]
    fn dry_run_does_not_persist() {
        let store = test_store();
        let items = vec![item(Uuid::new_v4(), "solo", t(10, 0), MemoryKind::Fact)];
        seed_items(&store, &items);

        let resp = store
            .consolidate_episodes(
                &ConsolidateEpisodesRequest {
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    since: Some(t(0, 0)),
                    session_gap_seconds: None,
                    dry_run: true,
                },
                Utc::now(),
            )
            .unwrap();
        assert_eq!(resp.episodes_created.len(), 1);

        // Dry run must not persist.
        let listed = store
            .list_episodes(&ListEpisodesRequest::default())
            .unwrap();
        assert!(listed.episodes.is_empty());
    }

    #[test]
    fn list_episodes_fts_query_finds_by_narrative() {
        let store = test_store();
        let items = vec![
            item(Uuid::new_v4(), "raccoon update", t(10, 0), MemoryKind::Fact),
            item(Uuid::new_v4(), "badger update", t(12, 30), MemoryKind::Fact),
        ];
        seed_items(&store, &items);
        store
            .consolidate_episodes(
                &ConsolidateEpisodesRequest {
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    since: Some(t(0, 0)),
                    session_gap_seconds: None,
                    dry_run: false,
                },
                Utc::now(),
            )
            .unwrap();

        let hits = store
            .list_episodes(&ListEpisodesRequest {
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                limit: Some(10),
                query: Some("raccoon".to_string()),
            })
            .unwrap();
        assert_eq!(hits.episodes.len(), 1);
        assert!(hits.episodes[0].narrative.contains("raccoon"));
    }
}
