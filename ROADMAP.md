# memd Roadmap

`ROADMAP.md` is the single roadmap source of truth for this repo.

<!-- ROADMAP_STATE
truth_date: 2026-04-14
version: v2
version_status: in_progress
current_phase: phase_a2
phase_status: pending
next_phase: phase_b2
next_step: extraction_from_inspiration_repos
active_blockers: []
v1_status: frozen_architecture_complete
-->

## Status Snapshot

- truth date: `2026-04-14`
- current version: `v2` (hardening)
- version status: `in_progress`
- v1 status: `frozen` — architecture complete, operations broken (honest score: 1.8/10)
- current phase: `Phase A2: Inspiration Extraction`
- phase status: `pending`
- next step: deep-read mempalace + supermemory, extract patterns into architecture lane

## V1 → V2 Decision

V1 built the architecture: 7 crates, 15 tables, 207 types, 90 client methods,
79 CLI commands, 6 harness presets, theory-locked 10-star model. **The architecture
is ahead of any competitor.**

V1 did NOT build a working product. Honest audit (2026-04-14):
- 1 of 12 capabilities delivers value (store a memory)
- 0 of 8 product-defining features work end-to-end
- Status noise drowns signal. Atlas dormant. Lanes unimplemented.
- No correction UX. No recall proof. No human surface.

V2 is the hardening version. Goal: make every existing feature actually work,
prove it with benchmarks, ship the human surface. No new architecture — just
make the architecture real.

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

## V1 Phases (Frozen — Architecture Complete)

V1 built the architecture. These phases are frozen. Do not add features to V1 phases.
Fix operational gaps in V2 phases instead.

| Phase | Name | V1 Status | Honest Status | Detail |
| --- | --- | --- | --- | --- |
| A | Raw Truth Spine | `verified` | works — captures events, NDJSON intact | [[phase-a-raw-truth-spine]] |
| B | Session Continuity | `verified` | broken — ghost refs, expired items in continuity | [[phase-b-session-continuity]] |
| C | Typed Memory | `verified` | works — kinds stored, but dedup incomplete | [[phase-c-typed-memory]] |
| D | Canonical Truth | `verified` | stub — mechanics exist, no correction UX | [[phase-d-canonical-truth]] |
| E | Wake Packet Compiler | `verified` | broken — status noise drowns facts/decisions | [[phase-e-wake-packet-compiler]] |
| F | Memory Atlas | `verified` | dormant — 974 lines never called from runtime | [[phase-f-memory-atlas]] |
| G | Procedural Learning | `verified` | partial — detection exists, never triggers in prod | [[phase-g-procedural-learning]] |
| H | Core Hardening | `verified` | partial — fixed some issues, core pipeline still broken | [[ralph-roadmap|ralph-roadmap]] |
| I | Human Dashboard | `started` | scaffold only — TanStack Router + Vite shell | [[phase-i-human-dashboard]] |
| J | Hive Coordination | `pending` | deferred to V2 | — |
| K | Overnight Evolution | `pending` | deferred to V2 | — |

## V2 Phases (Hardening — Make It Real)

Goal: 1.8/10 → 10/10. No new architecture. Make existing architecture work.
Each phase follows Ralph rules: bounded goal, pass gate, evidence, rollback.
Deep Ralph docs linked per phase — load one at a time.

| Phase | Name | Status | Depends | Detail |
| --- | --- | --- | --- | --- |
| A2 | Inspiration Extraction | `pending` | — | [[docs/phases/phase-a2-inspiration-extraction.md]] |
| B2 | Signal vs Noise | `pending` | A2 | [[docs/phases/phase-b2-signal-vs-noise.md]] |
| C2 | Ghost Cleanup | `pending` | B2 | [[docs/phases/phase-c2-ghost-cleanup.md]] |
| D2 | Correction Flow | `pending` | B2, C2 | [[docs/phases/phase-d2-correction-flow.md]] |
| E2 | Atlas Activation | `pending` | B2, C2 | [[docs/phases/phase-e2-atlas-activation.md]] |
| F2 | Ingestion Pipeline | `pending` | B2 | [[docs/phases/phase-f2-ingestion-pipeline.md]] |
| G2 | Lane Architecture | `pending` | A2, F2 | [[docs/phases/phase-g2-lane-architecture.md]] |
| H2 | Recall Proof | `pending` | G2, D2 | [[docs/phases/phase-h2-recall-proof.md]] |
| I2 | Human Dashboard | `pending` | D2, E2, G2 | [[docs/phases/phase-i2-human-dashboard.md]] |
| J2 | Isolation + Trust | `pending` | D2, G2 | [[docs/phases/phase-j2-isolation-trust.md]] |
| K2 | Observability | `pending` | D2, C2, J2 | [[docs/phases/phase-k2-observability.md]] |
| L2 | Hive Hardening | `pending` | C2, J2 | [[docs/phases/phase-l2-hive-hardening.md]] |
| M2 | Overnight Evolution | `pending` | F2, G2, K2 | [[docs/phases/phase-m2-overnight-evolution.md]] |
| N2 | Integrations Polish | `pending` | F2, I2, M2 | [[docs/phases/phase-n2-integrations-polish.md]] |

## Next Up

1. `Phase A2` — extract from mempalace + supermemory (patterns, not code)
2. `Phase B2` — kill the status noise loop (single highest-impact fix)
3. `Phase C2` — clean the ghost refs so continuity works
4. See [[docs/backlog/2026-04-14-steal-from-inspiration-repos.md]] for extraction plan
5. See [[docs/verification/MEMD-10-STAR.md]] for 10-star target
6. See [[docs/audits/2026-04-13-full-codebase-audit.md]] for V1 audit

## Operational Reality (2026-04-14 Audit — Zero Generosity)

Honest score: **1.8/10** (previously reported 3.3 — that was generous).
1 of 12 core capabilities delivers user value (store a memory).
0 of 8 product-defining capabilities work end-to-end.

- 46 open backlog items (8 critical, 17 high, 21 medium)
- 81 total items tracked (46 open, 35 closed/deferred from V1)
- Architecture is correct. Product doesn't work.
- "Verified" phases passed unit tests, not operational dogfood.
- Status noise drowns all signal. Lanes unimplemented. Atlas dormant.
  Correction has no UX. No proof recall changes behavior.

Fix order (Tier 1 — make core work):
1. Kill status noise (#42) — dedup checkpoint, cap working memory
2. Fix wake kind scoring (#2) — facts/decisions must outrank status
3. Drain ghost refs (#46) — filter expired from continuity
4. Wire procedure detection (#3) — already in worker, surface in runtime
5. First-class correction flow (#43) — CLI command + E2E proof
6. Operational dogfood gate — store fact, close session, recall it next session
7. Wake atlas integration (#44) — surface regions in wake packet

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

35. [[docs/backlog/2026-04-13-session-resume-no-memd-memory.md|session-resume-no-memd-memory]] — `partially closed`, fixed `2026-04-13`.
    Bootstrap hook JSON format fixed (hookSpecificOutput wrapper). Global + project hooks aligned.
    Stale `.planning/` refs in global CLAUDE.md fixed. Root causes C+D still open.

36. [[docs/backlog/2026-04-13-architecture-knowledge-not-in-lanes.md|architecture-knowledge-not-in-lanes]] — `partially closed`, `2026-04-13`.
    Core architecture fact stored via `memd remember --kind fact --tag memd-architecture`.
    Init pipeline seeding and wake compiler surfacing still needed.

37. [[docs/backlog/2026-04-14-no-source-ingestion-pipeline.md|no-source-ingestion-pipeline]] — `open`, `2026-04-14`.
    Doctrine violation: no ingestion step compiles `.memd/lanes/*/` source files into DB memory items.

38. [[docs/backlog/2026-04-14-lane-queries-grep-files-not-db.md|lane-queries-grep-files-not-db]] — `open`, `2026-04-14`.
    `memd inspiration` greps raw files (4 of 6 paths). Theory says lane queries hit the server.

39. [[docs/backlog/2026-04-14-theory-design-docs-not-ingestible.md|theory-design-docs-not-ingestible]] — `open`, `2026-04-14`.
    THEORY.md, DESIGN.md, architecture.md re-read every session. Should be architecture-lane items.

40. [[docs/backlog/2026-04-14-five-starter-lanes-no-source-material.md|five-starter-lanes-no-source-material]] — `open`, `2026-04-14`.
    5 of 6 starter lanes have zero source material. Only inspiration seeded.

41. [[docs/backlog/2026-04-14-no-change-detection-on-source-material.md|no-change-detection-on-source-material]] — `open`, `2026-04-14`.
    No mtime/hash tracking outside inspiration lane. Can't know when to re-ingest.

42. [[docs/backlog/2026-04-14-status-noise-runaway-checkpoint-loop.md|status-noise-runaway-checkpoint-loop]] — `open`, `2026-04-14`.
    CRITICAL: 15+ auto-checkpoint triggers flood DB with status records. 80-90% of working memory is noise.

43. [[docs/backlog/2026-04-14-correction-flow-no-user-pathway.md|correction-flow-no-user-pathway]] — `open`, `2026-04-14`.
    Correction mechanics exist but zero user pathway. No CLI command, no E2E proof.

44. [[docs/backlog/2026-04-14-atlas-fully-built-completely-dormant.md|atlas-fully-built-completely-dormant]] — `open`, `2026-04-14`.
    974 lines, 7 routes, 18 tests — never called from any production path.

45. [[docs/backlog/2026-04-14-no-behavior-changing-recall-proof.md|no-behavior-changing-recall-proof]] — `open`, `2026-04-14`.
    CRITICAL: No benchmark proving memd recall changes agent behavior. mempalace has 96.6% on LongMemEval.

46. [[docs/backlog/2026-04-14-ghost-refs-in-continuity-capsule.md|ghost-refs-in-continuity-capsule]] — `open`, `2026-04-14`.
    CRITICAL: Expired/deleted items still referenced in continuity fields. No filtering, no path validation.

47. [[docs/backlog/2026-04-14-scope-visibility-isolation-not-enforced.md|scope-visibility-isolation-not-enforced]] — `open`, `2026-04-14`.
    Scope and visibility fields exist but never checked at retrieval. No agent isolation.

48. [[docs/backlog/2026-04-14-source-quality-ranking-not-enforced.md|source-quality-ranking-not-enforced]] — `open`, `2026-04-14`.
    canonical > promoted > candidate not enforced in retrieval ranking.

49. [[docs/backlog/2026-04-14-memory-dedup-incomplete.md|memory-dedup-incomplete]] — `open`, `2026-04-14`.
    Doctrine: "one concept, one memory object." Dedup optional, duplicates accumulate.

50. [[docs/backlog/2026-04-14-ttl-enforcement-no-gc.md|ttl-enforcement-no-gc]] — `open`, `2026-04-14`.
    Items marked expired but never removed. No GC pass. Dead items pile up.

51. [[docs/backlog/2026-04-14-explain-drilldown-not-wired.md|explain-drilldown-not-wired]] — `open`, `2026-04-14`.
    No `memd explain <id>` CLI. Provenance chain opaque. Drilldown unimplemented.

52. [[docs/backlog/2026-04-14-tag-system-not-searchable.md|tag-system-not-searchable]] — `open`, `2026-04-14`.
    Tags stored but not queryable. No listing endpoint, no filter, no faceting.

53. [[docs/backlog/2026-04-14-event-spine-no-integrity-checks.md|event-spine-no-integrity-checks]] — `open`, `2026-04-14`.
    raw-spine.jsonl has no checksums, no validation, no repair. Corruption silent.

54. [[docs/backlog/2026-04-14-obsidian-integration-one-way-only.md|obsidian-integration-one-way-only]] — `open`, `2026-04-14`.
    Export works. Import stubbed. No two-way sync. Vault structure hardcoded.

55. [[docs/backlog/2026-04-14-steal-from-inspiration-repos.md|steal-from-inspiration-repos]] — `open`, `2026-04-14`.
    Deep extraction from mempalace + supermemory. Benchmarks, dedup, entity graph, ingestion, room detection.

56. [[docs/backlog/2026-04-14-no-cross-harness-continuity-proof.md|no-cross-harness-continuity-proof]] — `open`, `2026-04-14`.
    No test of starting work in one harness, continuing in another.

57. [[docs/backlog/2026-04-14-contradiction-detection-never-triggers.md|contradiction-detection-never-triggers]] — `open`, `2026-04-14`.
    Contested status exists but contradiction detection never fires in practice.

58. [[docs/backlog/2026-04-14-trust-hierarchy-unproven.md|trust-hierarchy-unproven]] — `open`, `2026-04-14`.
    human > canonical > promoted > candidate defined but never proven E2E.

59. [[docs/backlog/2026-04-14-no-token-efficiency-measurement.md|no-token-efficiency-measurement]] — `open`, `2026-04-14`.
    No per-kind token tracking, no cost measurement, no delta-only capture.

60. [[docs/backlog/2026-04-14-no-public-benchmark-parity.md|no-public-benchmark-parity]] — `open`, `2026-04-14`.
    No LongMemEval/LoCoMo/MemBench results. Datasets exist, no working harness.

61. [[docs/backlog/2026-04-14-no-compaction-quality-proof.md|no-compaction-quality-proof]] — `open`, `2026-04-14`.
    Memory compaction exists but quality after compaction unproven.

62. [[docs/backlog/2026-04-14-no-decay-calibration.md|no-decay-calibration]] — `open`, `2026-04-14`.
    Decay hardcoded at 21d/0.12. Never calibrated from real usage data.

63. [[docs/backlog/2026-04-14-no-consolidation-quality-proof.md|no-consolidation-quality-proof]] — `open`, `2026-04-14`.
    Consolidation runs in worker. Output quality unmeasured.

64. [[docs/backlog/2026-04-14-no-handoff-quality-proof.md|no-handoff-quality-proof]] — `open`, `2026-04-14`.
    Handoff packet quality and completeness unverified.

65. [[docs/backlog/2026-04-14-no-overnight-evolution-loop.md|no-overnight-evolution-loop]] — `open`, `2026-04-14`.
    dream/autodream/autoevolve loops not implemented.

66. [[docs/backlog/2026-04-14-no-live-memory-contract.md|no-live-memory-contract]] — `open`, `2026-04-14`.
    Theory says memory updates while agent works. No live contract enforced.

67. [[docs/backlog/2026-04-14-skill-gating-config-flags-only.md|skill-gating-config-flags-only]] — `open`, `2026-04-14`.
    Skill gating is config flags. No runtime enforcement, no evaluation gate.

68. [[docs/backlog/2026-04-14-no-multi-user-team-support.md|no-multi-user-team-support]] — `open`, `2026-04-14`.
    No team/org concept beyond agent identity.

69. [[docs/backlog/2026-04-14-rag-sidecar-disabled-no-fallback.md|rag-sidecar-disabled-no-fallback]] — `open`, `2026-04-14`.
    RAG backend disabled. No timeout, retry, or fallback cache.

70. [[docs/backlog/2026-04-14-no-data-recovery-procedure.md|no-data-recovery-procedure]] — `open`, `2026-04-14`.
    SQLite corruption = total loss. No backup/restore procedure.

71. [[docs/backlog/2026-04-14-no-admission-control-rate-limiting.md|no-admission-control-rate-limiting]] — `open`, `2026-04-14`.
    No rate limiting on memory API. Noisy agents flood unchecked.

72. [[docs/backlog/2026-04-14-no-metrics-tracing-observability.md|no-metrics-tracing-observability]] — `open`, `2026-04-14`.
    No structured logging, no metrics, no tracing. Debug requires source reading.

73. [[docs/backlog/2026-04-14-no-multi-project-isolation-proof.md|no-multi-project-isolation-proof]] — `open`, `2026-04-14`.
    No test proving one project's memory doesn't leak into another.

74. [[docs/backlog/2026-04-14-no-latency-briefing.md|no-latency-briefing]] — `open`, `2026-04-14`.
    No retrieval latency measurement. No SLA or performance contract.

75. [[docs/backlog/2026-04-14-concurrent-write-no-retry.md|concurrent-write-no-retry]] — `open`, `2026-04-14`.
    CRITICAL: SQLITE_BUSY on concurrent writes. No retry/backoff. Multi-agent deadlock.

76. [[docs/backlog/2026-04-14-no-session-orphan-detection.md|no-session-orphan-detection]] — `open`, `2026-04-14`.
    No way to distinguish crashed vs completed sessions.

77. [[docs/backlog/2026-04-14-stale-working-memory-cache.md|stale-working-memory-cache]] — `open`, `2026-04-14`.
    Working memory can be hours old. Corrections silently ignored.

78. [[docs/backlog/2026-04-14-no-correction-audit-trail.md|no-correction-audit-trail]] — `open`, `2026-04-14`.
    Supersede mechanics don't log who changed what or why.

79. [[docs/backlog/2026-04-14-no-backward-compatibility-contract.md|no-backward-compatibility-contract]] — `open`, `2026-04-14`.
    Schema changes break old bundles. No migration or compat guarantee.

80. [[docs/backlog/2026-04-14-no-selective-memory-reset.md|no-selective-memory-reset]] — `open`, `2026-04-14`.
    Can only reset everything. No surgical correction of single item.

81. [[docs/backlog/2026-04-14-incomplete-transaction-rollback.md|incomplete-transaction-rollback]] — `open`, `2026-04-14`.
    Partial failures in checkpoint pipeline leave inconsistent DB state.

## Recently Closed (V1)

- `Phase A` raw truth spine: `verified` (architecture only)
- `Phase B` session continuity: `verified` (architecture only)
- `Phase C` typed memory: `verified` (architecture only)
- `Phase D` canonical truth: `verified` (architecture only)

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
