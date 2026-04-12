use super::*;

#[derive(Clone, Default)]
pub(crate) struct MockHiveState {
    pub(crate) messages: Arc<Mutex<Vec<HiveMessageRecord>>>,
    pub(crate) claims: Arc<Mutex<Vec<HiveClaimRecord>>>,
    pub(crate) receipts: Arc<Mutex<Vec<HiveCoordinationReceiptRecord>>>,
    pub(crate) skill_policy_receipts: Arc<Mutex<Vec<SkillPolicyApplyReceipt>>>,
    pub(crate) skill_policy_activations: Arc<Mutex<Vec<memd_schema::SkillPolicyActivationEntry>>>,
}

#[derive(Clone, Default)]
pub(crate) struct MockRuntimeState {
    pub(crate) stored: Arc<Mutex<Vec<memd_schema::StoreMemoryRequest>>>,
    pub(crate) candidates: Arc<Mutex<Vec<memd_schema::CandidateMemoryRequest>>>,
    pub(crate) repaired: Arc<Mutex<Vec<memd_schema::RepairMemoryRequest>>>,
    pub(crate) session_upserts: Arc<Mutex<Vec<memd_schema::HiveSessionUpsertRequest>>>,
    pub(crate) session_retires: Arc<Mutex<Vec<memd_schema::HiveSessionRetireRequest>>>,
    pub(crate) session_records: Arc<Mutex<Vec<memd_schema::HiveSessionRecord>>>,
    pub(crate) messages: Arc<Mutex<Vec<HiveMessageRecord>>>,
    pub(crate) claims: Arc<Mutex<Vec<HiveClaimRecord>>>,
    pub(crate) receipts: Arc<Mutex<Vec<HiveCoordinationReceiptRecord>>>,
    pub(crate) task_records: Arc<Mutex<Vec<HiveTaskRecord>>>,
    pub(crate) search_count: Arc<Mutex<usize>>,
    pub(crate) search_requests: Arc<Mutex<Vec<memd_schema::SearchMemoryRequest>>>,
    pub(crate) source_requests: Arc<Mutex<Vec<memd_schema::SourceMemoryRequest>>>,
    pub(crate) context_compact_response: Arc<Mutex<Option<memd_schema::CompactContextResponse>>>,
    pub(crate) working_response: Arc<Mutex<Option<memd_schema::WorkingMemoryResponse>>>,
}

pub(crate) async fn mock_send_hive_message(
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

pub(crate) async fn mock_hive_inbox(
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
                && (req.include_acknowledged.unwrap_or(false) || message.acknowledged_at.is_none())
        })
        .cloned()
        .collect();
    Json(HiveMessagesResponse { messages })
}

pub(crate) async fn mock_hive_ack(
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

pub(crate) async fn mock_claim_acquire(
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

pub(crate) async fn mock_claim_release(
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

pub(crate) async fn mock_claim_transfer(
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

pub(crate) async fn mock_claims(
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

pub(crate) async fn mock_record_receipt(
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

pub(crate) async fn mock_runtime_record_receipt(
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

pub(crate) async fn mock_runtime_send_hive_message(
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

pub(crate) async fn mock_runtime_hive_inbox(
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
                && (req.include_acknowledged.unwrap_or(false) || message.acknowledged_at.is_none())
        })
        .cloned()
        .collect::<Vec<_>>();
    Json(HiveMessagesResponse { messages })
}

pub(crate) async fn mock_runtime_hive_ack(
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

pub(crate) async fn mock_runtime_receipts(
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

pub(crate) async fn mock_runtime_claims(
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

pub(crate) async fn mock_runtime_claim_acquire(
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

pub(crate) async fn mock_runtime_claim_release(
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

pub(crate) async fn mock_runtime_claim_transfer(
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

pub(crate) async fn mock_runtime_task_upsert(
    State(state): State<MockRuntimeState>,
    Json(req): Json<memd_schema::HiveTaskUpsertRequest>,
) -> Json<memd_schema::HiveTasksResponse> {
    let mut tasks = state
        .task_records
        .lock()
        .expect("lock runtime task records");
    let now = Utc::now();
    let task = if let Some(existing) = tasks.iter_mut().find(|task| task.task_id == req.task_id) {
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

pub(crate) async fn mock_runtime_task_assign(
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

pub(crate) async fn mock_record_skill_policy_apply(
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

pub(crate) async fn mock_skill_policy_apply_receipts(
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

pub(crate) async fn mock_context_compact(
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

pub(crate) async fn mock_working_memory(
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

pub(crate) async fn mock_inbox() -> Json<memd_schema::MemoryInboxResponse> {
    Json(memd_schema::MemoryInboxResponse {
        route: memd_schema::RetrievalRoute::Auto,
        intent: memd_schema::RetrievalIntent::General,
        items: Vec::new(),
    })
}

pub(crate) async fn mock_maintenance_report(
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

pub(crate) async fn mock_hive_tasks(
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

pub(crate) async fn mock_hive_coordination_inbox(
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

pub(crate) async fn mock_runtime_maintain(
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

pub(crate) async fn mock_skill_policy_activations(
    State(state): State<MockHiveState>,
    Query(req): Query<SkillPolicyActivationEntriesRequest>,
) -> Json<SkillPolicyActivationEntriesResponse> {
    let activations = state
        .skill_policy_activations
        .lock()
        .expect("lock skill policy activations")
        .iter()
        .filter(|activation| {
            req.project
                .as_ref()
                .is_none_or(|project| activation.project.as_ref() == Some(project))
                && req
                    .namespace
                    .as_ref()
                    .is_none_or(|namespace| activation.namespace.as_ref() == Some(namespace))
                && req
                    .workspace
                    .as_ref()
                    .is_none_or(|workspace| activation.workspace.as_ref() == Some(workspace))
        })
        .cloned()
        .collect();
    Json(SkillPolicyActivationEntriesResponse { activations })
}

pub(crate) async fn mock_workspace_memory() -> Json<memd_schema::WorkspaceMemoryResponse> {
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

pub(crate) async fn mock_source_memory(
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

pub(crate) async fn mock_search_memory(
    State(state): State<MockRuntimeState>,
    Json(req): Json<memd_schema::SearchMemoryRequest>,
) -> Json<memd_schema::SearchMemoryResponse> {
    *state.search_count.lock().expect("lock search count") += 1;
    state
        .search_requests
        .lock()
        .expect("lock search requests")
        .push(req.clone());
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

pub(crate) async fn mock_store_memory(
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

pub(crate) async fn mock_candidate_memory(
    State(state): State<MockRuntimeState>,
    Json(req): Json<memd_schema::CandidateMemoryRequest>,
) -> Json<memd_schema::CandidateMemoryResponse> {
    state
        .candidates
        .lock()
        .expect("lock candidates")
        .push(req.clone());
    Json(memd_schema::CandidateMemoryResponse {
        item: memd_schema::MemoryItem {
            id: uuid::Uuid::new_v4(),
            content: req.content,
            redundancy_key: Some("candidate".to_string()),
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
            status: memd_schema::MemoryStatus::Active,
            stage: memd_schema::MemoryStage::Candidate,
        },
        duplicate_of: None,
    })
}

pub(crate) async fn mock_repair_memory(
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

pub(crate) async fn mock_hive_session_upsert(
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
        ..memd_schema::HiveSessionRecord::default()
    };
    {
        let mut records = state.session_records.lock().expect("lock session records");
        records.push(record.clone());
    }
    Json(memd_schema::HiveSessionsResponse {
        sessions: vec![record],
    })
}

pub(crate) async fn mock_hive_sessions(
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

pub(crate) async fn mock_hive_board(
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

pub(crate) async fn mock_hive_roster(
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

pub(crate) async fn mock_hive_follow(
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

pub(crate) async fn mock_hive_session_retire(
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

pub(crate) async fn mock_hive_session_auto_retire(
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

pub(crate) async fn mock_healthz() -> Json<memd_schema::HealthResponse> {
    Json(memd_schema::HealthResponse {
        status: "ok".to_string(),
        items: 1,
    })
}

pub(crate) async fn mock_slow_hive_session_upsert(
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

pub(crate) async fn spawn_mock_memory_server() -> String {
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

pub(crate) fn spawn_blocking_mock_sidecar_server() -> String {
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

pub(crate) async fn spawn_mock_runtime_server(
    state: MockRuntimeState,
    slow_hive_upsert: bool,
) -> String {
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
        .route("/memory/candidates", post(mock_candidate_memory))
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

pub(crate) async fn spawn_mock_hive_server() -> String {
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

#[allow(dead_code)]
pub(crate) fn push_mock_runtime_hive_session(
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
            ..memd_schema::HiveSessionRecord::default()
        });
}
