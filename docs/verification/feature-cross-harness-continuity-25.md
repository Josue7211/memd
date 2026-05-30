> Secondary/reference doc. For current project truth start with [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]].

# Feature Proof: Cross-Harness Continuity (25-star slice)

Feature registry row: `feature.cross_harness_continuity`

## Honest Status

- Current status: `partial`
- Proof status: `strong`
- Dogfood status: `ad_hoc`
- External status: `none`

This is a strong local 25/5 proof: static surface validation plus a generated local cross-process replay in a temporary output directory. It does **not** prove seamless production continuity, sustained dogfood, real cross-session external continuity, or independent external replay.

## What This Proof Validates

`bash scripts/verify/feature-cross-harness-continuity-proof.sh` checks the local checkout for:

1. Required harness surfaces for Codex, Claude Code, OpenCode, OpenClaw, and Hermes docs/modules.
2. Shared bundle continuity surfaces across those harnesses:
   - `.memd/wake.md`
   - `.memd/mem.md`
   - `.memd/events.md`
3. Wake/resume/write-path parity in each required harness doc/module, with handoff/spill/checkpoint/capture/teach/remember accepted as local continuity write surfaces.
4. Shared harness index/preset/mod/shared modules enumerate the same local harnesses and strict context capability/access routes.
5. Native handoff/resume/wake/recovery artifacts:
   - `scripts/handoff-latest.sh`
   - `scripts/memd-continuity-status.sh`
   - `scripts/verify/25-5-harness-process-replay.sh`
   - `crates/memd-client/src/runtime/resume/mod.rs`
   - `crates/memd-client/src/runtime/resume/wakeup.rs`
   - `crates/memd-client/src/runtime/resume/recovery_signals.rs`
   - `docs/handoff/`
6. Existing dated local process replay JSON, when present, still reports `status: pass`, keeps Codex private visibility false, proves corrected memory object top-ranks, proves replay memory object IDs are present/unique, and includes strict packet sections.
7. A fresh local replay runs with `OUT_DIR` in a temporary directory and proves no registered `.memd` or `docs/verification/25-5-memory-os-runs` artifact dirtiness.
8. Registry/report/doc honesty: `proof_status` is `strong`, `external_status` remains `none`, dogfood remains `ad_hoc`, and no allowed claim implies external cross-session verification.

## Evidence Found in This Checkout

Local harness documentation exists for:

- `integrations/codex/README.md`
- `integrations/claude-code/README.md`
- `integrations/opencode/README.md`
- `integrations/openclaw/README.md`
- `integrations/hermes/README.md`

Local harness modules exist for:

- `crates/memd-client/src/harness/codex.rs`
- `crates/memd-client/src/harness/claude_code.rs`
- `crates/memd-client/src/harness/opencode.rs`
- `crates/memd-client/src/harness/openclaw.rs`
- `crates/memd-client/src/harness/hermes.rs`

Existing replay artifacts under `docs/verification/25-5-memory-os-runs/` are treated as local/ad hoc evidence only. The proof also generates a current replay in a temp directory to avoid registered artifact noise. Neither path is external proof.

## Allowed Claim

Strong local 25/5 proof validates cross-harness continuity surfaces across Codex, Claude Code, OpenCode, OpenClaw, and Hermes config surfaces; native wake/resume/handoff/recovery bundles; cross-process memory object consistency; local artifact cleanliness; and local-only claim boundaries.

## Forbidden Claims

Do not claim 25/25, seamless production cross-harness continuity, production readiness, real external cross-session continuity, sustained dogfood, or independent external verification from this local 25/5 proof alone.

## Refresh Procedure

Run after state schema, CLI/server protocol, import/export, handoff/resume/wake, generated harness pack, or integration doc changes:

```bash
bash scripts/verify/feature-cross-harness-continuity-proof.sh
bash scripts/verify/feature-registry-audit.sh
bash scripts/doc-lint.sh
git diff --check
```

Run cargo checks only when Rust/code paths are touched.
