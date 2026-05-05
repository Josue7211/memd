# V10 Self-Improvement Suite

- generated_at: `2026-05-05T23:04:49.213198+00:00`
- scenarios: `7/7`
- negative controls: `4/4`
- composite: `6.40/10`
- production floor: `true`
- 0.1.0 tagged: `false`

## Axes

- SC `7/10`: missed-correction detector + reingest candidate
- CR `6/10`: cross-session auto-apply
- PR `6/10`: routine detect/store/invoke/measure/prune
- CH `6/10`: V9 proof preserved
- RR `8/10`: 30-day retrieval feedback loop with delta cap
- TE `5/10`: V8 proof preserved; contingency armed
- TP `6/10`: V8 proof preserved; A10/B10 provenance pointers

## Evidence

- `docs/verification/v10-proof-runs/2026-05-05-self-improvement-suite.ndjson`
- `docs/verification/v10-proof-runs/2026-05-05-negative-controls.ndjson`
- `docs/verification/v10-proof-runs/2026-05-05-axis-evidence/`
- `cargo test -p memd-core missed_correction -- --nocapture`
- `cargo test -p memd-core auto_apply -- --nocapture`
- `cargo test -p memd-core routine -- --nocapture`
- `cargo test -p memd-core feedback_loop -- --nocapture`
