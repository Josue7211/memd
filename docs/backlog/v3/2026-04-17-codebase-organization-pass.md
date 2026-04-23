---
status: open
severity: medium
phase: A3
opened: 2026-04-17
scope: repo-structure, docs, integrations
---
# Codebase Organization Pass

- status: `open`
- severity: `medium`
- phase: new (V3-adjacent organization phase, or rolled into V2-N2)
- opened: `2026-04-17`
- scope: repo-structure, docs, integrations

## Problem

The codebase has grown faster than the layout. User quote (2026-04-17):
*"the codebase has to be more organized"*. Multiple symptoms visible right now:

- three parallel hooks directories (see
  `2026-04-17-hooks-scattered-across-three-dirs.md`)
- 84 backlog items in a flat `docs/backlog/` — no tagging by phase /
  milestone / severity, forcing full scans to find relevant items
- `docs/phases/` has V1, V2, and V3 phase docs intermixed with no
  milestone-level grouping
- `docs/handoff/` is chronological with no pointer to "the live handoff" —
  every new session has to open the latest file to find out
- `docs/plans/` holds M1/M2/M3/M4 execution plans plus ad-hoc plans; unclear
  which are still authoritative
- `docs/theory/` mixes models (`theory/models/`), donor extractions, and
  loose theory notes — no rubric for where a new doc goes
- `.memd/` top-level is flat and dense (23+ entries at root); knowing which
  are state vs config vs build output requires reading each

## Evidence

- `ls /home/josue/Documents/projects/memd/.memd` returns 23 top-level entries
  of mixed purpose (agents, backend.env, benchmarks, compiled, config.json,
  db files, env files, evals, events markers, evolution, experiments, hooks,
  lanes, local-server, loops, memd.db, memd.sqlite, models, rag-sidecar
  artifacts, scenarios, state, telemetry, verification, wake.md)
- `ls docs/backlog | wc -l` ≈ 84; no indexing doc beyond prose summary in
  `docs/verification/MEMD-10-STAR.md`
- `ls docs/phases` mixes `phase-a2-*`, `phase-b2-*`, `phase-a3-*`, `phase-b3-*`
  etc — no subfolder per version

## Fix (proposed shape; confirm before executing)

1. **docs/backlog/** → group by milestone: `docs/backlog/m1/`, `m2/`, `m3/`,
   `m4/`, `v3/`, `unassigned/`. Add `docs/backlog/INDEX.md` regenerated from
   frontmatter so search-by-phase is cheap. Frontmatter must include
   `milestone:`, `phase:`, `severity:`, `status:`.
2. **docs/phases/** → subfolder per version: `docs/phases/v1/`, `v2/`, `v3/`.
   Update internal wiki-links (there are many).
3. **docs/handoff/** → add `docs/handoff/LATEST.md` symlink (or
   `docs/handoff/INDEX.md`) pointing at the current live packet; update on
   every new handoff.
4. **docs/plans/** → mark superseded plans explicitly via frontmatter
   `status: superseded` and move to `docs/plans/archive/`.
5. **.memd/** → group under `state/`, `config/`, `artifacts/`, `docs/` so
   top-level shows six-ish entries, not twenty-three.
6. **integrations/hooks/** + **.memd/hooks/** + **.claude/hooks/** → see
   `2026-04-17-hooks-scattered-across-three-dirs.md`.
7. **docs/theory/** → `theory/models/` stays, `theory/donors/` for donor
   extraction pack, `theory/notes/` for loose theory.

## Acceptance

- repo tree diff reviewable in a single PR (large diff expected, mostly moves)
- all wiki-links inside the repo still resolve; `make lint-links` green
- wake packet surfaces an up-to-date `## Layout Map` so future sessions don't
  rediscover locations
- a new contributor can read `docs/README.md` once and know where to put a
  new backlog item, phase doc, handoff packet, plan, or theory note

## Risk

- big-bang reorg mid-V3 could destabilize execution; do this as an inter-phase
  seam (end of A3, start of B3) rather than mid-phase
- many wiki-links inside `.memd/lanes/architecture/*` point at `docs/` paths;
  moving without updating those references breaks donor anchors

## Relationship to other items

- blocks or is blocked by `2026-04-17-hooks-scattered-across-three-dirs.md`
  (do hooks first; it is the smallest, highest-signal cleanup)
- feeds into `2026-04-17-memd-process-too-soft-cross-harness.md` (clean
  layout is the precondition for mechanical enforcement)
