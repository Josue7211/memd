use super::*;

impl SqliteStore {
    pub fn acquire_hive_claim(
        &self,
        request: &HiveClaimAcquireRequest,
    ) -> anyhow::Result<HiveClaimsResponse> {
        self.prune_expired_hive_claims()?;

        let expires_at =
            chrono::Utc::now() + chrono::TimeDelta::seconds(request.ttl_seconds.max(1) as i64);
        let claim = HiveClaimRecord {
            scope: request.scope.trim().to_string(),
            session: request.session.trim().to_string(),
            tab_id: request.tab_id.clone(),
            agent: request.agent.clone(),
            effective_agent: request.effective_agent.clone(),
            project: request.project.clone(),
            namespace: request.namespace.clone(),
            workspace: request.workspace.clone(),
            host: request.host.clone(),
            pid: request.pid,
            acquired_at: chrono::Utc::now(),
            expires_at,
        };

        let conn = self.connect()?;
        let existing = conn
            .query_row(
                "SELECT payload_json FROM hive_claims WHERE scope = ?1",
                params![claim.scope.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("fetch existing hive claim")?;
        if let Some(payload) = existing {
            let existing_claim: HiveClaimRecord =
                serde_json::from_str(&payload).context("deserialize existing hive claim")?;
            if existing_claim.session != claim.session
                && existing_claim.expires_at > chrono::Utc::now()
            {
                anyhow::bail!(
                    "scope '{}' already claimed by {}",
                    claim.scope,
                    existing_claim
                        .effective_agent
                        .as_deref()
                        .unwrap_or(existing_claim.session.as_str())
                );
            }
        }

        let payload_json = serde_json::to_string(&claim).context("serialize hive claim")?;
        conn.execute(
            r#"
            INSERT INTO hive_claims (scope, session, project, namespace, workspace, expires_at, payload_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(scope) DO UPDATE SET
              session = excluded.session,
              project = excluded.project,
              namespace = excluded.namespace,
              workspace = excluded.workspace,
              expires_at = excluded.expires_at,
              payload_json = excluded.payload_json
            "#,
            params![
                claim.scope.as_str(),
                claim.session.as_str(),
                &claim.project,
                &claim.namespace,
                &claim.workspace,
                claim.expires_at.to_rfc3339(),
                payload_json,
            ],
        )
        .context("upsert hive claim")?;

        Ok(HiveClaimsResponse {
            claims: vec![claim],
        })
    }

    pub fn release_hive_claim(
        &self,
        request: &HiveClaimReleaseRequest,
    ) -> anyhow::Result<HiveClaimsResponse> {
        let conn = self.connect()?;
        let payload = conn
            .query_row(
                "SELECT payload_json FROM hive_claims WHERE scope = ?1 AND session = ?2",
                params![request.scope.trim(), request.session.trim()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("fetch hive claim for release")?;
        let Some(payload) = payload else {
            return Ok(HiveClaimsResponse { claims: Vec::new() });
        };
        let claim: HiveClaimRecord =
            serde_json::from_str(&payload).context("deserialize hive claim for release")?;
        conn.execute(
            "DELETE FROM hive_claims WHERE scope = ?1 AND session = ?2",
            params![request.scope.trim(), request.session.trim()],
        )
        .context("release hive claim")?;
        Ok(HiveClaimsResponse {
            claims: vec![claim],
        })
    }

    pub fn transfer_hive_claim(
        &self,
        request: &HiveClaimTransferRequest,
    ) -> anyhow::Result<HiveClaimsResponse> {
        self.prune_expired_hive_claims()?;

        let conn = self.connect()?;
        let payload = conn
            .query_row(
                "SELECT payload_json FROM hive_claims WHERE scope = ?1 AND session = ?2",
                params![request.scope.trim(), request.from_session.trim()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("fetch hive claim for transfer")?;
        let Some(payload) = payload else {
            return Ok(HiveClaimsResponse { claims: Vec::new() });
        };

        let mut claim: HiveClaimRecord =
            serde_json::from_str(&payload).context("deserialize hive claim for transfer")?;
        claim.session = request.to_session.trim().to_string();
        claim.tab_id = request.to_tab_id.clone();
        claim.agent = request.to_agent.clone();
        claim.effective_agent = request.to_effective_agent.clone();
        let updated_payload =
            serde_json::to_string(&claim).context("serialize transferred hive claim")?;
        conn.execute(
            r#"
            UPDATE hive_claims
            SET session = ?2, payload_json = ?3
            WHERE scope = ?1 AND session = ?4
            "#,
            params![
                request.scope.trim(),
                claim.session.as_str(),
                updated_payload,
                request.from_session.trim(),
            ],
        )
        .context("transfer hive claim")?;
        Ok(HiveClaimsResponse {
            claims: vec![claim],
        })
    }

    pub fn recover_hive_claim(
        &self,
        request: &HiveClaimRecoverRequest,
    ) -> anyhow::Result<HiveClaimsResponse> {
        self.prune_expired_hive_claims()?;

        let conn = self.connect()?;
        let payload = conn
            .query_row(
                "SELECT payload_json FROM hive_claims WHERE scope = ?1 AND session = ?2",
                params![request.scope.trim(), request.from_session.trim()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("fetch hive claim for recovery")?;
        let Some(payload) = payload else {
            return Ok(HiveClaimsResponse { claims: Vec::new() });
        };

        let mut claim: HiveClaimRecord =
            serde_json::from_str(&payload).context("deserialize hive claim for recovery")?;

        if let Some(to_session) = request
            .to_session
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            claim.session = to_session.to_string();
            claim.tab_id = request.to_tab_id.clone();
            claim.agent = request.to_agent.clone();
            claim.effective_agent = request.to_effective_agent.clone();
            let updated_payload =
                serde_json::to_string(&claim).context("serialize recovered hive claim")?;
            conn.execute(
                r#"
                UPDATE hive_claims
                SET session = ?2, payload_json = ?3
                WHERE scope = ?1 AND session = ?4
                "#,
                params![
                    request.scope.trim(),
                    claim.session.as_str(),
                    updated_payload,
                    request.from_session.trim(),
                ],
            )
            .context("recover hive claim into new owner")?;
            Ok(HiveClaimsResponse {
                claims: vec![claim],
            })
        } else {
            conn.execute(
                "DELETE FROM hive_claims WHERE scope = ?1 AND session = ?2",
                params![request.scope.trim(), request.from_session.trim()],
            )
            .context("delete hive claim during recovery")?;
            Ok(HiveClaimsResponse {
                claims: vec![claim],
            })
        }
    }

    pub fn hive_claims(&self, request: &HiveClaimsRequest) -> anyhow::Result<HiveClaimsResponse> {
        self.prune_expired_hive_claims()?;

        let limit = request.limit.unwrap_or(256).clamp(1, 1024) as i64;
        let active_only = request.active_only.unwrap_or(true);
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT payload_json
                FROM hive_claims
                WHERE (?1 IS NULL OR session = ?1)
                  AND (?2 IS NULL OR project = ?2)
                  AND (?3 IS NULL OR namespace = ?3)
                  AND (?4 IS NULL OR workspace = ?4)
                  AND (?5 = 0 OR expires_at > ?6)
                ORDER BY expires_at DESC
                LIMIT ?7
                "#,
            )
            .context("prepare hive claims query")?;
        let now = chrono::Utc::now().to_rfc3339();
        let rows = stmt
            .query_map(
                params![
                    request.session.clone(),
                    request.project.clone(),
                    request.namespace.clone(),
                    request.workspace.clone(),
                    if active_only { 1 } else { 0 },
                    now,
                    limit,
                ],
                |row| row.get::<_, String>(0),
            )
            .context("query hive claims")?;

        let mut claims = Vec::new();
        for row in rows {
            let payload = row.context("read hive claim row")?;
            claims.push(
                serde_json::from_str::<HiveClaimRecord>(&payload)
                    .context("deserialize hive claim payload")?,
            );
        }
        Ok(HiveClaimsResponse { claims })
    }

    pub(crate) fn prune_expired_dev_server_leases(&self) -> anyhow::Result<()> {
        let conn = self.connect()?;
        conn.execute(
            "DELETE FROM dev_server_leases WHERE expires_at <= ?1",
            params![chrono::Utc::now().to_rfc3339()],
        )
        .context("prune expired dev server leases")?;
        Ok(())
    }

    pub fn acquire_dev_server_lease(
        &self,
        request: &DevServerLeaseAcquireRequest,
    ) -> anyhow::Result<DevServerLeasesResponse> {
        self.prune_expired_dev_server_leases()?;
        let now = chrono::Utc::now();
        let scope = request.scope.trim().to_string();
        let session = request.session.trim().to_string();
        let expires_at = now + chrono::TimeDelta::seconds(request.ttl_seconds.max(1) as i64);
        let conn = self.connect()?;
        let existing = conn
            .query_row(
                "SELECT payload_json FROM dev_server_leases WHERE scope = ?1",
                params![scope.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("fetch existing dev server lease")?;
        let mut receipt_kind = "dev_server_acquire";

        if let Some(payload) = existing {
            let existing_lease: DevServerLeaseRecord =
                serde_json::from_str(&payload).context("deserialize existing dev server lease")?;
            if existing_lease.session != session && existing_lease.expires_at > now {
                let stale_cutoff =
                    now - chrono::TimeDelta::seconds(request.stale_after_seconds.max(1) as i64);
                if !request.recover_stale || existing_lease.last_heartbeat_at > stale_cutoff {
                    self.record_hive_coordination_receipt(&HiveCoordinationReceiptRequest {
                        kind: "dev_server_conflict".to_string(),
                        actor_session: session.clone(),
                        actor_agent: request.effective_agent.clone().or(request.agent.clone()),
                        target_session: Some(existing_lease.session.clone()),
                        task_id: None,
                        scope: Some(scope.clone()),
                        project: request.project.clone(),
                        namespace: request.namespace.clone(),
                        workspace: request.workspace.clone(),
                        summary: format!(
                            "Dev server {} is already leased by {}.",
                            request.url, existing_lease.session
                        ),
                    })?;
                    anyhow::bail!(
                        "dev_server_conflict: scope '{}' already leased by {}",
                        scope,
                        existing_lease
                            .effective_agent
                            .as_deref()
                            .unwrap_or(existing_lease.session.as_str())
                    );
                }
                receipt_kind = "dev_server_recover";
            } else {
                receipt_kind = "dev_server_heartbeat";
            }
        }

        let lease = DevServerLeaseRecord {
            scope: scope.clone(),
            host: request.host.trim().to_string(),
            port: request.port,
            url: request.url.trim().to_string(),
            repo_root: request.repo_root.trim().to_string(),
            repo_hash: request.repo_hash.trim().to_string(),
            command: request.command.clone(),
            session: session.clone(),
            tab_id: request.tab_id.clone(),
            agent: request.agent.clone(),
            effective_agent: request.effective_agent.clone(),
            project: request.project.clone(),
            namespace: request.namespace.clone(),
            workspace: request.workspace.clone(),
            host_name: request.host_name.clone(),
            pid: request.pid,
            acquired_at: now,
            last_heartbeat_at: now,
            expires_at,
        };

        let payload_json = serde_json::to_string(&lease).context("serialize dev server lease")?;
        conn.execute(
            r#"
            INSERT INTO dev_server_leases
              (scope, session, project, namespace, workspace, repo_hash, host, port, expires_at, last_heartbeat_at, payload_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(scope) DO UPDATE SET
              session = excluded.session,
              project = excluded.project,
              namespace = excluded.namespace,
              workspace = excluded.workspace,
              repo_hash = excluded.repo_hash,
              host = excluded.host,
              port = excluded.port,
              expires_at = excluded.expires_at,
              last_heartbeat_at = excluded.last_heartbeat_at,
              payload_json = excluded.payload_json
            "#,
            params![
                lease.scope.as_str(),
                lease.session.as_str(),
                &lease.project,
                &lease.namespace,
                &lease.workspace,
                lease.repo_hash.as_str(),
                lease.host.as_str(),
                lease.port,
                lease.expires_at.to_rfc3339(),
                lease.last_heartbeat_at.to_rfc3339(),
                payload_json,
            ],
        )
        .context("upsert dev server lease")?;

        self.record_hive_coordination_receipt(&HiveCoordinationReceiptRequest {
            kind: receipt_kind.to_string(),
            actor_session: lease.session.clone(),
            actor_agent: lease.effective_agent.clone().or(lease.agent.clone()),
            target_session: None,
            task_id: None,
            scope: Some(lease.scope.clone()),
            project: lease.project.clone(),
            namespace: lease.namespace.clone(),
            workspace: lease.workspace.clone(),
            summary: format!("{} {}", receipt_kind, lease.url),
        })?;

        Ok(DevServerLeasesResponse {
            leases: vec![lease],
        })
    }

    pub fn release_dev_server_lease(
        &self,
        request: &DevServerLeaseReleaseRequest,
    ) -> anyhow::Result<DevServerLeasesResponse> {
        let conn = self.connect()?;
        let payload = conn
            .query_row(
                "SELECT payload_json FROM dev_server_leases WHERE scope = ?1",
                params![request.scope.trim()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .context("fetch dev server lease for release")?;
        let Some(payload) = payload else {
            return Ok(DevServerLeasesResponse { leases: Vec::new() });
        };
        let lease: DevServerLeaseRecord =
            serde_json::from_str(&payload).context("deserialize dev server lease for release")?;
        if lease.session != request.session.trim() {
            anyhow::bail!(
                "dev_server_release_denied: scope '{}' is leased by {}",
                lease.scope,
                lease.session
            );
        }
        conn.execute(
            "DELETE FROM dev_server_leases WHERE scope = ?1 AND session = ?2",
            params![request.scope.trim(), request.session.trim()],
        )
        .context("release dev server lease")?;
        self.record_hive_coordination_receipt(&HiveCoordinationReceiptRequest {
            kind: "dev_server_release".to_string(),
            actor_session: lease.session.clone(),
            actor_agent: lease.effective_agent.clone().or(lease.agent.clone()),
            target_session: None,
            task_id: None,
            scope: Some(lease.scope.clone()),
            project: lease.project.clone(),
            namespace: lease.namespace.clone(),
            workspace: lease.workspace.clone(),
            summary: format!("Released dev server lease {}", lease.url),
        })?;
        Ok(DevServerLeasesResponse {
            leases: vec![lease],
        })
    }

    pub fn dev_server_leases(
        &self,
        request: &DevServerLeasesRequest,
    ) -> anyhow::Result<DevServerLeasesResponse> {
        self.prune_expired_dev_server_leases()?;
        let limit = request.limit.unwrap_or(128).clamp(1, 1024) as i64;
        let active_only = request.active_only.unwrap_or(true);
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                r#"
                SELECT payload_json
                FROM dev_server_leases
                WHERE (?1 IS NULL OR session = ?1)
                  AND (?2 IS NULL OR project = ?2)
                  AND (?3 IS NULL OR namespace = ?3)
                  AND (?4 IS NULL OR workspace = ?4)
                  AND (?5 IS NULL OR repo_hash = ?5)
                  AND (?6 = 0 OR expires_at > ?7)
                ORDER BY last_heartbeat_at DESC
                LIMIT ?8
                "#,
            )
            .context("prepare dev server leases query")?;
        let now = chrono::Utc::now().to_rfc3339();
        let rows = stmt
            .query_map(
                params![
                    request.session.clone(),
                    request.project.clone(),
                    request.namespace.clone(),
                    request.workspace.clone(),
                    request.repo_hash.clone(),
                    if active_only { 1 } else { 0 },
                    now,
                    limit,
                ],
                |row| row.get::<_, String>(0),
            )
            .context("query dev server leases")?;
        let mut leases = Vec::new();
        for row in rows {
            let payload = row.context("read dev server lease row")?;
            leases.push(
                serde_json::from_str::<DevServerLeaseRecord>(&payload)
                    .context("deserialize dev server lease payload")?,
            );
        }
        Ok(DevServerLeasesResponse { leases })
    }

    pub fn upsert_hive_session(
        &self,
        request: &HiveSessionUpsertRequest,
    ) -> anyhow::Result<HiveSessionsResponse> {
        self.prune_stale_hive_sessions()?;

        let now = chrono::Utc::now();
        let session = request.session.trim().to_string();
        let project = request
            .project
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let namespace = request
            .namespace
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let workspace = request
            .workspace
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let repo_root = request
            .repo_root
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let worktree_root = request
            .worktree_root
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let branch = request
            .branch
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let record = HiveSessionRecord {
            session: session.clone(),
            tab_id: request
                .tab_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            agent: request
                .agent
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            effective_agent: request
                .effective_agent
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            hive_system: request
                .hive_system
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            hive_role: request
                .hive_role
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            worker_name: request
                .worker_name
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            display_name: request
                .display_name
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            role: request
                .role
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            capabilities: request
                .capabilities
                .iter()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect(),
            hive_groups: request
                .hive_groups
                .iter()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect(),
            lane_id: request
                .lane_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            hive_group_goal: request
                .hive_group_goal
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            authority: request
                .authority
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            heartbeat_model: request
                .heartbeat_model
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            project: project.clone(),
            namespace: namespace.clone(),
            workspace: workspace.clone(),
            repo_root: repo_root.clone(),
            worktree_root: worktree_root.clone(),
            branch: branch.clone(),
            base_branch: request
                .base_branch
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            visibility: request
                .visibility
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            base_url: request
                .base_url
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            base_url_healthy: request.base_url_healthy,
            host: request
                .host
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            pid: request.pid,
            topic_claim: request
                .topic_claim
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            scope_claims: request
                .scope_claims
                .iter()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect(),
            task_id: request
                .task_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            focus: request
                .focus
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            pressure: request
                .pressure
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            next_recovery: request
                .next_recovery
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            next_action: request
                .next_action
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            working: request
                .working
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            touches: request
                .touches
                .iter()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect(),
            relationship_state: None,
            relationship_peer: None,
            relationship_reason: None,
            suggested_action: None,
            blocked_by: request
                .blocked_by
                .iter()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect(),
            cowork_with: request
                .cowork_with
                .iter()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect(),
            handoff_target: request
                .handoff_target
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            offered_to: request
                .offered_to
                .iter()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect(),
            needs_help: request.needs_help,
            needs_review: request.needs_review,
            handoff_state: request
                .handoff_state
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            confidence: request
                .confidence
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            risk: request
                .risk
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            status: request
                .status
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("live")
                .to_string(),
            last_seen: now,
            last_wake_at: request.last_wake_at,
        };
        let payload_json = serde_json::to_string(&record).context("serialize hive session")?;
        let session_key = hive_session_key(
            &session,
            HiveSessionKeyArgs {
                project: project.as_deref(),
                namespace: namespace.as_deref(),
                workspace: workspace.as_deref(),
                repo_root: repo_root.as_deref(),
                worktree_root: worktree_root.as_deref(),
                branch: branch.as_deref(),
                agent: record.agent.as_deref(),
                effective_agent: record.effective_agent.as_deref(),
                hive_system: record.hive_system.as_deref(),
                hive_role: record.hive_role.as_deref(),
                host: record.host.as_deref(),
            },
        );

        let mut conn = self.connect()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .context("begin hive session upsert transaction")?;
        tx.execute(
            r#"
            INSERT INTO hive_sessions (
              session_key, session, project, namespace, workspace, repo_root, worktree_root, branch, hive_system, hive_role, host, status, last_seen, last_wake_at, payload_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            ON CONFLICT(session_key) DO UPDATE SET
              session = excluded.session,
              project = excluded.project,
              namespace = excluded.namespace,
              workspace = excluded.workspace,
              repo_root = excluded.repo_root,
              worktree_root = excluded.worktree_root,
              branch = excluded.branch,
              hive_system = excluded.hive_system,
              hive_role = excluded.hive_role,
              host = excluded.host,
              status = excluded.status,
              last_seen = excluded.last_seen,
              last_wake_at = excluded.last_wake_at,
              payload_json = excluded.payload_json
            "#,
            params![
                session_key,
                record.session,
                &record.project,
                &record.namespace,
                &record.workspace,
                &record.repo_root,
                &record.worktree_root,
                &record.branch,
                &record.hive_system,
                &record.hive_role,
                &record.host,
                record.status,
                record.last_seen.to_rfc3339(),
                record.last_wake_at.map(|dt| dt.to_rfc3339()),
                payload_json,
            ],
        )
        .context("upsert hive session")?;
        tx.execute(
            "DELETE FROM hive_session_groups WHERE session_key = ?1",
            params![session_key],
        )
        .context("clear hive session groups")?;

        {
            let mut insert_group_stmt = tx.prepare(
                "INSERT OR REPLACE INTO hive_session_groups (session_key, hive_group) VALUES (?1, ?2)",
            )?;
            for hive_group in record.hive_groups.iter() {
                insert_group_stmt.execute(params![session_key, hive_group])?;
            }
        }
        tx.commit()
            .context("commit hive session upsert transaction")?;

        Ok(HiveSessionsResponse {
            sessions: vec![record],
        })
    }

    pub fn hive_sessions(
        &self,
        request: &HiveSessionsRequest,
    ) -> anyhow::Result<HiveSessionsResponse> {
        self.prune_stale_hive_sessions()?;

        let active_only = request.active_only.unwrap_or(true);
        let limit = request.limit.unwrap_or(128).clamp(1, 1024) as i64;
        let repo_root = request
            .repo_root
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let worktree_root = request
            .worktree_root
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let branch = request
            .branch
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let hive_system = request
            .hive_system
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let hive_role = request
            .hive_role
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let host = request
            .host
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let hive_group = request
            .hive_group
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);

        let conn = self.connect()?;
        let session_filter = request
            .session
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let project_filter = request
            .project
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let namespace_filter = request
            .namespace
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let workspace_filter = request
            .workspace
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());

        let mut stmt = conn
            .prepare(
                r#"
                SELECT payload_json
                FROM hive_sessions
                WHERE (?1 IS NULL OR session = ?1)
                  AND (?2 IS NULL OR project = ?2)
                  AND (?3 IS NULL OR namespace = ?3)
                  AND (?4 IS NULL OR workspace = ?4)
                  AND (?5 IS NULL OR repo_root = ?5)
                  AND (?6 IS NULL OR worktree_root = ?6)
                  AND (?7 IS NULL OR branch = ?7)
                  AND (?8 IS NULL OR hive_system = ?8)
                  AND (?9 IS NULL OR hive_role = ?9)
                  AND (?10 IS NULL OR host = ?10)
                  AND (
                    ?11 IS NULL OR EXISTS (
                      SELECT 1
                      FROM hive_session_groups
                      WHERE hive_session_groups.session_key = hive_sessions.session_key
                        AND hive_session_groups.hive_group = ?11
                    )
                  )
                ORDER BY last_seen DESC
                LIMIT ?12
                "#,
            )
            .context("prepare hive sessions query")?;
        let rows = stmt
            .query_map(
                params![
                    session_filter,
                    project_filter,
                    namespace_filter,
                    workspace_filter,
                    repo_root,
                    worktree_root,
                    branch,
                    hive_system,
                    hive_role,
                    host,
                    hive_group,
                    limit,
                ],
                |row| row.get::<_, String>(0),
            )
            .context("query hive sessions")?;

        let mut sessions = Vec::new();
        for row in rows {
            let payload = row.context("read hive session row")?;
            sessions.push(
                serde_json::from_str::<HiveSessionRecord>(&payload)
                    .context("deserialize hive session payload")?,
            );
        }

        let now = chrono::Utc::now();
        let mut sessions = collapse_hive_session_records(sessions);
        for session in sessions.iter_mut() {
            refresh_hive_session_presence(session, now);
        }
        if active_only {
            sessions.retain(|session| matches!(session.status.as_str(), "active" | "live"));
        }

        Ok(HiveSessionsResponse { sessions })
    }
}
