---
opened: 2026-05-05
phase: v9-close
status: handoff-ready
prev_handoff: 2026-05-05-v8-closed-v9-next.md
branch: main
handoff_head: top commit `docs(handoff): V9 closed V10 next`
repo_state: clean on main after handoff commit; ahead origin/main locally
next_step_a: push local stack if remote sync is desired
next_step_b: start V10 self-improvement from V9 close
public_gate_note: external stranger-review artifacts remain pending if public-review gate is required
---

# V9 Closed - V10 Next

One sentence: V9 multi-user/team memory closed at composite `5.60/10`;
`main` is local-only ahead of `origin/main`, and next work starts V10
self-improvement.

## Pickup

```bash
cd /Volumes/T7/memd
git switch main
git status --short --branch
sed -n '1,180p' docs/handoff/LATEST.md
```

Expected pickup state after this packet: clean `main`, ahead `origin/main`.
Push if remote sync is desired.

## Closed Gates

- A9-G9 phase plan specs exist and roadmap phase names align with milestone
  spec.
- A9: multi-user harness state contract, user-scoped DB identity columns,
  idempotent migration/backfill, and shared A→B→A fixtures landed.
- B9: Workspace visibility now requires a matching workspace before retrieval
  ranking.
- D9: content-hash dedup preserves co-authors via `memory_item_authors`.
- F9/G9: proof harness runs A9/B9/D9 substrate tests, validates all shared
  multi-user fixtures and visibility matrix, and writes dry-run + gate evidence.
- Scorecard and roadmap moved to V9 closed / V10 entry.

## Verification

- `cargo fmt --check` -> passed.
- `cargo test -p memd-server a9 -- --nocapture` -> passed (3 tests).
- `cargo test -p memd-server b9 -- --nocapture` -> passed (3 tests).
- `cargo test -p memd-server d9 -- --nocapture` -> passed (2 tests).
- `scripts/verify/v9-adversarial-dry-run.sh` -> passed.
- `scripts/verify/v9-adversarial-suite.sh` -> passed.
- `git diff --check` -> passed.

## Proof Artifacts

- F9 dry-run:
  `docs/verification/v9-runs/f9-dry-run.ndjson`
- G9 gate:
  `docs/verification/v9-proof-runs/2026-05-05-adversarial-suite.ndjson`
- G9 summary:
  `docs/verification/v9-proof-runs/2026-05-05-adversarial-suite.md`

## V9 Proof Metrics

```json
{"scenario_count":8,"pass_count":8,"fail_count":0}
{"negative_controls_fired":8}
{"session_continuity":6,"cross_harness":6}
{"non_owned_axes_unchanged":true}
{"composite":5.60}
```

## Caveats

- V9 closed as repo-owned proof using substrate tests and adversarial fixtures.
- External stranger-review artifacts are still pending if a public-review gate is
  required. Do not fabricate outside reviewer evidence or screencasts.
- Local `main` has not been pushed after the V9 stack in this session.

## Next

V10 owns self-improvement:

- overnight consolidation
- auto-correction from user behavior
- bench regression canary
- 10-STAR automated regeneration
- production-floor checks with every axis at least 3

Start by reading the V10 roadmap block and `docs/phases/v10/V10-INTEGRATION.md`.
Keep commits atomic and keep proof rerunnable.
