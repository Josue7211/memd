# Agent Zero Harness Pack

Agent Zero should use `memd` as the shared memory plane.

This preset comes from the shared harness schema.

- pack id: `agent-zero`
- entrypoint: `memd wake --output .memd --intent current_task`
- cache policy: zero-friction startup cache
- tone: minimal ceremony and fresh-session path

## Surface Set

- `MEMD_WAKEUP.md`
- `MEMD_MEMORY.md`
- `agents/AGENT_ZERO_WAKEUP.md`
- `agents/AGENT_ZERO_MEMORY.md`

## Default Verbs

- `wake`
- `remember`
- `handoff`

## Shared Core

memd owns the same memory control plane, compiled pages, and turn-scoped cache.

This pack is meant to feel frictionless for a fresh session:

1. resume from the bundle before starting work
2. read the generated wake and memory files for the current lane
3. write durable outcomes back with `memd remember`
4. emit a handoff when another client needs to take over

Use the Agent Zero-specific entrypoint:

```bash
.memd/agents/agent-zero.sh
```

If you are using a bundle, read:

- `.memd/MEMD_WAKEUP.md`
- `.memd/MEMD_MEMORY.md`
- `.memd/agents/AGENT_ZERO_WAKEUP.md`
- `.memd/agents/AGENT_ZERO_MEMORY.md`

## Shared Memory Loop

```bash
memd resume --output .memd
memd remember --output .memd --kind decision --content "Keep the zero-friction lane current."
memd handoff --output .memd --prompt
```

## Shell Hook Example

```bash
MEMD_PROJECT=my-project \
MEMD_AGENT=agent-zero \
./integrations/hooks/memd-context.sh
```
