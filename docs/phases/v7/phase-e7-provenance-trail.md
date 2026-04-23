---
phase: E7
name: Provenance Trail on Corrected Records
version: v7
status: planned
opened: 2026-04-22
depends_on: [B7]
axis: trust_provenance
plan_spec: docs/phases/v7/phase-e7-plan.md
---

# Phase E7: Provenance Trail on Corrected Records

## Goal

Every corrected canonical carries the full chain: original-canonical-id → correction-turn-id → judge-verdict → new-canonical-id. Click (or CLI-query) any fact, see every past version, who corrected when, and why.

## Why this phase exists

V5 E5 ProvenanceIntegrity gates `completeness=1.000` for ingest. E7 extends that to the correction lineage — not just "where did this come from" but "how did this replace what came before".

## Deliver

1. **Chain schema.** Add `correction_chain: Vec<ChainLink>` to canonical records promoted via correction; each link carries prior canonical id, correction record id, turn id, judge confidence, timestamp.
2. **CLI.** `memd fact provenance <id> [--chain]` emits full chain.
3. **Audit tool.** `memd fact provenance --audit-all` walks every canonical; hard-fails on broken chain link.
4. **Reuse V5 E5 auditor.** E5 completeness rule extends to chain integrity.

## Pass Gate

- pre: corrections have single source_turn but no chain back
- post: every correction-promoted canonical has complete chain; audit-all passes; CLI surfaces full lineage
- evidence: audit report, CLI tests, chain schema committed

## Product Win

"why does memd believe this?" has a click-through answer.

## Evidence

- chain schema
- audit report
- CLI output samples

## Fail Conditions

- Any broken link: hard fail.
- CLI truncates long chains silently: must page / stream, never drop.

## Non-Goals

- visual browser (V8 D8)
- cross-correction-source unification (out of V7 scope)
