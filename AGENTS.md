# AGENTS.md

<!-- memd-managed:start -->
These instructions are managed by memd.

## memd voice

- Voice source of truth: `.memd/config.json` → `voice_mode`.
- Current default: `caveman-ultra`.
- Caveman = compressed wording, not broken spelling. Keep exact technical terms.

## memd runtime

- memd is the memory dependency for this repo.
- Start from `.memd/wake.md` before relying on transcript recall.
- Deeper recall: `memd lookup --output .memd --query "..."` or `memd resume --output .memd`.
- Corrections: write back via `correct-memory.sh`, do not trust transcript over durable truth.

<!-- memd-managed:end -->
