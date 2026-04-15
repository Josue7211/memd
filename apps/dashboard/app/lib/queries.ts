import {
  useQuery,
  useMutation,
  useQueryClient,
} from "@tanstack/react-query";
import { api } from "./api";
import type {
  SearchMemoryRequest,
  StoreMemoryRequest,
  ExpireMemoryRequest,
  PromoteMemoryRequest,
  VerifyMemoryRequest,
  InboxDismissRequest,
  RepairMemoryRequest,
  CorrectMemoryRequest,
  HiveQueenActionRequest,
} from "./types";

// ── Keys ─────────────────────────────────────────────────────────────────────

export const keys = {
  health: ["health"] as const,
  working: (p?: { project?: string }) => ["working", p] as const,
  inbox: (p?: { project?: string }) => ["inbox", p] as const,
  search: (req: SearchMemoryRequest) => ["search", req] as const,
  timeline: (id: string) => ["timeline", id] as const,
  context: (p?: { project?: string }) => ["context", p] as const,
  explain: (id: string) => ["explain", id] as const,
  entityLinks: (p?: { entity_id?: string }) => ["entityLinks", p] as const,
  entitySearch: (q: string) => ["entitySearch", q] as const,
  atlasRegions: (p?: { project?: string }) => ["atlasRegions", p] as const,
  atlasExplore: (id: string) => ["atlasExplore", id] as const,
  atlasTrails: (p?: { project?: string }) => ["atlasTrails", p] as const,
  procedures: (p?: { project?: string; status?: string }) =>
    ["procedures", p] as const,
  sessions: (p?: { project?: string }) => ["sessions", p] as const,
  tasks: (p?: { project?: string }) => ["tasks", p] as const,
};

// ── Queries ──────────────────────────────────────────────────────────────────

export function useHealth() {
  return useQuery({
    queryKey: keys.health,
    queryFn: api.health,
    refetchInterval: 15_000,
  });
}

export function useWorking(params?: { project?: string }) {
  return useQuery({
    queryKey: keys.working(params),
    queryFn: () => api.working(params),
    refetchInterval: 30_000,
  });
}

export function useInbox(params?: { project?: string }) {
  return useQuery({
    queryKey: keys.inbox(params),
    queryFn: () => api.inbox(params),
  });
}

export function useSearch(req: SearchMemoryRequest, enabled = true) {
  return useQuery({
    queryKey: keys.search(req),
    queryFn: () => api.search(req),
    enabled,
  });
}

export function useTimeline(id: string) {
  return useQuery({
    queryKey: keys.timeline(id),
    queryFn: () => api.timeline({ id }),
    enabled: !!id,
  });
}

export function useContext(params?: { project?: string }) {
  return useQuery({
    queryKey: keys.context(params),
    queryFn: () => api.context(params ?? {}),
  });
}

export function useExplain(id: string) {
  return useQuery({
    queryKey: keys.explain(id),
    queryFn: () => api.explain({ id }),
    enabled: !!id,
  });
}

export function useEntityLinks(params?: { entity_id?: string }) {
  return useQuery({
    queryKey: keys.entityLinks(params),
    queryFn: () => api.entityLinks(params ?? {}),
    enabled: !!params?.entity_id,
  });
}

export function useEntitySearch(query: string) {
  return useQuery({
    queryKey: keys.entitySearch(query),
    queryFn: () => api.entitySearch({ query }),
    enabled: query.length > 0,
  });
}

export function useAtlasRegions(params?: { project?: string }) {
  return useQuery({
    queryKey: keys.atlasRegions(params),
    queryFn: () => api.atlasRegions(params),
  });
}

export function useAtlasExplore(regionId: string) {
  return useQuery({
    queryKey: keys.atlasExplore(regionId),
    queryFn: () => api.atlasExplore({ region_id: regionId }),
    enabled: !!regionId,
  });
}

export function useAtlasTrails(params?: { project?: string }) {
  return useQuery({
    queryKey: keys.atlasTrails(params),
    queryFn: () => api.atlasTrails(params),
  });
}

export function useProcedures(params?: {
  project?: string;
  status?: string;
}) {
  return useQuery({
    queryKey: keys.procedures(params),
    queryFn: () => api.procedures(params),
  });
}

export function useSessions(params?: { project?: string }) {
  return useQuery({
    queryKey: keys.sessions(params),
    queryFn: () => api.sessions(params),
    refetchInterval: 10_000,
  });
}

export function useTasks(params?: { project?: string }) {
  return useQuery({
    queryKey: keys.tasks(params),
    queryFn: () => api.tasks(params),
  });
}

export function useProfile(agent: string, project?: string) {
  return useQuery({
    queryKey: ["profile", agent, project] as const,
    queryFn: () => api.profile({ agent, project }),
    enabled: !!agent,
  });
}

export function useSource(params?: {
  source_agent?: string;
  project?: string;
}) {
  return useQuery({
    queryKey: ["source", params] as const,
    queryFn: () => api.source(params ?? {}),
    enabled: !!params?.source_agent,
  });
}

export function useAtlasExpand(entityId: string) {
  return useQuery({
    queryKey: ["atlasExpand", entityId] as const,
    queryFn: () => api.atlasExpand({ entity_id: entityId }),
    enabled: !!entityId,
  });
}

// ── Mutations ────────────────────────────────────────────────────────────────

function useInvalidate(...queryKeys: readonly (readonly unknown[])[]) {
  const qc = useQueryClient();
  return () => {
    for (const key of queryKeys) {
      qc.invalidateQueries({ queryKey: key });
    }
  };
}

export function useStore() {
  const invalidate = useInvalidate(["working"], ["inbox"], ["search"]);
  return useMutation({
    mutationFn: (req: StoreMemoryRequest) => api.store(req),
    onSuccess: invalidate,
  });
}

export function useExpire() {
  const invalidate = useInvalidate(["working"], ["inbox"], ["search"]);
  return useMutation({
    mutationFn: (req: ExpireMemoryRequest) => api.expire(req),
    onSuccess: invalidate,
  });
}

export function usePromote() {
  const invalidate = useInvalidate(["working"], ["inbox"], ["search"]);
  return useMutation({
    mutationFn: (req: PromoteMemoryRequest) => api.promote(req),
    onSuccess: invalidate,
  });
}

export function useVerify() {
  const invalidate = useInvalidate(["search"]);
  return useMutation({
    mutationFn: (req: VerifyMemoryRequest) => api.verify(req),
    onSuccess: invalidate,
  });
}

export function useInboxDismiss() {
  const invalidate = useInvalidate(["inbox"]);
  return useMutation({
    mutationFn: (req: InboxDismissRequest) => api.inboxDismiss(req),
    onSuccess: invalidate,
  });
}

export function useRepair() {
  const invalidate = useInvalidate(["inbox"], ["search"]);
  return useMutation({
    mutationFn: (req: RepairMemoryRequest) => api.repair(req),
    onSuccess: invalidate,
  });
}

export function useCorrect() {
  const invalidate = useInvalidate(["search"], ["working"], ["inbox"]);
  return useMutation({
    mutationFn: (req: CorrectMemoryRequest) => api.correct(req),
    onSuccess: invalidate,
  });
}

export function useProcedurePromote() {
  const invalidate = useInvalidate(["procedures"]);
  return useMutation({
    mutationFn: (req: { id: string }) => api.procedurePromote(req),
    onSuccess: invalidate,
  });
}

export function useProcedureRetire() {
  const invalidate = useInvalidate(["procedures"]);
  return useMutation({
    mutationFn: (req: { id: string }) => api.procedureRetire(req),
    onSuccess: invalidate,
  });
}

export function useQueenDeny() {
  return useMutation({
    mutationFn: (req: HiveQueenActionRequest) => api.queenDeny(req),
  });
}

export function useQueenReroute() {
  return useMutation({
    mutationFn: (req: HiveQueenActionRequest) => api.queenReroute(req),
  });
}

export function useQueenHandoff() {
  return useMutation({
    mutationFn: (req: HiveQueenActionRequest) => api.queenHandoff(req),
  });
}
