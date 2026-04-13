# Phase I Human Dashboard

<!-- PHASE_STATE
phase: i
status: in_progress
truth_date: 2026-04-13
version: v1
next_step: design_system_and_api_client
-->

- status: `in_progress`
- version: `v1`
- truth date: `2026-04-13`
- next step: design system + API client (#5-6)

## Purpose

Ship a production-quality human-facing web dashboard for memd. The human owns
the memory — they need a surface to see it, approve it, correct it, navigate it,
and control running agent sessions. Currently all API routes exist but the only
UI is 2329 lines of server-side rendered HTML string templates in `ui/mod.rs`.

## Design Spec

Full design document: [[docs/superpowers/specs/2026-04-13-phase-i-human-dashboard-design.md]]

## Stack

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Framework | TanStack Start | Router + Query baked in, Vite under hood, type-safe |
| Generative UI | OpenUI (`@openuidev/react-lang`) | AI-composed dynamic views for explain/debug |
| Graph viz | cytoscape.js | Force-directed + hierarchical, extensible styling |
| Styling | Tailwind CSS + custom design system | TRON x Apple: dark glass, purple accents |
| Embedding | rust-embed | Static files baked into binary at compile time |
| React | 19+ | Required by OpenUI, supported by TanStack Start |

## Design Language

TRON meets Apple. Neon precision on dark glass. Purple accent energy with
Apple-level restraint. Frosted glass panels, glowing graph nodes, clean
typography, generous whitespace. Data feels alive but never noisy.

## Deliver

### Backend Pre-requisites

1. **Extend `/healthz`** — add `eval_score`, `degraded`, pressure metrics
   (inbox_count, candidate_count, stale_count, expired_count) to HealthResponse
2. **`/dashboard/*` static file serving** — new `dashboard.rs` module using
   rust-embed, SPA fallback to index.html, wired into main.rs Router
3. **`POST /dashboard/ask` proxy** — server-side LLM proxy for OpenUI generative
   view (keeps API key server-side, avoids CORS)

### Frontend App (`apps/dashboard/`)

4. **Scaffold** — TanStack Start + Vite + Tailwind + TypeScript project
5. **Design system** — theme tokens, glass surfaces, typography, motion primitives
6. **API client** — typed client matching memd-schema, TanStack Query hooks
7. **Shell layout** — sidebar nav, page transitions, theme

### Views (9 total)

8. **Status Dashboard** (`/dashboard/`) — health card (eval score gauge, degraded
   flag), pressure metrics, working memory summary, recent activity timeline,
   active agent sessions
9. **Memory Browser** (`/dashboard/memory`) — full-text search, filter by kind/scope/
   project/stage/status/agent/tags, sort by created/updated/confidence. Detail view
   with provenance, correction history, entity links, actions (correct, expire,
   verify, promote, explain), Obsidian bridge
10. **Inbox** (`/dashboard/inbox`) — candidates awaiting promotion, items needing
    verification, repair suggestions. Bulk and individual approve/dismiss/promote
11. **Correction Flow** — from any memory detail: "this is wrong" → edit → store
    new item with `supersedes` → expire original → confirmation showing next wake
    will reflect the change
12. **Graph View** (`/dashboard/graph`) — force-directed cytoscape.js graph. Nodes
    colored by kind, sized by confidence. TRON glow styling. Click to expand detail
    sidebar. Search re-centers graph. Filter by kind/stage/project. Hybrid: overview
    default, search/click refocuses around seed node
13. **Atlas View** (`/dashboard/atlas`) — hierarchical regions as cluster cards,
    drill-down to nodes within region, evidence trails, region neighbors sidebar
14. **Procedures** (`/dashboard/procedures`) — list with cards (name, usage count,
    confidence), filter active/retired. Detail view with usage history, source traces.
    Promote/retire actions
15. **Agent Panel** (`/dashboard/agent`) — debug: current wake packet, working memory
    items with scores, continuity state. Control: push corrections, hive queen
    actions (reroute/deny/handoff), session management, task management
16. **Ask / Generative** (`/dashboard/ask`) — OpenUI conversational interface.
    Natural language queries generate custom component trees streamed via
    `<Renderer>`. Component library: MemoryCard, MemoryList, Timeline, DiffView,
    GraphSnippet, MetricCard, Table, Alert, CodeBlock

### Embedding

17. **rust-embed integration** — `DashboardAssets` struct embedding `apps/dashboard/dist/`,
    serve handler with mime_guess, SPA fallback, nested under `/dashboard` in Router
18. **Build pipeline** — `npm run build` → `dist/` → `cargo build` embeds automatically

## Pass Gate

- [ ] human can browse, search, and correct memory through the UI
- [ ] correction made in UI changes next agent session's wake packet (E2E verified)
- [ ] graph view shows linked memory items (not just flat list)
- [ ] inbox items can be dismissed/approved from UI
- [ ] status dashboard shows real health (not just "ok")

## Pass Gate Evidence Required

- screenshots of every view working
- correction → recall E2E through UI (store correction, verify wake packet changes)
- graph view showing real entity links between memory items
- agent-browser verification of all 9 views with zero console errors
- embedded binary serves dashboard without separate Node process

## Fail Conditions

- UI is cosmetic wrapper with no real memory interaction
- corrections made in UI don't affect agent behavior
- graph view has no real links (entity links still empty)
- dashboard requires separate Node runtime to serve

## Rollback

- revert UI that masks broken memory behavior with pretty surfaces
- revert embedding changes if they bloat binary unacceptably (>50MB delta)

## Build Order

Tightest pass-gate constraint first:

1. Backend pre-reqs (#1-3): healthz extension, dashboard serving, ask proxy
2. Scaffold + design system + API client (#4-7): foundation before any view
3. Status Dashboard (#8): simplest view, proves static serving works, gates criterion #5
4. Memory Browser (#9): search + filter + inspect, gates criterion #1
5. Inbox (#10): approve/dismiss, gates criterion #4
6. Correction Flow (#11): hardest gate — E2E proof that correction changes wake, gates criterion #2
7. Procedures (#14): straightforward CRUD view
8. Atlas (#13): hierarchical navigation
9. Graph View (#12): cytoscape integration, gates criterion #3
10. Agent Panel (#15): debug + control, complex but not gated
11. Ask / Generative (#16): OpenUI integration, spike first, build last

## API Endpoints Used

All endpoints already exist in memd-server (verified):

| View | Endpoints |
|------|-----------|
| Status | `/healthz`, `/memory/working`, `/memory/inbox`, `/memory/timeline`, `/coordination/sessions` |
| Memory Browser | `/memory/search`, `/memory/context`, `/memory/explain`, `/memory/entity/links`, `/memory/verify`, `/memory/expire`, `/memory/promote` |
| Inbox | `/memory/inbox`, `/memory/inbox/dismiss`, `/memory/promote`, `/memory/repair` |
| Correction | `/memory/store` (with `supersedes`), `/memory/expire` |
| Graph | `/memory/entity/links`, `/memory/entity/search`, `/memory/search` |
| Atlas | `/atlas/regions`, `/atlas/explore`, `/atlas/expand`, `/atlas/trails` |
| Procedures | `/procedures`, `/procedures/promote`, `/procedures/retire`, `/procedures/use` |
| Agent Panel | `/memory/working`, `/memory/context/compact`, `/coordination/sessions`, `/hive/queen/*`, `/coordination/tasks/*` |
| Ask | `POST /dashboard/ask` (new — LLM proxy) |

## Migration Plan

1. New dashboard ships at `/dashboard/*` — no existing routes affected
2. Existing SSR dashboard stays at `/` and `/ui/*` during transition
3. Once new dashboard covers all SSR features: redirect `/` → `/dashboard/`
4. Remove SSR HTML generation from `ui/mod.rs` (keep snapshot/action API helpers)

## Open

- OpenUI package compatibility with TanStack Start + Vite (spike required before Ask view)
- LLM provider configuration for Ask view (env var vs config file — decide during backend pre-req)
- Agent profile view in Agent Panel (nice-to-have, not gated)

## Links

- [[ROADMAP]]
- [[docs/superpowers/specs/2026-04-13-phase-i-human-dashboard-design.md|Full design spec]]
- [[docs/theory/models/2026-04-11-memd-ralph-roadmap.md|Ralph roadmap (Phase 11)]]
