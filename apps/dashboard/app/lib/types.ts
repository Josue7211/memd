// ── Enums ────────────────────────────────────────────────────────────────────

export type MemoryKind =
  | "fact"
  | "decision"
  | "preference"
  | "runbook"
  | "procedural"
  | "self_model"
  | "topology"
  | "status"
  | "live_truth"
  | "pattern"
  | "constraint";

export type MemoryScope = "local" | "synced" | "project" | "global";

export type MemoryStatus =
  | "active"
  | "stale"
  | "superseded"
  | "contested"
  | "expired";

export type MemoryStage = "candidate" | "canonical";

export type MemoryVisibility = "private" | "workspace" | "public";

export type SourceQuality = "canonical" | "derived" | "synthetic";

export type RetrievalRoute =
  | "auto"
  | "local_only"
  | "synced_only"
  | "project_only"
  | "global_only"
  | "local_first"
  | "synced_first"
  | "project_first"
  | "global_first"
  | "all";

export type RetrievalIntent =
  | "general"
  | "current_task"
  | "decision"
  | "runbook"
  | "procedural"
  | "self_model"
  | "topology"
  | "preference"
  | "fact"
  | "pattern";

export type EntityRelationKind =
  | "same_as"
  | "derived_from"
  | "supersedes"
  | "contradicts"
  | "related";

export type ProcedureKind = "workflow" | "policy" | "recovery";

export type ProcedureStatus = "candidate" | "promoted" | "retired";

// ── Core Types ───────────────────────────────────────────────────────────────

export interface MemoryItem {
  id: string;
  content: string;
  redundancy_key?: string;
  belief_branch?: string;
  preferred: boolean;
  kind: MemoryKind;
  scope: MemoryScope;
  project?: string;
  namespace?: string;
  workspace?: string;
  visibility: MemoryVisibility;
  source_agent?: string;
  source_system?: string;
  source_path?: string;
  source_quality?: SourceQuality;
  confidence: number;
  ttl_seconds?: number;
  created_at: string;
  updated_at: string;
  last_verified_at?: string;
  supersedes: string[];
  tags: string[];
  status: MemoryStatus;
  stage: MemoryStage;
}

export interface InboxMemoryItem {
  item: MemoryItem;
  reasons: string[];
}

export interface MemoryEntityRecord {
  id: string;
  name: string;
  kind?: string;
  project?: string;
  namespace?: string;
  created_at: string;
  updated_at: string;
}

export interface MemoryEntityLinkRecord {
  id: string;
  source_entity_id: string;
  target_entity_id: string;
  relation: EntityRelationKind;
  source_name?: string;
  target_name?: string;
  confidence: number;
  project?: string;
  created_at: string;
}

export interface MemoryEventRecord {
  id: string;
  entity_id?: string;
  event_type: string;
  summary: string;
  occurred_at: string;
  recorded_at: string;
  confidence: number;
  salience_score: number;
  project?: string;
  namespace?: string;
  source_agent?: string;
  source_system?: string;
  related_entity_ids: string[];
  tags: string[];
}

export interface AtlasRegion {
  id: string;
  name: string;
  description?: string;
  project?: string;
  namespace?: string;
  lane?: string;
  auto_generated: boolean;
  node_count: number;
  tags: string[];
  created_at: string;
  updated_at: string;
}

export interface AtlasNode {
  entity_id: string;
  name: string;
  kind?: string;
  region_id: string;
  confidence: number;
}

export interface AtlasLink {
  source_entity_id: string;
  target_entity_id: string;
  relation: EntityRelationKind;
  confidence: number;
}

export interface AtlasTrail {
  id: string;
  name: string;
  node_ids: string[];
  created_at: string;
}

export interface Procedure {
  id: string;
  name: string;
  description: string;
  kind: ProcedureKind;
  status: ProcedureStatus;
  trigger: string;
  steps: string[];
  success_criteria?: string;
  source_ids: string[];
  project?: string;
  namespace?: string;
  use_count: number;
  confidence: number;
  created_at: string;
  updated_at: string;
  tags: string[];
  session_count: number;
  last_session?: string;
  supersedes?: string;
}

export interface HiveSessionRecord {
  session: string;
  agent?: string;
  effective_agent?: string;
  hive_role?: string;
  display_name?: string;
  project?: string;
  namespace?: string;
  branch?: string;
  focus?: string;
  status: string;
  last_seen: string;
}

export interface HiveTaskRecord {
  id: string;
  title: string;
  description?: string;
  status: string;
  assigned_to?: string;
  project?: string;
  created_at: string;
  updated_at: string;
}

// ── Request Types ────────────────────────────────────────────────────────────

export interface SearchMemoryRequest {
  query?: string;
  route?: RetrievalRoute;
  intent?: RetrievalIntent;
  scopes?: MemoryScope[];
  kinds?: MemoryKind[];
  statuses?: MemoryStatus[];
  stages?: MemoryStage[];
  project?: string;
  namespace?: string;
  source_agent?: string;
  tags?: string[];
  limit?: number;
}

export interface StoreMemoryRequest {
  content: string;
  kind: MemoryKind;
  scope?: MemoryScope;
  project?: string;
  namespace?: string;
  source_agent?: string;
  source_system?: string;
  confidence?: number;
  supersedes?: string[];
  tags?: string[];
}

export interface ExpireMemoryRequest {
  ids: string[];
}

export interface PromoteMemoryRequest {
  ids: string[];
}

export interface VerifyMemoryRequest {
  ids: string[];
}

export interface InboxDismissRequest {
  ids: string[];
}

export interface RepairMemoryRequest {
  id: string;
  content: string;
}

export interface HiveQueenActionRequest {
  session: string;
  reason?: string;
  target?: string;
}

// ── Response Types ───────────────────────────────────────────────────────────

export interface HealthResponse {
  status: string;
  items: number;
  eval_score?: number;
  degraded?: boolean;
  inbox_count?: number;
  candidate_count?: number;
  stale_count?: number;
  expired_count?: number;
}

export interface SearchMemoryResponse {
  route: RetrievalRoute;
  intent: RetrievalIntent;
  items: MemoryItem[];
}

export interface StoreMemoryResponse {
  id: string;
  status: string;
}

export interface WorkingMemoryResponse {
  items: MemoryItem[];
  procedures?: Procedure[];
}

export interface MemoryInboxResponse {
  items: InboxMemoryItem[];
}

export interface TimelineMemoryResponse {
  route: RetrievalRoute;
  intent: RetrievalIntent;
  entity?: MemoryEntityRecord;
  events: MemoryEventRecord[];
}

export interface EntityLinksResponse {
  links: MemoryEntityLinkRecord[];
}

export interface EntitySearchResponse {
  entities: MemoryEntityRecord[];
}

export interface AtlasRegionsResponse {
  regions: AtlasRegion[];
}

export interface AtlasExploreResponse {
  region: AtlasRegion;
  nodes: AtlasNode[];
  links: AtlasLink[];
}

export interface AtlasExpandResponse {
  nodes: AtlasNode[];
  links: AtlasLink[];
}

export interface AtlasListTrailsResponse {
  trails: AtlasTrail[];
}

export interface ProcedureListResponse {
  procedures: Procedure[];
}

export interface HiveSessionsResponse {
  sessions: HiveSessionRecord[];
}

export interface HiveTasksResponse {
  tasks: HiveTaskRecord[];
}

export interface CompactContextResponse {
  context: string;
  item_count: number;
}

export interface ExplainMemoryResponse {
  explanation: string;
}

export interface ContextResponse {
  items: MemoryItem[];
}
