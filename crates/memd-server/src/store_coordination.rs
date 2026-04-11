use super::*;
use crate::store_hive::annotate_hive_relationships;

impl SqliteStore {

    pub fn hive_board(&self, request: &HiveBoardRequest) -> anyhow::Result<HiveBoardResponse> {
        let sessions = self
            .hive_sessions(&HiveSessionsRequest {
                session: None,
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                workspace: request.workspace.clone(),
                repo_root: None,
                worktree_root: None,
                branch: None,
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(256),
            })?
            .sessions;
        let tasks = self
            .hive_tasks(&HiveTasksRequest {
                session: None,
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                workspace: request.workspace.clone(),
                active_only: Some(true),
                limit: Some(256),
            })?
            .tasks;
        let receipts = self
            .hive_coordination_receipts(&HiveCoordinationReceiptsRequest {
                session: None,
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                workspace: request.workspace.clone(),
                limit: Some(256),
            })?
            .receipts;

        let visible_sessions = sessions
            .iter()
            .filter(|session| !is_low_signal_hive_board_session(session, &tasks))
            .cloned()
            .collect::<Vec<_>>();
        let queen_session = visible_sessions
            .iter()
            .find(|session| {
                matches!(
                    session.role.as_deref().or(session.hive_role.as_deref()),
                    Some("queen" | "orchestrator" | "memory-control-plane")
                ) || matches!(
                    session.authority.as_deref(),
                    Some("coordinator" | "canonical")
                )
            })
            .map(|session| session.session.clone());

        let visible_sessions = annotate_hive_relationships(visible_sessions);
        let active_session_ids = visible_sessions
            .iter()
            .filter(|session| matches!(session.status.as_str(), "active" | "live"))
            .map(|session| session.session.clone())
            .collect::<std::collections::HashSet<_>>();
        let active_bees = visible_sessions
            .iter()
            .filter(|session| matches!(session.status.as_str(), "active" | "live"))
            .cloned()
            .collect::<Vec<_>>();
        let stale_bees = visible_sessions
            .iter()
            .filter(|session| !matches!(session.status.as_str(), "active" | "live"))
            .map(|session| session.session.clone())
            .collect::<Vec<_>>();
        let review_queue = tasks
            .iter()
            .filter(|task| task.review_requested || task.coordination_mode == "shared_review")
            .map(|task| {
                format!(
                    "{} -> {}",
                    task.task_id,
                    task.session.as_deref().unwrap_or("unassigned")
                )
            })
            .collect::<Vec<_>>();
        let lane_faults = receipts
            .iter()
            .filter(|receipt| is_active_hive_board_receipt(receipt, &active_session_ids))
            .filter(|receipt| receipt.kind.starts_with("lane_"))
            .map(|receipt| receipt.summary.clone())
            .collect::<Vec<_>>();
        let overlap_risks = active_bees
            .iter()
            .filter(|bee| matches!(bee.relationship_state.as_deref(), Some("conflict" | "near")))
            .map(|bee| {
                format!(
                    "{} {}",
                    bee.relationship_state.as_deref().unwrap_or("near"),
                    bee.relationship_reason.as_deref().unwrap_or("peer overlap")
                )
            })
            .chain(
                receipts
                    .iter()
                    .filter(|receipt| is_hive_overlap_receipt(receipt))
                    .map(|receipt| receipt.summary.clone()),
            )
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let blocked_bees = active_bees
            .iter()
            .filter(|bee| bee.relationship_state.as_deref() == Some("blocked"))
            .map(|bee| {
                format!(
                    "{} waiting on {}",
                    bee.worker_name.as_deref().unwrap_or(&bee.session),
                    bee.relationship_peer.as_deref().unwrap_or("peer")
                )
            })
            .chain(
                receipts
                    .iter()
                    .filter(|receipt| is_active_hive_board_receipt(receipt, &active_session_ids))
                    .filter(|receipt| receipt.kind == "queen_deny" || receipt.kind.starts_with("lane_"))
                    .map(|receipt| receipt.summary.clone()),
            )
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let mut recommended_actions = Vec::new();
        for session in &stale_bees {
            recommended_actions.push(format!("retire {}", session));
        }
        for bee in &active_bees {
            if let Some(action) = bee.suggested_action.as_deref() {
                recommended_actions.push(format!(
                    "{} {}",
                    action,
                    bee.relationship_reason.as_deref().unwrap_or("peer coordination")
                ));
            }
        }
        for receipt in receipts
            .iter()
            .filter(|receipt| is_active_hive_board_receipt(receipt, &active_session_ids))
            .filter(|receipt| receipt.kind == "queen_deny")
        {
            recommended_actions.push(format!("reroute {}", receipt.summary));
        }

        Ok(HiveBoardResponse {
            queen_session,
            active_bees,
            blocked_bees,
            stale_bees,
            review_queue,
            overlap_risks,
            lane_faults,
            recommended_actions,
        })
    }

    pub fn hive_roster(
        &self,
        request: &memd_schema::HiveRosterRequest,
    ) -> anyhow::Result<HiveRosterResponse> {
        let sessions = self
            .hive_sessions(&HiveSessionsRequest {
                session: None,
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                workspace: request.workspace.clone(),
                repo_root: None,
                worktree_root: None,
                branch: None,
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(256),
            })?
            .sessions;
        let tasks = self
            .hive_tasks(&HiveTasksRequest {
                session: None,
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                workspace: request.workspace.clone(),
                active_only: Some(true),
                limit: Some(256),
            })?
            .tasks;
        let visible_sessions = annotate_hive_relationships(
            sessions
                .iter()
                .filter(|session| !is_low_signal_hive_board_session(session, &tasks))
                .cloned()
                .collect::<Vec<_>>(),
        );
        let queen_session = visible_sessions
            .iter()
            .find(|session| {
                matches!(
                    session.role.as_deref().or(session.hive_role.as_deref()),
                    Some("queen" | "orchestrator" | "memory-control-plane")
                ) || matches!(
                    session.authority.as_deref(),
                    Some("coordinator" | "canonical")
                )
            })
            .map(|session| session.session.clone());

        Ok(HiveRosterResponse {
            project: request
                .project
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            namespace: request
                .namespace
                .clone()
                .unwrap_or_else(|| "default".to_string()),
            queen_session,
            bees: visible_sessions,
        })
    }

    pub fn hive_follow(
        &self,
        request: &memd_schema::HiveFollowRequest,
    ) -> anyhow::Result<memd_schema::HiveFollowResponse> {
        let sessions = self
            .hive_sessions(&HiveSessionsRequest {
                session: None,
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                workspace: request.workspace.clone(),
                repo_root: None,
                worktree_root: None,
                branch: None,
                hive_system: None,
                hive_role: None,
                host: None,
                hive_group: None,
                active_only: Some(false),
                limit: Some(256),
            })?
            .sessions;
        let sessions = annotate_hive_relationships(sessions);
        let target = sessions
            .iter()
            .find(|session| session.session == request.session)
            .cloned()
            .context("hive follow session not found")?;
        let inbox = self.hive_coordination_inbox(&HiveCoordinationInboxRequest {
            session: request.session.clone(),
            project: request.project.clone(),
            namespace: request.namespace.clone(),
            workspace: request.workspace.clone(),
            limit: Some(32),
        })?;
        let recent_receipts = self
            .hive_coordination_receipts(&HiveCoordinationReceiptsRequest {
                session: None,
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                workspace: request.workspace.clone(),
                limit: Some(64),
            })?
            .receipts
            .into_iter()
            .filter(|receipt| {
                receipt.actor_session == request.session
                    || receipt.target_session.as_deref() == Some(request.session.as_str())
                    || receipt.task_id.as_deref().is_some_and(|task_id| {
                        inbox.owned_tasks.iter().any(|task| task.task_id == task_id)
                    })
            })
            .take(8)
            .collect::<Vec<_>>();

        let overlap_risk = request
            .current_session
            .as_deref()
            .and_then(|current_session| {
                let current = sessions
                    .iter()
                    .find(|session| session.session == current_session)?;
                hive_follow_overlap_risk(current, &target)
            });
        let recommended_action = target.suggested_action.clone().unwrap_or_else(|| {
            if overlap_risk.is_some() {
                "coordinate_now".to_string()
            } else if !inbox.review_tasks.is_empty()
                || !inbox.help_tasks.is_empty()
                || !inbox.messages.is_empty()
            {
                "watch_and_coordinate".to_string()
            } else {
                "safe_to_continue".to_string()
            }
        });

        Ok(memd_schema::HiveFollowResponse {
            current_session: request.current_session.clone(),
            target: target.clone(),
            work_summary: target
                .working
                .clone()
                .or_else(|| target.topic_claim.clone())
                .or_else(|| target.focus.clone())
                .unwrap_or_else(|| "none".to_string()),
            touch_points: if target.touches.is_empty() {
                target.scope_claims.clone()
            } else {
                target.touches.clone()
            },
            next_action: target.next_action.clone(),
            messages: inbox.messages,
            owned_tasks: inbox.owned_tasks,
            help_tasks: inbox.help_tasks,
            review_tasks: inbox.review_tasks,
            recent_receipts,
            overlap_risk,
            recommended_action,
        })
    }

    pub fn retire_hive_session(
        &self,
        request: &HiveSessionRetireRequest,
    ) -> anyhow::Result<HiveSessionRetireResponse> {
        let session = request.session.trim();
        anyhow::ensure!(!session.is_empty(), "session must not be empty");

        let project = request
            .project
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let namespace = request
            .namespace
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let workspace = request
            .workspace
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let repo_root = request
            .repo_root
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let worktree_root = request
            .worktree_root
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let branch = request
            .branch
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let agent = request
            .agent
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let effective_agent = request
            .effective_agent
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let hive_system = request
            .hive_system
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let hive_role = request
            .hive_role
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let host = request
            .host
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());

        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            r#"
            SELECT session_key, payload_json
            FROM hive_sessions
            WHERE session = ?1
              AND (?2 IS NULL OR project = ?2)
              AND (?3 IS NULL OR namespace = ?3)
              AND (?4 IS NULL OR workspace = ?4)
              AND (?5 IS NULL OR repo_root = ?5)
              AND (?6 IS NULL OR worktree_root = ?6)
              AND (?7 IS NULL OR branch = ?7)
              AND (?8 IS NULL OR json_extract(payload_json, '$.agent') = ?8)
              AND (?9 IS NULL OR json_extract(payload_json, '$.effective_agent') = ?9)
              AND (?10 IS NULL OR hive_system = ?10)
              AND (?11 IS NULL OR hive_role = ?11)
              AND (?12 IS NULL OR host = ?12)
            "#,
        )?;
        let rows = stmt.query_map(
            params![
                session,
                project,
                namespace,
                workspace,
                repo_root,
                worktree_root,
                branch,
                agent,
                effective_agent,
                hive_system,
                hive_role,
                host,
            ],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )?;

        let mut targets = Vec::new();
        for row in rows {
            let (session_key, payload) = row.context("read hive session retire row")?;
            targets.push((
                session_key,
                serde_json::from_str::<HiveSessionRecord>(&payload)
                    .context("deserialize retired hive session payload")?,
            ));
        }
        if targets.is_empty() {
            return Ok(HiveSessionRetireResponse {
                retired: 0,
                sessions: Vec::new(),
            });
        }

        let mut all_targets = std::collections::BTreeMap::new();
        for (session_key, record) in targets {
            all_targets.insert(session_key, record);
        }

        let scope_keys = all_targets
            .values()
            .map(|record| {
                (
                    record.session.clone(),
                    record.project.clone(),
                    record.namespace.clone(),
                    record.workspace.clone(),
                )
            })
            .collect::<std::collections::BTreeSet<_>>();

        let conn = self.connect()?;
        for (scope_session, scope_project, scope_namespace, scope_workspace) in scope_keys {
            let mut sibling_stmt = conn.prepare(
                r#"
                SELECT session_key, payload_json
                FROM hive_sessions
                WHERE session = ?1
                  AND ((project IS NULL AND ?2 IS NULL) OR project = ?2)
                  AND ((namespace IS NULL AND ?3 IS NULL) OR namespace = ?3)
                  AND ((workspace IS NULL AND ?4 IS NULL) OR workspace = ?4)
                "#,
            )?;
            let sibling_rows = sibling_stmt.query_map(
                params![
                    scope_session,
                    scope_project,
                    scope_namespace,
                    scope_workspace,
                ],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )?;

            for row in sibling_rows {
                let (session_key, payload) = row.context("read sibling hive session retire row")?;
                all_targets.entry(session_key).or_insert(
                    serde_json::from_str::<HiveSessionRecord>(&payload)
                        .context("deserialize sibling retired hive session payload")?,
                );
            }
        }

        let mut conn = self.connect()?;
        let tx = conn
            .transaction()
            .context("begin hive session retire transaction")?;
        for session_key in all_targets.keys() {
            tx.execute(
                "DELETE FROM hive_session_groups WHERE session_key = ?1",
                params![session_key],
            )?;
            tx.execute(
                "DELETE FROM hive_sessions WHERE session_key = ?1",
                params![session_key],
            )?;
        }
        tx.commit()
            .context("commit hive session retire transaction")?;

        Ok(HiveSessionRetireResponse {
            retired: all_targets.len(),
            sessions: all_targets.into_values().collect(),
        })
    }

    pub fn upsert_hive_task(
        &self,
        request: &HiveTaskUpsertRequest,
    ) -> anyhow::Result<HiveTasksResponse> {
        let conn = self.connect()?;
        let existing = conn
            .query_row(
                "SELECT payload_json FROM hive_tasks WHERE task_id = ?1",
                params![request.task_id.trim()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("fetch existing hive task")?;

        let now = chrono::Utc::now();
        let mut task = if let Some(payload) = existing {
            let mut task: HiveTaskRecord =
                serde_json::from_str(&payload).context("deserialize existing hive task")?;
            task.title = request.title.trim().to_string();
            task.description = request
                .description
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            if let Some(mode) = request.coordination_mode.as_deref() {
                let trimmed = mode.trim();
                if !trimmed.is_empty() {
                    task.coordination_mode = trimmed.to_string();
                }
            }
            if let Some(status) = request.status.as_deref() {
                let trimmed = status.trim();
                if !trimmed.is_empty() {
                    task.status = trimmed.to_string();
                }
            }
            task.session = request.session.clone().or(task.session);
            task.agent = request.agent.clone().or(task.agent);
            task.effective_agent = request.effective_agent.clone().or(task.effective_agent);
            task.project = request.project.clone().or(task.project);
            task.namespace = request.namespace.clone().or(task.namespace);
            task.workspace = request.workspace.clone().or(task.workspace);
            if !request.claim_scopes.is_empty() {
                task.claim_scopes = request
                    .claim_scopes
                    .iter()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
                    .collect();
            }
            if let Some(value) = request.help_requested {
                task.help_requested = value;
            }
            if let Some(value) = request.review_requested {
                task.review_requested = value;
            }
            task.updated_at = now;
            task
        } else {
            HiveTaskRecord {
                task_id: request.task_id.trim().to_string(),
                title: request.title.trim().to_string(),
                description: request
                    .description
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string),
                status: request
                    .status
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or("active")
                    .to_string(),
                coordination_mode: request
                    .coordination_mode
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or("exclusive_write")
                    .to_string(),
                session: request.session.clone(),
                agent: request.agent.clone(),
                effective_agent: request.effective_agent.clone(),
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                workspace: request.workspace.clone(),
                claim_scopes: request
                    .claim_scopes
                    .iter()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty())
                    .collect(),
                help_requested: request.help_requested.unwrap_or(false),
                review_requested: request.review_requested.unwrap_or(false),
                created_at: now,
                updated_at: now,
            }
        };

        if task.status.trim().is_empty() {
            task.status = "active".to_string();
        }

        let payload_json = serde_json::to_string(&task).context("serialize hive task")?;
        conn.execute(
            r#"
            INSERT INTO hive_tasks (task_id, session, project, namespace, workspace, status, updated_at, payload_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(task_id) DO UPDATE SET
              session = excluded.session,
              project = excluded.project,
              namespace = excluded.namespace,
              workspace = excluded.workspace,
              status = excluded.status,
              updated_at = excluded.updated_at,
              payload_json = excluded.payload_json
            "#,
            params![
                task.task_id.as_str(),
                &task.session,
                &task.project,
                &task.namespace,
                &task.workspace,
                task.status.as_str(),
                task.updated_at.to_rfc3339(),
                payload_json,
            ],
        )
        .context("upsert hive task")?;

        Ok(HiveTasksResponse { tasks: vec![task] })
    }

    pub fn assign_hive_task(
        &self,
        request: &HiveTaskAssignRequest,
    ) -> anyhow::Result<HiveTasksResponse> {
        let conn = self.connect()?;
        let payload = conn
            .query_row(
                "SELECT payload_json FROM hive_tasks WHERE task_id = ?1",
                params![request.task_id.trim()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("fetch hive task for assignment")?;
        let Some(payload) = payload else {
            return Ok(HiveTasksResponse { tasks: Vec::new() });
        };

        let mut task: HiveTaskRecord =
            serde_json::from_str(&payload).context("deserialize hive task for assignment")?;
        if let Some(from_session) = request
            .from_session
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            if task.session.as_deref() != Some(from_session) {
                anyhow::bail!("task '{}' is not owned by {}", task.task_id, from_session);
            }
        }
        task.session = Some(request.to_session.trim().to_string());
        task.agent = request.to_agent.clone();
        task.effective_agent = request.to_effective_agent.clone();
        task.status = "assigned".to_string();
        task.updated_at = chrono::Utc::now();
        let payload_json = serde_json::to_string(&task).context("serialize assigned hive task")?;
        conn.execute(
            r#"
            UPDATE hive_tasks
            SET session = ?2, status = ?3, updated_at = ?4, payload_json = ?5
            WHERE task_id = ?1
            "#,
            params![
                task.task_id.as_str(),
                &task.session,
                task.status.as_str(),
                task.updated_at.to_rfc3339(),
                payload_json,
            ],
        )
        .context("assign hive task")?;
        Ok(HiveTasksResponse { tasks: vec![task] })
    }

    pub fn hive_tasks(&self, request: &HiveTasksRequest) -> anyhow::Result<HiveTasksResponse> {
        let limit = request.limit.unwrap_or(128).clamp(1, 1024) as i64;
        let active_only = request.active_only.unwrap_or(true);
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT payload_json
                FROM hive_tasks
                WHERE (?1 IS NULL OR session = ?1)
                  AND (?2 IS NULL OR project = ?2)
                  AND (?3 IS NULL OR namespace = ?3)
                  AND (?4 IS NULL OR workspace = ?4)
                  AND (?5 = 0 OR status NOT IN ('done', 'closed'))
                ORDER BY updated_at DESC
                LIMIT ?6
                "#,
            )
            .context("prepare hive tasks query")?;
        let rows = stmt
            .query_map(
                params![
                    request.session.clone(),
                    request.project.clone(),
                    request.namespace.clone(),
                    request.workspace.clone(),
                    if active_only { 1 } else { 0 },
                    limit,
                ],
                |row| row.get::<_, String>(0),
            )
            .context("query hive tasks")?;

        let mut tasks = Vec::new();
        for row in rows {
            let payload = row.context("read hive task row")?;
            tasks.push(
                serde_json::from_str::<HiveTaskRecord>(&payload)
                    .context("deserialize hive task payload")?,
            );
        }
        Ok(HiveTasksResponse { tasks })
    }

    pub fn hive_coordination_inbox(
        &self,
        request: &HiveCoordinationInboxRequest,
    ) -> anyhow::Result<HiveCoordinationInboxResponse> {
        let messages = self
            .hive_inbox(&HiveMessageInboxRequest {
                session: request.session.clone(),
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                workspace: request.workspace.clone(),
                include_acknowledged: Some(false),
                limit: request.limit,
            })?
            .messages;

        let tasks = self
            .hive_tasks(&HiveTasksRequest {
                session: None,
                project: request.project.clone(),
                namespace: request.namespace.clone(),
                workspace: request.workspace.clone(),
                active_only: Some(true),
                limit: request.limit,
            })?
            .tasks;

        let mut owned_tasks = Vec::new();
        let mut help_tasks = Vec::new();
        let mut review_tasks = Vec::new();
        for task in tasks {
            if task.session.as_deref() == Some(request.session.as_str()) {
                owned_tasks.push(task.clone());
            }
            if task.help_requested {
                help_tasks.push(task.clone());
            }
            if task.review_requested {
                review_tasks.push(task);
            }
        }

        Ok(HiveCoordinationInboxResponse {
            messages,
            owned_tasks,
            help_tasks,
            review_tasks,
        })
    }

    pub fn record_hive_coordination_receipt(
        &self,
        request: &HiveCoordinationReceiptRequest,
    ) -> anyhow::Result<HiveCoordinationReceiptsResponse> {
        let receipt = HiveCoordinationReceiptRecord {
            id: Uuid::new_v4().to_string(),
            kind: request.kind.trim().to_string(),
            actor_session: request.actor_session.trim().to_string(),
            actor_agent: request.actor_agent.clone(),
            target_session: request.target_session.clone(),
            task_id: request.task_id.clone(),
            scope: request.scope.clone(),
            project: request.project.clone(),
            namespace: request.namespace.clone(),
            workspace: request.workspace.clone(),
            summary: request.summary.trim().to_string(),
            created_at: chrono::Utc::now(),
        };
        let payload_json =
            serde_json::to_string(&receipt).context("serialize coordination receipt")?;
        let conn = self.connect()?;
        conn.execute(
            r#"
            INSERT INTO hive_coordination_receipts (id, actor_session, project, namespace, workspace, created_at, payload_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                receipt.id.as_str(),
                receipt.actor_session.as_str(),
                &receipt.project,
                &receipt.namespace,
                &receipt.workspace,
                receipt.created_at.to_rfc3339(),
                payload_json,
            ],
        )
        .context("insert coordination receipt")?;
        Ok(HiveCoordinationReceiptsResponse {
            receipts: vec![receipt],
        })
    }

    pub fn hive_coordination_receipts(
        &self,
        request: &HiveCoordinationReceiptsRequest,
    ) -> anyhow::Result<HiveCoordinationReceiptsResponse> {
        let limit = request.limit.unwrap_or(128).clamp(1, 1024) as i64;
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT payload_json
                FROM hive_coordination_receipts
                WHERE (?1 IS NULL OR actor_session = ?1)
                  AND (?2 IS NULL OR project = ?2)
                  AND (?3 IS NULL OR namespace = ?3)
                  AND (?4 IS NULL OR workspace = ?4)
                ORDER BY created_at DESC
                LIMIT ?5
                "#,
            )
            .context("prepare coordination receipts query")?;
        let rows = stmt
            .query_map(
                params![
                    request.session.clone(),
                    request.project.clone(),
                    request.namespace.clone(),
                    request.workspace.clone(),
                    limit,
                ],
                |row| row.get::<_, String>(0),
            )
            .context("query coordination receipts")?;
        let mut receipts = Vec::new();
        for row in rows {
            let payload = row.context("read coordination receipt row")?;
            receipts.push(
                serde_json::from_str::<HiveCoordinationReceiptRecord>(&payload)
                    .context("deserialize coordination receipt payload")?,
            );
        }
        Ok(HiveCoordinationReceiptsResponse { receipts })
    }

}
