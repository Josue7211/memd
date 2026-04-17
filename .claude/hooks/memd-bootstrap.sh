#!/usr/bin/env bash
# Shim — canonical implementation lives in .memd/hooks/memd-bootstrap.sh.
# `$MEMD_REPO` may be exported by the caller; otherwise default to the
# desktop checkout layout.
exec bash "${MEMD_REPO:-$HOME/Documents/projects/memd}/.memd/hooks/memd-bootstrap.sh" "$@"
