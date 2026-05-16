# V11 Compiler SOTA Suite

- generated_at: `2026-05-12T18:44:08.852865+00:00`
- scenarios: `7/7`
- negative controls: `4/4`
- composite: `6.95/10`
- wake median tokens: `1480`
- silent correction latency: `900 ms`
- cost target respected: `true`

## Axes

- SC `8/10`: project-aware wake + compaction-aware recall
- CR `7/10`: silent-correction detection <= 1 s
- TE `7/10`: dynamic compiler + cost target + wake median <= 1500
- PR `6/10`: unchanged from V10
- CH `6/10`: unchanged from V9
- RR `8/10`: unchanged from V10
- TP `6/10`: unchanged from V8

## Evidence

- `docs/verification/v11-proof-runs/2026-05-12-compiler-sota-suite.ndjson`
- `docs/verification/v11-proof-runs/2026-05-12-negative-controls.ndjson`
- `docs/verification/v11-proof-runs/2026-05-12-axis-evidence/`
- `cargo test -p memd-core v11 -- --nocapture`
- `cargo test -p memd-server v11_schema -- --nocapture`
