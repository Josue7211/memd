use super::*;

impl SqliteStore {
    pub fn fts_search(&self, query: &str, limit: usize) -> anyhow::Result<Vec<(Uuid, f64)>> {
        let conn = self.connect()?;
        let (w_content, w_tags) = fts5_weights();
        let mut stmt = conn
            .prepare(
                r#"
                SELECT item_id, -bm25(memory_items_fts, ?3, ?4) AS score
                FROM memory_items_fts
                WHERE memory_items_fts MATCH ?1
                ORDER BY bm25(memory_items_fts, ?3, ?4)
                LIMIT ?2
                "#,
            )
            .context("prepare fts search query")?;

        let rows = stmt
            .query_map(params![query, limit as i64, w_content, w_tags], |row| {
                let id_str: String = row.get(0)?;
                let score: f64 = row.get(1)?;
                Ok((id_str, score))
            })
            .context("execute fts search")?;

        let mut results = Vec::new();
        for row in rows {
            let (id_str, score) = row.context("read fts result row")?;
            if let Ok(id) = Uuid::parse_str(&id_str) {
                results.push((id, score));
            }
        }

        Ok(results)
    }

    /// Replace all stored chunk vectors for `memory_id` with the supplied
    /// `(chunk_idx, vec_bytes)` list. `project`/`namespace` mirror the
    /// MemoryItem scope so dense search can prefilter the same slice
    /// `snapshot_for_scope` uses. Callers pass every chunk they want
    /// persisted; prior rows for this id are deleted first so stale
    /// chunks from a longer previous content don't linger.
    pub fn replace_memory_vector_chunks(
        &self,
        memory_id: Uuid,
        project: Option<&str>,
        namespace: Option<&str>,
        embedding_model: &str,
        dim: usize,
        chunks: &[(i64, Vec<u8>)],
    ) -> anyhow::Result<()> {
        let mut conn = self.connect()?;
        let tx = conn.transaction().context("begin vector upsert tx")?;
        tx.execute(
            "DELETE FROM memory_vectors WHERE memory_id = ?1",
            params![memory_id.to_string()],
        )
        .context("delete prior chunks")?;
        let now = chrono::Utc::now().to_rfc3339();
        {
            let mut stmt = tx
                .prepare(
                    r#"
                    INSERT INTO memory_vectors (memory_id, chunk_idx, project, namespace, embedding_model, dim, vec, updated_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                    "#,
                )
                .context("prepare insert chunk")?;
            for (idx, bytes) in chunks {
                stmt.execute(params![
                    memory_id.to_string(),
                    *idx,
                    project,
                    namespace,
                    embedding_model,
                    dim as i64,
                    bytes,
                    now,
                ])
                .context("insert chunk")?;
            }
        }
        tx.execute(
            "UPDATE memory_items SET embedding_model = ?1 WHERE id = ?2",
            params![embedding_model, memory_id.to_string()],
        )
        .context("stamp memory item embedding model")?;
        tx.commit().context("commit vector upsert tx")?;
        Ok(())
    }

    /// Load every (id, raw-bytes) vector matching the given scope. Null
    /// `project` / `namespace` filters degrade to "match any" (mirrors
    /// `list_for_scope`). Returns raw bytes; caller converts to f32 slices.
    pub fn list_vectors_for_scope(
        &self,
        project: Option<&str>,
        namespace: Option<&str>,
        embedding_model: &str,
    ) -> anyhow::Result<Vec<(Uuid, Vec<u8>)>> {
        let conn = self.connect()?;
        let mut clauses: Vec<&str> = vec!["embedding_model = ?"];
        if project.is_some() {
            clauses.push("project = ?");
        }
        if namespace.is_some() {
            clauses.push("namespace = ?");
        }
        let where_clause = if clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", clauses.join(" AND "))
        };
        let sql = format!(
            "SELECT memory_id, vec FROM memory_vectors{} LIMIT 20000",
            where_clause
        );
        let mut stmt = conn.prepare(&sql).context("prepare list vectors")?;

        let mut bound: Vec<&dyn rusqlite::ToSql> = Vec::new();
        bound.push(&embedding_model);
        if let Some(p) = project.as_ref() {
            bound.push(p);
        }
        if let Some(n) = namespace.as_ref() {
            bound.push(n);
        }
        let rows = stmt
            .query_map(rusqlite::params_from_iter(bound), |row| {
                let id_str: String = row.get(0)?;
                let vec_bytes: Vec<u8> = row.get(1)?;
                Ok((id_str, vec_bytes))
            })
            .context("query memory vectors")?;
        let mut out = Vec::new();
        for row in rows {
            let (id_str, bytes) = row.context("read vector row")?;
            if let Ok(id) = Uuid::parse_str(&id_str) {
                out.push((id, bytes));
            }
        }
        Ok(out)
    }

    pub fn items_needing_reembed(
        &self,
        embedding_model: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<MemoryItem>> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT payload_json
            FROM memory_items
            WHERE embedding_model IS NULL OR embedding_model != ?1
            ORDER BY updated_at DESC
            LIMIT ?2
            "#,
        )?;
        let rows = stmt.query_map(params![embedding_model, limit as i64], |row| {
            row.get::<_, String>(0)
        })?;
        let mut items = Vec::new();
        for row in rows {
            let payload = row?;
            items.push(serde_json::from_str::<MemoryItem>(&payload)?);
        }
        Ok(items)
    }

    pub fn count(&self) -> anyhow::Result<usize> {
        let conn = self.connect()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM memory_items", [], |row| row.get(0))
            .context("count memory items")?;
        Ok(count as usize)
    }

    pub fn schema_version(&self) -> anyhow::Result<u32> {
        let conn = self.connect()?;
        let v: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .context("read user_version")?;
        Ok(v as u32)
    }

    // K2.5: scan the memory_events spine for per-entity recorded_at
    // monotonicity and produce a deterministic rolling payload hash.
    // Ordering: (entity_id, recorded_at ASC, id ASC) so any stale/reordered
    // row shows up as a monotonic violation against its entity predecessor.
    // The rolling hash chains the payload bytes across every row; it's a
    // stable fingerprint callers can compare across invocations to detect
    // in-place tampering, not a Merkle proof.
    pub fn verify_spine(&self) -> anyhow::Result<memd_schema::SpineVerifyResponse> {
        use sha2::{Digest, Sha256};

        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, entity_id, recorded_at, payload_json
                FROM memory_events
                ORDER BY entity_id ASC, recorded_at ASC, id ASC
                "#,
            )
            .context("prepare spine verify query")?;

        let mut rows = stmt.query([]).context("execute spine verify query")?;

        let mut scanned: u64 = 0;
        let mut violations: u64 = 0;
        let mut first_violation: Option<memd_schema::SpineViolation> = None;
        let mut hasher = Sha256::new();

        let mut prev: Option<(String, chrono::DateTime<chrono::Utc>, String)> = None;

        while let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let entity_id: String = row.get(1)?;
            let recorded_at_raw: String = row.get(2)?;
            let payload_json: String = row.get(3)?;

            let recorded_at = chrono::DateTime::parse_from_rfc3339(&recorded_at_raw)
                .with_context(|| format!("parse recorded_at for event {id}"))?
                .with_timezone(&chrono::Utc);

            hasher.update(entity_id.as_bytes());
            hasher.update([0u8]);
            hasher.update(recorded_at_raw.as_bytes());
            hasher.update([0u8]);
            hasher.update(payload_json.as_bytes());
            hasher.update([0xff]);
            scanned += 1;

            if let Some((prev_entity, prev_at, prev_id)) = &prev
                && prev_entity == &entity_id
                && recorded_at < *prev_at
            {
                violations += 1;
                if first_violation.is_none()
                    && let (Ok(earlier), Ok(later)) =
                        (Uuid::parse_str(prev_id), Uuid::parse_str(&id))
                    && let Ok(entity_uuid) = Uuid::parse_str(&entity_id)
                {
                    first_violation = Some(memd_schema::SpineViolation {
                        entity_id: entity_uuid,
                        earlier_event_id: earlier,
                        later_event_id: later,
                        earlier_recorded_at: *prev_at,
                        later_recorded_at: recorded_at,
                    });
                }
            }

            prev = Some((entity_id, recorded_at, id));
        }

        let digest = hasher.finalize();
        let rolling_sha256 = digest
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();

        Ok(memd_schema::SpineVerifyResponse {
            scanned,
            monotonic_violations: violations,
            first_violation,
            rolling_sha256,
        })
    }

    pub fn drain_expired(
        &self,
        project: Option<&str>,
        namespace: Option<&str>,
        max_items: usize,
    ) -> anyhow::Result<usize> {
        let mut conn = self.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .context("begin drain transaction")?;

        let expired_status = serde_json::to_string(&memd_schema::MemoryStatus::Expired).unwrap();
        let mut param_values: Vec<String> = vec![expired_status];
        let mut sql = String::from(
            "DELETE FROM memory_items WHERE id IN (SELECT id FROM memory_items WHERE status = ?1",
        );
        if let Some(project) = project {
            param_values.push(project.to_string());
            sql.push_str(&format!(" AND project = ?{}", param_values.len()));
        }
        if let Some(namespace) = namespace {
            param_values.push(namespace.to_string());
            sql.push_str(&format!(" AND namespace = ?{}", param_values.len()));
        }
        param_values.push(max_items.to_string());
        sql.push_str(&format!(" LIMIT ?{})", param_values.len()));

        let params: Vec<&dyn rusqlite::ToSql> = param_values
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();
        let deleted = tx
            .execute(&sql, params.as_slice())
            .context("drain expired items")?;
        tx.commit()?;
        Ok(deleted)
    }

    pub fn dismiss_items(&self, ids: &[Uuid]) -> anyhow::Result<usize> {
        if ids.is_empty() {
            return Ok(0);
        }
        let expired_status = serde_json::to_string(&memd_schema::MemoryStatus::Expired).unwrap();
        let mut conn = self.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .context("begin dismiss transaction")?;

        let mut dismissed = 0usize;
        for id in ids {
            let payload = tx.query_row(
                "SELECT payload_json FROM memory_items WHERE id = ?1",
                [id.to_string()],
                |row| row.get::<_, String>(0),
            );
            let Ok(payload) = payload else { continue };
            let mut item: MemoryItem =
                serde_json::from_str(&payload).context("deserialize dismiss target")?;
            if item.status == memd_schema::MemoryStatus::Expired {
                continue;
            }
            item.status = memd_schema::MemoryStatus::Expired;
            item.updated_at = chrono::Utc::now();
            let updated_payload =
                serde_json::to_string(&item).context("serialize dismissed item")?;
            let updated_status = &expired_status;
            tx.execute(
                "UPDATE memory_items SET status = ?1, updated_at = ?2, payload_json = json_set(?3, '$.version', version + 1), version = version + 1 WHERE id = ?4",
                params![updated_status, item.updated_at.to_rfc3339(), updated_payload, id.to_string()],
            )
            .context("dismiss inbox item")?;
            dismissed += 1;
        }
        tx.commit()?;
        Ok(dismissed)
    }

    pub fn send_hive_message(
        &self,
        request: &HiveMessageSendRequest,
    ) -> anyhow::Result<HiveMessagesResponse> {
        let message = HiveMessageRecord {
            id: Uuid::new_v4().to_string(),
            kind: request.kind.trim().to_string(),
            from_session: request.from_session.trim().to_string(),
            from_agent: request
                .from_agent
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            to_session: request.to_session.trim().to_string(),
            project: request.project.clone(),
            namespace: request.namespace.clone(),
            workspace: request.workspace.clone(),
            content: request.content.trim().to_string(),
            created_at: chrono::Utc::now(),
            acknowledged_at: None,
        };
        let payload_json =
            serde_json::to_string(&message).context("serialize hive message payload")?;

        let conn = self.connect()?;
        conn.execute(
            r#"
            INSERT INTO hive_messages (
              id, to_session, project, namespace, workspace, created_at, acknowledged_at, payload_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                &message.id,
                &message.to_session,
                &message.project,
                &message.namespace,
                &message.workspace,
                message.created_at.to_rfc3339(),
                Option::<String>::None,
                payload_json,
            ],
        )
        .context("insert hive message")?;

        Ok(HiveMessagesResponse {
            messages: vec![message],
        })
    }

    pub fn hive_inbox(
        &self,
        request: &HiveMessageInboxRequest,
    ) -> anyhow::Result<HiveMessagesResponse> {
        let include_acknowledged = request.include_acknowledged.unwrap_or(false);
        let limit = request.limit.unwrap_or(64).clamp(1, 512) as i64;

        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT payload_json
                FROM hive_messages
                WHERE to_session = ?1
                  AND (?2 IS NULL OR project = ?2)
                  AND (?3 IS NULL OR namespace = ?3)
                  AND (?4 IS NULL OR workspace = ?4)
                  AND (?5 = 1 OR acknowledged_at IS NULL)
                ORDER BY created_at DESC
                LIMIT ?6
                "#,
            )
            .context("prepare hive inbox query")?;

        let rows = stmt
            .query_map(
                params![
                    request.session.trim(),
                    request.project.clone(),
                    request.namespace.clone(),
                    request.workspace.clone(),
                    if include_acknowledged { 1 } else { 0 },
                    limit,
                ],
                |row| row.get::<_, String>(0),
            )
            .context("query hive inbox")?;

        let mut messages = Vec::new();
        for row in rows {
            let payload = row.context("read hive inbox row")?;
            messages.push(
                serde_json::from_str::<HiveMessageRecord>(&payload)
                    .context("deserialize hive inbox payload")?,
            );
        }

        Ok(HiveMessagesResponse { messages })
    }

    pub fn ack_hive_message(
        &self,
        request: &HiveMessageAckRequest,
    ) -> anyhow::Result<HiveMessagesResponse> {
        let conn = self.connect()?;
        let payload = conn
            .query_row(
                "SELECT payload_json FROM hive_messages WHERE id = ?1 AND to_session = ?2",
                params![request.id.trim(), request.session.trim()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("fetch hive message for ack")?;

        let Some(payload) = payload else {
            return Ok(HiveMessagesResponse {
                messages: Vec::new(),
            });
        };

        let mut message: HiveMessageRecord =
            serde_json::from_str(&payload).context("deserialize hive message for ack")?;
        message.acknowledged_at = Some(chrono::Utc::now());
        let updated_payload =
            serde_json::to_string(&message).context("serialize acked hive message")?;

        conn.execute(
            "UPDATE hive_messages SET acknowledged_at = ?2, payload_json = ?3 WHERE id = ?1",
            params![
                &message.id,
                message
                    .acknowledged_at
                    .as_ref()
                    .map(chrono::DateTime::to_rfc3339),
                updated_payload,
            ],
        )
        .context("ack hive message")?;

        Ok(HiveMessagesResponse {
            messages: vec![message],
        })
    }
}
