# Hermes Harness Pack

Hermes uses `memd` as the shared memory control plane.

Hermes preset:

- pack id: `hermes`
- entrypoint: `memd wake --output .memd --intent current_task`
- cache policy: onboarding-first startup cache
- tone: adoption-focused harness

Surface set:

- `.memd/MEMD_WAKEUP.md`
- `.memd/MEMD_MEMORY.md`
- `.memd/agents/HERMES_WAKEUP.md`
- `.memd/agents/HERMES_MEMORY.md`

Default verbs:

- `wake`
- `resume`
- `remember`

Shared core:

`memd` owns the same memory control plane, compiled pages, and turn-scoped cache.

Recommended flow:

1. start from the wake surface
2. keep the startup path friendly
3. preserve the same truth pages underneath

Use the Hermes-specific entrypoint:

```bash
.memd/agents/hermes.sh
```
