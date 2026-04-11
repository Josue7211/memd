#!/usr/bin/env bash
set -euo pipefail

apply=0
base_branch="main"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --apply)
      apply=1
      shift
      ;;
    --base)
      base_branch="${2:?missing branch after --base}"
      shift 2
      ;;
    *)
      echo "unknown arg: $1" >&2
      echo "usage: $0 [--base main] [--apply]" >&2
      exit 1
      ;;
  esac
done

git rev-parse --show-toplevel >/dev/null 2>&1
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

worktree_branches="$(git worktree list --porcelain | awk '/^branch /{sub("refs/heads/","",$2); print $2}' | sort -u)"
merged_branches="$(git branch --merged "$base_branch" | sed 's/^..//' | sed "/^${base_branch}\$/d" | sort -u)"
prunable="$(comm -23 <(printf '%s\n' "$merged_branches" | sed '/^$/d') <(printf '%s\n' "$worktree_branches" | sed '/^$/d'))"
blocked="$(comm -12 <(printf '%s\n' "$merged_branches" | sed '/^$/d') <(printf '%s\n' "$worktree_branches" | sed '/^$/d'))"

echo "REPO $repo_root"
echo "BASE $base_branch"
echo
echo "PRUNABLE_BRANCHES"
printf '%s\n' "$prunable"
echo
echo "BLOCKED_BY_WORKTREE"
printf '%s\n' "$blocked"
echo

if [[ $apply -eq 0 ]]; then
  echo "dry-run only; rerun with --apply to delete prunable branches"
  exit 0
fi

if [[ -z "${prunable//$'\n'/}" ]]; then
  echo "nothing to delete"
  exit 0
fi

failed=0
while IFS= read -r branch; do
  [[ -z "$branch" ]] && continue
  if ! git branch -d "$branch"; then
    echo "FAILED $branch" >&2
    failed=1
  fi
done < <(printf '%s\n' "$prunable")

exit $failed
