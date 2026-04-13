# No Dogfood Verification Gate for Phase Completion

- status: `closed`
- found: `2026-04-13`
- scope: process
- severity: critical

## Summary

All phases A-G marked "verified" via cargo test. No phase had an operational dogfood
gate: store a real fact, resume a real session, verify the fact surfaces, verify
continuity is accurate. Tests prove code correctness, not product correctness.

## Symptom

- Phase G "verified" but procedures table empty in production
- Phase B "verified" but continuity references deleted files
- Phase E "verified" but wake packet excludes facts/decisions/procedures
- 10-star composite score ~3.3/10 despite all phases "verified"

## Root Cause

- Phase pass gates defined as: "does the code exist and do unit tests pass?"
- No gate for: "does the feature work when a real agent uses it?"
- No `memd eval` assertion checking operational health as part of phase verification
- Culture of "verified = tests green" instead of "verified = product works"

## Fix Shape

- Add 5-7 dogfood assertions to `eval_bundle_memory()`:
  1. Working memory contains ≥1 non-status kind item
  2. All inbox items reference paths that exist on disk
  3. Procedure table non-empty after maintenance cycle
  4. Wake packet contains ≥1 fact or decision
  5. Status heartbeat references only existing paths
  6. Continuity fields do not reference expired items
  7. Status records are <50% of working memory
- Gate phase completion on `memd eval --fail-below 65`
- Add `memd eval` to CI or pre-commit for ongoing verification

## Evidence

- ROADMAP.md phase table: all A-G "verified"
- `eval_bundle_memory()` scores ~35 (weak) on current state
- Full audit: `docs/audits/2026-04-13-full-codebase-audit.md`

## Dependencies

- blocked-by: [[docs/backlog/2026-04-13-inbox-never-drains.md|inbox-never-drains]], [[docs/backlog/2026-04-13-status-noise-floods-memory.md|status-noise-floods-memory]], [[docs/backlog/2026-04-13-wake-packet-kind-coverage.md|wake-packet-kind-coverage]], [[docs/backlog/2026-04-13-procedure-detection-never-triggers.md|procedure-detection-never-triggers]] (assertions meaningless until underlying issues fixed)
- blocks: all future phase completions (gate on eval score)

## Related

- [[docs/audits/2026-04-13-full-codebase-audit.md]] — full audit findings
- [[docs/theory/locks/2026-04-11-memd-evaluation-theory-lock-v1.md]] — evaluation axes and benchmark classes
- [[docs/verification/MEMD-10-STAR.md]] — 10-star target and non-negotiable guarantees
- [[crates/memd-client/src/evaluation/eval_report_runtime.rs]] — existing eval scoring (extend this)
