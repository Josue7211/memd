# Codex Harness Pack Design

## Goal

Ship the first memd harness pack for Codex CLI only.

The pack should make Codex feel native to memd by giving it:

- recall before the turn
- capture after the turn
- turn-scoped cache reuse
- project/container routing
- visible memory handoff files
- compact policy-aware output

This is the first step in the plugin-per-harness strategy extracted from
`supermemory`.

## Why This Exists

`memd` already has the core control plane:

- typed memory
- compiled memory pages
- event compiler lanes
- policy and skill gating
- local-first bundle state

What it does not yet have is a first-class harness pack that makes one agent
surface feel complete instead of merely supported.

Codex is the best first target because:

- the repo already has Codex-specific integration docs
- Codex is a primary day-to-day harness for this project
- the desired behavior is already clear: read once, reuse compiled memory,
  and keep the output compact

## Scope

In scope:

- Codex CLI entry path only
- pre-turn memory recall
- post-turn capture
- turn-scoped cache reuse
- project and namespace routing
- generated wakeup and memory handoff files
- compact status and lookup commands for Codex-specific workflows

Out of scope:

- OpenClaw pack
- Claude pack
- Hermes pack
- OpenAI SDK wrappers
- graph UI redesign
- new semantic backend work

## Recommended Approach

Use a full harness pack, not just a thin wrapper.

That means the Codex pack should include:

1. a setup entrypoint
2. a recall path
3. a capture path
4. a cache boundary
5. generated docs and scripts for the harness

This is better than a generic adapter blob because the harness-specific pack
can own the exact user experience, filenames, and command order Codex needs.

## Components

### 1. Pack Bootstrap

Purpose:
- create or refresh Codex-specific bundle wiring
- write the environment and helper scripts Codex should use

Inputs:
- `MEMD_PROJECT`
- `MEMD_NAMESPACE`
- `MEMD_BASE_URL`
- `MEMD_RAG_URL` when semantic fallback is configured

Outputs:
- Codex-ready helper scripts
- Codex wakeup memory files
- a compact pack manifest or readme for the harness

### 2. Pre-Turn Recall

Purpose:
- load the minimum needed memory before Codex answers
- prefer compiled memory and visible truth objects

Behavior:
- read `MEMD_MEMORY.md` and the compiled memory pages first
- reuse cached turn memory if the same turn key is already loaded
- fall back to semantic retrieval only when configured

### 3. Post-Turn Capture

Purpose:
- persist new durable facts, decisions, and task state after the turn
- keep the memory graph current without rereading raw source

Behavior:
- capture a compact turn summary
- write through the normal `memd` memory/event paths
- refresh compiled memory pages and wakeup files

### 4. Turn Cache

Purpose:
- avoid paying retrieval cost twice in one turn
- keep repeated tool loops cheap

Behavior:
- cache key includes project, namespace, agent, mode, and normalized query
- cached turn payload can be reused by recall, status, and event refresh
- cache is ephemeral and does not replace durable memory

### 5. Visible Hand-off Files

Purpose:
- make Codex memory inspectable on disk
- keep the agent’s working truth visible

Files:
- `.memd/MEMD_WAKEUP.md`
- `.memd/MEMD_MEMORY.md`
- `.memd/agents/CODEX_WAKEUP.md`
- `.memd/agents/CODEX_MEMORY.md`

Behavior:
- wakeup files stay compact
- memory files link into compiled memory pages
- the pack should make it obvious which file to read first

## Data Flow

1. Codex starts a task.
2. The pack resolves the project, namespace, and turn key.
3. Recall checks the turn cache first.
4. If cache misses, the pack reads compiled memory and optional semantic
   fallback.
5. The pack injects compact memory into the Codex working context.
6. Codex works and emits new task state.
7. Capture writes the turn result back through memd.
8. memd refreshes visible memory pages and handoff files.
9. The same turn key remains cacheable until the turn changes.

## Routing Rules

- `MEMD_PROJECT` is the primary project partition.
- `MEMD_NAMESPACE=codex` should scope Codex-specific memory and files.
- `MEMD_AGENT=codex` identifies the active harness.
- `MEMD_BASE_URL` points at the local or shared memd control plane.
- `MEMD_RAG_URL` is optional and only used when semantic fallback is enabled.

## Error Handling

- If recall fails, Codex should still continue with compact local truth.
- If capture fails, Codex should not lose the turn result.
- If the cache is stale or missing, the pack should fall back to the normal
  compiled-memory read path.
- If the backend is unavailable, the pack should keep local bundle truth intact.

## Files and Boundaries

Likely implementation touchpoints:

- `integrations/codex/README.md`
- `integrations/hooks/*`
- `crates/memd-client/src/main.rs`
- `docs/core/setup.md`
- `docs/core/api.md`
- any Codex-specific generated bundle helpers under `.memd/agents/`

The pack should not introduce a second source of truth.
It should only orchestrate the existing memd truth surfaces for the Codex
harness.

## Success Criteria

The Codex pack is successful if:

- a Codex task can start with one obvious command path
- the pack loads compact memory before the turn
- the same turn does not re-fetch the same memory payload twice
- new decisions and state are captured after the turn
- the visible wakeup/memory files stay in sync with the compiled pages
- the pack stays local-first and does not depend on a cloud-only path

## Test Plan

- verify the Codex pack writes the expected helper files
- verify recall uses the cached payload on repeated turn reads
- verify capture updates the bundle-visible memory files
- verify a failed backend does not block local bundle truth
- verify the Codex files still point at the compiled memory pages and not raw
  transcript dumps

## Implementation Order

1. Codex pack bootstrap and file layout
2. pre-turn recall path
3. post-turn capture path
4. turn-scoped cache
5. docs and smoke tests

## Open Question

Should the Codex pack embed the recall/capture logic inside generated helper
scripts only, or should it also expose a reusable Rust-level pack manifest so
other harness packs can reuse the same shape later?
