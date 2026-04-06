# Phase 47: `v6` Gap-Finding Research Loop Foundations - Context

**Gathered:** 2026-04-05
**Status:** Ready for planning
**Mode:** Auto-generated from live repo evidence

<domain>
## Phase Boundary

This phase turns the existing gap-reporting surface into a real research loop
foundation. The loop should inspect the repo, planning artifacts, eval outputs,
recent commits, and shared-memory wiring to detect the highest-value memory and
coordination gaps. The first slice should stay bounded and evidence-driven,
not auto-edit code or generate freeform essays.

</domain>

<decisions>
## Implementation Decisions

### Research-loop substrate
- **D-01:** Reuse the current `memd gap` / `GapReport` path as the substrate
  instead of inventing a second research command.
- **D-02:** Keep the output bounded and structured: JSON plus a concise
  markdown summary artifact.
- **D-03:** Make the loop evidence-first. Planning artifacts, eval snapshots,
  git state, runtime wiring, and recent work should all contribute to the
  prioritized gap list.

### Gap prioritization
- **D-04:** Treat hot-path memory quality, epistemic retrieval, and coworking
  safety as the top product goals for ranking gap candidates.
- **D-05:** Prefer signals that point to current operational pressure over
  generic roadmap prose.
- **D-06:** Keep the first loop conservative: identify and rank gaps, do not
  auto-fix them yet.

### Portability class
- **D-07:** The first slice is `portable` because it lives in the CLI and
  markdown artifacts, not a harness-specific adapter.

### the agent's Discretion
- Which repo/doc files are sampled beyond the core planning files, as long as
  the loop stays bounded and explainable
- Whether runtime wiring is shown as a single summary line or a small nested
  status block in the report
- Whether the markdown summary should be optimized for human review or for
  downstream automation first

</decisions>

<canonical_refs>
## Canonical References

### Roadmap and planning
- `ROADMAP.md` - phase 47 scope and the larger `v6` gap-finding direction
- `.planning/ROADMAP.md` - execution-facing phase ordering
- `.planning/STATE.md` - current product and open-loop state
- `.planning/PROJECT.md` - project framing and constraints

### Existing research / evaluation surfaces
- `crates/memd-client/src/main.rs` - `GapReport`, `ImproveReport`, eval, and
  resume orchestration
- `crates/memd-client/src/render.rs` - gap/improvement summary rendering
- `crates/memd-client/src/commands.rs` - parsing helpers used by the CLI
- `crates/memd-client/src/obsidian.rs` - compiled evidence and import surfaces

### Runtime context sources
- `README.md` - public bootstrap and project setup guidance
- `docs/setup.md` - explicit init / reload / project bootstrap details
- `AGENTS.md` and `CLAUDE.md` in the repo root when present
- `.planning/*` and repo docs as structured project memory

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable assets
- `gap_report` already combines planning evidence, eval state, resume state,
  coordination state, and recent commits into a structured prioritized report.
- `ImproveReport` already consumes the gap output and can be reused for
  bounded experiment / action loops later in `v6`.
- `read_bundle_status` now exposes runtime wiring, which gives this phase a
  compact signal for whether Codex, Claude, and OpenClaw are actually booted
  onto memd.

### Established patterns
- The project prefers bounded JSON + markdown artifacts over freeform logs.
- Evidence is meant to stay inspectable and provenance-friendly.
- Existing phase work tends to preserve current command surfaces and layer new
  behavior behind them instead of adding ad hoc parallel commands.

### Integration points
- `gap_report` should stay the single public loop surface for this phase.
- Evidence collection should read live repo files rather than only `.planning`
  artifacts.
- The report should surface the highest-value gaps for memory quality,
  epistemic retrieval, and coworking safety, not just raw counts.

</code_context>

<specifics>
## Specific Ideas

- The repo already has a gap-report command, so the main job is to widen its
  evidence scope and make it more explicitly research-oriented.
- The research loop should notice when memory bootstrap, runtime wiring, and
  repo docs disagree, because that is where user-visible drift tends to hide.
- Recent commits and git status should help prioritize active pressure over
  stale roadmap items.

</specifics>

<deferred>
## Deferred Ideas

- Auto-editing or auto-healing gaps belongs to later phases.
- Learned gap scoring and self-tuning should wait until the evidence surfaces
  are stable and reproducible.
- Dream / autodream consolidation remains downstream of research gaps, not
  part of this first slice.

</deferred>

---

*Phase: 47-gap-finding-research-loop-foundations*
*Context gathered: 2026-04-05*
