# Agent Zero Harness Pack

Agent Zero uses `memd` as the shared memory control plane.

Agent Zero preset:

- pack id: `agent-zero`
- entrypoint: `memd wake --output .memd --intent current_task`
- cache policy: zero-friction startup cache
- tone: minimal ceremony and fresh-session path

Surface set:

- `.memd/MEMD_WAKEUP.md`
- `.memd/MEMD_MEMORY.md`
- `.memd/agents/AGENT_ZERO_WAKEUP.md`
- `.memd/agents/AGENT_ZERO_MEMORY.md`

Default verbs:

- `wake`
- `remember`
- `handoff`

Shared core:

`memd` owns the same memory control plane, compiled pages, and turn-scoped cache.

Recommended flow:

1. boot with the smallest possible setup
2. keep the startup path low-friction
3. use the same bundle truth underneath

Use the Agent Zero-specific entrypoint:

```bash
.memd/agents/agent-zero.sh
```
