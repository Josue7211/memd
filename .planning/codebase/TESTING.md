# Testing

## Current Test Strategy

The repo relies primarily on Rust unit and integration-style tests executed via:

```bash
cargo test -q
```

## Evidence in Repo

- schema roundtrip tests in `crates/memd-schema/src/lib.rs`
- routing tests in `crates/memd-server/src/routing.rs`
- store and maintenance behavior tests in server/store-related code
- multimodal request tests in `crates/memd-multimodal/src/lib.rs`
- sidecar fixture and contract tests in `crates/memd-sidecar/src/fixtures.rs`

## Strengths

- request/response contract changes are often covered by serde roundtrip tests
- workspace-wide `cargo test -q` is fast enough to run frequently
- recent work has been validated immediately after schema and API changes

## Gaps

- end-to-end behavior across client, server, and integrations is still lightly covered
- hook/bootstrap scripts are validated mostly by code and manual reasoning, not deep automated tests
- roadmap and planning state are not automatically verified against implementation
- provenance and repair workflows need stronger direct coverage as they deepen

## Recommended Near-Term Additions

- tests for working-memory admission/eviction semantics
- tests for provenance drilldown once implemented
- tests for repair actions after they land
- tests that assert bundle-first behavior across all main CLI paths

## Operational Note

Because the repo is still compact, most changes can be validated quickly with
workspace tests, but that will become less sufficient as `v1` closes and `v2`
adds richer policy behavior.
