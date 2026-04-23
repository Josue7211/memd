---
date: 2026-04-22
phase: V4 plan-spec complete
status: ready-to-execute
next_phase: Execute A4 (Task A4.1 first)
---

# V4 Plan Specs Landed. Next: Execute A4.

## Why this handoff exists

Prior handoff (`c862b47`) seeded V4–V10 roadmap + drafted V4 phase docs but stopped short of implementation-grade plans. This task closed that gap. Seven phase plans + one cross-phase integration doc now exist on `research/mining`. A code agent can pick up A4 Task A4.1 without re-deriving architecture.

## What landed today (2026-04-22, 8 atomic commits after `c862b47`)

- `8581e92 docs(v4): phase-a4-plan implementation spec`
- `42d3665 docs(v4): phase-b4-plan implementation spec`
- `04fa689 docs(v4): phase-c4-plan implementation spec`
- `09defbf docs(v4): phase-d4-plan implementation spec`
- `22bd4a8 docs(v4): phase-e4-plan implementation spec`
- `03cdae6 docs(v4): phase-f4-plan implementation spec`
- `7e87767 docs(v4): phase-g4-plan implementation spec`
- `03adacd docs(v4): V4-INTEGRATION cross-phase plan`

Tree clean at `03adacd` after this handoff commit lands.

## What the next agent reads first

1. `docs/phases/v4/V4-INTEGRATION.md` — execution-order rules, shared fixtures, hook-contract diff, full 3-session dogfood script, flag-graduation calendar, commit strategy.
2. `docs/phases/v4/phase-a4-plan.md` — first phase to execute. Task A4.1 has ≤1-session acceptance criteria.
3. `docs/handoff/LATEST.md` (= `c862b47`) — V4–V10 roadmap context.

## V4 execution order (strict)

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

- A4 tasks 1–6 MUST land before B4 starts.
- C4 + D4 parallelize after B4 Task B4.6 (universal trace).
- E4 requires D4 compiler. F4 requires C4 Correction kind.
- G4 is the V4 completion gate — requires all.

Full cross-phase deps + parallelizability in `V4-INTEGRATION.md` §1.

## Per-phase specs — what each covers

Every plan carries the 10-point checklist from the prior handoff: surface area, schema changes, API shape, test matrix, fixtures, telemetry, feature flags, executable task list (≤1 session per step), bench impact, dependency graph.

| Phase | Axis delta target | Task count | Test count |
| --- | --- | --- | --- |
| A4 — Read-State Across Compaction | continuity +1 | 9 | 19 |
| B4 — Hook Contract Enforcement | continuity +1, trust +1 | 11 | 22 |
| C4 — Correction Capture E2E | correction +2 | 9 | 22 |
| D4 — Working-Context Compiler | token_eff +3 | 9 | 20 |
| E4 — Progressive-Depth Recall | token_eff +1, cross_harness +1 | 8 | 16 |
| F4 — Preference Replay + Drift | correction +1 | 8 | 14 |
| G4 — Session-Continuity Proof Harness | gate, composite ≥4.0 | 7 | 12 |

Execution commits are produced during phase execution — **not** by the plan-spec-land task. Each phase's internal task list commits per task (A4 = 9 commits, B4 = 11, etc.).

## Critical cross-phase artifacts introduced

These are the seams the phase plans share. Read `V4-INTEGRATION.md` §9 for the full cross-phase API surface table.

- `docs/contracts/hook-handoff.md` (A4) — consumed by B4 enforcer.
- `docs/contracts/hook-order.md` (B4) — consumed by G4 assertions.
- `docs/contracts/recall-depth.md` (E4) — consumed by G4.
- `docs/contracts/correction-lane.md` (C4) — consumed by F4 + G4.
- `.memd/logs/hook-trace.ndjson` (B4) — the substrate telemetry backbone, written from A4 onward.
- `MemoryKind::Correction` (C4) — new schema variant; D4 compiler has a CorrectionBucket pre-wired as placeholder until C4.1 lands.

## Operational context repeated for completeness

- **Branch:** `research/mining`. V3 K3 close-out items (#26, #29, #30, #31) still pending — parallelizable with V4 execution.
- **Rebuild:** `cargo build --release --target-dir /tmp/memd-target -p memd-client -p memd-server`.
- **memd-server for dogfood:** `MEMD_RATE_LIMIT_DISABLED=1 /tmp/memd-target/release/memd-server …`.
- **codex-lb for LLM-judge (C4 + F4):** `http://127.0.0.1:2455`, `$CODEX_LB_API_KEY`, models `gpt-5.4` / `gpt-5.4`. No `gpt-4o` routes.
- **Grader cache pattern:** reuse `.memd/benchmarks/grader-cache/<phase>/` namespace convention; C4 + F4 share a $5/mo budget pool.
- **tmpfs wipe on reboot:** persist state to `.memd/` or `docs/` before a reboot. `/tmp/memd-target/` vanishes.
- **MemoryKind `#[non_exhaustive]`:** verify before C4.1 — may already be annotated.

## Known landmines the plans already work around

- `docs/HARNESS_BRIDGES.md` inverts reality (backlog `2026-04-22-harness-bridges-report-inverted.md`). Read `~/.claude/settings.json` and `~/.codex/hooks.json` directly when wiring B4's enforce route.
- `PUBLIC_LEADERBOARD.md` auto-overwrite disabled (commit b244a7e). Do not re-introduce.
- `crates/memd-client/fixtures/` does **not** exist today — A4 Task A4.1 onward create it.
- `docs/contracts/` does **not** exist today — A4 Task A4.6 creates it.

## Exit criteria for V4 as a milestone

From `V4-INTEGRATION.md` §11:

1. All seven phase exit criteria met.
2. G4 harness passes 10/10 CI runs over 7 days.
3. `docs/verification/MEMD-10-STAR.md` regenerated by G4 — composite ≥4.0.
4. `docs/verification/milestones/MILESTONE-v4.md` filled in.
5. `ROADMAP.md` V4 → closed, V5 → in progress.
6. No open backlog items tagged `axis: session_continuity` or `axis: correction_retention` at severity `blocker`.
7. Final handoff points at `docs/phases/v5/` (V5 plan-spec phase kicks off after V4 closes).

## Next-agent first action

```
cd /home/josue/Documents/projects/memd
git status                                  # confirm clean
cat docs/phases/v4/V4-INTEGRATION.md        # orientation
cat docs/phases/v4/phase-a4-plan.md         # A4 spec
# Start Task A4.1 — read file_ledger.rs, write first failing unit test.
```

No questions, no re-derivation. Specs are the contract.
