---
phase: F8
name: Public Leaderboard Transparency + V8 Gate
version: v8
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [A8, B8, C8, D8, E8]
phase_doc: docs/phases/v8/phase-f8-public-leaderboard-transparency.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: trust_provenance, V8 completion gate
---

# Phase F8 — Implementation Plan

## 0. Executive summary

Transparency page rendered from PUBLIC_BENCHMARKS.md + method cards; retraction log; gaming-audit rule; stranger-test harness; V8 audit regen; 10-STAR composite ≥ 8.5; MILESTONE close.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `apps/memd-atlas/src/leaderboard/LeaderboardRoute.tsx` | /leaderboard page. |
| `apps/memd-atlas/src/leaderboard/MethodCardView.tsx` | MDX render of method card. |
| `apps/memd-atlas/src/leaderboard/RetractionLog.tsx` | List of retractions. |
| `apps/memd-atlas/src/leaderboard/GamingAuditBanner.tsx` | Rule-card banner. |
| `crates/memd-client/src/benchmark/v8_aggregator.rs` | V8 audit regen. |
| `docs/verification/V8_UI_AUDIT.md` | Regenerated audit. |
| `scripts/stranger-test-harness.md` | Instructions + recording template. |
| `apps/memd-atlas/tests/e2e/leaderboard.spec.ts` | E2E. |

### Files to modify

| Path | Change |
| --- | --- |
| `docs/verification/MEMD-10-STAR.md` | Regenerated. |
| `docs/verification/milestones/MILESTONE-v8.md` | Filled. |
| `ROADMAP.md` | V8 → closed, V9 → in progress. |
| Phase doc. |

## 2. Schema changes

None.

## 3. API shape

```
memd bench v8 --aggregate [--regenerate-audit] [--regenerate-10star]
```

HTTP:

```
GET /leaderboard/benchmarks       -> rendered PUBLIC_BENCHMARKS.md
GET /leaderboard/method-cards     -> listing
GET /leaderboard/retractions      -> log
```

## 4. Test matrix

1. `leaderboard_page_renders_from_markdown`
2. `method_card_mdx_render_round_trips`
3. `retraction_log_sorted_by_date_desc`
4. `gaming_audit_banner_always_visible`
5. `page_auto_refreshes_on_source_commit`
6. `v8_aggregator_rolls_up_ui_pass_rates`
7. `v8_aggregator_rolls_up_perf_numbers`
8. `star_regen_refuses_composite_below_8_5`
9. `star_regen_composite_accepts_at_or_above_8_5`
10. `stranger_test_artifact_schema_valid`
11. `e2e_smoke_green`

## 5. Fixtures

- `apps/memd-atlas/tests/fixtures/leaderboard-snapshot.md` — sample markdown.
- `docs/verification/v8-runs/stranger-test/<date>/` — live stranger-test artifacts.

## 6. Telemetry

V8 audit NDJSON + stranger-test artifacts.

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_V8_ALLOW_BELOW_TARGET` | `0` | Gate writer refuses <8.5 unless set. |

## 8. Task list

### Task F8.1 — transparency page

- [ ] Tests 1 + 2 + 3 + 4 failing.
- [ ] Commit: `feat(apps/f8): leaderboard transparency page (F8)`.

### Task F8.2 — auto-refresh

- [ ] Test 5 failing.
- [ ] Commit: `feat(apps/f8): auto-refresh on source commit (F8)`.

### Task F8.3 — V8 aggregator

- [ ] Tests 6 + 7 failing.
- [ ] Commit: `feat(bench/f8): V8 aggregator (F8)`.

### Task F8.4 — 10-STAR regen

- [ ] Tests 8 + 9 failing.
- [ ] Commit: `feat(bench/f8): V8 10-STAR writer (F8)`.

### Task F8.5 — stranger-test harness

- [ ] Test 10 failing.
- [ ] Write `scripts/stranger-test-harness.md` + recording template.
- [ ] Commit: `docs+feat(f8): stranger-test harness (F8)`.

### Task F8.6 — run stranger test

- [ ] Recruit outside reviewer; sidecar OFF; 5 screencasts + write-up.
- [ ] Commit: `bench(f8): stranger-test artifacts (F8)`.

### Task F8.7 — E2E + CI

- [ ] Test 11 failing.
- [ ] Commit: `ci(apps/f8): leaderboard smoke (F8)`.

### Task F8.8 — milestone close

- [ ] Composite ≥ 8.5; fill MILESTONE-v8; flip ROADMAP; open V9.
- [ ] Commit: `docs(milestone): V8 closed, composite ≥8.5 (F8)`.

## 9. Bench impact

V8 close. 10-STAR lifts to ≥ 8.5.

## 10. Dependency graph

- Requires: A8–E8 closed.
- Blocks: V9.

## Exit criteria (V8 milestone)

1. Tests 1–11 green.
2. Stranger test rates best-in-class on 5 surfaces.
3. Composite ≥ 8.5.
4. MILESTONE-v8 filled.
5. ROADMAP V8 closed.
6. Atomic commits.
