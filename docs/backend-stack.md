# Backend Stack Contract

`memd` does not implement the full LightRAG multimodal stack itself.

It owns the memory control plane. The backend stack is external and can be
plugged in when you want video, PDF, image, table, equation, and cross-modal
retrieval.

## Required Pieces

- `rag-sidecar`
  - bridges uploads and retrieval into the backend stack
  - handles request shaping and backend connectivity

- `MinerU`
  - extracts structure from PDFs and other canonical source documents
  - turns raw files into machine-readable content

- `RAGAnything`
  - provides multimodal / cross-modal retrieval and expansion
  - connects text, video, tables, images, and equations into one retrieval flow

- `LightRAG`
  - long-term semantic store
  - evidence-backed retrieval backend

## Integration Checklist

### End-to-End Multimodal Flow

Acceptance criteria:

- video, PDF, image, table, and equation inputs are all accepted by the stack
- uploads preserve source metadata through extraction and retrieval
- multimodal content can be queried through a single retrieval interface
- text-only fallback still works when no multimodal signal exists

### `rag-sidecar`

Acceptance criteria:

- accepts uploads and retrieval requests over HTTP
- preserves chunk/source metadata through the pipeline
- routes requests to text, multimodal, or graph retrieval modes
- supports video-aware retrieval modes when video inputs are present
- returns a health check that includes backend connectivity

### `MinerU`

Acceptance criteria:

- extracts canonical structure from PDF and document sources
- extracts transcript or metadata from video sources when present
- preserves tables, figures, and equation references
- emits machine-readable chunks that can be indexed
- can be invoked from the ingest pipeline without manual preprocessing

### `RAGAnything`

Acceptance criteria:

- expands retrieval across text, video, image, table, and equation content
- supports multimodal query routing
- preserves links between extracted source fragments
- can fall back to text-only retrieval when no multimodal signals exist

### `LightRAG`

Acceptance criteria:

- stores long-term semantic memory
- supports project and global namespaces
- returns evidence-backed retrieval results
- stays off the hot path for short-term context

## Build Checklist

1. Connect `rag-sidecar` to the configured LightRAG endpoint.
2. Run canonical document extraction through `MinerU`.
3. Pass extracted fragments through `RAGAnything` for multimodal expansion.
4. Index the resulting content into `LightRAG`.
5. Verify that video, PDF, image, table, and equation queries all resolve.
6. Keep text-only retrieval working as a fallback path.
7. Surface backend health in `memd status`.

## Product Boundary

`memd`:

- stores typed memory
- routes retrieval
- compacts working context
- spills durable candidates
- verifies freshness
- exposes inbox and explain

External backend stack:

- extracts documents
- expands multimodal context
- stores semantic memory
- serves long-term retrieval

This separation keeps `memd` portable while still allowing the full memory OS
stack to be assembled around it.
