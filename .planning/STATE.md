# STATE

> Planning state artifact. May lag current truth. Check [[ROADMAP]] and
> [[docs/WHERE-AM-I.md|WHERE-AM-I]] first.

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-04-05)

**Core value:** Give agents global, cross-project, project, and short-term memory that stays compact, durable, inspectable, and useful under real task pressure.
**Current focus:** see [[ROADMAP]] for current active phase and blockers.

**Biological stance:** copy the brain where it is strong:
- small high-priority working memory
- fresh local truth precedence
- layered memory
- selective consolidation
- cue-driven retrieval
- energy-efficient compressed cognition

Do not copy its weaknesses:
- confabulation
- hidden belief drift
- poor provenance
- contradiction collapse
- unsafe side effects

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
- phase 48 scenario harnesses now exist for resume, handoff, workspace retrieval, stale-session recovery, and coworking flows
- phase 49 composite scoring now combines eval, scenario, coordination, latency, and bloat signals into one explicit acceptance gate
- phase 50 bounded experiments now snapshot, run, score, consolidate, and restore rejected bundle changes automatically
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
- the roadmap now requires harness coverage enforcement so promoted skills, CLIs, and procedures cannot masquerade as universal when a harness mapping is missing
- the next learning-system gap is adapter generation and bridge surfacing for missing harness mappings, not just portability labels
- the next capability gap is full harness inventory: enabled plugins, plugin commands, hooks, agents, teams, and bridgeable install surfaces still need to become canonical memd data instead of hidden harness-local state
- user corrections now need to become learned operating policy so dream/autodream/refresh can stop repeating the same workflow mistakes
- the ceiling roadmap now targets a layered external cortex:
  - `live_truth` for freshest verified local reality
  - project brain for durable project knowledge
  - user policy for cross-project stable operating rules
  - promoted abstractions for reusable skills, adapters, and procedures
- the ceiling roadmap now treats token efficiency as a first-class success metric with a stated goal of up to `90%` best-case token reduction on iterative workflows without quality regression
- the ceiling roadmap now treats seamless compaction and short-context operation as core product behavior, not optional optimization
- the ceiling roadmap now requires a hard runtime safety boundary: normal memory operations may not mutate shared runtime state
- the ceiling roadmap now requires harness capability contracts so agents know what is actually runnable and stop guessing

## Open Loops

- the highest-priority open loop is replacing refresh-first behavior with live truth and truth-first retrieval.
- seamless compaction still needs to become event-driven and automatic so long work stops depending on long context sessions.
- capability discovery now surfaces a canonical capability-contract registry and bridge contract summary, but it still needs deeper per-harness contract semantics.
- the runtime safety boundary is now enforced on passive bundle refresh/resume paths so normal memory operations cannot mutate shared user runtime state; explicit init still owns bridge application.
- migration now has a provenance manifest and reuse path for existing bundle/source registries, but it still needs deeper conversion of old bundle state into the layered live-truth architecture.
- token optimization is now being enforced in the prompt budget estimator and optimization hints, but it still needs broader cross-harness policy coverage.
- self-evolution still needs to become a controlled policy-learning loop with promotion gates, rollback, and deprecation.
- cross-project memory still needs explicit promotion rules so global intelligence does not become contamination.
- bundle resume now carries a bounded live-truth lane plus a compact event spine derived from git status, diff summaries, and resume deltas, but that is still only a stepping stone toward a full event-driven cognition layer.
- the current system still needs to make user corrections durable enough that the next answer cannot fall back to stale assumptions.

## Next Command

Prioritize the ceiling substrate: make `memd-server` the canonical live-truth hub for fresh local reality, corrections, command outcomes, capability discoveries, focus, blockers, claims, and handoffs; make retrieval truth-first and compaction seamless; add harness capability contracts so agents know what is actually runnable; enforce runtime non-interference for normal memory operations; migrate existing bundles into the layered model safely; then push token efficiency toward the `90%` best-case target without accepting quality regressions.

OpenClaw stack peers must be treated as first-class services, not vague labels: `memd`, `claw-control`, `agent-shell`, and `agent-secrets` need explicit peer identity, capability, and authority metadata so one session can escalate dependency/runtime failures to the right active service peer and get a proper product-level fix instead of an isolated patch.

---
*Created: 2026-04-04 during GSD brownfield initialization*
