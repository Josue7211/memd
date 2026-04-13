# memd Full Codebase Audit — 2026-04-13

## Executive Summary

10 audit agents + 7 deep-read agents scanned every source file across 7 crates (~600KB+).
Phase G is marked "verified" but the operational pipeline is broken at every layer of the
live loop. Tests pass. The product doesn't work. This document is the complete findings.

---

## I. Theory vs Reality — The 7-Step Live Loop

The theory lock (v1) defines a live loop that should fire on every interaction:

| Step | Theory | Implementation | Operational | Gap |
|------|--------|---------------|-------------|-----|
| 1. Capture raw event | "capture once, keep raw evidence" | 15+ auto-checkpoint triggers | **works** — but floods with `kind=status` | status noise overwhelms signal |
| 2. Update working context | "tiny active packet" | `working_memory()` with 1600-char budget, 8-item admission | **broken** — 80-90% status records | no kind-based admission preference |
| 3. Update session continuity | "resume without rereads" | `ContinuityCapsule` with 5 fields | **broken** — references deleted files, pulls from expired inbox | no file validation, no expired filtering |
| 4. Write episodic memory | "events, experiences, timeline" | `record_event()` + entity linkage | **works** — events recorded on store/search | entities auto-created but never surfaced |
| 5. Repair semantic memory | "stable truths, decisions, constraints" | facts/decisions can be stored | **broken** — structurally excluded from wake by intent scoring | `context_score()` is kind-blind |
| 6. Update procedural memory | "learn reusable workflows" | `detect_procedures()` + full lifecycle | **broken** — detect never called in runtime | procedures table permanently empty |
| 7. Compile wake packet | "tiny action-ready resume" | `render_bundle_wakeup_markdown()` | **broken** — only surfaces Status + LiveTruth | fixed `intent=current_task` penalizes all non-project-scope kinds |

**Verdict: 2 of 7 live loop steps work operationally. 5 are broken.**

---

## II. Complete Issue Inventory

### CRITICAL (blocks the product contract)

| ID | Issue | Root Cause | Fix Complexity |
|----|-------|-----------|----------------|
| 27 | Status noise floods memory | 15+ auto-checkpoint triggers create `kind=status`. No dedup. 24h TTL. 10-20/day. 80-90% of working memory. | medium — add redundancy_key to status, reduce TTL, add kind preference in admission |
| 15 | Wake excludes facts/decisions/procedures | Fixed `intent=current_task` gives Project +1.15, Global -0.2. `context_score()` kind-blind. | medium — add kind bonus in context_score, or multi-intent wake |
| 28 | Procedure detection never triggers | `detect_procedures()` only called in tests + manual CLI. `maintain_runtime()` doesn't call it. Worker doesn't call it. | easy — one line in worker loop |
| 29 | Inbox never drains | No drain/acknowledge/clear endpoints. Expired items still shown. No GC. 6 ghost items from deleted `.planning/`. | medium — exclude expired from inbox, add drain endpoint |
| 22 | Continuity ghost refs | `left_off` and `blocker` pull from expired inbox items referencing deleted files. No file validation anywhere. | medium — filter expired in compact_inbox_items, validate paths |
| 25 | memd status lies | Reports `setup_ready=true, degraded=false` while all above is broken. Heartbeat shows "editing .planning/ROADMAP.md" (deleted file). | medium — extend status with eval score, set degraded if < 65 |
| 26 | No dogfood verification gate | All phases "verified" via cargo test. No e2e gate: store fact → recall next session → continuity works. | medium — extend eval_bundle_memory with 5 new assertions |

### HIGH (degrades the product significantly)

| ID | Issue | Root Cause | Fix Complexity |
|----|-------|-----------|----------------|
| 23 | Agent write helpers unreachable | Shell helpers exist (`.memd/agents/remember-long.sh`). wake.md says `remember-long`. Agents try `memd remember-long` which fails. RAG backend disabled. | easy — fix protocol line in wakeup.rs:261 |
| 30 | Atlas dormant | 7 routes, regions/explore/trails all implemented. Never called from dogfood loop. Not in wake/context/working. Entities auto-created but invisible. Entity links empty. | hard — wire atlas into resume/wake pipeline |
| — | Evaluation has no cost metrics | eval_bundle_memory scores memory quality only. No token count penalties, no latency scoring. Bloated prompts can score "strong". | medium — add token/latency dimensions |
| — | Sidecar has no resilience | No request timeouts. No retry logic. No fallback cache. Can hang on slow backend. | easy — add timeout + retry |

### MEDIUM (quality/correctness concerns)

| ID | Issue | Root Cause | Fix Complexity |
|----|-------|-----------|----------------|
| 10 | DEFERRED transactions | `.transaction()` defaults to DEFERRED. Concurrent writes → SQLITE_BUSY. 4 call sites. | easy — switch to `transaction_with_behavior(Immediate)` |
| 11 | Lane architecture gaps | Theory defines 6 lanes with dynamic activation. Implementation is grep-over-files, not DB tags. 5 of 6 lanes missing. | hard — Phase H feature work |
| 16 | Checkpoint/resume asymmetry | Checkpoint saves per-item metadata. Resume loads aggregate snapshot. No round-trip fidelity. | medium — align data models |
| 31 | Queen ops dead code | 3 routes (deny/reroute/handoff) in routes.rs. Zero client methods in lib.rs. Coordination modes not enforced. | medium — add client methods or remove routes |
| 32 | Missing integration tests | Consolidation, decay, workspace, source memory: 0 integration tests. 15/72 routes (21%) untested. Runtime maintain untested. | large — write tests |
| 20 | Multimodal stubs | PDF/Image/Video extraction returns placeholder strings. Mineru/RagAnything unwired. | defer — external dependency |
| — | Rendering asymmetry | Resume includes `## T` (task). Handoff doesn't. Truncation inconsistent across sections. Duplicate `compact_inline()`. | easy — align renderers |
| — | Dead code lint misleading | `#[allow(dead_code)]` on evaluation modules that ARE exported and used. False sense of unused code. | easy — remove suppression |
| — | Ephemeral session orphans | Auto-retire can orphan active tasks on ephemeral sessions (codex-fresh, session-live-*). | medium — check active tasks before retiring |

### LOW (design debt, not blocking)

| Issue | Note |
|-------|------|
| `persist_atlas_link()` marked dead code "Phase H" | Intentional deferral |
| Scope ordering asymmetry in routing | ProjectFirst vs GlobalFirst have different fallback chains |
| Intent bonus ranges vary widely | CurrentTask Local=0.55 vs Project=1.15 (0.6 gap). By design? |
| Working memory policy hardcoded | `memory_policy_snapshot()` returns hardcoded values, not configurable |
| Stale threshold fixed at 30 days | `STALE_AFTER_DAYS = 30` in keys/mod.rs. Not configurable per-project |

---

## III. Testability & Benchmarkability Assessment

### What's testable now

| System | Unit Tests | Integration Tests | E2E Dogfood |
|--------|-----------|------------------|-------------|
| Memory store/retrieve | yes (98 server tests) | yes (mock server) | **no** |
| Checkpoint/resume | yes | partial | **no** |
| Wake packet compile | yes | partial | **no** |
| Atlas (Phase F) | yes (18 tests) | yes | **no** |
| Procedural (Phase G) | yes (13 tests) | yes | **no** |
| Hive coordination | yes | yes | **no** |
| Consolidation | **no** | **no** | **no** |
| Decay | **no** | **no** | **no** |
| Workspace/source memory | **no** | **no** | **no** |
| Runtime maintain | **no** | **no** | **no** |
| Full live loop | **no** | **no** | **no** |

### What we need for dogfood benchmarking

The existing `eval_bundle_memory()` is 80% of the benchmark. It scores 0-100 with penalties.
Missing assertions that would catch the bugs we found:

1. **"Working memory contains ≥1 non-status kind item"** → catches #27
2. **"All inbox items reference paths that exist on disk"** → catches #22, #29
3. **"Procedure table is non-empty after maintenance cycle"** → catches #28
4. **"Wake packet contains ≥1 fact or decision"** → catches #15
5. **"memd status heartbeat references only existing paths"** → catches #25
6. **"Continuity fields do not reference expired items"** → catches #22
7. **"Status records are <50% of working memory"** → catches #27

### The 10-Star Scorecard (from evaluation theory lock)

| Axis | Weight | Current Score | Reason |
|------|--------|--------------|--------|
| Session continuity | 20% | **2/10** | left_off/blocker reference deleted files, inbox clogged |
| Correction retention | 15% | **3/10** | supersede mechanics exist, no first-class correction flow |
| Procedural reuse | 15% | **1/10** | code complete, detection never triggers, table empty |
| Cross-harness continuity | 15% | **4/10** | wake packets work, 6 harness presets, but content is status noise |
| Raw retrieval strength | 15% | **5/10** | search works, entity search works, but wake/working excludes most kinds |
| Token efficiency | 10% | **6/10** | 78% boot reduction achieved, working budget enforced |
| Trust + provenance | 10% | **5/10** | explain/provenance surfaces exist, source trust scoring works |

**Composite: ~3.3/10** (weighted). The product claim requires 8+.

---

## IV. What We Do Right

1. **Architecture is correct.** 10-star model, 7 memory kinds, 3 control functions, typed memory, correction flow — the theory is sound.
2. **Infrastructure is solid.** 15 DB tables, 40+ indexes, 50+ store methods, 61 routes, 90 client methods, 79 CLI commands, 6 harness presets — all wired, no stubs.
3. **Worker daemon functional.** Verification + decay + consolidation loop runs every 300s.
4. **Bundle init works.** Shell helpers, env files, hooks, agent profiles all generated correctly.
5. **Entity system auto-populates.** Entities created on every memory item with salience tracking.
6. **Deduplication is sophisticated.** Dual-key (canonical + redundancy) with stemming and stopword filtering.
7. **Hive core works.** Messages, claims, tasks, sessions, board/roster/follow — 75% functional.
8. **Evaluation framework exists.** `eval_bundle_memory()` with gap analysis, scenarios, composite scoring.
9. **Wake budget enforcement works.** 1200 chars for Claude Code, 1800 for others, line-by-line trimming.
10. **Raw spine preserved.** NDJSON audit log, source linkage, event tracking.

---

## V. What's Broken — Root Causes

The bugs share three root causes:

### Root Cause 1: Auto-checkpoint is a runaway feedback loop

The system records metadata about itself more aggressively than user content.
15+ triggers × 24h TTL × no dedup = status noise dominates every surface.

### Root Cause 2: Retrieval is kind-blind

`context_score()`, `working_item_priority()`, and `inbox_score()` don't examine `item.kind`.
Combined with `intent=current_task` scope scoring, Status records in Project scope
always outrank Facts/Decisions in Global scope. The architecture supports typed memory
but the retrieval layer treats all kinds identically.

### Root Cause 3: "Verified" means "tests pass" not "product works"

Phase pass gates were met by reading code and running isolated tests.
No phase had an operational dogfood gate: store a real fact, resume a real session,
verify the fact surfaces, verify continuity is accurate.

---

## VI. Comparison Chart — memd vs mempalace vs supermemory

| Capability | mempalace | supermemory | memd (current) | memd (target) |
|-----------|-----------|-------------|----------------|---------------|
| **Raw retrieval (LongMemEval)** | **96.6%** (raw mode) | ~85% (API-first) | untested (infra ready) | ≥95% |
| **Memory kinds** | loading-depth layers only | flat store | **7 kinds defined** but only Status surfaces | 7 kinds all surfacing |
| **Session continuity** | wake-up layers (L0-L3) | none | **broken** (ghost refs) | 5-field capsule working |
| **Correction flow** | not wired | manual only | mechanics exist, **not operational** | first-class correction → recall |
| **Procedural learning** | none | none | **code exists, never triggers** | auto-detect from event spine |
| **Cross-harness** | single client | API per-user | **6 harness presets**, wake works | one brain across all |
| **Hive coordination** | none | none | **75% wired** (sessions/claims/tasks) | full queen + coordination |
| **Atlas navigation** | palace graph (thin) | none | **fully implemented, dormant** | active in resume/wake |
| **Token efficiency** | raw-first (high cost) | API-managed | **78% boot reduction** | continued optimization |
| **Trust + provenance** | none | basic source tracking | **explain + source trust scoring** | full drilldown |
| **Benchmark suite** | LongMemEval only | none public | **eval + gap + scenarios exist** | 10-star composite |
| **Self-improvement** | none | none | **autoresearch + evolution framework** | gated with regression checks |
| **Entity system** | room/wing metadata | none | **auto-created, never surfaced** | searchable + atlas backbone |
| **Multimodal** | none | none | **stubs only** | PDF/image/video extraction |

### Where memd wins today
- typed memory model (7 kinds vs flat)
- multi-harness architecture (6 presets vs 1)
- hive coordination (unique)
- atlas navigation layer (unique, though dormant)
- procedural learning framework (unique, though unwired)
- evaluation/benchmark infrastructure (unique depth)

### Where memd loses today
- raw retrieval: mempalace at 96.6%, memd untested
- operational reliability: mempalace works, memd's live loop is broken
- simplicity: mempalace is ~5 files, memd is ~600KB across 7 crates

### The honest gap
memd has the right architecture. mempalace has the working product. supermemory has the API.
memd's theory is ahead. memd's operational reality is behind. The fix is not more features —
it's making what exists actually work.

---

## VII. Where We Are on the Journey

### Phase completion (honest assessment)

| Phase | Status | Tests | Operational |
|-------|--------|-------|-------------|
| A: Raw Truth Spine | verified | pass | **yes** — raw events stored |
| B: Session Continuity | verified | pass | **broken** — ghost refs, expired items |
| C: Typed Memory | verified | pass | **broken** — only Status surfaces |
| D: Canonical Truth | verified | pass | **partial** — corrections exist, not proven in flow |
| E: Wake Packet Compiler | verified | pass | **broken** — excludes most kinds |
| F: Memory Atlas | verified | pass | **dormant** — never called from runtime |
| G: Procedural Learning | verified | pass | **broken** — detect never triggers |
| H: Hive Coordination | pending | — | 75% wired, queen dead code |
| I: Overnight Evolution | pending | — | autoresearch framework exists |

**Honest reality: Phase A works. Phases B-G are architecturally complete but operationally broken.**

### Path to 10-star

1. **Fix the live loop** (issues 27, 15, 28, 29, 22, 25) — make Phases B-G actually work
2. **Add dogfood gate** (issue 26) — never mark a phase "verified" without operational proof
3. **Wire atlas** (issue 30) — Phase F should enhance resume/wake, not be dormant
4. **Complete Phase H** — hive queen, coordination enforcement, concurrent writes
5. **Run public benchmarks** — prove retrieval parity with mempalace on LongMemEval
6. **Ship Phase I** — overnight evolution with regression gates

### What to do next

Fix order (dependency chain):
1. Drain ghosts (#29, #22) — unblocks everything
2. Fix status noise (#27) — free working memory for real content
3. Fix wake kind exclusion (#15) — let facts surface
4. Wire procedure detection (#28) — fill procedures table
5. Fix status lies (#25) — make status reflect reality
6. Fix agent write helpers (#23) — one-line fix
7. Add dogfood assertions (#26) — prevent regression

After those 7 fixes, re-run `memd eval` and verify the 10-star score climbs from ~3.3 to ≥6.

---

## VIII. Fix Locations (Quick Reference)

| Fix | File | Line | Change |
|-----|------|------|--------|
| Exclude expired from inbox | `crates/memd-server/src/routes.rs` | 263-266 | Add `.filter(\|e\| e.item.status != Expired)` before inbox filter |
| Filter expired in continuity | `crates/memd-client/src/runtime/resume/mod.rs` | 1450-1457 | Filter `reasons.contains("expired")` in `compact_inbox_items()` |
| Add redundancy_key to status | `crates/memd-client/src/runtime/checkpoint.rs` | 1017-1044 | Add redundancy_key computation in `checkpoint_as_remember_args()` |
| Reduce status TTL | `crates/memd-client/src/runtime/checkpoint.rs` | ~809, 868, 911 | Change 86_400 → 3_600 |
| Kind bonus in context_score | `crates/memd-server/src/helpers.rs` | ~537-583 | Add +0.3 for Fact/Decision/Procedure, -0.2 for Status |
| Wire procedure detection | `crates/memd-worker/src/main.rs` | after consolidation | Add `client.procedure_detect()` call |
| Fix protocol line | `crates/memd-client/src/runtime/resume/wakeup.rs` | 261 | Change bare names to `.memd/agents/remember-long.sh` paths |
| Add eval assertions | `crates/memd-client/src/evaluation/eval_report_runtime.rs` | scoring section | Add 5 new penalty checks |
| DEFERRED → IMMEDIATE | 4 files with `.transaction()` | see grep | `transaction_with_behavior(TransactionBehavior::Immediate)` |

---

*Generated by full codebase audit: 10 audit agents + 7 deep-read agents, ~600KB source code.*
