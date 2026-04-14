# No Behavior-Changing Recall Proof

status: open
severity: critical
phase: Phase I
opened: 2026-04-14

## Problem

memd can store memory. There is no proof it changes agent behavior.
No benchmark showing: agent with memd recall produces different (better)
output than agent without. mempalace has 96.6% on LongMemEval. memd has
theory.

## Fix

1. Design recall benchmark: same prompt, with/without memd recall, measure output quality
2. Run against existing benchmarks (LongMemEval, LoCoMo, MemBench — datasets already in .memd/benchmarks/)
3. Prove fact recall changes agent response
4. Prove correction recall changes agent response
5. Prove procedure recall changes agent response
6. Publish results
