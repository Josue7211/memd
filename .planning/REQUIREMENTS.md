# Requirements: memd

**Defined:** 2026-04-04
**Core Value:** Give agents short-term and long-term memory that stays compact, durable, inspectable, and useful under real task pressure.

## v0 Requirements

### OSS Foundations

- [ ] **OSS-01**: Active development happens on a dedicated branch rather than directly on `main`.
- [ ] **OSS-02**: Large files are split only where the seam improves reuse and maintenance.
- [ ] **OSS-03**: Contribution, security, and review expectations are explicit enough for outside contributors.
- [ ] **OSS-04**: Version history and release workflow are documented for phased work.
- [ ] **OSS-05**: Public project guidance is separated from internal planning artifacts.

## Traceability

| Requirement | Phase / Version | Status |
|-------------|-----------------|--------|
| OSS-01 | v0 | In Progress |
| OSS-02 | v0 | In Progress |
| OSS-03 | v0 | In Progress |
| OSS-04 | v0 | In Progress |
| OSS-05 | v0 | In Progress |

## v1 Requirements

### Core Memory

- [ ] **CORE-01**: Agent can store typed candidate and canonical memory records.
- [ ] **CORE-02**: Agent can search memory with bounded responses and stable routing.
- [ ] **CORE-03**: Memory lifecycle supports verify, expire, supersede, and dedupe flows.
- [ ] **CORE-04**: Memory remains usable without any external semantic backend.

### Working and Retrieval

- [ ] **WORK-01**: Agent can fetch compact context by route and intent.
- [ ] **WORK-02**: Agent can fetch managed working memory with explicit budget reporting.
- [ ] **WORK-03**: Working memory exposes admission, eviction, and rehydration signals.
- [ ] **WORK-04**: Retrieval remains compact enough for hot-path agent use.

### Quality and Explainability

- [ ] **QUAL-01**: Memory inbox surfaces candidate, stale, contested, and superseded items.
- [ ] **QUAL-02**: Explain flow shows why a memory exists and how it was ranked.
- [ ] **QUAL-03**: Operators can inspect active memory policy defaults and thresholds.
- [ ] **QUAL-04**: Provenance can be drilled down from summary memory to source artifacts.
- [ ] **QUAL-05**: Operators can repair stale, contested, or malformed memory state.

### Integration

- [ ] **INTG-01**: Project bundle can configure server and semantic backend defaults.
- [ ] **INTG-02**: Agent attach flow works for Claude Code, Codex, Mission Control, and OpenClaw.
- [ ] **INTG-03**: Optional semantic backend stays behind the documented sidecar contract.
- [ ] **INTG-04**: Obsidian and multimodal ingest preserve memory usefulness and provenance.

## v2 Requirements

### Superhuman Memory

- **SUPR-01**: Working memory uses explicit admission, eviction, and rehydration policy.
- **SUPR-02**: Durable memory preserves trust, freshness, provenance, and contradiction state on every belief.
- **SUPR-03**: Conflicting beliefs remain branchable and inspectable.
- **SUPR-04**: Retrieval policy improves from usage outcomes instead of static heuristics alone.
- **SUPR-05**: Summaries can be reversed into deeper evidence without lossy hallucination.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Full cognition / planning stack | That belongs in `braind`, not `memd` |
| Transcript archival as the main memory model | Opposes compact, typed, high-signal memory |
| Direct backend-specific coupling | Breaks portability and control-plane boundaries |

## Traceability

| Requirement | Phase / Version | Status |
|-------------|-----------------|--------|
| CORE-01 | v1 | In Progress |
| CORE-02 | v1 | In Progress |
| CORE-03 | v1 | In Progress |
| CORE-04 | v1 | Complete |
| WORK-01 | v1 | Complete |
| WORK-02 | v1 | In Progress |
| WORK-03 | v1 | In Progress |
| WORK-04 | v1 | In Progress |
| QUAL-01 | v1 | Complete |
| QUAL-02 | v1 | Complete |
| QUAL-03 | v1 | Complete |
| QUAL-04 | v1 | Pending |
| QUAL-05 | v1 | Pending |
| INTG-01 | v1 | Complete |
| INTG-02 | v1 | Complete |
| INTG-03 | v1 | Complete |
| INTG-04 | v1 | In Progress |
| SUPR-01 | v2 | Pending |
| SUPR-02 | v2 | Pending |
| SUPR-03 | v2 | Pending |
| SUPR-04 | v2 | Pending |
| SUPR-05 | v2 | Pending |

**Coverage:**
- v1 requirements: 17 total
- Mapped to active roadmap work: 17
- Unmapped: 0

---
*Requirements defined: 2026-04-04*
*Last updated: 2026-04-04 after GSD brownfield initialization*
