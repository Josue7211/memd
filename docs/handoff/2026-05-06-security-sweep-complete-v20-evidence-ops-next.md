---
opened: 2026-05-06
phase: v20-evidence-ops
status: security-sweep-complete-evidence-gates-pending
prev_handoff: 2026-05-06-v20-evidence-ops-started.md
branch: main
commit: 0353045
directive: freeze substrate scope; only blocker fixes before 1.0.0
mode: 10-star-ceo
---

# Security Sweep Complete - V20 Evidence Ops Next

One sentence: V20 substrate code is complete, scoped security sweep found one HIGH and it is fixed in `0353045`; evidence ops remains the only honest path to `1.0.0`.

## Current Truth

- Latest commit: `0353045 fix(security): harden zk proof verification`.
- Worktree was clean after commit.
- Scoped security sweep covered recent roadmap/docs plus widened V20 code commit.
- Findings: one HIGH in V19 `verify-zk`, fixed; no confirmed CRITICAL, MID, or LOW findings.
- V19 verifier now rejects forged claim hashes and non-hex commitments.
- V19 proof suite fixture now uses real generated commitments.
- V20 remains code complete with provisional 10.00/10, but not release-closed.
- No planned substrate code remains before `1.0.0`; only evidence-blocker/security fixes are allowed.

## Verification

- `cargo test -p memd-core v19 -- --nocapture` passed.
- `RUN_DATE=2026-05-06 scripts/verify/v19-zk-provenance-suite.sh` passed.
- `RUN_DATE=2026-05-06 scripts/verify/v20-release-suite.sh` passed.
- Forged CLI proof now returns `"verified": false`.
- `git diff --check` passed.

## Security Finding Closed

- HIGH: V19 `verify-zk` previously accepted arbitrary 64-character commitments.
- Exploit shape: forged proof with `aaaaaaaa...`, `bbbbbbbb...`, `cccccccc...`, `dddddddd...` could verify before the fix.
- Fix: validate 64-character hex and recompute `public_claim_hash` from `claim_id`, `pre_commitment`, `post_commitment`, and `relation_commitment`.
- Regression tests: forged claim hash and non-hex commitment rejection.

## Next Actions

- Keep evidence ops active from `docs/verification/release-1-0-0/2026-05-06-evidence-ops-start.md`.
- Enroll three real users and three harness-user pairs.
- Put three devices on current `main`.
- Assign V19 external auditor and V20 third-party replay reviewer.
- Add weekly evidence review note by 2026-05-13.
- Run full whole-repo security + dependency + secret sweep before any `1.0.0` tag.
- Do not start V21 before honest `1.0.0` close; `docs/plans/2026-05-06-v21-v25-ceo-execution-plan.md` stays deferred strategy.

## Hard Stop

Do not tag `1.0.0` until dated real-user, external auditor, and third-party replay artifacts land. Earliest honest close remains 2026-08-06 if all windows pass.
