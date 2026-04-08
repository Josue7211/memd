# OpenCode Harness Pack

OpenCode uses `memd` as the shared memory control plane.

OpenCode preset:

- pack id: `opencode`
- entrypoint: `memd resume --output .memd --intent current_task`
- cache policy: shared-lane continuity cache
- tone: explicit continuity verbs

Surface set:

- `.memd/MEMD_MEMORY.md`
- `.memd/agents/OPENCODE_MEMORY.md`

Default verbs:

- `resume`
- `remember`
- `handoff`

Shared core:

`memd` owns the same memory control plane, compiled pages, and turn-scoped cache.

Recommended flow:

1. resume from the bundle before work starts
2. write durable outcomes back with `memd remember`
3. emit a handoff when another client takes over

Use the OpenCode-specific entrypoint:

```bash
.memd/agents/opencode.sh
```

Shared lane commands:

```bash
memd resume --output .memd
memd remember --output .memd --kind decision --content "Keep the shared lane current."
memd handoff --output .memd --prompt
```
