# Tag System Not Searchable or Filterable

status: open
severity: medium
phase: Phase I
opened: 2026-04-14

## Problem

Tags are stored on memory items but there's no GET /memory/tags endpoint,
no tag faceting, no filter-by-tag in retrieval queries. Tags are write-only.

## Fix

1. Add tag listing endpoint
2. Add tag filter to context/working queries
3. Implement tag faceting for dashboard
