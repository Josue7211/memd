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

#### Phase 3: `v2` Branchable Beliefs

- keep conflicting durable beliefs in explicit named branches instead of flattening them
- make competing records inspectable through explain and search surfaces
- preserve duplicate control by separating redundancy and canonical keys across belief branches
- keep the first branchable-belief slice compatible with the current SQLite payload model

Status: Complete

Success:

- conflicting beliefs can live in separate durable branches and operators can inspect sibling branches directly

#### Phase 4: `v2` Retrieval Feedback

- capture retrieval outcomes so future ranking can learn from use instead of only fixed heuristics
- expose lightweight retrieval feedback events and counters through the existing explain and policy surfaces
- keep the first feedback loop deterministic, bounded, and cheap enough for the hot path

Status: Complete

Success:

- `memd` has an explicit retrieval-feedback substrate for future adaptive ranking

#### Phase 5: `v2` Trust-Weighted Ranking

- make source-trust floors influence search and working-memory ranking instead of only policy display
- penalize weak or contested source lanes predictably without hiding them from inspection
- keep trust-aware ranking deterministic and explainable before any learned policy takes over

Status: Complete

Success:

- low-trust memory is demoted in ranking while remaining visible and auditable

#### Phase 6: `v2` Contradiction Resolution

- turn branchable belief lanes into an operator-visible resolution workflow
- expose preferred, contested, and unresolved branch state explicitly
- keep contradictory branches queryable while allowing one branch to become the current preferred lane

Status: Complete

Success:

- contradictory belief branches can be inspected and resolved without flattening history

#### Phase 7: `v2` Procedural and Self Model Memory

- make procedural memory and self-model memory first-class instead of implicit tags
- expose retrieval and repair surfaces for runbooks, capabilities, and failure modes
- keep the first slice narrow enough to remain compatible with current typed memory records

Status: Complete

Success:

- `memd` stops hand-waving procedural and self-model memory as future ideas

#### Phase 8: `v2` Reversible Compression and Rehydration

- add a bounded evidence rehydration model behind summary-first retrieval
- make explain and working-memory surfaces expose deeper evidence without dumping raw transcripts
- keep reversible compression compact, explicit, and compatible with the current artifact trail model

Status: Complete

Success:

- `memd` can move from compact summaries to deeper evidence without hallucinating the missing detail

#### Phase 9: `v2` Obsidian Compiled Evidence Workspace

- treat compiled markdown pages as a first-class evidence lane inside the vault
- let `obsidian compile` generate durable memory/evidence pages, not only query pages
- keep compiled wiki artifacts indexed and directly openable from the vault workspace
- preserve typed-memory provenance and rehydration details inside compiled markdown output
- preserve a path from raw source -> compiled wiki -> typed graph so repeated retrieval does not depend on rereading cold raw files

Status: Complete

Success:

- Obsidian is a real compiled memory workspace instead of only an ingest/writeback side path

### Milestone 3: `v3` Federated and Collective Memory

#### Phase 10: `v3` Shared Workspace Foundations

- define shared workspace scopes and namespace boundaries for multi-agent memory
- add permission-aware visibility for shared and private memory lanes
- keep handoff memory and trust tiers explicit across projects and collaborators
- preserve scope, provenance, and auditability when memory moves between agents

Status: Complete

Success:

- teams can share memory without flattening private and public context

#### Phase 11: `v3` Workspace Handoff Bundles

- package shared working memory, inbox pressure, workspace summaries, and recent evidence into resumable handoff bundles
- make agent and human handoff output preserve provenance, trust, visibility, and rehydration state
- add a shared handoff surface that can be emitted as both CLI output and compiled Obsidian pages
- keep handoff retrieval bounded so delegation does not become a transcript dump

Status: Complete

Success:

- shared work can be resumed from a compact handoff bundle instead of rebuilding state from scratch

#### Phase 12: `v3` Workspace Policy Corrections

- let operators correct workspace and visibility lanes through the audited repair path
- keep shared-lane corrections explicit instead of relying on raw re-store operations
- preserve reasons and lifecycle events when memory moves between private and shared lanes

Status: Complete

Success:

- workspace and visibility mistakes can be fixed without bypassing the normal memory audit trail

#### Phase 13: `v3` Workspace-Aware Retrieval Priorities

- prefer the active workspace lane before unrelated shared memory when retrieval has no explicit override
- keep cross-workspace recall available, but demoted behind the active lane for resume and handoff flows
- make the ranking behavior deterministic and explainable before any learned policy layer

Status: Complete

Success:

- shared-memory retrieval respects the active workspace instead of flattening all shared state together

### Milestone 4: `v4` Self-Optimizing Memory

#### Phase 14: `v4` Memory Evaluation Foundations

- add a first deterministic evaluation harness for the bundle-backed memory loop
- score the actual resume path instead of relying on ad hoc operator intuition
- surface weak working-memory, rehydration, workspace-lane, inbox, and semantic-fallback signals in one operator-facing report
- keep the first evaluation slice cheap, local, and explainable before any adaptive policy tuning starts

Status: Complete

Success:

- operators can evaluate bundle memory health from the same control plane that drives resume and handoff

#### Phase 15: `v4` Evaluation Snapshot Persistence

- let evaluation output persist as bundle artifacts instead of only terminal text
- write latest and timestamped evaluation snapshots under the bundle for future comparison
- keep the first persistence slice simple and local before adding automatic regression diffs

Status: Complete

Success:

- bundle memory quality can be recorded over time instead of only observed once

#### Phase 16: `v4` Evaluation Regression Diffs

- compare current bundle evaluation results against the latest saved baseline
- surface score drift and changed dimensions in both summary and persisted artifacts
- keep the first regression slice deterministic and local before adding automatic policy reactions

Status: Complete

Success:

- bundle evaluation can distinguish stable memory health from regression or improvement

#### Phase 17: `v4` Evaluation Failure Gates

- make bundle evaluation usable in automation instead of only operator review
- add explicit score-threshold and regression-failure gates to the CLI
- keep the first gate slice local, deterministic, and easy to wire into hooks, cron, or CI

Status: Complete

Success:

- `memd eval` can fail fast when memory quality drops below a required floor or regresses from baseline

#### Phase 18: `v4` Evaluation Recommendations

- turn raw evaluation findings into concrete corrective actions
- keep recommendations tied to the real resume snapshot instead of generic advice
- preserve a compact operator summary while adding richer markdown guidance in saved artifacts

Status: Complete

Success:

- bundle evaluation tells operators what to do next, not only what is broken

#### Phase 19: `v4` Hot-Path Memory Defaults

- keep bundle-backed short-term memory on the critical path
- move semantic recall behind explicit opt-in flags for resume and handoff
- align generated bundle docs and operator docs with the fast-default contract

Status: Complete

Success:

- default resume and handoff stay fast and local while deeper semantic recall remains available on demand

#### Phase 20: `v4` Short-Term Checkpoints

- add a lightweight command for current-task memory capture
- keep checkpoint writes compatible with the existing typed memory pipeline
- default checkpoints to short-term status memory instead of permanent lore

Status: Complete

Success:

- operators can capture short-term task state quickly without shaping full `remember` requests by hand

#### Phase 21: `v4` Checkpoint Refresh Writeback

- refresh bundle memory files immediately after short-term checkpoint writes
- keep checkpoint writeback on the fast local path without semantic fallback
- make short-term state visible to agents without waiting for a separate resume step

Status: Complete

Success:

- short-term checkpoint writes update the visible bundle memory surface immediately

#### Phase 22: `v4` Current-Task Resume Defaults

- make attach and agent launch surfaces bias toward current-task memory
- keep the fast local resume path intact while improving short-term retrieval intent
- update generated bundle docs so the default launch contract is explicit

Status: Complete

Success:

- default launch flows start from the current-task lane instead of a generic intent

#### Phase 23: `v4` Status Preview Alignment

- make bundle status preview reflect the real short-term launch path
- keep diagnostics on the same fast local lane as default resume
- document that `status` previews current-task memory instead of generic bundle state

Status: Complete

Success:

- `memd status` reports the same current-task hot path that default launches use

#### Phase 24: `v4` Hook Context Current-Task Defaults

- make the installed hook context flow default to current-task intent
- align CLI hook behavior with the generated shell and bundle launch surfaces
- keep hook-based short-term retrieval on the same fast local path

Status: Complete

Success:

- the default hook context path now resumes short-term memory with current-task intent

#### Phase 25: `v4` Agent-Safe Memory Surface Names

- stop using a generic bundle `MEMORY.md` filename that can collide with agent-native memory systems
- keep a memd-specific shared root memory file alongside agent-specific copies
- update integration docs so Codex and other clients use the non-colliding memd surface

Status: Complete

Success:

- memd no longer assumes ownership of a generic `MEMORY.md` filename inside the bundle

#### Phase 26: `v4` Claude Native Memory Bridge

- bridge bundle memory into Claude Code through native `CLAUDE.md` imports
- generate a Claude import target and example file inside the bundle
- treat `/memory`, imports, dream, and autodream as first-class parts of the Claude integration

Status: Complete

Success:

- Claude Code can load `memd` memory through its own native memory system instead of a parallel markdown convention

#### Phase 27: `v4` Resume Current-Task Snapshot

- surface current focus, pressure, next recovery, and active lane in bundle memory and prompt views
- keep the hot lane readable at a glance instead of forcing deep inspection
- make short-term memory feel actionable, not just present

Status: Complete

Success:

- operators can immediately see the active task state from the default resume surfaces

#### Phase 28: `v4` Resume Change Summaries

- persist the last hot-lane snapshot under the bundle
- show a compact “since last resume” delta in prompt and bundle views
- keep short-term pickup fast without transcript replay

Status: Complete

Success:

- resume can explain what changed since the last pickup instead of only dumping current state

#### Phase 29: `v4` Remember Refresh Writeback

- refresh bundle memory files immediately after durable `remember` writes
- keep visible bundle state aligned with durable memory writes
- avoid stale bundle surfaces between write and next resume

Status: Complete

Success:

- visible short-term memory stays aligned with durable writes in the same bundle workflow

#### Phase 30: `v4` Status and Summary Hot-Lane Alignment

- expose resume deltas through `memd status`
- enrich `resume --summary` with focus and pressure instead of only counts
- keep quick inspection aligned with the actual hot lane

Status: Complete

Success:

- lightweight status surfaces carry enough task-state signal to be useful on their own

#### Phase 31: `v4` Automatic Short-Term Memory Management

- capture meaningful short-term state transitions automatically
- avoid transcript dumping by storing only high-signal task changes
- keep the hot lane fresh while leaving dream/autodream to consolidate durable signal later

Status: Complete

Success:

- short-term memory improves with less manual checkpointing while remaining compact and useful

#### Phase 32: `v4` Epistemic Retrieval Behavior

- make retrieval prefer verified evidence over narrative continuity
- keep inferred, claimed, stale, and contested memory explicit in hot and deep recall
- reduce false confidence by making epistemic state affect ranking and inspection

Status: Complete

Success:

- `memd` helps agents remember more while also being wrong less often

### Milestone 5: Start `v5`

#### Phase 33: `v5` Peer Coordination MCP Foundations

- expose backend-brokered peer coordination through a stable MCP-facing contract
- reuse the shared message and claim backend instead of inventing bundle-local coordination again
- preserve session-qualified identity, assignment semantics, and claim safety across agent peers
- keep the first slice narrow around coordination primitives before richer orchestration layers

Status: Complete

Success:

- agent sessions can coordinate through `memd` natively instead of only through CLI wrappers

#### Phase 34: `v5` Shared Task Orchestration

- turn peer coordination primitives into explicit shared-task orchestration
- group assignment, help, review, and ownership around named shared tasks
- preserve session-qualified ownership and claim safety while making coworking easier to inspect
- keep the first orchestration slice narrow before heavier automation and planning layers

Status: Complete

Success:

- simultaneous agent sessions can coordinate through explicit shared tasks instead of only raw messages and scope claims

#### Phase 35: `v5` Coordination Inbox and Task Presence

- combine peer messages, shared task pressure, and ownership state into one coordination view
- surface help, review, and assignment pressure without manual cross-checking
- keep the first slice compact enough for resume, handoff, and MCP use
- preserve explicit session-qualified ownership and inspectability

Status: Complete

Success:

- active sessions can see coordination pressure from one compact surface instead of stitching together messages, tasks, and presence manually

#### Phase 36: `v5` Claim Recovery and Coordination Automation

- detect stale/dead ownership pressure from heartbeats and leased claims
- surface reclaimable claims and stalled shared tasks explicitly
- add safe recovery paths for rerouting blocked coworking lanes
- keep the first automation slice operator-visible and ownership-safe

Status: Complete

Success:

- active sessions can recover blocked shared work without silent ownership drift

#### Phase 37: `v5` Coordination Policy and Ownership Guards

- define lightweight coordination modes such as exclusive write, shared review, and help-only
- surface policy mismatches before overlapping work turns into conflict
- keep the first policy slice compatible with existing claims, tasks, inbox, and recovery flows
- preserve explicit operator-visible ownership rules instead of hidden heuristics

Status: Complete

Success:

- simultaneous sessions can distinguish exclusive ownership from collaborative support lanes before conflict occurs

#### Phase 38: `v5` Branch and Scope Recommendations

- recommend cleaner branches and scopes for active shared tasks
- derive suggestions from coordination modes, claims, and active ownership
- keep the first slice advisory instead of mutating git state automatically
- preserve explicit operator control over final work-boundary decisions

Status: Complete

Success:

- simultaneous sessions can split work more cleanly before implementation overlap begins

#### Phase 39: `v5` Coordination Audit Trail and Receipts

- record compact coordination receipts for assignment, recovery, help, review, and transfer actions
- expose bounded audit views through CLI and MCP surfaces
- keep the first slice structured and compact instead of transcript-like logging
- preserve compatibility with the current peer coordination model

Status: Complete

Success:

- operators can inspect recent coworking transitions without reconstructing them from raw state

#### Phase 40: `v5` Coordination Dashboard and History Views

- add cleaner dashboard-like coordination views for current pressure
- expose bounded history views over recent receipts
- keep the first slice compatible with existing CLI and MCP surfaces
- preserve compactness and inspectability over verbosity

Status: Complete

Success:

- operators can inspect live coworking pressure and recent history faster than raw coordination data

#### Phase 41: `v5` Coordination Drilldown and Filter Views

- add bounded drilldown surfaces for the most relevant coordination slices
- let operators isolate inbox, requests, recovery pressure, policy conflicts, and receipts faster
- keep the first slice compatible with the current CLI and MCP dashboard surfaces
- preserve compactness and operator control instead of introducing a noisy activity feed

Status: Complete

Success:

- operators can move from overview to the exact coordination slice they need without rereading the full dashboard

#### Phase 42: `v5` Coordination Watch and Alert Views

- add bounded watch surfaces for coordination pressure that changes during active coworking
- keep the first slice focused on live refresh and compact alertable summaries instead of a noisy activity feed
- preserve compatibility with the current CLI and MCP coordination categories
- make active pressure easier to notice before operators have to manually poll for it

Status: Complete

Success:

- operators can keep coordination pressure visible as it changes instead of repeatedly rerunning the same static summary

#### Phase 43: `v5` Coordination Subscription and Hook Surfaces

- expose compact coordination change feeds that other surfaces can reuse
- keep the first slice hook-friendly and bounded instead of inventing a heavyweight event system
- preserve compatibility with the current dashboard, drilldown, and watch categories
- reduce bespoke polling logic across CLI, MCP, and future UI surfaces

Status: Complete

Success:

- coordination pressure can feed other agent and operator surfaces through one stable change surface instead of one-off watchers

#### Phase 44: `v5` UI-Friendly Coordination Feed Surfaces

- expose the reusable coordination delta model through cleaner UI-oriented response shapes
- keep the first slice compatible with the current bounded dashboard, drilldown, watch, and change categories
- preserve compactness so richer operator surfaces can consume the feed without transcript bloat
- avoid inventing a second coordination event taxonomy for UI consumers

Status: Complete

Success:

- richer operator surfaces can consume the same bounded coordination delta model without custom adapter glue

#### Phase 45: `v5` Coordination Action Surfaces

- expose bounded coordination actions that richer operator surfaces can trigger directly
- keep the first slice aligned with the current inbox, requests, recovery, policy, and history categories
- preserve explicit operator control over assignment, recovery, and acknowledgement actions
- avoid inventing a separate UI-only coordination contract

Status: Complete

Success:

- richer operator surfaces can act on bounded coordination pressure through the same shared model they use for inspection

#### Phase 46: `v5` Policy-Aware Coordination Action Suggestions

- suggest the most appropriate bounded coordination actions from current pressure
- keep the first slice aligned with existing inbox, recovery, policy, and history categories
- preserve explicit operator choice instead of silently auto-executing actions
- avoid inventing a second recommendation taxonomy separate from existing coordination policy

Status: Complete

Success:

- richer operator surfaces can move from bounded coordination pressure to the right bounded action faster without losing operator control

### Milestone 6: Start `v6`

#### Phase 47: `v6` Gap-Finding Research Loop Foundations

- add a research loop that can inspect the repo, planning artifacts, eval outputs, and recent work to detect the highest-value memory and coordination gaps
- keep the first slice focused on finding and prioritizing gaps, not auto-editing code yet
- preserve explicit bounded research outputs instead of freeform narrative reports
- make the loop aware of real `memd` product goals such as hot-path memory quality, epistemic retrieval, and coworking safety

Status: Planned

Success:

- `memd` can identify its own highest-value quality gaps from live project evidence instead of relying only on manual triage

#### Phase 47.1: `v6` Native Dream and Autodream Foundations

- move dream and autodream from wrapper-only behavior into native `memd` lifecycle concepts
- add first-class consolidation queues, accepted-signal intake, and durable promotion handoff points
- keep the first slice focused on subsystem boundaries and data flow, not UI polish
- preserve compatibility with skills, CLI, MCP, and future app surfaces as thin entrypoints

Status: Planned

Success:

- dream and autodream become native `memd` capabilities instead of external-only orchestration

#### Phase 47.2: `v6` Token and Context Observability

- add a local session/transcript audit path that attributes context footprint by source class
- detect cache cliffs, idle-gap rebuild risk, redundant rereads, and high-bloat shell output patterns
- keep the first slice focused on observability and operator warnings, not provider-specific hacks
- preserve portability so the same audit model can work across Codex, Claude Code, OpenCode, and similar harnesses
- make the first operator-facing outputs actionable:
  - compact instead of continue
  - fork a fresh session with bundle resume
  - prefer compiled knowledge artifacts over raw rereads
  - suppress repeated same-session reads when evidence is still fresh

Status: Planned

Success:

- operators can see why token budget is being burned before limits hit, and `memd` can use that evidence in later self-improvement loops

#### Phase 47.3: `v6` Raw-to-Graph Compilation for Knowledge Workspaces

- add a compiled-knowledge path that transforms raw folders into reusable entity/relationship/evidence artifacts
- query compiled graph/wiki outputs before rereading raw files when the compiled lane is fresh enough
- preserve provenance classes on graph edges:
  - extracted
  - inferred
  - ambiguous
- keep the first slice filesystem-first so Obsidian-only setups benefit before semantic backends are required

Status: Planned

Success:

- medium-scale research workspaces stop paying repeated cold-read costs because `memd` can answer from compiled knowledge artifacts first

#### Phase 47.35: `v6` Large-Context Workflow Compression

- add a workflow for legitimately large-context jobs such as books, long reports, and large migration corpora
- split these jobs into compact reusable layers:
  - global brief
  - glossary / terminology memory
  - entity and reference sheets
  - chunk-local working windows
  - reconciliation / harmonization passes
- keep the first slice focused on preserving global coherence while avoiding giant-context usage on every intermediate turn
- preserve provenance and cross-chunk traceability so later review can justify translation or synthesis choices

Status: Planned

Success:

- `memd` can make short and medium sessions competitive with huge sessions for most long-form work while still supporting deliberate broad-context passes when they truly add value

#### Phase 47.4: `v6` Universal Design Memory

- add a typed design-memory lane for reusable design-system artifacts instead of treating design guidance as ad hoc prompt text
- support `DESIGN.md`-style artifacts with:
  - visual theme
  - color roles
  - typography hierarchy
  - component constraints
  - responsive rules
  - anti-slop / anti-pattern guidance
- preserve harness-aware metadata so frontend guidance can record which agents or shells are native, portable, or adapter-required
- keep the first slice focused on storage, retrieval, and inspectability before automated design extraction

Status: Planned

Success:

- UI and product design guidance becomes reusable memory that can move across sessions and harnesses without being re-explained every time

#### Phase 48: `v6` Scenario Harness for Memory and Coordination

- add stable scenario benches for resume, handoff, workspace retrieval, stale-session recovery, and coworking flows
- keep the first slice built around real product workflows instead of synthetic toy prompts
- preserve reproducibility so experiments can compare baseline against candidate behavior
- make scenario outputs compact enough to feed nightly research loops

Status: Planned

Success:

- self-improvement has stable, replayable targets that reflect real `memd` workflows

#### Phase 49: `v6` Composite Scoring and Acceptance Gates

- combine hard correctness checks with scenario scores for memory quality, coordination quality, latency, and bloat
- keep the first slice conservative so regressions fail fast
- preserve explicit weighting instead of hidden judgment
- make acceptance criteria clear enough for automated experiment loops

Status: Planned

Success:

- `memd` can judge whether an experiment actually improved the product instead of only compiling and passing tests

#### Phase 50: `v6` Bounded Experiment Runner and Learning Consolidation

- add a bounded experiment runner that works on temporary branches or reversible patches
- accept only experiments that clear the composite gates and discard regressions automatically
- log accepted and rejected experiments in a compact research trail
- consolidate accepted learnings into durable project memory and autodream inputs
- make the handoff explicit: autoresearch produces accepted findings, then autodream consolidates only those accepted findings

Status: Planned

Success:

- `memd` can improve itself through measured overnight loops without unsafe drift or silent truth changes

#### Phase 51: `v6` Harness-Aware Learning Registry

- add a registry for promoted skills, CLIs, harnesses, and procedures that records harness-specific strengths, weaknesses, compatibility, and portability class
- keep the first slice focused on metadata and promotion evidence, not fully automatic cross-harness execution yet
- preserve portability by keeping learning in the substrate while still allowing harness-native and adapter-required abstractions
- make rollback and deprecation explicit when a learned abstraction stops fitting a given harness

Status: Planned

Success:

- learned abstractions can move across agents and harnesses without pretending every shell exposes the same native capabilities

---
*Last updated: 2026-04-05 after shipping `v5` coordination receipts*
