# M1 Execution Plan: Make It Work

> This plan tells the next code session exactly what to do. No theory reading needed.
> No "figure out the architecture" needed. Just follow the steps.

## Context

M1 is reopened. The live loop is broken at 5 of 7 steps. 10-STAR composite: 1.8/10.
Three phases are reopened: B2 (Signal vs Noise), C2 (Ghost Cleanup), F2 (Ingestion Pipeline).
All three have detailed phase docs with pass gates, donor extractions, and rollback criteria.

## Root Cause

Agents don't remember because:
1. Working memory filled with status noise (80-90%) — facts/decisions/preferences evicted
2. Pipeline lifecycle (expire/promote/archive) never runs in production
3. Lane source material only exists for inspiration lane — architecture/design/workflow/preference lanes empty
4. TTL enforcement doesn't run — expired items accumulate forever
5. Wake packet excludes facts/decisions/procedures — surfaces status only

## Execution Order

Phases must execute in dependency order. Each phase has a pass gate in its phase doc.
Do not start a phase until its dependencies pass their gates.

### Step 1: B2 — Signal vs Noise

**Branch**: `research/mining` (current)
**Phase doc**: [[docs/phases/phase-b2-signal-vs-noise.md]]
**Depends on**: A2 (verified)
**Fixes**: gaps 1, 2

**What to build**:

1. **Status cap in working memory admission** — hard cap Status kind at 2 out of 8 slots.
   Find the admission function in the working context compiler. Add kind-based quota:
   `if kind == Status && status_count >= 2 → reject`.
   - Donor: B2-D2 (mempalace hard cap per type)

2. **Wake kind scoring reweight** — facts/decisions must outrank status in wake packet assembly.
   Find the wake compiler scoring function. Reweight:
   `Fact: 3.0, Decision: 3.0, Preference: 2.5, Procedure: 2.0, Status: 0.5`.
   - Donor: B2-D1 (supermemory priority dedup)

3. **Content hash dedup on checkpoint writes** — `SHA256(normalize(content))[0..16]`.
   If hash exists, reinforce (increment count, bump version) instead of inserting duplicate.
   - Donor: B2-D3 (Omegon direct Rust lift)

4. **Live memory contract** — new captures appear in working memory within one cycle.
   Store a fact via `memd remember` → next `memd wake` packet contains that fact.

**Pass gate**:
- `memd eval` score >= 65
- Store a fact → next wake packet contains it (not buried under status)
- Working memory has <= 2 status items out of 8 slots
- Status noise reduction >= 80%

**Verify nodes**: P1 (working context compiler), M1 (working context), S1 (wake packet)

### Step 2: C2 — Ghost Cleanup

**Phase doc**: [[docs/phases/phase-c2-ghost-cleanup.md]]
**Depends on**: B2
**Fixes**: gaps 4, 5

**What to build**:

1. **TTL enforcement in production** — expired items must be removed on every maintenance cycle,
   not just flagged. Find `gc_expired_items()` — ensure it runs in the `maintain` command
   AND in the live loop (not just tests).
   - Donor: C2-D1 (Omegon lifecycle-driven expiration)

2. **Phase-completion expiry** — when a phase flips to verified/complete, all its status records
   get TTL set to grace period. Find phase-status records in DB, add expiry trigger on status change.

3. **Consolidation loop runs** — `memd maintain --mode full --apply true` must actually execute
   in production. Wire it into the session lifecycle or a cron-like mechanism.

**Pass gate**:
- `memd status` shows 0 ghost refs
- Continuity capsule fields resolve to live items only
- Expired items cleaned from DB within 1 worker cycle
- Concurrent write test: 3 agents writing simultaneously, 0 SQLITE_BUSY errors

**Verify nodes**: P2 (session continuity), M2 (session continuity)

### Step 3: F2 — Ingestion Pipeline

**Phase doc**: [[docs/phases/phase-f2-ingestion-pipeline.md]]
**Depends on**: B2
**Fixes**: gaps 3, 8

**What to build**:

1. **Seed all 6 lanes** — currently only inspiration has source material.
   Create source files in `.memd/lanes/` for:
   - `architecture/` — extract from THEORY.md, DESIGN.md, architecture.md
   - `design/` — extract from DESIGN.md (product design, visual direction, progressive depth)
   - `research/` — extract from donor extraction docs, research loop docs
   - `workflow/` — "ROADMAP.md is authoritative", "backlogs in docs/backlog/", phase-flip rules
   - `preference/` — user corrections, architecture decisions (from backlog items)

2. **Ingest theory docs** — THEORY.md, DESIGN.md, architecture.md content must become
   `lane:architecture` memory items with `kind=Fact` or `kind=Topology`.

3. **Ingest workflow conventions** — "ROADMAP.md is authoritative", "backlogs in docs/backlog/",
   process rules → `lane:workflow` memory items.

4. **Ingest user preferences** — corrections and architecture decisions → `lane:preference` items.

5. **Change detection** — content hash tracking so modified files re-ingest, unchanged skip.
   Already implemented for inspiration lane — extend to all 6.

**Pass gate**:
- After `memd setup`, all 6 lanes have DB memory items
- `memd lookup --query "wake vs resume"` returns architecture-lane fact
- Modify a source file → next wake re-ingests only changed file
- Unchanged files not re-read (hash match = skip)

**Verify nodes**: I1 (turns, docs, artifacts, corrections), I2 (hooks, checkpoints, spill), M4 (semantic memory), M5 (procedural memory)

## M1 Node-by-Node E2E Tests

Every node in the architecture graph must pass its M1 criterion. Phase gates (B2/C2/F2)
cover some nodes. The tests below cover ALL nodes. Run after all three phases pass.

### Ingest Layer

| Node | M1 Criterion | E2E Test |
|------|-------------|----------|
| I1 | Captures all event types, no silent loss | `memd remember` (fact), `memd checkpoint` (status), `memd hook capture` (correction), `memd ingest` (doc) — verify all 4 event types appear in DB with source metadata |
| I2 | Hooks fire, checkpoints persist, spill drains | Trigger a hook → verify hook record in DB. Write checkpoint → verify persisted. Accumulate spill → run `memd hook spill --apply` → verify spill drained to 0 |
| I3 | Resume packet builds, handoff works | `memd resume` → verify packet contains what/where/changed/next. `memd handoff` → verify handoff packet builds without error |

### Control Plane

| Node | M1 Criterion | E2E Test |
|------|-------------|----------|
| P1 | Holds current-phase data, stale records expire | (B2 gate) Store fact + 3 status items → working memory holds fact, status capped at 2 |
| P2 | Answers what/where/changed/next without ghosts | (C2 gate) `memd status` → 0 ghost refs. Continuity fields resolve to live items only |
| P3 | Each kind retrieved correctly, promotion runs | `memd lookup --query X` with items of each kind (Fact, Decision, Procedure, Status, Preference) → each kind retrievable. `memd maintain --mode full --apply true` → promotion runs, candidate promoted to canonical |
| P4 | Corrections stored, provenance tracked | `memd correct --id <uuid> --content "new"` → correction stored. `memd explain <id>` → provenance shows original + correction |

### Typed Memory

| Node | M1 Criterion | E2E Test |
|------|-------------|----------|
| M1 | Budget enforced, admission/eviction works | (B2 gate) Insert 12 items → working memory holds 8, evicts 4 with reasons logged |
| M2 | Clean resume without ghost refs | (C2 gate) Resume after GC → 0 ghost refs in continuity |
| M3 | Events stored with time/context/outcome | `memd checkpoint` → verify episodic record has timestamp, context (project/session), and outcome fields |
| M4 | Architecture decisions stored, facts retrievable | (F2 gate) `memd lookup --query "wake vs resume"` returns architecture-lane fact |
| M5 | Procedures stored, detection triggers in prod | Store a procedure via `memd remember --kind procedure "how to deploy"` → `memd lookup --query "deploy"` returns it. Verify procedure detection trigger fires on matching context |
| M6 | Candidates held before promotion | Store item as candidate → verify it stays candidate until promotion criteria met. `memd maintain` → weak signal expires, strong signal survives |
| M7 | Promoted items durable and retrievable | Promote candidate → verify canonical item retrievable via `memd lookup`. Verify promoted item survives GC |

### Recall Surfaces

| Node | M1 Criterion | E2E Test |
|------|-------------|----------|
| S1 | Surfaces facts/decisions/preferences, not just status | (B2 gate) Wake packet contains fact/decision/preference. Status ≤ 2 of 8 slots |
| S2 | Regions exist, entities auto-created | Store 10+ items → `memd explore` returns non-empty regions. Entities auto-created for projects/namespaces |
| S3 | Canonical items retrievable on demand | Promote item to canonical → `memd lookup` with canonical filter returns it |
| S4 | Raw spine intact, events reachable | After ingest, verify raw spine records exist in `.memd/state/raw-spine.jsonl` or DB. Events traceable from canonical back to raw |
| S5 | Obsidian compile works | `memd obsidian compile` → verify vault directory created with markdown files |
| S6 | Briefing packet builds | `memd brief` or latency briefing endpoint → returns non-empty packet |

### Live Loop E2E (Integration)

After node tests pass, run the full loop from
[[docs/verification/NODE-VERIFICATION-MATRIX.md#live-loop-verification]]:

1. capture raw event → I1
2. update working context → P1 → M1
3. update session continuity → P2 → M2
4. write episodic record → M3
5. repair semantic truth → P3 → M4
6. update procedural memory → P3 → M5
7. compile wake packet → S1

**M1 gate test**: Store a real preference. Close session. Open new session. Verify preference
appears in wake packet. Verify stale completed-phase records are gone.

### Resume Loop E2E

1. load session continuity → P2 → M2
2. merge with semantic memory → M4
3. pull relevant procedures → M5
4. compile working context → P1 → M1
5. continue without big reread → S1

**Test**: After resume, agent has architecture knowledge without re-reading THEORY.md.

## What NOT To Do

- Do not touch M2 phases (D2, E2, G2, H2) — they depend on M1 being solid
- Do not touch dashboard (I2) — that's M4
- Do not add new features — make existing features work
- Do not refactor — fix the pipeline
- Do not skip live testing — synthetic-only gates are why we're here
- Do not bloat the roadmap — update phase docs with evidence, link from roadmap

## Amnesia Prevention Checklist

After M1 passes, before declaring done:

- [ ] Architecture decisions stored as `lane:architecture` items in DB
- [ ] Workflow conventions stored as `lane:workflow` items in DB
- [ ] User preferences stored as `lane:preference` items in DB
- [ ] Wake packet surfaces facts/decisions/preferences, not just status
- [ ] Stale completed-phase records expired from working memory
- [ ] New session wake contains theory knowledge without re-reading THEORY.md
- [ ] `memd lookup --query "10-star model"` returns architecture-lane fact
- [ ] `memd lookup --query "ROADMAP.md is authoritative"` returns workflow-lane fact

If any of these fail, M1 is not done. Period.

## Key Files

| What | Where |
|------|-------|
| Working memory admission | `crates/memd-server/src/` (find admission/eviction functions) |
| Wake packet compiler | `crates/memd-client/src/` (find wake assembly) |
| Ingestion pipeline | `crates/memd-client/src/` (find ingest/setup functions) |
| GC/maintenance | `crates/memd-server/src/store.rs` (gc_expired_items) |
| Lane source material | `.memd/lanes/*/` |
| Store migrations | `crates/memd-server/src/store_migrations.rs` |

## Phase Docs (read before starting each step)

- [[docs/phases/phase-b2-signal-vs-noise.md]]
- [[docs/phases/phase-c2-ghost-cleanup.md]]
- [[docs/phases/phase-f2-ingestion-pipeline.md]]
- [[docs/verification/NODE-VERIFICATION-MATRIX.md]]
- [[docs/verification/MEMD-10-STAR.md]]
