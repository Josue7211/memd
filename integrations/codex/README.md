# Codex Harness Pack

Codex uses `memd` as the shared memory control plane.

Codex preset:

- pack id: `codex`
- entrypoint: `memd wake --output .memd --intent current_task --write`
- cache policy: turn-scoped recall/capture cache
- tone: turn-first recall/capture pack

Surface set:

- `.memd/MEMD_WAKEUP.md`
- `.memd/MEMD_MEMORY.md`
- `.memd/agents/CODEX_WAKEUP.md`
- `.memd/agents/CODEX_MEMORY.md`

Default verbs:

- `wake`
- `resume`
- `checkpoint`

Shared core:

`memd` owns the same memory control plane, compiled pages, and turn-scoped cache.

Recommended flow:

1. wake at task start
2. resume before answering
3. checkpoint or capture after work
4. keep the bundle-local markdown as the hot path fallback

Use the Codex-specific entrypoint:

```bash
.memd/agents/codex.sh
```

For a shell-level integration, reuse the shared hook kit in [`../hooks`](../hooks).
