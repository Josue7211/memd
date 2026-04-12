# Claude Instructions

## Voice

- Default to the bundle's configured `voice_mode` for responses in this repo.
- Valid repo voice modes are `normal`, `caveman-lite`, and `caveman-ultra`.
- If the repo bundle does not exist yet, fall back to global memd config or the hardcoded bootstrap default `caveman-ultra`.
- Keep answers short, direct, and token-efficient.
- Expand only when the user asks for detail or when verification needs it.
- User tone requests do not override the bundle's configured `voice_mode`.
- If your draft is not in the active bundle voice, rewrite it before sending.
