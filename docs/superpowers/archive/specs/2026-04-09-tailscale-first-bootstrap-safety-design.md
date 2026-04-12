# Tailscale-First Bootstrap Safety Design

Date: 2026-04-09
Status: Proposed
Owner: memd

## Why

`memd` hive only works if sessions share one trusted authority plane. When the shared `memd-server` goes down, silently failing over to `localhost` is dangerous:

- it can split hive truth into local and shared copies
- it can let one machine act outside the shared authority plane
- it expands the local prompt-injection and local-process trust surface
- it can make cowork look healthy while the real hive is degraded

The current system is too loose. It lets emergency localhost repair become the active authority plane without a strong policy contract.

That is the wrong model for hive teams.

The correct model is:

- Tailscale/shared authority first
- localhost only as an explicit lower-trust exception
- no silent fallback

## Product Thesis

Bootstrap should protect hive trust before it protects convenience.

If the shared Tailscale authority plane is unavailable, `memd` must:

1. warn clearly
2. mark the hive degraded
3. prefer retrying the shared server
4. only allow localhost fallback with explicit user permission
5. default localhost fallback to read-only
6. repeat that warning at the start of every session while fallback remains active

The operator should always know:

- which authority plane is active
- whether hive truth is degraded
- whether localhost fallback is in effect
- what capabilities are blocked while fallback is active

## Core Rules

1. Shared authority is primary

- Tailscale/shared `memd-server` is the default authority plane for hive teams.
- Bootstrap should prefer the configured shared endpoint first.

2. Localhost is exceptional

- `localhost` is not a normal peer of the shared server.
- `localhost` is a lower-trust fallback mode.
- `localhost` fallback must never activate silently.

3. Fallback must warn

- If bootstrap cannot reach the shared server, it must show a hard warning before any localhost fallback is allowed.
- If localhost fallback remains active, the session start path must warn again every session.

4. Fallback must be permission-gated

- Localhost fallback requires explicit operator consent.
- Permission must be policy-driven, not implied by “server unreachable”.

5. Fallback is read-only by default

- Localhost fallback may support inspection and local recall.
- Coordination writes, queen actions, claim/task mutation, and other shared authority writes are blocked by default.

6. Deny beats allow

- Bundle policy may forbid localhost fallback even if global policy allows it.
- The effective policy is the strictest applicable one.

7. Fallback must be visible and auditable

- Activation, expiry, revocation, and blocked-write events must emit receipts.
- `status`, `awareness`, and bootstrap surfaces must show active fallback state.

## Threat Model

This policy exists because localhost fallback is not neutral.

Primary risks:

- prompt-injection risk from local lower-trust tooling or processes
- split-brain hive state across shared and local authorities
- silent drift where one session believes hive is healthy while others remain on the shared server
- accidental queen actions against the wrong authority plane

This design does not claim localhost is always unsafe. It says localhost is a different trust domain and must be treated as such.

## Bootstrap Flow

When bootstrap or reload starts:

1. resolve the configured shared authority endpoint
2. probe shared authority reachability
3. if reachable:
   - continue normally
   - clear degraded-authority warning state if no fallback is active
4. if unreachable:
   - mark authority as degraded
   - show a hard warning
   - offer explicit next actions

### Bootstrap Options

If the shared endpoint is down, bootstrap should offer:

1. `retry_shared` (recommended/default)
2. `localhost_read_only`
3. `abort`

`retry_shared` is the recommended default because it preserves one truth plane.

### Required Warning Content

The bootstrap warning must say all of the following clearly:

- shared hive authority is unavailable
- localhost fallback is lower trust
- localhost fallback increases prompt-injection and split-brain risk
- coordination writes will be blocked by default
- hive is degraded while fallback is active

## Session-Start Warning

If localhost fallback is active at session start:

- warn immediately during bootstrap/session initialization
- repeat the degraded-authority state even if the user already allowed it in a prior session

The operator should not be able to forget that the current session is in degraded mode.

The warning should include:

- active authority plane
- shared authority status
- fallback mode (`localhost_read_only`)
- blocked capabilities
- expiry state

## Policy Model

Localhost fallback policy should exist at two scopes:

### Global Policy

Machine/user-level policy determines whether localhost fallback is ever permitted on this machine.

Suggested values:

- `deny`
- `allow_read_only`

### Bundle Policy

Project/bundle policy determines whether this specific project may use localhost fallback.

Suggested values:

- `inherit`
- `deny`
- `allow_read_only`

### Effective Policy

Effective policy is resolved with strict precedence:

- bundle `deny` overrides everything
- global `deny` blocks fallback even if bundle says allow
- bundle `allow_read_only` only works if global policy does not deny it

No write-capable localhost mode is part of this design.

## Fallback Session State

If localhost fallback is activated, runtime state must record:

- `authority_mode = localhost_read_only`
- `shared_base_url`
- `fallback_base_url`
- `activated_at`
- `activated_by`
- `reason`
- `expires_at`
- `blocked_capabilities`

Suggested default expiry:

- session scoped by default

This is the safest default because it forces a fresh decision at next bootstrap.

## Enforcement Rules

While `authority_mode = localhost_read_only`:

- allow:
  - `status`
  - `awareness`
  - read-only `coordination`
  - local recall, lookup, and memory inspection
- block:
  - queen decisions
  - claim acquire/release/transfer/recover
  - task assignment/upsert/recover
  - message send/ack if it changes shared coordination state
  - any mutation that claims shared hive authority

Blocked operations must fail fast with a clear reason:

- shared authority unavailable
- localhost fallback active
- operation requires trusted shared authority

## Receipts And Visibility

The system must emit durable receipts for:

- fallback activation
- fallback expiry
- fallback revocation
- blocked shared write attempted during fallback
- return to shared authority

These receipts must be visible in:

- `memd status`
- `memd awareness`
- `memd coordination`
- bootstrap summary

## CLI Surface Expectations

Existing surfaces should expose authority state directly:

- `memd status`
  - active authority plane
  - degraded state
  - fallback mode
  - blocked capability count or summary
- `memd awareness`
  - current session authority mode
  - whether sessions are on shared or fallback authority
- `memd coordination`
  - explicit warning banner when in localhost fallback
  - blocked queen actions

Bootstrap and setup flows should also expose policy and current mode without requiring file inspection.

## Non-Goals

This design does not:

- restore the remote Tailscale deployment
- make localhost a fully trusted hive authority
- merge local fallback mutations back into shared authority
- support silent automatic write failover

## Done Definition

This feature is done when:

1. bootstrap warns clearly when shared authority is unavailable
2. localhost fallback requires explicit permission
3. localhost fallback is read-only only
4. session start warns every time fallback is active
5. fallback policy is enforced through global plus bundle policy with deny precedence
6. receipts are written for activation, blocking, and revocation
7. `status`, `awareness`, and `coordination` surface the active authority mode

