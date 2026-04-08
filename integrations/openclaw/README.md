# OpenClaw Harness Pack

OpenClaw uses `memd` as the shared memory control plane.

OpenClaw preset:

- pack id: `openclaw`
- entrypoint: `memd context --project <project> --agent openclaw --compact`
- cache policy: compact-first spill cache
- tone: compact context and spill at boundaries

Surface set:

- `.memd/MEMD_WAKEUP.md`
- `.memd/MEMD_MEMORY.md`
- `.memd/agents/OPENCLAW_WAKEUP.md`
- `.memd/agents/OPENCLAW_MEMORY.md`

Default verbs:

- `context`
- `resume`
- `spill`

Shared core:

`memd` owns the same memory control plane, compiled pages, and turn-scoped cache.

Recommended flow:

1. fetch compact context before a task
2. spill at the compaction boundary
3. keep the visible bundle markdown current

Use the OpenClaw-specific entrypoint:

```bash
.memd/agents/openclaw.sh
```

Or use the bundle-aware flow:

```bash
memd context --project my-project --agent openclaw --compact
memd hook spill --output .memd --stdin --apply
```
