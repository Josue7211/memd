# Uninstall memd

Secondary/reference doc. Start from [[ROADMAP]] for project truth.

Uninstall should not delete memory by default.

Preview first:

```bash
scripts/uninstall-memd.sh --dry-run
```

Remove the binary only:

```bash
scripts/uninstall-memd.sh
```

Project memory lives in `.memd/`. The uninstall script preserves it by default. Remove `.memd/` only if you intentionally want to delete local state.
