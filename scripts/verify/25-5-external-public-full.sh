#!/usr/bin/env bash
# Full external public-dataset proof gate.
#
# This is intentionally opt-in. It writes a blocked report by default so
# market-claim gates can point at the missing full-corpus proof without any
# agent accidentally launching a huge benchmark run during implementation.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${OUT_DIR:-$ROOT/docs/verification/25-5-memory-os-runs}"
RUN_DATE="${RUN_DATE:-$(date +%F)}"
REPORT="${REPORT:-$OUT_DIR/${RUN_DATE}-external-public-full.json}"
ALLOW_FULL_PUBLIC_PROOF="${ALLOW_FULL_PUBLIC_PROOF:-0}"

mkdir -p "$OUT_DIR"

write_blocked() {
  python3 - "$REPORT" <<'PY'
import json
import pathlib
import sys

report = pathlib.Path(sys.argv[1])
payload = {
    "suite": "25_5_external_public_full",
    "status": "blocked",
    "reason": "full external public proof is intentionally opt-in",
    "required": (
        "Set ALLOW_FULL_PUBLIC_PROOF=1 plus an explicit PUBLIC_BENCH_LIMIT, "
        "PUBLIC_BENCH_TIMEOUT, and RUN_LABEL if you really intend to run the "
        "long full-public proof. Do not run this during incremental implementation."
    ),
    "market_claim": "blocked",
}
report.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"25_5_external_public_full blocked report={report}")
PY
}

if [[ "$ALLOW_FULL_PUBLIC_PROOF" != "1" ]]; then
  write_blocked
  exit 2
fi

if [[ -z "${PUBLIC_BENCH_LIMIT:-}" ]]; then
  write_blocked
  exit 2
fi

export RUN_LABEL="${RUN_LABEL:-external-public-full}"
export SUITE_NAME="${SUITE_NAME:-25_5_external_public_full}"
export PUBLIC_BENCH_TIMEOUT="${PUBLIC_BENCH_TIMEOUT:-7200}"

exec "$ROOT/scripts/verify/25-5-external-public-scale.sh"
