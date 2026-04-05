---
phase: 13-v3-workspace-aware-retrieval-priorities
type: context
status: ready
---

## Goal

Bias shared-memory retrieval toward the active workspace lane so resume and
handoff flows stop treating all shared memory as equally local.

## Current Boundary

- Shared workspace lanes exist.
- Shared handoff bundles exist.
- Lane corrections are now audited through repair.

## Dependencies

- phase 10 shared workspace foundations are complete
- phase 11 workspace handoff bundles are complete
- phase 12 workspace policy corrections are complete
