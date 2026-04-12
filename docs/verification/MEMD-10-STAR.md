> Secondary/reference doc. For current project truth start with [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]].

# memd 10-Star Target

## Product Promise

`memd` is not just a memory database. At the 10-star bar, it is a memory
system and memory OS for agents:

- the agent reads memory once, then stays synced to a live backend while work
  changes
- it stores durable truth
- it retrieves the right memory on the hot path
- it lets corrections replace stale beliefs
- it changes later behavior because recall is trusted and timely
- it coordinates multiple agents and humans without flattening scope, privacy,
  or provenance
- it improves itself without regressing core recall

If `memd` cannot reliably do those things, it does not deserve the product
claim.

The live-memory contract is:

1. startup and task-entry read from the wake-up surface first
2. active work streams updates into the backend continuously
3. later recall comes from memory and compiled knowledge first
4. repo rereads are for fresh evidence, not for rebuilding identity or project truth every session

## Non-Negotiable Guarantees

The finished system must satisfy all of these:

1. Important memories are not merely stored; they are recallable under real task pressure.
2. User corrections and stronger evidence can replace stale beliefs durably.
3. Recalled memory can be inspected, explained, and traced back to evidence.
4. Shared memory never silently leaks, collides, or overwrites across agents, providers, or scopes.
5. Every major product claim has rerunnable proof, not planning optimism.
6. Self-improvement loops are subordinate to memory correctness and cannot quietly degrade it.
7. Live memory updates while the agent works; the system does not depend on “remember later” discipline.

## 10-Star Pillars

### 1. Core Memory Correctness

#### Current Reality

- durable storage exists
- hot-path retrieval has been fragile under synced/session noise
- some key regressions now exist, but the end-to-end contract is still not fully proven

#### 10-Star Behavior

- a user can state a fact once and the system reliably recalls it later when relevant
- storage, ranking, context assembly, working memory, and rendering act as one closed loop
- no important memory is lost because of arbitrary bucket order, prompt compression, or transient session noise

#### Key Gaps

- end-to-end proof is still incomplete
- recall is still too vulnerable to competing fresh-but-weaker state
- the user-visible contract is stronger than the tested contract

#### Proof Required

- store -> resume -> behavior-change regression tests
- adversarial noise tests under current-task pressure
- multi-turn scenario proof that the right fact survives and reappears without rereading source files

### 2. Correction And Belief Revision

#### Current Reality

- low-level supersede mechanics exist
- correction UX is weak and normal use does not reliably drive the lifecycle
- stale-belief suppression is not yet proven across realistic flows

#### 10-Star Behavior

- a user correction becomes durable memory immediately
- stale beliefs lose automatically when stronger corrective evidence arrives
- the system can show what changed, why it changed, and what it superseded

#### Key Gaps

- no first-class correction flow yet
- too much depends on manual repair semantics
- corrected truth is not yet proven to dominate later reasoning consistently

#### Proof Required

- correction E2E tests from user statement to later recall
- stale-vs-corrected belief precedence tests
- scenario tests where corrected truth changes later answers and task choices

### 3. Behavior-Changing Recall

#### Current Reality

- memory can be stored and sometimes surfaced
- behavior change is still mostly inferred, not proven
- prompt output remains compact enough that right-memory omission is still a risk

#### 10-Star Behavior

- retrieved memory measurably changes later agent behavior
- recall affects decisions, not just inspection views
- the system can show when a memory influenced a later action or answer

#### Key Gaps

- weak proof that recall changes outcomes
- limited observability around memory influence
- prompt compaction can still hide important records

#### Proof Required

- scenario harnesses where presence/absence of memory changes decisions
- explicit influence tracing in explain/inspection surfaces
- regression checks that detect silent recall-without-impact failures

### 4. Working-Memory Control

#### Current Reality

- working-memory policy surfaces exist
- some budget and rehydration behavior is implemented
- the true quality bar under pressure is still unverified

#### 10-Star Behavior

- working memory is small, explainable, and high-signal
- admission, eviction, rehydration, and trust-floor behavior are coherent
- the system stays useful under overload instead of degenerating into noise

#### Key Gaps

- budget behavior is not fully audited under mixed memory classes
- rehydration quality is not yet proven in realistic task recovery
- policy surfaces may be ahead of product truth

#### Proof Required

- over-capacity adversarial tests
- rehydration scenario tests after eviction
- cross-client consistency checks for working-memory output

### 5. Provenance And Explainability

#### Current Reality

- explain and provenance surfaces exist
- enough inspection exists to debug some failures
- source drilldown is still not proven strong enough for the full trust claim

#### 10-Star Behavior

- every important durable belief has inspectable source, freshness, trust, and verification state
- operators can see why a memory ranked, why it lost, and what evidence supports it
- the system never asks users to trust opaque retrieval

#### Key Gaps

- provenance drilldown is still likely partial
- some explainability may reflect metadata more than true decision causality
- inspection quality is uneven across surfaces

#### Proof Required

- evidence-trace audits from summary memory back to source artifacts
- ranking-explain tests for both winning and losing memories
- operator-debugging flows that can root-cause bad retrieval without code reading

### 6. Shared And Federated Memory

#### Current Reality

- workspace and visibility fields exist
- workspace-aware retrieval has some targeted coverage
- end-to-end shared-memory trust is still unverified

#### 10-Star Behavior

- many agents and humans can share memory safely
- private, workspace, org, and project scopes remain explicit and enforceable
- shared memory improves collaboration without flattening context

#### Key Gaps

- shared retrieval is not yet fully proven in real multi-client flows
- visibility enforcement is not yet audited at the product bar
- federated trust boundaries are still partially aspirational

#### Proof Required

- workspace/shared-memory E2E tests across clients
- visibility-boundary adversarial tests
- multi-agent collaboration scenarios with both local and shared truth present

### 7. Cross-Harness Portability

#### Current Reality

- attach flows and bundle defaults exist
- multiple harnesses are named as supported
- parity across those harnesses is not yet proven

#### 10-Star Behavior

- Codex, Claude Code, OpenClaw, OpenCode, and future clients can use one memory substrate without divergent semantics
- attach/import is low-friction and reliable
- memory behavior does not depend on which harness happened to be used

#### Key Gaps

- cross-harness audits remain sparse
- attach parity is under-tested
- some features may work in one client surface and degrade in another

#### Proof Required

- cross-harness audit suite for resume, handoff, correction, and shared-state flows
- bundle-overlay tests across clients
- parity reporting that flags harness-specific regressions immediately

### 8. Compiled Knowledge And Evidence Workspaces

#### Current Reality

- Obsidian and compiled evidence lanes are present in the vision and partially in the implementation surface
- raw-to-compiled knowledge workflows exist in planning truth more than proven product truth

#### 10-Star Behavior

- the system prefers fresh compiled knowledge over cold rereads when appropriate
- wiki/Obsidian/evidence workspaces are first-class retrieval lanes
- summaries are reversible and linked to source evidence, not disconnected note fragments

#### Key Gaps

- compiled knowledge behavior is not yet deeply audited
- precedence between compiled and raw sources is not yet proven
- evidence-workspace quality may still be stronger on paper than in runtime

#### Proof Required

- compiled-vs-raw retrieval scenario tests
- evidence recovery tests from summary back to workspace/source artifacts
- freshness and precedence tests for compiled knowledge lanes

### 9. Capability Contracts And Runtime Safety

#### Current Reality

- this is called out as the next architectural gap
- the repo has policy and coordination machinery, but a hard capability/safety boundary is not yet the product truth

#### 10-Star Behavior

- every harness and tool path advertises what it can actually do
- the system knows what is runnable, safe, local, shared, or forbidden
- normal memory operations cannot interfere with runtime correctness or cross-agent safety

#### Key Gaps

- capability-contract registry is not yet first-class shipped behavior
- runtime safety boundaries are still too soft
- memory and execution layers are not yet separated by a strong enough contract

#### Proof Required

- capability discovery and enforcement tests
- negative tests proving forbidden or unsupported actions are blocked cleanly
- scenario tests where multiple agents coordinate without stepping on each other

### 10. Self-Improvement Without Regression

#### Current Reality

- research loops and autodream direction exist
- loop orchestration is already a major part of the repo narrative
- the risk is that optimization machinery outruns product-truth validation

#### 10-Star Behavior

- `memd` can improve retrieval, compaction, and coordination quality automatically
- improvement loops are gated by hard memory-quality regressions
- accepted improvements become durable procedures without corrupting core recall

#### Key Gaps

- the regression gate around core memory truth is not yet strong enough
- token-efficiency optimization can still get ahead of correctness
- loop success needs stronger linkage to real feature audits

#### Proof Required

- pre/post loop regression sweeps against core memory features
- explicit stop conditions on recall degradation
- accepted-loop artifacts linked to feature-level audit evidence

### 11. Operator UX, Audits, And Observability

#### Current Reality

- debugging is possible but too code-aware
- audit docs now exist
- a lot of product truth still depends on expert investigation

#### 10-Star Behavior

- operators can tell what works, what is broken, and why without spelunking the codebase
- milestone truth and feature truth are always visible and rerunnable
- debugging bad memory behavior feels like operating a system, not reverse-engineering a mystery

#### Key Gaps

- the new verification system is seeded, not fully operationalized
- more scenario harnesses and audit automation are needed
- observability around “why this memory influenced behavior” is still immature

#### Proof Required

- milestone audits that can be rerun after changes
- operator runbooks tied to real commands and scenario suites
- dashboards or reports that surface regressions before users do

## Dependency Order

The 10-star version should not be built in random order.

Priority stack:

1. Core memory correctness
2. Correction and belief revision
3. Behavior-changing recall
4. Working-memory control
5. Provenance and explainability
6. Shared and federated memory
7. Cross-harness portability
8. Compiled knowledge and evidence workspaces
9. Capability contracts and runtime safety
10. Self-improvement without regression
11. Operator UX, audits, and observability

Reason:

- if core correctness and correction are weak, everything above them is built on false memory
- if behavior-changing recall is not proven, `memd` is still a database, not a memory OS
- if shared and cross-harness layers are weak, the product cannot safely scale beyond one local operator
- if self-improvement outruns truth, the system will optimize the wrong thing

## Mapping Back To Current Verification Work

This document does not replace the current verification registry.

- [FEATURES.md](./FEATURES.md) is the current auditable contract surface
- [RUNBOOK.md](./RUNBOOK.md) defines how to rerun audits
- milestone files under [milestones](./milestones) track product truth by version

Relationship:

- `FEATURES.md` answers: what exists now, and how do we verify it?
- `MEMD-10-STAR.md` answers: what must the whole system become to deserve the product claim?

## Bottom Line

The best possible version of `memd` is not “a project with many memory features.”

It is a system where:

- important things are remembered
- corrections become truth
- recalled truth changes behavior
- evidence is inspectable
- many agents can share memory safely
- self-improvement never outruns correctness
- every claim is backed by rerunnable proof

That is the standard the rest of the repo should be measured against.
