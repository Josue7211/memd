# Concerns

## 1. `v1` Completion Discipline

The project has a strong tendency to reach for `v2` ideas before `v1` repair and
provenance gaps are honestly closed. This is the main roadmap risk right now.

## 2. Large Binary Entry Files

Key orchestration files are growing:

- `crates/memd-client/src/main.rs`
- `crates/memd-server/src/main.rs`

They still work, but continued growth will make policy changes harder to reason
about and test.

## 3. Explainability Depth

The repo has explain, inbox, maintenance, and policy surfaces, but the repair
and provenance side is still shallower than the roadmap language wants.

## 4. Working Memory Semantics

Working memory has started moving toward a managed buffer, but admission,
eviction, and rehydration logic are still early. This is important because it
is on the path from `v1` memory OS toward `v2` superhuman behavior.

## 5. Source Trust and Contradiction Handling

The repo already has freshness and contradiction surfacing basics, but trust
weighting and branchable competing beliefs are not yet first-class enough.

## 6. Integration Surface Drift

Hooks, bundles, Obsidian, multimodal ingest, and backend contract work all move
quickly. That creates risk of docs and integration behavior drifting unless
there is tighter test coverage and phase discipline.

## 7. Architecture Boundary with `braind`

As ambition rises, there is a risk of dragging planning or cognition concerns
into `memd`. The boundary should remain: `memd` owns memory, `braind` owns cognition.
