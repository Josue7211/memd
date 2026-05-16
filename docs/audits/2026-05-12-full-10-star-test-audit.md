# memd Full 10-Star Test Audit - 2026-05-12

## Verdict

memd is **locally green across the executable gates audited on 2026-05-12**.

It is still **not honestly 1.0.0 / permanent 10-star complete** until the real evidence clock closes: 30-90 day dogfood, 3 users, 3 devices, V19 external auditor, and V20 independent third-party replay. The V20 `10.00` result is valid as synthetic proof, not as final production truth.

## Current Green Gates

- `cargo fmt --check` - passed.
- `cargo test --workspace --quiet` - passed.
- `CARGO_BUILD_JOBS=1 cargo clippy --workspace --all-targets -- -D warnings` - passed.
- `npm run build` in `apps/` - passed.
- `npm run build` in `apps/dashboard/` - passed.
- `scripts/verify/v8-operator-proof.sh` - passed after Playwright Chromium was installed; wrote `docs/verification/v8-runs/ui/operator/2026-05-12-g8-proof.ndjson`.
- `scripts/verify/v9-adversarial-suite.sh` - passed.
- `scripts/verify/v10-self-improvement-suite.sh` - passed.
- `scripts/verify/v11-compiler-sota-suite.sh` - passed.
- `scripts/verify/v12-interop-sota-suite.sh` - passed.
- `scripts/verify/v13-release-suite.sh` - passed.
- `scripts/verify/v14-telemetry-suite.sh` - passed.
- `scripts/verify/v15-self-tuning-suite.sh` - passed.
- `scripts/verify/v16-sync-suite.sh` - passed.
- `scripts/verify/v17-routine-marketplace-suite.sh` - passed.
- `scripts/verify/v18-correction-graph-suite.sh` - passed.
- `scripts/verify/v19-zk-provenance-suite.sh` - passed.
- `scripts/verify/v20-release-suite.sh` - passed.
- `scripts/verify/full-10-star-audit.sh` - passed end to end and wrote `docs/verification/full-10-star-audit/2026-05-12/summary.json`.

## Fixes Landed From Red Audit

- Fixed macOS `/private/var` vs `/var` path canonicalization failures in bundle/retrieval/e2e tests.
- Fixed D5 substrate fixture gaps with deterministic fallback depth queries.
- Fixed public benchmark artifact-root mismatch in all-write refresh tests.
- Fixed V4 proof harness overclaim guard by freezing the historical V4 close scorecard.
- Fixed server TTL visibility and debug histogram p95 edge tests.
- Fixed schema clippy failures.
- Fixed broad formatter drift.
- Fixed V10 proof script drift so it checks historical V10 evidence instead of current live V13/V20 text.
- Installed Playwright Chromium and improved V8 Chromium-missing error text.
- Added and ran `scripts/verify/full-10-star-audit.sh` as a single local/proof gate runner.

## UI And API Smoke

- Isolated API smoke from the original audit passed: health, store, search, context, working memory, source, correction, workspace boundary, and explain.
- Astro operator proof passed in Chromium with `console_errors: 0` and desktop/mobile screenshots.
- React dashboard production build passed.
- React dashboard live browser smoke passed against an isolated memd-server on `127.0.0.1:3080`: 8 routes rendered with zero browser console errors.
- Dashboard degraded-dependency behavior was observed: without a backing API server, Vite preview logs proxy `ECONNREFUSED` while the UI still renders. This is safe-visible degradation, not a silent pass.
- `scripts/verify/hive-production-proof.sh` passed against an isolated local DB: hive join, message/ack, task assign/help/review, handoff receipt, claim acquire/transfer/release, lane collision rejection, and reroute.

## Feature Coverage Result

| Area | Audit Result |
| --- | --- |
| Core store/search/context/working | Passed via isolated smoke and workspace tests |
| Correction lifecycle | Passed via isolated smoke and V18 correction graph suite |
| Provenance/explain/source | Passed via isolated smoke, V8, V19 |
| Workspace isolation | Passed via smoke and V9 adversarial suite |
| Shared/cross-harness/hive | Passed in V9/V12/V13 proof suites and local `hive-production-proof.sh` |
| Frontend builds | Astro and dashboard passed |
| UI browser proof | Operator passed; dashboard live smoke passed |
| Full Rust tests | Passed |
| Formatting | Passed |
| Clippy strictness | Passed with `CARGO_BUILD_JOBS=1` |
| V8-V20 proof suites | Passed |
| Full evidence clock | Pending by design; cannot be faked locally |

## Evidence Generated

Fresh 2026-05-12 artifacts were written under:

- `docs/verification/v8-runs/ui/operator/`
- `docs/verification/v9-proof-runs/`
- `docs/verification/v10-proof-runs/`
- `docs/verification/v11-proof-runs/`
- `docs/verification/v12-proof-runs/`
- `docs/verification/v14-proof-runs/`
- `docs/verification/v15-proof-runs/`
- `docs/verification/v16-proof-runs/`
- `docs/verification/v17-proof-runs/`
- `docs/verification/v18-proof-runs/`
- `docs/verification/v19-proof-runs/`
- `docs/verification/release-0-1-0/`
- `docs/verification/release-1-0-0/`
- `docs/verification/full-10-star-audit/2026-05-12/`

## Remaining Honest Caveats

1. Real-world evidence windows are still open and remain release blockers for an honest final 10-star claim.
2. V20 remains synthetic proof until independent replay and auditor artifacts exist.
3. Clippy on this external drive should use `CARGO_BUILD_JOBS=1`; higher parallelism can be killed by memory pressure before lint completion.
4. The external T7 filesystem does not support the hardlink pattern Cargo tries in incremental caches, so Rust commands print hardlink fallback warnings. They do not indicate code failure.
5. Dashboard's graph chunk is large but under the current warning limit. It should stay on the performance backlog.

## Ranked Improvements

1. Run the full evidence clock through August 6, 2026 at minimum if every required real-user/device/auditor/replay packet lands on schedule.
2. Commit/standardize the dashboard live API smoke into a first-class proof artifact with screenshots and route metrics.
3. Run `scripts/verify/hive-production-proof.sh` regularly on isolated local DBs, and run the Tailscale canary only when shared-backend mutation is approved.
4. Add CI cache placement guidance for external drives to suppress Cargo hardlink fallback noise.
5. Split or lazy-load the dashboard graph chunk if performance budgets tighten.

## 10-Star Status

Current honest status: **local/proof gates green, final 10-star evidence still pending**.

Minimum to re-evaluate final 1.0.0 truth:

- 30-90 day dogfood evidence complete.
- 3 real users complete.
- 3 device sync/conflict logs complete.
- V19 external auditor pass complete.
- V20 independent third-party replay complete.
- No blocker bugs found in the final evidence review.
