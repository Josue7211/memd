---
title: memd Gap × Milestone Coverage Matrix
date: 2026-04-22
sources: [MEMD-10-STAR.md, ROADMAP.md, MILESTONE-v4.md, phase-docs V4-V8]
---

# memd Gap × Milestone Coverage Matrix

## Executive Summary

35 listed gaps from `MEMD-10-STAR.md` v2026-04-13 mapped across V4–V10 phases. **2 unowned gaps** expose roadmap holes. **3 gaps over-claimed** risk scope creep. **Pillar 10 (Self-Improvement)** has no clear V4 plan. **V4 should add Gap-25 (live memory contract) as an early win.**

**0.1.0 release definition:** composite ≥6.0, pillars 1,2,3,4,5,7,9 at ≥6/10 each, pillars 6,8 at ≥5/10. Exit on 3-session dogfood + production readiness audit.

---

## 1. Gap × Phase Coverage Matrix

| # | Gap Name | Pillar | Tier | Owned By | Milestone | Confidence | Evidence Path |
|---|----------|--------|------|----------|-----------|------------|---|
| 1 | status noise floods memory | 1 | operational | B2 (verified) | M1 | **owned** | ROADMAP:92, MEMD-10-STAR:63 |
| 2 | wake excludes non-status kinds | 1 | operational | B2 (verified) | M1 | **owned** | ROADMAP:92, MEMD-10-STAR:64 |
| 3 | procedure detection never triggers | 10 | operational | F2 (verified) | M1 | **owned** | ROADMAP:95, MEMD-10-STAR:62 |
| 4 | inbox never drains | 1 | operational | C2 (verified) | M1 | **owned** | ROADMAP:92, MEMD-10-STAR:65 |
| 5 | continuity ghost refs | 1 | operational | C2 (verified) | M1 | **owned** | ROADMAP:92, MEMD-10-STAR:61 |
| 6 | memd status lies | 11 | operational | K2 (complete) | M4 | **owned** | ROADMAP:179, MEMD-10-STAR:68 |
| 7 | no dogfood verification gate | 11 | operational | K2 (complete) | M4 | **owned** | ROADMAP:179, MEMD-10-STAR:69 |
| 8 | agent write helpers unreachable | 7 | operational | F2 (verified) | M1 | **implied** | ROADMAP:95 names F2, but phase-f2 silent on gap-8 |
| 9 | no cross-session codebase memory | 7 | operational | L2 (complete) | M4 | **implied** | ROADMAP:185, L2 owns "handoff-quality" but not codebase-context explicitly |
| 10 | no correction flow E2E | 2 | architectural | D2 (verified) | M2 | **owned** | ROADMAP:114, test "h2_ab_influence_corrections_improve_retrieval" |
| 11 | no behavior-changing recall proof | 3 | architectural | H2 (verified) | M2 | **owned** | ROADMAP:117, phase-h2 focuses on scenario harness |
| 12 | no cross-harness continuity proof | 7 | architectural | H2 (verified) | M2 | **owned** | ROADMAP:117, "cross-harness retrieval test passes" |
| 13 | memory not navigable (flat, not graph) | 8 | architectural | E2 (verified) | M2 | **owned** | ROADMAP:115, atlas-theory-lock-v1 |
| 14 | atlas dormant | 8 | architectural | E2 (verified) | M2 | **implied** | ROADMAP:115, E2 partial — atlas wired but not called from runtime; A8 wakes it fully |
| 15 | contradiction detection never triggers | 2 | architectural | D2 (verified) | M2 | **owned** | ROADMAP:114, "entity-based contradiction detection (old_item entity lookup)" |
| 16 | trust hierarchy unproven | 2 | architectural | D2 (verified) | M2 | **owned** | ROADMAP:114, "trust_rank hierarchy" in schema |
| 17 | lanes are grep-over-files | 8 | architectural | G2 (verified) | M2 | **owned** | ROADMAP:116, "Lane tag in compact_record/wake packet" |
| 18 | no token efficiency measurement | eval | measurement | P2 (verified) | M3 | **owned** | ROADMAP:159, "Token efficiency measured" |
| 19 | no public benchmark parity (LongMemEval) | eval | measurement | P2 (verified) | M3 | **owned** | ROADMAP:159, "LongMemEval ≥ 80%" gate |
| 20 | no compaction quality proof | 4 | measurement | O2 (verified) | M3 | **owned** | ROADMAP:158, "Isolation + Trust" phase |
| 21 | no decay calibration | 10 | measurement | O2 (verified) | M3 | **owned** | ROADMAP:158, "Lifecycle Calibration" phase |
| 22 | no consolidation quality proof | 10 | measurement | O2 (verified) | M3 | **owned** | ROADMAP:158 |
| 23 | no handoff quality proof | 7 | measurement | J2 (verified) | M3 | **owned** | ROADMAP:157, "phase-j2-isolation-trust" |
| 24 | no overnight evolution (Phase I) | 10 | product | M2-evo (pending) | M4 | **owned** | ROADMAP:182, "M2-evo" phase listed as pending |
| 25 | no live memory contract | 1 | product | **unowned** | — | **unowned** | MEMD-10-STAR:25-31 defines non-negotiable; no V4-V10 phase owns proof |
| 26 | skill gating is config flags, not product | 9 | product | N2 (pending) | M4 | **owned** | ROADMAP:183, "Integrations Polish" |
| 27 | no human surface / dashboard UI | 11 | product | I2 (pending) | M4 | **owned** | ROADMAP:181, "Human Dashboard, 11 substeps" |
| 28 | no data recovery procedure | 9 | product | N2 (pending) | M4 | **owned** | ROADMAP:183, gap-30 in backlog |
| 29 | no semantic search baseline (RAG disabled) | 5 | product | B3 (complete) | V3 | **owned** | ROADMAP:216, "LME 0.86→≥0.92" intrinsic-only |
| 30 | no admission control / rate limiting | 9 | product | L2 (complete) | M4 | **owned** | ROADMAP:185, "per-agent write rate limit 100 soft / 200 hard" |
| 31 | no observability (metrics, tracing) | 11 | product | K2 (complete) | M4 | **owned** | ROADMAP:179, "structured tracing" |
| 32 | no privacy/visibility enforcement proof | 6 | product | J2 (verified) | M3 | **owned** | ROADMAP:157, "Isolation + Trust" |
| 33 | no multi-project isolation proof | 6 | product | J2 (verified) | M3 | **owned** | ROADMAP:157 |
| 34 | no multi-user/team support | 6 | product | D9 (pending) | V9 | **owned** | ROADMAP:335, "Multi-User / Team" |
| 35 | no latency briefing | 7 | product | K2 (complete) | M4 | **owned** | ROADMAP:179, "latency SLA" |

**Coverage totals:**
- **Owned:** 33 gaps (94%)
- **Implied:** 2 gaps (6%)
- **Unowned:** 0 gaps in table, but see below

---

## 2. Unowned Gaps Summary

### Gap-25: No Live Memory Contract

**Status:** UNOWNED — exposed by user directive 2026-04-21.

**Definition (MEMD-10-STAR:25-31):** "Live memory updates while the agent works — not 'remember later' discipline. ... If it can't do those things reliably, it doesn't deserve the product claim."

**Current state:**
- Hooks exist (write-path gate in A3)
- PreCompact captures ledger
- No phase proves live-update behavior under real task pressure

**Which milestone should own it?**

**Recommendation: V4 C4 (Correction Capture E2E) should expand scope to include Gap-25.**

- C4 already owns "user types correction → hook captures → judge confirms → record lands"
- Gap-25 is the superclass: any write (fact, preference, correction) must be live
- Expansion cost: low (C4 already has harness + hook tracing)
- V4 needs a visible win in 10-STAR pillar 1 (session continuity); "live contract proven" is a quick composite lift (session continuity 1→3 without it, 1→4 with it)

**Proposed C4 revised goal:** "User writes anything (fact, preference, correction) → hook captures → live downstream retrieval sees it without ceremony or 'remember' discipline."

---

## 3. Over-Claimed Gaps (2+ Phases)

| Gap | Claimed By | Conflict | Primary | Secondary | Rationale |
|-----|-----------|----------|---------|-----------|-----------|
| **Gap-14 (atlas dormant)** | E2, A8 | E2 "activates" atlas in M2; A8 "wakes" atlas in V8 | **A8** | E2 (wiring maintenance) | E2 activates retrieval hints + entity links (infrastructure). A8 ships the first UI surface. A8 is the real "wake"; E2 is prep. |
| **Gap-8 (agent write helpers unreachable)** | F2, L2 | F2 mentions "pipeline existed"; L2 mentions "write rate limit" | **L2** | F2 (legacy reference) | L2 owns the actual write-harness enforcement (Lamport lock, rate limit, handoff packet). F2 is historical. Delete gap-8 from F2 charter. |
| **Gap-9 (no cross-session codebase memory)** | F2, L2 | F2 "pipeline"; L2 "divergence receipts" | **L2** | F2 (pipeline foundation) | L2 owns proof (cross-harness E2E A→B→A with corrections). F2 is infrastructure. Clean separation: F2=pipeline, L2=proof. |

---

## 4. Pillar Coverage Analysis

### 10-STAR Pillar Scorecard

| Pillar | Current | Lifts In | Post-V10 Target | V4 Plan | Status |
|--------|---------|----------|-----------------|---------|--------|
| **1. Core Memory Correctness** | 3/10 | V4 A4, B4 | 8/10 | A4 ledger survival, D4 working-context compiler | ✓ owns |
| **2. Correction + Belief** | 2/10 | V4 C4, F4; V7 A7-G7 | 9/10 | C4 E2E capture, F4 drift | ✓ owns |
| **3. Behavior-Changing Recall** | 1/10 | V5 bench, V6 compiler, V7 C7 | 8/10 | E4 progressive-depth + G4 harness | ✓ implied via V5 |
| **4. Working-Memory Control** | 3/10 | V4 D4, E4 | 8/10 | D4 compiler, E4 progressive-depth | ✓ owns |
| **5. Provenance + Explainability** | 5/10 | V8 D8, V10 gap-audit | 9/10 | E4 hints in wake, A8 atlas | ✓ implied via V8 |
| **6. Shared + Federated Memory** | 3/10 | V9 A9-F9 | 9/10 | —none— | **GAP: V4 silent on multi-scope proof** |
| **7. Cross-Harness Portability** | 3/10 | V4 B4, E4; V5 C5; V8 A8 | 9/10 | B4 hook contract, L2 handoff | ✓ owns |
| **8. Navigable Knowledge + Evidence** | 2/10 | V8 A8-E8 | 9/10 | —none— | **GAP: V4 does not wake atlas** |
| **9. Capability Contracts + Safety** | 2/10 | M4 L2, N2; V9 merging | 8/10 | B4 enforcement, L2 complete | ✓ owns |
| **10. Self-Improvement** | 2/10 | V10 A10-F10 | 9/10 | —none— | **GAP: V4 has no story** |
| **11. Operator UX + Observability** | 2/10 | M4 K2, I2; V8 A8-F8 | 9/10 | —none— | **GAP: V4 silent; L2 infra done** |

**Summary:**
- Pillars 1, 2, 3, 4, 7, 9 → V4 has explicit ownership
- Pillar 5 (provenance) → implied via V4 E4 + V8
- Pillar 6 (federated) → zero V4 plan (deferred to V9)
- Pillar 8 (navigation) → zero V4 plan (deferred to V8 A8)
- Pillar 10 (self-improvement) → zero V4 plan (V10 owns it)
- Pillar 11 (observability) → L2 ships infra; K2 shipped; I2 pending; V4 silent on UX

**Red flag:** Pillar 10 (self-improvement 2→9 target) has no V4 foothold. V4 is "repair the live loop"; V10 is "auto-improve." But V4 could seed V10 with: "consolidation audit harness" (measure what consolidation breaks) as part of D4/E4. Today, D4 is "compress context"; it could also be "measure compression loss."

---

## 5. V4-Specific Gaps & Recommendations

Given the matrix above, V4 is **transaction-correct but not philosophy-complete**. Additions:

### A. Gap-25: Live Memory Contract (Early Win)

**Proposal:** Expand C4 from "correction capture" to "live-write contract" — any write surfaces downstream without ceremony.

**Concrete phase addition:**
- **Phase Name:** C4 revised → "Correction + Live-Write Contract E2E"
- **Add substep:** Post-correction recall test from a concurrent agent reading same memory space
- **Gate:** correction lands → immediately queryable by another agent thread, zero ceremony
- **Composite lift:** session-continuity 1→4 (currently C4 alone → 1→3)

### B. Consolidation-Audit Harness (Seed for V10)

**Proposal:** Add substep to D4 (working-context compiler):
- D4 compresses context X → output Y
- New: measure what facts/prefs were lost, classify loss as "recoverable from DB" vs "silent drop"
- Store report at `.memd/logs/consolidation-audit-YYYY-MM-DD.log`
- Baseline for V10 auto-consolidation: "any consolidation regressing recall >2% fails merge"

**Benefit:** V4 "repair" + V10 "improve" are now linked by measurement.

### C. Pillar-6 Foundation (No Code, Add Contract)

**Proposal:** V4 does not ship multi-user, but should add contract:
- **New file:** `docs/contracts/federated-memory-visibility.md`
- Define: local vs synced vs project vs global retrieval rules
- Define: privacy boundary test harness signature (to be implemented in V9)
- Rationale: V9 can't write good code without the contract; V4 can write the contract for free

**Benefit:** V9 unblocked day 1; V4 scope-zero; 10-STAR pillar 6 clarity +2.

---

## 6. "0.1.0 Release" Definition

### Product Promise

"memd 0.1.0 is a **durable, correct memory substrate for agents in a single session**. State survives compaction. Corrections are honored. Memd stays synced while work changes. Ready for production claude-code/codex dogfood, not ready for multi-user or public benchmarking."

### Composite Score Floor: 6.0/10

Target across V1-V4 only:

| Axis | Weight | V4 Target | Rationale |
|------|--------|-----------|-----------|
| Session continuity | 20% | **6/10** | A4 + B4 prove compaction + hook survival; not perfect, but reliable |
| Correction retention | 15% | **6/10** | C4 + F4 prove capture + replay; no UX yet |
| Procedural reuse | 15% | **4/10** | No change (V4 silent on auto-procedures); baseline from V3 |
| Cross-harness | 15% | **4/10** | L2 proves handoff quality; single-harness sessions work; no multi-harness E2E |
| Raw retrieval | 15% | **6/10** | V3 baseline (B3 landed); V4 adds progressive-depth (E4) |
| Token efficiency | 10% | **7/10** | D4 compiler proves <2k wake tokens; gains from V3 consolidation |
| Trust + provenance | 10% | **4/10** | L2 adds explain, K2 adds tracing; not user-facing yet |

**Composite: 2.15 → 6.0** (V1+V3+V4 combined)

### Load-Bearing Pillars (Must Hit ≥6/10)

1. **Pillar 1 (Core Memory Correctness):** 3→8 via V4. LOAD-BEARING. Without this, 0.1.0 fails day 1.
2. **Pillar 2 (Correction):** 2→6 via V4. LOAD-BEARING. "Corrections are honored" is the headline.
3. **Pillar 3 (Behavior-Changing Recall):** 1→4 via V4+V5 proof. CONDITIONAL LOAD-BEARING on V5 bench passing.
4. **Pillar 4 (Working-Memory Control):** 3→8 via V4. LOAD-BEARING. "Does not bloat context" is explicit promise.
5. **Pillar 7 (Cross-Harness):** 3→5 via L2. NICE-TO-HAVE (single-harness works fine for 0.1.0).

### Nice-to-Have Pillars (≥5/10 Acceptable)

6. **Pillar 5 (Provenance):** 5→7 via K2+L2. Keep or drop; not core for 0.1.0.
7. **Pillar 9 (Capability Contracts):** 2→6 via L2. Advisory; enforcement deferred to V8.
8. **Pillar 6 (Federated):** 3→3. Out of scope; note in release docs.
9. **Pillar 8 (Navigation):** 2→2. Out of scope; A8 in V8.
10. **Pillar 10 (Self-Improvement):** 2→2. Out of scope; V10 owns it.
11. **Pillar 11 (Observability):** 2→4. K2/L2 add infra; UX in V8.

### Minimum Feature Inventory (Prove These Work)

| Feature | Test | Success Criteria | Owner |
|---------|------|-----------------|-------|
| **State survives compaction** | A4 harness (3 sessions, auto-compact) | Ledger reloads post-compaction; zero re-reads | V4 A4 |
| **Correction captured → honored** | C4 scenario (user says "no"; next session retrieves corrected value) | Retrieval shows corrected fact; provenance links to user turn | V4 C4+F4 |
| **Wake context <2k tokens** | D4+E4 load test | Real dogfood session (30 turns) wake budget ≤ 2000 chars | V4 D4, E4 |
| **Hooks in order under load** | B4 stress test (100 concurrent writes) | No out-of-order hook invocations; Lamport clock monotonic | V4 B4, L2 |
| **Progressive-depth routing** | E4 bench (query → wake → lookup → resume) | Recall@k improves monotonically; no re-fetches | V4 E4 |
| **No silent overwrites** | L2 cross-harness (A→B→A corrections) | Divergence detected; receipts logged | V4 B4, M4 L2 |

### Exit Criteria Beyond Numbers

- [ ] **Dogfood:** 3-session claude-code recording with >2 compactions, >1 correction, zero silent data loss
- [ ] **Incident**: Zero P1 bugs in dogfood logs; any P2 must have a PR
- [ ] **Documentation:** RUNNING.md covers memd setup + hook behavior + failure modes
- [ ] **Rollback plan:** Feature-flagged behind `MEMD_V4_LIVE_LOOP=1`; can disable any phase per-session
- [ ] **Audit:** Codebase audit confirms every gap 1-9 is addressed (missing = hold release)
- [ ] **Composite score:** Re-run `memd composite` on v0.1.0 release candidate; must show ≥6.0

---

## 7. V4 Expansion Proposal

**Add to ROADMAP.md → V4 section:**

```
| Phase | Name | Status | Gaps | Phase Doc |
| A4 | Read-State Across Compaction | `planned` | 5, 25 (live-contract foundation) | [[docs/phases/v4/phase-a4-read-state-compaction.md]] |
| B4 | Hook Contract Enforcement | `planned` | 6, 8, 35 | [[docs/phases/v4/phase-b4-hook-contract.md]] |
| C4 | Correction + Live-Write E2E | `planned` | 10, 25, 26 | [[docs/phases/v4/phase-c4-correction-capture-e2e.md]] |
| D4 | Working-Context Compiler + Audit | `planned` | 1, 4, 18 | [[docs/phases/v4/phase-d4-working-context-compiler.md]] |
| E4 | Progressive-Depth Recall | `planned` | 2, 11, 13 | [[docs/phases/v4/phase-e4-progressive-depth-recall.md]] |
| F4 | Preference Replay + Drift | `planned` | 10, 16, 20 | [[docs/phases/v4/phase-f4-preference-drift.md]] |
| G4 | Session-Continuity Proof Harness | `planned` | 1, 25 (live-contract gate) | [[docs/phases/v4/phase-g4-continuity-proof.md]] |
```

**Notable:** A4/C4/G4 now explicitly own Gap-25.

---

## Appendix: File Citations

- MEMD-10-STAR.md:
  - 11 Pillars current state: lines 50–284
  - Gap inventory: lines 286–341
  - 10-STAR composite target: lines 343–369

- ROADMAP.md:
  - M1-M4 phases: lines 84–201
  - V4–V10 milestones: lines 244–358
  - L2 completion: line 185

- MILESTONE-v4.md:
  - Composite targets: lines 18–30
  - Completion gate: lines 36–45

- phase-a4-read-state-compaction.md:
  - Goal: lines 14–20
  - Deliver: lines 22–31

- phase-a5-cross-session-recall.md:
  - Why V5 exists: lines 17–19
  - Bench metrics: lines 26–29

- phase-a6-episodic-ingest.md:
  - Typed ingest: lines 14–20
  - Deliver: lines 23–29

- phase-a7-correction-lane-ingestion-verify.md:
  - Live-trace verify: lines 23–27

- phase-a8-atlas-navigation-ui.md:
  - Graph view: lines 25–29
  - Why A8 exists: lines 18–20
