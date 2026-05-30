#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SRC="$ROOT/crates/memd-client/src/runtime/inspiration_search.rs"
TEST="$ROOT/crates/memd-client/src/main_tests/skill_workflow_tests/mod.rs"
DOC="$ROOT/docs/verification/feature-shared-research-cache-25.md"

audit() { printf 'feature-shared-research-cache-proof: %s\n' "$*"; }
fail() { printf 'feature-shared-research-cache-proof: ERROR: %s\n' "$*" >&2; exit 1; }
require_file() { [[ -f "$1" ]] || fail "missing required file: $1"; }
require_grep() {
  local pattern=$1
  local file=$2
  local label=$3
  grep -Eq "$pattern" "$file" || fail "$label not found in $file"
}

require_file "$SRC"
require_file "$TEST"
require_file "$DOC"

# Implementation surface: local research/inspiration cache exists under memd state.
require_grep '\.inspiration-cache' "$SRC" 'cache directory name'
require_grep 'root=\{\}\|query=\{\}\|limit=\{\}' "$SRC" 'cache key includes root/query/limit'
require_grep 'normalize_query\(query\)' "$SRC" 'cache key query normalization'
require_grep 'len: metadata\.len\(\)' "$SRC" 'file length fingerprint'
require_grep 'modified' "$SRC" 'file modified-time fingerprint'
require_grep 'sha256: format!\("\{:x\}", Sha256::digest\(&content\)\)' "$SRC" 'file content sha256 fingerprint'

# Cache hit/miss semantics in implementation and existing regression test.
require_grep 'cache_hits: 0' "$SRC" 'cold cache miss counter'
require_grep 'cache_scanned: scanned' "$SRC" 'cold cache scanned counter'
require_grep 'cache_hits: cache\.files\.len\(\)' "$SRC" 'warm cache hit counter'
require_grep 'cache_scanned: 0' "$SRC" 'warm cache avoids scan'
require_grep 'inspiration_search_strong_local_cache_proof' "$TEST" 'targeted cache reuse test'
require_grep 'assert_eq!\(first\.cache_hits, 0\)' "$TEST" 'test cold miss assertion'
require_grep 'assert_eq!\(first\.cache_scanned, 1\)' "$TEST" 'test cold scan assertion'
require_grep 'assert_eq!\(second\.cache_hits, 1\)' "$TEST" 'test warm hit assertion'
require_grep 'assert_eq!\(second\.cache_scanned, 0\)' "$TEST" 'test warm no-scan assertion'
require_grep 'changed fingerprint must miss cache' "$TEST" 'test fingerprint invalidation assertion'
require_grep 'different root must not reuse RootA cache' "$TEST" 'test root isolation assertion'
require_grep 'private non-allowlisted data bled into cache' "$TEST" 'test no private-data bleed assertion'

# Source attribution: hit record and render include file, line, section, text.
require_grep 'struct InspirationHit' "$SRC" 'hit struct'
require_grep 'file: PathBuf' "$SRC" 'source file attribution field'
require_grep 'line: usize' "$SRC" 'line attribution field'
require_grep 'section: String' "$SRC" 'section attribution field'
require_grep 'text: String' "$SRC" 'matched text attribution field'
require_grep 'hit\.file\.display\(\)' "$SRC" 'rendered source file path'
require_grep 'hit\.line' "$SRC" 'rendered source line'
require_grep 'hit\.section' "$SRC" 'rendered source section'

# Contamination/private-data guardrails that are actually present.
require_grep 'cache\.root == root\.display\(\)\.to_string\(\)' "$SRC" 'cache root isolation check'
require_grep 'cache\.query == cache_key_query' "$SRC" 'cache query isolation check'
require_grep 'cache\.limit == limit' "$SRC" 'cache limit isolation check'
require_grep 'left\.path == right\.path' "$SRC" 'cache fingerprint path equality check'
require_grep 'left\.sha256 == right\.sha256' "$SRC" 'cache fingerprint content equality check'
require_grep 'const INSPIRATION_FILES' "$SRC" 'fixed inspiration file allowlist'
require_grep '\.memd/lanes/inspiration/INSPIRATION-LANE\.md' "$SRC" 'allowlisted lane file'
require_grep '\.memd/lanes/inspiration/INSPIRATION-ARCHITECTURE\.md' "$SRC" 'allowlisted architecture file'
require_grep '\.memd/lanes/inspiration/INSPIRATION-BACKLOG\.md' "$SRC" 'allowlisted backlog file'
require_grep '\.memd/lanes/inspiration/INSPIRATION-MATRIX\.md' "$SRC" 'allowlisted matrix file'

# Documentation must be honest about incomplete implementation.
require_grep 'strong local proof' "$DOC" 'honest strong local status'
require_grep 'No complete cross-repository shared cache workflow is proven here' "$DOC" 'cross-repo gap disclosure'
require_grep 'No RAG sidecar cache hit/miss proof is included' "$DOC" 'RAG gap disclosure'
require_grep 'No donor repository extraction pipeline' "$DOC" 'donor extraction gap disclosure'
require_grep 'not a full secret scanner' "$DOC" 'private-data guardrail limitation'

# Run the existing targeted Rust regression test when the cargo guard is available.
if [[ -x "$ROOT/scripts/memd-cargo-guard.sh" ]]; then
  audit 'running targeted Rust regression: inspiration_search_strong_local_cache_proof'
  (cd "$ROOT" && bash scripts/memd-cargo-guard.sh test -p memd-client inspiration_search_strong_local_cache_proof)
else
  audit 'WARNING: scripts/memd-cargo-guard.sh is not executable; static proof only'
fi

audit 'ok: strong local proof validates inspiration cache hit/miss, attribution, fingerprint invalidation, allowlist/root isolation, and no private-data bleed'
