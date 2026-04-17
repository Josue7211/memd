# Handoff Packet — V3 Milestone Seeded + V1/V2 Archive Cleanup

- created: `2026-04-16`
- from: `claude-code@session-7eab5dde`
- to: next session picking up M4 execution OR seeding V3 work
- branch: `research/mining`
- last commit: `ecce5ef docs: finalize M4 handoff packet`
- status: working tree DIRTY (no commit yet — see "Commit plan" below)

## Why this packet exists

Two parallel things landed this session, neither yet committed:

1. **V3 milestone seeded** — new "Make It Compete" version stacked on top of M4. Bench-delta gated. Targets memd→inspiration parity (mempalace 96.6% LongMemEval pure-cosine ceiling). 5 phases with `pending` status, full deliver/gate/donor anchors, ROADMAP entry trimmed to short table.
2. **V1 + stale V2 phase docs archived** — 9 files moved to `docs/phases/archive/`. Wikilinks updated in 3 docs that referenced them. WHERE-AM-I refreshed off v1/Phase G truth.

I2 work has NOT started. M4 step order unchanged: I2 (next) → M2-evo → N2 → V3.

## What V3 looks like

Roadmap block: `ROADMAP.md` `## V3: Make It Compete` (line ~194 onward). Short table, brief gate format, links to phase docs for detail.

Five phases, all `pending`, all bench-delta-gated:

| Phase | Name | Depends | Bench target |
|-------|------|---------|--------------|
| B3 | Activate Retrieval | M4 | LME 0.86→0.93, MemBench 0.35→0.50 |
| F3 | Reranker + Embeddings | B3 | LME 0.93→0.97, LoCoMo 0.42→0.55 |
| E3 | Atlas at Recall | B3, F3 | LoCoMo 0.55→0.65 |
| C3 | Consolidation + Sessions | B3, F3, E3 | LME 0.97→0.98, LoCoMo 0.65→0.70 |
| A3 | Bench Honesty | B3 | ConvoMem 0→0.50, MemPalace cross-baseline live |

Phase docs (new, untracked):
- [[docs/phases/phase-b3-activate-retrieval.md]]
- [[docs/phases/phase-f3-reranker-embeddings.md]]
- [[docs/phases/phase-e3-atlas-at-recall.md]]
- [[docs/phases/phase-c3-consolidation-sessions.md]]
- [[docs/phases/phase-a3-bench-honesty.md]]

Each carries: goal, why-this-phase, deliverables, pass gate (pre/post bench numbers + regression budget + evidence), donor anchors to `.memd/lanes/architecture/A2-*` extraction pack, rollback flags, out-of-scope.

## Why V3 exists at all

Polish (M4) ships visibility. V3 ships **score**. Three diagnoses drive every V3 phase:

1. **Sidecar disabled** — `.memd/config.json:48` has `rag.enabled=false`, `memd-server` does not import `memd-rag` (Grep verified), bench backend defaults to `lexical` (`crates/memd-client/src/benchmark/public_benchmark.rs:1439`). Memd's 0.860 LongMemEval is keyword-only retrieval. Dense path unreached.
2. **Atlas dormant** — schema/tables/API exist; nothing writes at ingest, nothing reads at recall (`docs/backlog/2026-04-14-atlas-fully-built-completely-dormant.md`).
3. **ConvoMem at 0.000** — almost certainly adapter/routing bug, not retrieval. Surfaces a zero in a four-row leaderboard. High optics cost, small fix effort.

Inspiration ceiling proof: mempalace 96.6% pure cosine, 100% with rerank ([[.memd/lanes/architecture/A2-09-retrieval-pipeline.md]]). BGE-large = +3.5pp empirical ([[.memd/lanes/architecture/A2-10-embedding-strategy.md]]). Storage-time dedup at 0.15 cosine ([[.memd/lanes/architecture/A2-04-dedup.md]]). Knowledge graph with valid_from/valid_to triples ([[.memd/lanes/architecture/A2-02-atlas-entity-graph.md]]).

## Why polish (M4) still ships first

V3 is opt-in and bench-gated. M4 is human-facing visibility (dashboard, observability, hive integrity). User chose explicitly: "lets not delete ito fc, and make a new V for improvement." M4 finishes on schedule; V3 stacks after, no merge without bench delta.

ConvoMem fix (A3) is the one V3 sub-task that is **parallelizable with M4** — it's an adapter bug not a retrieval issue, can run alongside if convenient. Formal A3 phase merge sits at end of V3 to capture full picture.

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

## Where we are on M4 (unchanged from prior packet)

Plan: `docs/plans/M4-EXECUTION-PLAN.md`. Serial order:

- Step 1 **K2 (Observability)** — done on `main`, 10/10
- Step 2 **L2 (Hive Hardening)** — done on `research/mining`, 9/9 (last commit `7ce2b7c`)
- Step 3 **I2 (Human Dashboard)** — NEXT (11 substeps; see [[docs/handoff/2026-04-16-L2-complete-next-I2.md]] for I2 entry plan)
- Step 4 **M2-evo (Overnight Evolution)** — blocked on K2 (met)
- Step 5 **N2 (Integrations Polish)** — blocked on I2 + M2-evo

Test baseline at L2 exit: 190 server + 430 client, both green. No regression introduced this session (docs-only edits + git mv, no source touched).

## I2 entry (unchanged — copy from prior packet for convenience)

First substep is `I2.2`: fix `EntitySearchResult` type mismatch causing graph-page crash. See [[docs/backlog/2026-04-15-graph-page-crash-entity-search-type-mismatch.md]]. Then `MemoryEntityRecord` mismatch ([[docs/backlog/2026-04-15-memory-entity-record-type-mismatch.md]]), then preference persistence ([[docs/backlog/2026-04-15-memd-preferences-not-persisted-across-sessions.md]]), then dashboard-served-from-server ([[docs/backlog/2026-04-15-dashboard-not-served-from-memd-server.md]]).

## Open questions for next session

1. **Commit timing** — bundle now or after I2.2 lands? User instinct typically: bundle docs separately from code.
2. **WHERE-AM-I content** — refreshed minimally to v2/M4/I2 truth. Could expand "If You Need More Detail" section with v3 pointers; current version is conservative.
3. **A3 parallel start** — ConvoMem adapter fix can run alongside M4. Worth queuing as a parallel branch off `main` (not blocking I2)?

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
