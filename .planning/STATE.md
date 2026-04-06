# STATE

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-04-05)

**Core value:** Give agents global, cross-project, project, and short-term memory that stays compact, durable, inspectable, and useful under real task pressure.
**Current focus:** finish `v5` as a global-first shared memory system with explicit cross-project overlays and provider-collision controls, then layer `v6` token optimization on top of that shared substrate.

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
- phase 46 policy-aware coordination suggestions now generate bounded action hints from current inbox, recovery, policy, and pressure signals
- MCP peer coordination now exposes a `coordination_suggestions` surface that returns those suggested actions for richer operator tools
- phase 47 gap-finding foundations now emit evidence-driven candidates from eval, resume, and coordination state plus baseline delta summaries
- shared short-term state now has a real-time canonical sync layer across machines and harnesses
- the next architectural gap is that global-first memory, cross-project live lanes, and provider-collision controls still need to become first-class shipped behavior
- the roadmap now explicitly treats token optimization as a primary `memd` capability, not an incidental benefit
- the roadmap now includes transcript/context observability, compiled raw-to-graph knowledge retrieval, universal design memory, and large-context workflow compression as planned capabilities
- the roadmap now includes a Codex-native `memd-reload` skill and matching shell shim so existing sessions can bootstrap memory immediately instead of waiting for the next tool use
- `memd init` now seeds a project bundle from existing project docs and memory files when it can infer a project root, so new project bundles are not blank
- bootstrap imports now write a fingerprinted source registry so unchanged local harness memories can be treated as read-once inputs
- provider collision controls now distinguish live sessions by provider, harness, machine, workspace, and effective agent
- the Codex reload skill now gives already-open sessions a one-command bootstrap path
- project init now seeds from repo docs and existing memory files instead of leaving a blank project bundle
- the unified `memd` front door now stays the default remembered Codex command
- bootstrap preflight now reports which project scaffolding exists before init
- repo introspection now uses a fingerprinted source registry and delta refresh instead of rereading unchanged files
- memory health and provenance are already carried through the schema and surfaced in ranking / maintenance views
- Codex, Claude, and OpenClaw bootstrap paths are now aligned around the shared memd substrate
- native consolidation already exists as the dream/autodream lifecycle substrate
- token and context observability are already surfaced in the hot client paths
- compiled knowledge workspace paths already exist and can outrank raw rereads when fresh
- large-context workflows are now treated as layered briefs, glossaries, entity sheets, chunk windows, and reconciliation passes
- the roadmap now includes a unified `memd` Codex skill as a single front door that routes to init or reload automatically
- the bootstrap flow still needs to preflight `AGENTS.md` / `CLAUDE.md` / planning docs so it can report what project scaffolding exists before init
- the roadmap now treats repo introspection, incremental git-aware sync, and memory provenance/health as first-class gaps
- the roadmap now includes explicit agent adoption so Codex, Claude, and OpenClaw all bootstrap from memd once and then treat it as the shared source of truth
- phase 47 gap-finding foundations are now implemented as a bounded, repo-aware research pass over planning artifacts, docs, git state, eval snapshots, and runtime wiring

## Open Loops

- the highest-priority open loop has moved from canonical short-term sync to global-first overlays and cross-project lanes on top of the shared substrate.
- the next open loop after that is making `memd` global-first, with explicit cross-project live lanes and provider-collision controls so one shared substrate can span harnesses safely.
- the bootstrap UX still needs a deeper import path for projects with lots of existing docs or memory files so the default seed can stay concise without losing important context.
- the single front door still needs to stay as one obvious command that auto-routes without making the user think about helper commands.
- the project bootstrap flow still needs to report whether agent scaffolding exists instead of assuming a separate Codex `/init`.
- the project bundle still needs structured import from repo state and git diffs, not just a bootstrap sweep.
- memory health needs visibility into stale, duplicated, inferred, and conflicting facts.
- the bootstrap source registry still needs delta-refresh wiring so changed local files can be reimported without rereading the unchanged ones.
- the runtime adoption layer still needs to be wired into every major agent surface, not just Codex and Claude/OpenClaw bootstrap surfaces.
- the next layer after canonical collision controls is broadening the same safety model into more UI/status surfaces and eventual automatic reconciliation.
- the gap loop now needs to drive the next `v6` phases, starting with dream/autodream foundations and token/context observability.
- after those `v5` gaps are closed, phase 47 refinement should continue with token observability and compiled-first retrieval on top of the shared sync substrate.

## Next Command

Prioritize the shared-memory substrate: make `memd-server` the canonical real-time state hub for focus, blockers, claims, heartbeats, branch/port awareness, and handoffs across machines and harnesses; then make `memd` global-first with cross-project live lanes and provider-collision controls, keep the Codex front door to one obvious command, make project bootstrap preflight explicit, add structured repo introspection and incremental sync, wire adoption into Codex/Claude/OpenClaw startup paths, and keep `memd init` seeding project bundles from existing docs before continuing phase 47 token observability and compiled-first retrieval.

---
*Created: 2026-04-04 during GSD brownfield initialization*
