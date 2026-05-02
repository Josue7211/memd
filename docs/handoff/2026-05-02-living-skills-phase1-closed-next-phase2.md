---
opened: 2026-05-02
phase: living-skills-phase-1
status: complete-pushed-pending-merge
prev_handoff: 2026-04-25-g4-harness-built-watch-active-next-v5-or-may2-close.md
branch: research/mining
upstream: origin/research/mining (set + pushed this session)
next_step_a: open Phase 2 plan — records-as-truth resync (`memd skill sync`), retire-deletes-record default, B6 distiller gating, ranked Active Skills
next_step_b: pick up V4 G4.7 close work — 7-day CI watch closes today (2026-05-02), composite rescore via G4.4 regenerator (gate ≥3.45)
parallel_track_caveat: Living Skills is independent of V4/V5; closing one does not block the other
deferred:
  - working_memory_retrieval_p95_under_100ms perf flake (pre-Phase-1 at e59b4d8, p95=512 vs 500 debug gate) — G-phase territory, not Phase 1's gap
  - homelab :8787 server still ships pre-Phase-1 schema; live dogfood requires locally-built memd-server
---

# Living Skills Phase 1 closed, Phase 2 unblocked, V4 G4.7 close-day arrived

Living Skills Phase 1 is **complete and pushed** on `research/mining`. Final
verification F.1–F.4 ticked. ROADMAP flipped from `in flight` to
`complete 2026-05-02`. Tree clean, upstream tracking set.

V4 G4.7 close-day is **today** — separate parallel track. The 7-day CI watch
opened 2026-04-25 closes 2026-05-02; composite rescore via the G4.4
regenerator is the gating ritual. Not done in this session — picked up by
the next agent.

## What landed (this session, 2026-05-02)

| # | Commit | Subject |
|---|--------|---------|
| 1 | `bd98b71` | feat(skills): tighten name validator to contract pattern + cache back-compat |
| 2 | `fb115d0` | docs(bench): seed SUBSTRATE_BENCHMARKS.md with provenance-integrity row |
| 3 | `2459f42` | docs(skills): tick phase 1 final verification (F.1–F.4) |
| 4 | `4cd35fa` | docs(roadmap): living skills phase 1 complete (research/mining) |

All four pushed to `origin/research/mining`. Phase 1 closing commit range
spans `babea58` (schema kind) → `4cd35fa` (roadmap flip).

## Verify-green commands

```sh
# Workspace tests (1 pre-existing perf flake noted)
cargo test --workspace 2>&1 | grep -E '^test result|FAIL' | tail

# Skill-specific
cargo test -p memd-core --lib skill_mirror
cargo test -p memd-client --bin memd cache_deserializes_pre_phase1
cargo test -p memd-client --bin memd skill_workflow

# Live dogfood (requires locally-built server — homelab :8787 is pre-Phase-1)
cargo build --release -p memd-server
MEMD_BIND_ADDR=127.0.0.1:18800 \
MEMD_DB_PATH=/tmp/memd-skills-dogfood.db \
MEMD_RATE_LIMIT_DISABLED=1 \
./target/release/memd-server &
./target/debug/memd --base-url http://127.0.0.1:18800 skill add \
  --name <slug> --description '<oneline>' --body-file <path.md> --output .memd
./target/debug/memd --base-url http://127.0.0.1:18800 wake --output .memd | grep -A8 "Active Skills"
./target/debug/memd --base-url http://127.0.0.1:18800 skill retire --name <slug> --output .memd
```

## Known traps

- **Cache poisoning**: any cross-version dogfood writes `kind=skill` into
  `.memd/state/resume-snapshot-cache.json`. The system `~/.local/bin/memd`
  is pre-Phase-1 and refuses to deserialize the cache after that. Fix:
  `rm .memd/state/resume-snapshot-cache.json` (regenerable, not tracked)
  OR install the new debug build into `~/.local/bin/memd` first. Memory'd
  to memd as a fact this session.
- **Wake on-disk vs streamed**: `memd wake` streams the wake-up to stdout.
  The on-disk `.memd/wake.md` is updated by the harness UserPromptSubmit
  hook, not this command. Don't `grep wake.md` to verify the dogfood —
  use the stdout pipe.
- **--output flag position**: must come AFTER `skill add` (or any skill
  subcommand). The pre-existing plan body had it before `skill add`,
  which fails with `unexpected argument '--output'`. Plan now corrected.
- **Phase 1 retire scope**: contract §6 + §10 say `skill retire` removes
  the mirror but leaves the record (returns
  `{"note":"record retirement pending (Phase 2)","retired":"<name>"}`).
  Don't read this as a bug. Phase 2 owns record-side retirement.

## Phase 2 entry handoff (Living Skills track)

ROADMAP entry already drafted:

> **Phase 2 (queued)** — Records-as-truth: `memd skill sync` regenerates
> the mirror from records, retire deletes record by default, B6 distiller
> treats skill records as semantic candidates with explicit gating, ranked
> Active Skills surface keyed on salience.

Plan doc not yet written. Suggested file:
`docs/superpowers/plans/<YYYY-MM-DD>-living-skills-phase2-records-as-truth.md`.

Phase 2 leans on three Phase 1 facts already in place:
1. `SkillCatalogEntry::record_id: Option<Uuid>` — round-trip ready.
2. `parse_skill_metadata` already extracts `record_id:` from frontmatter.
3. Mirror writer is atomic + name-validated; sync can rewrite without
   a separate validator pass.

Phase 2 should land:
- `memd skill sync` (records → mirror, with idempotent overwrite)
- `record_id` written into frontmatter on `skill add` (currently absent —
  contract §8 anticipates it but Phase 1 add does not write it)
- B6 distiller skill-record handling + explicit gating
- Salience-ranked Active Skills section in wake (replace alphabetical sort)
- `skill retire` default flips to delete-record (currently the soft note)

## V4 G4.7 close (parallel — pick up today)

Per the 2026-04-25 handoff, V4 close gates are:

1. **7-day CI nightly watch** closes today (2026-05-02). Inspect
   `.github/workflows/v4-proof-harness.yml` runs since 2026-04-25.
2. **Dogfood harvest** D4.8 / E4.7 / F4.7 — earliest 2026-05-01,
   read `.memd/logs/preference-drift.ndjson`.
3. **Composite rescore** via `scorecard::regenerate_scorecard()`
   against harvested NDJSON. Gate ≥ **3.45**.

Living Skills did not consume V4 close budget — it is a parallel track
on the same branch. Today's session lifted Phase 1 to `complete` but
left V4 G4.7 untouched.

## Truth-state at session close

- branch: `research/mining` clean, pushed, upstream `origin/research/mining`
- ahead of `main`: 161 commits (V3 tail + V4 + V5 + Living Skills Phase 1)
- ROADMAP `current_phase: G5`, `phase_status: watch-active` — unchanged
- Phase 1 Living Skills entry: `complete 2026-05-02`
- pre-existing perf flake known and memory'd
- homelab `:8787` server pre-Phase-1 schema known (deploy task)
