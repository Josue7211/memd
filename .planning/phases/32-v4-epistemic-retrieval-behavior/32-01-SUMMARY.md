# Phase 32 Summary: `v4` Epistemic Retrieval Behavior

## Completed

- added an epistemic ranking adjustment to context retrieval
- applied the same epistemic weighting to search ranking
- rewarded canonical and recently verified memory
- penalized synthetic and unverified memory when all else was equal
- added regression coverage for verified canonical vs unverified synthetic ranking

## Verification

- `cargo test -q -p memd-server`
- `cargo test -q`

## Outcome

Retrieval now prefers better-evidenced memory instead of following narrative
continuity alone.
