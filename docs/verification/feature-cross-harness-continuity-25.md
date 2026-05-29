> Secondary/reference doc. For current project truth start with [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]].

# Feature Proof: Cross-Harness Continuity (25-star slice)

Feature registry row: `feature.cross_harness_continuity`

## Honest Status

- Current status: `partial`
- Proof status: `partial`
- Dogfood status: `ad_hoc`
- External status: `none`

This is a local/static proof plus validation of existing local replay artifacts. It does **not** prove seamless production continuity, real cross-session external continuity, or independent external replay.

## What This Proof Validates

`bash scripts/verify/feature-cross-harness-continuity-proof.sh` checks the local checkout for:

1. Available harness surfaces for Codex, Hermes, OpenClaw, and Claude-style/Claude Code integrations when their docs or harness modules exist.
2. Shared bundle continuity surfaces across those harnesses:
   - `.memd/wake.md`
   - `.memd/mem.md`
   - `.memd/events.md`
3. Wake/resume/write-path parity in each available harness doc/module, with handoff/spill/checkpoint/capture/teach accepted as local continuity write surfaces.
4. Shared harness index/preset modules that enumerate the same local harnesses.
5. Handoff/resume/wake parity artifacts:
   - `scripts/handoff-latest.sh`
   - `scripts/memd-continuity-status.sh`
   - `scripts/verify/25-5-harness-process-replay.sh`
6. Existing dated local process replay JSON, when present, still reports `status: pass`, keeps Codex private visibility false, and includes strict packet sections.
7. Registry/report/doc honesty: `external_status` remains `none`, dogfood remains `ad_hoc`, and no allowed claim implies external cross-session verification.

## Evidence Found in This Checkout

Local harness documentation exists for:

- `integrations/codex/README.md`
- `integrations/hermes/README.md`
- `integrations/openclaw/README.md`
- `integrations/claude-code/README.md`

Local harness modules exist for:

- `crates/memd-client/src/harness/codex.rs`
- `crates/memd-client/src/harness/hermes.rs`
- `crates/memd-client/src/harness/openclaw.rs`
- `crates/memd-client/src/harness/claude_code.rs`

Existing replay artifacts under `docs/verification/25-5-memory-os-runs/` are treated as local/ad hoc evidence only. They are not external proof.

## Allowed Claim

Current local proof validates documented continuity surfaces across available Codex, Hermes, OpenClaw, and Claude-style harness packs, including shared wake/memory/event bundle surfaces, wake/resume/write-path parity, and existing local process replay artifact sanity checks when present.

## Forbidden Claims

Do not claim seamless cross-harness continuity, production-grade continuity, real external cross-session continuity, or independent external verification from this proof alone.

## Refresh Procedure

Run after state schema, CLI/server protocol, import/export, handoff/resume/wake, generated harness pack, or integration doc changes:

```bash
bash scripts/verify/feature-cross-harness-continuity-proof.sh
bash scripts/verify/feature-registry-audit.sh
bash scripts/doc-lint.sh
git diff --check
```

Run cargo checks only when Rust/code paths are touched.
