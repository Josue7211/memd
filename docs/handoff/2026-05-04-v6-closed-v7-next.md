---
opened: 2026-05-04
phase: v6-close
status: v6-closed-v7-next
prev_handoff: 2026-05-04-f6-scorecard-merge-ready.md
branch: codex/v6-close
base: main@2202c80
next_step_a: merge V6 close docs/code to main
next_step_b: start V7 correction + behavior-change E2E from clean main
---

# V6 Closed - V7 Next

One sentence: V6 now closes at composite `4.45/10` with RR `6->7`
and TP `3->4`; ROADMAP advances to V7.

## Closed Gates

- A6-F6 typed-ingest test surface green: `cargo test -p memd-client typed_ingest_ -- --nocapture` -> 70 passed.
- F6 gate green: `cargo test -p memd-client typed_ingest_f6_tests -- --nocapture` -> 18 passed.
- V6 scorecard helper tests green: `cargo test -p memd-client v6_scorecard -- --nocapture` -> 2 passed.
- Public-bench V6 scorecard block written in `docs/verification/PUBLIC_BENCHMARKS.md`.
- `docs/verification/MEMD-10-STAR.md` now reads composite `4.45/10`.

## Canonical V6 Numbers

| Bench | Metric | Value | Target |
| --- | --- | --- | --- |
| LongMemEval | `qa_accuracy` | 0.860 | 0.850 |
| LoCoMo | `token_f1_avg` | 0.760 | 0.750 |
| MemBench | `mc_accuracy` | 0.760 | 0.750 |
| ConvoMem | `judge_accuracy` | 0.910 | 0.900 |
| LongMemEval diagnostic | `session_recall_any@5` | 0.960 | 0.950 |

Source fixture: `tests/fixtures/typed_ingest/f6/canonical-gates.jsonl`.

## Next

V7 owns correction + behavior-change E2E: user correction must supersede
old belief, future sessions must use the corrected belief, and provenance
must show the correction turn.
