# Phase 36 Context: `v5` Claim Recovery and Coordination Automation

## Why This Phase Exists

`memd` now has peer primitives, shared tasks, and a coordination inbox, but
recovery from dead/stale sessions still depends on manual operator cleanup. The
next gap is helping active sessions reclaim or reroute stalled work safely.

## Inputs

- heartbeat-based session presence
- leased claims and claim transfer
- shared peer tasks and coordination inbox pressure

## Constraints

- preserve explicit claim safety and session-qualified ownership
- avoid silent reassignment without inspectable evidence
- keep the first automation slice bounded and operator-visible

## Target Outcome

The next phase should help active sessions recover stale claims and route
stalled shared tasks without collapsing ownership discipline.
