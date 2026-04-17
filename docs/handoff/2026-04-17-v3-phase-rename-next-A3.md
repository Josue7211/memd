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

V3 phase IDs were renamed so alphabet matches execution order. V3 reframed: **best product, not fastest bench score**. Every phase now dual-gated (bench delta AND product win). Working tree clean. Next concrete action: wire `memd-sidecar` into `memd-server` retrieval (`memd-server` currently has no `memd-rag` import). Everything below is the unpacking.

## V3 framing (read before executing any phase)

V3 is the **FINAL memory OS**. Not a better v1. Not catch-up. The last version anyone needs. Product must be **great without RAG**. Sidecar is an optional accelerator, not load-bearing. Competitor services (mempalace, supermemory, letta, mem0) out-perform memd today on surfaces benches don't measure (correction UX, atlas navigation, provenance transparency, episodic recall UX, agent handoff quality, hive divergence receipts, dedup explainability) and they do it without treating RAG as a crutch. Memd won't either.

**Three canonical decisions logged to memd on 2026-04-17:**
1. *"not looking for the fastest ship, looking for the best product"* — every phase is dual-gated (bench + product).
2. *"all the other services don't rely on rag for better benches and truly we shouldn't either, is supposed to be optional and a great product even without"* — A3 reworked to ship intrinsic retrieval wins first; sidecar becomes flag-gated accelerator with measured delta.
3. *"we need at least 70% on ALL benches WITHOUT the sidecar — that's where our competition is at — that's the bare minimum — this is the FINAL memory OS, we need to go above and beyond"* — V3 completion gate is ≥0.70 intrinsic on LongMemEval, LoCoMo, MemBench, and ConvoMem. Three of four start below 0.70. Floor is minimum, stretch is goal.

Every V3 phase doc now has a `## Product Win` section alongside `## Pass Gate`. Bench reports will carry two columns going forward: **intrinsic** (sidecar off, primary, 0.70 floor on all four) and **accelerated** (sidecar on, bonus, ≥+0.02 delta required).

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

## Next move — A3 Part 1 (intrinsic retrieval, sidecar stays off)

Phase doc: [[docs/phases/phase-a3-activate-retrieval.md]] (rewritten 2026-04-17 after the RAG-optional framing correction).

A3 is now two parts. Part 1 ships **intrinsic wins with sidecar OFF**. Part 2 wires sidecar as an optional accelerator. The first concrete work is entirely Part 1.

**Part 1 deliverables (start here, no sidecar required):**

1. **FTS5 scoring overhaul** — tune k1, b, per-field weights; move off default BM25 defaults.
2. **Query sanitization (SQL path)** — port mempalace `query_sanitizer.py` to Rust, applied before every FTS call.
3. **Atlas-driven query expansion** — when query names an entity with atlas edges, expand synonyms/aliases before FTS. No vectors needed.
4. **Layered wake packet** — L0/L1/L2/L3 assembled from SQL; no embeddings required.
5. **Priority dedup (SQL-side)** — canonical > working > search, exact-string dedup after fetch.
6. **Status admission cap** — kind=Status ≤ 2 in wake output, or TTL hard-cut at 1h.

Part 1 pass gate (updated 2026-04-17 per user directive — FINAL memory OS, 70% intrinsic floor on ALL benches): **intrinsic LongMemEval ≥ 0.92, MemBench ≥ 0.70, LoCoMo ≥ 0.55, ConvoMem ≥ 0.10** with `rag.enabled=false`. MemBench must clear the V3 0.70 floor here; LoCoMo clears it in B3, ConvoMem in E3. If these numbers don't move, do not start Part 2.

**Part 2 (after Part 1 ships):**

7. Add `memd-rag` dep to `crates/memd-server/Cargo.toml`. Wire `RagClient` behind `rag.enabled=true` flag. Sidecar contributes candidates into the ranking pipeline Part 1 built; it does not replace the intrinsic path.
8. Dual-mode benchmark: every run reports `intrinsic_score` and `accelerated_score` side by side.
9. Default stays off — `rag.enabled=false` remains the shipped default.

Target delta in accelerated mode: +0.03 on LME, +0.04 on MemBench vs intrinsic. If sidecar adds less than +0.02 on any metric, it's not pulling weight and should either be retuned or left disabled.

**Do not wire the sidecar before Part 1 ships.** That would make the sidecar load-bearing and put us right back where we are today — 0.86 without it, "just turn it on" as a crutch. The whole point of A3 is to not be that.

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

## Boot orientation

`memd wake --output .memd` + reading this packet is the whole orientation. Do not re-grep the codebase to rediscover state — the A3.1 pre-state (config false, server no `memd-rag`, bench default `lexical`) is already verified and written above. If you are tempted to verify, you are duplicating work memd already did.

One git command is still worth it to confirm the branch tip matches what this packet claims:

```bash
git log --oneline -1   # expect d0717f5 (handoff) or later
```

If that fails, state has drifted — re-read memd, not the codebase.
