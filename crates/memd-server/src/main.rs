mod atlas;
mod backup;
mod decay_calibration;
mod embed;
mod episodes;
mod errors;
mod helpers;
mod inspection;
mod keys;
mod latency;
mod procedural;
mod query_sanitize;
mod rag_bridge;
mod rate_limit;
mod repair;
mod routes;
mod routing;
mod status;
mod store;
mod store_dedup;
mod store_entities;
mod store_episodes;
mod store_hive;
mod store_hive_lifecycle;
mod store_ingestion;
mod store_migrations;
mod store_runtime_maintenance;
mod store_skill_policy;
mod token_headers;
mod ui;
mod working;

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;

pub(crate) use helpers::*;
pub(crate) use rag_bridge::*;
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
use memd_rag::RagClient;
use memd_schema::{
    AgentProfileRequest, AgentProfileResponse, AgentProfileUpsertRequest, AssociativeRecallHit,
    AssociativeRecallRequest, AssociativeRecallResponse, AtlasExpandRequest, AtlasExpandResponse,
    AtlasExploreRequest, AtlasExploreResponse, AtlasListTrailsRequest, AtlasListTrailsResponse,
    AtlasRegionsRequest, AtlasRegionsResponse, AtlasRenameRegionRequest, AtlasRenameRegionResponse,
    AtlasSaveTrailRequest, AtlasSaveTrailResponse, CandidateMemoryRequest, CandidateMemoryResponse,
    CompactContextResponse, CompactMemoryRecord, ConsolidateEpisodesRequest,
    ConsolidateEpisodesResponse, ContextRequest, ContextResponse, CorrectMemoryRequest,
    CorrectMemoryResponse, DecayDiagnosticsResponse, DedupScanRequest, DedupScanResponse,
    DivergenceRequest, DivergenceSummary, EntityLinkRequest, EntityLinkResponse,
    EntityLinksRequest, EntityLinksResponse, EntityMemoryRequest, EntityMemoryResponse,
    EntitySearchHit, EntitySearchRequest, EntitySearchResponse, ExpireMemoryRequest,
    ExpireMemoryResponse, ExplainMemoryRequest, ExplainMemoryResponse, HealthResponse,
    HiveBoardRequest, HiveBoardResponse, HiveClaimAcquireRequest, HiveClaimRecoverRequest,
    HiveClaimReleaseRequest, HiveClaimTransferRequest, HiveClaimsRequest, HiveClaimsResponse,
    HiveCoordinationInboxRequest, HiveCoordinationInboxResponse, HiveCoordinationReceiptRequest,
    HiveCoordinationReceiptsRequest, HiveCoordinationReceiptsResponse, HiveFollowRequest,
    HiveFollowResponse, HiveMessageAckRequest, HiveMessageInboxRequest, HiveMessageSendRequest,
    HiveMessagesResponse, HiveQueenActionRequest, HiveQueenActionResponse, HiveRosterRequest,
    HiveRosterResponse, HiveSessionAutoRetireRequest, HiveSessionAutoRetireResponse,
    HiveSessionRetireRequest, HiveSessionRetireResponse, HiveSessionUpsertRequest,
    HiveSessionsRequest, HiveSessionsResponse, HiveTaskAssignRequest, HiveTaskUpsertRequest,
    HiveTasksRequest, HiveTasksResponse, InboxDismissRequest, InboxDismissResponse,
    InboxMemoryItem, IngestLanesRequest, IngestLanesResponse, ListEpisodesRequest,
    ListEpisodesResponse, MaintainReport, MaintainReportRequest, MemoryConsolidationRequest,
    MemoryConsolidationResponse, MemoryContextFrame, MemoryDecayRequest, MemoryDecayResponse,
    MemoryDrainRequest, MemoryDrainResponse, MemoryEntityLinkRecord, MemoryEntityRecord,
    MemoryEventRecord, MemoryInboxRequest, MemoryInboxResponse, MemoryItem, MemoryKind,
    MemoryMaintenanceReportRequest, MemoryMaintenanceReportResponse, MemoryPolicyResponse,
    MemoryScope, MemoryStage, MemoryStatus, MemoryVisibility, ProcedureDetectRequest,
    ProcedureDetectResponse, ProcedureListRequest, ProcedureListResponse, ProcedureMatchRequest,
    ProcedureMatchResponse, ProcedurePromoteRequest, ProcedurePromoteResponse,
    ProcedureRecordRequest, ProcedureRecordResponse, ProcedureRetireRequest,
    ProcedureRetireResponse, ProcedureUseRequest, ProcedureUseResponse, PromoteMemoryRequest,
    PromoteMemoryResponse, RepairMemoryRequest, RepairMemoryResponse, RetrievalIntent,
    RetrievalRoute, SearchMemoryRequest, SearchMemoryResponse, SkillPolicyActivationEntriesRequest,
    SkillPolicyActivationEntriesResponse, SkillPolicyApplyReceiptsRequest,
    SkillPolicyApplyReceiptsResponse, SkillPolicyApplyRequest, SkillPolicyApplyResponse,
    SourceMemoryRequest, SourceMemoryResponse, SourceQuality, StoreMemoryRequest,
    StoreMemoryResponse, TimelineMemoryRequest, TimelineMemoryResponse, VerifyMemoryRequest,
    VerifyMemoryResponse, VisibleMemoryArtifactDetailResponse, VisibleMemorySnapshotResponse,
    VisibleMemoryUiActionRequest, VisibleMemoryUiActionResponse, WorkingMemoryRequest,
    WorkingMemoryResponse, WorkspaceMemoryRequest, WorkspaceMemoryResponse,
};
pub(crate) use routing::RetrievalPlan;
use serde::Deserialize;
use tower_http::trace::TraceLayer;
use tracing::{error, warn};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    store: SqliteStore,
    latency: std::sync::Arc<latency::LatencyHistogram>,
    rate_limiter: std::sync::Arc<rate_limit::RateLimiter>,
    rag: Option<std::sync::Arc<RagClient>>,
    embedder: Option<std::sync::Arc<embed::Embedder>>,
}

// B3-Part2-prereq: kill-switch for quadratic entity auto-link on the store hot path.
// When set, `auto_link_entity` and `create_wiki_links` are skipped in `store_item`.
// Motivation: both run `list_entities()` (full table scan + JSON deserialize per row),
// which stalls bulk ingest sweeps (e.g. LongMemEval ~26.5k stores) at ~100 items.
// Bench opts in; product keeps link graph by default.
fn store_auto_link_disabled() -> bool {
    match std::env::var("MEMD_STORE_AUTO_LINK_DISABLED") {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "on" | "yes")
        }
        Err(_) => false,
    }
}

impl AppState {
    fn store_item(
        &self,
        req: StoreMemoryRequest,
        stage: MemoryStage,
    ) -> anyhow::Result<(MemoryItem, Option<DuplicateMatch>)> {
        validate_source_quality(req.source_quality)?;
        let now = Utc::now();
        let lane = req
            .lane
            .or_else(|| detect_content_lane(&req.content, req.source_path.as_deref(), &req.tags));
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
            lane,
            version: 1,
            correction_meta: None,
        };

        let canonical_key = canonical_key(&item);
        let redundancy_key = redundancy_key(&item);
        let item = MemoryItem {
            redundancy_key: Some(redundancy_key.clone()),
            ..item
        };

        // E3-D1: storage-time near-duplicate guard. Gated on
        // MEMD_STORE_DEDUP=1; requires a live embedder in scope.
        if store_dedup::store_dedup_enabled()
            && let Some(existing) = self.find_near_duplicate_for_item(&item)?
        {
            let mut reinforced = existing.clone();
            reinforced.updated_at = Utc::now();
            reinforced.confidence = (reinforced.confidence + 0.05).min(1.0);
            for tag in &item.tags {
                if !reinforced.tags.iter().any(|t| t == tag) {
                    reinforced.tags.push(tag.clone());
                }
            }
            let existing_ck = crate::keys::canonical_key(&reinforced);
            let existing_rk = reinforced
                .redundancy_key
                .clone()
                .unwrap_or_else(|| crate::keys::redundancy_key(&reinforced));
            self.store.update(&reinforced, &existing_ck, &existing_rk)?;
            if let Err(e) = self.record_item_event(
                &reinforced,
                "reinforced",
                "near-duplicate store (cosine) reinforced existing item".to_string(),
            ) {
                warn!(error = %format_args!("{e:#}"), "record_item_event (cosine-reinforced)");
            }
            self.fanout_rag_ingest(reinforced.clone());
            return Ok((reinforced, None));
        }

        let duplicate =
            self.store
                .insert_or_get_duplicate(&item, &canonical_key, &redundancy_key)?;
        if let Some(found) = duplicate.as_ref()
            && let Some(revived) = self.revive_duplicate_on_explicit_store(
                found,
                &item,
                &canonical_key,
                &redundancy_key,
            )?
        {
            if let Err(e) = self.record_item_event(
                &revived,
                "restored",
                "duplicate memory item restored to active canonical state".to_string(),
            ) {
                warn!(error = %format_args!("{e:#}"), "record_item_event (restored)");
            }
            self.fanout_rag_ingest(revived.clone());
            self.maybe_upsert_vector(&revived);
            return Ok((revived, None));
        }
        if let Some(found) = duplicate.as_ref() {
            let mut reinforced = found.item.clone();
            reinforced.updated_at = Utc::now();
            reinforced.confidence = (reinforced.confidence + 0.05).min(1.0);
            let rk = found
                .item
                .redundancy_key
                .as_deref()
                .unwrap_or(&redundancy_key);
            self.store.update(&reinforced, &canonical_key, rk)?;
            if let Err(e) = self.record_item_event(
                &reinforced,
                "reinforced",
                "duplicate store reinforced existing item".to_string(),
            ) {
                warn!(error = %format_args!("{e:#}"), "record_item_event (reinforced)");
            }
            self.fanout_rag_ingest(reinforced.clone());
            self.maybe_upsert_vector(&reinforced);
            return Ok((reinforced, Some(found.clone())));
        }
        if duplicate.is_none() {
            let entity = self.store.resolve_entity_for_item(&item, &canonical_key)?;
            if let Err(e) = self.record_item_event_for_entity(
                &entity.record,
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
                warn!(error = %format_args!("{e:#}"), "record_item_event (stored)");
            }

            // Auto-expire excess status items to prevent noise accumulation
            if item.kind == MemoryKind::Status {
                if let Err(e) = self.expire_excess_status_items(&item, 4) {
                    warn!(error = %format_args!("{e:#}"), "expire_excess_status_items");
                }
            }

            // Auto-link co-occurring entities within the same project.
            // Gated by MEMD_STORE_AUTO_LINK_DISABLED: both branches here run
            // `list_entities()` (O(N) scan + JSON parse), which is quadratic
            // on bulk ingest. Bench sweeps set the flag to keep throughput flat.
            if !store_auto_link_disabled() {
                if item.kind != MemoryKind::Status {
                    if let Err(e) = self.auto_link_entity(&entity.record, &item) {
                        warn!(error = %format_args!("{e:#}"), "auto_link_entity");
                    }
                    if let Err(e) = self.create_named_entity_links(&entity.record, &item) {
                        warn!(error = %format_args!("{e:#}"), "create_named_entity_links");
                    }
                }

                // E2: Parse [[wiki links]] in content and create entity links
                if let Err(e) = self.create_wiki_links(&entity.record, &item) {
                    warn!(error = %format_args!("{e:#}"), "create_wiki_links");
                }
            }
        }
        self.fanout_rag_ingest(item.clone());
        self.maybe_upsert_vector(&item);
        Ok((item, duplicate))
    }

    fn find_near_duplicate_for_item(
        &self,
        item: &MemoryItem,
    ) -> anyhow::Result<Option<MemoryItem>> {
        let Some(embedder) = self.embedder.as_deref() else {
            return Ok(None);
        };
        let chunks = embed::chunk_text(
            &item.content,
            embed::chunk_max_chars(),
            embed::chunk_overlap_chars(),
        );
        if chunks.is_empty() {
            return Ok(None);
        }
        let vectors = match embedder.embed_batch_normalized(&chunks) {
            Ok(v) => v,
            Err(err) => {
                warn!(error = %format_args!("{err:#}"), "dedup embed failed");
                return Ok(None);
            }
        };
        let Some(first) = vectors.first() else {
            return Ok(None);
        };
        let hit = self.store.find_near_duplicate(
            item.project.as_deref(),
            item.namespace.as_deref(),
            embedder.model_code(),
            first,
            store_dedup::DEFAULT_DEDUP_COSINE_DISTANCE,
        )?;
        let Some(hit) = hit else { return Ok(None) };
        self.store.get(hit.existing_id)
    }

    fn maybe_upsert_vector(&self, item: &MemoryItem) {
        let Some(embedder) = self.embedder.as_deref() else {
            return;
        };
        let chunks = embed::chunk_text(
            &item.content,
            embed::chunk_max_chars(),
            embed::chunk_overlap_chars(),
        );
        if chunks.is_empty() {
            return;
        }
        let vectors = match embedder.embed_batch_normalized(&chunks) {
            Ok(v) => v,
            Err(err) => {
                warn!(error = %format_args!("{err:#}"), "embed batch failed");
                return;
            }
        };
        let rows: Vec<(i64, Vec<u8>)> = vectors
            .into_iter()
            .enumerate()
            .map(|(idx, v)| (idx as i64, embed::vec_to_bytes(&v)))
            .collect();
        if rows.is_empty() {
            return;
        }
        if let Err(err) = self.store.replace_memory_vector_chunks(
            item.id,
            item.project.as_deref(),
            item.namespace.as_deref(),
            embedder.model_code(),
            embedder.dim(),
            &rows,
        ) {
            warn!(error = %format_args!("{err:#}"), "replace_memory_vector_chunks failed");
        }
    }

    fn fanout_rag_ingest(&self, item: MemoryItem) {
        if let Some(rag) = self.rag.clone() {
            rag_bridge::spawn_ingest(rag, item);
        }
    }

    async fn rag_dense_candidates(
        &self,
        req: &SearchMemoryRequest,
    ) -> anyhow::Result<Vec<(Uuid, f64)>> {
        let Some(rag) = self.rag.as_deref() else {
            return Ok(Vec::new());
        };
        rag_bridge::fetch_dense_candidates(rag, req).await
    }

    async fn rag_health_surface(&self) -> memd_schema::RagHealthStatus {
        rag_bridge::health_surface(self.rag.as_deref()).await
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
        // V3/B3: project-scoped indexed lookup (see store.rs
        // `list_entities_by_project`). Pull 4 so we can still yield 3 after
        // filtering the new entity itself out.
        let entities = self.store.list_entities_by_project(project, 4)?;
        let candidates: Vec<&MemoryEntityRecord> = entities
            .iter()
            .filter(|e| e.id != new_entity.id)
            // E2: no salience gate — link on co-occurrence, not salience.
            // New entities start at 0.0 salience; gating blocked all links.
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
                valid_from: Some(item.updated_at),
                valid_to: None,
                source_item_id: Some(item.id),
                note: Some("auto-linked by co-occurrence".to_string()),
                context: None,
                tags: vec!["auto".to_string()],
            };
            self.store.upsert_entity_link(&link)?;
        }
        Ok(())
    }

    fn create_wiki_links(
        &self,
        source_entity: &MemoryEntityRecord,
        item: &MemoryItem,
    ) -> anyhow::Result<()> {
        let wiki_refs = parse_wiki_links(&item.content);
        if wiki_refs.is_empty() {
            return Ok(());
        }
        // V3/B3: per-wiki-ref alias-indexed lookup. Not project-scoped — the
        // original behavior was global so `[[alpha-svc]]` in project beta-svc
        // resolves to the alpha-svc entity. Entity-type substring match is
        // dropped (prior behavior matched every entity whose type contained
        // the wiki token, e.g. `[[task]]` hitting every task — noisy). Alias
        // matches are stricter and sufficient.
        for wiki_ref in wiki_refs {
            let matches = self
                .store
                .find_entities_by_alias_contains(None, &wiki_ref)?;
            let target = matches.iter().find(|e| e.id != source_entity.id);
            if let Some(target_entity) = target {
                let existing = self.store.links_for_entity(&EntityLinksRequest {
                    entity_id: source_entity.id,
                })?;
                let already_linked = existing.iter().any(|link| {
                    link.from_entity_id == target_entity.id || link.to_entity_id == target_entity.id
                });
                if already_linked {
                    continue;
                }
                let link = MemoryEntityLinkRecord {
                    id: Uuid::new_v4(),
                    from_entity_id: source_entity.id,
                    to_entity_id: target_entity.id,
                    relation_kind: memd_schema::EntityRelationKind::Related,
                    confidence: 0.7,
                    created_at: Utc::now(),
                    valid_from: Some(item.updated_at),
                    valid_to: None,
                    source_item_id: Some(item.id),
                    note: Some(format!("wiki link: [[{}]]", wiki_ref)),
                    context: None,
                    tags: vec!["wiki-link".to_string(), "auto".to_string()],
                };
                self.store.upsert_entity_link(&link)?;
            }
        }
        Ok(())
    }

    fn create_named_entity_links(
        &self,
        source_entity: &MemoryEntityRecord,
        item: &MemoryItem,
    ) -> anyhow::Result<()> {
        let mentions = crate::store_entities::extract_named_entity_aliases(&item.content);
        if mentions.is_empty() {
            return Ok(());
        }
        // V3/B3: per-mention exact-alias lookup via the aliases companion
        // table (NOCASE collation). Not project-scoped — the original
        // `list_entities()` scan was global and cross-project mentions are
        // load-bearing for NER link tests. Replaces the full-table scan
        // that stalled bulk ingests.
        let existing = self.store.links_for_entity(&EntityLinksRequest {
            entity_id: source_entity.id,
        })?;

        for mention in mentions.into_iter().take(8) {
            let matches = self.store.find_entities_by_alias_exact(None, &mention)?;
            let target = matches.iter().find(|entity| entity.id != source_entity.id);
            let Some(target_entity) = target else {
                continue;
            };
            let already_linked = existing.iter().any(|link| {
                (link.from_entity_id == target_entity.id || link.to_entity_id == target_entity.id)
                    && link.relation_kind == memd_schema::EntityRelationKind::Related
            });
            if already_linked
                && !existing.iter().any(|link| {
                    (link.from_entity_id == target_entity.id
                        || link.to_entity_id == target_entity.id)
                        && link.tags.iter().any(|tag| tag == "auto")
                })
            {
                continue;
            }

            let link = MemoryEntityLinkRecord {
                id: Uuid::new_v4(),
                from_entity_id: source_entity.id,
                to_entity_id: target_entity.id,
                relation_kind: memd_schema::EntityRelationKind::Related,
                confidence: 0.65,
                created_at: Utc::now(),
                valid_from: Some(item.updated_at),
                valid_to: None,
                source_item_id: Some(item.id),
                note: Some(format!("named entity mention: {mention}")),
                context: None,
                tags: vec!["ner".to_string(), "auto".to_string()],
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

    /// Scoped snapshot — hydrates only items matching the given project
    /// and/or namespace. Hot path for bench search where each question
    /// pins a fresh namespace; avoids global-corpus scan.
    pub(crate) fn snapshot_for_scope(
        &self,
        project: Option<&str>,
        namespace: Option<&str>,
    ) -> anyhow::Result<Vec<MemoryItem>> {
        let items = self.store.list_for_scope(project, namespace)?;
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
            warn!(error = %format_args!("{e:#}"), "record_item_event (promoted)");
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
        self.record_item_event_for_entity(&entity.record, item, event_type, summary)
    }

    fn record_item_event_for_entity(
        &self,
        entity: &MemoryEntityRecord,
        item: &MemoryItem,
        event_type: &str,
        summary: String,
    ) -> anyhow::Result<MemoryEventRecord> {
        let context = Some(entity_context_frame(entity, item));
        self.store.record_event(
            entity,
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
                salience_score: entity.salience_score,
            },
        )
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,memd_server=debug,tower_http=info"));
    let format = std::env::var("MEMD_LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string());
    let builder = tracing_subscriber::fmt().with_env_filter(filter);
    match format.as_str() {
        "json" => builder.json().init(),
        _ => builder.compact().init(),
    }
}

// K2.7: CLI subcommands for operating the on-disk store out-of-band.
//   memd-server backup [out.db]    -> write a snapshot of $MEMD_DB_PATH
//   memd-server restore <in.db>    -> restore $MEMD_DB_PATH from a snapshot
// When no subcommand is supplied we fall through to the HTTP server path.
// Subcommands deliberately run before binding the listener so no handler
// is racing the file swap during restore.
fn handle_cli_subcommand(db_path: &str) -> Option<i32> {
    let mut args = std::env::args().skip(1);
    let cmd = args.next()?;
    match cmd.as_str() {
        "backup" => {
            let out = args
                .next()
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| {
                    let dir = backup::snapshots_dir(std::path::Path::new(db_path));
                    dir.join(backup::snapshot_filename_now())
                });
            let db = std::path::Path::new(db_path);
            match backup::write_snapshot(db, &out) {
                Ok(bytes) => {
                    println!(
                        "backup: wrote {} ({} bytes) from {}",
                        out.display(),
                        bytes,
                        db.display()
                    );
                    if let Some(parent) = out.parent() {
                        if let Err(e) = backup::rotate_snapshots(parent, 5) {
                            warn!(error = %format_args!("{e:#}"), "rotate snapshots failed");
                        }
                    }
                    Some(0)
                }
                Err(e) => {
                    error!(error = %format_args!("{e:#}"), "backup failed");
                    Some(1)
                }
            }
        }
        "restore" => {
            let Some(src) = args.next() else {
                error!("restore requires a snapshot path argument");
                return Some(2);
            };
            match backup::restore_from(std::path::Path::new(&src), std::path::Path::new(db_path)) {
                Ok(()) => {
                    println!("restore: {} -> {}", src, db_path);
                    Some(0)
                }
                Err(e) => {
                    error!(error = %format_args!("{e:#}"), "restore failed");
                    Some(1)
                }
            }
        }
        other => {
            error!(cmd = %other, "unknown subcommand (expected: backup | restore)");
            Some(2)
        }
    }
}

fn schedule_reembed_sweep(state: AppState) {
    let Some(embedder) = state.embedder.clone() else {
        return;
    };
    let target_model = embedder.model_code().to_string();
    let _ = std::thread::Builder::new()
        .name("memd-reembed-sweep".to_string())
        .spawn(move || {
            loop {
                let items = match state.store.items_needing_reembed(&target_model, 64) {
                    Ok(items) => items,
                    Err(error) => {
                        warn!(error = %format_args!("{error:#}"), "items_needing_reembed failed");
                        break;
                    }
                };
                if items.is_empty() {
                    break;
                }
                for item in items {
                    state.maybe_upsert_vector(&item);
                }
            }
        });
}

#[tokio::main]
async fn main() {
    init_tracing();
    let db_path = std::env::var("MEMD_DB_PATH").unwrap_or_else(|_| ".memd/memd.db".to_string());
    if let Some(code) = handle_cli_subcommand(&db_path) {
        std::process::exit(code);
    }
    let bind_addr =
        std::env::var("MEMD_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8787".to_string());
    let store = match SqliteStore::open(&db_path) {
        Ok(store) => store,
        Err(e) => {
            error!(error = %format_args!("{e:#}"), %db_path, "failed to open database");
            std::process::exit(1);
        }
    };
    let embedder = if embed::intrinsic_dense_enabled() {
        let cache_dir = embed::default_cache_dir();
        match embed::Embedder::try_new(&cache_dir) {
            Ok(e) => {
                tracing::info!(
                    model = %e.model_code(),
                    cache_dir = %cache_dir.display(),
                    "intrinsic dense embedder ready"
                );
                Some(std::sync::Arc::new(e))
            }
            Err(err) => {
                error!(error = %format_args!("{err:#}"), "failed to init fastembed; intrinsic dense disabled");
                None
            }
        }
    } else {
        None
    };
    let state = AppState {
        store,
        latency: latency::LatencyHistogram::new(),
        rate_limiter: std::sync::Arc::new(rate_limit::RateLimiter::new()),
        rag: rag_bridge::build_rag_client(),
        embedder,
    };
    schedule_reembed_sweep(state.clone());
    let app = Router::new()
        .route("/", get(dashboard))
        .route("/ui/snapshot", get(get_visible_memory_snapshot))
        .route("/ui/artifact", get(get_visible_memory_artifact))
        .route("/ui/action", post(post_visible_memory_action))
        .route("/healthz", get(healthz))
        .route("/api/status", get(status::get_harness_status))
        .route("/api/memory/search", get(search_memory_get))
        .route("/api/diagnostics/spine/verify", get(status::verify_spine))
        .route("/api/diagnostics/latency", get(status::get_latency))
        .route("/memory/store", post(store_memory))
        .route("/memory/candidates", post(store_candidate))
        .route("/memory/promote", post(promote_memory))
        .route("/memory/expire", post(expire_memory))
        .route("/memory/verify", post(verify_memory))
        .route("/memory/repair", post(repair_memory))
        .route("/memory/correct", post(correct_memory))
        .route("/memory/search", post(search_memory))
        .route("/memory/authority/search", post(search_memory_authority))
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
        .route("/hive/divergence", get(get_hive_divergence))
        .route("/hive/queen/deny", post(post_hive_queen_deny))
        .route("/hive/queen/reroute", post(post_hive_queen_reroute))
        .route("/hive/queen/handoff", post(post_hive_queen_handoff))
        .route("/coordination/tasks/upsert", post(post_hive_task_upsert))
        .route("/coordination/tasks/assign", post(post_hive_task_assign))
        .route("/coordination/tasks", get(get_hive_tasks))
        .route("/memory/maintenance/decay", post(decay_memory))
        .route("/memory/maintenance/consolidate", post(consolidate_memory))
        .route("/episodes/consolidate", post(consolidate_episodes_handler))
        .route("/episodes/list", get(list_episodes_handler))
        .route("/memory/dedup/scan", post(dedup_scan_handler))
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
        .route("/ingest/lanes", post(post_ingest_lanes))
        .route("/api/diagnostics/decay", post(decay_diagnostics))
        .route(
            "/api/diagnostics/token-efficiency",
            post(token_efficiency_diagnostics),
        )
        .layer(axum::middleware::from_fn(
            token_headers::token_headers_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            rate_limit::rate_limit_middleware,
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // K2.7: periodic snapshot loop. MEMD_SNAPSHOT_INTERVAL_SECS=0 disables.
    // Runs off the hot path in a background tokio task, rotating to keep
    // the most recent MEMD_SNAPSHOT_KEEP (default 5).
    let snapshot_interval = std::env::var("MEMD_SNAPSHOT_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(300);
    let snapshot_keep = std::env::var("MEMD_SNAPSHOT_KEEP")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(5);
    if snapshot_interval > 0 {
        let db_path_bg = db_path.clone();
        tokio::spawn(async move {
            let mut ticker =
                tokio::time::interval(std::time::Duration::from_secs(snapshot_interval));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            // First tick fires immediately — skip it so startup isn't double-slow.
            ticker.tick().await;
            loop {
                ticker.tick().await;
                let db_path = db_path_bg.clone();
                let result = tokio::task::spawn_blocking(move || {
                    let db = std::path::Path::new(&db_path);
                    let dir = backup::snapshots_dir(db);
                    let out = dir.join(backup::snapshot_filename_now());
                    let bytes = backup::write_snapshot(db, &out)?;
                    let pruned = backup::rotate_snapshots(&dir, snapshot_keep)?;
                    Ok::<_, anyhow::Error>((out, bytes, pruned.len()))
                })
                .await;
                match result {
                    Ok(Ok((out, bytes, pruned))) => tracing::info!(
                        snapshot = %out.display(),
                        bytes,
                        pruned,
                        "periodic snapshot written"
                    ),
                    Ok(Err(e)) => warn!(error = %format_args!("{e:#}"), "snapshot task failed"),
                    Err(e) => warn!(error = %e, "snapshot task panicked"),
                }
            }
        });
    }

    let listener = match tokio::net::TcpListener::bind(&bind_addr).await {
        Ok(listener) => listener,
        Err(e) => {
            error!(
                error = %format_args!("{e:#}"),
                %bind_addr,
                "failed to bind (hint: port may be in use, set MEMD_BIND_ADDR to change)"
            );
            std::process::exit(1);
        }
    };
    tracing::info!(%bind_addr, %db_path, "memd-server listening");
    if let Err(e) = axum::serve(listener, app).await {
        error!(error = %format_args!("{e:#}"), "server exited unexpectedly");
        std::process::exit(1);
    }
}
