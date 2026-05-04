---
opened: 2026-05-04
phase: v7-close
status: v7-closed-v8-next
prev_handoff: 2026-05-04-v6-closed-v7-next.md
branch: codex/v7-correction-e2e
v7_close_commit: this handoff commit
repo_state: clean after commit
next_step_a: start V8 operator surfaces from clean V7 close
next_step_b: keep H7 default-on auto-commit toggle visible in V8 configure work
---

# V7 Closed - V8 Next

One sentence: V7 closes correction behavior-change E2E at composite `4.90/10`;
next session starts V8 operator surfaces.

## Closed Gates

- A7/B7: `correct_item` now writes first-class `correction_meta`
  (`corrects_id`, `source_turn`, `captured_by`, `confidence`) on the canonical
  replacement item.
- C7/E7/G7: `correction-behavior-change` substrate suite proves S2 uses the
  corrected value and S3 rollback returns to the original value while preserving
  the correction chain.
- D7: contradiction detection remains green via
  `d2_contradiction_marks_siblings_contested`.
- F7: explain summary surfaces a learned-from correction trail.
- H7: bundle config has `auto_commit.enabled` default-on; `memd configure
  auto_commit.enabled=false` toggles it off; CLI memory/handoff/checkpoint write
  paths auto-commit tracked dirty host-repo files before writes.

## Verification

- `cargo test -p memd-client v7_ -- --nocapture` -> 4 passed, 1 ignored.
- `cargo test -p memd-client v7_real_backend_correction_behavior_change_and_meta -- --ignored --nocapture` -> 1 passed.
- `cargo test -p memd-client explain_summary_surfaces_correction_learning_trail -- --nocapture` -> 1 passed.
- `cargo test -p memd-client git_auto_commit -- --nocapture` -> 2 passed.
- `cargo test -p memd-schema correction -- --nocapture` -> 3 passed.
- `cargo test -p memd-server correct_item_ -- --nocapture` -> 6 passed.
- `cargo test -p memd-server explain_shows_correction_events -- --nocapture` -> 1 passed.
- `cargo test -p memd-server d2_contradiction_marks_siblings_contested -- --nocapture` -> 1 passed.
- `cargo build --bin memd-server` -> passed.
- `MEMD_AUTO_COMMIT_ENABLED=false cargo run -p memd-client --bin memd -- benchmark substrate --suite correction-behavior-change --output /tmp/memd-v7-substrate --report /tmp/memd-v7-substrate/SUBSTRATE_BENCHMARKS.md --json` -> S2 and rollback rows pass.
- Temp config smoke: `memd configure --output /tmp/memd-v7-config auto_commit.enabled=false --summary` -> reports `auto_commit=off`.
- `git diff --check` -> clean.
- `cargo fmt --check` could not run because `cargo-fmt` is not installed for `stable-aarch64-apple-darwin`.

## V6 Commit Check

V6 was committed in the prior handoff. Main/origin had the V6 close commit
`68d37cf`; the uncommitted work found on pickup was V7 work, not leftover V6.

## Next

V8 owns operator surfaces: atlas, corrections, provenance, diff, rollback, and
configuration UX. Start from clean V7 head and keep `auto_commit.enabled`
visible, because V8 G8 expands `memd configure` into the full settings surface.
