> Secondary/reference doc. For current project truth start with [[ROADMAP]] and [[docs/WHERE-AM-I.md|WHERE-AM-I]].

# Feature Slice 25: Shared Research Cache and Donor Repo Extraction

## Honest status

Status: **strong local proof** for the implemented inspiration/research-lane cache slice; dogfood/external remain unverified.

The current repository has an implemented local inspiration/research lane search cache in `crates/memd-client/src/runtime/inspiration_search.rs` and a targeted Rust regression test that now exercises the safety-critical local gestures for this slice. This is strong local evidence for the implemented cache behavior, attribution, invalidation, and isolation boundaries. It is still not a complete shared multi-repo RAG/donor-extraction product proof.

## What this proof validates

The executable proof is `scripts/verify/feature-shared-research-cache-proof.sh`. It validates the existing implementation and artifacts for:

1. **Cache miss and cache hit behavior**
   - Confirms the implementation returns `cache_hits: 0` and scans files on a cold search.
   - Confirms the Rust test asserts a warm unchanged-file search returns `cache_hits=1` and `scanned=0`.
   - Runs the targeted test `inspiration_search_strong_local_cache_proof` through `scripts/memd-cargo-guard.sh` when available.
2. **Source attribution**
   - Confirms rendered search output includes source file path, line number, section, and matched text.
   - Confirms the implementation carries `file`, `line`, `section`, and `text` in `InspirationHit`.
3. **Fingerprint invalidation**
   - Confirms cache entries include path, length, modified time, and content `sha256` fingerprints.
   - Confirms changing an allowlisted inspiration file forces a cache miss/rescan and returns updated text.
4. **Allowlist and root isolation**
   - Confirms cache keys include the resolved root, normalized query, and limit.
   - Confirms cache reads require matching root, query, limit, file count, file path, length, modified timestamp, and content hash before returning a hit.
   - Confirms only the fixed inspiration lane files are searched, rather than arbitrary repository paths.
   - Confirms a second root with the same query does not reuse the first root cache.
5. **No private-data bleed for this local slice**
   - Confirms a non-allowlisted private file containing the query and a sentinel secret is not returned in hits, rendered summaries, or cache artifacts.
   - This is a narrow allowlist guardrail, not a full secret scanner, PII detector, or policy engine.

## Known gaps / not proven

- No complete cross-repository shared cache workflow is proven here.
- No RAG sidecar cache hit/miss proof is included; RAG remains optional and separate from this local inspiration search cache.
- No donor repository extraction pipeline with cloned donor repos, provenance manifests, license metadata, or freshness invalidation is proven by this slice.
- No adversarial private-data or secret-leak test exists for arbitrary donor content. The current local guardrail is the fixed-file allowlist plus cache-artifact sentinel check.
- No external replay or independent audit exists.

## Commands

```bash
bash scripts/verify/feature-shared-research-cache-proof.sh
bash scripts/verify/feature-registry-audit.sh
bash scripts/doc-lint.sh
git diff --check
```

## Claim boundary

Allowed claim: memd has strong local proof that the implemented inspiration/research lane search cache reuses unchanged-file results, preserves source attribution, invalidates on content fingerprint change, isolates cache entries by root/query/limit and allowlisted files, and does not bleed non-allowlisted private fixture data into hits/renders/cache artifacts.

Forbidden claim: do not claim production-safe shared research cache, complete RAG cache safety, complete donor repo extraction, dogfood validation, external verification, or contamination-free/private-data-safe arbitrary multi-repo sharing from this proof alone.
