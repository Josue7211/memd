# Update memd

Secondary/reference doc. Start from [[ROADMAP]] for project truth.

```bash
scripts/update-memd.sh --dry-run
scripts/update-memd.sh
```

Manual equivalent:

```bash
git pull --ff-only
scripts/install-memd.sh
memd doctor --summary
```

Do not delete `.memd` to update. `.memd` is user/project memory state. The update script prints this before doing work and preserves `.memd` by default.
