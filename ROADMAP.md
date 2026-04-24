# memd Roadmap

`ROADMAP.md` is the single roadmap source of truth for this repo.

<!-- ROADMAP_STATE
truth_date: 2026-04-23
version: v4
version_status: in_progress
current_milestone: V4
milestone_status: starting
current_phase: B4
phase_status: ready_to_execute
next_milestone: V5
post_v3_milestones: V4 → V5 → V6 → V7 → V8 → V9 → V10 → V11 → V12 → V13 → V14 → V15 → V16 → V17 → V18 → V19 → V20 (see V4–V20 block below; composite **8.50 at V13** = 0.1.0 release gate per docs/verification/0.1.0-CONTRACT.md; composite **10.00 at V20** = 1.0.0 release gate per docs/verification/1.0.0-CONTRACT.md; V10 production-floor, V13 ships 0.1.0, V14-V20 ceiling push pulls every axis to 10/10, V20 ships 1.0.0; 0.1.0 gate = composite ≥8.0 AND every axis ≥7; 1.0.0 gate = composite =10.00 AND every axis =10 per docs/theory/MEMD-SOTA-THEORY.md; V4 phase docs drafted, V5+ phase docs drafted at milestone-open)
next_step: B4 Task B4.1 — Hook Contract Enforcement. See docs/phases/v4/phase-b4-plan.md. A4 closed 2026-04-24 on research/mining (commits 60c369d..b7edcc5, 10-STAR session_continuity 1→2, composite 1.80→2.00, scripts/verify/a4-loop.sh 10/10). A4.9 default flip deferred to 2026-05-01 per docs/handoff/2026-04-24-a4-default-on.md. V3 K3 canonical-rerun tail still separable (codex-lb at http://127.0.0.1:2455/v1), does NOT block V4.
active_blockers: []
v3_tail_deferred: []
v3_tail_followups: ["canonical rerun: LongMemEval/LoCoMo/ConvoMem via codex-lb route (OPENAI_BASE_URL=http://127.0.0.1:2455/v1 OPENAI_API_KEY=$CODEX_LB_API_KEY)"]
v1_status: frozen_architecture_complete
v2_status: m4_deferred_for_v3
note: V3 active — FINAL memory OS, above and beyond. Floor: ≥0.70 intrinsic on ALL benches (LME/LoCoMo/MemBench/ConvoMem) without sidecar. A3 Continuity Foundation closed 2026-04-17: Part 1 (file-interaction ledger + prime-reads + PreCompact non-blocking + PreEdit prime), Part 2 (hooks consolidation under .memd/hooks, contract v0.2, write-path hook gate, preference replay), Part 3 (file_layout v0.3 guarantee, backlog/phases regroup under v1/v2/v3, LATEST.md symlink fix, MANIFEST.json + `memd hooks doctor` green/red, lifecycle-probe NDJSON log, cross-harness pre-send validator pure function + 4 tests). B3 Part 2 plumbing landed 2026-04-18 (optional RAG fan-out, dense candidate injection, healthz rag state, dual-mode bench rows, turn diagnostics opt-in). 2026-04-20: 500-Q intrinsic product-path rerun on the real dense blend lands `session_recall_any@5 = 0.936` — gate 0.92 passed. The prior 0.828/0.882 numbers were lexical-only fallback because the bench search path left `source_agent=None` and `MemoryVisibility::Private` denied every item; one-line fix at public_benchmark.rs:1770 unblocked dense. V3 phase order: A3 ✓ → B3 Intrinsic Retrieval → C3 Reranker → D3 Atlas → E3 Consolidation → F3 Bench Honesty.
last_handoff: blockers_resolved_2026-04-21
bench_cadence: every_two_phases  # test every TWO phases per user directive 2026-04-21
-->

## Status Snapshot

- truth date: `2026-04-23`
- current version: `v4` (Live Loop Repair — memd used-as-designed does not lose state, does not drop corrections, does not bloat context); v3 exited with K3 resolved 2026-04-23 (wrong-URL diagnosis, codex-lb live at `http://127.0.0.1:2455/v1`); canonical rerun is a separable follow-up, does not block V4
- version status: `in_progress`
- v1 status: `frozen` — architecture complete, operations broken (honest score: 1.8/10)
- v2/M4 status: `deferred` — K2 + L2 done; I2 + M2-evo + N2 paused for V3 (M4 polish ships visibility but not score; V3 ships score)
- current milestone: `V4: Live Loop Repair` (starting 2026-04-23) — lifts session_continuity 1→4, correction_retention 1→4, cross_harness 2→3, token_efficiency 2→4, trust_provenance 2→3; procedural_reuse seed-only 1→2; composite target 1.80 → 3.45
- current phase: `A4: Read-State Across Compaction` (ready_to_execute). V3 closed 2026-04-23 with B3 landed + scale-validated, K3 code+docs flipped to gpt-5.4 (commit 376946c), K3 proxy provisioning tail-deferred (infra task, non-blocking for V4). Prior V3 phase history: J3 closed 2026-04-21 with verdict `proxy-gap-deferred`. G3 closed 2026-04-21 (bench adapter parity — all 4 benches dispatch via `PublicBenchmarkBackend`). H3 closed 2026-04-21 (canonical metrics — GPT-4o judge cache + token-F1 scorer landed; full rerun deferred). I3 closed 2026-04-21 (leaderboard transparency — 8-field method cards, retraction log, gaming-audit rule, `scripts/regen-leaderboard.sh` gate in CI). J3 closed 2026-04-21: one canonical primary landed (MemBench `mc_accuracy=0.417`, recorded-unpinned, 0.70 floor missed) after bug-fix to `parse_membench_choices` unblocked MemBench full-eval; three others stayed `replay-pending` because the openclaw LiteLLM proxy does not route `gpt-4o-*` (blocks both LongMemEval judge and free-form generator). Diagnostic retrieval numbers captured: LongMemEval 0.900, LoCoMo 0.360, ConvoMem 0.950. K3 (new) owns the proxy unblock — see `docs/backlog/v3/2026-04-21-gpt4o-proxy-route-for-judge.md`.
- completed: `M0` (verified), `M1` (verified 2026-04-15, eval 95), `M2` (verified 2026-04-16), `M3` (verified 2026-04-16); partial `M4`: `K2` (complete 2026-04-16), `L2` (complete 2026-04-16); `I2`/`M2-evo`/`N2` deferred
- M1: `verified` — B2+C2+F2 pass gates, remote deployed, eval 95
- M2: `verified` — D2+G2+E2+H2 pass gates, 624 tests, benchmarks zero regression, node verification 15✓/6~/0✗, remote deployed
- M3: `verified` — J2+O2+P2 pass gates, 593 tests, benchmarks zero regression, node verification 18✓/4~/0✗, CI gate all pass, amnesia checklist 15/15
- M4 progress: `K2` complete (10/10 substeps on main, last commit `235d959`); `L2` complete (9/9 substeps on `research/mining`, last commit `7ce2b7c`). Tests at L2 exit: 190 server + 430 client.
- next step: `A4 Task A4.1` — land PostCompact restore contract + hook per `docs/phases/v4/phase-a4-plan.md`. A4 owner deliverables: `docs/contracts/hook-handoff.md`, `.memd/hooks/memd-postcompact-restore.{sh,ps1}`, `crates/memd-core/src/file_ledger/restore.rs`, `crates/memd-client/src/cli/cli_hook_doctor.rs`, CI scenario under `continuity_compaction_tests/`.
- V3 tail-deferred: `K3 Proxy Unblock + Canonical Rerun` — provision `gpt-5.4` on the openclaw LiteLLM proxy (or OpenAI-direct fallback capped at `MEMD_BENCH_JUDGE_BUDGET_USD=50`), then rerun LongMemEval / LoCoMo / ConvoMem canonical primaries. Success gate: flip those three rows from `replay-pending` to `verified` (if ≥0.70) or `recorded-unpinned` (if <0.70). MemBench separately needs a focused look at event-reasoning + role-tracking topics (0.000 / 0.100 per-topic in J3). Does NOT block V4 — V4 is runtime/dogfood work, not bench gates.
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
| H3 | Canonical Metrics | `complete` | LongMemEval GPT-4o-judged QA accuracy, LoCoMo token F1, MemBench MC accuracy (MQI deferred), ConvoMem accuracy — scorers + judge cache + cost ledger landed 2026-04-21 | [[phase-h3-canonical-metrics]] |
| I3 | Leaderboard Transparency | `complete` | per-row method card (8 required fields), retraction log for phantom LoCoMo 0.709 + MemBench 0.993 + LongMemEval 0.936, gaming-audit rule (score ≥0.90 requires audit trail), `scripts/regen-leaderboard.sh --check` gate in CI — landed 2026-04-21 | [[phase-i3-leaderboard-transparency]] |
| J3 | V3 Floor Verification | `complete_proxy_gap_deferred` | canonical-primary run 2026-04-21: MemBench `mc_accuracy=0.417` (recorded-unpinned, floor missed); LongMemEval/LoCoMo/ConvoMem `replay-pending` — blocked on openclaw LiteLLM missing gpt-4o route. Diagnostic retrieval numbers captured. `parse_membench_choices` bug-fix landed, `write_public_benchmark_docs` no longer overwrites leaderboard. | [[phase-j3-floor-verification]] |
| K3 | Proxy Unblock + Canonical Rerun | `pending` | provision gpt-4o on openclaw LiteLLM proxy (or OpenAI-direct fallback with `MEMD_BENCH_JUDGE_BUDGET_USD=50`), rerun LongMemEval/LoCoMo/ConvoMem canonical primaries, flip rows verified/recorded-unpinned per J3 verdict logic. Gate unblocks V3 completion contract. | [[docs/backlog/v3/2026-04-21-gpt4o-proxy-route-for-judge.md]] |

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

### V4–V20: The Path to 10-STAR (0.1.0 at V13, 1.0.0 at V20)

V3 shipped retrieval honesty. Public benches expose the generator reasoning cap, not a memd cap — retrieval diagnostics sit ≥0.95 on every bench. Six of seven 10-STAR axes (session continuity, correction retention, procedural reuse, cross-harness, token efficiency, trust+provenance) haven't moved since 2026-04-14 because V3 didn't touch them. Current composite (zero-generosity regrade, reconciled 2026-04-22): **1.80/10**.

Path forward is substrate-native: V4 fixes memd in real sessions, V5 builds benches that measure what memd is actually for, V6 ports typed ingest to public benches, V7–V10 ship correction E2E + operator surfaces + multi-user + self-improvement (production floor at V10 close — composite 6.40, every axis ≥3). V11–V13 push to SOTA: V11 Compiler SOTA (dynamic per-turn compiler, cross-project continuity, silent correction detection), V12 Interop SOTA (universal harness protocol, curated routine library, cryptographic provenance), V13 Evidence + 0.1.0 Release (bench domination ≥5pp margin, cross-device sync, third-party provenance replay). Every V4–V13 milestone owns a specific 10-STAR axis delta per [[docs/verification/0.1.0-CONTRACT.md]]; total lift targets **composite ≥8.0 AND every axis ≥7** by V13 (0.1.0 release gate).

V14–V20 push to ceiling: V14 Telemetry Foundation (real-user bench substrate), V15 Self-Tuning Compiler (per-user learned compiler), V16 Cross-Device Sync (CRDT, SC 10-close), V17 Cross-User Routine Economy (marketplace + federation at scale), V18 Correction Graph (multi-hop + silent detection ≥0.90 precision), V19 ZK Provenance (replayable correction proofs + compliance audit), V20 Info-Theoretic TE + Bench Ceiling + **1.0.0 Release** (every axis = 10, composite = 10.00, ≥10pp public-bench margin). V14–V20 axis deltas locked in [[docs/verification/1.0.0-AXIS-OWNERSHIP.md]]; release gate in [[docs/verification/1.0.0-CONTRACT.md]]. Theory binds: [[docs/theory/MEMD-SOTA-THEORY.md]] — "best SOTA memory OS for any harness" at 0.1.0, "ceiling-closed memory OS" at 1.0.0.

Status: all phase docs below are `planned`. Milestone audit docs stubbed at `docs/verification/milestones/MILESTONE-v{N}.md`. V4 phase docs are drafted; V5–V10 phase docs drafted at milestone-open to avoid stale content.

#### V4: Live Loop Repair — Axis Lifts: SC 1→4, CR 1→4, CH 2→3, TE 2→4, TP 2→3 (procedural_reuse seed only, 1→2)

Goal: memd used-as-designed in a real claude-code/codex session does not lose state, does not drop corrections, does not bloat context. Fixes 10-STAR gaps 1–9. Composite target: 1.80 → **3.45**. Integration contract: [[docs/phases/v4/V4-INTEGRATION.md]]. Milestone gates: [[docs/verification/milestones/MILESTONE-v4.md]].

| Phase | Name | Status | 10-STAR axes | Phase Doc |
| --- | --- | --- | --- | --- |
| A4 | Read-State Across Compaction (+ schema locks: Lamport, seq-iso, content-hash) | `planned` | session continuity | [[docs/phases/v4/phase-a4-read-state-compaction.md]] |
| B4 | Hook Contract Enforcement | `planned` | session continuity | [[docs/phases/v4/phase-b4-hook-contract.md]] |
| C4 | Correction Capture E2E (+ sampling gate P≥0.85) | `planned` | correction retention, trust+provenance | [[docs/phases/v4/phase-c4-correction-capture-e2e.md]] |
| D4 | Working-Context Compiler (+ kinds-coverage + cost ledger + 4-layer cap) | `planned` | token efficiency | [[docs/phases/v4/phase-d4-working-context-compiler.md]] |
| E4 | Progressive-Depth Recall (+ FTS5+RRF + query sanitization) | `planned` | token efficiency, cross-harness | [[docs/phases/v4/phase-e4-progressive-depth-recall.md]] |
| F4 | Preference Replay + Drift (includes F4.7 procedural-seed, no axis credit) | `planned` | correction retention, procedural_reuse (seed) | [[docs/phases/v4/phase-f4-preference-drift.md]] |
| G4 | Session-Continuity Proof Harness (multi-harness: claude-code → codex → claude-code) | `planned` | cross-harness gate + all V4 axes binding | [[docs/phases/v4/phase-g4-continuity-proof.md]] |

V4 completion gate: on a 3-session **multi-harness** dogfood (claude-code S1 → codex S2 → claude-code S3), state survives compaction, corrections issued in either harness are honored round-trip, wake context ≤2k tokens with zero continuity loss, F4.7 instrumentation reports ≥3 routine candidates observed. Evidence: recorded session trace + G4 harness NDJSON + regenerated 10-STAR scorecard (strict mode, axis scores ≤ MILESTONE-v4 targets).

Zero-code V4 contract add: [[docs/contracts/federated-memory-visibility.md]] closes Gap-25 (live memory contract); enforcement lands in V9.

#### V5: Substrate-Native Benchmark Suite — Axis Lift: PR 2→4, CH 3→4, RR 4→6

Goal: ship memd's own benchmark suite, open-source, reproducible. Public benches measure flat RAG; these measure what memd is actually for. **Procedural routine-detection flips live** (consumes F4.7 instrumentation from V4). Composite target: 3.45 → **4.20**.

| Phase | Name | Status | Measures | Phase Doc |
| --- | --- | --- | --- | --- |
| A5 | CrossSessionRecall | `planned` | recall across session cuts | deferred |
| B5 | CorrectionPropagation | `planned` | fact corrected → next-session retrieval uses new | deferred |
| C5 | CrossHarnessContinuity | `planned` | claude-code → codex handoff, truth conserved | deferred |
| D5 | ProgressiveDepth | `planned` | wake/lookup/resume quality ladder | deferred |
| E5 | ProvenanceIntegrity | `planned` | every retrieved record carries source | deferred |
| F5 | TypedRetrieval | `planned` | right type returned per query shape | deferred |
| G5 | AdversarialNoise | `planned` | canonical beats planted wrong facts | deferred |

V5 completion gate: all 7 bench suites run in CI, numbers in `docs/verification/SUBSTRATE_BENCHMARKS.md`, any memd competitor can run them.

#### V6: Typed Ingest for Public Benches — Axis Lift: RR 6→7, TP 3→4

Goal: memd stops pretending public benches are flat-RAG. Episodic/semantic/canonical/candidate typing applied to bench inputs; working-context compiler trims the prompt; progressive-depth routes re-queries. LME/LoCoMo/MemBench/ConvoMem numbers lift without benchmaxxing. Composite: 4.20 → **4.45**.

| Phase | Name | Status | Phase Doc |
| --- | --- | --- | --- |
| A6 | Episodic Ingest Pipeline | `planned` | deferred |
| B6 | Semantic Distillation | `planned` | deferred |
| C6 | Canonical Promotion | `planned` | deferred |
| D6 | Working-Context Compiler on Bench | `planned` | deferred |
| E6 | Progressive-Depth Routing | `planned` | deferred |
| F6 | Iterative Reasoning Harness | `planned` | deferred |

V6 gate: LME ≥0.85 / LoCoMo ≥0.75 / MemBench ≥0.75 / ConvoMem ≥0.90 canonical, intrinsic. No regression on retrieval diagnostics.

#### V7: Correction + Behavior-Change E2E — Axis Lift: SC 4→5, CR 4→5, TP 4→5

Goal: correction lane lives end-to-end. User says "no, X is Y" — next session uses Y, provenance shows the correction turn, rollback works. Composite: 4.45 → **4.90**.

| Phase | Name | Status |
| --- | --- | --- |
| A7 | Correction Lane Ingestion Verify | `planned` |
| B7 | Correction → Canonical Promotion | `planned` |
| C7 | Next-Session Behavior Change Test | `planned` |
| D7 | Contradiction Detection | `planned` |
| E7 | Provenance Trail on Corrected Records | `planned` |
| F7 | User-Visible "I learned X from Y" Surface | `planned` |
| G7 | Rollback on Bad Correction | `planned` |
| H7 | Atomic-Commit-by-Default (durability primitive, toggleable via `memd configure`) | `planned` |

V7 gate: correction bench in V5 suite shows 100% propagation, rollback test passes. H7: every memd write path atomically commits dirty tracked files in host repo; default ON; `memd configure auto_commit.enabled=false` toggles OFF for rebase/bisect/experiment workflows.

#### V8: Operator Surfaces — Axis Lift: TE 4→5, TP 5→6, stranger-test dogfood

Goal: user can see memd — atlas, corrections, provenance, diff, rollback. Composite: 4.90 → **5.10**.

| Phase | Name | Status |
| --- | --- | --- |
| A8 | Atlas Navigation UI | `planned` |
| B8 | Correction UX | `planned` |
| C8 | Memory Inspector | `planned` |
| D8 | Provenance Browser | `planned` |
| E8 | Diff + Rollback UI | `planned` |
| F8 | Public Leaderboard Transparency Page | `planned` |
| G8 | `memd configure` Settings CLI (canonical settings surface) | `planned` |

V8 gate: stranger test (outside reviewer, sidecar OFF) rates memd best-in-class vs mempalace/supermemory/letta/mem0 on 5 surfaces. G8 ships the `memd configure` CLI as the single canonical entry point for all runtime settings — subcommands `list/get/set/reset`, schema-validated against `.memd/config.json`, TAB-completion in zsh/bash. Exposes V7 H7 atomic-commit toggle plus V8 cost-ledger caps plus V9 visibility defaults plus V11-V13 feature flags. All ad-hoc settings surfaces are either deprecated or delegate to G8.

#### V9: Multi-User / Team — Axis Lift: SC 5→6, CH 4→6 (enforces federated-memory-visibility contract)

Goal: shared-namespace memory, visibility honored by retrieval, merge collisions resolved, team-wide correction propagation. Activates enforcement of [[docs/contracts/federated-memory-visibility.md]] (published in V4). Composite: 5.10 → **5.60**.

| Phase | Name | Status |
| --- | --- | --- |
| A9 | Shared Namespace Semantics | `planned` |
| B9 | Visibility/ACL Honored by Retrieval | `planned` |
| C9 | Merge Collision Governor Live | `planned` |
| D9 | Hive Divergence Receipts | `planned` |
| E9 | Multi-Agent Handoff Quality | `planned` |
| F9 | Team-Wide Correction Propagation | `planned` |

V9 gate: 2-user 3-agent dogfood holds truth across 10 sessions, divergence surfaced, no silent overwrites.

#### V10: Self-Improvement — Axis Lift: SC 6→7, CR 5→6, PR 4→6, RR 7→8 (production floor milestone)

Goal: memd improves itself — overnight consolidation, auto-correction from user behavior, bench regression canary, 10-STAR automated. Composite: 5.60 → **6.40**. **V10 is the production-floor milestone, not the release gate** — V10 close means every axis ≥3 (production floor). 0.1.0 release tag lands at V13 close, not V10.

| Phase | Name | Status |
| --- | --- | --- |
| A10 | Consolidation-as-Dream (overnight pass) | `planned` |
| B10 | Auto-Correction from User Behavior | `planned` |
| C10 | Memory-Driven Agentic Replay | `planned` |
| D10 | Bench-Score Regression Canary | `planned` |
| E10 | Gap-Audit Self-Scoring (10-STAR automated) | `planned` |
| F10 | Continuous-Deployment Memory | `planned` |

V10 gate: composite ≥6.0 AND every axis ≥3, self-improvement loop demonstrated over 30 days without regression, zero unowned 10-STAR gaps. Hands off to V11 Compiler SOTA.

#### V11: Compiler SOTA — Axis Lift: TE 5→7, SC 7→8, CR 6→7

Goal: push the compiler to SOTA baseline. Dynamic per-turn compiler (decides per-turn what kinds of memory at what depth), Shannon-ish baseline (no redundancy; every token pulls weight), $/M tunable cost ledger. Cross-project continuity (project-aware wake, no pollution from other workspaces). Silent correction detection (user rephrases or ignores a prior answer → memd infers correction without explicit UI). Composite: 6.40 → **6.95**. See [[docs/theory/MEMD-SOTA-THEORY.md]] for axis-level SOTA definitions.

| Phase | Name | Status |
| --- | --- | --- |
| A11 | Dynamic per-turn compiler (turn-intent-aware context selection) | `planned` |
| B11 | Shannon-baseline ablation test (every token pulls weight) | `planned` |
| C11 | $/M cost targeting (operator-tunable budget, exposed via `memd configure`) | `planned` |
| D11 | Project-aware wake (cross-project memory lookups with project provenance) | `planned` |
| E11 | Compaction-aware recall (compressed-optimal long-session context) | `planned` |
| F11 | Silent correction detection (contradiction latency ≤1s, user-behavior-inferred) | `planned` |
| G11 | V11 gate harness (TE/SC/CR assertions; strict-mode scorecard regen) | `planned` |

V11 gate: composite ≥6.95, TE=7, SC=8, CR=7, all others ≥ V10 post. Compiler ablation tests pass; project-scoped wake proven on 3+ workspace set; silent correction detection ≥70% precision ≥60% recall over dogfood corpus.

#### V12: Interop SOTA — Axis Lift: CH 6→8, PR 6→8, TP 6→8

Goal: memd speaks every major harness protocol (MCP, ACP, typed-channel custom). Any harness plugs in with <100 LOC shim. Live multi-harness session (user on claude-code AND codex simultaneously, memory syncs atomically). Curated routine library (browse, edit, compose A+B=C, cross-workspace sharing, per-project inheritance). Cryptographic provenance (signed audit entries, tamper-evident, browsable UI). Composite: 6.95 → **7.75**.

| Phase | Name | Status |
| --- | --- | --- |
| A12 | Routine library UI (`memd routines` browse/edit/merge/deprecate) | `planned` |
| B12 | Routine composition (`memd routines compose A B --output C`) | `planned` |
| C12 | Per-project routine inheritance (`.memd/config.json` cascade) | `planned` |
| D12 | Cross-workspace export/import (`memd routines export/import`) | `planned` |
| E12 | MCP protocol shim (memd as MCP memory backend; <50 LOC client shim) | `planned` |
| F12 | ACP integration (if applicable; defer to E12 outcome) | `planned` |
| G12 | Universal-protocol parity bench + live multi-harness atomic sync | `planned` |
| H12 | Signed audit entries (ed25519) at `.memd/state/audit.ndjson` | `planned` |
| I12 | Audit UI (`memd audit browse` + `memd audit explain`) | `planned` |
| J12 | Tamper-evidence external verifier (`memd audit verify --export`) | `planned` |

V12 gate: composite ≥7.75, CH=8 (universal-protocol parity bench passes), PR=8 (routine library curation dogfooded over 14 days), TP=8 (signed audit verified by external viewer). SOTA floor (every axis ≥7) held by V12 close.

#### V13: Evidence + 0.1.0 Release — Axis Lift: RR 8→9, SC 8→9, CR 7→8, PR 8→9, TP 8→9 (**0.1.0 release gate**)

Goal: bench domination (not parity) — beat published SOTA by ≥5pp on LoCoMo, LongMemEval, MemBench, ConvoMem simultaneously. Publish a new harder benchmark. Cross-device sync (desktop ↔ mac ↔ mobile, CRDT merge). Dormant-project recovery (30+ day gap with full focus recall). Behavior-inferred corrections + multi-hop correction chains. Routine auto-composition. Third-party provenance replay (export + independent verification). Composite: 7.75 → **8.50**. **0.1.0 release tag lands at V13 close.**

| Phase | Name | Status |
| --- | --- | --- |
| A13 | Public-bench domination (≥5pp margin on all four benches) | `planned` |
| B13 | Published harder-bench (current systems fail this bench) | `planned` |
| C13 | Cross-device CRDT sync (desktop ↔ mac ↔ mobile) | `planned` |
| D13 | Dormant-project recovery (30-day gap full-recall test) | `planned` |
| E13 | Behavior-inferred + multi-hop correction chains | `planned` |
| F13 | Routine auto-composition (memd suggests A+B=C) | `planned` |
| G13 | Third-party provenance replay (export + independent harness) | `planned` |
| H13 | V13 release harness (regenerates MEMD-10-STAR.md; if any axis regresses, release does not tag) | `planned` |

**0.1.0 release gate (V13 close):**
1. Composite ≥8.0 (V13 target 8.50)
2. Every axis ≥7 (SOTA floor)
3. Zero blocker-severity backlog with 10-STAR axis label
4. Reproducible proof run in `docs/verification/release-0-1-0/`
5. Head-to-head SOTA proof: ≥1 public bench per applicable axis, ≥5pp margin vs published best

**TE zero-margin flag:** TE closes at 7 (floor 7, zero margin). Any TE regression during V13 close blocks 0.1.0 tag. Contingency: roll back V13 other-axis credits, file V13.5 TE-recovery phase, re-run harness, tag only when TE≥7.

### V14–V20: Ceiling Push to 1.0.0 (composite 8.50 → 10.00)

Post-0.1.0 ceiling work. Every axis must reach 10/10. V14 lays the real-user telemetry substrate that V15/V20 need; V16/V17 close session_continuity + procedural_reuse + cross_harness through sync + federation; V18/V19 close correction_retention + trust_provenance via correction graph + ZK proofs; V20 proves info-theoretic TE optimality + bench domination ≥10pp margin and ships **1.0.0**. Axis ownership grid: [[docs/verification/1.0.0-AXIS-OWNERSHIP.md]]. Release contract: [[docs/verification/1.0.0-CONTRACT.md]].

#### V14: Telemetry Foundation — Axis Lift: TE 7→8

Real-user telemetry + bench-regression canary feeding V15/V20 compiler self-tuning. Composite: 8.50 → **8.60**. Milestone: [[docs/verification/milestones/MILESTONE-v14.md]].

| Phase | Name | Status |
| --- | --- | --- |
| A14 | Opt-in telemetry substrate (consent + schema + `.memd/telemetry.jsonl`) | `planned` |
| B14 | Per-turn compile-outcome metrics (tokens-in, tokens-compiled, ablation deltas) | `planned` |
| C14 | Real-user bench adapter (LME-like 30-turn workload from telemetry) | `planned` |
| D14 | Canary harness (nightly regression detection; slack/webhook alert) | `planned` |
| E14 | Privacy proof (PII-free aggregate rollup; user-reviewable) | `planned` |
| F14 | Cohort replay tooling (`memd telemetry replay --cohort N`) | `planned` |
| G14 | V14 gate harness (TE 7→8 assertion; ≥30-day dogfood) | `planned` |

#### V15: Self-Tuning Compiler — Axis Lift: TE 8→9

Per-user learned compiler: reads V14 telemetry, proposes compile-strategy deltas, auto-applies within safety envelope. Composite: 8.60 → **8.70**. Milestone: [[docs/verification/milestones/MILESTONE-v15.md]].

| Phase | Name | Status |
| --- | --- | --- |
| A15 | Compile-strategy delta proposer (per-user profile from V14 data) | `planned` |
| B15 | Safety envelope + rollback (no quality regression beyond 2pp) | `planned` |
| C15 | A/B harness (shadow-compile; reversible) | `planned` |
| D15 | Per-user profile storage + explain (`memd compiler explain`) | `planned` |
| E15 | Cross-user anonymized learnings (opt-in federation feed) | `planned` |
| F15 | V15 gate harness (TE 8→9; ≥90-day dogfood with ≥5 profiles) | `planned` |

#### V16: Cross-Device Sync — Axis Lift: SC 9→10, CH 8→9

CRDT sync across desktop/mac/mobile with offline merge + conflict UX. Closes session_continuity at ceiling. Composite: 8.70 → **9.05**. Milestone: [[docs/verification/milestones/MILESTONE-v16.md]].

| Phase | Name | Status |
| --- | --- | --- |
| A16 | CRDT state schema (Automerge or custom Yjs-equivalent) | `planned` |
| B16 | Sync transport (end-to-end encrypted; libp2p or WireGuard-based) | `planned` |
| C16 | Offline merge (two devices write offline → deterministic merge) | `planned` |
| D16 | Conflict UX (user sees divergence, picks winner, audit trail) | `planned` |
| E16 | Mobile client (iOS + Android read-only first) | `planned` |
| F16 | Sync chaos test (network partitions, concurrent writes, clock drift) | `planned` |
| G16 | V16 gate harness (SC=10, CH=9 assertions; ≥3-device dogfood) | `planned` |

#### V17: Cross-User Routine Economy — Axis Lift: PR 9→10, CH 9→10

Routine marketplace with trust + provenance + per-user reputation. Closes procedural_reuse and cross_harness at ceiling. Composite: 9.05 → **9.35**. Milestone: [[docs/verification/milestones/MILESTONE-v17.md]].

| Phase | Name | Status |
| --- | --- | --- |
| A17 | Routine marketplace schema (content-addressed + author + version) | `planned` |
| B17 | Trust layer (reputation + allowlist/blocklist) | `planned` |
| C17 | Parameterized routine generalization (infer variable bindings from ≥3 traces) | `planned` |
| D17 | Discovery UI (`memd routines marketplace search/browse/install`) | `planned` |
| E17 | Federation scale test (≥1000 users; per-user isolation preserved) | `planned` |
| F17 | Zero-data-leakage proof (adversarial: shared routine strips private citations) | `planned` |
| G17 | V17 gate harness (≥30-day marketplace dogfood; ≥5 cross-user installs) | `planned` |

#### V18: Correction Graph + Silent Detection — Axis Lift: CR 8→9

Multi-hop correction graph + silent detection ≥0.90 precision / ≥0.85 recall. Composite: 9.35 → **9.50**. Milestone: [[docs/verification/milestones/MILESTONE-v18.md]].

| Phase | Name | Status |
| --- | --- | --- |
| A18 | Correction graph data structure (edges: cites/supersedes/affects) | `planned` |
| B18 | Multi-hop propagation engine | `planned` |
| C18 | Silent correction detector v2 (LLM-judged + heuristic ensemble) | `planned` |
| D18 | Downstream-effect surfacing (affected-by chain in query result) | `planned` |
| E18 | Correction-graph export format (deterministic replay input) | `planned` |
| F18 | Third-party replay harness | `planned` |
| G18 | V18 gate harness (≥3-month dogfood; ≥50 multi-hop chains; detector metrics) | `planned` |

#### V19: Zero-Knowledge Provenance — Axis Lift: TP 9→10, CR 9→10

ZK proofs for correction-applied claims + compliance-grade audit UI + multi-party attestation. Closes correction_retention + trust_provenance at ceiling. Composite: 9.50 → **9.75**. Milestone: [[docs/verification/milestones/MILESTONE-v19.md]].

| Phase | Name | Status |
| --- | --- | --- |
| A19 | ZK proof system selection (groth16 / plonk / custom) | `planned` |
| B19 | Circuit implementation for correction-applied claim | `planned` |
| C19 | Standalone verifier (`memd audit verify-zk <proof>`) | `planned` |
| D19 | Multi-party attestation (two-of-three signing for high-stakes corrections) | `planned` |
| E19 | Compliance audit UI (SOC2-lite scenario dogfood) | `planned` |
| F19 | Third-party ZK replay (auditor verifies without seeing content) | `planned` |
| G19 | V19 gate harness (≥10 ZK proofs externally verified; TP=10, CR=10) | `planned` |

#### V20: Info-Theoretic TE + Bench Ceiling + 1.0.0 Release — Axis Lift: RR 9→10, TE 9→10 (**1.0.0 release gate**)

Info-theoretic optimal compiler (no token removable without quality loss) + ≥10pp public-bench margin + memd-authored harder benches + zero-shot domain generalization. Composite: 9.75 → **10.00**. Milestone: [[docs/verification/milestones/MILESTONE-v20.md]].

| Phase | Name | Status |
| --- | --- | --- |
| A20 | Info-theoretic TE prover (removal harness; optimal iff all deltas ≥ threshold) | `planned` |
| B20 | Bench-domination sweep (≥10pp on LoCoMo, LME, MemBench, ConvoMem) | `planned` |
| C20 | memd-published harder benches (SOTA competitors ≥15pp below memd) | `planned` |
| D20 | Zero-shot domain generalization (≤5pp delta vs tuned baseline) | `planned` |
| E20 | 1.0.0 release harness (every-axis=10 aggregate; zero-generosity regenerator) | `planned` |
| F20 | Third-party replay for every axis (external reviewer reproduces all proofs) | `planned` |
| G20 | 1.0.0 release tag + full proof bundle in `docs/verification/release-1-0-0/` | `planned` |

**1.0.0 release gate (V20 close):**
1. Composite = 10.00 exactly; every axis = 10
2. Info-theoretic TE proof: every token removal test fails quality threshold
3. ≥10pp margin on all four public benches simultaneously
4. memd-authored harder bench with SOTA competitors ≥15pp below memd
5. Zero-shot domain test: retrieval quality delta ≤5pp vs tuned
6. Third-party replay reproduces every axis proof from export
7. Reproducible proof bundle at `docs/verification/release-1-0-0/`
8. 1.0.0 tag on main

**V20 zero-margin flag:** Every axis has zero margin at V20 close. Any regression blocks 1.0.0 tag. **V20.5 recovery reserve** is pre-declared: if V20 misses any axis, V20.5 files a recovery phase scoped to that axis before 1.0.0 tags. Recovery phase may not claim new axis credit — it restores the axis to its V20 target.

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
