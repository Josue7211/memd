# Where Am I

Use this after session clear or when an agent feels lost.

## Read In This Order

1. [[ROADMAP]]
2. [[docs/handoff/2026-05-04-v6-closed-v7-next.md|V6 closed / V7 next handoff packet (2026-05-04)]]
3. backlog items linked from `ROADMAP`
4. active milestone note: [[docs/verification/milestones/MILESTONE-v7.md]]

## Current Truth

- active version: `v7` (Correction + Behavior-Change E2E)
- active milestone: `V7: Correction + Behavior-Change E2E`
- active phase: `V7-entry` — V6 typed-ingest closed at composite `4.45/10`; start V7 from clean `main`.
- next step: create a V7 branch from `main`, then prove correction behavior-change across a session boundary.
- V7 execution order: A7 → B7 → C7 → D7 → E7 → F7 → G7 → H7 (capture validation → canonical promotion → next-session behavior → contradiction detection → provenance trail → learned surface → rollback → atomic commit guard).
- main blocker: V7 not started yet; first hard proof is C7 `next_session_behavior_rate >= 0.05` with E7 provenance chain completeness `1.000`.
- v2/M4 status: K2 + L2 done; I2 + M2-evo + N2 paused. Resume after V3 or cherry-pick when needed.

## If You Need More Detail

- harness behavior: [[docs/core/setup.md|Setup]]
- detailed roadmap theory: [[2026-04-11-memd-ralph-roadmap]]
- canonical theory: [[2026-04-11-memd-canonical-theory-synthesis]]
- authoring conventions + where files go: see `docs/README.md` (if present) or the `## Process` section in [[ROADMAP]]

## Rule

If this file, `ROADMAP`, and live memd continuity disagree, fix the docs or the
memory immediately. Do not invent a third truth surface.
