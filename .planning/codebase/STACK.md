# Stack

## Runtime Shape

- Primary language: Rust 2024 edition via the workspace root [`Cargo.toml`](/home/josue/Documents/projects/memd/Cargo.toml).
- Main binaries:
  - `memd` CLI in [`crates/memd-client/src/main.rs`](/home/josue/Documents/projects/memd/crates/memd-client/src/main.rs)
  - `memd-server` HTTP API in [`crates/memd-server/src/main.rs`](/home/josue/Documents/projects/memd/crates/memd-server/src/main.rs)
  - `memd-worker` background verifier in [`crates/memd-worker/src/main.rs`](/home/josue/Documents/projects/memd/crates/memd-worker/src/main.rs)
- Supporting crates:
  - schema/types: [`crates/memd-schema/Cargo.toml`](/home/josue/Documents/projects/memd/crates/memd-schema/Cargo.toml)
  - shared logic: [`crates/memd-core/Cargo.toml`](/home/josue/Documents/projects/memd/crates/memd-core/Cargo.toml)
  - sidecar HTTP client helpers: [`crates/memd-sidecar/Cargo.toml`](/home/josue/Documents/projects/memd/crates/memd-sidecar/Cargo.toml)
  - retrieval / RAG helpers: [`crates/memd-rag/Cargo.toml`](/home/josue/Documents/projects/memd/crates/memd-rag/Cargo.toml)
  - multimodal helpers: [`crates/memd-multimodal/Cargo.toml`](/home/josue/Documents/projects/memd/crates/memd-multimodal/Cargo.toml)

## Core Dependencies

- HTTP server: `axum` in [`crates/memd-server/Cargo.toml`](/home/josue/Documents/projects/memd/crates/memd-server/Cargo.toml)
- HTTP client: `reqwest` in [`crates/memd-client/Cargo.toml`](/home/josue/Documents/projects/memd/crates/memd-client/Cargo.toml)
- Database: `rusqlite` with bundled SQLite in [`crates/memd-server/Cargo.toml`](/home/josue/Documents/projects/memd/crates/memd-server/Cargo.toml)
- Async runtime: `tokio` in client, server, and worker manifests
- CLI parsing: `clap` in [`crates/memd-client/Cargo.toml`](/home/josue/Documents/projects/memd/crates/memd-client/Cargo.toml)
- Serialization and IDs: `serde`, `serde_json`, `uuid`, `chrono` across the workspace
- File watching / local change detection: `notify` and `walkdir` in [`crates/memd-client/Cargo.toml`](/home/josue/Documents/projects/memd/crates/memd-client/Cargo.toml)

## Storage Model

- Durable memory is persisted in SQLite through server store logic in [`crates/memd-server/src/store.rs`](/home/josue/Documents/projects/memd/crates/memd-server/src/store.rs).
- API-facing memory shapes are defined in [`crates/memd-schema/src/lib.rs`](/home/josue/Documents/projects/memd/crates/memd-schema/src/lib.rs).
- Bundle and markdown state live under `.memd/` according to the client and docs, especially:
  - [`docs/policy/config.md`](/home/josue/Documents/projects/memd/docs/policy/config.md)
  - [`docs/core/obsidian.md`](/home/josue/Documents/projects/memd/docs/core/obsidian.md)

## Deployment

- Container build for server: [`deploy/docker/Dockerfile.memd-server`](/home/josue/Documents/projects/memd/deploy/docker/Dockerfile.memd-server)
- Compose deployment for OpenClaw VM: [`deploy/portainer/openclaw-vm/memd-server.compose.yml`](/home/josue/Documents/projects/memd/deploy/portainer/openclaw-vm/memd-server.compose.yml)
- Background verification worker via systemd:
  - [`deploy/systemd/memd-worker.service`](/home/josue/Documents/projects/memd/deploy/systemd/memd-worker.service)
  - [`deploy/systemd/memd-worker.timer`](/home/josue/Documents/projects/memd/deploy/systemd/memd-worker.timer)

## Memory Hot Path

The actual runtime path for everyday memory use is:

1. Agent or hook runs `memd resume` / `memd refresh` / `memd remember` through [`crates/memd-client/src/main.rs`](/home/josue/Documents/projects/memd/crates/memd-client/src/main.rs)
2. Client calls HTTP routes such as `/memory/store`, `/memory/search`, `/memory/context`, `/memory/context/compact`, and `/memory/working` described in [`docs/core/api.md`](/home/josue/Documents/projects/memd/docs/core/api.md)
3. Server ranks and assembles memory in [`crates/memd-server/src/main.rs`](/home/josue/Documents/projects/memd/crates/memd-server/src/main.rs) and [`crates/memd-server/src/working.rs`](/home/josue/Documents/projects/memd/crates/memd-server/src/working.rs)
4. Client renders prompt-facing state in [`crates/memd-client/src/render.rs`](/home/josue/Documents/projects/memd/crates/memd-client/src/render.rs)

## Why The Stack Matters For The Current Bug

- The stack is not event-native end to end. Memory recall depends heavily on explicit CLI flows and explicit rendering, not an always-on recall substrate.
- The strongest implemented live-truth path in the client is repo-change syncing via [`sync_recent_repo_live_truth`]( /home/josue/Documents/projects/memd/crates/memd-client/src/main.rs#L11794 ), which is narrower than “memories in general.”
- Because the user-facing behavior depends on the CLI invoking the right HTTP routes and then the renderer surfacing the right result, failures can happen even when storage and tests look healthy.
