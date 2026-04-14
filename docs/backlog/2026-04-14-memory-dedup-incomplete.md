# Memory Dedup Incomplete — Same Fact Stored Multiple Times

status: open
severity: medium
phase: Phase I
opened: 2026-04-14

## Problem

Doctrine: "one concept, one memory object." Dual-key dedup (canonical +
redundancy) exists but is optional. Some ingest paths don't set
redundancy_key. Duplicate facts accumulate in DB.

## Fix

1. Make redundancy_key mandatory at ingest
2. Audit all ingest paths for redundancy_key coverage
3. Add dedup check before insert — update existing if key matches
