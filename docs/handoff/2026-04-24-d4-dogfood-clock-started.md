---
opened: 2026-04-24T21:47-04:00
phase: D4
status: dogfood-clock-running
prev_handoff: 2026-04-24-d4-code-complete.md
next_step: collect 7d wake ledger, aggregate histogram, decide flip
day7_earliest: 2026-05-01
---

# D4 dogfood clock started

## Setup landed

- `~/.zshrc` exports `MEMD_D4_COMPILER=1` + `MEMD_WAKE_BUDGET_TOKENS=2000`
- `~/.local/bin/memd` rebuilt from `research/mining` tip (release, 1m29s, 27 MB)
- Smoke wake confirmed ledger writes:
  - `<bundle>/logs/wake-budget.ndjson`
  - `<bundle>/logs/wake-cost.ndjson`
- ROADMAP_STATE patched: `current_phase=D4`, `phase_status=dogfood-clock-running`
- memd checkpoint saved (current-task lane)

## What "done" looks like

After ≥7 calendar days of normal Claude Code / Codex use:

1. Aggregate every `wake-budget.ndjson` line across project + global bundles.
2. Pass criteria (from D4.8 plan):
   - mean `compiled_tokens` < 2000
   - p95 `compiled_tokens` < 2200
   - zero queries that succeeded pre-D4 fail post-D4 on the 20-scenario continuity-loss harness
3. If clear:
   - flip `compiler_enabled()` to default-ON (`crates/memd-client/src/runtime/resume/compiler/mod.rs:183`)
   - drop env-var gate, keep `--raw` escape hatch
   - rescore `token_efficiency` 1 → 4 in `docs/verification/MEMD-10-STAR.md` (closes D4.9)
   - move to V6 public-bench lift

## First-line baseline (single wake captured 21:46)

- `raw_tokens=1096`, `compiled_tokens=1466`, `budget_utilization=0.733`
- Demotions: canonical=4, focus=6 (overflow into "Demoted (use memd lookup)" section)
- Cost estimate: $0.0003665/wake at claude-code family

Note: first wake shows compiled > raw because bucket headers + demotion
section add fixed overhead. Steady-state win shows up as memory grows
past raw budget — that's exactly the case the histogram needs to prove.

## Rollback (if needed mid-clock)

```sh
sed -i '/^# memd D4 dogfood clock/,/^export MEMD_WAKE_BUDGET_TOKENS=2000$/d' ~/.zshrc
```

Or just `unset MEMD_D4_COMPILER` in the shell before invoking memd.

## Parallel work unblocked

E4 (Progressive Depth Recall) does not depend on dogfood pass.
Plan: `docs/phases/v4/phase-e4-plan.md`. Can start any time.
