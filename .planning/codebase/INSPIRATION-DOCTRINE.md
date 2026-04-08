# Inspiration Doctrine

## Core Rule

- Read raw source once.
- Compile it into visible memory objects.
- Reuse compiled memory by default.
- Reopen raw only when changed, uncertain, or repairing.

## Memory Rule

- Every memory must be visible, linkable, and inspectable.
- Every memory must keep source provenance.
- Every memory must support live updates from agent events.
- Every memory must support version history and rollback.

## Recall Rule

- LightRAG or similar semantic indexing is the find engine.
- It must not be the hidden source of truth.
- Search returns a memory object, not an orphaned blob.
- The UI always resolves recall back to a viewable object.

## Edit Rule

- Agents update memory by event and patch, not by full rewrite.
- Humans can edit the visible object directly.
- Raw source stays attached for drilldown.
- Reindex happens async after the patch.

## Self-Evolution Rule

- Memory evolves automatically from events.
- Repeated patterns can become skill drafts.
- Skills are proposed automatically, but gated before use.
- Skill activation requires evaluation, usefulness, and policy approval.

## Token Rule

- Summaries first.
- Raw source on demand.
- Delta updates over rereads.
- One concept, one memory object.

## Product Shape

- `raw/` = source archive
- `memory/` = visible working truth
- `events/` = live agent activity
- `index/` = semantic recall
- `ui/` = browse/edit surface
- `skills/` = gated proposals from repeated patterns
- `plugins/` = harness-specific integrations that auto-recall and auto-capture
- `cache/` = turn-scoped retrieval reuse so the same turn does not pay twice

## Final Doctrine

The best version of memd is a living memory system:

- read raw once
- compile memory once
- evolve memory continuously
- propose skills from repeated behavior
- package memory as per-harness plugins over one API
- keep humans in control of what ships
- keep every memory visible
- keep recall semantic and token efficient
- keep the graph as a separate browse surface, not hidden behind search
