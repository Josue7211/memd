---
status: open
severity: medium
phase: L2
opened: 2026-04-16
scope: memd-client, memd-server, hive
---
# Hive Handoff Accepts Ephemeral Proof / Fixture Sessions As Live Targets

- status: `open`
- found: `2026-04-16`
- scope: memd-client (primary), memd-server (secondary)
- severity: medium — handoff silently lands on a dead/test session, packet is
  lost until the user notices

## Summary

`memd hive handoff --to-session <X>` will happily accept `codex-fresh` /
`codex-stale` / `session-live-*` / `session-dogfood-*` as handoff targets
even though the code *already recognizes these as ephemeral proof sessions*.
The handoff message is delivered and a `queen_handoff` receipt is emitted,
but no live harness will ever read them — the target session isn't actually
running.

Observed on 2026-04-16 during end-of-L2 handoff: user has zero codex tabs
open, yet `memd hive roster` shows codex-fresh / codex-stale with
`status=live` and `last_seen` within 5 minutes. An agent building a handoff
picks one of them, the packet is "sent", and it silently vanishes.

## Symptom

- `memd hive roster` surfaces fixture sessions with `status=live`
- `memd hive handoff --to-session codex-fresh ...` succeeds with 0 warnings
- The receiving harness doesn't exist, so handoff is never acknowledged
- Next session wake-up sees "pending handoff to codex-fresh" in receipts and
  may treat it as a real coordination state

## Root Cause

The staleness → handoff-target pipeline has two cooperating gaps:

1. **Server-side (`crates/memd-server/src/store_hive.rs:227`)**
   `is_ephemeral_proof_hive_session(&session)` returns `true` for
   `"codex-fresh"`, `session-live-*`, `session-dogfood-*`. This is used
   only to *shorten the live-grace window from 15 min to 5 min*. It is
   not used to filter these sessions out of roster responses or to flag
   them on the wire.

2. **Client-side (`crates/memd-client/src/hive/ops_runtime.rs:1032` —
   `resolve_hive_target_entry`)**
   The resolver does exactly one check: "does the session string match an
   entry in awareness?" It does not:
   - consult `is_ephemeral_proof_hive_session` (or any wire equivalent)
   - verify `base_url_healthy == Some(true)`
   - verify the target has any heartbeat *after* its initial upsert
   - require `--allow-ephemeral` for proof sessions

Combined result: any fixture whose `last_seen` is inside the 5-min ephemeral
grace window is a valid handoff target.

## Evidence

Reproduction on current `research/mining` branch (clean repo, no codex
tab open):

```
$ memd hive roster --json | jq '.bees[] | {session, status, last_seen}'
{ "session": "session-7eab5dde", "status": "live", "last_seen": "2026-04-16T21:44:00Z" }
{ "session": "codex-fresh",      "status": "live", "last_seen": "2026-04-16T21:40:02Z" }
{ "session": "codex-stale",      "status": "live", "last_seen": "2026-04-16T21:39:59Z" }

$ memd hive handoff --to-session codex-fresh --task-id L2-ship --topic ...
hive_handoff from=claude-code to=codex (codex-fresh) message_id=2e37d42a-...
# no warning, no rejection, no prompt
```

Server code proving the fixture recognition already exists but is unused
at the handoff path:

```rust
// store_hive.rs:227
pub(crate) fn is_ephemeral_proof_hive_session(session: &HiveSessionRecord) -> bool {
    let session_name = session.session.trim();
    session_name == "codex-fresh"
        || session_name.starts_with("session-live-")
        || session_name.starts_with("session-dogfood-")
}
```

## Fix Shape

Three options, ordered cheapest → strongest:

1. **Warn-only** (CLI): `resolve_hive_target_entry` checks session name
   against the ephemeral pattern set and prints a stderr warning
   ("target 'codex-fresh' looks like a proof/fixture session — handoff
   may not be read. Continue? [y/N]"). Non-interactive callers require
   `--allow-ephemeral`.

2. **Server flag** (preferred): extend the wire response for sessions
   with `is_ephemeral_proof: bool`. `hive handoff` refuses targets where
   this is `true` unless `--allow-ephemeral`. Benefit: the set of
   "ephemeral" names lives in one place (server), and new fixture
   patterns don't require a client rebuild.

3. **Filter at roster** (nuclear): `memd hive roster` excludes ephemeral
   sessions by default, exposing them only under `--include-ephemeral`.
   Plus (2) on the handoff path as belt-and-suspenders.

## Test Plan

- Unit (server): assert `is_ephemeral_proof_hive_session` output gets
  surfaced to the wire (pending fix shape 2).
- Unit (client): `resolve_hive_target_entry` rejects / warns for
  codex-fresh / session-live-* / session-dogfood-* when flag absent.
- Integration: `memd hive handoff --to-session codex-fresh` exits
  non-zero without `--allow-ephemeral`; passes with it.

## Related

- `docs/plans/M4-EXECUTION-PLAN.md` L2.9 — the handoff quality gate
  (composite ≥ 0.8) validates *packet contents*; this bug is about
  *target liveness*. Complementary, not overlapping.
- Also consider whether `base_url_healthy` should be promoted from
  `Option<bool>` (currently always `None` on roster output) to an active
  ping probe.
