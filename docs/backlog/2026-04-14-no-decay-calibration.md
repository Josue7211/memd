# No Decay Calibration

- status: `open`
- severity: `medium`
- phase: `V2-M2`
- opened: `2026-04-14`
- scope: memd-core

## Problem

Decay loop runs with hardcoded thresholds (21 days / 0.12 confidence). Thresholds never calibrated against real usage patterns. No data-driven justification for pruning schedule.

## Fix

- Collect decay metrics from production (age distribution, confidence curves)
- Calibrate thresholds from real data
- Test sensitivity to threshold changes
- Document rationale for chosen thresholds
