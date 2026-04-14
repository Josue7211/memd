# Status Noise: Runaway Auto-Checkpoint Loop

status: open
severity: critical
phase: Phase I
opened: 2026-04-14

## Problem

Auto-checkpoint fires 15+ triggers per session. Each creates a kind=Status
record with 24h TTL. Result: 80-90% of working memory is status noise.
Facts, decisions, and procedures never surface in wake packets because
status items dominate the retrieval ranking.

This single loop breaks the entire product contract.

## Evidence

- Working memory ranked by context_score() in helpers.rs
- Kind bonus exists (+0.30 for Fact/Decision, -0.20 for Status)
- BUT intent scoring gives Project scope +1.15, overriding kind penalty
- Project-scope Status records outrank Global-scope Facts
- Wake packet is ~90% status items in practice

## Fix

1. Add redundancy_key dedup on checkpoint — same key = update, not insert
2. Reduce checkpoint TTL from 24h to something sane (1h? 30min?)
3. Cap status items in working memory admission (already partially done: cap at 2)
4. Verify cap actually works in production wake packets
5. Consider making checkpoint a single rolling record, not append-only
