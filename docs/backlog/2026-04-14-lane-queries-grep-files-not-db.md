# Lane Queries Grep Files Instead of Hitting DB

status: open
severity: high
phase: Phase I
opened: 2026-04-14

## Problem

`memd inspiration --query "..."` greps raw markdown files via hardcoded
`INSPIRATION_FILES` constant (4 of 6 paths — misses CLAUDE-CODE.md and
DOCTRINE.md). Theory says lane queries should hit the server after
ingestion, with file grep as bootstrap fallback only.

## Evidence

- `inspiration_search.rs:105-110` — hardcoded file list, substring match
- Theory lock D4: "Lane queries hit the server"

## Fix

After ingestion pipeline lands, rewrite `inspiration_search` to query DB
for items tagged with the inspiration lane. Keep file grep as fallback
for uninitialized bundles only.

## Depends On

- no-source-ingestion-pipeline
- #11 lane-architecture-gaps
