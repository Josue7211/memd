# Explain/Drilldown Surfaces Not Wired to CLI

status: open
severity: medium
phase: Phase I
opened: 2026-04-14

## Problem

Doctrine: "every memory must be visible, linkable, and inspectable."
Explainability scoring exists internally. Not surfaced in CLI. No
`memd explain <id>` command. No drilldown path from wake → canonical →
raw evidence. Provenance chain opaque to users.

## Fix

1. Add `memd explain <id>` CLI command
2. Show full provenance chain (source, confidence, corrections, raw evidence)
3. Wire evidence links in CLI and dashboard output
