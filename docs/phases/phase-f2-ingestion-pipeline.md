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

## Donor Extraction (from inspiration repos)

- **F2-D1** (mempalace `palace.py`): Hash manifest for idempotent re-ingestion. Store content hash + mtime per source file. Changed → re-ingest. Unchanged → skip. Deleted → mark stale.
- **F2-D2** (mempalace extractors): Type-specific extraction rules. `fact` (declarative), `decision` (choice+rationale), `procedure` (how-to+steps), `status` (current state, gets TTL). Map to memd MemoryKind enum.
- **F2-D3** (Omegon `types.rs` — **DIRECT RUST LIFT**): Content-hash dedup on ingest. `normalize_for_hash()` → SHA256 → first 16 hex chars. On ingest: hash exists = reinforce, new = insert.
- **F2-D4** (Omegon `omegon-codescan` crate — **DIRECT RUST LIFT**): Tree-sitter AST chunking for code files. Named boundary chunking (functions, structs, impl blocks). SHA256 change detection. Incremental indexing (skip if git HEAD unchanged).

See: `docs/theory/2026-04-14-donor-extraction-to-v2-phases.md` for full details.

## Rollback

- Revert if ingestion corrupts existing memory items
