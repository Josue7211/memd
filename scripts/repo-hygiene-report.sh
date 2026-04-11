#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
cd "$ROOT"

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
