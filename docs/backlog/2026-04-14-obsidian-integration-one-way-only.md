# Obsidian Integration One-Way Only

status: open
severity: medium
phase: Phase I
opened: 2026-04-14

## Problem

Theory: "Obsidian is first-class workspace and source lane." Export to
Obsidian works. Import from Obsidian is stubbed. No live two-way sync.
Vault structure hardcoded.

## Fix

1. Implement Obsidian import with conflict resolution
2. Document expected vault structure
3. Enable two-way sync (file watcher or periodic poll)
