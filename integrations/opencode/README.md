# OpenCode Harness Pack

OpenCode should use `memd` as the shared continuity plane.

This preset comes from the shared harness schema.

- pack id: `opencode`
- entrypoint: `memd resume --output .memd --intent current_task`
- cache policy: shared-lane continuity cache
- tone: explicit continuity and spill verbs

## Surface Set

- `wake.md`
- `mem.md`
- `events.md`

## Default Verbs

- `resume`
- `remember`
- `handoff`
- `spill`

## Shared Core

memd owns the same memory control plane, compiled pages, and turn-scoped cache.

This pack keeps the visible bundle local-first and makes the continuity path
explicit:

1. resume from the bundle before starting work
2. read the generated wake and memory files for the shared lane
3. ask or run `memd lookup` before claiming unknown important facts
4. save new user-taught facts with `memd teach --output .memd --content "..."`
5. compile strict context with capabilities/access before tool-sensitive work
6. write durable outcomes back with `memd remember`
7. emit a handoff when another client needs to take over
8. spill at compaction boundaries

Use the OpenCode-specific entrypoint:

```bash
.memd/agents/opencode.sh
```

If you are using a bundle, read:

- `.memd/wake.md`
- `.memd/mem.md`
- `.memd/events.md`

## Shared Memory Loop

```bash
memd resume --output .memd
memd context --agent opencode --intent current_task --format prompt --include-capabilities --include-access --safety strict
memd remember --output .memd --kind decision --content "Keep the shared lane current."
memd handoff --output .memd --prompt
memd hook spill --output .memd --stdin --apply
```

## Shell Hook Example

```bash
MEMD_PROJECT=my-project \
MEMD_AGENT=opencode \
./integrations/hooks/memd-context.sh
```
