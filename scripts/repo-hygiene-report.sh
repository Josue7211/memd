#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
cd "$ROOT"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOST_IO_GUARD="${HOST_IO_GUARD:-$SCRIPT_DIR/memd-host-io-guard.sh}"
if [ "${HOST_IO_GUARD_ENABLED:-1}" != "0" ] && [ "${HOST_IO_GUARD_ENABLED:-1}" != "false" ]; then
  "$HOST_IO_GUARD"
fi

if ! git rev-parse --show-toplevel >/dev/null 2>&1; then
  echo "not a git repo: $ROOT" >&2
  exit 1
fi

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

echo "REPO $repo_root"
echo

echo "TOP_DIR_SIZES"
du -sh .[!.]* * 2>/dev/null | sort -hr | head -40
echo

echo "BIGGEST_RUST_FILES"
find crates -type f -name '*.rs' -print0 \
  | xargs -0 wc -l \
  | sort -nr \
  | head -30
echo

MAX_SOURCE_LINES="${MEMD_HYGIENE_MAX_SOURCE_LINES:-3000}"

echo "OVERSIZED_SOURCE_FILES_OVER_${MAX_SOURCE_LINES}_LINES"
find crates apps integrations tests scripts \
  \( -path '*/node_modules' -o -path 'apps/dist' -o -path './apps/dist' -o -path 'apps/dashboard/dist' -o -path './apps/dashboard/dist' -o -path 'apps/.astro' -o -path './apps/.astro' \) -prune \
  -o -type f \( -name '*.rs' -o -name '*.ts' -o -name '*.tsx' -o -name '*.js' -o -name '*.astro' -o -name '*.sh' -o -name '*.py' \) -print 2>/dev/null \
  | while IFS= read -r path; do
      [[ -f "$path" ]] || continue
      lines="$(wc -l <"$path" | tr -d '[:space:]')"
      if [[ "$lines" -gt "$MAX_SOURCE_LINES" ]]; then
        printf '%s %s\n' "$lines" "$path"
      fi
    done \
  | sort -nr \
  | head -50
echo

echo "EMPTY_DIRECTORIES"
find . \
  \( -path './.git' -o -path './target' -o -path '*/node_modules' -o -path './apps/dist' -o -path './apps/dashboard/dist' -o -path './apps/.astro' -o -path './.memd' \) -prune \
  -o -type d -empty -print \
  | sort
echo

echo "REQUIRED_MANIFESTS"
for path in Cargo.toml Cargo.lock apps/package.json apps/package-lock.json apps/dashboard/package.json apps/dashboard/package-lock.json; do
  if [[ -f "$path" ]]; then
    printf 'present %s\n' "$path"
  else
    printf 'missing %s\n' "$path"
  fi
done
echo

echo "LOCAL_WORKTREES"
git worktree list
echo

worktree_branches="$(git worktree list --porcelain | awk '/^branch /{sub("refs/heads/","",$2); print $2}' | sort -u)"
merged_branches="$(git branch --merged main | sed 's/^..//' | sed '/^main$/d' | sort -u)"

echo "MERGED_BRANCHES_DELETABLE_NOW"
comm -23 <(printf '%s\n' "$merged_branches" | sed '/^$/d') <(printf '%s\n' "$worktree_branches" | sed '/^$/d')
echo

echo "MERGED_BRANCHES_BLOCKED_BY_WORKTREE"
comm -12 <(printf '%s\n' "$merged_branches" | sed '/^$/d') <(printf '%s\n' "$worktree_branches" | sed '/^$/d')
echo

echo "UNMERGED_LOCAL_BRANCHES"
git for-each-ref refs/heads --format='%(refname:short)' \
  | sort -u \
  | comm -23 - <(printf '%s\nmain\n' "$merged_branches" | sort -u)
echo

echo "PRUNABLE_WORKTREES"
git worktree list --porcelain | awk '
  /^worktree /{path=$2}
  /^prunable /{print path " :: " substr($0,10)}
'
echo

echo "LIKELY_LOCAL_JUNK"
for path in target .memd .monitor memd.db .watch-smoke-new; do
  if [ -e "$path" ]; then
    du -sh "$path" 2>/dev/null || true
  fi
done
echo

echo "TRACKED_SUPERPOWERS_FILES"
git ls-files .superpowers 2>/dev/null || true
echo

echo "DIRTY_STATUS"
git status --short
