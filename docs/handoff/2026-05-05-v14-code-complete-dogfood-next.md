---
opened: 2026-05-05
phase: v14-code-complete
status: handoff-ready
prev_handoff: 2026-05-05-v13-closed-v14-next.md
branch: main
repo_state: pending commit at packet creation
next_step_a: run V14 real-user telemetry dogfood window for >=30 days with >=3 users
next_step_b: final-close V14 only after dogfood evidence is present
release_note: V14 telemetry substrate code complete; 8.60 composite remains provisional until dogfood close
---

# V14 Code Complete - Dogfood Next

One sentence: V14 telemetry foundation is implemented and proof-tested; only
the real-user 30-day dogfood gate remains before honest final close.

## Pickup

```bash
cd /Volumes/T7/projects/memd
git status --short --branch
sed -n '1,160p' docs/handoff/LATEST.md
```

Expected pickup state after commit: clean `main`, ahead `origin/main`.

## Landed

- `memd telemetry enable|disable|status|record|report|export`
- Local-first events at `.memd/telemetry/events.ndjson`
- Stable ULID-shaped per-user hashes
- PII scrubbers for emails, IPs, common tokens, and user home paths
- Bench export scope with deterministic tiny noise and no session IDs
- `memd configure` keys:
  - `telemetry.enabled`
  - `telemetry.retention_days`
  - `telemetry.export_scope`
- Wake cost ledger mirrors to telemetry only when enabled
- Disabled telemetry creates no orphaned telemetry state

## Verification

- `cargo fmt --check` -> passed.
- `cargo test -p memd-core telemetry -- --nocapture` -> passed.
- `cargo test -p memd-client telemetry_v14 -- --nocapture` -> passed.
- `RUN_DATE=2026-05-05 scripts/verify/v14-telemetry-suite.sh` -> passed.

## Proof Artifacts

- `docs/verification/v14-proof-runs/2026-05-05-telemetry-suite.ndjson`
- `docs/verification/v14-proof-runs/2026-05-05-telemetry-suite.md`

## Gate State

- V14 code/substrate: complete.
- TE proof marker: `7 -> 8`, composite `8.50 -> 8.60` provisional.
- Remaining blocker: real-user dogfood window (`>=30 days`, `>=3 users`).
- Do not mark V14 final-closed until that wall-clock evidence exists.

## Next

Enable telemetry for at least three dogfooders, let the window run, then rerun
the V14 suite against real exported telemetry and final-close V14.
