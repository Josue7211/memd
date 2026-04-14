import type {
  HealthResponse,
  SearchMemoryRequest,
  SearchMemoryResponse,
  StoreMemoryRequest,
  StoreMemoryResponse,
  WorkingMemoryResponse,
  MemoryInboxResponse,
  TimelineMemoryResponse,
  EntityLinksResponse,
  EntitySearchResponse,
  ExplainMemoryResponse,
  ContextResponse,
  CompactContextResponse,
  ExpireMemoryRequest,
  PromoteMemoryRequest,
  VerifyMemoryRequest,
  InboxDismissRequest,
  RepairMemoryRequest,
  AtlasRegionsResponse,
  AtlasExploreResponse,
  AtlasExpandResponse,
  AtlasListTrailsResponse,
  ProcedureListResponse,
  HiveSessionsResponse,
  HiveTasksResponse,
  HiveQueenActionRequest,
} from "./types";

async function request<T>(
  path: string,
  init?: RequestInit,
): Promise<T> {
  const res = await fetch(path, {
    headers: { "Content-Type": "application/json", ...init?.headers },
    ...init,
  });
  if (!res.ok) {
    const text = await res.text().catch(() => "");
    throw new Error(`${res.status} ${res.statusText}: ${text}`);
  }
  return res.json();
}

function get<T>(path: string, params?: Record<string, string>): Promise<T> {
  const url = new URL(path, window.location.origin);
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      if (v != null) url.searchParams.set(k, v);
    }
  }
  return request<T>(url.pathname + url.search);
}

function post<T>(path: string, body?: unknown): Promise<T> {
  return request<T>(path, {
    method: "POST",
    body: body != null ? JSON.stringify(body) : undefined,
  });
}

// ── API ─────────────────────────────────────────────────────────────────────

export const api = {
  health: () => get<HealthResponse>("/healthz"),

  // ── Memory ───────────────────────────────────────────────────────────────

  search: (req: SearchMemoryRequest) =>
    post<SearchMemoryResponse>("/memory/search", req),

  store: (req: StoreMemoryRequest) =>
    post<StoreMemoryResponse>("/memory/store", req),

  working: (params?: { project?: string; namespace?: string }) =>
    get<WorkingMemoryResponse>("/memory/working", params as Record<string, string>),

  inbox: (params?: { project?: string }) =>
    get<MemoryInboxResponse>("/memory/inbox", params as Record<string, string>),

  inboxDismiss: (req: InboxDismissRequest) =>
    post<{ dismissed: number }>("/memory/inbox/dismiss", req),

  timeline: (params: { id: string; limit?: string }) =>
    get<TimelineMemoryResponse>("/memory/timeline", params),

  context: (params: { project?: string; namespace?: string }) =>
    get<ContextResponse>("/memory/context", params as Record<string, string>),

  contextCompact: (params: { project?: string; namespace?: string }) =>
    get<CompactContextResponse>(
      "/memory/context/compact",
      params as Record<string, string>,
    ),

  explain: (params: { id: string }) =>
    get<ExplainMemoryResponse>("/memory/explain", params),

  expire: (req: ExpireMemoryRequest) =>
    post<{ expired: number }>("/memory/expire", req),

  promote: (req: PromoteMemoryRequest) =>
    post<{ promoted: number }>("/memory/promote", req),

  verify: (req: VerifyMemoryRequest) =>
    post<{ verified: number }>("/memory/verify", req),

  repair: (req: RepairMemoryRequest) =>
    post<{ repaired: boolean }>("/memory/repair", req),

  // ── Entities ─────────────────────────────────────────────────────────────

  entityLinks: (params: { entity_id?: string; project?: string }) =>
    get<EntityLinksResponse>(
      "/memory/entity/links",
      params as Record<string, string>,
    ),

  entitySearch: (params: { query: string; project?: string }) =>
    get<EntitySearchResponse>(
      "/memory/entity/search",
      params as Record<string, string>,
    ),

  // ── Atlas ────────────────────────────────────────────────────────────────

  atlasRegions: (params?: { project?: string }) =>
    get<AtlasRegionsResponse>(
      "/atlas/regions",
      params as Record<string, string>,
    ),

  atlasExplore: (req: { region_id: string }) =>
    post<AtlasExploreResponse>("/atlas/explore", req),

  atlasExpand: (req: { entity_id: string }) =>
    post<AtlasExpandResponse>("/atlas/expand", req),

  atlasTrails: (params?: { project?: string }) =>
    get<AtlasListTrailsResponse>(
      "/atlas/trails",
      params as Record<string, string>,
    ),

  // ── Procedures ───────────────────────────────────────────────────────────

  procedures: (params?: { project?: string; status?: string }) =>
    get<ProcedureListResponse>(
      "/procedures",
      params as Record<string, string>,
    ),

  procedurePromote: (req: { id: string }) =>
    post<{ promoted: boolean }>("/procedures/promote", req),

  procedureRetire: (req: { id: string }) =>
    post<{ retired: boolean }>("/procedures/retire", req),

  procedureUse: (req: { id: string }) =>
    post<{ recorded: boolean }>("/procedures/use", req),

  // ── Coordination ─────────────────────────────────────────────────────────

  sessions: (params?: { project?: string }) =>
    get<HiveSessionsResponse>(
      "/coordination/sessions",
      params as Record<string, string>,
    ),

  tasks: (params?: { project?: string }) =>
    get<HiveTasksResponse>(
      "/coordination/tasks",
      params as Record<string, string>,
    ),

  queenDeny: (req: HiveQueenActionRequest) =>
    post<{ ok: boolean }>("/hive/queen/deny", req),

  queenReroute: (req: HiveQueenActionRequest) =>
    post<{ ok: boolean }>("/hive/queen/reroute", req),

  queenHandoff: (req: HiveQueenActionRequest) =>
    post<{ ok: boolean }>("/hive/queen/handoff", req),
} as const;
