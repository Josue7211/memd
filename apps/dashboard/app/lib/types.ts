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
  redundancy_key?: string | null;
  belief_branch?: string | null;
  preferred: boolean;
  kind: MemoryKind;
  scope: MemoryScope;
  project?: string | null;
  namespace?: string | null;
  workspace?: string | null;
  visibility: MemoryVisibility;
  source_agent?: string | null;
  source_system?: string | null;
  source_path?: string | null;
  source_quality?: SourceQuality | null;
  confidence: number;
  ttl_seconds?: number | null;
  created_at: string;
  updated_at: string;
  last_verified_at?: string | null;
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
  tab_id?: string | null;
  agent?: string | null;
  effective_agent?: string | null;
  hive_system?: string | null;
  hive_role?: string | null;
  worker_name?: string | null;
  display_name?: string | null;
  role?: string | null;
  capabilities?: string[];
  hive_groups?: string[];
  lane_id?: string | null;
  hive_group_goal?: string | null;
  authority?: string | null;
  project?: string | null;
  namespace?: string | null;
  branch?: string | null;
  focus?: string | null;
  status: string;
  last_seen?: string | null;
  last_wake_at?: string | null;
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

// ── Working Memory (compact format) ─────────────────────────────────────────

export interface WorkingRecord {
  id: string;
  record: string;
}

export interface EvictedRecord {
  id: string;
  record: string;
  reason: string;
}

export interface WorkingPolicy {
  admission_limit: number;
  max_chars_per_item: number;
  budget_chars: number;
  rehydration_limit: number;
}

// ── Request Types ────────────────────────────────────────────────────────────

export interface SearchMemoryRequest {
  query?: string;
  route?: RetrievalRoute;
  intent?: RetrievalIntent;
  scopes: MemoryScope[];
  kinds: MemoryKind[];
  statuses: MemoryStatus[];
  stages: MemoryStage[];
  tags: string[];
  project?: string;
  namespace?: string;
  source_agent?: string;
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
  route: RetrievalRoute;
  intent: RetrievalIntent;
  retrieval_order: string[];
  budget_chars: number;
  used_chars: number;
  remaining_chars: number;
  truncated: boolean;
  policy: WorkingPolicy;
  records: WorkingRecord[];
  evicted?: EvictedRecord[];
  rehydration_queue?: string[];
  procedures?: Procedure[];
}

export interface MemoryInboxResponse {
  route: RetrievalRoute;
  intent: RetrievalIntent;
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
