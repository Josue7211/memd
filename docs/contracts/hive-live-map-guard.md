# Hive Live Map Guard Contract

memd is the coordination/memory runtime. ClawControl is an app that may consume
memd. They are separate projects, separate lifecycles, and separate operational
surfaces.

This contract exists because hive work is only useful when agents can see the
current map before they collide. A memd agent must be able to update, test,
deploy, and prove memd without starting, stopping, killing, or rebuilding
ClawControl.

## Ownership Boundary

- memd authority owner: memd.
- memd authority container: `memd-authority`.
- memd authority stack: `memd-authority-stack`.
- memd authority network: `memd-authority-network`.
- memd authority data volume: `memd_authority_data`.
- migration authority port: `8788`.
- ClawControl runtimes are sibling app runtimes, not memd dependencies.

Bundled use is allowed. Shared lifecycle is not. A memd build/test/proof must not
run `cargo tauri dev`, raw Vite, ClawControl launchd jobs, or ClawControl dev
servers. If ClawControl is already running, memd may observe it as awareness
only. Observation does not give memd permission to restart it.

## Live Map Surface

The live codebase map is the cheap shared truth layer:

- `.memd/state/codebase-live-map.json`
- `.memd/state/codebase-live-map-events.ndjson`
- `.memd/state/host-io-guard.txt`
- `.memd/state/host-io-awareness.txt`

File interaction hooks and host I/O guards must write live-map events before the
next heartbeat. Agents should not need to reread the whole repo to discover that
another agent changed or blocked something. The live map says whether reread is
required.

If `codebase-live-map.json` reports `status=blocked` or
`reread_required=true`, agents must treat that as current shared state and stop
broad Git, Cargo, test, and repo-scan work until the guard clears or the map
names a narrower safe action.

## Hard Stops

Before broad Git, Cargo, test, deploy, or repo-scan work, run:

```bash
scripts/memd-host-io-guard.sh
```

Exit `75` means same-repo, filesystem, unknown, or host-owned tooling is blocking
work. Stop and report the blocker. Do not pile on new work.

For memd Cargo work, run:

```bash
scripts/memd-cargo-guard.sh -- <cargo args>
```

The guard gives memd its own Cargo home and target directory so ClawControl and
other projects cannot block memd through package-cache locks.

## Sibling Awareness

Sibling app activity on the same volume is awareness, not a default hard stop.
Examples:

- ClawControl already running.
- AgentShell adapter already running.
- Vite already running for another app.

These observations belong in `host-io-awareness.txt` and live-map events with
`reason=separate-existing-runtime`. They must not be used as a reason to kill,
restart, or launch the sibling app.

## Forbidden memd Workarounds

memd agents must not:

- launch ClawControl to test memd.
- launch Tauri to test memd.
- launch Vite or an app dev server to test memd.
- kill ClawControl to unblock memd.
- deploy memd into a ClawControl-named container, stack, image, network, or
  volume.
- publish focused test heartbeats to the real shared memd authority.

The only allowed ClawControl sync route is an explicit read of an already-running
ClawControl source:

```bash
MEMD_ALLOW_CLAWCONTROL_SYNC=1 CAPTURE_HTTP=1 IMPORT_CLAWCONTROL_BUNDLE=1 \
  scripts/live-state-sync-clawcontrol.sh
```

That route still must not launch ClawControl.

## Handoff Requirement

Every durable handoff must include a user-copyable next-agent prompt. That
prompt must tell the next agent to inspect `.memd/wake.md`, run
`scripts/memd-continuity-status.sh`, follow `NEXT_CONTINUITY_ACTION`, and keep
memd/ClawControl separate.
