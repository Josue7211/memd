---
date: 2026-04-22
phase: post-V3 planning
status: roadmap-v4-v10-seeded
next_phase: V4 full plan spec (A4-G4 implementation spec + test fixtures)
---

# V4–V10 Roadmap Seeded. Next: Full Plan Spec for V4.

## Why this handoff exists

V3 ships retrieval honesty. Public-bench canonical numbers expose a **generator-reasoning cap**, not a memd retrieval cap — retrieval diagnostics remain ≥0.95 on every bench. Six of seven 10-STAR axes have not moved since 2026-04-14; current composite regrades to **2.15/10**. Further public-bench tuning lifts one axis at most. The substrate win is untouched.

V4–V10 roadmap was written in commit `268b4a3` to route the path back to substrate quality. V4 phase docs A4–G4 are drafted; they carry goal, deliverables, pass gate, evidence, fail/rollback per V3 template. **They are not yet full implementation specs.** A next agent must take V4 from "phase docs" to "phase 1 plan" — the level of detail that lets a code agent execute without re-deriving architecture.

## What landed today (2026-04-22)

Three commits on `research/mining` after K3 active work:

- `268b4a3 docs(roadmap): V4-V10 milestone map + V4 Live Loop Repair phase docs` — `ROADMAP.md` "V4–V10: The Path to 10-STAR" block, milestone audit stubs `docs/verification/milestones/MILESTONE-v{4..10}.md`, V4 phase docs `docs/phases/v4/phase-{a4..g4}-*.md`.
- `09b5694 fix(bench/full_eval): 300s timeout + 3-attempt exponential backoff on generator calls` — K3 generator hardening from `crates/memd-client/src/benchmark/full_eval.rs`.
- `9b9ca23 docs(backlog): two harness-bridge gaps + regenerated INDEX` — codex hooks missing + HARNESS_BRIDGES inverted report backlog items.
- `b5cf3be docs(bench): regenerate PUBLIC_BENCHMARKS.md with latest 4 K3 runs` — auto-regenerated bench doc.

Tree clean at `b5cf3be`.

## V4–V10 composite targets (zero-generosity 10-STAR rescore basis)

| Milestone | Scope | Composite |
| --- | --- | --- |
| V3 (current) | retrieval honesty, canonical metrics, leaderboard transparency | 2.15 |
| V4 Live Loop Repair | session continuity + correction + token efficiency | → 4.0 |
| V5 Substrate-Native Benches | memd-shaped bench suite (7 suites A5–G5) | → 5.5 |
| V6 Typed Ingest for Public Benches | episodic/semantic/canonical applied to LME/LoCoMo/MemBench/ConvoMem | → 7.0 |
| V7 Correction + Behavior-Change E2E | correction lane end-to-end, rollback | → 7.8 |
| V8 Operator Surfaces | atlas/correction/inspector/provenance/diff UIs | → 8.5 |
| V9 Multi-User / Team | shared-namespace, hive divergence, collision governor | → 9.0 |
| V10 Self-Improvement | overnight consolidation, auto-correction, bench canary | → 9.5+ |

## V4 phase inventory (already drafted)

Each doc at `docs/phases/v4/phase-{id}-*.md` carries goal, why-exists, deliver, pass gate (pre/post/evidence/regression), product win, evidence, fail conditions, rollback.

| Phase | Name | Owns axis |
| --- | --- | --- |
| A4 | Read-State Across Compaction | session continuity |
| B4 | Hook Contract Enforcement | session continuity |
| C4 | Correction Capture E2E | correction retention |
| D4 | Working-Context Compiler | token efficiency |
| E4 | Progressive-Depth Recall | token efficiency, cross-harness |
| F4 | Preference Replay + Drift Detection | correction retention |
| G4 | Session-Continuity Proof Harness | V4 completion gate |

## Next-agent task: write the V4 full plan spec

**Deliverable:** for each V4 phase, produce an implementation-grade plan spec at `docs/phases/v4/phase-{id}-plan.md` (new file, distinct from the existing phase doc). Each plan spec must be detailed enough that a code agent can execute without further architecture decisions.

**Each plan spec must include:**

1. **Surface area.** Exact files to touch, new files to create, crates affected, binaries rebuilt.
2. **Schema changes.** Any new record fields, migration plan, backward-compat posture.
3. **API shape.** New CLI flags, new endpoints, new hook contracts — signature-level.
4. **Test matrix.** Unit tests (which crate, which module), integration tests (fixture path, expected assertions), E2E scenarios.
5. **Fixtures.** What fixture data to generate, where it lives (`crates/memd-client/fixtures/` vs `.memd/benchmarks/`), how to regenerate.
6. **Telemetry.** New log files, NDJSON fields, counter names.
7. **Feature flags.** Exact env vars, default values, graduation criteria.
8. **Task list (executable).** Ordered step-by-step in TaskCreate format, each step sized ≤1 session of work, explicit acceptance criteria per step.
9. **Bench impact.** Which V5 substrate benches (once they exist) this phase lights up.
10. **Dependency graph.** Which V4 phases must land first, which can parallelize.

**Cross-phase deliverable:** one doc `docs/phases/v4/V4-INTEGRATION.md` covering:
- The 3-session dogfood scenario G4 executes — scripted turn-by-turn, with expected memd state at each cut.
- Shared test fixtures (session transcripts, preference sets, correction turns) reused across A4–G4.
- Hook-contract diffs vs current `.memd/hooks/` layout.
- 10-STAR scorecard re-run template (how G4 writes back to `docs/verification/MEMD-10-STAR.md`).

## Operational context the next agent needs

**Branch:** `research/mining` (not main). All V3 K3 work lives here; V4 specs land on same branch until V3 K3 closes.

**Rebuild:** `cargo build --release --target-dir /tmp/memd-target -p memd-client -p memd-server` — NFS target directory required (see project rules: NFS Cargo builds → `/tmp/<project>-target`).

**memd-server for dogfood/bench runs:** `MEMD_RATE_LIMIT_DISABLED=1 /tmp/memd-target/release/memd-server ...` — the SOFT=100/HARD=200/60s limiter in `crates/memd-server/src/rate_limit.rs` will 429 any real session ingest otherwise.

**codex-lb proxy (for any LLM-judge test fixture generation):** OAuth-backed at `http://127.0.0.1:2455`; use `$CODEX_LB_API_KEY` not user's raw OpenAI key. Models available: `gpt-5.4`, `gpt-5.4-mini`. No `gpt-4o` routes (see `docs/backlog/v3/2026-04-21-gpt4o-proxy-route-for-judge.md`).

**Still-pending K3 work (separate, parallel):**
- #29 update PUBLIC_LEADERBOARD.md with K3 judge-swap disclosure + new canonical numbers (LME 0.544, MemBench 0.533, ConvoMem 0.825)
- #26 LoCoMo full-eval — lost to reboot tmpfs wipe, needs rerun or explicit skip
- #30 diagnose why any bench runs below 0.90 (substrate-level, not a scoring artifact)
- #31 K3 close + roadmap update + this handoff (this handoff is #31's handoff artifact)

These are K3 close-out items, not V4 planning items. The next agent should **not** block V4 plan spec on K3 close — the two streams are parallelizable.

## Known operational gotchas that bite V4 work

- **tmpfs on CachyOS clears on reboot.** Any long-running bench output in `/tmp/` (including `/tmp/memd-target/`) vanishes. Persist state to `.memd/` or `docs/` before a reboot. Lost one LoCoMo 98/200 run this way on 2026-04-21.
- **Grader cache miss on rerun.** gpt-5.4-mini is not fully deterministic at T=0; cache keys hash prompt+response+model, so rerunning the same bench does not replay the grader — it re-calls. Budget accordingly.
- **HARNESS_BRIDGES.md is currently wrong.** The report inverts reality (see backlog `2026-04-22-harness-bridges-report-inverted.md`). Do not trust its `wired: yes/no` columns when planning V4 hook-contract work — read `~/.claude/settings.json` and `~/.codex/hooks.json` directly.
- **PUBLIC_LEADERBOARD.md auto-overwrite is disabled** (commit b244a7e landed this). Runtime no longer clobbers the hand-curated leaderboard. Do not re-introduce the auto-write path in any V4 code.

## Related docs

- `ROADMAP.md` — V4–V10 block.
- `docs/phases/v4/` — all V4 phase docs (7 files).
- `docs/verification/milestones/MILESTONE-v4.md` → `MILESTONE-v10.md` — milestone audit stubs with axis deltas + completion gates.
- `docs/verification/MEMD-10-STAR.md` — 10-STAR contract; V4+ must update the per-axis scorecard on milestone close.
- `docs/phases/v3/phase-j3-floor-verification.md` — V3 completion phase; references the phase-doc template V4 docs follow.

## Exit criteria for "next full plan spec" task

- 7 plan specs at `docs/phases/v4/phase-{a4..g4}-plan.md` exist.
- 1 integration doc at `docs/phases/v4/V4-INTEGRATION.md` exists.
- Each spec satisfies the 10-point checklist above.
- Dependency graph across A4–G4 is explicit and honored (A4 before G4, B4 before C4 per phase-doc `depends_on`).
- Atomic commits on `research/mining`, one per phase spec (7 commits) plus one for the integration doc.
- Final handoff: `docs/handoff/2026-MM-DD-v4-plan-spec-complete-next-execute.md` referencing the new specs.
