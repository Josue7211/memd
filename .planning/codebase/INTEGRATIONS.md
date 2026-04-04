# Integrations

## External Memory Backend

The repo treats the semantic backend as optional and external.

Primary documents:

- `docs/rag.md`
- `docs/backend-api.md`
- `docs/backend-stack.md`
- `docs/backend-ownership.md`

## Backend Stack Contract

- `rag-sidecar` — HTTP boundary service
- `MinerU` — document extraction dependency
- `RAGAnything` — multimodal retrieval dependency
- `LightRAG` — intended semantic backend family

## First-Party Integrations

- Claude Code: `integrations/claude-code/README.md`
- Codex: `integrations/codex/README.md`
- OpenClaw: `integrations/openclaw/README.md`
- shared hook kit: `integrations/hooks/README.md`

Mission Control is mentioned in roadmap and attach flow but does not currently
have a dedicated integration directory in the repo.

## Bundle and Attach

Bundle bootstrap is driven from:

- `crates/memd-client/src/main.rs`

Generated project bundles contain:

- `config.json`
- `env` and `env.ps1`
- `backend.env` and `backend.env.ps1`
- `hooks/`
- `agents/`

## Multimodal and Obsidian

- multimodal ingest logic: `crates/memd-multimodal/src/lib.rs`
- obsidian bridge: `crates/memd-client/src/obsidian.rs`
- obsidian docs: `docs/obsidian.md`
- obsidian is now also part of the intended compiled-wiki product path, not only a vault import bridge

## Operational Scripts

- LightRAG status check: `scripts/lightrag-hourly-check.sh`
- monitor artifacts: `.monitor/lightrag-last-status.json`

## Integration Boundary Rule

`memd` is the control plane. External systems can plug in, but they should not
own canonical memory semantics.
