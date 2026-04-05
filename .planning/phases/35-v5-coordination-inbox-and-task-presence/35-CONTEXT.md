# Phase 35 Context: `v5` Coordination Inbox and Task Presence

## Why This Phase Exists

`memd` now has task-native orchestration, but coworking still requires agents
to pull together inbox state, task ownership, and session presence manually.
The next gap is a compact coordination view that tells a session what is
waiting, what it owns, and what needs help or review.

## Inputs

- brokered peer messages and inbox acknowledgement
- shared peer task records and assignment flows
- heartbeat-based live session presence
- user goal of simultaneous coworking without stepping on each other

## Constraints

- keep coordination source-of-truth inside `memd-server`
- preserve session-qualified ownership
- make the first presence view compact and automation-friendly

## Target Outcome

The next phase should add a coordination inbox/presence surface that combines
messages, shared tasks, and ownership pressure into one resumable coworking
view.
