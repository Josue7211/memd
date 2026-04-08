# OpenClaw Integration

OpenClaw should use `memd` as the shared memory control plane.

This is the second harness pack after Codex. It keeps the OpenClaw flow
compact: read context before task work, spill compact state at the boundary,
and keep the visible bundle markdown in sync.

Recommended flow:

1. fetch compact context before a task
2. spill compaction output into durable memory
3. rely on the inbox and explain views for review and cleanup

If you are using a bundle, read:

- `.memd/MEMD_MEMORY.md`
- `.memd/MEMD_WAKEUP.md`
- `.memd/agents/OPENCLAW_WAKEUP.md`
- `.memd/agents/OPENCLAW_MEMORY.md`

And use the OpenClaw-specific entrypoint:

```bash
.memd/agents/openclaw.sh
```

## Shell Hook

```bash
MEMD_PROJECT=my-project \
MEMD_AGENT=openclaw \
./integrations/hooks/memd-context.sh
```

Or use the bundle-aware flow:

```bash
memd context --project my-project --agent openclaw --compact
memd hook spill --output .memd --stdin --apply
```

Or after installing the hook kit:

```bash
memd-context
```

## PowerShell Hook

```powershell
$env:MEMD_PROJECT = "my-project"
$env:MEMD_AGENT = "openclaw"
./integrations/hooks/memd-context.ps1
```

Or after installing the hook kit:

```powershell
memd-hook-context.ps1
```

## Spill Hook

```bash
./integrations/hooks/memd-spill.sh --stdin --apply < compaction.json
```

Or after installing the hook kit:

```bash
memd-hook-spill --stdin --apply < compaction.json
```

## Windows Spill Hook

```powershell
Get-Content .\compaction.json -Raw | ./integrations/hooks/memd-spill.ps1 -Apply
```
