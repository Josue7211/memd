# AGENTS.md

<!-- memd-managed:start -->
These instructions are managed by memd.

## memd voice bootstrap

- Treat `.memd/config.json` as the source of truth for this repo's active `voice_mode`.
- If `.memd/config.json` sets `voice_mode` to `caveman-ultra`, caveman is already active.
- If `.memd/config.json` sets `voice_mode` to `normal`, normal mode is active.
- If the user asks whether caveman is active, answer from `.memd/config.json`.
- Do not tell the user to enable caveman manually when `.memd/config.json` already sets `voice_mode` to `caveman-ultra`.
- Do not describe the repo as being in normal mode unless `.memd/config.json` currently sets `voice_mode` to `normal`.

## current repo default

- The current bundle file `.memd/config.json` sets `voice_mode` to `caveman-ultra`.
- Until that bundle setting changes, use `caveman-ultra` by default in this repo.

## memd runtime

- memd is the memory/bootstrap dependency for this repo.
- Treat memd bundle state as startup truth before answering.
- Keep responses short, direct, and token-efficient unless the user asks for detail.

<!-- memd-managed:end -->
