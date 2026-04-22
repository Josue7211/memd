---
status: open
severity: high
phase: tbd
opened: 2026-04-22
scope: memd-cli, harness-bridge, codex
---

# Codex hooks.json Missing memd SessionStart / UserPromptSubmit Bridges

## Problem

`~/.codex/hooks.json` registers only promptbook hooks. There is no memd invocation at any codex lifecycle event. As a result, when a user launches codex:

- Codex never executes `.memd/agents/codex.sh` or `memd wake` at session start.
- Codex never injects `wake.md` / durable truth / voice_mode into its initial context.
- Codex never runs `memd lookup` before memory-dependent answers (the user-visible "codex struggles" symptom).

The shared memd store on disk is reachable, and `memd` CLI is on PATH, but nothing in codex's event surface triggers a read. Codex has to be manually briefed every session.

## Current codex hooks.json state

```
SessionStart        → node ~/.promptbook/hooks/codex/session-start.js
UserPromptSubmit    → node ~/.promptbook/hooks/codex/prompt-count.js
Stop                → node ~/.promptbook/hooks/codex/stop.js
```

Zero memd entries.

## Expected state (mirror of claude wiring)

```
SessionStart        → bash ~/.codex/hooks/memd-bootstrap.sh   (or ~/.memd/hooks/memd-bootstrap.sh)
UserPromptSubmit    → bash ~/.codex/hooks/memd-bootstrap.sh   (cached bootstrap, same as claude)
Stop / PreCompact   → bash ~/Documents/projects/memd/.memd/hooks/memd-precompact-save.sh
```

(Plus existing promptbook hooks preserved — composition, not replacement.)

## Impact

- Continuous-memory promise violated on codex. User expectation "memd works across all harnesses" is architecturally true (store is shared) but operationally false (codex has no reader).
- Any workflow that relies on voice_mode, durable truth, or prior-session correction replay silently degrades on codex.
- `HARNESS_BRIDGES.md` reports this gap as "wired: yes, missing: none" (see companion bug `2026-04-22-harness-bridges-report-inverted.md`), masking the regression.

## Fix

1. Ship a `memd install --harness codex` subcommand (or repair the existing installer) that merges memd hook entries into `~/.codex/hooks.json` without clobbering promptbook entries.
2. Provide a canonical `~/.memd/hooks/memd-bootstrap.sh` that both claude and codex can point at (see bug 3 — cross-path fragility: claude currently points at `~/.codex/hooks/memd-session-context.js`, which leaks implementation detail across harnesses).
3. Add a `memd doctor --harness codex` check that reads `~/.codex/hooks.json` and reports missing SessionStart/UserPromptSubmit memd entries.
4. Update `HARNESS_BRIDGES.md` to derive `wired` status from the real config files, so this gap surfaces automatically.

## Related

- Companion bug: `2026-04-22-harness-bridges-report-inverted.md` (the wrong status report).
- Cross-path fragility note: claude's `SessionStart` hook currently invokes `$HOME/.codex/hooks/memd-session-context.js`. A path under `~/.codex/` being called by claude means codex uninstall can silently break claude. Consolidate under `~/.memd/hooks/` when fixing.
