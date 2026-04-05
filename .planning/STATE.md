# STATE

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-04-05)

**Core value:** Give agents short-term and long-term memory that stays compact, durable, inspectable, and useful under real task pressure.
**Current focus:** Start `v5` coordination subscription and hook surfaces on top of the watch-aware coworking substrate.

## Current Status

- Brownfield GSD initialization completed
- `.planning/config.json` exists
- `.planning/codebase/` map created
- top-level roadmap is organized around `v0` through `v5`
- `v0` OSS-ready project foundations are complete on a dedicated branch
- branch/version history, contribution, security, and release docs are in place
- `v1` repair, provenance, and working-memory gaps are closed enough to move on
- `v2` foundations are in place with explicit trust floors, rehydration lanes, and policy hooks
- `v2` branchable beliefs are in place with explicit belief branches and sibling inspection
- `v2` retrieval feedback is in place with durable retrieval events and compact explain counters
- `v2` trust-weighted ranking is in place across search and working memory
- `v2` contradiction resolution is in place with preferred branch state
- `v2` reversible compression and rehydration are closed in the planning record
- `v2` Obsidian compiled evidence workspace is closed in the planning record
- `v3` workspace-aware retrieval priorities are closed in the planning record
- the first `v4` memory evaluation harness is in place for bundle-backed resume quality
- bundle evaluation snapshots can now be written for future comparison
- bundle evaluation now compares against the latest baseline and reports drift
- bundle evaluation can now fail on score thresholds or regressions for automation use
- bundle evaluation now emits concrete corrective recommendations from live resume state
- bundle resume and handoff now keep semantic recall off the hot path unless explicitly requested
- bundle workflows now have a dedicated short-term checkpoint command for current-task state
- short-term checkpoints now refresh visible bundle memory files immediately after writeback
- default attach and agent launch surfaces now resume with `current_task` intent
- bundle status preview now mirrors the same current-task hot path
- the installed hook-context path now defaults to `current_task` intent too
- bundle root memory is now written to `MEMD_MEMORY.md` to avoid collisions with agent-native `MEMORY.md` files
- Claude-native bundle imports now bridge `memd` into `CLAUDE.md` and `/memory`
- resume surfaces now show a current-task snapshot and compact change summaries
- durable `remember` writes now refresh visible bundle memory immediately
- automatic short-term capture now records meaningful coordination transitions
- retrieval ranking now prefers verified canonical evidence over unverified synthetic continuity
- peer coordination now has brokered messages, claims, claim transfer, and assignment-friendly handoff primitives
- the first MCP-native peer coordination bridge now exposes brokered coworking tools directly to agent clients
- shared-task orchestration now exists across backend, CLI, and MCP surfaces
- coordination inbox and task presence now exist as a compact coworking surface
- stale-session recovery now exists across backend, CLI, and MCP surfaces
- coordination policy and ownership guards now distinguish exclusive-write and collaborative lanes
- advisory branch and scope recommendations now exist across coordination views
- compact coordination receipts now record recent coworking transitions

## Open Loops

- planning roadmap now needs phase 43 execution so live coordination pressure can feed other agent and operator surfaces without bespoke polling logic

## Next Command

Execute phase 43 so coordination changes can plug into other surfaces through a stable subscription or hook layer.

---
*Created: 2026-04-04 during GSD brownfield initialization*
