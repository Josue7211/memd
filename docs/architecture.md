# Architecture

## Diagram

See the repo diagram source at [docs/architecture.excalidraw](./architecture.excalidraw).
The landing page preview is [docs/architecture-dark.png](./architecture-dark.png).

## Summary

`memd` is a memory control plane, not just a storage layer.

The core idea is:

- harness packs produce live events and compact turn state
- `memd` routes that state by project, session, tab, and intent
- memd compiles visible pages from the live/bundle state
- Obsidian gives the human-readable graph through wikilinks and backlinks
- LightRAG indexes the same compiled truth for semantic recall

The important rule is read-once, reuse-many:

- raw sources are ingested once
- compiled pages become the default review surface
- source evidence stays linked and drillable
- semantic recall should point back to the compiled page, not replace it

## What Lives Where

### Harness Packs

Harness packs are the entry point for live work.

They do the turn-local jobs:

- pull the smallest useful context
- capture turn output and checkpoints
- preserve session and tab scope
- keep repeated reads in the same turn on a cache

Current packs:

- Codex
- OpenClaw
- Hermes is the next planned harness pack.

### Control Plane

`memd` owns the policy layer in front of every backend.

It decides:

- what is hot enough to surface
- what should stay compact
- what should be evicted or rehydrated
- how to rank by freshness, trust, provenance, and scope
- how to handle contested or stale facts

This is the layer that should get better over time.

### Visible Memory

Visible memory is the product surface.

It includes:

- `MEMD_MEMORY.md`
- lane pages
- item pages
- event pages
- skill pages
- pack pages
- Obsidian wikilink navigation

This is where a user should answer:

- what do I know
- what am I doing
- what changed
- what needs attention

### Semantic Backend

LightRAG is the long-term recall backend.

It should:

- index the same truth compiled by memd
- help find related memory when direct lookup is not enough
- stay behind the control plane
- never become the only visible representation of memory

### Provenance and Verification

Mempalace is a good reminder that raw evidence matters.

`memd` should keep:

- source links
- confidence
- freshness
- contradiction state
- verification state
- promotion history

If a fact is important, it needs a path back to the source and a reason for being trusted.

## Layer Model

### Tier 0: Local Working Memory

Purpose:

- per-session scratchpad
- active hypotheses
- current task context

Properties:

- fast
- volatile
- not canonical

### Tier 1: Synced Short-Term State

Purpose:

- active project focus
- recent decisions
- current blockers
- machine and session status

Properties:

- shared across machines
- short TTL
- optimized for current work

### Tier 2: Dreamed Candidate Memory

Purpose:

- compressed repeated signal
- reusable patterns
- candidate facts for promotion

Properties:

- not canonical
- requires policy evaluation

### Tier 3: Canonical Long-Term Memory

Purpose:

- durable project and global knowledge

Split:

- `project`
- `global`

Backends:

- structured metadata in `memd`
- semantic retrieval in LightRAG or another backend
- graph layer later

## Control Plane

`memd` owns:

- routing
- lifecycle
- dedupe
- TTL
- freshness
- supersession
- ranking
- retrieval shaping

LightRAG is the intended long-term semantic backend path; `memd` stays the control plane in front of it.

No external component should write canonical long-term memory directly.

The core binaries are cross-platform. Only deploy helpers like `deploy/systemd/` are Linux-specific.

## Selective Router

Retrieval requests are classified by:

- route
- intent

The router then picks the smallest useful tier order instead of treating every query as a full corpus search.

Examples:

- `current_task` prefers local and synced state first
- `decision`, `runbook`, and `topology` prefer project memory first
- `preference` and `pattern` prefer global memory first

## Memory Inbox

The manager also exposes an inbox for items that need human or policy attention.

This is where:

- candidate memories wait for promotion
- stale canonical memories wait for verification
- contested items wait for resolution
- superseded items wait for cleanup

If the system cannot show you what needs attention, it turns into a black box. That dies fast in practice.

The server also serves a small built-in dashboard at `/` so the inbox, explain view, search, and compact context can be inspected without needing a separate frontend.

## Working Memory Controller

Working memory is a managed buffer, not just the top N search hits.

The controller should report:

- why an item was admitted
- why an item was evicted
- why an item should be rehydrated

The reasons should be policy-visible and deterministic, using factors such as:

- freshness
- source trust
- contradiction or contested state
- recent use
- verification recency

The output should stay compact on the hot path and move the detailed source trail into explain or source-memory drilldown.

## Reversible Compression

`memd` should keep the hot path compact without destroying the evidence behind it.

That means:

- compact summaries stay first
- explain and source drilldown preserve the raw artifact trail
- policy hooks stay visible so future learned retrieval can observe why the system surfaced something

## Retrieval Order

1. local
2. synced short-term
3. project long-term
4. global long-term

Compact summaries should outrank raw documents. Raw documents are fallback evidence, not the default first payload.
