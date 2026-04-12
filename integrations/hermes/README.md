# Hermes Integration

Hermes should use `memd` as the shared memory control plane.

This preset comes from the shared harness schema.

- pack id: `hermes`
- entrypoint: `memd wake --output .memd --intent current_task`
- cache policy: onboarding-first startup cache
- tone: adoption-focused harness

## Surface Set

- `MEMD_WAKEUP.md`
- `MEMD_MEMORY.md`
- `agents/HERMES_WAKEUP.md`
- `agents/HERMES_MEMORY.md`

## Default Verbs

- `wake`
- `resume`
- `remember`
- `spill`

## Shared Core

memd owns the same memory control plane, compiled pages, and turn-scoped cache.

This is the adoption-focused harness pack. It keeps the same core memory loop
as the other packs, but it frames the experience around onboarding, cloud-first
access, and self-host later.

Recommended flow:

1. load the bundle before the task starts
2. read compact wakeup and memory files before answering
3. capture durable outcomes after the turn
4. spill at compaction boundaries when the turn gets dense
5. keep Obsidian and other platform gateways pointed at the same truth

If you are using a bundle, Hermes should read:

- `.memd/MEMD_WAKEUP.md`
- `.memd/MEMD_MEMORY.md`
- `.memd/agents/HERMES_WAKEUP.md`
- `.memd/agents/HERMES_MEMORY.md`

Use the Hermes-specific entrypoint:

```bash
.memd/agents/hermes.sh
```

## Startup Surface

Hermes should present `memd` as the startup surface, not as an afterthought.

That means:

- compact context before the agent speaks
- durable capture after the turn
- spill at compaction boundaries
- visible bundle files on disk
- local-first fallback when the backend is unavailable

## Short-Term Memory

Refresh Hermes' loaded memory surface on the hot path:

```bash
memd wake --output .memd --intent current_task --write
memd checkpoint --output .memd --content "Current blocker: ..."
```

Use semantic fallback only when deeper recall is needed:

```bash
memd resume --output .memd --intent current_task --semantic
```

Before answering memory-dependent questions, use bundle-aware recall:

```bash
memd lookup --output .memd --query "what did we already decide about this?"
```

## Write Path

Durable outcomes should still go through the normal `memd` paths:

```bash
printf 'decision: keep onboarding compact and visible\n' | memd hook capture --output .memd --stdin --promote-kind decision
memd hook spill --output .memd --stdin --apply
```

If Hermes needs a compaction boundary, use the shared spill flow:

```bash
cat <<'JSON' | memd hook spill --stdin --apply
{
  "session": {
    "project": "my-project",
    "agent": "hermes",
    "task": "onboard user into memd"
  },
  "goal": "Keep adoption simple without losing durable memory",
  "hard_constraints": ["compact retrieval only"],
  "active_work": ["preserve the current turn state"],
  "decisions": [],
  "open_loops": [],
  "exact_refs": [],
  "next_actions": [],
  "do_not_drop": [],
  "memory": {
    "route": "auto",
    "intent": "current_task",
    "retrieval_order": ["local", "synced", "project", "global"],
    "records": []
  }
}
JSON
```

## Notes

- Hermes should stay adoption-friendly without becoming cloud-only
- the pack should keep the same visible truth as the other agent packs
- Obsidian and other gateways should read from the same compiled memory
