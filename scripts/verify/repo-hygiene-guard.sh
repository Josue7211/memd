#!/usr/bin/env bash
set -euo pipefail

ROOT="${ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
cd "$ROOT"

MAX_UNTRACKED_FILES="${MEMD_HYGIENE_MAX_UNTRACKED_FILES:-200}"
MAX_UNTRACKED_LINES="${MEMD_HYGIENE_MAX_UNTRACKED_LINES:-50000}"
MAX_VISIBLE_CACHE_PATHS="${MEMD_HYGIENE_MAX_VISIBLE_CACHE_PATHS:-0}"

UNTRACKED_LIST="$(mktemp "${TMPDIR:-/tmp}/memd-hygiene-untracked.XXXXXX")"
trap 'rm -f "$UNTRACKED_LIST"' EXIT

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

echo "repo_hygiene root=$ROOT"
echo "untracked_files=$untracked_count max=$MAX_UNTRACKED_FILES"
echo "untracked_lines=$untracked_lines max=$MAX_UNTRACKED_LINES"
echo "visible_cache_paths=$visible_cache_count max=$MAX_VISIBLE_CACHE_PATHS"

if ((${#largest_rows[@]} > 0)); then
  echo "largest_untracked_files bytes lines path"
  printf '%s\n' "${largest_rows[@]}" | sort -nr | head -20
fi

if [[ -n "$visible_cache_paths" ]]; then
  echo "cache_path_violations"
  printf '%s\n' "$visible_cache_paths"
fi

failed=0
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

exit "$failed"
