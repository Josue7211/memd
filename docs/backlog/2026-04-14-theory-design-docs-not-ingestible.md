# Theory/Design Docs Not Ingestible as Lane Source Material

status: open
severity: high
phase: Phase I
opened: 2026-04-14

## Problem

THEORY.md, DESIGN.md, architecture.md, and theory locks contain core
memd knowledge that agents need every session. These should be source
material for the architecture lane — ingested once, compiled into memory,
queried from DB. Instead agents re-read ~20KB of docs from disk each time.

## Fix

1. Register theory/design docs as architecture lane source material
2. Ingestion pipeline compiles them into architecture-lane-tagged items
3. Wake packet surfaces relevant architecture items automatically
4. Agents stop needing to `Read` these files — knowledge is in memory
