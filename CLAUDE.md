# Claude Instructions

@.memd/agents/CLAUDE_IMPORTS.md

## Memory Rules

- memd is the ONLY memory system. Never write to the Claude Code memory directory (`~/.claude/projects/*/memory/`).
- `memd wake --output .memd` MUST be the first action on the first message of every session, before any other work.
- Use memd lookup, memd checkpoint, and the lane helpers (remember-short, correct-memory, etc.) for all memory operations.
- Never do manual codebase scans to reconstruct context that memd already provides.
