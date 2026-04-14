# No Source Material Ingestion Pipeline

status: open
severity: critical
phase: Phase I
opened: 2026-04-14

extraction source:
- `mempalace/mempalace/convo_miner.py`
- `mempalace/mempalace/general_extractor.py`
- A2 note: `.memd/lanes/architecture/A2-03-ingestion-pipeline.md`

## Problem

Doctrine says: "Read raw source once. Compile into visible memory objects."
No ingestion step exists. `memd wake` and `memd setup` never compile
`.memd/lanes/*/` source files into DB memory items. Source material is
static markdown that agents re-read from disk every session.

## Fix

Add ingestion to `memd wake` or `memd setup`:
- Walk `.memd/lanes/*/` source files
- Hash content, compare to last ingestion
- Create/update lane-tagged memory items in DB
- Track ingestion metadata (hash, mtime, source path)

## Depends On

- #11 lane-architecture-gaps (lane field on memory_items table)
