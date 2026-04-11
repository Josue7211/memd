# Integrations

## Primary Internal Integration

- `memd-client` integrates with `memd-server` over HTTP.
- Key API surfaces are documented in [`docs/core/api.md`](/home/josue/Documents/projects/memd/docs/core/api.md):
  - `POST /memory/store`
  - `POST /memory/search`
  - `GET /memory/context`
  - `GET /memory/context/compact`
  - `GET /memory/working`
  - repair / verify / promote routes
- Practical implication: if the client does not issue the right request at the right time, memory does not affect behavior even if the server can store it.

## Agent / Harness Integrations

- Codex integration docs: [`integrations/codex/README.md`](/home/josue/Documents/projects/memd/integrations/codex/README.md)
- Claude Code integration docs: [`integrations/claude-code/README.md`](/home/josue/Documents/projects/memd/integrations/claude-code/README.md)
- Hook installers and wrapper scripts:
  - [`integrations/hooks/install.sh`](/home/josue/Documents/projects/memd/integrations/hooks/install.sh)
  - [`integrations/hooks/memd-context.sh`](/home/josue/Documents/projects/memd/integrations/hooks/memd-context.sh)
  - [`integrations/hooks/memd-spill.sh`](/home/josue/Documents/projects/memd/integrations/hooks/memd-spill.sh)
- OpenClaw integration docs: [`integrations/openclaw/README.md`](/home/josue/Documents/projects/memd/integrations/openclaw/README.md)
- Opencode integration docs: [`integrations/opencode/README.md`](/home/josue/Documents/projects/memd/integrations/opencode/README.md)

## MCP Integration

- Peer coordination bridge is implemented in Node:
  - package manifest: [`integrations/mcp-peer/package.json`](/home/josue/Documents/projects/memd/integrations/mcp-peer/package.json)
  - server: [`integrations/mcp-peer/server.js`](/home/josue/Documents/projects/memd/integrations/mcp-peer/server.js)
- Uses `@modelcontextprotocol/sdk` and talks back to `memd` via HTTP plus local `.memd` bundle files.
- This bridge is coordination-oriented: bundle discovery, peer presence, claims, assignments, and coworking context.

## Deployment / Ops Integrations

- Dockerized server build in [`deploy/docker/Dockerfile.memd-server`](/home/josue/Documents/projects/memd/deploy/docker/Dockerfile.memd-server)
- Compose deployment for a Portainer/OpenClaw VM in [`deploy/portainer/openclaw-vm/memd-server.compose.yml`](/home/josue/Documents/projects/memd/deploy/portainer/openclaw-vm/memd-server.compose.yml)
- Periodic worker integration through systemd in:
  - [`deploy/systemd/memd-worker.service`](/home/josue/Documents/projects/memd/deploy/systemd/memd-worker.service)
  - [`deploy/systemd/memd-worker.timer`](/home/josue/Documents/projects/memd/deploy/systemd/memd-worker.timer)

## External System Boundaries

- SQLite is the durable persistence layer through `rusqlite`.
- No external hosted database, auth provider, or SaaS integration is apparent from the manifests inspected here.
- The system is mostly local-first plus self-hosted HTTP server.

## Integration Hot Spots For Memory Failure

- Hook-based startup depends on explicit `memd resume --output .memd` or `memd refresh --output .memd` flows documented in:
  - [`integrations/hooks/README.md`](/home/josue/Documents/projects/memd/integrations/hooks/README.md)
  - [`integrations/codex/README.md`](/home/josue/Documents/projects/memd/integrations/codex/README.md)
  - [`integrations/claude-code/README.md`](/home/josue/Documents/projects/memd/integrations/claude-code/README.md)
- If a harness does not actually execute those flows before answering, `memd` is effectively bypassed.
- The current live-truth sync path is wired to repo changes in [`crates/memd-client/src/main.rs`](/home/josue/Documents/projects/memd/crates/memd-client/src/main.rs), but that does not prove equivalent ingest for arbitrary user memories or corrections.
- The result is an integration gap: `memd` may exist on disk and on the server, while the active agent still behaves like it has no memory.

## Debugging Questions To Carry Forward

- Which harnesses actually invoke `memd resume` before work, and which only document it?
- Which memory writes are automatic versus requiring explicit `memd remember` or repair flows?
- Does the active agent consume `/memory/context` output directly, or only bundle markdown projections?
- Where is the guaranteed integration point for “user said X, store it now, use it next turn”?
