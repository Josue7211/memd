# No Change Detection on Source Material

status: open
severity: medium
phase: Phase I
opened: 2026-04-14

## Problem

Doctrine says: "Reopen raw only when changed, uncertain, or repairing."
No mechanism tracks whether source files have changed since last ingestion.
Without content hashing or mtime tracking, the system can't know when to
re-ingest vs reuse compiled memory.

## Fix

Track per-file ingestion metadata:
- Content hash (SHA-256 of file contents)
- Last ingestion timestamp
- Source path
- Store in `.memd/state/ingestion-manifest.json` or DB table
- On wake: compare current hash to manifest, only re-ingest changed files
