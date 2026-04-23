# Handoff Packet — L2 Complete, Next Up: I2

- created: `2026-04-16`
- from: `claude-code@session-7eab5dde`
- to: next session (any harness) picking up M4 execution
- branch: `research/mining`
- last commit: `7ce2b7c memd auto-commit: L2 phase complete — all 9 substeps shipped`
- status: parked to file (no live peer harness in roster; `codex-fresh` / `codex-stale`
  are fixtures — see `docs/backlog/2026-04-16-hive-handoff-accepts-ephemeral-proof-sessions.md`)

## Where we are on M4

Plan: `docs/plans/M4-EXECUTION-PLAN.md`. Five steps. Serial order:

- Step 1 **K2 (Observability)** — done on `main`, 10/10 substeps
- Step 2 **L2 (Hive Hardening)** — DONE on `research/mining`, 9/9 substeps (this session)
- Step 3 **I2 (Human Dashboard)** — NEXT (11 substeps)
- Step 4 **M2-evo (Overnight Evolution)** — blocked on K2 (met)
- Step 5 **N2 (Integrations Polish)** — blocked on I2 + M2-evo

## What L2 shipped (this session)

All nine L2 substeps landed as individual commits, each with green `cargo test -p`:

| # | Commit   | Substep                                                             |
|---|----------|---------------------------------------------------------------------|
| 1 | bcea729  | L2.3a: queen deny hard block at task write path                     |
| 2 | 9f554f9  | L2.3b: queen reroute via `json_set` + receipt summary               |
| 3 | 2f2c2d6  | L2.3c: queen handoff Lamport lock — task ownership at write path    |
| 4 | f721a20  | L2.4: `HiveHandoffPacket` carries `WorkingContextSnapshot`          |
| 5 | 33dcc6b  | L2.5: `GET /hive/divergence` — per-branch decision summary          |
| 6 | cc80d80  | L2.6: per-agent write rate limiting middleware (soft 100, hard 200) |
| 7 | 2e6a175  | L2.7: 10×100 concurrent writes, no SQLITE_BUSY surfaces             |
| 8 | 471779c  | L2.8: cross-harness E2E — A→handoff→B→corrections→A round trip      |
| 9 | 9412496  | L2.9: handoff quality rubric, 0.8 composite acceptance threshold    |

Test baseline at handoff time:
- `memd-server`: 190 passed
- `memd-client`: 430 passed (one timing flake on
  `bootstrap_hook_refuses_cached_wake_without_session_receipt` seen once under full-suite
  load; passes isolated + on re-run — treat as flake, not blocker)

## Invariants to honor going into I2

- Wire stability: all new schema fields landed with
  `#[serde(default, skip_serializing_if = "Option::is_none")]` so old clients keep parsing.
  Keep that pattern.
- Correction chain: never mutate in place; new row with `supersedes=<old_id>`.
- Rate limiter is a single `Mutex<HashMap>` fixed-window bucket — not governor — because
  we wanted zero extra deps; good enough for single-node.
- SQLite: `PRAGMA journal_mode=WAL` + `busy_timeout=5000` is what keeps L2.7 green.
  Don't pull those.
- Ephemeral proof sessions (`codex-fresh`, `session-live-*`, `session-dogfood-*`) are
  recognized server-side only to shorten live-grace. Do NOT auto-handoff to these
  (see backlog entry above). If you need a peer for a real E2E, spawn one.

## Next session — I2 entry point

Read `docs/plans/M4-EXECUTION-PLAN.md:328-448` (Step 3: I2 — Human Dashboard).
Phase doc: `docs/phases/phase-i2-human-dashboard.md`.

Order inside I2 (suggested, dependency-aware):
1. **I2.2 / I2.3** — fix the two `types.ts` mismatches first (graph page currently
   crashes; do this before anything else renders). Lines: `apps/dashboard/app/lib/types.ts:100-108`
   and `:400-402`.
2. **I2.5** — preference persistence round-trip (roadmap blocker). Write the
   round-trip test first, then fix the save path at
   `crates/memd-client/src/runtime/resume/wakeup.rs:44-83`.
3. **I2.4** — drop the hardcoded Tailscale IP from `.env`, fall back to
   `window.location.origin` in `apps/dashboard/app/lib/api.ts`.
4. **I2.1** — serve dashboard from `memd-server` via `tower-http::services::ServeDir`
   at `/dashboard/*`, root redirect `/` → `/dashboard/`.
5. **I2.6, I2.7, I2.9, I2.10** — dashboard UX (graph refactor, static/dynamic split,
   correction UI, honest status scoring).
6. **I2.8** — `memd state --compact` CLI flag.
7. **I2.11** — Playwright E2E gate + mandatory browser test (zero console errors).

Pass-gate checklist is under `### Step 3` pass gate in the plan — don't declare I2
done until every bullet is green, including the 3-click fact-find manual test.

## Open backlog touching this work

- `docs/backlog/2026-04-16-hive-handoff-accepts-ephemeral-proof-sessions.md` — medium
  severity. Fix shape 2 (server `is_ephemeral_proof: bool` wire flag + client refusal
  without `--allow-ephemeral`) is preferred but NOT started.

## Recovery commands (run on session wake)

```
memd wake --output .memd
git log --oneline -10
cargo test -p memd-server --lib 2>&1 | tail -5
cargo test -p memd-client --lib 2>&1 | tail -5
cat docs/handoff/2026-04-16-L2-complete-next-I2.md
```
