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

    #[test]
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

    #[test]
    fn cli_parses_hive_queen_subcommand() {
        let cli = Cli::try_parse_from([
            "memd",
            "hive",
            "queen",
            "--output",
            ".memd",
            "--deny-session",
            "session-avicenna",
            "--summary",
        ])
        .expect("hive queen command should parse");

        match cli.command {
            Commands::Hive(args) => match args.command {
                Some(HiveSubcommand::Queen(queen)) => {
                    assert_eq!(queen.output, PathBuf::from(".memd"));
                    assert_eq!(queen.deny_session.as_deref(), Some("session-avicenna"));
                    assert!(queen.summary);
                }
                other => panic!("expected hive queen subcommand, got {other:?}"),
            },
            other => panic!("expected hive command, got {other:?}"),
        }
    }

    #[test]
    fn render_hive_queen_summary_surfaces_explicit_actions() {
        let response = HiveQueenResponse {
            queen_session: "session-queen".to_string(),
            suggested_actions: vec![
                "reroute Lorentz off crates/memd-client/src/main.rs".to_string(),
                "retire stale bee session-old".to_string(),
            ],
            action_cards: vec![HiveQueenActionCard {
                action: "reroute".to_string(),
                priority: "high".to_string(),
                target_session: Some("session-lorentz".to_string()),
                target_worker: Some("Lorentz".to_string()),
                task_id: Some("review-parser".to_string()),
                scope: Some("crates/memd-client/src/main.rs".to_string()),
                reason: "shared scope is colliding".to_string(),
                follow_command: Some(
                    "memd hive follow --session session-lorentz --summary".to_string(),
                ),
                deny_command: Some(
                    "memd hive queen --deny-session session-lorentz --summary".to_string(),
                ),
                reroute_command: Some(
                    "memd hive queen --reroute-session session-lorentz --summary".to_string(),
                ),
                retire_command: None,
            }],
            recent_receipts: vec![
                "queen_assign session-lorentz review-parser".to_string(),
                "queen_deny session-avicenna overlap-main-rs".to_string(),
            ],
        };

        let summary = render_hive_queen_summary(&response);
        assert!(summary.contains("queen=session-queen"));
        assert!(summary.contains("reroute Lorentz"));
        assert!(summary.contains("queen_deny session-avicenna"));
        assert!(summary.contains("## Action Cards"));
        assert!(summary.contains("follow=memd hive follow --session session-lorentz --summary"));
        assert!(
            summary.contains("reroute=memd hive queen --reroute-session session-lorentz --summary")
        );
    }

    #[test]
    fn render_hive_board_summary_surfaces_board_sections() {
        let response = HiveBoardResponse {
            queen_session: Some("session-queen".to_string()),
            active_bees: vec![memd_schema::HiveSessionRecord {
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
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: None,
                needs_help: false,
                needs_review: true,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: "active".to_string(),
                last_seen: Utc::now(),
            }],
            blocked_bees: vec!["Avicenna overlap on crates/memd-client/src/main.rs".to_string()],
            stale_bees: vec!["session-old".to_string()],
            review_queue: vec!["review-parser -> Lorentz".to_string()],
            overlap_risks: vec![
                "Lorentz vs Avicenna on crates/memd-client/src/main.rs".to_string(),
            ],
            lane_faults: vec!["lane_fault session-avicenna shared worktree".to_string()],
            recommended_actions: vec!["reroute Avicenna".to_string()],
        };

        let summary = render_hive_board_summary(&response);
        assert!(summary.contains("## Active Bees"));
        assert!(summary.contains("## Review Queue"));
        assert!(summary.contains("## Recommended Actions"));
        assert!(summary.contains("Lorentz (session-lorentz)"));
    }

    #[test]
    fn hive_board_response_includes_dashboard_panels() {
        let response = HiveBoardResponse {
            queen_session: Some("session-queen".to_string()),
            active_bees: vec![memd_schema::HiveSessionRecord {
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
                repo_root: None,
                worktree_root: None,
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
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: None,
                needs_help: false,
                needs_review: true,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: "active".to_string(),
                last_seen: Utc::now(),
            }],
            blocked_bees: vec!["Avicenna overlap".to_string()],
            stale_bees: vec!["session-old".to_string()],
            review_queue: vec!["review-parser -> Lorentz".to_string()],
            overlap_risks: vec!["Lorentz vs Avicenna".to_string()],
            lane_faults: vec!["lane_fault session-avicenna".to_string()],
            recommended_actions: vec!["reroute Avicenna".to_string()],
        };

        let json = serde_json::to_value(&response).expect("serialize board");
        assert!(json.get("active_bees").is_some());
        assert!(json.get("review_queue").is_some());
        assert!(json.get("lane_faults").is_some());
        assert!(json.get("recommended_actions").is_some());
    }

    #[tokio::test]
    async fn run_hive_handoff_command_emits_message_and_receipt_for_target_worker() {
        let dir = std::env::temp_dir().join(format!("memd-hive-handoff-{}", uuid::Uuid::new_v4()));
        let output = dir.join(".memd");
        fs::create_dir_all(&output).expect("create output dir");

        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
        write_test_bundle_config(&output, &base_url);
        fs::write(
            output.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "session-anscombe",
  "workspace": null,
  "visibility": "workspace",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}}
"#,
                base_url
            ),
        )
        .expect("rewrite bundle config");

        {
            let mut sessions = state.session_records.lock().expect("lock session records");
            sessions.push(memd_schema::HiveSessionRecord {
                session: "session-anscombe".to_string(),
                tab_id: None,
                agent: Some("codex".to_string()),
                effective_agent: Some("Anscombe@session-anscombe".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("orchestrator".to_string()),
                worker_name: Some("Anscombe".to_string()),
                display_name: None,
                role: Some("queen".to_string()),
                capabilities: vec!["coordination".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: Some("lane-queen".to_string()),
                hive_group_goal: None,
                authority: Some("coordinator".to_string()),
                heartbeat_model: None,
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                repo_root: None,
                worktree_root: Some("/tmp/queen".to_string()),
                branch: Some("queen".to_string()),
                base_branch: Some("main".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some(base_url.clone()),
                base_url_healthy: Some(true),
                host: None,
                pid: None,
                topic_claim: Some("Parser overlap cleanup".to_string()),
                scope_claims: vec![
                    "task:parser-refactor".to_string(),
                    "crates/memd-client/src/main.rs".to_string(),
                ],
                task_id: Some("parser-refactor".to_string()),
                focus: Some("handoff parser lane".to_string()),
                pressure: None,
                next_recovery: Some("finish overlap guard cleanup".to_string()),
                next_action: Some("finish overlap guard cleanup".to_string()),
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: "active".to_string(),
                last_seen: Utc::now(),
            });
            sessions.push(memd_schema::HiveSessionRecord {
                session: "session-avicenna".to_string(),
                tab_id: None,
                agent: Some("codex".to_string()),
                effective_agent: Some("Avicenna@session-avicenna".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("worker".to_string()),
                worker_name: Some("Avicenna".to_string()),
                display_name: None,
                role: Some("worker".to_string()),
                capabilities: vec!["refactor".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: Some("lane-parser".to_string()),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: None,
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                repo_root: None,
                worktree_root: Some("/tmp/parser".to_string()),
                branch: Some("feature/parser".to_string()),
                base_branch: Some("main".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some(base_url.clone()),
                base_url_healthy: Some(true),
                host: None,
                pid: None,
                topic_claim: Some("Receive parser handoff".to_string()),
                scope_claims: vec!["task:parser-refactor".to_string()],
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
            });
        }

        let response = run_hive_handoff_command(
            &HiveHandoffArgs {
                output: output.clone(),
                to_session: None,
                to_worker: Some("Avicenna".to_string()),
                task_id: Some("parser-refactor".to_string()),
                topic: Some("Parser overlap cleanup".to_string()),
                scope: vec![
                    "task:parser-refactor".to_string(),
                    "crates/memd-client/src/main.rs".to_string(),
                ],
                next_action: Some("Finish overlap guard cleanup".to_string()),
                blocker: Some("render lane is converging".to_string()),
                note: Some("Keep render.rs out of scope".to_string()),
                json: false,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("run hive handoff");

        assert_eq!(response.packet.to_session, "session-avicenna");
        assert_eq!(response.packet.to_worker.as_deref(), Some("Avicenna"));
        assert_eq!(response.packet.task_id.as_deref(), Some("parser-refactor"));
        assert_eq!(
            response.packet.scope_claims,
            vec![
                "task:parser-refactor".to_string(),
                "crates/memd-client/src/main.rs".to_string()
            ]
        );
        assert!(response.message_id.is_some());

        let messages = state.messages.lock().expect("lock runtime messages");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].kind, "handoff");
        assert_eq!(messages[0].to_session, "session-avicenna");
        assert_eq!(messages[0].workspace.as_deref(), Some("shared"));
        assert!(messages[0].content.contains("handoff_packet"));
        assert!(messages[0].content.contains("task=parser-refactor"));

        let receipts = state.receipts.lock().expect("lock runtime receipts");
        assert_eq!(receipts.len(), 1);
        assert_eq!(receipts[0].kind, "queen_handoff");
        assert_eq!(
            receipts[0].target_session.as_deref(),
            Some("session-avicenna")
        );
        assert_eq!(receipts[0].task_id.as_deref(), Some("parser-refactor"));

        fs::remove_dir_all(&dir).expect("cleanup handoff temp dir");
    }

    #[tokio::test]
    async fn hive_handoff_is_visible_in_target_inbox_and_follow_surfaces() {
        let dir = std::env::temp_dir().join(format!(
            "memd-hive-handoff-follow-{}",
            uuid::Uuid::new_v4()
        ));
        let sender_output = dir.join("sender/.memd");
        let target_output = dir.join("target/.memd");
        fs::create_dir_all(&sender_output).expect("create sender output dir");
        fs::create_dir_all(&target_output).expect("create target output dir");

        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
        write_test_bundle_config(&sender_output, &base_url);
        write_test_bundle_config(&target_output, &base_url);
        fs::write(
            sender_output.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "avicenna",
  "session": "session-avicenna",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}}
"#,
                base_url
            ),
        )
        .expect("rewrite sender bundle config");
        fs::write(
            target_output.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "noether",
  "session": "session-noether",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}}
"#,
                base_url
            ),
        )
        .expect("rewrite target bundle config");

        {
            let mut sessions = state.session_records.lock().expect("lock session records");
            sessions.push(memd_schema::HiveSessionRecord {
                session: "session-avicenna".to_string(),
                tab_id: None,
                agent: Some("avicenna".to_string()),
                effective_agent: Some("avicenna@session-avicenna".to_string()),
                hive_system: Some("avicenna".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("Avicenna".to_string()),
                display_name: None,
                role: Some("worker".to_string()),
                capabilities: vec!["coordination".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                lane_id: Some("lane-parser".to_string()),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: None,
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                repo_root: None,
                worktree_root: Some("/tmp/parser".to_string()),
                branch: Some("feature/parser".to_string()),
                base_branch: Some("main".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some(base_url.clone()),
                base_url_healthy: Some(true),
                host: None,
                pid: None,
                topic_claim: Some("Parser handoff".to_string()),
                scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
                task_id: Some("review-parser".to_string()),
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: Some("Send parser handoff".to_string()),
                needs_help: false,
                needs_review: false,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: "active".to_string(),
                last_seen: Utc::now(),
            });
            sessions.push(memd_schema::HiveSessionRecord {
                session: "session-noether".to_string(),
                tab_id: None,
                agent: Some("noether".to_string()),
                effective_agent: Some("noether@session-noether".to_string()),
                hive_system: Some("noether".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("Noether".to_string()),
                display_name: None,
                role: Some("worker".to_string()),
                capabilities: vec!["review".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                lane_id: Some("lane-review".to_string()),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: None,
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                repo_root: None,
                worktree_root: Some("/tmp/review".to_string()),
                branch: Some("review/parser".to_string()),
                base_branch: Some("main".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some(base_url.clone()),
                base_url_healthy: Some(true),
                host: None,
                pid: None,
                topic_claim: Some("Receive parser handoff".to_string()),
                scope_claims: vec!["crates/memd-client/src/main.rs".to_string()],
                task_id: Some("review-parser".to_string()),
                focus: None,
                pressure: None,
                next_recovery: None,
                next_action: Some("Review parser handoff".to_string()),
                needs_help: false,
                needs_review: true,
                handoff_state: None,
                confidence: None,
                risk: None,
                status: "active".to_string(),
                last_seen: Utc::now(),
            });
        }

        let handoff = run_hive_handoff_command(
            &HiveHandoffArgs {
                output: sender_output.clone(),
                to_session: None,
                to_worker: Some("Noether".to_string()),
                task_id: Some("review-parser".to_string()),
                topic: Some("Review parser handoff".to_string()),
                scope: vec!["crates/memd-client/src/main.rs".to_string()],
                next_action: Some("Reply with review notes".to_string()),
                blocker: None,
                note: Some("Stay on parser review.".to_string()),
                json: false,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("run hive handoff");

        assert!(handoff.message_id.is_some());

        let inbox = run_messages_command(
            &MessagesArgs {
                output: target_output.clone(),
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
            &base_url,
        )
        .await
        .expect("read target inbox");
        assert_eq!(inbox.messages.len(), 1);
        assert_eq!(inbox.messages[0].kind, "handoff");
        assert_eq!(inbox.messages[0].to_session, "session-noether");
        assert!(inbox.messages[0].content.contains("task=review-parser"));

        let follow = run_hive_follow_command(&HiveFollowArgs {
            output: target_output.clone(),
            session: Some("session-noether".to_string()),
            worker: None,
            watch: false,
            interval_secs: 5,
            json: false,
            summary: false,
        })
        .await
        .expect("run hive follow");
        assert_eq!(follow.target.session, "session-noether");
        assert_eq!(follow.messages.len(), 1);
        assert_eq!(follow.messages[0].id, inbox.messages[0].id);
        assert_eq!(follow.recent_receipts.len(), 1);
        assert_eq!(follow.recent_receipts[0].kind, "queen_handoff");
        assert_eq!(follow.recommended_action, "watch_and_coordinate");

        fs::remove_dir_all(&dir).expect("cleanup handoff follow temp dir");
    }

    #[tokio::test]
    async fn run_hive_board_command_prunes_retired_stale_bees_from_default_view() {
        let dir =
            std::env::temp_dir().join(format!("memd-hive-board-retire-{}", uuid::Uuid::new_v4()));
        let output = dir.join(".memd");
        fs::create_dir_all(&output).expect("create output dir");

        let state = MockRuntimeState::default();
        {
            let mut sessions = state.session_records.lock().expect("lock session records");
            sessions.push(memd_schema::HiveSessionRecord {
                session: "codex-a".to_string(),
                tab_id: None,
                agent: Some("codex".to_string()),
                effective_agent: Some("Anscombe@codex-a".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("orchestrator".to_string()),
                worker_name: Some("Anscombe".to_string()),
                display_name: None,
                role: Some("queen".to_string()),
                capabilities: vec!["coordination".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: Some("lane-queen".to_string()),
                hive_group_goal: None,
                authority: Some("coordinator".to_string()),
                heartbeat_model: None,
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: Some("feature/queen".to_string()),
                base_branch: Some("main".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: None,
                base_url_healthy: Some(true),
                host: None,
                pid: None,
                topic_claim: Some("Route hive board".to_string()),
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
                status: "active".to_string(),
                last_seen: Utc::now(),
            });
            sessions.push(memd_schema::HiveSessionRecord {
                session: "session-stale".to_string(),
                tab_id: None,
                agent: Some("codex".to_string()),
                effective_agent: Some("Lorentz@session-stale".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("Lorentz".to_string()),
                display_name: None,
                role: Some("worker".to_string()),
                capabilities: vec!["review".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: Some("lane-review".to_string()),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: None,
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: Some("feature/review".to_string()),
                base_branch: Some("main".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: None,
                base_url_healthy: Some(true),
                host: None,
                pid: None,
                topic_claim: Some("Old stale work".to_string()),
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
                status: "active".to_string(),
                last_seen: Utc::now() - chrono::TimeDelta::minutes(45),
            });
        }

        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
        write_test_bundle_config(&output, &base_url);
        let board = run_hive_board_command(
            &HiveArgs {
                command: None,
                agent: None,
                project: None,
                namespace: None,
                global: false,
                project_root: None,
                seed_existing: true,
                session: None,
                tab_id: None,
                hive_system: None,
                hive_role: None,
                capability: Vec::new(),
                hive_group: Vec::new(),
                hive_group_goal: None,
                authority: None,
                output: output.clone(),
                base_url: base_url.clone(),
                rag_url: None,
                route: "auto".to_string(),
                intent: "current_task".to_string(),
                workspace: None,
                visibility: None,
                publish_heartbeat: true,
                force: false,
                summary: true,
            },
            &base_url,
        )
        .await
        .expect("board");

        assert!(board.stale_bees.is_empty());
        let sessions = state.session_records.lock().expect("lock session records");
        assert!(
            sessions
                .iter()
                .all(|session| session.session != "session-stale")
        );

        fs::remove_dir_all(dir).expect("cleanup board retire dir");
    }

    #[test]
    fn build_hive_heartbeat_derives_first_class_intent_fields() {
        let dir =
            std::env::temp_dir().join(format!("memd-heartbeat-intent-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("state")).expect("create temp dir");
        std::fs::write(
            dir.join("state/claims.json"),
            serde_json::to_string_pretty(&SessionClaimsState {
                claims: vec![SessionClaim {
                    scope: "task:queen-bee-awareness".to_string(),
                    session: Some("session-live".to_string()),
                    tab_id: None,
                    agent: Some("codex".to_string()),
                    effective_agent: Some("codex@session-live".to_string()),
                    project: Some("memd".to_string()),
                    workspace: Some("shared".to_string()),
                    host: None,
                    pid: None,
                    acquired_at: Utc::now(),
                    expires_at: Utc::now() + chrono::TimeDelta::minutes(15),
                }],
            })
            .expect("serialize claims"),
        )
        .expect("write claims");

        let snapshot = BundleResumeState {
            focus: Some("Refine hive overlap awareness".to_string()),
            pressure: Some(
                "file_edited: crates/memd-client/src/main.rs | scope=task:queen-bee-awareness"
                    .to_string(),
            ),
            next_recovery: Some("publish overlap-safe hive quickview".to_string()),
            lane: None,
            working_records: 0,
            inbox_items: 0,
            rehydration_items: 0,
            recorded_at: Utc::now(),
        };
        std::fs::write(
            dir.join("state/last-resume.json"),
            serde_json::to_string_pretty(&snapshot).expect("serialize resume"),
        )
        .expect("write resume");

        let heartbeat = build_hive_heartbeat(&dir, None).expect("build heartbeat");
        assert_eq!(
            heartbeat.topic_claim.as_deref(),
            Some("Refine hive overlap awareness")
        );
        assert!(
            heartbeat
                .scope_claims
                .iter()
                .any(|scope| scope == "task:queen-bee-awareness")
        );
        assert!(
            heartbeat
                .scope_claims
                .iter()
                .any(|scope| scope == "crates/memd-client/src/main.rs")
        );
        assert_eq!(heartbeat.task_id.as_deref(), Some("queen-bee-awareness"));

        std::fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn derive_hive_display_name_uses_session_for_generic_agents() {
        assert_eq!(
            derive_hive_display_name(Some("codex"), Some("session-6d422e56")).as_deref(),
            Some("Codex 6d422e56")
        );
        assert_eq!(
            derive_hive_display_name(Some("claude-code"), Some("codex-fresh")).as_deref(),
            Some("Claude fresh")
        );
        assert_eq!(
            derive_hive_display_name(Some("Lorentz"), Some("session-x")),
            None
        );
    }

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

    #[tokio::test]
    async fn resolve_bootstrap_authority_requires_explicit_localhost_fallback_consent() {
        let state = MockRuntimeState::default();
        let localhost_fallback_base_url = spawn_mock_runtime_server(state, false).await;
        let original = std::env::var_os("MEMD_LOCALHOST_FALLBACK_BASE_URL");
        unsafe {
            std::env::set_var(
                "MEMD_LOCALHOST_FALLBACK_BASE_URL",
                &localhost_fallback_base_url,
            );
        }

        let result = resolve_bootstrap_authority(InitArgs {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            global: false,
            project_root: None,
            seed_existing: false,
            agent: "codex".to_string(),
            session: Some("codex-a".to_string()),
            tab_id: None,
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            capability: vec!["memory".to_string()],
            hive_group: vec!["project:demo".to_string()],
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            output: std::env::temp_dir()
                .join(format!("memd-bootstrap-authority-{}", uuid::Uuid::new_v4())),
            base_url: "http://memd.invalid:8787".to_string(),
            rag_url: None,
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            voice_mode: Some(default_voice_mode()),
            force: false,
            allow_localhost_read_only_fallback: false,
        })
        .await;

        if let Some(value) = original {
            unsafe {
                std::env::set_var("MEMD_LOCALHOST_FALLBACK_BASE_URL", value);
            }
        } else {
            unsafe {
                std::env::remove_var("MEMD_LOCALHOST_FALLBACK_BASE_URL");
            }
        }

        let err = result.expect_err("missing consent should block localhost fallback");
        assert!(
            err.to_string()
                .contains("--allow-localhost-read-only-fallback")
        );
        assert!(err.to_string().contains(&localhost_fallback_base_url));
    }

    #[test]
    fn read_previous_maintain_report_uses_latest_timestamped_report() {
        let dir =
            std::env::temp_dir().join(format!("memd-maintain-history-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        let maintain_dir = dir.join("maintenance");
        fs::create_dir_all(&maintain_dir).expect("create maintenance dir");
        fs::write(
            maintain_dir.join("20260409T120000Z.json"),
            serde_json::to_string_pretty(&MaintainReport {
                mode: "compact".to_string(),
                receipt_id: Some("receipt-older".to_string()),
                compacted_items: 1,
                refreshed_items: 0,
                repaired_items: 0,
                findings: vec!["older".to_string()],
                generated_at: Utc::now(),
            })
            .expect("serialize older"),
        )
        .expect("write older report");
        fs::write(
            maintain_dir.join("20260409T130000Z.json"),
            serde_json::to_string_pretty(&MaintainReport {
                mode: "auto".to_string(),
                receipt_id: Some("receipt-newer".to_string()),
                compacted_items: 4,
                refreshed_items: 1,
                repaired_items: 1,
                findings: vec!["newer".to_string()],
                generated_at: Utc::now(),
            })
            .expect("serialize newer"),
        )
        .expect("write newer report");
        fs::write(
            maintain_dir.join("latest.json"),
            serde_json::to_string_pretty(&MaintainReport {
                mode: "auto".to_string(),
                receipt_id: Some("receipt-latest-link".to_string()),
                compacted_items: 4,
                refreshed_items: 1,
                repaired_items: 1,
                findings: vec!["latest".to_string()],
                generated_at: Utc::now(),
            })
            .expect("serialize latest"),
        )
        .expect("write latest report");

        let report = read_previous_maintain_report(&dir)
            .expect("read previous maintain report")
            .expect("expected previous maintain report");

        assert_eq!(report.receipt_id.as_deref(), Some("receipt-newer"));
        assert_eq!(report.mode, "auto");
        assert_eq!(report.compacted_items, 4);

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[tokio::test]
    async fn read_bundle_status_emits_truth_summary() {
        let dir = std::env::temp_dir().join(format!("memd-status-truth-{}", uuid::Uuid::new_v4()));
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

        let status = read_bundle_status(&dir, SHARED_MEMD_BASE_URL)
            .await
            .expect("status via runtime base url");
        let truth = status.get("truth_summary").expect("truth summary present");
        assert_eq!(
            truth.get("retrieval_tier").and_then(JsonValue::as_str),
            Some("working")
        );
        assert!(
            truth
                .get("records")
                .and_then(JsonValue::as_array)
                .is_some_and(|records| !records.is_empty())
        );
        assert!(
            truth
                .get("source_count")
                .and_then(JsonValue::as_u64)
                .is_some()
        );

        fs::remove_dir_all(dir).expect("cleanup status dir");
    }

    #[tokio::test]
    async fn read_bundle_status_surfaces_evolution_summary() {
        let dir =
            std::env::temp_dir().join(format!("memd-status-evolution-{}", uuid::Uuid::new_v4()));
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

        let mut report = test_experiment_report(&dir, true, false, 96, 100, Utc::now());
        report.composite.scenario = Some("self_evolution".to_string());
        report.improvement.final_changes =
            vec!["retune pass/fail gate for self-evolution proposals".to_string()];
        write_experiment_artifacts(&dir, &report).expect("write experiment artifacts");

        let status = read_bundle_status(&dir, SHARED_MEMD_BASE_URL)
            .await
            .expect("status via runtime base url");
        let evolution = status.get("evolution").expect("evolution summary present");
        assert_eq!(
            evolution.get("proposal_state").and_then(JsonValue::as_str),
            Some("accepted_proposal")
        );
        assert_eq!(
            evolution.get("scope_class").and_then(JsonValue::as_str),
            Some("runtime_policy")
        );
        assert_eq!(
            evolution.get("scope_gate").and_then(JsonValue::as_str),
            Some("auto_merge")
        );

        fs::remove_dir_all(dir).expect("cleanup status dir");
    }

    #[tokio::test]
    async fn write_bundle_heartbeat_times_out_slow_hive_publish() {
        let dir =
            std::env::temp_dir().join(format!("memd-heartbeat-timeout-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create heartbeat dir");
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), true).await;
        fs::write(
            dir.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "hive_system": "codex",
  "hive_role": "agent",
  "hive_groups": ["openclaw-stack", "runtime-core"],
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

        let started = std::time::Instant::now();
        write_bundle_heartbeat(&dir, None, false)
            .await
            .expect("write heartbeat");
        assert!(started.elapsed() < std::time::Duration::from_secs(3));

        let heartbeat = read_bundle_heartbeat(&dir)
            .expect("read heartbeat")
            .expect("heartbeat present");
        assert_eq!(heartbeat.session.as_deref(), Some("codex-a"));
        assert_eq!(heartbeat.base_url.as_deref(), Some(base_url.as_str()));
        assert!(
            heartbeat
                .hive_groups
                .iter()
                .any(|group| group == "project:demo")
        );

        let session_upserts = state.session_upserts.lock().expect("lock session upserts");
        assert_eq!(session_upserts.len(), 1);
        assert!(
            session_upserts[0]
                .hive_groups
                .iter()
                .any(|group| group == "project:demo")
        );
        assert_eq!(
            session_upserts[0].worker_name.as_deref(),
            Some("Demo Codex a")
        );
        assert_eq!(session_upserts[0].role.as_deref(), Some("agent"));

        fs::remove_dir_all(dir).expect("cleanup heartbeat dir");
    }

    #[tokio::test]
    async fn write_bundle_heartbeat_retires_superseded_stale_sessions() {
        let dir =
            std::env::temp_dir().join(format!("memd-heartbeat-retire-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create heartbeat dir");
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
        state
            .session_records
            .lock()
            .expect("lock session records")
            .push(memd_schema::HiveSessionRecord {
                session: "codex-stale".to_string(),
                tab_id: Some("tab-alpha".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@codex-stale".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("codex".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some(default_heartbeat_model()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some(base_url.clone()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(4242),
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
                last_seen: Utc::now() - chrono::TimeDelta::minutes(8),
            });
        fs::write(
            dir.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-fresh",
  "hive_system": "codex",
  "hive_role": "agent",
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

        write_bundle_heartbeat(&dir, None, false)
            .await
            .expect("write heartbeat");

        let retires = state.session_retires.lock().expect("lock session retires");
        assert_eq!(retires.len(), 1);
        assert_eq!(retires[0].session, "codex-stale");
        assert_eq!(retires[0].agent.as_deref(), Some("codex"));
        drop(retires);

        let records = state.session_records.lock().expect("lock session records");
        assert!(records.iter().all(|record| record.session != "codex-stale"));

        fs::remove_dir_all(dir).expect("cleanup heartbeat dir");
    }

    #[tokio::test]
    async fn write_bundle_memory_files_surfaces_hive_state_in_compiled_memory_pages() {
        let dir =
            std::env::temp_dir().join(format!("memd-memory-hive-pages-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp dir");
        let state = MockRuntimeState::default();
        push_mock_runtime_hive_session(
            &state,
            "queen-1",
            "Avicenna",
            "queen",
            Some("queen-routing"),
            Some("Route hive work"),
            vec!["docs/hive.md".to_string()],
        );
        push_mock_runtime_hive_session(
            &state,
            "bee-1",
            "Lorentz",
            "worker",
            Some("parser-refactor"),
            Some("Parser lane refactor"),
            vec![
                "task:parser-refactor".to_string(),
                "crates/memd-client/src/main.rs".to_string(),
            ],
        );
        state
            .task_records
            .lock()
            .expect("lock task records")
            .push(HiveTaskRecord {
                task_id: "parser-refactor".to_string(),
                title: "Refine parser overlap flow".to_string(),
                description: None,
                status: "active".to_string(),
                coordination_mode: "exclusive_write".to_string(),
                session: Some("bee-1".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@bee-1".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                claim_scopes: vec!["crates/memd-client/src/main.rs".to_string()],
                help_requested: false,
                review_requested: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        state
            .receipts
            .lock()
            .expect("lock receipts")
            .push(HiveCoordinationReceiptRecord {
                id: "receipt-queen-deny".to_string(),
                kind: "queen_deny".to_string(),
                actor_session: "queen-1".to_string(),
                actor_agent: Some("dashboard".to_string()),
                target_session: Some("bee-1".to_string()),
                task_id: Some("parser-refactor".to_string()),
                scope: Some("crates/memd-client/src/main.rs".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                summary: "Queen denied overlapping lane or scope work for session bee-1."
                    .to_string(),
                created_at: Utc::now(),
            });
        state
            .messages
            .lock()
            .expect("lock messages")
            .push(HiveMessageRecord {
                id: "msg-handoff".to_string(),
                kind: "handoff".to_string(),
                from_session: "queen-1".to_string(),
                from_agent: Some("dashboard".to_string()),
                to_session: "bee-1".to_string(),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                content: "handoff_scope: crates/memd-client/src/main.rs".to_string(),
                created_at: Utc::now(),
                acknowledged_at: None,
            });
        let base_url = spawn_mock_runtime_server(state, false).await;
        fs::write(
            dir.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "bee-1",
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

        let snapshot = codex_test_snapshot("demo", "main", "codex");
        write_bundle_memory_files(&dir, &snapshot, None, false)
            .await
            .expect("write bundle memory files");

        let memory =
            fs::read_to_string(dir.join("MEMD_MEMORY.md")).expect("read generated bundle memory");
        assert!(memory.contains("## Hive"));
        assert!(memory.contains("queen=queen-1"));
        assert!(memory.contains("active_bees=Avicenna(queen-1)/queen-routing"));
        assert!(memory.contains("focus=Lorentz"));

        let workspace_page = fs::read_to_string(dir.join("compiled/memory/workspace.md"))
            .expect("read workspace page");
        assert!(workspace_page.contains("## Hive"));
        assert!(workspace_page.contains("bee Lorentz (bee-1)"));

        let workspace_key = memory_object_lane_item_key(&snapshot, MemoryObjectLane::Workspace, 0)
            .expect("workspace key");
        let workspace_item_path =
            bundle_compiled_memory_item_path(&dir, MemoryObjectLane::Workspace, 0, &workspace_key);
        let workspace_item_page =
            fs::read_to_string(workspace_item_path).expect("read workspace item page");
        assert!(workspace_item_page.contains("## Hive"));
        assert!(
            workspace_item_page.contains("focus=Lorentz")
                || workspace_item_page.contains("focus=bee-1")
        );

        fs::remove_dir_all(dir).expect("cleanup memory hive page dir");
    }

    #[tokio::test]
    async fn write_bundle_memory_files_prunes_stale_compiled_memory_outputs() {
        let dir = std::env::temp_dir()
            .join(format!("memd-memory-prune-{}", uuid::Uuid::new_v4()));
        let compiled = dir.join("compiled").join("memory");
        let stale_item = compiled.join("items/working/working-99-deadbeef.md");
        let stale_lane = compiled.join("obsolete.md");
        fs::create_dir_all(stale_item.parent().expect("stale item parent"))
            .expect("create stale item dir");
        fs::write(&stale_item, "# stale compiled item\n").expect("write stale item");
        fs::write(&stale_lane, "# stale compiled lane\n").expect("write stale lane");

        let snapshot = codex_test_snapshot("demo", "main", "codex");
        write_bundle_memory_files(&dir, &snapshot, None, false)
            .await
            .expect("write bundle memory files");

        assert!(
            !stale_item.exists(),
            "stale compiled item page should be pruned on rewrite"
        );
        assert!(
            !stale_lane.exists(),
            "stale compiled lane page should be pruned on rewrite"
        );
        assert!(compiled.join("working.md").exists());
        assert!(compiled.join("context.md").exists());

        fs::remove_dir_all(dir).expect("cleanup memory prune dir");
    }

    #[tokio::test]
    async fn retire_hive_session_entry_uses_awareness_identity() {
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
        state
            .session_records
            .lock()
            .expect("lock session records")
            .push(memd_schema::HiveSessionRecord {
                session: "codex-stale".to_string(),
                tab_id: Some("tab-alpha".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@codex-stale".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("codex".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                lane_id: None,
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some(default_heartbeat_model()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some(base_url.clone()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(4242),
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
                last_seen: Utc::now() - chrono::TimeDelta::minutes(8),
            });

        let retired = retire_hive_session_entry(
            &MemdClient::new(&base_url).expect("client"),
            &ProjectAwarenessEntry {
                project_dir: "remote".to_string(),
                bundle_root: format!("remote:{base_url}:codex-stale"),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                agent: Some("codex".to_string()),
                session: Some("codex-stale".to_string()),
                tab_id: Some("tab-alpha".to_string()),
                effective_agent: Some("codex@codex-stale".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string()],
                hive_groups: vec!["project:demo".to_string()],
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                base_url: Some(base_url.clone()),
                presence: "stale".to_string(),
                host: Some("workstation".to_string()),
                pid: Some(4242),
                active_claims: 0,
                workspace: Some("shared".to_string()),
                visibility: Some("workspace".to_string()),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                last_updated: Some(Utc::now() - chrono::TimeDelta::minutes(8)),
            },
            "recovered to codex-fresh",
        )
        .await
        .expect("retire stale entry");
        assert_eq!(retired, 1);

        let retires = state.session_retires.lock().expect("lock session retires");
        assert_eq!(retires.len(), 1);
        assert_eq!(retires[0].session, "codex-stale");
        assert_eq!(
            retires[0].reason.as_deref(),
            Some("recovered to codex-fresh")
        );
    }

    #[tokio::test]
    async fn read_bundle_resume_publishes_resume_state_and_hive_groups() {
        let dir =
            std::env::temp_dir().join(format!("memd-resume-runtime-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create resume dir");
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
  "hive_system": "codex",
  "hive_role": "agent",
  "hive_groups": ["openclaw-stack", "runtime-core"],
  "workspace": "shared",
  "visibility": "workspace",
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

        let snapshot = read_bundle_resume(
            &ResumeArgs {
                output: dir.clone(),
                project: None,
                namespace: None,
                agent: None,
                workspace: None,
                visibility: None,
                route: None,
                intent: Some("current_task".to_string()),
                limit: Some(8),
                rehydration_limit: Some(4),
                semantic: false,
                prompt: false,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("read bundle resume");
        write_bundle_memory_files(&dir, &snapshot, None, false)
            .await
            .expect("write bundle memory files");

        let stored = state.stored.lock().expect("lock stored");
        assert_eq!(stored.len(), 1);
        assert_eq!(
            stored[0].source_system.as_deref(),
            Some("memd-resume-state")
        );
        assert_eq!(stored[0].project.as_deref(), Some("demo"));
        assert_eq!(stored[0].workspace.as_deref(), Some("shared"));
        assert!(stored[0].tags.iter().any(|tag| tag == "resume_state"));
        drop(stored);

        let session_upserts = state.session_upserts.lock().expect("lock session upserts");
        assert!(!session_upserts.is_empty());
        let last = session_upserts.last().expect("session upsert recorded");
        assert_eq!(last.session, "codex-a");
        assert_eq!(
            last.hive_groups,
            vec![
                "openclaw-stack".to_string(),
                "project:demo".to_string(),
                "runtime-core".to_string()
            ]
        );
        assert_eq!(last.base_url.as_deref(), Some(base_url.as_str()));

        fs::remove_dir_all(dir).expect("cleanup resume dir");
    }

    #[tokio::test]
    async fn read_bundle_resume_keeps_recalled_project_fact_visible_in_bundle_memory() {
        let dir =
            std::env::temp_dir().join(format!("memd-resume-project-fact-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create resume dir");
        let state = MockRuntimeState::default();
        *state
            .context_compact_response
            .lock()
            .expect("lock context response") = Some(memd_schema::CompactContextResponse {
            route: memd_schema::RetrievalRoute::LocalFirst,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            retrieval_order: vec![
                memd_schema::MemoryScope::Local,
                memd_schema::MemoryScope::Synced,
                memd_schema::MemoryScope::Project,
            ],
            records: vec![memd_schema::CompactMemoryRecord {
                id: uuid::Uuid::new_v4(),
                record: "remembered project fact: memd must preserve important user corrections"
                    .to_string(),
            }],
        });
        *state
            .working_response
            .lock()
            .expect("lock working response") = Some(memd_schema::WorkingMemoryResponse {
            route: memd_schema::RetrievalRoute::LocalFirst,
            intent: memd_schema::RetrievalIntent::CurrentTask,
            retrieval_order: vec![
                memd_schema::MemoryScope::Local,
                memd_schema::MemoryScope::Synced,
                memd_schema::MemoryScope::Project,
            ],
            budget_chars: 1600,
            used_chars: 220,
            remaining_chars: 1380,
            truncated: false,
            policy: memd_schema::WorkingMemoryPolicyState {
                admission_limit: 8,
                max_chars_per_item: 220,
                budget_chars: 1600,
                rehydration_limit: 4,
            },
            records: vec![
                memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record:
                        "remembered project fact: memd must preserve important user corrections"
                            .to_string(),
                },
                memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "resume state noise: synced session snapshot".to_string(),
                },
            ],
            evicted: Vec::new(),
            rehydration_queue: Vec::new(),
            traces: Vec::new(),
            semantic_consolidation: None,
        });
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

        let snapshot = read_bundle_resume(
            &ResumeArgs {
                output: dir.clone(),
                project: None,
                namespace: None,
                agent: None,
                workspace: None,
                visibility: None,
                route: None,
                intent: Some("current_task".to_string()),
                limit: Some(8),
                rehydration_limit: Some(4),
                semantic: false,
                prompt: false,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("read bundle resume");
        write_bundle_memory_files(&dir, &snapshot, None, false)
            .await
            .expect("write bundle memory files");
        refresh_live_bundle_event_pages(&dir, &snapshot, None).expect("refresh live event pages");

        assert!(
            snapshot.working.records[0]
                .record
                .contains("memd must preserve important user corrections")
        );

        let markdown =
            fs::read_to_string(dir.join("MEMD_MEMORY.md")).expect("read generated bundle memory");
        assert!(markdown.contains("## Scope"));
        assert!(markdown.contains("# memd memory [tab=tab-alpha]"));
        assert!(markdown.contains("session: `codex-a`"));
        assert!(markdown.contains("effective agent: `codex@codex-a`"));
        assert!(markdown.contains("memd must preserve important user corrections"));
        assert!(markdown.contains("resume state noise"));
        assert!(markdown.contains("MEMD_EVENTS.md"));
        let context_page = fs::read_to_string(dir.join("compiled/memory/context.md"))
            .expect("read compiled context page");
        assert!(context_page.contains("# memd memory object: Context [tab=tab-alpha]"));
        assert!(context_page.contains("session: `codex-a`"));
        assert!(context_page.contains("- id=") || context_page.contains("- none"));
        let working_page = fs::read_to_string(dir.join("compiled/memory/working.md"))
            .expect("read compiled working page");
        assert!(working_page.contains("# memd memory object: Working [tab=tab-alpha]"));
        assert!(working_page.contains("session: `codex-a`"));
        assert!(working_page.contains("memd must preserve important user corrections"));
        assert!(working_page.contains("items/working/"));
        let working_key = memory_object_lane_item_key(&snapshot, MemoryObjectLane::Working, 0)
            .expect("working key");
        let working_item_path =
            bundle_compiled_memory_item_path(&dir, MemoryObjectLane::Working, 0, &working_key);
        let working_item_page =
            fs::read_to_string(&working_item_path).expect("read compiled working item page");
        assert!(working_item_page.contains("# memd memory item: Working [tab=tab-alpha]"));
        assert!(working_item_page.contains("session: `codex-a`"));
        assert!(working_item_page.contains("memd must preserve important user corrections"));
        let inbox_page = fs::read_to_string(dir.join("compiled/memory/inbox.md"))
            .expect("read compiled inbox page");
        assert!(inbox_page.contains("# memd memory object: Inbox"));
        let recovery_page = fs::read_to_string(dir.join("compiled/memory/recovery.md"))
            .expect("read compiled recovery page");
        assert!(recovery_page.contains("# memd memory object: Recovery"));
        let semantic_page = fs::read_to_string(dir.join("compiled/memory/semantic.md"))
            .expect("read compiled semantic page");
        assert!(semantic_page.contains("# memd memory object: Semantic"));
        let workspace_page = fs::read_to_string(dir.join("compiled/memory/workspace.md"))
            .expect("read compiled workspace page");
        assert!(workspace_page.contains("# memd memory object: Workspace"));
        let event_log =
            fs::read_to_string(dir.join("MEMD_EVENTS.md")).expect("read generated event log");
        assert!(event_log.contains("# memd event log"));
        assert!(event_log.contains("event compiler"));
        assert!(event_log.contains("live_snapshot") || event_log.contains("resume_snapshot"));
        let event_index = fs::read_to_string(dir.join("compiled/events/latest.md"))
            .expect("read compiled event index");
        assert!(event_index.contains("# memd event index"));
        assert!(path_text_contains(&event_index, "compiled/events/items/"));
        let wakeup =
            fs::read_to_string(dir.join("MEMD_WAKEUP.md")).expect("read generated wakeup memory");
        assert!(wakeup.contains("# memd wake-up"));
        assert!(wakeup.contains("Read first."));
        assert!(wakeup.contains("memd must preserve important user corrections"));
        assert!(wakeup.contains("Default voice: caveman ultra"));
        let remember_decision = fs::read_to_string(dir.join("agents/remember-decision.sh"))
            .expect("read remember decision helper");
        let remember_short =
            fs::read_to_string(dir.join("agents/remember-short.sh")).expect("read short helper");
        let remember_long =
            fs::read_to_string(dir.join("agents/remember-long.sh")).expect("read long helper");
        let watch = fs::read_to_string(dir.join("agents/watch.sh")).expect("read watch helper");
        assert!(remember_decision.contains("args=(remember --output"));
        assert!(remember_decision.contains("--kind \"decision\""));
        assert!(remember_decision.contains("--tag \"basic-memory\""));
        assert!(remember_short.contains("args=(checkpoint --output"));
        assert!(remember_short.contains("--tag basic-memory --tag short-term"));
        assert!(remember_long.contains("--kind \"fact\""));
        assert!(remember_long.contains("--tag \"long-term\""));
        assert!(watch.contains("memd watch --root"));
        let capture_live =
            fs::read_to_string(dir.join("agents/capture-live.sh")).expect("read capture helper");
        assert!(capture_live.contains("args=(hook capture --output"));
        assert!(capture_live.contains("--tag basic-memory --tag live-capture"));
        let sync_semantic =
            fs::read_to_string(dir.join("agents/sync-semantic.sh")).expect("read semantic helper");
        assert!(sync_semantic.contains("args=(rag sync)"));
        assert!(sync_semantic.contains("MEMD_PROJECT"));
        let claude_imports =
            fs::read_to_string(dir.join("agents/CLAUDE_IMPORTS.md")).expect("read claude imports");
        assert!(claude_imports.contains(".memd/agents/remember-short.sh"));
        assert!(claude_imports.contains(".memd/agents/remember-decision.sh"));
        assert!(claude_imports.contains(".memd/agents/remember-long.sh"));
        assert!(claude_imports.contains(".memd/agents/correct-memory.sh"));
        assert!(claude_imports.contains(".memd/agents/sync-semantic.sh"));
        assert!(claude_imports.contains("@../MEMD_EVENTS.md"));
        assert!(claude_imports.contains("@CLAUDE_CODE_EVENTS.md"));

        fs::remove_dir_all(dir).expect("cleanup resume dir");
    }

    #[tokio::test]
    async fn checkpoint_refreshes_live_event_pages() {
        let dir = std::env::temp_dir().join(format!("memd-live-events-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
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
                source_path: Some("checkpoint".to_string()),
                confidence: Some(0.9),
                ttl_seconds: Some(60),
                tag: vec!["checkpoint".to_string()],
                content: Some("refresh live event pages".to_string()),
                input: None,
                stdin: false,
            },
            &base_url,
        )
        .await
        .expect("checkpoint");

        let snapshot = read_bundle_resume(
            &autoresearch_resume_args_with_limits(&dir, 8, 4, true),
            &base_url,
        )
        .await
        .expect("read bundle resume");
        write_bundle_memory_files(&dir, &snapshot, None, false)
            .await
            .expect("write bundle memory files");
        refresh_live_bundle_event_pages(&dir, &snapshot, None).expect("refresh live event pages");

        let events = read_bundle_event_log(&dir).expect("read bundle event log");
        assert_eq!(events.len(), 1);
        assert!(events[0].summary.contains("project=demo"));
        assert!(events[0].summary.contains("tokens="));
        let root_events =
            fs::read_to_string(dir.join("MEMD_EVENTS.md")).expect("read generated event log");
        assert!(root_events.contains("# memd event log"));
        assert!(root_events.contains("event compiler"));
        assert!(root_events.contains("compiled/events/"));
        let compiled = fs::read_to_string(dir.join("compiled/events/latest.md"))
            .expect("read compiled event index");
        assert!(compiled.contains("# memd event index"));
        fs::remove_dir_all(dir).expect("cleanup live events dir");
    }

    #[test]
    fn compiled_memory_search_resolves_lane_and_item_pages() {
        let root = std::env::temp_dir().join(format!("memd-memory-query-{}", uuid::Uuid::new_v4()));
        let compiled = root.join("compiled").join("memory");
        let items = compiled.join("items").join("working");
        fs::create_dir_all(&items).expect("create compiled memory dir");
        fs::write(
            compiled.join("working.md"),
            "# memd memory object: Working\n\n- [open](items/working/working-01-abcd1234.md)\n",
        )
        .expect("write lane page");
        fs::write(
            items.join("working-01-abcd1234.md"),
            "# memd memory item: Working\n\n- id=abc123 record=\"current_task: keep memory visible\"\n",
        )
        .expect("write item page");

        let lane_hits = search_compiled_memory_pages(&root, "working", 8)
            .expect("search compiled memory pages");
        assert!(
            lane_hits
                .iter()
                .any(|hit| path_text_ends_with(&hit.path, "working.md"))
        );
        assert!(
            lane_hits
                .iter()
                .any(|hit| path_text_ends_with(&hit.path, "working-01-abcd1234.md"))
        );

        let resolved =
            resolve_compiled_memory_page(&root, "working").expect("resolve compiled memory page");
        assert!(path_text_ends_with(&resolved, "working.md"));

        let item_resolved = resolve_compiled_memory_page(&root, "working-01-abcd1234")
            .expect("resolve compiled memory item");
        assert!(path_text_ends_with(
            &item_resolved,
            "working-01-abcd1234.md"
        ));

        fs::remove_dir_all(root).expect("cleanup memory query temp dir");
    }

    #[test]
    fn compiled_memory_lane_shortcut_resolves_lane_page() {
        let root = std::env::temp_dir().join(format!("memd-memory-lane-{}", uuid::Uuid::new_v4()));
        let compiled = root.join("compiled").join("memory");
        fs::create_dir_all(&compiled).expect("create compiled memory dir");
        fs::write(
            compiled.join("working.md"),
            "# memd memory object: Working\n\n- [open](items/working/working-01-abcd1234.md)\n",
        )
        .expect("write lane page");

        let resolved =
            resolve_compiled_memory_page(&root, "working").expect("resolve lane shortcut");
        assert!(path_text_ends_with(&resolved, "working.md"));

        fs::remove_dir_all(root).expect("cleanup memory lane temp dir");
    }

    #[test]
    fn compiled_event_search_resolves_kind_and_item_pages() {
        let root = std::env::temp_dir().join(format!("memd-event-query-{}", uuid::Uuid::new_v4()));
        let compiled = root.join("compiled").join("events");
        let items = compiled.join("items").join("live_snapshot");
        fs::create_dir_all(&items).expect("create compiled event dir");
        fs::write(
            compiled.join("live_snapshot.md"),
            "# memd event lane: Live Snapshot\n\n- [event-01-abcd1234](items/live_snapshot/event-01-abcd1234.md)\n",
        )
        .expect("write event lane page");
        fs::write(
            items.join("event-01-abcd1234.md"),
            "# memd event item: Live Snapshot\n\n- summary: live_snapshot project=demo pressure=\"trim context\"\n",
        )
        .expect("write event item page");

        let hits =
            search_compiled_event_pages(&root, "pressure", 8).expect("search compiled event pages");
        assert!(!hits.is_empty());
        assert!(
            hits.iter()
                .any(|hit| hit.path.ends_with("event-01-abcd1234.md"))
        );

        let index = render_compiled_event_index(&root).expect("render compiled event index");
        assert!(index.kind_count >= 1);
        assert!(index.item_count >= 1);
        assert!(
            index
                .pages
                .iter()
                .any(|page| page.ends_with("live_snapshot.md"))
        );
        assert!(
            index
                .pages
                .iter()
                .any(|page| page.contains("event-01-abcd1234.md"))
        );

        let resolved = resolve_compiled_event_page(&root, "live_snapshot")
            .expect("resolve compiled event page");
        assert!(resolved.ends_with("live_snapshot.md"));

        let item_resolved = resolve_compiled_event_page(&root, "event-01-abcd1234")
            .expect("resolve compiled event item");
        assert!(item_resolved.ends_with("event-01-abcd1234.md"));

        fs::remove_dir_all(root).expect("cleanup event query temp dir");
    }

    #[test]
    fn compiled_event_index_summary_and_json_include_lane_counts() {
        let root = std::env::temp_dir().join(format!("memd-event-index-{}", uuid::Uuid::new_v4()));
        let compiled = root.join("compiled").join("events");
        let items = compiled.join("items").join("live_snapshot");
        fs::create_dir_all(&items).expect("create compiled event dir");
        fs::write(
            compiled.join("live_snapshot.md"),
            "# memd event lane: Live Snapshot\n\n- [event-01-abcd1234](items/live_snapshot/event-01-abcd1234.md)\n",
        )
        .expect("write event lane page");
        fs::write(
            items.join("event-01-abcd1234.md"),
            "# memd event item: Live Snapshot\n\n- summary: live_snapshot project=demo pressure=\"trim context\"\n",
        )
        .expect("write event item page");

        let index = render_compiled_event_index(&root).expect("render compiled event index");
        let summary = render_compiled_event_index_summary(&root, &index);
        assert!(summary.contains("event index"));
        assert!(summary.contains("kinds=1"));
        assert!(summary.contains("items=1"));
        let json = render_compiled_event_index_json(&root, &index);
        assert_eq!(json.root, root.display().to_string());
        assert_eq!(json.kind_count, 1);
        assert_eq!(json.item_count, 1);
        assert!(
            json.pages
                .iter()
                .any(|page| page.ends_with("live_snapshot.md"))
        );
        assert!(
            json.pages
                .iter()
                .any(|page| page.contains("event-01-abcd1234.md"))
        );

        fs::remove_dir_all(root).expect("cleanup event index temp dir");
    }

    #[test]
    fn compiled_memory_item_target_takes_precedence_over_lane_and_open() {
        let args = MemoryArgs {
            root: None,
            query: None,
            open: Some("working".to_string()),
            lane: Some("working".to_string()),
            item: Some("working-01-abcd1234".to_string()),
            list: false,
            lanes_only: false,
            items_only: false,
            filter: None,
            grouped: false,
            expand_items: false,
            json: false,
            limit: 12,
            summary: true,
            quality: false,
        };

        assert_eq!(
            compiled_memory_target(&args).as_deref(),
            Some("working-01-abcd1234")
        );
    }

    #[test]
    fn compiled_memory_index_lists_lane_and_item_pages() {
        let root = std::env::temp_dir().join(format!("memd-memory-index-{}", uuid::Uuid::new_v4()));
        let compiled = root.join("compiled").join("memory");
        let items = compiled.join("items").join("working");
        fs::create_dir_all(&items).expect("create compiled memory dir");
        fs::write(
            compiled.join("working.md"),
            "# memd memory object: Working\n\n- [open](items/working/working-01-abcd1234.md)\n",
        )
        .expect("write lane page");
        fs::write(
            items.join("working-01-abcd1234.md"),
            "# memd memory item: Working\n\n- id=abc123 record=\"current_task: keep memory visible\"\n",
        )
        .expect("write item page");

        let index = render_compiled_memory_index(&root).expect("render compiled memory index");
        assert!(index.lane_count >= 1);
        assert!(index.item_count >= 1);
        assert!(
            index
                .pages
                .iter()
                .any(|page| path_text_ends_with(page, "working.md"))
        );
        assert!(
            index
                .pages
                .iter()
                .any(|page| path_text_contains(page, "working-01-abcd1234.md"))
        );

        fs::remove_dir_all(root).expect("cleanup memory index temp dir");
    }

    #[test]
    fn compiled_memory_index_grouped_markdown_uses_lane_sections_and_links() {
        let root = std::env::temp_dir().join(format!(
            "memd-memory-index-grouped-{}",
            uuid::Uuid::new_v4()
        ));
        let compiled = root.join("compiled").join("memory");
        let items = compiled.join("items").join("working");
        fs::create_dir_all(&items).expect("create compiled memory dir");
        fs::write(compiled.join("working.md"), "# Working\n").expect("write lane page");
        fs::write(items.join("working-01-abcd1234.md"), "# Item\n").expect("write item page");

        let index = render_compiled_memory_index(&root).expect("render compiled memory index");
        let markdown = render_compiled_memory_index_grouped_markdown(&root, &index, true);
        assert!(markdown.contains("## Working"));
        assert!(markdown.contains("[Working]("));
        assert!(path_text_contains(&markdown, "compiled/memory/working.md"));
        assert!(markdown.contains("[working-01-abcd1234]("));
        assert!(path_text_contains(&markdown, "working-01-abcd1234.md"));

        fs::remove_dir_all(root).expect("cleanup memory index grouped temp dir");
    }

    #[test]
    fn compiled_memory_index_grouped_markdown_collapses_items_by_default() {
        let root = std::env::temp_dir().join(format!(
            "memd-memory-index-grouped-compact-{}",
            uuid::Uuid::new_v4()
        ));
        let compiled = root.join("compiled").join("memory");
        let items = compiled.join("items").join("working");
        fs::create_dir_all(&items).expect("create compiled memory dir");
        fs::write(compiled.join("working.md"), "# Working\n").expect("write lane page");
        fs::write(items.join("working-01-abcd1234.md"), "# Item\n").expect("write item page");
        fs::write(items.join("working-02-fedcba98.md"), "# Item 2\n").expect("write item page");

        let index = render_compiled_memory_index(&root).expect("render compiled memory index");
        let markdown = render_compiled_memory_index_grouped_markdown(&root, &index, false);
        assert!(markdown.contains("## Working"));
        assert!(markdown.contains("[Working]("));
        assert!(markdown.contains("+2 more item(s)") || markdown.contains("+1 more item(s)"));
        assert!(!markdown.contains("working-02-fedcba98"));

        fs::remove_dir_all(root).expect("cleanup memory index grouped compact temp dir");
    }

    #[test]
    fn compiled_memory_index_json_exports_paths_and_counts() {
        let root =
            std::env::temp_dir().join(format!("memd-memory-index-json-{}", uuid::Uuid::new_v4()));
        let compiled = root.join("compiled").join("memory");
        let items = compiled.join("items").join("working");
        fs::create_dir_all(&items).expect("create compiled memory dir");
        fs::write(compiled.join("working.md"), "# Working\n").expect("write lane page");
        fs::write(items.join("working-01-abcd1234.md"), "# Item\n").expect("write item page");

        let index = render_compiled_memory_index(&root).expect("render compiled memory index");
        let json = render_compiled_memory_index_json(&root, &index);
        assert_eq!(json.root, root.display().to_string());
        assert_eq!(json.tab_id, "none");
        assert_eq!(json.lane_count, 1);
        assert_eq!(json.item_count, 1);
        assert!(
            json.pages
                .iter()
                .any(|page| path_text_ends_with(page, "working.md"))
        );
        assert!(
            json.pages
                .iter()
                .any(|page| path_text_contains(page, "working-01-abcd1234.md"))
        );
        assert!(json.entries.iter().any(|entry| {
            entry.kind == "lane" && entry.lane == "working" && entry.relative_path == "working.md"
        }));
        assert!(json.entries.iter().any(|entry| {
            entry.kind == "item"
                && entry.lane == "working"
                && normalize_path_text(&entry.relative_path)
                    == "items/working/working-01-abcd1234.md"
        }));

        fs::remove_dir_all(root).expect("cleanup memory index json temp dir");
    }

    #[test]
    fn compiled_memory_index_summary_stays_compact() {
        let root = std::env::temp_dir().join(format!(
            "memd-memory-index-summary-{}",
            uuid::Uuid::new_v4()
        ));
        let compiled = root.join("compiled").join("memory");
        let items = compiled.join("items").join("working");
        fs::create_dir_all(&items).expect("create compiled memory dir");
        fs::write(compiled.join("working.md"), "# Working\n").expect("write lane page");
        fs::write(items.join("working-01-abcd1234.md"), "# Item\n").expect("write item page");

        let index = render_compiled_memory_index(&root).expect("render compiled memory index");
        let summary = render_compiled_memory_index_summary(&root, &index);
        assert!(summary.contains("memory index root="));
        assert!(summary.contains("lanes=1"));
        assert!(summary.contains("items=1"));
        assert!(summary.contains("pages=2"));

        fs::remove_dir_all(root).expect("cleanup memory index summary temp dir");
    }

    #[test]
    fn compiled_memory_search_ranks_exact_path_matches_before_generic_hits() {
        let root =
            std::env::temp_dir().join(format!("memd-memory-search-rank-{}", uuid::Uuid::new_v4()));
        let compiled = root.join("compiled").join("memory");
        fs::create_dir_all(&compiled).expect("create compiled memory dir");
        fs::write(
            compiled.join("working.md"),
            "# Working\n\nworking memory is the current lane.\n",
        )
        .expect("write working page");
        fs::write(
            compiled.join("notes.md"),
            "# Notes\n\nthis working note is more generic.\n",
        )
        .expect("write notes page");

        let hits = search_compiled_memory_pages(&root, "working", 2).expect("search memory");
        assert_eq!(hits.len(), 2);
        assert!(hits[0].score >= hits[1].score);
        assert!(path_text_ends_with(&hits[0].path, "working.md"));
        assert!(!hits[0].reasons.is_empty());

        fs::remove_dir_all(root).expect("cleanup memory search temp dir");
    }

    #[test]
    fn compiled_memory_quality_report_scores_scope_and_probes() {
        let root =
            std::env::temp_dir().join(format!("memd-memory-quality-{}", uuid::Uuid::new_v4()));
        let compiled = root.join("compiled").join("memory");
        fs::create_dir_all(&compiled).expect("create compiled memory dir");
        fs::write(
            root.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "session-alpha",
  "tab_id": "tab-alpha"
}
"#,
        )
        .expect("write runtime config");
        fs::write(
            root.join("MEMD_MEMORY.md"),
            "# memd memory\n\n## Scope\n\n- source_note: [[Working]]\n",
        )
        .expect("write memory surface");
        fs::write(
            compiled.join("working.md"),
            "# Working\n\nworking memory is the current lane.\n",
        )
        .expect("write working page");

        let report = build_compiled_memory_quality_report(&root).expect("build quality report");
        assert_eq!(report.benchmark_target, "supermemory");
        assert!(report.score > 0);
        assert!(report.page_count >= 1);
        assert!(
            report
                .dimensions
                .iter()
                .any(|dimension| dimension.name == "freshness")
        );
        assert!(
            report
                .dimensions
                .iter()
                .any(|dimension| dimension.name == "contradiction")
        );
        assert!(
            report
                .dimensions
                .iter()
                .any(|dimension| dimension.name == "token_efficiency")
        );
        assert!(
            report
                .probes
                .iter()
                .any(|probe| probe.query == "working" && probe.best_score > 0)
        );
        assert!(
            report
                .recommendations
                .iter()
                .any(|rec| rec.contains("surface")
                    || rec.contains("scope")
                    || rec.contains("rank"))
        );

        fs::remove_dir_all(root).expect("cleanup memory quality temp dir");
    }

    #[test]
    fn compiled_memory_index_filters_lanes_items_and_text() {
        let root =
            std::env::temp_dir().join(format!("memd-memory-index-filter-{}", uuid::Uuid::new_v4()));
        let compiled = root.join("compiled").join("memory");
        let items = compiled.join("items").join("working");
        fs::create_dir_all(&items).expect("create compiled memory dir");
        fs::write(compiled.join("working.md"), "# Working\n").expect("write lane page");
        fs::write(items.join("working-01-abcd1234.md"), "# Item\n").expect("write item page");

        let index = render_compiled_memory_index(&root).expect("render compiled memory index");
        let lanes_only = filter_compiled_memory_index(index.clone(), true, false, None);
        assert!(
            lanes_only
                .pages
                .iter()
                .all(|page| !path_text_contains(page, "/items/"))
        );
        assert_eq!(lanes_only.lane_count, 1);
        assert_eq!(lanes_only.item_count, 0);

        let items_only = filter_compiled_memory_index(index.clone(), false, true, None);
        assert!(
            items_only
                .pages
                .iter()
                .all(|page| path_text_contains(page, "/items/"))
        );
        assert_eq!(items_only.lane_count, 0);
        assert_eq!(items_only.item_count, 1);

        let filtered = filter_compiled_memory_index(index, false, false, Some("working-01"));
        assert!(
            filtered
                .pages
                .iter()
                .any(|page| path_text_contains(page, "working-01-abcd1234.md"))
        );
        assert_eq!(filtered.pages.len(), 1);

        fs::remove_dir_all(root).expect("cleanup memory index filter temp dir");
    }

    #[test]
    fn harness_pack_index_lists_known_packs() {
        let root = std::env::temp_dir().join(format!("memd-pack-index-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).expect("create pack index root");

        let index =
            crate::harness::index::build_harness_pack_index(&root, Some("demo"), Some("main"));
        assert_eq!(index.pack_count, 6);
        assert!(index.packs.iter().any(|pack| pack.name == "Codex"));
        assert!(index.packs.iter().any(|pack| pack.name == "Claude Code"));
        assert!(index.packs.iter().any(|pack| pack.name == "Agent Zero"));
        assert!(index.packs.iter().any(|pack| pack.name == "Hermes"));
        assert!(index.packs.iter().any(|pack| pack.name == "OpenCode"));
        assert!(index.packs.iter().any(|pack| pack.name == "OpenClaw"));

        let summary = render_harness_pack_index_summary(&root, &index, None);
        assert!(summary.contains("pack index root="));
        assert!(summary.contains("packs=6"));
        assert!(summary.contains("Codex"));
        assert!(summary.contains("Claude Code"));
        assert!(summary.contains("Agent Zero"));
        assert!(summary.contains("Hermes"));
        assert!(summary.contains("OpenCode"));
        assert!(summary.contains("OpenClaw"));

        fs::remove_dir_all(root).expect("cleanup pack index temp dir");
    }

    #[test]
    fn hermes_pack_manifest_exposes_onboarding_wake_capture_and_files() {
        let bundle_root =
            std::env::temp_dir().join(format!("memd-hermes-pack-test-{}", uuid::Uuid::new_v4()));
        let manifest =
            crate::harness::hermes::build_hermes_harness_pack(&bundle_root, "demo", "main");

        assert_eq!(manifest.agent, "hermes");
        assert!(
            manifest
                .files
                .iter()
                .any(|path| path.ends_with("HERMES_WAKEUP.md"))
        );
        assert!(
            manifest
                .files
                .iter()
                .any(|path| path.ends_with("HERMES_MEMORY.md"))
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
                .any(|line| line.contains("onboarding-friendly wake"))
        );

        let markdown = render_hermes_harness_pack_markdown(&manifest);
        assert!(markdown.contains("HERMES_WAKEUP.md"));
        assert!(markdown.contains("HERMES_MEMORY.md"));
        assert!(markdown.contains("onboarding-friendly wake"));
        assert!(markdown.contains("memd hook capture --output .memd --stdin --summary"));
    }

    #[test]
    fn agent_zero_pack_manifest_exposes_resume_handoff_and_files() {
        let bundle_root = std::env::temp_dir().join(format!(
            "memd-agent-zero-pack-test-{}",
            uuid::Uuid::new_v4()
        ));
        let manifest =
            crate::harness::agent_zero::build_agent_zero_harness_pack(&bundle_root, "demo", "main");

        assert_eq!(manifest.agent, "agent-zero");
        assert!(
            manifest
                .files
                .iter()
                .any(|path| path.ends_with("AGENT_ZERO_WAKEUP.md"))
        );
        assert!(
            manifest
                .files
                .iter()
                .any(|path| path.ends_with("AGENT_ZERO_MEMORY.md"))
        );
        assert!(
            manifest
                .commands
                .iter()
                .any(|cmd| cmd.contains("memd resume --output .memd"))
        );
        assert!(
            manifest
                .commands
                .iter()
                .any(|cmd| cmd.contains("memd handoff --output .memd --prompt"))
        );
        assert!(
            manifest
                .behaviors
                .iter()
                .any(|line| line.contains("zero-friction resume"))
        );

        let markdown = render_agent_zero_harness_pack_markdown(&manifest);
        assert!(markdown.contains("AGENT_ZERO_WAKEUP.md"));
        assert!(markdown.contains("AGENT_ZERO_MEMORY.md"));
        assert!(markdown.contains("zero-friction resume"));
        assert!(markdown.contains("memd remember --output .memd --kind decision"));
    }

    #[tokio::test]
    async fn agent_zero_pack_refreshes_visible_bundle_files_from_turn_state() {
        let bundle_root =
            std::env::temp_dir().join(format!("memd-agent-zero-refresh-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(bundle_root.join("agents")).expect("create agent-zero bundle dir");
        fs::write(
            bundle_root.join("MEMD_MEMORY.md"),
            "# Memory\n\nstale bundle\n",
        )
        .expect("seed memory file");
        fs::write(
            bundle_root.join("agents").join("AGENT_ZERO_MEMORY.md"),
            "# Agent Zero Memory\n\nstale agent bundle\n",
        )
        .expect("seed agent-zero agent memory file");

        let snapshot = codex_test_snapshot("demo", "main", "agent-zero");
        let manifest =
            crate::harness::agent_zero::build_agent_zero_harness_pack(&bundle_root, "demo", "main");

        let written = refresh_harness_pack_files(
            &bundle_root,
            &snapshot,
            &manifest,
            "agent-zero",
            "refresh",
            &harness_pack_query_from_snapshot(&snapshot),
        )
        .await
        .expect("refresh agent zero pack files");

        assert!(written.contains(&bundle_root.join("MEMD_MEMORY.md")));
        assert!(written.contains(&bundle_root.join("agents").join("AGENT_ZERO_MEMORY.md")));
        let refreshed = fs::read_to_string(bundle_root.join("MEMD_MEMORY.md"))
            .expect("read refreshed memory file");
        assert!(refreshed.contains("# memd memory"));
        assert!(refreshed.contains("keep the live wake surface current"));
    }

    #[test]
    fn opencode_pack_manifest_exposes_resume_handoff_and_files() {
        let bundle_root =
            std::env::temp_dir().join(format!("memd-opencode-pack-test-{}", uuid::Uuid::new_v4()));
        let manifest =
            crate::harness::opencode::build_opencode_harness_pack(&bundle_root, "demo", "main");

        assert_eq!(manifest.agent, "opencode");
        assert!(
            manifest
                .files
                .iter()
                .any(|path| path.ends_with("OPENCODE_WAKEUP.md"))
        );
        assert!(
            manifest
                .files
                .iter()
                .any(|path| path.ends_with("OPENCODE_MEMORY.md"))
        );
        assert!(
            manifest
                .commands
                .iter()
                .any(|cmd| cmd.contains("memd resume --output .memd"))
        );
        assert!(
            manifest
                .commands
                .iter()
                .any(|cmd| cmd.contains("memd handoff --output .memd --prompt"))
        );
        assert!(
            manifest
                .behaviors
                .iter()
                .any(|line| line.contains("write durable outcomes back"))
        );

        let markdown = render_opencode_harness_pack_markdown(&manifest);
        assert!(markdown.contains("OPENCODE_WAKEUP.md"));
        assert!(markdown.contains("OPENCODE_MEMORY.md"));
        assert!(markdown.contains("emit a shared handoff"));
        assert!(markdown.contains("memd remember --output .memd --kind decision"));
    }

    #[tokio::test]
    async fn opencode_pack_refreshes_visible_bundle_files_from_turn_state() {
        let bundle_root =
            std::env::temp_dir().join(format!("memd-opencode-refresh-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(bundle_root.join("agents")).expect("create opencode bundle dir");
        fs::write(
            bundle_root.join("MEMD_MEMORY.md"),
            "# Memory\n\nstale bundle\n",
        )
        .expect("seed memory file");
        fs::write(
            bundle_root.join("agents").join("OPENCODE_MEMORY.md"),
            "# OpenCode Memory\n\nstale agent bundle\n",
        )
        .expect("seed opencode agent memory file");

        let snapshot = codex_test_snapshot("demo", "main", "opencode");
        let manifest =
            crate::harness::opencode::build_opencode_harness_pack(&bundle_root, "demo", "main");

        let written = refresh_harness_pack_files(
            &bundle_root,
            &snapshot,
            &manifest,
            "opencode",
            "refresh",
            &harness_pack_query_from_snapshot(&snapshot),
        )
        .await
        .expect("refresh opencode pack files");

        assert!(written.contains(&bundle_root.join("MEMD_MEMORY.md")));
        assert!(written.contains(&bundle_root.join("agents").join("OPENCODE_MEMORY.md")));
        let refreshed = fs::read_to_string(bundle_root.join("MEMD_MEMORY.md"))
            .expect("read refreshed memory file");
        assert!(refreshed.contains("# memd memory"));
        assert!(refreshed.contains("keep the live wake surface current"));
    }

    #[test]
    fn harness_pack_index_query_matches_roles_commands_and_behaviors() {
        let root =
            std::env::temp_dir().join(format!("memd-pack-index-query-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).expect("create pack index root");

        let index =
            crate::harness::index::build_harness_pack_index(&root, Some("demo"), Some("main"));

        let spill = crate::harness::index::filter_harness_pack_index(index.clone(), Some("spill"));
        assert!(spill.packs.iter().any(|pack| pack.name == "OpenClaw"));
        assert!(!spill.packs.iter().any(|pack| pack.name == "Codex"));

        let capture =
            crate::harness::index::filter_harness_pack_index(index.clone(), Some("capture"));
        assert_eq!(capture.packs.len(), 2);
        assert!(capture.packs.iter().any(|pack| pack.name == "Codex"));
        assert!(capture.packs.iter().any(|pack| pack.name == "Hermes"));

        let compact =
            crate::harness::index::filter_harness_pack_index(index.clone(), Some("turn-scoped"));
        assert_eq!(compact.packs.len(), 3);

        let compact =
            crate::harness::index::filter_harness_pack_index(index, Some("compact context"));
        assert_eq!(compact.packs.len(), 1);
        assert_eq!(compact.packs[0].name, "OpenClaw");

        fs::remove_dir_all(root).expect("cleanup pack index query temp dir");
    }

    #[test]
    fn harness_pack_index_json_contains_structured_entries() {
        let root =
            std::env::temp_dir().join(format!("memd-pack-index-json-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).expect("create pack index root");

        let index =
            crate::harness::index::build_harness_pack_index(&root, Some("demo"), Some("main"));
        let json = render_harness_pack_index_json(&index);
        assert_eq!(json.root, root.display().to_string());
        assert_eq!(json.pack_count, 6);
        assert_eq!(json.packs.len(), 6);
        assert!(json.packs.iter().any(|pack| pack.name == "Codex"));
        assert!(json.packs.iter().any(|pack| pack.name == "Claude Code"));
        assert!(json.packs.iter().any(|pack| pack.name == "Agent Zero"));
        assert!(json.packs.iter().any(|pack| pack.name == "Hermes"));
        assert!(json.packs.iter().any(|pack| pack.name == "OpenCode"));
        assert!(json.packs.iter().any(|pack| pack.name == "OpenClaw"));

        fs::remove_dir_all(root).expect("cleanup pack index json temp dir");
    }

    #[tokio::test]
    async fn read_bundle_handoff_resolves_target_bundle_and_preserves_workspace_state() {
        let root =
            std::env::temp_dir().join(format!("memd-handoff-runtime-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("current");
        let target_project = root.join("target");
        let current_bundle = current_project.join(".memd");
        let target_bundle = target_project.join(".memd");
        fs::create_dir_all(current_bundle.join("state")).expect("create current bundle state");
        fs::create_dir_all(target_bundle.join("state")).expect("create target bundle state");

        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state, false).await;

        fs::write(
            current_bundle.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "{}",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}}
"#,
                base_url
            ),
        )
        .expect("write current config");

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
                lane_id: Some(target_project.display().to_string()),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some(default_heartbeat_model()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
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
                focus: Some("resume delegated workspace".to_string()),
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

        let snapshot = read_bundle_handoff(
            &HandoffArgs {
                output: current_bundle.clone(),
                target_session: Some("claude-b".to_string()),
                project: None,
                namespace: None,
                agent: None,
                workspace: None,
                visibility: None,
                route: None,
                intent: Some("current_task".to_string()),
                limit: Some(8),
                rehydration_limit: Some(4),
                source_limit: Some(6),
                semantic: false,
                prompt: false,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("read handoff");

        assert_eq!(snapshot.target_session.as_deref(), Some("claude-b"));
        assert_path_tail(
            snapshot.target_bundle.as_deref().unwrap_or_default(),
            &target_bundle,
        );
        assert_eq!(
            snapshot.resume.agent.as_deref(),
            Some("claude-code@claude-b")
        );
        assert_eq!(snapshot.resume.workspace.as_deref(), Some("shared"));
        assert_eq!(snapshot.resume.visibility.as_deref(), Some("workspace"));

        fs::remove_dir_all(root).expect("cleanup handoff runtime dir");
    }

    #[tokio::test]
    async fn read_bundle_resume_uses_cache_before_backend() {
        let dir = std::env::temp_dir().join(format!("memd-resume-cache-{}", uuid::Uuid::new_v4()));
        let output = dir.join(".memd");
        fs::create_dir_all(output.join("state")).expect("create bundle state dir");
        fs::write(
            output.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "tab_id": "tab-alpha",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:59999",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}
"#,
        )
        .expect("write config");

        let args = ResumeArgs {
            output: output.clone(),
            project: None,
            namespace: None,
            agent: None,
            workspace: None,
            visibility: None,
            route: None,
            intent: Some("current_task".to_string()),
            limit: Some(8),
            rehydration_limit: Some(4),
            semantic: true,
            prompt: false,
            summary: false,
        };
        let runtime = read_bundle_runtime_config(&output)
            .expect("read runtime")
            .expect("runtime config");
        let base_url =
            resolve_bundle_command_base_url("http://127.0.0.1:59999", runtime.base_url.as_deref());
        let cache_key = build_resume_snapshot_cache_key(&args, Some(&runtime), &base_url);
        let snapshot = ResumeSnapshot {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex@codex-a".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            context: memd_schema::CompactContextResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                retrieval_order: Vec::new(),
                records: Vec::new(),
            },
            working: memd_schema::WorkingMemoryResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                retrieval_order: Vec::new(),
                budget_chars: 0,
                used_chars: 0,
                remaining_chars: 0,
                truncated: false,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 0,
                    max_chars_per_item: 0,
                    budget_chars: 0,
                    rehydration_limit: 0,
                },
                records: Vec::new(),
                evicted: Vec::new(),
                rehydration_queue: Vec::new(),
                traces: Vec::new(),
                semantic_consolidation: None,
            },
            inbox: memd_schema::MemoryInboxResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                items: Vec::new(),
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: Vec::new(),
            },
            sources: memd_schema::SourceMemoryResponse {
                sources: Vec::new(),
            },
            semantic: None,
            claims: SessionClaimsState::default(),
            recent_repo_changes: vec!["changed file".to_string()],
            change_summary: vec!["summary".to_string()],
            resume_state_age_minutes: Some(1),
            refresh_recommended: false,
        };
        cache::write_resume_snapshot_cache(&output, &cache_key, &snapshot)
            .expect("write resume cache");

        let resumed = read_bundle_resume(&args, "http://127.0.0.1:59999")
            .await
            .expect("resume from cache");

        assert_eq!(resumed.project.as_deref(), Some("demo"));
        assert_eq!(resumed.agent.as_deref(), Some("codex@codex-a"));
        assert_eq!(resumed.workspace.as_deref(), Some("shared"));
        assert!(resumed.semantic.is_none());
        assert_eq!(resumed.change_summary, vec!["summary".to_string()]);

        fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[tokio::test]
    async fn read_bundle_handoff_uses_cache_before_backend() {
        let dir = std::env::temp_dir().join(format!("memd-handoff-cache-{}", uuid::Uuid::new_v4()));
        let output = dir.join(".memd");
        fs::create_dir_all(output.join("state")).expect("create bundle state dir");
        fs::write(
            output.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "tab_id": "tab-alpha",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:59998",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}
"#,
        )
        .expect("write config");

        let args = HandoffArgs {
            output: output.clone(),
            target_session: None,
            project: None,
            namespace: None,
            agent: None,
            workspace: None,
            visibility: None,
            route: None,
            intent: Some("current_task".to_string()),
            limit: Some(8),
            rehydration_limit: Some(4),
            source_limit: Some(6),
            semantic: false,
            prompt: false,
            summary: false,
        };
        let runtime = read_bundle_runtime_config(&output)
            .expect("read runtime")
            .expect("runtime config");
        let resolved_base_url =
            resolve_bundle_command_base_url("http://127.0.0.1:59998", runtime.base_url.as_deref());
        let resume_args = ResumeArgs {
            output: output.clone(),
            project: None,
            namespace: None,
            agent: None,
            workspace: None,
            visibility: None,
            route: None,
            intent: Some("current_task".to_string()),
            limit: Some(8),
            rehydration_limit: Some(4),
            semantic: false,
            prompt: false,
            summary: false,
        };
        let resume_key =
            build_resume_snapshot_cache_key(&resume_args, Some(&runtime), &resolved_base_url);
        let handoff_key = cache::build_turn_key(
            Some(&output.display().to_string()),
            None,
            Some("none"),
            "handoff",
            &format!(
                "resume_key={resume_key}|source_limit=6|target_session=none|target_bundle={}",
                output.display()
            ),
        );
        let resume_snapshot = ResumeSnapshot {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex@codex-a".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            context: memd_schema::CompactContextResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                retrieval_order: Vec::new(),
                records: Vec::new(),
            },
            working: memd_schema::WorkingMemoryResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                retrieval_order: Vec::new(),
                budget_chars: 0,
                used_chars: 0,
                remaining_chars: 0,
                truncated: false,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 0,
                    max_chars_per_item: 0,
                    budget_chars: 0,
                    rehydration_limit: 0,
                },
                records: Vec::new(),
                evicted: Vec::new(),
                rehydration_queue: Vec::new(),
                traces: Vec::new(),
                semantic_consolidation: None,
            },
            inbox: memd_schema::MemoryInboxResponse {
                route: RetrievalRoute::Auto,
                intent: RetrievalIntent::CurrentTask,
                items: Vec::new(),
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: Vec::new(),
            },
            sources: memd_schema::SourceMemoryResponse {
                sources: Vec::new(),
            },
            semantic: None,
            claims: SessionClaimsState::default(),
            recent_repo_changes: vec!["changed file".to_string()],
            change_summary: vec!["summary".to_string()],
            resume_state_age_minutes: Some(1),
            refresh_recommended: false,
        };
        let handoff = HandoffSnapshot {
            generated_at: Utc::now(),
            resume: resume_snapshot,
            sources: memd_schema::SourceMemoryResponse {
                sources: Vec::new(),
            },
            target_session: None,
            target_bundle: Some(output.display().to_string()),
        };
        cache::write_handoff_snapshot_cache(&output, &handoff_key, &handoff)
            .expect("write handoff cache");

        let resumed = read_bundle_handoff(&args, "http://127.0.0.1:59998")
            .await
            .expect("handoff from cache");
        let expected_target_bundle = output.display().to_string();

        assert_eq!(
            resumed.target_bundle.as_deref(),
            Some(expected_target_bundle.as_str())
        );
        assert_eq!(resumed.resume.project.as_deref(), Some("demo"));
        assert_eq!(resumed.resume.agent.as_deref(), Some("codex@codex-a"));
        assert!(resumed.resume.semantic.is_none());

        fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[test]
    fn invalidate_bundle_runtime_caches_removes_resume_and_handoff_snapshots() {
        let dir =
            std::env::temp_dir().join(format!("memd-runtime-cache-prune-{}", uuid::Uuid::new_v4()));
        let output = dir.join(".memd");
        fs::create_dir_all(output.join("state")).expect("create state dir");
        fs::write(output.join("state/resume-snapshot-cache.json"), "{}\n")
            .expect("write resume cache");
        fs::write(output.join("state/handoff-snapshot-cache.json"), "{}\n")
            .expect("write handoff cache");

        invalidate_bundle_runtime_caches(&output).expect("invalidate bundle caches");

        assert!(!output.join("state/resume-snapshot-cache.json").exists());
        assert!(!output.join("state/handoff-snapshot-cache.json").exists());

        fs::remove_dir_all(dir).expect("cleanup runtime cache dir");
    }

    #[test]
    fn set_bundle_base_url_updates_config_and_env_files() {
        let dir =
            std::env::temp_dir().join(format!("memd-bundle-base-url-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write config");
        fs::write(dir.join("env"), "MEMD_BASE_URL=http://127.0.0.1:8787\n").expect("write env");
        fs::write(
            dir.join("env.ps1"),
            "$env:MEMD_BASE_URL = \"http://127.0.0.1:8787\"\n",
        )
        .expect("write env.ps1");

        set_bundle_base_url(&dir, "http://127.0.0.1:9797").expect("set bundle base url");

        let config = fs::read_to_string(dir.join("config.json")).expect("read config");
        let env = fs::read_to_string(dir.join("env")).expect("read env");
        let env_ps1 = fs::read_to_string(dir.join("env.ps1")).expect("read env.ps1");
        assert!(config.contains(r#""base_url": "http://127.0.0.1:9797""#));
        assert!(env.contains("MEMD_BASE_URL=http://127.0.0.1:9797"));
        assert!(env_ps1.contains("$env:MEMD_BASE_URL = \"http://127.0.0.1:9797\""));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[tokio::test]
    async fn hive_join_forces_shared_base_url_for_stale_bundle() {
        let dir = std::env::temp_dir().join(format!("memd-hive-join-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "current_task",
  "hive_groups": ["openclaw-stack"]
}
"#,
        )
        .expect("write config");

        let response = run_hive_join_command(&HiveJoinArgs {
            output: dir.clone(),
            base_url: default_hive_join_base_url(),
            all_active: false,
            all_local: false,
            publish_heartbeat: false,
            summary: false,
        })
        .await
        .expect("join hive");

        let response = match response {
            HiveJoinResponse::Single(response) => response,
            other => panic!("expected single response, got {other:?}"),
        };
        assert_eq!(response.base_url, SHARED_MEMD_BASE_URL);
        assert_eq!(response.session.as_deref(), Some("codex-a"));
        let config = fs::read_to_string(dir.join("config.json")).expect("read config");
        let env = fs::read_to_string(dir.join("env")).expect("read env");
        assert!(config.contains(r#""base_url": "http://100.104.154.24:8787""#));
        assert!(env.contains("MEMD_BASE_URL=http://100.104.154.24:8787"));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[tokio::test]
    async fn hive_join_all_active_rewrites_live_bundles_in_project() {
        let root =
            std::env::temp_dir().join(format!("memd-hive-join-all-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("alpha");
        let sibling_project = root.join("beta");
        let current_bundle = current_project.join(".memd");
        let sibling_bundle = sibling_project.join(".memd");
        fs::create_dir_all(&current_bundle).expect("create current bundle");
        fs::create_dir_all(&sibling_bundle).expect("create sibling bundle");

        for bundle_root in [&current_bundle, &sibling_bundle] {
            fs::create_dir_all(bundle_root.join("state")).expect("create state dir");
            fs::write(
                bundle_root.join("config.json"),
                format!(
                    r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "{}",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:8787",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task",
  "hive_groups": ["openclaw-stack"]
}}
"#,
                    bundle_root
                        .parent()
                        .and_then(|path| path.file_name())
                        .and_then(|value| value.to_str())
                        .unwrap_or("bundle")
                ),
            )
            .expect("write config");
            let heartbeat = build_hive_heartbeat(bundle_root, None).expect("build heartbeat");
            fs::write(
                bundle_heartbeat_state_path(bundle_root),
                serde_json::to_string_pretty(&heartbeat).expect("serialize heartbeat") + "\n",
            )
            .expect("write heartbeat");
        }

        let response = run_hive_join_command(&HiveJoinArgs {
            output: current_bundle.clone(),
            base_url: default_hive_join_base_url(),
            all_active: true,
            all_local: false,
            publish_heartbeat: false,
            summary: false,
        })
        .await
        .expect("join all active");

        match response {
            HiveJoinResponse::Batch(batch) => {
                assert_eq!(batch.base_url, SHARED_MEMD_BASE_URL);
                assert_eq!(batch.joined.len(), 2);
            }
            other => panic!("expected batch response, got {other:?}"),
        }

        for bundle_root in [&current_bundle, &sibling_bundle] {
            let config = fs::read_to_string(bundle_root.join("config.json")).expect("read config");
            let env = fs::read_to_string(bundle_root.join("env")).expect("read env");
            assert!(config.contains(r#""base_url": "http://100.104.154.24:8787""#));
            assert!(env.contains("MEMD_BASE_URL=http://100.104.154.24:8787"));
        }

        fs::remove_dir_all(root).expect("cleanup project root");
    }

    #[tokio::test]
    async fn hive_command_propagates_hive_metadata_to_active_sibling_bundles() {
        let root =
            std::env::temp_dir().join(format!("memd-hive-propagate-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("alpha");
        let sibling_project = root.join("beta");
        let current_bundle = current_project.join(".memd");
        let sibling_bundle = sibling_project.join(".memd");
        fs::create_dir_all(current_bundle.join("state")).expect("create current state dir");
        fs::create_dir_all(sibling_bundle.join("state")).expect("create sibling state dir");

        fs::write(
            current_bundle.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "tab_id": "tab-a",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:8787",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}
"#,
        )
        .expect("write current config");
        fs::write(
            sibling_bundle.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-b",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:8787",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task"
}
"#,
        )
        .expect("write sibling config");
        write_test_bundle_heartbeat(
            &current_bundle,
            &test_hive_heartbeat_state("codex-a", "codex", "tab-a", "live", Utc::now()),
        );
        write_test_bundle_heartbeat(
            &sibling_bundle,
            &test_hive_heartbeat_state("codex-b", "codex", "tab-b", "live", Utc::now()),
        );

        let response = run_hive_command(&HiveArgs {
            command: None,
            global: false,
            project_root: None,
            seed_existing: false,
            project: None,
            namespace: None,
            agent: None,
            session: None,
            tab_id: None,
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            capability: Vec::new(),
            hive_group: vec!["project:demo".to_string()],
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            output: current_bundle.clone(),
            base_url: SHARED_MEMD_BASE_URL.to_string(),
            rag_url: None,
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            publish_heartbeat: true,
            force: false,
            summary: false,
        })
        .await
        .expect("run hive command");

        assert_eq!(response.hive_system.as_deref(), Some("codex"));
        assert_eq!(response.hive_role.as_deref(), Some("agent"));

        let sibling_runtime = read_bundle_runtime_config(&sibling_bundle)
            .expect("read sibling runtime")
            .expect("sibling runtime config");
        assert_eq!(sibling_runtime.hive_system.as_deref(), Some("codex"));
        assert_eq!(sibling_runtime.hive_role.as_deref(), Some("agent"));
        assert_eq!(sibling_runtime.authority.as_deref(), Some("participant"));
        assert_eq!(
            sibling_runtime.base_url.as_deref(),
            Some(SHARED_MEMD_BASE_URL)
        );
        assert!(
            sibling_runtime
                .hive_groups
                .iter()
                .any(|group| group == "project:demo")
        );

        let sibling_heartbeat = read_bundle_heartbeat(&sibling_bundle)
            .expect("read sibling heartbeat")
            .expect("sibling heartbeat");
        assert_eq!(sibling_heartbeat.hive_system.as_deref(), Some("codex"));
        assert_eq!(sibling_heartbeat.hive_role.as_deref(), Some("agent"));
        assert_eq!(sibling_heartbeat.authority.as_deref(), Some("participant"));
        assert!(
            sibling_heartbeat
                .hive_groups
                .iter()
                .any(|group| group == "project:demo")
        );

        fs::remove_dir_all(root).expect("cleanup project root");
    }

    #[tokio::test]
    async fn hive_join_all_local_rewrites_all_local_bundles_in_project() {
        let root =
            std::env::temp_dir().join(format!("memd-hive-join-local-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("alpha");
        let sibling_project = root.join("beta");
        let current_bundle = current_project.join(".memd");
        let sibling_bundle = sibling_project.join(".memd");
        fs::create_dir_all(&current_bundle).expect("create current bundle");
        fs::create_dir_all(&sibling_bundle).expect("create sibling bundle");

        for (bundle_root, session) in [(&current_bundle, "codex-a"), (&sibling_bundle, "codex-b")] {
            fs::write(
                bundle_root.join("config.json"),
                format!(
                    r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "{}",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:8787",
  "auto_short_term_capture": false,
  "route": "auto",
  "intent": "current_task",
  "hive_groups": ["openclaw-stack"]
}}
"#,
                    session
                ),
            )
            .expect("write config");
        }

        let response = run_hive_join_command(&HiveJoinArgs {
            output: current_bundle.clone(),
            base_url: default_hive_join_base_url(),
            all_active: false,
            all_local: true,
            publish_heartbeat: false,
            summary: false,
        })
        .await
        .expect("join all local");

        let batch = match response {
            HiveJoinResponse::Batch(batch) => batch,
            other => panic!("expected batch response, got {other:?}"),
        };
        assert_eq!(batch.base_url, SHARED_MEMD_BASE_URL);
        assert_eq!(batch.mode, "all-local");
        assert_eq!(batch.joined.len(), 2);

        for bundle_root in [&current_bundle, &sibling_bundle] {
            let config = fs::read_to_string(bundle_root.join("config.json")).expect("read config");
            let env = fs::read_to_string(bundle_root.join("env")).expect("read env");
            assert!(config.contains(r#""base_url": "http://100.104.154.24:8787""#));
            assert!(env.contains("MEMD_BASE_URL=http://100.104.154.24:8787"));
        }

        fs::remove_dir_all(root).expect("cleanup project root");
    }

    #[test]
    fn set_bundle_route_and_intent_update_config_and_env_files() {
        let dir = std::env::temp_dir().join(format!("memd-bundle-intent-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write config");
        fs::write(dir.join("env"), "MEMD_ROUTE=auto\nMEMD_INTENT=general\n").expect("write env");
        fs::write(
            dir.join("env.ps1"),
            "$env:MEMD_ROUTE = \"auto\"\n$env:MEMD_INTENT = \"general\"\n",
        )
        .expect("write env.ps1");

        set_bundle_route(&dir, "lexical").expect("set bundle route");
        set_bundle_intent(&dir, "current_task").expect("set bundle intent");

        let config = fs::read_to_string(dir.join("config.json")).expect("read config");
        let env = fs::read_to_string(dir.join("env")).expect("read env");
        let env_ps1 = fs::read_to_string(dir.join("env.ps1")).expect("read env.ps1");
        assert!(config.contains(r#""route": "lexical""#));
        assert!(config.contains(r#""intent": "current_task""#));
        assert!(env.contains("MEMD_ROUTE=lexical"));
        assert!(env.contains("MEMD_INTENT=current_task"));
        assert!(env_ps1.contains("$env:MEMD_ROUTE = \"lexical\""));
        assert!(env_ps1.contains("$env:MEMD_INTENT = \"current_task\""));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn set_bundle_hive_metadata_updates_config_and_env_files() {
        let dir =
            std::env::temp_dir().join(format!("memd-bundle-hive-meta-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write config");
        fs::write(dir.join("env"), "").expect("write env");
        fs::write(dir.join("env.ps1"), "").expect("write env.ps1");

        set_bundle_hive_system(&dir, "agent-shell").expect("set hive system");
        set_bundle_hive_role(&dir, "runtime-shell").expect("set hive role");
        set_bundle_capabilities(&dir, &["shell".to_string(), "exec".to_string()])
            .expect("set capabilities");
        set_bundle_hive_groups(
            &dir,
            &["runtime-core".to_string(), "dependency-owners".to_string()],
        )
        .expect("set hive groups");
        set_bundle_hive_group_goal(&dir, "stabilize runtime dependencies").expect("set group goal");
        set_bundle_authority(&dir, "worker").expect("set authority");

        let config = fs::read_to_string(dir.join("config.json")).expect("read config");
        let env = fs::read_to_string(dir.join("env")).expect("read env");
        let env_ps1 = fs::read_to_string(dir.join("env.ps1")).expect("read env.ps1");
        assert!(config.contains(r#""hive_system": "agent-shell""#));
        assert!(config.contains(r#""hive_role": "runtime-shell""#));
        assert!(config.contains(r#""hive_group_goal": "stabilize runtime dependencies""#));
        assert!(config.contains(r#""authority": "worker""#));
        assert!(config.contains(r#""hive_groups": ["#) || config.contains("\"hive_groups\": ["));
        assert!(env.contains("MEMD_PEER_SYSTEM=agent-shell"));
        assert!(env.contains("MEMD_PEER_ROLE=runtime-shell"));
        assert!(
            env.contains("MEMD_PEER_GROUPS=dependency-owners,runtime-core")
                || env.contains("MEMD_PEER_GROUPS=runtime-core,dependency-owners")
        );
        assert!(env.contains("MEMD_PEER_GROUP_GOAL=stabilize runtime dependencies"));
        assert!(env.contains("MEMD_PEER_AUTHORITY=worker"));
        assert!(env_ps1.contains("$env:MEMD_PEER_SYSTEM = \"agent-shell\""));
        assert!(env_ps1.contains("$env:MEMD_PEER_ROLE = \"runtime-shell\""));
        assert!(env_ps1.contains("$env:MEMD_PEER_GROUP_GOAL = \"stabilize runtime dependencies\""));
        assert!(env_ps1.contains("$env:MEMD_PEER_AUTHORITY = \"worker\""));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn hive_project_command_is_exposed_in_cli_help() {
        use clap::CommandFactory;

        let mut command = Cli::command();
        let help = command
            .find_subcommand_mut("hive-project")
            .expect("hive-project command")
            .render_long_help()
            .to_string();
        assert!(help.contains("hive-project"));
        assert!(help.contains("--enable"));
        assert!(help.contains("--disable"));
        assert!(help.contains("--status"));
    }

    #[tokio::test]
    async fn hive_project_enable_and_disable_update_bundle_state() {
        let dir = std::env::temp_dir().join(format!("memd-hive-project-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "current_task"
}
"#,
        )
        .expect("write config");
        fs::write(dir.join("env"), "").expect("write env");
        fs::write(dir.join("env.ps1"), "").expect("write env.ps1");

        let enabled = run_hive_project_command(&HiveProjectArgs {
            output: dir.clone(),
            enable: true,
            disable: false,
            status: false,
            summary: false,
        })
        .await
        .expect("enable hive project");
        assert!(enabled.enabled);
        assert_eq!(enabled.anchor.as_deref(), Some("project:demo"));

        let config = fs::read_to_string(dir.join("config.json")).expect("read config");
        assert!(config.contains("\"hive_project_enabled\": true"));
        assert!(config.contains("\"hive_project_anchor\": \"project:demo\""));

        let disabled = run_hive_project_command(&HiveProjectArgs {
            output: dir.clone(),
            enable: false,
            disable: true,
            status: false,
            summary: false,
        })
        .await
        .expect("disable hive project");
        assert!(!disabled.enabled);
        assert!(disabled.anchor.is_none());

        let config = fs::read_to_string(dir.join("config.json")).expect("read config");
        assert!(config.contains("\"hive_project_enabled\": false"));
        assert!(!config.contains("\"hive_project_anchor\": \"project:demo\""));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[tokio::test]
    async fn hive_project_enable_then_hive_join_then_hive_fix_all_work_together() {
        let dir =
            std::env::temp_dir().join(format!("memd-hive-project-e2e-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "current_task"
}
"#,
        )
        .expect("write config");
        fs::write(dir.join("env"), "").expect("write env");
        fs::write(dir.join("env.ps1"), "").expect("write env.ps1");

        let enabled = run_hive_project_command(&HiveProjectArgs {
            output: dir.clone(),
            enable: true,
            disable: false,
            status: false,
            summary: false,
        })
        .await
        .expect("enable hive project");
        assert!(enabled.enabled);
        assert_eq!(enabled.anchor.as_deref(), Some("project:demo"));
        assert_eq!(enabled.live_session.as_deref(), Some("codex-a"));

        let shell = render_agent_shell_profile(&dir, Some("codex"));
        let attach = render_attach_snippet("bash", &dir).expect("attach snippet");
        assert!(shell.contains(SHARED_MEMD_BASE_URL));
        assert!(attach.contains(SHARED_MEMD_BASE_URL));

        let joined = run_hive_join_command(&HiveJoinArgs {
            output: dir.clone(),
            base_url: "http://127.0.0.1:8787".to_string(),
            all_active: false,
            all_local: false,
            publish_heartbeat: false,
            summary: false,
        })
        .await
        .expect("join hive");
        let single = match joined {
            HiveJoinResponse::Single(response) => response,
            other => panic!("expected single response, got {other:?}"),
        };
        assert_eq!(single.base_url, SHARED_MEMD_BASE_URL);

        let runtime = read_bundle_runtime_config(&dir)
            .expect("reload bundle runtime config")
            .expect("bundle runtime config");
        assert!(runtime.hive_project_enabled);
        assert_eq!(runtime.hive_project_anchor.as_deref(), Some("project:demo"));
        assert_eq!(runtime.base_url.as_deref(), Some(SHARED_MEMD_BASE_URL));

        let config = fs::read_to_string(dir.join("config.json")).expect("read config");
        assert!(config.contains(r#""hive_project_enabled": true"#));
        assert!(config.contains(r#""base_url": "http://100.104.154.24:8787""#));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn set_bundle_scope_metadata_updates_config_and_env_files() {
        let dir =
            std::env::temp_dir().join(format!("memd-bundle-scope-meta-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write config");
        fs::write(dir.join("env"), "").expect("write env");
        fs::write(dir.join("env.ps1"), "").expect("write env.ps1");

        set_bundle_project(&dir, "clawcontrol-rollout").expect("set project");
        set_bundle_namespace(&dir, "main").expect("set namespace");
        set_bundle_workspace(&dir, "openclaw-stack").expect("set workspace");
        set_bundle_visibility(&dir, "workspace").expect("set visibility");

        let config = fs::read_to_string(dir.join("config.json")).expect("read config");
        let env = fs::read_to_string(dir.join("env")).expect("read env");
        let env_ps1 = fs::read_to_string(dir.join("env.ps1")).expect("read env.ps1");
        assert!(config.contains(r#""project": "clawcontrol-rollout""#));
        assert!(config.contains(r#""namespace": "main""#));
        assert!(config.contains(r#""workspace": "openclaw-stack""#));
        assert!(config.contains(r#""visibility": "workspace""#));
        assert!(env.contains("MEMD_PROJECT=clawcontrol-rollout"));
        assert!(env.contains("MEMD_NAMESPACE=main"));
        assert!(env.contains("MEMD_WORKSPACE=openclaw-stack"));
        assert!(env.contains("MEMD_VISIBILITY=workspace"));
        assert!(env_ps1.contains("$env:MEMD_PROJECT = \"clawcontrol-rollout\""));
        assert!(env_ps1.contains("$env:MEMD_NAMESPACE = \"main\""));
        assert!(env_ps1.contains("$env:MEMD_WORKSPACE = \"openclaw-stack\""));
        assert!(env_ps1.contains("$env:MEMD_VISIBILITY = \"workspace\""));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn write_init_bundle_persists_authority_policy_state_and_env_files() {
        let project_root =
            std::env::temp_dir().join(format!("memd-init-root-{}", uuid::Uuid::new_v4()));
        let output =
            std::env::temp_dir().join(format!("memd-init-output-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&project_root).expect("create project root");

        write_init_bundle(&InitArgs {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            global: false,
            project_root: Some(project_root.clone()),
            seed_existing: false,
            agent: "codex".to_string(),
            session: Some("codex-a".to_string()),
            tab_id: None,
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            capability: vec!["memory".to_string()],
            hive_group: vec!["project:demo".to_string()],
            hive_group_goal: Some("keep the bundle safe".to_string()),
            authority: Some("participant".to_string()),
            output: output.clone(),
            base_url: SHARED_MEMD_BASE_URL.to_string(),
            rag_url: None,
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            voice_mode: Some(default_voice_mode()),
            allow_localhost_read_only_fallback: false,
            force: true,
        })
        .expect("write init bundle");

        let config = fs::read_to_string(output.join("config.json")).expect("read config");
        let env = fs::read_to_string(output.join("env")).expect("read env");
        let env_ps1 = fs::read_to_string(output.join("env.ps1")).expect("read env.ps1");
        assert!(config.contains(r#""authority_policy""#));
        assert!(config.contains(r#""localhost_fallback_policy": "deny""#));
        assert!(config.contains(r#""authority_state""#));
        assert!(config.contains(r#""mode": "shared""#));
        assert!(config.contains(&format!(r#""shared_base_url": "{}""#, SHARED_MEMD_BASE_URL)));
        assert!(env.contains("MEMD_AUTHORITY_MODE=shared"));
        assert!(env.contains("MEMD_LOCALHOST_FALLBACK_POLICY=deny"));
        assert!(env.contains(&format!("MEMD_SHARED_BASE_URL={SHARED_MEMD_BASE_URL}")));
        assert!(env.contains("MEMD_AUTHORITY_DEGRADED=false"));
        assert!(env_ps1.contains("$env:MEMD_AUTHORITY_MODE = \"shared\""));
        assert!(env_ps1.contains("$env:MEMD_LOCALHOST_FALLBACK_POLICY = \"deny\""));
        assert!(env_ps1.contains(&format!(
            "$env:MEMD_SHARED_BASE_URL = \"{}\"",
            SHARED_MEMD_BASE_URL
        )));
        assert!(env_ps1.contains("$env:MEMD_AUTHORITY_DEGRADED = \"false\""));

        fs::remove_dir_all(project_root).expect("cleanup init project root");
        fs::remove_dir_all(output).expect("cleanup init output");
    }

    #[test]
    fn render_agent_profiles_surface_authority_warning_when_localhost_fallback_is_active() {
        let dir =
            std::env::temp_dir().join(format!("memd-authority-profile-{}", uuid::Uuid::new_v4()));
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
        fs::write(dir.join("env"), "").expect("write env");
        fs::write(dir.join("env.ps1"), "").expect("write env.ps1");

        let shell = render_agent_shell_profile(&dir, Some("codex"));
        let ps1 = render_agent_ps1_profile(&dir, Some("codex"));
        assert!(shell.contains("memd authority warning:"));
        assert!(shell.contains("localhost fallback is lower trust"));
        assert!(ps1.contains("memd authority warning:"));
        assert!(ps1.contains("localhost fallback is lower trust"));

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn hive_project_state_round_trips_through_bundle_runtime_config() {
        let runtime = BundleRuntimeConfig {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            session: Some("codex-a".to_string()),
            tab_id: Some("tab-alpha".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            capabilities: vec!["claim".to_string()],
            hive_groups: vec!["openclaw-stack".to_string()],
            hive_group_goal: Some("coordinate the project hive".to_string()),
            authority: Some("participant".to_string()),
            base_url: Some("http://100.104.154.24:8787".to_string()),
            route: Some("auto".to_string()),
            intent: Some("current_task".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            heartbeat_model: Some("gpt-4.1-mini".to_string()),
            auto_short_term_capture: true,
            hive_project_enabled: true,
            hive_project_anchor: Some("project:demo".to_string()),
            hive_project_joined_at: Some(Utc::now()),
            authority_policy: BundleAuthorityPolicy::default(),
            authority_state: BundleAuthorityState::default(),
        };
        let json = serde_json::to_string(&runtime).unwrap();
        assert!(json.contains("\"hive_project_enabled\":true"));
        assert!(json.contains("\"hive_project_anchor\":\"project:demo\""));
    }

    #[test]
    fn merge_bundle_runtime_config_prefers_overlay_scope() {
        let runtime = BundleRuntimeConfig {
            project: Some("global".to_string()),
            namespace: Some("global".to_string()),
            agent: Some("codex".to_string()),
            session: Some("codex-a".to_string()),
            tab_id: None,
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string(), "coordination".to_string()],
            hive_groups: vec!["openclaw-stack".to_string()],
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            hive_project_enabled: false,
            hive_project_anchor: None,
            hive_project_joined_at: None,
            base_url: Some("http://127.0.0.1:8787".to_string()),
            route: Some("auto".to_string()),
            intent: Some("general".to_string()),
            workspace: Some("global".to_string()),
            visibility: Some("private".to_string()),
            heartbeat_model: Some("llama-desktop/qwen".to_string()),
            auto_short_term_capture: true,
            authority_policy: BundleAuthorityPolicy::default(),
            authority_state: BundleAuthorityState::default(),
        };
        let overlay = BundleRuntimeConfig {
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            agent: None,
            session: None,
            tab_id: None,
            hive_system: Some("claw-control".to_string()),
            hive_role: Some("orchestrator".to_string()),
            capabilities: vec!["control".to_string(), "coordination".to_string()],
            hive_groups: vec!["openclaw-stack".to_string(), "control-plane".to_string()],
            hive_group_goal: None,
            authority: Some("coordinator".to_string()),
            hive_project_enabled: false,
            hive_project_anchor: None,
            hive_project_joined_at: None,
            base_url: None,
            route: Some("lexical".to_string()),
            intent: Some("current_task".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            heartbeat_model: None,
            auto_short_term_capture: false,
            authority_policy: BundleAuthorityPolicy::default(),
            authority_state: BundleAuthorityState::default(),
        };

        let merged = merge_bundle_runtime_config(runtime, overlay);
        assert_eq!(merged.project.as_deref(), Some("memd"));
        assert_eq!(merged.namespace.as_deref(), Some("main"));
        assert_eq!(merged.session.as_deref(), Some("codex-a"));
        assert_eq!(merged.hive_system.as_deref(), Some("claw-control"));
        assert_eq!(merged.hive_role.as_deref(), Some("orchestrator"));
        assert_eq!(merged.route.as_deref(), Some("lexical"));
        assert_eq!(merged.intent.as_deref(), Some("current_task"));
        assert_eq!(merged.workspace.as_deref(), Some("team-alpha"));
        assert_eq!(merged.visibility.as_deref(), Some("workspace"));
        assert_eq!(merged.base_url.as_deref(), Some("http://127.0.0.1:8787"));
        assert!(merged.auto_short_term_capture);
    }

    #[test]
    fn init_infers_service_hive_profile_for_claw_control() {
        let args = InitArgs {
            project: None,
            namespace: None,
            global: false,
            project_root: None,
            seed_existing: false,
            agent: "claw-control".to_string(),
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

        let profile = resolve_hive_profile(&args, None);
        assert_eq!(profile.hive_system.as_deref(), Some("claw-control"));
        assert_eq!(profile.hive_role.as_deref(), Some("orchestrator"));
        assert_eq!(profile.authority.as_deref(), Some("coordinator"));
        assert!(profile.capabilities.iter().any(|value| value == "control"));
        assert!(
            profile
                .capabilities
                .iter()
                .any(|value| value == "coordination")
        );
    }

    #[test]
    fn infer_service_agent_from_project_root_name() {
        assert_eq!(
            infer_service_agent_from_path(Path::new("/tmp/clawcontrol-rollout")).as_deref(),
            Some("claw-control")
        );
        assert_eq!(
            infer_service_agent_from_path(Path::new("/tmp/clawcontrol-agentshell")).as_deref(),
            Some("agent-shell")
        );
        assert_eq!(
            infer_service_agent_from_path(Path::new("/tmp/clawcontrol-agent-secrets-v2"))
                .as_deref(),
            Some("agent-secrets")
        );
        assert_eq!(
            infer_service_agent_from_path(Path::new("/tmp/workspace")).as_deref(),
            Some("openclaw")
        );
    }

    #[test]
    fn infer_worker_agent_from_env_prefers_explicit_worker_name() {
        unsafe {
            std::env::set_var("MEMD_WORKER_NAME", "Avicenna");
        }
        assert_eq!(infer_worker_agent_from_env().as_deref(), Some("Avicenna"));
        unsafe {
            std::env::remove_var("MEMD_WORKER_NAME");
        }
    }

    #[test]
    fn default_bundle_worker_name_prefers_session_backed_label_for_generic_agents() {
        assert_eq!(
            default_bundle_worker_name("codex", Some("session-6d422e56")),
            "Codex 6d422e56"
        );
        assert_eq!(
            default_bundle_worker_name("claude-code", Some("session-review-a")),
            "Claude review-a"
        );
        assert_eq!(
            default_bundle_worker_name("openclaw", Some("lane-a")),
            "Openclaw"
        );
    }

    #[test]
    fn default_bundle_worker_name_for_project_prefers_project_scoped_label_for_generic_agents() {
        assert_eq!(
            default_bundle_worker_name_for_project(
                Some("memd"),
                "codex",
                Some("session-6d422e56")
            ),
            "Memd Codex 6d422e56"
        );
        assert_eq!(
            default_bundle_worker_name_for_project(
                Some("demo"),
                "claude-code",
                Some("session-review-a")
            ),
            "Demo Claude review-a"
        );
        assert_eq!(
            default_bundle_worker_name_for_project(
                Some("memd"),
                "openclaw",
                Some("lane-a")
            ),
            "Openclaw"
        );
    }

    #[test]
    fn resolve_hive_command_base_url_prefers_global_bundle_override() {
        let _home_lock = lock_home_mutation();
        let home = std::env::temp_dir().join(format!("memd-home-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(home.join(".memd")).expect("create fake home bundle");
        fs::write(
            home.join(".memd").join("config.json"),
            r#"{
  "project": "global",
  "namespace": "global",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://100.104.154.24:8787"
}
"#,
        )
        .expect("write fake global config");
        let original_home = std::env::var_os("HOME");
        unsafe {
            std::env::set_var("HOME", &home);
        }

        let resolved = resolve_hive_command_base_url(SHARED_MEMD_BASE_URL);
        assert_eq!(resolved, "http://100.104.154.24:8787");

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
    fn default_base_url_prefers_global_bundle_override() {
        let _home_lock = lock_home_mutation();
        let home = std::env::temp_dir().join(format!("memd-default-home-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(home.join(".memd")).expect("create fake home bundle");
        fs::write(
            home.join(".memd").join("config.json"),
            r#"{
  "project": "global",
  "namespace": "global",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://100.104.154.24:8787"
}
"#,
        )
        .expect("write fake global config");
        let original_home = std::env::var_os("HOME");
        unsafe {
            std::env::set_var("HOME", &home);
        }

        let resolved = default_base_url();
        assert_eq!(resolved, "http://100.104.154.24:8787");

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
    fn resolve_bundle_command_base_url_honors_env_override() {
        let original_base_url = std::env::var_os("MEMD_BASE_URL");
        unsafe {
            std::env::set_var("MEMD_BASE_URL", "http://127.0.0.1:8787");
        }

        let resolved = resolve_bundle_command_base_url(
            "http://127.0.0.1:8787",
            Some("http://100.104.154.24:8787"),
        );
        assert_eq!(resolved, "http://127.0.0.1:8787");

        if let Some(value) = original_base_url {
            unsafe {
                std::env::set_var("MEMD_BASE_URL", value);
            }
        } else {
            unsafe {
                std::env::remove_var("MEMD_BASE_URL");
            }
        }
    }

    #[test]
    fn resolve_project_bundle_overlay_uses_local_bundle_from_global_root() {
        let temp_root =
            std::env::temp_dir().join(format!("memd-overlay-root-{}", uuid::Uuid::new_v4()));
        let global_root = temp_root.join("global");
        let repo_root = temp_root.join("repo");
        let local_bundle = repo_root.join(".memd");
        fs::create_dir_all(&local_bundle).expect("create local bundle");
        fs::write(
            local_bundle.join("config.json"),
            r#"{
  "project": "memd",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-a",
  "base_url": "http://127.0.0.1:8787",
  "route": "lexical",
  "intent": "current_task",
  "workspace": "team-alpha",
  "visibility": "workspace"
}
"#,
        )
        .expect("write local config");

        let overlay = resolve_project_bundle_overlay(&global_root, &repo_root, &global_root)
            .expect("resolve overlay")
            .expect("overlay present");
        assert_eq!(overlay.project.as_deref(), Some("memd"));
        assert_eq!(overlay.namespace.as_deref(), Some("main"));
        assert_eq!(overlay.route.as_deref(), Some("lexical"));
        assert_eq!(overlay.intent.as_deref(), Some("current_task"));

        fs::remove_dir_all(temp_root).expect("cleanup overlay temp");
    }

    #[test]
    fn resolve_live_session_overlay_uses_global_session_for_current_project_bundle() {
        let _home_lock = lock_home_mutation();
        let temp_root =
            std::env::temp_dir().join(format!("memd-live-overlay-{}", uuid::Uuid::new_v4()));
        let home = temp_root.join("home");
        let repo_root = temp_root.join("repo");
        let global_root = home.join(".memd");
        let local_bundle = repo_root.join(".memd");
        fs::create_dir_all(&global_root).expect("create global bundle");
        fs::create_dir_all(&local_bundle).expect("create local bundle");
        fs::write(
            global_root.join("config.json"),
            r#"{
  "project": "global",
  "namespace": "global",
  "agent": "codex",
  "session": "codex-fresh",
  "tab_id": "tab-alpha",
  "base_url": "http://100.104.154.24:8787"
}
"#,
        )
        .expect("write global config");
        fs::write(
            local_bundle.join("config.json"),
            r#"{
  "project": "memd",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-stale",
  "base_url": "http://100.104.154.24:8787",
  "workspace": "shared",
  "visibility": "workspace"
}
"#,
        )
        .expect("write local config");

        let original_home = std::env::var_os("HOME");
        let original_dir = std::env::current_dir().expect("read cwd");
        unsafe {
            std::env::set_var("HOME", &home);
        }
        std::env::set_current_dir(&repo_root).expect("set repo cwd");

        let overlay =
            resolve_live_session_overlay(&local_bundle, &repo_root, &default_global_bundle_root())
                .expect("resolve live overlay")
                .expect("overlay present");
        assert_eq!(overlay.session.as_deref(), Some("codex-fresh"));
        assert_eq!(overlay.tab_id.as_deref(), Some("tab-alpha"));

        std::env::set_current_dir(&original_dir).expect("restore cwd");
        if let Some(value) = original_home {
            unsafe {
                std::env::set_var("HOME", value);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
        fs::remove_dir_all(temp_root).expect("cleanup live overlay temp");
    }

    #[test]
    fn resolve_live_session_overlay_skips_plain_local_bundle_without_hive_scope() {
        let _home_lock = lock_home_mutation();
        let temp_root = std::env::temp_dir().join(format!(
            "memd-live-overlay-no-scope-{}",
            uuid::Uuid::new_v4()
        ));
        let home = temp_root.join("home");
        let repo_root = temp_root.join("repo");
        let global_root = home.join(".memd");
        let local_bundle = repo_root.join(".memd");
        fs::create_dir_all(&global_root).expect("create global bundle");
        fs::create_dir_all(&local_bundle).expect("create local bundle");
        fs::write(
            global_root.join("config.json"),
            r#"{
  "project": "global",
  "namespace": "global",
  "agent": "codex",
  "session": "codex-fresh",
  "tab_id": "tab-alpha",
  "base_url": "http://100.104.154.24:8787"
}
"#,
        )
        .expect("write global config");
        fs::write(
            local_bundle.join("config.json"),
            r#"{
  "project": "memd",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-stale",
  "base_url": "http://100.104.154.24:8787",
  "route": "auto",
  "intent": "current_task"
}
"#,
        )
        .expect("write local config");

        let original_home = std::env::var_os("HOME");
        let original_dir = std::env::current_dir().expect("read cwd");
        unsafe {
            std::env::set_var("HOME", &home);
        }
        std::env::set_current_dir(&repo_root).expect("set repo cwd");

        let overlay =
            resolve_live_session_overlay(&local_bundle, &repo_root, &default_global_bundle_root())
                .expect("resolve live overlay");
        assert!(overlay.is_none());

        std::env::set_current_dir(&original_dir).expect("restore cwd");
        if let Some(value) = original_home {
            unsafe {
                std::env::set_var("HOME", value);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
        fs::remove_dir_all(temp_root).expect("cleanup live overlay temp");
    }

    #[tokio::test]
    async fn run_hive_command_reports_live_session_rebind() {
        let _home_lock = lock_home_mutation();
        let temp_root =
            std::env::temp_dir().join(format!("memd-hive-rebind-{}", uuid::Uuid::new_v4()));
        let home = temp_root.join("home");
        let repo_root = temp_root.join("repo");
        let global_root = home.join(".memd");
        let local_bundle = repo_root.join(".memd");
        fs::create_dir_all(global_root.join("state")).expect("create global state");
        fs::create_dir_all(local_bundle.join("state")).expect("create local state");
        fs::write(
            global_root.join("config.json"),
            format!(
                r#"{{
  "project": "global",
  "namespace": "global",
  "agent": "codex",
  "session": "codex-fresh",
  "tab_id": "tab-alpha",
  "base_url": "{SHARED_MEMD_BASE_URL}"
}}
"#
            ),
        )
        .expect("write global config");
        fs::write(
            local_bundle.join("config.json"),
            format!(
                r#"{{
  "project": "memd",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-stale",
  "base_url": "{SHARED_MEMD_BASE_URL}",
  "route": "auto",
  "intent": "current_task",
  "workspace": "shared",
  "visibility": "workspace"
}}
"#
            ),
        )
        .expect("write local config");

        let original_home = std::env::var_os("HOME");
        let original_dir = std::env::current_dir().expect("read cwd");
        unsafe {
            std::env::set_var("HOME", &home);
        }
        std::env::set_current_dir(&repo_root).expect("set repo cwd");

        let response = run_hive_command(&HiveArgs {
            command: None,
            agent: None,
            project: None,
            namespace: None,
            global: false,
            project_root: Some(repo_root.clone()),
            seed_existing: false,
            session: None,
            tab_id: None,
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            capability: Vec::new(),
            hive_group: vec!["project:memd".to_string()],
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            output: local_bundle.clone(),
            base_url: SHARED_MEMD_BASE_URL.to_string(),
            rag_url: None,
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            publish_heartbeat: false,
            force: false,
            summary: false,
        })
        .await
        .expect("run hive command");

        assert_eq!(response.bundle_session.as_deref(), Some("codex-stale"));
        assert_eq!(response.live_session.as_deref(), Some("codex-fresh"));
        assert_eq!(response.session.as_deref(), Some("codex-fresh"));
        assert_eq!(
            response.rebased_from_session.as_deref(),
            Some("codex-stale")
        );

        let summary = render_hive_wire_summary(&response);
        assert!(summary.contains("bundle_session=codex-stale"));
        assert!(summary.contains("live_session=codex-fresh"));
        assert!(summary.contains("rebased_from=codex-stale"));

        std::env::set_current_dir(&original_dir).expect("restore cwd");
        if let Some(value) = original_home {
            unsafe {
                std::env::set_var("HOME", value);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
        fs::remove_dir_all(temp_root).expect("cleanup hive rebind temp");
    }

    #[tokio::test]
    async fn run_hive_command_surfaces_lane_reroute() {
        let root =
            std::env::temp_dir().join(format!("memd-hive-wire-reroute-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("current");
        let target_project = root.join("target");
        let current_bundle = current_project.join(".memd");
        let target_bundle = target_project.join(".memd");
        fs::create_dir_all(&current_bundle).expect("create current bundle");
        fs::create_dir_all(&target_bundle).expect("create target bundle");
        fs::create_dir_all(current_project.join(".planning")).expect("create current planning");
        fs::create_dir_all(target_project.join(".planning")).expect("create target planning");
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;

        write_test_bundle_config(&current_bundle, &base_url);
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
                base_url
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
                base_url: Some(base_url.clone()),
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

        let response = run_hive_command(&HiveArgs {
            command: None,
            agent: None,
            project: None,
            namespace: None,
            global: false,
            project_root: Some(current_project.clone()),
            seed_existing: false,
            session: None,
            tab_id: None,
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            capability: Vec::new(),
            hive_group: vec!["project:demo".to_string()],
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            output: current_bundle.clone(),
            base_url: base_url.clone(),
            rag_url: None,
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            publish_heartbeat: false,
            force: false,
            summary: false,
        })
        .await
        .expect("run hive command");

        assert!(response.lane_rerouted);
        assert!(response.lane_created);
        assert!(response.lane_surface.is_some());
        assert_ne!(PathBuf::from(&response.output), current_bundle);

        let receipts = state.receipts.lock().expect("lock receipts");
        assert!(
            receipts
                .iter()
                .any(|receipt| receipt.kind == "lane_reroute")
        );

        fs::remove_dir_all(root).expect("cleanup hive reroute root");
    }

    #[tokio::test]
    async fn run_tasks_command_blocks_mutating_writes_in_localhost_read_only_mode() {
        let dir = std::env::temp_dir().join(format!("memd-tasks-ro-{}", uuid::Uuid::new_v4()));
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

        let err = run_tasks_command(
            &TasksArgs {
                output: dir.clone(),
                upsert: true,
                assign_to_session: None,
                target_session: None,
                task_id: Some("task-1".to_string()),
                title: Some("keep work moving".to_string()),
                description: Some("refresh the plan".to_string()),
                status: Some("open".to_string()),
                mode: None,
                scope: vec!["src/main.rs".to_string()],
                request_help: false,
                request_review: false,
                all: false,
                view: None,
                summary: false,
                json: false,
            },
            SHARED_MEMD_BASE_URL,
        )
        .await
        .expect_err("shared write in localhost read-only mode should fail");
        assert!(
            err.to_string()
                .contains("localhost read-only fallback active")
        );

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[tokio::test]
    async fn run_hive_command_auto_creates_isolated_worker_lane_for_new_bundle() {
        let root =
            std::env::temp_dir().join(format!("memd-hive-auto-create-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("current");
        fs::create_dir_all(current_project.join(".planning")).expect("create current planning");
        fs::write(current_project.join("README.md"), "# current\n").expect("write readme");
        init_test_git_repo(&root);
        checkout_test_branch(&root, "feature/hive-shared");

        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
        let response = run_hive_command(&HiveArgs {
            command: None,
            agent: Some("codex".to_string()),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            global: false,
            project_root: Some(current_project.clone()),
            seed_existing: false,
            session: Some("codex-a".to_string()),
            tab_id: None,
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            capability: Vec::new(),
            hive_group: vec!["project:demo".to_string()],
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            output: default_init_output_path(),
            base_url: base_url.clone(),
            rag_url: None,
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            publish_heartbeat: false,
            force: false,
            summary: false,
        })
        .await
        .expect("run hive command");

        assert!(!response.lane_rerouted);
        assert!(response.lane_created);
        let lane = response.lane_surface.expect("lane surface");
        assert_eq!(
            lane.get("action").and_then(JsonValue::as_str),
            Some("auto_create")
        );
        let output = PathBuf::from(&response.output);
        assert_ne!(output, current_project.join(".memd"));
        assert!(output.join("config.json").exists());

        let receipts = state.receipts.lock().expect("lock receipts");
        assert!(receipts.iter().any(|receipt| receipt.kind == "lane_create"));

        fs::remove_dir_all(root).expect("cleanup hive auto create root");
    }

    #[tokio::test]
    async fn run_coordination_command_records_queen_decisions() {
        let dir =
            std::env::temp_dir().join(format!("memd-coordination-queen-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(dir.join("state")).expect("create temp dir");
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;
        fs::write(
            dir.join("config.json"),
            format!(
                r#"{{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "queen-a",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "{}",
  "route": "auto",
  "intent": "current_task"
}}
"#,
                base_url
            ),
        )
        .expect("write config");

        let response = run_coordination_command(
            &CoordinationArgs {
                output: dir.clone(),
                view: Some("lanes".to_string()),
                changes_only: false,
                watch: false,
                interval_secs: 30,
                recover_session: None,
                retire_session: None,
                to_session: Some("bee-b".to_string()),
                deny_session: Some("bee-b".to_string()),
                reroute_session: Some("bee-c".to_string()),
                handoff_scope: Some("file:src/main.rs".to_string()),
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("coordination response");

        assert!(
            response
                .lane_receipts
                .iter()
                .any(|receipt| receipt.kind == "queen_deny")
        );
        assert!(
            response
                .lane_receipts
                .iter()
                .any(|receipt| receipt.kind == "queen_reroute")
        );
        assert!(
            response
                .lane_receipts
                .iter()
                .any(|receipt| receipt.kind == "queen_handoff")
        );

        fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[tokio::test]
    async fn run_coordination_command_falls_back_to_local_truth_when_backend_unreachable() {
        let dir = std::env::temp_dir().join(format!(
            "memd-coordination-offline-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(dir.join("state")).expect("create temp dir");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "queen-a",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:9",
  "route": "auto",
  "intent": "current_task"
}
"#,
        )
        .expect("write config");

        let response = run_coordination_command(
            &CoordinationArgs {
                output: dir.clone(),
                view: Some("overview".to_string()),
                changes_only: false,
                watch: false,
                interval_secs: 30,
                recover_session: None,
                retire_session: None,
                to_session: None,
                deny_session: None,
                reroute_session: None,
                handoff_scope: None,
                summary: false,
            },
            "http://127.0.0.1:9",
        )
        .await
        .expect("offline coordination response");

        assert_eq!(response.current_session, "queen-a");
        assert!(response.inbox.messages.is_empty());
        assert!(response.inbox.owned_tasks.is_empty());
        assert!(response.receipts.is_empty());

        fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[tokio::test]
    async fn run_coordination_command_fails_fast_for_mutations_when_backend_unreachable() {
        let dir = std::env::temp_dir().join(format!(
            "memd-coordination-offline-mutation-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(dir.join("state")).expect("create temp dir");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "namespace": "main",
  "agent": "codex",
  "session": "queen-a",
  "workspace": "shared",
  "visibility": "workspace",
  "base_url": "http://127.0.0.1:9",
  "route": "auto",
  "intent": "current_task"
}
"#,
        )
        .expect("write config");

        let err = run_coordination_command(
            &CoordinationArgs {
                output: dir.clone(),
                view: Some("overview".to_string()),
                changes_only: false,
                watch: false,
                interval_secs: 30,
                recover_session: None,
                retire_session: None,
                to_session: None,
                deny_session: Some("bee-b".to_string()),
                reroute_session: None,
                handoff_scope: None,
                summary: false,
            },
            "http://127.0.0.1:9",
        )
        .await
        .expect_err("offline mutation should fail fast");

        assert!(
            err.to_string().contains("coordination backend unreachable"),
            "{err}"
        );

        fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[tokio::test]
    async fn read_bundle_status_reports_live_session_rebind() {
        let _home_lock = lock_home_mutation();
        let temp_root =
            std::env::temp_dir().join(format!("memd-status-rebind-{}", uuid::Uuid::new_v4()));
        let home = temp_root.join("home");
        let repo_root = temp_root.join("repo");
        let global_root = home.join(".memd");
        let local_bundle = repo_root.join(".memd");
        fs::create_dir_all(global_root.join("state")).expect("create global state");
        fs::create_dir_all(local_bundle.join("state")).expect("create local state");
        fs::write(
            global_root.join("config.json"),
            format!(
                r#"{{
  "project": "global",
  "namespace": "global",
  "agent": "codex",
  "session": "codex-fresh",
  "tab_id": "tab-alpha",
  "base_url": "{SHARED_MEMD_BASE_URL}"
}}
"#
            ),
        )
        .expect("write global config");
        fs::write(
            local_bundle.join("config.json"),
            format!(
                r#"{{
  "project": "memd",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-stale",
  "base_url": "{SHARED_MEMD_BASE_URL}",
  "route": "auto",
  "intent": "current_task",
  "workspace": "shared",
  "visibility": "workspace"
}}
"#
            ),
        )
        .expect("write local config");

        let original_home = std::env::var_os("HOME");
        let original_dir = std::env::current_dir().expect("read cwd");
        unsafe {
            std::env::set_var("HOME", &home);
        }
        std::env::set_current_dir(&repo_root).expect("set repo cwd");

        let status = read_bundle_status(&local_bundle, SHARED_MEMD_BASE_URL)
            .await
            .expect("read bundle status");
        let overlay = status
            .get("session_overlay")
            .expect("session overlay present");
        assert_eq!(
            overlay
                .get("bundle_session")
                .and_then(serde_json::Value::as_str),
            Some("codex-stale")
        );
        assert_eq!(
            overlay
                .get("live_session")
                .and_then(serde_json::Value::as_str),
            Some("codex-fresh")
        );
        assert_eq!(
            overlay
                .get("rebased_from")
                .and_then(serde_json::Value::as_str),
            Some("codex-stale")
        );

        std::env::set_current_dir(&original_dir).expect("restore cwd");
        if let Some(value) = original_home {
            unsafe {
                std::env::set_var("HOME", value);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
        fs::remove_dir_all(temp_root).expect("cleanup status rebind temp");
    }

    #[tokio::test]
    async fn run_session_command_rebinds_local_bundle_to_live_session() {
        let _home_lock = lock_home_mutation();
        let temp_root =
            std::env::temp_dir().join(format!("memd-session-rebind-{}", uuid::Uuid::new_v4()));
        let home = temp_root.join("home");
        let repo_root = temp_root.join("repo");
        let global_root = home.join(".memd");
        let local_bundle = repo_root.join(".memd");
        fs::create_dir_all(global_root.join("state")).expect("create global state");
        fs::create_dir_all(local_bundle.join("state")).expect("create local state");
        fs::write(
            global_root.join("config.json"),
            format!(
                r#"{{
  "project": "global",
  "namespace": "global",
  "agent": "codex",
  "session": "codex-fresh",
  "tab_id": "tab-alpha",
  "base_url": "{SHARED_MEMD_BASE_URL}"
}}
"#
            ),
        )
        .expect("write global config");
        fs::write(
            local_bundle.join("config.json"),
            format!(
                r#"{{
  "project": "memd",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-stale",
  "tab_id": "tab-stale",
  "base_url": "{SHARED_MEMD_BASE_URL}",
  "workspace": "shared",
  "visibility": "workspace"
}}
"#
            ),
        )
        .expect("write local config");
        fs::write(
            local_bundle.join("env"),
            "MEMD_SESSION=codex-stale\nMEMD_AGENT=codex@codex-stale\n",
        )
        .expect("write env");
        fs::write(
            local_bundle.join("env.ps1"),
            "$env:MEMD_SESSION = \"codex-stale\"\n$env:MEMD_AGENT = \"codex@codex-stale\"\n",
        )
        .expect("write env ps1");

        let original_home = std::env::var_os("HOME");
        let original_dir = std::env::current_dir().expect("read cwd");
        unsafe {
            std::env::set_var("HOME", &home);
        }
        std::env::set_current_dir(&repo_root).expect("set repo cwd");

        let response = run_session_command(
            &SessionArgs {
                output: local_bundle.clone(),
                rebind: true,
                reconcile: false,
                retire_session: None,
                summary: false,
            },
            SHARED_MEMD_BASE_URL,
        )
        .await
        .expect("run session command");
        assert_eq!(response.action, "rebind");
        assert_eq!(response.bundle_session.as_deref(), Some("codex-fresh"));
        assert_eq!(response.live_session.as_deref(), Some("codex-fresh"));

        let config = fs::read_to_string(local_bundle.join("config.json")).expect("read config");
        assert!(config.contains("\"session\": \"codex-fresh\""));
        assert!(config.contains("\"tab_id\": \"tab-alpha\""));

        std::env::set_current_dir(&original_dir).expect("restore cwd");
        if let Some(value) = original_home {
            unsafe {
                std::env::set_var("HOME", value);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
        fs::remove_dir_all(temp_root).expect("cleanup session rebind temp");
    }

    #[tokio::test]
    async fn claude_runtime_stack_emits_coordinated_truthful_continuous_summary() {
        let _home_lock = lock_home_mutation();
        let temp_root =
            std::env::temp_dir().join(format!("memd-runtime-stack-{}", uuid::Uuid::new_v4()));
        let home = temp_root.join("home");
        let repo_root = temp_root.join("repo");
        let sibling_root = temp_root.join("sibling");
        let global_root = home.join(".memd");
        let local_bundle = repo_root.join(".memd");
        let sibling_bundle = sibling_root.join(".memd");
        fs::create_dir_all(global_root.join("state")).expect("create global state");
        fs::create_dir_all(local_bundle.join("state")).expect("create local state");
        fs::create_dir_all(sibling_bundle.join("state")).expect("create sibling state");
        fs::write(
            global_root.join("config.json"),
            format!(
                r#"{{
  "project": "global",
  "namespace": "global",
  "agent": "codex",
  "session": "codex-fresh",
  "tab_id": "tab-alpha",
  "base_url": "{SHARED_MEMD_BASE_URL}"
}}
"#
            ),
        )
        .expect("write global config");
        fs::write(
            local_bundle.join("config.json"),
            format!(
                r#"{{
  "project": "memd",
  "namespace": "main",
  "agent": "codex",
  "session": "codex-stale",
  "base_url": "{SHARED_MEMD_BASE_URL}",
  "route": "auto",
  "intent": "current_task",
  "workspace": "shared",
  "visibility": "workspace"
}}
"#
            ),
        )
        .expect("write local config");
        fs::write(
            sibling_bundle.join("config.json"),
            format!(
                r#"{{
  "project": "memd-helper",
  "namespace": "main",
  "agent": "claude-code",
  "session": "claude-live",
  "tab_id": "tab-beta",
  "hive_system": "codex",
  "hive_role": "agent",
  "hive_groups": ["project:memd"],
  "authority": "participant",
  "base_url": "{SHARED_MEMD_BASE_URL}",
  "workspace": "shared",
  "visibility": "workspace"
}}
"#
            ),
        )
        .expect("write sibling config");
        fs::write(
            bundle_heartbeat_state_path(&sibling_bundle),
            serde_json::to_string_pretty(&BundleHeartbeatState {
                session: Some("claude-live".to_string()),
                agent: Some("claude-code".to_string()),
                effective_agent: Some("claude-code@claude-live".to_string()),
                tab_id: Some("tab-beta".to_string()),
                hive_system: Some("codex".to_string()),
                hive_role: Some("agent".to_string()),
                worker_name: Some("claude-code".to_string()),
                display_name: None,
                role: Some("agent".to_string()),
                capabilities: vec!["memory".to_string(), "coordination".to_string()],
                hive_groups: vec!["project:memd".to_string()],
                lane_id: Some(sibling_root.display().to_string()),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                heartbeat_model: Some(default_heartbeat_model()),
                project: Some("memd-helper".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                repo_root: Some(repo_root.display().to_string()),
                worktree_root: Some(sibling_root.display().to_string()),
                branch: Some("feature/claude-live".to_string()),
                base_branch: Some("main".to_string()),
                visibility: Some("workspace".to_string()),
                base_url: Some(SHARED_MEMD_BASE_URL.to_string()),
                base_url_healthy: Some(true),
                host: Some("workstation".to_string()),
                pid: Some(4242),
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: Some("Handle coordination backlog".to_string()),
                pressure: Some("Keep the hive lane clean".to_string()),
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
            .expect("serialize sibling heartbeat")
                + "\n",
        )
        .expect("write sibling heartbeat");

        let original_home = std::env::var_os("HOME");
        let original_dir = std::env::current_dir().expect("read cwd");
        unsafe {
            std::env::set_var("HOME", &home);
        }
        std::env::set_current_dir(&repo_root).expect("set repo cwd");

        let wire = run_hive_command(&HiveArgs {
            command: None,
            agent: None,
            project: None,
            namespace: None,
            global: false,
            project_root: Some(repo_root.clone()),
            seed_existing: false,
            session: None,
            tab_id: None,
            hive_system: Some("codex".to_string()),
            hive_role: Some("agent".to_string()),
            capability: Vec::new(),
            hive_group: vec!["project:memd".to_string()],
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            output: local_bundle.clone(),
            base_url: SHARED_MEMD_BASE_URL.to_string(),
            rag_url: None,
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            publish_heartbeat: false,
            force: false,
            summary: false,
        })
        .await
        .expect("run hive command");
        assert_eq!(wire.session.as_deref(), Some("codex-fresh"));

        let mut awareness = read_project_awareness(&AwarenessArgs {
            output: local_bundle.clone(),
            root: Some(temp_root.clone()),
            include_current: true,
            summary: false,
        })
        .await
        .expect("read awareness");
        awareness.entries.push(ProjectAwarenessEntry {
            project_dir: "remote".to_string(),
            bundle_root: format!("remote:{SHARED_MEMD_BASE_URL}:session-stale"),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            agent: Some("claude-code".to_string()),
            session: Some("session-stale".to_string()),
            tab_id: None,
            effective_agent: Some("claude-code@session-stale".to_string()),
            hive_system: None,
            hive_role: None,
            capabilities: vec!["memory".to_string()],
            hive_groups: Vec::new(),
            hive_group_goal: None,
            authority: None,
            base_url: Some(SHARED_MEMD_BASE_URL.to_string()),
            presence: "stale".to_string(),
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
            last_updated: Some(Utc::now() - chrono::TimeDelta::minutes(9)),
        });
        awareness.entries.push(ProjectAwarenessEntry {
            project_dir: "remote".to_string(),
            bundle_root: format!("remote:{SHARED_MEMD_BASE_URL}:session-dead"),
            project: Some("memd".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            agent: Some("codex".to_string()),
            session: Some("session-dead".to_string()),
            tab_id: None,
            effective_agent: Some("codex@session-dead".to_string()),
            hive_system: None,
            hive_role: None,
            capabilities: vec!["memory".to_string()],
            hive_groups: Vec::new(),
            hive_group_goal: None,
            authority: None,
            base_url: Some(SHARED_MEMD_BASE_URL.to_string()),
            presence: "dead".to_string(),
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
            last_updated: Some(Utc::now() - chrono::TimeDelta::minutes(30)),
        });
        awareness.entries.push(ProjectAwarenessEntry {
            project_dir: "remote".to_string(),
            bundle_root: format!("remote:{SHARED_MEMD_BASE_URL}:session-superseded"),
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
            base_url: Some(SHARED_MEMD_BASE_URL.to_string()),
            presence: "stale".to_string(),
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
            last_updated: Some(Utc::now() - chrono::TimeDelta::minutes(12)),
        });
        let summary = render_project_awareness_summary(&awareness);
        assert!(summary.contains("current_session:"));
        assert!(summary.contains("active_hive_sessions:"));
        assert!(summary.contains("! stale_remote_sessions="));
        assert!(summary.contains("stale_sessions:"));
        assert!(summary.contains("hidden_remote_dead="));
        assert!(summary.contains("hidden_superseded_stale=1"));
        assert!(summary.contains("session=codex-fresh"));
        assert!(summary.contains("truth=current"));

        std::env::set_current_dir(&original_dir).expect("restore cwd");
        if let Some(value) = original_home {
            unsafe {
                std::env::set_var("HOME", value);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
        fs::remove_dir_all(temp_root).expect("cleanup runtime stack temp");
    }

    #[test]
    fn shared_awareness_scope_prefers_workspace_over_project_filters() {
        let runtime = BundleRuntimeConfig {
            project: Some("repo-a".to_string()),
            namespace: Some("main".to_string()),
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
            workspace: Some("initiative-alpha".to_string()),
            visibility: None,
            heartbeat_model: None,
            auto_short_term_capture: true,
            authority_policy: BundleAuthorityPolicy::default(),
            authority_state: BundleAuthorityState::default(),
        };

        let (project, namespace) = shared_awareness_scope(Some(&runtime));
        assert!(project.is_none());
        assert!(namespace.is_none());

        let runtime = BundleRuntimeConfig {
            workspace: None,
            ..runtime
        };
        let (project, namespace) = shared_awareness_scope(Some(&runtime));
        assert_eq!(project.as_deref(), Some("repo-a"));
        assert_eq!(namespace.as_deref(), Some("main"));
    }

    #[test]
    fn describes_eval_changes_against_baseline() {
        let baseline = BundleEvalResponse {
            bundle_root: ".memd".to_string(),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            status: "usable".to_string(),
            score: 72,
            working_records: 2,
            context_records: 1,
            rehydration_items: 1,
            inbox_items: 3,
            workspace_lanes: 1,
            semantic_hits: 0,
            findings: Vec::new(),
            baseline_score: None,
            score_delta: None,
            changes: Vec::new(),
            recommendations: Vec::new(),
        };
        let snapshot = ResumeSnapshot {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "general".to_string(),
            context: memd_schema::CompactContextResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::General,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                records: vec![memd_schema::CompactMemoryRecord {
                    id: uuid::Uuid::new_v4(),
                    record: "ctx".to_string(),
                }],
            },
            working: memd_schema::WorkingMemoryResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::General,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                budget_chars: 1600,
                used_chars: 100,
                remaining_chars: 1500,
                truncated: false,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 8,
                    max_chars_per_item: 220,
                    budget_chars: 1600,
                    rehydration_limit: 4,
                },
                records: vec![
                    memd_schema::CompactMemoryRecord {
                        id: uuid::Uuid::new_v4(),
                        record: "one".to_string(),
                    },
                    memd_schema::CompactMemoryRecord {
                        id: uuid::Uuid::new_v4(),
                        record: "two".to_string(),
                    },
                    memd_schema::CompactMemoryRecord {
                        id: uuid::Uuid::new_v4(),
                        record: "three".to_string(),
                    },
                ],
                evicted: Vec::new(),
                rehydration_queue: vec![
                    memd_schema::MemoryRehydrationRecord {
                        id: None,
                        kind: "source".to_string(),
                        label: "artifact".to_string(),
                        summary: "more".to_string(),
                        reason: None,
                        source_agent: None,
                        source_system: None,
                        source_path: None,
                        source_quality: None,
                        recorded_at: None,
                    },
                    memd_schema::MemoryRehydrationRecord {
                        id: None,
                        kind: "source".to_string(),
                        label: "artifact-2".to_string(),
                        summary: "more".to_string(),
                        reason: None,
                        source_agent: None,
                        source_system: None,
                        source_path: None,
                        source_quality: None,
                        recorded_at: None,
                    },
                ],
                traces: Vec::new(),
                semantic_consolidation: None,
            },
            inbox: memd_schema::MemoryInboxResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::General,
                items: vec![],
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: vec![
                    memd_schema::WorkspaceMemoryRecord {
                        project: Some("demo".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: Some("team-alpha".to_string()),
                        visibility: memd_schema::MemoryVisibility::Workspace,
                        item_count: 3,
                        active_count: 3,
                        candidate_count: 0,
                        contested_count: 0,
                        source_lane_count: 1,
                        avg_confidence: 0.9,
                        trust_score: 0.9,
                        last_seen_at: None,
                        tags: vec![],
                    },
                    memd_schema::WorkspaceMemoryRecord {
                        project: Some("demo".to_string()),
                        namespace: Some("main".to_string()),
                        workspace: Some("shared".to_string()),
                        visibility: memd_schema::MemoryVisibility::Workspace,
                        item_count: 2,
                        active_count: 2,
                        candidate_count: 0,
                        contested_count: 0,
                        source_lane_count: 1,
                        avg_confidence: 0.8,
                        trust_score: 0.8,
                        last_seen_at: None,
                        tags: vec![],
                    },
                ],
            },
            sources: memd_schema::SourceMemoryResponse {
                sources: Vec::new(),
            },
            semantic: Some(memd_rag::RagRetrieveResponse {
                status: "ok".to_string(),
                mode: memd_rag::RagRetrieveMode::Auto,
                items: vec![memd_rag::RagRetrieveItem {
                    content: "semantic".to_string(),
                    source: Some("wiki/demo.md".to_string()),
                    score: 0.9,
                }],
            }),
            claims: SessionClaimsState::default(),
            recent_repo_changes: vec!["repo clean".to_string()],
            change_summary: Vec::new(),
            resume_state_age_minutes: None,
            refresh_recommended: false,
        };

        let changes = describe_eval_changes(&baseline, 88, &snapshot);
        assert!(changes.iter().any(|value| value.contains("score 72 -> 88")));
        assert!(changes.iter().any(|value| value.contains("working 2 -> 3")));
        assert!(
            changes
                .iter()
                .any(|value| value.contains("rehydration 1 -> 2"))
        );
        assert!(changes.iter().any(|value| value.contains("inbox 3 -> 0")));
        assert!(changes.iter().any(|value| value.contains("lanes 1 -> 2")));
        assert!(
            changes
                .iter()
                .any(|value| value.contains("semantic 0 -> 1"))
        );
    }

    #[test]
    fn eval_failure_reason_respects_score_threshold() {
        let response = BundleEvalResponse {
            bundle_root: ".memd".to_string(),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            status: "weak".to_string(),
            score: 62,
            working_records: 0,
            context_records: 0,
            rehydration_items: 0,
            inbox_items: 0,
            workspace_lanes: 0,
            semantic_hits: 0,
            findings: vec!["no working memory".to_string()],
            baseline_score: Some(70),
            score_delta: Some(-8),
            changes: vec!["score 70 -> 62".to_string()],
            recommendations: vec!["capture durable memory".to_string()],
        };

        let reason = eval_failure_reason(&response, Some(70), false).expect("threshold failure");
        assert!(reason.contains("score 62"));
        assert!(reason.contains("threshold 70"));
    }

    #[test]
    fn eval_failure_reason_respects_regression_gate() {
        let response = BundleEvalResponse {
            bundle_root: ".memd".to_string(),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            status: "usable".to_string(),
            score: 79,
            working_records: 3,
            context_records: 2,
            rehydration_items: 2,
            inbox_items: 1,
            workspace_lanes: 1,
            semantic_hits: 2,
            findings: Vec::new(),
            baseline_score: Some(83),
            score_delta: Some(-4),
            changes: vec!["score 83 -> 79".to_string()],
            recommendations: vec!["write a fresh baseline".to_string()],
        };

        let reason = eval_failure_reason(&response, None, true).expect("regression failure");
        assert!(reason.contains("baseline 83"));
        assert!(reason.contains("to 79"));
        assert!(reason.contains("delta -4"));
    }

    #[test]
    fn eval_failure_reason_passes_when_gates_are_clear() {
        let response = BundleEvalResponse {
            bundle_root: ".memd".to_string(),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            status: "strong".to_string(),
            score: 91,
            working_records: 4,
            context_records: 3,
            rehydration_items: 2,
            inbox_items: 0,
            workspace_lanes: 2,
            semantic_hits: 3,
            findings: Vec::new(),
            baseline_score: Some(89),
            score_delta: Some(2),
            changes: vec!["score 89 -> 91".to_string()],
            recommendations: Vec::new(),
        };

        assert!(eval_failure_reason(&response, Some(80), true).is_none());
    }

    #[test]
    fn build_eval_recommendations_surfaces_actionable_followups() {
        let snapshot = ResumeSnapshot {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            workspace: Some("team-alpha".to_string()),
            visibility: Some("workspace".to_string()),
            route: "auto".to_string(),
            intent: "general".to_string(),
            context: memd_schema::CompactContextResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::General,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                records: Vec::new(),
            },
            working: memd_schema::WorkingMemoryResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::General,
                retrieval_order: vec![memd_schema::MemoryScope::Project],
                budget_chars: 1600,
                used_chars: 0,
                remaining_chars: 1600,
                truncated: false,
                policy: memd_schema::WorkingMemoryPolicyState {
                    admission_limit: 8,
                    max_chars_per_item: 220,
                    budget_chars: 1600,
                    rehydration_limit: 4,
                },
                records: Vec::new(),
                evicted: Vec::new(),
                rehydration_queue: Vec::new(),
                traces: Vec::new(),
                semantic_consolidation: None,
            },
            inbox: memd_schema::MemoryInboxResponse {
                route: memd_schema::RetrievalRoute::Auto,
                intent: memd_schema::RetrievalIntent::General,
                items: vec![
                    memd_schema::InboxMemoryItem {
                        item: memd_schema::MemoryItem {
                            id: uuid::Uuid::new_v4(),
                            content: "one".to_string(),
                            redundancy_key: None,
                            belief_branch: None,
                            preferred: true,
                            kind: memd_schema::MemoryKind::Decision,
                            scope: memd_schema::MemoryScope::Project,
                            project: Some("demo".to_string()),
                            namespace: Some("main".to_string()),
                            workspace: Some("team-alpha".to_string()),
                            visibility: memd_schema::MemoryVisibility::Workspace,
                            source_agent: None,
                            source_system: None,
                            source_path: None,
                            source_quality: None,
                            confidence: 0.6,
                            ttl_seconds: None,
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                            last_verified_at: None,
                            supersedes: Vec::new(),
                            tags: Vec::new(),
                            status: memd_schema::MemoryStatus::Active,
                            stage: memd_schema::MemoryStage::Candidate,
                        },
                        reasons: Vec::new(),
                    };
                    6
                ],
            },
            workspaces: memd_schema::WorkspaceMemoryResponse {
                workspaces: Vec::new(),
            },
            sources: memd_schema::SourceMemoryResponse {
                sources: Vec::new(),
            },
            semantic: Some(memd_rag::RagRetrieveResponse {
                status: "ok".to_string(),
                mode: memd_rag::RagRetrieveMode::Auto,
                items: Vec::new(),
            }),
            claims: SessionClaimsState::default(),
            recent_repo_changes: vec!["repo clean".to_string()],
            change_summary: Vec::new(),
            resume_state_age_minutes: None,
            refresh_recommended: false,
        };

        let recommendations = build_eval_recommendations(&snapshot, 62);
        assert!(
            recommendations
                .iter()
                .any(|value| value.contains("memd remember"))
        );
        assert!(
            recommendations
                .iter()
                .any(|value| value.contains("compact context"))
        );
        assert!(
            recommendations
                .iter()
                .any(|value| value.contains("rehydrate deeper context"))
        );
        assert!(
            recommendations
                .iter()
                .any(|value| value.contains("workspace or visibility"))
        );
        assert!(
            recommendations
                .iter()
                .any(|value| value.contains("inbox pressure"))
        );
        assert!(
            recommendations
                .iter()
                .any(|value| value.contains("LightRAG"))
        );
        assert!(
            recommendations
                .iter()
                .any(|value| value.contains("write a fresh baseline"))
        );
    }

    #[tokio::test]
    async fn run_maintain_command_persists_scan_report() {
        let dir = std::env::temp_dir().join(format!("memd-maintain-scan-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{"project":"demo","namespace":"main","agent":"codex","session":"session-a","auto_short_term_capture":false}"#,
        )
        .expect("write config");

        let base_url = spawn_mock_memory_server().await;
        let report = run_maintain_command(
            &MaintainArgs {
                output: dir.clone(),
                mode: "scan".to_string(),
                apply: false,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("run maintain scan");

        assert_eq!(report.mode.as_str(), "scan");
        assert!(report.receipt_id.is_some());
        assert!(report.findings.iter().any(|value| value.contains("memory")));
        assert!(dir.join("maintenance").join("latest.json").exists());
        assert!(dir.join("maintenance").join("latest.md").exists());

        fs::remove_dir_all(dir).expect("cleanup temp bundle");
    }

    #[test]
    fn render_maintain_summary_surfaces_receipt_and_counts() {
        let summary = render_maintain_summary(&MaintainReport {
            mode: "compact".to_string(),
            receipt_id: Some("receipt-1".to_string()),
            compacted_items: 3,
            refreshed_items: 0,
            repaired_items: 1,
            findings: vec!["compacted stale duplicates".to_string()],
            generated_at: Utc::now(),
        });
        assert!(summary.contains("maintain mode=compact"));
        assert!(summary.contains("receipt=receipt-1"));
        assert!(summary.contains("compacted=3"));
        assert!(summary.contains("repaired=1"));
    }

    #[test]
    fn suggest_coordination_actions_emits_multi_priority_output() {
        let now = Utc::now();
        let inbox = HiveCoordinationInboxResponse {
            messages: vec![
                HiveMessageRecord {
                    id: "m-1".to_string(),
                    kind: "status_check".to_string(),
                    from_session: "hive-a".to_string(),
                    from_agent: None,
                    to_session: "codex".to_string(),
                    project: None,
                    namespace: None,
                    workspace: None,
                    content: "review this artifact".to_string(),
                    created_at: now,
                    acknowledged_at: None,
                },
                HiveMessageRecord {
                    id: "m-2".to_string(),
                    kind: "help_request".to_string(),
                    from_session: "hive-b".to_string(),
                    from_agent: None,
                    to_session: "codex".to_string(),
                    project: None,
                    namespace: None,
                    workspace: None,
                    content: "another request".to_string(),
                    created_at: now,
                    acknowledged_at: None,
                },
            ],
            owned_tasks: vec![],
            help_tasks: vec![],
            review_tasks: vec![],
        };

        let stale_sessions = vec!["hive-stale"];
        let active_hives = vec![ProjectAwarenessEntry {
            project_dir: "remote".to_string(),
            bundle_root: "remote:http://127.0.0.1:8787:hive-helper".to_string(),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: None,
            branch: None,
            base_branch: None,
            agent: Some("agent-shell".to_string()),
            session: Some("hive-helper".to_string()),
            tab_id: Some("tab-helper".to_string()),
            effective_agent: Some("agent-shell@hive-helper".to_string()),
            hive_system: Some("agent-shell".to_string()),
            hive_role: Some("runtime-shell".to_string()),
            capabilities: vec!["shell".to_string(), "exec".to_string()],
            hive_groups: vec!["runtime-core".to_string(), "dependency-owners".to_string()],
            hive_group_goal: Some("stabilize runtime execution".to_string()),
            authority: Some("worker".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            presence: "active".to_string(),
            host: Some("vm-a".to_string()),
            pid: Some(42),
            active_claims: 0,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            topic_claim: None,
            scope_claims: Vec::new(),
            task_id: None,
            focus: Some("repair runtime dependencies".to_string()),
            pressure: None,
            next_recovery: None,
            last_updated: Some(now),
        }];
        let claims = vec![
            SessionClaim {
                scope: "shared/src.rs".to_string(),
                session: Some("hive-stale".to_string()),
                tab_id: None,
                agent: Some("claude".to_string()),
                effective_agent: Some("codex".to_string()),
                project: None,
                workspace: None,
                host: None,
                pid: None,
                acquired_at: now,
                expires_at: now,
            },
            SessionClaim {
                scope: "shared/src.rs".to_string(),
                session: Some("hive-contender".to_string()),
                tab_id: None,
                agent: None,
                effective_agent: None,
                project: None,
                workspace: None,
                host: None,
                pid: None,
                acquired_at: now,
                expires_at: now,
            },
        ];
        let tasks = vec![
            HiveTaskRecord {
                task_id: "task-exclusive".to_string(),
                title: "edit shared".to_string(),
                description: None,
                status: "assigned".to_string(),
                coordination_mode: "exclusive_write".to_string(),
                session: Some("hive-owner".to_string()),
                agent: Some("hive-owner".to_string()),
                effective_agent: None,
                project: None,
                namespace: None,
                workspace: None,
                claim_scopes: vec!["shared/src.rs".to_string()],
                help_requested: false,
                review_requested: false,
                created_at: now,
                updated_at: now,
            },
            HiveTaskRecord {
                task_id: "task-review".to_string(),
                title: "run review".to_string(),
                description: None,
                status: "in_progress".to_string(),
                coordination_mode: "shared_review".to_string(),
                session: Some("codex".to_string()),
                agent: Some("coder".to_string()),
                effective_agent: None,
                project: None,
                namespace: None,
                workspace: None,
                claim_scopes: vec![],
                help_requested: false,
                review_requested: false,
                created_at: now,
                updated_at: now,
            },
            HiveTaskRecord {
                task_id: "task-help".to_string(),
                title: "parallel assist".to_string(),
                description: None,
                status: "in_progress".to_string(),
                coordination_mode: "help_only".to_string(),
                session: Some("codex".to_string()),
                agent: Some("coder".to_string()),
                effective_agent: None,
                project: None,
                namespace: None,
                workspace: None,
                claim_scopes: vec![],
                help_requested: false,
                review_requested: false,
                created_at: now,
                updated_at: now,
            },
        ];
        let policy_conflicts = vec!["runtime dependency conflict for shared scope".to_string()];

        let suggestions = suggest_coordination_actions(
            &inbox,
            &stale_sessions,
            &active_hives,
            &claims,
            &tasks,
            "codex",
            &policy_conflicts,
            None,
            &[],
        );

        assert_eq!(
            suggestions
                .iter()
                .filter(|s| s.action == "ack_message")
                .count(),
            2,
            "each inbox message should produce its own ack suggestion"
        );
        assert!(suggestions.iter().any(|s| s.action == "recover_session"));
        assert!(suggestions.iter().any(|s| s.action == "assign_scope"));
        assert!(suggestions.iter().any(|s| s.action == "request_review"));
        assert!(suggestions.iter().any(|s| s.action == "request_help"));
        assert!(
            suggestions
                .iter()
                .filter(|s| matches!(s.action.as_str(), "request_review" | "request_help"))
                .all(|s| s.target_session.as_deref() == Some("hive-helper"))
        );
        assert!(
            suggestions
                .iter()
                .any(|s| s.stale_session.as_deref() == Some("hive-stale"))
        );
    }

    #[test]
    fn suggest_coordination_actions_retires_stale_session_without_owned_work() {
        let suggestions = suggest_coordination_actions(
            &HiveCoordinationInboxResponse {
                messages: Vec::new(),
                owned_tasks: Vec::new(),
                help_tasks: Vec::new(),
                review_tasks: Vec::new(),
            },
            &["hive-stale-empty"],
            &[],
            &[],
            &[],
            "codex",
            &[],
            None,
            &[],
        );

        assert!(
            suggestions
                .iter()
                .any(|value| value.action == "retire_session"
                    && value.stale_session.as_deref() == Some("hive-stale-empty"))
        );
        assert!(
            !suggestions
                .iter()
                .any(|value| value.action == "recover_session"),
            "stale sessions without owned work should retire instead of recover"
        );
    }

    #[test]
    fn build_gap_candidates_generates_core_gap_signals() {
        let output = std::env::temp_dir().join(format!("memd-gap-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&output).expect("create temp output");

        let runtime = None;
        let resume = None;
        let commits = vec!["abc".to_string(), "def".to_string()];
        let mut evidence = Vec::new();

        let candidates = build_gap_candidates(
            &output,
            &runtime,
            &resume,
            None,
            None,
            None,
            None,
            None,
            &commits,
            &mut evidence,
            None,
        );

        assert!(
            candidates
                .iter()
                .any(|value| value.id == "memory:no_eval_snapshot"),
            "baseline eval signal should be present when no eval exists"
        );
        assert!(
            candidates
                .iter()
                .any(|value| value.id == "memory:missing_resume_state"),
            "resume signal should be present when resume and state are missing"
        );
        assert!(
            candidates
                .iter()
                .any(|value| value.id == "coordination:coordination_unreachable"),
            "coordination signal should be present when coordination snapshot is unavailable"
        );
        assert!(
            !evidence.is_empty(),
            "recent commits should generate at least one evidence string"
        );

        fs::remove_dir_all(&output).expect("cleanup temp output");
    }

    #[test]
    fn build_gap_candidates_surfaces_unhived_active_sessions() {
        let output =
            std::env::temp_dir().join(format!("memd-gap-awareness-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&output).expect("create temp output");

        let awareness = ProjectAwarenessResponse {
            root: "server:http://127.0.0.1:8787".to_string(),
            current_bundle: output.display().to_string(),
            collisions: vec!["base_url http://127.0.0.1:8787 used by 2 bundles".to_string()],
            entries: vec![
                ProjectAwarenessEntry {
                    project_dir: output.display().to_string(),
                    bundle_root: output.display().to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("session-a".to_string()),
                    tab_id: Some("tab-a".to_string()),
                    effective_agent: Some("codex@session-a".to_string()),
                    hive_system: Some("codex".to_string()),
                    hive_role: Some("agent".to_string()),
                    capabilities: vec!["memory".to_string()],
                    hive_groups: vec!["project:memd".to_string()],
                    hive_group_goal: Some("coordinate memd".to_string()),
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: None,
                    visibility: None,
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: Some(Utc::now()),
                },
                ProjectAwarenessEntry {
                    project_dir: "/tmp/other".to_string(),
                    bundle_root: "/tmp/other/.memd".to_string(),
                    project: Some("memd".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: None,
                    branch: None,
                    base_branch: None,
                    agent: Some("codex".to_string()),
                    session: Some("session-b".to_string()),
                    tab_id: None,
                    effective_agent: Some("codex@session-b".to_string()),
                    hive_system: None,
                    hive_role: None,
                    capabilities: Vec::new(),
                    hive_groups: Vec::new(),
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: None,
                    pid: None,
                    active_claims: 0,
                    workspace: None,
                    visibility: None,
                    topic_claim: None,
                    scope_claims: Vec::new(),
                    task_id: None,
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: Some(Utc::now()),
                },
            ],
        };

        let mut evidence = Vec::new();
        let candidates = build_gap_candidates(
            &output,
            &None,
            &None,
            None,
            None,
            None,
            Some(&awareness),
            None,
            &[],
            &mut evidence,
            None,
        );

        assert!(
            candidates
                .iter()
                .any(|value| value.id == "coordination:unhived_active_sessions")
        );
        assert!(
            candidates
                .iter()
                .any(|value| value.id == "coordination:awareness_collisions")
        );

        fs::remove_dir_all(&output).expect("cleanup temp output");
    }

    #[test]
    fn build_gap_candidates_does_not_surface_superseded_stale_remote_sessions() {
        let output = std::env::temp_dir().join(format!(
            "memd-gap-stale-sessions-test-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&output).expect("create temp output");

        let awareness = ProjectAwarenessResponse {
            root: "server:http://127.0.0.1:8787".to_string(),
            current_bundle: output.display().to_string(),
            collisions: Vec::new(),
            entries: vec![ProjectAwarenessEntry {
                project_dir: "remote".to_string(),
                bundle_root: "remote:http://127.0.0.1:8787:session-dead".to_string(),
                project: Some("memd".to_string()),
                namespace: Some("main".to_string()),
                repo_root: None,
                worktree_root: None,
                branch: None,
                base_branch: None,
                agent: Some("codex".to_string()),
                session: Some("session-dead".to_string()),
                tab_id: None,
                effective_agent: Some("codex@session-dead".to_string()),
                hive_system: None,
                hive_role: None,
                capabilities: vec!["memory".to_string()],
                hive_groups: Vec::new(),
                hive_group_goal: None,
                authority: Some("participant".to_string()),
                base_url: Some("http://127.0.0.1:8787".to_string()),
                presence: "stale".to_string(),
                host: None,
                pid: None,
                active_claims: 0,
                workspace: None,
                visibility: None,
                topic_claim: None,
                scope_claims: Vec::new(),
                task_id: None,
                focus: None,
                pressure: None,
                next_recovery: None,
                last_updated: Some(Utc::now()),
            }],
        };

        let mut evidence = Vec::new();
        let candidates = build_gap_candidates(
            &output,
            &None,
            &None,
            None,
            None,
            None,
            Some(&awareness),
            None,
            &[],
            &mut evidence,
            None,
        );

        assert!(
            candidates
                .iter()
                .all(|value| value.id != "coordination:stale_remote_sessions")
        );

        fs::remove_dir_all(&output).expect("cleanup temp output");
    }

    #[test]
    fn build_gap_candidates_surfaces_loop_manifest_drift() {
        let output =
            std::env::temp_dir().join(format!("memd-gap-docs-drift-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&output).expect("create temp output");

        let mut evidence = Vec::new();
        let candidates = build_gap_candidates(
            &output,
            &None,
            &None,
            None,
            None,
            None,
            None,
            Some(8),
            &[],
            &mut evidence,
            None,
        );

        assert!(
            candidates
                .iter()
                .any(|value| value.id == "docs:loop_manifest_drift")
        );

        fs::remove_dir_all(&output).expect("cleanup temp output");
    }

    #[test]
    fn collect_gap_repo_evidence_surfaces_repo_docs_and_runtime_signals() {
        let root =
            std::env::temp_dir().join(format!("memd-gap-repo-evidence-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(root.join("docs")).expect("create docs dir");
        fs::create_dir_all(root.join(".planning")).expect("create planning dir");
        fs::write(
            root.join("README.md"),
            "# memd\n\nThis repo uses memd for memory and setup.",
        )
        .expect("write readme");
        fs::write(
            root.join("ROADMAP.md"),
            "# roadmap\n\n## v6\nmemd gap research loop.",
        )
        .expect("write roadmap");
        fs::write(
            root.join("docs").join("setup.md"),
            "memd init and codex bootstrap",
        )
        .expect("write setup");
        fs::write(
            root.join(".planning").join("STATE.md"),
            "## Open Loops\n- gap research loop",
        )
        .expect("write state");

        let evidence = collect_gap_repo_evidence(&root);
        assert!(
            evidence.iter().any(|line| line.contains("git branch:")),
            "expected git branch evidence"
        );
        assert!(
            evidence.iter().any(|line| line.contains("README.md:")),
            "expected README evidence"
        );
        assert!(
            evidence.iter().any(|line| line.contains("ROADMAP.md:")),
            "expected ROADMAP evidence"
        );
        assert!(
            evidence.iter().any(|line| line.contains("docs/setup.md:")),
            "expected setup doc evidence"
        );
        assert!(
            evidence.iter().any(|line| line.contains("runtime wiring:")),
            "expected runtime wiring evidence"
        );

        fs::remove_dir_all(root).expect("cleanup temp repo");
    }

    #[test]
    fn prioritize_gap_candidates_orders_high_to_low_priority() {
        let candidates = vec![
            GapCandidate {
                id: "memory:a".to_string(),
                area: "memory".to_string(),
                priority: 40,
                severity: "low".to_string(),
                signal: "low".to_string(),
                evidence: Vec::new(),
                recommendation: "low-priority".to_string(),
            },
            GapCandidate {
                id: "coordination:b".to_string(),
                area: "coordination".to_string(),
                priority: 90,
                severity: "high".to_string(),
                signal: "high".to_string(),
                evidence: Vec::new(),
                recommendation: "high-priority".to_string(),
            },
            GapCandidate {
                id: "memory:c".to_string(),
                area: "memory".to_string(),
                priority: 70,
                severity: "medium".to_string(),
                signal: "medium".to_string(),
                evidence: Vec::new(),
                recommendation: "medium-priority".to_string(),
            },
        ];
        let sorted = prioritize_gap_candidates(candidates, 2);
        assert_eq!(sorted[0].priority, 90);
        assert_eq!(sorted[1].priority, 70);
    }

    #[test]
    fn evaluate_gap_changes_detects_count_and_status_shift() {
        let baseline = GapReport {
            bundle_root: ".memd".to_string(),
            project: None,
            namespace: None,
            agent: None,
            session: None,
            workspace: None,
            visibility: None,
            limit: 8,
            commits_checked: 0,
            eval_status: Some("usable".to_string()),
            eval_score: Some(70),
            eval_score_delta: Some(-5),
            candidate_count: 6,
            high_priority_count: 2,
            top_priorities: Vec::new(),
            candidates: Vec::new(),
            recommendations: Vec::new(),
            changes: Vec::new(),
            evidence: Vec::new(),
            generated_at: Utc::now(),
            previous_candidate_count: None,
        };

        let current = GapReport {
            bundle_root: ".memd".to_string(),
            project: None,
            namespace: None,
            agent: None,
            session: None,
            workspace: None,
            visibility: None,
            limit: 8,
            commits_checked: 2,
            eval_status: Some("weak".to_string()),
            eval_score: Some(66),
            eval_score_delta: Some(-10),
            candidate_count: 2,
            high_priority_count: 1,
            top_priorities: Vec::new(),
            candidates: Vec::new(),
            recommendations: Vec::new(),
            changes: Vec::new(),
            evidence: Vec::new(),
            generated_at: Utc::now(),
            previous_candidate_count: None,
        };

        let changes = evaluate_gap_changes(&current, Some(&baseline));
        assert!(
            changes
                .iter()
                .any(|value| value.contains("candidate_count 6 -> 2"))
        );
        assert!(
            changes
                .iter()
                .any(|value| value.contains("eval_score Some(70) -> Some(66)"))
        );
        assert!(
            changes
                .iter()
                .any(|value| value.contains("eval_status=weak"))
        );
    }

    fn test_gap_report(
        candidate_count: usize,
        high_priority_count: usize,
        eval_score: Option<u8>,
        top_priorities: Vec<String>,
    ) -> GapReport {
        GapReport {
            bundle_root: ".memd".to_string(),
            project: None,
            namespace: None,
            agent: None,
            session: None,
            workspace: None,
            visibility: None,
            limit: 8,
            commits_checked: 0,
            eval_status: None,
            eval_score,
            eval_score_delta: None,
            candidate_count,
            high_priority_count,
            top_priorities,
            candidates: Vec::new(),
            recommendations: Vec::new(),
            changes: Vec::new(),
            evidence: Vec::new(),
            generated_at: Utc::now(),
            previous_candidate_count: None,
        }
    }

    #[test]
    fn build_improvement_actions_dedupes_and_limits() {
        let mut gap = test_gap_report(3, 2, Some(61), vec!["memory:low_eval_score".to_string()]);
        gap.candidates.push(GapCandidate {
            id: "memory:low_eval_score".to_string(),
            area: "memory".to_string(),
            priority: 95,
            severity: "high".to_string(),
            signal: "low_eval_score".to_string(),
            evidence: vec!["evidence".to_string()],
            recommendation: "refresh eval".to_string(),
        });
        let coordination = CoordinationResponse {
            bundle_root: ".memd".to_string(),
            current_session: "codex".to_string(),
            inbox: HiveCoordinationInboxResponse {
                messages: Vec::new(),
                owned_tasks: Vec::new(),
                help_tasks: Vec::new(),
                review_tasks: Vec::new(),
            },
            active_hives: Vec::new(),
            recovery: CoordinationRecoverySummary {
                stale_hives: Vec::new(),
                reclaimable_claims: Vec::new(),
                stalled_tasks: Vec::new(),
                retireable_sessions: Vec::new(),
            },
            lane_fault: None,
            lane_receipts: Vec::new(),
            policy_conflicts: Vec::new(),
            suggestions: (0..10)
                .map(|index| CoordinationSuggestion {
                    action: "ack_message".to_string(),
                    priority: "medium".to_string(),
                    target_session: None,
                    task_id: None,
                    scope: None,
                    message_id: Some(format!("dup-{index}")),
                    reason: "dedupe check".to_string(),
                    stale_session: None,
                })
                .chain(std::iter::once(CoordinationSuggestion {
                    action: "ack_message".to_string(),
                    priority: "high".to_string(),
                    target_session: None,
                    task_id: None,
                    scope: None,
                    message_id: Some("dup-0".to_string()),
                    reason: "dedupe check".to_string(),
                    stale_session: None,
                }))
                .collect(),
            boundary_recommendations: Vec::new(),
            receipts: Vec::new(),
        };
        let actions = build_improvement_actions(&gap, Some(&coordination));
        assert!(
            actions.len() <= 8,
            "action list should be bounded by apply_improvement cap"
        );
        assert!(
            actions
                .iter()
                .filter(|value| value.action == "refresh_eval")
                .count()
                == 1,
            "low_eval_score only yields one refresh_eval action"
        );
        assert!(
            actions
                .iter()
                .filter(|value| value.message_id.as_deref() == Some("dup-0"))
                .count()
                <= 1,
            "duplicate suggestion keys should dedupe"
        );
    }

    #[test]
    fn build_improvement_actions_includes_retire_session_suggestions() {
        let gap = test_gap_report(
            2,
            1,
            Some(70),
            vec!["coordination:stale_remote_sessions".to_string()],
        );
        let coordination = CoordinationResponse {
            bundle_root: ".memd".to_string(),
            current_session: "codex".to_string(),
            inbox: HiveCoordinationInboxResponse {
                messages: Vec::new(),
                owned_tasks: Vec::new(),
                help_tasks: Vec::new(),
                review_tasks: Vec::new(),
            },
            active_hives: Vec::new(),
            recovery: CoordinationRecoverySummary {
                stale_hives: Vec::new(),
                reclaimable_claims: Vec::new(),
                stalled_tasks: Vec::new(),
                retireable_sessions: Vec::new(),
            },
            lane_fault: None,
            lane_receipts: Vec::new(),
            policy_conflicts: Vec::new(),
            suggestions: vec![CoordinationSuggestion {
                action: "retire_session".to_string(),
                priority: "medium".to_string(),
                target_session: None,
                task_id: None,
                scope: None,
                message_id: None,
                reason: "retire stale session".to_string(),
                stale_session: Some("session-stale".to_string()),
            }],
            boundary_recommendations: Vec::new(),
            receipts: Vec::new(),
        };

        let actions = build_improvement_actions(&gap, Some(&coordination));
        assert!(actions.iter().any(|value| {
            value.action == "retire_session"
                && value.target_session.as_deref() == Some("session-stale")
        }));
    }

    #[test]
    fn render_coordination_summary_surfaces_retireable_sessions() {
        let summary = render_coordination_summary(
            &CoordinationResponse {
                bundle_root: ".memd".to_string(),
                current_session: "codex".to_string(),
                inbox: HiveCoordinationInboxResponse {
                    messages: Vec::new(),
                    owned_tasks: Vec::new(),
                    help_tasks: Vec::new(),
                    review_tasks: Vec::new(),
                },
                active_hives: vec![ProjectAwarenessEntry {
                    project_dir: "remote".to_string(),
                    bundle_root: "remote:http://127.0.0.1:8787:active".to_string(),
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    repo_root: None,
                    worktree_root: Some("/tmp/peer".to_string()),
                    branch: Some("feature/peer".to_string()),
                    base_branch: Some("main".to_string()),
                    agent: Some("claude-code".to_string()),
                    session: Some("active".to_string()),
                    tab_id: None,
                    effective_agent: Some("claude-code@active".to_string()),
                    hive_system: Some("claude-code".to_string()),
                    hive_role: Some("agent".to_string()),
                    capabilities: vec!["memory".to_string()],
                    hive_groups: vec!["project:demo".to_string()],
                    hive_group_goal: None,
                    authority: Some("participant".to_string()),
                    base_url: Some("http://127.0.0.1:8787".to_string()),
                    presence: "active".to_string(),
                    host: Some("workstation".to_string()),
                    pid: Some(2),
                    active_claims: 1,
                    workspace: Some("shared".to_string()),
                    visibility: Some("workspace".to_string()),
                    topic_claim: Some("Refine parser overlap flow".to_string()),
                    scope_claims: vec![
                        "task:parser-refactor".to_string(),
                        "crates/memd-client/src/main.rs".to_string(),
                    ],
                    task_id: Some("parser-refactor".to_string()),
                    focus: None,
                    pressure: None,
                    next_recovery: None,
                    last_updated: Some(Utc::now()),
                }],
                recovery: CoordinationRecoverySummary {
                    stale_hives: Vec::new(),
                    reclaimable_claims: Vec::new(),
                    stalled_tasks: Vec::new(),
                    retireable_sessions: vec![ProjectAwarenessEntry {
                        project_dir: "remote".to_string(),
                        bundle_root: "remote:http://127.0.0.1:8787:stale".to_string(),
                        project: Some("demo".to_string()),
                        namespace: Some("main".to_string()),
                        repo_root: None,
                        worktree_root: None,
                        branch: None,
                        base_branch: None,
                        agent: Some("codex".to_string()),
                        session: Some("stale".to_string()),
                        tab_id: None,
                        effective_agent: Some("codex@stale".to_string()),
                        hive_system: Some("codex".to_string()),
                        hive_role: Some("agent".to_string()),
                        capabilities: vec!["memory".to_string()],
                        hive_groups: vec!["project:demo".to_string()],
                        hive_group_goal: None,
                        authority: Some("participant".to_string()),
                        base_url: Some("http://127.0.0.1:8787".to_string()),
                        presence: "stale".to_string(),
                        host: Some("workstation".to_string()),
                        pid: Some(1),
                        active_claims: 0,
                        workspace: Some("shared".to_string()),
                        visibility: Some("workspace".to_string()),
                        topic_claim: None,
                        scope_claims: Vec::new(),
                        task_id: None,
                        focus: None,
                        pressure: None,
                        next_recovery: None,
                        last_updated: Some(Utc::now()),
                    }],
                },
                lane_fault: Some(serde_json::json!({
                    "kind": "unsafe_same_branch",
                    "session": "claude-b",
                    "branch": "feature/hive-shared",
                    "worktree_root": "/tmp/worktree"
                })),
                lane_receipts: vec![HiveCoordinationReceiptRecord {
                    id: "lane-1".to_string(),
                    kind: "queen_deny".to_string(),
                    actor_session: "queen".to_string(),
                    actor_agent: Some("codex@queen".to_string()),
                    target_session: Some("claude-b".to_string()),
                    task_id: None,
                    scope: None,
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("shared".to_string()),
                    summary: "Queen denied overlap".to_string(),
                    created_at: Utc::now(),
                }],
                policy_conflicts: Vec::new(),
                suggestions: Vec::new(),
                boundary_recommendations: Vec::new(),
                receipts: Vec::new(),
            },
            Some("overview"),
        );

        assert!(summary.contains("retireable_sessions=1"));
        assert!(summary.contains("lane_fault=yes"));
        assert!(summary.contains("lane_receipts=1"));
        assert!(summary.contains("## Active Hive"));
        assert!(summary.contains("task=parser-refactor"));
        assert!(summary.contains("work=\"Refine parser overlap flow\""));
    }

    #[test]
    fn confirmed_hive_overlap_reason_detects_scope_and_topic_conflicts() {
        let target = ProjectAwarenessEntry {
            project_dir: "remote".to_string(),
            bundle_root: "remote:http://127.0.0.1:8787:active".to_string(),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: Some("/tmp/peer".to_string()),
            branch: Some("feature/peer".to_string()),
            base_branch: Some("main".to_string()),
            agent: Some("claude-code".to_string()),
            session: Some("active".to_string()),
            tab_id: None,
            effective_agent: Some("claude-code@active".to_string()),
            hive_system: Some("claude-code".to_string()),
            hive_role: Some("agent".to_string()),
            capabilities: vec!["memory".to_string()],
            hive_groups: vec!["project:demo".to_string()],
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            presence: "active".to_string(),
            host: Some("workstation".to_string()),
            pid: Some(2),
            active_claims: 1,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            topic_claim: Some("Refine parser overlap flow".to_string()),
            scope_claims: vec![
                "task:parser-refactor".to_string(),
                "crates/memd-client/src/main.rs".to_string(),
            ],
            task_id: Some("parser-refactor".to_string()),
            focus: None,
            pressure: None,
            next_recovery: None,
            last_updated: Some(Utc::now()),
        };

        let scope_conflict = confirmed_hive_overlap_reason(
            &target,
            Some("queen-refactor"),
            Some("Different task"),
            &["crates/memd-client/src/main.rs".to_string()],
        )
        .expect("scope conflict");
        assert!(scope_conflict.contains("already owns scope"));

        let topic_conflict = confirmed_hive_overlap_reason(
            &ProjectAwarenessEntry {
                scope_claims: Vec::new(),
                task_id: None,
                ..target
            },
            None,
            Some("Refine parser overlap flow"),
            &[],
        )
        .expect("topic conflict");
        assert!(topic_conflict.contains("already owns topic"));
    }

    #[test]
    fn confirmed_hive_overlap_reason_ignores_generic_project_scope() {
        let target = ProjectAwarenessEntry {
            project_dir: "remote".to_string(),
            bundle_root: "remote:http://127.0.0.1:8787:peer".to_string(),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            repo_root: None,
            worktree_root: Some("/tmp/peer".to_string()),
            branch: Some("feature/peer".to_string()),
            base_branch: Some("main".to_string()),
            agent: Some("codex".to_string()),
            session: Some("peer".to_string()),
            tab_id: None,
            effective_agent: Some("Peer@peer".to_string()),
            hive_system: Some("codex".to_string()),
            hive_role: Some("worker".to_string()),
            capabilities: vec!["coordination".to_string()],
            hive_groups: vec!["project:demo".to_string()],
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            base_url: Some("http://127.0.0.1:8787".to_string()),
            presence: "active".to_string(),
            host: Some("workstation".to_string()),
            pid: Some(3),
            active_claims: 1,
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            topic_claim: Some("Review parser handoff".to_string()),
            scope_claims: vec!["project".to_string()],
            task_id: Some("peer-review".to_string()),
            focus: None,
            pressure: None,
            next_recovery: None,
            last_updated: Some(Utc::now()),
        };

        assert!(
            confirmed_hive_overlap_reason(
                &target,
                Some("current-task"),
                Some("Different topic"),
                &["project".to_string()],
            )
            .is_none()
        );
    }

    #[tokio::test]
    async fn enrich_hive_heartbeat_with_runtime_intent_prefers_owned_task_state() {
        let state = MockRuntimeState::default();
        {
            let mut tasks = state.task_records.lock().expect("lock task records");
            tasks.push(HiveTaskRecord {
                task_id: "parser-refactor".to_string(),
                title: "Refine parser overlap flow".to_string(),
                description: None,
                status: "in_progress".to_string(),
                coordination_mode: "exclusive_write".to_string(),
                session: Some("codex-a".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@codex-a".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                claim_scopes: vec![
                    "task:parser-refactor".to_string(),
                    "crates/memd-client/src/main.rs".to_string(),
                ],
                help_requested: false,
                review_requested: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }
        let base_url = spawn_mock_runtime_server(state, false).await;
        let mut heartbeat =
            test_hive_heartbeat_state("codex-a", "codex", "tab-a", "live", Utc::now());
        heartbeat.base_url = Some(base_url);
        heartbeat.project = Some("demo".to_string());
        heartbeat.namespace = Some("main".to_string());
        heartbeat.workspace = Some("shared".to_string());
        heartbeat.session = Some("codex-a".to_string());
        heartbeat.topic_claim = Some("editing fallback".to_string());

        enrich_hive_heartbeat_with_runtime_intent(&mut heartbeat)
            .await
            .expect("enrich heartbeat");

        assert_eq!(heartbeat.task_id.as_deref(), Some("parser-refactor"));
        assert_eq!(
            heartbeat.topic_claim.as_deref(),
            Some("Refine parser overlap flow")
        );
        assert_eq!(heartbeat.display_name.as_deref(), Some("Codex a"));
        assert!(
            heartbeat
                .scope_claims
                .iter()
                .any(|scope| scope == "task:parser-refactor")
        );
    }

    #[tokio::test]
    async fn enrich_hive_heartbeat_with_runtime_intent_overrides_workspace_topic_placeholder() {
        let state = MockRuntimeState::default();
        {
            let mut tasks = state.task_records.lock().expect("lock task records");
            tasks.push(HiveTaskRecord {
                task_id: "remote-proof-refactor".to_string(),
                title: "Remote proof overlap flow".to_string(),
                description: None,
                status: "in_progress".to_string(),
                coordination_mode: "exclusive_write".to_string(),
                session: Some("codex-a".to_string()),
                agent: Some("codex".to_string()),
                effective_agent: Some("codex@codex-a".to_string()),
                project: Some("demo".to_string()),
                namespace: Some("main".to_string()),
                workspace: Some("shared".to_string()),
                claim_scopes: vec![
                    "task:remote-proof-refactor".to_string(),
                    "crates/memd-client/src/main.rs".to_string(),
                ],
                help_requested: false,
                review_requested: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }
        let base_url = spawn_mock_runtime_server(state, false).await;
        let mut heartbeat =
            test_hive_heartbeat_state("codex-a", "codex", "tab-a", "live", Utc::now());
        heartbeat.base_url = Some(base_url);
        heartbeat.project = Some("demo".to_string());
        heartbeat.namespace = Some("main".to_string());
        heartbeat.workspace = Some("shared".to_string());
        heartbeat.session = Some("codex-a".to_string());
        heartbeat.topic_claim = Some("ws=shared".to_string());

        enrich_hive_heartbeat_with_runtime_intent(&mut heartbeat)
            .await
            .expect("enrich heartbeat");

        assert_eq!(heartbeat.task_id.as_deref(), Some("remote-proof-refactor"));
        assert_eq!(
            heartbeat.topic_claim.as_deref(),
            Some("Remote proof overlap flow")
        );
        assert_eq!(heartbeat.display_name.as_deref(), Some("Codex a"));
    }

    #[test]
    fn render_session_summary_surfaces_rebind_and_retire_state() {
        let summary = render_session_summary(&SessionResponse {
            action: "rebind+retire".to_string(),
            bundle_root: ".memd".to_string(),
            bundle_session: Some("codex-fresh".to_string()),
            live_session: Some("codex-fresh".to_string()),
            rebased_from: Some("codex-stale".to_string()),
            tab_id: Some("tab-alpha".to_string()),
            reconciled: true,
            reconciled_retired_sessions: 2,
            retired_sessions: 1,
            retire_target: Some("codex-old".to_string()),
            heartbeat: Some(serde_json::json!({"status":"active"})),
        });

        assert!(summary.contains("action=rebind+retire"));
        assert!(summary.contains("bundle_session=codex-fresh"));
        assert!(summary.contains("live_session=codex-fresh"));
        assert!(summary.contains("rebased_from=codex-stale"));
        assert!(summary.contains("reconciled=yes"));
        assert!(summary.contains("reconciled_retired=2"));
        assert!(summary.contains("retired=1"));
        assert!(summary.contains("retire_target=codex-old"));
        assert!(summary.contains("heartbeat=published"));
    }

    #[test]
    fn render_tasks_summary_surfaces_task_taxonomy_counts() {
        let now = Utc::now();
        let summary = render_tasks_summary(&TasksResponse {
            bundle_root: ".memd".to_string(),
            current_session: Some("codex-a".to_string()),
            tasks: vec![
                HiveTaskRecord {
                    task_id: "t1".to_string(),
                    title: "exclusive open".to_string(),
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
                    created_at: now,
                    updated_at: now,
                },
                HiveTaskRecord {
                    task_id: "t2".to_string(),
                    title: "shared review".to_string(),
                    description: None,
                    status: "needs_review".to_string(),
                    coordination_mode: "shared_review".to_string(),
                    session: Some("codex-b".to_string()),
                    agent: Some("codex".to_string()),
                    effective_agent: Some("codex@codex-b".to_string()),
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("shared".to_string()),
                    claim_scopes: vec![],
                    help_requested: false,
                    review_requested: true,
                    created_at: now,
                    updated_at: now,
                },
                HiveTaskRecord {
                    task_id: "t3".to_string(),
                    title: "closed".to_string(),
                    description: None,
                    status: "done".to_string(),
                    coordination_mode: "shared_review".to_string(),
                    session: Some("codex-c".to_string()),
                    agent: Some("codex".to_string()),
                    effective_agent: Some("codex@codex-c".to_string()),
                    project: Some("demo".to_string()),
                    namespace: Some("main".to_string()),
                    workspace: Some("shared".to_string()),
                    claim_scopes: vec![],
                    help_requested: false,
                    review_requested: false,
                    created_at: now,
                    updated_at: now,
                },
            ],
        });

        assert!(summary.contains("count=3"));
        assert!(summary.contains("open=2"));
        assert!(summary.contains("help=1"));
        assert!(summary.contains("review=1"));
        assert!(summary.contains("exclusive=1"));
        assert!(summary.contains("shared=2"));
        assert!(summary.contains("active_sessions=2"));
        assert!(summary.contains("owned=1"));
    }

    #[test]
    fn render_capabilities_runtime_summary_surfaces_harness_breakdown() {
        let summary = render_capabilities_runtime_summary(&CapabilitiesResponse {
            bundle_root: ".memd".to_string(),
            generated_at: Utc::now(),
            discovered: 7,
            universal: 2,
            bridgeable: 3,
            harness_native: 2,
            bridge_actions: 4,
            wired_harnesses: 2,
            filters: serde_json::json!({}),
            harnesses: vec![
                CapabilityHarnessSummary {
                    harness: "codex".to_string(),
                    capabilities: 3,
                    installed: 2,
                    bridge_actions: 1,
                },
                CapabilityHarnessSummary {
                    harness: "claude-code".to_string(),
                    capabilities: 4,
                    installed: 4,
                    bridge_actions: 3,
                },
            ],
            records: Vec::new(),
        });

        assert!(summary.contains("discovered=7"));
        assert!(summary.contains("bridge_actions=4"));
        assert!(summary.contains("wired_harnesses=2"));
        assert!(summary.contains("shown=0"));
        assert!(summary.contains("codex:3/2/1"));
        assert!(summary.contains("claude-code:4/4/3"));
    }

    #[test]
    fn render_memory_surface_summary_surfaces_truth_and_tiers() {
        let summary = render_memory_surface_summary(&MemorySurfaceResponse {
            bundle_root: ".memd".to_string(),
            truth_summary: TruthSummary {
                retrieval_tier: RetrievalTier::Hot,
                truth: "current".to_string(),
                freshness: "fresh".to_string(),
                confidence: 0.97,
                action_hint: "keep current truth hot".to_string(),
                source_count: 2,
                contested_sources: 0,
                compact_records: 5,
                records: vec![TruthRecordSummary {
                    lane: "live_truth".to_string(),
                    truth: "current".to_string(),
                    freshness: "fresh".to_string(),
                    retrieval_tier: RetrievalTier::Hot,
                    confidence: 0.97,
                    provenance: "event_spine / compact".to_string(),
                    preview: "Current live truth head".to_string(),
                }],
            },
            context_records: 2,
            working_records: 3,
            inbox_items: 1,
            source_lanes: 2,
            rehydration_queue: 1,
            semantic_hits: 2,
            change_summary: 1,
            estimated_prompt_tokens: 180,
            refresh_recommended: false,
            contradiction_pressure: 2,
            superseded_pressure: 1,
            contradiction_reasons: vec!["live_truth:current:fresh".to_string()],
            superseded_reasons: vec!["refresh_recommended".to_string()],
            records: vec![TruthRecordSummary {
                lane: "live_truth".to_string(),
                truth: "current".to_string(),
                freshness: "fresh".to_string(),
                retrieval_tier: RetrievalTier::Hot,
                confidence: 0.97,
                provenance: "event_spine / compact".to_string(),
                preview: "Current live truth head".to_string(),
            }],
        });

        assert!(summary.contains("truth=current"));
        assert!(summary.contains("freshness=fresh"));
        assert!(summary.contains("retrieval=hot"));
        assert!(summary.contains("working:3"));
        assert!(summary.contains("sources:2"));
        assert!(summary.contains("tok=180"));
        assert!(summary.contains("contradictions=2"));
        assert!(summary.contains("superseded=1"));
        assert!(summary.contains("head=live_truth"));
    }

    #[test]
    fn render_claims_summary_surfaces_continuity_overlay() {
        let summary = render_claims_summary(&ClaimsResponse {
            bundle_root: ".memd".to_string(),
            bundle_session: Some("codex-stale".to_string()),
            live_session: Some("codex-fresh".to_string()),
            rebased_from: Some("codex-stale".to_string()),
            current_session: Some("codex-fresh".to_string()),
            current_tab_id: Some("tab-a".to_string()),
            claims: Vec::new(),
        });

        assert!(summary.contains("bundle_session=codex-stale"));
        assert!(summary.contains("live_session=codex-fresh"));
        assert!(summary.contains("rebased_from=codex-stale"));
    }

    #[test]
    fn render_messages_summary_surfaces_continuity_overlay() {
        let summary = render_messages_summary(&MessagesResponse {
            bundle_root: ".memd".to_string(),
            bundle_session: Some("codex-stale".to_string()),
            live_session: Some("codex-fresh".to_string()),
            rebased_from: Some("codex-stale".to_string()),
            current_session: Some("codex-fresh".to_string()),
            messages: Vec::new(),
        });

        assert!(summary.contains("bundle_session=codex-stale"));
        assert!(summary.contains("live_session=codex-fresh"));
        assert!(summary.contains("rebased_from=codex-stale"));
    }

    #[test]
    fn run_capabilities_command_filters_records() {
        let output =
            std::env::temp_dir().join(format!("memd-capabilities-filter-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&output).expect("create output");

        let response = run_capabilities_command(&CapabilitiesArgs {
            output: output.clone(),
            harness: Some("codex".to_string()),
            kind: None,
            portability: None,
            query: Some("memory".to_string()),
            limit: 8,
            summary: true,
            json: false,
        })
        .expect("capabilities response");

        assert!(
            response
                .records
                .iter()
                .all(|record| record.harness == "codex")
        );
        assert!(response.records.len() <= 8);

        fs::remove_dir_all(output).expect("cleanup output");
    }

    #[tokio::test]
    async fn run_tasks_command_supports_owned_view() {
        let dir =
            std::env::temp_dir().join(format!("memd-tasks-view-owned-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create temp dir");
        let state = MockRuntimeState::default();
        {
            let mut tasks = state.task_records.lock().expect("lock task records");
            tasks.push(HiveTaskRecord {
                task_id: "owned-1".to_string(),
                title: "owned".to_string(),
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
                help_requested: false,
                review_requested: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
            tasks.push(HiveTaskRecord {
                task_id: "shared-2".to_string(),
                title: "other".to_string(),
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
  "base_url": "{}",
  "route": "auto",
  "intent": "current_task"
}}
"#,
                base_url
            ),
        )
        .expect("write config");

        let response = run_tasks_command(
            &TasksArgs {
                output: dir.clone(),
                upsert: false,
                assign_to_session: None,
                target_session: None,
                task_id: None,
                title: None,
                description: None,
                status: None,
                mode: None,
                scope: Vec::new(),
                request_help: false,
                request_review: false,
                all: false,
                view: Some("owned".to_string()),
                summary: true,
                json: false,
            },
            SHARED_MEMD_BASE_URL,
        )
        .await
        .expect("tasks response");

        assert_eq!(response.tasks.len(), 1);
        assert_eq!(response.tasks[0].task_id, "owned-1");

        fs::remove_dir_all(dir).expect("cleanup temp dir");
    }

    #[tokio::test]
    async fn run_tasks_command_rejects_colliding_assignment_target_lane() {
        let root =
            std::env::temp_dir().join(format!("memd-tasks-collision-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("current");
        let target_project = root.join("target");
        let current_bundle = current_project.join(".memd");
        let target_bundle = target_project.join(".memd");
        fs::create_dir_all(&current_bundle).expect("create current bundle");
        fs::create_dir_all(&target_bundle).expect("create target bundle");
        fs::create_dir_all(current_project.join(".planning")).expect("create current planning");
        fs::create_dir_all(target_project.join(".planning")).expect("create target planning");
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state, false).await;

        write_test_bundle_config(&current_bundle, &base_url);
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
                base_url
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
                base_url: Some(base_url.clone()),
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

        let err = run_tasks_command(
            &TasksArgs {
                output: current_bundle.clone(),
                upsert: false,
                assign_to_session: Some("claude-b".to_string()),
                target_session: None,
                task_id: Some("task-1".to_string()),
                title: None,
                description: None,
                status: None,
                mode: None,
                scope: Vec::new(),
                request_help: false,
                request_review: false,
                all: false,
                view: None,
                summary: false,
                json: false,
            },
            SHARED_MEMD_BASE_URL,
        )
        .await
        .expect_err("colliding task assignment should fail");
        assert!(
            err.to_string()
                .contains("unsafe hive cowork target collision")
        );

        fs::remove_dir_all(root).expect("cleanup tasks collision dir");
    }

    #[tokio::test]
    async fn run_tasks_command_rejects_colliding_help_target_lane() {
        let root = std::env::temp_dir().join(format!(
            "memd-tasks-help-collision-{}",
            uuid::Uuid::new_v4()
        ));
        let current_project = root.join("current");
        let target_project = root.join("target");
        let current_bundle = current_project.join(".memd");
        let target_bundle = target_project.join(".memd");
        fs::create_dir_all(&current_bundle).expect("create current bundle");
        fs::create_dir_all(&target_bundle).expect("create target bundle");
        fs::create_dir_all(current_project.join(".planning")).expect("create current planning");
        fs::create_dir_all(target_project.join(".planning")).expect("create target planning");
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;

        write_test_bundle_config(&current_bundle, &base_url);
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
                base_url
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
                base_url: Some(base_url.clone()),
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

        let err = run_tasks_command(
            &TasksArgs {
                output: current_bundle.clone(),
                upsert: false,
                assign_to_session: None,
                target_session: Some("claude-b".to_string()),
                task_id: Some("task-1".to_string()),
                title: Some("need help".to_string()),
                description: None,
                status: None,
                mode: None,
                scope: vec!["src/main.rs".to_string()],
                request_help: true,
                request_review: false,
                all: false,
                view: None,
                summary: false,
                json: false,
            },
            SHARED_MEMD_BASE_URL,
        )
        .await
        .expect_err("colliding help request should fail");
        assert!(
            err.to_string()
                .contains("unsafe hive cowork target collision")
        );

        fs::remove_dir_all(root).expect("cleanup tasks help collision dir");
    }

    #[tokio::test]
    async fn run_tasks_command_rejects_colliding_review_target_lane() {
        let root = std::env::temp_dir().join(format!(
            "memd-tasks-review-collision-{}",
            uuid::Uuid::new_v4()
        ));
        let current_project = root.join("current");
        let target_project = root.join("target");
        let current_bundle = current_project.join(".memd");
        let target_bundle = target_project.join(".memd");
        fs::create_dir_all(&current_bundle).expect("create current bundle");
        fs::create_dir_all(&target_bundle).expect("create target bundle");
        fs::create_dir_all(current_project.join(".planning")).expect("create current planning");
        fs::create_dir_all(target_project.join(".planning")).expect("create target planning");
        let base_url = spawn_mock_runtime_server(MockRuntimeState::default(), false).await;

        write_test_bundle_config(&current_bundle, &base_url);
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
                base_url
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
                base_url: Some(base_url.clone()),
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

        let err = run_tasks_command(
            &TasksArgs {
                output: current_bundle.clone(),
                upsert: false,
                assign_to_session: None,
                target_session: Some("claude-b".to_string()),
                task_id: Some("task-1".to_string()),
                title: Some("need review".to_string()),
                description: None,
                status: None,
                mode: None,
                scope: vec!["src/main.rs".to_string()],
                request_help: false,
                request_review: true,
                all: false,
                view: None,
                summary: false,
                json: false,
            },
            SHARED_MEMD_BASE_URL,
        )
        .await
        .expect_err("colliding review request should fail");
        assert!(
            err.to_string()
                .contains("unsafe hive cowork target collision")
        );

        fs::remove_dir_all(root).expect("cleanup tasks review collision dir");
    }

    #[tokio::test]
    async fn hive_join_reroutes_colliding_worker_lane_into_new_worktree() {
        let root =
            std::env::temp_dir().join(format!("memd-hive-lane-reroute-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("current");
        let target_project = root.join("target");
        let current_bundle = current_project.join(".memd");
        let target_bundle = target_project.join(".memd");
        fs::create_dir_all(&current_bundle).expect("create current bundle");
        fs::create_dir_all(&target_bundle).expect("create target bundle");
        fs::create_dir_all(current_project.join(".planning")).expect("create current planning");
        fs::create_dir_all(target_project.join(".planning")).expect("create target planning");
        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state.clone(), false).await;

        write_test_bundle_config(&current_bundle, &base_url);
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
                base_url
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
                base_url: Some(base_url.clone()),
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
        let conflict = detect_bundle_lane_collision(&current_bundle, Some("codex-a"))
            .await
            .expect("detect lane collision");
        assert!(conflict.is_some(), "expected lane collision before join");

        let response = run_hive_join_command(&HiveJoinArgs {
            output: current_bundle.clone(),
            base_url: base_url.clone(),
            all_active: false,
            all_local: false,
            publish_heartbeat: false,
            summary: false,
        })
        .await
        .expect("reroute join");

        let response = match response {
            HiveJoinResponse::Single(response) => response,
            other => panic!("expected single response, got {other:?}"),
        };
        let rerouted_output = PathBuf::from(&response.output);
        assert!(response.lane_rerouted);
        assert!(response.lane_created);
        assert!(response.lane_surface.is_some());
        assert_ne!(rerouted_output, current_bundle);
        assert!(rerouted_output.join("config.json").exists());

        let rerouted_project = rerouted_output
            .parent()
            .expect("rerouted bundle parent")
            .to_path_buf();
        let rerouted_branch =
            git_stdout(&rerouted_project, &["branch", "--show-current"]).expect("rerouted branch");
        assert_ne!(rerouted_branch, "feature/hive-shared");
        assert_ne!(
            detect_git_worktree_root(&rerouted_project).expect("rerouted worktree root"),
            detect_git_worktree_root(&current_project).expect("current worktree root")
        );

        let rerouted_runtime = read_bundle_runtime_config_raw(&rerouted_output)
            .expect("read rerouted runtime")
            .expect("rerouted runtime config");
        assert_ne!(rerouted_runtime.session.as_deref(), Some("codex-a"));
        let status = read_bundle_status(&rerouted_output, SHARED_MEMD_BASE_URL)
            .await
            .expect("read rerouted status");
        let lane = status.get("lane_surface").expect("lane surface present");
        assert_eq!(
            lane.get("action").and_then(JsonValue::as_str),
            Some("auto_reroute")
        );
        assert_eq!(
            lane.get("conflict_session").and_then(JsonValue::as_str),
            Some("claude-b")
        );
        let receipts = state.receipts.lock().expect("lock receipts");
        assert!(
            receipts
                .iter()
                .any(|receipt| receipt.kind == "lane_reroute")
        );

        fs::remove_dir_all(root).expect("cleanup hive reroute dir");
    }

    #[test]
    fn cli_accepts_reload_as_refresh_alias() {
        let cli = Cli::try_parse_from(["memd", "reload", "--output", ".memd", "--summary"])
            .expect("reload alias should parse");

        match cli.command {
            Commands::Refresh(args) => {
                assert_eq!(args.output, PathBuf::from(".memd"));
                assert!(args.summary);
            }
            other => panic!("expected refresh command, got {other:?}"),
        }
    }

    #[test]
    fn improvement_progress_tracks_candidate_score_and_priority_change() {
        let baseline = test_gap_report(10, 3, Some(82), vec!["a".to_string(), "b".to_string()]);
        let fewer_candidates =
            test_gap_report(8, 3, Some(82), vec!["a".to_string(), "b".to_string()]);
        let better_score = test_gap_report(10, 3, Some(84), vec!["a".to_string(), "b".to_string()]);
        let changed_priorities =
            test_gap_report(10, 3, Some(82), vec!["x".to_string(), "a".to_string()]);
        let no_change = test_gap_report(10, 3, Some(82), vec!["a".to_string(), "b".to_string()]);

        assert!(improvement_progress(&baseline, &fewer_candidates));
        assert!(improvement_progress(&baseline, &better_score));
        assert!(improvement_progress(&baseline, &changed_priorities));
        assert!(!improvement_progress(&baseline, &no_change));
    }

    #[tokio::test]
    async fn run_scenario_command_writes_artifacts_and_scores_with_mocked_backend() {
        let dir =
            std::env::temp_dir().join(format!("memd-scenario-command-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create scenario temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "agent": "codex",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write config");

        let base_url = spawn_mock_memory_server().await;
        let report = run_scenario_command(
            &ScenarioArgs {
                output: dir.clone(),
                scenario: Some("bundle_health".to_string()),
                write: false,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("run scenario command");

        assert_eq!(report.scenario, "bundle_health");
        assert!(report.passed_checks >= 1);
        assert_eq!(report.failed_checks, 0);
        assert!(report.score >= 28);
        assert!(report.max_score >= report.score);
        assert!(
            report
                .checks
                .iter()
                .any(|check| check.name == "runtime_config" && check.status == "pass")
        );
        assert!(!report.checks.is_empty());

        write_scenario_artifacts(&dir, &report).expect("write scenario artifacts");
        let scenario_dir = dir.join("scenarios");
        let latest_json = scenario_dir.join("latest.json");
        let latest_markdown = scenario_dir.join("latest.md");
        assert!(latest_json.exists());
        assert!(latest_markdown.exists());

        let latest = fs::read_to_string(&latest_json).expect("read latest.json");
        let parsed: ScenarioReport =
            serde_json::from_str(&latest).expect("parse latest scenario json");
        assert_eq!(parsed.scenario, "bundle_health");
        let markdown = fs::read_to_string(&latest_markdown).expect("read latest.md");
        assert!(markdown.contains("# memd scenario report: bundle_health"));
        let entries = fs::read_dir(&scenario_dir)
            .expect("read scenario dir")
            .collect::<Result<Vec<_>, _>>()
            .expect("scenario dir entries");
        assert!(entries.iter().any(|entry| {
            entry
                .path()
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.ends_with(".json") && name != "latest.json")
        }));

        fs::remove_dir_all(dir).expect("cleanup scenario temp bundle");
    }

    #[tokio::test]
    async fn run_scenario_command_supports_named_v6_workflows() {
        let dir = std::env::temp_dir().join(format!(
            "memd-scenario-command-workflows-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).expect("create scenario temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "agent": "codex",
  "session": "session-alpha",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write config");

        let base_url = spawn_mock_memory_server().await;
        let scenarios = [
            "bundle_health",
            "resume_after_pause",
            "handoff",
            "workspace_retrieval",
            "stale_session_recovery",
            "coworking",
        ];
        for scenario in scenarios {
            let report = run_scenario_command(
                &ScenarioArgs {
                    output: dir.clone(),
                    scenario: Some(scenario.to_string()),
                    write: false,
                    summary: false,
                },
                &base_url,
            )
            .await
            .expect("run scenario command");

            assert_eq!(report.scenario, scenario);
            assert_eq!(report.failed_checks, 0);
            assert!(!report.checks.is_empty());
            assert!(report.max_score > 0);
        }

        fs::remove_dir_all(dir).expect("cleanup scenario temp bundle");
    }

    #[tokio::test]
    async fn run_scenario_command_rejects_unknown_scenario() {
        let dir = std::env::temp_dir().join(format!(
            "memd-scenario-command-unknown-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).expect("create scenario temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "agent": "codex",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write config");

        let base_url = spawn_mock_memory_server().await;
        let result = run_scenario_command(
            &ScenarioArgs {
                output: dir.clone(),
                scenario: Some("not_a_real_scenario".to_string()),
                write: false,
                summary: false,
            },
            &base_url,
        )
        .await;

        assert!(result.is_err());
        assert!(
            result
                .err()
                .expect("scenario should be rejected")
                .to_string()
                .contains("supported")
        );

        fs::remove_dir_all(dir).expect("cleanup scenario temp bundle");
    }

    #[tokio::test]
    async fn run_composite_command_combines_saved_eval_and_scenario_reports() {
        let dir =
            std::env::temp_dir().join(format!("memd-composite-command-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).expect("create composite temp bundle");
        fs::write(
            dir.join("config.json"),
            r#"{
  "project": "demo",
  "agent": "codex",
  "route": "auto",
  "intent": "general"
}
"#,
        )
        .expect("write config");

        let eval = high_scoring_eval(&dir);
        write_bundle_eval_artifacts(&dir, &eval).expect("write eval artifacts");

        let scenario = high_scoring_scenario(&dir);
        write_scenario_artifacts(&dir, &scenario).expect("write scenario artifacts");

        let base_url = spawn_mock_memory_server().await;
        let composite = run_composite_command(
            &CompositeArgs {
                output: dir.clone(),
                scenario: Some("bundle_health".to_string()),
                write: false,
                summary: false,
            },
            &base_url,
        )
        .await
        .expect("run composite");

        assert_eq!(composite.max_score, 100);
        assert!(composite.score > 0);
        assert!(
            composite
                .dimensions
                .iter()
                .any(|dimension| dimension.name == "correctness")
        );
        assert!(
            composite
                .gates
                .iter()
                .any(|gate| gate.name == "hard_correctness")
        );
        assert!(composite.gates.iter().any(|gate| gate.name == "acceptance"));

        write_composite_artifacts(&dir, &composite).expect("write composite artifacts");
        let composite_dir = dir.join("composite");
        assert!(composite_dir.join("latest.json").exists());
        assert!(composite_dir.join("latest.md").exists());

        fs::remove_dir_all(dir).expect("cleanup composite temp bundle");
    }

    #[test]
    fn cli_parses_benchmark_command() {
        let cli = Cli::try_parse_from(["memd", "benchmark", "--output", ".memd", "--summary"])
            .expect("benchmark command should parse");

        match cli.command {
            Commands::Benchmark(args) => {
                assert_eq!(args.output, PathBuf::from(".memd"));
                assert!(args.summary);
                assert!(args.subcommand.is_none());
            }
            other => panic!("expected benchmark command, got {other:?}"),
        }
    }

    #[test]
    fn cli_parses_public_longmemeval_benchmark_command() {
        let cli = Cli::try_parse_from([
            "memd",
            "benchmark",
            "public",
            "--mode",
            "raw",
            "--limit",
            "20",
            "--output",
            ".memd",
            "longmemeval",
        ])
        .expect("public benchmark command should parse");

        match cli.command {
            Commands::Benchmark(args) => match args.subcommand {
                Some(BenchmarkSubcommand::Public(public_args)) => {
                    assert_eq!(public_args.dataset, "longmemeval");
                    assert_eq!(public_args.mode.as_deref(), Some("raw"));
                    assert_eq!(public_args.limit, Some(20));
                    assert_eq!(public_args.out, PathBuf::from(".memd"));
                    assert!(!public_args.write);
                    assert!(!public_args.json);
                }
                other => panic!("expected public benchmark subcommand, got {other:?}"),
            },
            other => panic!("expected benchmark command, got {other:?}"),
        }
    }

    #[test]
    fn cli_parses_public_longmemeval_sidecar_backend() {
        let cli = Cli::try_parse_from([
            "memd",
            "benchmark",
            "public",
            "--mode",
            "hybrid",
            "--retrieval-backend",
            "sidecar",
            "--rag-url",
            "http://127.0.0.1:9981",
            "--output",
            ".memd",
            "longmemeval",
        ])
        .expect("public benchmark sidecar command should parse");

        match cli.command {
            Commands::Benchmark(args) => match args.subcommand {
                Some(BenchmarkSubcommand::Public(public_args)) => {
                    assert_eq!(public_args.dataset, "longmemeval");
                    assert_eq!(public_args.mode.as_deref(), Some("hybrid"));
                    assert_eq!(public_args.retrieval_backend.as_deref(), Some("sidecar"));
                    assert_eq!(
                        public_args.rag_url.as_deref(),
                        Some("http://127.0.0.1:9981")
                    );
                }
                other => panic!("expected public benchmark subcommand, got {other:?}"),
            },
            other => panic!("expected benchmark command, got {other:?}"),
        }
    }

    #[test]
    fn public_benchmark_paths_default_under_memd_benchmarks() {
        let output = PathBuf::from(".memd");
        assert_eq!(
            public_benchmark_dataset_cache_dir(&output),
            PathBuf::from(".memd/benchmarks/datasets")
        );
        assert_eq!(
            public_benchmark_dataset_entry_dir(&output, "longmemeval"),
            PathBuf::from(".memd/benchmarks/datasets/longmemeval")
        );
        assert_eq!(
            public_benchmark_dataset_cache_path(
                &output,
                "longmemeval",
                "longmemeval_s_cleaned.json"
            ),
            PathBuf::from(".memd/benchmarks/datasets/longmemeval/longmemeval_s_cleaned.json")
        );
        assert_eq!(
            public_benchmark_runs_dir(&output),
            PathBuf::from(".memd/benchmarks/public")
        );
        assert_eq!(
            public_benchmark_run_artifacts_dir(&output, "longmemeval"),
            PathBuf::from(".memd/benchmarks/public/longmemeval/latest")
        );
    }

    #[test]
    fn supported_public_benchmark_ids_lists_all_mem_palace_targets() {
        assert_eq!(
            supported_public_benchmark_ids(),
            &["longmemeval", "locomo", "convomem", "membench"]
        );
    }

    #[test]
    fn public_benchmark_source_catalog_pins_longmemeval_download() {
        let source = public_benchmark_dataset_source("longmemeval").expect("catalog entry");
        assert_eq!(source.benchmark_id, "longmemeval");
        assert_eq!(source.access_mode, "auto-download");
        assert!(source
            .source_url
            .is_some_and(|url| url.ends_with("longmemeval_s_cleaned.json")));
        assert_eq!(
            source.expected_checksum,
            Some("sha256:d6f21ea9d60a0d56f34a05b609c79c88a451d2ae03597821ea3d5a9678c3a442")
        );
        assert_eq!(source.split, "cleaned-small");
    }

    #[test]
    fn public_benchmark_source_catalog_pins_locomo_download() {
        let source = public_benchmark_dataset_source("locomo").expect("catalog entry");
        assert_eq!(source.benchmark_id, "locomo");
        assert_eq!(source.access_mode, "auto-download");
        assert!(source
            .source_url
            .is_some_and(|url| url.contains("/snap-research/locomo/")));
        assert_eq!(
            source.expected_checksum,
            Some("sha256:79fa87e90f04081343b8c8debecb80a9a6842b76a7aa537dc9fdf651ea698ff4")
        );
        assert_eq!(source.split, "locomo10");
    }

    #[test]
    fn public_benchmark_source_catalog_pins_convomem_download() {
        let source = public_benchmark_dataset_source("convomem").expect("catalog entry");
        assert_eq!(source.benchmark_id, "convomem");
        assert_eq!(source.access_mode, "auto-download");
        assert!(source
            .source_url
            .is_some_and(|url| url.contains("huggingface.co/datasets/Salesforce/ConvoMem/tree/main")));
        assert_eq!(source.default_filename, "convomem-evidence-sample.json");
        assert_eq!(source.expected_checksum, None);
        assert_eq!(source.split, "evidence-sample");
    }

    #[test]
    fn public_benchmark_source_catalog_pins_membench_download() {
        let source = public_benchmark_dataset_source("membench").expect("catalog entry");
        assert_eq!(source.benchmark_id, "membench");
        assert_eq!(source.access_mode, "auto-download");
        assert!(source
            .source_url
            .is_some_and(|url| url.contains("/import-myself/Membench/")));
        assert_eq!(source.default_filename, "membench-firstagent.json");
        assert_eq!(source.expected_checksum, None);
        assert_eq!(source.split, "FirstAgent");
    }

    #[tokio::test]
    async fn resolve_public_benchmark_dataset_rejects_unknown_sources() {
        let dir = std::env::temp_dir().join(format!(
            "memd-public-benchmark-manual-required-{}",
            uuid::Uuid::new_v4()
        ));
        let output = dir.join(".memd");
        fs::create_dir_all(&output).expect("create output dir");

        let error = resolve_public_benchmark_dataset(&PublicBenchmarkArgs {
            dataset: "unknown-benchmark".to_string(),
            mode: Some("raw".to_string()),
            retrieval_backend: None,
            rag_url: None,
            top_k: Some(5),
            limit: Some(2),
            dataset_root: None,
            reranker: None,
            write: false,
            json: false,
            out: output.clone(),
        })
        .await
        .expect_err("unknown benchmark should be rejected");
        assert!(error
            .to_string()
            .contains("no public benchmark dataset source is registered"));

        fs::remove_dir_all(dir).expect("cleanup manual-required dir");
    }

    #[test]
    fn write_public_benchmark_dataset_cache_metadata_roundtrips_json() {
        let dir = std::env::temp_dir().join(format!(
            "memd-public-benchmark-cache-metadata-{}",
            uuid::Uuid::new_v4()
        ));
        let output = dir.join(".memd");
        fs::create_dir_all(&output).expect("create output dir");

        let metadata = PublicBenchmarkDatasetCacheMetadata {
            benchmark_id: "longmemeval".to_string(),
            source_url: "https://huggingface.co/datasets/xiaowu0162/longmemeval-cleaned/resolve/main/longmemeval_s_cleaned.json".to_string(),
            local_path: output
                .join("benchmarks")
                .join("datasets")
                .join("longmemeval")
                .join("longmemeval_s_cleaned.json")
                .display()
                .to_string(),
            checksum: "sha256:abc123".to_string(),
            expected_checksum: Some("sha256:abc123".to_string()),
            verification_status: "verified".to_string(),
            fetched_at: Utc::now(),
            bytes: 123,
        };

        let path = write_public_benchmark_dataset_cache_metadata(&output, &metadata)
            .expect("write cache metadata");
        assert_eq!(
            path,
            public_benchmark_dataset_cache_metadata_path(&output, "longmemeval")
        );
        let contents = fs::read_to_string(&path).expect("read cache metadata");
        let parsed: PublicBenchmarkDatasetCacheMetadata =
            serde_json::from_str(&contents).expect("parse cache metadata");
        assert_eq!(parsed.benchmark_id, "longmemeval");
        assert_eq!(parsed.verification_status, "verified");
        assert_eq!(parsed.bytes, 123);

        fs::remove_dir_all(dir).expect("cleanup cache metadata dir");
    }

    #[test]
    fn load_public_benchmark_dataset_normalizes_longmemeval_array_format() {
        let dir = std::env::temp_dir().join(format!(
            "memd-public-benchmark-longmemeval-normalize-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).expect("create normalize dir");
        let path = dir.join("longmemeval_s_cleaned.json");
        fs::write(
            &path,
            serde_json::to_string_pretty(&json!([
                {
                    "question_id": "q1",
                    "question_type": "temporal-reasoning",
                    "question": "What happened first?",
                    "answer": "the gps failed",
                    "question_date": "2023/04/10",
                    "haystack_dates": ["2023/04/10"],
                    "haystack_session_ids": ["s1"],
                    "answer_session_ids": ["s1"],
                    "haystack_sessions": [[
                        {"role": "user", "content": "The GPS failed after service.", "has_answer": true},
                        {"role": "assistant", "content": "That sounds annoying.", "has_answer": false}
                    ]]
                }
            ]))
            .expect("serialize synthetic longmemeval"),
        )
        .expect("write synthetic longmemeval");

        let dataset =
            load_public_benchmark_dataset("longmemeval", &path).expect("normalize dataset");
        assert_eq!(dataset.benchmark_id, "longmemeval");
        assert_eq!(dataset.version, "upstream");
        assert_eq!(dataset.items.len(), 1);
        assert_eq!(dataset.items[0].item_id, "q1");
        assert_eq!(dataset.items[0].gold_answer, "the gps failed");
        assert_eq!(dataset.items[0].claim_class, "raw");
        assert_eq!(
            dataset.items[0]
                .metadata
                .get("answer_session_ids")
                .and_then(JsonValue::as_array)
                .map(Vec::len),
            Some(1)
        );
        assert!(dataset.items[0]
            .metadata
            .get("haystack_text")
            .and_then(JsonValue::as_str)
            .is_some_and(|text| text.contains("GPS failed after service")));

        fs::remove_dir_all(dir).expect("cleanup normalize dir");
    }

    #[test]
    fn load_public_benchmark_dataset_normalizes_locomo_array_format() {
        let dir = std::env::temp_dir().join(format!(
            "memd-public-benchmark-locomo-normalize-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).expect("create normalize dir");
        let path = dir.join("locomo10.json");
        fs::write(
            &path,
            serde_json::to_string_pretty(&json!([
                {
                    "sample_id": "sample-001",
                    "conversation": {
                        "speaker_a": "Caroline",
                        "speaker_b": "Mel",
                        "session_1_date_time": "2023-05-07",
                        "session_1": [
                            {"speaker": "Caroline", "dia_id": "D1:1", "text": "I went to the LGBTQ support group on May 7."},
                            {"speaker": "Mel", "dia_id": "D1:2", "text": "That sounds meaningful."}
                        ]
                    },
                    "session_summary": {
                        "session_1_summary": "Caroline discussed attending a support group."
                    },
                    "qa": [
                        {
                            "question": "When did Caroline go to the LGBTQ support group?",
                            "answer": "7 May 2023",
                            "evidence": ["D1:1"],
                            "category": 2
                        }
                    ]
                }
            ]))
            .expect("serialize synthetic locomo"),
        )
        .expect("write synthetic locomo");

        let dataset = load_public_benchmark_dataset("locomo", &path).expect("normalize dataset");
        assert_eq!(dataset.benchmark_id, "locomo");
        assert_eq!(dataset.version, "upstream");
        assert_eq!(dataset.items.len(), 1);
        assert_eq!(dataset.items[0].item_id, "sample-001::0");
        assert_eq!(
            dataset.items[0].query,
            "When did Caroline go to the LGBTQ support group?"
        );
        assert_eq!(dataset.items[0].gold_answer, "7 May 2023");
        assert_eq!(dataset.items[0].claim_class, "raw");
        assert_eq!(
            dataset.items[0]
                .metadata
                .get("category_name")
                .and_then(JsonValue::as_str),
            Some("Temporal")
        );
        assert!(dataset.items[0]
            .metadata
            .get("conversation_text")
            .and_then(JsonValue::as_str)
            .is_some_and(|text| text.contains("Caroline: I went to the LGBTQ support group")));

        fs::remove_dir_all(dir).expect("cleanup normalize dir");
    }

    #[test]
    fn load_public_benchmark_dataset_normalizes_locomo_adversarial_answer_fallback() {
        let dir = std::env::temp_dir().join(format!(
            "memd-public-benchmark-locomo-adversarial-normalize-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).expect("create normalize dir");
        let path = dir.join("locomo10.json");
        fs::write(
            &path,
            serde_json::to_string_pretty(&json!([
                {
                    "sample_id": "sample-adv-001",
                    "conversation": {
                        "speaker_a": "Caroline",
                        "speaker_b": "Mel",
                        "session_1_date_time": "2023-05-07",
                        "session_1": [
                            {"speaker": "Caroline", "dia_id": "D1:3", "text": "After the race I realized self-care is important."}
                        ]
                    },
                    "session_summary": {
                        "session_1_summary": "Caroline reflected on self-care."
                    },
                    "qa": [
                        {
                            "question": "What did Caroline realize after her charity race?",
                            "adversarial_answer": "self-care is important",
                            "evidence": ["D1:3"],
                            "category": 5
                        }
                    ]
                }
            ]))
            .expect("serialize synthetic locomo adversarial"),
        )
        .expect("write synthetic locomo adversarial");

        let dataset = load_public_benchmark_dataset("locomo", &path).expect("normalize dataset");
        assert_eq!(dataset.items.len(), 1);
        assert_eq!(dataset.items[0].gold_answer, "self-care is important");
        assert_eq!(
            dataset.items[0]
                .metadata
                .get("category_name")
                .and_then(JsonValue::as_str),
            Some("Adversarial")
        );

        fs::remove_dir_all(dir).expect("cleanup normalize dir");
    }

    #[test]
    fn load_public_benchmark_dataset_normalizes_membench_object_format() {
        let dir = std::env::temp_dir().join(format!(
            "memd-public-benchmark-membench-normalize-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&dir).expect("create normalize dir");
        let path = dir.join("membench-firstagent.json");
        fs::write(
            &path,
            serde_json::to_string_pretty(&json!({
                "movie": [
                    {
                        "tid": 0,
                        "message_list": [
                            [
                                {
                                    "sid": 0,
                                    "user_message": "I like courtroom dramas.",
                                    "assistant_message": "Courtroom dramas can be intense.",
                                    "time": "'2024-10-01 08:00' Tuesday",
                                    "place": "Boston, MA"
                                }
                            ]
                        ],
                        "QA": {
                            "qid": 0,
                            "question": "According to the movies I mentioned, what kind of movies might I prefer to watch?",
                            "answer": "Drama",
                            "target_step_id": [[0, 0]],
                            "choices": {
                                "A": "Musical",
                                "B": "Drama",
                                "C": "Horror",
                                "D": "Children"
                            },
                            "ground_truth": "B",
                            "time": "'2024-10-01 08:13' Tuesday"
                        }
                    }
                ]
            }))
            .expect("serialize synthetic membench"),
        )
        .expect("write synthetic membench");

        let dataset =
            load_public_benchmark_dataset("membench", &path).expect("normalize dataset");
        assert_eq!(dataset.benchmark_id, "membench");
        assert_eq!(dataset.version, "upstream");
        assert_eq!(dataset.items.len(), 1);
        assert_eq!(dataset.items[0].item_id, "movie::0::0");
        assert_eq!(dataset.items[0].gold_answer, "Drama");
        assert_eq!(
            dataset.items[0]
                .metadata
                .get("topic")
                .and_then(JsonValue::as_str),
            Some("movie")
        );
        assert_eq!(
            dataset.items[0]
                .metadata
                .get("target_step_id")
                .and_then(JsonValue::as_array)
                .map(Vec::len),
            Some(1)
        );
        assert!(dataset.items[0]
            .metadata
            .get("conversation_text")
            .and_then(JsonValue::as_str)
            .is_some_and(|text| text.contains("user: I like courtroom dramas.")));

        fs::remove_dir_all(dir).expect("cleanup normalize dir");
    }

    #[test]
    fn normalize_convomem_evidence_items_builds_fixture_rows() {
        let fixture = normalize_convomem_evidence_items(&[
            json!({
                "question": "What color do I use for hot leads in my personal spreadsheet?",
                "answer": "Green",
                "message_evidences": [{"speaker": "User", "text": "I use green for hot leads in my personal spreadsheet."}],
                "conversations": [{
                    "id": "conv-1",
                    "containsEvidence": true,
                    "model_name": "gpt-4o",
                    "messages": [
                        {"speaker": "User", "text": "I use green for hot leads in my personal spreadsheet."},
                        {"speaker": "Assistant", "text": "That sounds organized."}
                    ]
                }],
                "category": "user_evidence",
                "scenario_description": "Telemarketer",
                "personId": "person-1"
            })
        ])
        .expect("normalize convomem sample");

        assert_eq!(fixture.benchmark_id, "convomem");
        assert_eq!(fixture.items.len(), 1);
        assert_eq!(fixture.items[0].gold_answer, "Green");
        assert_eq!(
            fixture.items[0]
                .metadata
                .get("category")
                .and_then(JsonValue::as_str),
            Some("user_evidence")
        );
        assert!(fixture.items[0]
            .metadata
            .get("conversation_text")
            .and_then(JsonValue::as_str)
            .is_some_and(|text| text.contains("User: I use green for hot leads")));
    }

    #[test]
    fn build_longmemeval_run_report_tracks_session_and_turn_metrics() {
        let dataset = PublicBenchmarkDatasetFixture {
            benchmark_id: "longmemeval".to_string(),
            benchmark_name: "LongMemEval".to_string(),
            version: "upstream".to_string(),
            split: "cleaned-small".to_string(),
            description: "synthetic longmemeval".to_string(),
            items: vec![PublicBenchmarkDatasetFixtureItem {
                item_id: "q1".to_string(),
                question_id: "q1".to_string(),
                query: "what happened first".to_string(),
                claim_class: "raw".to_string(),
                gold_answer: "gps failed".to_string(),
                metadata: json!({
                    "question_type": "temporal-reasoning",
                    "question_date": "2023/04/10",
                    "haystack_dates": ["2023/04/10", "2023/04/09"],
                    "haystack_session_ids": ["s1", "s2"],
                    "answer_session_ids": ["s1"],
                    "haystack_sessions": [
                        [
                            {"role": "user", "content": "The GPS failed after service."},
                            {"role": "assistant", "content": "That sounds annoying."}
                        ],
                        [
                            {"role": "user", "content": "I bought floor mats."},
                            {"role": "assistant", "content": "Nice purchase."}
                        ]
                    ],
                    "haystack_text": "user: The GPS failed after service.\nassistant: That sounds annoying."
                }),
            }],
        };

        let report = build_longmemeval_run_report(
            &dataset,
            5,
            "raw",
            None,
            &PublicBenchmarkRetrievalConfig {
                longmemeval_backend: LongMemEvalRetrievalBackend::Lexical,
                sidecar_base_url: None,
            },
        )
        .expect("longmemeval report");
        assert_eq!(
            report.metrics.get("session_recall_any@5").copied(),
            Some(1.0)
        );
        assert_eq!(report.metrics.get("turn_recall_any@5").copied(), Some(1.0));
        assert_eq!(
            report.metrics.get("session_ndcg_any@10").copied(),
            Some(1.0)
        );
        assert_eq!(report.item_count, 1);
        assert_eq!(
            report.items[0]
                .correctness
                .as_ref()
                .and_then(|value: &JsonValue| value.get("session_metrics"))
                .and_then(|value: &JsonValue| value.get("recall_any@5"))
                .and_then(JsonValue::as_f64),
            Some(1.0)
        );
    }

    #[test]
    fn build_longmemeval_run_report_supports_sidecar_backend_ordering() {
        let base_url = spawn_blocking_mock_sidecar_server();
        let dataset = PublicBenchmarkDatasetFixture {
            benchmark_id: "longmemeval".to_string(),
            benchmark_name: "LongMemEval".to_string(),
            version: "upstream".to_string(),
            split: "cleaned-small".to_string(),
            description: "synthetic longmemeval".to_string(),
            items: vec![PublicBenchmarkDatasetFixtureItem {
                item_id: "q-sidecar".to_string(),
                question_id: "q-sidecar".to_string(),
                query: "which session should receive the handoff".to_string(),
                claim_class: "raw".to_string(),
                gold_answer: "target".to_string(),
                metadata: json!({
                    "question_type": "handoff",
                    "question_date": "2026/04/09",
                    "haystack_dates": ["2026/04/09", "2026/04/08"],
                    "haystack_session_ids": ["current", "target"],
                    "answer_session_ids": ["target"],
                    "haystack_sessions": [
                        [
                            {"role": "user", "content": "keep this in the current worker lane"},
                            {"role": "assistant", "content": "staying local"}
                        ],
                        [
                            {"role": "user", "content": "send the handoff packet to the target session"},
                            {"role": "assistant", "content": "route everything to target"}
                        ]
                    ]
                }),
            }],
        };

        let report = build_longmemeval_run_report(
            &dataset,
            5,
            "raw",
            None,
            &PublicBenchmarkRetrievalConfig {
                longmemeval_backend: LongMemEvalRetrievalBackend::Sidecar,
                sidecar_base_url: Some(base_url),
            },
        )
        .expect("sidecar longmemeval report");

        assert_eq!(report.metrics.get("session_recall_any@1").copied(), Some(1.0));
        assert_eq!(
            report.items[0]
                .ranked_items
                .first()
                .and_then(|item| item.get("item_id"))
                .and_then(JsonValue::as_str),
            Some("target")
        );
        assert!(
            report.items[0]
                .retrieval_scores
                .first()
                .copied()
                .unwrap_or_default()
                > 0.9
        );
    }

    fn public_benchmark_fixture_path(dataset: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../fixtures/public-benchmarks/{dataset}-mini.json"
        ))
    }

    #[test]
    fn write_public_benchmark_manifest_roundtrips_json() {
        let dir = std::env::temp_dir().join(format!(
            "memd-public-benchmark-manifest-{}",
            uuid::Uuid::new_v4()
        ));
        let output = dir.join(".memd");
        fs::create_dir_all(&output).expect("create output dir");

        let manifest = PublicBenchmarkManifest {
            benchmark_id: "longmemeval".to_string(),
            benchmark_version: "mini".to_string(),
            dataset_name: "LongMemEval Mini".to_string(),
            dataset_source_url: "https://example.invalid/longmemeval-mini.json".to_string(),
            dataset_local_path: output
                .join("benchmarks")
                .join("datasets")
                .join("longmemeval-mini.json")
                .display()
                .to_string(),
            dataset_checksum: "sha256:abc123".to_string(),
            dataset_split: "validation".to_string(),
            git_sha: Some("deadbeef".to_string()),
            dirty_worktree: false,
            run_timestamp: Utc::now(),
            mode: "raw".to_string(),
            top_k: 5,
            reranker_id: None,
            reranker_provider: None,
            limit: Some(2),
            runtime_settings: json!({
                "cache": true,
                "seed": 42
            }),
            hardware_summary: "cpu-only".to_string(),
            duration_ms: 11,
            token_usage: Some(json!({"prompt": 120, "completion": 0})),
            cost_estimate_usd: Some(0.0),
        };

        let manifest_path =
            write_public_benchmark_manifest(&output, &manifest).expect("write public manifest");
        assert_eq!(
            manifest_path,
            public_benchmark_manifest_json_path(&output, "longmemeval")
        );

        let contents = fs::read_to_string(&manifest_path).expect("read manifest");
        let parsed: PublicBenchmarkManifest = serde_json::from_str(&contents).expect("parse");
        assert_eq!(parsed.benchmark_id, "longmemeval");
        assert_eq!(parsed.mode, "raw");
        assert_eq!(parsed.top_k, 5);
        assert_eq!(parsed.limit, Some(2));
        assert_eq!(parsed.dataset_split, "validation");
        assert!(!parsed.dirty_worktree);

        fs::remove_dir_all(dir).expect("cleanup public benchmark manifest dir");
    }

    #[test]
    fn write_public_benchmark_run_report_roundtrips_json() {
        let dir = std::env::temp_dir().join(format!(
            "memd-public-benchmark-run-{}",
            uuid::Uuid::new_v4()
        ));
        let output = dir.join(".memd");
        fs::create_dir_all(&output).expect("create output dir");

        let report = PublicBenchmarkRunReport {
            manifest: PublicBenchmarkManifest {
                benchmark_id: "longmemeval".to_string(),
                benchmark_version: "mini".to_string(),
                dataset_name: "LongMemEval Mini".to_string(),
                dataset_source_url: "https://example.invalid/longmemeval-mini.json".to_string(),
                dataset_local_path: output
                    .join("benchmarks")
                    .join("datasets")
                    .join("longmemeval-mini.json")
                    .display()
                    .to_string(),
                dataset_checksum: "sha256:abc123".to_string(),
                dataset_split: "validation".to_string(),
                git_sha: Some("deadbeef".to_string()),
                dirty_worktree: false,
                run_timestamp: Utc::now(),
                mode: "hybrid".to_string(),
                top_k: 8,
                reranker_id: Some("reranker-v1".to_string()),
                reranker_provider: Some("memd".to_string()),
                limit: Some(3),
                runtime_settings: json!({
                    "cache": true,
                    "seed": 13
                }),
                hardware_summary: "cpu-only".to_string(),
                duration_ms: 19,
                token_usage: Some(json!({"prompt": 220, "completion": 32})),
                cost_estimate_usd: Some(0.01),
            },
            metrics: BTreeMap::from([("accuracy".to_string(), 0.8), ("recall".to_string(), 1.0)]),
            item_count: 2,
            failures: vec![json!({"item_id": "longmemeval-mini-002", "reason": "miss"})],
            items: vec![PublicBenchmarkItemResult {
                item_id: "longmemeval-mini-001".to_string(),
                question_id: "longmemeval-mini-001".to_string(),
                claim_class: "raw".to_string(),
                question: Some("What should be resumed next?".to_string()),
                question_type: Some("continuity".to_string()),
                ranked_items: vec![json!({"rank": 1, "text": "resume next step"})],
                retrieved_items: vec![json!({"rank": 1, "text": "resume next step"})],
                retrieval_scores: vec![0.93],
                hit: true,
                answer: Some("resume next step".to_string()),
                observed_answer: Some("resume next step".to_string()),
                correctness: Some(json!({"score": 1.0})),
                latency_ms: 14,
                token_usage: Some(json!({"prompt": 12, "completion": 4})),
                cost_estimate_usd: Some(0.0),
            }],
        };

        let report_path =
            write_public_benchmark_run_report(&output, &report).expect("write public report");
        assert_eq!(
            report_path,
            public_benchmark_report_md_path(&output, "longmemeval")
        );

        let contents = fs::read_to_string(&report_path).expect("read report");
        assert!(contents.contains("# memd public benchmark"));
        assert!(contents.contains("longmemeval"));
        assert!(contents.contains("| LongMemEval Mini | mini | hybrid |"));
        assert!(contents.contains("## Latest Run Detail: LongMemEval Mini"));

        let jsonl_path = public_benchmark_results_jsonl_path(&output, "longmemeval");
        let first_row = fs::read_to_string(&jsonl_path)
            .expect("read public benchmark jsonl")
            .lines()
            .next()
            .expect("jsonl first row")
            .to_string();
        let first_row: JsonValue = serde_json::from_str(&first_row).expect("parse public jsonl");
        assert_eq!(
            first_row.get("question").and_then(JsonValue::as_str),
            Some("What should be resumed next?")
        );
        assert_eq!(
            first_row.get("question_type").and_then(JsonValue::as_str),
            Some("continuity")
        );
        assert_eq!(
            first_row.get("answer").and_then(JsonValue::as_str),
            Some("resume next step")
        );
        assert_eq!(
            first_row
                .get("observed_answer")
                .and_then(JsonValue::as_str),
            Some("resume next step")
        );
        assert_eq!(
            first_row
                .get("ranked_items")
                .and_then(JsonValue::as_array)
                .map(Vec::len),
            Some(1)
        );

        fs::remove_dir_all(dir).expect("cleanup public benchmark run dir");
    }

    #[test]
    fn write_public_benchmark_run_artifacts_writes_manifest_and_report() {
        let dir = std::env::temp_dir().join(format!(
            "memd-public-benchmark-artifacts-{}",
            uuid::Uuid::new_v4()
        ));
        let output = dir.join(".memd");
        fs::create_dir_all(&output).expect("create output dir");

        let report = PublicBenchmarkRunReport {
            manifest: PublicBenchmarkManifest {
                benchmark_id: "longmemeval".to_string(),
                benchmark_version: "mini".to_string(),
                dataset_name: "LongMemEval Mini".to_string(),
                dataset_source_url: "https://example.invalid/longmemeval-mini.json".to_string(),
                dataset_local_path: output
                    .join("benchmarks")
                    .join("datasets")
                    .join("longmemeval-mini.json")
                    .display()
                    .to_string(),
                dataset_checksum: "sha256:abc123".to_string(),
                dataset_split: "validation".to_string(),
                git_sha: Some("deadbeef".to_string()),
                dirty_worktree: false,
                run_timestamp: Utc::now(),
                mode: "hybrid".to_string(),
                top_k: 8,
                reranker_id: Some("reranker-v1".to_string()),
                reranker_provider: Some("memd".to_string()),
                limit: Some(3),
                runtime_settings: json!({
                    "cache": true,
                    "seed": 13
                }),
                hardware_summary: "cpu-only".to_string(),
                duration_ms: 19,
                token_usage: Some(json!({"prompt": 220, "completion": 32})),
                cost_estimate_usd: Some(0.01),
            },
            metrics: BTreeMap::from([("accuracy".to_string(), 0.8)]),
            item_count: 1,
            failures: Vec::new(),
            items: vec![PublicBenchmarkItemResult {
                item_id: "longmemeval-mini-001".to_string(),
                question_id: "lm-mini-001".to_string(),
                claim_class: "raw".to_string(),
                question: Some("What should be resumed next?".to_string()),
                question_type: Some("continuity".to_string()),
                ranked_items: vec![json!({"rank": 1, "text": "resume next step"})],
                retrieved_items: vec![json!({"rank": 1, "text": "resume next step"})],
                retrieval_scores: vec![0.93],
                hit: true,
                answer: Some("resume next step".to_string()),
                observed_answer: Some("resume next step".to_string()),
                correctness: Some(json!({"score": 1.0})),
                latency_ms: 14,
                token_usage: Some(json!({"prompt": 12, "completion": 4})),
                cost_estimate_usd: Some(0.0),
            }],
        };

        let receipt =
            write_public_benchmark_run_artifacts(&output, &report).expect("write artifacts");
        assert_eq!(
            receipt.run_dir,
            public_benchmark_run_artifacts_dir(&output, "longmemeval")
        );
        assert_eq!(
            receipt.manifest_path,
            public_benchmark_manifest_json_path(&output, "longmemeval")
        );
        assert_eq!(
            receipt.results_path,
            public_benchmark_results_json_path(&output, "longmemeval")
        );
        assert_eq!(
            receipt.results_jsonl_path,
            public_benchmark_results_jsonl_path(&output, "longmemeval")
        );
        assert_eq!(
            receipt.report_path,
            public_benchmark_report_md_path(&output, "longmemeval")
        );
        assert!(receipt.manifest_path.exists());
        assert!(receipt.results_path.exists());
        assert!(receipt.results_jsonl_path.exists());
        assert!(receipt.report_path.exists());

        let manifest: PublicBenchmarkManifest = serde_json::from_str(
            &fs::read_to_string(&receipt.manifest_path).expect("read manifest"),
        )
        .expect("parse manifest");
        let results: PublicBenchmarkRunReport =
            serde_json::from_str(&fs::read_to_string(&receipt.results_path).expect("read results"))
                .expect("parse results");
        assert_eq!(manifest.reranker_id.as_deref(), Some("reranker-v1"));
        assert_eq!(manifest.reranker_provider.as_deref(), Some("memd"));
        assert_eq!(
            manifest.token_usage,
            Some(json!({"prompt": 220, "completion": 32}))
        );
        assert_eq!(results.manifest.mode, "hybrid");
        assert_eq!(results.items.len(), 1);
        assert_eq!(results.items[0].claim_class, "raw");

        fs::remove_dir_all(dir).expect("cleanup public benchmark artifacts dir");
    }

    #[tokio::test]
    async fn run_public_longmemeval_command_writes_artifacts_and_docs() {
        let dir = std::env::temp_dir().join(format!(
            "memd-public-benchmark-command-{}",
            uuid::Uuid::new_v4()
        ));
        let output = dir.join(".memd");
        let docs_root = dir.join("repo");
        fs::create_dir_all(&output).expect("create output dir");
        fs::create_dir_all(docs_root.join(".git")).expect("create git dir");

        let fixture = public_benchmark_fixture_path("longmemeval");
        let report = run_public_benchmark_command(&PublicBenchmarkArgs {
            dataset: "longmemeval".to_string(),
            mode: Some("raw".to_string()),
            retrieval_backend: None,
            rag_url: None,
            top_k: Some(5),
            limit: Some(2),
            dataset_root: Some(fixture),
            reranker: None,
            write: false,
            json: false,
            out: output.clone(),
        })
        .await
        .expect("run public benchmark");

        assert_eq!(report.manifest.benchmark_id, "longmemeval");
        assert_eq!(report.manifest.mode, "raw");
        assert_eq!(report.manifest.dataset_name, "LongMemEval");
        assert_eq!(report.item_count, 2);
        assert!(report.metrics.get("accuracy").copied().unwrap_or(0.0) > 0.0);

        let receipt =
            write_public_benchmark_run_artifacts(&output, &report).expect("write artifacts");
        assert!(receipt.manifest_path.exists());
        assert!(receipt.results_path.exists());
        assert!(receipt.results_jsonl_path.exists());
        assert!(receipt.report_path.exists());

        write_public_benchmark_docs(&docs_root, &output, &report)
            .expect("write public benchmark docs");
        let docs = fs::read_to_string(public_benchmark_docs_path(&docs_root))
            .expect("read public benchmark docs");
        assert!(docs.contains("# memd public benchmark"));
        assert!(docs.contains("LongMemEval"));
        assert!(docs.contains("results"));
        assert!(docs.contains("## Target Inventory"));
        assert!(docs.contains("- longmemeval: implemented"));
        assert!(docs.contains("- locomo: implemented"));
        assert!(docs.contains("- convomem: implemented"));
        assert!(docs.contains("- membench: implemented"));
        let leaderboard = fs::read_to_string(public_benchmark_leaderboard_docs_path(&docs_root))
            .expect("read public leaderboard docs");
        assert!(leaderboard.contains("# memd public leaderboard"));
        assert!(leaderboard.contains("fixture-backed"));
        assert!(leaderboard.contains("dataset-grade / retrieval-local"));
        assert!(
            leaderboard
                .contains("declared parity targets: longmemeval, locomo, convomem, membench")
        );

        fs::remove_dir_all(dir).expect("cleanup public benchmark command dir");
    }

    #[tokio::test]
    async fn run_public_longmemeval_hybrid_command_sets_metadata() {
        let dir = std::env::temp_dir().join(format!(
            "memd-public-benchmark-hybrid-{}",
            uuid::Uuid::new_v4()
        ));
        let output = dir.join(".memd");
        fs::create_dir_all(&output).expect("create output dir");

        let fixture = public_benchmark_fixture_path("longmemeval");
        let report = run_public_benchmark_command(&PublicBenchmarkArgs {
            dataset: "longmemeval".to_string(),
            mode: Some("hybrid".to_string()),
            retrieval_backend: None,
            rag_url: None,
            top_k: Some(5),
            limit: Some(1),
            dataset_root: Some(fixture),
            reranker: Some("test-reranker".to_string()),
            write: false,
            json: false,
            out: output.clone(),
        })
        .await
        .expect("run hybrid public benchmark");

        assert_eq!(report.manifest.mode, "hybrid");
        assert_eq!(
            report.manifest.reranker_id.as_deref(),
            Some("test-reranker")
        );
        assert_eq!(
            report.manifest.reranker_provider.as_deref(),
            Some("declared")
        );
        assert_eq!(
            report.manifest.token_usage,
            Some(json!({
                "prompt_tokens": 0,
                "completion_tokens": 0,
                "reranker_tokens": 0,
            }))
        );
        assert_eq!(report.manifest.cost_estimate_usd, Some(0.0));
        assert_eq!(report.items.len(), 1);
        assert_eq!(report.items[0].claim_class, "raw");

        fs::remove_dir_all(dir).expect("cleanup public benchmark hybrid dir");
    }

    #[tokio::test]
    async fn render_public_leaderboard_marks_fixture_backed_partial_parity() {
        let dir =
            std::env::temp_dir().join(format!("memd-public-leaderboard-{}", uuid::Uuid::new_v4()));
        let output = dir.join(".memd");
        fs::create_dir_all(&output).expect("create output dir");

        let fixture = public_benchmark_fixture_path("longmemeval");
        let report = run_public_benchmark_command(&PublicBenchmarkArgs {
            dataset: "longmemeval".to_string(),
            mode: Some("raw".to_string()),
            retrieval_backend: None,
            rag_url: None,
            top_k: Some(5),
            limit: Some(2),
            dataset_root: Some(fixture),
            reranker: None,
            write: false,
            json: false,
            out: output.clone(),
        })
        .await
        .expect("run public benchmark");

        let leaderboard_report =
            build_public_benchmark_leaderboard_report(std::slice::from_ref(&report));
        let markdown = render_public_leaderboard(&leaderboard_report);
        assert!(markdown.contains("# memd public leaderboard"));
        assert!(markdown.contains("fixture-backed"));
        assert!(markdown.contains("dataset-grade / retrieval-local"));
        assert!(markdown.contains("not a full MemPalace parity claim"));
        assert!(markdown.contains("run mode is benchmark execution mode"));
        assert!(
            markdown.contains("implemented mini adapters: longmemeval, locomo, convomem, membench")
        );
        assert!(markdown.contains("| LongMemEval | upstream | raw | raw |"));
        assert!(
            markdown.contains("declared parity targets: longmemeval, locomo, convomem, membench")
        );

        fs::remove_dir_all(dir).expect("cleanup public leaderboard dir");
    }

    #[tokio::test]
    async fn write_public_benchmark_docs_aggregates_all_latest_runs() {
        let dir = std::env::temp_dir().join(format!(
            "memd-public-benchmark-suite-docs-{}",
            uuid::Uuid::new_v4()
        ));
        let output = dir.join(".memd");
        let docs_root = dir.join("repo");
        fs::create_dir_all(&output).expect("create output dir");
        fs::create_dir_all(docs_root.join(".git")).expect("create git dir");

        for dataset in ["longmemeval", "locomo", "convomem", "membench"] {
            let report = run_public_benchmark_command(&PublicBenchmarkArgs {
                dataset: dataset.to_string(),
                mode: Some("raw".to_string()),
                retrieval_backend: None,
                rag_url: None,
                top_k: Some(5),
                limit: Some(2),
                dataset_root: Some(public_benchmark_fixture_path(dataset)),
                reranker: None,
                write: false,
                json: false,
                out: output.clone(),
            })
            .await
            .expect("run public benchmark");
            write_public_benchmark_run_artifacts(&output, &report).expect("write public artifacts");
        }

        let latest_report = run_public_benchmark_command(&PublicBenchmarkArgs {
            dataset: "locomo".to_string(),
            mode: Some("hybrid".to_string()),
            retrieval_backend: None,
            rag_url: None,
            top_k: Some(5),
            limit: Some(2),
            dataset_root: Some(public_benchmark_fixture_path("locomo")),
            reranker: Some("test-reranker".to_string()),
            write: false,
            json: false,
            out: output.clone(),
        })
        .await
        .expect("run latest public benchmark");
        write_public_benchmark_run_artifacts(&output, &latest_report)
            .expect("write latest public artifacts");
        write_public_benchmark_docs(&docs_root, &output, &latest_report)
            .expect("write public benchmark docs");

        let docs = fs::read_to_string(public_benchmark_docs_path(&docs_root))
            .expect("read public benchmark docs");
        assert!(docs.contains("# memd public benchmark suite"));
        assert!(docs.contains("| LongMemEval |"));
        assert!(docs.contains("| LoCoMo |"));
        assert!(docs.contains("| ConvoMem |"));
        assert!(docs.contains("| MemBench |"));
        assert!(docs.contains("## Latest Run Detail: LoCoMo"));

        let leaderboard = fs::read_to_string(public_benchmark_leaderboard_docs_path(&docs_root))
            .expect("read public leaderboard docs");
        assert!(leaderboard.contains("| LongMemEval |"));
        assert!(leaderboard.contains("| LoCoMo |"));
        assert!(leaderboard.contains("| ConvoMem |"));
        assert!(leaderboard.contains("| MemBench |"));

        fs::remove_dir_all(dir).expect("cleanup public benchmark suite docs dir");
    }

    #[tokio::test]
    async fn run_public_locomo_command_writes_artifacts() {
        let dir = std::env::temp_dir().join(format!(
            "memd-public-benchmark-locomo-{}",
            uuid::Uuid::new_v4()
        ));
        let output = dir.join(".memd");
        fs::create_dir_all(&output).expect("create output dir");

        let fixture = public_benchmark_fixture_path("locomo");
        let report = run_public_benchmark_command(&PublicBenchmarkArgs {
            dataset: "locomo".to_string(),
            mode: Some("raw".to_string()),
            retrieval_backend: None,
            rag_url: None,
            top_k: Some(5),
            limit: Some(2),
            dataset_root: Some(fixture),
            reranker: None,
            write: false,
            json: false,
            out: output.clone(),
        })
        .await
        .expect("run locomo public benchmark");

        assert_eq!(report.manifest.benchmark_id, "locomo");
        assert_eq!(report.manifest.dataset_name, "LoCoMo");
        assert_eq!(report.item_count, 2);

        let receipt =
            write_public_benchmark_run_artifacts(&output, &report).expect("write locomo artifacts");
        assert!(receipt.manifest_path.exists());
        assert!(receipt.results_path.exists());
        assert!(receipt.results_jsonl_path.exists());
        assert!(receipt.report_path.exists());

        fs::remove_dir_all(dir).expect("cleanup public benchmark locomo dir");
    }

    #[tokio::test]
    async fn run_public_convomem_command_writes_artifacts() {
        let dir = std::env::temp_dir().join(format!(
            "memd-public-benchmark-convomem-{}",
            uuid::Uuid::new_v4()
        ));
        let output = dir.join(".memd");
        fs::create_dir_all(&output).expect("create output dir");

        let fixture = public_benchmark_fixture_path("convomem");
        let report = run_public_benchmark_command(&PublicBenchmarkArgs {
            dataset: "convomem".to_string(),
            mode: Some("raw".to_string()),
            retrieval_backend: None,
            rag_url: None,
            top_k: Some(5),
            limit: Some(2),
            dataset_root: Some(fixture),
            reranker: None,
            write: false,
            json: false,
            out: output.clone(),
        })
        .await
        .expect("run convomem public benchmark");

        assert_eq!(report.manifest.benchmark_id, "convomem");
        assert_eq!(report.manifest.dataset_name, "ConvoMem");
        assert_eq!(report.item_count, 2);

        let receipt = write_public_benchmark_run_artifacts(&output, &report)
            .expect("write convomem artifacts");
        assert!(receipt.manifest_path.exists());
        assert!(receipt.results_path.exists());
        assert!(receipt.results_jsonl_path.exists());
        assert!(receipt.report_path.exists());

        fs::remove_dir_all(dir).expect("cleanup public benchmark convomem dir");
    }

    #[tokio::test]
    async fn run_public_membench_command_writes_artifacts() {
        let dir = std::env::temp_dir().join(format!(
            "memd-public-benchmark-membench-{}",
            uuid::Uuid::new_v4()
        ));
        let output = dir.join(".memd");
        fs::create_dir_all(&output).expect("create output dir");

        let fixture = public_benchmark_fixture_path("membench");
        let report = run_public_benchmark_command(&PublicBenchmarkArgs {
            dataset: "membench".to_string(),
            mode: Some("raw".to_string()),
            retrieval_backend: None,
            rag_url: None,
            top_k: Some(5),
            limit: Some(2),
            dataset_root: Some(fixture),
            reranker: None,
            write: false,
            json: false,
            out: output.clone(),
        })
        .await
        .expect("run membench public benchmark");

        assert_eq!(report.manifest.benchmark_id, "membench");
        assert_eq!(report.manifest.dataset_name, "MemBench");
        assert_eq!(report.item_count, 2);

        let receipt = write_public_benchmark_run_artifacts(&output, &report)
            .expect("write membench artifacts");
        assert!(receipt.manifest_path.exists());
        assert!(receipt.results_path.exists());
        assert!(receipt.results_jsonl_path.exists());
        assert!(receipt.report_path.exists());

        fs::remove_dir_all(dir).expect("cleanup public benchmark membench dir");
    }

    #[test]
    fn cli_parses_verify_feature_command() {
        let cli = Cli::try_parse_from([
            "memd",
            "verify",
            "feature",
            "feature.bundle.resume",
            "--output",
            ".memd",
            "--summary",
        ])
        .expect("verify feature command should parse");

        match cli.command {
            Commands::Verify(args) => match args.command {
                VerifyCommand::Feature(feature_args) => {
                    assert_eq!(feature_args.feature_id, "feature.bundle.resume");
                    assert_eq!(feature_args.output, PathBuf::from(".memd"));
                    assert!(feature_args.summary);
                }
                other => panic!("expected verify feature command, got {other:?}"),
            },
            other => panic!("expected verify command, got {other:?}"),
        }
    }

    #[test]
    fn cli_parses_verify_sweep_lane() {
        let cli = Cli::try_parse_from([
            "memd", "verify", "sweep", "--lane", "nightly", "--output", ".memd",
        ])
        .expect("verify sweep command should parse");

        match cli.command {
            Commands::Verify(args) => match args.command {
                VerifyCommand::Sweep(sweep_args) => {
                    assert_eq!(sweep_args.output, PathBuf::from(".memd"));
                    assert_eq!(sweep_args.lane, "nightly");
                }
                other => panic!("expected verify sweep command, got {other:?}"),
            },
            other => panic!("expected verify command, got {other:?}"),
        }
    }

    #[test]
    fn run_verify_list_command_reports_registry_verifiers_and_fixtures() {
        let dir = std::env::temp_dir().join(format!("memd-verify-list-{}", uuid::Uuid::new_v4()));
        let repo_root = dir.join("repo");
        let output = repo_root.join(".memd");
        fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
        fs::create_dir_all(&output).expect("create output dir");
        write_test_benchmark_registry(&repo_root);

        let report = run_verify_list_command(&VerifyListArgs {
            output: output.clone(),
            lane: Some("nightly".to_string()),
            summary: false,
        })
        .expect("run verify list");

        assert!(report.registry_loaded);
        assert!(report.registry_verifiers > 0);
        assert!(report.registry_fixtures > 0);
        assert_eq!(report.lane.as_deref(), Some("nightly"));
        let summary = render_verify_summary(&report);
        assert!(summary.contains("verifiers="));
        assert!(summary.contains("fixtures="));

        fs::remove_dir_all(dir).expect("cleanup verify list dir");
    }

    #[test]
    fn materialize_continuity_fixture_creates_temp_bundle() {
        let fixture = test_continuity_fixture_record();
        let env = materialize_fixture(&fixture, None).expect("materialize fixture");
        assert!(env.bundle_root.join("config.json").exists());
        assert_eq!(env._fixture_id, "fixture.continuity_bundle");
    }

    #[test]
    fn materialize_fixture_writes_seed_files_into_bundle() {
        let mut fixture = test_continuity_fixture_record();
        fixture.seed_files = vec!["state/checkpoint.txt".to_string()];

        let env = materialize_fixture(&fixture, None).expect("materialize fixture");

        let seeded = env.bundle_root.join("state/checkpoint.txt");
        assert!(seeded.exists());
        let contents = fs::read_to_string(seeded).expect("read seeded file");
        assert!(contents.contains("resume next step"));
    }

    #[tokio::test]
    async fn materialize_hive_fixture_creates_named_session_bundles() {
        let base_url = spawn_mock_runtime_server(MockRuntimeState::default(), false).await;
        let fixture = FixtureRecord {
            id: "fixture.hive-two-session-bundle.test".to_string(),
            kind: "bundle_fixture".to_string(),
            description: "hive".to_string(),
            seed_files: Vec::new(),
            seed_config: json!({
                "project": "demo",
                "namespace": "main",
                "agent": "codex",
                "session": "sender",
                "workspace": "shared",
                "base_url": base_url
            }),
            seed_memories: Vec::new(),
            seed_events: Vec::new(),
            seed_sessions: vec!["sender".to_string(), "target".to_string()],
            seed_claims: Vec::new(),
            seed_vault: None,
            backend_mode: "normal".to_string(),
            isolation: "fresh_temp_dir".to_string(),
            cleanup_policy: "destroy".to_string(),
        };

        let env = materialize_fixture(&fixture, None).expect("materialize hive fixture");

        let sender_bundle = env
            .fixture_vars
            .get("sender_bundle")
            .map(PathBuf::from)
            .expect("sender bundle path");
        let target_bundle = env
            .fixture_vars
            .get("target_bundle")
            .map(PathBuf::from)
            .expect("target bundle path");
        assert!(sender_bundle.join("config.json").exists());
        assert!(target_bundle.join("config.json").exists());
        assert_eq!(env.bundle_root, sender_bundle);
        let sender_config = read_bundle_runtime_config(&env.bundle_root)
            .expect("read sender runtime config")
            .expect("sender runtime config present");
        let target_config = read_bundle_runtime_config(&target_bundle)
            .expect("read target runtime config")
            .expect("target runtime config present");
        assert_eq!(sender_config.agent.as_deref(), Some("Sender"));
        assert_eq!(target_config.agent.as_deref(), Some("Target"));
        assert_eq!(
            env.fixture_vars
                .get("target_session")
                .is_some_and(|value| value.starts_with("target-")),
            true
        );
    }

    #[tokio::test]
    async fn propagate_hive_metadata_does_not_overwrite_sibling_worker_identity() {
        let _env_lock = lock_env_mutation();
        let root =
            std::env::temp_dir().join(format!("memd-hive-propagate-{}", uuid::Uuid::new_v4()));
        let current_project = root.join("current");
        let sibling_project = root.join("sibling");
        fs::create_dir_all(&current_project).expect("create current project");
        fs::create_dir_all(&sibling_project).expect("create sibling project");

        let current_bundle = current_project.join(".memd");
        let sibling_bundle = sibling_project.join(".memd");
        let base_url = "http://127.0.0.1:9".to_string();

        write_init_bundle(&InitArgs {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            global: false,
            project_root: Some(current_project.clone()),
            seed_existing: false,
            agent: "openclaw".to_string(),
            session: Some("session-live-openclaw".to_string()),
            tab_id: None,
            hive_system: Some("openclaw".to_string()),
            hive_role: Some("agent".to_string()),
            capability: vec!["coordination".to_string(), "memory".to_string()],
            hive_group: vec!["project:demo".to_string()],
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            output: current_bundle.clone(),
            base_url: base_url.clone(),
            rag_url: None,
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            voice_mode: Some(default_voice_mode()),
            allow_localhost_read_only_fallback: false,
            force: true,
        })
        .expect("write current bundle");
        set_bundle_hive_project_state(
            &current_bundle,
            true,
            Some("project:demo"),
            Some(Utc::now()),
        )
        .expect("enable current project hive");

        write_init_bundle(&InitArgs {
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            global: false,
            project_root: Some(sibling_project.clone()),
            seed_existing: false,
            agent: "hermes".to_string(),
            session: Some("session-live-hermes".to_string()),
            tab_id: None,
            hive_system: Some("hermes".to_string()),
            hive_role: Some("agent".to_string()),
            capability: vec!["coordination".to_string(), "memory".to_string()],
            hive_group: vec!["project:demo".to_string()],
            hive_group_goal: None,
            authority: Some("participant".to_string()),
            output: sibling_bundle.clone(),
            base_url: base_url.clone(),
            rag_url: None,
            route: "auto".to_string(),
            intent: "current_task".to_string(),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            allow_localhost_read_only_fallback: false,
            force: true,
        })
        .expect("write sibling bundle");
        set_bundle_hive_project_state(
            &sibling_bundle,
            true,
            Some("project:demo"),
            Some(Utc::now()),
        )
        .expect("enable sibling project hive");

        unsafe {
            std::env::set_var("MEMD_WORKER_NAME", "Hermes");
        }
        refresh_bundle_heartbeat(&sibling_bundle, None, false)
            .await
            .expect("refresh sibling heartbeat");

        unsafe {
            std::env::set_var("MEMD_WORKER_NAME", "Openclaw");
        }
        refresh_bundle_heartbeat(&current_bundle, None, false)
            .await
            .expect("refresh current heartbeat");

        let runtime = read_bundle_runtime_config(&current_bundle)
            .expect("read current runtime")
            .expect("current runtime present");

        propagate_hive_metadata_to_active_project_bundles(&current_bundle, &runtime, true)
            .await
            .expect("propagate hive metadata");

        let sibling_heartbeat =
            read_bundle_heartbeat(&sibling_bundle).expect("read sibling heartbeat file");
        let sibling_heartbeat = sibling_heartbeat.expect("sibling heartbeat present");
        assert_eq!(sibling_heartbeat.agent.as_deref(), Some("hermes"));
        assert_eq!(sibling_heartbeat.worker_name.as_deref(), Some("Hermes"));

        unsafe {
            std::env::remove_var("MEMD_WORKER_NAME");
        }
        fs::remove_dir_all(root).expect("cleanup propagation test root");
    }

    #[tokio::test]
    async fn run_resume_feature_verifier_writes_evidence_artifacts() {
        let fixture = test_continuity_fixture_record();
        let verifier = VerifierRecord {
            id: "verifier.feature.bundle.resume".to_string(),
            name: "Resume feature".to_string(),
            verifier_type: "feature_contract".to_string(),
            pillar: "memory-continuity".to_string(),
            family: "bundle-runtime".to_string(),
            subject_ids: vec!["feature.bundle.resume".to_string()],
            fixture_id: fixture.id.clone(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: Vec::new(),
            assertions: Vec::new(),
            metrics: vec!["prompt_tokens".to_string()],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "acceptable".to_string(),
            status: "declared".to_string(),
            lanes: vec!["fast".to_string()],
            helper_hooks: Vec::new(),
        };

        let run = run_verifier_record(&verifier, &fixture, None)
            .await
            .expect("run verifier");
        assert_eq!(run.verifier_id, "verifier.feature.bundle.resume");
        assert!(!run.evidence_ids.is_empty());
        let materialized = materialize_fixture(&fixture, None).expect("materialize fixture again");
        write_verifier_run_artifacts(
            &materialized.bundle_root,
            &run,
            &json!({"verifier_id": verifier.id, "confidence_tier": "live_primary"}),
        )
        .expect("write verifier artifacts");
        assert!(
            verification_reports_dir(&materialized.bundle_root)
                .join("latest.json")
                .exists()
        );
        assert!(verification_evidence_dir(&materialized.bundle_root).exists());
    }

    #[tokio::test]
    async fn run_verifier_record_executes_wake_step_and_writes_wakeup() {
        let fixture = test_continuity_fixture_record();
        let verifier = VerifierRecord {
            id: "verifier.feature.bundle.wake.steps".to_string(),
            name: "Wake feature with steps".to_string(),
            verifier_type: "feature_contract".to_string(),
            pillar: "memory-continuity".to_string(),
            family: "bundle-runtime".to_string(),
            subject_ids: vec!["feature.bundle.wake".to_string()],
            fixture_id: fixture.id.clone(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: vec![VerifierStepRecord {
                kind: "cli".to_string(),
                run: Some("memd wake --output {{bundle}}".to_string()),
                name: None,
                left: None,
                right: None,
            }],
            assertions: vec![VerifierAssertionRecord {
                kind: "file_contains".to_string(),
                path: Some("MEMD_WAKEUP.md".to_string()),
                equals_fixture: None,
                contains_fixture: None,
                exists: Some(true),
                metric: None,
                op: None,
                left: None,
                right: None,
                name: None,
            }],
            metrics: vec!["prompt_tokens".to_string()],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "acceptable".to_string(),
            status: "declared".to_string(),
            lanes: vec!["fast".to_string()],
            helper_hooks: Vec::new(),
        };

        let run = run_verifier_record(&verifier, &fixture, None)
            .await
            .expect("run wake verifier");

        assert_eq!(run.status, "passing");
        assert!(
            run.metrics_observed
                .get("prompt_tokens")
                .and_then(JsonValue::as_u64)
                .is_some_and(|value| value > 0)
        );
    }

    #[tokio::test]
    async fn run_verifier_record_executes_resume_steps_and_records_prompt_tokens() {
        let fixture = test_continuity_fixture_record();
        let verifier = VerifierRecord {
            id: "verifier.feature.bundle.resume.steps".to_string(),
            name: "Resume feature with steps".to_string(),
            verifier_type: "feature_contract".to_string(),
            pillar: "memory-continuity".to_string(),
            family: "bundle-runtime".to_string(),
            subject_ids: vec!["feature.bundle.resume".to_string()],
            fixture_id: fixture.id.clone(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: vec![
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd checkpoint --output {{bundle}}".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd resume --output {{bundle}}".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
            ],
            assertions: vec![VerifierAssertionRecord {
                kind: "json_path".to_string(),
                path: Some("resume.project".to_string()),
                equals_fixture: None,
                contains_fixture: None,
                exists: Some(true),
                metric: None,
                op: None,
                left: None,
                right: None,
                name: None,
            }],
            metrics: vec!["prompt_tokens".to_string()],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "acceptable".to_string(),
            status: "declared".to_string(),
            lanes: vec!["fast".to_string()],
            helper_hooks: Vec::new(),
        };

        let run = run_verifier_record(&verifier, &fixture, None)
            .await
            .expect("run verifier");

        assert_eq!(run.status, "passing");
        assert!(
            run.metrics_observed
                .get("prompt_tokens")
                .and_then(JsonValue::as_u64)
                .is_some_and(|value| value > 0)
        );
    }

    #[tokio::test]
    async fn run_verifier_record_executes_compare_steps_and_records_delta_metrics() {
        let fixture = test_continuity_fixture_record();
        let verifier = VerifierRecord {
            id: "verifier.compare.resume.steps".to_string(),
            name: "Resume compare with steps".to_string(),
            verifier_type: "comparative".to_string(),
            pillar: "efficiency".to_string(),
            family: "memory-continuity".to_string(),
            subject_ids: vec!["journey.resume-handoff-attach".to_string()],
            fixture_id: fixture.id.clone(),
            baseline_modes: vec!["no_mempath".to_string(), "with_memd".to_string()],
            steps: vec![
                VerifierStepRecord {
                    kind: "helper".to_string(),
                    run: None,
                    name: Some("run_resume_without_memd".to_string()),
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "helper".to_string(),
                    run: None,
                    name: Some("run_resume_with_memd".to_string()),
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "compare".to_string(),
                    run: None,
                    name: None,
                    left: Some("no_mempath".to_string()),
                    right: Some("with_memd".to_string()),
                },
            ],
            assertions: vec![VerifierAssertionRecord {
                kind: "metric_compare".to_string(),
                path: None,
                equals_fixture: None,
                contains_fixture: None,
                exists: None,
                metric: Some("prompt_tokens".to_string()),
                op: Some("<".to_string()),
                left: Some("with_memd".to_string()),
                right: Some("no_mempath".to_string()),
                name: None,
            }],
            metrics: vec![
                "prompt_tokens".to_string(),
                "reconstruction_steps".to_string(),
                "token_delta".to_string(),
            ],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "strong".to_string(),
            status: "declared".to_string(),
            lanes: vec!["comparative".to_string()],
            helper_hooks: Vec::new(),
        };

        let run = run_verifier_record(&verifier, &fixture, None)
            .await
            .expect("run comparative verifier");

        assert_eq!(run.status, "passing");
        assert_eq!(
            run.metrics_observed
                .get("token_delta")
                .and_then(JsonValue::as_i64),
            Some(500)
        );
        assert_eq!(
            run.metrics_observed
                .get("with_memd_better")
                .and_then(JsonValue::as_bool),
            Some(true)
        );
    }

    #[tokio::test]
    async fn run_verifier_record_supports_file_contains_assertions() {
        let mut fixture = test_continuity_fixture_record();
        fixture.seed_files = vec!["state/checkpoint.txt".to_string()];
        let verifier = VerifierRecord {
            id: "verifier.feature.file-assert".to_string(),
            name: "File assertion verifier".to_string(),
            verifier_type: "feature_contract".to_string(),
            pillar: "memory-continuity".to_string(),
            family: "bundle-runtime".to_string(),
            subject_ids: vec!["feature.bundle.resume".to_string()],
            fixture_id: fixture.id.clone(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: Vec::new(),
            assertions: vec![VerifierAssertionRecord {
                kind: "file_contains".to_string(),
                path: Some("state/checkpoint.txt".to_string()),
                equals_fixture: None,
                contains_fixture: Some("task.next_action".to_string()),
                exists: None,
                metric: None,
                op: None,
                left: None,
                right: None,
                name: None,
            }],
            metrics: Vec::new(),
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "acceptable".to_string(),
            status: "declared".to_string(),
            lanes: vec!["fast".to_string()],
            helper_hooks: Vec::new(),
        };

        let run = run_verifier_record(&verifier, &fixture, None)
            .await
            .expect("run verifier with file assertion");

        assert_eq!(run.status, "passing");
    }

    #[tokio::test]
    async fn run_verifier_record_executes_messages_send_ack_flow() {
        let base_url = spawn_mock_runtime_server(MockRuntimeState::default(), false).await;
        let fixture = FixtureRecord {
            id: "fixture.hive-two-session-bundle.test".to_string(),
            kind: "bundle_fixture".to_string(),
            description: "hive".to_string(),
            seed_files: Vec::new(),
            seed_config: json!({
                "project": "demo",
                "namespace": "main",
                "agent": "codex",
                "session": "sender",
                "workspace": "shared",
                "base_url": base_url
            }),
            seed_memories: Vec::new(),
            seed_events: Vec::new(),
            seed_sessions: vec!["sender".to_string(), "target".to_string()],
            seed_claims: Vec::new(),
            seed_vault: None,
            backend_mode: "normal".to_string(),
            isolation: "fresh_temp_dir".to_string(),
            cleanup_policy: "destroy".to_string(),
        };
        let verifier = VerifierRecord {
            id: "verifier.feature.hive.messages-send-ack".to_string(),
            name: "Messages send and ack".to_string(),
            verifier_type: "feature_contract".to_string(),
            pillar: "coordination".to_string(),
            family: "coordination-hive".to_string(),
            subject_ids: vec!["feature.hive.messages".to_string()],
            fixture_id: fixture.id.clone(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: vec![
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd messages --output {{sender_bundle}} --send --target-session {{target_session}} --kind handoff --content \"Pick up the parser refactor\"".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd messages --output {{target_bundle}} --inbox".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "helper".to_string(),
                    run: None,
                    name: Some("capture_message_id".to_string()),
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd messages --output {{target_bundle}} --ack {{message_id}}".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
            ],
            assertions: vec![VerifierAssertionRecord {
                kind: "helper".to_string(),
                path: None,
                equals_fixture: None,
                contains_fixture: None,
                exists: None,
                metric: None,
                op: None,
                left: None,
                right: None,
                name: Some("assert_message_acknowledged".to_string()),
            }],
            metrics: vec!["delivery_count".to_string()],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "acceptable".to_string(),
            status: "declared".to_string(),
            lanes: vec!["nightly".to_string()],
            helper_hooks: Vec::new(),
        };

        let run = run_verifier_record(&verifier, &fixture, Some(&base_url))
            .await
            .expect("run hive message verifier");

        assert_eq!(run.status, "passing");
        assert_eq!(
            run.metrics_observed
                .get("delivery_count")
                .and_then(JsonValue::as_u64),
            Some(1)
        );
    }

    #[tokio::test]
    async fn run_verifier_record_executes_claim_transfer_flow() {
        let base_url = spawn_mock_runtime_server(MockRuntimeState::default(), false).await;
        let fixture = FixtureRecord {
            id: "fixture.hive-claims-bundle.test".to_string(),
            kind: "bundle_fixture".to_string(),
            description: "hive claims".to_string(),
            seed_files: Vec::new(),
            seed_config: json!({
                "project": "demo",
                "namespace": "main",
                "agent": "codex",
                "session": "sender",
                "workspace": "shared",
                "base_url": base_url
            }),
            seed_memories: Vec::new(),
            seed_events: Vec::new(),
            seed_sessions: vec!["sender".to_string(), "target".to_string()],
            seed_claims: Vec::new(),
            seed_vault: None,
            backend_mode: "normal".to_string(),
            isolation: "fresh_temp_dir".to_string(),
            cleanup_policy: "destroy".to_string(),
        };
        let verifier = VerifierRecord {
            id: "verifier.feature.hive.claims-transfer".to_string(),
            name: "Claims transfer".to_string(),
            verifier_type: "feature_contract".to_string(),
            pillar: "coordination".to_string(),
            family: "coordination-hive".to_string(),
            subject_ids: vec!["feature.hive.claims".to_string()],
            fixture_id: fixture.id.clone(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: vec![
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd claims --output {{sender_bundle}} --acquire --scope task:parser-refactor".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd claims --output {{sender_bundle}} --transfer-to-session {{target_session}} --scope task:parser-refactor".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
            ],
            assertions: vec![VerifierAssertionRecord {
                kind: "json_path".to_string(),
                path: Some("claims_transfer.claims.0.session".to_string()),
                equals_fixture: Some("target_session".to_string()),
                contains_fixture: None,
                exists: None,
                metric: None,
                op: None,
                left: None,
                right: None,
                name: None,
            }],
            metrics: vec!["claim_count".to_string()],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "acceptable".to_string(),
            status: "declared".to_string(),
            lanes: vec!["nightly".to_string()],
            helper_hooks: Vec::new(),
        };

        let run = run_verifier_record(&verifier, &fixture, Some(&base_url))
            .await
            .expect("run hive claims verifier");

        assert_eq!(run.status, "passing");
        assert_eq!(
            run.metrics_observed
                .get("claim_count")
                .and_then(JsonValue::as_u64),
            Some(1)
        );
    }

    #[tokio::test]
    async fn run_verifier_record_executes_task_assignment_flow() {
        let base_url = spawn_mock_runtime_server(MockRuntimeState::default(), false).await;
        let fixture = FixtureRecord {
            id: "fixture.hive-tasks-bundle.test".to_string(),
            kind: "bundle_fixture".to_string(),
            description: "hive tasks".to_string(),
            seed_files: Vec::new(),
            seed_config: json!({
                "project": "demo",
                "namespace": "main",
                "agent": "codex",
                "session": "sender",
                "workspace": "shared",
                "base_url": base_url
            }),
            seed_memories: Vec::new(),
            seed_events: Vec::new(),
            seed_sessions: vec!["sender".to_string(), "target".to_string()],
            seed_claims: Vec::new(),
            seed_vault: None,
            backend_mode: "normal".to_string(),
            isolation: "fresh_temp_dir".to_string(),
            cleanup_policy: "destroy".to_string(),
        };
        let verifier = VerifierRecord {
            id: "verifier.feature.hive.tasks-assign".to_string(),
            name: "Tasks assign".to_string(),
            verifier_type: "feature_contract".to_string(),
            pillar: "coordination".to_string(),
            family: "coordination-hive".to_string(),
            subject_ids: vec!["feature.hive.tasks".to_string()],
            fixture_id: fixture.id.clone(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: vec![
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd tasks --output {{sender_bundle}} --upsert --task-id parser-refactor --title \"Parser refactor\" --status in_progress --mode exclusive_write --scope task:parser-refactor".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd tasks --output {{sender_bundle}} --assign-to-session {{target_session}} --task-id parser-refactor".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
            ],
            assertions: vec![VerifierAssertionRecord {
                kind: "json_path".to_string(),
                path: Some("tasks_assign.tasks.0.session".to_string()),
                equals_fixture: Some("target_session".to_string()),
                contains_fixture: None,
                exists: None,
                metric: None,
                op: None,
                left: None,
                right: None,
                name: None,
            }],
            metrics: vec!["task_count".to_string()],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "acceptable".to_string(),
            status: "declared".to_string(),
            lanes: vec!["nightly".to_string()],
            helper_hooks: Vec::new(),
        };

        let run = run_verifier_record(&verifier, &fixture, Some(&base_url))
            .await
            .expect("run hive task verifier");

        assert_eq!(run.status, "passing");
        assert_eq!(
            run.metrics_observed
                .get("task_count")
                .and_then(JsonValue::as_u64),
            Some(1)
        );
    }

    #[tokio::test]
    async fn run_verifier_record_executes_hive_transfer_assign_journey() {
        let base_url = spawn_mock_runtime_server(MockRuntimeState::default(), false).await;
        let fixture = FixtureRecord {
            id: "fixture.hive-journey-bundle.test".to_string(),
            kind: "bundle_fixture".to_string(),
            description: "hive journey".to_string(),
            seed_files: Vec::new(),
            seed_config: json!({
                "project": "demo",
                "namespace": "main",
                "agent": "codex",
                "session": "sender",
                "workspace": "shared",
                "base_url": base_url
            }),
            seed_memories: Vec::new(),
            seed_events: Vec::new(),
            seed_sessions: vec!["sender".to_string(), "target".to_string()],
            seed_claims: Vec::new(),
            seed_vault: None,
            backend_mode: "normal".to_string(),
            isolation: "fresh_temp_dir".to_string(),
            cleanup_policy: "destroy".to_string(),
        };
        let verifier = VerifierRecord {
            id: "verifier.journey.hive-transfer-assign".to_string(),
            name: "Hive transfer assign journey".to_string(),
            verifier_type: "journey".to_string(),
            pillar: "memory-continuity".to_string(),
            family: "coordination-hive".to_string(),
            subject_ids: vec!["journey.hive.transfer-assign".to_string()],
            fixture_id: fixture.id.clone(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: vec![
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd messages --output {{sender_bundle}} --send --target-session {{target_session}} --kind handoff --content \"Pick up {{task.id}}\"".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd messages --output {{target_bundle}} --inbox".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "helper".to_string(),
                    run: None,
                    name: Some("capture_message_id".to_string()),
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd messages --output {{target_bundle}} --ack {{message_id}}".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd claims --output {{sender_bundle}} --acquire --scope task:{{task.id}}".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd claims --output {{sender_bundle}} --transfer-to-session {{target_session}} --scope task:{{task.id}}".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd tasks --output {{sender_bundle}} --upsert --task-id {{task.id}} --title \"Parser refactor\" --status in_progress --mode exclusive_write --scope task:{{task.id}}".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd tasks --output {{sender_bundle}} --assign-to-session {{target_session}} --task-id {{task.id}}".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
            ],
            assertions: vec![
                VerifierAssertionRecord {
                    kind: "helper".to_string(),
                    path: None,
                    equals_fixture: None,
                    contains_fixture: None,
                    exists: None,
                    metric: None,
                    op: None,
                    left: None,
                    right: None,
                    name: Some("assert_message_acknowledged".to_string()),
                },
                VerifierAssertionRecord {
                    kind: "json_path".to_string(),
                    path: Some("claims_transfer.claims.0.session".to_string()),
                    equals_fixture: Some("target_session".to_string()),
                    contains_fixture: None,
                    exists: None,
                    metric: None,
                    op: None,
                    left: None,
                    right: None,
                    name: None,
                },
                VerifierAssertionRecord {
                    kind: "json_path".to_string(),
                    path: Some("tasks_assign.tasks.0.session".to_string()),
                    equals_fixture: Some("target_session".to_string()),
                    contains_fixture: None,
                    exists: None,
                    metric: None,
                    op: None,
                    left: None,
                    right: None,
                    name: None,
                },
            ],
            metrics: vec![
                "delivery_count".to_string(),
                "claim_count".to_string(),
                "task_count".to_string(),
            ],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "strong".to_string(),
            status: "declared".to_string(),
            lanes: vec!["nightly".to_string()],
            helper_hooks: vec![
                "capture_message_id".to_string(),
                "assert_message_acknowledged".to_string(),
            ],
        };

        let run = run_verifier_record(&verifier, &fixture, Some(&base_url))
            .await
            .expect("run hive journey verifier");

        assert_eq!(run.status, "passing");
        assert_eq!(run.gate_result, "strong");
        assert_eq!(
            run.metrics_observed
                .get("delivery_count")
                .and_then(JsonValue::as_u64),
            Some(1)
        );
        assert_eq!(
            run.metrics_observed
                .get("claim_count")
                .and_then(JsonValue::as_u64),
            Some(1)
        );
        assert_eq!(
            run.metrics_observed
                .get("task_count")
                .and_then(JsonValue::as_u64),
            Some(1)
        );
    }

    #[tokio::test]
    async fn run_verifier_record_contains_hive_claim_collision() {
        let base_url = spawn_mock_runtime_server(MockRuntimeState::default(), false).await;
        let fixture = FixtureRecord {
            id: "fixture.hive-adversarial-bundle.test".to_string(),
            kind: "bundle_fixture".to_string(),
            description: "hive adversarial".to_string(),
            seed_files: Vec::new(),
            seed_config: json!({
                "project": "demo",
                "namespace": "main",
                "agent": "codex",
                "session": "sender",
                "workspace": "shared",
                "base_url": base_url
            }),
            seed_memories: Vec::new(),
            seed_events: Vec::new(),
            seed_sessions: vec!["sender".to_string(), "target".to_string()],
            seed_claims: Vec::new(),
            seed_vault: None,
            backend_mode: "normal".to_string(),
            isolation: "fresh_temp_dir".to_string(),
            cleanup_policy: "destroy".to_string(),
        };
        let verifier = VerifierRecord {
            id: "verifier.adversarial.hive-claim-collision".to_string(),
            name: "Hive claim collision containment".to_string(),
            verifier_type: "adversarial".to_string(),
            pillar: "memory-continuity".to_string(),
            family: "coordination-hive".to_string(),
            subject_ids: vec![
                "feature.hive.claims".to_string(),
                "journey.hive.transfer-assign".to_string(),
            ],
            fixture_id: fixture.id.clone(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: vec![
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some(
                        "memd claims --output {{sender_bundle}} --acquire --scope task:{{task.id}}"
                            .to_string(),
                    ),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli_expect_error".to_string(),
                    run: Some(
                        "memd claims --output {{target_bundle}} --acquire --scope task:{{task.id}}"
                            .to_string(),
                    ),
                    name: None,
                    left: None,
                    right: None,
                },
            ],
            assertions: vec![
                VerifierAssertionRecord {
                    kind: "json_path".to_string(),
                    path: Some("expected_error.message".to_string()),
                    equals_fixture: None,
                    contains_fixture: None,
                    exists: Some(true),
                    metric: None,
                    op: None,
                    left: None,
                    right: None,
                    name: None,
                },
                VerifierAssertionRecord {
                    kind: "json_path".to_string(),
                    path: Some("claims_acquire.claims.0.session".to_string()),
                    equals_fixture: Some("sender_session".to_string()),
                    contains_fixture: None,
                    exists: None,
                    metric: None,
                    op: None,
                    left: None,
                    right: None,
                    name: None,
                },
            ],
            metrics: vec!["claim_count".to_string()],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "acceptable".to_string(),
            status: "declared".to_string(),
            lanes: vec!["nightly".to_string()],
            helper_hooks: Vec::new(),
        };

        let run = run_verifier_record(&verifier, &fixture, Some(&base_url))
            .await
            .expect("run hive adversarial verifier");

        assert_eq!(run.status, "passing");
        assert_eq!(run.gate_result, "acceptable");
        assert_eq!(
            run.metrics_observed
                .get("claim_count")
                .and_then(JsonValue::as_u64),
            Some(1)
        );
    }

    #[tokio::test]
    async fn run_verifier_record_contains_hive_task_lane_collision() {
        let base_url = spawn_mock_runtime_server(MockRuntimeState::default(), false).await;
        let fixture = FixtureRecord {
            id: "fixture.hive-task-lane-adversarial-bundle.test".to_string(),
            kind: "bundle_fixture".to_string(),
            description: "hive task lane adversarial".to_string(),
            seed_files: Vec::new(),
            seed_config: json!({
                "project": "demo",
                "namespace": "main",
                "agent": "codex",
                "session": "sender",
                "workspace": "shared",
                "base_url": base_url
            }),
            seed_memories: Vec::new(),
            seed_events: Vec::new(),
            seed_sessions: vec!["sender".to_string(), "target".to_string()],
            seed_claims: Vec::new(),
            seed_vault: None,
            backend_mode: "normal".to_string(),
            isolation: "fresh_temp_dir".to_string(),
            cleanup_policy: "destroy".to_string(),
        };
        let verifier = VerifierRecord {
            id: "verifier.adversarial.hive-task-lane-collision".to_string(),
            name: "Hive task lane collision containment".to_string(),
            verifier_type: "adversarial".to_string(),
            pillar: "memory-continuity".to_string(),
            family: "coordination-hive".to_string(),
            subject_ids: vec![
                "feature.hive.tasks".to_string(),
                "journey.hive.transfer-assign".to_string(),
            ],
            fixture_id: fixture.id.clone(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: vec![
                VerifierStepRecord {
                    kind: "helper".to_string(),
                    run: None,
                    name: Some("setup_target_lane_collision".to_string()),
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli".to_string(),
                    run: Some("memd tasks --output {{sender_bundle}} --upsert --task-id {{task.id}} --title \"Parser refactor\" --status in_progress --mode exclusive_write --scope task:{{task.id}}".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli_expect_error".to_string(),
                    run: Some("memd tasks --output {{sender_bundle}} --assign-to-session {{target_session}} --task-id {{task.id}}".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
            ],
            assertions: vec![VerifierAssertionRecord {
                kind: "json_path".to_string(),
                path: Some("expected_error.message".to_string()),
                equals_fixture: None,
                contains_fixture: None,
                exists: Some(true),
                metric: None,
                op: None,
                left: None,
                right: None,
                name: None,
            }],
            metrics: vec!["task_count".to_string(), "expected_error_count".to_string()],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "acceptable".to_string(),
            status: "declared".to_string(),
            lanes: vec!["nightly".to_string()],
            helper_hooks: vec!["setup_target_lane_collision".to_string()],
        };

        let run = run_verifier_record(&verifier, &fixture, Some(&base_url))
            .await
            .expect("run hive task lane adversarial verifier");

        assert_eq!(run.status, "passing");
        assert_eq!(run.gate_result, "acceptable");
        assert_eq!(
            run.metrics_observed
                .get("expected_error_count")
                .and_then(JsonValue::as_u64),
            Some(1)
        );
    }

    #[tokio::test]
    async fn run_verifier_record_contains_hive_message_lane_collision() {
        let base_url = spawn_mock_runtime_server(MockRuntimeState::default(), false).await;
        let fixture = FixtureRecord {
            id: "fixture.hive-message-lane-adversarial-bundle.test".to_string(),
            kind: "bundle_fixture".to_string(),
            description: "hive message lane adversarial".to_string(),
            seed_files: Vec::new(),
            seed_config: json!({
                "project": "demo",
                "namespace": "main",
                "agent": "codex",
                "session": "sender",
                "workspace": "shared",
                "base_url": base_url
            }),
            seed_memories: Vec::new(),
            seed_events: Vec::new(),
            seed_sessions: vec!["sender".to_string(), "target".to_string()],
            seed_claims: Vec::new(),
            seed_vault: None,
            backend_mode: "normal".to_string(),
            isolation: "fresh_temp_dir".to_string(),
            cleanup_policy: "destroy".to_string(),
        };
        let verifier = VerifierRecord {
            id: "verifier.adversarial.hive-message-lane-collision".to_string(),
            name: "Hive message lane collision containment".to_string(),
            verifier_type: "adversarial".to_string(),
            pillar: "memory-continuity".to_string(),
            family: "coordination-hive".to_string(),
            subject_ids: vec![
                "feature.hive.messages".to_string(),
                "journey.hive.transfer-assign".to_string(),
            ],
            fixture_id: fixture.id.clone(),
            baseline_modes: vec!["with_memd".to_string()],
            steps: vec![
                VerifierStepRecord {
                    kind: "helper".to_string(),
                    run: None,
                    name: Some("setup_target_lane_collision".to_string()),
                    left: None,
                    right: None,
                },
                VerifierStepRecord {
                    kind: "cli_expect_error".to_string(),
                    run: Some("memd messages --output {{sender_bundle}} --send --target-session {{target_session}} --kind handoff --content \"Pick up {{task.id}}\"".to_string()),
                    name: None,
                    left: None,
                    right: None,
                },
            ],
            assertions: vec![VerifierAssertionRecord {
                kind: "json_path".to_string(),
                path: Some("expected_error.message".to_string()),
                equals_fixture: None,
                contains_fixture: None,
                exists: Some(true),
                metric: None,
                op: None,
                left: None,
                right: None,
                name: None,
            }],
            metrics: vec!["expected_error_count".to_string()],
            evidence_requirements: vec!["live_primary".to_string()],
            gate_target: "acceptable".to_string(),
            status: "declared".to_string(),
            lanes: vec!["nightly".to_string()],
            helper_hooks: vec!["setup_target_lane_collision".to_string()],
        };

        let run = run_verifier_record(&verifier, &fixture, Some(&base_url))
            .await
            .expect("run hive message lane adversarial verifier");

        assert_eq!(run.status, "passing");
        assert_eq!(run.gate_result, "acceptable");
        assert_eq!(
            run.metrics_observed
                .get("expected_error_count")
                .and_then(JsonValue::as_u64),
            Some(1)
        );
    }

    #[tokio::test]
    async fn run_verify_feature_command_executes_seeded_handoff_verifier() {
        let dir = std::env::temp_dir().join(format!(
            "memd-verify-handoff-command-{}",
            uuid::Uuid::new_v4()
        ));
        let repo_root = dir.join("repo");
        let output = repo_root.join(".memd");
        fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
        fs::create_dir_all(&output).expect("create output dir");
        write_test_benchmark_registry(&repo_root);

        let report = run_verify_feature_command(&VerifyFeatureArgs {
            feature_id: "feature.bundle.handoff".to_string(),
            output: output.clone(),
            summary: false,
        })
        .await
        .expect("run verify handoff feature command");

        assert_eq!(report.subject.as_deref(), Some("feature.bundle.handoff"));
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding == "verifier_run_status=passing")
        );

        fs::remove_dir_all(dir).expect("cleanup verify handoff dir");
    }

    #[tokio::test]
    async fn run_verify_feature_command_executes_seeded_verifier() {
        let dir = std::env::temp_dir().join(format!(
            "memd-verify-feature-command-{}",
            uuid::Uuid::new_v4()
        ));
        let repo_root = dir.join("repo");
        let output = repo_root.join(".memd");
        fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
        fs::create_dir_all(&output).expect("create output dir");
        write_test_benchmark_registry(&repo_root);

        let report = run_verify_feature_command(&VerifyFeatureArgs {
            feature_id: "feature.bundle.resume".to_string(),
            output: output.clone(),
            summary: false,
        })
        .await
        .expect("run verify feature command");

        assert_eq!(report.subject.as_deref(), Some("feature.bundle.resume"));
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding == "verifier_run_status=passing")
        );
        assert!(
            verification_reports_dir(&output)
                .join("latest.json")
                .exists()
        );

        fs::remove_dir_all(dir).expect("cleanup verify feature dir");
    }

    #[tokio::test]
    async fn run_verify_compare_command_executes_seeded_verifier() {
        let dir = std::env::temp_dir().join(format!(
            "memd-verify-compare-command-{}",
            uuid::Uuid::new_v4()
        ));
        let repo_root = dir.join("repo");
        let output = repo_root.join(".memd");
        fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
        fs::create_dir_all(&output).expect("create output dir");
        write_test_benchmark_registry(&repo_root);

        let report = run_verify_compare_command(&VerifyCompareArgs {
            verifier_id: "verifier.compare.resume-no-memd-vs-with-memd".to_string(),
            output: output.clone(),
            summary: false,
        })
        .await
        .expect("run verify compare command");

        assert_eq!(
            report.subject.as_deref(),
            Some("verifier.compare.resume-no-memd-vs-with-memd")
        );
        assert_eq!(report.baseline.as_deref(), Some("no_mempath,with_memd"));
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding == "gate_result=strong")
        );
        assert!(
            verification_reports_dir(&output)
                .join("latest.json")
                .exists()
        );

        fs::remove_dir_all(dir).expect("cleanup verify compare dir");
    }

    #[tokio::test]
    async fn run_verify_journey_command_executes_seeded_hive_journey() {
        let dir = std::env::temp_dir().join(format!(
            "memd-verify-journey-command-{}",
            uuid::Uuid::new_v4()
        ));
        let repo_root = dir.join("repo");
        let output = repo_root.join(".memd");
        fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
        fs::create_dir_all(&output).expect("create output dir");
        write_test_benchmark_registry(&repo_root);

        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state, false).await;
        write_test_bundle_config(&output, &base_url);

        let report = run_verify_journey_command(&VerifyJourneyArgs {
            journey_id: "journey.hive.transfer-assign".to_string(),
            output: output.clone(),
            summary: false,
        })
        .await
        .expect("run verify journey command");

        assert_eq!(
            report.subject.as_deref(),
            Some("journey.hive.transfer-assign")
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding == "verifier_run_status=passing")
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding == "gate_result=strong")
        );

        fs::remove_dir_all(dir).expect("cleanup verify journey dir");
    }

    #[tokio::test]
    async fn run_verify_adversarial_command_executes_seeded_hive_collision_verifier() {
        let dir = std::env::temp_dir().join(format!(
            "memd-verify-adversarial-command-{}",
            uuid::Uuid::new_v4()
        ));
        let repo_root = dir.join("repo");
        let output = repo_root.join(".memd");
        fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
        fs::create_dir_all(&output).expect("create output dir");
        write_test_benchmark_registry(&repo_root);

        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state, false).await;
        write_test_bundle_config(&output, &base_url);

        let report = run_verify_adversarial_command(&VerifyAdversarialArgs {
            verifier_id: "verifier.adversarial.hive-claim-collision".to_string(),
            output: output.clone(),
            summary: false,
        })
        .await
        .expect("run verify adversarial command");

        assert_eq!(
            report.subject.as_deref(),
            Some("verifier.adversarial.hive-claim-collision")
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding == "verifier_run_status=passing")
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding == "gate_result=acceptable")
        );

        fs::remove_dir_all(dir).expect("cleanup verify adversarial dir");
    }

    #[tokio::test]
    async fn run_verify_adversarial_command_executes_seeded_hive_task_lane_collision_verifier() {
        let dir = std::env::temp_dir().join(format!(
            "memd-verify-adversarial-task-lane-command-{}",
            uuid::Uuid::new_v4()
        ));
        let repo_root = dir.join("repo");
        let output = repo_root.join(".memd");
        fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
        fs::create_dir_all(&output).expect("create output dir");
        write_test_benchmark_registry(&repo_root);

        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state, false).await;
        write_test_bundle_config(&output, &base_url);

        let report = run_verify_adversarial_command(&VerifyAdversarialArgs {
            verifier_id: "verifier.adversarial.hive-task-lane-collision".to_string(),
            output: output.clone(),
            summary: false,
        })
        .await
        .expect("run verify adversarial task lane command");

        assert_eq!(
            report.subject.as_deref(),
            Some("verifier.adversarial.hive-task-lane-collision")
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding == "verifier_run_status=passing")
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding == "gate_result=acceptable")
        );

        fs::remove_dir_all(dir).expect("cleanup verify adversarial task lane dir");
    }

    #[tokio::test]
    async fn run_verify_adversarial_command_executes_seeded_hive_message_lane_collision_verifier() {
        let dir = std::env::temp_dir().join(format!(
            "memd-verify-adversarial-message-lane-command-{}",
            uuid::Uuid::new_v4()
        ));
        let repo_root = dir.join("repo");
        let output = repo_root.join(".memd");
        fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
        fs::create_dir_all(&output).expect("create output dir");
        write_test_benchmark_registry(&repo_root);

        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state, false).await;
        write_test_bundle_config(&output, &base_url);

        let report = run_verify_adversarial_command(&VerifyAdversarialArgs {
            verifier_id: "verifier.adversarial.hive-message-lane-collision".to_string(),
            output: output.clone(),
            summary: false,
        })
        .await
        .expect("run verify adversarial message lane command");

        assert_eq!(
            report.subject.as_deref(),
            Some("verifier.adversarial.hive-message-lane-collision")
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding == "verifier_run_status=passing")
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding == "gate_result=acceptable")
        );

        fs::remove_dir_all(dir).expect("cleanup verify adversarial message lane dir");
    }

    #[test]
    fn derived_only_evidence_caps_gate_at_fragile() {
        let gate = resolve_verifier_gate("acceptable", &["derived".to_string()], true, true, true);
        assert_eq!(gate, "fragile");
    }

    #[test]
    fn comparative_loss_caps_gate_at_acceptable() {
        let gate =
            resolve_verifier_gate("strong", &["live_primary".to_string()], true, true, false);
        assert_eq!(gate, "acceptable");
    }

    #[tokio::test]
    async fn nightly_sweep_fails_on_tier_zero_failure() {
        let report = run_verify_sweep_for_test("nightly", vec![test_failing_tier_zero_verifier()])
            .await
            .expect("run nightly sweep");
        assert!(!report.ok);
    }

    #[tokio::test]
    async fn nightly_sweep_reports_noncritical_failures_without_failing() {
        let report = run_verify_sweep_for_test("nightly", vec![test_failing_tier_two_verifier()])
            .await
            .expect("run nightly sweep");
        assert!(report.ok);
        assert_eq!(report.failures.len(), 1);
    }

    #[tokio::test]
    async fn run_feature_benchmark_command_scores_feature_inventory_and_writes_artifacts() {
        let dir =
            std::env::temp_dir().join(format!("memd-feature-benchmark-{}", uuid::Uuid::new_v4()));
        let repo_root = dir.join("repo");
        let output = repo_root.join(".memd");
        fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
        fs::create_dir_all(&output).expect("create benchmark temp bundle");

        let state = MockRuntimeState::default();
        let base_url = spawn_mock_runtime_server(state, false).await;
        write_test_bundle_config(&output, &base_url);
        write_bundle_command_catalog_files(&output).expect("write command catalog");
        write_test_benchmark_registry(&repo_root);

        let snapshot = test_autoresearch_snapshot(
            false,
            vec!["keep the hive compact".to_string()],
            vec!["crates/memd-client/src/main.rs".to_string()],
        );
        write_bundle_memory_files(&output, &snapshot, None, false)
            .await
            .expect("write bundle memory files");
        refresh_live_bundle_event_pages(&output, &snapshot, None).expect("refresh event pages");

        let eval = high_scoring_eval(&output);
        write_bundle_eval_artifacts(&output, &eval).expect("write eval artifacts");
        let scenario = high_scoring_scenario(&output);
        write_scenario_artifacts(&output, &scenario).expect("write scenario artifacts");
        write_maintain_artifacts(
            &output,
            &MaintainReport {
                mode: "scan".to_string(),
                receipt_id: Some("maint-1".to_string()),
                compacted_items: 2,
                refreshed_items: 1,
                repaired_items: 0,
                findings: vec!["memory drift low".to_string()],
                generated_at: Utc::now(),
            },
        )
        .expect("write maintain artifacts");

        let experiment = test_experiment_report(&output, true, false, 92, 100, Utc::now());
        let experiments_dir = experiment_reports_dir(&output);
        fs::create_dir_all(&experiments_dir).expect("create experiments dir");
        fs::write(
            experiments_dir.join("latest.json"),
            serde_json::to_string_pretty(&experiment).expect("serialize experiment") + "\n",
        )
        .expect("write experiment latest");

        let evolution_dir = evolution_reports_dir(&output);
        fs::create_dir_all(&evolution_dir).expect("create evolution dir");
        let proposal = EvolutionProposalReport {
            bundle_root: output.display().to_string(),
            project: Some("demo".to_string()),
            namespace: Some("main".to_string()),
            agent: Some("codex".to_string()),
            session: Some("codex-a".to_string()),
            workspace: Some("shared".to_string()),
            visibility: Some("workspace".to_string()),
            proposal_id: "prop-1".to_string(),
            scenario: Some("self_evolution".to_string()),
            topic: "feature benchmark".to_string(),
            branch: "auto/evolution/feature-benchmark".to_string(),
            state: "accepted_proposal".to_string(),
            scope_class: "low_risk_evaluation_code".to_string(),
            scope_gate: "auto_merge".to_string(),
            authority_tier: "bundle".to_string(),
            allowed_write_surface: vec!["crates/memd-client/src/main.rs".to_string()],
            merge_eligible: true,
            durable_truth: false,
            accepted: true,
            restored: false,
            composite_score: 92,
            composite_max: 100,
            evidence: vec!["benchmark gate passed".to_string()],
            scope_reasons: vec!["bounded change".to_string()],
            generated_at: Utc::now(),
            durability_due_at: None,
        };
        fs::write(
            evolution_dir.join("latest-proposal.json"),
            serde_json::to_string_pretty(&proposal).expect("serialize proposal") + "\n",
        )
        .expect("write proposal latest");
        let branch = EvolutionBranchManifest {
            proposal_id: "prop-1".to_string(),
            branch: "auto/evolution/feature-benchmark".to_string(),
            branch_prefix: "auto/evolution".to_string(),
            project_root: Some(output.display().to_string()),
            head_sha: Some("abc123".to_string()),
            base_branch: Some("main".to_string()),
            status: "ready".to_string(),
            merge_eligible: true,
            durable_truth: false,
            scope_class: "low_risk_evaluation_code".to_string(),
            scope_gate: "auto_merge".to_string(),
            generated_at: Utc::now(),
            notes: vec!["benchmark branch".to_string()],
        };
        fs::write(
            evolution_dir.join("latest-branch.json"),
            serde_json::to_string_pretty(&branch).expect("serialize branch") + "\n",
        )
        .expect("write branch latest");

        let report = run_feature_benchmark_command(
            &BenchmarkArgs {
                output: output.clone(),
                write: false,
                summary: false,
                subcommand: None,
            },
            &base_url,
        )
        .await
        .expect("run feature benchmark");

        assert_eq!(report.areas.len(), 10);
        assert!(report.score > 0);
        assert!(
            report
                .evidence
                .iter()
                .any(|item| item.contains("benchmark_registry root="))
        );
        assert!(
            report
                .areas
                .iter()
                .any(|area| area.slug == "coordination_hive" && area.score > 0)
        );
        assert!(report.areas.iter().any(|area| {
            area.slug == "core_memory"
                && area
                    .evidence
                    .iter()
                    .any(|item| item.contains("memory_quality="))
        }));

        write_feature_benchmark_artifacts(&output, &report).expect("write benchmark artifacts");
        let benchmark_dir = feature_benchmark_reports_dir(&output);
        assert!(benchmark_dir.join("latest.json").exists());
        assert!(benchmark_dir.join("latest.md").exists());
        let (loaded_root, registry) = load_benchmark_registry_for_output(&output)
            .expect("load benchmark registry")
            .expect("registry present");
        write_benchmark_registry_docs(&loaded_root, &registry, &report)
            .expect("write benchmark registry docs");
        assert!(benchmark_registry_markdown_path(&loaded_root, "BENCHMARKS.md").exists());
        assert!(benchmark_registry_markdown_path(&loaded_root, "LOOPS.md").exists());
        assert!(benchmark_registry_markdown_path(&loaded_root, "COVERAGE.md").exists());
        assert!(benchmark_registry_markdown_path(&loaded_root, "SCORES.md").exists());
        assert!(benchmark_registry_markdown_path(&loaded_root, "MORNING.md").exists());
        assert!(
            benchmark_telemetry_dir(&output)
                .join("latest.json")
                .exists()
        );
        assert!(benchmark_telemetry_dir(&output).join("latest.md").exists());

        fs::remove_dir_all(dir).expect("cleanup benchmark dir");
    }

    #[tokio::test]
    async fn load_benchmark_registry_from_output_reads_repo_root_registry() {
        let dir = std::env::temp_dir().join(format!(
            "memd-benchmark-registry-load-{}",
            uuid::Uuid::new_v4()
        ));
        let repo_root = dir.join("repo");
        let output = repo_root.join(".memd");
        fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
        fs::create_dir_all(&output).expect("create output dir");
        write_test_benchmark_registry(&repo_root);

        let (loaded_root, registry) = load_benchmark_registry_for_output(&output)
            .expect("load benchmark registry")
            .expect("registry should be discovered");
        assert_eq!(loaded_root, repo_root);
        assert_eq!(registry.version, "v1");
        assert!(!registry.features.is_empty());
        assert!(!registry.loops.is_empty());

        fs::remove_dir_all(dir).expect("cleanup benchmark registry load dir");
    }

    #[test]
    fn write_benchmark_registry_docs_writes_expected_markdown_outputs() {
        let dir = std::env::temp_dir().join(format!(
            "memd-benchmark-registry-docs-{}",
            uuid::Uuid::new_v4()
        ));
        let repo_root = dir.join("repo");
        fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
        write_test_benchmark_registry(&repo_root);
        let registry = test_benchmark_registry();
        let benchmark = test_feature_benchmark_report(&repo_root.join(".memd"));

        let report = build_benchmark_registry_docs_report(&repo_root, &registry, &benchmark);
        assert!(
            report
                .benchmarks_markdown
                .contains("# memd benchmark registry")
        );
        assert!(report.loops_markdown.contains("# memd benchmark loops"));
        assert!(report.loops_markdown.contains("Coverage Gaps"));
        assert!(
            report
                .coverage_markdown
                .contains("# memd benchmark coverage")
        );
        assert!(report.coverage_markdown.contains("Coverage Summary"));
        assert!(report.coverage_markdown.contains("Benchmark Gaps"));
        assert!(report.scores_markdown.contains("# memd benchmark scores"));
        assert!(report._comparative_report.is_some());
        assert!(report.scores_markdown.contains("Comparative Evidence"));

        write_benchmark_registry_docs(&repo_root, &registry, &benchmark)
            .expect("write benchmark registry docs");
        assert!(benchmark_registry_markdown_path(&repo_root, "BENCHMARKS.md").exists());
        assert!(benchmark_registry_markdown_path(&repo_root, "LOOPS.md").exists());
        assert!(benchmark_registry_markdown_path(&repo_root, "COVERAGE.md").exists());
        assert!(benchmark_registry_markdown_path(&repo_root, "SCORES.md").exists());
        assert!(benchmark_registry_markdown_path(&repo_root, "MORNING.md").exists());
        assert!(
            benchmark_telemetry_dir(Path::new(&benchmark.bundle_root))
                .join("latest.json")
                .exists()
        );
        assert!(
            benchmark_telemetry_dir(Path::new(&benchmark.bundle_root))
                .join("latest.md")
                .exists()
        );

        let benchmarks_md = fs::read_to_string(benchmark_registry_markdown_path(
            &repo_root,
            "BENCHMARKS.md",
        ))
        .expect("read benchmarks md");
        assert!(benchmarks_md.contains("Current benchmark score"));
        let morning_md =
            fs::read_to_string(benchmark_registry_markdown_path(&repo_root, "MORNING.md"))
                .expect("read morning md");
        assert!(morning_md.contains("# memd morning summary"));
        assert!(morning_md.contains("Continuity Failures"));
        assert!(morning_md.contains("Verification Regressions"));

        fs::remove_dir_all(dir).expect("cleanup benchmark registry docs dir");
    }

    #[test]
    fn build_benchmark_gap_candidates_surfaces_unbenchmarked_continuity_feature() {
        let mut registry = test_benchmark_registry();
        registry.features = vec![BenchmarkFeatureRecord {
            id: "feature.bundle.resume".to_string(),
            name: "Resume".to_string(),
            pillar: "memory-continuity".to_string(),
            family: "bundle-runtime".to_string(),
            tier: "tier-0-continuity-critical".to_string(),
            continuity_critical: true,
            user_contract: "resume restores continuity".to_string(),
            source_contract_refs: Vec::new(),
            commands: vec!["memd resume".to_string()],
            routes: Vec::new(),
            files: vec!["crates/memd-client/src/main.rs".to_string()],
            journey_ids: Vec::new(),
            loop_ids: Vec::new(),
            quality_dimensions: vec!["continuity".to_string()],
            drift_risks: vec!["continuity-drift".to_string()],
            failure_modes: vec!["resume misses task state".to_string()],
            coverage_status: "unbenchmarked".to_string(),
            last_verified_at: None,
        }];

        let gaps = build_benchmark_gap_candidates(&registry);
        assert!(
            gaps.iter()
                .any(|gap| gap.id == "benchmark:unbenchmarked_continuity_feature")
        );
    }

    #[test]
    fn build_telemetry_benchmark_coverage_surfaces_registry_gaps() {
        let dir =
            std::env::temp_dir().join(format!("memd-benchmark-telemetry-{}", uuid::Uuid::new_v4()));
        let repo_root = dir.join("repo");
        let output = repo_root.join(".memd");
        fs::create_dir_all(repo_root.join(".git")).expect("create git dir");
        fs::create_dir_all(feature_benchmark_reports_dir(&output)).expect("create benchmark dir");
        write_test_benchmark_registry(&repo_root);
        let report = test_feature_benchmark_report(&output);
        fs::write(
            feature_benchmark_reports_dir(&output).join("latest.json"),
            serde_json::to_string_pretty(&report).expect("serialize report") + "\n",
        )
        .expect("write benchmark report");

        let coverage = build_telemetry_benchmark_coverage(&output)
            .expect("build telemetry coverage")
            .expect("telemetry coverage");
        assert_eq!(coverage.continuity_critical_total, 11);
        assert_eq!(coverage.continuity_critical_benchmarked, 0);
        assert_eq!(coverage.missing_loop_count, 11);
        assert!(
            coverage
                .gap_candidates
                .iter()
                .any(|gap| gap.id == "benchmark:unbenchmarked_continuity_feature")
        );

        fs::remove_dir_all(dir).expect("cleanup telemetry dir");
    }

    #[test]
    fn render_morning_operator_summary_surfaces_top_regressions() {
        let summary = render_morning_operator_summary(&MorningOperatorSummary {
            current_benchmark_score: 88,
            current_benchmark_max_score: 100,
            top_continuity_failures: vec!["resume continuity drift".to_string()],
            top_verification_regressions: vec![
                "verifier.feature.bundle.resume status=failing gate=fragile".to_string(),
            ],
            top_verification_pressure: vec![
                "verifier.feature.hive.messages-send-ack status=passing gate=acceptable target=acceptable continuity_critical=true".to_string(),
            ],
            top_drift_risks: vec!["surface drift in MEMD_MEMORY.md".to_string()],
            top_token_regressions: vec!["handoff packet +420 tokens".to_string()],
            top_no_memd_losses: vec!["resume still loses to no-memd baseline".to_string()],
            proposed_next_actions: vec!["fix resume journey before expanding registry".to_string()],
        });

        assert!(summary.contains("resume continuity drift"));
        assert!(summary.contains("Verification Regressions"));
        assert!(summary.contains("Verification Pressure"));
        assert!(summary.contains("verifier.feature.bundle.resume status=failing gate=fragile"));
        assert!(
            summary
                .contains("verifier.feature.hive.messages-send-ack status=passing gate=acceptable")
        );
        assert!(summary.contains("handoff packet +420 tokens"));
        assert!(summary.contains("fix resume journey before expanding registry"));
        assert!(summary.contains("# memd morning summary"));
    }

    #[test]
    fn build_morning_operator_summary_surfaces_acceptable_continuity_verifiers() {
        let registry = test_benchmark_registry();
        let benchmark = test_feature_benchmark_report(Path::new(".memd"));
        let verification_report = VerifySweepReport {
            lane: "nightly".to_string(),
            ok: true,
            total: 10,
            passed: 10,
            failures: Vec::new(),
            runs: vec![
                VerifierRunRecord {
                    verifier_id: "verifier.journey.resume-handoff-attach".to_string(),
                    status: "passing".to_string(),
                    gate_result: "acceptable".to_string(),
                    evidence_ids: vec![
                        "evidence:verifier.journey.resume-handoff-attach:latest".to_string(),
                    ],
                    metrics_observed: BTreeMap::new(),
                },
                VerifierRunRecord {
                    verifier_id: "verifier.feature.hive.messages-send-ack".to_string(),
                    status: "passing".to_string(),
                    gate_result: "acceptable".to_string(),
                    evidence_ids: vec![
                        "evidence:verifier.feature.hive.messages-send-ack:latest".to_string(),
                    ],
                    metrics_observed: BTreeMap::new(),
                },
            ],
            bundle_root: ".memd".to_string(),
            repo_root: None,
        };

        let summary = build_morning_operator_summary(
            &registry,
            &benchmark,
            None,
            None,
            Some(&verification_report),
        );

        assert!(summary.top_verification_regressions.iter().any(|item| {
            item.contains("verifier.journey.resume-handoff-attach")
                && item.contains("target=strong")
        }));
        assert!(
            !summary
                .top_verification_regressions
                .iter()
                .any(|item| item.contains("nightly verify lane nightly is green"))
        );
        let journey_index = summary
            .top_verification_regressions
            .iter()
            .position(|item| item.contains("verifier.journey.resume-handoff-attach"))
            .expect("journey verifier should be ranked");
        assert_eq!(journey_index, 0);
        assert!(
            summary
                .proposed_next_actions
                .iter()
                .any(|item| item.contains("upgrade verifier gates with highest target pressure"))
        );
        assert!(
            summary
                .proposed_next_actions
                .iter()
                .any(|item| item.contains("verifier.journey.resume-handoff-attach"))
        );
        assert!(
            !summary
                .proposed_next_actions
                .iter()
                .any(|item| item.contains("verifier.feature.hive.messages-send-ack"))
        );
        assert!(
            !summary
                .top_verification_regressions
                .iter()
                .any(|item| item.contains("verifier.feature.hive.messages-send-ack"))
        );
        assert!(
            summary
                .top_verification_pressure
                .iter()
                .any(|item| item.contains("verifier.feature.hive.messages-send-ack"))
        );
    }

    #[test]
    fn build_no_memd_delta_report_surfaces_token_and_reconstruction_improvement() {
        let report = build_no_memd_delta_report(
            &BaselineMetrics {
                prompt_tokens: 2200,
                reread_count: 5,
                reconstruction_steps: 4,
            },
            &BaselineMetrics {
                prompt_tokens: 1200,
                reread_count: 2,
                reconstruction_steps: 1,
            },
        );

        assert_eq!(report.token_delta, 1000);
        assert_eq!(report.reread_delta, 3);
        assert_eq!(report.reconstruction_delta, 3);
        assert!(report.with_memd_better);
    }

    #[test]
    fn continuity_failure_caps_gate_at_fragile() {
        let scorecard = resolve_benchmark_scorecard(
            &BenchmarkSubjectMetrics {
                correctness: 92,
                continuity: 35,
                reliability: 88,
                token_efficiency: 70,
                no_memd_delta: Some(12),
            },
            &BenchmarkEvidenceSummary {
                has_contract_evidence: true,
                has_workflow_evidence: true,
                has_continuity_evidence: false,
                has_comparative_evidence: true,
                has_drift_failure: false,
            },
            true,
        );

        assert_eq!(scorecard.gate, "fragile");
    }

    #[test]
    fn no_memd_loss_caps_feature_at_acceptable() {
        let scorecard = resolve_benchmark_scorecard(
            &BenchmarkSubjectMetrics {
                correctness: 95,
                continuity: 90,
                reliability: 90,
                token_efficiency: 65,
                no_memd_delta: Some(-4),
            },
            &BenchmarkEvidenceSummary {
                has_contract_evidence: true,
                has_workflow_evidence: true,
                has_continuity_evidence: true,
                has_comparative_evidence: true,
                has_drift_failure: false,
            },
            true,
        );

        assert_eq!(scorecard.gate, "acceptable");
    }

    #[test]
    fn write_continuity_journey_artifacts_writes_expected_outputs() {
        let dir = std::env::temp_dir().join(format!(
            "memd-continuity-journey-artifacts-{}",
            uuid::Uuid::new_v4()
        ));
        let output = dir.join(".memd");
        fs::create_dir_all(&output).expect("create output dir");
        let report = ContinuityJourneyReport {
            journey_id: "journey.continuity.resume-handoff-attach".to_string(),
            journey_name: "Resume To Handoff To Attach".to_string(),
            gate_decision: BenchmarkGateDecision {
                gate: "acceptable".to_string(),
                resolved_score: 75,
                reasons: vec!["continuity evidence present".to_string()],
            },
            metrics: BenchmarkSubjectMetrics {
                correctness: 90,
                continuity: 85,
                reliability: 80,
                token_efficiency: 78,
                no_memd_delta: Some(9),
            },
            evidence: BenchmarkEvidenceSummary {
                has_contract_evidence: true,
                has_workflow_evidence: true,
                has_continuity_evidence: true,
                has_comparative_evidence: true,
                has_drift_failure: false,
            },
            baseline_modes: vec![
                "baseline.no-memd".to_string(),
                "baseline.with-memd".to_string(),
            ],
            feature_ids: vec![
                "feature.bundle.resume".to_string(),
                "feature.bundle.handoff".to_string(),
            ],
            artifact_paths: Vec::new(),
            summary: "resume continuity evidence".to_string(),
            generated_at: Some(Utc::now()),
        };

        write_continuity_journey_artifacts(&output, &report)
            .expect("write continuity journey artifacts");
        let continuity_dir = benchmark_telemetry_dir(&output);
        assert!(continuity_dir.join("latest.json").exists());
        assert!(continuity_dir.join("latest.md").exists());

        let markdown =
            fs::read_to_string(continuity_dir.join("latest.md")).expect("read continuity markdown");
        assert!(markdown.contains("Resume To Handoff To Attach"));
        assert!(markdown.contains("Gate: `acceptable`"));

        fs::remove_dir_all(dir).expect("cleanup continuity journey artifacts dir");
    }
