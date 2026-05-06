---
opened: 2026-05-06
phase: v15-code-complete
status: handoff-ready
prev_handoff: 2026-05-05-v14-code-complete-dogfood-next.md
branch: main
repo_state: pending commit at packet creation
next_step_a: run V14 real-user telemetry dogfood window for >=30 days with >=3 users
next_step_b: run V15 self-tuning dogfood window for >=60 days with >=3 harness-user pairs
release_note: V15 self-tuning compiler substrate code complete; 8.70 composite remains provisional until dogfood close
---

# V15 Code Complete - Dogfood Next

One sentence: V15 self-tuning compiler is implemented and proof-tested; only
the real-user 60-day tuning window remains before honest final close.

## Pickup

```bash
cd /Volumes/T7/projects/memd
git status --short --branch
sed -n '1,160p' docs/handoff/LATEST.md
```

Expected pickup state after commit: clean `main`, ahead `origin/main`.

## Landed

- `memd_core::self_tuning` pure tuning substrate:
  - `CompilerMode`
  - `QualityGuard`
  - `TuningTelemetryPoint`
  - `TuningProfile`
  - static vs dynamic vs self-tuning budget selection
- V14 telemetry-to-profile runtime:
  - reads quality-bearing `.memd/telemetry/events.ndjson`
  - groups by user hash and harness
  - writes `.memd/compiler/tuning-profiles.json`
- `memd compiler` CLI:
  - `tune`
  - `profiles`
  - `ab-bench`
- `memd configure` keys:
  - `compiler.mode`
  - `compiler.self_tuning.min_samples`
  - `compiler.self_tuning.min_quality_score`
  - `compiler.self_tuning.max_quality_regression`
  - `compiler.self_tuning.max_budget_regression_pct`
- Quality guard rejects insufficient samples, low quality, quality regression, budget regression, and no-savings candidates.

## Verification

- `cargo fmt --check` -> passed.
- `cargo test -p memd-core self_tuning -- --nocapture` -> passed.
- `cargo test -p memd-client self_tuning_v15 -- --nocapture` -> passed.
- `RUN_DATE=2026-05-06 scripts/verify/v15-self-tuning-suite.sh` -> passed.

## Proof Artifacts

- `docs/verification/v15-proof-runs/2026-05-06-self-tuning-suite.ndjson`
- `docs/verification/v15-proof-runs/2026-05-06-self-tuning-suite.md`

## Gate State

- V15 code/substrate: complete.
- Synthetic proof: 3 harness-user pairs accepted.
- Minimum token savings vs V11 dynamic: `27.73%`.
- Minimum quality delta: `+0.02`.
- TE proof marker: `8 -> 9`, composite `8.60 -> 8.70` provisional.
- Remaining blocker: real-user self-tuning window (`>=60 days`, `>=3 harness-user pairs`).
- V14 blocker still open: real-user telemetry window (`>=30 days`, `>=3 users`).
- Do not mark V15 final-closed until wall-clock evidence exists.

## Next

Enable V14 telemetry and V15 self-tuning for at least three harness-user pairs,
let the windows run, then rerun the V14 and V15 suites against real exported
telemetry before final close.
