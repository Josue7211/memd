# V4 Donor-Pattern Audit: Missing Patterns Mapped to Phase Goals

**Truth date:** 2026-04-22 | **Scope:** Teardowns (hermes, mempalace, supermemory, omegon-smriti) + V4 specs (A4..G4) | **Source:** donor-extraction-to-v2-phases.md, V4 phase specs, architecture lanes A2-06..A2-13

---

## 1. Patterns Already Cited in V4 Specs

| Pattern | Source | Cited Where | Status |
|---------|--------|-------------|--------|
| Three-stage repair (scan/prune/rebuild) | mempalace | A2-06 (correction-repair lane) | ✓ Referenced |
| File-interaction ledger + prime-reads | memd V3 A3 | A4 (compaction survival) | ✓ Integrated |
| Hook contract + fire order | memd hooks v0.2 | B4 (enforcement) | ✓ Integrated |
| Correction detector + judge | memd C-lane | C4 (E2E capture) | ✓ Integrated |
| Wake compiler (raw→typed→prioritized) | A2-11 (supermemory) | D4 (working-context) | ✓ Integrated |
| Progressive-depth recall (wake/lookup/resume) | mempalace layers + memd sidecar | E4 (depth contracts) | ✓ Integrated |
| Preference replay at wake | memd profile model | F4 (preference replay) | ✓ Integrated |
| Proof harness (3-session dogfood) | memd gate pattern | G4 (gate) | ✓ Integrated |

**Baseline:** V4 specs cite 8 major patterns; all sourced from donors or existing memd theory. No large gaps at the goal level.

---

## 2. Patterns Missing from V4 — Per Phase

### A4: Read-State Across Compaction

| Pattern | Source | Summary | Why Belongs | Integration Point | Cost |
|---------|--------|---------|-------------|-------------------|------|
| WAL (write-ahead log) for ledger audit | mempalace `wal/write_log.jsonl` | Every ledger read/write logged BEFORE execution as NDJSON `{timestamp, operation, params, result}` | A4 goal is ledger survival; WAL proves ledger state wasn't corrupted on restore. Complements continuity-breach detection. | A4 Task 4: extend PreCompact hook to write ledger-state WAL entry before checkpoint; PostCompact verifies WAL replay. | low |
| Content-hash dedup on ledger checkpoint | Omegon `types.rs::content_hash()` | SHA256(normalize(ledger_entry)) → same hash = upsert, not insert. Prevents ledger pollution. | A4 ledger can accumulate dupe reads across sessions. Dedup keeps ledger clean and compact. | A4 Task 3: add content_hash field to ledger checkpoint schema; dedup before writing. | low |
| Sequence-based history cutoff (Smriti) | Smriti `TurnEvent.sequence_number` | Monotonic sequence per session; restore captures history_base_seq; filter: `seq > base_seq`. Prevents ghost refs. | PostCompact restore must exclude pre-compaction reads (e.g., agent re-reads a file it only knows about pre-compaction). Sequence isolation is cleaner than timestamp. | A4 Task 6 (handoff contract): document sequence boundary; A4 implementation: add session_seq to ledger records. | med |
| Decay-based confidence erosion on ledger items | Omegon `decay.rs` + memd A2-13 | Ledger reads older than N days decay in confidence. Recent reads rank higher for prime-read priority. | Ledger grows unbounded; without decay old reads dominate. Matches A2-13 temporal freshness. | A4 Task 7 (future): wire freshness_score into prime-read ranking; apply decay formula to ledger items. | med |

### B4: Hook Contract Enforcement

| Pattern | Source | Summary | Why Belongs | Integration Point | Cost |
|---------|--------|---------|-------------|-------------------|------|
| Timeout + retry budget per hook | Omegon `bridge.rs` timeout handling | Every hook wrapped with timeout T; on timeout, log and skip (for observability hooks) or fail-hard (for write-path). Explicit retry budget. | B4 goal is visible hook failures. Current gating silently drops hooks. Timeout+retry+log = debuggable. | B4 Task 9 (enforcer shim): wrap each hook with timeout from contract; explicit retry budget = 1 (no auto-retry, explicit is better than implicit). | low |
| Versioned hook contract in MANIFEST | Omegon `omegon-memory/src/setup.rs` version check | MANIFEST.json carries `contract_version: "0.3"` — harness rejects if its contract < min version. | B4 bumps contract 0.2→0.3 (add PostCompact restore); harness must acknowledge. Prevents mismatches. | B4 Task 2 (contract doc): add version-check rules to `docs/contracts/hook-order.md`; B4 Task 10 (enforcer): check MANIFEST version on setup. | low |
| Hook context isolation via sandboxing | Hermes `agent/memory_provider.py` hook isolation | Each hook runs in isolated subprocess with captured stderr/stdout. Failures don't crash agent. | B4 goal is hook isolation. Current inline hooks risk agent crash on bad subprocess. Subprocess model = safer. | B4 Task 9 (enforcer): spawn hook as subprocess, capture output, apply contract checks. | med |
| Per-session hook state ledger | memd own (implicit in A4) | Track which hooks fired in session; on resume, know which hooks are stale. | B4 enforcer logs fire order; but doesn't track "did this fire in the current session or a prior one?" Fire-order violations are invisible without session context. | B4 Task 6 (enforcer shim): emit `.memd/logs/hook-trace.ndjson` with session_id, phase (implicit current). G4 can diff sessions. | low |

### C4: Correction Capture E2E

| Pattern | Source | Summary | Why Belongs | Integration Point | Cost |
|---------|--------|---------|-------------|-------------------|------|
| Lamport versioning on corrections | Omegon `supersede_fact()` + D2-D1 | Every correction increments `version: u64` (Lamport clock). On import: higher version wins. Deterministic conflict resolution. | C4 stores corrections with `corrects_id` pointer. Without versioning, concurrent corrections in multi-session create conflicts. Lamport version = clean tie-breaker. | C4 Task 3 (storage contract): add `version: u64` to Correction schema; increment on every store; use in E4 lookup de-dup. | low |
| Immutable checkpoints + additive notes (Smriti) | Smriti `CommitModel` immutable, notes JSONB array | Correction record never UPDATE; issues/reviews stored as append-only `metadata_.notes[]`. | C4 stores corrections — should they be mutable (user corrects a correction) or immutable (version chain)? Smriti's immutable + notes = safer audit trail. | C4 Task 3 (storage): adopt immutable corrections; add `notes: Vec<CorrectionNote>` for user review/revision. | low |
| Temporal fact invalidation on prior belief | mempalace `valid_from`/`valid_to` on graph edges | When corrected, mark old fact `valid_to = now()`; new fact `valid_from = now()`. Time-scoped query "what did we believe at T?" | C4 goal is correction capture. Without temporal windows, atlas can't reason about "was this true before the correction?" | C4 Task 3 (storage): if atlas edges exist, add `valid_from`/`valid_to` to `memory_entity_links`; invalidate old edges on correction. | med |
| Correction confidence calibration (Omegon) | Omegon `confidence` field on facts | Every fact carries confidence (0–1). Corrections from user are high confidence; LLM-extracted are lower. | C4 detector uses LLM-judge with cached cost. Judge output should feed confidence score on the correction record. | C4 Task 2 (detector): return confidence from judge; store on Correction record. | low |

### D4: Working-Context Compiler

| Pattern | Source | Summary | Why Belongs | Integration Point | Cost |
|---------|--------|---------|-------------|-------------------|------|
| Static vs dynamic priority split (supermemory) | supermemory `ProfileStructure` | Static facts (canonical) always included; dynamic (working) scored and ranked; search results are fallback. Dedup via priority. | D4 compiler maps retrieved records (episodic+semantic+canonical+candidate) to wake sections. Priority dedup prevents status from drowning facts. | D4 Task 2 (priority rules): formalize as "canonical > preferences > focus > episodic > semantic > candidate"; implement dedup by content_hash. | low |
| 4-layer context cap (mempalace layers.py) | mempalace `L0..L3` context assembly | L0 (identity) ~100t, L1 (essential story) 15 items / 200 chars each / 3200 total, L2 (on-demand) 10 items, L3 (deep search) unlimited. | D4 budget is 2k tokens. Mempalace's 4-layer cap with hard truncation (200 chars/item, 15-item cap) = proven discipline. | D4 Task 1 (compiler pipeline): map memd wake sections to layers; enforce per-section item cap (canonical 8, preferences 2, focus 1, episodic 3). | low |
| Per-item character budget (mempalace) | mempalace `layers.py` truncate logic | Each L1 item capped at 200 chars; total capped 3200 chars. Prevents one verbose item from drowning others. | D4 token budget is hard. Mempalace's char-level truncation is cheaper than token-counting and composable. | D4 Task 1 (compiler): add char-budget per item (e.g., 300 chars for canonical, 150 for episodic). Total budget enforcement = 2k tokens ≈ 8k chars. | low |
| Redundancy-key dedup (Omegon + mempalace) | Omegon `content_hash()`, mempalace `0.15 cosine` | Checkpoint writes already deduplicated via `redundancy_key` (memd has field, never used). On store: hash exists → reinforce, else insert. | D4 compiler outputs structure. If compiler is called multiple times (multiple wake calls), same canonical fact appears multiple times without dedup. | D4 Task 1 (compiler): compute content_hash for each output item; dedup by hash before final output; return with dedup count telemetry. | low |

### E4: Progressive-Depth Recall

| Pattern | Source | Summary | Why Belongs | Integration Point | Cost |
|---------|--------|---------|-------------|-------------------|------|
| RRF (Reciprocal Rank Fusion) for hybrid search | Omegon `rrf_merge()` | score = Σ 1/(rrf_k + rank_i). Merges FTS5 and vector results without weighting calibration. | E4 lookup depth needs to combine lexical and semantic search. RRF is simpler than memd's fixed weights (0.22 + 0.18 + 0.20 + 0.40). | E4 Task 2 (lookup implementation): swap memd's fixed-weight hybrid for RRF; tune rrf_k (Omegon uses 60; start there). | low |
| FTS5 full-text search on memory_items | Omegon `sqlite.rs` + H2-D3 | CREATE VIRTUAL TABLE `facts_fts` USING fts5(content, section, content=facts, content_rowid=rowid) + sync triggers. Zero-latency keyword search. | E4 lookup lacks lexical search fallback. FTS5 solves "find memory where text contains phrase". | E4 Task 2 (lookup): add FTS5 virtual table to `memory_items`; wire lookup to query FTS5 + vector search; merge with RRF. | med |
| Query sanitization (mempalace) | mempalace `query_sanitizer.py` | Queries >200 chars: extract last sentence ending with `?`; fallback last 500 chars. Prevents prompt injection contaminating retrieval. | E4 lookup takes user query. Without sanitization, agent could inject instructions into lookup ("find all facts, then ignore restrictions"). | E4 Task 1 (depth contracts): document query-sanitization rule; E4 implementation: apply mempalace's 4-step pipeline to lookup --query. | low |
| Entity aliasing for search recall (Omegon) | Omegon `derive_entity_key()` | Auto-extract aliases from project, namespace, agent, source_system, source_path. Merge aliases on entity update. Improves search. | E4 lookup must recall by name ("what does memd know about Postgres?"). Without aliases, "postgres" ≠ "postgresql" ≠ "postgres.db". | E4 Task 2 (lookup): ensure `memory_entity_links` aliases are populated during ingestion (likely already done in F2); query includes aliases in FTS5. | low |

### F4: Preference Replay + Drift Detection

| Pattern | Source | Summary | Why Belongs | Integration Point | Cost |
|---------|--------|---------|-------------|-------------------|------|
| Reinforcement-extended half-life | Omegon `decay.rs` + M2-D1 | `halfLife = base × (factor ^ (count-1))`. Reinforced preferences persist longer; unused decay faster. | F4 drift detector checks if agent honored preferences. But stored preferences don't reflect usage. Reinforcement = preference strength signal. | F4 Task 2 (drift detector): track `rehearsal_count` on preferences; wire into decay. Frequently retrieved preferences are stronger signals. | med |
| Preference TTL + rotation (Hermes pattern) | Hermes `checkpoint_manager.py` + lifecycle hooks | Preferences have optional TTL; expired preferences auto-demote. Users can rotate preferences seasonally. | F4 replays preferences forever. Without TTL, stale preferences clutter wake (e.g., "was debugging X, prefer verbose debug replies" from 6 months ago). | F4 Task 1 (preference schema): add optional `ttl_seconds` field; F4 drift detector: filter expired preferences before comparison. | med |
| Additive preference notes (Smriti pattern) | Smriti `metadata_.notes` + C4 immutable pattern | When user corrects/updates a preference, store as append-only note, never mutate the preference record itself. | F4 user says "actually prefer terse now". Should this replace old preference (risk losing audit trail) or append as note? Additive = safer. | F4 Task 1 (schema): add `notes: Vec<PreferenceNote>` to preference records; F4 implementation: write note, don't mutate preference. | low |

### G4: Session-Continuity Proof Harness

| Pattern | Source | Summary | Why Belongs | Integration Point | Cost |
|---------|--------|---------|-------------|-------------------|------|
| Fault-injection test variants (Omegon) | Omegon test pattern | Harness includes negative controls: skip A4 restore → expect failure; inject B4 silent swallow → expect detection. Proves harness is honest. | G4 harness must prove V4 works. Without negative controls, harness passing is not evidence of correctness (might be lax assertions). | G4 Task 3 (harness implementation): add fault-injection module; 3 test variants (skip A4, break B4, drop C4 provenance). | low |
| Reproducible dogfood scenario versioning | memd own (implicit) | 3-session script committed to repo, tagged with V4 version. Future V5+ harness can replay same scenario on new code. | G4 scenario is ephemeral right now (lives in V4-INTEGRATION). If V5 changes memory model, G4's proof becomes unverifiable. | G4 Task 1 (dogfood script): commit as `fixtures/harness/v4-proof-scenario.jsonl` with version header; G4 harness loads from file. | low |
| Scorecard delta history (caveman pattern) | caveman `benchmarks/run.py` + INSPIRATION-MATRIX | Each benchmark run appends delta entry (timestamp, score delta, run_id). Prior scorecards reconstructible. | G4 regenerates 10-STAR composite. Without delta history, next run doesn't know if score change is regression or continuation. | G4 Task 4 (scorecard regeneration): append delta entry to `docs/verification/MEMD-10-STAR.md` with timestamp + run_id + prior score. | low |

---

## 3. Hermes-Specific Patterns Applicable to V4

| Pattern | Hermes File | V4 Alignment | Category | Where It Lands |
|---------|-------------|--------------|----------|-----------------|
| Procedural memory as first-class (teardown §2) | `agent/memory_manager.py` | Hermes treats skills as learnable procedures. memd doesn't yet. | Future (V5+) | M2-evo (overnight memory work) — not V4 scope |
| Lifecycle hooks (teardown §3) | `cron/__init__.py` + `tools/checkpoint_manager.py` | Hermes: pre-turn prefetch, post-turn sync, session-end extraction. memd hooks are write-path-focused. | Procedural/operational | B4 (hook contract can include procedural hooks in future; V4 = write-path only) |
| Checkpoint + rollback thinking (teardown §4) | `tools/checkpoint_manager.py` | Automatic snapshots before mutation; rollback capability. memd doesn't yet rollback. | Correction/recovery | C4 (immutable corrections + audit trail is precursor; explicit rollback = V5+ scope) |
| Cross-platform continuity (teardown §5) | docs + integration | One substrate supports terminal, messaging, IDE, background agents. memd is multi-harness; Hermes proves it works. | Operational | N2 (integrations polish — beyond V4) |
| **Hermes verdict:** No direct V4 applicability. Hermes is agent-runtime-focused; memd is memory-substrate-focused. Hermes' strongest lesson (procedural memory is mandatory) lands in V5+ scope, not V4. ✗ |

---

## 4. Cross-Phase Patterns V4 Misses (Structural)

| Pattern | Source | Spans | Why Critical | Recommendation |
|---------|--------|-------|--------------|-----------------|
| Lamport versioning on all MemoryItems | Omegon D2-D1 + L2-D4 | A4 (ledger), C4 (corrections), E4 (lookup conflict resolution), L2 (hive sync) | Without version, concurrent writes in multi-session/multi-harness create silent clobbers. Deterministic tie-breaker. | **Plumb into A4 Task 3:** add `version: u64` to `MemoryItem` schema; increment on every store; document conflict resolution rule in D4 (higher version wins on lookup dedup). Cost: low. |
| Content-hash dedup pipeline | Omegon B2-D3 + F2-D3 | B2 (signal vs noise, v2), D4 (compiler), G4 (harness dedup) | Every write should check `sha256(normalize(content))` — same hash = reinforce (increment count, bump timestamp) not insert. Prevents checkpoint noise. | **Implement in D4 Task 1:** compiler output deduped by content_hash. Reuse Omegon's normalize + hash functions. Cost: low. |
| Query sanitization (4-step) | mempalace A2-09 + H2-D1 | C4 (correction detector query), D4 (compiler internal queries), E4 (user lookup --query) | Contaminated queries poison retrieval; sanitization extracts clean intent. mempalace proves it works. | **Plumb into E4 Task 1:** add `query_sanitize()` function (reuse mempalace's algorithm); call on every user query before retrieval. Cost: low. |
| Sequence-based session isolation | Smriti C2-D2 + L2-D1 | A4 (restore boundary), E4 (lookup scoping), F4 (preference scope) | Without sequence numbers, "what was the state at session boundary?" is ambiguous (timestamps can be close). Sequence is deterministic. | **Plumb into A4 Task 6 (handoff contract):** document `session_sequence: u64` as mandatory on every memory write. G4 harness verifies sequence boundaries. Cost: low. |
| Confidence scoring on every record | Omegon + mempalace + C4 | A4 (ledger confidence), C4 (correction confidence), D4 (priority scoring), E4 (lookup ranking) | Confidence is the currency of prioritization. Without it, status items and facts are indistinguishable. | **Implement in C4 Task 3:** ensure `confidence: f32` is mandatory on all MemoryItems; wire into D4 priority rules and E4 ranking. Cost: low (field exists, need to populate correctly). |

---

## 5. Top-5 Priority Adds (Ranked by Axis-Lift × Translation Ease)

| Rank | Pattern | Source | Phases | Lift Potential | Translation | Recommendation |
|------|---------|--------|--------|----------------|-------------|-----------------|
| **1** | Lamport versioning on MemoryItem | Omegon L2-D4 | A4→L2 | Session-continuity +2, cross-harness +1 | low (1 field + 1 comparison rule) | **MUST-HAVE:** Block E4/L2 without this. Impacts lookup dedup, hive sync, conflict resolution. |
| **2** | Content-hash dedup in D4 compiler | Omegon B2-D3, F2-D3 | D4 | Token-efficiency +2 (reduce wake bloat) | low (reuse Omegon code) | **SHOULD-HAVE:** Prevents duplicate facts in wake. Improves D4 token budget compliance. |
| **3** | FTS5 full-text search for lookup | Omegon H2-D3 | E4 | Raw-retrieval +2 (keyword fallback) | med (schema change, trigger setup) | **SHOULD-HAVE:** Handles keyword queries that semantic search misses ("find X where name contains…"). Required for E4 lookup depth. |
| **4** | RRF hybrid-search merge | Omegon H2-D2 | E4 | Token-efficiency +1 (better ranking) | low (replace scoring formula) | **NICE-TO-HAVE:** Simpler than memd's fixed weights. Calibration-free. Can add in E4 or defer to V5. |
| **5** | Query sanitization (4-step) | mempalace A2-09 | C4, E4 | Correction-retention +1 (cleaner captures), raw-retrieval +1 (cleaner lookups) | low (reuse mempalace algorithm) | **NICE-TO-HAVE:** Prevents prompt injection in corrections and lookups. Safe to defer to post-V4 hardening. |

---

## Donor Coverage Summary

### Well-Mined (Patterns Cited in V4)
- **mempalace:** retrieval pipeline (A2-09), layers (A2-09), repair (A2-06), temporal validity (A2-13)
- **Omegon:** decay profiles (donor extraction doc), startup/status (omegon-smriti teardown §2), runtime DB split (omegon-smriti teardown §3)
- **Smriti:** claims, freshness, divergence, isolation (omegon-smriti teardown §6, L2 phases)
- **supermemory:** profile split (A2-11), adapter pattern (N2), graph (I2)

### Under-Mined (Patterns Missing from V4 but Applicable)
- **Omegon:** Lamport versioning (not cited — **critical gap**), content-hash dedup (cited in V2 extraction doc but not wired into V4 phases), FTS5 (cited in V2 extraction doc but not in V4 E4), RRF merge (cited but not prioritized)
- **mempalace:** Query sanitization (cited in A2-09 but not connected to C4/E4 user paths), 4-layer context cap (cited in A2-09 but not in D4 priority rules)
- **Hermes:** Lifecycle hooks beyond write-path (not in V4 scope, but should be noted as V5+ work), procedural memory (not in V4 scope)

### Rejection Justification
- **Hermes:** Agent-runtime-focused; memd is memory-substrate-focused. Procedural learning is V5+ scope. Cross-platform continuity is N2 (integrations), not V4.
- **supermemory:** Profile split is reference-only (A2-11); ontology flattening is rejected (supermemory-teardown §6).

---

## V4 Action Items

### Mandatory (Block Execution)
1. **Plumb Lamport versioning before E4:** Add `version: u64` to MemoryItem schema in A4 implementation. Document conflict resolution rule. Cost: low.

### High Priority (Fold Into Phase Deliverables)
2. **Content-hash dedup in D4 Task 1:** Compiler output deduped by hash. Reuse Omegon's normalize+hash.
3. **Query sanitization in E4 Task 1:** Add mempalace's 4-step pipeline to lookup --query path.
4. **Sequence isolation in A4 Task 6:** Document `session_sequence: u64` mandate in handoff contract.

### Medium Priority (Can Defer to Post-V4 Hardening)
5. **FTS5 virtual table in E4 Task 2:** Setup trigger-synced FTS5 on memory_items. Can ship as experimental flag.
6. **RRF merge in E4 Task 2:** Swap fixed weights for RRF. Can tune in sidecar without core DB change.

### Low Priority (V5+ Scope)
7. **Hermes lifecycle hooks:** Procedural memory, cron automation → M2-evo (V5+).
8. **Preference TTL + rotation:** F4 can ship without; add in post-V4 hardening pass.

---

## Hermes Final Verdict

Hermes has **no V4-critical patterns**. Its strongest contribution (procedural memory is mandatory, always-on loops are real infrastructure) is orthogonal to V4's session-continuity + correction-retention axes. Hermes' operational insights land in **M2-evo (overnight evolution) in V5+**, not V4.

Quote from hermes-theory-teardown: *"Hermes is strongest as: procedural-memory inspiration, always-on loop inspiration, runtime-hook inspiration. memd should steal those strengths and combine them with raw-first truth retention, typed memory kinds, canonical memory, memory atlas, human-owned multiharness continuity."*

V4 focuses on the latter (truth retention, typing, canonical, atlas, multiharness). Hermes' procedural + always-on can wait for V5.

---

**Evidence Files:**
- `/home/josue/Documents/projects/memd/docs/theory/teardowns/2026-04-11-hermes-theory-teardown.md`
- `/home/josue/Documents/projects/memd/docs/theory/donors/2026-04-14-donor-extraction-to-v2-phases.md`
- `/home/josue/Documents/projects/memd/docs/phases/v4/phase-{a4..g4}-*.md` (all 7 + integration)
- `/home/josue/Documents/projects/memd/.memd/lanes/architecture/A2-{06,07,09,11,13}.md`
