---
phase: B4
name: Hook Contract Enforcement
version: v4
status: complete
opened: 2026-04-22
closed: 2026-04-24
depends_on: [A4]
backlog_items: [hooks-scattered, pipeline-lifecycle-broken]
axis: session_continuity
plan_spec: docs/phases/v4/phase-b4-plan.md
contract: docs/contracts/hook-order.md
---

# Phase B4: Hook Contract Enforcement

## Goal

Every memd hook — PreCompact, PostCompact, PreEdit, PreRead, UserPromptSubmit, SessionStart — has a documented fire order, a documented contract, and an enforced guard. Out-of-order fires or silent skips fail loudly, not silently.

## Why this phase exists

V3 A3 Part 2 consolidated hooks under `.memd/hooks`, landed contract v0.2, added write-path hook gate. That set the file layout. But the runtime contract is still soft: a hook can silently fail (network timeout, bad JSON, crashed subprocess) and the session proceeds as if nothing happened. Session-continuity axis stuck low because hook failures corrupt ledger state invisibly.

## Deliver

1. **Fire-order contract.** Canonical order documented in `docs/contracts/hook-order.md` with rationale per hook.
2. **Runtime enforcer.** New `memd hooks enforce` runtime shim that:
   - wraps each hook call
   - asserts fire order (blocks if out-of-order)
   - times out at documented budget
   - logs every hook fire + result to `.memd/logs/hook-trace.ndjson`
3. **Failure modes.** Hook timeout / crash / bad-output → visible to user, not swallowed. Session either halts (for write-path hooks) or logs-and-continues (for observability hooks), per contract.
4. **Integration tests.** 12 tests covering: normal flow, each hook timing out, each hook returning bad JSON, hooks fired in wrong order, hooks fired concurrently (race test).
5. **`memd hooks doctor` red when contract violated.** Existing doctor command extended to verify enforcer output.

## Pass Gate

- pre: hook failures currently silent; no trace; `hooks doctor` green despite real breaches
- post: every hook fire logged; 0 silent swallows in 24h dogfood; `hooks doctor` correctly red on planted fault
- evidence: `.memd/logs/hook-trace.ndjson` from 24h run + fault-injection test output
- regression budget: no more than +50ms hook latency per call; total hook overhead ≤ 200ms per turn

## Product Win

When memd misbehaves, the user sees why. Hook failures surface as "memd skipped X because Y" instead of "memd forgot a thing for no reason." Trust + debuggability.

## Evidence

- `docs/contracts/hook-order.md`
- `.memd/logs/hook-trace.ndjson` from 24h dogfood
- 12 integration tests green in CI
- Fault-injection demo video or log

## Fail Conditions

- Latency budget blown (>200ms/turn): profile, likely async the observability hooks.
- Concurrent-race tests flake: add explicit serialization, document it.

## Rollback

Enforcer behind `MEMD_HOOK_ENFORCE=1`. Old behavior preserved. Graduate to default-on after 1 week clean.

---

## Post-close addendum (2026-04-24)

B4 landed across commits `10bca6b..e669558` and closed clean. Advisor
review after close identified two gaps that shipped in `43f3c8b`:

- `FireOrderValidator` was never called from `run_hook_enforce`, so
  "runtime blocks out-of-order" was documented but unenforced.
  `validate_fire_order()` now replays the trace per-session and halts
  on `OrderSwap` (PostCompact before PreCompact). `MissingPredecessor`
  is scoped to `hooks doctor --check contract` — see
  `docs/contracts/hook-order.md §2` → "Runtime vs doctor scoping".
- Exit code `4` (lock contended) added by B4.7 was missing from the
  contract table and flag table; `43f3c8b` added both rows.

Roadmap advanced to `current_phase=C4` in `c751303`.
