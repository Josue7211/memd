use super::*;

impl SqliteStore {
    pub fn find_duplicate(
        &self,
        redundancy_key: &str,
        canonical_key: &str,
        stage: &memd_schema::MemoryStage,
        exclude_id: Uuid,
    ) -> anyhow::Result<Option<DuplicateMatch>> {
        self.find_by_any_key(redundancy_key, canonical_key, stage)
            .map(|found| found.filter(|duplicate| duplicate.id != exclude_id))
    }

    pub(super) fn find_by_any_key(
        &self,
        redundancy_key: &str,
        canonical_key: &str,
        stage: &memd_schema::MemoryStage,
    ) -> anyhow::Result<Option<DuplicateMatch>> {
        let stage = serde_json::to_string(stage).context("serialize lookup stage")?;
        let conn = self.connect()?;
        let row = conn.query_row(
            r#"
            SELECT id, payload_json
            FROM memory_items
            WHERE (redundancy_key = ?1 OR canonical_key = ?2) AND stage = ?3
            "#,
            params![redundancy_key, canonical_key, stage],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        );

        match row {
            Ok((id, payload)) => {
                let item: MemoryItem =
                    serde_json::from_str(&payload).context("deserialize duplicate memory item")?;
                Ok(Some(DuplicateMatch {
                    id: Uuid::parse_str(&id).context("parse duplicate id")?,
                    item,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err).context("lookup duplicate excluding current item"),
        }
    }

    pub(super) fn get_entity_by_key(
        &self,
        entity_key: &str,
    ) -> anyhow::Result<Option<MemoryEntityRecord>> {
        let conn = self.connect()?;
        let payload = conn.query_row(
            "SELECT payload_json FROM memory_entities WHERE entity_key = ?1",
            [entity_key],
            |row| row.get::<_, String>(0),
        );

        match payload {
            Ok(payload) => {
                let entity: MemoryEntityRecord =
                    serde_json::from_str(&payload).context("deserialize memory entity payload")?;
                Ok(Some(entity))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err).context("fetch memory entity by key"),
        }
    }

    pub(super) fn upsert_entity(
        &self,
        entity_key: &str,
        record: &MemoryEntityRecord,
    ) -> anyhow::Result<()> {
        let payload_json = serde_json::to_string(record).context("serialize memory entity")?;
        let entity_id = record.id.to_string();
        let project = record.context.as_ref().and_then(|ctx| ctx.project.clone());
        let mut conn = self.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .context("begin upsert_entity tx")?;
        tx.execute(
            r#"
            INSERT INTO memory_entities (id, entity_key, entity_type, updated_at, payload_json)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(entity_key) DO UPDATE SET
              entity_type = excluded.entity_type,
              updated_at = excluded.updated_at,
              payload_json = excluded.payload_json
            "#,
            params![
                entity_id,
                entity_key,
                record.entity_type,
                record.updated_at.to_rfc3339(),
                payload_json,
            ],
        )
        .context("upsert memory entity")?;

        // Sync aliases companion table. `upsert_entity` may reference an
        // existing row via `entity_key`; resolve the canonical id first so
        // UPDATE and INSERT both converge on the same alias set.
        let canonical_id: String = tx
            .query_row(
                "SELECT id FROM memory_entities WHERE entity_key = ?1",
                params![entity_key],
                |row| row.get::<_, String>(0),
            )
            .context("resolve canonical entity id for alias sync")?;
        tx.execute(
            "DELETE FROM memory_entity_aliases WHERE entity_id = ?1",
            params![canonical_id],
        )
        .context("clear prior aliases for entity")?;
        {
            let mut insert = tx.prepare(
                "INSERT OR IGNORE INTO memory_entity_aliases (entity_id, alias, project) \
                 VALUES (?1, ?2, ?3)",
            )?;
            for alias in record.aliases.iter() {
                if alias.trim().is_empty() {
                    continue;
                }
                insert.execute(params![canonical_id, alias, project])?;
            }
        }
        tx.commit().context("commit upsert_entity tx")?;
        Ok(())
    }

    /// V3/B3: project-scoped recent entities via `idx_memory_entities_project_updated`.
    /// Replaces full-table `list_entities()` scan in `auto_link_entity`.
    pub fn list_entities_by_project(
        &self,
        project: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<MemoryEntityRecord>> {
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                "SELECT payload_json FROM memory_entities \
                 WHERE project_id = ?1 \
                 ORDER BY updated_at DESC LIMIT ?2",
            )
            .context("prepare list_entities_by_project")?;
        let rows = stmt.query_map(params![project, limit as i64], |row| {
            row.get::<_, String>(0)
        })?;
        let mut out = Vec::new();
        for row in rows {
            let payload = row.context("read entity row")?;
            let entity: MemoryEntityRecord =
                serde_json::from_str(&payload).context("deserialize memory entity payload")?;
            out.push(entity);
        }
        Ok(out)
    }

    /// V3/B3: exact (NOCASE) alias lookup via `idx_memory_entity_aliases_project_alias`.
    /// Replaces the full scan in `create_named_entity_links`.
    pub fn find_entities_by_alias_exact(
        &self,
        project: Option<&str>,
        alias: &str,
    ) -> anyhow::Result<Vec<MemoryEntityRecord>> {
        let conn = self.connect()?;
        let mut out = Vec::new();
        let mut stmt = if project.is_some() {
            conn.prepare(
                "SELECT e.payload_json FROM memory_entities e \
                 JOIN memory_entity_aliases a ON a.entity_id = e.id \
                 WHERE a.project IS ?1 AND a.alias = ?2 \
                 ORDER BY e.updated_at DESC",
            )?
        } else {
            conn.prepare(
                "SELECT e.payload_json FROM memory_entities e \
                 JOIN memory_entity_aliases a ON a.entity_id = e.id \
                 WHERE a.alias = ?1 \
                 ORDER BY e.updated_at DESC",
            )?
        };
        let mut rows = if let Some(p) = project {
            stmt.query(params![p, alias])?
        } else {
            stmt.query(params![alias])?
        };
        while let Some(row) = rows.next()? {
            let payload: String = row.get(0)?;
            let entity: MemoryEntityRecord =
                serde_json::from_str(&payload).context("deserialize memory entity payload")?;
            out.push(entity);
        }
        Ok(out)
    }

    /// V3/B3: substring alias lookup for `create_wiki_links`. Optionally
    /// project-scoped. Uses `LIKE '%token%'` over `memory_entity_aliases`,
    /// so the scan cost is proportional to the alias table (one row per
    /// alias) rather than to `memory_entities` full payload deserialization.
    /// No LIMIT: the original `list_entities()` path considered every
    /// entity, and dropping matches beyond a top-N window would silently
    /// change wiki-link resolution.
    pub fn find_entities_by_alias_contains(
        &self,
        project: Option<&str>,
        needle: &str,
    ) -> anyhow::Result<Vec<MemoryEntityRecord>> {
        let conn = self.connect()?;
        let pattern = format!("%{}%", needle);
        let mut out = Vec::new();
        let mut stmt = if project.is_some() {
            conn.prepare(
                "SELECT e.payload_json FROM memory_entities e \
                 JOIN memory_entity_aliases a ON a.entity_id = e.id \
                 WHERE a.project IS ?1 AND a.alias LIKE ?2 \
                 ORDER BY e.updated_at DESC",
            )?
        } else {
            conn.prepare(
                "SELECT e.payload_json FROM memory_entities e \
                 JOIN memory_entity_aliases a ON a.entity_id = e.id \
                 WHERE a.alias LIKE ?1 \
                 ORDER BY e.updated_at DESC",
            )?
        };
        let mut rows = if let Some(p) = project {
            stmt.query(params![p, pattern])?
        } else {
            stmt.query(params![pattern])?
        };
        while let Some(row) = rows.next()? {
            let payload: String = row.get(0)?;
            let entity: MemoryEntityRecord =
                serde_json::from_str(&payload).context("deserialize memory entity payload")?;
            out.push(entity);
        }
        Ok(out)
    }

    pub(super) fn insert_event(
        &self,
        event: &MemoryEventRecord,
        memory_item_id: Option<Uuid>,
    ) -> anyhow::Result<()> {
        let payload_json = serde_json::to_string(event).context("serialize memory event")?;
        self.insert_event_payload(event, memory_item_id, payload_json)
    }

    pub(super) fn insert_event_payload(
        &self,
        event: &MemoryEventRecord,
        memory_item_id: Option<Uuid>,
        payload_json: String,
    ) -> anyhow::Result<()> {
        let conn = self.connect()?;
        conn.execute(
            r#"
            INSERT INTO memory_events (
              id, memory_item_id, entity_id, event_type, occurred_at, recorded_at, payload_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                event.id.to_string(),
                memory_item_id.map(|value| value.to_string()),
                event
                    .entity_id
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                event.event_type,
                event.occurred_at.to_rfc3339(),
                event.recorded_at.to_rfc3339(),
                payload_json,
            ],
        )
        .context("insert memory event")?;
        Ok(())
    }

    // ── Ingestion Manifest ──────────────────────────────────────────

    /// Look up a manifest entry by source path.
    pub fn ingestion_manifest_get(
        &self,
        source_path: &str,
    ) -> anyhow::Result<Option<IngestionManifestEntry>> {
        let conn = self.connect()?;
        let row = conn.query_row(
            "SELECT source_path, content_hash, mtime_epoch, lane, project, namespace, last_ingested_at, memory_item_id FROM ingestion_manifest WHERE source_path = ?1",
            [source_path],
            |row| {
                Ok(IngestionManifestEntry {
                    source_path: row.get(0)?,
                    content_hash: row.get(1)?,
                    mtime_epoch: row.get(2)?,
                    lane: row.get(3)?,
                    project: row.get(4)?,
                    namespace: row.get(5)?,
                    last_ingested_at: row.get(6)?,
                    memory_item_id: row.get(7)?,
                })
            },
        );
        match row {
            Ok(entry) => Ok(Some(entry)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err).context("lookup ingestion manifest entry"),
        }
    }

    /// Upsert a manifest entry after ingesting a file.
    pub fn ingestion_manifest_upsert(&self, entry: &IngestionManifestEntry) -> anyhow::Result<()> {
        let conn = self.connect()?;
        conn.execute(
            r#"
            INSERT INTO ingestion_manifest (
              source_path, content_hash, mtime_epoch, lane, project, namespace, last_ingested_at, memory_item_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(source_path) DO UPDATE SET
              content_hash = excluded.content_hash,
              mtime_epoch = excluded.mtime_epoch,
              lane = excluded.lane,
              last_ingested_at = excluded.last_ingested_at,
              memory_item_id = excluded.memory_item_id
            "#,
            params![
                entry.source_path,
                entry.content_hash,
                entry.mtime_epoch,
                entry.lane,
                entry.project,
                entry.namespace,
                entry.last_ingested_at,
                entry.memory_item_id,
            ],
        )
        .context("upsert ingestion manifest entry")?;
        Ok(())
    }

    /// List all manifest entries for a project/namespace.
    pub fn ingestion_manifest_list(
        &self,
        project: Option<&str>,
        namespace: Option<&str>,
    ) -> anyhow::Result<Vec<IngestionManifestEntry>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT source_path, content_hash, mtime_epoch, lane, project, namespace, last_ingested_at, memory_item_id
            FROM ingestion_manifest
            WHERE (?1 IS NULL OR project = ?1)
              AND (?2 IS NULL OR namespace = ?2)
            ORDER BY last_ingested_at DESC
            "#,
        )?;
        let entries = stmt
            .query_map(params![project, namespace], |row| {
                Ok(IngestionManifestEntry {
                    source_path: row.get(0)?,
                    content_hash: row.get(1)?,
                    mtime_epoch: row.get(2)?,
                    lane: row.get(3)?,
                    project: row.get(4)?,
                    namespace: row.get(5)?,
                    last_ingested_at: row.get(6)?,
                    memory_item_id: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()
            .context("list ingestion manifest")?;
        Ok(entries)
    }

    pub fn upsert_capabilities(
        &self,
        req: &CapabilitySyncRequest,
    ) -> anyhow::Result<CapabilitySyncResponse> {
        let conn = self.connect()?;
        let now = chrono::Utc::now();
        let mut records = Vec::new();
        for input in &req.records {
            if input.harness.trim().is_empty()
                || input.kind.trim().is_empty()
                || input.name.trim().is_empty()
            {
                continue;
            }
            let mut record = input.clone();
            record.project = record.project.or_else(|| req.project.clone());
            record.namespace = record.namespace.or_else(|| req.namespace.clone());
            record.workspace = record.workspace.or_else(|| req.workspace.clone());
            record.user_id = record.user_id.or_else(|| req.user_id.clone());
            record.agent = record.agent.or_else(|| req.agent.clone());
            record.updated_at = Some(now);
            let key = capability_key(&record);
            let payload_json = serde_json::to_string(&record).context("serialize capability")?;
            conn.execute(
                r#"
                INSERT INTO capabilities (
                  capability_key, project, namespace, workspace, user_id, harness, kind, name, status, updated_at, payload_json
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                ON CONFLICT(capability_key) DO UPDATE SET
                  project = excluded.project,
                  namespace = excluded.namespace,
                  workspace = excluded.workspace,
                  user_id = excluded.user_id,
                  harness = excluded.harness,
                  kind = excluded.kind,
                  name = excluded.name,
                  status = excluded.status,
                  updated_at = excluded.updated_at,
                  payload_json = excluded.payload_json
                "#,
                params![
                    key,
                    record.project,
                    record.namespace,
                    record.workspace,
                    record.user_id,
                    record.harness,
                    record.kind,
                    record.name,
                    record.status,
                    now.to_rfc3339(),
                    payload_json,
                ],
            )
            .context("upsert capability")?;
            records.push(record);
        }
        Ok(CapabilitySyncResponse {
            upserted: records.len(),
            total: self
                .list_capabilities(&CapabilityListRequest {
                    project: req.project.clone(),
                    namespace: req.namespace.clone(),
                    workspace: req.workspace.clone(),
                    user_id: req.user_id.clone(),
                    harness: None,
                    kind: None,
                    query: None,
                    limit: Some(500),
                })?
                .total,
            records,
        })
    }

    pub fn list_capabilities(
        &self,
        req: &CapabilityListRequest,
    ) -> anyhow::Result<CapabilityListResponse> {
        let conn = self.connect()?;
        let limit = req.limit.unwrap_or(100).clamp(1, 5_000);
        let query = req
            .query
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| format!("%{}%", value.to_ascii_lowercase()));
        let mut stmt = conn.prepare(
            r#"
            SELECT payload_json
            FROM capabilities
            WHERE (?1 IS NULL OR project = ?1)
              AND (?2 IS NULL OR namespace = ?2)
              AND (?3 IS NULL OR workspace = ?3)
              AND (?4 IS NULL OR user_id = ?4)
              AND (?5 IS NULL OR harness = ?5)
              AND (?6 IS NULL OR kind = ?6)
              AND (
                ?7 IS NULL
                OR lower(name) LIKE ?7
                OR lower(harness) LIKE ?7
                OR lower(kind) LIKE ?7
                OR lower(payload_json) LIKE ?7
              )
            ORDER BY updated_at DESC, harness ASC, kind ASC, name ASC
            LIMIT ?8
            "#,
        )?;
        let records = stmt
            .query_map(
                params![
                    req.project,
                    req.namespace,
                    req.workspace,
                    req.user_id,
                    req.harness,
                    req.kind,
                    query,
                    limit as i64,
                ],
                |row| row.get::<_, String>(0),
            )
            .context("query capabilities")?
            .map(|row| {
                let payload = row?;
                serde_json::from_str::<CapabilityRecord>(&payload).map_err(|err| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(err),
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .context("read capabilities")?;
        Ok(CapabilityListResponse {
            total: records.len(),
            records,
        })
    }

    pub fn upsert_access_routes(
        &self,
        req: &AccessRouteSyncRequest,
    ) -> anyhow::Result<AccessRouteSyncResponse> {
        let conn = self.connect()?;
        let now = chrono::Utc::now();
        let mut routes = Vec::new();
        for input in &req.routes {
            if input.id.trim().is_empty() || input.provider.trim().is_empty() {
                continue;
            }
            if input.secret_values_stored {
                continue;
            }
            let mut route = input.clone();
            route.project = route.project.or_else(|| req.project.clone());
            route.namespace = route.namespace.or_else(|| req.namespace.clone());
            route.workspace = route.workspace.or_else(|| req.workspace.clone());
            route.user_id = route.user_id.or_else(|| req.user_id.clone());
            route.agent = route.agent.or_else(|| req.agent.clone());
            route.updated_at = Some(now);
            let key = access_route_key(&route);
            let payload_json = serde_json::to_string(&route).context("serialize access route")?;
            conn.execute(
                r#"
                INSERT INTO access_routes (
                  route_key, project, namespace, workspace, user_id, provider, status, scope, updated_at, payload_json
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                ON CONFLICT(route_key) DO UPDATE SET
                  project = excluded.project,
                  namespace = excluded.namespace,
                  workspace = excluded.workspace,
                  user_id = excluded.user_id,
                  provider = excluded.provider,
                  status = excluded.status,
                  scope = excluded.scope,
                  updated_at = excluded.updated_at,
                  payload_json = excluded.payload_json
                "#,
                params![
                    key,
                    route.project,
                    route.namespace,
                    route.workspace,
                    route.user_id,
                    route.provider,
                    route.status,
                    route.scope,
                    now.to_rfc3339(),
                    payload_json,
                ],
            )
            .context("upsert access route")?;
            routes.push(route);
        }
        Ok(AccessRouteSyncResponse {
            upserted: routes.len(),
            total: self
                .list_access_routes(&AccessRouteListRequest {
                    project: req.project.clone(),
                    namespace: req.namespace.clone(),
                    workspace: req.workspace.clone(),
                    user_id: req.user_id.clone(),
                    provider: None,
                    query: None,
                    limit: Some(500),
                })?
                .total,
            routes,
        })
    }

    pub fn list_access_routes(
        &self,
        req: &AccessRouteListRequest,
    ) -> anyhow::Result<AccessRouteListResponse> {
        let conn = self.connect()?;
        let limit = req.limit.unwrap_or(100).clamp(1, 500);
        let query = req
            .query
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| format!("%{}%", value.to_ascii_lowercase()));
        let mut stmt = conn.prepare(
            r#"
            SELECT payload_json
            FROM access_routes
            WHERE (?1 IS NULL OR project = ?1)
              AND (?2 IS NULL OR namespace = ?2)
              AND (?3 IS NULL OR workspace = ?3)
              AND (?4 IS NULL OR user_id = ?4)
              AND (?5 IS NULL OR provider = ?5)
              AND (
                ?6 IS NULL
                OR lower(provider) LIKE ?6
                OR lower(status) LIKE ?6
                OR lower(scope) LIKE ?6
                OR lower(payload_json) LIKE ?6
              )
            ORDER BY updated_at DESC, provider ASC, scope ASC
            LIMIT ?7
            "#,
        )?;
        let routes = stmt
            .query_map(
                params![
                    req.project,
                    req.namespace,
                    req.workspace,
                    req.user_id,
                    req.provider,
                    query,
                    limit as i64,
                ],
                |row| row.get::<_, String>(0),
            )
            .context("query access routes")?
            .map(|row| {
                let payload = row?;
                serde_json::from_str::<AccessRouteRecord>(&payload).map_err(|err| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(err),
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .context("read access routes")?;
        Ok(AccessRouteListResponse {
            total: routes.len(),
            routes,
        })
    }

    pub fn upsert_token_savings(
        &self,
        req: &TokenSavingsSyncRequest,
    ) -> anyhow::Result<TokenSavingsSyncResponse> {
        let conn = self.connect()?;
        let now = chrono::Utc::now();
        let mut records = Vec::new();
        for input in &req.records {
            let mut record = input.clone();
            record.project = record.project.or_else(|| req.project.clone());
            record.namespace = record.namespace.or_else(|| req.namespace.clone());
            record.workspace = record.workspace.or_else(|| req.workspace.clone());
            record.user_id = record.user_id.or_else(|| req.user_id.clone());
            record.agent = record.agent.or_else(|| req.agent.clone());
            record.updated_at = Some(now);
            let payload_json =
                serde_json::to_string(&record).context("serialize token savings record")?;
            conn.execute(
                r#"
                INSERT INTO token_savings (
                  token_savings_id, project, namespace, workspace, user_id, agent,
                  operation, model_tier, intent, ts, tokens_saved, updated_at, payload_json
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                ON CONFLICT(token_savings_id) DO UPDATE SET
                  project = excluded.project,
                  namespace = excluded.namespace,
                  workspace = excluded.workspace,
                  user_id = excluded.user_id,
                  agent = excluded.agent,
                  operation = excluded.operation,
                  model_tier = excluded.model_tier,
                  intent = excluded.intent,
                  ts = excluded.ts,
                  tokens_saved = excluded.tokens_saved,
                  updated_at = excluded.updated_at,
                  payload_json = excluded.payload_json
                "#,
                params![
                    record.id.to_string(),
                    record.project,
                    record.namespace,
                    record.workspace,
                    record.user_id,
                    record.agent,
                    record.operation,
                    record.model_tier,
                    record.intent,
                    record.ts.to_rfc3339(),
                    record.tokens_saved as i64,
                    now.to_rfc3339(),
                    payload_json,
                ],
            )
            .context("upsert token savings")?;
            records.push(record);
        }
        Ok(TokenSavingsSyncResponse {
            upserted: records.len(),
            total: self
                .list_token_savings(&TokenSavingsListRequest {
                    project: req.project.clone(),
                    namespace: req.namespace.clone(),
                    workspace: req.workspace.clone(),
                    user_id: req.user_id.clone(),
                    agent: req.agent.clone(),
                    since: None,
                    limit: Some(500),
                })?
                .total,
            records,
        })
    }

    pub fn list_token_savings(
        &self,
        req: &TokenSavingsListRequest,
    ) -> anyhow::Result<TokenSavingsListResponse> {
        let conn = self.connect()?;
        let limit = req.limit.unwrap_or(100).clamp(1, 1000);
        let since = req.since.map(|value| value.to_rfc3339());
        let mut stmt = conn.prepare(
            r#"
            SELECT payload_json
            FROM token_savings
            WHERE (?1 IS NULL OR project = ?1)
              AND (?2 IS NULL OR namespace = ?2)
              AND (?3 IS NULL OR workspace = ?3)
              AND (?4 IS NULL OR user_id = ?4)
              AND (?5 IS NULL OR agent = ?5)
              AND (?6 IS NULL OR ts >= ?6)
            ORDER BY ts DESC
            LIMIT ?7
            "#,
        )?;
        let records = stmt
            .query_map(
                params![
                    req.project,
                    req.namespace,
                    req.workspace,
                    req.user_id,
                    req.agent,
                    since,
                    limit as i64,
                ],
                |row| row.get::<_, String>(0),
            )
            .context("query token savings")?
            .map(|row| {
                let payload = row?;
                serde_json::from_str::<TokenSavingsRecord>(&payload).map_err(|err| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(err),
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .context("read token savings")?;
        Ok(TokenSavingsListResponse {
            total: records.len(),
            measured_input_tokens: records
                .iter()
                .filter(|record| record.waste_kind.is_none())
                .map(|record| record.baseline_input_tokens)
                .sum(),
            measured_output_tokens: records
                .iter()
                .filter(|record| record.waste_kind.is_none())
                .map(|record| record.output_tokens)
                .sum(),
            measured_tokens_saved: records
                .iter()
                .filter(|record| record.waste_kind.is_none())
                .map(|record| record.tokens_saved)
                .sum(),
            source_reuse_events: records
                .iter()
                .filter(|record| record.operation == "source_read_avoided")
                .count(),
            source_reuse_tokens: records
                .iter()
                .filter(|record| record.operation == "source_read_avoided")
                .map(|record| record.tokens_saved)
                .sum(),
            wasted_events: records
                .iter()
                .filter(|record| record.wasted_tokens > 0)
                .count(),
            wasted_tokens: records.iter().map(|record| record.wasted_tokens).sum(),
            wasted_raw_reread_tokens: token_waste_for_kind(&records, "raw_source_reread"),
            wasted_giant_diff_tokens: token_waste_for_kind(&records, "giant_diff"),
            wasted_cache_exposure_tokens: token_waste_for_kind(&records, "repo_cache_exposure"),
            records,
        })
    }
}

fn token_waste_for_kind(records: &[TokenSavingsRecord], kind: &str) -> usize {
    records
        .iter()
        .filter(|record| record.waste_kind.as_deref() == Some(kind))
        .map(|record| record.wasted_tokens)
        .sum()
}

fn capability_key(record: &CapabilityRecord) -> String {
    [
        record.project.as_deref().unwrap_or(""),
        record.namespace.as_deref().unwrap_or(""),
        record.workspace.as_deref().unwrap_or(""),
        record.user_id.as_deref().unwrap_or(""),
        record.harness.as_str(),
        record.kind.as_str(),
        record.name.as_str(),
        record.source_path.as_str(),
    ]
    .join("\u{1f}")
}

fn access_route_key(record: &AccessRouteRecord) -> String {
    [
        record.project.as_deref().unwrap_or(""),
        record.namespace.as_deref().unwrap_or(""),
        record.workspace.as_deref().unwrap_or(""),
        record.user_id.as_deref().unwrap_or(""),
        record.provider.as_str(),
        record.id.as_str(),
        record.scope.as_str(),
    ]
    .join("\u{1f}")
}
