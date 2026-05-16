# Dev Server Guard Contract

Status: active

## Rule

Agents must not start duplicate webapp/dev servers for the same repo and port.

Before launching a server, use:

```bash
scripts/dev-server-guard.sh --port <port> -- <command...>
```

The wrapper is intentionally thin. The source of truth is the Rust CLI:

```bash
memd dev-server guard --port <port> --host 127.0.0.1 -- <command...>
memd dev-server list --output .memd --summary
memd dev-server release --port <port> --host 127.0.0.1
```

Examples:

```bash
scripts/dev-server-guard.sh --port 4321 -- npm run dev -- --host 127.0.0.1 --port 4321
scripts/dev-server-guard.sh --port 5173 -- npm run dev -- --host 127.0.0.1 --port 5173
```

## Behavior

- If the port is already listening, the guard prints the existing URL and exits `0`.
- If the port is free, the guard acquires hive lease `resource:dev-server:<repo-hash>:<host>:<port>`.
- If another active session owns that lease, launch is refused with `409 Conflict`.
- The lease records session, agent, workspace, repo root, command, pid, URL, TTL, and heartbeat.
- The lease is released when the guarded command exits.
- Expired or stale-heartbeat leases can be recovered by the next guarded launch.
- Acquire, heartbeat, conflict, recovery, and release emit coordination receipts.

## Why

Hive communication and claims work, but an agent that directly runs `npm run dev`
can bypass coordination. This guard turns dev-server launch into an explicit
cross-agent footstep visible through `memd dev-server list`, coordination
receipts, and hive board summaries.
