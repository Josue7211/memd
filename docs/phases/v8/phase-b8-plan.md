---
phase: B8
name: Correction UX
version: v8
kind: implementation-plan
status: ready-to-execute
opened: 2026-04-22
depends_on: [A8, V7]
phase_doc: docs/phases/v8/phase-b8-correction-ux.md
granularity: "one step = ≤1 agent session; TDD; commit per task"
axis: correction_retention, trust_provenance
---

# Phase B8 — Implementation Plan

## 0. Executive summary

Correction modal in atlas UI. Before/after retrieval preview. Inline judge. Inline D7 contradiction flow. Undo button.

## 1. Surface area

### Files to create

| Path | Responsibility |
| --- | --- |
| `apps/memd-atlas/src/correction/CorrectionModal.tsx` | Modal editor. |
| `apps/memd-atlas/src/correction/BeforeAfterPreview.tsx` | Prospective retrieval diff. |
| `apps/memd-atlas/src/correction/JudgeInline.tsx` | Live judge confidence. |
| `apps/memd-atlas/src/correction/ContradictionFlow.tsx` | D7 receipt rendering. |
| `crates/memd-server/src/routes/correction_ui.rs` | HTTP endpoints for propose/preview/judge. |
| `apps/memd-atlas/tests/e2e/correction.spec.ts` | E2E. |

### Files to modify

| Path | Change |
| --- | --- |
| `apps/memd-atlas/src/panel/NodePanel.tsx` | Add "Correct" + "Undo" buttons. |
| Phase doc. |

## 2. Schema changes

None.

HTTP:

```
POST /correction/propose        {prior_canonical_id, new_content}
POST /correction/preview         {prior_canonical_id, new_content}  -> prospective top-5
POST /correction/judge           {prior_canonical_id, new_content}  -> {confidence, rationale}
POST /correction/commit          {proposal_id}
```

## 3. API shape

User clicks "Correct" → modal → types new value → sees preview + judge → clicks commit.

## 4. Test matrix

1. `modal_opens_from_node_panel`
2. `modal_submits_via_propose_endpoint`
3. `preview_runs_retrieval_with_prospective_canonical`
4. `preview_diff_matches_post_commit_retrieval` (round-trip)
5. `judge_inline_emits_confidence_under_3s_p95`
6. `contradiction_flow_renders_on_d7_receipt`
7. `undo_button_calls_v7_rollback`
8. `correction_e2e_flow_1_happy`
9. `correction_e2e_flow_2_contradiction_resolve`
10. `correction_e2e_flow_3_rollback`

## 5. Fixtures

- `apps/memd-atlas/tests/fixtures/correction-scenarios.json` — 10 end-to-end flows.

## 6. Telemetry

Per-correction UI event NDJSON: propose → preview → judge → commit/cancel timing.

## 7. Feature flags

| Var | Default | Effect |
| --- | --- | --- |
| `MEMD_B8_CORRECTION_UX` | `0` | Serve UI routes only when set. |

## 8. Task list

### Task B8.1 — HTTP routes

- [ ] Implement propose/preview/judge/commit.
- [ ] Commit: `feat(server/b8): correction-ui routes (B8)`.

### Task B8.2 — modal

- [ ] Tests 1 + 2 failing.
- [ ] Commit: `feat(apps/b8): correction modal (B8)`.

### Task B8.3 — preview

- [ ] Tests 3 + 4 failing.
- [ ] Commit: `feat(apps/b8): before/after preview (B8)`.

### Task B8.4 — judge inline

- [ ] Test 5 failing.
- [ ] Commit: `feat(apps/b8): judge inline (B8)`.

### Task B8.5 — contradiction flow

- [ ] Test 6 failing.
- [ ] Commit: `feat(apps/b8): contradiction flow (B8)`.

### Task B8.6 — undo

- [ ] Test 7 failing.
- [ ] Commit: `feat(apps/b8): undo button (B8)`.

### Task B8.7 — E2E

- [ ] Tests 8 + 9 + 10 failing.
- [ ] Commit: `test(apps/b8): 3 E2E correction flows (B8)`.

## 9. Bench impact

None direct. Indirect: easier corrections → more captured → B5 lift over time.

## 10. Dependency graph

- Requires: A8, V7 C4/B7/D7/G7.
- Blocks: F8.

## Exit criteria

1. Tests 1–10 green.
2. Judge p95 < 3s.
3. Preview matches post-commit retrieval.
4. Atomic commits.
