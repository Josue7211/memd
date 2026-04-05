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

---
*Created: 2026-04-04 during GSD brownfield initialization*
