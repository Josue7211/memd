# External Replay and Auditor Readiness Proof - 25-Star Slice

Secondary/reference doc. Start from [[ROADMAP]] for project truth.

## Scope

Feature: `feature.external_replay_auditor_proof`

This proof is a strong local readiness check for external replay and auditor handoff. It verifies public-facing instructions, checklist structure, registry wiring, concrete artifact paths, schema/checksum expectations, immutability guards, and a fresh local `25-5-external-public-smoke` generation/validation path. It does not prove that an independent outside auditor has run the proof.

## Honest Status

- Current status: partial
- Proof status: strong local readiness proof
- Dogfood status: none
- External status: planned
- Independent external replay: not verified
- 25/25 blocker: yes, until a real independent replay/auditor artifact is recorded

Allowed claim from this slice: strong local external-replay/auditor readiness is wired and checked by `bash scripts/verify/feature-external-replay-auditor-proof.sh`, including local public-smoke artifact generation/validation without persistent dirty proof output.

Forbidden claim: do not claim externally verified, third-party replayed, or auditor-approved from this document alone.

## What the proof validates

1. Replay bundle/schema/instructions
   - `docs/verification/features.schema.json` is valid JSON and describes required registry fields.
   - `docs/verification/features.registry.json` contains exactly one external replay/auditor feature row.
   - The row links this proof doc and the executable proof command.
   - Existing public replay runner `scripts/verify/25-5-external-public-smoke.sh` is still listed as replay preparation.
   - Registry proof artifacts are concrete local files, not globs/descriptions, and the registered `external-public-smoke.json` report validates as a passing memd/no-RAG public replay artifact.
   - Release proof bundle directory `docs/verification/release-1-0-0/` exists when present and contains paired `.md`/`.ndjson` proof artifacts.

2. Auditor checklist
   - `docs/verification/EXTERNAL-25-STAR-VERIFIERS.md` lists EV-01 through EV-12, required pass evidence, evidence packet fields, and the close rule.
   - `docs/verification/25-star-human-trial-template.md` contains external verifier instructions, result form fields, and minimum pass criteria.

3. Artifact immutability/checksums, if artifacts are present
   - Release bundle files are regular files, not symlinks.
   - SHA-256 digests are computed for local release bundle files during the proof run.
   - `.ndjson` artifacts are parsed line-by-line as JSON and must include status records.
   - Public benchmark checksum rows in `docs/verification/PUBLIC_BENCHMARKS.md` are inspected; if referenced dataset files exist locally, their SHA-256 must match the documented `sha256:` value. Missing local datasets are reported as skipped, not external proof.

4. 25-5-external-public-smoke artifact generation/validation
   - The proof executes `scripts/verify/25-5-external-public-smoke.sh` with `OUT_DIR` and `DATASET_CACHE_DIR` pointed at a temporary directory, `PUBLIC_BENCH_LIMIT=1` by default, and `MEMD_RAG_URL` unset.
   - The generated report must cover LongMemEval, LoCoMo, MemBench, and ConvoMem; use the memd backend; expose public source URLs and `sha256:` dataset checksums; pass accuracy/recall gates; and contain item-level hit/top-id evidence.
   - The temporary output is deleted and the proof compares git porcelain before/after, failing if the generation leaves persistent dirty noise.

## Evidence boundaries

This slice creates a reproducible local readiness proof. It intentionally leaves `external_status` as `planned` because no independent external artifact is included in this commit.

A future external close requires a separate artifact that records at least:

```text
verifier_id:
person/role:
os:
harness:
commit:
commands_run:
result: pass|fail|blocked
help_needed: yes|no
blockers:
artifacts:
replayed_bundle_or_commit:
checksum_manifest:
```

## Run

```bash
bash scripts/verify/feature-external-replay-auditor-proof.sh
```

Expected result: `feature-external-replay-auditor-proof: ok`.
