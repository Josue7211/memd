---
phase: N2
name: Integrations Polish
version: v2
status: pending
depends_on: [F2, I2, M2]
backlog_items: [82, 83, 84, 85]
---

# Phase N2: Integrations Polish

## Goal

Obsidian two-way sync. Harness plugin packaging. Team support foundations.

## Deliver

- Obsidian import with conflict resolution
- Two-way sync (file watcher or periodic poll)
- Harness SDK packaging model (from supermemory extraction)
- Multi-user/team concept foundations
- RAG sidecar with timeout, retry, fallback

## Pass Gate

- Edit note in Obsidian → appears in memd within 1 cycle
- Store fact in memd → appears in Obsidian vault within 1 cycle
- Conflict: both sides edit → resolution documented and working
- RAG query with 5s timeout → fallback to non-RAG retrieval

## Evidence

- Two-way sync test
- Conflict resolution test
- RAG fallback test

## Fail Conditions

- Sync loop (both sides endlessly updating)
- Conflict loses data
- RAG timeout blocks main retrieval

## Rollback

- Disable sync if data loss detected
