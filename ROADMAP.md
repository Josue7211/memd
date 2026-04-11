# memd Roadmap

## Vision

`memd` is a universal memory substrate for agents and agent-powered applications.

Priority 1 is Codex memory: if Codex cannot persist, retrieve, and inspect its
own state across sessions, the system is not doing its job.

The top-line product contract is seamless live memory:

- the agent reads memory once on startup or task entry
- the live backend becomes the canonical source of active truth while work
  continues
- edits, decisions, blockers, and corrections stream into memory as the agent
  works
- later recall comes from memory and the knowledge base first, not from
  repeatedly reconstructing context from repo rereads or transcript bloat

It should solve memory as infrastructure:

- token-efficient delivery is the first constraint
- local working memory
- shared short-term state
- cross-project long-term knowledge
- durable retrieval with evidence
- lifecycle, dedupe, freshness, and contradiction handling
- brain-inspired primitives:
  - attention
  - salience
  - working memory
  - episodic memory
  - semantic memory
  - procedural memory
  - associative recall
  - object permanence
  - contextual validity
  - rehearsal
  - forgetting
  - self-model
  - social/source memory

The target is not a feature. The target is an open-source platform.

## Product Standard

`memd` only wins if it feels like real memory instead of memory tooling.

The product standard is:

- zero-friction memory
- seamless live memory
- epistemic memory
- short-term-first memory
- global-first memory with project overlays
- universal capability memory
- native multi-agent interoperability
- inspectable memory
- token-efficient memory by default

In practical terms:

- wake-up should feel automatic
- the agent should read once and then stay synced live while it keeps working
- recall should come from memory and the knowledge base before expensive repo
  rereads
- resume should be a compact working-memory view, not the center of the product
- `memd` should behave like a global memory add-on before it behaves like a per-repo tool
- short-term state should stay sharp without transcript bloat
- short-term state must sync quickly across machines and harnesses when active work changes
- infrastructure hives like `memd`, `claw-control`, `agent-shell`, and `agent-secrets` should publish as first-class service hives so product sessions can route help to the right active specialist instead of guessing
- `memd init` and `memd refresh` should discover enabled plugins, skills, commands, hooks, agents, and team surfaces across configured harnesses instead of only reading markdown files
- once a capability is discovered, `memd` should classify it as universal, harness-native, bridgeable, or blocked and publish that capability map as shared memory
- bridgeable capabilities should spread automatically to other harnesses through generated bridge surfaces or explicit install guidance, so the system behaves like controlled self-propagation instead of manual per-harness setup
- cross-project state should sync through scoped live lanes instead of flattening every repo into one pool
- verified, inferred, claimed, stale, and contested memory should stay explicit
- Codex, Claude Code, OpenClaw, and OpenCode should switch over one substrate without collisions
- users should be able to inspect what the system remembers, why, and what changed
- the default path should avoid expensive rereads, oversized reinjection, and stale-session rebuild waste
- when legitimately huge context is required, the system should make those passes deliberate, structured, and higher-yield instead of paying giant-context cost on every turn

## Current Status

`memd` is no longer a simple phase-by-phase project. It is becoming the
agent's own memory substrate.

The first practical consumer is Codex, so the memory stack should be optimized
for Codex continuity before broader integrations get the same polish.

The roadmap now starts with repository foundations so the project can be
worked on cleanly in public before the memory stack keeps expanding.

The right way to track progress now is by capability versions:

- `v0`: OSS-ready project foundations
- `v1`: human-inspired memory OS
- `v2`: superhuman AI brain
- `v3`: federated and collective memory
- `v4`: self-optimizing memory
- `v5`: memory-native cognition infrastructure

Current version state:

- `v0`: complete
- `v1`: in progress
- `v2`: in progress
- `v3`: in progress
- `v4`: in progress
- `v5`: in progress

These states are product-truth states, not "some phases landed in code" states.
Nothing past `v0` should be treated as complete until the end-to-end user
behavior is verified under real multi-session and multi-harness use.

What is already real in the repo:

- the core works without RAG
- LightRAG is the intended long-term semantic backend
- Obsidian vault ingest already exists as a filesystem-first source lane
- Obsidian compiled evidence pages now exist as a first-class workspace lane
- project bundles make the long-term path configurable
- clients attach through the same control plane
- a first MCP-native hive coordination bridge now sits on top of the brokered backend substrate
- graph/entity primitives exist
- salience, rehearsal, decay, and consolidation exist
- explain, inbox, maintenance, and policy inspection exist
- branchable belief lanes exist for competing durable memory records
- `memd gap` now emits evidence-driven improvement candidates from eval, resume, and coordination lanes

What is still not good enough yet:

- the live event spine is still too weak, so important active-state changes do
  not persist aggressively enough while the agent works
- wake-up, recall, capture, promotion, and correction still need to function as
  one memory loop instead of adjacent commands
- the bootstrap source registry still needs delta-refresh wiring so changed local files can be reimported without rereading unchanged ones
- runtime adoption still needs to reach every major agent surface beyond the core bootstrap hooks
- hive groups, shared-memory recall, and cross-session continuity are still below the product bar
- the operator-facing status and summary views can still expose the layered memory model more clearly
- capability discovery still needs to inventory plugin-backed workflows, not just memory files and a curated skill list
- bridge generation still needs to turn discovered portable capabilities into actual installed surfaces across harnesses instead of leaving them trapped in one plugin system
- corrections from the user still need to consolidate into durable operating policy so `memd`, dream, and autoresearch stop repeating the same deployment or workflow mistakes

## Deployment Tiers

The memory stack should scale in layers instead of forcing the heaviest setup
from day one.

### Tier 1: Obsidian Only

- raw source material
- compiled entity/relationship wiki artifacts
- markdown wiki pages
- compiled output pages
- direct file browsing and backlinks
- no semantic backend required

### Tier 2: Shared Sync

- everything in Obsidian-only
- shared vault sync
- shared workspace lanes
- private/workspace/public visibility
- resumable handoff bundles across agents and humans

### Tier 3: LightRAG

- everything in shared sync
- semantic retrieval
- concept graph traversal
- long-range relatedness
- multimodal retrieval at larger scale

What is still missing before `v1` is truly complete:

- deeper repair tooling
- provenance drilldown from summaries to raw artifacts
- stricter working-memory admission and eviction behavior
- more explicit episodic, procedural, self-model, and source-trust layers

## Product Shape

`memd` is the control plane.

Backends and producers work behind it:

- local memory
- short-term sync
- auto-dream / consolidation
- Obsidian vault and compiled markdown wiki workflows
- compiled knowledge graphs and reusable intermediate artifacts
- semantic retrieval
- graph relationships
- verification workers
- entity permanence and contextual validity
- salience and rehearsal
- attention and relevance gating
- episodic traces and timeline recall
- associative retrieval
- adaptive forgetting and decay

Clients consume it through one API:

- Codex
- Claude Code
- Mission Control
- OpenClaw
- generic HTTP/CLI users

Capability surfaces should also be first-class:

- skills
- plugins
- commands
- hooks
- agents
- teams
- harness-native tools
- bridge/install instructions

Artifact classes should also be first-class:

- operational memory
- evidence memory
- compiled knowledge/wiki artifacts
- graph/entity artifacts
- design memory:
  - design systems
  - visual rules
  - component constraints
  - anti-slop guidance
  - harness-specific UI strengths and weaknesses

## Versions

### v0: OSS-Ready Project Foundations

Goal:

- move the repository onto a real branch strategy before the larger memory
  stack keeps growing

Deliver:

- a dedicated working branch strategy for active development and release cuts
- file splitting where it improves reuse and maintenance
- version history conventions for phased work and releases
- branch flow for development and future public contributions
- contribution rules, review expectations, and security guidance
- public project documentation that is separate from internal planning

Success:

- active work happens on a branch by default
- new contributors can find the rules, the branches, and the release shape
- the repo stops depending on oral context for basic collaboration
- large files are split only where the seam is real, not to chase line counts

### v1: Human-Inspired Memory OS

Goal:

- build a durable, brain-inspired memory substrate that already beats naive
  chat history and basic RAG

Deliver:

- architecture, schema, retrieval policy, and promotion policy
- Rust core with SQLite-backed local mode
- global install / global bundle behavior so agents can use `memd` as a system memory layer before repo-local overlays are added
- compact retrieval and budgeted context delivery
- shared short-term sync
- dream/candidate ingestion and promotion gates
- Obsidian vault bridge for markdown-native ingest and writeback
- compiled-wiki workflow support where raw sources and derived notes can live in the same knowledge workspace
- compiled graph workflow support where raw evidence can be transformed once into reusable entity/relationship artifacts
- long-document and large-context workflow support:
  - global brief/spec extraction
  - glossary and terminology memory
  - entity/reference sheets
  - chunk-local working windows
  - cross-chunk consistency memory
  - final reconciliation passes that use broad context only when needed
- LightRAG adapter and bundle-first backend configuration
- tiered deployment shape:
  - Obsidian-only
  - shared sync
  - LightRAG-augmented retrieval
- backend stack contract for `rag-sidecar`, `MinerU`, and `RAGAnything`
- one-command attach flow for Claude Code, Codex, Mission Control, and OpenClaw
- freshness, contradiction surfacing, inbox, explain, and maintenance views
- graph/entity primitives, salience, rehearsal, decay, and consolidation
- provenance-typed graph edges for compiled knowledge:
  - extracted
  - inferred
  - ambiguous
- design memory as a typed artifact lane:
  - `DESIGN.md`-style specs
  - reusable visual system prompts and constraints
  - harness-aware design guidance such as frontend strengths, weaknesses, and portability
- working memory, timeline traces, and policy inspection

Success:

- agents can carry context across sessions and machines without drowning in
  transcripts
- `memd` works as a global memory substrate first, with project memory layered on top instead of replacing the global lane
- memory remains compact, typed, inspectable, and evidence-backed
- markdown-native research and wiki workflows can stay in Obsidian without giving up typed memory, provenance, or agent automation
- the same knowledge base can scale from solo file-native use to shared sync to semantic retrieval
- repeated raw rereads can be replaced by cheaper compiled graph/wiki retrieval at small and medium scales
- reusable design guidance can be recalled like memory instead of re-explained in every UI task
- long-form work such as books, corpora, or large migrations can preserve coherence without paying maximum context cost on every intermediate step
- the substrate behaves like a brain-inspired control plane, not a bag of notes

Implementation history:

- Phase 0: specs
- Phase 1: Rust core
- Phase 2: retrieval layer
- Phase 3: short-term sync
- Phase 4: dream pipeline
- Phase 5: long-term memory backend
- Phase 5.1: RAG adapter hardening
- Phase 5.2: backend stack contract
- Phase 6.1: agent attach automation
- Phase 7.1: memory quality enforcement
- Phase 8.1: graph and learning
- Phase 8.2: human-like memory model
- Phase 8.3: brain-inspired memory stack
- Phase 8.4: memory operations and explainability

Current gaps:

- repair actions are still shallow
- provenance drilldown is not deep enough
- working-memory admission and eviction policy is not explicit enough
- procedural, self-model, and source-trust layers are still incomplete

### v2: Superhuman AI Brain

Goal:

- stop copying biological limits and turn memory into a machine-advantaged
  reasoning substrate

Deliver:

- branchable world models for unresolved contradictions and competing beliefs
- reversible compression with summary-first retrieval and raw evidence recovery
- provenance-native memory where every durable belief carries source,
  freshness, trust, and verification state
- compiled wiki material and Obsidian evidence pages can be treated as first-class evidence lanes, not only as loose note text
- raw folders can be compiled into reusable graph/wiki intermediates so the system queries compiled knowledge before cold-reading source files again
- explicit working-memory admission, eviction, and rehydration policy
- retrieval as a learned control loop instead of a fixed heuristic table
- trust-weighted source memory across humans, agents, tools, files, and sensors
- parallel recall of multiple candidate explanations instead of one-thread
  fetch
- explicit uncertainty handling so low-confidence memories do not masquerade as
  truth

Success:

- `memd` uses biology for structure without inheriting biological bottlenecks
- the system can remember far more than it keeps hot without becoming incoherent
- contradictions remain navigable instead of being flattened away
- Obsidian-scale knowledge bases can stay markdown-native at small/medium scale, with semantic backends added only when retrieval pressure demands it
- retrieval becomes part of cognition instead of an accessory to storage

### v3: Federated and Collective Memory

Goal:

- let many agents and humans share memory without destroying scope, privacy, or
  trust

Deliver:

- shared workspace and org-level namespaces
- permission-aware memory visibility
- trust tiers for source and agent provenance
- handoff memory for delegation across agents and humans
- shared sync as a first-class deployment tier
- cross-project / initiative live memory lanes for related repos and workstreams
- real-time canonical short-term sync on `memd-server` for:
  - focus
  - blockers
  - next recovery step
  - branch and claimed scope
  - ports and base URLs
  - heartbeat and presence
  - help, review, and handoff alerts
- private/local memory boundaries that do not leak into shared state
- explicit merge and divergence handling between local and shared truth
- provider collision controls:
  - stable session identity
  - source-aware writes
  - claim / lease enforcement
  - contested-memory handling instead of silent overwrite

Success:

- teams can share useful memory without flattening private and public context
- handoffs preserve reasoning state instead of forcing re-derivation
- organizational memory stays scoped, attributable, and auditable
- if one agent changes active code state on one machine, another agent on another machine can see the relevant short-term update fast enough to cowork safely
- changes that affect sibling projects can propagate through a cross-project lane without polluting unrelated project memory
- different providers can share one memory substrate without silently overwriting each other’s truth

### v4: Self-Optimizing Memory

Goal:

- make the memory system improve its own policies from evidence and outcomes
- make dream, autodream, and autoresearch native parts of the memory lifecycle rather than external wrappers

Deliver:

- evaluation harnesses for routing, retrieval, promotion, and repair quality
- adaptive policy tuning from usage feedback
- automatic rehearsal, decay, and consolidation scheduling
- native dream and autodream subsystems for:
  - consolidation queue management
  - promotion candidate handling
  - pruning and decay
  - accepted-research intake
- budget enforcement for token and storage growth
- A/B routing for competing memory strategies
- regression detection for memory quality, not just system health
- evolution engine that turns repeated workflows into monitored, reusable skills, CLIs, tools, and other promotable abstractions
- tier-aware policy evolution so Obsidian-only, shared sync, and LightRAG setups can tune differently
- harness-aware promotion so learned skills, CLIs, and harnesses carry compatibility, strengths, weaknesses, and portability class across agents
- harness coverage enforcement so promoted skills that are not mapped for a given harness are marked adapter-required and paired with a bridge plan instead of being treated as universal
- automatic short-term memory management:
  - capture meaningful task-state changes without transcript dumping
  - keep the hot lane fresh with minimal manual effort
  - let dream and autodream consolidate high-signal short-term memory into durable memory
- operator-facing hot-lane inspectability:
  - show what changed since last resume
  - keep current focus, pressure, and next recovery visible across prompt, bundle, and status surfaces
- operator-facing token efficiency observability:
  - attribute context footprint by source:
    - system/tool surface
    - memory injection
    - hook payloads
    - file reads
    - shell output
  - detect redundant rereads and repeated high-bloat command patterns
  - surface cache-cliff and idle-gap risk before an expensive resumed turn
- anti-waste control policies:
  - prefer hot-lane and compiled artifacts before cold raw rereads
  - suppress repeated same-session file rereads when prior evidence is still valid
  - budget context injection per surface
  - recommend fresh-session handoff when stale-session continuation would be more expensive than a compact resume
  - optimize legitimate large-context jobs by splitting them into:
    - durable global state
    - compact local windows
    - explicit reconciliation passes
- universal design memory evolution:
  - learn reusable UI/system specs from stable project outputs
  - promote accepted design memory into portable design artifacts
  - keep harness-native and adapter-required design guidance explicit

Evolution Engine:

- capture repeated traces, commands, and repair loops from real work
- mine recurring workflows that are expensive, fragile, or over-reasoned
- promote stable patterns into skills, wrappers, dedicated CLIs, or other tools
- record which harnesses and agents each promoted abstraction helps, hurts, only partially fits, or is native to
- keep lineage, quality metrics, and rollback history for every promoted abstraction
- share proven improvements across agents when the scope is safe
- retire or downgrade abstractions that stop paying for themselves
- require measured gains in success rate, token cost, or cycle time before promotion

Success:

- memory policy improves from usage instead of hand tuning
- regressions are detectable before they become user pain
- the system can self-correct under load and over time
- repeated work gets cheaper because the system learns and promotes the right abstractions
- the memory loop feels alive: capture, resume, inspect, consolidate, repeat
- dream/autodream/autoresearch live inside `memd`, with skills, CLI, MCP, and UI acting as surfaces over the same subsystem
- token optimization is a competitive advantage of the substrate, not an accidental side effect
- no promoted skill, CLI, or procedure is allowed to masquerade as universal when a harness mapping is missing; missing mappings are surfaced as adapter-required work

### v5: Memory-Native Cognition Infrastructure

Goal:

- make memory an active substrate for reasoning, planning, and long-horizon
  agent identity

Deliver:

- memory-backed long-horizon planning loops
- branch-aware execution state tied to world-model memory
- reflective self-model memory for strengths, weaknesses, and strategy
- durable goal and subgoal memory with continuity across sessions
- memory-aware tool selection and verification planning
- simulation and replay support for counterfactual reasoning
- skill invocation as part of planning, not just a post-hoc helper lookup
- first-class inspectability surfaces for:
  - knowledge workspace
  - memory systems view
  - eventual brain-view telemetry hooks for `braind`
- provenance-native cognition behavior where verified evidence outranks narrative continuity

Success:

- the agent does not just store context; it thinks through memory
- identity, goals, plans, and evidence remain coherent over long horizons
- `memd` becomes part of the cognition stack, not just a support service
- cognition can select, compose, and learn skills as part of normal reasoning
- the system becomes less confidently wrong, not just less forgetful

### v6: Measured Self-Improvement

Goal:

- let `memd` improve itself through bounded experiments that are scored,
  reversible, and evidence-backed

Deliver:

- scenario harnesses for real memory workflows:
  - resume after pause
  - cross-agent handoff
  - shared-project coworking
  - stale-session recovery
  - verified evidence outranking synthetic continuity
- a composite scorer that combines:
  - hard correctness gates
  - short-term memory quality
  - coordination quality
  - operator friction
  - latency and bloat
- an experiment runner that:
  - works on temp branches
  - measures baseline vs candidate
  - accepts only winning changes
  - discards regressions automatically
- tiered experiment safety:
  - safe auto-accept changes
  - eval-gated behavioral changes
  - human-review-only trust/provenance changes
- a promotion registry for learned abstractions that stores:
  - strengths
  - weaknesses
  - portability class:
    - portable
    - harness-native
    - adapter-required
  - compatible harnesses
  - risky harnesses
  - promotion evidence
  - rollback history
- accepted-learning consolidation into durable project memory and autodream inputs
- explicit autoresearch -> autodream flow:
  - autoresearch runs first to find gaps and test bounded improvements
  - autodream runs after to consolidate accepted learnings, not speculative failures
- difficulty-aware overnight evolution queue:
  - collect failures, correction memories, verifier regressions, and benchmark misses as candidate improvement inputs
  - classify candidates by difficulty so easy fixes can land as policy/docs/prompts, medium fixes can tune tools/config, and hard fixes can escalate into isolated code-evolution proposals
  - scale budgets, retries, isolation, and required evidence by difficulty instead of treating every overnight improvement attempt the same
  - allow autoresearch to mine recurring failures and memory artifacts for bounded improvement loops rather than relying only on hand-authored loop selection
- token and context observability for agent sessions:
  - transcript/session import into a compact local audit store
  - cost and bloat attribution by turn, workflow, tool class, and memory payload class
  - cache-cliff, idle-gap, and resume-rebuild warnings
  - repeated read/output waste detection
  - hooks that can warn, compact, fork, or recommend fresh-session resume when context cost is about to spike

Success:

- `memd` learns by winning measured experiments, not by silently changing truth
- repeated quality improvements become cheaper and more reliable over time
- the system can improve hot-path memory and coordination behavior overnight without unsafe drift
- learning lives in the substrate, while promoted abstractions can be marked portable, harness-native, or adapter-required instead of being forced into false universality
- operators can see where token budget is being burned instead of guessing after rate limits hit

Core loop:

- discover the highest-value gap from repo state, planning artifacts, eval outputs, and recent work
- replay stable memory and coordination scenarios
- score baseline vs candidate on correctness, quality, latency, and bloat
- accept only bounded winning changes
- promote only the abstractions whose strengths and weaknesses are understood per harness
- consolidate accepted learnings back into durable project memory
- let autodream compress accepted research outcomes into durable procedures, lessons, and future experiment seeds

### v7: OSS-Ready Project Infrastructure

Goal:

- make the repo easy for other people to understand, branch, review, release,
  and extend without inheriting the current work-in-progress sprawl

Deliver:

- a clean branch and version-history strategy for phased work
- contribution guidelines that reflect the actual engineering workflow
- review, release, and changelog conventions that support outside contributors
- repository rules for scope, file splitting, and when refactors should happen
- documentation that separates public project guidance from internal planning

Success:

- the project can be picked up by someone new without needing oral context
- phased work maps cleanly to branches and versioned releases
- contribution and review expectations are explicit enough to run in the open
- the repo reads like a maintained open-source project, not a private scratchpad

## Immediate Next Steps

1. Add automatic short-term memory management so important state transitions get captured without transcript bloat.
2. Keep the hot lane sharp with better replacement, cleanup, and branch/workspace-aware current-task state.
3. Make epistemic state first-class in retrieval behavior: verified, inferred, claimed, stale, contested.
4. Tighten native multi-agent bridges so switching clients feels like changing terminals, not losing the brain.
5. Make `memd` global-first: default global bundle root, then project and cross-project overlays.
6. Add cross-project live lanes so related repos can sync active state without flattening all memory.
7. Expand inspectability from bundle files into richer workspace and UI surfaces.
8. Add a Codex-native `memd-reload` skill and matching shell shim so existing sessions can bootstrap memory immediately instead of waiting for the next tool use.
9. Make `memd init` seed a project bundle from existing repo docs, planner files, and existing Claude memory when run inside a project so the first project lane is immediately useful.
10. Make the user-facing flow obviously usable: one front door that auto-detects setup vs reload so the user only has to remember `$memd`.
11. Add a bootstrap preflight that detects `AGENTS.md` / `CLAUDE.md` / planning docs, seeds from whatever exists, and does not require a separate Codex `/init` step.
12. Treat repo introspection as first-class memory input: `AGENTS.md`, `CLAUDE.md`, `.planning/*`, `README.md`, `ROADMAP.md`, `docs/*`, lockfiles, config, and git history should all feed a structured project bundle with source and confidence.
13. Record a fingerprinted source registry for imported harness memories so unchanged files stay read-once and only changed sources get re-imported.
13. Add git-aware incremental sync so memd updates memory from diffs and changed files instead of only one-time bootstrap sweeps.
14. Keep global, project, and cross-project memory layered but queryable together, with explicit provenance and confidence on each memory item.
15. Make Codex, Claude, and OpenClaw all bootstrap from memd once, then treat memd as the shared memory source of truth for future sessions.
16. Let dream and autodream consolidate high-signal short-term memory into durable memory after the hot lane is stable, and prune stale or noisy items aggressively.
17. Build measured autoresearch for `memd`: scenario harness, composite scorer, experiment runner, and accepted-learning consolidation.

## Auto Research Loops

`memd` now coordinates eight compact autoresearch loops inspired by the prompt-surface screenshot plan so that optimization work is measurable, reversible, and loop-driven. Each loop is scoped to a single hypothesis, reports a percent-improvement and token-cost delta when it runs, and stops once its metric budget is met so the system can orbit toward the next loop without burning gilt-edge context.

The prioritized loops are:

1. **Prompt Surface Compression** – target: the repeated resume/handoff bundles and handoff text that drive everyday coworking. Metric: chars/tokens saved after compaction, stop condition: no new reductions in two consecutive passes, risk: low.
2. **Live Truth Freshness** – target: re-reading just-changed files, misaligned beliefs, and stale assertions. Metric: reread count and stale-belief occurrences per loop, stop when replays hit baseline freshness and no stale-belief regressions appear, risk: low to medium.
3. **Capability Contract Detection** – target: mismatches between skill wrappers and the missing CLI binaries so that we know automatically when an interface is broken. Metric: wrong-interface failures detected vs overall interactions, stop once all known capability surfaces map to a documented contract, risk: medium.
4. **Event Spine Compaction** – target: noisy event deltas from the integration stream, coordination hooks, and budget telemetry. Metric: token burn per resume due to unfiltered events, stop when compaction yields <5% incremental cost, risk: low.
5. **Correction Learning** – target: repetitive user-corrected mistakes and policy drift in hooked tooling. Metric: correction recurrence rate and newly repeated corrections prevented; stop on zero recurrence for three loops, risk: medium.
6. **Long-Context Avoidance** – target: bursts where the system re-injects massive transcripts instead of leveraging compact state. Metric: average prompt size and long-context spikes per session; stop when the working set stays within its target budget without losing accuracy, risk: low.
7. **Cross-Harness Portability** – target: ensuring memories, skills, and promoted abstractions stay portable across Codex, Claude Code, and other harnesses. Metric: contract coverage and adapter-required warnings; stop when every promoted abstraction has a portability class defined, risk: medium.
8. **Controlled Self-Evolution** – target: because actual self-improvement must accept only validated wins, this loop enforces rollout gates, pod-based evaluations, and percent-improvement telemetry. Metric: accepted-change rate, rollback incidents, and promotion evidence coverage; stop when promotion confidence reaches the threshold defined in the promotion registry, risk: high so run it later once the baseline is stable.

Queued next for this loop:

- difficulty-gated overnight evolution that turns repeated failures, weak benchmark rows, regression artifacts, and high-friction memory traces into bounded improvement attempts
- easy candidates may propose docs, policy, prompt, or skill/CLI promotions directly
- medium candidates may run autoresearch-style experiments against bounded config, retrieval, or workflow surfaces
- hard candidates must stay on isolated evolution branches with stronger evidence, repeatability checks, and review gates before promotion

Autoresearch triggers these loops sequentially and reports the percent improvement/per-loop so operators can understand where token budgets are being saved. Each accepted loop result feeds directly into the evolution engine, autopromotes relevant artifacts, and handsoffs to autodream for consolidation. When loops exhaust their remaining gains, autoresearch either spins a refresh cycle or surfaces a new loop candidate from `memd gap` so the system never settles for stale performance.

See `docs/research-loops.md` for the detailed checklist, stop conditions, and how each loop contributes to the 90% token-reduction goal.
## Non-Goals

- transcript dumping
- vendor lock-in
- using RAG as the only memory layer
- mixing project-local truth with global truth
- letting one provider silently overwrite another provider’s memory
