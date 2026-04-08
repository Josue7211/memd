# Research Loop Manifest

Autoresearch uses a loop-first workflow to keep `memd`’s memory substrate lean, reliable, and token efficient. Each loop represents a small, reversible experiment with a clear target, metric, stop condition, and expected risk profile. Loops run one after another, report the percent improvement and token savings they delivered, then hand accepted lessons to autodream for consolidation. When autoresearch detects a loop has exhausted its gains, it either refreshes the loop (with a higher token-efficiency threshold) or picks the next candidate from the gap queue.

The eight loops are arranged by expected savings and risk so that low-risk/token-high loops run first:

1. **Prompt Surface Compression**
   - *Target*: Resume, handoff, and prompt surfaces that echo the previous turn verbatim.
   - *Metric*: Characters/tokens removed without losing intent or actionable context.
   - *Stop Condition*: Two consecutive loops hit the compaction delta floor (no improvement beyond a 2% ceiling).
   - *Risk*: Low.

2. **Live Truth Freshness**
   - *Target*: Re-reading files/outputs that were just changed and validating stale beliefs.
   - *Metric*: Reread count per session, stale-belief warnings emitted.
   - *Stop Condition*: Freshness metric meets the freshness baseline and no stale beliefs remain in the hot lane.
   - *Risk*: Low to medium.

3. **Capability Contract Detection**
   - *Target*: Misalignment between what skills advertise and the actual CLI/binary surfaces (e.g., `gsd-*` availability).
   - *Metric*: Wrong-interface failure rate, uncovered capability gaps.
   - *Stop Condition*: Every known skill resolves to a documented contract (portable, harness-native, or adapter-required).
   - *Risk*: Medium.

4. **Event Spine Compaction**
   - *Target*: Noisy integration events that bloat event spines in `integrationEvents` or other coordination channels.
   - *Metric*: Token burn per coordination request, event duplication rate.
   - *Stop Condition*: Token cost delta drops below 5% of the baseline event stream.
   - *Risk*: Low.

5. **Correction Learning**
   - *Target*: Repeat corrections surfaced through user feedback or policy updates.
   - *Metric*: Correction recurrence, frequency of forking to manual overrides.
   - *Stop Condition*: Same correction does not occur more than once across three consecutive loops.
   - *Risk*: Medium.

6. **Long-Context Avoidance**
   - *Target*: Turns that blast the session with entire transcripts instead of leveraging compact state.
   - *Metric*: Average prompt length, frequency of long-context spikes.
   - *Stop Condition*: Prompt length stays within the budget without ignoring verifiable context; spikes are scheduled explicitly.
   - *Risk*: Low.

7. **Cross-Harness Portability**
   - *Target*: Memory artifacts that must stay consistent across Codex, Claude Code, OpenClaw, and other harnesses.
   - *Metric*: Contract coverage per harness, adapter-required warnings outstanding.
   - *Stop Condition*: Each promoted artifact records its portability class and is either mapped or flagged with an adaptor plan.
   - *Risk*: Medium.

8. **Controlled Self-Evolution**
   - *Target*: Making sure the evolution engine quantifies gains before promoting new skills or automation.
   - *Metric*: Accepted-change rate, rollback incidents, promotion evidence completeness.
   - *Stop Condition*: Promotion confidence reaches the registry’s threshold and regressions stay below a defined guardrail.
   - *Risk*: High (run once the baseline is stable).

Loop Instrumentation:

- Each loop includes a percent-improvement report that explicitly states the loop’s token savings and success/failure.
- Autoresearch routines append loop results to `.memd/loops/` with status, telemetry, and links to revised artifacts.
- Accepted loops go through the evolution engine; the portable artifacts are tagged with their portability class (portable, harness-native, adapter-required) and versioned with rollback history.
- Once a loop is accepted, autodream ingests the approved changes, compacts frequent patterns, and seeds the next loop with the highest-scoring gaps.
- When tokens spike, the loop controller may pause the current loop, compact the hot lane, and resume from the last safe checkpoint to avoid wasted reads.

Loop Execution & Telemetry:

- `autoresearch` runs the loop queue sequentially by default. Use `autoresearch --loop <name|number>` to re-run a loop or `autoresearch --auto` to sweep until every loop either saturates or hits a guardrail.
- Each pass creates a record under `.memd/loops/loop-<slug>.json` that stores the percent improvement, token savings, success status, and any promoted artifacts. Operators can inspect the loop journal with `memd loops` (default list), `memd loops --summary`, or `memd loops --loop <slug>`, or by reading `.memd/loops/`.
- The percent-improvement report clearly states token cost savings and is included in the operator-facing loop summary; if a loop tries to repeat the same savings, the stop condition prevents it from burning tokens for minimal gain.
- When `memd gap` surfaces new optimization ideas, autoresearch adds them to the queue as candidate loops (tagged with priority and expected risk). Loops remain reversible until their accepted changes are consolidated via autodream.
- On completion, every loop appends its percent-improvement number to the running `loops.summary.json` and the evolution engine uses that value to decide whether the promoted artifact deserves a portability classification upgrade or a rollback.
- If a loop fails any guardrail (e.g., wrong-interface detection spikes during capability-contract detection), the controller records the failure, signals the operator, and either retries with a narrower scope or moves on to the next loop after the issue is resolved.

Autoresearch is still invoked via `memd autoresearch --loop <slug>` or `memd autoresearch --auto` when you want to refresh loops directly. Run `memd autoresearch --manifest` to print the current loop roster before selecting one. When each run completes, the resulting log file already makes itself available to `memd loops` so you can project the percent-improvement telemetry without re-reading large transcripts. For a quick telemetry snapshot, use `memd telemetry` (or `memd telemetry --json`) to read the `loops.summary.json` ledger, view status counts, and see average/best percent-improvement and token-saving metrics without having to enumerate each loop artifact.
