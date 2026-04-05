# Architecture

## Pattern

`memd` is a layered control-plane architecture:

1. schema and memory types
2. compaction and core logic
3. server-side persistence and retrieval shaping
4. client-side CLI and integration workflows
5. optional backend adapter for semantic retrieval

## Primary Entry Points

- server entry: `crates/memd-server/src/main.rs`
- client entry: `crates/memd-client/src/main.rs`
- worker entry: `crates/memd-worker/src/main.rs`

## Layer Boundaries

### Schema Layer

- `crates/memd-schema/src/lib.rs`
- owns shared domain types, requests, responses, and serialization contracts

### Core Logic Layer

- `crates/memd-core/src/lib.rs`
- owns compaction packet generation and spill derivation

### Server Layer

- `crates/memd-server/src/main.rs`
- `crates/memd-server/src/store.rs`
- `crates/memd-server/src/routing.rs`
- owns SQLite persistence, retrieval policy, working memory, maintenance, and dashboard

### Client Layer

- `crates/memd-client/src/lib.rs`
- `crates/memd-client/src/main.rs`
- `crates/memd-client/src/obsidian.rs`
- owns CLI surface, bundle initialization, integration hooks, and bridge workflows

### Backend Adapter Layer

- `crates/memd-rag/src/lib.rs`
- `crates/memd-sidecar/src/lib.rs`
- `crates/memd-multimodal/src/lib.rs`
- owns backend contract, multimodal ingest request shaping, and semantic retrieval bridge

## Data Flow

1. client requests memory operations
2. server persists or retrieves typed memory
3. maintenance logic updates freshness/salience/consolidation state
4. optional sidecar/backend path handles semantic retrieval and multimodal ingest

## Architectural Direction

The current roadmap pushes the repo from a brain-inspired memory manager toward
a memory substrate for larger cognitive systems such as `braind`.
