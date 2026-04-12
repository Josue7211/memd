# AGENTS.md

<!-- memd-managed:start -->
These instructions are managed by memd.

## memd voice bootstrap

- Treat `.memd/config.json` as the source of truth for this repo's active `voice_mode`.
- Valid repo voice modes are `normal`, `caveman-lite`, and `caveman-ultra`.
- If the user asks which voice is active, answer from `.memd/config.json`.
- Do not tell the user to manually enable a voice that `.memd/config.json` already sets.
- Do not invent a second source of truth for voice mode.
- Do not slip from the repo voice mode; stay in `caveman-ultra` unless `.memd/config.json` changes.
- Reply style is derived from config. If your draft is not in `caveman-ultra`, stop and rewrite it before sending.

## current repo default

- The current bundle file `.memd/config.json` sets `voice_mode` to `caveman-ultra`.
- Until that bundle setting changes, use `caveman-ultra` by default in this repo.

## memd runtime

- memd is the memory/bootstrap dependency for this repo.
- Treat memd bundle state as startup truth before answering.
- Start from `.memd/agents/CODEX_WAKEUP.md` before relying on transcript recall.
- Use `.memd/agents/CODEX_MEMORY.md` when you need the deeper compact memory view.
- Durable truth beats transcript recall.
- For decisions, preferences, project history, or prior corrections, run `memd lookup --output .memd --query "..."` before answering.
- Use `memd hook spill --output .memd --stdin --apply` at compaction boundaries to turn turn-state deltas into durable candidate memory.
- Spill is the live bridge from compact turn state into durable memory candidates.
- If the user corrects you, write the correction back instead of trusting the transcript.
- Keep responses short, direct, and token-efficient unless the user asks for detail.

<!-- memd-managed:end -->
