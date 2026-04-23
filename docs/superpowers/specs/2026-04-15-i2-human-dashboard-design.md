# I2: Human Dashboard — Design Spec

Supersedes: `2026-04-13-phase-i-human-dashboard-design.md` (V1 spec).

## Overview

Ship all 5 stub dashboard pages + add correction flow to Memory page + add eval score to Status page. The V1 scaffold gave us 3 working pages (Status, Memory, Inbox) and a full React 19 + TanStack Router + Tailwind 4 + Vite shell. I2 finishes the job.

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Graph lib | react-force-graph-2d + react-force-graph-3d | Same API, swap on toggle. D3 force under the hood. Supermemory force constants. |
| 2D/3D toggle | Google Maps/Earth style | User preference. Both renderers share data hook. Toggle persists in localStorage. |
| Force constants | supermemory donor (E2-D3) | charge=-2000, collision doc=70px mem=35px, alpha decay=0.025, 150 pre-settle ticks |
| Ask page | Search + explain (no LLM/OpenUI) | Simpler. No API key dependency. `/memory/search` + `/memory/explain` covers it. |
| Framework | Keep V1 stack | React 19, TanStack Router, TanStack Query, Tailwind 4, Vite 6 |
| Embedding | rust-embed (from V1 spec) | Static files baked into binary. Already designed. |

## What Exists (V1 Scaffold)

Working pages — no changes needed unless noted:
- **Status** (`index.tsx`) — health, working memory, sessions, harness health. **Needs**: eval score panel.
- **Memory** (`memory.tsx`) — search, filter, expandable items. **Needs**: correction button + flow.
- **Inbox** (`inbox.tsx`) — promote, dismiss, repair. Complete.

Supporting infra:
- `lib/api.ts` — 24 endpoints wired
- `lib/queries.ts` — React Query hooks with refetch intervals
- `lib/types.ts` — Full TypeScript types matching memd-schema
- `components/ui/` — badge, glass-panel, metric-card, empty-state, harness-health

## Work Items (7)

### 1. Memory page — Correction flow

Add "Correct" button on expanded memory items (next to existing metadata).

**Flow:**
1. User expands item in Memory browser
2. Clicks "Correct" button
3. Inline textarea opens pre-filled with current content
4. User edits content
5. Save calls `POST /memory/correct` with `{ id, new_content }`
6. Old item becomes `superseded`, new corrected item is `canonical`
7. UI refreshes search results, shows success toast

**Components:**
- `CorrectionEditor` — inline textarea + save/cancel, reusable (Memory + Inbox)

**API:** `POST /memory/correct` (exists)

### 2. Atlas page — Region graph with 2D/3D toggle

Semantic region navigation. The "zoomed out" view of memory.

**Default state:** Regions as large cluster nodes. Click to drill in.

**Top level:**
- Load regions via `GET /atlas/regions`
- Each region = node (sized by `node_count`, colored by primary kind)
- Edges between regions that share entity links

**Drill-down (click region):**
- `POST /atlas/explore` loads nodes + edges within region
- Nodes = entities, edges = relations (same_as, derived_from, supersedes, contradicts, related)
- Click node → sidebar shows linked memory items via entity
- "Back to regions" button

**2D/3D toggle:**
- Toolbar button: "2D" / "3D" with icon
- Swap `ForceGraph2D` ↔ `ForceGraph3D` component
- Same `graphData={{ nodes, links }}` prop
- Persist choice in `localStorage`

**Force config (supermemory E2-D3):**
```js
charge: -2000
collisionRadius: node => node.type === 'region' ? 70 : 35
alphaDecay: 0.025
warmupTicks: 150
```

**Node styling:**
- Color by `kind` (use existing kind color map from badge.tsx)
- Size by `node_count` (regions) or `confidence` (entities)
- Label: name, truncated

**Edge styling:**
- Color by relation type
- Directed arrows for supersedes/derived_from
- Dashed for `related`

**Trail saving:** Button to save current view as trail via `POST /atlas/trails/save`

**Components:**
- `AtlasGraph` — wrapper that renders ForceGraph2D or ForceGraph3D
- `GraphToggle` — 2D/3D switch button
- `RegionSidebar` — region detail + entity list
- `EntityDetail` — entity info + linked memory items
- `useGraphData` — hook that fetches + transforms atlas data into nodes/links

**API:** `/atlas/regions`, `/atlas/explore`, `/atlas/expand`, `/atlas/trails`, `/atlas/trails/save`

### 3. Graph page — Entity relationship explorer

Raw entity-level graph. The "zoomed in" view. Different from Atlas (which groups by region).

**Features:**
- Search bar → `GET /memory/entity/search` to find entities
- Results as nodes → `GET /memory/entity/links` to get edges
- Same 2D/3D toggle as Atlas (shared `GraphToggle` + `useGraphData` pattern)
- Click node → sidebar shows entity's memory items
- Edge labels: relation type (same_as, derived_from, supersedes, contradicts, related)
- Filter: toggle relation types on/off

**Shared with Atlas:** `GraphToggle`, `EntityDetail`, force config, 2D/3D renderer wrapper

**Components:**
- `EntityGraph` — ForceGraph2D/3D wrapper for entity data
- `EntitySearchBar` — search input for finding entities
- `RelationFilter` — checkboxes to toggle edge types

**API:** `/memory/entity/search`, `/memory/entity/links`

### 4. Procedures page — Viewer + management

List and manage learned procedures.

**Layout:**
- Tab bar: Candidate | Promoted | Retired
- Each tab loads `/procedures?status=<tab>`
- Procedure cards showing:
  - Name, description
  - Kind badge (workflow / policy / recovery)
  - Status badge
  - Trigger condition
  - Use count, confidence bar
  - Last used timestamp

**Expandable detail:**
- Steps list (ordered)
- Success criteria
- Source traces (source_ids → link to memory items)
- Tags

**Actions:**
- Candidate tab: "Promote" button → `POST /procedures/promote`
- Promoted tab: "Retire" button → `POST /procedures/retire`
- All tabs: "Record use" → `POST /procedures/use`

**Components:**
- `ProcedureCard` — card with badges, metrics, actions
- `ProcedureSteps` — ordered step list
- `ProcedureStatusTabs` — tab bar with counts

**API:** `/procedures` (GET), `/procedures/promote`, `/procedures/retire`, `/procedures/use`

### 5. Agent page — Profile + session viewer

View agent profiles and active sessions.

**Layout:**
- Left: agent list from active sessions (`GET /coordination/sessions`)
- Right: selected agent detail

**Agent detail sections:**
- **Profile** (`GET /memory/profile`): agent name, capabilities, hive role
- **Sessions**: active sessions for this agent, status, last seen, project/namespace
- **Memory by source** (`GET /memory/source`): items this agent stored
- **Working memory** (`GET /memory/working`): what this agent currently sees
- **Hive controls** (if queen role available):
  - Reroute (`POST /hive/queen/reroute`)
  - Deny (`POST /hive/queen/deny`)
  - Handoff (`POST /hive/queen/handoff`)

**Components:**
- `AgentList` — sidebar list of agents from sessions
- `AgentProfile` — profile metadata display
- `AgentMemory` — source memory list
- `HiveControls` — queen action buttons

**API:** `/coordination/sessions`, `/memory/profile`, `/memory/source`, `/memory/working`, `/hive/queen/*`

### 6. Ask page — Query interface

Simple "ask memd anything" search. No LLM dependency.

**Layout:**
- Large search input at top (like Google)
- Results below, ranked by relevance
- Each result: memory card (reuse Memory page components)
- "Explain" button on each result → `GET /memory/explain` → shows why item was retrieved

**Features:**
- Search via `POST /memory/search` with query text
- Results use same expandable card as Memory page
- Explain panel shows retrieval route, intent, scoring breakdown
- Recent queries in localStorage for quick re-search

**Components:**
- `AskInput` — large search input with submit
- `ExplainPanel` — retrieval explanation display
- Reuse: memory item cards from Memory page

**API:** `/memory/search`, `/memory/explain`

### 7. Status page — Eval score

Add honest eval score panel to existing Status dashboard.

**New panel:** "Eval Score" metric card
- Display eval score from `/healthz` (needs backend extension)
- Color: green ≥ 80, amber ≥ 50, red < 50
- Show last eval timestamp
- Show pressure metrics: inbox count, candidate count, stale count

**Backend change:** Extend `GET /healthz` response to include:
```json
{
  "status": "ok",
  "items": 150,
  "eval_score": 95.0,
  "pressure": {
    "inbox": 3,
    "candidates": 12,
    "stale": 5,
    "expired": 2
  }
}
```

Populate from store queries in healthz handler.

## New Dependencies

```json
{
  "react-force-graph-2d": "^1.x",
  "react-force-graph-3d": "^1.x",
  "three": "^0.x"
}
```

`three` is a peer dep of react-force-graph-3d (WebGL renderer).

## Shared Components (new)

| Component | Used by | Purpose |
|-----------|---------|---------|
| `GraphToggle` | Atlas, Graph | 2D/3D switch button |
| `ForceGraphWrapper` | Atlas, Graph | Renders 2D or 3D based on toggle state |
| `EntityDetail` | Atlas, Graph | Entity info sidebar |
| `CorrectionEditor` | Memory, Inbox | Inline content editor for corrections |
| `ExplainPanel` | Ask, Memory | Retrieval explanation display |

## File Changes

**New files:**
```
app/routes/atlas.tsx          — rewrite (was stub)
app/routes/graph.tsx          — rewrite (was stub)
app/routes/procedures.tsx     — rewrite (was stub)
app/routes/agent.tsx          — rewrite (was stub)
app/routes/ask.tsx            — rewrite (was stub)
app/components/graph/         — ForceGraphWrapper, GraphToggle, useGraphData
app/components/atlas/         — RegionSidebar, AtlasGraph
app/components/entity/        — EntityDetail, EntitySearchBar, RelationFilter
app/components/procedures/    — ProcedureCard, ProcedureSteps, ProcedureStatusTabs
app/components/agent/         — AgentList, AgentProfile, AgentMemory, HiveControls
app/components/ask/           — AskInput, ExplainPanel
app/components/correction/    — CorrectionEditor
```

**Modified files:**
```
app/routes/index.tsx          — add eval score panel
app/routes/memory.tsx         — add correction button + CorrectionEditor
lib/api.ts                    — add missing endpoint wrappers
lib/queries.ts                — add query hooks for new endpoints
lib/types.ts                  — add missing types
package.json                  — add react-force-graph-2d/3d, three
```

**Backend:**
```
crates/memd-server/src/routes.rs  — extend healthz response with eval_score + pressure
```

## Pass Gate (from phase doc)

1. ✅ Find specific fact in < 3 clicks — Memory search (already works)
2. 🔨 Correct wrong fact through UI — CorrectionEditor on Memory page
3. 🔨 Atlas graph renders with clickable nodes — Atlas page with react-force-graph
4. 🔨 Status shows real eval score — Eval score panel on Status page
5. 🔨 Zero console errors — Browser test after build

## Non-Goals

- Mobile responsive (desktop tool)
- Multi-user auth (single user)
- OpenUI / LLM-powered Ask page (too complex for this phase, simple search suffices)
- rust-embed integration (deferred — dev server is sufficient for I2 gate)
