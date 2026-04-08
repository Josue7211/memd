# API

## Design Rules

- compact responses by default
- hard limits by default
- scope-aware retrieval order
- typed retrieval routing by route and intent
- no transcript-sized payloads
- semantic backends stay behind the manager

## Endpoints

### `GET /healthz`

Returns:

- service status
- current persisted item count
- candidate and canonical items are both counted

### `GET /`

Serves the built-in dashboard for health, context, inbox, search, and explain.

### `POST /memory/store`

Stores a typed memory item.

Rules:

- empty content is rejected
- caller must provide `kind` and `scope`
- omitted confidence defaults to a moderate value
- writes directly to canonical memory
- items are persisted in SQLite
- optional `belief_branch` keeps competing durable beliefs separate without flattening them into one record
- `kind` now includes explicit `procedural` and `self_model` lanes for runbooks, capabilities, and failure-mode memory

### `POST /memory/candidates`

Stores a candidate memory item.

Rules:

- intended for auto-dream and client writeback pipelines
- candidate memories are not canonical truth
- exact candidate duplicates collapse onto the existing candidate item
- `belief_branch` can be set before promotion so competing hypotheses stay separable

### `POST /memory/promote`

Promotes a candidate into canonical memory.

Rules:

- promotion can adjust scope, confidence, tags, and TTL
- promotion can also move a record onto a named `belief_branch`
- exact canonical duplicates collapse onto the existing canonical item
- this is the intended path from dream output into durable memory

### `POST /memory/expire`

Explicitly expires or demotes a memory item.

Rules:

- default target status is `expired`
- can also be used to mark an item as `stale`, `superseded`, or `contested`
- intended for policy workers and future verification jobs

### `POST /memory/verify`

Marks a memory item as freshly verified.

Rules:

- updates `last_verified_at`
- can optionally adjust confidence
- can optionally reset status back to `active`

### `POST /memory/repair`

Runs an explicit bounded repair action for a memory item.

Rules:

- supports `verify`, `expire`, `supersede`, `contest`, and `correct_metadata`
- supports `prefer_branch` to mark one belief branch as the current preferred contradiction lane
- keeps the lifecycle explicit and auditable
- can update workspace and visibility lanes through the same audited repair path
- can update source metadata, tags, confidence, and supersede links when needed
- returns the repaired item and the reasons the action was applied

### `GET /memory/inbox`

Surfaces memory that needs attention.

Rules:

- includes candidate items and non-active items by default
- intended for review, promotion, verification, and cleanup
- returns reasons for attention on each item
- filtered by project and namespace when provided
- optional `belief_branch` limits review to one hypothesis lane
- route and intent can be used to bias what rises to the top
- retrieval intent now includes `procedural` and `self_model` for workflow recall and agent self-knowledge recall

### `GET /memory/explain`

Explains why a specific memory item exists.

Rules:

- returns the item itself
- returns the resolved entity when available
- returns a bounded recent event timeline when available
- returns canonical and redundancy keys
- returns source and lifecycle reasons
- returns source-memory drilldown for the item's project, namespace, and source tuple
- returns sibling belief branches for competing records with the same redundancy lane
- returns whether the current branch is preferred and whether contradiction state is unresolved
- returns a bounded rehydration lane so compact summaries can zoom back into deeper evidence
- returns explicit policy hooks for retrieval, verification, promotion, and conflict handling
- returns compact retrieval-feedback counters derived from durable retrieval events
- returns explicit trust demotion hooks when the top source lane falls below the policy floor
- returns explicit procedural and self-model hooks when those first-class memory lanes are involved
- optional `belief_branch` rejects mismatched lookups instead of silently crossing branches
- route and intent are echoed in the response

### `GET /memory/entity`

Returns the object-permanence view for a specific memory item.

Rules:

- exact-id lookup only
- returns the resolved entity when available
- returns a bounded recent event list
- intended for object identity and state inspection without pulling full context

### `GET /memory/timeline`

Returns the recent timeline for a specific memory item.

Rules:

- exact-id lookup only
- returns the resolved entity when available
- returns a bounded event list ordered by recency
- intended for "what changed" queries without scanning broader memory

### `POST /memory/maintenance/decay`

Runs a bounded salience decay sweep over inactive entities.

Rules:

- intended for the background worker
- updates entity salience and rehearsal state
- can emit decay events into the timeline
- keeps unused traces from staying artificially hot forever

### `GET /memory/policy`

Returns the live policy snapshot that the server is currently applying.
The CLI mirrors this with `memd policy --summary` for a compact runtime view.
Use `memd skill-policy --summary` to inspect just the skill lifecycle gate.
Use `memd skills --summary` to inspect the visible skill lane, including
built-in read-only routes and project-local custom skills.
Use `memd skills --query <term>` to drill into one skill and see its file,
status, and recommended next step.
Use `memd memory --query <term>` to search the compiled memory pages under
`.memd/compiled/memory/`.
Use `memd memory --open <lane-or-slug>` to open one compiled memory page or
item page directly.
Use `memd memory --lane <name>` as a shortcut for the lane page.
Use `memd memory --item <slug>` to jump straight to a compiled memory item.
Use `memd memory --list --lanes-only`, `--items-only`, or `--filter <text>` to
narrow the compiled memory index without opening page content.
Use `memd memory --list --grouped` to render the compiled memory index as
lane-grouped markdown links. Add `--expand-items` to show every item link under
each lane instead of collapsing the lane section.
Use `memd memory --list --json` to export the compiled memory index as a
tool-friendly JSON object.
Use `memd memory --quality --summary` to score the current compiled memory
surface against the quality rubric and see the biggest gaps fast.
The compiled memory index includes project, namespace, session, and optional
tab ID so two live Codex tabs can stay separate without guessing.
Use `memd events --summary` to inspect the live event compiler lane.
Use `memd events --list` to export the compiled event index as JSON.
Use `memd events --query <term>` to search the compiled event pages.
Use `memd events --open <target>` to open one compiled event page directly.
The live event lane is refreshed by `hook capture` and `checkpoint`.
Use `memd skill-policy --query --summary` to inspect stored apply receipts and
activation records.
Use `memd skill-policy --write --apply` to write batch artifacts under
`state/skill-policy-batch.*` and the low-risk activate queue under
`state/skill-policy-review-queue.*`, `state/skill-policy-activate-queue.*`,
and `state/skill-policy-apply-receipt.*`.
The same apply receipt is also posted to
`POST /coordination/skill-policy/apply` for durable server-side audit.
Activations are persisted separately and can be queried at
`GET /coordination/skill-policy/activations`.
Use `memd packs --root .memd --summary` to browse visible harness packs and
`memd packs --root .memd --json` when a UI needs structured pack data.
Use `memd peer --summary` and `memd claims --summary` to see session plus tab
labels in the coordination surfaces when multiple Codex tabs are live.

Rules:

- exposes the default retrieval order
- exposes route defaults by intent
- exposes working-memory, promotion, decay, and consolidation thresholds
- exposes the retrieval-feedback channels tracked by the server
- exposes the default source-trust floor used by policy-aware ranking
- exposes the live runtime doctrine for read-once, event-driven, gated memory
  behavior
- exposes the bundle-local event compiler lane and its compiled event pages
- intended for operator inspection and debugging

### `POST /memory/search`

Searches stored memory using:

- optional text query
- optional retrieval route
- optional retrieval intent
- optional scope filters
- optional kind filters
- optional status filters
- optional project filter
- optional namespace filter
- optional workspace filter
- optional visibility filter
- optional belief-branch filter
- optional source agent filter
- optional tags
- optional stage filter
- bounded result count
- bounded per-item content length

The response echoes the resolved retrieval route and intent.

Ranking rules:

- source trust below the policy floor is demoted, not hidden
- high-trust source lanes receive a small deterministic boost
- contested or weak source lanes remain inspectable through explain and inbox flows

### `GET /memory/context`

Returns the compact context package for a client.

Default retrieval order:

1. `local`
2. `synced`
3. `project`
4. `global`

Rules:

- route and intent are resolved before retrieval
- returns active items only
- bounded by a small default limit
- optional visibility filters narrow the hot path without changing route resolution
- when `workspace` is provided, matching workspace memory is preferred in ranking instead of acting as a strict exclusion wall

Codex uses the same bundle-local harness pack flow behind these APIs:

- `memd wake` reads compiled memory before the turn
- `memd checkpoint` and `memd hook capture` refresh the live lane after the turn
- repeated reads in the same turn reuse the turn-scoped cache
- if backend recall or capture fails, the CLI keeps using `.memd/MEMD_WAKEUP.md`
  and `.memd/MEMD_MEMORY.md` plus the Codex agent copies already on disk

OpenClaw is the second harness pack and comes from the shared preset schema. It follows the same visible-bundle rule, but its primary flow is compact context before the task and spill at compaction boundaries.

Hermes is the adoption-focused harness pack and comes from the shared preset schema. It follows the same visible-bundle rule, but its primary flow is onboarding-friendly wake and capture with cloud-first reach and self-host later.

Agent Zero is the zero-friction harness pack and comes from the shared preset schema. It follows the same visible-bundle rule, but its primary flow is fast resume, durable remember, and clean handoff for fresh sessions.

OpenCode is the shared-lane harness pack and comes from the shared preset schema. It follows the same visible-bundle rule, but its primary flow is resume, remember, and handoff for clients that prefer explicit continuity commands.

- cross-workspace shared memory can still appear when it remains relevant
- project-scoped items outrank unrelated global memory
- item content is compacted before response
- TTL-expired items are automatically demoted before retrieval
- old unverified canonical items are marked stale before retrieval

### `GET /memory/context/compact`

Returns the same ranked context as `/memory/context`, but as compact records optimized for low token overhead.

Rules:

- route and intent are echoed in the response
- stable field ordering
- flattened metadata
- compact content payload
- intended for agent hot-path retrieval

### `GET /memory/working`

Returns the managed working-memory buffer for a task.

Rules:

- uses an explicit total character budget
- applies an admission limit for the hot set
- inherits optional workspace and visibility filters from the request
- reports evicted records when the candidate set overflows the buffer
- exposes a bounded rehydration queue using the same evidence shape as `/memory/explain`
- can optionally trigger semantic consolidation for recent traces
- uses source trust as a deterministic ranking input and carries bounded source metadata in rehydration records

### `GET /memory/source`

Returns aggregated source lanes for matching memory items.

Rules:

- supports optional project, namespace, workspace, and visibility filters
- groups by source lane plus workspace visibility
- preserves trust score, confidence, and status mix for each lane
- intended for provenance drilldown, repair triage, and shared-workspace inspection

### `GET /memory/workspaces`

Returns aggregated workspace lanes for matching memory items.

Rules:

- supports optional project, namespace, workspace, visibility, source-agent, and source-system filters
- groups by project, namespace, workspace, and visibility
- reports how many distinct source lanes are contributing to each shared lane
- preserves trust, confidence, and contested-state visibility for handoff inspection
- intended for operator-facing shared-memory status, not raw recall ranking

## Runtime

- default bind: `127.0.0.1:8787`
- default SQLite path: `./memd.db`
- override database path with `MEMD_DB_PATH`
- shared deployments should be reachable only over Tailscale or another
  private VPN/private network, with `MEMD_BASE_URL` pointed at the service
  endpoint

## Future Endpoints

- `POST /memory/graph/search`
