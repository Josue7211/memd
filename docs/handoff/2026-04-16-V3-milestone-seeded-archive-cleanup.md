# Handoff Packet — V3 Active, M4 Deferred, Entry at A3

- created: `2026-04-16` (revised after V3-active flip; renamed 2026-04-17 after phase-ID-to-execution-order rename)
- from: `claude-code@session-7eab5dde`
- to: next session executing **V3 / A3** (sidecar wire-up)
- branch: `research/mining`
- last clean commit: `09f90d4 memd auto-commit: V3-flip checkpoint`
- status: V3 active. M4 deferred mid-flight. ROADMAP_STATE on v3/V3/A3. Phase IDs renamed 2026-04-17 so A=1st…E=5th.

## 2026-04-17 rename (read first)

Phase IDs now match execution order. Old → new:

| Old ID | New ID | Phase name |
|--------|--------|------------|
| B3 | **A3** | Activate Retrieval (runs 1st) |
| F3 | **B3** | Reranker + Embeddings (runs 2nd) |
| E3 | **C3** | Atlas at Recall (runs 3rd) |
| C3 | **D3** | Consolidation + Sessions (runs 4th) |
| A3 | **E3** | Bench Honesty (runs 5th) |

Everywhere below that says "B3" for Activate Retrieval means **A3**. Everywhere that says "A3" for Bench Honesty means **E3**. The rest of this packet has been patched inline; this note exists only so a reader who already has the old IDs in their head sees the map up front.

## Why this packet exists

Two parallel things landed this session, neither yet committed:

1. **V3 milestone seeded** — new "Make It Compete" version stacked on top of M4. Bench-delta gated. Targets memd→inspiration parity (mempalace 96.6% LongMemEval pure-cosine ceiling). 5 phases with `pending` status, full deliver/gate/donor anchors, ROADMAP entry trimmed to short table.
2. **V1 + stale V2 phase docs archived** — 9 files moved to `docs/phases/archive/`. Wikilinks updated in 3 docs that referenced them. WHERE-AM-I refreshed off v1/Phase G truth.

I2 work has NOT started. M4 step order unchanged: I2 (next) → M2-evo → N2 → V3.

## What V3 looks like

Roadmap block: `ROADMAP.md` `## V3: Make It Compete` (line ~194 onward). Short table, brief gate format, links to phase docs for detail.

Five phases, all `pending`, all bench-delta-gated (IDs now in execution order):

| Phase | Name | Depends | Bench target |
|-------|------|---------|--------------|
| A3 | Activate Retrieval | (M4 dep relaxed) | LME 0.86→0.93, MemBench 0.35→0.50 |
| B3 | Reranker + Embeddings | A3 | LME 0.93→0.97, LoCoMo 0.42→0.55 |
| C3 | Atlas at Recall | A3, B3 | LoCoMo 0.55→0.65 |
| D3 | Consolidation + Sessions | A3, B3, C3 | LME 0.97→0.98, LoCoMo 0.65→0.70 |
| E3 | Bench Honesty | A3 | ConvoMem 0→0.50, MemPalace cross-baseline live |

Phase docs:
- [[docs/phases/phase-b3-activate-retrieval.md]]
- [[docs/phases/phase-c3-reranker-embeddings.md]]
- [[docs/phases/phase-d3-atlas-at-recall.md]]
- [[docs/phases/phase-e3-consolidation-sessions.md]]
- [[docs/phases/phase-f3-bench-honesty.md]]

Each carries: goal, why-this-phase, deliverables, pass gate (pre/post bench numbers + regression budget + evidence), donor anchors to `.memd/lanes/architecture/A2-*` extraction pack, rollback flags, out-of-scope.

## Why V3 exists at all

Polish (M4) ships visibility. V3 ships **score**. Three diagnoses drive every V3 phase:

1. **Sidecar disabled** — `.memd/config.json:48` has `rag.enabled=false`, `memd-server` does not import `memd-rag` (Grep verified), bench backend defaults to `lexical` (`crates/memd-client/src/benchmark/public_benchmark.rs:1439`). Memd's 0.860 LongMemEval is keyword-only retrieval. Dense path unreached.
2. **Atlas dormant** — schema/tables/API exist; nothing writes at ingest, nothing reads at recall (`docs/backlog/2026-04-14-atlas-fully-built-completely-dormant.md`).
3. **ConvoMem at 0.000** — almost certainly adapter/routing bug, not retrieval. Surfaces a zero in a four-row leaderboard. High optics cost, small fix effort.

Inspiration ceiling proof: mempalace 96.6% pure cosine, 100% with rerank ([[.memd/lanes/architecture/A2-09-retrieval-pipeline.md]]). BGE-large = +3.5pp empirical ([[.memd/lanes/architecture/A2-10-embedding-strategy.md]]). Storage-time dedup at 0.15 cosine ([[.memd/lanes/architecture/A2-04-dedup.md]]). Knowledge graph with valid_from/valid_to triples ([[.memd/lanes/architecture/A2-02-atlas-entity-graph.md]]).

## Why V3 starts now (not after M4)

User flip 2026-04-16: M4 polish "doesn't move the score." Score is the goal. V3 ships score. M4 (I2/M2-evo/N2) deferred until V3 lands or until a V3 phase needs M4 infra (e.g. D3 may need M2-evo's overnight loop — at that point cherry-pick the loop into D3, don't resurrect all of M4).

K2 + L2 already done on `main` + `research/mining` — they were the M4 pieces that hardened core surfaces (observability, hive integrity). I2 (dashboard), M2-evo (overnight evo infra), N2 (integrations polish) are paused. Resume after V3.

A3's `depends_on: [M4]` was relaxed to `[]` in this flip — sidecar wiring is orthogonal to dashboard/observability/hive polish. See `docs/phases/phase-b3-activate-retrieval.md` frontmatter `notes` line.

## V1 + stale V2 archive

Files moved via `git mv` to `docs/phases/archive/`:

| Original | Reason |
|----------|--------|
| `phase-a-raw-truth-spine.md` | V1 verified, superseded by f2 |
| `phase-b-session-continuity.md` | V1 verified, superseded by c2 |
| `phase-c-typed-memory.md` | V1 verified, superseded by g2 |
| `phase-d-canonical-truth.md` | V1 verified, superseded by d2 |
| `phase-e-wake-packet-compiler.md` | V1 verified, superseded by b2 |
| `phase-f-memory-atlas.md` | V1 verified, superseded by e2 |
| `phase-g-procedural-learning.md` | V1 verified, no V2 successor (procedural moved into M2-evo) |
| `phase-i-human-dashboard.md` | V1 stub, superseded by V2 i2 |
| `phase-a3-mining.md` | Stale V2 phase ID, conflicted with new V3 A3 phase ID |

Wikilinks repaired in:
- `docs/WHERE-AM-I.md` — refreshed Current Truth block off v1/Phase G stale state, now points at v2/M4/I2 with v3 mention
- `docs/verification/NODE-VERIFICATION-MATRIX.md` (lines 85-92) — added `archive/` prefix to V1-phase column
- `docs/verification/milestones/MILESTONE-v1.md` (lines 34-40) — added `archive/` prefix to phase wikilinks (this file is itself a historical archive, paths to `archive/` are appropriate)

Verification: `Grep '\[\[phase-(a-raw|b-session|c-typed|d-canonical|e-wake-packet|f-memory-atlas|g-procedural|h-hive|i-human-dashboard|a3-mining)' --glob '!*/archive/*'` returns no matches.

## Commit plan (uncommitted as of this packet)

`git status` shows:
- **M**: `ROADMAP.md` (V3 block + Status Snapshot pointer)
- **M**: `docs/WHERE-AM-I.md` (refreshed)
- **M**: `docs/verification/NODE-VERIFICATION-MATRIX.md` (wikilink fix)
- **M**: `docs/verification/milestones/MILESTONE-v1.md` (wikilink fix)
- **R** ×9: archive moves under `docs/phases/`
- **??** ×5: new V3 phase docs

Two-commit suggestion (next session decides):
1. `docs: archive v1 + stale v2 phase docs, repair wikilinks` — all R + 3 wikilink edits + WHERE-AM-I refresh
2. `docs: seed V3 milestone with bench-delta gates` — ROADMAP edit + 5 V3 phase docs

Single-commit alternative (lower noise): bundle as `docs: archive v1/v2 + seed V3 bench-parity milestone`.

User has not asked for commit yet. Hold until asked.

## V3 execution order

| # | Phase | Depends | Why this slot |
|---|-------|---------|---------------|
| 1 | **A3 Activate Retrieval** | (M4 dep relaxed) | Sidecar disabled = bench is keyword-only. Foundational. |
| 2 | **B3 Reranker + Embeddings** | A3 | Reranker only meaningful once dense path is live. |
| 3 | **C3 Atlas at Recall** | A3, B3 | Multi-hop expansion stacks on dense+rerank. |
| 4 | **D3 Consolidation + Sessions** | A3, B3, C3 | Long-tail decay tuning needs all retrieval signals first. May cherry-pick M2-evo overnight-loop infra. |
| 5 | **E3 Bench Honesty** | A3 | ConvoMem adapter fix can start in parallel with A3 (adapter bug, not retrieval). Formal phase merge ships last. |

## A3 entry — first move

`docs/phases/phase-b3-activate-retrieval.md` deliverable 1: **wire `memd-sidecar` into `memd-server` retrieval**.

Concrete starting points (verified diagnosis):
- `.memd/config.json:48` — `rag.enabled = false` → flip to `true` after server-side import lands
- `crates/memd-server/src/lib.rs` (or equivalent) — `memd-rag` is NOT imported (Grep verified). Add it, route entity-search and lookup through the dense path.
- `crates/memd-client/src/benchmark/public_benchmark.rs:1439` — bench backend default = `lexical`. Flip to `sidecar` once `MEMD_RAG_URL` resolution is documented.

Donor anchor: `.memd/lanes/architecture/A2-09-retrieval-pipeline.md` (mempalace pipeline shape: sanitize → embed → vector → filter → rank → assemble).

Then deliverables 2-6 from the A3 phase doc (config defaults, query sanitization, layered context, priority dedup, status admission cap).

**Do not start with deliverables 2-6 before deliverable 1.** The bench delta proves the dense path is reaching the bench harness — without that proof, every subsequent V3 phase is gambling.

## E3 parallel option (ConvoMem adapter)

ConvoMem adapter fix (E3 deliverable 1) is parallelizable with A3 — it's an adapter/routing bug, not a retrieval-quality problem. Worth queuing on a side branch off `main` if a parallel agent is available. Formal E3 phase merge still sits at end of V3 to capture the whole picture (cross-baseline replay + per-phase leaderboard refresh + bench claim governance).

## M4 deferred — what's parked

Plan: `docs/plans/M4-EXECUTION-PLAN.md` (still authoritative for M4 when it resumes).

- **I2 Human Dashboard** — 11 substeps, entry was `I2.2 fix EntitySearchResult type mismatch`. See prior packet [[docs/handoff/2026-04-16-L2-complete-next-I2.md]] for full I2 entry plan when M4 resumes.
- **M2-evo Overnight Evolution** — infra; D3 may need its dream-loop pieces, cherry-pick at that point.
- **N2 Integrations Polish** — last in M4 order.

Test baseline at deferral point: 190 server + 430 client, both green. No source touched this session.

## Open questions for next session

1. **E3 parallel start** — queue ConvoMem adapter fix as a side branch off `main` while A3 runs on `research/mining`?
2. **D3 ↔ M2-evo cherry-pick boundary** — when D3 wants overnight loop infra, do we cherry-pick from the M4 plan or rebuild minimal? Decide at D3 entry, not now.

## Donor lane (where V3 detail lives)

All V3 phase docs anchor to extraction pack:

- `.memd/lanes/architecture/A2-01-benchmark-harness.md` — mempalace bench harness
- `.memd/lanes/architecture/A2-02-atlas-entity-graph.md` — KG schema, pre-graph extraction, temporal corrections
- `.memd/lanes/architecture/A2-04-dedup.md` — storage-time dedup at 0.15 cosine
- `.memd/lanes/architecture/A2-09-retrieval-pipeline.md` — sanitize → embed → vector → filter → rank → assemble
- `.memd/lanes/architecture/A2-10-embedding-strategy.md` — MiniLM default, BGE-large +3.5pp
- `.memd/lanes/architecture/A2-11-context-compilation-profile.md` — supermemory priority dedup
- `.memd/lanes/architecture/A2-13-temporal-freshness.md` — TTL/freshness signals

Read these BEFORE starting any V3 phase. Phase docs are **plans**; the lane docs are the **proof**.
