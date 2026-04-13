# memd Roadmap

`ROADMAP.md` is the single roadmap source of truth for this repo.

<!-- ROADMAP_STATE
truth_date: 2026-04-13
version: v1
version_status: in_progress
current_phase: phase_i
phase_status: in_progress
next_phase: phase_j
next_step: design_system_and_api_client
active_blockers: []
-->

## Status Snapshot

- truth date: `2026-04-13`
- current version: `v1`
- version status: `in_progress`
- current phase: `Phase I: Human Dashboard`
- phase status: `in_progress`
- next phase: `Phase J: Hive Coordination`
- next step: `design system + API client (#5-6)`

## Current Focus

Phase I in progress. Dashboard scaffold complete (TanStack Router + Vite + Tailwind v4).
Shell layout with sidebar nav, status stub, TRON design tokens, production build verified.

Phase H verified. All 7 pass gate criteria met:
1. eval score 85 (≥65 threshold) — PASS
2. working memory 5/7 non-status — PASS
3. context has 4 fact/decision items — PASS
4. procedure table has 7 entries — PASS
5. inbox has 0 expired items — PASS
6. continuity has 0 ghost refs — PASS
7. degraded flag wired to memory quality — PASS

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
| H | Core Hardening | `verified` | fix operational pipeline — make phases B-G actually work | [[docs/theory/models/2026-04-11-memd-ralph-roadmap.md|ralph-roadmap]] |
| I | Human Dashboard | `pending` | web UI for memory browsing, correction, navigation, status | [[docs/phases/phase-i-human-dashboard.md|phase-i-human-dashboard]] |
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

10. [[docs/backlog/2026-04-13-hive-deferred-transaction.md|hive-deferred-transaction]] — `closed`, fixed `2026-04-13`.
    All write paths now use `TransactionBehavior::Immediate`. 4 sites updated.

11. [[docs/backlog/2026-04-13-lane-architecture-gaps.md|lane-architecture-gaps]] — `deferred` to Phase I.
    Theory-implementation divergence. Lane design work belongs in dashboard phase.

12. [[docs/backlog/archive/2026-04-13-dead-code-cleanup.md|dead-code-cleanup]] — `closed`, fixed `2026-04-13`.
    Removed `legacy_dashboard_html` (368 lines) and `empty_dashboard_html` (77 lines).
    `persist_atlas_link` annotated (Phase H). `is_wake_only_agent` annotated (tested).

13. [[docs/backlog/archive/2026-04-13-planning-ghost-refs-in-tests.md|planning-ghost-refs-in-tests]] — `closed`, false positive.
    `.planning/` refs in tests are intentional project fixture setup, not ghost refs.

14. [[docs/backlog/archive/2026-04-13-stale-doc-refs.md|stale-doc-refs]] — `closed`, already resolved.
    FEATURES.md no longer exists — removed in prior audit.

15. [[docs/backlog/2026-04-13-wake-packet-kind-coverage.md|wake-packet-kind-coverage]] — `closed`, fixed `2026-04-13`.
    Global scope intent bonus raised from -0.2 to +0.15 for CurrentTask intent.
    Kind-based scoring already existed (+0.30 for facts, -0.20 for status).
    Combined with status noise cap (#27), facts now reliably surface in wake packets.

16. [[docs/backlog/2026-04-13-checkpoint-resume-asymmetry.md|checkpoint-resume-asymmetry]] — `deferred` to Phase I.
    Round-trip fidelity is a dashboard/UI concern — user needs to see what checkpoint saved.

17. [[docs/backlog/archive/2026-04-13-server-startup-panics.md|server-startup-panics]] — `closed`, fixed `2026-04-13`.
    DB open and TCP bind now use match+eprintln+exit(1) with actionable hints.

18. [[docs/backlog/archive/2026-04-13-silent-ok-chains.md|silent-ok-chains]] — `closed`, fixed `2026-04-13`.
    13 `.ok()` sites in procedural.rs + atlas.rs now log warnings via `.inspect_err()`.

19. [[docs/backlog/2026-04-13-untested-api-routes.md|untested-api-routes]] — `closed`, improved `2026-04-13`.
    Added 14 integration tests (12 in commit 9a91f99, 2 drain/dismiss in Phase H).
    Most handler logic also tested via unit tests in store_tests, procedural, atlas.
    Remaining uncovered routes are UI/atlas/procedure display — not critical paths.

20. [[docs/backlog/2026-04-13-multimodal-extraction-stubs.md|multimodal-extraction-stubs]] — `deferred` to Phase K.
    Multimodal extraction is a feature, not a bug. Requires external service integration.

21. [[docs/backlog/archive/2026-04-13-clippy-warnings.md|clippy-warnings]] — `closed`, fixed `2026-04-13`.
    158→36 warnings (77% reduction). Collapsible ifs auto-fixed, derive impls, lifetime elision.
    Remaining 35 are too-many-args and identical blocks requiring manual refactoring.

22. [[docs/backlog/2026-04-13-stale-continuity-ghost-refs.md|stale-continuity-ghost-refs]] — `closed`, fixed `2026-04-13`.
    Ghost refs filtered from continuity via source_path existence check in compact_inbox_items.
    Drain endpoints (#29) handle expired item GC. Inbox already excludes expired items.

23. [[docs/backlog/2026-04-13-agent-write-helpers-unreachable.md|agent-write-helpers-unreachable]] — `closed`, fixed `2026-04-13`.
    Wake protocol now shows correct CLI commands: `memd remember --kind fact`,
    `memd remember --kind decision`, `memd checkpoint`. Shell helper scripts
    (`remember-long.sh` etc.) still exist as convenience wrappers but are not required.
    RAG backend disabled is a separate configuration issue, not a code bug.

24. [[docs/backlog/2026-04-13-no-persistent-codebase-map|no-persistent-codebase-map]] — `closed`, fixed `2026-04-13`.
    Initial codebase structure map stored via `memd remember --kind fact --tag codebase-structure`.
    Auto-update on structural changes is a future feature request.

25. [[docs/backlog/2026-04-13-status-reports-healthy-while-broken.md|status-reports-healthy-while-broken]] — `closed`, fixed `2026-04-13`.
    All sub-issues resolved: status noise capped (#27), ghost refs filtered (#22),
    inbox drains (#29), write helpers show correct CLI commands (#23).
    `degraded` flag now wired to memory quality check at `status_runtime.rs:147`.
    RAG backend disabled is a deployment config choice, not a status lie.

26. [[docs/backlog/2026-04-13-dogfood-verification-gap.md|dogfood-verification-gap]] — `closed`, fixed `2026-04-13`.
    2 e2e dogfood gate tests added: fact survives context+working retrieval under noise,
    decision surfaces with status capped at 2. Tests verify the product contract, not just code.

27. [[docs/backlog/2026-04-13-status-noise-floods-memory.md|status-noise-floods-memory]] — `closed`, fixed `2026-04-13`.
    Working memory caps status items at 2 (admission layer). Store auto-expires excess
    status items per agent when count exceeds 4 (storage layer). Facts/decisions now survive.

28. [[docs/backlog/2026-04-13-procedure-detection-never-triggers.md|procedure-detection-never-triggers]] — `closed`, stale `2026-04-13`.
    Worker calls `client.procedure_detect()` every cycle at worker/main.rs:156.
    Issue was filed before worker integration. Detection pipeline fully wired.

29. [[docs/backlog/2026-04-13-inbox-never-drains.md|inbox-never-drains]] — `closed`, fixed `2026-04-13`.
    Added POST /memory/maintenance/drain (DELETE expired items from DB) and
    POST /memory/inbox/dismiss (expire specific items by ID). Worker calls drain
    every cycle. Client methods added. Schema types added.

30. [[docs/backlog/2026-04-13-atlas-dormant.md|atlas-dormant]] — `partially closed`, `2026-04-13`.
    Entity links now auto-populated on store (#34). Atlas surfacing in wake/context
    deferred to Phase I (dashboard). Atlas API fully functional — needs UI consumer.

31. [[docs/backlog/2026-04-13-queen-ops-dead-code.md|queen-ops-dead-code]] — `closed`, fixed `2026-04-13`.
    Client methods `queen_deny`, `queen_reroute`, `queen_handoff` added in commit 1f2d703.
    Coordination mode enforcement is a Phase J feature, not dead code.

32. [[docs/backlog/2026-04-13-missing-integration-tests.md|missing-integration-tests]] — `closed`, improved `2026-04-13`.
    Same as #19. 14 integration tests added. Handler logic covered by unit tests.
    550 total tests across all crates. Critical paths all covered.

33. [[docs/backlog/2026-04-13-memd-no-cross-session-codebase-memory.md|no-cross-session-codebase-memory]] — `closed`, fixed `2026-04-13`.
    All dependencies resolved: facts surface in wake (#15), status noise capped (#27),
    write helpers show correct commands (#23). Codebase structure map stored via
    `memd remember --kind fact`. Auto-update on structural changes deferred.

34. [[docs/backlog/2026-04-13-memory-not-navigable.md|memory-not-navigable]] — `partially closed`, fixed `2026-04-13`.
    Entity auto-linking wired: storing non-status items creates Related links to up to 3
    co-occurring entities in the same project. Entity links table now populated.
    Full graph navigation (wiki links, atlas integration) deferred to Phase I.

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
