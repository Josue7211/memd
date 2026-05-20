#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOST_IO_GUARD="${HOST_IO_GUARD:-$SCRIPT_DIR/memd-host-io-guard.sh}"
if [ "${HOST_IO_GUARD_ENABLED:-1}" != "0" ] && [ "${HOST_IO_GUARD_ENABLED:-1}" != "false" ]; then
  bash "$HOST_IO_GUARD"
fi

ROOT="${ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
cd "$ROOT"

MAX_UNTRACKED_FILES="${MEMD_HYGIENE_MAX_UNTRACKED_FILES:-200}"
MAX_UNTRACKED_LINES="${MEMD_HYGIENE_MAX_UNTRACKED_LINES:-50000}"
MAX_VISIBLE_CACHE_PATHS="${MEMD_HYGIENE_MAX_VISIBLE_CACHE_PATHS:-0}"
MAX_EMPTY_DIRS="${MEMD_HYGIENE_MAX_EMPTY_DIRS:-0}"
MAX_SOURCE_LINES="${MEMD_HYGIENE_MAX_SOURCE_LINES:-2500}"
FAIL_ON_OVERSIZED="${MEMD_HYGIENE_FAIL_ON_OVERSIZED:-1}"
REQUIRED_MANIFESTS=(
  "Cargo.toml"
  "Cargo.lock"
  "apps/package.json"
  "apps/package-lock.json"
  "apps/dashboard/package.json"
  "apps/dashboard/package-lock.json"
)

UNTRACKED_LIST="$(mktemp "${TMPDIR:-/tmp}/memd-hygiene-untracked.XXXXXX")"
EMPTY_DIRS_LIST="$(mktemp "${TMPDIR:-/tmp}/memd-hygiene-empty-dirs.XXXXXX")"
OVERSIZED_LIST="$(mktemp "${TMPDIR:-/tmp}/memd-hygiene-oversized.XXXXXX")"
MISSING_MANIFESTS_LIST="$(mktemp "${TMPDIR:-/tmp}/memd-hygiene-manifests.XXXXXX")"
trap 'rm -f "$UNTRACKED_LIST" "$EMPTY_DIRS_LIST" "$OVERSIZED_LIST" "$MISSING_MANIFESTS_LIST"' EXIT

git ls-files -o --exclude-standard >"$UNTRACKED_LIST"
untracked_count="$(sed '/^$/d' "$UNTRACKED_LIST" | wc -l | tr -d '[:space:]')"
untracked_lines=0
largest_rows=()

while IFS= read -r path; do
  [[ -n "$path" ]] || continue
  [[ -f "$path" ]] || continue
  lines="$(wc -l <"$path" | tr -d '[:space:]')"
  bytes="$(wc -c <"$path" | tr -d '[:space:]')"
  untracked_lines=$((untracked_lines + lines))
  largest_rows+=("$bytes $lines $path")
done <"$UNTRACKED_LIST"

visible_cache_paths="$(
  {
    git ls-files 'docs/verification/**/cache/**' 'docs/verification/**/*cache*/**'
    git ls-files -o --exclude-standard 'docs/verification/**/cache/**' 'docs/verification/**/*cache*/**'
  } | sort -u
)"
visible_cache_count="$(
  if [[ -n "$visible_cache_paths" ]]; then
    printf '%s\n' "$visible_cache_paths" | sed '/^$/d' | wc -l | tr -d '[:space:]'
  else
    printf '0'
  fi
)"

for manifest in "${REQUIRED_MANIFESTS[@]}"; do
  if [[ ! -f "$manifest" ]]; then
    printf '%s\n' "$manifest" >>"$MISSING_MANIFESTS_LIST"
  fi
done
missing_manifest_count="$(sed '/^$/d' "$MISSING_MANIFESTS_LIST" | wc -l | tr -d '[:space:]')"

find . \
  \( -path './.git' -o -path './target' -o -path '*/node_modules' -o -path './apps/dist' -o -path './apps/dashboard/dist' -o -path './apps/.astro' -o -path './.memd' \) -prune \
  -o -type d -empty -print \
  | sort >"$EMPTY_DIRS_LIST"
empty_dir_count="$(sed '/^$/d' "$EMPTY_DIRS_LIST" | wc -l | tr -d '[:space:]')"

find crates apps integrations tests scripts \
  \( -path '*/node_modules' -o -path 'apps/dist' -o -path './apps/dist' -o -path 'apps/dashboard/dist' -o -path './apps/dashboard/dist' -o -path 'apps/.astro' -o -path './apps/.astro' \) -prune \
  -o -type f \( -name '*.rs' -o -name '*.ts' -o -name '*.tsx' -o -name '*.js' -o -name '*.astro' -o -name '*.sh' -o -name '*.py' \) -print 2>/dev/null \
  | while IFS= read -r path; do
      [[ -f "$path" ]] || continue
      lines="$(wc -l <"$path" | tr -d '[:space:]')"
      if ((lines > MAX_SOURCE_LINES)); then
        printf '%s %s\n' "$lines" "$path"
      fi
    done \
  | sort -nr >"$OVERSIZED_LIST"
oversized_count="$(sed '/^$/d' "$OVERSIZED_LIST" | wc -l | tr -d '[:space:]')"

echo "repo_hygiene root=$ROOT"
echo "missing_manifests=$missing_manifest_count"
echo "empty_dirs=$empty_dir_count max=$MAX_EMPTY_DIRS"
echo "untracked_files=$untracked_count max=$MAX_UNTRACKED_FILES"
echo "untracked_lines=$untracked_lines max=$MAX_UNTRACKED_LINES"
echo "visible_cache_paths=$visible_cache_count max=$MAX_VISIBLE_CACHE_PATHS"
echo "oversized_source_files=$oversized_count max_lines=$MAX_SOURCE_LINES fail_on_oversized=$FAIL_ON_OVERSIZED"

if ((missing_manifest_count > 0)); then
  echo "missing_manifest_paths"
  cat "$MISSING_MANIFESTS_LIST"
fi

if ((empty_dir_count > 0)); then
  echo "empty_dirs"
  cat "$EMPTY_DIRS_LIST"
fi

if ((${#largest_rows[@]} > 0)); then
  echo "largest_untracked_files bytes lines path"
  printf '%s\n' "${largest_rows[@]}" | sort -nr | head -20
fi

if [[ -n "$visible_cache_paths" ]]; then
  echo "cache_path_violations"
  printf '%s\n' "$visible_cache_paths"
fi

if ((oversized_count > 0)); then
  echo "oversized_source_files lines path"
  head -30 "$OVERSIZED_LIST"
fi

failed=0
if ((missing_manifest_count > 0)); then
  echo "FAIL required manifests are missing" >&2
  failed=1
fi
if ((empty_dir_count > MAX_EMPTY_DIRS)); then
  echo "FAIL empty directory count exceeds limit" >&2
  failed=1
fi
if ((untracked_count > MAX_UNTRACKED_FILES)); then
  echo "FAIL untracked file count exceeds limit" >&2
  failed=1
fi
if ((untracked_lines > MAX_UNTRACKED_LINES)); then
  echo "FAIL untracked line count exceeds limit" >&2
  failed=1
fi
if ((visible_cache_count > MAX_VISIBLE_CACHE_PATHS)); then
  echo "FAIL repo-visible verification cache paths found" >&2
  failed=1
fi
if [[ "$FAIL_ON_OVERSIZED" == "1" || "$FAIL_ON_OVERSIZED" == "true" ]]; then
  if ((oversized_count > 0)); then
    echo "FAIL oversized source files found" >&2
    failed=1
  fi
fi

exit "$failed"
