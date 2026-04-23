---
doc: memd-sota-theory
status: active
opened: 2026-04-22
depends_on: [../verification/MEMD-10-STAR.md, ../verification/0.1.0-CONTRACT.md, ../verification/0.1.0-AXIS-OWNERSHIP.md]
---

# memd SOTA Theory — Best Memory OS for Any Harness

> North-star doc. The roadmap binds to this. Every milestone's scope decisions
> are judged against whether they move memd toward SOTA on the 7 axes defined
> by the 10-STAR scorecard. 0.1.0 ships at SOTA (composite ≥8.0, every axis
> ≥7), not at production floor.

## What "SOTA memory OS for any harness" means

Three claims compressed into one tagline:

1. **Memory OS, not memory service.** memd is a substrate — schema, lanes,
   hooks, compiler, retrieval, provenance, ordering guarantees. Applications
   (harnesses) plug in; memd does not embed into any single application.
2. **For any harness.** claude-code, codex, gemini, cursor, zed, amp, aider,
   custom MCP clients — all get the same memory view with the same semantic
   guarantees. No harness is privileged. No harness is locked out.
3. **SOTA.** On every axis where public benches exist, memd is at or above
   the best published number by a meaningful margin. On axes where no public
   bench exists, memd publishes its own bench and demonstrates the SOTA
   property empirically (recorded traces, reproducible from git).

## Why the existing landscape does not satisfy this

Brief teardown — see `docs/theory/teardowns/` for depth.

- **Mem0**: strong on correction + surface-level recall, weak on procedural,
  no cross-harness story. Provenance surfaces missing. SOTA on LoCoMo token
  F1 but hits a ceiling because the substrate is conversational-only.
- **Supermemory**: strong on ingestion + schema flexibility, weak on
  session_continuity across time and compaction. No routine-detection. Good
  retrieval but not substrate-native — benches are RAG-flat.
- **MemMachine**: strong on correction primitives (similar philosophy to
  memd), weak on procedural + cross-harness. Public benches show ceiling
  around LoCoMo 0.70 F1.
- **Letta** (prev. MemGPT): strong on continuity via tiered memory, weak on
  correction retention + provenance. Session model is single-harness.
- **mempalace**: strong on 4-layer context cap (L0–L3, donor for memd D4),
  weak on everything else — research prototype, not a product.
- **Hermes / Omegon / Smriti**: donor repos — strong on specific primitives
  (ordering, hybrid retrieval, content-hash dedup) that memd has absorbed.

None ship a cross-harness substrate with all 7 axes lifted. That is the
opening memd exploits.

## The 7 axes — what 10/10 and 7/10 mean

The 10-STAR scorecard is zero-generosity. 10/10 is "best-in-class across every
dimension of this axis, with headroom above every competitor." 7/10 is "SOTA
on the dominant dimension, competitive on the rest." 0.1.0 ships when every
axis hits 7; V13 close target is an 8–9 on most axes.

### 1. Session continuity (SC) — weight 20%

**What it measures.** Does session N resume session N-1's in-flight state
with zero loss across time, across compaction events, across harness switches,
and across device switches?

**7/10 (SOTA baseline).** Same-harness same-device same-workspace resumption
is perfect for 90+ turn sessions with one compaction in the middle. Cross-
harness within-device is a small quality step but ≥0.95 fidelity. Cross-
device is handled via sync (not necessarily perfect).

**9/10 (V13 target).** Cross-project continuity (the user works on project A
then B then A again; wake-A correctly re-hydrates A's focus without being
polluted by B). Cross-device sync with CRDT-style merge on conflicts.
Compaction-aware recall (wake produces compressed-but-optimal context, not
truncated).

**10/10 (future).** Multi-month dormant projects resume with no measurable
quality delta vs same-session. Session state can be replayed from cold
start and produces identical behavior turn-for-turn.

**Benchmark.** Substrate-native (memd's own SC suite). Public proxies:
LongMemEval multi-session, LoCoMo 300-turn.

### 2. Correction retention (CR) — weight 15%

**What it measures.** User says "no, X is Y, not Z" — does a future session
use Y, does provenance show the correction, does the wrong value ever leak
back in? Does silent correction (user doesn't explicitly say "no" but their
behavior implies prior answer was wrong) get detected?

**7/10 (SOTA baseline).** Explicit corrections are preserved, change future
behavior, carry provenance, do not regress. Contradiction detection triggers
on direct conflicts with <5 s latency.

**8/10 (V13 target).** Silent correction detection — user repeatedly rephrases
a question or ignores a prior answer, memd infers the prior answer was
wrong and flags/corrects without explicit UI. Multi-hop correction chains
(correction A about B downstream-affects C).

**10/10 (future).** Correction graph across all memory with cryptographic
audit trail; third party can replay corrections from export.

**Benchmark.** Substrate CorrectionPropagation + V7 C7 next-session-behavior
harness. Public proxies: LoCoMo multi-turn subset, Mem0 correction eval.

### 3. Procedural reuse (PR) — weight 15%

**What it measures.** User repeats a workflow — memd detects the repetition,
stores the sequence as a routine, invokes it next time to save tokens + save
user re-description. Does the routine library stay curated or devolve into
noise?

**7/10 (SOTA baseline).** Routines auto-detected from ≥3 observed repetitions,
stored, invoked with user consent, measurable token savings.

**9/10 (V13 target).** Curated routine library — user edits, composes (A+B=C),
shares across workspaces and across users. Routines inherit per-project.
Pruning: stale routines deprecated, replaced, merged.

**10/10 (future).** Cross-user routine economy (discover routines
contributed by other memd users, with trust + provenance). Routine
generalization — memd infers variable bindings and produces parameterized
routines from example traces.

**Benchmark.** No public bench — memd publishes its own Procedural-Reuse
bench. V5 F5 live-fire + V10 C10 self-improvement + V12/V13 library curation.

### 4. Cross-harness (CH) — weight 15%

**What it measures.** Same memory view in claude-code and codex and gemini
and cursor and aider — identical items, identical ordering, identical
isolation boundaries, identical corrections surfaced. Multi-user federation
works with explicit visibility.

**7/10 (SOTA baseline).** Two harnesses (claude-code + codex) flip cleanly
with ≥0.98 parity on retrieval, corrections survive harness switch, no
leak between users within same workspace.

**8/10 (V13 target).** Universal harness protocol — memd speaks MCP, ACP,
and a custom protocol for harnesses that want typed channels. Any harness
plugs in with <100 LOC shim. Live multi-harness session (user runs
claude-code AND codex simultaneously, both see same memory state).

**10/10 (future).** Federation at scale — thousands of users, per-user
isolation + explicit sharing + collision resolution. Harness marketplace
where memd is the default memory backend.

**Benchmark.** G4 cross-harness flip → G9 multi-user adversarial suite →
G12 universal-protocol parity bench.

### 5. Raw retrieval (RR) — weight 15%

**What it measures.** Given a query over the memory store, does the right
item come back with high precision and recall, at depth, with progressive
cost? Does performance match or beat public benches?

**7/10 (SOTA baseline).** memd matches published SOTA on LoCoMo /
LongMemEval / MemBench / ConvoMem within 2pp. Progressive depth works.
FTS + semantic hybrid with learned weights.

**9/10 (V13 target).** Beats published SOTA by ≥5pp on all four public
benches simultaneously. Domain-tuned retrieval (code / docs /
conversational). Personalized per-user query-pattern learning.

**10/10 (future).** Dominates every public bench by ≥10pp margin, publishes
new harder benches that existing systems fail on, generalizes to zero-shot
domains.

**Benchmark.** Public bench battery — LoCoMo (token F1), LongMemEval
(judged accuracy), MemBench (MC accuracy), ConvoMem (accuracy).

### 6. Token efficiency (TE) — weight 10%

**What it measures.** Wake context is compact yet sufficient. Per-turn
compiler decides what to include based on turn intent. Cost is measurable,
tunable, and scales sub-linearly with memory size.

**7/10 (SOTA baseline — V13 target).** Dynamic per-turn compiler decides
per-turn what kinds of memory to include at what depth. Wake median ≤1500
tokens for typical workload. Operator sees $/M ledger and can tune targets.
Shannon-ish baseline (minimal redundancy, every token pulls weight).

**9/10 (future).** Self-tuning compiler — memd learns per-user per-harness
what token budget yields best downstream task quality, adjusts automatically.
Cost scales sub-linearly with memory size via adaptive indexing.

**10/10 (future future).** Information-theoretic optimal — no context token
can be removed without measurable downstream task quality degradation.

**Benchmark.** V4 D4 kinds-coverage + cost ledger → V8 E8 cost UI →
V11 dynamic compiler. Capped at 7/10 for V13 because 9/10 requires
production telemetry from real users which is post-release.

### 7. Trust + provenance (TP) — weight 10%

**What it measures.** Any value in context can be explained — where did it
come from, what corrected it, what are the alternatives, what is its
confidence. Audit trail is complete. Third parties can verify.

**7/10 (SOTA baseline).** Every displayed value answers "source?" and
"correction history?" with ≤2 queries. Explain API works end-to-end.

**8/10 (V12 exit).** Cryptographic provenance — trace is signed, tamper-
evident. Full audit UI (browse corrections, promotions, reads, over time).

**9/10 (V13 target).** Third-party verifiable — export a provenance
snapshot; independent replay reproduces the same answer from the same
memory state. Compliance-grade audit trails.

**10/10 (future).** Zero-knowledge provenance proofs (a user can prove a
correction was applied without revealing the correction content).

**Benchmark.** V7 F7 user-visible surface → V8 D8 provenance browser →
V12 TP work → V13 export + third-party replay harness.

## Why the roadmap ends at V13

V4–V10 deliver production floor (every axis ≥3). V11–V13 deliver SOTA
(every axis ≥7). 0.1.0 tag lands at V13 close. Everything beyond — cross-
user routine economy, zero-knowledge provenance, self-tuning compiler,
cross-device sync at scale — is 0.2.0+ territory, driven by real user
telemetry, not pre-planned at release time.

V13 is the SOTA ceiling that current memd theory can commit to with high
confidence before release. Anything higher requires the dataset of real
usage that only a released product generates.

## What is not in scope for 0.1.0 (even at SOTA)

- Multi-device mobile app (desktop + mac + CLI is enough for 0.1.0)
- Cloud-hosted memd (0.1.0 is self-hosted / local + optional sync)
- Non-English language optimization (English first, i18n is 0.2.x)
- GUI for non-operators (the user is a developer; the UI is CLI +
  `memd configure` + operator surfaces from V8)
- Regulated-industry compliance audits (HIPAA, SOC2) — post-release work
  with real deployment partners

## How this doc is used

- **Roadmap binds here.** Every milestone doc in `docs/verification/milestones/`
  cites this file in `depends_on`. If a milestone's scope drifts, it's
  because it conflicts with the theory — resolve by revising the milestone
  or revising the theory, not by silently diverging.
- **Release contract references this.** `0.1.0-CONTRACT.md` cites this
  file. The per-axis SOTA floor (≥7) is derived from the 7/10 definitions
  above.
- **Scope-creep veto.** If a proposed feature is "neat but doesn't move
  any axis toward SOTA," it's cut. The theory is the filter.

## Changelog

- 2026-04-22 — opened. Extracted from the implicit theory behind MEMD-10-STAR
  and the V4–V10 roadmap; made explicit because V11–V13 SOTA push needs a
  theory doc to bind against. The 7-axis definitions (7/10 SOTA baseline,
  9–10 future) are the actual commitment; the bench list per axis is
  advisory and will evolve with public bench ecosystem changes.
