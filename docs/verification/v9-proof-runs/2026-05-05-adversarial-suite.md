# V9 Adversarial Suite

- mode: `gate`
- generated_at: `2026-05-05T17:50:10.163719+00:00`
- scenarios: `8/8`
- negative controls: `8/8`
- SC: `6/10`
- CH: `6/10`
- composite: `5.60/10`

## Evidence

- `cargo test -p memd-server a9 -- --nocapture`
- `cargo test -p memd-server b9 -- --nocapture`
- `cargo test -p memd-server d9 -- --nocapture`
- shared multi-user fixtures under `crates/memd-client/fixtures/shared/multi-user/`
- `docs/contracts/federated-visibility-matrix.json`
