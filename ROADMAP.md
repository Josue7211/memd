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
| H | Hive Coordination | `pending` | shared second brain across harnesses | [[2026-04-11-memd-ralph-roadmap]] |
| I | Overnight Evolution | `pending` | dream/autodream/autoresearch with trust gates | [[2026-04-11-memd-ralph-roadmap]] |

## Next Up

1. Start `Phase H` (Hive Coordination).

## Phase E Follow-Up (Closed)

All audit tail items resolved:
- Boot context slimmed 78% (12.5KB → 2.2KB)
- Shell env quoting fixed
- CODEX_MEMORY zombies killed
- Cross-harness wake proof closed
- Phase-flip rule defined
- [[docs/superpowers/plans/2026-04-12-phase-e-cross-harness-wake-proof.md|Detailed Phase E cross-harness wake proof plan]]

## Immediate Backlog

1. [[2026-04-12-phase-g-10-star-gaps]] — `in_progress`.
   16 gaps triaged: 8 closed (auto-promote, auto-retire, conflict detection,
   supersedes, wake budget, doc fixes), 4 deferred medium, 3 deferred as features,
   1 deferred to Phase H.

2. [[2026-04-12-claude-code-bootstrap-bridge-gap]] — `closed`, fixed `2026-04-12`.
   Boot context slimmed 78%. CODEX_MEMORY zombies killed. SessionStart hook gutted.

3. [[2026-04-12-shell-unsafe-memd-env-generation]] — `closed`, fixed `2026-04-12`.
   All env values now shell-single-quoted via `rewrite_shell_env` helper.

4. [[2026-04-12-roadmap-state-audit-tail-drift]] — `closed`, fixed `2026-04-12`.
   Fixed by closing Phase E audit tail and aligning all state sources. Phase-flip rule added.

5. [[2026-04-13-ambiguous-glob-imports]] — `closed`, fixed `2026-04-13`.
   Removed duplicate re-exports from evaluation/mod.rs. Dropped `allow(ambiguous_glob_imports)`.

6. [[2026-04-13-silent-event-loss]] — `closed`, fixed `2026-04-13`.
   8 production `let _ =` sites replaced with `if let Err(e)` + eprintln warn logging.

7. [[2026-04-13-healthz-masks-db-errors]] — `closed`, fixed `2026-04-13`.
   healthz now returns 500 on DB errors instead of silent 200 with 0 items.

8. [[2026-04-13-flaky-handoff-verifier-test]] — `closed`, fixed `2026-04-13`.
   3 verify-feature tests now spawn mock servers with dynamic ports instead of hardcoded 59999.

9. [[2026-04-13-stale-per-harness-bundle-files]] — `closed`, fixed `2026-04-13`.
   17 dead files deleted. Cleanup step added to `write_agent_profiles` init.

10. [[2026-04-13-hive-deferred-transaction]] — `open`.
    `.transaction()` uses DEFERRED. Concurrent harness writes → SQLITE_BUSY.

11. [[2026-04-13-lane-architecture-gaps]] — `open`.
    Theory-implementation divergence. Grep-over-files instead of DB tags.
    5 of 6 lanes missing. `INSPIRATION_FILES` misses 2 of 6 files.

12. [[2026-04-13-dead-code-cleanup]] — `closed`, fixed `2026-04-13`.
    Removed `legacy_dashboard_html` (368 lines) and `empty_dashboard_html` (77 lines).
    `persist_atlas_link` annotated (Phase H). `is_wake_only_agent` annotated (tested).

13. [[2026-04-13-planning-ghost-refs-in-tests]] — `closed`, false positive.
    `.planning/` refs in tests are intentional project fixture setup, not ghost refs.

14. [[2026-04-13-stale-doc-refs]] — `closed`, already resolved.
    FEATURES.md no longer exists — removed in prior audit.

15. [[2026-04-13-wake-packet-kind-coverage]] — `open`.
    Wake packets only surface kinds matching retrieval intent. Others invisible.

16. [[2026-04-13-checkpoint-resume-asymmetry]] — `open`.
    Checkpoint saves per-item metadata. Resume loads aggregate snapshot. No round-trip.

17. [[2026-04-13-server-startup-panics]] — `closed`, fixed `2026-04-13`.
    DB open and TCP bind now use match+eprintln+exit(1) with actionable hints.

18. [[2026-04-13-silent-ok-chains]] — `closed`, fixed `2026-04-13`.
    13 `.ok()` sites in procedural.rs + atlas.rs now log warnings via `.inspect_err()`.

19. [[2026-04-13-untested-api-routes]] — `open`.
    15 of 72 routes (21%) untested. Mostly coordination/tasks — Phase H territory.

20. [[2026-04-13-multimodal-extraction-stubs]] — `open`.
    PDF/Image/Video extraction returns placeholder strings. Mineru/RagAnything unwired.

21. [[2026-04-13-clippy-warnings]] — `closed`, fixed `2026-04-13`.
    158→36 warnings (77% reduction). Collapsible ifs auto-fixed, derive impls, lifetime elision.
    Remaining 35 are too-many-args and identical blocks requiring manual refactoring.

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
