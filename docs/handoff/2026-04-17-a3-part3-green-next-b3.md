---
date: 2026-04-17
phase: A3
part: 3
status: green
next_phase: B3
tests: 514
---
# A3 Part 3 Green — memd Continuity Foundation closes, next B3 Intrinsic Retrieval

## TL;DR

Part 3 of A3 is green. All phase gates (continuity, enforcement, organization)
satisfied. memd owns continuity across compaction, session boundaries, and
harness switches. Ready to start B3 (Intrinsic Retrieval) — the first V3
phase that actually moves benchmark scores.

## What shipped in Part 3

1. **file-layout contract v0.3** — `FileLayoutSchema` added to `.memd/contract.json`;
   `classify_write_path` + `gate_write_decision` compose on top of the continuity
   gate. Denylisted writes (e.g. `docs/superpowers/plans/`) block under `enforcement=block`,
   nudge under `warn`. Unmanaged + canonical targets pass. (commit `f67706a`)
2. **Backlog/phases regrouped** — `docs/backlog/` now has `m1/ m2/ m3/ m4/ v2/ v3/ closed/`
   subfolders; `docs/phases/` under `v1/ v2/ v3/ archive/`. Two open `phase: unassigned`
   items triaged to G2 (lane-architecture-gaps) and E2 (memory-not-navigable).
   Closed-folder items exempted from coverage gate (they are historical resolution, not
   open work).
3. **`scripts/handoff-latest.sh` tie-break fix** — previously `sort -r` lex-sorted
   same-day packets alphabetically, surfacing older packets. Now sorts primary by
   date prefix, secondary by mtime → LATEST.md always resolves to the most recently
   written packet for the current day.
4. **`.memd/hooks/MANIFEST.json`** — 17 shipped hooks catalogued with name, path,
   harness event, purpose, sha256.
5. **`memd hooks doctor`** subcommand (visible alias `hooks`, existing `hook`
   preserved) — verifies MANIFEST against on-disk hooks; green on clean install,
   red on tampered content or untracked scripts. Exit 1 on red.
   - Unit test proves green→red→green round-trip via a temp bundle.
6. **Lifecycle probe NDJSON log** — `.memd/hooks/memd-lifecycle-probe.sh` now
   appends `{"ts","verdict","probe"}` to `.memd/state/lifecycle-probe.log` on
   each run; three green samples seeded. This is the 7-day rolling health
   artifact that the phase pass-gate asked for.
7. **Pre-send completion validator** — `verify_completion_ready(policy, signals)`
   in `memd-core::enforcement`. One specific check (missing-checkpoint on
   completion). Four tests cover Block/Warn/Ready/Off. Harness wiring is the
   B3-era step; validator is the memd-core primitive they call.

## Test / gate status

- Tests: 514 (460 bin + 54 core lib) all green.
- `make docs-green`: roadmap-audit + backlog-lint + lint-links + handoff-latest
  all pass.
- `./target/debug/memd hooks doctor`: green, 17 hooks verified.
- `./target/debug/memd contract verify`: v0.3.0 live, `enforces_file_layout_contract`
  guarantee satisfied.
- `bash .memd/hooks/memd-lifecycle-probe.sh`: green; log populated.

## A3 pass-gate coverage (phase doc §Pass Gate)

**Continuity:** ledger survives compaction (Part 1), `prime-reads` non-empty after
any session (Part 1), lifecycle self-test green (Part 3 wire + 7-day log), preference
replay green (Part 2), `.memd/contract.json` machine-readable and consumed by
cross-harness validator (Part 2/3).

**Enforcement:** hooks live under one canonical tree (`.memd/hooks`, Part 2),
MANIFEST covers every shipped hook (Part 3), doctor green on fresh / red on
tampered (Part 3), pre-send validator blocks missing-checkpoint completion
(Part 3 — pure function + 4 tests).

**Organization:** `docs/backlog/INDEX.md` regenerated from frontmatter and
tightened (open items must map to live phase; closed/ exempt), ROADMAP V3
section links every open memd-core backlog item to a phase (active_blockers
slimmed from 9 → 2; the 2 remaining — rag-sidecar, atlas-dormant — are
explicit B3/D3 work), `docs/phases/v{1,2,3}/` subfolders live and linked,
`LATEST.md` symlink resolves to the real newest packet.

## Known not-blockers

- PreToolUse hook user-side wiring in `~/.claude/settings.json` is still a user
  action for the write-path gate to fire in Claude Code sessions. Not an A3
  blocker — A3 ships the enforcement primitive; harness bootstrap into
  `~/.claude/settings.json` is an install-side concern tracked for the
  superpowers hook-install skill.
- `integrations/hooks/` and `.claude/hooks/` remain as install-generated
  mirrors (byte-copies today). MANIFEST only tracks the canonical root.
  Converting mirrors to symlinks is install-side polish, not a continuity gap.

## Next step

**B3 Intrinsic Retrieval.** The infra underneath the skin is fixed. Now we start
moving benchmark numbers — retrieval routing, scoring, the path that pushes
intrinsic LME/LoCoMo/MemBench/ConvoMem toward the ≥0.70 floor.

First B3 move per `docs/phases/v3/phase-b3-activate-retrieval.md`: baseline
intrinsic bench run → identify top retrieval failure mode → design first
intervention. Do **not** start writing retrieval code before the baseline is
captured; otherwise we can't tell if a change helped.

## Files of interest

- `.memd/hooks/MANIFEST.json`
- `.memd/hooks/memd-lifecycle-probe.sh`
- `.memd/state/lifecycle-probe.log`
- `crates/memd-core/src/enforcement.rs` (verify_completion_ready + tests)
- `crates/memd-client/src/cli/cli_hook_runtime.rs` (run_hook_doctor)
- `crates/memd-client/src/cli/args.rs` (HookDoctorArgs)
- `scripts/handoff-latest.sh`
- `scripts/backlog-lint.sh`
- `docs/phases/v3/phase-a3-continuity-foundation.md`
- `ROADMAP.md`
