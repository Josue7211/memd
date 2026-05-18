#!/usr/bin/env bash
set -euo pipefail

# Some Apple/Xcode ar builds on the T7 toolchain reject deterministic `D`
# flags emitted by native Rust build scripts. Keep memd Cargo isolated and
# strip only that unsupported modifier before delegating to the real archiver.

REAL_AR="${MEMD_REAL_AR:-/usr/bin/ar}"
args=()
for arg in "$@"; do
  case "$arg" in
    -*D*)
      cleaned="-${arg#-}"
      cleaned="${cleaned//D/}"
      [[ "$cleaned" == "-" ]] || args+=("$cleaned")
      ;;
    *D*)
      if [[ "$arg" =~ ^[A-Za-z]+$ ]]; then
        cleaned="${arg//D/}"
        [[ -z "$cleaned" ]] || args+=("$cleaned")
      else
        args+=("$arg")
      fi
      ;;
    *)
      args+=("$arg")
      ;;
  esac
done

if [[ "${MEMD_AR_WRAPPER_DEBUG:-0}" == "1" ]]; then
  printf '%s\n' "${args[@]}"
  exit 0
fi

exec "$REAL_AR" "${args[@]}"
