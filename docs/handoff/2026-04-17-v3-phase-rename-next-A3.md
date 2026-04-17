---
date: 2026-04-17
from: claude-code@session-7eab5dde
to: next session executing V3 / A3.1 (sidecar wire-up)
branch: research/mining
last_clean_commit: d0c00dd docs: rename V3 phases to execution order
status: V3 active. Phase IDs now in execution order. A3 is entry. No source code touched this session.
supersedes: docs/handoff/2026-04-16-V3-milestone-seeded-archive-cleanup.md (still readable for V3-seed history; this packet is the live one)
---

# Handoff Packet — V3 Phases Renamed, Next = A3.1 Sidecar Wire-Up

## TL;DR

V3 phase IDs were renamed so alphabet matches execution order. Working tree clean. Next concrete action: wire `memd-sidecar` into `memd-server` retrieval (`memd-server` currently has no `memd-rag` import). Everything below is the unpacking.

## What happened this session

1. **Picked up V3 handoff** from 2026-04-16 packet. Verified repo state matched (clean tree, ROADMAP_STATE on v3/V3/B3 pending).
2. **User flagged bad phase IDs** — old V3 ordering put Activate Retrieval at `B3`, Reranker at `F3`, Bench Honesty at `A3`. Alphabet did not match execution. Fixed.
3. **Renamed V3 phases**, committed as `d0c00dd`:

   | Old | New | Phase name | Execution slot |
   |-----|-----|------------|----------------|
   | B3 | **A3** | Activate Retrieval | 1 (entry) |
   | F3 | **B3** | Reranker + Embeddings | 2 |
   | E3 | **C3** | Atlas at Recall | 3 |
   | C3 | **D3** | Consolidation + Sessions | 4 |
   | A3 | **E3** | Bench Honesty | 5 (ConvoMem adapter fix parallelizable) |

   Touched: 5 phase doc renames + content patches (frontmatter, donor-anchor IDs, `depends_on`, out-of-scope cross-refs), `ROADMAP.md` V3 table + `ROADMAP_STATE`, `docs/WHERE-AM-I.md`, prior handoff packet.

4. **No source code touched.** Bench-delta gates, deliverables, and dependency order unchanged — only IDs.

## Current repo state

- branch: `research/mining`
- working tree: **clean**
- last commit: `d0c00dd docs: rename V3 phases to execution order`
- `ROADMAP_STATE`: `current_phase=A3`, `phase_status=pending`, `next_step=A3.1 wire memd-sidecar into memd-server retrieval`
- memd decision logged (rename rationale + supersedes prior ordering)

## Next move — A3.1 (first deliverable of A3)

Phase doc: [[docs/phases/phase-a3-activate-retrieval.md]]

**Deliverable 1**: wire `memd-sidecar` into `memd-server` retrieval.

Concrete diagnosis (verified live this session):

- `.memd/config.json:48` → `rag.enabled = false`
- `memd-server` has **zero** `memd_rag` / `memd-rag` references (Grep confirmed). Compare: `memd-client` consumes it extensively (`RagClient`, `RagIngestRequest`, `RagRetrieveMode`, `RagRetrieveResponse` etc across 10+ files).
- `crates/memd-client/src/benchmark/public_benchmark.rs:1439` → default backend = `"lexical"` (sidecar option exists but not default).
- `crates/memd-server/Cargo.toml` — needs `memd-rag = { path = "../memd-rag" }` dep added (by analogy with `memd-client/Cargo.toml:13`).

Concrete execution order:

1. Add `memd-rag` dep to `crates/memd-server/Cargo.toml`
2. Locate server's entity-search + lookup call sites (likely in `store_entities.rs`, `routes.rs`, `atlas.rs` — inspect before touching)
3. Inject `RagClient` behind a config flag; route retrieval through it when `rag.enabled=true` AND `rag_url` resolves
4. Add fallback: if sidecar unreachable, surface error (do NOT silently fall back to lexical — that is how we got 0.86 LME in the first place)
5. Flip `.memd/config.json:48` default to `true` **after** server code compiles + tests pass
6. Flip `public_benchmark.rs:1439` default to `"sidecar"` **after** `MEMD_RAG_URL` resolution chain is documented
7. Run `cargo test -p memd-server -p memd-client`
8. Regenerate [[docs/verification/PUBLIC_LEADERBOARD.md]] pre/post to prove bench delta reaches harness

**Do not start deliverables 2–6** (query sanitization, layered wake, priority dedup, status admission cap) before deliverable 1 compiles + benches. Without bench delta proving the dense path reaches harness, every subsequent V3 phase is gambling.

## Donor anchors (read before code)

- [[.memd/lanes/architecture/A2-09-retrieval-pipeline.md]] — mempalace pipeline: sanitize → embed → vector → filter → rank → assemble
- [[.memd/lanes/architecture/A2-10-embedding-strategy.md]] — MiniLM default (384-dim, cosine, L2-normalized); BGE-large +3.5pp lands in B3 (Reranker)
- [[.memd/lanes/architecture/A2-11-context-compilation-profile.md]] — supermemory priority dedup
- [[.memd/lanes/architecture/A2-13-temporal-freshness.md]] — TTL + freshness decay

Phase docs are **plans**; lane docs are the **proof**.

## Parallel option: E3.1 (ConvoMem adapter)

`E3 Bench Honesty` deliverable 1 (ConvoMem adapter fix) is an adapter/routing bug, not a retrieval-quality problem. Parallelizable with A3 on a side branch off `main`. Current score 0.000 is almost certainly a mismatch between upstream `Salesforce/ConvoMem evidence_questions` shape and memd's adapter. Write failing test → trace adapter → fix → prove green. Formal E3 phase merge still sits at end of V3 (it also captures cross-baseline replay + per-phase leaderboard governance).

## M4 deferred — what's parked

Plan: `docs/plans/M4-EXECUTION-PLAN.md` (authoritative when M4 resumes).

- `I2 Human Dashboard` — 11 substeps; entry was `I2.2 fix EntitySearchResult type mismatch`. See [[docs/handoff/2026-04-16-L2-complete-next-I2.md]] for full I2 entry plan.
- `M2-evo Overnight Evolution` — infra; D3 may cherry-pick dream-loop pieces.
- `N2 Integrations Polish` — last in M4 order.

K2 + L2 already done on `main` + `research/mining`.

## Open questions for next session

1. Start A3.1 solo, or queue E3.1 ConvoMem adapter in parallel on a side branch off `main`?
2. `MEMD_RAG_URL` resolution chain — document before flipping bench default, or flip behind an env-var gate and document as part of the same commit?
3. D3 ↔ M2-evo cherry-pick boundary — decide at D3 entry, not now.

## Verification commands (fast orientation on next boot)

```bash
git log --oneline -5
grep -n '"enabled"' .memd/config.json               # expect false (A3.1 pre-state)
grep -rn memd_rag crates/memd-server/src | head     # expect empty (A3.1 pre-state)
sed -n '1435,1455p' crates/memd-client/src/benchmark/public_benchmark.rs
memd wake --output .memd | tail -20
```

If any of those four check lines shift, update this packet's "Current repo state" block before starting work.
