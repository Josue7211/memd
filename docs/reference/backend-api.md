> Secondary/reference doc. For current project truth start with [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]].

# Backend API Contract

This is the HTTP contract between `memd-rag` and `rag-sidecar`.

`memd` core should not speak directly to `LightRAG`. It speaks to the sidecar
adapter, which then talks to the multimodal backend stack.

`memd-rag` only needs to know about `rag-sidecar`.

## Base Expectations

- JSON over HTTP
- stable health check
- explicit upload and retrieval endpoints
- source metadata preserved end to end
- multimodal support when the backend is configured

## Endpoints

### `GET /healthz`

Purpose:

- report liveness
- report backend connectivity
- report whether multimodal dependencies are reachable

Response shape:

```json
{
  "status": "ok",
  "backend": {
    "connected": true,
    "name": "lightrag",
    "multimodal": true
  }
}
```

### `POST /v1/ingest`

Purpose:

- accept canonical documents or memory exports from `memd`
- preserve metadata for downstream indexing

Request shape:

```json
{
  "project": "demo",
  "namespace": "global",
  "source": {
    "id": "uuid",
    "kind": "memory",
    "content": "canonical content",
    "mime": "application/pdf",
    "bytes": 4096,
    "source_quality": "derived",
    "source_agent": "memd",
    "source_path": "/path/to/file",
    "tags": ["compaction", "canonical"]
  }
}
```

Response shape:

```json
{
  "status": "accepted",
  "track_id": "uuid",
  "items": 1
}
```

### `POST /v1/retrieve`

Purpose:

- retrieve semantic context
- route between text, multimodal, and graph strategies

Request shape:

```json
{
  "query": "decision cache",
  "project": "demo",
  "namespace": "global",
  "mode": "auto",
  "limit": 8,
  "include_cross_modal": true
}
```

Response shape:

```json
{
  "status": "ok",
  "mode": "multimodal",
  "items": [
    {
      "content": "...",
      "source": "MinerU",
      "score": 0.91
    }
  ]
}
```

## Mode Semantics

- `text`
  - semantic search only

- `multimodal`
  - semantic search plus cross-modal expansion
  - enabled for video, PDF, image, table, and equation inputs

- `graph`
  - graph-neighbor expansion over semantic results

- `auto`
  - choose the narrowest useful mode from query intent and signal detection

## Adapter Expectations

`memd-rag` should:

- normalize `MemoryItem` into ingest records
- write canonical spill output to `/v1/ingest`
- preserve source metadata such as MIME type, byte size, source path, and tags
- expose health via `/healthz`
- support search/retrieval calls through `/v1/retrieve`
- fail closed when the sidecar is unreachable

## Compatibility Rules

- `memd` does not need to know whether the backend is LightRAG, RAGAnything,
  or a future compatible service.
- `rag-sidecar` owns those details.
- The contract must preserve enough metadata for re-indexing and audit.
- The adapter must fail closed when the sidecar is unreachable.
