---
phase: F2
name: Ingestion Pipeline
version: v2
status: pending
depends_on: [B2]
backlog_items: [37, 39, 41]
---

# Phase F2: Ingestion Pipeline

## Goal

Source files compiled into DB memory items. Read once, store forever.

## Deliver

- Ingestion step in `memd wake` or `memd setup`
- Walk `.memd/lanes/*/` source files
- Content hash tracking for change detection
- Lane queries hit server, not file grep
- Theory/design docs ingestible as architecture-lane items

## Pass Gate

- After `memd setup`, lane source files exist as DB memory items
- `memd inspiration --query "caveman"` returns DB result, not file grep
- Modify a source file → next wake re-ingests only changed file
- Unchanged files not re-read (hash match = skip)
- `memd lookup --query "wake vs resume"` returns architecture-lane fact

## Evidence

- Ingestion manifest showing file hashes and timestamps
- Before/after: `memd inspiration` query path (file vs DB)
- Change detection test: modify file, run wake, verify re-ingest
- No-change test: run wake twice, verify no re-ingest

## Fail Conditions

- Source files not in DB after setup
- Lane queries still grep files
- Re-ingest on unchanged files (wasted work)

## Rollback

- Revert if ingestion corrupts existing memory items
