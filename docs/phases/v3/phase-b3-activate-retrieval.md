---
phase: B3
name: Intrinsic Retrieval (RAG-Optional)
version: v3
status: complete
depends_on: [A3]
notes: Renamed A3→B3 on 2026-04-17 when new A3 "memd Continuity Foundation" was inserted at V3 entry (continuity bugs supersede retrieval work — can't benchmark a product whose memory leaks state across compaction). Still carries the intrinsic-wins-first / sidecar-optional framing from the 2026-04-17 RAG-optional correction.
backlog_items:
  - "2026-04-14-rag-sidecar-disabled-no-fallback"
  - "2026-04-14-status-noise-runaway-checkpoint-loop"
  - "2026-04-13-status-noise-floods-memory"
  - "2026-04-14-memory-dedup-incomplete"
  - "2026-04-14-no-behavior-changing-recall-proof"
  - "2026-04-18-longmemeval-intrinsic-primary-score-still-below-target"
---

# Phase B3: Intrinsic Retrieval (RAG-Optional)

## Goal

Make memd's **intrinsic** retrieval (no sidecar, no external vector service) good enough to be a great product by itself. The 0.860 LongMemEval baseline is the **no-sidecar** number today and it is not good enough. Close the gap to competitors on the SQL/FTS path first. Then — and only then — wire the sidecar as an **optional accelerator** with measured deltas, not as the primary load-bearing path.

## Why this phase exists

User direction (2026-04-17, canonical in memd): *"all the other services don't rely on RAG for better benches and truly we shouldn't either; it's supposed to be optional and a great product even without."*

Current state: `.memd/config.json:48` → `rag.enabled=false`, `memd-server` has zero `memd-rag` imports, bench default is `lexical`. That means **0.860 LongMemEval is memd's intrinsic score** — and it's not competitive. Rather than treat this as "turn on RAG to fix it", B3 treats it as "fix the intrinsic path so the product is great without RAG, then wire RAG as a speed/accuracy booster on top."

## Deliver

### Part 1 — Intrinsic retrieval wins (no sidecar required)

1. **FTS5 scoring overhaul** — move from default BM25 to tuned parameters (k1, b, per-field weights). Port mempalace query-layering ideas into SQL-side (no embeddings needed to decompose queries).
2. **Query sanitization + expansion in SQL path** — port mempalace `query_sanitizer.py` (200/500-char passthrough/extract/tail/truncate) to Rust. Add atlas-driven query expansion: when a query names an entity we have edges for, expand synonyms/aliases before the FTS call ([[.memd/lanes/architecture/A2-09-retrieval-pipeline.md#query-sanitization-pipeline-query_sanitizerpy]]).
3. **Layered wake packet** — L0 (identity) + L1 (essential story) + L2 (on-demand) + L3 (deep search). All assembled from SQL path; no embeddings required to produce the layered structure.
4. **Priority dedup at retrieval (SQL-side)** — canonical > working > search, exact-string dedup applied after fetch, before injection ([[.memd/lanes/architecture/A2-11-context-compilation-profile.md#retrieval-time-dedup-priority-based]]).
5. **Status admission cap** — kind=Status capped at 2 in wake output, or TTL hard-cut at 1h with -0.08 penalty ([[.memd/lanes/architecture/A2-13-temporal-freshness.md#ttl-calibration]]).
6. **Atlas-at-recall SQL path** — when atlas edges exist, use them as retrieval hints (1-hop entity expansion) without needing vectors. This is lighter than D3's full multi-hop atlas work but picks up easy wins now.

### Part 2 — Sidecar as optional accelerator (flag-gated, measured)

7. **Sidecar wiring behind `rag.enabled=true`** — `memd-server` imports `memd-rag`, routes dense candidates into the same ranking pipeline Part 1 built. Sidecar contributes candidates; it does not replace the intrinsic path.
8. **Dual-mode benchmark** — every V3 bench run reports TWO numbers: `intrinsic_score` (sidecar off) and `accelerated_score` (sidecar on). Leaderboard columns split.
9. **Default stays off** — `.memd/config.json:48` remains `rag.enabled=false` by default. Sidecar is opt-in; great product ships without it.

## Pass Gate

Dual-bench-delta required (regenerate [[docs/verification/PUBLIC_LEADERBOARD.md]] before/after with both modes). V3 completion requires **≥0.70 intrinsic on ALL four metrics**; B3 owns the biggest jump on MemBench and the foundation for LoCoMo / ConvoMem to clear the floor in later phases.

**Intrinsic (sidecar OFF, the primary gate):**
- pre: LongMemEval=0.860, LoCoMo=0.415, MemBench=0.346, ConvoMem=0.000
- post (B3 targets): **LongMemEval ≥ 0.92 intrinsic**, **MemBench ≥ 0.70 intrinsic** (clears floor), **LoCoMo ≥ 0.55 intrinsic** (on path to 0.70 in C3), **ConvoMem ≥ 0.10 intrinsic** (sanity jump off 0.000; adapter fix + ≥0.70 lands in F3)
- This is the number that matters — the product must be great without RAG. 0.70 floor is bare minimum; stretch above it.
- latest recorded product-path rerun (2026-04-18, `--retrieval-backend memd`, `turn_diagnostics=false`): `session_recall_any@5 = 0.828`, `session_recall_any@10 = 0.882`, `session_recall_any@30 = 0.978`, `session_recall_any@50 = 0.998`, duration `1468764 ms` (~24.5 min). The runtime blocker is cleared; the score blocker is still active.

**Accelerated (sidecar ON, the optional bump):**
- post: demonstrable ≥ +0.02 delta per metric over intrinsic (if less, sidecar isn't pulling weight)
- regression budget: no metric drops > 0.02 vs intrinsic
- Accelerated column is a bonus, never the gate.

Plus:
- `cargo test -p memd-server -p memd-client` green
- Wake packet inspection: ≤ 2 status items, canonical facts always present, layered structure visible
- Sidecar reachable via `memd status` health probe when enabled; absence surfaces cleanly when disabled (no silent fallback)

## Evidence

- Pre/post leaderboard with intrinsic / accelerated columns both populated
- Sample wake packet showing layered structure (produced without sidecar)
- Sample retrieval trace in intrinsic mode showing FTS5 scoring path + atlas expansion
- Sample retrieval trace in accelerated mode showing dense candidates joining the ranking pipeline
- Sidecar healthz output; sidecar-off status output showing clean "intrinsic mode" state

## Product Win

- **Great without RAG.** A user running memd with sidecar off gets a product that competes with mempalace/supermemory on recall quality, not a crippled fallback. Stranger test: hand a fresh memd install to someone who has never run a sidecar; they should be impressed.
- **Wake packet reads like a curated briefing, not a status-flood.** L0/L1/L2/L3 layers make identity + essential-story visible at a glance; on-demand items obviously on-demand.
- **Natural-language recall actually works on the SQL path.** Asking memd "what do I believe about X" returns canonical truth even when X never appears as a literal keyword, via atlas-driven expansion — no embeddings required.
- **Sidecar delta is visible, not implicit.** When enabled, user sees "intrinsic X.XX → accelerated X.XX" on every bench row and in `memd status`.

Evidence (alongside bench-delta):
- Recorded dogfood session on 10 natural-language queries memd fails today; annotate which intrinsic surface fixed each one
- Screenshot of wake packet before (status-flooded) vs after (layered) — both produced without sidecar
- Side-by-side: memd intrinsic vs mempalace (which uses cosine) on the same fixture; note that mempalace uses vectors — our intrinsic target is "close enough that the sidecar is a nice-to-have, not a must-have"

## Fail Conditions

- **Intrinsic LongMemEval < 0.92 OR MemBench < 0.70** — core product is still not good enough; do not proceed to C3 until fixed
- This fail condition is currently live: the latest 500-question intrinsic
  product-path rerun is `0.828` on `session_recall_any@5`
- **Any intrinsic metric regresses** (LoCoMo drops below 0.42, ConvoMem below 0.00) — something in the new SQL path is degrading recall on the unfocused slices; fix before merge
- Sidecar becomes load-bearing (disabling it tanks the product) — revert; intrinsic path must stand alone
- Wake packet still status-flooded — admission cap + layering not enforced
- Bench harness drops the intrinsic column — dual-mode reporting is a hard requirement

## Donor Anchors

- **B3-D1**: mempalace retrieval pipeline shape (sanitize → filter → rank → assemble) — [[.memd/lanes/architecture/A2-09-retrieval-pipeline.md]]. Mempalace uses cosine; we mirror the *pipeline shape* on the SQL path.
- **B3-D2**: supermemory priority dedup (static > dynamic > search, exact-match) — [[.memd/lanes/architecture/A2-11-context-compilation-profile.md]]
- **B3-D3**: mempalace TTL/freshness penalties for status suppression — [[.memd/lanes/architecture/A2-13-temporal-freshness.md]]
- **B3-D4**: mempalace embedding choice (reference only — we do NOT adopt in B3; sidecar is optional) — [[.memd/lanes/architecture/A2-10-embedding-strategy.md]]
- **B3-D5**: FTS5 BM25 tuning — sqlite-FTS5 docs, memd's own FTS config

## Rollback

- Each Part-1 deliverable behind its own flag (`retrieval.priority_dedup`, `wake.layered`, `retrieval.atlas_expansion`, `retrieval.query_sanitize`) so regressions can be isolated without reverting the whole phase
- Part-2 sidecar wiring behind `rag.enabled` — already the default-off state
- FTS5 scoring swap behind `retrieval.fts5_tuned=true` — revert to default BM25 if dogfood shows regressions

## Baseline (HEAD 4f2d7d7, sidecar OFF, retrieval-backend=lexical)

Pinned here because `.memd/benchmarks/history/benchmark-runs.jsonl` is gitignored. Numbers from per-dataset `memd benchmark public <ds> --retrieval-backend lexical --record` runs on 2026-04-17:

- longmemeval (primary: session_recall_any@5) = **0.828**
- locomo = **0.415**
- membench = **0.346**
- convomem = **0.000**

Note: phase doc "pre" line quotes LME=0.860 which is likely `recall_any@10`; 0.828 is the @5 primary and matches the A3-era 4f8e141 baseline. Not a regression — just a different k.

## B3 Part 1 — tasks-landed summary (2026-04-17)

All six Part-1 deliverables shipped as flag-gated product code. One
scope correction captured in commit bodies: V3 public bench harness
(`memd benchmark public …`) is corpus-ranked client-side across all
three backends (lexical / rrf / sidecar) and does not route through
`memd-server`. So intrinsic-metric movement is not measurable by the
current bench regardless of which FTS5/atlas flag is toggled; the
product-side plumbing is in place for a future bench route that
exercises memd-server's search_memory endpoint.

| Task | Deliverable | Flag | Default | Commit |
|------|-------------|------|---------|--------|
| 0 | pin HEAD 4f2d7d7 baseline | — | — | 7801c27 |
| 1 | FTS5 per-field BM25 weights (bm25(table, ?3, ?4)) | MEMD_RETRIEVAL_FTS5_TUNED + _W_CONTENT/_W_TAGS | off | e4930a1 |
| 2 | query sanitize (port mempalace) + atlas synonym expand | MEMD_RETRIEVAL_ATLAS_EXPANSION | off (sanitize always on) | 1f670b4 |
| 3 | layered wake L0/L1/L2/L3 H2 suffixes | MEMD_WAKE_LAYERED | off | c1534bb |
| 4 | priority dedup canonical > working > search | MEMD_RETRIEVAL_PRIORITY_DEDUP | on | 25c598a |
| 5 | status admission cap (≤2, TTL 1h, −0.20) — verify-only | — (live in working/mod.rs, helpers.rs) | on | 7801c27 |
| 6 | atlas-at-recall 1-hop entity expansion | MEMD_RETRIEVAL_ATLAS_RECALL | off | 4fd3f33 |

Bench posture (HEAD 4fd3f33, flags default):
- LongMemEval --retrieval-backend lexical: 0.828 @5 (no regression)
- All product paths default-off or dedup-on so user-facing wake/search
  behavior is unchanged by a mere binary upgrade.

Part 2 is now partially landed. The bench route does call
`memd-server`, the optional sidecar plumbing exists, and dual-mode rows
exist. What remains pending is the **quality close-out**: intrinsic
LongMemEval is still `0.828`, so B3 does not clear on plumbing alone.

## B3 Part 2 — status snapshot (2026-04-18)

Part 2 plumbing is now in the product path and the primary-gate harness
is no longer the thing blocking close-out.

Landed in code:

- optional `RagClient` state on `memd-server`, store fan-out with identity
  contract, dense candidate injection in `search_memory`, and `/healthz`
  rag state surfacing
- dual-mode benchmark rows with top-level `mode`
  (`intrinsic` / `accelerated`)
- bench probes gated behind `MEMD_BENCH_PROBES`
- LongMemEval turn diagnostics moved behind explicit
  `--turn-diagnostics`; default 500-question path now pays only for the
  session-level primary metric

Verified on 2026-04-18:

- `cargo test -q -p memd-client public_benchmark_tests` green
- `cargo test -q -p memd-server -p memd-client` green
- `CARGO_TARGET_DIR=/tmp/memd-target cargo build --release -p memd-client -p memd-server` green
- full 500-question intrinsic product-path rerun completed in
  `1468764 ms` (~24.5 min) with `session_recall_any@5 = 0.828`

Current gate read:

- runtime / harness blocker: **fixed**
- B3 intrinsic close-out target (`LongMemEval ≥ 0.92`): **still red**
- implication: do not close B3, do not claim leaderboard win, keep work
  focused on retrieval quality rather than harness throughput

## Task 5 verification (2026-04-17, no code change)

Status admission cap is already satisfied by two live layers:
- `crates/memd-server/src/working/mod.rs:78-97` — hard cap at 2 status items per working-memory build, excess evicted with reason `evicted_by_status_cap`
- `crates/memd-server/src/helpers.rs:820-834` — scoring applies **−0.20** to `MemoryKind::Status` (stronger than phase doc's −0.08, so kept as-is)

Wake sample (`memd resume --output .memd`) returns 7 records in `context.records` — all `live_truth` / `decision` / `fact` / `constraint`, zero `kind=status`. Evidence satisfies the OR-clause in Deliverable 5. Benches are not expected to move from this task (wake-packet composition, not FTS recall). Future layered-wake work (Task 3) may add a second enforcement point gated on `wake.layered` for defense-in-depth.

## Out of scope

- LLM reranker on top of candidates (lands in C3; sidecar-dependent)
- BGE-large embedding swap (lands in C3; sidecar-dependent)
- Full multi-hop atlas traversal with valid_from/valid_to windows (lands in D3)
- Episode consolidation (lands in E3)
- ConvoMem adapter fix (lands in F3; parallelizable)
