# Structure

## Top-Level Layout

- `Cargo.toml` — workspace manifest
- `README.md` — product and developer overview
- `ROADMAP.md` — strategic versioned roadmap
- `docs/` — design, API, policy, and backend contract docs
- `crates/` — Rust workspace crates
- `integrations/` — agent and hook integration assets
- `scripts/` — operational helper scripts
- `.monitor/` — local monitoring artifacts
- `.planning/` — GSD planning state

## Crates Directory

- `crates/memd-schema/`
- `crates/memd-core/`
- `crates/memd-client/`
- `crates/memd-server/`
- `crates/memd-worker/`
- `crates/memd-rag/`
- `crates/memd-sidecar/`
- `crates/memd-multimodal/`

## Docs Directory

Primary docs include:

- `docs/architecture.md`
- `docs/api.md`
- `docs/rag.md`
- `docs/backend-api.md`
- `docs/backend-stack.md`
- `docs/backend-ownership.md`
- `docs/obsidian.md`
- `docs/promotion-policy.md`
- `docs/source-policy.md`

## Integration Assets

- `integrations/hooks/` — install scripts and runtime hook entrypoints
- `integrations/claude-code/`
- `integrations/codex/`
- `integrations/openclaw/`

## Naming Conventions

- crate names follow `memd-*`
- docs are grouped by system concern
- integration directories are named after host environments
- most product logic lives in `src/main.rs` for server/client binaries today

## Structural Observation

The repo is still compact enough to navigate quickly, but `crates/memd-client/src/main.rs`
and `crates/memd-server/src/main.rs` are becoming large orchestration files and
will likely need further decomposition over time.
