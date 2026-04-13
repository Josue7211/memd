# Phase I: Human Dashboard — Design Spec

## Overview

Ship a production-quality human-facing dashboard for memd. The human owns the memory — they need a surface to see it, approve it, correct it, navigate it, and control agent sessions. The dashboard embeds in the memd binary: one command (`memd serve`), one port, zero external runtime.

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Framework | TanStack Router + Vite | File-based routing + Query, pure SPA build for rust-embed (Start requires Node SSR runtime) |
| Generative UI | OpenUI (@openuidev/react-lang) | AI-composed dynamic views for explain/debug/control flows |
| Graph viz | cytoscape.js | Proven force-directed + hierarchical layouts, extensible styling |
| Styling | Tailwind CSS + custom design system | TRON meets Apple: dark glass, purple accents, neon precision, frosted panels |
| Embedding | rust-embed | Static files baked into binary at compile time. Zero runtime deps. |
| React version | 19+ | Required by OpenUI, supported by TanStack Start |

## Architecture

### Runtime Model

```
[memd binary]
  ├── Axum API server (existing — /memory/*, /atlas/*, /procedures/*, etc.)
  ├── rust-embed static file server (new — /dashboard/*)
  └── Fallback: SPA index.html for client-side routing
```

- `memd serve` starts both API and dashboard on the same port
- Dashboard files served from `/dashboard/*` prefix
- API routes unchanged — no breaking changes
- Existing SSR dashboard at `/` and `/ui/*` preserved during transition, removed when new dashboard covers all features

### Development Model

```
[Vite dev server :5173]  →  proxy /api/*  →  [memd serve :3080]
```

- Frontend lives in `apps/dashboard/` (new directory, separate from marketing site in `apps/`)
- `npm run dev` starts Vite with hot reload
- Vite proxy forwards API calls to running memd server
- `npm run build` outputs to `apps/dashboard/dist/`
- `cargo build` embeds `dist/` via rust-embed

### Directory Structure

```
apps/dashboard/
  ├── package.json
  ├── tsconfig.json
  ├── vite.config.ts
  ├── tailwind.config.ts
  ├── app/
  │   ├── routes/
  │   │   ├── __root.tsx            # Shell layout, nav, theme
  │   │   ├── index.tsx             # Status dashboard (home)
  │   │   ├── memory.tsx            # Memory browser
  │   │   ├── memory.$id.tsx        # Memory item detail
  │   │   ├── inbox.tsx             # Inbox view
  │   │   ├── graph.tsx             # Force-directed graph view
  │   │   ├── atlas.tsx             # Atlas hierarchical view
  │   │   ├── atlas.$region.tsx     # Atlas region drill-down
  │   │   ├── procedures.tsx        # Procedure list
  │   │   ├── procedures.$id.tsx    # Procedure detail
  │   │   ├── agent.tsx             # Agent debug + control panel
  │   │   └── ask.tsx               # OpenUI generative view
  │   ├── components/
  │   │   ├── layout/               # Shell, nav, sidebar
  │   │   ├── memory/               # Memory cards, lists, filters
  │   │   ├── graph/                # Cytoscape wrapper, node renderers
  │   │   ├── atlas/                # Region clusters, zoom controls
  │   │   ├── inbox/                # Inbox items, approve/reject/dismiss
  │   │   ├── procedures/           # Procedure cards, promote/retire
  │   │   ├── agent/                # Wake packet viewer, session controls
  │   │   ├── openui/               # OpenUI library definition, renderer
  │   │   └── ui/                   # Design system primitives
  │   ├── lib/
  │   │   ├── api.ts                # Typed API client (memd endpoints)
  │   │   ├── queries.ts            # TanStack Query hooks per endpoint
  │   │   └── types.ts              # TypeScript types matching memd-schema
  │   └── styles/
  │       ├── globals.css           # Tailwind base + design tokens
  │       └── theme.ts              # TRON/Apple theme config
  └── public/
      └── favicon.svg
```

## Design System: TRON x Apple

### Philosophy

Neon precision on dark glass. The data feels alive but never noisy. Every glow earns its place. Apple-level restraint keeps it from going full sci-fi.

### Color Tokens

```
--bg-primary:        #0a0a0f          (deep void)
--bg-surface:        #12121a          (elevated panel)
--bg-glass:          rgba(18,18,26,0.7) (frosted glass)
--border-subtle:     rgba(139,92,246,0.15) (purple ghost line)
--border-active:     rgba(139,92,246,0.5)  (purple glow)
--accent-primary:    #8b5cf6          (purple 500)
--accent-bright:     #a78bfa          (purple 400 — hover/active)
--accent-glow:       rgba(139,92,246,0.25) (ambient glow)
--text-primary:      #f0f0f5          (near-white)
--text-secondary:    #8888a0          (muted)
--text-tertiary:     #555570          (ghost)
--status-current:    #34d399          (green — active/healthy)
--status-stale:      #fbbf24          (amber — needs attention)
--status-expired:    #f87171          (red — action required)
--status-candidate:  #60a5fa          (blue — pending review)
```

### Surface Treatment

- Panels: `backdrop-filter: blur(12px)` + subtle purple border
- Cards: Slight elevation with `0 0 20px rgba(139,92,246,0.08)` glow on hover
- Active elements: Purple border glow + subtle shadow spread
- Graph nodes: Glow rings that pulse on state change

### Typography

- Primary: Inter (system-level clarity, Apple-adjacent)
- Monospace: JetBrains Mono (code, IDs, paths)
- Scale: 12/14/16/20/24/32 with tight line-heights for data density

### Motion

- Page transitions: 150ms ease-out fade
- Panel open/close: 200ms ease-out slide
- Graph node hover: 100ms glow intensity ramp
- Streaming UI (OpenUI): progressive reveal, no layout shift

## Views

### 1. Status Dashboard (Home — `/dashboard/`)

The landing view. At a glance: is memd healthy?

**Sections:**
- **Health card** — eval score gauge, degraded flag, last eval timestamp
- **Pressure metrics** — inbox count, candidate count, stale count, expired count
- **Working memory summary** — top items by kind (facts, decisions, status), count of non-status items
- **Recent activity timeline** — last N memory events (stores, corrections, promotions, expirations)
- **Active sessions** — running agent sessions with harness, project, last activity

**API endpoints used:**
- `GET /memory/working` — working memory items
- `GET /memory/inbox` — inbox count + items
- `GET /memory/timeline` — recent events
- `GET /coordination/sessions` — active sessions
- `GET /healthz` — server health

**Pre-requisite: Extend `/healthz` endpoint.** Current healthz returns `{ status, items }`. Phase I requires: `eval_score` (float), `degraded` (bool), and pressure metrics (inbox_count, candidate_count, stale_count, expired_count). Add these fields to `HealthResponse` in memd-schema and populate from store queries in the healthz handler. This is a backend change that must land before the Status Dashboard view can pass gate criterion #5.

### 2. Memory Browser (`/dashboard/memory`)

Search, filter, inspect all memory items.

**Features:**
- Full-text search via `/memory/search`
- Filter by: kind, scope, project, namespace, stage, status, agent, tags
- Sort by: created, updated, confidence, relevance
- Memory cards showing: title/content preview, kind badge, stage badge, status indicator, freshness, confidence bar, entity tags
- Click to open detail view

**Detail view (`/dashboard/memory/:id`):**
- Full content
- Provenance chain (source system, source path, producer, quality)
- Correction history (supersedes / superseded-by)
- Related items (entity links)
- Actions: correct ("this is wrong"), expire, verify, promote, explain
- Obsidian bridge: open in vault if source_path exists

**API endpoints used:**
- `POST /memory/search` — search
- `GET /memory/context` — full context
- `GET /memory/explain` — explain a memory item
- `GET /memory/entity/links` — entity connections
- `POST /memory/verify` — mark verified
- `POST /memory/expire` — expire item
- `POST /memory/promote` — promote candidate

### 3. Inbox (`/dashboard/inbox`)

Pending items that need human attention.

**Features:**
- Candidate items awaiting promotion
- Items needing verification
- Repair suggestions
- Bulk actions: approve selected, dismiss selected
- Individual actions: promote, dismiss, inspect, correct

**API endpoints used:**
- `GET /memory/inbox` — inbox items
- `POST /memory/inbox/dismiss` — dismiss items
- `POST /memory/promote` — promote candidates
- `POST /memory/repair` — apply repairs

### 4. Correction Flow

Available from any memory detail view.

**Flow:**
1. User clicks "This is wrong" on a memory item
2. Modal opens with current content pre-filled
3. User edits the content (or writes explanation of what's wrong)
4. Submit creates a new memory item that `supersedes` the original
5. Original gets expired, new corrected version becomes canonical
6. Confirmation shows: "Correction saved. Next agent wake will see the updated version."

**API endpoints used:**
- `POST /memory/store` — store corrected version with `supersedes: Vec<Uuid>` linking to the original item
- `POST /memory/expire` — expire original

### 5. Graph View (`/dashboard/graph`)

Force-directed graph of memory items and their connections.

**Default state:** Overview of all memory items as nodes, entity links as edges. Nodes colored by kind, sized by confidence/importance. Clusters emerge naturally from connection density.

**Interaction:**
- Click node: expand detail sidebar, highlight connections
- Search: graph re-centers on matching node, fades non-connected nodes
- Filter: toggle kinds, stages, projects on/off
- Zoom: scroll wheel, pinch
- Hover: glow ring + tooltip with title, kind, freshness

**Node styling (TRON aesthetic):**
- Facts: solid purple glow ring
- Decisions: bright purple with inner pulse
- Status: dim blue, smaller
- Procedures: green glow ring
- Candidates: dashed border, low opacity
- Expired/stale: red/amber dim ring

**Edge styling:**
- Entity links: thin purple lines
- Supersedes: directed arrow, amber
- Related: dotted line, low opacity

**Library:** cytoscape.js with `cose-bilkent` layout (force-directed with clustering) for overview, `concentric` layout for focused exploration.

### 6. Atlas View (`/dashboard/atlas`)

Hierarchical navigation: regions → nodes → evidence.

**Top level:** Region cards showing name, node count, primary kinds, last updated. Regions rendered as large cluster nodes in a simplified graph.

**Region drill-down (`/dashboard/atlas/:region`):**
- Nodes within the region as a focused graph
- Evidence trail for each node
- Region neighbors sidebar

**API endpoints used:**
- `GET /atlas/regions` — all regions
- `POST /atlas/explore` — explore a region
- `POST /atlas/expand` — expand node context
- `GET /atlas/trails` — navigation trails

### 7. Procedures (`/dashboard/procedures`)

View and manage learned operating procedures.

**List view:**
- Procedure cards: name, description, usage count, last used, confidence
- Filter: active, retired, low-confidence
- Actions: promote, retire, view detail

**Detail view:**
- Full procedure content
- Usage history
- Source traces (how it was learned)
- Promote to canonical / retire

**API endpoints used:**
- `GET /procedures` — all procedures
- `POST /procedures/promote` — promote
- `POST /procedures/retire` — retire
- `POST /procedures/use` — record usage

### 8. Agent Panel (`/dashboard/agent`)

Debug + control view for running agent sessions.

**Debug section (what the agent sees):**
- Current wake packet contents (reconstructed via `/memory/working` + `/memory/context/compact`)
- Working memory items with scores
- Continuity state (doing, left_off, changed, next, blocker)
- Active session info (harness, project, namespace)

**Control section:**
- Push correction to session: write a memory item tagged for the agent's current project
- Hive coordination: reroute focused bee, deny, handoff
- Session management: view sessions, retire stale sessions
- Task management: view/assign hive tasks

**API endpoints used:**
- `GET /memory/working` — working memory
- `GET /memory/context/compact` — compact context
- `GET /coordination/sessions` — sessions
- `POST /hive/queen/reroute` — reroute
- `POST /hive/queen/deny` — deny
- `POST /hive/queen/handoff` — handoff
- `POST /coordination/tasks/upsert` — create/update tasks
- `POST /coordination/tasks/assign` — assign tasks

### 9. Ask / Generative View (`/dashboard/ask`)

OpenUI-powered conversational interface.

**Behavior:**
- Text input: "explain why this decision was superseded" / "show me everything related to auth" / "what changed in memd project since yesterday"
- OpenUI renders a custom component tree from the AI response
- Component library constrained to: memory cards, timelines, charts, tables, diff views, graph snippets
- Streaming: components appear progressively as the LLM generates

**OpenUI component library:**
- `MemoryCard` — display a memory item
- `MemoryList` — filtered list of items
- `Timeline` — chronological event list
- `DiffView` — before/after comparison (corrections)
- `GraphSnippet` — small inline graph of related nodes
- `MetricCard` — single stat with label
- `Table` — tabular data
- `Alert` — status/warning message
- `CodeBlock` — formatted code/config

**Integration:**
- User query → append OpenUI system prompt (auto-generated from component library) → LLM call → stream to `<Renderer>`
- LLM has access to memd API responses as context (fetched server-side before generation)

**Package:** `@openuidev/react-lang` (core renderer + component definition). Requires React 19+. Install via npm: `npm install @openuidev/react-lang`. Optionally `@openuidev/react-headless` for chat state management.

**Pre-requisite spike:** Before building the Ask view, verify OpenUI works in the TanStack Start + Vite pipeline. Create a minimal test: define one component, render a hardcoded OpenUI output string, confirm it renders. If the package has build issues with Vite, fall back to a custom streaming renderer that parses structured JSON from the LLM instead of OpenUI Lang. The Ask view is the last view built — this spike can happen in parallel with earlier views.

**LLM routing:** The dashboard needs an API key for the LLM call. Add a `POST /dashboard/ask` server-side proxy endpoint in memd-server that accepts a user query, fetches relevant memd context, appends the OpenUI system prompt, calls the configured LLM, and streams the response back. This keeps the API key server-side and avoids CORS issues. The LLM provider/model is configured via memd server config (environment variable or config file).

## Embedding Strategy

### Build Pipeline

```
1. cd apps/dashboard && npm run build    →  dist/ (static files)
2. cargo build                           →  rust-embed includes dist/
3. memd serve                            →  Axum serves embedded files
```

### Rust Integration

```rust
// crates/memd-server/src/dashboard.rs
use axum::{
    Router,
    extract::Path,
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "apps/dashboard/dist/"]
struct DashboardAssets;

async fn serve_dashboard(Path(path): Path<String>) -> Response {
    // Try to serve the exact file first
    if let Some(file) = DashboardAssets::get(&path) {
        let mime = mime_guess::from_path(&path).first_or_octet_stream();
        ([(header::CONTENT_TYPE, mime.as_ref())], file.data).into_response()
    } else {
        // SPA fallback: return index.html for client-side routing
        match DashboardAssets::get("index.html") {
            Some(index) => Html(std::str::from_utf8(&index.data).unwrap_or("")).into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        }
    }
}

pub fn dashboard_router() -> Router {
    Router::new()
        .route("/*path", get(serve_dashboard))
        .route("/", get(|_| serve_dashboard(Path("index.html".to_string()))))
}
```

Wire into main.rs:
```rust
// In the Router::new() chain:
.nest("/dashboard", dashboard::dashboard_router())
```

- Route prefix: `/dashboard/*`
- SPA fallback: any non-API, non-file request under `/dashboard/` returns `index.html`
- Content-Type detection via `mime_guess` crate
- Cache headers: immutable for hashed assets, no-cache for index.html
- **Pre-requisite:** Add `rust-embed` and `mime_guess` to `memd-server` Cargo.toml dependencies

## Pass Gate Criteria

From the roadmap spec:

1. **Human can browse, search, and correct memory through the UI** — Memory browser + correction flow
2. **Correction made in UI changes next agent session's wake packet** — E2E: correct → expire → new canonical → verify wake packet contains corrected version
3. **Graph view shows linked memory items (not just flat list)** — Cytoscape force-directed graph with entity link edges
4. **Inbox items can be dismissed/approved from UI** — Inbox view with promote/dismiss actions
5. **Status dashboard shows real health (not just "ok")** — Eval score gauge, degraded flag, pressure metrics

## Testing Strategy

- **Unit tests:** TanStack Query hooks, API client functions, component logic
- **Integration tests:** Correction E2E flow (store correction → verify wake packet changes)
- **Browser tests:** agent-browser verification of every view, zero console errors
- **Embedding test:** `cargo test` verifies rust-embed serves index.html and assets correctly

## Migration Plan

1. New dashboard ships at `/dashboard/*`
2. Existing SSR dashboard stays at `/` and `/ui/*` — no breakage
3. Once new dashboard covers all features: redirect `/` → `/dashboard/`, remove SSR code from `ui/mod.rs`
4. `ui/mod.rs` helper functions (`build_visible_memory_snapshot`, etc.) may be preserved as API response builders if needed by other consumers

## Non-Goals

- Mobile-responsive design (desktop tool, not a phone app)
- Multi-user auth (single-user local tool)
- Offline mode (requires memd server running)
- Custom theme editor (one theme, done right)
