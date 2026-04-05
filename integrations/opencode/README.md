# OpenCode Integration

OpenCode should use `memd` as the shared memory control plane.

Recommended flow:

1. resume from the bundle before starting work
2. read the generated memory markdown file for durable continuity
3. write durable outcomes back with `memd remember`
4. emit a shared handoff when another client needs to take over

If you are using a bundle, read:

- `.memd/MEMD_MEMORY.md`
- `.memd/agents/OPENCODE_MEMORY.md`

Use the OpenCode-specific entrypoint:

```bash
.memd/agents/opencode.sh
```

## Shared Memory Loop

```bash
memd resume --output .memd
memd remember --output .memd --kind decision --content "Keep the shared lane current."
memd handoff --output .memd --prompt
```

## Shell Hook Example

```bash
MEMD_PROJECT=my-project \
MEMD_AGENT=opencode \
./integrations/hooks/memd-context.sh
```
