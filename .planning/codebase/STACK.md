# Stack

## Overview

`memd` is a Rust workspace centered on a local memory control plane with
optional semantic backend integration.

## Languages

- Rust (`edition = 2024`) across all core crates
- Markdown for docs and integration guidance
- Shell and PowerShell for hook/bootstrap scripts

## Workspace Crates

- `crates/memd-schema` — shared request/response and domain types
- `crates/memd-core` — compaction and spill logic
- `crates/memd-server` — Axum-based SQLite-backed API server
- `crates/memd-client` — CLI and Rust client SDK
- `crates/memd-worker` — background maintenance worker
- `crates/memd-rag` — compatibility adapter over the sidecar contract
- `crates/memd-sidecar` — sidecar HTTP contract client/types
- `crates/memd-multimodal` — multimodal ingest planning and request generation

## Runtime

- local server default bind: `127.0.0.1:8787`
- local persistence via SQLite
- optional semantic backend behind `rag-sidecar`

## Key Configuration

- `MEMD_DB_PATH` — server database path override
- `MEMD_BUNDLE_ROOT` — active bundle root for project-local config
- `MEMD_RAG_URL` — semantic backend URL fallback

## Build and Test

- workspace manifest: `Cargo.toml`
- lockfile: `Cargo.lock`
- standard validation: `cargo test -q`

## Notable Supporting Assets

- API docs: `docs/api.md`
- architecture docs: `docs/architecture.md`
- roadmap: `ROADMAP.md`
- hooks: `integrations/hooks/`
- platform integrations: `integrations/claude-code/`, `integrations/codex/`, `integrations/openclaw/`

## Practical Read Order

1. `README.md`
2. `docs/architecture.md`
3. `crates/memd-schema/src/lib.rs`
4. `crates/memd-server/src/main.rs`
5. `crates/memd-client/src/main.rs`
