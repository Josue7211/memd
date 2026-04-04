# GSD Roadmap: memd

This planning roadmap mirrors the strategic roadmap in [`ROADMAP.md`](../ROADMAP.md)
but keeps execution anchored to phase-shaped work inside `.planning/`.

## Active Milestone

### Milestone 0: OSS Foundations

#### Phase 0: Branches, Version History, and Contribution Rules

Status: Complete

- move active development onto a dedicated branch strategy
- split large files only where it improves reuse and maintenance
- establish a branch strategy for phased work and release history
- make contribution, review, and security expectations explicit
- separate public project guidance from internal planning artifacts

Success:

- active work happens on a branch by default
- external contributors can understand how to work on the project without tribal knowledge
- phased work maps cleanly to branches, commits, and versioned history

### Milestone 1: Finish `v1`

#### Phase 1: `v1` Completion

- complete provenance drilldown from compact memory to raw artifacts
- add repair actions for stale, contested, and malformed memory
- harden working-memory admission, eviction, and rehydration behavior
- tighten source-trust and procedural/self-model surfaces enough to call `v1` complete

Status: Complete

Success:

- `v1` can be described as complete without hand-waving around missing repair and provenance features

### Milestone 2: Start `v2`

#### Phase 2: `v2` Foundations

- explicit working-memory controller semantics
- trust-weighted source memory
- reversible compression
- first learned retrieval-policy hooks

Status: Complete

Success:

- `memd` begins moving from brain-inspired memory toward machine-advantaged memory

## Immediate Next Phase

All current roadmap phases are complete.

---
*Created: 2026-04-04 during GSD brownfield initialization*
