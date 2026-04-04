# Backend Ownership Split

This repo owns the memory control plane.

The multimodal backend stack is separate. The point of this document is to make
the boundary explicit so implementation work does not blur the contracts.

## Ownership Matrix

| Component | Owned by memd repo | Responsibility |
| --- | --- | --- |
| `memd` core | Yes | routing, compaction, spill, verification, inbox, explain, attach/status/init |
| `memd-rag` adapter | Yes | client-side bridge from `memd` into the backend stack |
| `rag-sidecar` | No | HTTP bridge into the multimodal backend stack |
| `MinerU` | No | canonical extraction from PDFs and other source documents |
| `RAGAnything` | No | multimodal / cross-modal retrieval and expansion |
| `LightRAG` | No | long-term semantic storage and retrieval |

## Contract Boundaries

### `memd`

`memd` owns:

- the typed memory schema
- the retrieval router
- compaction and spill
- freshness and verification policy
- the project bundle and attach flow
- the client-side RAG adapter

`memd` does not own:

- multimodal extraction
- document parsing
- cross-modal expansion
- semantic backend internals

### `rag-sidecar`

`rag-sidecar` is the boundary service that:

- accepts uploads and retrieval requests
- preserves source metadata
- routes requests into the backend stack
- exposes health for downstream connectivity

### `MinerU`

`MinerU` is responsible for:

- extracting structure from PDF and document sources
- preserving layout-aware content when possible
- producing machine-readable fragments for downstream indexing

### `RAGAnything`

`RAGAnything` is responsible for:

- multimodal retrieval and expansion
- linking text, video, image, table, and equation content
- preserving cross-modal references for retrieval

### `LightRAG`

`LightRAG` is responsible for:

- long-term semantic storage
- evidence-backed retrieval
- project and global namespaces

## End-to-End Flow

1. `memd` receives a spill or sync event.
2. The `memd-rag` adapter forwards canonical content to `rag-sidecar`.
3. `rag-sidecar` invokes `MinerU` for document extraction when needed.
4. Extracted fragments flow through `RAGAnything` for multimodal expansion.
5. The resulting records are stored in `LightRAG`.
6. Retrieval requests flow back through `rag-sidecar` to the backend stack.

## Acceptance Criteria

- `memd` remains usable without the external backend stack.
- The external stack can be plugged in without changing the `memd` core schema.
- Multimodal support covers video, PDF, image, table, and equation inputs.
- Text-only fallback still works when no multimodal signal exists.
- The status command can distinguish local bundle health from backend health.
