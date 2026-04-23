---
phase: E2
name: Atlas Activation
version: v2
status: reopened
depends_on: [B2, C2]
backlog_items: [44, 51, 52]
reopened_at: 2026-04-15
reopened_reason: Atlas code exists but entity links don't populate from real usage. Navigation dead-ends. Wake packet doesn't include atlas region hints. Progressive zoom (wake→region→entity→evidence) not proven end-to-end.
---

# Phase E2: Atlas Activation

Current status: `reopened` — atlas infrastructure exists but is inert. Entity links table empty in production. Wake packet doesn't include region hints. Navigation from wake to raw evidence untested with real data.

## Reopened Scope

- **Entity links populated**: co-occurrence analysis must run and populate links from real items
- **Wake includes atlas hints**: top regions surfaced in wake packet
- **Progressive zoom proven**: wake → region → entity → evidence in ≤ 4 hops with real data
- **Wiki link resolution**: `[[entity]]` in stored content creates navigable entity link
- **Backlinks work**: navigate from entity back to all items that reference it

## Node Verification (from [[docs/verification/NODE-VERIFICATION-MATRIX.md]])

This phase owns M2-tier verification for:
- S2 (memory atlas): navigable wake→region→node→evidence, backlinks work

## Goal

Atlas wakes from dormancy. Memory is navigable: wake → region → node → evidence.

## Deliver

- Atlas wired into resume/wake path (top regions in wake packet)
- Entity links auto-populated from co-occurrence
- Client methods for explore/region/trail
- Wiki link parsing in stored content
- Progressive zoom working: wake → region → entity → raw evidence

## Pass Gate

- `memd explore` returns non-empty regions for a project with 10+ items
- Entity links table has rows (not permanently 0)
- Wake packet includes atlas region hint
- Wiki link `[[entity]]` in stored content creates entity link
- Navigate from wake to raw evidence in ≤ 4 hops

## Evidence

- Atlas region count before/after wiring
- Entity links table row count
- Navigation trace from wake → evidence (logged)
- Wiki link resolution test

## Fail Conditions

- Atlas still unreachable from runtime
- Entity links don't populate
- Navigation dead-ends before evidence

## Donor Extraction (from inspiration repos)

- **E2-D1** (mempalace `knowledge_graph.py`): SQLite triples with temporal validity — `(subject, predicate, object, valid_from, valid_to, source_id, confidence)`. Add temporal columns to `memory_entity_links`.
- **E2-D2** (Omegon `sqlite.rs` edges table — **DIRECT RUST LIFT**): Knowledge graph edges with `reinforcement_count`, `last_reinforced`, `decay_rate`. Edges strengthen on repeated co-occurrence. Weak edges decay and archive. Directly stealable Rust schema.
- **E2-D3** (supermemory `memory-graph/`): D3 force config reference — charge=-2000, collision doc=70px memory=35px, alpha decay=0.025, 150 pre-settlement ticks. Edge types: derives, updates, extends.
- **E2-D4** (Omegon `sqlite.rs` — **DIRECT RUST LIFT**): Entity aliasing and auto-extraction. Auto-extract aliases from project, namespace, agent, source_path, file_name. Merge on entity update (union of sets).

See: `docs/theory/2026-04-14-donor-extraction-to-v2-phases.md` for full details.

## Rollback

- Revert atlas integration if it bloats wake packet beyond budget
