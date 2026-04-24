# B4 — deferred default-on for `MEMD_HOOK_ENFORCE`

Written: 2026-04-24. Flip target: **2026-05-01** (7-day dogfood window
from B4.9 hook-script commit, 2026-04-24).

## Status

B4 is functionally complete. 14/14 hook-contract integration tests pass
(`hook_contract_tests`, including doctor `--check contract`). The
PreCompact and PostCompact hook scripts now wrap their inner `memd hook`
call with `memd hooks enforce` when `MEMD_HOOK_ENFORCE=1`. The flag
defaults to `0` right now — the wrapper is opt-in for the same reason
A4 was: we want a silent dogfood window before making enforce
unconditional.

## What flips on 2026-05-01

Two-file change plus MANIFEST resha + `scripts/sync-integration-hooks.sh`:

- `.memd/hooks/memd-precompact-save.sh`: change
  ```bash
  if [ "${MEMD_HOOK_ENFORCE:-0}" = "1" ]; then
  ```
  to
  ```bash
  if [ "${MEMD_HOOK_ENFORCE:-1}" != "0" ]; then
  ```
- `.memd/hooks/memd-postcompact-restore.sh`: same-shape inversion on
  the `MEMD_HOOK_ENFORCE` check.
- `.memd/hooks/memd-postcompact-restore.ps1`: mirror the default flip
  (`if ($env:MEMD_HOOK_ENFORCE -eq "1")` → treat unset/empty as
  "enforce on unless explicitly 0").

After editing, recompute sha256 for all three scripts, patch
`MANIFEST.json`, run `scripts/sync-integration-hooks.sh`, and verify
with `memd hooks doctor --project-root .` (must stay green).

## Gate for flipping

Flip iff **all** are true at the end of the 7-day window:

1. `<BUNDLE_ROOT>/logs/hook-trace.ndjson` shows zero
   `failure_class: "order-violation"` lines on the PreCompact /
   PostCompact events across all dogfood sessions.
2. p99 `elapsed_ms` for `{PreCompact, PostCompact, PreRead, PreEdit}`
   ≤ 200 ms. Compute with:
   ```bash
   jq -rc 'select(.failure_class != "timeout") | {event, ms: .elapsed_ms}' \
     "$BUNDLE_ROOT/logs/hook-trace.ndjson" \
     | sort | uniq -c | sort -n | tail
   ```
3. `memd hooks doctor --check contract --project-root .` returns green.
4. Zero `failure_class: "inner-nonzero"` on log-class-default events
   (the contract's silent-swallow class — if any show up, investigate
   before flipping).

If any gate is red: do NOT flip, open a follow-up phase, keep the
flag at `0`.

## Rollback

One-line env override restores pre-flip behavior for any user:

```bash
export MEMD_HOOK_ENFORCE=0
```

No state migration required — the trace file + lock sidecar are both
created lazily and cost nothing when the flag is off.

## Commit shape (for flip day)

```
feat(hooks): default MEMD_HOOK_ENFORCE=1 after dogfood (B4.10)

7-day dogfood window on research/mining closed with p99 ≤ 200 ms
and zero order-violation / silent-swallow trace lines. Hook scripts
now treat MEMD_HOOK_ENFORCE unset/empty as "1".

Gate: docs/handoff/2026-04-24-b4-default-on.md §"Gate for flipping".
Part of docs/phases/v4/phase-b4-plan.md §4.10.
```

## Why 7 days, not 24 h

Same reasoning as A4.9: the hook-trace is sampled per real turn. A
single day only exercises SessionStart/Stop a handful of times on a
developer workstation, which is not enough tail samples to trust p99
against the 200 ms contract. Seven days ≈ 35–50 PreCompact samples
here, which covers the budget-heavy events meaningfully.
