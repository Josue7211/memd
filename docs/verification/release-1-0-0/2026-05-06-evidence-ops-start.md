# 1.0.0 Evidence Ops Start - 2026-05-06

Status: evidence clock opened; no 1.0.0 close until dated artifacts land

## Position

V20 code and synthetic proof substrate are complete. No planned substrate code
remains before 1.0.0. From here, code changes are limited to blocker fixes found
by dogfood, external replay, auditor review, or public bench rerun evidence.

## Evidence Windows

| Gate | Start | Earliest honest close | Evidence needed |
| --- | --- | --- | --- |
| V14 telemetry dogfood | 2026-05-06 | 2026-06-05 | >=30 days, >=3 real users, telemetry export + report |
| V15 self-tuning dogfood | 2026-05-06 | 2026-07-05 | >=60 days, >=3 harness-user pairs, profile deltas + rollback proof |
| V16 sync dogfood | 2026-05-06 | 2026-08-04 | >=90 days, 3 devices, conflict/visibility logs |
| V17 marketplace dogfood | 2026-05-06 | 2026-06-05 | >=30 days, cross-user installs, leakage audit |
| V18 correction graph dogfood | 2026-05-06 | 2026-08-06 | >=3 calendar months, >=50 multi-hop chains |
| V19 external auditor smoke | 2026-05-06 | pending auditor | audit export, verifier transcript, tamper check |
| V20 third-party replay | 2026-05-06 | pending reviewer | replay bundle, independent rerun transcript |

Earliest honest 1.0.0 close: 2026-08-06, assuming every real gate passes.

## Operating Rules

- Freeze substrate scope.
- Fix only blockers revealed by evidence.
- Keep weekly evidence review notes in this directory.
- Do not tag `1.0.0` from synthetic proof alone.
- Every accepted gate needs dated artifacts, commands, and reviewer/user count.

## First-Week Actions

1. Enroll three real dogfood users and three harness-user pairs.
2. Put at least three devices on current `main` for V16.
3. Run marketplace install/share flow across at least three users.
4. Start collecting correction graph chains with chain ids.
5. Assign V19 auditor and V20 replay reviewer.
6. Add first weekly review note by 2026-05-13.

## 2026-05-26 Internal Alpha Hive/Hivemind Validation

Status: internal owner-smoke passed; this is not external dogfood credit and does not close 1.0.0.

Evidence artifacts: `docs/verification/hive-runs/2026-05-26-internal-alpha/`

Verified commands:

```bash
CARGO_TARGET_DIR=/mnt/t7/cargo-target cargo test -p memd-client hive_coordination_tests -- --nocapture
CARGO_TARGET_DIR=/mnt/t7/cargo-target cargo test -p memd-client awareness_hive_tests -- --nocapture
MEMD_CARGO_TARGET_DIR=/mnt/t7/cargo-target CARGO_TARGET_DIR=/mnt/t7/cargo-target \
  MEMD_HIVE_PROOF_KEEP_TMP=1 scripts/verify/hive-production-proof.sh --tailscale-canary
```

Observed result:

- `hive_coordination_tests`: 42 passed, 0 failed.
- `awareness_hive_tests`: 51 passed, 0 failed.
- `hive-production-proof.sh --tailscale-canary`: exit 0.
- Local isolated hive proof exercised roster/follow, messages/inbox/ack, task assignment/help/review, handoff receipts, claim acquire/transfer/release, dev-server lease hard-block/board visibility, lane collision rejection, and hive-join reroute.
- Shared authority canary passed against `http://100.104.154.24:8788` under an isolated `hive-canary-*` namespace and cleaned up sessions/claims.

Scope note: this upgrades hive/hivemind internal validation from failing to owner-smoke green, but real-user dogfood, elapsed-window, multi-device sync, external auditor, and third-party replay gates remain open.

