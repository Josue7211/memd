---
status: closed
severity: high
phase: unassigned
opened: 2026-04-14
scope: unspecified
---
# Deep Research Not Stored as Shared memd Memory

status: closed
severity: high
fixed: 2026-04-14
phase: dogfood
opened: 2026-04-14

## Problem

When agents deep-mine inspiration repos (mempalace, supermemory, Omegon, Smriti),
the extracted knowledge lives only in the conversation transcript and in static
`.memd/lanes/architecture/` markdown files. It is never ingested into memd's own
memory substrate as queryable, scope-tagged items.

Result: every new session that needs implementation details from those repos must
re-read the raw source files from scratch. "Read once, compile forever" — the
exact pattern we extracted from mempalace — is not applied to our own research.

## Observed Impact

- A2 extraction session ran 3 deep-read subagents across 4 repos, consuming
  ~150k tokens of context to re-mine files that were already fully analyzed
  in a prior session.
- The prior session's analysis was lost to context eviction. No memd memories
  captured the concrete details (thresholds, schemas, algorithms).
- This is a direct dog-food failure: memd exists to prevent exactly this.

## Root Causes

1. **No shared/synced scope for research artifacts** — current memd scopes
   (local, synced, global) don't have a clear "shared research" workflow.
   Research naturally falls between local (too narrow) and synced (too broad).
2. **No ingest path for lane architecture docs** — `.memd/lanes/architecture/*.md`
   are markdown files on disk, not compiled memory items in the DB. They can't
   be recalled via `memd lookup`.
3. **Agent sessions don't auto-capture research findings** — there's no hook
   or workflow that says "you just deep-mined a repo, store the results as
   durable memories before the session ends."

## Fix Applied

1. Wired `--route memory` override in `ingest_auto_route()` — when route is
   "memory" or "text", `.md` files go to the memory DB instead of the RAG sidecar.
   (`crates/memd-client/src/runtime/ingest_runtime.rs`)
2. Added `memd ingest-sources` CLI command — batch-ingests all `.md`/`.txt` files
   from a directory into the main memory DB as canonical items with lane tags.
   (`crates/memd-client/src/runtime/ingest_runtime.rs`, `cli/args.rs`, `cli/mod.rs`)
3. Uses `scope=project`, `kind=fact`, tags `lane:<name>`, `research`, `ingested-source`.
   Dedup via redundancy_key prevents duplicate creation on re-runs.
4. Wake compiler already surfaces canonical project items through `context_compact`.
   No wake changes needed.

### Initial ingest run (2026-04-14)

- 15 architecture lane docs → canonical memory (A2-01 through A2-13, index, infra)
- 6 inspiration lane docs → canonical memory
- 2 theory root docs, 8 theory locks, 4 models, 4 teardowns → canonical memory
- Total: 39 research docs ingested as queryable items
- Verified: `memd lookup --query "mempalace" --tag "research"` returns 5 items

## Related Backlog

- `2026-04-14-no-source-ingestion-pipeline.md` — lane docs not ingestible
- `2026-04-13-session-resume-no-memd-memory.md` — session state not stored
- `2026-04-14-theory-design-docs-not-ingestible.md` — design docs not queryable
- `2026-04-14-five-starter-lanes-no-source-material.md` — lanes exist but empty
