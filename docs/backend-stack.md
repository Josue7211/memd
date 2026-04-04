# Backend Stack Contract

`memd` does not implement the full LightRAG multimodal stack itself.

It owns the memory control plane. The backend stack is external and can be
plugged in when you want PDF, image, table, equation, and cross-modal
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
  - connects text, tables, images, and equations into one retrieval flow

- `LightRAG`
  - long-term semantic store
  - evidence-backed retrieval backend

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
