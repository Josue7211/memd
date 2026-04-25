---
opened: 2026-04-24
phase: D4
status: code-complete-dogfood-deferred
prev_handoff: 2026-04-24-c4-substrate-complete.md
next_step: dogfood gate (D4.8) — needs 7 days of live captures behind MEMD_D4_COMPILER=1
---

# D4 code complete — dogfood gate is the only thing left

## Pickup quickstart (30s read)

- **Branch**: `research/mining` (8 commits ahead of `e7bfdbb`)
- **Tip**: `efa54fe docs(d4): mark code-complete, defer D4.8 dogfood + D4.9 rescore`
- **Verify green**: `CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd` → 552 passed
- **Next executable phase**: **E4 Progressive Depth Recall** — plan at `docs/phases/v4/phase-e4-plan.md`, spec at `docs/phases/v4/phase-e4-progressive-depth-recall.md`. No code-side blocker on D4.8.
- **Only thing blocked**: D4.8 (7-day live dogfood) and D4.9 (10-STAR rescore) — measurement gates, not code.
- **Don't do**: don't flip `compiler_enabled()` default to ON until D4.8 passes. Don't re-run the anonymizer plan (sealed dirs lack typed records — see "Fixture deviation" below).

## What landed this session

| Task | Commit | Status |
|------|--------|--------|
| D4.1 scaffold compiler module + bucket types | `1001127` | landed |
| D4.2 priority rules | `6c30c86` | landed |
| D4.3 cross-bucket dedupe | `cd6e1be` | landed |
| D4.4 budget + demotion + kinds-coverage gate | `8bfda04` | landed |
| D4.5 markdown render | `91422d3` | landed |
| D4.6 CLI wiring (`--raw`, `--budget-tokens`, `--include-bucket`, `--exclude-bucket`) + cost ledger | `fb7362f` | landed |
| D4.7 fixtures + 20-scenario continuity-loss harness | `69813e7` | landed |
| D4.8 7-day dogfood gate | n/a | **deferred** |
| D4.9 token_efficiency 10-STAR rescore | n/a | **blocked on D4.8** |

## Test totals

- compiler unit tests (`runtime/resume/compiler/tests.rs`): 18/18
- continuity-loss harness (`main_tests/wake_continuity_tests/`): 3/3 (60 dim assertions across 20 fixtures + canary + histogram)
- memd-client full suite: 552/552 (was 549 pre-D4.7)
- workspace-wide: green

## Why D4.8 / D4.9 are deferred

D4.8 is the wake-size histogram + continuity-loss measurement gate. It
needs 7 calendar days of real wake captures with `MEMD_D4_COMPILER=1`
exported to compare pre/post token cost on real session shapes. Cannot
be satisfied in a single agent session — synthetic fixtures already
prove the pure-transform contract; only live data proves real-shape
load.

D4.9 is the 10-STAR `token_efficiency` axis rescore (target 1 → 4)
that consumes the D4.8 histogram as evidence.

## Pickup from here

1. Export `MEMD_D4_COMPILER=1` (and optionally `MEMD_WAKE_BUDGET_TOKENS=2000`) in your shell rc.
2. Use Claude Code / Codex normally for ≥7 days.
3. Inspect `<bundle>/logs/wake-budget.ndjson` and `wake-cost.ndjson` —
   one line per wake with raw_tokens, compiled tokens, fill ratio per
   bucket, demotion counts, USD cost estimate by model family.
4. Aggregate the histogram. Pass criteria (per phase plan):
   - mean compiled wake size < 2000 chars
   - p95 compiled wake size < 2200 chars
   - zero queries that succeeded pre-D4 fail post-D4 on the
     continuity-loss harness
5. If gates clear → flip `compiler_enabled()` to default-on (drop the
   env-var gate, keep `--raw` as escape hatch) and rescore
   `token_efficiency` 1 → 4 in `docs/verification/MEMD-10-STAR.md`.

## Notable design decisions

- **Char-as-token**: `tokens` everywhere in the compiler is char count,
  not BPE. Matches existing `compute_wake_token_metrics`. Plan calls it
  "tokens" for legacy parity; budget unit is chars.
- **Floor wins absolute** (D4.4): per-bucket floors (canonical=4,
  pref=3, focus=1) bypass both class caps and the total cap. Floor
  entries can never be demoted.
- **CLI overrides** (D4.6): `--include-bucket` bypasses class+total
  caps for forced visibility. `--exclude-bucket` drops the bucket
  before priority is even applied.
- **Demotion section is opt-out by emptiness**: if no bucket overflows,
  the trailing `## Demoted (use memd lookup)` section is omitted —
  empty input renders an empty body, not a noise floor.
- **Fixture deviation** (D4.7): plan said "anonymize 20 dogfood
  sessions". Sealed `.memd/state/session-*` dirs hold only
  file-interaction snapshots — not the typed records the compiler
  operates on (those live in `memory.db`). Synthetic
  `CompilerInput` fixtures were authored instead. Real-data validation
  moves entirely to D4.8.
- **Serde on `CompilerInput`**: derived to make fixtures inspectable
  and editable without Rust knowledge.

## Dependent phases

- **E4** Progressive Depth Recall reads the demotion hints to decide
  what `memd lookup` should resurface — D4.5/D4.6 ship the hint format.
- **F4** Preference Drift consumes the preference bucket directly.
- **G4** Continuity Proof reuses the 20-scenario harness as part of
  its cross-harness invariant suite.
- **V6** public-bench lift expects the compiler-on path — gated by
  D4.8 / D4.9.

## Files touched this session

- `crates/memd-client/src/runtime/resume/compiler/{mod,priority,dedupe,budget,render,ledger,buckets,tests}.rs` — pure-transform pipeline + 18 unit tests
- `crates/memd-client/src/cli/args.rs` — 4 new `WakeArgs` fields
- `crates/memd-client/src/bundle/turn_runtime.rs` — wake compiler route + ledger emit
- `crates/memd-client/src/main_tests/wake_continuity_tests/mod.rs` — 3 harness tests
- `crates/memd-client/fixtures/d4/scenarios/01-20*.json` — 20 continuity-loss fixtures
- `ROADMAP.md` — `current_phase=D4`, `phase_status=code-complete-dogfood-deferred`

## Next executable phase

**E4: Progressive Depth Recall** — `docs/phases/v4/phase-e4-progressive-depth-recall.md`. Builds on the demotion-hint format D4.5 ships. No code-side blocker on D4.8.
