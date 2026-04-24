---
date: 2026-04-24
kind: handoff
from: v4-b4-executor
to: v4-c4-executor
status: ready-to-execute
entry_phase: C4
branch: research/mining (ahead of main by 22 commits)
---

# Handoff — B4 closed, execute C4

## TL;DR

B4 (Hook Contract Enforcement) shipped clean. Every memd hook now runs under a contract-gated wrapper — `memd hooks enforce` — with a normative event-token vocabulary, per-event budget timer, per-(session, event) advisory lock, and append-only NDJSON trace. `memd hooks doctor --check contract` audits the trace for timeouts, silent swallows, and manifest gaps. 10-STAR axes 1 (session_continuity) and 7 (trust+provenance) moved 2 → 3; composite 2.00 → 2.30. Next agent opens `docs/phases/v4/phase-c4-plan.md` (or the C4 phase doc if the plan file isn't there yet) and executes C4 atomic.

## Repo state at handoff

- Branch: `research/mining` at `e669558`, 22 commits ahead of `main` (`3306a74`). Not pushed. Not merged.
- B4 commit range: `10bca6b..e669558` (12 atomic commits including phase-close).
- Working tree: clean.
- Roadmap: `current_phase=C4`, `phase_status=ready_to_execute` (update before starting).

## B4 commits (chronological)

```
10bca6b docs(contracts): hook-order canonical contract (B4)
bea863b feat(memd-core/hook_runtime): ndjson trace writer (B4)
3aa13c9 feat(memd-core/hook_runtime): budget-bounded command wrapper (B4)
049afaa feat(memd-core/hook_runtime): fire-order validator + failure classes (B4)
05d3b5d feat(memd-client/hooks): enforce verb (B4)
a0bed28 feat(memd-client/hooks): universal trace emission (B4.6)
5f652dc feat(memd-core/hook_runtime): per-event per-session serialization (B4.7)
2d65910 feat(memd-client/hooks): doctor --check contract (B4.8)
aa6eee3 feat(hooks): scripts route through enforce wrapper behind flag (B4.9)
6c32977 docs(handoff): B4.10 deferred flag flip — schedule MEMD_HOOK_ENFORCE=1
08143d4 docs(10-star): axes 1+7 rescored after B4 enforcer green (B4.11)
e669558 docs(handoff): B4 closed — next agent executes C4
```

## What is proven

- Normative contract: `docs/contracts/hook-order.md` — event tokens, budgets, failure classes, exit codes, feature flags.
- Runtime: `memd hooks enforce --event <Event> --session-id <id> --output <bundle> -- <inner>` wraps any command behind `MEMD_HOOK_ENFORCE=1`. Unknown event tokens exit 3. Halt-class timeouts exit 2, halt-class inner failures exit 1, lock contention exits 4. Log-class defaults never propagate inner failure.
- Trace: `<bundle>/logs/hook-trace.ndjson` — one line per fire with ts_ms, ULID trace_id, harness, session_id, event, budget_ms, elapsed_ms, exit_code, failure_class. 100 MiB cap triggers a `truncation-required` beacon.
- Serialization: `<bundle>/state/session-<sid>/hook.<event>.lock` holds the advisory lock. Second concurrent fire waits `MEMD_HOOK_LOCK_WAIT_MS` (default 1 s) then exits 4 with an order-violation trace line.
- Doctor: `memd hooks doctor --check contract` audits trace + MANIFEST.json; surfaces timeouts, silent swallows, parse errors, and required-event coverage gaps.
- Wiring: `.memd/hooks/memd-precompact-save.sh`, `.memd/hooks/memd-postcompact-restore.{sh,ps1}` route through the wrapper behind `MEMD_HOOK_ENFORCE=1`. MANIFEST.json now carries `contract_version: "0.3"`.
- Tests: 14/14 integration scenarios in `crates/memd-client/src/main_tests/hook_contract_tests/mod.rs`. 21/21 unit tests under `memd-core::hook_runtime::`.

## What is deferred

- **B4.10 default flip** — schedule doc at `docs/handoff/2026-04-24-b4-default-on.md`. Flip date 2026-05-01 gated on p99 ≤ 200 ms and zero order-violation / silent-swallow trace lines during dogfood. Flag is `MEMD_HOOK_ENFORCE`, default 0 today. Companion to A4.9 (already deferred).
- **V3 tail** — canonical rerun of LongMemEval / LoCoMo / ConvoMem via codex-lb. Separable, does not block C4.

## Next step — C4 entry

Read `docs/phases/v4/ROADMAP.md` for the C4 scope. C4 depends on B4 only for the trace surface — any new hook C4 introduces must land with its event token in `docs/contracts/hook-order.md` first, budget and failure-class defaults too.

C4 MUST NOT:
- Mutate event tokens already in the contract. Add, don't redefine.
- Add a new `failure_class` value without updating `memd-core::hook_runtime::FailureClass` + the doctor contract check.
- Bypass the wrapper. If a new runtime path fires a hook, it goes through `maybe_emit_hook_trace` or `run_hook_enforce`.

## Invariants (do not regress)

1. Unknown event token → exit 3 (contract-parse) — halt-class, not log.
2. Halt-class timeout → exit 2. Halt-class inner-nonzero → exit 1. Lock contention → exit 4. Log-class inner failure → exit 0 but trace line records the class.
3. `MEMD_HOOK_ENFORCE` default stays `0` until 2026-05-01 gate clears.
4. Trace lines are append-only; rotation is V7's problem. The 100 MiB cap emits `truncation-required` and stops appending further payloads.
5. Per-(session, event) lock is the only serialization mechanism. Do not add a second.
6. A4 invariants (PostCompact exit 0 on no-sealed-ledger, restore idempotency, newest-sealed stem-u64 selection, breach log format, `MEMD_A4_LEDGER_SURVIVAL=0` default until 2026-05-01) still stand.

## How to verify before C4.1

```
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-core hook_runtime::
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd hook_contract_tests
CARGO_TARGET_DIR=/tmp/memd-target cargo test -p memd-client --bin memd continuity_compaction_tests::
scripts/verify/a4-loop.sh 10
```

All four must be green. If any is red, do NOT start C4 — diagnose first.

## 10-STAR snapshot after B4

| Axis | Before | After | Weight | Contribution |
|------|--------|-------|--------|--------------|
| Session continuity | 2/10 | 3/10 | 20% | +0.20 |
| Correction retention | 1/10 | 1/10 | 15% | — |
| Procedural reuse | 1/10 | 1/10 | 15% | — |
| Cross-harness continuity | 2/10 | 2/10 | 15% | — |
| Raw retrieval strength | 4/10 | 4/10 | 15% | — |
| Token efficiency | 2/10 | 2/10 | 10% | — |
| Trust + provenance | 2/10 | 3/10 | 10% | +0.10 |

**Composite: 2.00 → 2.30.**

V4 milestone composite target: 3.45. Remaining lift from C4+D4+E4+F4+G4.
