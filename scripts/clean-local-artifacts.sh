#!/usr/bin/env bash
set -euo pipefail

ROOT="${ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
cd "$ROOT"

MODE="dry-run"
SCOPE="default"

usage() {
  cat <<'USAGE'
Usage: scripts/clean-local-artifacts.sh [--apply] [--cargo-only|--node-only|--memd-only]

Dry-runs by default. With --apply, removes ignored local artifacts only via
git clean -X, so tracked files are preserved.

Scopes:
  default     target, .memd ignored runtime files, app build/cache directories
  cargo-only  target only
  node-only   app node/build caches only
  memd-only   ignored .memd runtime files only
USAGE
}

while (($# > 0)); do
  case "$1" in
    --apply)
      MODE="apply"
      ;;
    --dry-run)
      MODE="dry-run"
      ;;
    --cargo-only)
      SCOPE="cargo"
      ;;
    --node-only)
      SCOPE="node"
      ;;
    --memd-only)
      SCOPE="memd"
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

cargo_paths=(target)
node_paths=(
  apps/.astro
  apps/dist
  apps/node_modules
  apps/dashboard/dist
  apps/dashboard/node_modules
  integrations/mac-bridge/node_modules
)
memd_paths=(
  .memd
  .monitor
  .watch-smoke-new
  docs/verification/25-5-memory-os-runs/external-public-cache
  docs/verification/25-5-memory-os-runs/promptwall-cache
)

paths=()
case "$SCOPE" in
  cargo) paths=("${cargo_paths[@]}") ;;
  node) paths=("${node_paths[@]}") ;;
  memd) paths=("${memd_paths[@]}") ;;
  default) paths=("${cargo_paths[@]}" "${node_paths[@]}" "${memd_paths[@]}") ;;
esac

echo "clean_local_artifacts root=$ROOT mode=$MODE scope=$SCOPE"
echo
echo "candidate_sizes"
for path in "${paths[@]}"; do
  if [[ -e "$path" ]]; then
    du -sh "$path" 2>/dev/null || true
  fi
done
echo

if [[ "$MODE" == "apply" ]]; then
  echo "removing ignored artifacts"
  git clean -fdX -- "${paths[@]}"
else
  echo "dry_run_ignored_artifacts"
  git clean -ndX -- "${paths[@]}"
  echo
  echo "rerun with --apply to remove these ignored artifacts"
fi
