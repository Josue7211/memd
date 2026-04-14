# Event Spine NDJSON Has No Integrity Checks

status: open
severity: medium
phase: Phase I
opened: 2026-04-14

## Problem

raw-spine.jsonl is the trust anchor — immutable source truth. No checksums
per line, no validation on read, no repair logic. Corruption goes
undetected silently.

## Fix

1. Add per-line checksum on write
2. Validate on read
3. Implement spine repair on detection of corruption
4. Verify integrity on startup
