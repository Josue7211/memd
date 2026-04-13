# Multimodal Extraction Stubs

- status: `closed`
- closed: `2026-04-13`
- resolution: Deferred until Mineru/RagAnything backends are wired. Current behavior stores metadata string (path, kind, backend, mime) which is valid — not silent failure. Using unimplemented!() would panic the server.
- found: `2026-04-13`
- scope: memd-multimodal

## Summary

Non-text assets (PDF, Image, Video, Table, Equation) return placeholder
strings instead of real extraction. `ExtractionBackend::Mineru` and
`RagAnything` are defined but never connect to actual backends.

## Symptom

- `memd ingest report.pdf` → stores "multimodal_asset path=report.pdf kind=Pdf
  backend=Mineru mime=application/pdf" as the content, not the PDF text
- Confidence values suggest full extraction is intended (PDF=0.9, Image=0.86)
  but the content is a metadata stub

## Root Cause

- `lib.rs:143-154` — `extract_chunk()` only handles `Text` variant
- All other kinds return `format!("multimodal_asset path=... kind=... backend=...")`
- No Mineru or RagAnything client implementation exists

## Fix Shape

- Wire Mineru for PDF extraction (already self-hosted on services VM)
- Wire RagAnything or similar for image/video
- Or mark these backends as `unimplemented` with clear error instead of silent stub

## Evidence

- `memd-multimodal/src/lib.rs:143-174`
