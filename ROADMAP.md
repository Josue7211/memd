# memd Roadmap

`ROADMAP.md` is the single roadmap source of truth for this repo.

<!-- ROADMAP_STATE
truth_date: 2026-04-13
version: v1
version_status: in_progress
current_phase: phase_g
phase_status: verified
next_phase: phase_h
next_step: start_phase_h
active_blockers: []
-->

## Status Snapshot

- truth date: `2026-04-13`
- current version: `v1`
- version status: `in_progress`
- current phase: `Phase G: Procedural Learning`
- phase status: `verified`
- next phase: `Phase H: Hive Coordination`
- next step: `start Phase H`

## Current Focus

Phase G verified. Procedural memory: 7 API routes, 1 DB table, 9 procedural
tests, 98 total server tests. Full lifecycle: record → promote → match → use
→ retire. Auto-detect from event spine. Cross-session tracking. Wake packet
integration. Voice modes aligned with upstream caveman skill (7 modes).

## Blockers

None.

## Status Rules

- `pending`: not started
- `pending`: not started
- `in_progress`: active build work
- `blocked`: cannot move without external fix or decision
- `verified`: engineering verification passed
- `verified_with_audit_tail`: engineering verification passed, follow-up audit still open
- `complete`: human-tested and accepted at the product level

## Phase-Flip Rule

One rule for state transitions:
- `pending` → `in_progress`: when first task starts
- `in_progress` → `verified`: when engineering verification passes AND all audit items closed
- `in_progress` → `verified_with_audit_tail`: when verification passes but audit items remain
- `verified_with_audit_tail` → `verified`: when all audit items close
- `verified` → `complete`: when human accepts at product level

When flipping, update ALL three sources: ROADMAP.md frontmatter, phase doc frontmatter,
and phase doc body status. The live memd state follows from the next `memd wake`.

## Product Contract

`memd` is a multiharness second-brain memory substrate for humans and agents.
It must preserve active truth, durable memory, provenance, corrections,
continuity, and recovery across sessions, tools, harnesses, and artifacts
without collapsing back into transcript reconstruction.

Current priority harnesses are Codex, OpenCode, Hermes, and OpenClaw. They are
the proving ground for this contract, not the whole definition of the product.

## Canonical Phases

Use phases for execution order. Detailed phase plans live in linked docs.

| Phase | Name | Status | Purpose | Detail |
| --- | --- | --- | --- | --- |
| A | Raw Truth Spine | `verified` | capture once, keep raw evidence, preserve source linkage | [[phase-a-raw-truth-spine]] |
| B | Session Continuity | `verified` | fresh-session resume without transcript rebuild | [[phase-b-session-continuity]] |
| C | Typed Memory | `verified` | explicit memory kinds instead of one flat store | [[phase-c-typed-memory]] |
| D | Canonical Truth | `verified` | corrections, trust, freshness, conflict handling | [[phase-d-canonical-truth]] |
| E | Wake Packet Compiler | `verified` | compile small action-ready memory packets | [[phase-e-wake-packet-compiler]] |
| F | Memory Atlas | `verified` | packet -> region -> evidence navigation | [[phase-f-memory-atlas]] |
| G | Procedural Learning | `verified` | learn reusable operating procedures | [[phase-g-procedural-learning]] |
| H | Core Hardening | `pending` | fix operational pipeline — make phases B-G actually work | [[docs/theory/models/2026-04-11-memd-ralph-roadmap.md|ralph-roadmap]] |
| I | Human Dashboard | `pending` | web UI for memory browsing, correction, navigation, status | [[docs/theory/models/2026-04-11-memd-ralph-roadmap.md|ralph-roadmap]] |
| J | Hive Coordination | `pending` | shared second brain across harnesses | [[docs/theory/models/2026-04-11-memd-ralph-roadmap.md|ralph-roadmap]] |
| K | Overnight Evolution | `pending` | dream/autodream/autoresearch with trust gates | [[docs/theory/models/2026-04-11-memd-ralph-roadmap.md|ralph-roadmap]] |

## Next Up

1. Start `Phase H` (Core Hardening) — fix 9 operational pipeline issues.
2. Then `Phase I` (Human Dashboard).
3. Then `Phase J` (Hive Coordination).
4. Then `Phase K` (Overnight Evolution).
5. See [[docs/audits/2026-04-13-full-codebase-audit.md]] for audit findings.
6. See [[docs/verification/MEMD-10-STAR.md]] for 35-gap 10-star target.

## Operational Reality (2026-04-13 Audit)

Phase A-G are architecturally complete but operationally broken. Full audit:
- 10 audit agents + 7 deep-read agents scanned every source file (~600KB)
- Live loop: 2/7 steps work. 5/7 broken.
- 10-star composite score: ~3.3/10 (target: 8+)
- 16 open backlog items (7 critical, 4 high, 5 medium)
- Fix order: drain ghosts → fix status noise → fix wake kind exclusion →
  wire procedure detection → fix status lies → fix agent helpers → add dogfood gate

## Phase E Follow-Up (Closed)

All audit tail items resolved:
- Boot context slimmed 78% (12.5KB → 2.2KB)
- Shell env quoting fixed
- CODEX_MEMORY zombies killed
- Cross-harness wake proof closed
- Phase-flip rule defined
- [[docs/superpowers/plans/2026-04-12-phase-e-cross-harness-wake-proof.md|Detailed Phase E cross-harness wake proof plan]]

## Immediate Backlog

1. [[docs/backlog/2026-04-12-phase-g-10-star-gaps.md|phase-g-10-star-gaps]] — `in_progress`.
   16 gaps triaged: 8 closed (auto-promote, auto-retire, conflict detection,
   supersedes, wake budget, doc fixes), 4 deferred medium, 3 deferred as features,
   1 deferred to Phase H.

2. [[docs/backlog/closed/2026-04-12-claude-code-bootstrap-bridge-gap.md|claude-code-bootstrap-bridge-gap]] — `closed`, fixed `2026-04-12`.
   Boot context slimmed 78%. CODEX_MEMORY zombies killed. SessionStart hook gutted.

3. [[docs/backlog/closed/2026-04-12-shell-unsafe-memd-env-generation.md|shell-unsafe-memd-env-generation]] — `closed`, fixed `2026-04-12`.
   All env values now shell-single-quoted via `rewrite_shell_env` helper.

4. [[docs/backlog/closed/2026-04-12-roadmap-state-audit-tail-drift.md|roadmap-state-audit-tail-drift]] — `closed`, fixed `2026-04-12`.
   Fixed by closing Phase E audit tail and aligning all state sources. Phase-flip rule added.

5. [[docs/backlog/archive/2026-04-13-ambiguous-glob-imports.md|ambiguous-glob-imports]] — `closed`, fixed `2026-04-13`.
   Removed duplicate re-exports from evaluation/mod.rs. Dropped `allow(ambiguous_glob_imports)`.

6. [[docs/backlog/archive/2026-04-13-silent-event-loss.md|silent-event-loss]] — `closed`, fixed `2026-04-13`.
   8 production `let _ =` sites replaced with `if let Err(e)` + eprintln warn logging.

7. [[docs/backlog/archive/2026-04-13-healthz-masks-db-errors.md|healthz-masks-db-errors]] — `closed`, fixed `2026-04-13`.
   healthz now returns 500 on DB errors instead of silent 200 with 0 items.

8. [[docs/backlog/archive/2026-04-13-flaky-handoff-verifier-test.md|flaky-handoff-verifier-test]] — `closed`, fixed `2026-04-13`.
   3 verify-feature tests now spawn mock servers with dynamic ports instead of hardcoded 59999.

9. [[docs/backlog/archive/2026-04-13-stale-per-harness-bundle-files.md|stale-per-harness-bundle-files]] — `closed`, fixed `2026-04-13`.
   17 dead files deleted. Cleanup step added to `write_agent_profiles` init.

10. [[docs/backlog/2026-04-13-hive-deferred-transaction.md|hive-deferred-transaction]] — `open`.
    `.transaction()` uses DEFERRED. Concurrent harness writes → SQLITE_BUSY.

11. [[docs/backlog/2026-04-13-lane-architecture-gaps.md|lane-architecture-gaps]] — `open`.
    Theory-implementation divergence. Grep-over-files instead of DB tags.
    5 of 6 lanes missing. `INSPIRATION_FILES` misses 2 of 6 files.

12. [[docs/backlog/archive/2026-04-13-dead-code-cleanup.md|dead-code-cleanup]] — `closed`, fixed `2026-04-13`.
    Removed `legacy_dashboard_html` (368 lines) and `empty_dashboard_html` (77 lines).
    `persist_atlas_link` annotated (Phase H). `is_wake_only_agent` annotated (tested).

13. [[docs/backlog/archive/2026-04-13-planning-ghost-refs-in-tests.md|planning-ghost-refs-in-tests]] — `closed`, false positive.
    `.planning/` refs in tests are intentional project fixture setup, not ghost refs.

14. [[docs/backlog/archive/2026-04-13-stale-doc-refs.md|stale-doc-refs]] — `closed`, already resolved.
    FEATURES.md no longer exists — removed in prior audit.

15. [[docs/backlog/2026-04-13-wake-packet-kind-coverage.md|wake-packet-kind-coverage]] — `open`. **CRITICAL**
    Wake packets only surface Status + LiveTruth. Facts, Decisions, Preferences, Procedures
    structurally excluded. Root cause: fixed `intent=current_task` in wake gives Project scope
    +1.15 and Global scope -0.2. `context_score()` is kind-blind. No kind-based filtering in
    context/working/inbox APIs. Even with status noise fixed, facts still don't surface.

16. [[docs/backlog/2026-04-13-checkpoint-resume-asymmetry.md|checkpoint-resume-asymmetry]] — `open`.
    Checkpoint saves per-item metadata. Resume loads aggregate snapshot. No round-trip.

17. [[docs/backlog/archive/2026-04-13-server-startup-panics.md|server-startup-panics]] — `closed`, fixed `2026-04-13`.
    DB open and TCP bind now use match+eprintln+exit(1) with actionable hints.

18. [[docs/backlog/archive/2026-04-13-silent-ok-chains.md|silent-ok-chains]] — `closed`, fixed `2026-04-13`.
    13 `.ok()` sites in procedural.rs + atlas.rs now log warnings via `.inspect_err()`.

19. [[docs/backlog/2026-04-13-untested-api-routes.md|untested-api-routes]] — `open`.
    15 of 72 routes (21%) untested. Mostly coordination/tasks — Phase H territory.

20. [[docs/backlog/2026-04-13-multimodal-extraction-stubs.md|multimodal-extraction-stubs]] — `open`.
    PDF/Image/Video extraction returns placeholder strings. Mineru/RagAnything unwired.

21. [[docs/backlog/archive/2026-04-13-clippy-warnings.md|clippy-warnings]] — `closed`, fixed `2026-04-13`.
    158→36 warnings (77% reduction). Collapsible ifs auto-fixed, derive impls, lifetime elision.
    Remaining 35 are too-many-args and identical blocks requiring manual refactoring.

22. [[docs/backlog/2026-04-13-stale-continuity-ghost-refs.md|stale-continuity-ghost-refs]] — `open`. **CRITICAL**
    Full lifecycle broken: inbox created from git status without file existence check →
    expired items NOT removed (filter includes expired) → no drain endpoint → continuity
    `left_off` and `blocker` pull from expired ghost items → `memd status` reports healthy.
    Need: file validation at creation, expired item exclusion, drain endpoints, GC.

23. [[docs/backlog/2026-04-13-agent-write-helpers-unreachable.md|agent-write-helpers-unreachable]] — `open`.
    Shell helpers exist (`.memd/agents/remember-long.sh` etc.) but agents can't use them.
    wake.md protocol says `remember-long` — agents try `memd remember-long` which fails.
    Fix: either add CLI subcommand aliases or fix wake.md to show actual shell paths.
    Related: `MEMD_BUNDLE_BACKEND_ENABLED=false` means `sync-semantic` has no backend.
    Full agent write pipeline through Phase G is not operationally wired.

24. [[docs/backlog/2026-04-13-no-persistent-codebase-map|no-persistent-codebase-map]] — `closed`, fixed `2026-04-13`.
    Initial codebase structure map stored via `memd remember --kind fact --tag codebase-structure`.
    Auto-update on structural changes is a future feature request.

25. [[docs/backlog/2026-04-13-status-reports-healthy-while-broken.md|status-reports-healthy-while-broken]] — `open`.
    `memd status` returns `setup_ready=true, degraded=false, status=ok` while:
    - heartbeat references nonexistent `.planning/` paths
    - working memory is 100% status noise, zero user content
    - inbox is clogged with ghost refs to deleted files
    - write helpers are unreachable from agents
    - RAG backend is disabled
    Status is a liveness check, not a health check. Needs deep health verification.

26. [[docs/backlog/2026-04-13-dogfood-verification-gap.md|dogfood-verification-gap]] — `open`. **CRITICAL**
    All phases marked "verified" via cargo test, but operational pipeline broken.
    No end-to-end dogfood gate: store fact → recall next session → continuity works
    → inbox drains → procedures learned → write helpers callable.
    Need reusable benchmark suite that tests the product, not just the code.

27. [[docs/backlog/2026-04-13-status-noise-floods-memory.md|status-noise-floods-memory]] — `open`. **CRITICAL**
    15+ auto-checkpoint triggers each create `kind=status` records. No deduplication.
    24h TTL, 10-20 accumulate per day. Working memory budget (1600 chars, 8 items)
    consumed 80-90% by status noise. `checkpoint_as_remember_args()` forces kind=status.
    User facts/decisions evicted by freshness bias. Root: runaway meta-recording.

28. [[docs/backlog/2026-04-13-procedure-detection-never-triggers.md|procedure-detection-never-triggers]] — `open`. **CRITICAL**
    Phase G "verified" but `detect_procedures()` only called in tests + manual CLI.
    `maintain_runtime()` doesn't call it. No hook, no scheduler, no runtime trigger.
    Procedures table permanently empty during real usage. 90% wired, 0% operational.

29. [[docs/backlog/2026-04-13-inbox-never-drains.md|inbox-never-drains]] — `open`. **CRITICAL**
    No drain endpoints (no acknowledge/dismiss/clear). Expired items still appear
    in inbox (filter `status != Active` includes expired). No GC for expired memory items.
    Items persist indefinitely. 6 current ghost items from deleted `.planning/` files.

30. [[docs/backlog/2026-04-13-atlas-dormant.md|atlas-dormant]] — `open`.
    Atlas Phase F fully implemented (7 routes, regions, trails, explore, expand).
    Never called from dogfood loop. Not in wake packets, not in context, not in
    working memory. Entities auto-created but never surfaced. Entity links empty.

31. [[docs/backlog/2026-04-13-queen-ops-dead-code.md|queen-ops-dead-code]] — `open`.
    3 queen routes (deny, reroute, handoff) implemented in routes.rs but NO client
    methods in lib.rs. Coordination modes (exclusive_write, shared_review) stored
    but not enforced. Overlap detection post-hoc only.

32. [[docs/backlog/2026-04-13-missing-integration-tests.md|missing-integration-tests]] — `open`.
    Consolidation, decay, workspace memory, source memory have zero integration tests.
    Runtime maintain flow untested. 15/72 API routes (21%) untested.

33. [[docs/backlog/2026-04-13-memd-no-cross-session-codebase-memory.md|no-cross-session-codebase-memory]] — `open`. **HIGH**
    memd doesn't remember codebase structure across sessions. Agent re-scans every time.
    Core product promise ("read once, remember once, reuse everywhere") not working.
    Compound issue: facts excluded from wake (#15), status noise (#27), write helpers (#23).

34. [[docs/backlog/2026-04-13-memory-not-navigable.md|memory-not-navigable]] — `open`. **CRITICAL**
    Core product promise is "obsidian hybrid" — navigable, linked memory. But memory items
    are flat text blobs. Entity links table empty. Atlas dormant. No wiki link parsing in
    content. No auto-linking from co-occurrence. Memory is a flat store, not a graph.
    Infrastructure exists (atlas, entities, links, trails) — nothing connects them.

## Recently Closed

- `Phase A` raw truth spine: `verified`
- `Phase B` session continuity: `verified`
- `Phase C` typed memory: `verified`
- `Phase D` canonical truth: `verified`

## Reference Docs

- [[docs/core/setup.md|Setup and harness behavior]]
- [[docs/verification/milestones/MILESTONE-v1.md|Milestone v1 verification]]
- [[docs/strategy/research-loops.md|Research loops]]
- [[docs/theory/models/2026-04-11-memd-ralph-roadmap.md|Detailed Ralph roadmap spec]]
- [[docs/theory/models/2026-04-11-memd-canonical-theory-synthesis.md|Canonical theory synthesis]]

## Non-Goals

- transcript dumping
- vendor lock-in
- using RAG as the only memory layer
- mixing project-local truth with global truth
- letting one provider silently overwrite another provider’s memory
