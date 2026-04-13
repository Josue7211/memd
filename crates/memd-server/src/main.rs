mod atlas;
mod helpers;
mod inspection;
mod keys;
mod procedural;
mod repair;
mod routes;
mod routing;
mod store;
mod store_entities;
mod store_hive;
mod store_hive_lifecycle;
mod store_migrations;
mod store_runtime_maintenance;
mod store_skill_policy;
mod ui;
mod working;

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;

pub(crate) use helpers::*;
pub(crate) use routes::*;
pub(crate) use store::{DuplicateMatch, RecordEventArgs, SqliteStore};

use std::collections::{HashSet, VecDeque};

use anyhow::Context;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::Html,
    routing::{get, post},
};
use chrono::Utc;
pub(crate) use keys::{apply_lifecycle, canonical_key, redundancy_key, validate_source_quality};
use memd_schema::{
    AtlasExpandRequest, AtlasExpandResponse, AtlasExploreRequest, AtlasExploreResponse,
    AtlasListTrailsRequest, AtlasListTrailsResponse, AtlasRegionsRequest, AtlasRegionsResponse,
    AtlasRenameRegionRequest, AtlasRenameRegionResponse, AtlasSaveTrailRequest,
    AtlasSaveTrailResponse,
    AgentProfileRequest, AgentProfileResponse, AgentProfileUpsertRequest, AssociativeRecallHit,
    AssociativeRecallRequest, AssociativeRecallResponse, CandidateMemoryRequest,
    CandidateMemoryResponse, CompactContextResponse, CompactMemoryRecord, ContextRequest,
    ContextResponse, EntityLinkRequest, EntityLinkResponse, EntityLinksRequest,
    EntityLinksResponse, EntityMemoryRequest, EntityMemoryResponse, EntitySearchHit,
    EntitySearchRequest, EntitySearchResponse, ExpireMemoryRequest, ExpireMemoryResponse,
    ExplainMemoryRequest, ExplainMemoryResponse, HealthResponse, HiveBoardRequest,
    HiveBoardResponse, HiveClaimAcquireRequest, HiveClaimRecoverRequest, HiveClaimReleaseRequest,
    HiveClaimTransferRequest, HiveClaimsRequest, HiveClaimsResponse, HiveCoordinationInboxRequest,
    HiveCoordinationInboxResponse, HiveCoordinationReceiptRequest, HiveCoordinationReceiptsRequest,
    HiveCoordinationReceiptsResponse, HiveFollowRequest, HiveFollowResponse, HiveMessageAckRequest,
    HiveMessageInboxRequest, HiveMessageSendRequest, HiveMessagesResponse, HiveQueenActionRequest,
    HiveQueenActionResponse, HiveRosterRequest, HiveRosterResponse, HiveSessionAutoRetireRequest,
    HiveSessionAutoRetireResponse, HiveSessionRetireRequest, HiveSessionRetireResponse,
    HiveSessionUpsertRequest, HiveSessionsRequest, HiveSessionsResponse, HiveTaskAssignRequest,
    HiveTaskUpsertRequest, HiveTasksRequest, HiveTasksResponse, InboxMemoryItem, MaintainReport,
    MaintainReportRequest, MemoryConsolidationRequest, MemoryConsolidationResponse,
    MemoryContextFrame, MemoryDecayRequest, MemoryDecayResponse, MemoryDrainRequest,
    MemoryDrainResponse, InboxDismissRequest, InboxDismissResponse, MemoryEntityLinkRecord,
    MemoryEntityRecord, MemoryEventRecord, MemoryInboxRequest, MemoryInboxResponse, MemoryItem,
    MemoryKind, MemoryMaintenanceReportRequest, MemoryMaintenanceReportResponse,
    MemoryPolicyResponse, MemoryScope, MemoryStage, MemoryStatus, MemoryVisibility,
    PromoteMemoryRequest, PromoteMemoryResponse, RepairMemoryRequest, RepairMemoryResponse,
    RetrievalIntent, RetrievalRoute, SearchMemoryRequest, SearchMemoryResponse,
    SkillPolicyActivationEntriesRequest, SkillPolicyActivationEntriesResponse,
    SkillPolicyApplyReceiptsRequest, SkillPolicyApplyReceiptsResponse, SkillPolicyApplyRequest,
    SkillPolicyApplyResponse, SourceMemoryRequest, SourceMemoryResponse, SourceQuality,
    StoreMemoryRequest, StoreMemoryResponse, TimelineMemoryRequest, TimelineMemoryResponse,
    VerifyMemoryRequest, VerifyMemoryResponse, VisibleMemoryArtifactDetailResponse,
    ProcedureDetectRequest, ProcedureDetectResponse, ProcedureListRequest, ProcedureListResponse,
    ProcedureMatchRequest, ProcedureMatchResponse, ProcedurePromoteRequest,
    ProcedurePromoteResponse, ProcedureRecordRequest, ProcedureRecordResponse,
    ProcedureRetireRequest, ProcedureRetireResponse, ProcedureUseRequest, ProcedureUseResponse,
    VisibleMemorySnapshotResponse, VisibleMemoryUiActionRequest, VisibleMemoryUiActionResponse,
    WorkingMemoryRequest, WorkingMemoryResponse, WorkspaceMemoryRequest, WorkspaceMemoryResponse,
};
pub(crate) use routing::RetrievalPlan;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    store: SqliteStore,
}

impl AppState {
    fn store_item(
        &self,
        req: StoreMemoryRequest,
        stage: MemoryStage,
    ) -> anyhow::Result<(MemoryItem, Option<DuplicateMatch>)> {
        validate_source_quality(req.source_quality)?;
        let now = Utc::now();
        let item = MemoryItem {
            id: Uuid::new_v4(),
            content: req.content.trim().to_string(),
            redundancy_key: None,
            belief_branch: req.belief_branch,
            preferred: false,
            kind: req.kind,
            scope: req.scope,
            project: req.project,
            namespace: req.namespace,
            workspace: req.workspace,
            visibility: req.visibility.unwrap_or_default(),
            source_agent: req.source_agent,
            source_system: req.source_system,
            source_path: req.source_path,
            confidence: req.confidence.unwrap_or(0.7),
            ttl_seconds: req.ttl_seconds,
            created_at: now,
            updated_at: now,
            last_verified_at: req.last_verified_at,
            supersedes: req.supersedes,
            tags: req.tags,
            status: req.status.unwrap_or(MemoryStatus::Active),
            source_quality: req.source_quality.or(Some(SourceQuality::Canonical)),
            stage,
        };

        let canonical_key = canonical_key(&item);
        let redundancy_key = redundancy_key(&item);
        let item = MemoryItem {
            redundancy_key: Some(redundancy_key.clone()),
            ..item
        };
        let entity = self.store.resolve_entity_for_item(&item, &canonical_key)?;
        let duplicate =
            self.store
                .insert_or_get_duplicate(&item, &canonical_key, &redundancy_key)?;
        if let Some(found) = duplicate.as_ref()
            && let Some(revived) = self.revive_duplicate_on_explicit_store(
                found,
                &item,
                &canonical_key,
                &redundancy_key,
            )? {
                if let Err(e) = self.record_item_event(
                    &revived,
                    "restored",
                    "duplicate memory item restored to active canonical state".to_string(),
                ) {
                    eprintln!("warn: record_item_event (restored): {e:#}");
                }
                return Ok((revived, None));
            }
        if duplicate.is_none() {
            if let Err(e) = self.record_item_event(
                &item,
                event_type_for_stage(stage),
                format!(
                    "{} memory item stored",
                    match stage {
                        MemoryStage::Candidate => "candidate",
                        MemoryStage::Canonical => "canonical",
                    }
                ),
            ) {
                eprintln!("warn: record_item_event (stored): {e:#}");
            }

            // Auto-expire excess status items to prevent noise accumulation
            if item.kind == MemoryKind::Status {
                if let Err(e) = self.expire_excess_status_items(&item, 4) {
                    eprintln!("warn: expire_excess_status_items: {e:#}");
                }
            }

            // Auto-link co-occurring entities within the same project
            if item.kind != MemoryKind::Status {
                if let Err(e) = self.auto_link_entity(&entity.record, &item) {
                    eprintln!("warn: auto_link_entity: {e:#}");
                }
            }
        }
        Ok((item, duplicate))
    }

    fn revive_duplicate_on_explicit_store(
        &self,
        duplicate: &DuplicateMatch,
        incoming: &MemoryItem,
        canonical_key: &str,
        redundancy_key: &str,
    ) -> anyhow::Result<Option<MemoryItem>> {
        if incoming.stage != MemoryStage::Canonical || incoming.status != MemoryStatus::Active {
            return Ok(None);
        }
        if duplicate.item.status == MemoryStatus::Active {
            return Ok(None);
        }

        let mut revived = duplicate.item.clone();
        revived.content = incoming.content.clone();
        revived.belief_branch = incoming.belief_branch.clone();
        revived.project = incoming.project.clone();
        revived.namespace = incoming.namespace.clone();
        revived.workspace = incoming.workspace.clone();
        revived.visibility = incoming.visibility;
        revived.source_agent = incoming.source_agent.clone();
        revived.source_system = incoming.source_system.clone();
        revived.source_path = incoming.source_path.clone();
        revived.source_quality = incoming.source_quality;
        revived.confidence = incoming.confidence;
        revived.ttl_seconds = incoming.ttl_seconds;
        revived.last_verified_at = incoming.last_verified_at;
        revived.tags = incoming.tags.clone();
        revived.status = MemoryStatus::Active;
        revived.updated_at = Utc::now();
        revived.supersedes.extend(incoming.supersedes.clone());
        revived.supersedes.retain(|id| *id != revived.id);
        revived.supersedes.sort_unstable();
        revived.supersedes.dedup();
        let revived = MemoryItem {
            redundancy_key: Some(redundancy_key.to_string()),
            ..revived
        };
        self.store.update(&revived, canonical_key, redundancy_key)?;
        Ok(Some(revived))
    }

    fn auto_link_entity(
        &self,
        new_entity: &MemoryEntityRecord,
        item: &MemoryItem,
    ) -> anyhow::Result<()> {
        let Some(project) = &item.project else {
            return Ok(());
        };
        let entities = self.store.list_entities()?;
        let candidates: Vec<&MemoryEntityRecord> = entities
            .iter()
            .filter(|e| e.id != new_entity.id)
            .filter(|e| {
                e.context
                    .as_ref()
                    .and_then(|ctx| ctx.project.as_deref())
                    == Some(project.as_str())
            })
            .filter(|e| e.salience_score > 0.1)
            .take(3)
            .collect();

        for candidate in candidates {
            let existing_links = self.store.links_for_entity(&EntityLinksRequest {
                entity_id: new_entity.id,
            })?;
            let already_linked = existing_links.iter().any(|link| {
                link.from_entity_id == candidate.id || link.to_entity_id == candidate.id
            });
            if already_linked {
                continue;
            }
            let link = MemoryEntityLinkRecord {
                id: Uuid::new_v4(),
                from_entity_id: new_entity.id,
                to_entity_id: candidate.id,
                relation_kind: memd_schema::EntityRelationKind::Related,
                confidence: 0.5,
                created_at: Utc::now(),
                note: Some("auto-linked by co-occurrence".to_string()),
                context: None,
                tags: vec!["auto".to_string()],
            };
            self.store.upsert_entity_link(&link)?;
        }
        Ok(())
    }

    fn expire_excess_status_items(
        &self,
        new_item: &MemoryItem,
        max_keep: usize,
    ) -> anyhow::Result<()> {
        let all = self.store.list()?;
        let mut status_items: Vec<MemoryItem> = all
            .into_iter()
            .filter(|item| {
                item.kind == MemoryKind::Status
                    && item.status == MemoryStatus::Active
                    && item.project == new_item.project
                    && item.source_agent == new_item.source_agent
                    && item.id != new_item.id
            })
            .collect();
        if status_items.len() < max_keep {
            return Ok(());
        }
        // Sort oldest first
        status_items.sort_by(|a, b| a.updated_at.cmp(&b.updated_at));
        let expire_count = status_items.len() - max_keep + 1;
        for item in status_items.into_iter().take(expire_count) {
            let mut expired = item;
            expired.status = MemoryStatus::Expired;
            expired.updated_at = Utc::now();
            let ck = canonical_key(&expired);
            let rk = redundancy_key(&expired);
            self.store.update(&expired, &ck, &rk)?;
        }
        Ok(())
    }

    fn snapshot(&self) -> anyhow::Result<Vec<MemoryItem>> {
        let items = self.store.list()?;
        let mut hydrated = Vec::with_capacity(items.len());
        for item in items {
            let (item, changed) = apply_lifecycle(item);
            if changed {
                let canonical_key = canonical_key(&item);
                let redundancy_key = redundancy_key(&item);
                self.store.update(&item, &canonical_key, &redundancy_key)?;
            }
            hydrated.push(item);
        }
        Ok(hydrated)
    }

    fn promote_item(
        &self,
        req: PromoteMemoryRequest,
    ) -> anyhow::Result<(MemoryItem, Option<DuplicateMatch>)> {
        let mut item = self
            .store
            .get(req.id)?
            .ok_or_else(|| anyhow::anyhow!("memory item not found"))?;

        item.scope = req.scope.unwrap_or(item.scope);
        item.project = req.project.or(item.project);
        item.namespace = req.namespace.or(item.namespace);
        item.workspace = req.workspace.or(item.workspace);
        item.visibility = req.visibility.unwrap_or(item.visibility);
        item.belief_branch = req.belief_branch.or(item.belief_branch);
        item.confidence = req.confidence.unwrap_or(item.confidence);
        item.ttl_seconds = req.ttl_seconds.or(item.ttl_seconds);
        if let Some(tags) = req.tags {
            item.tags = tags;
        }
        item.status = req.status.unwrap_or(MemoryStatus::Active);
        item.stage = MemoryStage::Canonical;
        item.updated_at = Utc::now();

        let canonical_key = canonical_key(&item);
        let redundancy_key_value = redundancy_key(&item);
        if let Some(duplicate) = self.store.find_duplicate(
            &redundancy_key_value,
            &canonical_key,
            &item.stage,
            item.id,
        )? {
            return Ok((item, Some(duplicate)));
        }

        let item = MemoryItem {
            redundancy_key: Some(redundancy_key_value),
            ..item
        };
        let redundancy_key_value = redundancy_key(&item);
        self.store
            .update(&item, &canonical_key, &redundancy_key_value)?;
        if let Err(e) = self.record_item_event(
            &item,
            "promoted",
            "memory item promoted to canonical stage".to_string(),
        ) {
            eprintln!("warn: record_item_event (promoted): {e:#}");
        }
        Ok((item, None))
    }

    fn record_item_event(
        &self,
        item: &MemoryItem,
        event_type: &str,
        summary: String,
    ) -> anyhow::Result<MemoryEventRecord> {
        let canonical_key = canonical_key(item);
        let entity = self.store.resolve_entity_for_item(item, &canonical_key)?;
        let context = Some(entity_context_frame(&entity.record, item));
        self.store.record_event(
            &entity.record,
            item.id,
            RecordEventArgs {
                event_type: event_type.to_string(),
                summary,
                occurred_at: item.updated_at,
                project: item.project.clone(),
                namespace: item.namespace.clone(),
                workspace: item.workspace.clone(),
                source_agent: item.source_agent.clone(),
                source_system: item.source_system.clone(),
                source_path: item.source_path.clone(),
                related_entity_ids: Vec::new(),
                tags: item.tags.clone(),
                context,
                confidence: item.confidence,
                salience_score: entity.record.salience_score,
            },
        )
    }
}

#[tokio::main]
async fn main() {
    let db_path = std::env::var("MEMD_DB_PATH").unwrap_or_else(|_| ".memd/memd.db".to_string());
    let bind_addr =
        std::env::var("MEMD_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8787".to_string());
    let store = match SqliteStore::open(&db_path) {
        Ok(store) => store,
        Err(e) => {
            eprintln!("error: failed to open database at {db_path}: {e:#}");
            std::process::exit(1);
        }
    };
    let state = AppState { store };
    let app = Router::new()
        .route("/", get(dashboard))
        .route("/ui/snapshot", get(get_visible_memory_snapshot))
        .route("/ui/artifact", get(get_visible_memory_artifact))
        .route("/ui/action", post(post_visible_memory_action))
        .route("/healthz", get(healthz))
        .route("/memory/store", post(store_memory))
        .route("/memory/candidates", post(store_candidate))
        .route("/memory/promote", post(promote_memory))
        .route("/memory/expire", post(expire_memory))
        .route("/memory/verify", post(verify_memory))
        .route("/memory/repair", post(repair_memory))
        .route("/memory/search", post(search_memory))
        .route("/memory/context", get(get_context))
        .route("/memory/context/compact", get(get_compact_context))
        .route("/memory/working", get(get_working_memory))
        .route("/memory/inbox", get(get_inbox))
        .route("/memory/entity", get(get_entity))
        .route("/memory/entity/search", get(get_entity_search))
        .route("/memory/entity/link", post(post_entity_link))
        .route("/memory/entity/links", get(get_entity_links))
        .route("/memory/entity/recall", get(get_entity_recall))
        .route("/memory/timeline", get(get_timeline))
        .route(
            "/memory/profile",
            get(get_agent_profile).post(post_agent_profile),
        )
        .route("/memory/source", get(get_source_memory))
        .route("/memory/workspaces", get(get_workspace_memory))
        .route("/memory/explain", get(get_explain))
        .route("/coordination/messages/send", post(post_hive_message))
        .route("/coordination/messages/inbox", get(get_hive_inbox))
        .route("/coordination/messages/ack", post(post_hive_ack))
        .route("/coordination/inbox", get(get_hive_coordination_inbox))
        .route(
            "/coordination/receipts/record",
            post(post_hive_coordination_receipt),
        )
        .route(
            "/coordination/receipts",
            get(get_hive_coordination_receipts),
        )
        .route(
            "/coordination/skill-policy/apply",
            post(post_skill_policy_apply_receipt).get(get_skill_policy_apply_receipts),
        )
        .route(
            "/coordination/skill-policy/activations",
            get(get_skill_policy_activations),
        )
        .route(
            "/coordination/claims/acquire",
            post(post_hive_claim_acquire),
        )
        .route(
            "/coordination/claims/release",
            post(post_hive_claim_release),
        )
        .route(
            "/coordination/claims/transfer",
            post(post_hive_claim_transfer),
        )
        .route(
            "/coordination/claims/recover",
            post(post_hive_claim_recover),
        )
        .route("/coordination/claims", get(get_hive_claims))
        .route(
            "/coordination/sessions/upsert",
            post(post_hive_session_upsert),
        )
        .route(
            "/coordination/sessions/retire",
            post(post_hive_session_retire),
        )
        .route(
            "/coordination/sessions/auto-retire",
            post(post_hive_session_auto_retire),
        )
        .route("/coordination/sessions", get(get_hive_sessions))
        .route("/hive/board", get(get_hive_board))
        .route("/hive/roster", get(get_hive_roster))
        .route("/hive/follow", get(get_hive_follow))
        .route("/hive/queen/deny", post(post_hive_queen_deny))
        .route("/hive/queen/reroute", post(post_hive_queen_reroute))
        .route("/hive/queen/handoff", post(post_hive_queen_handoff))
        .route("/coordination/tasks/upsert", post(post_hive_task_upsert))
        .route("/coordination/tasks/assign", post(post_hive_task_assign))
        .route("/coordination/tasks", get(get_hive_tasks))
        .route("/memory/maintenance/decay", post(decay_memory))
        .route("/memory/maintenance/consolidate", post(consolidate_memory))
        .route("/memory/maintenance/drain", post(drain_memory))
        .route("/memory/maintenance/report", get(get_maintenance_report))
        .route("/memory/inbox/dismiss", post(dismiss_inbox))
        .route("/runtime/maintain", post(post_runtime_maintain))
        .route("/memory/policy", get(get_memory_policy))
        .route("/atlas/regions", get(get_atlas_regions))
        .route("/atlas/explore", post(post_atlas_explore))
        .route("/atlas/expand", post(post_atlas_expand))
        .route("/atlas/rename", post(post_atlas_rename))
        .route("/atlas/trails", get(get_atlas_trails))
        .route("/atlas/trails/save", post(post_atlas_trail_save))
        .route("/atlas/generate", post(post_atlas_generate))
        .route("/procedures", get(get_procedures))
        .route("/procedures/record", post(post_procedure_record))
        .route("/procedures/match", post(post_procedure_match))
        .route("/procedures/promote", post(post_procedure_promote))
        .route("/procedures/use", post(post_procedure_use))
        .route("/procedures/retire", post(post_procedure_retire))
        .route("/procedures/detect", post(post_procedure_detect))
        .with_state(state);

    let listener = match tokio::net::TcpListener::bind(&bind_addr).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("error: failed to bind to {bind_addr}: {e:#}");
            eprintln!("hint: port may be in use, set MEMD_BIND_ADDR to change");
            std::process::exit(1);
        }
    };
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("error: server exited unexpectedly: {e:#}");
        std::process::exit(1);
    }
}
