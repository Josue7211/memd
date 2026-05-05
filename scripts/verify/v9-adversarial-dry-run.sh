#!/usr/bin/env bash
# F9 rehearsal wrapper for the V9 adversarial suite.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MODE=dry-run "$REPO_ROOT/scripts/verify/v9-adversarial-suite.sh"
