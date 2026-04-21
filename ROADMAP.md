# memd Roadmap

`ROADMAP.md` is the single roadmap source of truth for this repo.

<!-- ROADMAP_STATE
truth_date: 2026-04-20
version: v3
version_status: in_progress
current_milestone: V3
milestone_status: in_progress
current_phase: H3
phase_status: pending
next_milestone: V3
next_step: G3 Bench Adapter Parity complete 2026-04-21. All 4 benches (LongMemEval, LoCoMo, MemBench, ConvoMem) now dispatch through `PublicBenchmarkBackend` enum (Lexical/Sidecar/Rrf/Memd) via `dispatch_context_retrieval_ranked`. `--retrieval-backend memd` routes every bench through memd-server's `/memory/store` + `/memory/search` path with per-item namespace isolation. 4 parity tests (one per bench) + 1 fallback test pass. `make bench-public-memd` added for parity cadence. Test baseline: 480 passed / 1 pre-existing dataset-checksum fail (unrelated). Next: H3 Canonical Metrics — publish LongMemEval GPT-4o-judged QA accuracy, LoCoMo token F1, MemBench MC accuracy (MQI deferred), ConvoMem accuracy as the primary columns; demote `hit_rate@5` to secondary.
active_blockers: [non-canonical-metric-gap, phantom-locomo-membench-scores]
v1_status: frozen_architecture_complete
v2_status: m4_deferred_for_v3
note: V3 active — FINAL memory OS, above and beyond. Floor: ≥0.70 intrinsic on ALL benches (LME/LoCoMo/MemBench/ConvoMem) without sidecar. A3 Continuity Foundation closed 2026-04-17: Part 1 (file-interaction ledger + prime-reads + PreCompact non-blocking + PreEdit prime), Part 2 (hooks consolidation under .memd/hooks, contract v0.2, write-path hook gate, preference replay), Part 3 (file_layout v0.3 guarantee, backlog/phases regroup under v1/v2/v3, LATEST.md symlink fix, MANIFEST.json + `memd hooks doctor` green/red, lifecycle-probe NDJSON log, cross-harness pre-send validator pure function + 4 tests). B3 Part 2 plumbing landed 2026-04-18 (optional RAG fan-out, dense candidate injection, healthz rag state, dual-mode bench rows, turn diagnostics opt-in). 2026-04-20: 500-Q intrinsic product-path rerun on the real dense blend lands `session_recall_any@5 = 0.936` — gate 0.92 passed. The prior 0.828/0.882 numbers were lexical-only fallback because the bench search path left `source_agent=None` and `MemoryVisibility::Private` denied every item; one-line fix at public_benchmark.rs:1770 unblocked dense. V3 phase order: A3 ✓ → B3 Intrinsic Retrieval → C3 Reranker → D3 Atlas → E3 Consolidation → F3 Bench Honesty.
last_handoff: blockers_resolved_2026-04-21
bench_cadence: every_two_phases  # test every TWO phases per user directive 2026-04-21
-->

## Status Snapshot

- truth date: `2026-04-18`
- current version: `v3` (FINAL memory OS — above and beyond; ≥0.70 intrinsic on ALL benches without sidecar is the floor) — v2/M4 deferred mid-flight
- version status: `in_progress`
- v1 status: `frozen` — architecture complete, operations broken (honest score: 1.8/10)
- v2/M4 status: `deferred` — K2 + L2 done; I2 + M2-evo + N2 paused for V3 (M4 polish ships visibility but not score; V3 ships score)
- current milestone: `V3: Make It Compete` (Tier 5 — FINAL memory OS, above and beyond the 70% competition floor without sidecar) — in progress
- current phase: `F3: Bench Honesty` (reopened 2026-04-21). E3 code-complete 2026-04-21. F3 reopened after the paired E3+F3 bench run exposed: (1) adapter parity gap — LoCoMo/MemBench/ConvoMem hard-wired to lexical word-overlap in `build_context_retrieval_run_report`, no memd retrieval path (only LongMemEval has a backend dispatcher); (2) non-canonical metrics — memd reports retrieval-diagnostic (`hit_rate@5`) not industry-canonical (QA accuracy / token-F1 / MC accuracy). Prior "verified release board" numbers LoCoMo `>0.80` and MemBench `0.993` are retracted — they do not reproduce from head. F3 split into G3 Bench Adapter Parity → H3 Canonical Metrics → I3 Leaderboard Transparency → J3 V3 Floor Verification. V3 completion gate now binds to J3's paired canonical-metric run.
- completed: `M0` (verified), `M1` (verified 2026-04-15, eval 95), `M2` (verified 2026-04-16), `M3` (verified 2026-04-16); partial `M4`: `K2` (complete 2026-04-16), `L2` (complete 2026-04-16); `I2`/`M2-evo`/`N2` deferred
- M1: `verified` — B2+C2+F2 pass gates, remote deployed, eval 95
- M2: `verified` — D2+G2+E2+H2 pass gates, 624 tests, benchmarks zero regression, node verification 15✓/6~/0✗, remote deployed
- M3: `verified` — J2+O2+P2 pass gates, 593 tests, benchmarks zero regression, node verification 18✓/4~/0✗, CI gate all pass, amnesia checklist 15/15
- M4 progress: `K2` complete (10/10 substeps on main, last commit `235d959`); `L2` complete (9/9 substeps on `research/mining`, last commit `7ce2b7c`). Tests at L2 exit: 190 server + 430 client.
- next step: `E3 Consolidation + Sessions` — turn the current release board into a cleaner, more durable post-D3 state: long-tail LongMemEval bump, cross-session LoCoMo lift toward `≥0.80`, and consolidation/session behavior that preserves the already-cleared floor while pushing the stretch targets.
- M4 deferred: `I2` (Human Dashboard, 11 substeps), `M2-evo` (Overnight Evolution), `N2` (Integrations Polish) all paused. Resume after V3 ships bench parity, OR cherry-pick if a V3 phase needs M4 infra (e.g. M2-evo dream loop overlap with D3).
- V3 targets (floor, intrinsic/sidecar-OFF): LME ≥0.70, LoCoMo ≥0.70, MemBench ≥0.70, ConvoMem ≥0.70 — 70% is where competition sits, that is bare minimum. Stretch (intrinsic): LME ≥0.92, LoCoMo ≥0.75, MemBench ≥0.75, ConvoMem ≥0.75. Accelerated (sidecar ON) is bonus, not gate. See `## V3` block below.
- M0 benchmark baseline: LongMemEval 82.8%, LoCoMo 41.5%, MemBench 34.6%, ConvoMem 0.0% (retrieval-only)
- latest B3 intrinsic product-path rerun (2026-04-20, dense blend): LongMemEval 500Q `session_recall_any@5 = 0.936`, `@10 = 0.976`, `@30 = 1.000`, `@50 = 1.000`, duration `7916435 ms` (~132 min), `turn_diagnostics=false`. Gate 0.92 cleared. Weak type: single-session-preference 0.600 (30Qs).
- prior M1 benchmark: LongMemEval 90% full-eval (50 items, LLM-graded, `session_recall_any@10`=96%). Retrieval-only baseline (500 items) was 82.8%. These are different metrics — do not compare directly.
- M3 benchmark: LME 82.8% (gate 80%), LoCoMo 41.5% (gate 41.4%), MemBench 34.6% (gate 30%), ConvoMem 0.0% — zero regression
- 10-STAR composite: 1.8/10 (zero-generosity regrade 2026-04-14)

## Blockers

- ~~**longmemeval-intrinsic-primary-score-still-below-target**~~ — cleared 2026-04-20. 500Q `session_recall_any@5 = 0.936` on the dense blend. Root cause was bench harness `source_agent=None` against `MemoryVisibility::Private` items, not retrieval quality. Backlog note moved to closed.
- **rag-sidecar-disabled-no-fallback** (high, product): sidecar remains optional by design, so intrinsic retrieval quality must stand on its own. Any attempt to hide the intrinsic miss behind accelerated mode violates the V3 product contract. See `docs/backlog/v3/2026-04-14-rag-sidecar-disabled-no-fallback.md`.
- **atlas-fully-built-completely-dormant** (high, product): atlas recall hints exist, but the broader atlas surface is still far from the product win required in later V3 phases. See `docs/backlog/v3/2026-04-14-atlas-fully-built-completely-dormant.md`.

## Process

- Status rules, phase-flip rules, product contract: [[docs/policy/INDEX.md]]
- V1 phases (frozen): [[docs/verification/milestones/MILESTONE-v1.md]]
- V1 → V2 migration mapping: [[docs/verification/MEMD-10-STAR.md]]

## V2 Milestones (Hardening — Make It Real)

Goal: 1.8/10 → 10/10. No new architecture. Make existing architecture work.

Milestones follow the 10-STAR tiers exactly. Each tier fixes a class of gaps.
Every node in the architecture graph (see `docs/core/architecture.md` mermaid diagram)
must pass verification at each milestone gate before moving to the next. No skipping.

Each phase has a Ralph doc (bounded goal, pass gate, evidence, rollback).
Benchmarks re-run at every milestone gate. Load one phase doc at a time.

Every milestone gate verifies every node in the architecture graph. Per-node
criteria: [[docs/verification/NODE-VERIFICATION-MATRIX.md]]

### M0: Baseline + Research (no code changes)

Establish the "before" number. Extract patterns from competitors.

| Phase | Name | Status | Backlog | Detail |
| --- | --- | --- | --- | --- |
| A2 | Inspiration Extraction | `verified` | #55 | [[docs/phases/phase-a2-inspiration-extraction.md]] |
| — | Benchmark Baseline | `verified` | #45, #60 | LME 82.8%, LoCoMo 41.5%, MB 34.6%, CM 0.0% (retrieval-only) |

**Gate**: Extraction notes for 8+ targets. Benchmark numbers recorded. **PASSED 2026-04-14.**

### M1: Make It Work — Tier 1 (REOPENED — 10-STAR gaps 1-9)

Fix the operational pipeline. Every stage of the live loop must function.

| Phase | Name | Status | Gaps | Phase Doc | Theory Lock |
| --- | --- | --- | --- | --- | --- |
| B2 | Signal vs Noise | `verified` | 1, 2 | [[phase-b2-signal-vs-noise]] | [[memd-theory-lock-v1]] |
| C2 | Ghost Cleanup | `verified` | 4, 5 | [[phase-c2-ghost-cleanup]] | [[memd-theory-lock-v1]] |
| F2 | Ingestion Pipeline | `verified` | 3, 8 | [[phase-f2-ingestion-pipeline]] | [[memd-theory-lock-v1]] |

**Verified 2026-04-15**: All three phases pass gates. Eval score 95 (gate >= 65).
Commits: `566feff` (B2), `d959c36` (C2). F2 no code changes (pipeline existed).
Remote server deployed at services VM via systemd user service.

**Verification**:
- Gap details: [[docs/verification/MEMD-10-STAR.md#tier-1]]
- Node criteria: [[docs/verification/NODE-VERIFICATION-MATRIX.md]]
- Feature registry: [[docs/verification/FEATURES.md]]
- Live loop flow: [[docs/core/architecture.md#live-loop]]

**Execution plan**: [[docs/plans/M1-EXECUTION-PLAN.md]]
**Gate**: All nodes pass M1 tier. Live loop runs end-to-end.
**Test**: Store preference → new session → wake surfaces it. Stale records gone.

### M2: Make It Correct — Tier 2 (10-STAR gaps 10-17) — VERIFIED

Fix architectural gaps.

| Phase | Name | Status | Gaps | Phase Doc | Theory Lock |
| --- | --- | --- | --- | --- | --- |
| D2 | Correction Flow | `verified` | 10, 15, 16 | [[phase-d2-correction-flow]] | [[memd-canonical-promotion-theory-lock-v1]] |
| E2 | Atlas Activation | `verified` | 13, 14 | [[phase-e2-atlas-activation]] | [[memd-atlas-theory-lock-v1]] |
| G2 | Lane Architecture | `verified` | 17 | [[phase-g2-lane-architecture]] | [[memd-lane-theory-lock-v1]] |
| H2 | Recall Proof | `verified` | 11, 12 | [[phase-h2-recall-proof]] | [[memd-evaluation-theory-lock-v1]] |

**Server-side progress (2026-04-15)**:
- D2: Entity-based contradiction detection (old_item entity lookup), correction tag boost, preferred=true, trust_rank hierarchy. 3 tests added (e2e correction, contradiction marks siblings contested, existing 6 pass).
- G2: Lane tag in compact_record/wake packet. Lane diversity admission (cap=5 per lane). Backfill lanes wired to maintain. 
- E2: Salience gate removed from auto_link_entity (new entities start at 0.0). Entity link backfill wired to maintain. Atlas navigation test (4 hops). 
- H2: Cross-session correction persistence test passes. Cross-harness retrieval test passes. Eval-framework items remain: correction retention eval, lane relevance eval, A/B influence test, benchmark re-run.
- Total: 623 tests, 0 failures (+5 new M2 tests from 618 baseline).

**Decisions logged**:
- Contradiction detection only works for path-based entities (shared source_path). Content-based entities have different canonical_key → different entity → no siblings. This is a design limitation, not a bug. Future: topic-extraction entity keys.
- Query lane boost (G2.2) implemented — `query: Option<String>` added to WorkingMemoryRequest. `detect_content_lane` runs on query text to detect lane context. Differential scoring: +0.08 same-lane match, +0.02 different-lane, +0.06 no-query-context (backward compat). Reasons trace includes `lane_match`/`lane_mismatch`. 2 unit tests added. CLI `memd working --query "..."` wired.
- Entity link backfill findings appear in API response but not persisted payload_json. Non-blocking data gap.

**Remaining for M2 gate**:
- [x] H2 correction retention eval — passive checks in eval_bundle_memory (superseded leak detection)
- [x] H2 lane diversity eval — passive check in eval_bundle_memory (lane diversity)
- [x] H2 A/B influence test — server test (h2_ab_influence_corrections_improve_retrieval)
- [x] H2 benchmark re-run — LME 82.8% (gate 80%), LoCoMo 41.5% (gate 41.5%), MemBench 34.6% (gate 30%) — zero regression
- [x] Node-by-node code-level verification — 15 ✓, 6 ~, 0 ✗. [[docs/verification/milestones/M2-NODE-VERIFICATION.md]]
- [x] Deploy new binary to remote server + smoke test — deployed to openclaw via systemd user service, correction flow verified

**Verification**:
- Gap details: [[docs/verification/MEMD-10-STAR.md#tier-2]]
- Node criteria: [[docs/verification/NODE-VERIFICATION-MATRIX.md]]
- Node verification: [[docs/verification/milestones/M2-NODE-VERIFICATION.md]]
- Feature registry: [[docs/verification/FEATURES.md]]

**Execution plan**: [[docs/plans/M2-EXECUTION-PLAN.md]]
**Gate**: All nodes pass M2 tier. Correction→recall proven. Atlas navigable.
Lanes are DB-tag routing. Cross-harness works. Benchmark re-run no regression.
**VERIFIED 2026-04-16**: 626 tests, 0 failures. Node verification 15✓/6~/0✗. Benchmarks zero regression.
Binary deployed to openclaw. Correction flow smoke-tested on remote. G2.2 query lane boost implemented (differential: +0.08 same-lane, +0.02 different-lane, +0.06 no-query-context).

### M3: Make It Provable — Tier 3 (10-STAR gaps 18-23)

Fix measurement gaps.

| Phase | Name | Status | Gaps | Phase Doc | Theory Lock |
| --- | --- | --- | --- | --- | --- |
| J2 | Isolation + Trust | `verified` | 20, 23 | [[phase-j2-isolation-trust]] | [[memd-theory-lock-v1]] |
| O2 | Lifecycle Calibration | `verified` | 21, 22 | [[phase-o2-lifecycle-calibration]] | [[memd-canonical-promotion-theory-lock-v1]] |
| P2 | Measurement Proof | `verified` | 18, 19 | [[phase-p2-measurement-proof]] | [[memd-evaluation-theory-lock-v1]] |

**Verification**:
- Gap details: [[docs/verification/MEMD-10-STAR.md#tier-3]]
- Node criteria: [[docs/verification/NODE-VERIFICATION-MATRIX.md]]
- Benchmark registry: [[docs/verification/benchmark-registry.json]]
- Public benchmarks: [[docs/verification/PUBLIC_BENCHMARKS.md]]

**Execution plan**: [[docs/plans/M3-EXECUTION-PLAN.md]]
**Gate**: All nodes pass M3 tier. LongMemEval ≥ 80%. Token efficiency measured.
Decay calibrated. Benchmark ≥ 90%.
**VERIFIED 2026-04-16**: 593 tests, 0 failures. Node verification 18✓/4~/0✗. Benchmarks zero regression (LME 82.8%, LoCoMo 41.5%, MemBench 34.6%). CI gate all pass. Amnesia checklist 15/15.
P2 fixes: dead code wired (wake token metrics + extract_kind_from_record), CI gate metric names aligned (f1_score→accuracy), LoCoMo threshold float tolerance (0.415→0.414), diagnostics report enhanced (multi-operation token efficiency + --output for wake metrics).

### M4: Make It 10-Star — Tier 4 (10-STAR gaps 24-35)

Product gaps. Dashboard last.

| Phase | Name | Status | Gaps | Phase Doc | Theory Lock |
| --- | --- | --- | --- | --- | --- |
| K2 | Observability | `complete` | 32 | [[phase-k2-observability]] | [[memd-theory-lock-v1]] |
| L2 | Hive Hardening | `complete` | 28, 33, 34, 35 | [[phase-l2-hive-hardening]] | [[memd-hive-theory-lock-v1]] |
| I2 | Human Dashboard | `pending` | 27 | [[phase-i2-human-dashboard]] | — |
| M2-evo | Overnight Evolution | `pending` | 24, 25 | [[phase-m2-overnight-evolution]] | [[memd-theory-lock-v1]] |
| N2 | Integrations Polish | `pending` | 26, 29, 30, 31 | [[phase-n2-integrations-polish]] | [[memd-theory-lock-v1]] |

**M4 progress (2026-04-16)**: `K2` complete on `main` — 10/10 substeps (structured tracing, error classes, `memd explain`, tag search, spine integrity, latency SLA, backup/restore, schema-migration backcompat, `HarnessStatus`, per-response token headers). `L2` complete on `research/mining` — 9/9 substeps (queen deny/reroute/handoff Lamport lock, `WorkingContextSnapshot` in handoff packet, `/hive/divergence`, per-agent write rate limit 100 soft / 200 hard per 60s, 10×100 concurrent-write stress, cross-harness E2E A→B→A with corrections, 0.8 composite handoff-quality gate). Tests at L2 exit: 190 server + 430 client. Handoff: `docs/handoff/2026-04-16-L2-complete-next-I2.md`.

**Open backlog map (pending phases + active blockers)**:
- `I2`: `2026-04-16-no-human-surface-dashboard-ui`, `2026-04-15-dashboard-not-served-from-memd-server`, `2026-04-15-graph-page-crash-entity-search-type-mismatch`, `2026-04-15-memory-entity-record-type-mismatch`, `2026-04-15-dashboard-env-hardcoded-tailscale-ip`, `2026-04-15-memd-preferences-not-persisted-across-sessions`
- `M2-evo`: `2026-04-14-no-overnight-evolution-loop`, `2026-04-14-no-live-memory-contract`, `2026-04-16-working-memory-stale-records`, `2026-04-16-pipeline-lifecycle-broken`
- `N2`: `2026-04-14-skill-gating-config-flags-only`, `2026-04-14-rag-sidecar-disabled-no-fallback`, `2026-04-14-no-data-recovery-procedure`, `2026-04-14-no-admission-control-rate-limiting`, `2026-04-17-memd-process-too-soft-cross-harness`, `2026-04-17-memd-read-state-lost-across-compaction`, `2026-04-17-hooks-scattered-across-three-dirs`
- `B3`: `2026-04-18-longmemeval-intrinsic-primary-score-still-below-target`
- `cross-cutting`: `2026-04-17-codebase-organization-pass` (inter-phase seam, end of A3 or start of B3)

**Verification**:
- Gap details: [[docs/verification/MEMD-10-STAR.md#tier-4]]
- Node criteria: [[docs/verification/NODE-VERIFICATION-MATRIX.md]]
- Feature registry: [[docs/verification/FEATURES.md]]

**Gate**: All nodes pass M4 tier. Private items don't leak. Evolution proposes procedures.
Dashboard: browse, correct, navigate in browser. Zero console errors. Benchmark ≥ 90%.
**Demo**: "Store a fact. Correct it. Navigate it. Prove it. All in the UI."

### V3: Make It Compete — Tier 5 (FINAL memory OS, above and beyond)

V3 ships the **FINAL memory OS**. Not a better v1. Not catch-up. The last version anyone needs. That means **≥0.70 intrinsic on ALL benches without the sidecar** (LongMemEval, LoCoMo, MemBench, ConvoMem) as the **floor** — 70% is where competition sits today, that is bare minimum — and every phase should push **above and beyond** the floor toward a stretch target. Sidecar/RAG is an optional accelerator, not load-bearing. Competitor services (mempalace, supermemory, letta, mem0) out-perform memd today on surfaces benches don't measure (correction UX, atlas navigation, provenance transparency, episodic recall UX, agent handoff quality, hive divergence receipts, dedup explainability) — and they do it without treating RAG as a crutch. Memd won't either.

Reference ceiling: mempalace 96.6% LongMemEval pure-cosine, 100% with rerank ([[.memd/lanes/architecture/A2-09-retrieval-pipeline.md]]). memd 86.0% with sidecar disabled on LME only — LoCoMo 0.415, MemBench 0.346, ConvoMem 0.000 intrinsic ([[docs/backlog/2026-04-14-rag-sidecar-disabled-no-fallback.md]]). Three of four metrics sit below the 70% floor today. The job is to clear the floor on all four without depending on the sidecar.

Every V3 phase is **dual-gated**: measured bench delta AND product-quality win (see each phase doc's `## Product Win` section). Every phase reports **intrinsic (sidecar-off) score** as the primary number, with an accelerated (sidecar-on) column as a secondary delta. Bench without product-win = benchmaxxing. Rag-dependent score without matching intrinsic score = crutch. No merge on any gate alone.

Phase IDs are in execution order (A3 first, J3 last). Reshuffled 2026-04-17 to insert `A3 memd Continuity Foundation` at entry after user directive made memd core-continuity bugs a hard precondition to any retrieval phase. Old A3–E3 shifted to B3–F3. Expanded 2026-04-21 after bench-honesty research revealed adapter-parity gap + non-canonical metrics in F3 — F3 reopened, G3/H3/I3/J3 added.

| Phase | Name | Status | Owns (backlog / target) | Phase Doc |
| --- | --- | --- | --- | --- |
| A3 | memd Continuity Foundation | `complete` | read-state-lost-across-compaction, hooks-scattered, codebase-organization, process-too-soft, pipeline-lifecycle-broken, working-memory-stale-records, preferences-not-persisted, no-live-memory-contract, file-structure-not-enforced-in-code | [[phase-a3-continuity-foundation]] |
| B3 | Intrinsic Retrieval (RAG-Optional) | `complete` | LME 0.86→**≥0.92**, MemBench 0.35→**≥0.70**, LoCoMo 0.42→**≥0.55** (on path to ≥0.70), ConvoMem→≥0.10 | [[phase-b3-activate-retrieval]] |
| C3 | Reranker + Embeddings | `complete` | LME ≥0.95, LoCoMo 0.55→**≥0.70** | [[phase-c3-reranker-embeddings]] |
| D3 | Atlas at Recall | `complete` | LoCoMo ≥0.75, MemBench ≥0.75 | [[phase-d3-atlas-at-recall]] |
| E3 | Consolidation + Sessions | `code_complete_bench_deferred` | LME long-tail +0.03, LoCoMo ≥0.80 | [[phase-e3-consolidation-sessions]] |
| F3 | Bench Honesty | `reopened` | ConvoMem 0→**≥0.70**, MemPalace cross-baseline live — reopened 2026-04-21 after adapter + canonical-metric gaps surfaced; split into G3/H3/I3/J3 | [[phase-f3-bench-honesty]] |
| G3 | Bench Adapter Parity | `complete` | all 4 benches dispatch through `PublicBenchmarkBackend` enum (Lexical/Sidecar/Rrf/Memd); `--retrieval-backend memd` routes LoCoMo/MemBench/ConvoMem through memd retrieval; 4 parity tests + fallback test green; `make bench-public-memd` target live (complete 2026-04-21) | [[phase-g3-bench-adapter-parity]] |
| H3 | Canonical Metrics | `pending` | LongMemEval GPT-4o-judged QA accuracy, LoCoMo token F1, MemBench MC accuracy (MQI deferred), ConvoMem accuracy — the metrics mem0/supermemory/letta/mempalace publish against | [[phase-h3-canonical-metrics]] |
| I3 | Leaderboard Transparency | `pending` | per-row method card (backend/metric/judge/fixture/commit/repro cmd), retraction log for phantom LoCoMo>0.80 + MemBench 0.993, gaming-audit rule (score ≥0.90 requires audit trail) | [[phase-i3-leaderboard-transparency]] |
| J3 | V3 Floor Verification | `pending` | paired intrinsic(sidecar-off)/accelerated(sidecar-on) run on canonical metrics; stranger-reproducible; one run → one verdict → V3 ships on the truth | [[phase-j3-floor-verification]] |

**Roadmap-coverage rule** (user directive 2026-04-17 "every backlog issue should be in the roadmap for a fix"): every backlog item MUST have a `phase:` frontmatter field pointing at the V3 or M4 phase that owns its fix. `docs/backlog/INDEX.md` is regenerated from frontmatter by `make backlog-index`; coverage audit runs in A3 and blocks A3 exit if any item is unassigned.

**Donor anchors**: [[.memd/lanes/architecture/A2-09-retrieval-pipeline.md]] (mempalace pipeline), [[.memd/lanes/architecture/A2-10-embedding-strategy.md]] (model choice), [[.memd/lanes/architecture/A2-11-context-compilation-profile.md]] (priority dedup), [[.memd/lanes/architecture/A2-13-temporal-freshness.md]] (decay calibration), [[docs/theory/2026-04-14-donor-extraction-to-v2-phases.md]] (full mapping).

**Dual-gate format** per phase doc:
- `## Pass Gate` — bench delta: `pre / post / evidence / regression budget`, evidence = regenerated [[docs/verification/PUBLIC_LEADERBOARD.md]]
- `## Product Win` — qualitative UX/product gain: what a dogfooder feels, how it compares to competitor surface, evidence = recorded session trace / sample outputs / comparison note

**V3 completion gate** (measured by J3 — paired intrinsic/accelerated run on canonical metrics per H3, with transparency per I3):
- **Metrics are canonical, not proxy**: LongMemEval = GPT-4o-judged QA accuracy (industry per Mem0/Supermemory), LoCoMo = token F1 (industry per Mem0/MemMachine/Letta), MemBench = MC accuracy (MQI composite deferred pending upstream weights), ConvoMem = accuracy. Retrieval-diagnostic metrics (`hit_rate@5`) ship as secondary columns only. Numbers claimed on proxy metrics cannot satisfy the gate.
- **Bench floor (intrinsic, sidecar OFF) — ≥0.70 on ALL four canonical metrics**: LongMemEval ≥ 0.70, LoCoMo ≥ 0.70, MemBench ≥ 0.70, ConvoMem ≥ 0.70. This is the floor, not the goal — 70% is where competition already sits (Mem0 93.4% LME / 91.6% LoCoMo, Supermemory 81.6-84.6% LME), so it is the bare minimum for a FINAL memory OS. A version that ships with any metric below 0.70 intrinsic is not done.
- **Bench stretch (intrinsic, sidecar OFF) — above and beyond**: LongMemEval ≥ 0.92, LoCoMo ≥ 0.80, MemBench ≥ 0.75, ConvoMem ≥ 0.75. Goal is clear daylight over the 70% floor, not a hairline pass.
- Bench (accelerated, sidecar ON): demonstrable positive delta per metric (≥ +0.02 over intrinsic) or the sidecar is not pulling weight. No metric drops > 0.02 accelerated vs intrinsic. Accelerated numbers are a bonus column, never the gate.
- Product: on 5 dogfood surfaces (wake quality, correction UX, atlas navigation, episode readability, leaderboard verifiability) memd reads as best-in-class — not parity, better — against mempalace/supermemory/letta/mem0 to a stranger who didn't build it. Stranger test is run with sidecar OFF.

**Demo**: "Same query, before and after — show the score AND hand the user the memory surface. They should want to use it."

## Benchmarks

Continuous gate at every milestone. Regression = stop.
Protocol + cadence: [[docs/verification/PUBLIC_BENCHMARKS.md]]

## Mining

Donor-to-phase mapping: [[docs/theory/2026-04-14-donor-extraction-to-v2-phases.md]]

## Backlog

84 items tracked. Full index: `docs/backlog/` directory.
Summary: [[docs/verification/MEMD-10-STAR.md#complete-gap-inventory]]

## Reference Docs

- [[docs/core/setup.md|Setup and harness behavior]]
- [[docs/verification/milestones/MILESTONE-v1.md|Milestone v1 verification]]
- [[docs/strategy/research-loops.md|Research loops]]
- [[docs/theory/models/2026-04-11-memd-ralph-roadmap.md|Detailed Ralph roadmap spec]]
- [[docs/theory/models/2026-04-11-memd-canonical-theory-synthesis.md|Canonical theory synthesis]]

## Non-Goals

- transcript dumping
- vendor lock-in
- using RAG as the only memory layer
- mixing project-local truth with global truth
- letting one provider silently overwrite another provider’s memory
