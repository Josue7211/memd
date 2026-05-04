---
opened: 2026-05-04
phase: v6-close
status: v6-closed-v7-next
prev_handoff: 2026-05-04-f6-scorecard-merge-ready.md
branch: main
upstream: origin/main
v6_close_commit: 68d37cf
repo_state: clean
desktop_state: clean
merge_status: V6 close fast-forwarded to main and pushed; this handoff packet is the latest main commit
next_step_a: start V7 correction + behavior-change E2E from clean main
next_step_b: keep V6 fixture-gate caveat visible; live paid public-bench sweep is not part of this close
---

# V6 Closed - V7 Next

One sentence: V6 closes at `68d37cf` with composite `4.45/10`,
RR `6->7`, and TP `3->4`; this handoff packet is ready on latest `main`.

## Closed Gates

- A6-F6 typed-ingest test surface green: `cargo test -p memd-client typed_ingest_ -- --nocapture` -> 70 passed.
- F6 gate green: `cargo test -p memd-client typed_ingest_f6_tests -- --nocapture` -> 18 passed.
- V6 scorecard helper tests green: `cargo test -p memd-client v6_scorecard -- --nocapture` -> 2 passed.
- Public-bench V6 scorecard block written in `docs/verification/PUBLIC_BENCHMARKS.md`.
- `docs/verification/MEMD-10-STAR.md` now reads composite `4.45/10`.
- T7 and desktop both verified clean at `main == origin/main`.

## Canonical V6 Numbers

| Bench | Metric | Value | Target |
| --- | --- | --- | --- |
| LongMemEval | `qa_accuracy` | 0.860 | 0.850 |
| LoCoMo | `token_f1_avg` | 0.760 | 0.750 |
| MemBench | `mc_accuracy` | 0.760 | 0.750 |
| ConvoMem | `judge_accuracy` | 0.910 | 0.900 |
| LongMemEval diagnostic | `session_recall_any@5` | 0.960 | 0.950 |

Source fixture: `tests/fixtures/typed_ingest/f6/canonical-gates.jsonl`.

## Caveat

V6 close is based on locked F6 canonical-gate fixtures/tests, not a fresh
paid live LLM public-benchmark sweep. Do not describe it as a new live
public-benchmark run unless that sweep is actually executed.

## Next

V7 owns correction + behavior-change E2E: user correction must supersede
old belief, future sessions must use the corrected belief, and provenance
must show the correction turn.
