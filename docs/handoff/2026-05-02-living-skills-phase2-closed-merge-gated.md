---
opened: 2026-05-02
phase: living-skills-phase-2
status: complete-pending-merge-to-main
prev_handoff: 2026-05-02-v4-closed-phase2-queued-v5-remediation.md
branch: research/mining
upstream: origin/research/mining (7 unpushed commits this session)
next_step_a: user OK to merge research/mining → main (175 commits ahead — V3 tail + V4 + V5 + Living Skills Phase 1+2)
next_step_b: push research/mining (7 unpushed) before any merge
deferred:
  - substrate-bench reproducibility_script_matches_within_0_03_on_fresh_clone — flaked once on full workspace run, passed in isolation; pre-existing, not Phase 2 territory
  - working_memory_retrieval_p95_under_100ms perf flake — still G-phase territory, untouched
  - homelab :8787 server still pre-Phase-2 schema — live dogfood requires locally-built memd-server
  - salience population path — schema/sort shipped, but no current code stamps non-None values; deferred to a future phase
---

# Living Skills Phase 2 closed, merge to main gated on user OK

Living Skills Phase 2 is **complete on `research/mining`**. P2.1–P2.7 all
shipped. Tree clean. 7 commits unpushed this session; 175 ahead of `main`
total.

Phase 2 inverted the truth direction: records are now the source of truth,
the mirror is regenerable. `memd skill sync` reconstructs `.memd/skills/`
byte-stable from records. Wake renders Active Skills sorted by salience
desc, name asc on tie.

P2.6 (B6 distiller gating) collapsed to zero implementation work — the
consumption path doesn't exist. CandidateKind enum (Fact|Decision|Preference)
structurally cannot emit Skill, and the distiller has no memory read path.
Documented in P2.7 contract §11 as a structural guarantee.

## What landed (this session)

| # | Commit | Subject |
|---|--------|---------|
| 1 | `7871cfd` | fix(v5): decouple G5 aggregator + perf flake from host env |
| 2 | `571e6c2` | feat(F4.7): per-turn drift driver hook |
| 3 | `3b14e3f` | feat(P2.1): pure sync regenerator for skill mirror |
| 4 | `86c18ce` | feat(P2.3): record_id in frontmatter + render_skill_md as record content |
| 5 | `b029941` | feat(P2.4): skill retire default flip + --keep-record |
| 6 | `9b24e50` | feat(P2.2): memd skill sync CLI + apply_sync I/O applier + parser |
| 7 | `a012e73` | feat(P2.5): salience-ranked Active Skills section + canonical parser |
| 8 | `c89327d` | docs(P2.7): bump skill-record contract to v2 (records-as-truth) |

Plus pre-Phase-2 V5 remediation (#1) and F4.7 driver wiring (#2).

## Verify-green commands

```sh
# Targeted Phase 2 surfaces
cargo test -p memd-schema --lib                           # 42/42
cargo test -p memd-core --lib skill_mirror                # 20/20
cargo test -p memd-client --bin memd active_skills        # wake render
cargo test -p memd-client --bin memd skill_                # CLI + sync

# Full workspace (pre-existing flakes noted)
cargo fmt --all -- --check
cargo test --workspace 2>&1 | grep -E '^test result|FAIL' | tail
```

## Phase 2 contract (v2) highlights

`docs/contracts/skill-record.md` — full text. Key shifts from v1:

- `SkillFrontmatter` adds `record_id: Option<Uuid>` and `salience: Option<f32>`.
  `Eq` derive dropped (`Option<f32>` precludes it; PartialEq suffices for
  assert_eq! and serde).
- Records-as-truth: full `render_skill_md()` is the record content. Sync
  reconstructs the mirror without consulting disk state.
- `memd skill sync [--dry-run] [--prune]` documented. Pulls all
  `kind=Skill, status=Active` records across project/synced/global scopes,
  parses via `SkillBody::parse_skill_md`, regenerates mirror.
- `skill retire` default flipped: deletes the record by default;
  `--keep-record` is the opt-out (preserves history).
- Wake Active Skills sorts by salience desc, name asc on tie. Single
  canonical parser — `parse_skill_md` is the only YAML reader; the forked
  `split_skill_frontmatter` was retired.
- B6 boundary documented as structural rather than flag-gated.

## Known traps (Phase 2 specific)

- **Salience is schema-only**: every code path that constructs
  `SkillFrontmatter` writes `salience: None`. The field round-trips,
  the wake render sorts on it, but no production code stamps it yet.
  All-`None` skills sort alphabetically (Phase 1 behavior preserved).
  A future phase owns the population formula.
- **Retire default change is breaking**: scripts that relied on Phase 1
  retire (record kept implicitly) must add `--keep-record` explicitly
  or they will silently delete records.
- **Sync requires a running daemon**: unlike `list`/`show` which read disk
  directly, `memd skill sync` calls `client.search`. Offline operation
  is preserved for catalog discovery, not for sync.
- **substrate-bench flake** (pre-existing): one repro test
  (`reproducibility_script_matches_within_0_03_on_fresh_clone`) flaked
  once on the full workspace run during P2.5 verification, passed in
  isolation. Not Phase 2 territory; advisor confirmed: "note it, don't
  chase it."

## Merge gate (next step)

Merging `research/mining` → `main` is **destructive** (175 commits ahead
spanning V3 tail through Phase 2). Per global rule: do not merge without
explicit user OK. Do not merge while 7 commits are unpushed.

Suggested order on resume:
1. `git push origin research/mining` (7 commits ahead of `origin/research/mining`).
2. Confirm `cargo test --workspace` is green on a clean run.
3. Ask user: merge strategy (squash? merge commit? rebase?) and timing.
4. Execute on user OK.

## Truth-state at session close

- branch: `research/mining` clean, 7 commits ahead of upstream
- ahead of `main`: 175 commits
- Phase 2 contract: v2, shipped
- ROADMAP entry: not flipped this session — pickup task for next agent
- pre-existing flakes (substrate-bench repro, working_memory perf): known and noted
- homelab `:8787` server still pre-Phase-2: separate deploy task
