# memd status Reports Healthy While Pipeline Broken

- status: `open`
- found: `2026-04-13`
- scope: memd-client
- severity: critical

## Summary

`memd status` returns `setup_ready=true, degraded=false, status=ok` while working
memory is 100% status noise, inbox is clogged with ghosts, procedures table is empty,
wake packet excludes all non-status kinds, and heartbeat references deleted files.
Status is a liveness check, not a health check.

## Symptom

- `memd status` → `{"setup_ready": true, "degraded": false, "server": {"status": "ok"}}`
- Heartbeat working: "editing .planning/ROADMAP.md" (deleted file)
- All critical issues (#27, #15, #28, #29, #22) invisible to status

## Root Cause

- Status checks: bundle exists? config exists? env exists? server responds?
- Does NOT check: working memory useful? inbox clean? procedures exist? wake packet complete?
- No integration with `eval_bundle_memory()` scoring
- `degraded` flag only set by authority/shared-URL issues, not memory quality

## Fix Shape

- Run `eval_bundle_memory()` as part of status (or a lightweight subset)
- If eval score < 65 ("weak"), set `degraded: true`
- Add health fields: `working_quality`, `inbox_health`, `procedure_count`
- Validate heartbeat paths exist before storing

## Evidence

- `crates/memd-client/src/cli/mod.rs` — `read_bundle_status()` implementation
- `crates/memd-client/src/evaluation/eval_report_runtime.rs` — `eval_bundle_memory()` scoring
- Current output: server ok, degraded false, heartbeat references deleted files

## Dependencies

- blocked-by: [[docs/backlog/2026-04-13-stale-continuity-ghost-refs.md|stale-continuity-ghost-refs]] (heartbeat ghost refs must be fixed first)
- blocked-by: [[docs/backlog/2026-04-13-dogfood-verification-gap.md|dogfood-verification-gap]] (eval scoring must exist to drive degraded flag)
- independent of status noise fix — can check eval score regardless

## Related

- [[docs/audits/2026-04-13-full-codebase-audit.md]] — full audit findings
- [[docs/verification/MEMD-10-STAR.md]] — pillar 11: operator UX, audits, observability
- [[docs/theory/locks/2026-04-11-memd-evaluation-theory-lock-v1.md]] — Class C: loop telemetry
