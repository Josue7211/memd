---
status: superseded
milestone: M4
superseded_by: ROADMAP.md + individual phase docs
superseded_date: 2026-04-17
---

# M4 Execution Plan: Make It 10-Star

> This plan tells the next code session exactly what to do. No theory reading needed.
> No "figure out the architecture" needed. Just follow the steps.

## Context

M3 is verified (593 tests, 0 failures, benchmarks zero regression, node verification 18✓/4~/0✗,
amnesia checklist 15/15). Visibility enforced at SQL level. Decay constants configurable.
Token efficiency measured per-kind and per-operation. Benchmark CI gate wired. Measurement proven.

M4 fixes **operability, evolution, integration, and surface** — the system is correct and
measured, but can't be debugged without reading Rust, doesn't propose skills overnight,
doesn't sync two ways with Obsidian, and the human dashboard is broken. M4 closes
10-STAR Tier-4 gaps 24–35. Five phases. 593 existing tests at session start.

## Root Cause

The system isn't 10-star because:
1. No structured logging — tracing crate absent. All debug info via `println!` and flat
   `(StatusCode, String)` errors. Operators read Rust to find why a call failed.
2. No `memd explain <id>` provenance chain — `inspection/mod.rs:12-78` returns raw payload
   without source, confidence, corrections, or evidence. Operators can't audit a memory
   without re-reading raw spine.
3. No latency SLA enforcement — M3 measured briefing latency but nothing asserts P95 < 100ms
   on working-memory retrieval. No alert, no regression gate.
4. No backup/restore procedure documented or tested — `store.rs` writes to SQLite but there
   is no tested restore path. A corrupt DB is data-loss.
5. No schema migration backward-compat contract — migrations at `store_migrations.rs` apply
   forward but there is no proof an old-schema spine can still be replayed after migration.
6. Queen ops (deny/reroute/handoff) exist at `routes.rs:893-1069` as endpoints but
   enforcement is advisory — conflicting writes still land. `coordination_mode` on
   `HiveSessionState` is a free-text string (`lib.rs:1134`), not an enum with enforced
   semantics.
7. Hive handoff packet at `lib.rs:1113-1125` has fields but no `working_context` snapshot —
   resuming in another harness drops in-flight working memory.
8. No Lamport versioning on `MemoryItem` (`lib.rs:220-249`) — cross-harness conflict
   resolution depends on timestamps. `HiveSessionState` has `state_version` (line 1410)
   but the pattern never extended to items.
9. No rate limiting on write endpoints — >100 writes/minute from a runaway agent floods the
   DB with no throttle.
10. No cross-harness E2E test — start in Codex, continue in Claude Code with full context
    is claimed but never proven.
11. No procedure detection in runtime — `procedural.rs:447-612` has `detect_procedures`
    but only the worker calls it on a 300s interval (`memd-worker/src/main.rs:44-69`).
    The "do something 3 times → propose procedure" signal arrives up to 5 minutes late.
12. No skill gating — `MemoryPolicySkillGating` at `lib.rs:2250-2257` is struct scaffolding
    with no connected gate check. Proposed skills land immediately without a threshold
    score to block bad ones.
13. Decay formula at `store.rs:712-714` uses flat exponential. No reinforcement extension —
    items accessed often decay at the same rate as items never touched.
14. No episode narratives — `Episode` struct absent from schema. Evolution loop can't
    consolidate session events into a narrative index with FTS5.
15. No autodream/autoevolve loops — worker runs maintenance every 300s but the
    dream-consolidate-propose cycle is not wired as an overnight job.
16. No consolidation quality gate — O2 measured it but M4 needs to reject low-quality
    consolidations before they persist.
17. Dashboard not served from `memd-server` — `apps/dashboard/` is a standalone SPA on
    port 5173. Backlog: `2026-04-15-dashboard-not-served-from-memd-server`.
18. Graph page crashes — type mismatch between server `EntitySearchResult` and client
    `apps/dashboard/app/lib/types.ts:100-108`. Backlog:
    `2026-04-15-graph-page-crash-entity-search-type-mismatch`.
19. `MemoryEntityRecord` shape diverges — server returns extra fields, client type at
    `lib/types.ts:400-402` lacks `access_count`. Backlog:
    `2026-04-15-memory-entity-record-type-mismatch`.
20. Dashboard `.env` hardcodes Tailscale IP — breaks on any machine that isn't the dev
    desktop. Backlog: `2026-04-15-dashboard-env-hardcoded-tailscale-ip`.
21. Preferences don't persist across sessions — wake packet at
    `crates/memd-client/src/runtime/resume/wakeup.rs:44-83` reads preferences but the
    save path evicts them. Roadmap blocker. Backlog:
    `2026-04-15-memd-preferences-not-persisted-across-sessions`.
22. Obsidian sync is one-way — `obsidian/watch_runtime.rs:1-62` watches files for import
    but there is no export path. Backlog: #56 (two-way sync).
23. No harness SDK packaging — harness clients at `memd-client/src/harness/cache.rs:1-243`
    are ad-hoc. No thin-adapter pattern. Multi-harness support needs a packaging model.

## Dependency Graph

```
M3 (verified) ──┬── K2 (Observability)          ──┐
                 │                                  │
                 ├── L2 (Hive Hardening)         ──┤
                 │                                  │
                 ├── I2 (Human Dashboard)        ──┤
                 │                                  │
                 └── M2-evo (Overnight Evolution)──┤
                         ↑ blocks on K2             │
                                                    │
                 N2 (Integrations Polish) ─────────┘
                         ↑ blocks on I2 + M2-evo
```

K2, L2, I2 start in parallel — all depend only on M3.
M2-evo blocks on K2 (needs structured logging + latency SLA before autodream loops land).
N2 blocks on I2 (needs dashboard shape fixed before SDK packaging) and M2-evo (needs skill
gating before harness adapters surface skills).

## Execution Order

Steps 1–3 (K2, L2, I2) can execute in any order — all depend only on M3.
Step 4 (M2-evo) blocks on K2. Step 5 (N2) blocks on I2 + M2-evo.
Recommended serial order for a single-developer session:
K2 first (observability primitives unblock everything else), then L2 (hive correctness
while observability is fresh), then I2 (dashboard fixes — these are user-facing blockers),
then M2-evo (overnight evolution — needs K2 tracing to see itself run), then N2
(integrations polish — capstone).

### Step 1: K2 — Observability

**Branch**: `research/mining` (current)
**Phase doc**: [[docs/phases/phase-k2-observability.md]]
**Depends on**: D2, C2, J2 (all verified through M3)
**Fixes**: gaps 30, 32, 35
**Backlog items**: #59, #72, #74

**What already exists** (verify, don't rebuild):

- `inspection/mod.rs:12-78` — `explain_memory` endpoint returns raw payload. No source
  chain, no corrections, no evidence lineage.
- `store.rs:90-91, 422-423` — `PRAGMA journal_mode=WAL`, `PRAGMA busy_timeout=5000`
  already set (L2-D3 pre-lifted in M2). Verify, do not re-lift.
- `routes.rs` error returns — flat `(StatusCode, String)`. No structured error class,
  no provider/model/attempt fields.
- Benchmark scorers + diagnostics endpoints from M3/P2 — latency is measured per op
  but not gated against an SLA.
- `store_migrations.rs` — forward migrations run on startup. No backward-compat test.

**What to build**:

1. **Add `tracing` + `tracing-subscriber` crate** — wire structured logging into
   `memd-server`, `memd-worker`, `memd-client`. JSON output in prod, pretty in dev.
   Span every HTTP request with `request_id`, `agent_id`, `project`, `operation`.
   - File: `crates/memd-server/Cargo.toml` (add tracing deps)
   - File: `crates/memd-server/src/main.rs` (init subscriber, env-var log level)
   - File: `crates/memd-server/src/routes.rs` (replace `println!` with `tracing::info!`)
   - Donor: none (standard Rust tracing pattern)

2. **Structured error classification** — replace `(StatusCode, String)` with
   `UpstreamErrorClass` → `RecoveryAction` mapping. Every failure carries provider,
   operation, attempt count, retry budget. Log structured error on every failed path.
   - File: `crates/memd-server/src/errors.rs` (new — `UpstreamErrorClass` enum)
   - File: `crates/memd-server/src/routes.rs` (all `Result<_, (StatusCode, String)>`
     return sites — migrate to `Result<_, MemdError>`)
   - Donor: **K2-D2 DIRECT RUST LIFT** (Omegon `upstream_errors.rs`)

3. **`memd explain <id>` provenance chain** — extend `inspection/mod.rs:12-78` to
   return: the item, all source items (corrections, parents), confidence timeline,
   lifecycle events (created/promoted/consolidated/superseded), entity links, trust
   rank history. CLI renders as a tree.
   - File: `crates/memd-server/src/inspection/mod.rs:12-78` (extend response shape)
   - File: `crates/memd-client/src/cli/explain.rs` (new CLI command)
   - Emit: `ExplainReport { item, sources[], lifecycle[], entities[], trust_chain[] }`

4. **Tag search and filtering endpoint** — `GET /api/memory/search?tag=<name>&kind=<kind>`
   filtered at SQL level. Tags index exists on `memory_items.tags_json` — add a
   generated column or helper table for tag indexing if SQL `LIKE '%<tag>%'` performance
   fails.
   - File: `crates/memd-server/src/routes.rs` (new search endpoint)
   - File: `crates/memd-server/src/store.rs` (tag query helper)

5. **Event spine integrity checks** — on startup, walk `event_spine` table and verify:
   monotonic sequence, no gaps, checksum per event. If corruption detected, log
   structured error and refuse to serve (fail closed). Add `memd spine verify` CLI
   for on-demand audit.
   - File: `crates/memd-server/src/store.rs` (spine_verify function)
   - File: `crates/memd-server/src/main.rs` (call spine_verify at startup)
   - File: `crates/memd-client/src/cli/spine.rs` (new CLI)

6. **Latency SLA enforcement** — M3 measures P50/P95/P99 briefing latency. M4 asserts
   P95 < 100ms for working-memory retrieval. Record every retrieval duration to a
   rolling histogram. Expose `/api/diagnostics/latency`. Fail CI if P95 > 100ms on the
   benchmark suite.
   - File: `crates/memd-server/src/working/mod.rs:15-272` (record histogram on admission)
   - File: `crates/memd-server/src/routes.rs` (latency diagnostics endpoint)
   - File: `crates/memd-client/src/benchmark/` (add latency gate)

7. **Backup/restore procedure** — `memd backup` writes a self-contained archive: SQLite
   snapshot + spine events + current policy. `memd restore <path>` atomically replaces
   the data dir. E2E test: backup → corrupt DB file → restore → verify 100% items
   retrievable.
   - File: `crates/memd-client/src/cli/backup.rs` (new — backup/restore CLI)
   - File: `crates/memd-server/src/tests/mod.rs` (corruption-injection test)

8. **Schema migration backward-compat contract** — freeze current schema version as
   `SCHEMA_V_M3`. On next schema change, add a migration test that loads a captured
   M3 spine, migrates forward, replays operations, and asserts identical state.
   - File: `crates/memd-server/src/store_migrations.rs` (add version constant)
   - File: `crates/memd-server/src/tests/migration_compat.rs` (new — replay test)
   - Artifact: `crates/memd-server/src/tests/fixtures/m3_spine.json` (captured M3 state)

9. **`HarnessStatus` compiled state surface** — one struct captures: git branch, memory
   health counts, context class, P95 latency, benchmark gate status. Serve as
   `GET /api/status`. Operator brief without reading Rust.
   - File: `crates/memd-server/src/status.rs` (new module)
   - File: `crates/memd-server/src/routes.rs` (wire status endpoint)
   - Donor: **K2-D1** (Omegon `status.rs`)

10. **Token tracking per-request bridge** — M3 counts tokens per-kind per-operation.
    M4 extends: every HTTP response carries `{ input_tokens, output_tokens,
    cache_read_tokens, cache_creation_tokens }` header. Diagnostics rolls up per-endpoint.
    - File: `crates/memd-server/src/routes.rs` (response header middleware)
    - Donor: **K2-D3** (Omegon `bridge.rs`)

**Pass gate**:
- `memd explain <id>` shows source, confidence, corrections, evidence chain for 5 item types
- Tag search endpoint filters at SQL level, not in-memory
- Spine corruption detected at startup; `memd spine verify` runs on demand
- Structured logs parseable by `jq` — request_id threads through every span
- P95 retrieval latency < 100ms on working-memory benchmark (CI-enforced)
- Backup → corrupt DB → restore → all items retrievable (E2E test green)
- Migration compat: M3 spine replays forward with identical final state
- `GET /api/status` returns `HarnessStatus` — one call, operator brief
- Response headers carry token counts on every memory op

**Verify nodes**: S6 (briefing latency measured + SLA-enforced), I1 (capture rate visible
in status), M7 (canonical quality viewable via explain), P1 (budget efficiency in diagnostics)

---

### Step 2: L2 — Hive Hardening

**Branch**: `research/mining` (current)
**Phase doc**: [[docs/phases/phase-l2-hive-hardening.md]]
**Depends on**: C2, J2 (both verified through M3)
**Fixes**: gaps 28, 31, 34
**Backlog items**: #53, #67, #70, #71, #75, #81

**What already exists** (verify, don't rebuild):

- `MemoryItem` at `crates/memd-schema/src/lib.rs:220-249` — no `version` field. Every
  mutation overwrites without conflict detection.
- `HiveSessionState` at `lib.rs:1410` — has `state_version: u64`. The pattern exists
  but never extended to items.
- `HiveHandoffPacket` at `lib.rs:1113-1125` — has resume fields but no `working_context`
  snapshot. Dropping in-flight working memory on handoff.
- `coordination_mode` at `lib.rs:1134, 1191-1193` — free-text `String`. No enum, no
  enforced semantics.
- Queen ops at `crates/memd-server/src/routes.rs:893-1069` — deny/reroute/handoff
  endpoints exist. Return OK. No write blocked on deny. No rebroadcast on reroute.
- `store.rs:90-91, 422-423` — `PRAGMA busy_timeout = 5000`, WAL mode both present.
  **L2-D3 already lifted in M2.** Add a verification test, do not re-lift.
- Rate limiting — none. Any agent can hammer `/api/memory/store`.

**What to build**:

1. **Lamport versioning on `MemoryItem`** — add `version: u64` field. Increment on every
   mutation. On import (`store_item`): if `incoming.version <= stored.version`, skip
   write and return `Conflict`. Deterministic, timestamp-independent.
   - File: `crates/memd-schema/src/lib.rs:220-249` (add `version: u64`)
   - File: `crates/memd-server/src/store.rs` (increment on mutation; conflict check on import)
   - File: `crates/memd-server/src/store_migrations.rs` (backfill: set `version = 1` on all existing rows)
   - Donor: **L2-D4 DIRECT RUST LIFT** (Omegon `sqlite.rs`)

2. **`CoordinationMode` enum** — replace `String` with enum: `Solo`, `Cooperative`,
   `Competitive`, `Locked`. Enforce semantics: `Locked` blocks all writes except from
   holding agent; `Competitive` requires explicit deny to block; `Cooperative` auto-merges
   via Lamport version.
   - File: `crates/memd-schema/src/lib.rs:1134` (replace String with enum)
   - File: `crates/memd-server/src/routes.rs:893-1069` (enforce mode on write path)
   - Migration: parse existing string values, map to enum variants, default Solo

3. **Queen deny/reroute/handoff enforcement** — wire the endpoints into the write path.
   `deny(item_id, reason)` inserts a deny record; subsequent `store_item` with same
   content hash is rejected. `reroute(item_id, target_lane)` updates the item's lane
   before promotion. `handoff(from_agent, to_agent, packet)` transfers working context
   and Lamport-locks the item so no other agent writes while the handoff is pending.
   - File: `crates/memd-server/src/routes.rs:893-1069` (enforcement logic)
   - File: `crates/memd-server/src/store.rs` (deny table + handoff lock)

4. **Handoff packet carries working context** — extend `HiveHandoffPacket` at
   `lib.rs:1113-1125` with `working_context: Option<WorkingContextSnapshot>`. Snapshot
   includes: current 8-slot working memory, active continuity fields, unresolved
   procedure candidates. Resume rebuilds working memory from the snapshot.
   - File: `crates/memd-schema/src/lib.rs:1113-1125` (extend struct)
   - File: `crates/memd-client/src/runtime/resume/` (snapshot builder + rebuild path)
   - Donor: **L2-D1** (Smriti `FreshnessInfo`) — freshness pattern guides delta resume

5. **`DivergenceSummary` on queen board** — when two harnesses write contradicting facts,
   generate a divergence summary: normalize text, diff decisions, cap at 2 branches and
   3 decisions per side. Expose at `GET /api/hive/divergence`.
   - File: `crates/memd-server/src/routes.rs` (new divergence endpoint)
   - File: `crates/memd-server/src/hive/divergence.rs` (new module)
   - Donor: **L2-D2** (Smriti `DivergenceSummary`)

6. **Rate limiting** — token-bucket per `agent_id`: 100 writes/minute soft cap, 200/minute
   hard cap. Soft cap returns `429 Too Many Requests` with `Retry-After` header. Hard
   cap rejects outright. Expose rate-limit state in `HarnessStatus`.
   - File: `crates/memd-server/src/middleware/rate_limit.rs` (new)
   - File: `crates/memd-server/src/routes.rs` (apply middleware to write endpoints)
   - Crate: `governor` (token-bucket impl)

7. **SQLITE_BUSY retry verification** — write a concurrency test: 10 threads each
   running 100 writes. Assert zero `SQLITE_BUSY` surfaces to the client (busy_timeout
   absorbs contention). If fails, L2-D3 did not survive M2 — treat as regression.
   - File: `crates/memd-server/src/tests/concurrency.rs` (new test)

8. **Cross-harness E2E test** — scripted scenario:
   (a) Start work in harness A (Codex-style): store 3 facts, 2 decisions, 1 procedure candidate.
   (b) Build handoff packet from A.
   (c) Open harness B (Claude-Code-style): resume from packet.
   (d) Assert all 3 facts, 2 decisions, 1 procedure candidate present in B's working context.
   (e) Continue work in B: correct a fact, add a decision.
   (f) Return to A: wake up and pick up B's additions — corrected fact surfaces, new decision present.
   - File: `crates/memd-server/src/tests/cross_harness.rs` (new test)

9. **Handoff quality comparison** — score handoff completeness at handoff time
   (extends J2 handoff scorer from M3): fact coverage, decision coverage, trust
   distribution, working-context depth. Reject handoff below quality threshold 0.8.
   - File: `crates/memd-client/src/runtime/resume/` (extend scorer, add threshold gate)

**Pass gate**:
- Every `MemoryItem` carries `version: u64`; stale imports rejected
- `CoordinationMode` enum replaces string; semantics enforced on write path
- Queen deny blocks conflicting writes; reroute updates lane; handoff transfers context with lock
- Handoff packet carries `WorkingContextSnapshot`; resume rebuilds working memory from it
- `GET /api/hive/divergence` returns bounded summary (≤2 branches × 3 decisions)
- Rate limit: 100 writes/min soft, 200/min hard, per-agent
- Concurrency test: 10 threads × 100 writes each, zero SQLITE_BUSY surfaces
- Cross-harness E2E: A→handoff→B→corrections→A picks up all changes
- Handoff quality score ≥ 0.8 or handoff rejected

**Verify nodes**: I3 (handoff quality scored + gated), P3 (coordination mode enforced
during promotion), M4 (cross-session stability extended to cross-harness), M6 (Lamport
versioning backs conflict-free promotion)

---

### Step 3: I2 — Human Dashboard

**Branch**: `research/mining` (current)
**Phase doc**: [[docs/phases/phase-i2-human-dashboard.md]] (status: in_progress)
**Depends on**: D2, E2, G2 (all verified through M3)
**Fixes**: gaps 27, 33
**Backlog items**: dashboard-not-served-from-memd-server, graph-page-crash-entity-search-type-mismatch,
memory-entity-record-type-mismatch, dashboard-env-hardcoded-tailscale-ip,
memd-preferences-not-persisted-across-sessions (roadmap blocker)

**What already exists** (verify, don't rebuild):

- `apps/dashboard/` — React + TanStack Router + Tailwind SPA. Dev server on port 5173.
  Standalone — not served by `memd-server`.
- `apps/dashboard/app/lib/types.ts:100-108` — `EntitySearchResult` client type. Diverges
  from server shape — graph page crashes on shape mismatch.
- `apps/dashboard/app/lib/types.ts:400-402` — `MemoryEntityRecord` client type. Missing
  `access_count` field that server emits.
- `apps/dashboard/app/routes/graph.tsx:35-38` — graph page crash site. Reads
  `entity.kind` which doesn't exist on server response.
- `apps/dashboard/.env` — `VITE_MEMD_SERVER=http://100.x.y.z:6420` (Tailscale IP hardcoded).
- `crates/memd-client/src/runtime/resume/wakeup.rs:44-83` — wake packet read path.
  Preferences appear in wake output but don't survive a save/restore round trip.
- `crates/memd-schema/src/lib.rs:1405-1422` — `MemoryEntityRecord` server-side shape.
  Has `access_count` (or needs it added — verify field presence).
- Existing dashboard routes: browse, search, graph, procedures, status. Status page
  renders eval scores but reads from a stub endpoint.

**What to build**:

1. **Serve dashboard from `memd-server`** — build step produces `apps/dashboard/dist/`.
   Server adds `tower-http::services::ServeDir` at `/dashboard/*`. Root redirect
   `/` → `/dashboard/`. No more separate dev server for users (dev retains hot-reload
   via vite but users hit the server).
   - File: `crates/memd-server/Cargo.toml` (add tower-http ServeDir feature)
   - File: `crates/memd-server/src/main.rs` (mount static handler)
   - File: `apps/dashboard/package.json` (add `build:prod` → `dist/`)
   - File: `crates/memd-server/build.rs` (new — run dashboard build if `MEMD_BUILD_DASHBOARD=1`)

2. **Fix entity search type mismatch** — align client `EntitySearchResult` with server
   shape. Canonical source: server. Regenerate client type from server response.
   - File: `apps/dashboard/app/lib/types.ts:100-108` (replace with server-aligned type)
   - File: `apps/dashboard/app/routes/graph.tsx:35-38` (use corrected fields)
   - Add: TypeScript type-check as CI gate — fail PR if `tsc --noEmit` errors

3. **Fix `MemoryEntityRecord` type mismatch** — add missing `access_count: number` to
   client type. If server emits more fields, mirror them. Document as "server is source
   of truth" in a comment header on `types.ts`.
   - File: `apps/dashboard/app/lib/types.ts:400-402` (extend type)
   - Verify: server at `crates/memd-schema/src/lib.rs:1405-1422` actually carries
     `access_count` — if not, add it schema-side and backfill

4. **Remove hardcoded Tailscale IP** — `.env.example` ships `VITE_MEMD_SERVER=`
   (empty — same-origin). Default behavior when empty: use `window.location.origin`.
   Runtime override via env var for Tailscale usage but no default IP burned in.
   - File: `apps/dashboard/.env.example` (ship empty var)
   - File: `apps/dashboard/.env` (remove hardcoded IP — gitignored already)
   - File: `apps/dashboard/app/lib/api.ts` (fallback to `window.location.origin`)

5. **Preference persistence across sessions** — root cause: save path at `wakeup.rs`
   reads preferences but on checkpoint the save helper truncates `preferences_json`.
   Fix: round-trip test first — store 3 preferences → checkpoint → wake → verify all 3
   return. Then fix the save path.
   - File: `crates/memd-client/src/runtime/resume/wakeup.rs:44-83` (audit read path)
   - File: `crates/memd-client/src/runtime/resume/` (audit save path; fix truncation)
   - File: `crates/memd-server/src/tests/preferences.rs` (new — round-trip test)

6. **Data-driven graph component** — refactor `graph.tsx` to accept pre-fetched
   `GraphApiDocument[]` as prop. Page component does the fetch; graph component is
   pure-render. Add `onLoadMore` callback for pagination.
   - File: `apps/dashboard/app/routes/graph.tsx` (split into container + view)
   - File: `apps/dashboard/app/components/MemoryGraph.tsx` (new — pure component)
   - Donor: **I2-D1** (supermemory `memory-graph/`)

7. **Static vs dynamic profile split** — browse page shows two panels: "Canonical"
   (static facts, pinned at top) and "Working" (dynamic context, live updates). Helps
   users tell what is permanent vs transient.
   - File: `apps/dashboard/app/routes/browse.tsx` (split panels)
   - File: `apps/dashboard/app/lib/projection.ts` (new — static/dynamic splitter)
   - Donor: **I2-D2** (supermemory `shared/types.ts`)

8. **Compact state brief `--compact`** — `memd state --compact` omits artifact content,
   keeps labels + recovery commands. Full detail via `memd explain <id>`. Dashboard
   "compact status" view uses same endpoint.
   - File: `crates/memd-client/src/cli/state.rs` (add `--compact` flag)
   - File: `apps/dashboard/app/routes/status.tsx` (render compact view)
   - Donor: **I2-D3** (Smriti CLI `--compact`)

9. **Correction UI** — browse → click item → "edit" or "supersede". Edit produces
   `HumanCorrection` trust-rank-4 item linked to original. Supersede marks original
   as superseded, inserts the replacement. Mark contested creates a contested lane tag.
   - File: `apps/dashboard/app/routes/browse.tsx` (correction modal)
   - File: `apps/dashboard/app/components/CorrectionForm.tsx` (new)
   - File: `crates/memd-server/src/routes.rs` (wire `/api/memory/correct`, `/supersede`)

10. **Honest status scoring** — status page shows: latest benchmark scores (from
    benchmark-registry), P95 latency (from K2 diagnostics), handoff quality
    distribution, divergence count. No lies, no stub data. If a metric isn't measured,
    show "not measured" not a fake number.
    - File: `apps/dashboard/app/routes/status.tsx` (connect real endpoints)
    - File: `crates/memd-server/src/routes.rs` (status aggregation endpoint)

11. **Browser test — zero console errors** — Playwright script: load `/dashboard/`,
    navigate every route (browse, search, graph, procedures, status), assert zero
    console errors, assert each route renders non-empty. CI gate.
    - File: `apps/dashboard/tests/e2e.spec.ts` (new Playwright test)
    - CI: add job to run on every PR touching `apps/dashboard/`

**Pass gate**:
- Dashboard served from `memd-server` on same origin as API
- User can find a specific fact in ≤ 3 clicks (timed manual test; documented)
- User can correct a wrong fact via UI; correction persists as trust-rank-4
- Graph renders with clickable nodes — zero console errors
- Status page shows real benchmark scores, latency, handoff quality — no stub data
- Preferences persist across checkpoint + wake (round-trip test green)
- Dashboard env has no hardcoded IP; defaults to same-origin
- Type-check gate: `tsc --noEmit` green on every PR
- Playwright E2E: every route loads, zero console errors

**Verify nodes**: S3 (deep-dive quality visible via UI), S5 (sync quality surfaced),
M2 (resume quality scored + displayed), M7 (canonical quality browsable)

---

### Step 4: M2-evo — Overnight Evolution

**Branch**: `research/mining` (current)
**Phase doc**: [[docs/phases/phase-m2-overnight-evolution.md]]
**Depends on**: F2, G2 (verified through M3), K2 (needs structured logging + latency SLA)
**Fixes**: gaps 24, 26
**Backlog items**: #62, #63, #65

**What already exists** (verify, don't rebuild):

- `crates/memd-worker/src/main.rs:44-69` — worker loop, 300s interval. Calls
  `maintain_runtime` only. No dream/autoevolve split.
- `crates/memd-server/src/procedural.rs:447-612` — `detect_procedures` scans event
  spine for repeated action sequences. Called from worker path only.
- `crates/memd-server/src/store.rs:636-783` — `decay_entities`. M3 made constants
  configurable. Formula at line 712-714 is flat exponential — no reinforcement extension.
- `crates/memd-schema/src/lib.rs:1803-1834` — `Procedure` struct. No gate-threshold
  field. Every detected procedure persists immediately.
- `crates/memd-schema/src/lib.rs:2250-2257` — `MemoryPolicySkillGating` struct
  scaffolding. Fields exist; no connected check.
- `MemoryPolicyDecay` at `lib.rs:2186-2196` — from O2. Configurable constants wired
  into `decay_entities`.
- Episode — no struct. Evolution loop has nothing to consolidate into narratives.

**What to build**:

1. **Reinforcement-extended half-life decay** — replace flat formula at
   `store.rs:712-714` with: `half_life = base_half_life × (reinforcement_factor ^ (access_count - 1))`,
   capped at 90 days. Each access extends half-life; unused items decay at baseline.
   Requires `access_count` on `MemoryItem` + `MemoryEntityRecord` (increment on retrieval).
   - File: `crates/memd-schema/src/lib.rs:220-249, 1405-1422` (add `access_count: u64`)
   - File: `crates/memd-server/src/store.rs:636-783` (replace decay formula)
   - File: `crates/memd-server/src/helpers.rs:27-113` (increment access_count on retrieval)
   - File: `crates/memd-schema/src/lib.rs:2186-2196` (extend `MemoryPolicyDecay` with
     `base_half_life_days`, `reinforcement_factor`, `max_half_life_days`)
   - Donor: **M2-D1 DIRECT RUST LIFT** (Omegon `decay.rs`)

2. **`Episode` schema + FTS5** — add struct
   `Episode { id, mind, title, narrative, date, session_id }` and junction table
   `episode_facts(episode_id, fact_id, relation)`. FTS5 index on `narrative` column.
   Evolution loop consolidates session events into episodic narratives.
   - File: `crates/memd-schema/src/lib.rs` (add `Episode` struct near other items)
   - File: `crates/memd-server/src/store_migrations.rs` (new migration:
     `CREATE TABLE episodes`, `CREATE TABLE episode_facts`, `CREATE VIRTUAL TABLE
     episodes_fts USING fts5(narrative)`)
   - File: `crates/memd-server/src/episodes.rs` (new module — CRUD + FTS5 query)
   - Donor: **M2-D2 DIRECT RUST LIFT** (Omegon `types.rs`)

3. **Runtime procedure detection** — `detect_procedures` currently only runs in worker
   (300s delay). Add runtime path: after every write, check if the last N actions match
   a repeated pattern. If 3+ repetitions detected, surface a candidate procedure
   immediately. Flag as `source: Runtime` vs `source: WorkerScan` for diagnostics.
   - File: `crates/memd-server/src/procedural.rs:447-612` (add runtime trigger)
   - File: `crates/memd-server/src/routes.rs` (call after `store_item` on write path)
   - File: `crates/memd-schema/src/lib.rs:1803-1834` (add `detection_source` enum)

4. **Skill gating with evaluation gate** — connect `MemoryPolicySkillGating` at
   `lib.rs:2250-2257` to a gate function. Gate runs a benchmark-style scoring pass
   against a held-out test set: a proposed procedure is executed on N sample inputs,
   outputs scored, pass only if score ≥ threshold (default 0.7). Until pass, procedure
   persists as `status: Proposed`; only `Approved` surfaces in wake packet.
   - File: `crates/memd-server/src/procedural.rs` (new — `score_proposed_procedure` fn)
   - File: `crates/memd-server/src/routes.rs` (new endpoint: `POST /api/procedure/score`)
   - File: `crates/memd-schema/src/lib.rs:1803-1834` (add
     `status: ProcedureStatus::{Proposed, Approved, Rejected}`)
   - File: `crates/memd-client/src/cli/procedure.rs` (new CLI: `memd procedure score <id>`)
   - Gate threshold: config at `MemoryPolicySkillGating.min_score` (default 0.7)

5. **Dream loop (autodream)** — nightly job (or N-hour cadence):
   (a) Pull last-24h event spine.
   (b) Group events into logical episodes (session boundaries).
   (c) Generate narrative per episode (template-based; LLM optional).
   (d) Insert into `episodes` table with FTS5 indexing.
   (e) Link facts created in that session via `episode_facts`.
   Dream is idempotent — re-running with same window is a no-op.
   - File: `crates/memd-worker/src/dream.rs` (new module)
   - File: `crates/memd-worker/src/main.rs:44-69` (dispatch dream on nightly cadence)
   - Config: `dream_interval_hours` (default 24)

6. **Autoevolve loop** — periodic job (after dream):
   (a) Run `detect_procedures` on full spine (not just last-N).
   (b) Score each proposal via gate function.
   (c) Approved ones flip to `ProcedureStatus::Approved`, surface in wake.
   (d) Rejected ones log reason; repeated rejections increase gate threshold per-pattern
       (prevent noisy patterns from nagging).
   - File: `crates/memd-worker/src/autoevolve.rs` (new module)
   - File: `crates/memd-worker/src/main.rs:44-69` (dispatch autoevolve after dream)

7. **Consolidation quality gate** — extend O2 consolidation-quality scoring (from M3):
   reject consolidations below quality threshold 0.8. Rejected consolidations don't
   persist; log reason. Accepted ones flow through normal consolidation path.
   - File: `crates/memd-server/src/routes.rs:1315-1502` (wrap consolidation in gate check)
   - File: `crates/memd-schema/src/lib.rs` (extend consolidation policy with threshold)

8. **Decay calibration from real usage data** — after autoevolve runs for 7 days,
   analyze real access patterns. Run M3 decay-sensitivity tool against real data.
   Propose updated constants if data shows current defaults are mis-tuned. Output
   evidence report to `docs/verification/decay-calibration-m4.md`.
   - File: `crates/memd-client/src/cli/calibrate.rs` (new CLI: `memd calibrate decay`)
   - Artifact: `docs/verification/decay-calibration-m4.md` (evidence)

9. **Procedure-proposal E2E test** — scripted scenario:
   (a) Perform action X three times (with variations).
   (b) Wait for runtime detection (< 1s, not 300s).
   (c) Assert procedure candidate surfaces with `status: Proposed`.
   (d) Call score endpoint.
   (e) Assert either Approved (score ≥ 0.7) or Rejected (with reason).
   (f) Wake: assert Approved surfaces, Proposed does not, Rejected does not.
   - File: `crates/memd-server/src/tests/procedure_proposal.rs` (new test)

**Pass gate**:
- Reinforcement-extended decay: accessed items persist longer; unused items decay faster
  (proven via A/B on real data)
- Episode schema + FTS5 index created; dream loop populates episodes nightly
- Runtime procedure detection surfaces candidate in < 1s after 3rd repetition
- Skill gating: proposed procedures score against test set; only score ≥ 0.7 surfaces in wake
- Autoevolve loop runs after dream; consolidates proposals; logs rejections
- Consolidation quality gate rejects < 0.8 consolidations
- Decay calibration report justifies chosen constants with 7-day real-usage data
- Procedure-proposal E2E: do-3-times → propose → score → gate → wake (full round trip)

**Verify nodes**: M5 (procedure reuse rate + gate threshold documented), M6
(candidate→canonical conversion rate with gated approval), P3 (decay calibrated from
real usage, not sensitivity-only), S2 (navigation coverage extends to episodes via FTS5)

---

### Step 5: N2 — Integrations Polish

**Branch**: `research/mining` (current)
**Phase doc**: [[docs/phases/phase-n2-integrations-polish.md]]
**Depends on**: F2 (verified), I2 (step 3), M2-evo (step 4)
**Fixes**: gaps 25, 29
**Backlog items**: #54, #56, #64, #69, #79

**What already exists** (verify, don't rebuild):

- `crates/memd-client/src/obsidian/watch_runtime.rs:1-62` — Obsidian file watcher.
  Import-only. No export path. Any markdown change in vault gets imported; no path
  writes back to vault.
- `crates/memd-client/src/harness/cache.rs:1-243` — per-harness turn cache. Unbounded —
  no max size, no eviction.
- Harness clients (claude-code, codex) — ad-hoc adapters per harness. No shared
  packaging pattern.
- No MCP server in memd — the client talks REST. No multi-transport.
- RAG — no sidecar. No timeout, no retry, no fallback. Either disabled or blocking.
- Config — `.memd/config.json` has harness fields but no SKILL.md instruction file.

**What to build**:

1. **Obsidian two-way sync** — import side exists. Add export side:
   (a) On canonical promotion, write markdown to `<vault>/memd/<kind>/<slug>.md`.
   (b) YAML frontmatter carries `id`, `kind`, `lane`, `trust_rank`, `version`.
   (c) File watcher detects external edits; diffs against last-known content; imports
       the change as a correction (Lamport version bump).
   (d) Loop detector: if same content round-trips 2× within 10s, suppress the echo.
   - File: `crates/memd-client/src/obsidian/export.rs` (new)
   - File: `crates/memd-client/src/obsidian/watch_runtime.rs:1-62` (extend with loop
     detection)
   - File: `crates/memd-client/src/obsidian/conflict.rs` (new — merge strategy: Lamport
     version wins; tie → last-write-wins with log)

2. **Conflict resolution** — both sides edit same item within sync window:
   (a) Compare Lamport versions. Higher version wins.
   (b) If versions equal: compare timestamps. Last-write wins.
   (c) Losing side saved as `{slug}.conflict.md` for human review.
   (d) Test: concurrent edit scenario — both sides modify, one wins cleanly, loser
       preserved.
   - File: `crates/memd-client/src/obsidian/conflict.rs` (merge logic)
   - File: `crates/memd-client/src/tests/obsidian_sync.rs` (new test)

3. **Harness SDK packaging** — thin-adapter pattern. Each harness adapter is < 200 lines.
   Shape: `accept_config` → `wrap_core` → `return_framework_wrapper`. Core
   (`memd-client::core`) centralized. Adapters: claude-code, codex, cursor, future-x.
   - File: `crates/memd-client/src/harness/mod.rs` (extract shared core)
   - File: `crates/memd-client/src/harness/claude_code.rs` (refactor to < 200 lines)
   - File: `crates/memd-client/src/harness/codex.rs` (refactor to < 200 lines)
   - File: `crates/memd-client/src/harness/README.md` (adapter guide)
   - Donor: **N2-D1** (supermemory `packages/tools/`)

4. **LRU turn cache** — replace unbounded cache at `harness/cache.rs:1-243` with LRU,
   max 100 entries. Key: `{container}:{thread}:{mode}:{normalized_message}`. Prevents
   duplicate API calls within a turn.
   - File: `crates/memd-client/src/harness/cache.rs:1-243` (replace with LRU)
   - Crate: `lru` (standard LRU impl)
   - Donor: **N2-D2** (supermemory `shared/cache.ts`)

5. **`SKILL.md` versioned instruction file** — ship
   `crates/memd-client/assets/SKILL.md` that teaches agents:
   when to remember, when NOT to checkpoint, multi-agent etiquette. Version at top
   (`v1.0.0`). Harness adapters surface it on connection.
   - File: `crates/memd-client/assets/SKILL.md` (new — instruction file)
   - File: `crates/memd-client/src/harness/mod.rs` (expose `skill_markdown()` fn)
   - Donor: **N2-D3** (Smriti `SKILL.md`)

6. **MCP server with multi-transport** — add `memd-mcp` subcommand exposing memd ops
   via MCP. Transports: `stdio` (default), `http`, `sse`. Config per transport.
   Harnesses that speak MCP (Claude Code, others) connect via this.
   - File: `crates/memd-client/src/mcp/mod.rs` (new — MCP server)
   - File: `crates/memd-client/src/mcp/transport.rs` (stdio/http/sse)
   - File: `crates/memd-client/src/cli/mcp.rs` (new CLI: `memd mcp serve`)
   - Donor: **N2-D4 DIRECT RUST LIFT** (Omegon `plugins/mcp.rs`)

7. **RAG sidecar with timeout + fallback** — optional RAG retrieval augmentation:
   (a) Query memd retrieval (primary).
   (b) In parallel, query RAG sidecar with 5s timeout.
   (c) If RAG returns in time: merge scores.
   (d) If timeout: fall back to memd-only result (do not block).
   (e) Log every timeout; if timeout rate > 10% rolling window, disable sidecar until operator re-enables.
   - File: `crates/memd-server/src/rag/mod.rs` (new sidecar client)
   - File: `crates/memd-server/src/helpers.rs:27-113` (call sidecar alongside retrieval)
   - Config: `rag.timeout_ms` (default 5000), `rag.fallback_threshold_rate` (default 0.1)

8. **Multi-user / team foundations** — introduce `user_id` field on `MemoryItem` (null =
   single-user legacy). Filter in retrieval: default scope = current user. `--team`
   flag expands to include team-shared items (those with `visibility: Public`).
   Migration backfills existing items with `user_id: None`.
   - File: `crates/memd-schema/src/lib.rs:220-249` (add `user_id: Option<String>`)
   - File: `crates/memd-server/src/store_migrations.rs` (backfill)
   - File: `crates/memd-server/src/helpers.rs:27-113` (apply user-scope filter)
   - Documentation-only for team UX — full multi-tenant is post-M4

9. **RAG fallback E2E test** — scripted scenario:
   (a) Configure sidecar with 100ms response time; query; assert merged result.
   (b) Configure sidecar with 10s response time; query; assert fallback to memd-only
       result within 5s timeout budget.
   (c) Simulate 11% timeout rate over 100 queries; assert sidecar auto-disabled.
   - File: `crates/memd-server/src/tests/rag_sidecar.rs` (new test)

10. **Obsidian sync E2E test** — scripted scenario:
    (a) Store fact in memd; assert appears in vault within 1 cycle.
    (b) Edit note in vault; assert appears in memd within 1 cycle.
    (c) Edit same item on both sides within window; assert higher-version wins;
        loser saved as `.conflict.md`.
    - File: `crates/memd-client/src/tests/obsidian_sync.rs` (new test)

**Pass gate**:
- Obsidian two-way sync: edit in vault → memd within 1 cycle; store in memd → vault within 1 cycle
- Conflict: both-side edit resolves via Lamport version; loser preserved as `.conflict.md`
- Harness adapters < 200 lines each; shared core at `harness/mod.rs`
- Turn cache LRU-bounded at 100 entries
- `SKILL.md` versioned and surfaced by adapters
- MCP server supports stdio/http/sse
- RAG query with 5s timeout → fallback to memd-only if late; auto-disable if > 10% timeouts
- Multi-user `user_id` field present; user-scope filter applied on retrieval
- All E2E tests (Obsidian sync, RAG fallback) green

**Verify nodes**: S5 (sync quality: vault↔memd coverage ≥ 90%), I3 (handoff quality
extended to harness SDK), M7 (canonical items surfaced to vault), S2 (RAG augments
navigation without blocking)

## Node-by-Node M4 E2E Tests

Every node in the architecture graph must pass its M4 criterion. Phase gates (K2/L2/I2/M2-evo/N2)
cover some nodes. The tests below cover ALL nodes. Run after all five phases pass.

### Ingest Layer

| Node | M4 Criterion | E2E Test |
|------|-------------|----------|
| I1 | Capture path structured-traced | Ingest 10 items → pipe logs to `jq` → assert every span has `request_id`, `agent_id`, `operation`, `kind`. Assert structured `UpstreamErrorClass` on any failure (not flat string) |
| I2 | Spill survives backup/restore | Fill spill buffer → `memd backup` → corrupt DB → `memd restore` → assert spill items retrievable; `memd spine verify` green |
| I3 | Handoff carries WorkingContextSnapshot + quality-gated | A builds continuity (5 items) → handoff packet → assert snapshot present → quality score ≥ 0.8 or rejected → B resumes → B working memory matches A |

### Control Plane

| Node | M4 Criterion | E2E Test |
|------|-------------|----------|
| P1 | Budget ops rate-limited + token-metered | Agent writes 101 in 60s → 101st returns 429 with `Retry-After`. Response header carries `input_tokens`/`output_tokens` on every budget op |
| P2 | Continuity preserves preferences cross-session | Store 3 prefs → checkpoint → wake → assert all 3 surface in wake packet (round-trip test green) |
| P3 | Promotion uses Lamport + reinforcement-extended decay + coordination-mode enforcement | Set mode=Locked → agent B write attempt rejected with 409. Stale Lamport version rejected as Conflict. Access item A 10× vs never-access B → A half-life > B |
| P4 | Correction UI persists trust-rank-4 | Dashboard: click item → edit → save → assert correction persists with `trust_rank=4`; surfaces in `memd explain <id>` chain |

### Typed Memory

| Node | M4 Criterion | E2E Test |
|------|-------------|----------|
| M1 | Working memory rebuilt after cross-harness handoff | A stores 5 items; handoff → B resumes → B working memory has same 5 items; continue work in B → handoff back → A picks up B's additions |
| M2 | Resume quality scored + surfaced in honest status | Cold resume → quality score generated → dashboard status page renders real score (not stub). If missing dimension, shows "not measured" |
| M3 | Episode narratives + FTS5 index populated by dream loop | Worker runs dream on 24h spine → `episodes` table populated → FTS5 query on narrative returns match. `episode_facts` junction rows present |
| M4 | Cross-harness E2E with Lamport conflict resolution | Codex-style A → handoff → Claude-Code-style B → continues → A wakes → sees B's changes. Concurrent same-item write → higher Lamport wins; stale rejected |
| M5 | Runtime procedure detection + skill gating | Perform action X 3 times → candidate surfaces within 1s (not 300s) → score ≥ 0.7 → Approved, surfaces in wake. Score < 0.7 → Rejected, not surfaced |
| M6 | Candidate→canonical conversion uses Lamport + gated approval + reinforcement decay | Run autoevolve → approved proposals flip to canonical; rejected logged. Run 7-day calibration → decay report shows accessed items persisted, unused decayed |
| M7 | Canonical quality viewable via `memd explain <id>` | Pick 5 canonical items → `memd explain <id>` each → assert source chain, confidence, corrections, lifecycle, entity links, trust history present |

### Recall Surfaces

| Node | M4 Criterion | E2E Test |
|------|-------------|----------|
| S1 | Wake packet carries per-kind tokens + status reference | Wake → response headers include token counts per kind. `HarnessStatus` available via `GET /api/status` in < 50ms — one call, operator brief |
| S2 | Navigation augmented by RAG sidecar with timeout + fallback | Sidecar 10s latency → query returns within 5s using memd-only result. 11% timeout rate over 100 queries → sidecar auto-disabled, logged. Episode FTS5 extends navigation coverage |
| S3 | Deep-dive via `memd explain` + dashboard correction UI | `memd explain <id>` returns full provenance chain. Dashboard: click item → modal with full chain, correct/supersede/contest buttons → correction persists |
| S4 | Evidence completeness via spine verify + backup/restore + migration compat | `memd spine verify` green at startup. Backup → corrupt → restore → 100% items retrievable. Load M3 spine fixture → migrate forward → replay → identical final state |
| S5 | Obsidian two-way sync with conflict resolution | Store in memd → vault has file within 1 cycle. Edit vault → memd has change within 1 cycle. Both-side edit → higher Lamport wins; loser saved as `.conflict.md` |
| S6 | Briefing latency SLA-enforced + served from memd-server | Benchmark suite → P95 working-memory retrieval < 100ms; CI gate fails if exceeded. `GET /dashboard/` served on same origin as API (port 6420); Playwright E2E green with zero console errors |

### Live Loop M4 Test

Run the full cycle and measure every layer:

1. Capture raw event → I1 → tracing span starts with `UpstreamErrorClass` on any failure
2. Update working context → P1 → M1 → rate-limit + token header recorded; latency in histogram
3. Update session continuity → P2 → M2 → preferences persist; resume quality scored
4. Write episodic record → M3 → dream loop populates FTS5-indexed narrative
5. Repair semantic truth → P3 → M4 → Lamport version bumped; coordination mode enforced
6. Update procedural memory → P3 → M5 → runtime detection fires < 1s; gated by skill threshold
7. Compile wake packet → S1 → per-kind tokens in response header; `HarnessStatus` reference
8. Export to Obsidian → S5 → file written with YAML frontmatter (Lamport version in metadata)
9. Import from Obsidian → S5 → S3 → conflict resolution applied; deep-dive via `memd explain`
10. Augment with RAG sidecar → S2 → 5s timeout, fallback to memd-only on miss
11. Verify evidence → S4 → spine check, backup/restore, migration compat all green

**M4 gate test**: Run the full cycle 3 times. All 11 steps emit structured logs.
Observability dashboard (`/dashboard/status`) shows live metrics for every step.
No measurement gaps. Every node produces trace events.

### Evolution Loop M4 Test

1. Worker boots → structured log "worker started"
2. 24h mark → dream loop runs → episodes created (log records count)
3. 24h + 1h mark → autoevolve runs → procedure candidates scored
4. Approved candidates flow to wake packet on next session
5. Access patterns feed decay calibration
6. After 7 days → `memd calibrate decay` → evidence report emitted

**M4 gate test**: Let the system run for 7 days with synthetic traffic. Dream loop fires
7 times. Autoevolve fires 7 times. Decay calibration report at day 7 shows real-usage data.

### Observability Loop M4 Test

1. Every request has a `request_id` in logs
2. Every error is structured (`UpstreamErrorClass`)
3. Every retrieval hits the latency histogram
4. `GET /api/status` returns `HarnessStatus` in < 50ms
5. `memd explain <id>` returns full chain for any canonical item
6. Spine verification runs on startup — corruption is fatal
7. Backup → restore round trip proves data survival

**M4 gate test**: `memd diagnostics report --m4` outputs all observability dimensions.
Tracing logs parseable by `jq`. No "N/A" or "not measured" in any dimension.

## What NOT To Do

- Do not touch M5 scope — M4 delivers the 10-star gaps. Further optimization is M5 territory.
- Do not redesign the hive protocol — enforce the existing one. If semantics are unclear,
  that's a finding for M5, not a blocker for M4.
- Do not rewrite the dashboard — fix the broken pieces. A new framework choice is a
  separate initiative.
- Do not chase exotic MCP transports — stdio/http/sse cover the field. Dockerized
  transport gateways are post-M4.
- Do not widen Obsidian conflict resolution beyond Lamport + last-write-wins. Richer
  strategies are M5.
- Do not enable RAG by default — it's an opt-in sidecar. Primary retrieval must never
  depend on it.
- Do not add LLM calls to the dream loop for M4 — template-based narratives. Optional
  LLM enhancement is a follow-up.
- Do not change the decay formula parameters without 7-day real-usage data justification.
- Do not ship skill gating with threshold < 0.7 — set a conservative default; tune later.
- Do not merge M4 work until ALL five phases pass gates.

## Amnesia Prevention Checklist

After M4 passes, before declaring done:

- [ ] `tracing` crate wired; logs are JSON in prod, pretty in dev
- [ ] `UpstreamErrorClass` replaces flat errors on every `routes.rs` return path
- [ ] `memd explain <id>` returns full provenance chain for 5 item types
- [ ] Tag search endpoint filters at SQL level
- [ ] Spine verification runs at startup; corruption is fatal
- [ ] P95 working-memory retrieval latency < 100ms (CI-enforced)
- [ ] Backup → corrupt → restore round trip passes E2E
- [ ] Migration compat test: M3 spine replays forward identically
- [ ] `GET /api/status` returns `HarnessStatus` in < 50ms
- [ ] Response headers carry token counts on every memory op
- [ ] `MemoryItem.version: u64` present; stale imports rejected as Conflict
- [ ] `CoordinationMode` enum enforced on write path
- [ ] Queen deny/reroute/handoff actually block/redirect/transfer (not advisory)
- [ ] `HiveHandoffPacket` carries `WorkingContextSnapshot`
- [ ] `DivergenceSummary` endpoint returns bounded (≤2 branches × 3 decisions)
- [ ] Rate limit 100/min soft + 200/min hard per agent
- [ ] Concurrency test: 10×100 writes, zero SQLITE_BUSY surfaces
- [ ] Cross-harness E2E: A → handoff → B → back → all changes survive
- [ ] Handoff quality score ≥ 0.8 or handoff rejected
- [ ] Dashboard served from memd-server at `/dashboard/*`
- [ ] Entity search type mismatch fixed; graph renders clean
- [ ] `MemoryEntityRecord` type aligned client/server
- [ ] Dashboard env has no hardcoded IP; same-origin default
- [ ] Preferences persist across checkpoint + wake (round-trip test green)
- [ ] `memd state --compact` works; dashboard uses it
- [ ] Correction UI persists trust-rank-4 corrections
- [ ] Playwright E2E: every dashboard route loads with zero console errors
- [ ] Reinforcement-extended half-life: A/B shows accessed items persist longer
- [ ] `Episode` schema + FTS5 index created; dream populates nightly
- [ ] Runtime procedure detection: 3×-repeat → candidate in < 1s
- [ ] Skill gating: score ≥ 0.7 → Approved; < 0.7 → Rejected; only Approved surfaces
- [ ] Autodream + autoevolve loops wired in worker; structured logs
- [ ] Consolidation quality gate rejects < 0.8 outputs
- [ ] Decay calibration report with 7-day real-usage data
- [ ] Obsidian two-way sync: both directions within 1 cycle
- [ ] Conflict: Lamport-higher wins; loser saved as `.conflict.md`
- [ ] Harness adapters each < 200 lines; shared core extracted
- [ ] LRU turn cache bounded at 100 entries
- [ ] `SKILL.md` versioned and surfaced by adapters
- [ ] MCP server supports stdio/http/sse transports
- [ ] RAG sidecar: 5s timeout, auto-disable at > 10% timeout rate
- [ ] `user_id: Option<String>` on `MemoryItem`; user-scope filter applied
- [ ] `memd diagnostics report --m4` outputs every dimension with no "not measured" gaps

If any of these fail, M4 is not done. Period.

## Key Files

| What | Where |
|------|-------|
| Schema: MemoryItem | `crates/memd-schema/src/lib.rs:220-249` |
| Schema: HiveHandoffPacket | `crates/memd-schema/src/lib.rs:1113-1125` |
| Schema: coordination_mode | `crates/memd-schema/src/lib.rs:1134, 1191-1193` |
| Schema: HiveSessionState state_version | `crates/memd-schema/src/lib.rs:1410` |
| Schema: Procedure | `crates/memd-schema/src/lib.rs:1803-1834` |
| Schema: MemoryPolicyDecay | `crates/memd-schema/src/lib.rs:2186-2196` |
| Schema: MemoryPolicySkillGating | `crates/memd-schema/src/lib.rs:2250-2257` |
| Schema: MemoryEntityRecord | `crates/memd-schema/src/lib.rs:1405-1422` |
| Server: routes (queen ops) | `crates/memd-server/src/routes.rs:893-1069` |
| Server: routes (consolidation) | `crates/memd-server/src/routes.rs:1315-1502` |
| Server: store (SQLite pragmas) | `crates/memd-server/src/store.rs:90-91, 422-423` |
| Server: store (decay formula) | `crates/memd-server/src/store.rs:636-783` |
| Server: store (decay core) | `crates/memd-server/src/store.rs:712-714` |
| Server: store migrations | `crates/memd-server/src/store_migrations.rs` |
| Server: inspection (explain) | `crates/memd-server/src/inspection/mod.rs:12-78` |
| Server: working memory compiler | `crates/memd-server/src/working/mod.rs:15-272` |
| Server: helpers (build_context) | `crates/memd-server/src/helpers.rs:27-113` |
| Server: procedural detection | `crates/memd-server/src/procedural.rs:447-612` |
| Server: memory policy snapshot | `crates/memd-server/src/working/mod.rs:717-826` |
| Worker: main loop | `crates/memd-worker/src/main.rs:44-69` |
| Client: wake packet | `crates/memd-client/src/runtime/resume/wakeup.rs:44-83` |
| Client: harness turn cache | `crates/memd-client/src/harness/cache.rs:1-243` |
| Client: Obsidian watch | `crates/memd-client/src/obsidian/watch_runtime.rs:1-62` |
| Dashboard: client types | `apps/dashboard/app/lib/types.ts:100-108, 400-402` |
| Dashboard: graph route | `apps/dashboard/app/routes/graph.tsx:35-38` |
| Dashboard: env | `apps/dashboard/.env` |
| Tests: existing suite | `crates/memd-server/src/tests/mod.rs` |
| Tests: benchmark registry | `docs/verification/benchmark-registry.json` |
| New: errors module | `crates/memd-server/src/errors.rs` |
| New: status module | `crates/memd-server/src/status.rs` |
| New: episodes module | `crates/memd-server/src/episodes.rs` |
| New: hive/divergence | `crates/memd-server/src/hive/divergence.rs` |
| New: middleware/rate_limit | `crates/memd-server/src/middleware/rate_limit.rs` |
| New: worker/dream | `crates/memd-worker/src/dream.rs` |
| New: worker/autoevolve | `crates/memd-worker/src/autoevolve.rs` |
| New: obsidian/export | `crates/memd-client/src/obsidian/export.rs` |
| New: obsidian/conflict | `crates/memd-client/src/obsidian/conflict.rs` |
| New: MCP server | `crates/memd-client/src/mcp/mod.rs` |
| New: RAG sidecar | `crates/memd-server/src/rag/mod.rs` |
| New: CLI backup/restore | `crates/memd-client/src/cli/backup.rs` |
| New: CLI explain | `crates/memd-client/src/cli/explain.rs` |
| New: CLI spine verify | `crates/memd-client/src/cli/spine.rs` |
| New: CLI calibrate | `crates/memd-client/src/cli/calibrate.rs` |
| New: CLI mcp serve | `crates/memd-client/src/cli/mcp.rs` |
| New: CLI procedure score | `crates/memd-client/src/cli/procedure.rs` |
| New: SKILL.md asset | `crates/memd-client/assets/SKILL.md` |

## Phase Docs (read before starting each step)

- [[docs/phases/phase-k2-observability.md]]
- [[docs/phases/phase-l2-hive-hardening.md]]
- [[docs/phases/phase-i2-human-dashboard.md]]
- [[docs/phases/phase-m2-overnight-evolution.md]]
- [[docs/phases/phase-n2-integrations-polish.md]]
- [[docs/verification/NODE-VERIFICATION-MATRIX.md]]
- [[docs/verification/MEMD-10-STAR.md]]
- [[docs/theory/2026-04-14-donor-extraction-to-v2-phases.md]]

## Theory Locks (read only if stuck on a design decision)

- [[memd-theory-lock-v1]] (L2 hive semantics, N2 multi-user)
- [[memd-canonical-promotion-theory-lock-v1]] (M2-evo gating, decay reinforcement)
- [[memd-evaluation-theory-lock-v1]] (K2 latency SLA, M2-evo scoring threshold)

## Prior-Milestone References

- [[docs/plans/M1-EXECUTION-PLAN.md]] — Make It Work (baseline capabilities)
- [[docs/plans/M2-EXECUTION-PLAN.md]] — Make It Correct (lane routing, corrections)
- [[docs/plans/M3-EXECUTION-PLAN.md]] — Make It Provable (enforcement + measurement)
