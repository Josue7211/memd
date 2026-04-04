# Backend Implementation Plan

This is the execution plan for the multimodal backend stack around `memd`.

## Phase A: Contract

Deliver:

- ownership split
- request/response contract
- health semantics
- fallback behavior

Done when:

- the repo clearly says what `memd` owns and what the backend stack owns
- the external stack is explicitly multimodal

## Phase B: Bridge

Deliver:

- `rag-sidecar` integration path
- adapter targets that point at the sidecar
- bundle-level backend configuration

Done when:

- `memd rag sync` can talk to the sidecar endpoint
- `memd status` can report backend reachability

## Phase C: Extraction

Deliver:

- `MinerU` document extraction path
- PDF and document normalization
- source metadata preservation

Done when:

- canonical documents are emitted as machine-readable chunks
- extracted chunks retain enough provenance to be useful later

## Phase D: Multimodal Retrieval

Deliver:

- `RAGAnything` query path
- video/image/table/equation expansion
- text fallback routing

Done when:

- queries can resolve against multimodal content without breaking text-only use

## Phase E: Semantic Storage

Deliver:

- `LightRAG` namespace strategy
- canonical memory export
- project/global split

Done when:

- long-term semantic recall works without being the hot path

## Ownership

- `memd` repo owns the control plane and adapter.
- The external backend stack owns extraction, multimodal expansion, and long-term storage.

## First Integration Milestone

1. `memd` writes canonical spill output to `rag-sidecar`.
2. `rag-sidecar` accepts the record and exposes health.
3. `MinerU`, `RAGAnything`, and `LightRAG` can be wired underneath without changing the `memd` contract.
