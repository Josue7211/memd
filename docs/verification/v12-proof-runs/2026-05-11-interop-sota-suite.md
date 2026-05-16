# V12 Interop SOTA Suite

- generated_at: `2026-05-12T03:18:45.704241+00:00`
- scenarios: `7/7`
- negative controls: `4/4`
- composite: `7.75/10`
- protocol parity delta: `0.0 <= 0.02`
- signed audit entries: `4`
- tamper detected: `true`

## Axes

- PR `8/10`: routine browse/edit/merge + compose + inheritance + cross-workspace export/import
- CH `8/10`: MCP/ACP/typed-channel envelope + dual-harness atomic session
- TP `8/10`: ed25519 signed audit + browse/explain + external tamper verification
- SC `8/10`: integrated from V11
- CR `7/10`: integrated from V11
- RR `8/10`: integrated from V10
- TE `7/10`: integrated from V11

## Evidence

- `docs/verification/v12-proof-runs/2026-05-11-interop-sota-suite.ndjson`
- `docs/verification/v12-proof-runs/2026-05-11-negative-controls.ndjson`
- `docs/verification/v12-proof-runs/2026-05-11-axis-evidence/`
- `cargo test -p memd-core v12 -- --nocapture`
- `cargo test -p memd-core routine::library -- --nocapture`
- `cargo test -p memd-core interop -- --nocapture`
- `cargo test -p memd-core audit -- --nocapture`
