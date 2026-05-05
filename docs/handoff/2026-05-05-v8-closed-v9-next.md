---
opened: 2026-05-05
phase: v8-close
status: handoff-ready
prev_handoff: 2026-05-04-v7-closed-v8-next.md
branch: main
v8_feature_head: a2ef015
handoff_head: top commit `docs(handoff): prepare V8 close packet`
repo_state: clean on main; ahead origin/main by 5 before this handoff commit
next_step_a: push handoff stack if remote sync is desired
next_step_b: start V9 multi-user/team from V8 internal close
public_gate_note: external stranger-review artifacts remain pending if public-review gate is required
---

# V8 Closed Internal - V9 Next

One sentence: V8 operator surfaces closed internally at composite `5.10/10`;
`main` is clean locally, ahead `origin/main`, and next work starts V9
multi-user/team memory.

## Pickup

```bash
cd /Volumes/T7/memd
git switch main
git status --short --branch
sed -n '1,180p' docs/handoff/LATEST.md
```

Expected pickup state after this packet: `main...origin/main [ahead 6]` clean.
If remote sync is needed, push the six local commits.

## Closed Gates

- A8-F8: operator console at `/operator` now surfaces atlas navigation,
  correction preview, memory inspector, provenance depth 3, cost ledger,
  rollback audit, and transparency/public proof panels.
- G8: `memd configure` canonical settings CLI landed with `list`, `get`, `set`,
  `reset`, and `show-schema`.
- G8 proof harness landed as a repeatable script and captured TE + TP evidence.
- Scorecard and roadmap moved to V8 internal close / V9 entry.

## Verification

- `cargo fmt --check` -> passed.
- `git diff --check` -> passed.
- `cargo test -p memd-client configure -- --nocapture` -> passed
  (2 filtered tests matched).
- `scripts/verify/v8-operator-proof.sh` -> passed.
- Proof NDJSON:
  `docs/verification/v8-runs/ui/operator/2026-05-05-g8-proof.ndjson`
- Screenshots:
  `docs/verification/v8-runs/ui/operator/operator-desktop.png`
  and `docs/verification/v8-runs/ui/operator/operator-mobile.png`

## V8 Proof Metrics

```json
{"cost_ledger_visible":true,"budget_tunable":true,"budget_cap_after_edit":2000}
{"provenance_depth_max":3,"correction_history_visible":true,"alternate_candidates_visible":true}
{"continuity_data_visible":true}
{"configure_suite":{"pass_count":7,"fail_count":0}}
{"console_errors":0,"memory_inspector_filter_visible_nodes":1}
```

## Caveats

- V8 is closed as internal/repo-owned proof.
- External stranger-review artifacts are still pending if a public-review gate is
  required. Do not fabricate outside reviewer evidence or screencasts.
- Local `main` has not been pushed after the V8 stack in this session.

## Next

V9 owns multi-user/team memory:

- shared namespaces
- visibility/ACL honored by retrieval
- merge collision governor
- hive divergence receipts
- multi-agent handoff quality
- team-wide correction propagation

Start by reading the V9 roadmap block and phase docs, then keep commits atomic.

## Commit Stack On Main

- `0f9be88` `feat(g8): add canonical configure settings CLI`
- `4b00653` `feat(v8): add operator surfaces UI`
- `0f7f4e5` `test(g8): add repeatable operator proof harness`
- `d49688b` `docs(v8): close operator surfaces milestone`
- `a2ef015` `test(v8): refresh operator proof screenshots`
