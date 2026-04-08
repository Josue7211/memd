# Codex Integration

This is the Codex harness pack rendered from the shared preset schema.

- pack id: `codex`
- entrypoint: `memd wake --output .memd --intent current_task --write`
- cache policy: turn-scoped recall/capture cache
- tone: turn-first recall/capture pack

## Surface Set

- `MEMD_WAKEUP.md`
- `MEMD_MEMORY.md`
- `agents/CODEX_WAKEUP.md`
- `agents/CODEX_MEMORY.md`

## Default Verbs

- `wake`
- `resume`
- `checkpoint`

## Shared Core

memd owns the same memory control plane, compiled pages, and turn-scoped cache.

Codex should use the same `memd` surface as every other agent.

Because Codex does not have a built-in durable `memory.md` surface here,
`memd` now maintains bundle-local markdown memory files for it:

- `.memd/MEMD_WAKEUP.md`
- `.memd/MEMD_MEMORY.md`
- `.memd/agents/CODEX_WAKEUP.md`
- `.memd/agents/CODEX_MEMORY.md`

Those files are refreshed by:

- `memd wake --output .memd --intent current_task --write`
- `memd resume --output .memd`
- `memd handoff --output .memd`
- `memd checkpoint --output .memd --content "..."`
- `memd hook capture --output .memd --stdin --summary`

After Codex recall or capture, `memd` refreshes the same local wake/memory files again so the bundle stays in sync without introducing a second source of truth. The turn cache is keyed from project, namespace, agent, mode, and normalized query so repeated reads inside the same turn stay cheap. The generated launcher also stamps a tab ID automatically from the terminal session when one is not already set, so two Codex tabs stay separate without a manual export.

Recommended flow:

1. refresh wake-up at task start
2. use `memd lookup` before answering about prior decisions, preferences, or project history
3. write durable decisions/preferences through the generated helper scripts
4. stream live changes through `memd hook capture`
5. promote durable findings and supersede stale beliefs when corrections land
6. let wake/resume/refresh/handoff write short-term status snapshots automatically
7. use `.memd/agents/watch.sh` for live code-edit capture when working in the repo

If the backend is unavailable, Codex should keep using the local bundle markdown already on disk rather than stalling the turn. Keep using the local bundle markdown until the backend comes back.

If you want a shell-level integration, reuse the shared hook kit in
[`../hooks`](../hooks).

Hermes uses the same shared memory core, but it presents the adoption/onboarding surface instead of the Codex tab-centric workflow.

## Read Context

```bash
memd context --project <project> --agent codex --compact
```

## Read The Bundle Memory File

```bash
cat .memd/MEMD_WAKEUP.md
```

Or use the Codex-specific wake-up copy:

```bash
cat .memd/agents/CODEX_WAKEUP.md
```

Then inspect the deeper compact memory view:

```bash
cat .memd/MEMD_MEMORY.md
```

Or use the Codex-specific copy:

```bash
cat .memd/agents/CODEX_MEMORY.md
```

Use the Codex-specific resume entrypoint:

```bash
.memd/agents/codex.sh
```

## Pre-Answer Lookup

```bash
memd lookup --output .memd --query "what did we already decide about memory recall?"
```

Generated bundle shortcuts:

```bash
.memd/agents/lookup.sh --query "what did we already decide?"
.memd/agents/recall-decisions.sh --query "memory recall"
.memd/agents/recall-preferences.sh --query "design taste"
.memd/agents/recall-design.sh --query "design memory"
.memd/agents/recall-history.sh --query "what happened last session?"
.memd/agents/remember-short.sh --content "Current blocker: correction recall still needs ranking work"
.memd/agents/remember-decision.sh --content "decision: keep lookup-before-answer in the hot path"
.memd/agents/remember-preference.sh --content "preference: keep wake-up files small and query-driven"
.memd/agents/remember-long.sh --content "fact: semantic sync is optional and canonical memory lives in memd first"
.memd/agents/capture-live.sh --content "status: currently debugging correction recall"
.memd/agents/correct-memory.sh --content "corrected fact: backend health does not prove usable memory"
.memd/agents/sync-semantic.sh
```

## Hook Context

```bash
memd hook context --project <project> --agent codex
```

## Shell Hook Example

```bash
MEMD_PROJECT=my-project \
MEMD_AGENT=codex \
./integrations/hooks/memd-context.sh
```

## Search Memory

```bash
cat <<'JSON' | memd search --stdin
{
  "query": "postgres",
  "scopes": ["project", "global"],
  "kinds": ["fact", "topology", "runbook"],
  "statuses": ["active", "stale"],
  "project": "my-project",
  "namespace": "codex",
  "source_agent": "codex",
  "tags": ["infra"],
  "stages": ["canonical"],
  "limit": 10,
  "max_chars_per_item": 240
}
JSON
```

## Verification

```bash
cat <<'JSON' | memd verify --stdin
{
  "id": "uuid-from-search-or-context",
  "confidence": 0.95,
  "status": "active"
}
JSON
```

## Spill Compaction Packet

```bash
cat <<'JSON' | memd hook spill --stdin --apply
{
  "session": {
    "project": "my-project",
    "agent": "codex",
    "task": "fix retrieval routing"
  },
  "goal": "Preserve memory without token waste",
  "hard_constraints": ["compact retrieval only"],
  "active_work": ["verification worker scans stale canonical items"],
  "decisions": [],
  "open_loops": [],
  "exact_refs": [],
  "next_actions": [],
  "do_not_drop": [],
  "memory": {
    "route": "auto",
    "intent": "general",
    "retrieval_order": ["local", "synced", "project", "global"],
    "records": []
  }
}
JSON
```

## Shell Spill Example

```bash
MEMD_BASE_URL=http://100.104.154.24:8787 \
./integrations/hooks/memd-spill.sh --stdin --apply < compaction.json
```
