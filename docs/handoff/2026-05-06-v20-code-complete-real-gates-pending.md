---
opened: 2026-05-06
phase: v20-code-complete-real-gates-pending
status: code-complete-handoff
prev_handoff: 2026-05-06-v16-to-v20-ceo-mode-execute-all.md
branch: main
repo_state: dirty until final commit
directive: preserve real dogfood gates; do not tag 1.0.0 yet
mode: 10-star-ceo
---

# V20 Code Complete - Real Gates Pending

One sentence: V16 through V20 proof substrates are landed and green; final
1.0.0 close is blocked only by real elapsed dogfood, external auditor, and
third-party replay evidence.

## What Landed

- V16 CRDT/sync substrate: `crates/memd-core/src/v16.rs`
- V17 routine marketplace substrate + CLI: `memd routines marketplace search|browse|install`
- V18 correction graph + detector/replay substrate
- V19 pragmatic ZK provenance substrate + `memd audit verify-zk <proof>`
- V20 aggregate release harness under `docs/verification/release-1-0-0/`
- Config keys: `sync.enabled`, `sync.relay_url`, `sync.conflict_policy`

## Proof Runs

- `scripts/verify/v16-sync-suite.sh`
- `scripts/verify/v17-routine-marketplace-suite.sh`
- `scripts/verify/v18-correction-graph-suite.sh`
- `scripts/verify/v19-zk-provenance-suite.sh`
- `scripts/verify/v20-release-suite.sh`

Artifacts:

- `docs/verification/v16-proof-runs/2026-05-06-sync-suite.md`
- `docs/verification/v17-proof-runs/2026-05-06-routine-marketplace-suite.md`
- `docs/verification/v18-proof-runs/2026-05-06-correction-graph-suite.md`
- `docs/verification/v19-proof-runs/2026-05-06-zk-provenance-suite.md`
- `docs/verification/release-1-0-0/2026-05-06-v20-release-suite.md`

## Gates Still Open

- V14: real >=30-day telemetry dogfood, >=3 users.
- V15: real >=60-day self-tuning window, >=3 harness-user pairs.
- V16: real >=90-day 3-device sync dogfood.
- V17: real >=30-day marketplace dogfood with cross-user installs.
- V18: real >=3-month dogfood with >=50 multi-hop correction chains.
- V19: external auditor smoke artifacts.
- V20: third-party replay for every axis.
- Public-review stranger artifacts remain separate public-review gate.

## Next

Do not lower the bar and do not cut `1.0.0` early. Start/collect the real
windows above, then rerun the V20 release suite and attach external replay
artifacts before final close.
