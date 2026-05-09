---
status: open
severity: high
phase: C5
opened: 2026-04-22
scope: memd-cli, harness-bridge
---

# HARNESS_BRIDGES.md Report Is Inverted vs Real Harness Configs

## Problem

`.memd/agents/HARNESS_BRIDGES.md` (regenerated on every wake) reports claude as `wired: no, portability: adapter-required, missing: hook` and codex as `wired: yes, portability: harness-native, missing: none`. Real state on this workstation is the opposite:

- `~/.claude/settings.json` has five memd hooks wired: `UserPromptSubmit → memd-bootstrap.sh`, `SessionStart → memd-session-context.js`, `PreToolUse` on Edit/Write → `memd-preedit-prime.sh`, `PostToolUse` on Read/Edit/Write → `memd-file-interaction.sh`, `PreCompact → memd-precompact-save.sh`. Claude receives memd wake context on every user prompt and on every session start.
- `~/.codex/hooks.json` registers only promptbook hooks (`session-start.js`, `prompt-count.js`, `stop.js`). No memd invocation anywhere. Codex sessions never boot the memd bundle into context.

## Evidence

Confirmed 2026-04-22 against:

- `/home/josue/.claude/settings.json` (memd hooks present)
- `/home/josue/.codex/hooks.json` (no memd hooks)
- `/home/josue/Documents/projects/secret-broker/.memd/agents/HARNESS_BRIDGES.md` (generated 2026-04-22T04:32:31, claims opposite)

## Impact

- False positive: users trust the report and assume codex has continuous memd memory. It does not. "Codex struggles" reports tie directly to this gap — memd store is shared on disk but codex sessions never read it.
- False negative: claude appears "not wired" when it is deeply wired, causing users to try to re-install bridges that already exist.
- Bridge-status gate decisions made from this report are structurally wrong.

## Root Cause Hypothesis

The `HARNESS_BRIDGES.md` generator either reads a stale `contract.json` snapshot or uses a hard-coded harness table rather than inspecting the actual harness config files (`~/.claude/settings.json`, `~/.codex/hooks.json`). Needs verification against the generator source.

## Fix

1. Point the generator at the real harness config files. For each harness, check whether its config registers the expected memd hook(s) before claiming `wired: yes`.
2. Add an integration test: seed a fake harness config with known wiring, assert the generated matrix row matches.
3. On every wake, include a one-line `[WARN]` diff when generated-vs-previous state flipped to catch regressions fast.

## Related

- Companion bug: `2026-04-22-codex-memd-hooks-missing.md` (the actual install gap the report should have surfaced).
