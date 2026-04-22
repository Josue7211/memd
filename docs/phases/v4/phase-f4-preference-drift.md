---
phase: F4
name: Preference Replay + Drift Detection
version: v4
status: planned
opened: 2026-04-22
depends_on: [C4]
backlog_items: [preferences-not-persisted, preferences-drift-silent]
axis: correction_retention
---

# Phase F4: Preference Replay + Drift Detection

## Goal

User preferences (voice, style, workflow rules) replayed at wake, monitored for drift, surfaced when agent behavior diverges. Today: preferences stored, partially loaded at wake, silently ignored when agent forgets. No drift alarm.

## Why this phase exists

Correction-retention axis gains come from corrections (C4) and preferences (F4). Preferences are the "quiet" correction — user sets once, expects forever. Current pain: user has to re-state preferences across sessions because agent drops them.

## Deliver

1. **Preference replay at wake.** D4 compiler includes a "Preferences" section, top-priority. Not optional, not demoted.
2. **Drift detector.** After every N agent turns (default 10), check last-agent-behavior against stored preferences via cached LLM-judge. Flag divergence as `preference-drift` record.
3. **Drift surface.** User sees drift in next wake: "⚠ last session you asked for terse replies; I was verbose 3x."
4. **Preference correction path.** C4 correction detector extended: a correction to a preference-drift record promotes the preference (bumps confidence).
5. **Preference bench stub.** V5 E5/F5 own full measurement; F4 lands the hooks.

## Pass Gate

- pre: preferences stored; drift undetected; user restates same preference ≥2x per week
- post: 7-day dogfood shows drift detections surface; user-restate rate drops ≥50%
- evidence: drift-detection log + user-restate count pre vs post
- regression budget: LLM-judge cost ≤ $2/week; no wake bloat beyond D4 budget

## Product Win

User sets a preference once; memd holds the line. When agent slips, user sees it, not a silent drift. Feels like "memd has my back."

## Evidence

- `.memd/logs/preference-drift.ndjson`
- User-restate count comparison
- Sample drift surfaces from 7-day dogfood

## Fail Conditions

- Drift detector false-positives annoy user: tune N, tighten judge prompt.
- LLM-judge cost blown: reduce check frequency, cache aggressively.

## Rollback

Behind `MEMD_F4_PREF_DRIFT=1`.
