# Where Am I

Use this after session clear or when an agent feels lost.

## Read In This Order

1. [[ROADMAP]]
2. [[docs/handoff/2026-04-18-b3-part2-runtime-green-score-red-next-retrieval.md|B3 runtime-green / score-red handoff packet (2026-04-18)]] — supersedes [[docs/handoff/2026-04-18-b3-part2-prereq-green-next-part2.md]]
3. backlog items linked from `ROADMAP`
4. active phase summary note: [[phase-b3-activate-retrieval]]

## Current Truth

- active version: `v3` (FINAL memory OS — 70% intrinsic floor on all four benches) — v2/M4 deferred mid-flight
- active milestone: `V3: Make It Compete`
- active phase: `B3: Intrinsic Retrieval (RAG-Optional)` (in progress) — A3 is closed; B3 Part 2 plumbing landed, the 500-Q intrinsic product-path run now completes, and the remaining blocker is score quality (`0.828 < 0.92`)
- next step: `B3 retrieval-quality pass` — inspect the 500-Q LongMemEval misses on the memd-backed product path and move `session_recall_any@5` from `0.828` to the B3 target `≥0.92`. See [[phase-b3-activate-retrieval]].
- V3 execution order: A3 → B3 → C3 → D3 → E3 → F3 (Continuity Foundation → Intrinsic Retrieval → Reranker → Atlas → Consolidation → Bench Honesty). IDs match execution order after the 2026-04-17 reshuffles.
- main blocker: `longmemeval-intrinsic-primary-score-still-below-target` — the harness/runtime issue is fixed, but the intrinsic primary metric is still red
- v2/M4 status: K2 + L2 done; I2 + M2-evo + N2 paused. Resume after V3 or cherry-pick when needed.

## If You Need More Detail

- harness behavior: [[docs/core/setup.md|Setup]]
- detailed roadmap theory: [[2026-04-11-memd-ralph-roadmap]]
- canonical theory: [[2026-04-11-memd-canonical-theory-synthesis]]
- authoring conventions + where files go: see `docs/README.md` (if present) or the `## Process` section in [[ROADMAP]]

## Rule

If this file, `ROADMAP`, and live memd continuity disagree, fix the docs or the
memory immediately. Do not invent a third truth surface.
