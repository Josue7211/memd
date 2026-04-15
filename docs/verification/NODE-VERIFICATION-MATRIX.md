# Architecture Graph — Node Verification Matrix

> Authoritative per-node verification criteria for the architecture graph
> (see `docs/core/architecture.md` mermaid diagram). Each milestone gate
> verifies every node. A node can be ✓ (passes), ~ (partial), or ✗ (fails).
> A milestone cannot close with any ✗ in its tier.

## Ingest Layer

| Node | What It Does | M1: Works | M2: Correct | M3: Provable | M4: 10-Star |
|------|-------------|-----------|-------------|--------------|-------------|
| I1 | Turns, docs, artifacts, corrections | Captures all event types, no silent loss | Corrections routed to P4 | Capture rate measured | Multi-source ingest |
| I2 | Hooks, checkpoints, spill | Hooks fire, checkpoints persist, spill drains | Spill→promotion pipeline | Spill latency measured | Auto-checkpoint on session end |
| I3 | Resume and handoff packets | Resume packet builds, handoff works | Handoff preserves correction state | Handoff quality scored | Cross-harness handoff proven |

## Control Plane

| Node | What It Does | M1: Works | M2: Correct | M3: Provable | M4: 10-Star |
|------|-------------|-----------|-------------|--------------|-------------|
| P1 | Working context compiler | Holds current-phase data, stale records expire | Lane-aware admission | Budget efficiency measured | Per-project configurable policy |
| P2 | Session continuity | Answers what/where/changed/next without ghosts | Cross-session preferences persist | Continuity quality scored | Cross-harness continuity |
| P3 | Typed retrieval + promotion | Each kind retrieved correctly, promotion runs | Lane-based routing (DB tags not grep), contradiction detection | Promotion quality measured, decay calibrated | Consolidation loop runs overnight |
| P4 | Correction, provenance, authority | Corrections stored, provenance tracked | Corrections change future recall, trust hierarchy enforced | Correction retention scored | Human correction outranks all |

## Typed Memory

| Node | What It Does | M1: Works | M2: Correct | M3: Provable | M4: 10-Star |
|------|-------------|-----------|-------------|--------------|-------------|
| M1 | Working context | Budget enforced, admission/eviction works | Reasons visible, rehydration works | Adversarial over-capacity tests | Policy configurable |
| M2 | Session continuity | Clean resume without ghost refs | Preferences + architecture persist | Resume quality scored | Cross-machine resume |
| M3 | Episodic memory | Events stored with time/context/outcome | Timeline navigable via atlas | Episodic retrieval quality scored | Episodic→canonical promotion |
| M4 | Semantic memory | Architecture decisions stored, facts retrievable | Corrections update semantic fast, lanes tag correctly | Cross-session stability proven | Semantic dedup across harnesses |
| M5 | Procedural memory | Procedures stored, detection triggers in prod | Procedures surface in relevant lanes | Reuse rate measured | Procedures shared via hive |
| M6 | Candidate memory | Candidates held before promotion | Promotion criteria enforced, weak signal expires | Candidate→canonical conversion rate | Auto-promotion from overnight loop |
| M7 | Canonical memory | Promoted items durable and retrievable | Canonical outranks candidate in retrieval | Canonical quality scored | Full provenance chain to raw evidence |

## Recall Surfaces

| Node | What It Does | M1: Works | M2: Correct | M3: Provable | M4: 10-Star |
|------|-------------|-----------|-------------|--------------|-------------|
| S1 | Wake packet | Surfaces facts/decisions/preferences, not just status | Lane-relevant items included, architecture decisions present | Token efficiency measured | Wake smarter each session |
| S2 | Memory atlas | Regions exist, entities auto-created | Navigable: wake→region→node→evidence, backlinks work | Navigation coverage scored | Full graph traversal |
| S3 | Canonical deep dive | Canonical items retrievable on demand | Drilldown from summary to evidence | Deep-dive quality scored | Progressive zoom |
| S4 | Raw evidence | Raw spine intact, events reachable | Source linkage from canonical to raw | Evidence completeness scored | Never-lossy guarantee proven |
| S5 | Obsidian workspace | Obsidian compile works | Two-way sync, readable vault | Sync quality scored | Graph view in Obsidian |
| S6 | Latency briefing | Briefing packet builds | Compact semantic briefing | Briefing latency measured | KV/prefix reuse hints |

## Live Loop Verification

The live loop (architecture.md §Live Loop) must pass end-to-end at each tier:

1. capture raw event → I1
2. update working context → P1 → M1
3. update session continuity → P2 → M2
4. write episodic record → M3
5. repair semantic truth → P3 → M4
6. update procedural memory → P3 → M5
7. compile wake packet → S1

**M1 test**: Run the full loop with a real preference. Verify it reaches S1 on next wake.
**M2 test**: Run the loop with a correction. Verify P4 fires and M4 updates.
**M3 test**: Measure token cost and retrieval quality across the loop.
**M4 test**: Run the loop across two harnesses. Verify shared truth via hive.

## Consolidation Loop Verification

1. inspect recent episodic records → M3
2. extract candidate truths and procedures → M6
3. merge duplicates → P3
4. expire weak signal → P3
5. promote strong signal into canonical → M7

## Resume Loop Verification

1. load session continuity → P2 → M2
2. merge with semantic memory → M4
3. pull relevant procedures → M5
4. compile working context → P1 → M1
5. continue without big reread → S1

## Phase Doc Links

| Node | Primary Phase Doc | V1 Phase Doc |
|------|------------------|--------------|
| I1, I2, I3 | [[phase-f2-ingestion-pipeline.md]] | [[phase-a-raw-truth-spine.md]] |
| P1 | [[phase-b2-signal-vs-noise.md]] | [[phase-e-wake-packet-compiler.md]] |
| P2 | [[phase-c2-ghost-cleanup.md]] | [[phase-b-session-continuity.md]] |
| P3 | [[phase-g2-lane-architecture.md]] | [[phase-c-typed-memory.md]] |
| P4 | [[phase-d2-correction-flow.md]] | [[phase-d-canonical-truth.md]] |
| M1–M7 | [[phase-b2-signal-vs-noise.md]], [[phase-c2-ghost-cleanup.md]] | [[phase-c-typed-memory.md]] |
| S1 | [[phase-b2-signal-vs-noise.md]] | [[phase-e-wake-packet-compiler.md]] |
| S2 | [[phase-e2-atlas-activation.md]] | [[phase-f-memory-atlas.md]] |
| S5 | [[phase-n2-integrations-polish.md]] | — |
| S6 | [[phase-l2-hive-hardening.md]] | — |

## Theory Doc Links

| Topic | Doc |
|-------|-----|
| 10-Star Model | [[docs/theory/models/2026-04-11-memd-10-star-memory-model-v2.md]] |
| Theory Lock | [[docs/theory/locks/2026-04-11-memd-theory-lock-v1.md]] |
| Lane Theory | [[docs/theory/locks/2026-04-13-memd-lane-theory-lock-v1.md]] |
| Retrieval Theory | [[docs/theory/locks/2026-04-11-memd-retrieval-theory-lock-v1.md]] |
| Promotion Theory | [[docs/theory/locks/2026-04-11-memd-canonical-promotion-theory-lock-v1.md]] |
| Atlas Theory | [[docs/theory/locks/2026-04-11-memd-atlas-theory-lock-v1.md]] |
| Evaluation Theory | [[docs/theory/locks/2026-04-11-memd-evaluation-theory-lock-v1.md]] |
| Hive Theory | [[docs/theory/locks/2026-04-11-memd-hive-theory-lock-v1.md]] |
| 10-STAR Scorecard | [[docs/verification/MEMD-10-STAR.md]] |
