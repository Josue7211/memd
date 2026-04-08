# OpenCode Harness Pack

OpenCode should use `memd` as the shared continuity plane.

This preset comes from the shared harness schema.

- pack id: `opencode`
- entrypoint: `memd resume --output .memd --intent current_task`
- cache policy: shared-lane continuity cache
- tone: explicit continuity verbs

## Surface Set

- `MEMD_WAKEUP.md`
- `MEMD_MEMORY.md`
- `agents/OPENCODE_WAKEUP.md`
- `agents/OPENCODE_MEMORY.md`

## Default Verbs

- `resume`
- `remember`
- `handoff`

## Shared Core

memd owns the same memory control plane, compiled pages, and turn-scoped cache.

This pack keeps the visible bundle local-first and makes the continuity path
explicit:

1. resume from the bundle before starting work
2. read the generated wake and memory files for the shared lane
3. write durable outcomes back with `memd remember`
4. emit a handoff when another client needs to take over

Use the OpenCode-specific entrypoint:

```bash
.memd/agents/opencode.sh
```

If you are using a bundle, read:

- `.memd/MEMD_WAKEUP.md`
- `.memd/MEMD_MEMORY.md`
- `.memd/agents/OPENCODE_WAKEUP.md`
- `.memd/agents/OPENCODE_MEMORY.md`

## Shared Memory Loop

```bash
memd resume --output .memd
memd remember --output .memd --kind decision --content "Keep the shared lane current."
memd handoff --output .memd --prompt
```

## Shell Hook Example

```bash
MEMD_PROJECT=my-project \
MEMD_AGENT=opencode \
./integrations/hooks/memd-context.sh
```
