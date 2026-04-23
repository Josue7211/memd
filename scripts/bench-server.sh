#!/usr/bin/env bash
# Launch memd-server with bench-safe settings.
#
# Why: historical note — the store hot path used to run three O(N)
# `list_entities()` scans per `/memory/store` (auto_link_entity +
# create_wiki_links + create_named_entity_links), which stalled bulk ingest
# sweeps around N=100. V3/B3 landed an M6 migration with a `project_id` virtual
# generated column + indexed `memory_entity_aliases` table, so those calls are
# now O(log N). Default is to exercise the real indexed path so bench runs
# catch regressions. Set `MEMD_STORE_AUTO_LINK_DISABLED=1` as an opt-in escape
# hatch if the hot path ever regresses.
#
# Usage (from repo root):
#   CARGO_TARGET_DIR=/tmp/memd-target cargo build --release -p memd-server
#   scripts/bench-server.sh                # binds 127.0.0.1:18787, uses .memd/bench.db
#   MEMD_BIND_ADDR=127.0.0.1:19000 scripts/bench-server.sh
#   MEMD_DB_PATH=/tmp/lme.db scripts/bench-server.sh
#
# Then in a separate shell:
#   MEMD_BASE_URL=http://127.0.0.1:18787 /tmp/memd-target/release/memd \
#     benchmark public longmemeval --retrieval-backend memd --limit 200 ...

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/memd-target}"
BIN="${MEMD_SERVER_BIN:-$TARGET_DIR/release/memd-server}"

if [[ ! -x "$BIN" ]]; then
  echo "bench-server: missing $BIN" >&2
  echo "  build with: CARGO_TARGET_DIR=$TARGET_DIR cargo build --release -p memd-server" >&2
  exit 1
fi

export MEMD_STORE_AUTO_LINK_DISABLED="${MEMD_STORE_AUTO_LINK_DISABLED:-0}"
export MEMD_RATE_LIMIT_DISABLED="${MEMD_RATE_LIMIT_DISABLED:-1}"
export MEMD_BIND_ADDR="${MEMD_BIND_ADDR:-127.0.0.1:18787}"
export MEMD_DB_PATH="${MEMD_DB_PATH:-$REPO_ROOT/.memd/bench.db}"
export MEMD_LOG_FORMAT="${MEMD_LOG_FORMAT:-pretty}"

mkdir -p "$(dirname "$MEMD_DB_PATH")"

echo "bench-server: bind=$MEMD_BIND_ADDR db=$MEMD_DB_PATH auto_link_disabled=$MEMD_STORE_AUTO_LINK_DISABLED rate_limit_disabled=$MEMD_RATE_LIMIT_DISABLED" >&2
exec "$BIN" "$@"
