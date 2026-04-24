---
date: 2026-04-24
kind: handoff
from: v3-tail-cleanup-session
to: v4-executor
status: ready-to-execute
entry_phase: A4
branch: research/mining (fast-forwarded to main at 3306a74)
---

# Handoff — V4 starts now, execute A4

## TL;DR

V3 closed. K3 resolved at commit `3306a74` (wrong-URL diagnosis; codex-lb at `http://127.0.0.1:2455/v1` routes gpt-5.4 end-to-end). Leaderboard + ROADMAP reflect reality. V4 plan-specs landed 2026-04-22; next agent opens `docs/phases/v4/phase-a4-plan.md` and executes A4 Task A4.1 → A4.9 atomic.

## Repo state at handoff

- Branch: `main` at `3306a74`, `research/mining` at `3306a74` (fast-forwarded, pushed).
- Recent commits: `3306a74 docs(k3): K3 resolved` · `9999747 chore(v4): kickoff prep` · `376946c docs+code(k3): purge gpt-5.4-mini`.
- Working tree: clean.
- Roadmap state: `current_milestone: V4`, `current_phase: A4`, `phase_status: ready_to_execute`, `v3_tail_deferred: []`, `v3_tail_followups: [canonical rerun via codex-lb route]`.
- Build: `cargo check -p memd-client -p memd-server` last green 18.38s on `/tmp/memd-target`.

## V4 mission

Live Loop Repair. Used-as-designed, memd does not lose state, does not drop corrections, does not bloat context. Composite target **1.80 → 3.45**. Seven phases A4..G4; strict order:

```
A4 ──► B4 ──► C4 ──┐
              │    │
              └► D4 ──► E4 ──┐
                 │            │
                 └──► F4 ─────┤
                              │
                              ▼
                              G4
```

## Entry phase — A4 (Read-State Across Compaction)

Plan spec: `docs/phases/v4/phase-a4-plan.md`. Phase doc: `docs/phases/v4/phase-a4-read-state-compaction.md`. 10-STAR axis: `session_continuity` (+1 minimum, 1 → 2).

Nine atomic tasks A4.1..A4.9, TDD inside each (red → green → commit). Key surface:

- **New files**: `docs/contracts/hook-handoff.md`, `.memd/hooks/memd-postcompact-restore.{sh,ps1}`, `crates/memd-core/src/file_ledger/restore.rs`, `crates/memd-client/src/cli/cli_hook_doctor.rs`, `crates/memd-client/src/main_tests/continuity_compaction_tests/mod.rs`.
- **Fixtures**: `crates/memd-client/fixtures/a4/` (scaffolded — `.gitkeep` exists; populate `pre-compact-ledger.json`, `post-compact-expected.json`, 5-file synthetic transcript).
- **Modify**: `crates/memd-core/src/file_ledger.rs` (re-export `restore`), `crates/memd-client/src/cli/args.rs` (`HookMode::Restore` + `--check ordering`), `.memd/hooks/MANIFEST.json` (add PostCompact entries for `claude-code` and `codex` harnesses), `integrations/hooks/` (auto-sync via `scripts/sync-integration-hooks.sh` after MANIFEST).

### Exit criteria (A4)

1. Ledger restore round-trip tests green.
2. PostCompact hook wired + harness-side config regenerated from MANIFEST.
3. Ledger counts into wake budget.
4. `MEMD_A4_LEDGER_SURVIVAL=1` graduated after 7-day clean window.
5. Hook-handoff contract doc committed.
6. 10-STAR session-continuity axis delta staged for G4 regen.

## Phase dependencies (do not violate)

- B4 cannot start until A4 Task A4.6 (handoff contract) commits.
- C4 + D4 parallelize **only after** B4 Task B4.6 (universal trace) lands.
- E4 requires D4 wake compiler. F4 requires C4 Correction kind + judge client.
- G4 runs last, requires everything.

## Shared fixture ownership (`docs/phases/v4/V4-INTEGRATION.md` §2)

| Fixture | Owner | Consumers |
| --- | --- | --- |
| `shared/sessions/session-1.jsonl` | G4 | A4, C4, D4 |
| `shared/preferences/prefs-5.jsonl` | F4 | G4, D4 |
| `shared/transcripts/aligned-10turn.jsonl` | F4 | G4 |
| `shared/transcripts/drift-10turn.jsonl` | F4 | G4 |
| `shared/hook-traces/canonical-trace.ndjson` | B4 | G4, A4 doctor check |

Rule: each `fixtures/<phase>/` holds phase-exclusive only. Promote to `shared/` the moment a second phase references. Compat shim (symlink or `pub use`) for one phase cycle.

## Build + env invariants

- **NFS rule**: `CARGO_TARGET_DIR=/tmp/memd-target` for every build (workspace lives on NFS at `/mnt/storage/projects/memd`).
- **Server**: `MEMD_RATE_LIMIT_DISABLED=1` for dogfood + bench.
- **Judge model**: `gpt-5.4` canonical (no `gpt-5.4-mini` — purged at `376946c`).
- **Bench route (when needed)**: `OPENAI_BASE_URL=http://127.0.0.1:2455/v1 OPENAI_API_KEY=$CODEX_LB_API_KEY` (codex-lb OAuth fanout, flat-rate). NOT the openclaw LiteLLM at `:4000` — that allowlist excludes gpt-5.4.
- **Commit cadence**: atomic per task, TDD per task. One phase = one plan spec = one file.
- **Never hand-edit**: `docs/verification/SUBSTRATE_BENCHMARKS.md`, `PUBLIC_BENCHMARKS.md` (aggregator-regenerated only).

## V3 tail follow-up (does NOT block V4)

Canonical rerun of LongMemEval / LoCoMo / ConvoMem primaries via codex-lb inline env override. Flip `docs/verification/PUBLIC_LEADERBOARD.md` rows from `replay-pending` to `verified` (if ≥0.70) or `recorded-unpinned` (if <0.70). Runnable anytime between V4 phases; separate session.

## Pointers

- Roadmap: `ROADMAP.md`
- Milestone: `docs/verification/milestones/MILESTONE-v4.md`
- V4 integration: `docs/phases/v4/V4-INTEGRATION.md`
- 10-STAR source: `docs/verification/MEMD-10-STAR.md` (current composite ~2.15 → V4 target 3.45)
- Prior handoff: `docs/handoff/2026-04-22-v4-plan-spec-complete-next-execute.md`
- K3 resolution trail: `docs/backlog/v3/2026-04-23-gpt5.4-proxy-route-for-judge.md`
- Two-proxy URL split (memd fact id surfaced via `memd lookup --query "two-proxy"`).

## First actions for next agent

1. `git pull` on main, confirm at `3306a74` or later.
2. `memd wake --output .memd` (voice: caveman-ultra).
3. Open `docs/phases/v4/phase-a4-plan.md`, read executive summary + §1 surface area + Task A4.1.
4. Red: write the failing test for `file_ledger::restore::locate_latest_sealed`. Green. Commit. Move to A4.2.
