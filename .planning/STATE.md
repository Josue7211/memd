# STATE

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-04-04)

**Core value:** Give agents short-term and long-term memory that stays compact, durable, inspectable, and useful under real task pressure.
**Current focus:** Queue the next short-term memory slice after checkpoint refresh writeback.

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

## Open Loops

- planning roadmap needs the next `v4` phase queued from the strategic roadmap

## Next Command

Queue the next `v4` self-optimizing memory phase after evaluation gates, then continue execution.

---
*Created: 2026-04-04 during GSD brownfield initialization*
