# A4 — deferred default-on for `MEMD_A4_LEDGER_SURVIVAL`

Written: 2026-04-24. Flip target: **2026-05-01** (7-day dogfood window from
A4.4 hook-script commit `99e176e`, 2026-04-24).

## Status

A4 is functionally complete. All 10 scenarios green 10/10 on
`scripts/verify/a4-loop.sh`. The PostCompact hook scripts honor
`MEMD_A4_LEDGER_SURVIVAL` and no-op when unset / `0`. The flag defaults to
`0` right now because the ordering check cannot yet observe B4's hook-trace
emitter; we want a silent-dogfood window first.

## What flips on 2026-05-01

Single-line change in both hook scripts:

- `.memd/hooks/memd-postcompact-restore.sh`: change
  ```bash
  if \[\[ "${MEMD_A4_LEDGER_SURVIVAL:-0}" != "1" \]\]; then
  ```
  to
  ```bash
  if \[\[ "${MEMD_A4_LEDGER_SURVIVAL:-1}" == "0" \]\]; then
  ```
- `.memd/hooks/memd-postcompact-restore.ps1`: same-shape inversion in the
  PowerShell check.

`scripts/sync-integration-hooks.sh` must be rerun after the flip so the
`integrations/hooks/` mirror is rebuilt; MANIFEST sha256s need to be
regenerated with `memd hook doctor`.

## Gate for flipping

Flip iff **all** are true at the end of the 7-day window:

1. Zero lines in `<BUNDLE_ROOT>/logs/continuity-breach.log` with
   `breach=tool-before-restore` or `breach=missing-restore` across the
   dogfood bundles.
2. `scripts/verify/a4-loop.sh 10` still passes on `main`.
3. B4 hook-trace emitter has NOT regressed the restore contract (run
   `memd hook doctor --check ordering` with real trace).

If any gate is red: do NOT flip, open a follow-up phase, keep the flag at
`0`.

## Rollback

One-line env override restores pre-flip behavior for any user:

```bash
export MEMD_A4_LEDGER_SURVIVAL=0
```

No state migration required — restore is idempotent and the sealed ledger
is written regardless of the flag.

## Commit shape (for flip day)

```
feat(a4): default MEMD_A4_LEDGER_SURVIVAL=1 after dogfood window
```

Single commit; touches both hook scripts + regenerated integrations mirror +
MANIFEST sha256s.

## Why the default is currently 0

Risk: PostCompact fires before the first user prompt of the post-compaction
turn. If the restore CLI ever hangs (e.g. filesystem stall, lock contention),
the turn blocks. Default-0 + hook script always `exit 0` keeps that risk at
zero during dogfood. Once B4 observability shows the CLI finishes in O(10ms)
on every real bundle, we can flip.
