# Architecture

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
