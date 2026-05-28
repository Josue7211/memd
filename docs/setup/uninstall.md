# Uninstall memd

Secondary/reference doc. Start from [[ROADMAP]] for project truth.

Uninstall should not delete memory by default.

Remove the binary:

```bash
rm -f "$HOME/.local/bin/memd"
```

Project memory lives in `.memd/`. Remove it only if you intentionally want to delete local state.
