# Phase 25 Context: `v4` Agent-Safe Memory Surface Names

## Why This Phase Exists

`memd` was writing a generic `MEMORY.md` at the bundle root. That works for
clients like Codex that do not already own a native memory filename, but it can
collide with agents such as Claude Code that already reserve `MEMORY.md` for
their own memory conventions.

This phase makes the shared memd root memory surface explicitly memd-owned.

## Inputs

- existing bundle root memory write path
- agent-specific memory files under `.memd/agents/`
- requirement to avoid collisions with agent-native memory files

## Constraints

- preserve the agent-specific memory copies
- keep the shared root file inspectable and stable
- update integration docs to point at the non-colliding file

## Target Outcome

The bundle root shared memory file should be `MEMD_MEMORY.md`, not `MEMORY.md`.
