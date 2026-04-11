    use super::*;
    use std::sync::{Arc, Mutex, OnceLock};

    use crate::render::{
        render_agent_zero_harness_pack_markdown, render_claude_code_harness_pack_markdown,
        render_codex_harness_pack_markdown, render_command_catalog_markdown,
        render_command_catalog_summary, render_hermes_harness_pack_markdown,
        render_openclaw_harness_pack_markdown, render_opencode_harness_pack_markdown,
    };
    use axum::{
        Json, Router,
        extract::{Query, State},
        http::StatusCode,
        routing::{get, post},
    };
    use memd_schema::{
        BenchmarkEvidenceSummary, BenchmarkFeatureRecord, BenchmarkGateDecision,
        BenchmarkSubjectMetrics, ContinuityJourneyReport, HiveClaimAcquireRequest, HiveClaimRecord,
        HiveClaimReleaseRequest, HiveClaimTransferRequest, HiveClaimsRequest, HiveClaimsResponse,
        HiveCoordinationInboxResponse, HiveCoordinationReceiptRecord,
        HiveCoordinationReceiptRequest, HiveCoordinationReceiptsResponse, HiveMessageAckRequest,
        HiveMessageInboxRequest, HiveMessageRecord, HiveMessageSendRequest, HiveMessagesResponse,
        HiveTaskRecord, SkillPolicyActivationRecord, SkillPolicyApplyReceipt,
        SkillPolicyApplyReceiptsRequest, SkillPolicyApplyReceiptsResponse, SkillPolicyApplyRequest,
        SkillPolicyApplyResponse, VerifierAssertionRecord, VerifierStepRecord,
    };

    #[path = "evaluation_runtime_tests_tail.rs"]
    mod evaluation_runtime_tests_tail;

    static HOME_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn lock_home_mutation() -> std::sync::MutexGuard<'static, ()> {
        HOME_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("HOME mutation lock poisoned")
    }

    fn lock_env_mutation() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env mutation lock poisoned")
    }

    fn normalize_path_text(value: impl AsRef<Path>) -> String {
        value.as_ref().to_string_lossy().replace('\\', "/")
    }

    fn path_text_contains(value: impl AsRef<Path>, needle: &str) -> bool {
        normalize_path_text(value).contains(needle)
    }

    fn path_text_ends_with(value: impl AsRef<Path>, needle: &str) -> bool {
        normalize_path_text(value).ends_with(needle)
    }

    fn assert_path_tail(actual: &str, expected: &Path) {
        let expected = fs::canonicalize(expected).unwrap_or_else(|_| expected.to_path_buf());
        assert!(
            Path::new(actual).ends_with(&expected),
            "path {actual:?} did not end with {expected:?}"
        );
    }

    fn codex_test_snapshot(project: &str, namespace: &str, agent: &str) -> ResumeSnapshot {
        ResumeSnapshot {
            project: Some(project.to_string()),
            namespace: Some(namespace.to_string()),
            agent: Some(agent.to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            context: memd_schema::CompactContextResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                records: vec![memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "keep the live wake surface current".to_string(),
                }],
            },
            working: memd_schema::WorkingMemoryResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                budget_chars: 1600,
                used_chars: 120,
                remaining_chars: 1480,
                truncated: false,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 8,
                    max_chars_per_item: 220,
                    budget_chars: 1600,
                    rehydration_limit: 4,
                },
                records: vec![memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "follow the codex pack turn boundary".to_string(),
                }],
                evicted: Vec::new(),
                rehydration_queue: vec![memd_schema::MemoryRehydrationRecord {
                    id: None,
                    kind: "source".to_string(),
                    label: "handoff".to_string(),
                    summary: "reload the bundled wake and memory files".to_string(),
                    reason: None,
                    source_agent: None,
                    source_system: None,
                    source_path: None,
                    source_quality: None,
                    recorded_at: None,
                }],
                traces: Vec::new(),
                semantic_consolidation: None,
            },
            inbox: memd_schema::MemoryInboxResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::CurrentTask,
                items: vec![memd_schema::InboxMemoryItem {
                    item: memd_schema::MemoryItem {
                        id: uuid::Uuid::new_v4(),
                        content: "capture the latest turn result".to_string(),
                        redundancy_key: None,
                        belief_branch: None,
                        preferred: true,
                        kind: memd_schema::MemoryKind::Status,
                        scope: memd_schema::MemoryScope::Project,
                        project: Some(project.to_string()),
                        namespace: Some(namespace.to_string()),
                        workspace: Some("team-alpha".to_string()),
                        visibility: memd_schema::MemoryVisibility::Workspace,
                        source_agent: None,
                        source_system: None,
                        source_path: None,
                        source_quality: None,
                        confidence: 0.8,
                        ttl_seconds: Some(86_400),
                        created_at: chrono::Utc::now(),
                        status: memd_schema::MemoryStatus::Active,
                        stage: memd_schema::MemoryStage::Candidate,
                        last_verified_at: None,
                        supersedes: Vec::new(),
                        updated_at: chrono::Utc::now(),
                        tags: vec!["checkpoint".to_string()],
                    },
                    reasons: vec!["current-turn".to_string()],
                }],
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: vec![memd_schema::WorkspaceMemoryRecord {
                    project: Some(project.to_string()),
                    namespace: Some(namespace.to_string()),
                    workspace: Some("team-alpha".to_string()),
                    visibility: memd_schema::MemoryVisibility::Workspace,
                    item_count: 3,
                    active_count: 2,
                    candidate_count: 1,
                    contested_count: 0,
                    source_lane_count: 1,
                    avg_confidence: 0.84,
                    trust_score: 0.91,
                    last_seen_at: None,
                    tags: Vec::new(),
                }],
            },
            sources: memd_schema::SourceMemoryResponse {
                sources: Vec::new(),
            },
            semantic: None,
            claims: SessionClaimsState::default(),
            recent_repo_changes: vec!["status M crates/memd-client/src/main.rs".to_string()],
            change_summary: vec!["focus -> follow the codex pack turn boundary".to_string()],
            resume_state_age_minutes: None,
            refresh_recommended: false,
        }
    }

    #[derive(Clone, Default)]
    struct MockHiveState {
        messages: Arc<Mutex<Vec<HiveMessageRecord>>>,
        claims: Arc<Mutex<Vec<HiveClaimRecord>>>,
        receipts: Arc<Mutex<Vec<HiveCoordinationReceiptRecord>>>,
        skill_policy_receipts: Arc<Mutex<Vec<SkillPolicyApplyReceipt>>>,
        skill_policy_activations: Arc<Mutex<Vec<memd_schema::SkillPolicyActivationEntry>>>,
    }

    #[derive(Clone, Default)]
    struct MockRuntimeState {
        stored: Arc<Mutex<Vec<memd_schema::StoreMemoryRequest>>>,
        repaired: Arc<Mutex<Vec<memd_schema::RepairMemoryRequest>>>,
        session_upserts: Arc<Mutex<Vec<memd_schema::HiveSessionUpsertRequest>>>,
        session_retires: Arc<Mutex<Vec<memd_schema::HiveSessionRetireRequest>>>,
        session_records: Arc<Mutex<Vec<memd_schema::HiveSessionRecord>>>,
        messages: Arc<Mutex<Vec<HiveMessageRecord>>>,
        claims: Arc<Mutex<Vec<HiveClaimRecord>>>,
        receipts: Arc<Mutex<Vec<HiveCoordinationReceiptRecord>>>,
        task_records: Arc<Mutex<Vec<HiveTaskRecord>>>,
        search_count: Arc<Mutex<usize>>,
        source_requests: Arc<Mutex<Vec<memd_schema::SourceMemoryRequest>>>,
        context_compact_response: Arc<Mutex<Option<memd_schema::CompactContextResponse>>>,
        working_response: Arc<Mutex<Option<memd_schema::WorkingMemoryResponse>>>,
    }

    async fn mock_send_hive_message(
        State(state): State<MockHiveState>,
        Json(req): Json<HiveMessageSendRequest>,
    ) -> Json<HiveMessagesResponse> {
        let message = HiveMessageRecord {
            id: uuid::Uuid::new_v4().to_string(),
            kind: req.kind,
            from_session: req.from_session,
            from_agent: req.from_agent,
            to_session: req.to_session,
            project: req.project,
            namespace: req.namespace,
            workspace: req.workspace,
            content: req.content,
            created_at: Utc::now(),
            acknowledged_at: None,
        };
        state
            .messages
            .lock()
            .expect("lock messages")
            .push(message.clone());
        Json(HiveMessagesResponse {
            messages: vec![message],
        })
    }

    async fn mock_hive_inbox(
        State(state): State<MockHiveState>,
        Query(req): Query<HiveMessageInboxRequest>,
    ) -> Json<HiveMessagesResponse> {
        let messages = state
            .messages
            .lock()
            .expect("lock messages")
            .iter()
            .filter(|message| {
                message.to_session == req.session
                    && req
                        .project
                        .as_ref()
                        .is_none_or(|project| message.project.as_ref() == Some(project))
                    && req
                        .namespace
                        .as_ref()
                        .is_none_or(|namespace| message.namespace.as_ref() == Some(namespace))
                    && req
                        .workspace
                        .as_ref()
                        .is_none_or(|workspace| message.workspace.as_ref() == Some(workspace))
                    && (req.include_acknowledged.unwrap_or(false)
                        || message.acknowledged_at.is_none())
            })
            .cloned()
            .collect();
        Json(HiveMessagesResponse { messages })
    }

    async fn mock_hive_ack(
        State(state): State<MockHiveState>,
        Json(req): Json<HiveMessageAckRequest>,
    ) -> Json<HiveMessagesResponse> {
        let mut messages = state.messages.lock().expect("lock messages");
        let mut acked = Vec::new();
        for message in messages.iter_mut() {
            if message.id == req.id && message.to_session == req.session {
                message.acknowledged_at = Some(Utc::now());
                acked.push(message.clone());
            }
        }
        Json(HiveMessagesResponse { messages: acked })
    }

    async fn mock_claim_acquire(
        State(state): State<MockHiveState>,
        Json(req): Json<HiveClaimAcquireRequest>,
    ) -> Result<Json<HiveClaimsResponse>, (StatusCode, String)> {
        let mut claims = state.claims.lock().expect("lock claims");
        claims.retain(|claim| claim.expires_at > Utc::now());
        if let Some(existing) = claims
            .iter()
            .find(|claim| claim.scope == req.scope && claim.session != req.session)
            .cloned()
        {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "scope '{}' already claimed by {}",
                    existing.scope,
                    existing
                        .effective_agent
                        .as_deref()
                        .or(Some(existing.session.as_str()))
                        .unwrap_or("unknown")
                ),
            ));
        }
        claims.retain(|claim| !(claim.scope == req.scope && claim.session == req.session));
        let claim = HiveClaimRecord {
            scope: req.scope,
            session: req.session,
            tab_id: req.tab_id,
            agent: req.agent,
            effective_agent: req.effective_agent,
            project: req.project,
            namespace: req.namespace,
            workspace: req.workspace,
            host: req.host,
            pid: req.pid,
            acquired_at: Utc::now(),
            expires_at: Utc::now() + chrono::TimeDelta::seconds(req.ttl_seconds as i64),
        };
        claims.push(claim.clone());
        Ok(Json(HiveClaimsResponse {
            claims: vec![claim],
        }))
    }

    async fn mock_claim_release(
        State(state): State<MockHiveState>,
        Json(req): Json<HiveClaimReleaseRequest>,
    ) -> Json<HiveClaimsResponse> {
        let mut claims = state.claims.lock().expect("lock claims");
        let mut released = Vec::new();
        claims.retain(|claim| {
            let matches = claim.scope == req.scope && claim.session == req.session;
            if matches {
                released.push(claim.clone());
            }
            !matches
        });
        Json(HiveClaimsResponse { claims: released })
    }

    async fn mock_claim_transfer(
        State(state): State<MockHiveState>,
        Json(req): Json<HiveClaimTransferRequest>,
    ) -> Json<HiveClaimsResponse> {
        let mut claims = state.claims.lock().expect("lock claims");
        let mut transferred = Vec::new();
        for claim in claims.iter_mut() {
            if claim.scope == req.scope && claim.session == req.from_session {
                claim.session = req.to_session.clone();
                claim.agent = req.to_agent.clone();
                claim.effective_agent = req.to_effective_agent.clone();
                transferred.push(claim.clone());
            }
        }
        Json(HiveClaimsResponse {
            claims: transferred,
        })
    }

    async fn mock_claims(
        State(state): State<MockHiveState>,
        Query(req): Query<HiveClaimsRequest>,
    ) -> Json<HiveClaimsResponse> {
        let claims = state
            .claims
            .lock()
            .expect("lock claims")
            .iter()
            .filter(|claim| {
                req.session
                    .as_ref()
                    .is_none_or(|session| &claim.session == session)
                    && req
                        .project
                        .as_ref()
                        .is_none_or(|project| claim.project.as_ref() == Some(project))
                    && req
                        .namespace
                        .as_ref()
                        .is_none_or(|namespace| claim.namespace.as_ref() == Some(namespace))
                    && req
                        .workspace
                        .as_ref()
                        .is_none_or(|workspace| claim.workspace.as_ref() == Some(workspace))
                    && (!req.active_only.unwrap_or(true) || claim.expires_at > Utc::now())
            })
            .cloned()
            .collect();
        Json(HiveClaimsResponse { claims })
    }

    async fn mock_record_receipt(
        State(state): State<MockHiveState>,
        Json(req): Json<HiveCoordinationReceiptRequest>,
    ) -> Json<HiveCoordinationReceiptsResponse> {
        let receipt = HiveCoordinationReceiptRecord {
            id: uuid::Uuid::new_v4().to_string(),
            kind: req.kind,
            actor_session: req.actor_session,
            actor_agent: req.actor_agent,
            target_session: req.target_session,
            task_id: req.task_id,
            scope: req.scope,
            project: req.project,
            namespace: req.namespace,
            workspace: req.workspace,
            summary: req.summary,
            created_at: Utc::now(),
        };
        state
            .receipts
            .lock()
            .expect("lock receipts")
            .push(receipt.clone());
        Json(HiveCoordinationReceiptsResponse {
            receipts: vec![receipt],
        })
    }

    async fn mock_runtime_record_receipt(
        State(state): State<MockRuntimeState>,
        Json(req): Json<HiveCoordinationReceiptRequest>,
    ) -> Json<HiveCoordinationReceiptsResponse> {
        let receipt = HiveCoordinationReceiptRecord {
            id: uuid::Uuid::new_v4().to_string(),
            kind: req.kind,
            actor_session: req.actor_session,
            actor_agent: req.actor_agent,
            target_session: req.target_session,
            task_id: req.task_id,
            scope: req.scope,
            project: req.project,
            namespace: req.namespace,
            workspace: req.workspace,
            summary: req.summary,
            created_at: Utc::now(),
        };
        state
            .receipts
            .lock()
            .expect("lock runtime receipts")
            .push(receipt.clone());
        Json(HiveCoordinationReceiptsResponse {
            receipts: vec![receipt],
        })
    }

    async fn mock_runtime_send_hive_message(
        State(state): State<MockRuntimeState>,
        Json(req): Json<HiveMessageSendRequest>,
    ) -> Json<HiveMessagesResponse> {
        let message = HiveMessageRecord {
            id: uuid::Uuid::new_v4().to_string(),
            kind: req.kind,
            from_session: req.from_session,
            from_agent: req.from_agent,
            to_session: req.to_session,
            project: req.project,
            namespace: req.namespace,
            workspace: req.workspace,
            content: req.content,
            created_at: Utc::now(),
            acknowledged_at: None,
        };
        state
            .messages
            .lock()
            .expect("lock runtime messages")
            .push(message.clone());
        Json(HiveMessagesResponse {
            messages: vec![message],
        })
    }

    async fn mock_runtime_hive_inbox(
        State(state): State<MockRuntimeState>,
        Query(req): Query<HiveMessageInboxRequest>,
    ) -> Json<HiveMessagesResponse> {
        let messages = state
            .messages
            .lock()
            .expect("lock runtime messages")
            .iter()
            .filter(|message| {
                message.to_session == req.session
                    && req
                        .project
                        .as_ref()
                        .is_none_or(|project| message.project.as_ref() == Some(project))
                    && req
                        .namespace
                        .as_ref()
                        .is_none_or(|namespace| message.namespace.as_ref() == Some(namespace))
                    && req
                        .workspace
                        .as_ref()
                        .is_none_or(|workspace| message.workspace.as_ref() == Some(workspace))
                    && (req.include_acknowledged.unwrap_or(false)
                        || message.acknowledged_at.is_none())
            })
            .cloned()
            .collect::<Vec<_>>();
        Json(HiveMessagesResponse { messages })
    }

    async fn mock_runtime_hive_ack(
        State(state): State<MockRuntimeState>,
        Json(req): Json<HiveMessageAckRequest>,
    ) -> Json<HiveMessagesResponse> {
        let mut messages = state.messages.lock().expect("lock runtime messages");
        let mut acked = Vec::new();
        for message in messages.iter_mut() {
            if message.id == req.id && message.to_session == req.session {
                message.acknowledged_at = Some(Utc::now());
                acked.push(message.clone());
            }
        }
        Json(HiveMessagesResponse { messages: acked })
    }

    async fn mock_runtime_receipts(
        State(state): State<MockRuntimeState>,
        Query(req): Query<HiveCoordinationReceiptsRequest>,
    ) -> Json<HiveCoordinationReceiptsResponse> {
        let receipts = state
            .receipts
            .lock()
            .expect("lock runtime receipts")
            .iter()
            .filter(|receipt| {
                req.session
                    .as_ref()
                    .is_none_or(|value| receipt.actor_session == *value)
                    && req
                        .project
                        .as_ref()
                        .is_none_or(|value| receipt.project.as_ref() == Some(value))
                    && req
                        .namespace
                        .as_ref()
                        .is_none_or(|value| receipt.namespace.as_ref() == Some(value))
                    && req
                        .workspace
                        .as_ref()
                        .is_none_or(|value| receipt.workspace.as_ref() == Some(value))
            })
            .cloned()
            .collect::<Vec<_>>();
        Json(HiveCoordinationReceiptsResponse { receipts })
    }

    async fn mock_runtime_claims(
        State(state): State<MockRuntimeState>,
        Query(req): Query<HiveClaimsRequest>,
    ) -> Json<HiveClaimsResponse> {
        let claims = state
            .claims
            .lock()
            .expect("lock runtime claims")
            .iter()
            .filter(|claim| {
                req.session
                    .as_ref()
                    .is_none_or(|session| &claim.session == session)
                    && req
                        .project
                        .as_ref()
                        .is_none_or(|project| claim.project.as_ref() == Some(project))
                    && req
                        .namespace
                        .as_ref()
                        .is_none_or(|namespace| claim.namespace.as_ref() == Some(namespace))
                    && req
                        .workspace
                        .as_ref()
                        .is_none_or(|workspace| claim.workspace.as_ref() == Some(workspace))
                    && (!req.active_only.unwrap_or(true) || claim.expires_at > Utc::now())
            })
            .cloned()
            .collect::<Vec<_>>();
        Json(HiveClaimsResponse { claims })
    }

    async fn mock_runtime_claim_acquire(
        State(state): State<MockRuntimeState>,
        Json(req): Json<HiveClaimAcquireRequest>,
    ) -> Result<Json<HiveClaimsResponse>, (StatusCode, String)> {
        let mut claims = state.claims.lock().expect("lock runtime claims");
        claims.retain(|claim| claim.expires_at > Utc::now());
        if let Some(existing) = claims
            .iter()
            .find(|claim| claim.scope == req.scope && claim.session != req.session)
            .cloned()
        {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "scope '{}' already claimed by {}",
                    existing.scope,
                    existing
                        .effective_agent
                        .as_deref()
                        .or(Some(existing.session.as_str()))
                        .unwrap_or("unknown")
                ),
            ));
        }
        claims.retain(|claim| !(claim.scope == req.scope && claim.session == req.session));
        let claim = HiveClaimRecord {
            scope: req.scope,
            session: req.session,
            tab_id: req.tab_id,
            agent: req.agent,
            effective_agent: req.effective_agent,
            project: req.project,
            namespace: req.namespace,
            workspace: req.workspace,
            host: req.host,
            pid: req.pid,
            acquired_at: Utc::now(),
            expires_at: Utc::now() + chrono::TimeDelta::seconds(req.ttl_seconds as i64),
        };
        claims.push(claim.clone());
        Ok(Json(HiveClaimsResponse {
            claims: vec![claim],
        }))
    }

    async fn mock_runtime_claim_release(
        State(state): State<MockRuntimeState>,
        Json(req): Json<HiveClaimReleaseRequest>,
    ) -> Json<HiveClaimsResponse> {
        let mut claims = state.claims.lock().expect("lock runtime claims");
        let mut released = Vec::new();
        claims.retain(|claim| {
            let matches = claim.scope == req.scope && claim.session == req.session;
            if matches {
                released.push(claim.clone());
            }
            !matches
        });
        Json(HiveClaimsResponse { claims: released })
    }

    async fn mock_runtime_claim_transfer(
        State(state): State<MockRuntimeState>,
        Json(req): Json<HiveClaimTransferRequest>,
    ) -> Json<HiveClaimsResponse> {
        let mut claims = state.claims.lock().expect("lock runtime claims");
        let mut transferred = Vec::new();
        for claim in claims.iter_mut() {
            if claim.scope == req.scope && claim.session == req.from_session {
                claim.session = req.to_session.clone();
                claim.tab_id = req.to_tab_id.clone();
                claim.agent = req.to_agent.clone();
                claim.effective_agent = req.to_effective_agent.clone();
                transferred.push(claim.clone());
            }
        }
        Json(HiveClaimsResponse {
            claims: transferred,
        })
    }

    async fn mock_runtime_task_upsert(
        State(state): State<MockRuntimeState>,
        Json(req): Json<memd_schema::HiveTaskUpsertRequest>,
    ) -> Json<memd_schema::HiveTasksResponse> {
        let mut tasks = state
            .task_records
            .lock()
            .expect("lock runtime task records");
        let now = Utc::now();
        let task = if let Some(existing) = tasks.iter_mut().find(|task| task.task_id == req.task_id)
        {
            existing.title = req.title.clone();
            existing.description = req.description.clone();
            if let Some(status) = req.status.clone() {
                existing.status = status;
            }
            if let Some(mode) = req.coordination_mode.clone() {
                existing.coordination_mode = mode;
            }
            if req.session.is_some() {
                existing.session = req.session.clone();
            }
            if req.agent.is_some() {
                existing.agent = req.agent.clone();
            }
            if req.effective_agent.is_some() {
                existing.effective_agent = req.effective_agent.clone();
            }
            if req.project.is_some() {
                existing.project = req.project.clone();
            }
            if req.namespace.is_some() {
                existing.namespace = req.namespace.clone();
            }
            if req.workspace.is_some() {
                existing.workspace = req.workspace.clone();
            }
            if !req.claim_scopes.is_empty() {
                existing.claim_scopes = req.claim_scopes.clone();
            }
            if let Some(help_requested) = req.help_requested {
                existing.help_requested = help_requested;
            }
            if let Some(review_requested) = req.review_requested {
                existing.review_requested = review_requested;
            }
            existing.updated_at = now;
            existing.clone()
        } else {
            let task = HiveTaskRecord {
                task_id: req.task_id,
                title: req.title,
                description: req.description,
                status: req.status.unwrap_or_else(|| "open".to_string()),
                coordination_mode: req
                    .coordination_mode
                    .unwrap_or_else(|| "shared".to_string()),
                session: req.session,
                agent: req.agent,
                effective_agent: req.effective_agent,
                project: req.project,
                namespace: req.namespace,
                workspace: req.workspace,
                claim_scopes: req.claim_scopes,
                help_requested: req.help_requested.unwrap_or(false),
                review_requested: req.review_requested.unwrap_or(false),
                created_at: now,
                updated_at: now,
            };
            tasks.push(task.clone());
            task
        };
        Json(memd_schema::HiveTasksResponse { tasks: vec![task] })
    }

    async fn mock_runtime_task_assign(
        State(state): State<MockRuntimeState>,
        Json(req): Json<memd_schema::HiveTaskAssignRequest>,
    ) -> Json<memd_schema::HiveTasksResponse> {
        let mut tasks = state
            .task_records
            .lock()
            .expect("lock runtime task records");
        let mut assigned = Vec::new();
        for task in tasks.iter_mut() {
            if task.task_id == req.task_id {
                task.session = Some(req.to_session.clone());
                task.agent = req.to_agent.clone();
                task.effective_agent = req.to_effective_agent.clone();
                task.updated_at = Utc::now();
                assigned.push(task.clone());
            }
        }
        Json(memd_schema::HiveTasksResponse { tasks: assigned })
    }

    async fn mock_record_skill_policy_apply(
        State(state): State<MockHiveState>,
        Json(req): Json<SkillPolicyApplyRequest>,
    ) -> Json<SkillPolicyApplyResponse> {
        let receipt = SkillPolicyApplyReceipt {
            id: uuid::Uuid::new_v4().to_string(),
            bundle_root: req.bundle_root,
            runtime_defaulted: req.runtime_defaulted,
            source_queue_path: req.source_queue_path,
            applied_count: req.applied_count,
            skipped_count: req.skipped_count,
            project: req.project,
            namespace: req.namespace,
            workspace: req.workspace,
            created_at: Utc::now(),
        };
        state
            .skill_policy_receipts
            .lock()
            .expect("lock skill policy receipts")
            .push(receipt.clone());
        {
            let mut activations = state
                .skill_policy_activations
                .lock()
                .expect("lock skill policy activations");
            for record in req.applied {
                activations.push(memd_schema::SkillPolicyActivationEntry {
                    receipt_id: receipt.id.clone(),
                    bundle_root: receipt.bundle_root.clone(),
                    runtime_defaulted: receipt.runtime_defaulted,
                    source_queue_path: receipt.source_queue_path.clone(),
                    record,
                    project: receipt.project.clone(),
                    namespace: receipt.namespace.clone(),
                    workspace: receipt.workspace.clone(),
                    created_at: receipt.created_at,
                });
            }
        }
        Json(SkillPolicyApplyResponse { receipt })
    }

    async fn mock_skill_policy_apply_receipts(
        State(state): State<MockHiveState>,
        Query(req): Query<SkillPolicyApplyReceiptsRequest>,
    ) -> Json<SkillPolicyApplyReceiptsResponse> {
        let receipts = state
            .skill_policy_receipts
            .lock()
            .expect("lock skill policy receipts")
            .iter()
            .filter(|receipt| {
                req.project
                    .as_ref()
                    .is_none_or(|project| receipt.project.as_ref() == Some(project))
                    && req
                        .namespace
                        .as_ref()
                        .is_none_or(|namespace| receipt.namespace.as_ref() == Some(namespace))
                    && req
                        .workspace
                        .as_ref()
                        .is_none_or(|workspace| receipt.workspace.as_ref() == Some(workspace))
            })
            .cloned()
            .collect();
        Json(SkillPolicyApplyReceiptsResponse { receipts })
    }

    async fn mock_context_compact(
        State(state): State<MockRuntimeState>,
    ) -> Json<memd_schema::CompactContextResponse> {
        let response = state
            .context_compact_response
            .lock()
            .expect("lock context response")
            .clone()
            .unwrap_or(memd_schema::CompactContextResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::General,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                records: vec![memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "context record".to_string(),
                }],
            });
        Json(response)
    }

    async fn mock_working_memory(
        State(state): State<MockRuntimeState>,
    ) -> Json<memd_schema::WorkingMemoryResponse> {
        let response = state
            .working_response
            .lock()
            .expect("lock working response")
            .clone()
            .unwrap_or(memd_schema::WorkingMemoryResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::General,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                budget_chars: 1600,
                used_chars: 240,
                remaining_chars: 1360,
                truncated: false,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 8,
                    max_chars_per_item: 220,
                    budget_chars: 1600,
                    rehydration_limit: 4,
                },
                records: vec![memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "working record".to_string(),
                }],
                evicted: Vec::new(),
                rehydration_queue: Vec::new(),
                traces: Vec::new(),
                semantic_consolidation: None,
            });
        Json(response)
    }

    async fn mock_inbox() -> Json<memd_schema::MemoryInboxResponse> {
        Json(memd_schema::MemoryInboxResponse {
            route: memd_schema::RetrievalRoute::Auto,
            intent: memd_schema::RetrievalIntent::General,
            items: Vec::new(),
        })
    }

    async fn mock_maintenance_report(
        Query(req): Query<MemoryMaintenanceReportRequest>,
    ) -> Json<memd_schema::MemoryMaintenanceReportResponse> {
        let mode = req.mode.unwrap_or_else(|| "scan".to_string());
        Json(memd_schema::MemoryMaintenanceReportResponse {
            reinforced_candidates: 2,
            cooled_candidates: 1,
            consolidated_candidates: 3,
            stale_items: 4,
            skipped: 1,
            highlights: vec!["memory maintenance mock".to_string()],
            receipt_id: Some(uuid::Uuid::new_v4().to_string()),
            mode: Some(mode.clone()),
            compacted_items: if mode == "compact" { 3 } else { 0 },
            refreshed_items: if mode == "refresh" { 2 } else { 0 },
            repaired_items: if mode == "repair" { 1 } else { 0 },
            generated_at: Utc::now(),
        })
    }

    async fn mock_hive_tasks(
        State(state): State<MockRuntimeState>,
        Query(req): Query<HiveTasksRequest>,
    ) -> Json<memd_schema::HiveTasksResponse> {
        let tasks = state
            .task_records
            .lock()
            .expect("lock task records")
            .iter()
            .filter(|task| {
                req.project
                    .as_ref()
                    .is_none_or(|project| task.project.as_ref() == Some(project))
                    && req
                        .namespace
                        .as_ref()
                        .is_none_or(|namespace| task.namespace.as_ref() == Some(namespace))
                    && req
                        .workspace
                        .as_ref()
                        .is_none_or(|workspace| task.workspace.as_ref() == Some(workspace))
                    && req
                        .session
                        .as_ref()
                        .is_none_or(|session| task.session.as_ref() == Some(session))
                    && (!req.active_only.unwrap_or(false)
                        || (task.status != "done" && task.status != "closed"))
            })
            .cloned()
            .collect::<Vec<_>>();
        Json(memd_schema::HiveTasksResponse { tasks })
    }

    async fn mock_hive_coordination_inbox(
        State(state): State<MockRuntimeState>,
        Query(req): Query<HiveCoordinationInboxRequest>,
    ) -> Json<HiveCoordinationInboxResponse> {
        let tasks = state
            .task_records
            .lock()
            .expect("lock task records")
            .clone();
        let owned_tasks = tasks
            .iter()
            .filter(|task| task.session.as_deref() == Some(req.session.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        let help_tasks = tasks
            .iter()
            .filter(|task| task.help_requested)
            .cloned()
            .collect::<Vec<_>>();
        let review_tasks = tasks
            .iter()
            .filter(|task| task.review_requested)
            .cloned()
            .collect::<Vec<_>>();
        Json(HiveCoordinationInboxResponse {
            messages: vec![HiveMessageRecord {
                id: "msg-1".to_string(),
                kind: "help_request".to_string(),
                from_session: "codex-b".to_string(),
                from_agent: Some("codex@codex-b".to_string()),
                to_session: req.session,
                project: req.project,
                namespace: req.namespace,
                workspace: req.workspace,
                content: "Need help on shared task".to_string(),
                created_at: Utc::now(),
                acknowledged_at: None,
            }],
            owned_tasks,
            help_tasks,
            review_tasks,
        })
    }

    async fn mock_runtime_maintain(
        Json(req): Json<memd_schema::MaintainReportRequest>,
    ) -> Json<MaintainReport> {
        let mode = req.mode.clone();
        Json(MaintainReport {
            mode: mode.clone(),
            receipt_id: Some(uuid::Uuid::new_v4().to_string()),
            compacted_items: if mode == "compact" { 3 } else { 1 },
            refreshed_items: if mode == "refresh" { 2 } else { 0 },
            repaired_items: if mode == "repair" { 1 } else { 0 },
            findings: vec![
                format!("memory maintain mode={mode}"),
                if req.apply {
                    "apply requested".to_string()
                } else {
                    "scan only".to_string()
                },
            ],
            generated_at: Utc::now(),
        })
    }

    async fn mock_skill_policy_activations(
        State(state): State<MockHiveState>,
        Query(req): Query<SkillPolicyActivationEntriesRequest>,
    ) -> Json<SkillPolicyActivationEntriesResponse> {
        let activations =
            state
                .skill_policy_activations
                .lock()
                .expect("lock skill policy activations")
                .iter()
                .filter(|activation| {
                    req.project
                        .as_ref()
                        .is_none_or(|project| activation.project.as_ref() == Some(project))
                        && req.namespace.as_ref().is_none_or(|namespace| {
                            activation.namespace.as_ref() == Some(namespace)
                        })
                        && req.workspace.as_ref().is_none_or(|workspace| {
                            activation.workspace.as_ref() == Some(workspace)
                        })
                })
                .cloned()
                .collect();
        Json(SkillPolicyActivationEntriesResponse { activations })
    }

    async fn mock_workspace_memory() -> Json<memd_schema::WorkspaceMemoryResponse> {
        Json(memd_schema::WorkspaceMemoryResponse {
            workspaces: vec![memd_schema::WorkspaceMemoryRecord {
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("team-alpha".to_string()),
                visibility: memd_schema::MemoryVisibility::Workspace,
                item_count: 6,
                active_count: 4,
                candidate_count: 0,
                contested_count: 0,
                source_lane_count: 2,
                avg_confidence: 0.8,
                trust_score: 0.92,
                last_seen_at: Some(Utc::now()),
                tags: vec!["baseline".to_string()],
            }],
        })
    }

    async fn mock_source_memory(
        State(state): State<MockRuntimeState>,
        Query(req): Query<memd_schema::SourceMemoryRequest>,
    ) -> Json<memd_schema::SourceMemoryResponse> {
        state
            .source_requests
            .lock()
            .expect("lock source requests")
            .push(req);
        Json(memd_schema::SourceMemoryResponse {
            sources: Vec::new(),
        })
    }

    async fn mock_search_memory(
        State(state): State<MockRuntimeState>,
        Json(req): Json<memd_schema::SearchMemoryRequest>,
    ) -> Json<memd_schema::SearchMemoryResponse> {
        *state.search_count.lock().expect("lock search count") += 1;
        let query = req.query.clone().unwrap_or_default();
        let items = if req.tags.iter().any(|tag| tag == "resume_state") {
            Vec::new()
        } else if req
            .tags
            .iter()
            .any(|tag| tag == "caveman-ultra" || tag == "token-efficient")
            && req.query.is_some()
        {
            Vec::new()
        } else if req.query.is_none()
            && req
                .tags
                .iter()
                .any(|tag| tag == "caveman-ultra" || tag == "token-efficient")
        {
            vec![memd_schema::MemoryItem {
                id: uuid::Uuid::new_v4(),
                content: "preference: use caveman ultra as the default response style for all sessions; keep replies very short and token-efficient.".to_string(),
                redundancy_key: Some("Preference|Global|memd|main||caveman|default|efficient|preference|response|session|style|token|ultra|use|very".to_string()),
                belief_branch: None,
                preferred: false,
                kind: memd_schema::MemoryKind::Preference,
                scope: memd_schema::MemoryScope::Global,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: None,
                visibility: memd_schema::MemoryVisibility::Private,
                source_agent: Some("codex@session-a".to_string()),
                source_system: Some("codex".to_string()),
                source_path: Some("session".to_string()),
                source_quality: Some(memd_schema::SourceQuality::Canonical),
                confidence: 0.98,
                ttl_seconds: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_verified_at: Some(Utc::now()),
                supersedes: Vec::new(),
                tags: vec!["caveman-ultra".to_string(), "token-efficient".to_string()],
                status: memd_schema::MemoryStatus::Active,
                stage: memd_schema::MemoryStage::Canonical,
            }]
        } else if query.contains("stale belief") {
            vec![memd_schema::MemoryItem {
                id: uuid::Uuid::new_v4(),
                content: "stale belief".to_string(),
                redundancy_key: Some("Fact|Project|demo|main||stale|belief".to_string()),
                belief_branch: None,
                preferred: false,
                kind: memd_schema::MemoryKind::Fact,
                scope: memd_schema::MemoryScope::Project,
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                visibility: memd_schema::MemoryVisibility::Workspace,
                source_agent: Some("codex@session-a".to_string()),
                source_system: Some("memd".to_string()),
                source_path: None,
                source_quality: Some(memd_schema::SourceQuality::Canonical),
                confidence: 0.72,
                ttl_seconds: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_verified_at: Some(Utc::now()),
                supersedes: Vec::new(),
                tags: vec!["correction-target".to_string()],
                status: memd_schema::MemoryStatus::Stale,
                stage: memd_schema::MemoryStage::Canonical,
            }]
        } else {
            vec![memd_schema::MemoryItem {
                id: uuid::Uuid::new_v4(),
                content: "hive resume state".to_string(),
                redundancy_key: Some("Status|Project|demo|main||hive|resume|state".to_string()),
                belief_branch: None,
                preferred: false,
                kind: memd_schema::MemoryKind::Status,
                scope: memd_schema::MemoryScope::Project,
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                visibility: memd_schema::MemoryVisibility::Workspace,
                source_agent: Some("codex@session-a".to_string()),
                source_system: Some("memd-resume-state".to_string()),
                source_path: None,
                source_quality: Some(memd_schema::SourceQuality::Derived),
                confidence: 0.94,
                ttl_seconds: Some(86_400),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_verified_at: Some(Utc::now()),
                supersedes: Vec::new(),
                tags: vec!["resume_state".to_string(), "session_state".to_string()],
                status: memd_schema::MemoryStatus::Active,
                stage: memd_schema::MemoryStage::Canonical,
            }]
        };
        Json(memd_schema::SearchMemoryResponse {
            route: req.route.unwrap_or(memd_schema::RetrievalRoute::Auto),
            intent: req.intent.unwrap_or(memd_schema::RetrievalIntent::General),
            items,
        })
    }

    async fn mock_store_memory(
        State(state): State<MockRuntimeState>,
        Json(req): Json<memd_schema::StoreMemoryRequest>,
    ) -> Json<memd_schema::StoreMemoryResponse> {
        state.stored.lock().expect("lock stored").push(req.clone());
        Json(memd_schema::StoreMemoryResponse {
            item: memd_schema::MemoryItem {
                id: uuid::Uuid::new_v4(),
                content: req.content,
                redundancy_key: Some("stored".to_string()),
                belief_branch: req.belief_branch,
                preferred: false,
                kind: req.kind,
                scope: req.scope,
                project: req.project,
                namespace: req.namespace,
                workspace: req.workspace,
                visibility: req
                    .visibility
                    .unwrap_or(memd_schema::MemoryVisibility::Private),
                source_agent: req.source_agent,
                source_system: req.source_system,
                source_path: req.source_path,
                source_quality: req.source_quality,
                confidence: req.confidence.unwrap_or(0.7),
                ttl_seconds: req.ttl_seconds,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_verified_at: req.last_verified_at,
                supersedes: req.supersedes,
                tags: req.tags,
                status: req.status.unwrap_or(memd_schema::MemoryStatus::Active),
                stage: memd_schema::MemoryStage::Canonical,
            },
        })
    }

    async fn mock_repair_memory(
        State(state): State<MockRuntimeState>,
        Json(req): Json<memd_schema::RepairMemoryRequest>,
    ) -> Json<memd_schema::RepairMemoryResponse> {
        state
            .repaired
            .lock()
            .expect("lock repaired")
            .push(req.clone());
        Json(memd_schema::RepairMemoryResponse {
            item: memd_schema::MemoryItem {
                id: req.id,
                content: req.content.unwrap_or_else(|| "repaired".to_string()),
                redundancy_key: Some("repaired".to_string()),
                belief_branch: None,
                preferred: false,
                kind: memd_schema::MemoryKind::Status,
                scope: memd_schema::MemoryScope::Project,
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: req.workspace,
                visibility: req
                    .visibility
                    .unwrap_or(memd_schema::MemoryVisibility::Workspace),
                source_agent: req.source_agent,
                source_system: req.source_system,
                source_path: req.source_path,
                source_quality: req.source_quality,
                confidence: req.confidence.unwrap_or(0.7),
                ttl_seconds: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_verified_at: Some(Utc::now()),
                supersedes: req.supersedes,
                tags: req.tags.unwrap_or_default(),
                status: req.status.unwrap_or(memd_schema::MemoryStatus::Active),
                stage: memd_schema::MemoryStage::Canonical,
            },
            mode: req.mode,
            reasons: vec!["mock_repair".to_string()],
        })
    }

    async fn mock_hive_session_upsert(
        State(state): State<MockRuntimeState>,
        Json(req): Json<memd_schema::HiveSessionUpsertRequest>,
    ) -> Json<memd_schema::HiveSessionsResponse> {
        state
            .session_upserts
            .lock()
            .expect("lock session upserts")
            .push(req.clone());
        let record = memd_schema::HiveSessionRecord {
            session: req.session,
            tab_id: req.tab_id,
            agent: req.agent,
            effective_agent: req.effective_agent,
            hive_system: req.hive_system,
            hive_role: req.hive_role,
            worker_name: req.worker_name,
            display_name: req.display_name,
            role: req.role,
            capabilities: req.capabilities,
            hive_groups: req.hive_groups,
            lane_id: req.lane_id,
            hive_group_goal: req.hive_group_goal,
            authority: req.authority,
            heartbeat_model: req.heartbeat_model,
            project: req.project,
            namespace: req.namespace,
            repo_root: req.repo_root,
            worktree_root: req.worktree_root,
            branch: req.branch,
            base_branch: req.base_branch,
            workspace: req.workspace,
            visibility: req.visibility,
            base_url: req.base_url,
            base_url_healthy: req.base_url_healthy,
            host: req.host,
            pid: req.pid,
            topic_claim: req.topic_claim,
            scope_claims: req.scope_claims,
            task_id: req.task_id,
            focus: req.focus,
            pressure: req.pressure,
            next_recovery: req.next_recovery,
            next_action: req.next_action,
            needs_help: req.needs_help,
            needs_review: req.needs_review,
            handoff_state: req.handoff_state,
            confidence: req.confidence,
            risk: req.risk,
            status: req.status.unwrap_or_else(|| "live".to_string()),
            last_seen: Utc::now(),
        };
        {
            let mut records = state.session_records.lock().expect("lock session records");
            records.push(record.clone());
        }
        Json(memd_schema::HiveSessionsResponse {
            sessions: vec![record],
        })
    }

    async fn mock_hive_sessions(
        State(state): State<MockRuntimeState>,
        Query(req): Query<memd_schema::HiveSessionsRequest>,
    ) -> Json<memd_schema::HiveSessionsResponse> {
        let sessions = state
            .session_records
            .lock()
            .expect("lock session records")
            .iter()
            .filter(|record| {
                req.session
                    .as_ref()
                    .is_none_or(|value| record.session == *value)
                    && req
                        .project
                        .as_ref()
                        .is_none_or(|value| record.project.as_ref() == Some(value))
                    && req
                        .namespace
                        .as_ref()
                        .is_none_or(|value| record.namespace.as_ref() == Some(value))
                    && req
                        .workspace
                        .as_ref()
                        .is_none_or(|value| record.workspace.as_ref() == Some(value))
            })
            .cloned()
            .collect::<Vec<_>>();
        Json(memd_schema::HiveSessionsResponse { sessions })
    }

    async fn mock_hive_board(
        State(state): State<MockRuntimeState>,
        Query(req): Query<memd_schema::HiveBoardRequest>,
    ) -> Json<memd_schema::HiveBoardResponse> {
        let sessions = state
            .session_records
            .lock()
            .expect("lock session records")
            .iter()
            .filter(|record| {
                req.project
                    .as_ref()
                    .is_none_or(|value| record.project.as_ref() == Some(value))
                    && req
                        .namespace
                        .as_ref()
                        .is_none_or(|value| record.namespace.as_ref() == Some(value))
                    && req
                        .workspace
                        .as_ref()
                        .is_none_or(|value| record.workspace.as_ref() == Some(value))
            })
            .cloned()
            .collect::<Vec<_>>();
        let tasks = state
            .task_records
            .lock()
            .expect("lock task records")
            .iter()
            .filter(|task| {
                req.project
                    .as_ref()
                    .is_none_or(|value| task.project.as_ref() == Some(value))
                    && req
                        .namespace
                        .as_ref()
                        .is_none_or(|value| task.namespace.as_ref() == Some(value))
                    && req
                        .workspace
                        .as_ref()
                        .is_none_or(|value| task.workspace.as_ref() == Some(value))
            })
            .cloned()
            .collect::<Vec<_>>();
        let receipts = state
            .receipts
            .lock()
            .expect("lock receipts")
            .iter()
            .filter(|receipt| {
                req.project
                    .as_ref()
                    .is_none_or(|value| receipt.project.as_ref() == Some(value))
                    && req
                        .namespace
                        .as_ref()
                        .is_none_or(|value| receipt.namespace.as_ref() == Some(value))
                    && req
                        .workspace
                        .as_ref()
                        .is_none_or(|value| receipt.workspace.as_ref() == Some(value))
            })
            .cloned()
            .collect::<Vec<_>>();
        let queen_session = sessions
            .iter()
            .find(|session| {
                matches!(
                    session.role.as_deref().or(session.hive_role.as_deref()),
                    Some("queen" | "orchestrator" | "memory-control-plane")
                )
            })
            .map(|session| session.session.clone());
        let active_bees = sessions
            .iter()
            .filter(|session| session.status == "active")
            .cloned()
            .collect::<Vec<_>>();
        let stale_bees = sessions
            .iter()
            .filter(|session| session.status != "active")
            .map(|session| session.session.clone())
            .collect::<Vec<_>>();
        let review_queue = tasks
            .iter()
            .filter(|task| task.review_requested)
            .map(|task| {
                format!(
                    "{} -> {}",
                    task.task_id,
                    task.session.as_deref().unwrap_or("unassigned")
                )
            })
            .collect::<Vec<_>>();
        let overlap_risks = receipts
            .iter()
            .filter(|receipt| receipt.kind.contains("overlap") || receipt.summary.contains("scope"))
            .map(|receipt| receipt.summary.clone())
            .collect::<Vec<_>>();
        let blocked_bees = receipts
            .iter()
            .filter(|receipt| receipt.kind == "queen_deny" || receipt.kind.starts_with("lane_"))
            .map(|receipt| receipt.summary.clone())
            .collect::<Vec<_>>();
        let lane_faults = receipts
            .iter()
            .filter(|receipt| receipt.kind.starts_with("lane_"))
            .map(|receipt| receipt.summary.clone())
            .collect::<Vec<_>>();
        let recommended_actions = receipts
            .iter()
            .filter(|receipt| receipt.kind.starts_with("queen_"))
            .map(|receipt| receipt.summary.clone())
            .collect::<Vec<_>>();
        Json(memd_schema::HiveBoardResponse {
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

    async fn mock_hive_roster(
        State(state): State<MockRuntimeState>,
        Query(req): Query<memd_schema::HiveRosterRequest>,
    ) -> Json<memd_schema::HiveRosterResponse> {
        let bees = state
            .session_records
            .lock()
            .expect("lock session records")
            .iter()
            .filter(|record| {
                req.project
                    .as_ref()
                    .is_none_or(|value| record.project.as_ref() == Some(value))
                    && req
                        .namespace
                        .as_ref()
                        .is_none_or(|value| record.namespace.as_ref() == Some(value))
                    && req
                        .workspace
                        .as_ref()
                        .is_none_or(|value| record.workspace.as_ref() == Some(value))
            })
            .cloned()
            .collect::<Vec<_>>();
        let queen_session = bees
            .iter()
            .find(|session| {
                matches!(
                    session.role.as_deref().or(session.hive_role.as_deref()),
                    Some("queen" | "orchestrator" | "memory-control-plane")
                )
            })
            .map(|session| session.session.clone());
        Json(memd_schema::HiveRosterResponse {
            project: req.project.unwrap_or_else(|| "unknown".to_string()),
            namespace: req.namespace.unwrap_or_else(|| "default".to_string()),
            queen_session,
            bees,
        })
    }

    async fn mock_hive_follow(
        State(state): State<MockRuntimeState>,
        Query(req): Query<memd_schema::HiveFollowRequest>,
    ) -> Json<memd_schema::HiveFollowResponse> {
        let target = state
            .session_records
            .lock()
            .expect("lock session records")
            .iter()
            .find(|record| record.session == req.session)
            .cloned()
            .expect("target hive session exists");
        let messages = state
            .messages
            .lock()
            .expect("lock messages")
            .iter()
            .filter(|message| message.to_session == req.session)
            .cloned()
            .collect::<Vec<_>>();
        let owned_tasks = state
            .task_records
            .lock()
            .expect("lock task records")
            .iter()
            .filter(|task| task.session.as_deref() == Some(req.session.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        let recent_receipts = state
            .receipts
            .lock()
            .expect("lock receipts")
            .iter()
            .filter(|receipt| {
                receipt.actor_session == req.session
                    || receipt.target_session.as_deref() == Some(req.session.as_str())
            })
            .cloned()
            .collect::<Vec<_>>();
        let recommended_action = if !messages.is_empty() {
            "watch_and_coordinate".to_string()
        } else {
            "safe_to_continue".to_string()
        };
        Json(memd_schema::HiveFollowResponse {
            current_session: req.current_session,
            target: target.clone(),
            work_summary: target
                .topic_claim
                .clone()
                .or(target.focus.clone())
                .unwrap_or_else(|| "none".to_string()),
            touch_points: target.scope_claims.clone(),
            next_action: target.next_action.clone(),
            messages,
            owned_tasks,
            help_tasks: Vec::new(),
            review_tasks: Vec::new(),
            recent_receipts,
            overlap_risk: None,
            recommended_action,
        })
    }

    async fn mock_hive_session_retire(
        State(state): State<MockRuntimeState>,
        Json(req): Json<memd_schema::HiveSessionRetireRequest>,
    ) -> Json<memd_schema::HiveSessionRetireResponse> {
        state
            .session_retires
            .lock()
            .expect("lock session retires")
            .push(req.clone());
        let mut records = state.session_records.lock().expect("lock session records");
        let mut retired = Vec::new();
        records.retain(|record| {
            let matches = record.session == req.session
                && req
                    .project
                    .as_ref()
                    .is_none_or(|value| record.project.as_ref() == Some(value))
                && req
                    .namespace
                    .as_ref()
                    .is_none_or(|value| record.namespace.as_ref() == Some(value))
                && req
                    .workspace
                    .as_ref()
                    .is_none_or(|value| record.workspace.as_ref() == Some(value))
                && req
                    .agent
                    .as_ref()
                    .is_none_or(|value| record.agent.as_ref() == Some(value))
                && req
                    .effective_agent
                    .as_ref()
                    .is_none_or(|value| record.effective_agent.as_ref() == Some(value))
                && req
                    .hive_system
                    .as_ref()
                    .is_none_or(|value| record.hive_system.as_ref() == Some(value))
                && req
                    .hive_role
                    .as_ref()
                    .is_none_or(|value| record.hive_role.as_ref() == Some(value))
                && req
                    .host
                    .as_ref()
                    .is_none_or(|value| record.host.as_ref() == Some(value));
            if matches {
                retired.push(record.clone());
            }
            !matches
        });
        Json(memd_schema::HiveSessionRetireResponse {
            retired: retired.len(),
            sessions: retired,
        })
    }

    async fn mock_hive_session_auto_retire(
        State(state): State<MockRuntimeState>,
        Json(req): Json<memd_schema::HiveSessionAutoRetireRequest>,
    ) -> Json<memd_schema::HiveSessionAutoRetireResponse> {
        let tasks = state
            .task_records
            .lock()
            .expect("lock task records")
            .clone();
        let receipts = state.receipts.lock().expect("lock receipts").clone();
        let stale_cutoff = Utc::now() - chrono::TimeDelta::minutes(15);
        let mut records = state.session_records.lock().expect("lock session records");
        let mut retired = Vec::new();
        records.retain(|record| {
            let in_scope = req
                .project
                .as_ref()
                .is_none_or(|value| record.project.as_ref() == Some(value))
                && req
                    .namespace
                    .as_ref()
                    .is_none_or(|value| record.namespace.as_ref() == Some(value))
                && req
                    .workspace
                    .as_ref()
                    .is_none_or(|value| record.workspace.as_ref() == Some(value));
            if !in_scope || record.last_seen >= stale_cutoff {
                return true;
            }
            let owns_active_task = tasks.iter().any(|task| {
                task.session.as_deref() == Some(record.session.as_str())
                    && task.status != "done"
                    && task.status != "closed"
            });
            let has_pending_handoff = receipts.iter().any(|receipt| {
                receipt.kind == "queen_handoff"
                    && (receipt.actor_session == record.session
                        || receipt.target_session.as_deref() == Some(record.session.as_str()))
            });
            if owns_active_task || has_pending_handoff {
                return true;
            }
            retired.push(record.session.clone());
            false
        });
        Json(memd_schema::HiveSessionAutoRetireResponse { retired })
    }

    async fn mock_healthz() -> Json<memd_schema::HealthResponse> {
        Json(memd_schema::HealthResponse {
            status: "ok".to_string(),
            items: 1,
        })
    }

    async fn mock_slow_hive_session_upsert(
        State(state): State<MockRuntimeState>,
        Json(req): Json<memd_schema::HiveSessionUpsertRequest>,
    ) -> Json<memd_schema::HiveSessionsResponse> {
        state
            .session_upserts
            .lock()
            .expect("lock session upserts")
            .push(req.clone());
        tokio::time::sleep(Duration::from_secs(5)).await;
        Json(memd_schema::HiveSessionsResponse {
            sessions: Vec::new(),
        })
    }

    async fn spawn_mock_memory_server() -> String {
        let state = MockRuntimeState::default();
        let app = Router::new()
            .route("/memory/context/compact", get(mock_context_compact))
            .route("/memory/working", get(mock_working_memory))
            .route("/memory/inbox", get(mock_inbox))
            .route("/memory/workspaces", get(mock_workspace_memory))
            .route("/memory/maintenance/report", get(mock_maintenance_report))
            .route("/runtime/maintain", post(mock_runtime_maintain))
            .with_state(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind mock memory server");
        let addr = listener.local_addr().expect("mock memory server addr");
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("serve mock memory server");
        });
        tokio::time::sleep(Duration::from_millis(25)).await;
        format!("http://{}", addr)
    }

    fn spawn_blocking_mock_sidecar_server() -> String {
        use std::io::Write;
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind blocking mock sidecar");
        let addr = listener.local_addr().expect("blocking mock sidecar addr");
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut stream = stream;
                let mut buffer = [0u8; 8192];
                let read = stream.read(&mut buffer).unwrap_or(0);
                let request = String::from_utf8_lossy(&buffer[..read]);
                let response_body = if request.starts_with("POST /v1/retrieve ") {
                    serde_json::to_string(&json!({
                        "status": "ok",
                        "mode": "text",
                        "items": [
                            {
                                "content": "retrieved target",
                                "source": "target",
                                "score": 0.95
                            },
                            {
                                "content": "retrieved current",
                                "source": "current",
                                "score": 0.25
                            }
                        ]
                    }))
                    .expect("serialize blocking mock retrieve")
                } else {
                    serde_json::to_string(&json!({
                        "status": "ok",
                        "track_id": uuid::Uuid::new_v4(),
                        "items": 1
                    }))
                    .expect("serialize blocking mock ingest")
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    response_body.len(),
                    response_body
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });
        format!("http://{}", addr)
    }

    async fn spawn_mock_runtime_server(state: MockRuntimeState, slow_hive_upsert: bool) -> String {
        let hive_route = if slow_hive_upsert {
            post(mock_slow_hive_session_upsert)
        } else {
            post(mock_hive_session_upsert)
        };
        let app = Router::new()
            .route("/healthz", get(mock_healthz))
            .route("/memory/context/compact", get(mock_context_compact))
            .route("/memory/working", get(mock_working_memory))
            .route("/memory/inbox", get(mock_inbox))
            .route("/memory/workspaces", get(mock_workspace_memory))
            .route("/memory/maintenance/report", get(mock_maintenance_report))
            .route("/runtime/maintain", post(mock_runtime_maintain))
            .route("/memory/source", get(mock_source_memory))
            .route("/memory/search", post(mock_search_memory))
            .route("/memory/store", post(mock_store_memory))
            .route("/memory/repair", post(mock_repair_memory))
            .route("/coordination/inbox", get(mock_hive_coordination_inbox))
            .route(
                "/coordination/messages/send",
                post(mock_runtime_send_hive_message),
            )
            .route("/coordination/messages/inbox", get(mock_runtime_hive_inbox))
            .route("/coordination/messages/ack", post(mock_runtime_hive_ack))
            .route("/coordination/tasks", get(mock_hive_tasks))
            .route("/coordination/tasks/upsert", post(mock_runtime_task_upsert))
            .route("/coordination/tasks/assign", post(mock_runtime_task_assign))
            .route("/coordination/claims", get(mock_runtime_claims))
            .route(
                "/coordination/claims/acquire",
                post(mock_runtime_claim_acquire),
            )
            .route(
                "/coordination/claims/release",
                post(mock_runtime_claim_release),
            )
            .route(
                "/coordination/claims/transfer",
                post(mock_runtime_claim_transfer),
            )
            .route("/coordination/sessions/upsert", hive_route)
            .route("/coordination/sessions", get(mock_hive_sessions))
            .route("/hive/board", get(mock_hive_board))
            .route("/hive/roster", get(mock_hive_roster))
            .route("/hive/follow", get(mock_hive_follow))
            .route(
                "/coordination/receipts/record",
                post(mock_runtime_record_receipt),
            )
            .route("/coordination/receipts", get(mock_runtime_receipts))
            .route(
                "/coordination/sessions/auto-retire",
                post(mock_hive_session_auto_retire),
            )
            .route(
                "/coordination/sessions/retire",
                post(mock_hive_session_retire),
            )
            .with_state(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind mock runtime server");
        let addr = listener.local_addr().expect("mock runtime server addr");
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("serve mock runtime server");
        });
        tokio::time::sleep(Duration::from_millis(25)).await;
        format!("http://{}", addr)
    }

    async fn spawn_mock_hive_server() -> String {
        let state = MockHiveState::default();
        let app = Router::new()
            .route("/coordination/messages/send", post(mock_send_hive_message))
            .route("/coordination/messages/inbox", get(mock_hive_inbox))
            .route("/coordination/messages/ack", post(mock_hive_ack))
            .route("/coordination/receipts/record", post(mock_record_receipt))
            .route(
                "/coordination/skill-policy/apply",
                post(mock_record_skill_policy_apply).get(mock_skill_policy_apply_receipts),
            )
            .route(
                "/coordination/skill-policy/activations",
                get(mock_skill_policy_activations),
            )
            .route("/coordination/claims/acquire", post(mock_claim_acquire))
            .route("/coordination/claims/release", post(mock_claim_release))
            .route("/coordination/claims/transfer", post(mock_claim_transfer))
            .route("/coordination/claims", get(mock_claims))
            .with_state(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind mock hive server");
        let addr = listener.local_addr().expect("mock hive server addr");
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("serve mock hive server");
        });
        tokio::time::sleep(Duration::from_millis(25)).await;
        format!("http://{}", addr)
    }

    fn push_mock_runtime_hive_session(
        state: &MockRuntimeState,
        session: &str,
        worker_name: &str,
        role: &str,
        task_id: Option<&str>,
        topic_claim: Option<&str>,
        scope_claims: Vec<String>,
    ) {
        state
            .session_records
            .lock()
            .expect("lock session records")
            .push(memd_schema::HiveSessionRecord {
                session: session.to_string(),
                tab_id: None,
                agent: Some("codex".to_string()),
                effective_agent: Some(format!("codex@{session}")),
                hive_system: Some("codex".to_string()),
                hive_role: Some(role.to_string()),
                worker_name: Some(worker_name.to_string()),
                display_name: None,
                role: Some(role.to_string()),
                capabilities: vec!["memory".to_string(), "coordination".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                lane_id: Some(format!("{session}-lane")),
                hive_group_goal: None,
                authority: Some(if role == "queen" {
                    "coordinator".to_string()
                } else {
                    "participant".to_string()
                }),
                heartbeat_model: Some(default_heartbeat_model()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: Some(format!("feature/{session}")),
                base_branch: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: None,
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(100),
                topic_claim: topic_claim.map(str::to_string),
                scope_claims,
                task_id: task_id.map(str::to_string),
                focus: topic_claim.map(str::to_string),
                pressure: None,
                next_recovery: None,
                next_action: Some("continue".to_string()),
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: Some("0.91".to_string()),
                risk: None,
                status: "active".to_string(),
                last_seen: Utc::now(),
            });
    }

    #[test]
    fn derives_help_request_message_from_scope() {
        let message = derive_outbound_message(&MessagesArgs {
            output: PathBuf::from(".memd"),
            send: true,
            inbox: false,
            ack: None,
            target_session: Some("claude-b".to_string()),
            kind: None,
            request_help: true,
            request_review: false,
            assign_scope: None,
            scope: Some("file:src/main.rs".to_string()),
            content: None,
            summary: false,
        })
        .expect("derive help request");

        assert_eq!(message.0, "help_request");
        assert!(message.1.contains("file:src/main.rs"));
    }

    #[test]
    fn derives_review_request_message_from_scope() {
        let message = derive_outbound_message(&MessagesArgs {
            output: PathBuf::from(".memd"),
            send: true,
            inbox: false,
            ack: None,
            target_session: Some("claude-b".to_string()),
            kind: None,
            request_help: false,
            request_review: true,
            assign_scope: None,
            scope: Some("task:parser-refactor".to_string()),
            content: None,
            summary: false,
        })
        .expect("derive review request");

        assert_eq!(message.0, "review_request");
        assert!(message.1.contains("task:parser-refactor"));
    }

    #[test]
    fn derives_assignment_message_from_assign_scope() {
        let message = derive_outbound_message(&MessagesArgs {
            output: PathBuf::from(".memd"),
            send: true,
            inbox: false,
            ack: None,
            target_session: Some("claude-b".to_string()),
            kind: None,
            request_help: false,
            request_review: false,
            assign_scope: Some("task:parser-refactor".to_string()),
            scope: None,
            content: None,
            summary: false,
        })
        .expect("derive assignment");

        assert_eq!(message.0, "assignment");
        assert!(message.1.contains("task:parser-refactor"));
    }

    #[test]
    fn resolves_nested_bundle_rag_config() {
        let config = BundleConfigFile {
            project: None,
            namespace: None,
            agent: None,
            session: None,
            tab_id: None,
            hive_system: None,
            hive_role: None,
            capabilities: Vec::new(),
            hive_groups: Vec::new(),
            hive_group_goal: None,
            authority: None,
            hive_project_enabled: false,
            hive_project_anchor: None,
            hive_project_joined_at: None,
            base_url: None,
            route: None,
            intent: None,
            workspace: None,
            visibility: None,
            heartbeat_model: Some(default_heartbeat_model()),
            voice_mode: Some(default_voice_mode()),
            auto_short_term_capture: true,
            rag_url: None,
            authority_policy: BundleAuthorityPolicy::default(),
            authority_state: BundleAuthorityState::default(),
            backend: Some(BundleBackendConfigFile {
                rag: Some(BundleRagConfigFile {
                    enabled: Some(true),
                    url: Some("http://127.0.0.1:9000".to_string()),
                }),
            }),
        };

        let resolved = resolve_bundle_rag_config(config).expect("bundle rag config");
        assert!(resolved.enabled);
        assert!(resolved.configured);
        assert_eq!(resolved.url.as_deref(), Some("http://127.0.0.1:9000"));
        assert_eq!(resolved.source, "backend.rag");
    }

    #[test]
    fn resolves_legacy_bundle_rag_url() {
        let config = BundleConfigFile {
            project: None,
            namespace: None,
            agent: None,
            session: None,
            tab_id: None,
            hive_system: None,
            hive_role: None,
            capabilities: Vec::new(),
            hive_groups: Vec::new(),
            hive_group_goal: None,
            authority: None,
            hive_project_enabled: false,
            hive_project_anchor: None,
            hive_project_joined_at: None,
            base_url: None,
            route: None,
            intent: None,
            workspace: None,
            visibility: None,
            heartbeat_model: Some(default_heartbeat_model()),
            voice_mode: Some(default_voice_mode()),
            auto_short_term_capture: true,
            rag_url: Some("http://127.0.0.1:9000".to_string()),
            backend: None,
            authority_policy: BundleAuthorityPolicy::default(),
            authority_state: BundleAuthorityState::default(),
        };

        let resolved = resolve_bundle_rag_config(config).expect("bundle rag config");
        assert!(resolved.enabled);
        assert!(resolved.configured);
        assert_eq!(resolved.url.as_deref(), Some("http://127.0.0.1:9000"));
        assert_eq!(resolved.source, "rag_url");
    }

    #[test]
    fn serializes_bundle_config_with_nested_rag_state() {
        let config = BundleConfig {
            schema_version: 2,
            project: "demo".to_string(),
            namespace: Some("main".to_string()),
            agent: "codex".to_string(),
            session: "session-demo".to_string(),
            tab_id: None,
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string(), "coordination".to_string()],
            hive_groups: vec!["openclaw-stack".to_string()],
            hive_group_goal: None,
            hive_project_enabled: false,
            hive_project_anchor: None,
            hive_project_joined_at: None,
            authority: Some("participant".to_string()),
            base_url: "http://127.0.0.1:8787".to_string(),
            route: "auto".to_string(),
            intent: "general".to_string(),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            heartbeat_model: default_heartbeat_model(),
            voice_mode: default_voice_mode(),
            auto_short_term_capture: true,
            authority_policy: BundleAuthorityPolicy::default(),
            authority_state: BundleAuthorityState::default(),
            backend: BundleBackendConfig {
                rag: BundleRagConfig {
                    enabled: true,
                    provider: "lightrag-compatible".to_string(),
                    url: Some("http://127.0.0.1:9000".to_string()),
                },
            },
            hooks: BundleHooksConfig {
                context: "hooks/memd-context.sh".to_string(),
                capture: "hooks/memd-capture.sh".to_string(),
                spill: "hooks/memd-spill.sh".to_string(),
                context_ps1: "hooks/memd-context.ps1".to_string(),
                capture_ps1: "hooks/memd-capture.ps1".to_string(),
                spill_ps1: "hooks/memd-spill.ps1".to_string(),
            },
            rag_url: Some("http://127.0.0.1:9000".to_string()),
        };

        let json = serde_json::to_value(config).expect("serialize bundle config");
        assert_eq!(json["schema_version"], 2);
        assert_eq!(json["namespace"], "main");
        assert_eq!(json["backend"]["rag"]["enabled"], true);
        assert_eq!(json["backend"]["rag"]["provider"], "lightrag-compatible");
        assert_eq!(json["backend"]["rag"]["url"], "http://127.0.0.1:9000");
        assert_eq!(json["workspace"], "team-alpha");
        assert_eq!(json["visibility"], "workspace");
        assert_eq!(json["hooks"]["capture"], "hooks/memd-capture.sh");
        assert_eq!(json["hooks"]["capture_ps1"], "hooks/memd-capture.ps1");
        assert_eq!(json["rag_url"], "http://127.0.0.1:9000");
    }

    #[test]
    fn writes_bundle_memory_placeholder_with_hot_path_guidance() {
        let dir =
            std::env::temp_dir().join(format!("memd-bundle-placeholder-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        let config = BundleConfig {
            schema_version: 2,
            project: "demo".to_string(),
            namespace: Some("main".to_string()),
            agent: "codex".to_string(),
            session: "session-demo".to_string(),
            tab_id: None,
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string(), "coordination".to_string()],
            hive_groups: vec!["openclaw-stack".to_string()],
            hive_group_goal: None,
            hive_project_enabled: false,
            hive_project_anchor: None,
            hive_project_joined_at: None,
            authority: Some("participant".to_string()),
            base_url: "http://127.0.0.1:8787".to_string(),
            route: "auto".to_string(),
            intent: "general".to_string(),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            heartbeat_model: default_heartbeat_model(),
            voice_mode: default_voice_mode(),
            auto_short_term_capture: true,
            authority_policy: BundleAuthorityPolicy::default(),
            authority_state: BundleAuthorityState::default(),
            backend: BundleBackendConfig {
                rag: BundleRagConfig {
                    enabled: true,
                    provider: "lightrag-compatible".to_string(),
                    url: Some("http://127.0.0.1:9000".to_string()),
                },
            },
            hooks: BundleHooksConfig {
                context: "hooks/memd-context.sh".to_string(),
                capture: "hooks/memd-capture.sh".to_string(),
                spill: "hooks/memd-spill.sh".to_string(),
                context_ps1: "hooks/memd-context.ps1".to_string(),
                capture_ps1: "hooks/memd-capture.ps1".to_string(),
                spill_ps1: "hooks/memd-spill.ps1".to_string(),
            },
            rag_url: Some("http://127.0.0.1:9000".to_string()),
        };

        write_bundle_memory_placeholder(&dir, &config, None, None).expect("write placeholder");
        write_native_agent_bridge_files(&dir).expect("write native bridge");

        let markdown = fs::read_to_string(dir.join("MEMD_MEMORY.md")).expect("read placeholder");
        assert!(markdown.contains("memd resume --output"));
        assert!(markdown.contains("--semantic"));
        assert!(markdown.contains("fast local hot path"));
        assert!(markdown.contains("slower deep recall"));
        assert!(markdown.contains("installed `$gsd-*` skills as the primary GSD interface"));
        assert!(markdown.contains("standalone `gsd-*` shell binaries"));
        assert!(markdown.contains("`$gsd-autonomous` is installed as a skill"));
        let claude_imports = fs::read_to_string(dir.join("agents").join("CLAUDE_IMPORTS.md"))
            .expect("read claude imports");
        assert!(claude_imports.contains("@../MEMD_WAKEUP.md"));
        assert!(claude_imports.contains("@../MEMD_MEMORY.md"));
        assert!(claude_imports.contains("@CLAUDE_CODE_WAKEUP.md"));
        assert!(claude_imports.contains("@CLAUDE_CODE_MEMORY.md"));
        assert!(claude_imports.contains("/memory"));
        assert!(claude_imports.contains("use installed `$gsd-*` skills as the GSD interface"));
        assert!(claude_imports.contains("standalone `gsd-*` shell binaries"));
        assert!(claude_imports.contains("`$gsd-autonomous` is installed as a skill"));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn wake_fallback_writes_placeholder_memory_and_wakeup_files() {
        let dir = std::env::temp_dir().join(format!("memd-wake-fallback-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");

        let config = BundleConfigFile {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            session: Some("session-demo".to_string()),
            tab_id: Some("tab-alpha".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            route: Some("auto".to_string()),
            intent: Some("current_task".to_string()),
            voice_mode: Some(default_voice_mode()),
            ..Default::default()
        };
        write_bundle_config_file(&dir.join("config.json"), &config).expect("write config");

        let args = WakeArgs {
            output: dir.clone(),
            project: None,
            namespace: None,
            agent: None,
            workspace: None,
            visibility: None,
            route: None,
            intent: None,
            limit: None,
            rehydration_limit: None,
            semantic: false,
            verbose: false,
            write: true,
            summary: false,
        };

        write_bundle_turn_fallback_artifacts(
            &dir,
            args.project.as_deref(),
            args.namespace.as_deref(),
            args.agent.as_deref(),
            args.workspace.as_deref(),
            args.visibility.as_deref(),
            args.route.as_deref(),
            args.intent.as_deref(),
            "# memd wake-up\n\n- fallback\n",
        )
        .expect("write wake fallback");

        let memory = fs::read_to_string(dir.join("MEMD_MEMORY.md")).expect("read memory");
        let wakeup = fs::read_to_string(dir.join("MEMD_WAKEUP.md")).expect("read wakeup");
        assert!(memory.contains("## Bundle Defaults"));
        assert!(memory.contains("project: demo"));
        assert!(memory.contains("namespace: main"));
        assert!(memory.contains("agent: codex"));
        assert!(memory.contains("session: session-demo"));
        assert!(memory.contains("tab: tab-alpha"));
        assert!(memory.contains("## Voice"));
        assert!(memory.contains("caveman ultra"));
        assert!(wakeup.contains("fallback"));
        assert!(dir.join("agents").join("CODEX_MEMORY.md").exists());
        assert!(dir.join("agents").join("CLAUDE_CODE_MEMORY.md").exists());

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn checkpoint_fallback_writes_placeholder_memory_without_agent() {
        let dir =
            std::env::temp_dir().join(format!("memd-checkpoint-fallback-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");

        let config = BundleConfigFile {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            session: Some("session-demo".to_string()),
            tab_id: Some("tab-alpha".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            route: Some("auto".to_string()),
            intent: Some("current_task".to_string()),
            voice_mode: Some(default_voice_mode()),
            ..Default::default()
        };
        write_bundle_config_file(&dir.join("config.json"), &config).expect("write config");

        write_bundle_turn_placeholder_memory(
            &dir,
            Some("demo"),
            Some("main"),
            None,
            Some("team-alpha"),
            Some("workspace"),
            Some("auto"),
            Some("current_task"),
        )
        .expect("write placeholder memory");

        let memory = fs::read_to_string(dir.join("MEMD_MEMORY.md")).expect("read memory");
        assert!(memory.contains("project: demo"));
        assert!(memory.contains("namespace: main"));
        assert!(memory.contains("session: session-demo"));
        assert!(memory.contains("tab: tab-alpha"));
        assert!(memory.contains("agent: codex"));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn fallback_memory_and_wakeup_surfaces_include_authority_warning() {
        let dir =
            std::env::temp_dir().join(format!("memd-authority-markdown-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");

        fs::write(
            dir.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "{}",
  "route": "auto",
  "intent": "current_task",
  "authority_policy": {{
    "shared_primary": true,
    "localhost_fallback_policy": "allow_read_only"
  }},
  "authority_state": {{
    "mode": "localhost_read_only",
    "degraded": true,
    "shared_base_url": "{}",
    "fallback_base_url": "http://127.0.0.1:8787",
    "reason": "tailscale is unavailable"
  }}
}}
"#,
                SHARED_MEMD_BASE_URL, SHARED_MEMD_BASE_URL
            ),
        )
        .expect("write config");

        write_memory_markdown_files(&dir, "# memd memory\n\nbody\n").expect("write memory");
        write_wakeup_markdown_files(&dir, "# memd wake-up\n\nbody\n").expect("write wakeup");

        let memory = fs::read_to_string(dir.join("MEMD_MEMORY.md")).expect("read memory");
        let wakeup = fs::read_to_string(dir.join("MEMD_WAKEUP.md")).expect("read wakeup");
        assert!(memory.contains("## Session Start Warning"));
        assert!(memory.contains("shared authority unavailable"));
        assert!(wakeup.contains("## Session Start Warning"));
        assert!(wakeup.contains("localhost fallback is lower trust"));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn copies_hook_assets_with_live_capture_scripts() {
        let dir = std::env::temp_dir().join(format!("memd-hook-assets-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create hook temp dir");

        copy_hook_assets(&dir).expect("copy hook assets");

        assert!(dir.join("memd-context.sh").exists());
        assert!(dir.join("memd-context.ps1").exists());
        assert!(dir.join("memd-capture.sh").exists());
        assert!(dir.join("memd-capture.ps1").exists());
        assert!(dir.join("memd-spill.sh").exists());
        assert!(dir.join("memd-spill.ps1").exists());

        let install = fs::read_to_string(dir.join("install.sh")).expect("read install.sh");
        assert!(install.contains("memd-capture"));
        assert!(install.contains("memd-hook-capture"));

        fs::remove_dir_all(dir).expect("cleanup hook temp dir");
    }

    #[test]
    fn writes_command_catalog_markdown_into_bundle_root() {
        let dir = std::env::temp_dir().join(format!("memd-command-docs-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create command docs bundle");

        write_bundle_command_catalog_files(&dir).expect("write command catalog");

        let commands = fs::read_to_string(dir.join("COMMANDS.md")).expect("read command catalog");
        assert!(commands.contains("# memd commands"));
        assert!(commands.contains("/memory"));
        assert!(commands.contains("$gsd-autonomous"));

        fs::remove_dir_all(dir).expect("cleanup command docs bundle");
    }

    #[test]
    fn codex_pack_manifest_exposes_recall_capture_cache_and_files() {
        let bundle_root =
            std::env::temp_dir().join(format!("memd-codex-pack-test-{}", uuid::Uuid::new_v4()));
        let manifest =
            crate::harness::codex::build_codex_harness_pack(&bundle_root, "demo", "main");

        assert_eq!(manifest.agent, "codex");
        assert!(
            manifest
                .files
                .iter()
                .any(|path| path.ends_with("CODEX_WAKEUP.md"))
        );
        assert!(
            manifest
                .files
                .iter()
                .any(|path| path.ends_with("CODEX_MEMORY.md"))
        );
        assert!(
            manifest.commands.iter().any(|cmd| {
                cmd.contains("memd wake --output .memd --intent current_task --write")
            })
        );
        assert!(
            manifest
                .commands
                .iter()
                .any(|cmd| cmd.contains("memd hook capture --output .memd"))
        );
        assert!(
            manifest
                .behaviors
                .iter()
                .any(|line| line.contains("turn-scoped cache"))
        );

        let markdown = render_codex_harness_pack_markdown(&manifest);
        assert!(markdown.contains("CODEX_WAKEUP.md"));
        assert!(markdown.contains("CODEX_MEMORY.md"));
        assert!(markdown.contains("turn-scoped cache"));
        assert!(markdown.contains("memd hook capture --output .memd --stdin --summary"));
    }

    #[test]
    fn claude_code_pack_manifest_exposes_native_bridge_and_files() {
        let bundle_root = std::env::temp_dir().join(format!(
            "memd-claude-code-pack-test-{}",
            uuid::Uuid::new_v4()
        ));
        let manifest = crate::harness::claude_code::build_claude_code_harness_pack(
            &bundle_root,
            "demo",
            "main",
        );

        assert_eq!(manifest.agent, "claude-code");
        assert!(
            manifest
                .files
                .iter()
                .any(|path| path.ends_with("CLAUDE_CODE_WAKEUP.md"))
        );
        assert!(
            manifest
                .files
                .iter()
                .any(|path| path.ends_with("CLAUDE_CODE_MEMORY.md"))
        );
        assert!(
            manifest
                .files
                .iter()
                .any(|path| path.ends_with("CLAUDE_IMPORTS.md"))
        );
        assert!(
            manifest
                .commands
                .iter()
                .any(|cmd| cmd.contains("memd lookup --output .memd"))
        );
        assert!(
            manifest
                .behaviors
                .iter()
                .any(|line| line.contains("native Claude import bridge"))
        );

        let markdown = render_claude_code_harness_pack_markdown(&manifest);
        assert!(markdown.contains("CLAUDE_CODE_WAKEUP.md"));
        assert!(markdown.contains("CLAUDE_IMPORTS.md"));
        assert!(markdown.contains("native Claude import bridge"));
        assert!(markdown.contains("memd lookup --output .memd --query"));
    }

    #[test]
    fn command_catalog_includes_slash_and_skill_commands() {
        let bundle_root = std::env::temp_dir().join(format!(
            "memd-command-catalog-test-{}",
            uuid::Uuid::new_v4()
        ));
        let catalog = crate::command_catalog::build_command_catalog(&bundle_root);

        assert!(catalog.commands.iter().any(|entry| entry.name == "/memory"));
        assert!(
            catalog
                .commands
                .iter()
                .any(|entry| entry.name == "$gsd-autonomous")
        );
        assert!(
            catalog
                .commands
                .iter()
                .any(|entry| entry.name == ".memd/agents/claude-code.sh")
        );

        let summary = render_command_catalog_summary(&catalog, None);
        assert!(summary.contains("commands root="));
        assert!(summary.contains("commands="));
        assert!(summary.contains("/memory"));

        let markdown = render_command_catalog_markdown(&catalog);
        assert!(markdown.contains("# memd commands"));
        assert!(markdown.contains("## Native memd CLI"));
        assert!(markdown.contains("## Bridge surfaces"));
        assert!(markdown.contains("## Bundle helpers"));
        assert!(markdown.contains("/memory"));
        assert!(markdown.contains("$gsd-autonomous"));
        assert!(markdown.contains("bundle-root-present"));
        assert!(markdown.contains("codex-skill-installed"));
        assert!(markdown.contains(".memd/agents/claude-code.sh"));
    }

    #[test]
    fn openclaw_pack_manifest_exposes_context_spill_cache_and_files() {
        let bundle_root =
            std::env::temp_dir().join(format!("memd-openclaw-pack-test-{}", uuid::Uuid::new_v4()));
        let manifest =
            crate::harness::openclaw::build_openclaw_harness_pack(&bundle_root, "demo", "main");

        assert_eq!(manifest.agent, "openclaw");
        assert!(
            manifest
                .files
                .iter()
                .any(|path| path.ends_with("OPENCLAW_WAKEUP.md"))
        );
        assert!(
            manifest
                .files
                .iter()
                .any(|path| path.ends_with("OPENCLAW_MEMORY.md"))
        );
        assert!(manifest.commands.iter().any(|cmd| {
            cmd.contains("memd context --project <project> --agent openclaw --compact")
        }));
        assert!(
            manifest
                .commands
                .iter()
                .any(|cmd| cmd.contains("memd hook spill --output .memd --stdin --apply"))
        );
        assert!(
            manifest
                .behaviors
                .iter()
                .any(|line| line.contains("turn-scoped cache"))
        );

        let markdown = render_openclaw_harness_pack_markdown(&manifest);
        assert!(markdown.contains("OPENCLAW_WAKEUP.md"));
        assert!(markdown.contains("OPENCLAW_MEMORY.md"));
        assert!(markdown.contains("turn-scoped cache"));
        assert!(markdown.contains("memd hook spill --output .memd --stdin --apply"));
    }

    #[test]
    fn harness_registry_exposes_shared_preset_ids_and_defaults() {
        let registry = crate::harness::preset::HarnessPresetRegistry::default_registry();

        assert!(registry.packs.iter().any(|pack| pack.pack_id == "codex"));
        assert!(registry.packs.iter().any(|pack| pack.pack_id == "openclaw"));
        assert!(registry.packs.iter().any(|pack| pack.pack_id == "hermes"));
        assert!(registry.packs.iter().any(|pack| pack.pack_id == "opencode"));
        assert!(
            registry
                .packs
                .iter()
                .any(|pack| pack.pack_id == "agent-zero")
        );

        let codex = registry
            .get("codex")
            .expect("codex preset should exist in the shared registry");
        assert_eq!(codex.default_verbs, vec!["wake", "resume", "checkpoint"]);
    }

    #[tokio::test]
    async fn codex_pack_refreshes_wakeup_and_memory_files_after_capture() {
        let bundle_root =
            std::env::temp_dir().join(format!("memd-codex-refresh-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&bundle_root).expect("create bundle root");
        let snapshot = codex_test_snapshot("demo", "main", "codex");
        let manifest =
            crate::harness::codex::build_codex_harness_pack(&bundle_root, "demo", "main");

        let written = refresh_harness_pack_files(
            &bundle_root,
            &snapshot,
            &manifest,
            "codex",
            "refresh",
            &harness_pack_query_from_snapshot(&snapshot),
        )
        .await
        .expect("refresh codex pack files");

        assert!(written.iter().any(|path| path.ends_with("MEMD_WAKEUP.md")));
        assert!(written.iter().any(|path| path.ends_with("MEMD_MEMORY.md")));
        assert!(
            written
                .iter()
                .any(|path| path_text_ends_with(path, "agents/CODEX_WAKEUP.md"))
        );
        assert!(
            written
                .iter()
                .any(|path| path_text_ends_with(path, "agents/CODEX_MEMORY.md"))
        );
        assert!(bundle_root.join("MEMD_WAKEUP.md").exists());
        assert!(bundle_root.join("MEMD_MEMORY.md").exists());
        assert!(bundle_root.join("agents").join("CODEX_WAKEUP.md").exists());
        assert!(bundle_root.join("agents").join("CODEX_MEMORY.md").exists());

        fs::remove_dir_all(bundle_root).expect("cleanup codex refresh temp dir");
    }

    #[tokio::test]
    async fn openclaw_pack_refreshes_wakeup_and_memory_files_after_capture() {
        let bundle_root =
            std::env::temp_dir().join(format!("memd-openclaw-refresh-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&bundle_root).expect("create bundle root");
        let snapshot = codex_test_snapshot("demo", "main", "openclaw");
        let manifest =
            crate::harness::openclaw::build_openclaw_harness_pack(&bundle_root, "demo", "main");

        let written = refresh_harness_pack_files(
            &bundle_root,
            &snapshot,
            &manifest,
            "openclaw",
            "refresh",
            &harness_pack_query_from_snapshot(&snapshot),
        )
        .await
        .expect("refresh openclaw pack files");

        assert!(written.iter().any(|path| path.ends_with("MEMD_WAKEUP.md")));
        assert!(written.iter().any(|path| path.ends_with("MEMD_MEMORY.md")));
        assert!(
            written
                .iter()
                .any(|path| path.ends_with("agents/OPENCLAW_WAKEUP.md"))
        );
        assert!(
            written
                .iter()
                .any(|path| path.ends_with("agents/OPENCLAW_MEMORY.md"))
        );
        assert!(
            bundle_root
                .join("state")
                .join("openclaw-turn-cache.json")
                .exists()
        );
        assert!(bundle_root.join("MEMD_WAKEUP.md").exists());
        assert!(bundle_root.join("MEMD_MEMORY.md").exists());
        assert!(
            bundle_root
                .join("agents")
                .join("OPENCLAW_WAKEUP.md")
                .exists()
        );
        assert!(
            bundle_root
                .join("agents")
                .join("OPENCLAW_MEMORY.md")
                .exists()
        );

        fs::remove_dir_all(bundle_root).expect("cleanup openclaw refresh temp dir");
    }

    #[tokio::test]
    async fn hermes_pack_refreshes_wakeup_and_memory_files_after_capture() {
        let bundle_root =
            std::env::temp_dir().join(format!("memd-hermes-refresh-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&bundle_root).expect("create bundle root");
        let snapshot = codex_test_snapshot("demo", "main", "hermes");
        let manifest =
            crate::harness::hermes::build_hermes_harness_pack(&bundle_root, "demo", "main");

        let written = refresh_harness_pack_files(
            &bundle_root,
            &snapshot,
            &manifest,
            "hermes",
            "refresh",
            &harness_pack_query_from_snapshot(&snapshot),
        )
        .await
        .expect("refresh hermes pack files");

        assert!(written.iter().any(|path| path.ends_with("MEMD_WAKEUP.md")));
        assert!(written.iter().any(|path| path.ends_with("MEMD_MEMORY.md")));
        assert!(
            written
                .iter()
                .any(|path| path.ends_with("agents/HERMES_WAKEUP.md"))
        );
        assert!(
            written
                .iter()
                .any(|path| path.ends_with("agents/HERMES_MEMORY.md"))
        );
        assert!(bundle_root.join("MEMD_WAKEUP.md").exists());
        assert!(bundle_root.join("MEMD_MEMORY.md").exists());
        assert!(bundle_root.join("agents").join("HERMES_WAKEUP.md").exists());
        assert!(bundle_root.join("agents").join("HERMES_MEMORY.md").exists());

        fs::remove_dir_all(bundle_root).expect("cleanup hermes refresh temp dir");
    }

    #[test]
    fn harness_pack_turn_key_is_stable_for_repeated_recall() {
        for agent in ["codex", "hermes", "openclaw", "opencode", "agent-zero"] {
            let first = harness_pack_turn_key(
                Some("demo"),
                Some("main"),
                Some(agent),
                "full",
                "What did we decide?",
            );
            let second = harness_pack_turn_key(
                Some("demo"),
                Some("main"),
                Some(agent),
                "full",
                "  What    did    we decide?  ",
            );

            assert_eq!(first, second, "turn key should be stable for {agent}");
        }
    }

    #[test]
    fn codex_pack_backend_failure_falls_back_to_local_bundle_truth() {
        let bundle_root =
            std::env::temp_dir().join(format!("memd-codex-local-truth-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&bundle_root).expect("create bundle root");
        fs::write(bundle_root.join("MEMD_WAKEUP.md"), "# local wakeup\n").expect("seed wakeup");
        fs::write(bundle_root.join("MEMD_MEMORY.md"), "# local memory\n").expect("seed memory");

        let wakeup = read_codex_pack_local_markdown(&bundle_root, "MEMD_WAKEUP.md")
            .expect("read wakeup fallback")
            .expect("local wakeup fallback");
        let memory = read_codex_pack_local_markdown(&bundle_root, "MEMD_MEMORY.md")
            .expect("read memory fallback")
            .expect("local memory fallback");

        assert!(wakeup.contains("local wakeup"));
        assert!(memory.contains("local memory"));

        fs::remove_dir_all(bundle_root).expect("cleanup codex fallback temp dir");
    }

    #[test]
    fn codex_pack_docs_cover_operational_flow_and_fallback() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../");
        let setup = fs::read_to_string(repo_root.join("docs/setup.md")).expect("read setup docs");
        let api = fs::read_to_string(repo_root.join("docs/api.md")).expect("read api docs");
        let positioning =
            fs::read_to_string(repo_root.join("docs/oss-positioning.md")).expect("read oss docs");
        let agent_zero = fs::read_to_string(repo_root.join("integrations/agent-zero/README.md"))
            .expect("read agent zero docs");
        let codex = fs::read_to_string(repo_root.join("integrations/codex/README.md"))
            .expect("read codex docs");
        let opencode = fs::read_to_string(repo_root.join("integrations/opencode/README.md"))
            .expect("read opencode docs");
        let hooks = fs::read_to_string(repo_root.join("integrations/hooks/README.md"))
            .expect("read hooks docs");

        assert!(setup.contains("Codex is the first harness pack"));
        assert!(setup.contains("reads compiled memory before the turn"));
        assert!(setup.contains("turn-scoped cache"));
        assert!(setup.contains(".memd/MEMD_WAKEUP.md"));
        assert!(setup.contains(".memd/agents/CODEX_MEMORY.md"));
        assert!(setup.contains("Hermes is the adoption-focused harness pack"));
        assert!(setup.contains("Agent Zero is the zero-friction harness pack"));
        assert!(setup.contains("OpenCode is the shared-lane harness pack"));
        assert!(setup.contains(".memd/agents/hermes.sh"));
        assert!(setup.contains(".memd/agents/agent-zero.sh"));
        assert!(setup.contains(".memd/agents/opencode.sh"));

        assert!(api.contains("bundle-local harness pack flow"));
        assert!(api.contains("memd checkpoint"));
        assert!(api.contains("turn-scoped cache"));
        assert!(api.contains(".memd/MEMD_MEMORY.md"));
        assert!(api.contains("Hermes is the adoption-focused harness pack"));
        assert!(api.contains("Agent Zero is the zero-friction harness pack"));
        assert!(api.contains("OpenCode is the shared-lane harness pack"));

        assert!(positioning.contains("Codex is the first harness pack"));
        assert!(positioning.contains("local-first fallback path"));
        assert!(positioning.contains("Hermes is the adoption-focused harness pack"));
        assert!(positioning.contains("Agent Zero is the zero-friction harness pack"));
        assert!(positioning.contains("OpenCode is the shared-lane harness pack"));

        assert!(codex.contains("turn-first recall/capture pack"));
        assert!(codex.contains("memd hook capture --output .memd --stdin --summary"));
        assert!(codex.contains("Keep using the local bundle markdown"));
        assert!(codex.contains(
            "turn cache is keyed from project, namespace, agent, mode, and normalized query"
        ));
        assert!(codex.contains("Hermes uses the same shared memory core"));

        assert!(agent_zero.contains("zero-friction lane"));
        assert!(agent_zero.contains(".memd/agents/AGENT_ZERO_MEMORY.md"));
        assert!(agent_zero.contains("memd handoff --output .memd --prompt"));

        assert!(opencode.contains("shared continuity plane"));
        assert!(opencode.contains(".memd/agents/OPENCODE_MEMORY.md"));
        assert!(opencode.contains("memd handoff --output .memd --prompt"));

        assert!(hooks.contains("pre-turn read step"));
        assert!(hooks.contains("memd hook capture --stdin --summary"));
        assert!(hooks.contains("existing local bundle truth"));
        assert!(hooks.contains("Hermes is the adoption-focused harness pack"));
        assert!(hooks.contains("Agent Zero is the zero-friction harness pack"));
        assert!(hooks.contains("OpenCode is the shared-lane harness pack"));
    }

    #[test]
    fn init_bootstrap_summarizes_existing_project_files() {
        let project_root =
            std::env::temp_dir().join(format!("memd-init-bootstrap-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(project_root.join(".planning")).expect("create project root");
        fs::write(
            project_root.join("CLAUDE.md"),
            "# project instructions\n\nremember memd",
        )
        .expect("write claude");
        fs::write(
            project_root.join("DESIGN.md"),
            "# design\n\nuse clean typography",
        )
        .expect("write design");
        fs::write(
            project_root.join(".planning").join("STATE.md"),
            "# state\n\nactive",
        )
        .expect("write state");
        fs::write(project_root.join("README.md"), "# demo repo").expect("write readme");

        let args = InitArgs {
            project: None,
            namespace: None,
            global: false,
            project_root: Some(project_root.clone()),
            seed_existing: true,
            agent: "codex".to_string(),
            session: None,
            tab_id: None,
            hive_system: None,
            hive_role: None,
            capability: Vec::new(),
            hive_group: Vec::new(),
            hive_group_goal: None,
            authority: None,
            output: project_root.join(".memd"),
            base_url: "http://127.0.0.1:8787".to_string(),
            rag_url: None,
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            workspace: None,
            visibility: None,
            voice_mode: Some(default_voice_mode()),
            allow_localhost_read_only_fallback: false,
            force: false,
        };

        let bootstrap = build_project_bootstrap_memory(Some(project_root.as_path()), "demo", &args)
            .expect("build bootstrap")
            .expect("bootstrap context");
        assert!(bootstrap.markdown.contains("CLAUDE.md"));
        assert!(bootstrap.markdown.contains("DESIGN.md"));
        assert!(bootstrap.markdown.contains(".planning/STATE.md"));
        assert!(bootstrap.markdown.contains("README.md"));
        assert!(bootstrap.markdown.contains("project instructions"));
        assert!(bootstrap.markdown.contains("clean typography"));
        assert_eq!(bootstrap.registry.project, "demo");
        assert!(
            bootstrap
                .registry
                .sources
                .iter()
                .any(|source| source.path.ends_with("CLAUDE.md"))
        );
        assert!(
            bootstrap
                .registry
                .sources
                .iter()
                .any(|source| source.kind == "design")
        );

        fs::remove_dir_all(project_root).expect("cleanup temp project");
    }

    #[test]
    fn init_bootstrap_writes_source_registry() {
        let project_root =
            std::env::temp_dir().join(format!("memd-init-registry-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(project_root.join(".planning")).expect("create planning");
        fs::write(project_root.join("AGENTS.md"), "# agents\n\nmemd").expect("write agents");
        fs::write(project_root.join("README.md"), "# demo repo").expect("write readme");

        let output = project_root.join(".memd");
        let args = InitArgs {
            project: None,
            namespace: None,
            global: false,
            project_root: Some(project_root.clone()),
            seed_existing: true,
            agent: "codex".to_string(),
            session: None,
            tab_id: None,
            hive_system: None,
            hive_role: None,
            capability: Vec::new(),
            hive_group: Vec::new(),
            hive_group_goal: None,
            authority: None,
            output: output.clone(),
            base_url: "http://127.0.0.1:8787".to_string(),
            rag_url: None,
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            workspace: None,
            visibility: None,
            voice_mode: Some(default_voice_mode()),
            allow_localhost_read_only_fallback: false,
            force: false,
        };

        let bootstrap = build_project_bootstrap_memory(Some(project_root.as_path()), "demo", &args)
            .expect("build bootstrap")
            .expect("bootstrap context");
        write_bundle_source_registry(&output, &bootstrap.registry).expect("write registry");

        let registry_path = bundle_source_registry_path(&output);
        let raw = fs::read_to_string(&registry_path).expect("read registry");
        let registry: BootstrapSourceRegistry = serde_json::from_str(&raw).expect("parse registry");

        assert_eq!(registry.project, "demo");
        assert!(
            registry
                .sources
                .iter()
                .any(|source| source.path == "AGENTS.md")
        );
        assert!(
            registry
                .sources
                .iter()
                .any(|source| source.path == "README.md")
        );
        assert!(registry.sources.len() >= 2);

        fs::remove_dir_all(project_root).expect("cleanup temp project");
    }

    #[test]
    fn refresh_bootstrap_memory_detects_changed_source() {
        let project_root =
            std::env::temp_dir().join(format!("memd-refresh-registry-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(project_root.join(".planning")).expect("create planning");
        fs::write(project_root.join("README.md"), "# demo repo").expect("write readme");

        let output = project_root.join(".memd");
        let args = InitArgs {
            project: None,
            namespace: None,
            global: false,
            project_root: Some(project_root.clone()),
            seed_existing: true,
            agent: "codex".to_string(),
            session: None,
            tab_id: None,
            hive_system: None,
            hive_role: None,
            capability: Vec::new(),
            hive_group: Vec::new(),
            hive_group_goal: None,
            authority: None,
            output: output.clone(),
            base_url: "http://127.0.0.1:8787".to_string(),
            rag_url: None,
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            workspace: None,
            visibility: None,
            voice_mode: Some(default_voice_mode()),
            allow_localhost_read_only_fallback: false,
            force: false,
        };

        let bootstrap = build_project_bootstrap_memory(Some(project_root.as_path()), "demo", &args)
            .expect("build bootstrap")
            .expect("bootstrap context");
        write_bundle_source_registry(&output, &bootstrap.registry).expect("write registry");

        fs::write(project_root.join("README.md"), "# demo repo\n\nchanged")
            .expect("rewrite readme");

        let refreshed = tokio::runtime::Runtime::new()
            .expect("create runtime")
            .block_on(refresh_project_bootstrap_memory(&output))
            .expect("refresh bootstrap")
            .expect("changed sources");

        assert!(refreshed.0.contains("Project source refresh"));
        let registry = refreshed.1;
        assert!(
            registry
                .sources
                .iter()
                .any(|source| source.path == "README.md")
        );
        let refreshed_readme = registry
            .sources
            .iter()
            .find(|source| source.path == "README.md")
            .expect("refreshed readme source");
        let initial_readme = bootstrap
            .registry
            .sources
            .iter()
            .find(|source| source.path == "README.md")
            .expect("initial readme source");
        assert!(refreshed_readme.imported_at >= initial_readme.imported_at);

        fs::remove_dir_all(project_root).expect("cleanup temp project");
    }

    #[test]
    fn capability_registry_detects_bridgeable_superpowers_plugin() {
        let _home_lock = lock_home_mutation();
        let home = std::env::temp_dir().join(format!("memd-cap-registry-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(home.join(".claude")).expect("create claude root");
        fs::create_dir_all(
            home.join(".codex")
                .join("plugins")
                .join("cache")
                .join("claude-plugins-official")
                .join("superpowers")
                .join("5.0.6")
                .join(".codex"),
        )
        .expect("create codex cache");
        fs::write(
            home.join(".claude").join("settings.json"),
            r#"{
  "enabledPlugins": {
    "superpowers@claude-plugins-official": true
  }
}
"#,
        )
        .expect("write settings");
        fs::write(
            home.join(".codex")
                .join("plugins")
                .join("cache")
                .join("claude-plugins-official")
                .join("superpowers")
                .join("5.0.6")
                .join(".codex")
                .join("INSTALL.md"),
            "# install\n",
        )
        .expect("write install doc");

        let original_home = std::env::var_os("HOME");
        unsafe {
            std::env::set_var("HOME", &home);
        }

        let registry = build_bundle_capability_registry(None);
        let record = registry
            .capabilities
            .iter()
            .find(|record| {
                record.harness == "claude"
                    && record.kind == "plugin"
                    && record.name == "superpowers"
            })
            .expect("superpowers plugin record");
        assert_eq!(record.status, "enabled");
        assert_eq!(record.portability_class, "bridgeable");
        assert!(path_text_contains(
            record.bridge_hint.as_deref().unwrap_or_default(),
            ".codex/INSTALL.md"
        ));

        if let Some(value) = original_home {
            unsafe {
                std::env::set_var("HOME", value);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
        fs::remove_dir_all(home).expect("cleanup fake home");
    }

    #[test]
    fn capability_registry_collects_harness_surface_artifacts() {
        let _home_lock = lock_home_mutation();
        let home = std::env::temp_dir().join(format!(
            "memd-cap-registry-artifacts-{}",
            uuid::Uuid::new_v4()
        ));
        let cache_root = home
            .join(".codex")
            .join("plugins")
            .join("cache")
            .join("claude-plugins-official")
            .join("superpowers")
            .join("5.0.6");
        fs::create_dir_all(home.join(".claude")).expect("create claude root");
        fs::create_dir_all(home.join(".claude").join("agents")).expect("create agents dir");
        fs::create_dir_all(home.join(".claude").join("teams")).expect("create teams dir");
        fs::create_dir_all(home.join(".claude").join("hooks")).expect("create hooks dir");
        fs::create_dir_all(home.join(".claude").join("command")).expect("create command dir");
        fs::create_dir_all(cache_root.join("command")).expect("create plugin command dir");
        fs::create_dir_all(cache_root.join("hooks")).expect("create plugin hook dir");
        fs::write(
            home.join(".claude").join("agents").join("ops.md"),
            "# ops agent\n",
        )
        .expect("write agent");
        fs::write(
            home.join(".claude").join("teams").join("platform.md"),
            "# team platform\n",
        )
        .expect("write team");
        fs::write(
            home.join(".claude")
                .join("hooks")
                .join("memd-session-context.js"),
            "module.exports = {};\n",
        )
        .expect("write hook");
        fs::write(
            home.join(".claude").join("command").join("memd.md"),
            "# command\n",
        )
        .expect("write command");
        fs::write(
            home.join(".claude").join("settings.json"),
            r#"{
  "enabledPlugins": {
    "superpowers@claude-plugins-official": true
  }
}
"#,
        )
        .expect("write settings");
        fs::create_dir_all(cache_root.join(".codex")).expect("create cache codex dir");
        fs::write(
            cache_root.join("command").join("plugin.md"),
            "# plugin command\n",
        )
        .expect("write plugin command");
        fs::write(cache_root.join("hooks").join("plugin.mjs"), "export {}\n")
            .expect("write plugin hook");

        let original_home = std::env::var_os("HOME");
        unsafe {
            std::env::set_var("HOME", &home);
        }

        let registry = build_bundle_capability_registry(None);
        let agent = registry
            .capabilities
            .iter()
            .find(|record| {
                record.harness == "claude"
                    && record.kind == "agent"
                    && record.name == "agent:ops.md"
            })
            .expect("claude agent record");
        assert_eq!(agent.portability_class, "universal");
        let team = registry
            .capabilities
            .iter()
            .find(|record| {
                record.harness == "claude"
                    && record.kind == "team"
                    && record.name == "team:platform.md"
            })
            .expect("claude team record");
        assert_eq!(team.portability_class, "universal");
        let hook = registry
            .capabilities
            .iter()
            .find(|record| {
                record.harness == "claude"
                    && record.kind == "hook"
                    && record.name == "hook:memd-session-context.js"
            })
            .expect("claude hook record");
        assert_eq!(hook.portability_class, "universal");
        let command = registry
            .capabilities
            .iter()
            .find(|record| {
                record.harness == "claude"
                    && record.kind == "command"
                    && record.name == "command:memd.md"
            })
            .expect("claude command record");
        assert_eq!(command.portability_class, "universal");
        let plugin_command = registry
            .capabilities
            .iter()
            .find(|record| {
                record.harness == "claude"
                    && record.kind == "command"
                    && record.name == "superpowers:plugin.md"
            })
            .expect("claude plugin command record");
        assert_eq!(plugin_command.status, "discovered");
        assert_eq!(plugin_command.portability_class, "harness-native");
        let plugin_hook = registry
            .capabilities
            .iter()
            .find(|record| {
                record.harness == "claude"
                    && record.kind == "hook"
                    && record.name == "superpowers:plugin.mjs"
            })
            .expect("claude plugin hook record");
        assert_eq!(plugin_hook.status, "discovered");
        assert_eq!(plugin_hook.portability_class, "harness-native");

        if let Some(value) = original_home {
            unsafe {
                std::env::set_var("HOME", value);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
        fs::remove_dir_all(home).expect("cleanup fake home");
    }

    #[test]
    fn capability_registry_collects_openclaw_and_opencode_artifacts() {
        let _home_lock = lock_home_mutation();
        let home = std::env::temp_dir().join(format!(
            "memd-cap-registry-openclaw-opencode-{}",
            uuid::Uuid::new_v4()
        ));
        let openclaw_workspace = home.join(".openclaw").join("workspace");
        let opencode_root = home.join(".config").join("opencode");

        fs::create_dir_all(openclaw_workspace.join("agents")).expect("create openclaw agents");
        fs::create_dir_all(openclaw_workspace.join("teams")).expect("create openclaw teams");
        fs::create_dir_all(openclaw_workspace.join("hooks")).expect("create openclaw hooks");
        fs::create_dir_all(openclaw_workspace.join("command")).expect("create openclaw command");
        fs::create_dir_all(opencode_root.join("agents")).expect("create opencode agents");
        fs::create_dir_all(opencode_root.join("teams")).expect("create opencode teams");
        fs::create_dir_all(opencode_root.join("hooks")).expect("create opencode hooks");
        fs::create_dir_all(opencode_root.join("command")).expect("create opencode command");
        fs::create_dir_all(opencode_root.join("plugins")).expect("create opencode plugins");

        fs::write(
            openclaw_workspace.join("agents").join("shift.md"),
            "# openclaw agent\n",
        )
        .expect("write openclaw agent");
        fs::write(
            openclaw_workspace.join("teams").join("core.md"),
            "# openclaw team\n",
        )
        .expect("write openclaw team");
        fs::write(
            openclaw_workspace
                .join("hooks")
                .join("memd-session-context.js"),
            "module.exports = {};\n",
        )
        .expect("write openclaw hook");
        fs::write(
            openclaw_workspace.join("command").join("memd.md"),
            "# openclaw command\n",
        )
        .expect("write openclaw command");

        fs::write(
            opencode_root.join("agents").join("ops.md"),
            "# opencode agent\n",
        )
        .expect("write opencode agent");
        fs::write(
            opencode_root.join("teams").join("dev.md"),
            "# opencode team\n",
        )
        .expect("write opencode team");
        fs::write(
            opencode_root.join("hooks").join("memd-session-context.js"),
            "module.exports = {};\n",
        )
        .expect("write opencode hook");
        fs::write(
            opencode_root.join("command").join("memd.md"),
            "# opencode command\n",
        )
        .expect("write opencode command");
        fs::write(
            opencode_root.join("plugins").join("memd-plugin.mjs"),
            "export {}\n",
        )
        .expect("write opencode plugin");

        let original_home = std::env::var_os("HOME");
        unsafe {
            std::env::set_var("HOME", &home);
        }

        let registry = build_bundle_capability_registry(None);

        let openclaw_agent = registry
            .capabilities
            .iter()
            .find(|record| {
                record.harness == "openclaw"
                    && record.kind == "agent"
                    && record.name == "agent:shift.md"
            })
            .expect("openclaw agent record");
        assert_eq!(openclaw_agent.portability_class, "harness-native");

        let openclaw_team = registry
            .capabilities
            .iter()
            .find(|record| {
                record.harness == "openclaw"
                    && record.kind == "team"
                    && record.name == "team:core.md"
            })
            .expect("openclaw team record");
        assert_eq!(openclaw_team.portability_class, "harness-native");

        let openclaw_hook = registry
            .capabilities
            .iter()
            .find(|record| {
                record.harness == "openclaw"
                    && record.kind == "hook"
                    && record.name == "hook:memd-session-context.js"
            })
            .expect("openclaw hook record");
        assert_eq!(openclaw_hook.portability_class, "harness-native");

        let openclaw_command = registry
            .capabilities
            .iter()
            .find(|record| {
                record.harness == "openclaw"
                    && record.kind == "command"
                    && record.name == "command:memd.md"
            })
            .expect("openclaw command record");
        assert_eq!(openclaw_command.portability_class, "harness-native");

        let opencode_agent = registry
            .capabilities
            .iter()
            .find(|record| {
                record.harness == "opencode"
                    && record.kind == "agent"
                    && record.name == "agent:ops.md"
            })
            .expect("opencode agent record");
        assert_eq!(opencode_agent.portability_class, "harness-native");

        let opencode_team = registry
            .capabilities
            .iter()
            .find(|record| {
                record.harness == "opencode"
                    && record.kind == "team"
                    && record.name == "team:dev.md"
            })
            .expect("opencode team record");
        assert_eq!(opencode_team.portability_class, "harness-native");

        let opencode_plugin = registry
            .capabilities
            .iter()
            .find(|record| record.harness == "opencode" && record.kind == "plugin")
            .expect("opencode plugin record");
        assert_eq!(opencode_plugin.portability_class, "universal");

        if let Some(value) = original_home {
            unsafe {
                std::env::set_var("HOME", value);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
        fs::remove_dir_all(home).expect("cleanup fake home");
    }

    #[test]
    fn capability_bridges_install_superpowers_into_agents_skills() {
        let _home_lock = lock_home_mutation();
        let home = std::env::temp_dir().join(format!("memd-cap-bridge-{}", uuid::Uuid::new_v4()));
        let cache_root = home
            .join(".codex")
            .join("plugins")
            .join("cache")
            .join("claude-plugins-official")
            .join("superpowers")
            .join("5.0.6");
        fs::create_dir_all(home.join(".claude")).expect("create claude root");
        fs::create_dir_all(cache_root.join("skills")).expect("create skills dir");
        fs::create_dir_all(home.join(".agents").join("skills")).expect("create agents dir");
        fs::write(
            home.join(".claude").join("settings.json"),
            r#"{
  "enabledPlugins": {
    "superpowers@claude-plugins-official": true
  }
}
"#,
        )
        .expect("write settings");
        fs::write(cache_root.join("skills").join("README.md"), "superpowers")
            .expect("write bridge marker");

        let original_home = std::env::var_os("HOME");
        unsafe {
            std::env::set_var("HOME", &home);
        }

        let bridges = apply_capability_bridges();
        let action = bridges
            .actions
            .iter()
            .find(|action| action.harness == "codex" && action.capability == "superpowers")
            .expect("superpowers bridge action");
        assert!(matches!(
            action.status.as_str(),
            "bridged" | "already-bridged"
        ));
        let target = home.join(".agents").join("skills").join("superpowers");
        assert!(target.exists());
        assert!(
            fs::symlink_metadata(&target)
                .expect("read target metadata")
                .file_type()
                .is_symlink()
        );

        if let Some(value) = original_home {
            unsafe {
                std::env::set_var("HOME", value);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
        fs::remove_dir_all(home).expect("cleanup fake home");
    }

    #[test]
    fn capability_bridge_inspection_reports_available_without_mutating_targets() {
        let _home_lock = lock_home_mutation();
        let home = std::env::temp_dir().join(format!("memd-cap-inspect-{}", uuid::Uuid::new_v4()));
        let source = home
            .join(".codex")
            .join("plugins")
            .join("cache")
            .join("claude-plugins-official")
            .join("superpowers")
            .join("5.0.6")
            .join("skills");
        let target = home.join(".agents").join("skills").join("superpowers");

        fs::create_dir_all(&source).expect("create source skills dir");
        fs::create_dir_all(target.parent().expect("target parent")).expect("create target parent");
        fs::write(source.join("README.md"), "superpowers").expect("write source marker");

        let action = inspect_directory_skill_bridge("codex", "superpowers", &source, &target);
        assert_eq!(action.status, "available");
        assert_eq!(action.target_path, target.display().to_string());
        assert!(!target.exists());

        fs::remove_dir_all(home).expect("cleanup fake home");
    }

    #[test]
    fn capability_bridges_install_superpowers_into_opencode_plugin_roots() {
        let _home_lock = lock_home_mutation();
        let home =
            std::env::temp_dir().join(format!("memd-cap-bridge-opencode-{}", uuid::Uuid::new_v4()));
        let cache_root = home
            .join(".codex")
            .join("plugins")
            .join("cache")
            .join("claude-plugins-official")
            .join("superpowers")
            .join("5.0.6");
        let source_opencode_plugins = cache_root.join(".opencode").join("plugins");
        let target_modern = home
            .join(".config")
            .join("opencode")
            .join("plugins")
            .join("superpowers");
        let target_legacy = home.join(".opencode").join("plugins").join("superpowers");

        fs::create_dir_all(home.join(".claude")).expect("create claude root");
        fs::create_dir_all(source_opencode_plugins.join("superpowers"))
            .expect("create source opencode plugin directory");
        fs::write(
            home.join(".claude").join("settings.json"),
            r#"{
  "enabledPlugins": {
    "superpowers@claude-plugins-official": true
  }
}
"#,
        )
        .expect("write settings");
        fs::write(
            source_opencode_plugins
                .join("superpowers")
                .join("memd-plugin.mjs"),
            "export {}\n",
        )
        .expect("write bridge marker");

        let original_home = std::env::var_os("HOME");
        unsafe {
            std::env::set_var("HOME", &home);
        }

        let bridges = apply_capability_bridges();
        let actions: Vec<_> = bridges
            .actions
            .iter()
            .filter(|action| action.harness == "opencode" && action.capability == "superpowers")
            .collect();
        assert_eq!(actions.len(), 2);
        for action in &actions {
            assert!(matches!(
                action.status.as_str(),
                "bridged" | "already-bridged"
            ));
        }
        let summary = render_capability_bridge_summary(&bridges);
        assert!(summary.contains(&target_modern.display().to_string()));
        assert!(summary.contains(&target_legacy.display().to_string()));
        assert!(
            actions
                .iter()
                .any(|action| action.target_path == target_modern.display().to_string())
        );
        assert!(
            actions
                .iter()
                .any(|action| action.target_path == target_legacy.display().to_string())
        );

        for target in [&target_modern, &target_legacy] {
            assert!(target.exists());
            let metadata = fs::symlink_metadata(target).expect("read target metadata");
            assert!(metadata.file_type().is_symlink());
        }

        if let Some(value) = original_home {
            unsafe {
                std::env::set_var("HOME", value);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
        fs::remove_dir_all(home).expect("cleanup fake home");
    }

    #[test]
    fn render_capability_registry_summary_includes_claude_family_bridgeable_records() {
        let registry = CapabilityRegistry {
            generated_at: Utc::now(),
            project_root: None,
            capabilities: vec![
                CapabilityRecord {
                    harness: "claude".to_string(),
                    kind: "skill".to_string(),
                    name: "universal-skill".to_string(),
                    status: "enabled".to_string(),
                    portability_class: "universal".to_string(),
                    source_path: "src/universal.md".to_string(),
                    bridge_hint: None,
                    hash: None,
                    notes: Vec::new(),
                },
                CapabilityRecord {
                    harness: "clawcode".to_string(),
                    kind: "plugin".to_string(),
                    name: "bridge-plugin".to_string(),
                    status: "enabled".to_string(),
                    portability_class: "bridgeable".to_string(),
                    source_path: "src/plugin.md".to_string(),
                    bridge_hint: Some("link-to-plugin".to_string()),
                    hash: None,
                    notes: Vec::new(),
                },
                CapabilityRecord {
                    harness: "clawcode".to_string(),
                    kind: "plugin".to_string(),
                    name: "cl-family-bridgeable".to_string(),
                    status: "enabled".to_string(),
                    portability_class: "claude-family-bridgeable".to_string(),
                    source_path: "src/cl-fam.md".to_string(),
                    bridge_hint: Some("link-to-fork".to_string()),
                    hash: None,
                    notes: Vec::new(),
                },
            ],
        };

        let summary = render_capability_registry_summary(&registry);
        assert!(summary.contains("bridgeable: 2"));
        assert!(summary.contains("### Bridgeable capabilities"));
        assert!(summary.contains("clawcode / plugin / bridge-plugin [bridgeable]"));
        assert!(summary.contains(
            "clawcode / plugin / cl-family-bridgeable [claude-family-bridgeable] -> link-to-fork"
        ));
    }

    #[test]
    fn render_capability_bridge_summary_includes_opencode_targets() {
        let home =
            std::env::temp_dir().join(format!("memd-cap-bridge-summary-{}", uuid::Uuid::new_v4()));
        let registry = CapabilityBridgeRegistry {
            generated_at: Utc::now(),
            actions: vec![
                CapabilityBridgeAction {
                    harness: "opencode".to_string(),
                    capability: "superpowers".to_string(),
                    status: "bridged".to_string(),
                    source_path: home
                        .join(".codex")
                        .join("plugins")
                        .join("cache")
                        .join("claude-plugins-official")
                        .join("superpowers")
                        .join("5.0.6")
                        .join(".opencode")
                        .join("plugins")
                        .join("superpowers")
                        .display()
                        .to_string(),
                    target_path: home
                        .join(".config")
                        .join("opencode")
                        .join("plugins")
                        .join("superpowers")
                        .display()
                        .to_string(),
                    notes: vec!["created native skill bridge".to_string()],
                },
                CapabilityBridgeAction {
                    harness: "opencode".to_string(),
                    capability: "superpowers".to_string(),
                    status: "already-bridged".to_string(),
                    source_path: home
                        .join(".codex")
                        .join("plugins")
                        .join("cache")
                        .join("claude-plugins-official")
                        .join("superpowers")
                        .join("5.0.6")
                        .join(".opencode")
                        .join("plugins")
                        .join("superpowers")
                        .display()
                        .to_string(),
                    target_path: home
                        .join(".opencode")
                        .join("plugins")
                        .join("superpowers")
                        .display()
                        .to_string(),
                    notes: vec!["already-bridged".to_string()],
                },
            ],
        };

        let summary = render_capability_bridge_summary(&registry);
        assert!(summary.contains("## Capability Bridges"));
        assert!(summary.contains("bridged: 1"));
        assert!(summary.contains("already_bridged: 1"));
        assert!(summary.contains("- opencode / superpowers -> "));
        let modern_target = home
            .join(".config")
            .join("opencode")
            .join("plugins")
            .join("superpowers")
            .display()
            .to_string();
        let legacy_target = home
            .join(".opencode")
            .join("plugins")
            .join("superpowers")
            .display()
            .to_string();
        assert!(summary.contains(&modern_target));
        assert!(summary.contains(&legacy_target));
    }

    #[test]
    fn detect_claude_family_harness_roots_finds_clawcode_shape() {
        let home =
            std::env::temp_dir().join(format!("memd-clawcode-home-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(home.join(".claude")).expect("create claude root");
        fs::create_dir_all(home.join(".clawcode")).expect("create clawcode root");
        fs::write(home.join(".claude").join("settings.json"), "{}").expect("write claude settings");
        fs::write(home.join(".clawcode").join("settings.json"), "{}")
            .expect("write clawcode settings");

        let roots = detect_claude_family_harness_roots(&home);
        assert!(roots.iter().any(|root| root.harness == "claude"));
        assert!(roots.iter().any(|root| root.harness == "clawcode"));

        fs::remove_dir_all(home).expect("cleanup fake home");
    }

    #[test]
    fn capability_registry_detects_claude_family_fork_plugin_state() {
        let _home_lock = lock_home_mutation();
        let home = std::env::temp_dir().join(format!("memd-clawcode-cap-{}", uuid::Uuid::new_v4()));
        let cache_root = home
            .join(".codex")
            .join("plugins")
            .join("cache")
            .join("claude-plugins-official")
            .join("superpowers")
            .join("5.0.6")
            .join(".codex");
        fs::create_dir_all(home.join(".clawcode")).expect("create clawcode root");
        fs::create_dir_all(&cache_root).expect("create codex cache");
        fs::write(
            home.join(".clawcode").join("settings.json"),
            r#"{
  "enabledPlugins": {
    "superpowers@claude-plugins-official": true
  }
}
"#,
        )
        .expect("write clawcode settings");
        fs::write(cache_root.join("INSTALL.md"), "# install\n").expect("write install");

        let original_home = std::env::var_os("HOME");
        unsafe {
            std::env::set_var("HOME", &home);
        }

        let registry = build_bundle_capability_registry(None);
        let record = registry
            .capabilities
            .iter()
            .find(|record| record.harness == "clawcode" && record.name == "superpowers")
            .expect("clawcode superpowers record");
        assert_eq!(record.kind, "plugin");
        assert_eq!(record.status, "enabled");
        assert_eq!(record.portability_class, "claude-family-bridgeable");

        if let Some(value) = original_home {
            unsafe {
                std::env::set_var("HOME", value);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
        fs::remove_dir_all(home).expect("cleanup fake home");
    }

    #[test]
    fn init_output_prefers_project_root_when_seeded_from_repo() {
        let project_root =
            std::env::temp_dir().join(format!("memd-init-output-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&project_root).expect("create temp project root");

        let args = InitArgs {
            project: None,
            namespace: None,
            global: false,
            project_root: Some(project_root.clone()),
            seed_existing: true,
            agent: "codex".to_string(),
            session: None,
            tab_id: None,
            hive_system: None,
            hive_role: None,
            capability: Vec::new(),
            hive_group: Vec::new(),
            hive_group_goal: None,
            authority: None,
            output: default_init_output_path(),
            base_url: "http://127.0.0.1:8787".to_string(),
            rag_url: None,
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            workspace: None,
            visibility: None,
            voice_mode: Some(default_voice_mode()),
            allow_localhost_read_only_fallback: false,
            force: false,
        };

        let resolved = resolve_init_output_path(&args, Some(project_root.as_path()));
        assert_eq!(resolved, project_root.join(".memd"));

        fs::remove_dir_all(project_root).expect("cleanup temp project");
    }

    #[test]
    fn init_output_prefers_global_bundle_when_requested() {
        let project_root =
            std::env::temp_dir().join(format!("memd-init-global-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&project_root).expect("create temp project root");

        let args = InitArgs {
            project: None,
            namespace: None,
            global: true,
            project_root: Some(project_root.clone()),
            seed_existing: true,
            agent: "codex".to_string(),
            session: None,
            tab_id: None,
            hive_system: None,
            hive_role: None,
            capability: Vec::new(),
            hive_group: Vec::new(),
            hive_group_goal: None,
            authority: None,
            output: default_init_output_path(),
            base_url: "http://127.0.0.1:8787".to_string(),
            rag_url: None,
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            workspace: None,
            visibility: None,
            voice_mode: Some(default_voice_mode()),
            allow_localhost_read_only_fallback: false,
            force: false,
        };

        let resolved = resolve_init_output_path(&args, Some(project_root.as_path()));
        assert_eq!(resolved, default_global_bundle_root());

        fs::remove_dir_all(project_root).expect("cleanup temp project");
    }

    #[path = "evaluation_runtime_tests_bundle_state.rs"]
    mod evaluation_runtime_tests_bundle_state;

    #[path = "evaluation_runtime_tests_awareness.rs"]
    mod evaluation_runtime_tests_awareness;

    #[test]
    fn render_awareness_entry_line_prefers_first_class_topic_scope_and_task_fields() {
        let now = Utc::now();
        let entry = ProjectAwarenessEntry {
            project_dir: "/tmp/projects/current".to_string(),
            bundle_root: "/tmp/projects/current/.memd".to_string(),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: Some("/tmp/projects/current".to_string()),
            branch: Some("feature/queen".to_string()),
            base_branch: Some("main".to_string()),
            agent: Some("codex".to_string()),
            session: Some("session-live".to_string()),
            tab_id: None,
            effective_agent: Some("codex@session-live".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string()],
            hive_groups: vec!["project:memd".to_string()],
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            presence: "active".to_string(),
            host: None,
            pid: None,
            active_claims: 1,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            topic_claim: Some("Refine hive overlap awareness".to_string()),
            scope_claims: vec![
                "task:queen-bee-awareness".to_string(),
                "crates/memd-client/src/main.rs".to_string(),
            ],
            task_id: Some("queen-bee-awareness".to_string()),
            focus: Some("stale focus fallback".to_string()),
            pressure: Some("stale pressure fallback".to_string()),
            next_recovery: None,
            last_updated: Some(now),
        };

        let line = render_awareness_entry_line(&entry, "current", &entry.bundle_root);
        assert!(line.contains("task=queen-bee-awareness"));
        assert!(line.contains("work=\"Refine hive overlap awareness\""));
        assert!(line.contains("touches=task:queen-bee-awareness,crates/memd-client/src/main.rs"));
    }

    #[test]
    fn render_hive_roster_summary_prefers_worker_names_and_role_lane_task() {
        let response = HiveRosterResponse {
            project: "memd".to_string(),
            namespace: "main".to_string(),
            queen_session: Some("session-queen".to_string()),
            bees: vec![memd_schema::HiveSessionRecord {
                session: "session-lorentz".to_string(),
                tab_id: None,
                agent: Some("codex".to_string()),
                effective_agent: Some("Lorentz@session-lorentz".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("reviewer".to_string()),
                worker_name: Some("Lorentz".to_string()),
                display_name: None,
                role: Some("reviewer".to_string()),
                capabilities: vec!["review".to_string(), "coordination".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: Some("lane-review".to_string()),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                repo_root: Some("/repo".to_string()),
                worktree_root: Some("/repo-review".to_string()),
                branch: Some("review/parser".to_string()),
                base_branch: Some("main".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: None,
                pid: None,
                topic_claim: Some("Review parser handoff".to_string()),
                scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
                task_id: Some("review-parser".to_string()),
                focus: Some("Review overlap guard output".to_string()),
                pressure: None,
                next_recovery: None,
                next_action: Some("Review overlap guard output".to_string()),
                needs_help: false,
                needs_review: true,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: "active".to_string(),
                last_seen: Utc::now(),
            }],
        };

        let summary = render_hive_roster_summary(&response);
        assert!(summary.contains("Lorentz (session-lorentz)"));
        assert!(summary.contains("role=reviewer"));
        assert!(summary.contains("lane=lane-review"));
        assert!(summary.contains("task=review-parser"));
        assert!(summary.contains("caps=review,coordination"));
    }

    #[test]
    fn render_hive_roster_summary_prefers_display_name_for_generic_workers() {
        let response = HiveRosterResponse {
            project: "memd".to_string(),
            namespace: "main".to_string(),
            queen_session: Some("session-queen".to_string()),
            bees: vec![memd_schema::HiveSessionRecord {
                session: "session-6d422e56".to_string(),
                tab_id: None,
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@session-6d422e56".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("codex".to_string()),
                display_name: Some("Codex 6d422e56".to_string()),
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: Some("lane-main".to_string()),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                repo_root: Some("/repo".to_string()),
                worktree_root: Some("/repo".to_string()),
                branch: Some("main".to_string()),
                base_branch: Some("main".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: None,
                pid: None,
                topic_claim: Some("Parser refactor".to_string()),
                scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
                task_id: Some("parser-refactor".to_string()),
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: None,
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: "active".to_string(),
                last_seen: Utc::now(),
            }],
        };

        let summary = render_hive_roster_summary(&response);
        assert!(summary.contains("Codex 6d422e56 (session-6d422e56)"));
        assert!(!summary.contains("- codex (session-6d422e56)"));
    }

    #[test]
    fn cli_parses_hive_follow_subcommand() {
        let cli = Cli::try_parse_from([
            "memd",
            "hive",
            "follow",
            "--output",
            ".memd",
            "--worker",
            "Lorentz",
            "--summary",
        ])
        .expect("hive follow command should parse");

        match cli.command {
            Commands::Hive(args) => match args.command {
                Some(HiveSubcommand::Follow(follow)) => {
                    assert_eq!(follow.output, PathBuf::from(".memd"));
                    assert_eq!(follow.worker.as_deref(), Some("Lorentz"));
                    assert!(follow.summary);
                }
                other => panic!("expected hive follow subcommand, got {other:?}"),
            },
            other => panic!("expected hive command, got {other:?}"),
        }
    }

    #[test]
    fn cli_parses_hive_handoff_subcommand() {
        let cli = Cli::try_parse_from([
            "memd",
            "hive",
            "handoff",
            "--output",
            ".memd",
            "--to-worker",
            "Avicenna",
            "--task-id",
            "parser-refactor",
            "--scope",
            "crates/memd-client/src/main.rs,task:parser-refactor",
            "--summary",
        ])
        .expect("hive handoff command should parse");

        match cli.command {
            Commands::Hive(args) => match args.command {
                Some(HiveSubcommand::Handoff(handoff)) => {
                    assert_eq!(handoff.output, PathBuf::from(".memd"));
                    assert_eq!(handoff.to_worker.as_deref(), Some("Avicenna"));
                    assert_eq!(handoff.task_id.as_deref(), Some("parser-refactor"));
                    assert_eq!(
                        handoff.scope,
                        vec![
                            "crates/memd-client/src/main.rs".to_string(),
                            "task:parser-refactor".to_string()
                        ]
                    );
                    assert!(handoff.summary);
                }
                other => panic!("expected hive handoff subcommand, got {other:?}"),
            },
            other => panic!("expected hive command, got {other:?}"),
        }
    }

    #[test]
    fn cli_parses_hive_follow_watch_subcommand() {
        let cli = Cli::try_parse_from([
            "memd",
            "hive",
            "follow",
            "--output",
            ".memd",
            "--worker",
            "Lorentz",
            "--watch",
            "--interval-secs",
            "2",
        ])
        .expect("hive follow watch command should parse");

        match cli.command {
            Commands::Hive(args) => match args.command {
                Some(HiveSubcommand::Follow(follow)) => {
                    assert_eq!(follow.output, PathBuf::from(".memd"));
                    assert_eq!(follow.worker.as_deref(), Some("Lorentz"));
                    assert!(follow.watch);
                    assert_eq!(follow.interval_secs, 2);
                }
                other => panic!("expected hive follow subcommand, got {other:?}"),
            },
            other => panic!("expected hive command, got {other:?}"),
        }
    }

    #[test]
    fn render_hive_handoff_summary_surfaces_packet_fields() {
        let response = HiveHandoffResponse {
            packet: HiveHandoffPacket {
                from_session: "session-anscombe".to_string(),
                from_worker: Some("Anscombe".to_string()),
                to_session: "session-avicenna".to_string(),
                to_worker: Some("Avicenna".to_string()),
                task_id: Some("parser-refactor".to_string()),
                topic_claim: Some("Parser overlap cleanup".to_string()),
                scope_claims: vec![
                    "task:parser-refactor".to_string(),
                    "crates/memd-client/src/main.rs".to_string(),
                ],
                next_action: Some("Finish overlap guard cleanup".to_string()),
                blocker: Some("render lane is about to converge".to_string()),
                note: Some("Keep render.rs out of scope".to_string()),
                created_at: Utc::now(),
            },
            receipt_kind: "queen_handoff".to_string(),
            receipt_summary: "Handoff to Avicenna (session-avicenna) task=parser-refactor"
                .to_string(),
            message_id: Some("msg-1".to_string()),
            recommended_follow: "memd hive follow --session session-avicenna --summary".to_string(),
        };

        let summary = render_hive_handoff_summary(&response);
        assert!(summary.contains("hive_handoff from=Anscombe (session-anscombe)"));
        assert!(summary.contains("to=Avicenna (session-avicenna)"));
        assert!(summary.contains("task=parser-refactor"));
        assert!(summary.contains("scopes=task:parser-refactor,crates/memd-client/src/main.rs"));
        assert!(summary.contains("receipt_kind=queen_handoff"));
        assert!(
            summary.contains("follow=\"memd hive follow --session session-avicenna --summary\"")
        );
    }

    #[test]
    fn render_hive_follow_summary_surfaces_messages_receipts_and_overlap_risk() {
        let response = HiveFollowResponse {
            current_session: Some("session-current".to_string()),
            target: memd_schema::HiveSessionRecord {
                session: "session-lorentz".to_string(),
                tab_id: None,
                agent: Some("codex".to_string()),
                effective_agent: Some("Lorentz@session-lorentz".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("reviewer".to_string()),
                worker_name: Some("Lorentz".to_string()),
                display_name: None,
                role: Some("reviewer".to_string()),
                capabilities: vec!["review".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: Some("lane-review".to_string()),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                repo_root: Some("/repo".to_string()),
                worktree_root: Some("/repo-review".to_string()),
                branch: Some("review/parser".to_string()),
                base_branch: Some("main".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: None,
                pid: None,
                topic_claim: Some("Review parser handoff".to_string()),
                scope_claims: vec![
                    "task:review-parser".to_string(),
                    "crates/memd-client/src/main.rs".to_string(),
                ],
                task_id: Some("review-parser".to_string()),
                focus: Some("Review overlap guard output".to_string()),
                pressure: None,
                next_recovery: None,
                next_action: Some("Reply with review notes".to_string()),
                needs_help: false,
                needs_review: true,
                handoff_state: None,
                confidence: None,
                risk: Some("medium".to_string()),
                status: "active".to_string(),
                last_seen: Utc::now(),
            },
            work_summary: "Review parser handoff".to_string(),
            touch_points: vec![
                "task:review-parser".to_string(),
                "crates/memd-client/src/main.rs".to_string(),
            ],
            next_action: Some("Reply with review notes".to_string()),
            messages: vec![HiveMessageRecord {
                id: "msg-1".to_string(),
                kind: "note".to_string(),
                from_session: "session-queen".to_string(),
                from_agent: Some("Anscombe".to_string()),
                to_session: "session-lorentz".to_string(),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                content: "Stay on parser review and avoid render.rs.".to_string(),
                created_at: Utc::now(),
                acknowledged_at: None,
            }],
            owned_tasks: vec![HiveTaskRecord {
                task_id: "review-parser".to_string(),
                title: "Review parser handoff".to_string(),
                description: None,
                status: "active".to_string(),
                coordination_mode: "shared_review".to_string(),
                session: Some("session-lorentz".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("Lorentz@session-lorentz".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                claim_scopes: vec!["crates/memd-client/src/main.rs".to_string()],
                help_requested: false,
                review_requested: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }],
            help_tasks: Vec::new(),
            review_tasks: Vec::new(),
            recent_receipts: vec![HiveCoordinationReceiptRecord {
                id: "receipt-1".to_string(),
                kind: "queen_handoff".to_string(),
                actor_session: "session-queen".to_string(),
                actor_agent: Some("Anscombe".to_string()),
                target_session: Some("session-lorentz".to_string()),
                task_id: Some("review-parser".to_string()),
                scope: Some("crates/memd-client/src/main.rs".to_string()),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                summary: "Queen handed off parser review scope to Lorentz.".to_string(),
                created_at: Utc::now(),
            }],
            overlap_risk: Some(
                "confirmed hive overlap: target session session-lorentz already claims crates/memd-client/src/main.rs".to_string(),
            ),
            recommended_action: "coordinate_now".to_string(),
        };

        let summary = render_hive_follow_summary(&response);
        assert!(summary.contains("hive_follow worker=Lorentz session=session-lorentz"));
        assert!(summary.contains("recommended_action=coordinate_now"));
        assert!(summary.contains("overlap_risk=confirmed hive overlap"));
        assert!(summary.contains("## Messages"));
        assert!(summary.contains("Stay on parser review and avoid render.rs."));
        assert!(summary.contains("## Tasks"));
        assert!(summary.contains("owned review-parser status=active"));
        assert!(summary.contains("## Receipts"));
        assert!(summary.contains("queen_handoff actor=Anscombe"));
    }

    #[test]
    fn render_hive_follow_watch_frame_includes_timestamp_and_summary() {
        let response = HiveFollowResponse {
            current_session: Some("session-current".to_string()),
            target: memd_schema::HiveSessionRecord {
                session: "session-lorentz".to_string(),
                tab_id: None,
                agent: Some("codex".to_string()),
                effective_agent: Some("Lorentz@session-lorentz".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("reviewer".to_string()),
                worker_name: Some("Lorentz".to_string()),
                display_name: None,
                role: Some("reviewer".to_string()),
                capabilities: vec!["review".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: Some("lane-review".to_string()),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: None,
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                repo_root: Some("/repo".to_string()),
                worktree_root: Some("/repo-review".to_string()),
                branch: Some("review/parser".to_string()),
                base_branch: Some("main".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                base_url_healthy: Some(true),
                host: None,
                pid: None,
                topic_claim: Some("Review parser handoff".to_string()),
                scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
                task_id: Some("review-parser".to_string()),
                focus: Some("Review overlap guard output".to_string()),
                pressure: None,
                next_recovery: None,
                next_action: Some("Reply with review notes".to_string()),
                needs_help: false,
                needs_review: true,
                handoff_state: None,
                confidence: None,
                risk: Some("medium".to_string()),
                status: "active".to_string(),
                last_seen: Utc::now(),
            },
            work_summary: "Review parser handoff".to_string(),
            touch_points: vec!["crates/memd-client/src/main.rs".to_string()],
            next_action: Some("Reply with review notes".to_string()),
            messages: Vec::new(),
            owned_tasks: Vec::new(),
            help_tasks: Vec::new(),
            review_tasks: Vec::new(),
            recent_receipts: Vec::new(),
            overlap_risk: None,
            recommended_action: "safe_to_continue".to_string(),
        };

        let frame = render_hive_follow_watch_frame(
            &response,
            DateTime::parse_from_rfc3339("2026-04-09T22:30:00Z")
                .expect("parse timestamp")
                .with_timezone(&Utc),
        );
        assert!(frame.contains("== hive follow 2026-04-09T22:30:00+00:00 =="));
        assert!(frame.contains("hive_follow worker=Lorentz session=session-lorentz"));
    }

    #[test]
    fn build_hive_heartbeat_prefers_explicit_worker_name_env() {
        let _env_lock = lock_env_mutation();
        let dir =
            std::env::temp_dir().join(format!("memd-heartbeat-worker-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "openclaw",
  "session": "session-openclaw",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "current_task"
}
"#,
        )
        .expect("write config");

        unsafe {
            std::env::set_var("MEMD_WORKER_NAME", "Openclaw");
        }
        let heartbeat = build_hive_heartbeat(&dir, None).expect("build heartbeat");
        unsafe {
            std::env::remove_var("MEMD_WORKER_NAME");
        }

        assert_eq!(heartbeat.worker_name.as_deref(), Some("Openclaw"));
        assert!(heartbeat.display_name.is_none());

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn build_hive_heartbeat_uses_project_scoped_worker_name_for_generic_agents() {
        let _env_lock = lock_env_mutation();
        let dir = std::env::temp_dir()
            .join(format!("memd-heartbeat-generic-worker-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "memd",
  "namespace": "main",
  "agent": "codex",
  "session": "session-6d422e56",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "current_task"
}
"#,
        )
        .expect("write config");

        let heartbeat = build_hive_heartbeat(&dir, None).expect("build heartbeat");

        assert_eq!(
            heartbeat.worker_name.as_deref(),
            Some("Memd Codex 6d422e56")
        );
        assert!(heartbeat.display_name.is_none());

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[path = "evaluation_runtime_tests_hive_commands.rs"]
    mod evaluation_runtime_tests_hive_commands;

    #[test]
    #[test]
    fn project_awareness_summary_marks_freshness_and_supersession_from_last_updated() {
        let now = Utc::now();
        let response = ProjectAwarenessResponse {
            root: "server:http://127.0.0.1:8787".to_string(),
            current_bundle: "/tmp/projects/current/.memd".to_string(),
            collisions: Vec::new(),
            entries: vec![
                ProjectAwarenessEntry {
                    project_dir: "/tmp/projects/current".to_string(),
                    bundle_root: "/tmp/projects/current/.memd".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("session-fresh".to_string()),
                    tab_id: Some("tab-alpha".to_string()),
                    effective_agent: Some("codex@session-fresh".to_string()),
                    hive_system: Some("codex".to_string()),
                    hive_role: Some("agent".to_string()),
                    capabilities: vec!["memory".to_string()],
                    hive_groups: vec!["project:memd".to_string()],
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: None,
                    visibility: Some("all".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: Some(now),
                },
                ProjectAwarenessEntry {
                    project_dir: "remote".to_string(),
                    bundle_root: "remote:http://127.0.0.1:8787:session-aging".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("session-aging".to_string()),
                    tab_id: Some("tab-beta".to_string()),
                    effective_agent: Some("codex@session-aging".to_string()),
                    hive_system: Some("codex".to_string()),
                    hive_role: Some("agent".to_string()),
                    capabilities: vec!["memory".to_string()],
                    hive_groups: vec!["project:memd".to_string()],
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: None,
                    visibility: Some("all".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: Some(now - chrono::TimeDelta::minutes(10)),
                },
                ProjectAwarenessEntry {
                    project_dir: "remote".to_string(),
                    bundle_root: "remote:http://127.0.0.1:8787:session-superseded".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("session-superseded".to_string()),
                    tab_id: None,
                    effective_agent: Some("codex@session-superseded".to_string()),
                    hive_system: None,
                    hive_role: None,
                    capabilities: vec!["memory".to_string()],
                    hive_groups: Vec::new(),
                    hive_group_goal: None,
                    authority: None,
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "stale".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: None,
                    visibility: Some("all".to_string()),
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: Some(now - chrono::TimeDelta::minutes(9)),
                },
            ],
        };

        let summary = render_project_awareness_summary(&response);
        assert!(summary.contains("memd [current] | presence=active truth=current"));
        assert!(summary.contains("memd [hive-session] | presence=active truth=aging"));
        assert!(summary.contains("! superseded_stale_sessions=1 sessions=session-superseded"));
        assert!(summary.contains("hidden_superseded_stale=1"));
    }

    #[test]
    fn awareness_merge_prefers_fresher_local_session_metadata_over_stale_remote_row() {
        let entries = merge_project_awareness_entries(
            vec![ProjectAwarenessEntry {
                project_dir: "/tmp/projects/current".to_string(),
                bundle_root: "/tmp/projects/current/.memd".to_string(),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                agent: Some("codex".to_string()),
                session: Some("session-2c2c883c".to_string()),
                tab_id: Some("tab-alpha".to_string()),
                effective_agent: Some("codex@session-2c2c883c".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string(), "coordination".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                presence: "active".to_string(),
                host: None,
                pid: None,
                active_claims: 1,
                workspace: Some("memd".to_string()),
                visibility: Some("workspace".to_string()),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: Some("Ship the hive fix".to_string()),
                pressure: Some("Repair awareness".to_string()),
                next_recovery: None,
                last_updated: Some(Utc::now()),
            }],
            vec![ProjectAwarenessEntry {
                project_dir: "remote".to_string(),
                bundle_root: "remote:http://127.0.0.1:8787:session-2c2c883c".to_string(),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                agent: Some("codex".to_string()),
                session: Some("session-2c2c883c".to_string()),
                tab_id: Some("tab-alpha".to_string()),
                effective_agent: Some("codex@session-2c2c883c".to_string()),
                hive_system: None,
                hive_role: None,
                capabilities: vec!["memory".to_string()],
                hive_groups: Vec::new(),
                hive_group_goal: None,
                authority: None,
                base_url: Some("http://127.0.0.1:8787".to_string()),
                presence: "stale".to_string(),
                host: None,
                pid: None,
                active_claims: 0,
                workspace: Some("memd".to_string()),
                visibility: Some("workspace".to_string()),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                last_updated: Some(Utc::now() - chrono::TimeDelta::minutes(10)),
            }],
        );

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].project_dir, "/tmp/projects/current");
        assert_eq!(entries[0].presence, "active");
        assert_eq!(entries[0].hive_system.as_deref(), Some("codex"));
        assert_eq!(entries[0].hive_groups, vec!["project:memd".to_string()]);
    }

    #[test]
    fn awareness_merge_keeps_distinct_sessions_when_remote_rows_are_not_duplicates() {
        let entries = merge_project_awareness_entries(
            vec![ProjectAwarenessEntry {
                project_dir: "/tmp/projects/current".to_string(),
                bundle_root: "/tmp/projects/current/.memd".to_string(),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                agent: Some("codex".to_string()),
                session: Some("session-2c2c883c".to_string()),
                tab_id: Some("tab-alpha".to_string()),
                effective_agent: Some("codex@session-2c2c883c".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string(), "coordination".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                presence: "active".to_string(),
                host: None,
                pid: None,
                active_claims: 1,
                workspace: Some("memd".to_string()),
                visibility: Some("workspace".to_string()),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                last_updated: Some(Utc::now()),
            }],
            vec![ProjectAwarenessEntry {
                project_dir: "remote".to_string(),
                bundle_root: "remote:http://127.0.0.1:8787:session-6d422e56".to_string(),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                agent: Some("codex".to_string()),
                session: Some("session-6d422e56".to_string()),
                tab_id: Some("tab-beta".to_string()),
                effective_agent: Some("codex@session-6d422e56".to_string()),
                hive_system: None,
                hive_role: None,
                capabilities: vec!["memory".to_string()],
                hive_groups: Vec::new(),
                hive_group_goal: None,
                authority: None,
                base_url: Some("http://127.0.0.1:8787".to_string()),
                presence: "active".to_string(),
                host: None,
                pid: None,
                active_claims: 0,
                workspace: Some("memd".to_string()),
                visibility: Some("workspace".to_string()),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                last_updated: Some(Utc::now()),
            }],
        );

        assert_eq!(entries.len(), 2);
        assert!(
            entries
                .iter()
                .any(|entry| entry.session.as_deref() == Some("session-2c2c883c"))
        );
        assert!(
            entries
                .iter()
                .any(|entry| entry.session.as_deref() == Some("session-6d422e56"))
        );
    }

    #[test]
    fn session_collision_warnings_surface_shared_session_reuse() {
        let warnings = session_collision_warnings(&[
            ProjectAwarenessEntry {
                project_dir: "/tmp/projects/a".to_string(),
                bundle_root: "/tmp/projects/a/.memd".to_string(),
                project: Some("a".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                agent: Some("codex".to_string()),
                session: Some("shared-session".to_string()),
                tab_id: Some("tab-a".to_string()),
                effective_agent: Some("codex@shared-session".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string(), "coordination".to_string()],
                hive_groups: vec!["openclaw-stack".to_string()],
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                presence: "active".to_string(),
                host: None,
                pid: None,
                active_claims: 0,
                workspace: Some("initiative-alpha".to_string()),
                visibility: Some("workspace".to_string()),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                last_updated: None,
            },
            ProjectAwarenessEntry {
                project_dir: "/tmp/projects/b".to_string(),
                bundle_root: "/tmp/projects/b/.memd".to_string(),
                project: Some("b".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                agent: Some("claude-code".to_string()),
                session: Some("shared-session".to_string()),
                tab_id: Some("tab-a".to_string()),
                effective_agent: Some("claude-code@shared-session".to_string()),
                hive_system: Some("claude-code".to_string()),
                hive_role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string(), "coordination".to_string()],
                hive_groups: vec!["openclaw-stack".to_string()],
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                base_url: Some("http://127.0.0.1:9797".to_string()),
                presence: "active".to_string(),
                host: None,
                pid: None,
                active_claims: 0,
                workspace: Some("initiative-alpha".to_string()),
                visibility: Some("workspace".to_string()),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                last_updated: None,
            },
        ]);

        assert_eq!(warnings.len(), 1);
        assert!(
            warnings[0].contains("session shared-session tab tab-a in workspace initiative-alpha")
        );
        assert!(warnings[0].contains("2 bundles"));
        assert!(warnings[0].contains("2 agents"));
        assert!(warnings[0].contains("2 endpoints"));
    }

    #[test]
    fn branch_collision_warnings_surface_same_branch_and_worktree_faults() {
        let warnings = branch_collision_warnings(&[
            ProjectAwarenessEntry {
                project_dir: "/tmp/projects/a".to_string(),
                bundle_root: "/tmp/projects/a/.memd".to_string(),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: Some("/tmp/repo".to_string()),
                worktree_root: Some("/tmp/repo".to_string()),
                branch: Some("feature/hive-a".to_string()),
                base_branch: Some("main".to_string()),
                agent: Some("codex".to_string()),
                session: Some("session-a".to_string()),
                tab_id: Some("tab-a".to_string()),
                effective_agent: Some("codex@session-a".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                presence: "active".to_string(),
                host: None,
                pid: None,
                active_claims: 0,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                last_updated: None,
            },
            ProjectAwarenessEntry {
                project_dir: "/tmp/projects/b".to_string(),
                bundle_root: "/tmp/projects/b/.memd".to_string(),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: Some("/tmp/repo".to_string()),
                worktree_root: Some("/tmp/repo".to_string()),
                branch: Some("feature/hive-a".to_string()),
                base_branch: Some("main".to_string()),
                agent: Some("claude-code".to_string()),
                session: Some("session-b".to_string()),
                tab_id: Some("tab-b".to_string()),
                effective_agent: Some("claude-code@session-b".to_string()),
                hive_system: Some("claude-code".to_string()),
                hive_role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                base_url: Some("http://127.0.0.1:9797".to_string()),
                presence: "active".to_string(),
                host: None,
                pid: None,
                active_claims: 0,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                last_updated: None,
            },
        ]);

        assert!(warnings.iter().any(|value| {
            value.contains("unsafe_same_branch repo=/tmp/repo branch=feature/hive-a")
        }));
        assert!(
            warnings
                .iter()
                .any(|value| value.contains("unsafe_same_worktree worktree=/tmp/repo"))
        );
    }

    #[test]
    fn heartbeat_presence_labels_age_bands() {
        assert_eq!(heartbeat_presence_label(Utc::now()), "active");
        assert_eq!(
            heartbeat_presence_label(Utc::now() - chrono::TimeDelta::minutes(5)),
            "stale"
        );
        assert_eq!(
            heartbeat_presence_label(Utc::now() - chrono::TimeDelta::minutes(30)),
            "dead"
        );
    }

    #[test]
    fn render_bundle_heartbeat_summary_surfaces_presence_and_focus() {
        let state = BundleHeartbeatState {
            session: Some("codex-a".to_string()),
            agent: Some("codex".to_string()),
            effective_agent: Some("codex@codex-a".to_string()),
            tab_id: None,
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            worker_name: Some("codex".to_string()),
            display_name: None,
            role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string(), "coordination".to_string()],
            hive_groups: vec!["openclaw-stack".to_string()],
            lane_id: Some("/tmp/demo".to_string()),
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat_model: Some(default_heartbeat_model()),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            workspace: Some("team-alpha".to_string()),
            repo_root: Some("/tmp/demo".to_string()),
            worktree_root: Some("/tmp/demo".to_string()),
            branch: Some("feature/test-bee".to_string()),
            base_branch: Some("main".to_string()),
            visibility: Some("workspace".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            base_url_healthy: Some(true),
            host: Some("workstation".to_string()),
            pid: Some(4242),
            topic_claim: None,
            scope_claims: Vec::new(),
            task_id: None,
            focus: Some("Finish the live heartbeat lane".to_string()),
            pressure: Some("Avoid memory drift".to_string()),
            next_recovery: None,
            next_action: None,
            needs_help: false,
            needs_review: false,
            handoff_state: None,
            confidence: None,
            risk: None,
            status: "live".to_string(),
            last_seen: Utc::now(),
            authority_mode: Some("shared".to_string()),
            authority_degraded: false,
        };

        let summary = render_bundle_heartbeat_summary(&state);
        assert!(summary.contains("heartbeat project=demo"));
        assert!(summary.contains("agent=codex@codex-a"));
        assert!(summary.contains("session=codex-a"));
        assert!(summary.contains("presence=active"));
        assert!(summary.contains("focus=\"Finish the live heartbeat lane\""));
        assert!(summary.contains("pressure=\"Avoid memory drift\""));
    }

    #[test]
    fn render_hive_wire_summary_marks_rebased_live_session() {
        let summary = render_hive_wire_summary(&HiveWireResponse {
            action: "updated".to_string(),
            output: ".memd".to_string(),
            project_root: Some("/tmp/demo".to_string()),
            agent: "codex".to_string(),
            bundle_session: Some("codex-stale".to_string()),
            live_session: Some("codex-fresh".to_string()),
            rebased_from_session: Some("codex-stale".to_string()),
            session: Some("codex-fresh".to_string()),
            tab_id: Some("tab-alpha".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            hive_groups: vec!["project:memd".to_string()],
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            heartbeat: None,
            lane_rerouted: false,
            lane_created: false,
            lane_surface: None,
        });

        assert!(summary.contains("bundle_session=codex-stale"));
        assert!(summary.contains("live_session=codex-fresh"));
        assert!(summary.contains("session=codex-fresh"));
        assert!(summary.contains("rebased_from=codex-stale"));
    }

    #[tokio::test]
    async fn resolve_target_session_bundle_finds_matching_session() {
        let root =
            std::env::temp_dir().join(format!("memd-target-session-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("current");
        let target_project = root.join("target");
        fs::create_dir_all(current_project.join(".memd").join("state")).expect("create current");
        fs::create_dir_all(target_project.join(".memd").join("state")).expect("create target");

        fs::write(
            current_project.join(".memd").join("config.json"),
            r#"{
  "project": "current",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write current config");
        fs::write(
            target_project.join(".memd").join("config.json"),
            r#"{
  "project": "target",
  "agent": "claude-code",
  "session": "claude-b",
  "base_url": "http://127.0.0.1:9797",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write target config");
        fs::write(
            target_project
                .join(".memd")
                .join("state")
                .join("heartbeat.json"),
            serde_json::to_string_pretty(&BundleHeartbeatState {
                session: Some("claude-b".to_string()),
                agent: Some("claude-code".to_string()),
                effective_agent: Some("claude-code@claude-b".to_string()),
                tab_id: None,
                hive_system: Some("claude-code".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("claude-code".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string(), "coordination".to_string()],
                hive_groups: vec!["openclaw-stack".to_string()],
                lane_id: Some(target_project.display().to_string()),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some(default_heartbeat_model()),
                project: Some("target".to_string()),
                namespace: None,
                workspace: Some("research".to_string()),
                repo_root: Some(root.display().to_string()),
                worktree_root: Some(target_project.display().to_string()),
                branch: Some("feature/claude-b".to_string()),
                base_branch: Some("main".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some("http://127.0.0.1:9797".to_string()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(4242),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: Some("Handle the delegated task".to_string()),
                pressure: None,
                next_recovery: None,
                next_action: None,
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: "live".to_string(),
                last_seen: Utc::now(),
                authority_mode: Some("shared".to_string()),
                authority_degraded: false,
            })
            .expect("serialize heartbeat"),
        )
        .expect("write heartbeat");

        let resolved = resolve_target_session_bundle(&current_project.join(".memd"), "claude-b")
            .await
            .expect("resolve target")
            .expect("matching session");
        assert_eq!(resolved.project.as_deref(), Some("target"));
        assert_eq!(resolved.session.as_deref(), Some("claude-b"));
        assert_path_tail(&resolved.bundle_root, &target_project.join(".memd"));

        fs::remove_dir_all(root).expect("cleanup target-session root");
    }

    #[tokio::test]
    async fn claims_acquire_and_release_scope() {
        let dir = std::env::temp_dir().join(format!("memd-claims-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(dir.join("state")).expect("create claims dir");
        let base_url = spawn_mock_runtime_server(MockRuntimeState::default(), false).await;
        fs::write(
            dir.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "agent": "codex",
  "session": "codex-a",
  "workspace": "shared",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "general"
}}
"#,
                base_url
            ),
        )
        .expect("write config");
        fs::write(
            dir.join("state").join("heartbeat.json"),
            serde_json::to_string_pretty(&BundleHeartbeatState {
                session: Some("codex-a".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@codex-a".to_string()),
                tab_id: None,
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("codex".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string(), "coordination".to_string()],
                hive_groups: vec!["openclaw-stack".to_string()],
                lane_id: Some(dir.display().to_string()),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some(default_heartbeat_model()),
                project: Some("demo".to_string()),
                namespace: None,
                workspace: Some("shared".to_string()),
                repo_root: Some(dir.display().to_string()),
                worktree_root: Some(dir.display().to_string()),
                branch: Some("feature/codex-a".to_string()),
                base_branch: Some("main".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some(base_url.clone()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(1111),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: None,
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: "live".to_string(),
                last_seen: Utc::now(),
                authority_mode: Some("shared".to_string()),
                authority_degraded: false,
            })
            .expect("serialize heartbeat"),
        )
        .expect("write heartbeat");

        let acquired = run_claims_command(
            &ClaimsArgs {
                output: dir.clone(),
                acquire: true,
                release: false,
                transfer_to_session: None,
                scope: Some("file:src/main.rs".to_string()),
                ttl_secs: 900,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("acquire claim");
        assert_eq!(acquired.claims.len(), 1);
        assert_eq!(acquired.claims[0].scope, "file:src/main.rs");

        let released = run_claims_command(
            &ClaimsArgs {
                output: dir.clone(),
                acquire: false,
                release: true,
                transfer_to_session: None,
                scope: Some("file:src/main.rs".to_string()),
                ttl_secs: 900,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("release claim");
        assert_eq!(released.claims.len(), 1);
        assert_eq!(released.claims[0].scope, "file:src/main.rs");

        fs::remove_dir_all(dir).expect("cleanup claims dir");
    }

    #[tokio::test]
    async fn claims_transfer_scope_to_target_session() {
        let root =
            std::env::temp_dir().join(format!("memd-claim-transfer-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("current");
        let target_project = root.join("target");
        let current_bundle = current_project.join(".memd");
        let target_bundle = target_project.join(".memd");
        fs::create_dir_all(current_bundle.join("state")).expect("create current claims dir");
        fs::create_dir_all(target_bundle.join("state")).expect("create target claims dir");
        let base_url = spawn_mock_runtime_server(MockRuntimeState::default(), false).await;

        fs::write(
            current_bundle.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "agent": "codex",
  "session": "codex-a",
  "workspace": "shared",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "general"
}}
"#,
                base_url
            ),
        )
        .expect("write current config");
        fs::write(
            current_bundle.join("state").join("heartbeat.json"),
            serde_json::to_string_pretty(&BundleHeartbeatState {
                session: Some("codex-a".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@codex-a".to_string()),
                tab_id: None,
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("codex".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string(), "coordination".to_string()],
                hive_groups: vec!["openclaw-stack".to_string()],
                lane_id: Some(current_project.display().to_string()),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some(default_heartbeat_model()),
                project: Some("demo".to_string()),
                namespace: None,
                workspace: Some("shared".to_string()),
                repo_root: Some(root.display().to_string()),
                worktree_root: Some(current_project.display().to_string()),
                branch: Some("feature/codex-a".to_string()),
                base_branch: Some("main".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some(base_url.clone()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(1111),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: None,
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: "live".to_string(),
                last_seen: Utc::now(),
                authority_mode: Some("shared".to_string()),
                authority_degraded: false,
            })
            .expect("serialize current heartbeat"),
        )
        .expect("write current heartbeat");

        fs::write(
            target_bundle.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "agent": "claude-code",
  "session": "claude-b",
  "workspace": "shared",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "general"
}}
"#,
                base_url
            ),
        )
        .expect("write target config");
        fs::write(
            target_bundle.join("state").join("heartbeat.json"),
            serde_json::to_string_pretty(&BundleHeartbeatState {
                session: Some("claude-b".to_string()),
                agent: Some("claude-code".to_string()),
                effective_agent: Some("claude-code@claude-b".to_string()),
                tab_id: None,
                hive_system: Some("claude-code".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("claude-code".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string(), "coordination".to_string()],
                hive_groups: vec!["openclaw-stack".to_string()],
                lane_id: Some(current_project.display().to_string()),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some(default_heartbeat_model()),
                project: Some("demo".to_string()),
                namespace: None,
                workspace: Some("shared".to_string()),
                repo_root: Some(root.display().to_string()),
                worktree_root: Some(target_project.display().to_string()),
                branch: Some("feature/claude-b".to_string()),
                base_branch: Some("main".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some(base_url.clone()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(2222),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: None,
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: "live".to_string(),
                last_seen: Utc::now(),
                authority_mode: Some("shared".to_string()),
                authority_degraded: false,
            })
            .expect("serialize target heartbeat"),
        )
        .expect("write target heartbeat");

        let acquired = run_claims_command(
            &ClaimsArgs {
                output: current_bundle.clone(),
                acquire: true,
                release: false,
                transfer_to_session: None,
                scope: Some("task:parser-refactor".to_string()),
                ttl_secs: 900,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("acquire claim");
        assert_eq!(acquired.claims[0].session.as_deref(), Some("codex-a"));

        let transferred = run_claims_command(
            &ClaimsArgs {
                output: current_bundle.clone(),
                acquire: false,
                release: false,
                transfer_to_session: Some("claude-b".to_string()),
                scope: Some("task:parser-refactor".to_string()),
                ttl_secs: 900,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("transfer claim");
        assert_eq!(transferred.claims.len(), 1);
        assert_eq!(transferred.claims[0].session.as_deref(), Some("claude-b"));
        assert_eq!(
            transferred.claims[0].effective_agent.as_deref(),
            Some("claude-code@claude-b")
        );

        fs::remove_dir_all(root).expect("cleanup transfer dir");
    }

    #[tokio::test]
    async fn messages_send_and_ack_for_target_session() {
        let root = std::env::temp_dir().join(format!("memd-messages-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("current");
        let target_project = root.join("target");
        let current_bundle = current_project.join(".memd");
        let target_bundle = target_project.join(".memd");
        fs::create_dir_all(&current_bundle).expect("create current bundle");
        fs::create_dir_all(&target_bundle).expect("create target bundle");
        let current_base_url = spawn_mock_hive_server().await;
        let target_base_url = spawn_mock_hive_server().await;

        fs::write(
            current_bundle.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "agent": "codex",
  "session": "codex-a",
  "workspace": "shared",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "general"
}}
"#,
                current_base_url
            ),
        )
        .expect("write config");
        fs::write(
            target_bundle.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "agent": "claude-code",
  "session": "claude-b",
  "workspace": "shared",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "general"
}}
"#,
                target_base_url
            ),
        )
        .expect("write target config");

        let sent = run_messages_command(
            &MessagesArgs {
                output: current_bundle.clone(),
                send: true,
                inbox: false,
                ack: None,
                target_session: Some("claude-b".to_string()),
                kind: Some("handoff".to_string()),
                request_help: false,
                request_review: false,
                assign_scope: None,
                scope: None,
                content: Some("Pick up the parser refactor".to_string()),
                summary: false,
            },
            &current_base_url,
        )
        .await
        .expect("send message");
        assert_eq!(sent.messages.len(), 1);
        assert_eq!(sent.messages[0].to_session, "claude-b");

        let inbox = run_messages_command(
            &MessagesArgs {
                output: target_bundle.clone(),
                send: false,
                inbox: true,
                ack: None,
                target_session: None,
                kind: None,
                request_help: false,
                request_review: false,
                assign_scope: None,
                scope: None,
                content: None,
                summary: false,
            },
            &target_base_url,
        )
        .await
        .expect("read inbox");
        assert_eq!(inbox.messages.len(), 1);
        let message_id = inbox.messages[0].id.clone();

        let acked = run_messages_command(
            &MessagesArgs {
                output: target_bundle.clone(),
                send: false,
                inbox: true,
                ack: Some(message_id),
                target_session: None,
                kind: None,
                request_help: false,
                request_review: false,
                assign_scope: None,
                scope: None,
                content: None,
                summary: false,
            },
            &target_base_url,
        )
        .await
        .expect("ack message");
        assert!(acked.messages[0].acknowledged_at.is_some());

        fs::remove_dir_all(root).expect("cleanup messages dir");
    }

    #[tokio::test]
    async fn messages_send_rejects_colliding_target_session_lane() {
        let root =
            std::env::temp_dir().join(format!("memd-messages-collision-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("current");
        let target_project = root.join("target");
        let current_bundle = current_project.join(".memd");
        let target_bundle = target_project.join(".memd");
        fs::create_dir_all(&current_bundle).expect("create current bundle");
        fs::create_dir_all(&target_bundle).expect("create target bundle");
        fs::create_dir_all(current_project.join(".planning")).expect("create current planning");
        fs::create_dir_all(target_project.join(".planning")).expect("create target planning");
        let current_base_url = spawn_mock_hive_server().await;

        write_test_bundle_config(&current_bundle, &current_base_url);
        fs::write(
            target_bundle.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "claude-code",
  "session": "claude-b",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}}
"#,
                current_base_url
            ),
        )
        .expect("write target config");
        fs::write(current_project.join("README.md"), "# current\n").expect("write readme");
        fs::write(target_project.join("NOTES.md"), "# target\n").expect("write notes");
        init_test_git_repo(&root);
        checkout_test_branch(&root, "feature/hive-shared");

        write_test_bundle_heartbeat(
            &target_bundle,
            &BundleHeartbeatState {
                session: Some("claude-b".to_string()),
                agent: Some("claude-code".to_string()),
                effective_agent: Some("claude-code@claude-b".to_string()),
                tab_id: None,
                hive_system: Some("claude-code".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("claude-code".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["openclaw-stack".to_string()],
                lane_id: Some(root.display().to_string()),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some(default_heartbeat_model()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                repo_root: Some(root.display().to_string()),
                worktree_root: Some(root.display().to_string()),
                branch: Some("feature/hive-shared".to_string()),
                base_branch: Some("master".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some(current_base_url.clone()),
                base_url_healthy: Some(true),
                host: None,
                pid: None,
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: None,
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: "live".to_string(),
                last_seen: Utc::now(),
                authority_mode: Some("shared".to_string()),
                authority_degraded: false,
            },
        );

        let err = run_messages_command(
            &MessagesArgs {
                output: current_bundle.clone(),
                send: true,
                inbox: false,
                ack: None,
                target_session: Some("claude-b".to_string()),
                kind: Some("handoff".to_string()),
                request_help: false,
                request_review: false,
                assign_scope: None,
                scope: None,
                content: Some("do the overlap work".to_string()),
                summary: false,
            },
            &current_base_url,
        )
        .await
        .expect_err("colliding target lane should fail");
        assert!(
            err.to_string()
                .contains("unsafe hive cowork target collision")
        );

        fs::remove_dir_all(root).expect("cleanup messages collision dir");
    }

    #[tokio::test]
    async fn checkpoint_uses_bundle_runtime_base_url_instead_of_cli_default() {
        let dir =
            std::env::temp_dir().join(format!("memd-checkpoint-runtime-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create checkpoint dir");
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
        fs::write(
            dir.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "workspace": "shared",
  "visibility": "workspace",
  "tab_id": "tab-alpha",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}}
"#,
                base_url
            ),
        )
        .expect("write config");

        checkpoint_with_bundle_defaults(
            &CheckpointArgs {
                output: dir.clone(),
                project: None,
                namespace: None,
                workspace: None,
                visibility: None,
                source_path: Some("test".to_string()),
                confidence: Some(0.9),
                ttl_seconds: Some(60),
                tag: vec!["checkpoint".to_string()],
                content: Some("runtime-targeted checkpoint".to_string()),
                input: None,
                stdin: false,
            },
            SHARED_MEMD_BASE_URL,
        )
        .await
        .expect("checkpoint via runtime base url");

        let stored = state.stored.lock().expect("lock stored");
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].content, "runtime-targeted checkpoint");
        assert_eq!(stored[0].project.as_deref(), Some("demo"));
        assert_eq!(stored[0].namespace.as_deref(), Some("main"));
        assert_eq!(stored[0].workspace.as_deref(), Some("shared"));

        fs::remove_dir_all(dir).expect("cleanup checkpoint dir");
    }

    #[tokio::test]
    async fn status_uses_bundle_runtime_base_url_instead_of_cli_default() {
        let dir =
            std::env::temp_dir().join(format!("memd-status-runtime-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(dir.join("hooks")).expect("create hooks dir");
        fs::create_dir_all(dir.join("agents")).expect("create agents dir");
        let state = MockRuntimeState::default();
        {
            let mut tasks = state.task_records.lock().expect("lock task records");
            tasks.push(HiveTaskRecord {
                task_id: "task-1".to_string(),
                title: "exclusive task".to_string(),
                description: None,
                status: "in_progress".to_string(),
                coordination_mode: "exclusive_write".to_string(),
                session: Some("codex-a".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@codex-a".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                claim_scopes: vec!["src/main.rs".to_string()],
                help_requested: true,
                review_requested: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
            tasks.push(HiveTaskRecord {
                task_id: "task-2".to_string(),
                title: "review task".to_string(),
                description: None,
                status: "needs_review".to_string(),
                coordination_mode: "shared_review".to_string(),
                session: Some("codex-b".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@codex-b".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                claim_scopes: Vec::new(),
                help_requested: false,
                review_requested: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }
        let base_url = spawn_mock_runtime_server(state, false).await;
        fs::write(
            dir.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "workspace": "shared",
  "visibility": "workspace",
  "tab_id": "tab-alpha",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}}
"#,
                base_url
            ),
        )
        .expect("write config");
        fs::write(dir.join("env"), "MEMD_BASE_URL=test\n").expect("write env");
        fs::write(dir.join("env.ps1"), "$env:MEMD_BASE_URL='test'\n").expect("write env ps1");
        write_maintain_artifacts(
            &dir,
            &MaintainReport {
                mode: "scan".to_string(),
                receipt_id: Some("receipt-0".to_string()),
                compacted_items: 1,
                refreshed_items: 0,
                repaired_items: 0,
                findings: vec!["baseline".to_string()],
                generated_at: Utc::now() - chrono::TimeDelta::minutes(10),
            },
        )
        .expect("write baseline maintenance artifact");
        write_maintain_artifacts(
            &dir,
            &MaintainReport {
                mode: "compact".to_string(),
                receipt_id: Some("receipt-1".to_string()),
                compacted_items: 3,
                refreshed_items: 1,
                repaired_items: 0,
                findings: vec!["trimmed stale memory".to_string()],
                generated_at: Utc::now(),
            },
        )
        .expect("write maintenance artifact");

        let status = read_bundle_status(&dir, SHARED_MEMD_BASE_URL)
            .await
            .expect("status via runtime base url");
        assert_eq!(
            status
                .get("server")
                .and_then(|value| value.get("status"))
                .and_then(|value| value.as_str()),
            Some("ok")
        );
        assert_eq!(
            status
                .get("server")
                .and_then(|value| value.get("items"))
                .and_then(|value| value.as_u64()),
            Some(1)
        );
        assert!(
            status
                .get("capability_surface")
                .and_then(|value| value.get("discovered"))
                .and_then(|value| value.as_u64())
                .is_some()
        );
        assert_eq!(
            status
                .get("cowork_surface")
                .and_then(|value| value.get("tasks"))
                .and_then(|value| value.as_u64()),
            Some(2)
        );
        assert_eq!(
            status
                .get("cowork_surface")
                .and_then(|value| value.get("inbox_messages"))
                .and_then(|value| value.as_u64()),
            Some(1)
        );
        assert_eq!(
            status
                .get("maintenance_surface")
                .and_then(|value| value.get("mode"))
                .and_then(|value| value.as_str()),
            Some("compact")
        );
        assert_eq!(
            status
                .get("maintenance_surface")
                .and_then(|value| value.get("history_count"))
                .and_then(|value| value.as_u64()),
            Some(2)
        );
        assert_eq!(
            status
                .get("maintenance_surface")
                .and_then(|value| value.get("history_modes"))
                .and_then(|value| value.as_array())
                .map(|values| values.len()),
            Some(2)
        );

        fs::remove_dir_all(dir).expect("cleanup status dir");
    }

    #[tokio::test]
    async fn read_bundle_status_surfaces_localhost_read_only_authority_warning() {
        let dir =
            std::env::temp_dir().join(format!("memd-status-authority-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(dir.join("hooks")).expect("create hooks dir");
        fs::create_dir_all(dir.join("agents")).expect("create agents dir");
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state, false).await;
        fs::write(
            dir.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "workspace": "shared",
  "visibility": "workspace",
  "tab_id": "tab-alpha",
  "base_url": "{}",
  "route": "auto",
  "intent": "current_task",
  "authority_policy": {{
    "shared_primary": true,
    "localhost_fallback_policy": "allow_read_only"
  }},
  "authority_state": {{
    "mode": "localhost_read_only",
    "degraded": true,
    "shared_base_url": "{}",
    "fallback_base_url": "http://127.0.0.1:8787",
    "reason": "tailscale is unavailable"
  }}
}}
"#,
                base_url, SHARED_MEMD_BASE_URL
            ),
        )
        .expect("write config");
        fs::write(dir.join("env"), "MEMD_BASE_URL=test\n").expect("write env");
        fs::write(dir.join("env.ps1"), "$env:MEMD_BASE_URL='test'\n").expect("write env ps1");

        let status = read_bundle_status(&dir, SHARED_MEMD_BASE_URL)
            .await
            .expect("status via runtime base url");
        assert_eq!(
            status.get("authority").and_then(JsonValue::as_str),
            Some("localhost_read_only")
        );
        assert_eq!(
            status.get("shared_primary").and_then(JsonValue::as_bool),
            Some(true)
        );
        assert_eq!(
            status
                .get("localhost_read_only_allowed")
                .and_then(JsonValue::as_bool),
            Some(true)
        );
        assert_eq!(
            status.get("degraded").and_then(JsonValue::as_bool),
            Some(true)
        );
        assert_eq!(
            status.get("shared_base_url").and_then(JsonValue::as_str),
            Some(SHARED_MEMD_BASE_URL)
        );
        assert_eq!(
            status.get("fallback_base_url").and_then(JsonValue::as_str),
            Some("http://127.0.0.1:8787")
        );
        assert!(
            status
                .get("authority_warning")
                .and_then(JsonValue::as_array)
                .is_some_and(|warning| warning
                    .iter()
                    .any(|line| line.as_str() == Some("shared authority unavailable")))
        );
        let defaults = status.get("defaults").expect("defaults present");
        assert_eq!(
            defaults
                .get("authority_policy")
                .and_then(|value| value.get("localhost_fallback_policy"))
                .and_then(JsonValue::as_str),
            Some("allow_read_only")
        );
        assert_eq!(
            defaults
                .get("authority_state")
                .and_then(|value| value.get("mode"))
                .and_then(JsonValue::as_str),
            Some("localhost_read_only")
        );

        fs::remove_dir_all(dir).expect("cleanup status dir");
    }

    #[path = "evaluation_runtime_tests_runtime_io.rs"]
    mod evaluation_runtime_tests_runtime_io;

    #[tokio::test]
