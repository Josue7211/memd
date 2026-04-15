# I2 Human Dashboard — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement all 5 stub dashboard pages + correction flow on Memory page + eval score on Status page.

**Architecture:** React 19 SPA with TanStack Router (file-based routes) + TanStack React Query for server state + Tailwind 4 for styling. Existing V1 scaffold provides 3 working pages (Status, Memory, Inbox) and all API wiring (api.ts, queries.ts, types.ts). We add new pages that follow existing patterns exactly. Graph pages use react-force-graph-2d/3d with 2D/3D toggle.

**Tech Stack:** React 19, TanStack Router v1.120, TanStack Query v5.62, Tailwind CSS 4, Vite 6, react-force-graph-2d, react-force-graph-3d, Three.js

**Spec:** `docs/superpowers/specs/2026-04-15-i2-human-dashboard-design.md`

---

## File Map

### New files

| File | Responsibility |
|------|---------------|
| `apps/dashboard/app/components/correction/correction-editor.tsx` | Inline editor for correcting memory items (shared: Memory + Inbox) |
| `apps/dashboard/app/components/graph/force-graph-wrapper.tsx` | Renders ForceGraph2D or ForceGraph3D based on toggle state |
| `apps/dashboard/app/components/graph/graph-toggle.tsx` | 2D/3D toggle button with localStorage persistence |
| `apps/dashboard/app/components/graph/use-graph-mode.ts` | Hook: read/write graph mode to localStorage |
| `apps/dashboard/app/components/graph/entity-detail.tsx` | Sidebar showing entity info + linked memory items |
| `apps/dashboard/app/components/graph/graph-constants.ts` | Force config constants (supermemory E2-D3 donor) |

### Modified files

| File | Changes |
|------|---------|
| `apps/dashboard/package.json` | Add react-force-graph-2d, react-force-graph-3d, three, @types/three |
| `apps/dashboard/app/lib/types.ts` | Add CorrectMemoryRequest, CorrectMemoryResponse, PressureMetrics, extended HealthResponse |
| `apps/dashboard/app/lib/api.ts` | Add `correct` endpoint, `profile`, `source` endpoints |
| `apps/dashboard/app/lib/queries.ts` | Add useCorrect, useProfile, useSource, useAtlasExpand mutation hooks |
| `apps/dashboard/app/routes/index.tsx` | Add eval score + pressure metrics panel |
| `apps/dashboard/app/routes/memory.tsx` | Add "Correct" button + CorrectionEditor in expanded view |
| `apps/dashboard/app/routes/atlas.tsx` | Full rewrite: region graph with 2D/3D toggle |
| `apps/dashboard/app/routes/graph.tsx` | Full rewrite: entity explorer with 2D/3D toggle |
| `apps/dashboard/app/routes/procedures.tsx` | Full rewrite: tabbed procedure list with actions |
| `apps/dashboard/app/routes/agent.tsx` | Full rewrite: agent profile + session viewer |
| `apps/dashboard/app/routes/ask.tsx` | Full rewrite: search + explain interface |
| `crates/memd-schema/src/lib.rs:2217-2220` | Extend HealthResponse with eval_score + pressure |
| `crates/memd-server/src/routes.rs:3-10` | Populate new healthz fields from store queries |

---

## Task 1: Install graph dependencies

**Files:**
- Modify: `apps/dashboard/package.json`

- [ ] **Step 1: Install react-force-graph + three**

```bash
cd apps/dashboard && npm install react-force-graph-2d react-force-graph-3d three && npm install -D @types/three
```

- [ ] **Step 2: Verify install succeeded**

```bash
cd apps/dashboard && node -e "require('react-force-graph-2d'); require('react-force-graph-3d'); console.log('OK')"
```

Expected: `OK` (no errors)

- [ ] **Step 3: Verify dev server still starts**

```bash
cd apps/dashboard && npx vite build 2>&1 | tail -5
```

Expected: Build completes without errors.

- [ ] **Step 4: Commit**

```bash
git add apps/dashboard/package.json apps/dashboard/package-lock.json
git commit -m "chore: add react-force-graph-2d/3d + three for I2 dashboard graphs"
```

---

## Task 2: Extend types for correction + healthz

**Files:**
- Modify: `apps/dashboard/app/lib/types.ts`

- [ ] **Step 1: Add CorrectMemoryRequest and CorrectMemoryResponse types**

Add after `RepairMemoryRequest` (line 301):

```typescript
export interface CorrectMemoryRequest {
  id: string;
  content: string;
  reason?: string;
  tags?: string[];
  confidence?: number;
}

export interface CorrectMemoryResponse {
  old_item: MemoryItem;
  new_item: MemoryItem;
  contested: string[];
}
```

- [ ] **Step 2: Add PressureMetrics and extend HealthResponse**

Replace the existing `HealthResponse` (line 311-314):

```typescript
export interface PressureMetrics {
  inbox: number;
  candidates: number;
  stale: number;
  expired: number;
}

export interface HealthResponse {
  status: string;
  items: number;
  eval_score?: number;
  pressure?: PressureMetrics;
}
```

- [ ] **Step 3: Add AgentProfile type**

Add after `HiveTaskRecord` (line 230):

```typescript
export interface AgentProfile {
  agent: string;
  project?: string;
  namespace?: string;
  capabilities?: string[];
  preferences?: Record<string, string>;
  created_at: string;
  updated_at: string;
}

export interface AgentProfileResponse {
  profile?: AgentProfile;
}

export interface SourceMemoryResponse {
  items: MemoryItem[];
}
```

- [ ] **Step 4: Commit**

```bash
git add apps/dashboard/app/lib/types.ts
git commit -m "feat(dashboard): add correction, pressure, and agent profile types"
```

---

## Task 3: Extend API client + query hooks

**Files:**
- Modify: `apps/dashboard/app/lib/api.ts`
- Modify: `apps/dashboard/app/lib/queries.ts`

- [ ] **Step 1: Add correction endpoint to api.ts**

Add after the `repair` entry (line 109):

```typescript
  correct: (req: CorrectMemoryRequest) =>
    post<CorrectMemoryResponse>("/memory/correct", req),
```

Add the imports for `CorrectMemoryRequest`, `CorrectMemoryResponse`, `AgentProfileResponse`, `SourceMemoryResponse` to the import block at line 1.

- [ ] **Step 2: Add profile and source endpoints to api.ts**

Add after `entitySearch` (line 123):

```typescript
  profile: (params: { agent: string; project?: string }) =>
    get<AgentProfileResponse>(
      "/memory/profile",
      params as Record<string, string>,
    ),

  source: (params: { source_agent?: string; project?: string }) =>
    get<SourceMemoryResponse>(
      "/memory/source",
      params as Record<string, string>,
    ),
```

- [ ] **Step 3: Add atlasTrailsSave to api.ts**

Add after `atlasTrails` (line 143):

```typescript
  atlasTrailsSave: (req: { name: string; node_ids: string[] }) =>
    post<{ id: string }>("/atlas/trails/save", req),
```

- [ ] **Step 4: Add useCorrect mutation to queries.ts**

Add after `useRepair` (line 214):

```typescript
export function useCorrect() {
  const invalidate = useInvalidate(["search"], ["working"], ["inbox"]);
  return useMutation({
    mutationFn: (req: CorrectMemoryRequest) => api.correct(req),
    onSuccess: invalidate,
  });
}
```

Add `CorrectMemoryRequest` to the imports at line 15.

- [ ] **Step 5: Add useProfile, useSource, useAtlasExpand hooks to queries.ts**

Add after the existing query hooks:

```typescript
export function useProfile(agent: string, project?: string) {
  return useQuery({
    queryKey: ["profile", agent, project] as const,
    queryFn: () => api.profile({ agent, project }),
    enabled: !!agent,
  });
}

export function useSource(params?: { source_agent?: string; project?: string }) {
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
```

- [ ] **Step 6: Verify build**

```bash
cd apps/dashboard && npx tsc --noEmit 2>&1 | head -20
```

Expected: No type errors.

- [ ] **Step 7: Commit**

```bash
git add apps/dashboard/app/lib/api.ts apps/dashboard/app/lib/queries.ts
git commit -m "feat(dashboard): wire correction, profile, source, and atlas expand endpoints"
```

---

## Task 4: Shared graph components

**Files:**
- Create: `apps/dashboard/app/components/graph/graph-constants.ts`
- Create: `apps/dashboard/app/components/graph/use-graph-mode.ts`
- Create: `apps/dashboard/app/components/graph/graph-toggle.tsx`
- Create: `apps/dashboard/app/components/graph/force-graph-wrapper.tsx`
- Create: `apps/dashboard/app/components/graph/entity-detail.tsx`

- [ ] **Step 1: Create graph constants (supermemory E2-D3 donor)**

```typescript
// apps/dashboard/app/components/graph/graph-constants.ts

/** Force config from supermemory E2-D3 donor extraction */
export const FORCE_CONFIG = {
  charge: -2000,
  alphaDecay: 0.025,
  warmupTicks: 150,
} as const;

/** Node radius by type */
export function nodeRadius(node: { type?: string; nodeCount?: number; confidence?: number }) {
  if (node.type === "region") return Math.max(12, Math.min(40, (node.nodeCount ?? 1) * 2));
  return 4 + (node.confidence ?? 0.5) * 8;
}

/** Kind → color mapping (matches badge.tsx kindColors) */
export const KIND_COLORS: Record<string, string> = {
  fact: "#a855f7",
  decision: "#8b5cf6",
  preference: "#6366f1",
  runbook: "#0ea5e9",
  procedural: "#10b981",
  self_model: "#f59e0b",
  topology: "#06b6d4",
  status: "#71717a",
  live_truth: "#f43f5e",
  pattern: "#d946ef",
  constraint: "#ef4444",
  region: "#8b5cf6",
};

/** Relation → color mapping */
export const RELATION_COLORS: Record<string, string> = {
  same_as: "#8b5cf6",
  derived_from: "#06b6d4",
  supersedes: "#f59e0b",
  contradicts: "#ef4444",
  related: "#555570",
};
```

- [ ] **Step 2: Create useGraphMode hook**

```typescript
// apps/dashboard/app/components/graph/use-graph-mode.ts
import { useState, useCallback } from "react";

export type GraphMode = "2d" | "3d";

const STORAGE_KEY = "memd-graph-mode";

function readMode(): GraphMode {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    return v === "3d" ? "3d" : "2d";
  } catch {
    return "2d";
  }
}

export function useGraphMode() {
  const [mode, setModeState] = useState<GraphMode>(readMode);

  const setMode = useCallback((m: GraphMode) => {
    setModeState(m);
    try { localStorage.setItem(STORAGE_KEY, m); } catch { /* noop */ }
  }, []);

  const toggle = useCallback(() => {
    setModeState((prev) => {
      const next = prev === "2d" ? "3d" : "2d";
      try { localStorage.setItem(STORAGE_KEY, next); } catch { /* noop */ }
      return next;
    });
  }, []);

  return { mode, setMode, toggle } as const;
}
```

- [ ] **Step 3: Create GraphToggle component**

```tsx
// apps/dashboard/app/components/graph/graph-toggle.tsx
import type { GraphMode } from "./use-graph-mode";

export function GraphToggle({
  mode,
  onToggle,
}: {
  mode: GraphMode;
  onToggle: () => void;
}) {
  return (
    <button
      onClick={onToggle}
      className="px-3 py-1.5 rounded-lg text-xs font-medium border border-border-subtle bg-bg-surface/60 text-text-secondary hover:border-border-active hover:text-accent-bright transition-colors"
    >
      {mode === "2d" ? "Switch to 3D" : "Switch to 2D"}
    </button>
  );
}
```

- [ ] **Step 4: Create ForceGraphWrapper**

```tsx
// apps/dashboard/app/components/graph/force-graph-wrapper.tsx
import { useRef, useCallback, useEffect } from "react";
import ForceGraph2DComp from "react-force-graph-2d";
import ForceGraph3DComp from "react-force-graph-3d";
import type { GraphMode } from "./use-graph-mode";
import { FORCE_CONFIG, nodeRadius, KIND_COLORS, RELATION_COLORS } from "./graph-constants";

interface GraphNode {
  id: string;
  name: string;
  kind?: string;
  type?: string;
  nodeCount?: number;
  confidence?: number;
}

interface GraphLink {
  source: string;
  target: string;
  relation?: string;
}

interface ForceGraphWrapperProps {
  mode: GraphMode;
  nodes: GraphNode[];
  links: GraphLink[];
  width: number;
  height: number;
  onNodeClick?: (node: GraphNode) => void;
}

export function ForceGraphWrapper({
  mode,
  nodes,
  links,
  width,
  height,
  onNodeClick,
}: ForceGraphWrapperProps) {
  const fgRef = useRef<any>(null);

  const graphData = { nodes, links };

  const nodeColor = useCallback(
    (node: GraphNode) => KIND_COLORS[node.kind ?? node.type ?? ""] ?? "#555570",
    [],
  );

  const linkColor = useCallback(
    (link: GraphLink) => RELATION_COLORS[link.relation ?? ""] ?? "#555570",
    [],
  );

  const nodeVal = useCallback(
    (node: GraphNode) => nodeRadius(node),
    [],
  );

  useEffect(() => {
    if (fgRef.current) {
      fgRef.current.d3Force("charge")?.strength(FORCE_CONFIG.charge);
    }
  }, [mode]);

  const commonProps = {
    ref: fgRef,
    graphData,
    width,
    height,
    nodeColor,
    nodeVal,
    nodeLabel: (n: GraphNode) => n.name,
    linkColor,
    linkDirectionalArrowLength: 4,
    linkDirectionalArrowRelPos: 1,
    onNodeClick: onNodeClick as any,
    warmupTicks: FORCE_CONFIG.warmupTicks,
    d3AlphaDecay: FORCE_CONFIG.alphaDecay,
  };

  // Static imports at module scope (see imports at top of file)
  if (mode === "3d") {
    return <ForceGraph3DComp {...commonProps} />;
  }

  return <ForceGraph2DComp {...commonProps} />;
}

export type { GraphNode, GraphLink };
```

- [ ] **Step 5: Create EntityDetail sidebar**

```tsx
// apps/dashboard/app/components/graph/entity-detail.tsx
import { useEntityLinks } from "../../lib/queries";
import { GlassPanel } from "../ui/glass-panel";
import { KindBadge, ConfidenceBar } from "../ui/badge";
import type { MemoryEntityRecord } from "../../lib/types";

export function EntityDetail({
  entity,
  onClose,
}: {
  entity: MemoryEntityRecord | null;
  onClose: () => void;
}) {
  const { data } = useEntityLinks(
    entity ? { entity_id: entity.id } : undefined,
  );

  if (!entity) return null;

  return (
    <div className="w-80 shrink-0 border-l border-border-subtle bg-bg-surface/60 overflow-y-auto">
      <div className="p-4 border-b border-border-subtle flex items-center justify-between">
        <h3 className="text-sm font-medium text-text-primary truncate">
          {entity.name}
        </h3>
        <button
          onClick={onClose}
          className="text-text-tertiary hover:text-text-primary text-xs"
        >
          ✕
        </button>
      </div>

      <div className="p-4 space-y-4">
        {entity.kind && <KindBadge kind={entity.kind as any} />}

        <div className="text-xs space-y-2">
          <div>
            <span className="text-text-tertiary uppercase tracking-wide">ID</span>
            <p className="font-mono text-text-secondary">{entity.id.slice(0, 12)}</p>
          </div>
          {entity.project && (
            <div>
              <span className="text-text-tertiary uppercase tracking-wide">Project</span>
              <p className="text-text-secondary">{entity.project}</p>
            </div>
          )}
        </div>

        {data && data.links.length > 0 && (
          <GlassPanel padding="sm">
            <p className="text-[11px] tracking-wide uppercase text-text-tertiary mb-2">
              Links ({data.links.length})
            </p>
            <div className="space-y-1.5">
              {data.links.map((link) => (
                <div key={link.id} className="flex items-center gap-2 text-xs">
                  <span className="text-accent-bright font-mono">
                    {(link.source_entity_id === entity.id
                      ? link.target_name
                      : link.source_name
                    )?.slice(0, 20) ?? "?"}
                  </span>
                  <span className="text-text-tertiary">{link.relation}</span>
                  <ConfidenceBar value={link.confidence} />
                </div>
              ))}
            </div>
          </GlassPanel>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 6: Verify build**

```bash
cd apps/dashboard && npx tsc --noEmit 2>&1 | head -20
```

Expected: No type errors.

- [ ] **Step 7: Commit**

```bash
git add apps/dashboard/app/components/graph/
git commit -m "feat(dashboard): add shared graph components — ForceGraphWrapper, toggle, entity detail"
```

---

## Task 5: CorrectionEditor shared component

**Files:**
- Create: `apps/dashboard/app/components/correction/correction-editor.tsx`

- [ ] **Step 1: Create CorrectionEditor**

```tsx
// apps/dashboard/app/components/correction/correction-editor.tsx
import { useState } from "react";
import { useCorrect } from "../../lib/queries";

export function CorrectionEditor({
  itemId,
  currentContent,
  onClose,
}: {
  itemId: string;
  currentContent: string;
  onClose: () => void;
}) {
  const [content, setContent] = useState(currentContent);
  const [reason, setReason] = useState("");
  const correct = useCorrect();

  const canSubmit = content.trim().length > 0 && content !== currentContent;

  return (
    <div className="mt-3 p-4 rounded-lg border border-border-active bg-bg-primary/80 space-y-3">
      <p className="text-xs font-medium text-accent-bright uppercase tracking-wide">
        Correct this memory
      </p>

      <textarea
        value={content}
        onChange={(e) => setContent(e.target.value)}
        rows={4}
        className="w-full px-3 py-2 rounded-lg bg-bg-surface border border-border-subtle text-sm text-text-primary placeholder:text-text-tertiary focus:outline-none focus:border-accent-primary resize-y"
      />

      <input
        type="text"
        value={reason}
        onChange={(e) => setReason(e.target.value)}
        placeholder="Reason for correction (optional)"
        className="w-full px-3 py-2 rounded-lg bg-bg-surface border border-border-subtle text-sm text-text-primary placeholder:text-text-tertiary focus:outline-none focus:border-accent-primary"
      />

      <div className="flex gap-2">
        <button
          disabled={!canSubmit || correct.isPending}
          onClick={() => {
            correct.mutate(
              { id: itemId, content: content.trim(), reason: reason || undefined },
              {
                onSuccess: () => {
                  setContent(currentContent);
                  setReason("");
                  onClose();
                },
              },
            );
          }}
          className="px-4 py-1.5 rounded-lg text-xs font-medium bg-accent-primary/20 text-accent-bright border border-accent-primary/40 hover:bg-accent-primary/30 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
        >
          {correct.isPending ? "Saving..." : "Save Correction"}
        </button>
        <button
          onClick={onClose}
          className="px-4 py-1.5 rounded-lg text-xs font-medium text-text-tertiary border border-border-subtle hover:border-border-active transition-colors"
        >
          Cancel
        </button>
      </div>

      {correct.isError && (
        <p className="text-xs text-status-expired">
          {(correct.error as Error).message}
        </p>
      )}
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add apps/dashboard/app/components/correction/
git commit -m "feat(dashboard): add CorrectionEditor shared component"
```

---

## Task 6: Backend — extend healthz with pressure metrics

**Files:**
- Modify: `crates/memd-schema/src/lib.rs:2217-2220`
- Modify: `crates/memd-server/src/routes.rs:3-10`

- [ ] **Step 1: Extend HealthResponse in schema**

In `crates/memd-schema/src/lib.rs`, replace the HealthResponse struct (line 2217-2220):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureMetrics {
    pub inbox: usize,
    pub candidates: usize,
    pub stale: usize,
    pub expired: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub items: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_score: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pressure: Option<PressureMetrics>,
}
```

- [ ] **Step 2: Update healthz handler to populate pressure**

In `crates/memd-server/src/routes.rs`, replace the healthz function (lines 3-11):

```rust
pub(crate) async fn healthz(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, (StatusCode, String)> {
    let items = state.store.count().map_err(internal_error)?;

    // Compute pressure metrics from store
    let inbox_items = state.store.inbox_items(None, None).unwrap_or_default();
    let all_items = state.store.list().unwrap_or_default();
    let candidates = all_items.iter().filter(|i| i.stage == memd_schema::MemoryStage::Candidate).count();
    let stale = all_items.iter().filter(|i| i.status == memd_schema::MemoryStatus::Stale).count();
    let expired = all_items.iter().filter(|i| i.status == memd_schema::MemoryStatus::Expired).count();

    Ok(Json(HealthResponse {
        status: "ok".to_string(),
        items,
        eval_score: None, // populated when eval is run
        pressure: Some(PressureMetrics {
            inbox: inbox_items.len(),
            candidates,
            stale,
            expired,
        }),
    }))
}
```

- [ ] **Step 3: Add PressureMetrics import to main.rs**

In `crates/memd-server/src/main.rs`, add `PressureMetrics` to the existing `HealthResponse` import line.

- [ ] **Step 4: Run server tests**

```bash
cargo test -p memd-server 2>&1 | tail -10
```

Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/memd-schema/src/lib.rs crates/memd-server/src/routes.rs crates/memd-server/src/main.rs
git commit -m "feat(server): extend healthz with pressure metrics for dashboard"
```

---

## Task 7: Status page — add eval score + pressure panel

**Files:**
- Modify: `apps/dashboard/app/routes/index.tsx`

- [ ] **Step 1: Add pressure metrics row after the existing metrics grid**

After the existing `grid grid-cols-4` div (line 75), add a new conditional section:

```tsx
      {/* Pressure metrics */}
      {h?.pressure && (
        <div className="grid grid-cols-4 gap-4">
          <MetricCard
            label="Eval Score"
            value={h.eval_score != null ? `${Math.round(h.eval_score)}` : "—"}
            color={
              h.eval_score == null
                ? "text-text-tertiary"
                : h.eval_score >= 80
                  ? "text-status-current"
                  : h.eval_score >= 50
                    ? "text-status-stale"
                    : "text-status-expired"
            }
          />
          <MetricCard
            label="Candidates"
            value={h.pressure.candidates}
            color="text-status-candidate"
          />
          <MetricCard
            label="Stale"
            value={h.pressure.stale}
            color="text-status-stale"
          />
          <MetricCard
            label="Expired"
            value={h.pressure.expired}
            color="text-status-expired"
          />
        </div>
      )}
```

- [ ] **Step 2: Verify build**

```bash
cd apps/dashboard && npx tsc --noEmit 2>&1 | head -20
```

Expected: No type errors.

- [ ] **Step 3: Commit**

```bash
git add apps/dashboard/app/routes/index.tsx
git commit -m "feat(dashboard): add eval score + pressure metrics to Status page"
```

---

## Task 8: Memory page — add correction flow

**Files:**
- Modify: `apps/dashboard/app/routes/memory.tsx`

- [ ] **Step 1: Add correction state + import CorrectionEditor**

Add import at top of file:

```typescript
import { CorrectionEditor } from "../components/correction/correction-editor";
```

Add state for correction in `MemoryBrowser` function (after `expanded` state, line 35):

```typescript
const [correcting, setCorrecting] = useState<string | null>(null);
```

- [ ] **Step 2: Add Correct button + CorrectionEditor in expanded MemoryRow**

In the `MemoryRow` expanded section (after the supersedes div, around line 190), add:

```tsx
          {/* Actions */}
          <div className="flex gap-2 pt-2">
            <button
              onClick={(e) => {
                e.stopPropagation();
                setCorrecting(correcting === item.id ? null : item.id);
              }}
              className="px-3 py-1.5 rounded-lg text-xs font-medium text-accent-bright border border-accent-primary/30 hover:bg-accent-primary/10 transition-colors"
            >
              Correct
            </button>
          </div>

          {/* Correction editor */}
          {correcting === item.id && (
            <CorrectionEditor
              itemId={item.id}
              currentContent={item.content}
              onClose={() => setCorrecting(null)}
            />
          )}
```

Note: `correcting` and `setCorrecting` must be passed down from `MemoryBrowser` to `MemoryRow`. Update the `MemoryRow` props to include:

```typescript
function MemoryRow({
  item,
  isExpanded,
  onToggle,
  correcting,
  setCorrecting,
}: {
  item: MemoryItem;
  isExpanded: boolean;
  onToggle: () => void;
  correcting: string | null;
  setCorrecting: (id: string | null) => void;
}) {
```

And update the caller in `MemoryBrowser`:

```tsx
<MemoryRow
  key={item.id}
  item={item}
  isExpanded={expanded === item.id}
  onToggle={() => setExpanded(expanded === item.id ? null : item.id)}
  correcting={correcting}
  setCorrecting={setCorrecting}
/>
```

- [ ] **Step 3: Verify build**

```bash
cd apps/dashboard && npx tsc --noEmit 2>&1 | head -20
```

Expected: No type errors.

- [ ] **Step 4: Commit**

```bash
git add apps/dashboard/app/routes/memory.tsx
git commit -m "feat(dashboard): add correction flow to Memory browser"
```

---

## Task 9: Atlas page — region graph with 2D/3D toggle

**Files:**
- Modify: `apps/dashboard/app/routes/atlas.tsx` (full rewrite)

- [ ] **Step 1: Implement Atlas page**

Replace entire content of `apps/dashboard/app/routes/atlas.tsx`:

```tsx
import { createFileRoute } from "@tanstack/react-router";
import { useState, useCallback, useRef, useEffect } from "react";
import { useAtlasRegions, useAtlasExplore, useAtlasExpand } from "../lib/queries";
import { GlassPanel } from "../components/ui/glass-panel";
import { EmptyState } from "../components/ui/empty-state";
import { GraphToggle } from "../components/graph/graph-toggle";
import { ForceGraphWrapper } from "../components/graph/force-graph-wrapper";
import { useGraphMode } from "../components/graph/use-graph-mode";
import type { GraphNode, GraphLink } from "../components/graph/force-graph-wrapper";
import type { AtlasRegion } from "../lib/types";

export const Route = createFileRoute("/atlas")({
  component: AtlasPage,
});

function AtlasPage() {
  const { mode, toggle } = useGraphMode();
  const [selectedRegion, setSelectedRegion] = useState<string | null>(null);
  const [selectedEntity, setSelectedEntity] = useState<string | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [dims, setDims] = useState({ width: 800, height: 600 });

  const regions = useAtlasRegions();
  const explore = useAtlasExplore(selectedRegion ?? "");

  // Measure container
  useEffect(() => {
    if (!containerRef.current) return;
    const obs = new ResizeObserver((entries) => {
      const { width, height } = entries[0].contentRect;
      setDims({ width, height: Math.max(400, height) });
    });
    obs.observe(containerRef.current);
    return () => obs.disconnect();
  }, []);

  // Build graph data from regions or region detail
  const { nodes, links } = buildGraphData(regions.data?.regions, explore.data, selectedRegion);

  const handleNodeClick = useCallback(
    (node: GraphNode) => {
      if (node.type === "region") {
        setSelectedRegion(node.id);
        setSelectedEntity(null);
      } else {
        setSelectedEntity(node.id);
      }
    },
    [],
  );

  return (
    <div className="flex flex-col h-screen">
      {/* Toolbar */}
      <div className="flex items-center justify-between px-8 py-4 border-b border-border-subtle">
        <div className="flex items-center gap-4">
          <h1 className="text-2xl font-semibold tracking-tight">Atlas</h1>
          {selectedRegion && explore.data && (
            <>
              <span className="text-text-tertiary">›</span>
              <span className="text-sm text-accent-bright">{explore.data.region.name}</span>
              <button
                onClick={() => { setSelectedRegion(null); setSelectedEntity(null); }}
                className="text-xs text-text-tertiary hover:text-text-primary transition-colors"
              >
                ← Back to regions
              </button>
            </>
          )}
        </div>
        <GraphToggle mode={mode} onToggle={toggle} />
      </div>

      {/* Graph area */}
      <div ref={containerRef} className="flex-1 relative">
        {regions.isLoading && (
          <div className="absolute inset-0 flex items-center justify-center text-text-tertiary">Loading...</div>
        )}
        {regions.data && nodes.length === 0 && (
          <EmptyState title="No atlas regions" description="Store more memories to generate regions" />
        )}
        {nodes.length > 0 && (
          <ForceGraphWrapper
            mode={mode}
            nodes={nodes}
            links={links}
            width={dims.width}
            height={dims.height}
            onNodeClick={handleNodeClick}
          />
        )}
      </div>

      {/* Region sidebar */}
      {selectedRegion && explore.data && (
        <RegionSidebar
          region={explore.data.region}
          nodeCount={explore.data.nodes.length}
          onClose={() => setSelectedRegion(null)}
        />
      )}
    </div>
  );
}

function buildGraphData(
  regions?: AtlasRegion[],
  exploreData?: { region: AtlasRegion; nodes: any[]; links: any[] },
  selectedRegion?: string | null,
): { nodes: GraphNode[]; links: GraphLink[] } {
  // Drill-down mode: show entities within region
  if (selectedRegion && exploreData) {
    const nodes: GraphNode[] = exploreData.nodes.map((n) => ({
      id: n.entity_id,
      name: n.name || `Entity ${n.entity_id.slice(0, 8)}`,
      kind: n.kind,
      type: "entity",
      confidence: n.confidence,
    }));
    const links: GraphLink[] = exploreData.links.map((l) => ({
      source: l.source_entity_id,
      target: l.target_entity_id,
      relation: l.relation,
    }));
    return { nodes, links };
  }

  // Top level: show regions as nodes
  if (!regions) return { nodes: [], links: [] };

  const nodes: GraphNode[] = regions.map((r) => ({
    id: r.id,
    name: r.name,
    kind: "region",
    type: "region",
    nodeCount: r.node_count,
    confidence: 1,
  }));

  return { nodes, links: [] };
}

function RegionSidebar({
  region,
  nodeCount,
  onClose,
}: {
  region: AtlasRegion;
  nodeCount: number;
  onClose: () => void;
}) {
  return (
    <div className="absolute right-0 top-0 w-72 h-full border-l border-border-subtle bg-bg-surface/90 overflow-y-auto">
      <div className="p-4 border-b border-border-subtle flex items-center justify-between">
        <h3 className="text-sm font-medium truncate">{region.name}</h3>
        <button onClick={onClose} className="text-text-tertiary hover:text-text-primary text-xs">✕</button>
      </div>
      <div className="p-4 space-y-3 text-xs">
        {region.description && <p className="text-text-secondary">{region.description}</p>}
        <div>
          <span className="text-text-tertiary uppercase tracking-wide">Entities</span>
          <p className="text-text-primary text-lg font-semibold">{nodeCount}</p>
        </div>
        {region.tags.length > 0 && (
          <div className="flex flex-wrap gap-1">
            {region.tags.map((t) => (
              <span key={t} className="px-2 py-0.5 rounded text-[11px] bg-white/5 text-text-tertiary border border-white/5">
                {t}
              </span>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Verify build**

```bash
cd apps/dashboard && npx tsc --noEmit 2>&1 | head -20
```

- [ ] **Step 3: Commit**

```bash
git add apps/dashboard/app/routes/atlas.tsx
git commit -m "feat(dashboard): implement Atlas page with 2D/3D force graph"
```

---

## Task 10: Graph page — entity relationship explorer

**Files:**
- Modify: `apps/dashboard/app/routes/graph.tsx` (full rewrite)

- [ ] **Step 1: Implement Graph page**

Replace entire content of `apps/dashboard/app/routes/graph.tsx`:

```tsx
import { createFileRoute } from "@tanstack/react-router";
import { useState, useCallback, useRef, useEffect } from "react";
import { useEntitySearch, useEntityLinks } from "../lib/queries";
import { EmptyState } from "../components/ui/empty-state";
import { GraphToggle } from "../components/graph/graph-toggle";
import { ForceGraphWrapper } from "../components/graph/force-graph-wrapper";
import { EntityDetail } from "../components/graph/entity-detail";
import { useGraphMode } from "../components/graph/use-graph-mode";
import type { GraphNode, GraphLink } from "../components/graph/force-graph-wrapper";
import type { MemoryEntityRecord, EntityRelationKind } from "../lib/types";

export const Route = createFileRoute("/graph")({
  component: GraphPage,
});

const ALL_RELATIONS: EntityRelationKind[] = [
  "same_as", "derived_from", "supersedes", "contradicts", "related",
];

function GraphPage() {
  const { mode, toggle } = useGraphMode();
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedEntity, setSelectedEntity] = useState<MemoryEntityRecord | null>(null);
  const [visibleRelations, setVisibleRelations] = useState<Set<EntityRelationKind>>(new Set(ALL_RELATIONS));
  const containerRef = useRef<HTMLDivElement>(null);
  const [dims, setDims] = useState({ width: 800, height: 600 });

  const search = useEntitySearch(searchQuery);
  const links = useEntityLinks();

  // Measure container
  useEffect(() => {
    if (!containerRef.current) return;
    const obs = new ResizeObserver((entries) => {
      const { width, height } = entries[0].contentRect;
      setDims({ width, height: Math.max(400, height) });
    });
    obs.observe(containerRef.current);
    return () => obs.disconnect();
  }, []);

  // Build nodes from search results + link endpoints
  const { nodes, graphLinks } = buildEntityGraph(
    search.data?.entities ?? [],
    links.data?.links ?? [],
    visibleRelations,
  );

  const handleNodeClick = useCallback(
    (node: GraphNode) => {
      const entity = search.data?.entities.find((e) => e.id === node.id);
      if (entity) setSelectedEntity(entity);
    },
    [search.data],
  );

  const toggleRelation = (rel: EntityRelationKind) => {
    setVisibleRelations((prev) => {
      const next = new Set(prev);
      if (next.has(rel)) next.delete(rel);
      else next.add(rel);
      return next;
    });
  };

  return (
    <div className="flex h-screen">
      <div className="flex-1 flex flex-col">
        {/* Toolbar */}
        <div className="flex items-center gap-4 px-8 py-4 border-b border-border-subtle">
          <h1 className="text-2xl font-semibold tracking-tight">Graph</h1>

          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search entities..."
            className="flex-1 max-w-md px-4 py-2 rounded-lg bg-bg-primary border border-border-subtle text-sm text-text-primary placeholder:text-text-tertiary focus:outline-none focus:border-accent-primary transition-colors"
          />

          {/* Relation filters */}
          <div className="flex gap-1">
            {ALL_RELATIONS.map((rel) => (
              <button
                key={rel}
                onClick={() => toggleRelation(rel)}
                className={`px-2 py-1 rounded text-[11px] border transition-colors ${
                  visibleRelations.has(rel)
                    ? "bg-accent-primary/15 text-accent-bright border-accent-primary/40"
                    : "bg-glass text-text-tertiary border-border-subtle"
                }`}
              >
                {rel.replace(/_/g, " ")}
              </button>
            ))}
          </div>

          <GraphToggle mode={mode} onToggle={toggle} />
        </div>

        {/* Graph area */}
        <div ref={containerRef} className="flex-1 relative">
          {nodes.length === 0 && (
            <EmptyState
              title={searchQuery ? "No entities found" : "Search for entities"}
              description="Type an entity name to explore relationships"
            />
          )}
          {nodes.length > 0 && (
            <ForceGraphWrapper
              mode={mode}
              nodes={nodes}
              links={graphLinks}
              width={dims.width - (selectedEntity ? 320 : 0)}
              height={dims.height}
              onNodeClick={handleNodeClick}
            />
          )}
        </div>
      </div>

      {/* Entity detail sidebar */}
      <EntityDetail
        entity={selectedEntity}
        onClose={() => setSelectedEntity(null)}
      />
    </div>
  );
}

function buildEntityGraph(
  entities: MemoryEntityRecord[],
  allLinks: { source_entity_id: string; target_entity_id: string; relation: EntityRelationKind }[],
  visibleRelations: Set<EntityRelationKind>,
): { nodes: GraphNode[]; graphLinks: GraphLink[] } {
  const entityIds = new Set(entities.map((e) => e.id));

  // Include entities that are link endpoints of our search results
  const relevantLinks = allLinks.filter(
    (l) =>
      visibleRelations.has(l.relation) &&
      (entityIds.has(l.source_entity_id) || entityIds.has(l.target_entity_id)),
  );

  // Collect all relevant node IDs
  const allIds = new Set(entityIds);
  for (const l of relevantLinks) {
    allIds.add(l.source_entity_id);
    allIds.add(l.target_entity_id);
  }

  const nodes: GraphNode[] = [];
  for (const id of allIds) {
    const entity = entities.find((e) => e.id === id);
    nodes.push({
      id,
      name: entity?.name ?? id.slice(0, 8),
      kind: entity?.kind ?? undefined,
      type: "entity",
      confidence: 0.7,
    });
  }

  const graphLinks: GraphLink[] = relevantLinks.map((l) => ({
    source: l.source_entity_id,
    target: l.target_entity_id,
    relation: l.relation,
  }));

  return { nodes, graphLinks };
}
```

- [ ] **Step 2: Verify build**

```bash
cd apps/dashboard && npx tsc --noEmit 2>&1 | head -20
```

- [ ] **Step 3: Commit**

```bash
git add apps/dashboard/app/routes/graph.tsx
git commit -m "feat(dashboard): implement Graph page — entity relationship explorer with 2D/3D"
```

---

## Task 11: Procedures page — tabbed list with actions

**Files:**
- Modify: `apps/dashboard/app/routes/procedures.tsx` (full rewrite)

- [ ] **Step 1: Implement Procedures page**

Replace entire content of `apps/dashboard/app/routes/procedures.tsx`:

```tsx
import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { useProcedures, useProcedurePromote, useProcedureRetire } from "../lib/queries";
import { GlassPanel } from "../components/ui/glass-panel";
import { ProcedureStatusBadge, KindBadge, ConfidenceBar } from "../components/ui/badge";
import { EmptyState } from "../components/ui/empty-state";
import type { Procedure, ProcedureStatus, ProcedureKind } from "../lib/types";

export const Route = createFileRoute("/procedures")({
  component: ProceduresPage,
});

const TABS: { label: string; status: ProcedureStatus }[] = [
  { label: "Promoted", status: "promoted" },
  { label: "Candidate", status: "candidate" },
  { label: "Retired", status: "retired" },
];

const kindMap: Record<ProcedureKind, string> = {
  workflow: "bg-emerald-500/15 text-emerald-300 border-emerald-500/30",
  policy: "bg-sky-500/15 text-sky-300 border-sky-500/30",
  recovery: "bg-amber-500/15 text-amber-300 border-amber-500/30",
};

function ProceduresPage() {
  const [tab, setTab] = useState<ProcedureStatus>("promoted");
  const [expanded, setExpanded] = useState<string | null>(null);
  const { data, isLoading } = useProcedures({ status: tab });
  const promote = useProcedurePromote();
  const retire = useProcedureRetire();

  return (
    <div className="p-8 max-w-6xl space-y-6">
      <h1 className="text-2xl font-semibold tracking-tight">Procedures</h1>

      {/* Tabs */}
      <div className="flex gap-1">
        {TABS.map((t) => (
          <button
            key={t.status}
            onClick={() => { setTab(t.status); setExpanded(null); }}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
              tab === t.status
                ? "bg-accent-primary/15 text-accent-bright border border-accent-primary/40"
                : "text-text-tertiary hover:text-text-secondary border border-transparent"
            }`}
          >
            {t.label}
          </button>
        ))}
      </div>

      {/* List */}
      <GlassPanel padding="none">
        {isLoading && (
          <div className="p-8 text-center text-text-tertiary text-sm">Loading...</div>
        )}
        {data && data.procedures.length === 0 && (
          <EmptyState title={`No ${tab} procedures`} />
        )}
        {data?.procedures.map((proc) => (
          <ProcedureRow
            key={proc.id}
            procedure={proc}
            isExpanded={expanded === proc.id}
            onToggle={() => setExpanded(expanded === proc.id ? null : proc.id)}
            onPromote={() => promote.mutate({ id: proc.id })}
            onRetire={() => retire.mutate({ id: proc.id })}
          />
        ))}
      </GlassPanel>
    </div>
  );
}

function ProcedureRow({
  procedure: p,
  isExpanded,
  onToggle,
  onPromote,
  onRetire,
}: {
  procedure: Procedure;
  isExpanded: boolean;
  onToggle: () => void;
  onPromote: () => void;
  onRetire: () => void;
}) {
  return (
    <div className="group">
      <button
        onClick={onToggle}
        className="w-full flex items-center gap-3 px-5 py-3 text-left hover:bg-hover transition-colors border-b border-border-subtle last:border-0"
      >
        <span className={`inline-flex items-center px-2 py-0.5 rounded text-[11px] font-medium border ${kindMap[p.kind]}`}>
          {p.kind}
        </span>
        <ProcedureStatusBadge status={p.status} />
        <span className="flex-1 truncate text-sm text-text-primary">{p.name}</span>
        <span className="text-xs text-text-tertiary tabular-nums">{p.use_count} uses</span>
        <div className="w-20">
          <ConfidenceBar value={p.confidence} />
        </div>
      </button>

      {isExpanded && (
        <div className="px-5 pb-5 space-y-4 border-b border-border-subtle bg-hover">
          {p.description && (
            <p className="text-sm text-text-secondary pt-3">{p.description}</p>
          )}

          {p.trigger && (
            <div className="text-xs">
              <span className="text-text-tertiary uppercase tracking-wide">Trigger: </span>
              <span className="text-text-secondary">{p.trigger}</span>
            </div>
          )}

          {/* Steps */}
          {p.steps.length > 0 && (
            <div>
              <p className="text-[11px] tracking-wide uppercase text-text-tertiary mb-2">Steps</p>
              <ol className="list-decimal list-inside space-y-1 text-sm text-text-secondary">
                {p.steps.map((step, i) => (
                  <li key={i}>{step}</li>
                ))}
              </ol>
            </div>
          )}

          {p.success_criteria && (
            <div className="text-xs">
              <span className="text-text-tertiary uppercase tracking-wide">Success: </span>
              <span className="text-text-secondary">{p.success_criteria}</span>
            </div>
          )}

          {p.tags.length > 0 && (
            <div className="flex flex-wrap gap-1">
              {p.tags.map((t) => (
                <span key={t} className="px-2 py-0.5 rounded text-[11px] bg-white/5 text-text-tertiary border border-white/5">
                  {t}
                </span>
              ))}
            </div>
          )}

          {/* Source traces */}
          {p.source_ids.length > 0 && (
            <div>
              <p className="text-[11px] tracking-wide uppercase text-text-tertiary mb-1">Sources</p>
              <div className="flex flex-wrap gap-1.5">
                {p.source_ids.map((id) => (
                  <span key={id} className="font-mono text-[11px] text-accent-bright">
                    {id.slice(0, 12)}
                  </span>
                ))}
              </div>
            </div>
          )}

          {/* Actions */}
          <div className="flex gap-2 pt-2">
            {p.status === "candidate" && (
              <button
                onClick={onPromote}
                className="px-3 py-1.5 rounded-lg text-xs font-medium bg-emerald-500/15 text-emerald-300 border border-emerald-500/30 hover:bg-emerald-500/25 transition-colors"
              >
                Promote
              </button>
            )}
            {p.status === "promoted" && (
              <button
                onClick={onRetire}
                className="px-3 py-1.5 rounded-lg text-xs font-medium bg-red-500/15 text-red-300 border border-red-500/30 hover:bg-red-500/25 transition-colors"
              >
                Retire
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 2: Verify build**

```bash
cd apps/dashboard && npx tsc --noEmit 2>&1 | head -20
```

- [ ] **Step 3: Commit**

```bash
git add apps/dashboard/app/routes/procedures.tsx
git commit -m "feat(dashboard): implement Procedures page with tabs + promote/retire"
```

---

## Task 12: Agent page — profile + sessions

**Files:**
- Modify: `apps/dashboard/app/routes/agent.tsx` (full rewrite)

- [ ] **Step 1: Implement Agent page**

Replace entire content of `apps/dashboard/app/routes/agent.tsx`:

```tsx
import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import {
  useSessions,
  useProfile,
  useWorking,
  useQueenDeny,
  useQueenReroute,
  useQueenHandoff,
} from "../lib/queries";
import { GlassPanel } from "../components/ui/glass-panel";
import { EmptyState } from "../components/ui/empty-state";
import type { HiveSessionRecord } from "../lib/types";

export const Route = createFileRoute("/agent")({
  component: AgentPage,
});

function AgentPage() {
  const sessions = useSessions();
  const [selected, setSelected] = useState<HiveSessionRecord | null>(null);

  const agents = sessions.data?.sessions ?? [];
  const agentName = selected?.effective_agent ?? selected?.agent ?? "";
  const profile = useProfile(agentName, selected?.project ?? undefined);
  const working = useWorking(selected?.project ? { project: selected.project } : undefined);

  const deny = useQueenDeny();
  const reroute = useQueenReroute();
  const handoff = useQueenHandoff();

  return (
    <div className="flex h-screen">
      {/* Agent list */}
      <div className="w-64 shrink-0 border-r border-border-subtle bg-bg-surface/60 overflow-y-auto">
        <div className="px-4 py-3 border-b border-border-subtle">
          <h2 className="text-sm font-medium text-text-secondary">Sessions</h2>
        </div>
        {agents.length === 0 && <EmptyState title="No active sessions" />}
        {agents.map((s) => (
          <button
            key={s.session}
            onClick={() => setSelected(s)}
            className={`w-full text-left px-4 py-3 border-b border-border-subtle hover:bg-hover transition-colors ${
              selected?.session === s.session ? "bg-accent-primary/10 border-l-2 border-l-accent-primary" : ""
            }`}
          >
            <p className="text-sm font-mono text-accent-bright truncate">
              {s.effective_agent ?? s.agent ?? s.session.slice(0, 12)}
            </p>
            <div className="flex items-center gap-2 mt-1">
              {s.hive_role && (
                <span className="text-[10px] uppercase tracking-wide text-text-tertiary">{s.hive_role}</span>
              )}
              <span className="text-[10px] uppercase tracking-wide text-text-tertiary">{s.status}</span>
            </div>
            {s.project && <p className="text-xs text-text-tertiary mt-0.5 truncate">{s.project}</p>}
          </button>
        ))}
      </div>

      {/* Detail */}
      <div className="flex-1 overflow-y-auto p-8 space-y-6">
        {!selected ? (
          <EmptyState title="Select an agent" description="Choose from the sidebar to view details" />
        ) : (
          <>
            <h1 className="text-2xl font-semibold tracking-tight">
              {selected.effective_agent ?? selected.agent ?? "Unknown Agent"}
            </h1>

            {/* Session info */}
            <GlassPanel>
              <h2 className="text-sm font-medium text-text-secondary mb-3">Session</h2>
              <div className="grid grid-cols-3 gap-3 text-xs">
                <Field label="Session ID" value={selected.session} mono />
                <Field label="Status" value={selected.status} />
                <Field label="Hive Role" value={selected.hive_role ?? "—"} />
                <Field label="Project" value={selected.project ?? "—"} />
                <Field label="Namespace" value={selected.namespace ?? "—"} />
                <Field label="Worker" value={selected.worker_name ?? "—"} />
                <Field label="Last Seen" value={selected.last_seen ?? "—"} />
                <Field label="Last Wake" value={selected.last_wake_at ?? "—"} />
                <Field label="Focus" value={selected.focus ?? "—"} />
              </div>
            </GlassPanel>

            {/* Profile */}
            {profile.data?.profile && (
              <GlassPanel>
                <h2 className="text-sm font-medium text-text-secondary mb-3">Profile</h2>
                <div className="text-xs space-y-2">
                  {profile.data.profile.capabilities && profile.data.profile.capabilities.length > 0 && (
                    <div>
                      <span className="text-text-tertiary uppercase tracking-wide">Capabilities</span>
                      <div className="flex flex-wrap gap-1 mt-1">
                        {profile.data.profile.capabilities.map((c) => (
                          <span key={c} className="px-2 py-0.5 rounded text-[11px] bg-white/5 text-text-tertiary border border-white/5">{c}</span>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              </GlassPanel>
            )}

            {/* Working memory summary */}
            {working.data && (
              <GlassPanel>
                <h2 className="text-sm font-medium text-text-secondary mb-3">Working Memory</h2>
                <div className="text-xs text-text-tertiary">
                  {working.data.records.length} items · {working.data.used_chars}/{working.data.budget_chars} chars
                  {working.data.truncated && " (truncated)"}
                </div>
              </GlassPanel>
            )}

            {/* Hive controls */}
            <GlassPanel>
              <h2 className="text-sm font-medium text-text-secondary mb-3">Hive Controls</h2>
              <div className="flex gap-2">
                <button
                  onClick={() => reroute.mutate({ session: selected.session })}
                  className="px-3 py-1.5 rounded-lg text-xs font-medium bg-sky-500/15 text-sky-300 border border-sky-500/30 hover:bg-sky-500/25 transition-colors"
                >
                  Reroute
                </button>
                <button
                  onClick={() => deny.mutate({ session: selected.session })}
                  className="px-3 py-1.5 rounded-lg text-xs font-medium bg-amber-500/15 text-amber-300 border border-amber-500/30 hover:bg-amber-500/25 transition-colors"
                >
                  Deny
                </button>
                <button
                  onClick={() => handoff.mutate({ session: selected.session })}
                  className="px-3 py-1.5 rounded-lg text-xs font-medium bg-emerald-500/15 text-emerald-300 border border-emerald-500/30 hover:bg-emerald-500/25 transition-colors"
                >
                  Handoff
                </button>
              </div>
            </GlassPanel>
          </>
        )}
      </div>
    </div>
  );
}

function Field({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div>
      <p className="text-[11px] tracking-wide uppercase text-text-tertiary mb-0.5">{label}</p>
      <p className={`text-text-secondary truncate ${mono ? "font-mono text-[11px]" : ""}`}>{value}</p>
    </div>
  );
}
```

- [ ] **Step 2: Verify build**

```bash
cd apps/dashboard && npx tsc --noEmit 2>&1 | head -20
```

- [ ] **Step 3: Commit**

```bash
git add apps/dashboard/app/routes/agent.tsx
git commit -m "feat(dashboard): implement Agent page with profile + session viewer + hive controls"
```

---

## Task 13: Ask page — search + explain

**Files:**
- Modify: `apps/dashboard/app/routes/ask.tsx` (full rewrite)

- [ ] **Step 1: Implement Ask page**

Replace entire content of `apps/dashboard/app/routes/ask.tsx`:

```tsx
import { createFileRoute } from "@tanstack/react-router";
import { useState } from "react";
import { useSearch, useExplain } from "../lib/queries";
import { GlassPanel } from "../components/ui/glass-panel";
import { KindBadge, StageBadge, StatusDot, ConfidenceBar, ScopeLabel } from "../components/ui/badge";
import { EmptyState } from "../components/ui/empty-state";
import type { MemoryItem, SearchMemoryRequest } from "../lib/types";

export const Route = createFileRoute("/ask")({
  component: AskPage,
});

function AskPage() {
  const [query, setQuery] = useState("");
  const [submitted, setSubmitted] = useState("");
  const [explaining, setExplaining] = useState<string | null>(null);

  const searchReq: SearchMemoryRequest = {
    query: submitted || undefined,
    scopes: ["local", "synced", "project", "global"],
    kinds: ["fact", "decision", "preference", "runbook", "procedural", "self_model", "topology", "live_truth", "pattern", "constraint"],
    statuses: ["active"],
    stages: ["canonical", "candidate"],
    tags: [],
    limit: 20,
  };

  const { data, isLoading } = useSearch(searchReq, !!submitted);
  const explain = useExplain(explaining ?? "");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitted(query.trim());
    setExplaining(null);
  };

  return (
    <div className="p-8 max-w-4xl mx-auto space-y-8">
      <h1 className="text-2xl font-semibold tracking-tight text-center">Ask memd</h1>

      {/* Search bar */}
      <form onSubmit={handleSubmit}>
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="What do you want to know?"
          className="w-full px-6 py-4 rounded-xl bg-bg-surface border border-border-subtle text-lg text-text-primary placeholder:text-text-tertiary focus:outline-none focus:border-accent-primary transition-colors"
          autoFocus
        />
      </form>

      {/* Results */}
      {isLoading && (
        <div className="text-center text-text-tertiary text-sm">Searching...</div>
      )}

      {data && data.items.length === 0 && submitted && (
        <EmptyState title="No results" description="Try different words or broaden your search" />
      )}

      {data && data.items.length > 0 && (
        <div className="space-y-3">
          {data.items.map((item) => (
            <AskResult
              key={item.id}
              item={item}
              isExplaining={explaining === item.id}
              explanation={explaining === item.id ? explain.data?.explanation : undefined}
              onExplain={() => setExplaining(explaining === item.id ? null : item.id)}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function AskResult({
  item,
  isExplaining,
  explanation,
  onExplain,
}: {
  item: MemoryItem;
  isExplaining: boolean;
  explanation?: string;
  onExplain: () => void;
}) {
  return (
    <GlassPanel hover>
      <div className="flex items-start gap-3">
        <div className="flex flex-wrap gap-1.5 shrink-0">
          <KindBadge kind={item.kind} />
          <StageBadge stage={item.stage} />
        </div>
        <div className="flex-1 min-w-0">
          <p className="text-sm text-text-primary leading-relaxed">{item.content}</p>
          <div className="flex items-center gap-3 mt-2">
            <StatusDot status={item.status} />
            <ScopeLabel value={item.scope} />
            <div className="w-24">
              <ConfidenceBar value={item.confidence} />
            </div>
            <button
              onClick={onExplain}
              className="ml-auto text-xs text-accent-bright hover:opacity-80 transition-opacity"
            >
              {isExplaining ? "Hide" : "Explain"}
            </button>
          </div>
        </div>
      </div>

      {isExplaining && explanation && (
        <div className="mt-3 pt-3 border-t border-border-subtle">
          <p className="text-xs text-text-secondary whitespace-pre-wrap">{explanation}</p>
        </div>
      )}
    </GlassPanel>
  );
}
```

- [ ] **Step 2: Verify build**

```bash
cd apps/dashboard && npx tsc --noEmit 2>&1 | head -20
```

- [ ] **Step 3: Commit**

```bash
git add apps/dashboard/app/routes/ask.tsx
git commit -m "feat(dashboard): implement Ask page with search + explain"
```

---

## Task 14: Browser verification

**Files:** None (testing only)

- [ ] **Step 1: Start memd server**

```bash
cd /home/josue/Documents/projects/memd && cargo run -p memd-server &
```

- [ ] **Step 2: Start dev server**

```bash
cd /home/josue/Documents/projects/memd/apps/dashboard && npm run dev &
```

- [ ] **Step 3: Test every page with agent-browser**

Open each route and verify:
- `http://localhost:5173/dashboard/` — Status page with eval score + pressure
- `http://localhost:5173/dashboard/memory` — Memory browser with Correct button
- `http://localhost:5173/dashboard/atlas` — Atlas graph renders (2D/3D toggle)
- `http://localhost:5173/dashboard/graph` — Entity graph renders
- `http://localhost:5173/dashboard/procedures` — Procedures list with tabs
- `http://localhost:5173/dashboard/agent` — Agent panel with session list
- `http://localhost:5173/dashboard/ask` — Ask page with search
- Zero console errors on ALL pages

- [ ] **Step 4: Test correction flow**

1. Navigate to Memory page
2. Search for a fact
3. Expand it, click "Correct"
4. Edit content, save
5. Verify the corrected item appears in results

- [ ] **Step 5: Test 2D/3D toggle**

1. Navigate to Atlas page
2. Click 3D toggle, verify graph switches
3. Click 2D toggle, verify graph switches back
4. Navigate to Graph page, verify toggle persists

- [ ] **Step 6: Fix any console errors found**

- [ ] **Step 7: Final commit**

```bash
git add -A
git commit -m "fix(dashboard): resolve browser verification issues"
```
