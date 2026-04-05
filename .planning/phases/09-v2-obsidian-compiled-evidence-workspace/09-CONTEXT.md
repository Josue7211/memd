# Phase 09: `v2` Obsidian Compiled Evidence Workspace - Context

**Gathered:** 2026-04-04
**Status:** Completed from shipped repo state
**Mode:** Backfilled from implemented behavior

<domain>
## Phase Boundary

Treat compiled markdown pages as a first-class evidence lane inside the vault.
`obsidian compile` should be able to generate durable memory and evidence pages,
not only transient query pages, while preserving typed-memory provenance and
rehydration details in the markdown output.

</domain>

<decisions>
## Implementation Decisions

### compiled pages are evidence, not exports

Compiled query pages, compiled memory pages, and writeback pages all live under
the vault as durable markdown artifacts with index coverage and direct-open
support.

### typed provenance remains the source of truth

The vault page reflects typed-memory provenance and rehydration details instead
of flattening everything into plain note text.

</decisions>

<code_context>
## Existing Code Insights

- `crates/memd-client/src/obsidian.rs` owns compiled note and compiled memory
  page generation.
- `crates/memd-client/src/main.rs` owns `obsidian compile` routing and index
  updates.
- the shared rehydration model from phase 8 already exists and is reused here.

</code_context>

<specifics>
## Specific Ideas

- support `obsidian compile --id <uuid>` for specific memory pages
- keep compiled memory pages under `.memd/compiled/memory/`
- keep compiled query pages and compiled memory pages under one index
- preserve provenance, source links, and rehydration details in compiled output

</specifics>

<deferred>
## Deferred Ideas

- richer semantic overlays beyond the current bounded evidence/semantic lanes
- deeper vault-native browsing UI outside the CLI/Obsidian path

</deferred>
