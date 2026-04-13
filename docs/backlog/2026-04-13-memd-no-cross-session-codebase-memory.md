# memd Does Not Remember Codebase Structure Across Sessions

- status: `closed` (resolved by upstream fixes: wake-packet-kind-coverage, status-noise-floods-memory, agent-write-helpers-unreachable)
- found: `2026-04-13`
- scope: memd-client, memd (product)
- severity: high

## Summary

Every new session, the agent has to re-scan the entire codebase from scratch.
memd should store codebase structure once ("read once, update live") and recall
it in future sessions. This is the core product promise — "read once, remember
once, reuse everywhere" — and it's not working. Agent was reminded 3 times in
one session that this should work.

## Symptom

- Agent starts session → doesn't know codebase structure → runs full scan
- Facts stored via `memd remember` exist in DB but don't appear in wake packet
- Agent told to "remember the codebase" has to be reminded repeatedly
- wake.md shows status noise instead of codebase facts

## Root Cause

Compound issue from multiple bugs:
1. **#15 wake-packet-kind-coverage** — facts excluded from wake by intent scoring
2. **#27 status-noise-floods-memory** — status records crowd out facts in working memory
3. **#23 agent-write-helpers-unreachable** — `remember-long` protocol broken
4. **No auto-structure-capture** — memd has no mechanism to automatically capture and
   update codebase structure on init or structural changes

## Fix Shape

- Fix #15 and #27 first — facts must surface in wake/working
- Add `memd remember --kind fact --tag codebase-structure` as part of `memd init`
- Or: add auto-structure-capture during `memd wake` that detects significant repo changes
- Ensure codebase structure facts have high confidence (0.95+) and no TTL
- Verify: after storing, `memd wake` shows the fact in Durable Truth section

## Evidence

- Session transcript: agent told 3x to remember codebase structure
- `memd remember --kind fact` stores successfully but fact invisible in wake
- `memd lookup --query "codebase structure"` returns the stored fact
- wake.md shows only status records in Durable Truth section
- Depends on: #15, #27, #23

## Dependencies

- blocked-by: [[docs/backlog/2026-04-13-wake-packet-kind-coverage.md|wake-packet-kind-coverage]] (facts must surface in wake)
- blocked-by: [[docs/backlog/2026-04-13-status-noise-floods-memory.md|status-noise-floods-memory]] (facts must not be evicted by status)
- blocked-by: [[docs/backlog/2026-04-13-agent-write-helpers-unreachable.md|agent-write-helpers-unreachable]] (agents must be able to store typed facts)
- **fix all 3 upstream issues first**, then verify this works

## Related

- [[docs/audits/2026-04-13-full-codebase-audit.md]] — full audit findings
- [[docs/theory/models/2026-04-11-memd-10-star-memory-model-v2.md]] — "read once, remember once, reuse everywhere"
- [[docs/theory/locks/2026-04-11-memd-theory-lock-v1.md]] — live loop step 5 (repair semantic memory)
- [[docs/theory/locks/2026-04-11-memd-evaluation-theory-lock-v1.md]] — Journey 1: fresh session resume
