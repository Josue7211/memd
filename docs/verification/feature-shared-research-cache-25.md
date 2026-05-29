> Secondary/reference doc. For current project truth start with [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]].

# Feature Slice 25: Shared Research Cache and Donor Repo Extraction

## Honest status

Status: **partial local proof only**.

The current repository has an implemented local inspiration/research lane search cache in `crates/memd-client/src/runtime/inspiration_search.rs` and a targeted Rust regression test for cache reuse. This is useful evidence for the shared research cache slice, but it is not a complete shared multi-repo RAG/donor-extraction product proof.

## What this proof validates

The executable proof is `scripts/verify/feature-shared-research-cache-proof.sh`. It validates the existing implementation and artifacts for:

1. **Cache miss and cache hit behavior**
   - Confirms the implementation returns `cache_hits: 0` and scans files on a cold search.
   - Confirms the existing Rust test asserts a warm search returns `cache_hits=1` and `scanned=0`.
   - Runs the targeted test `inspiration_search_reuses_cache_for_unchanged_files` through `scripts/memd-cargo-guard.sh` when available.
2. **Source attribution**
   - Confirms rendered search output includes source file path, line number, section, and matched text.
   - Confirms the implementation carries `file`, `line`, `section`, and `text` in `InspirationHit`.
3. **Contamination guardrails currently present**
   - Confirms cache keys include the resolved root, normalized query, and limit.
   - Confirms cache reads require matching root, query, limit, file count, file path, length, and modified timestamp before returning a hit.
   - Confirms only the fixed inspiration lane files are searched, rather than arbitrary repository paths.
4. **Private-data guardrails currently present**
   - Confirms the current implementation searches only `.memd/lanes/inspiration/INSPIRATION-*.md` files.
   - This is a narrow allowlist guardrail, not a full secret scanner, PII detector, or policy engine.

## Known gaps / not proven

- No complete cross-repository shared cache workflow is proven here.
- No RAG sidecar cache hit/miss proof is included; RAG remains optional and separate from this local inspiration search cache.
- No donor repository extraction pipeline with cloned donor repos, provenance manifests, license metadata, or freshness invalidation is proven by this slice.
- No adversarial private-data or secret-leak test exists for arbitrary donor content. The current guardrail is only the fixed-file allowlist plus source attribution.
- No external replay or independent audit exists.

## Commands

```bash
bash scripts/verify/feature-shared-research-cache-proof.sh
bash scripts/verify/feature-registry-audit.sh
bash scripts/doc-lint.sh
git diff --check
```

## Claim boundary

Allowed claim: memd has partial local proof that the inspiration/research lane search cache reuses unchanged-file results, preserves source attribution, and avoids cross-root cache reuse for the current allowlisted lane files.

Forbidden claim: do not claim production-safe shared research cache, complete RAG cache safety, complete donor repo extraction, or contamination-free/private-data-safe multi-repo sharing from this proof alone.
