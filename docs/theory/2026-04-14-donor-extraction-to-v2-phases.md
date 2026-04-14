# Donor Extraction → V2 Phase Mapping

truth_date: 2026-04-14
status: active
purpose: Map every steal-worthy pattern from donor repos to the V2 phase where it lands.

Each entry: what to steal, from whom, how it maps to memd Rust, which phase owns it.

---

## B2 — Signal vs Noise

### B2-D1: Priority-based retrieval dedup (supermemory)

**Source**: `supermemory/packages/tools/src/tools-shared.ts`

**Pattern**: At retrieval time, dedup across memory categories using priority ordering:
static (permanent) > dynamic (recent) > search results. Exact string matching.

**Rust adaptation**: In working memory assembly (`working/mod.rs`), after scoring,
partition results into canonical facts, working context, and search results. Dedup
by content hash across partitions. Canonical always wins.

```rust
// pseudocode for wake packet assembly
let canonical = items.iter().filter(|i| i.stage == Canonical && i.kind != Status);
let working = items.iter().filter(|i| i.stage != Canonical);
let mut seen = HashSet::new();
for item in canonical { seen.insert(content_hash(&item.content)); emit(item); }
for item in working { if seen.insert(content_hash(&item.content)) { emit(item); } }
```

### B2-D2: Status item cap at working memory admission (mempalace layers.py)

**Source**: `mempalace/layers.py` — L1 essential story caps at 15 drawers, 3200 chars

**Pattern**: Hard cap on how many items of each type appear in wake context.
mempalace caps total items; memd should cap Status specifically.

**Rust adaptation**: Already partially implemented (status cap = 2 in `working/mod.rs`).
Extend: cap Status at 2, cap total at 8, enforce character budget per item (220 chars).

### B2-D3: Redundancy key dedup on checkpoint writes (mempalace + Omegon)

**Source**: `mempalace/dedup.py` (0.15 cosine threshold), `omegon-memory/src/sqlite.rs` (content_hash dedup)

**Pattern**: Both systems dedup at write time. mempalace uses cosine similarity within
groups. Omegon uses SHA256 content hash — if hash matches, reinforce instead of insert.

**Rust adaptation — STEAL FROM OMEGON DIRECTLY**:
```rust
// omegon-memory/src/sqlite.rs store_fact() pattern:
// 1. Compute content_hash = SHA256(normalize(content))[0..16]
// 2. Check: SELECT id FROM facts WHERE mind=? AND content_hash=?
// 3. If exists: reinforce (increment count, update timestamp, bump version)
// 4. If new: INSERT
```
memd already has `redundancy_key` field — wire it into `checkpoint_with_bundle_defaults()`.
On checkpoint: compute key, check existing, update if exists instead of insert.

### B2-D4: Decay-based confidence erosion (Omegon)

**Source**: `omegon-memory/src/decay.rs`

**Pattern**: Exponential decay with reinforcement-extended half-life:
```
confidence = e^(-ln(2) × days_since / halfLife)
halfLife = base × (reinforcement_factor ^ (count-1)), capped at 90 days

Profiles:
  Standard: base=14d, factor=1.8, max=90d
  Global: base=30d, factor=2.5, max=90d
  RecentWork: base=2d, factor=1.0 (no extension)
```

**Rust adaptation — STEAL DIRECTLY**: memd has decay (21d/0.12 hardcoded) but no
reinforcement extension. Port Omegon's `DecayProfile` struct and `compute_confidence()`
function. Map memd kinds to profiles:
- Status → RecentWork (base=2d, no reinforcement)
- Fact/Decision → Standard (base=14d, reinforcement extends)
- Global scope → Global (base=30d, longer persistence)

---

## C2 — Ghost Cleanup

### C2-D1: Lifecycle-driven expiration (Omegon)

**Source**: `omegon-memory/src/sqlite.rs` — `apply_lifecycle()`

**Pattern**: TTL expiration applied as lifecycle transition, not separate GC pass.
On any fact access, check: `created_at + ttl_seconds <= now → status = Expired`.

**Rust adaptation**: memd already has `MemoryStatus::Expired` and `ttl_seconds`.
Wire lifecycle check into retrieval path: before returning items from working memory,
filter expired. Run batch expiration in worker cycle (already partially there via drain).

### C2-D2: Session-scoped sequence isolation (Smriti)

**Source**: `smriti/backend/app/db/models.py` — `TurnEvent.sequence_number`

**Pattern**: Monotonically increasing sequence number per session. When mounting a
checkpoint, capture `history_base_seq`. Filter: `sequence_number > base_seq`.
Prevents pre-mount data from contaminating restored context.

**Rust adaptation**: memd continuity capsule should track a monotonic sequence counter.
On resume, filter working memory to items created after the resume point's sequence.
Eliminates ghost refs from prior sessions leaking into current context.

---

## D2 — Correction Flow

### D2-D1: Supersede with Lamport versioning (Omegon)

**Source**: `omegon-memory/src/sqlite.rs` — `supersede_fact()`

**Pattern**: Archive original fact (status='archived'), insert replacement with:
- `supersedes` field pointing to original ID
- `version` incremented (Lamport clock for conflict resolution)
- Original preserved for audit trail

**Rust adaptation — STEAL DIRECTLY**: memd already has `supersedes: Vec<Uuid>` and
`MemoryStatus::Superseded`. Missing: Lamport version counter for conflict resolution
on concurrent corrections. Add `version: u64` to `MemoryItem`, increment on every
mutation, use for import conflict resolution (higher version wins).

### D2-D2: Temporal fact invalidation (mempalace knowledge_graph.py)

**Source**: `mempalace/knowledge_graph.py` — `valid_from`/`valid_to` on triples

**Pattern**: Old fact gets `valid_to = now()`, new fact gets `valid_from = now()`.
Time-scoped query: `WHERE valid_from <= ? AND (valid_to IS NULL OR valid_to > ?)`.
Preserves history without duplicates.

**Rust adaptation**: Apply to atlas entity links (`memory_entity_links` table).
Add `valid_from`/`valid_to` columns. On correction, invalidate old link, create new.
Enables "what did we believe at time T?" queries.

### D2-D3: Write-ahead log for correction audit (mempalace)

**Source**: `mempalace/wal/write_log.jsonl`

**Pattern**: Every write (add/delete) logged BEFORE execution as JSONL:
`{"timestamp", "operation", "params", "result"}`. Enables unauthorized write detection.

**Rust adaptation**: memd already has lifecycle events on memory items. Extend to log
correction operations (who corrected what, when, why) in `memory_events` table.
Add event_type = "corrected" with payload containing old content, new content, reason.

### D2-D4: Immutable checkpoints with additive notes (Smriti)

**Source**: `smriti/backend/app/db/models.py` — `CommitModel` (never modified after INSERT)

**Pattern**: Checkpoints are immutable snapshots. Annotations stored as additive-only
`metadata_.notes[]` JSONB array. Review happens after creation, flags issues without
modifying the checkpoint itself.

**Rust adaptation**: For memd corrections, preserve the original item (mark Superseded),
create new item with correction. Never UPDATE content in place. Annotation as separate
event linked to item ID.

---

## E2 — Atlas Activation

### E2-D1: SQLite triples with temporal validity (mempalace)

**Source**: `mempalace/knowledge_graph.py`

**Schema**:
```sql
subject TEXT, predicate TEXT, object TEXT,
valid_from TEXT, valid_to TEXT,
source_id TEXT, confidence REAL
```

**Rust adaptation**: memd already has `memory_entity_links` table. Add temporal columns
(`valid_from`, `valid_to`). Populate during ingestion as pre-graph pass. Query with
temporal filtering for atlas navigation.

### E2-D2: Knowledge graph edges with reinforcement (Omegon)

**Source**: `omegon-memory/src/sqlite.rs` — `edges` table

**Schema (Omegon)**:
```sql
CREATE TABLE edges (
    id TEXT PRIMARY KEY,
    source_fact_id TEXT, target_fact_id TEXT,
    relation TEXT, description TEXT,
    confidence REAL, last_reinforced TEXT,
    reinforcement_count INTEGER, decay_rate REAL,
    status TEXT, created_at TEXT, created_session TEXT,
    source_mind TEXT, target_mind TEXT
);
```

**Rust adaptation — STEAL SCHEMA**: memd's `memory_entity_links` should gain:
`reinforcement_count`, `last_reinforced`, `decay_rate`. Edges strengthen on repeated
co-occurrence. Weak edges decay and eventually archive. This makes atlas navigation
quality improve with usage.

### E2-D3: D3 force-directed graph config (supermemory)

**Source**: `supermemory/packages/memory-graph/src/constants.ts`

**Config**: charge=-2000, collision doc=70px memory=35px, alpha decay=0.025,
pre-settlement=150 ticks. Edge types: derives, updates, extends.

**Rust adaptation**: Reference config for I2 dashboard graph component. Data-driven:
graph receives pre-fetched data, app handles pagination. Graph component is separate
package from core memory.

### E2-D4: Entity aliasing and auto-extraction (Omegon)

**Source**: `omegon-memory/src/sqlite.rs` — `derive_entity_key()`, alias merging

**Pattern**: Auto-extract entity aliases from: project, namespace, agent, source_system,
source_path, file_name, memory_kind. Merge aliases on entity update (union of sets).
Used in search haystacks.

**Rust adaptation**: memd already has `MemoryEntityRecord.aliases: Vec<String>`. Wire
auto-extraction: when storing a memory item, extract entity aliases from metadata fields
and merge into entity record. Improves entity search recall.

---

## F2 — Ingestion Pipeline

### F2-D1: Hash manifest for idempotent re-ingestion (mempalace)

**Source**: `mempalace/palace.py` — `file_already_mined()` + `source_mtime` metadata

**Pattern**: Store content hash + mtime per ingested file. On re-run:
hash matches → skip. Hash differs → delete old chunks, re-ingest. File deleted → mark stale.

**Rust adaptation**: Add `ingestion_manifest` table to memd DB:
```sql
CREATE TABLE ingestion_manifest (
    source_path TEXT PRIMARY KEY,
    content_hash TEXT,  -- SHA256
    mtime_epoch INTEGER,
    lane TEXT,
    last_ingested_at TEXT
);
```

### F2-D2: Type-specific extraction (mempalace)

**Source**: `mempalace/convo_miner.py`, `general_extractor.py`

**Pattern**: Content classified into types at ingest time:
- `fact`: declarative state ("X uses Y")
- `decision`: choice with rationale ("chose X because Y")
- `procedure`: how-to with steps ("to deploy, run X then Y")
- `status`: current state, expected to change (gets TTL)

Only `status` gets TTL. Facts/decisions/procedures are durable.

**Rust adaptation**: Map to memd's `MemoryKind` enum (already has Fact, Decision,
Procedural, Status). Ingestion pipeline should classify incoming content into kinds
and assign appropriate TTL (Status → 24h, others → None).

### F2-D3: Content-hash dedup on ingest (Omegon)

**Source**: `omegon-memory/src/types.rs` — `content_hash()`, `normalize_for_hash()`

**STEAL DIRECTLY**:
```rust
pub fn normalize_for_hash(s: &str) -> String {
    let s = s.trim();
    let s = s.strip_prefix("- ").unwrap_or(s);
    s.to_lowercase()
     .split_whitespace()
     .collect::<Vec<_>>()
     .join(" ")
}

pub fn content_hash(content: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(normalize_for_hash(content).as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..8])  // first 8 bytes = 16 hex chars
}
```
On ingest: compute hash, check existing, reinforce if exists, insert if new.

### F2-D4: Tree-sitter AST chunking for code files (Omegon)

**Source**: `omegon-codescan/src/code.rs`, `indexer.rs`

**Pattern**: Tree-sitter parsing for Rust, TypeScript, Python, Go. Chunks at named
boundaries: functions, structs, impl blocks, modules, enums. Regex fallback for
parse failures. Incremental: skip if git HEAD unchanged. SHA256 change detection.

**Rust adaptation — STEAL DIRECTLY**: memd needs code-aware chunking for source ingestion.
Omegon's `omegon-codescan` crate is directly portable. Use tree-sitter for lane source
material that contains code blocks.

---

## G2 — Lane Architecture

### G2-D1: 4-priority room routing (mempalace)

**Source**: `mempalace/miner.py` — `detect_room()`

**Algorithm**: path component → filename → content keywords (first 2KB) → fallback "general".
94-entry folder map → 13 canonical rooms. Zero LLM dependency.

**Rust adaptation**: Build lane auto-activation as 4-priority routing:
1. Source path components matched against lane keyword map
2. Filename matched against lane names
3. Content keywords scored (first 2KB window)
4. Fallback to "general" lane

### G2-D2: Layered memory inheritance (Omegon minds)

**Source**: `omegon-memory/src/sqlite.rs` — `minds` table with `parent` field

**Pattern**: Minds (memory namespaces) can have parent minds. Child inherits from parent.
Layer field on facts: "project" | "persona" | "working". Query can scope to layer.

**Rust adaptation**: memd lanes already have hierarchy potential. Add parent_lane concept
so lane queries can inherit from broader scopes. Maps to `MemoryScope` precedence:
Local inherits from Project inherits from Global.

### G2-D3: Section-based fact organization (Omegon)

**Source**: `omegon-memory/src/types.rs` — `Section` enum

**Sections**: Architecture, Decisions, Constraints, KnownIssues, PatternsConventions,
Specs, RecentWork. Facts are grouped by section for context rendering.

**Rust adaptation**: Map to memd's existing `MemoryKind` + `tags`. Consider adding
a `section` field or using tags for grouping in wake packet rendering.

---

## H2 — Recall Proof

### H2-D1: Ephemeral store-per-query benchmark (mempalace)

**Source**: `mempalace/benchmarks/longmemeval_bench.py`

**Pattern**:
```
for question in dataset:
    store = create_ephemeral_store()
    ingest(store, question.corpus)
    retrieved = search(store, question.query, k=10)
    score = dcg_ndcg(retrieved, question.gold_ids)
    destroy(store)
```
Clean isolation per question. DCG@k + NDCG@k + Recall@k scoring.

**Rust adaptation**: memd already has namespace isolation. Create ephemeral namespace
per test question. Already partially implemented in `public_benchmark.rs`.

### H2-D2: RRF hybrid search (Omegon)

**Source**: `omegon-memory/src/sqlite.rs` — `rrf_merge()`

**STEAL DIRECTLY**:
```rust
pub fn rrf_merge(
    fts_results: &[ScoredFact],
    vec_results: &[ScoredFact],
    rrf_k: f64,
    limit: usize
) -> Vec<ScoredFact> {
    // Reciprocal Rank Fusion: score = Σ 1/(rrf_k + rank_i)
    let mut scores: HashMap<String, f64> = HashMap::new();
    for (rank, item) in fts_results.iter().enumerate() {
        *scores.entry(item.fact.id.clone()).or_default() += 1.0 / (rrf_k + rank as f64);
    }
    for (rank, item) in vec_results.iter().enumerate() {
        *scores.entry(item.fact.id.clone()).or_default() += 1.0 / (rrf_k + rank as f64);
    }
    // merge, sort by score desc, take limit
}
```
memd sidecar already has hybrid search but uses fixed weights. RRF is simpler and
more robust — scores from different retrieval methods don't need calibration.

### H2-D3: FTS5 full-text search with sync triggers (Omegon)

**Source**: `omegon-memory/src/sqlite.rs`

**STEAL DIRECTLY**:
```sql
CREATE VIRTUAL TABLE facts_fts USING fts5(content, section, content=facts, content_rowid=rowid);
CREATE TRIGGER facts_ai AFTER INSERT ON facts BEGIN
    INSERT INTO facts_fts(rowid, content, section) VALUES (new.rowid, new.content, new.section);
END;
-- + UPDATE and DELETE triggers
```
memd has no FTS. Adding FTS5 to `memory_items` table gives instant keyword search
without the sidecar. Sidecar provides vector search; FTS5 provides exact match.

---

## I2 — Human Dashboard

### I2-D1: Data-driven graph component (supermemory)

**Source**: `supermemory/packages/memory-graph/`

**Pattern**: React component receives pre-fetched `GraphApiDocument[]` data.
No API calls from graph. App provides `onLoadMore` callback for pagination.
Variants: "console" (embedded) and "consumer" (standalone).

**Rust adaptation**: Dashboard graph component should consume memd atlas API data.
Server provides `/memory/entity/links` data. Frontend renders with D3 force layout.

### I2-D2: Stable vs recent profile projection (supermemory)

**Source**: `supermemory/packages/tools/src/shared/types.ts` — `ProfileStructure`

**Pattern**: Split retrieved memory into `static` (permanent facts) and `dynamic`
(recent context) for display. Priority: static > dynamic > search.

**Rust adaptation**: Dashboard memory view should separate canonical facts from working
memory. Canonical facts pinned at top, working memory below, search results on demand.

### I2-D3: Compact state brief (Smriti)

**Source**: `smriti` CLI — `smriti state --compact`

**Pattern**: Omit artifact content in compact mode. Preserve labels + recovery commands.
Full artifacts retrievable separately: `smriti checkpoint show <id> --full-artifacts`.

**Rust adaptation**: `memd state` should have `--compact` flag. Show key facts, current
focus, active workspace. Full details via `memd explain <id>`.

---

## J2 — Isolation + Trust

### J2-D1: Worktree-first parallel execution (Omegon)

**Source**: `omegon-git/src/worktree.rs`

**STEAL DIRECTLY** (Rust):
```rust
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
    pub backend: String,  // "jj" or "git"
}

pub fn create_smart(repo_path: &Path, name: &str) -> Result<WorktreeInfo> {
    if is_jj_repo(repo_path) {
        create_jj_workspace(repo_path, name)
    } else {
        create_git_worktree(repo_path, name)
    }
}
```
memd hive should treat worktrees as physical isolation boundary. Memory coordination
sits on top. Prevents file-level collision entirely.

### J2-D2: Advisory claims with TTL (Smriti)

**Source**: `smriti/backend/app/db/models.py` — `WorkClaim`

**Schema**:
```python
agent: str
branch_name: str
scope: str           # one-sentence intent
task_id: Optional[str]
intent_type: str     # implement | review | investigate | docs | test
status: str          # active | done | abandoned
expires_at: datetime # 4h default TTL
```

**Rust adaptation**: memd already has `hive_claims` table. Enrich with:
- `intent_type` field (maps to Smriti's implement/review/investigate/docs/test)
- Query-time TTL filtering: `WHERE expires_at > datetime('now')`
- Non-blocking (advisory, not locks)

### J2-D3: Namespace-scoped memory isolation (Omegon minds)

**Source**: `omegon-memory/src/sqlite.rs` — `minds` table

**Pattern**: Each "mind" is an isolated memory namespace. Facts belong to exactly one
mind. Search scoped to mind. Export/import per mind. Cascade delete on mind removal.

**Rust adaptation**: memd already has project/namespace isolation. Ensure retrieval
enforces scope boundaries — currently scope/visibility fields exist but aren't checked
at retrieval time (backlog #47).

### J2-D4: Secrets redaction with Aho-Corasick DFA (Omegon)

**Source**: `omegon-secrets/src/lib.rs`

**Pattern**: All secrets registered in redaction set. Aho-Corasick DFA built from
secret values. Every tool output passed through `redact()` before display.
Path guard checks tool args for sensitive paths.

**Rust adaptation**: Not directly memd's scope (memd is memory, not agent runtime).
But if memd surfaces memory content in dashboards/APIs, redaction of sensitive content
in stored memories is relevant. Reference for N2 harness packs.

---

## K2 — Observability

### K2-D1: Compiled operator state surface (Omegon status.rs)

**Source**: `omegon/src/status.rs` — `HarnessStatus`

**Pattern**: One struct captures everything an operator needs:
git branch, active persona, installed plugins, MCP servers, memory health,
inference backends, context class, thinking level, capability tier.

**Rust adaptation**: `memd state` should compile to one observable surface:
- Memory health (item counts by kind/status/stage)
- Active session info
- Working memory composition
- Hive coordination state
- Freshness warnings
- Atlas coverage

### K2-D2: Structured upstream error classification (Omegon)

**Source**: `omegon/src/upstream_errors.rs`

**Pattern**: Every upstream failure classified into `UpstreamErrorClass` with mapped
`RecoveryAction`. Failures logged as `UpstreamFailureLogEntry` with provider, model,
attempt count, delay.

**Rust adaptation**: memd server should classify errors (DB errors, timeout, conflict)
with explicit recovery actions. Currently errors are `(StatusCode, String)` — too flat.

### K2-D3: Token tracking per request (Omegon bridge.rs)

**Source**: `omegon/src/bridge.rs` — `LlmEvent::Done { input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens }`

**Pattern**: Every LLM call returns exact token counts. Tracked per session.

**Rust adaptation**: memd should track token efficiency of wake packets and context
rendering. Measure: how many tokens does each wake packet consume? How many are
signal vs noise? Feeds into B2 optimization.

---

## L2 — Hive Hardening

### L2-D1: Freshness check with commit-based baseline (Smriti)

**Source**: `smriti/backend/app/api/routes/chat.py` — `FreshnessInfo`

**Pattern**: Client provides `since_commit_id`. Server walks commit chain from HEAD
back to that commit. Returns: `changed: bool`, `new_checkpoints_count`, list of
new checkpoints with author_agent and message.

**Rust adaptation**: memd hive sessions should support freshness queries:
"what changed since my last wake?" Returns new/updated items since a given timestamp
or sequence number. Enables agents to detect stale state before acting.

### L2-D2: Multi-branch divergence detection (Smriti)

**Source**: `smriti/backend/app/api/routes/chat.py` — `DivergenceSummary`

**Pattern**: Compare decisions between branches. Normalize text (lowercase, strip
non-alphanumeric, collapse whitespace). Diff: `main_only_decisions` vs
`branch_only_decisions`. Cap at 2 branches, 3 decisions per side.

**Rust adaptation**: When multiple hive sessions are active, detect when their
working memory contains contradictory decisions. Surface in `memd state` or
hive board as divergence signal.

### L2-D3: SQLITE_BUSY retry with backoff (Omegon)

**Source**: `omegon-memory/src/sqlite.rs` — `PRAGMA busy_timeout = 5000`

**Pattern**: SQLite WAL mode + 5-second busy timeout. Handles concurrent access
without custom retry logic.

**Rust adaptation — STEAL DIRECTLY**: memd has SQLITE_BUSY on concurrent writes
(backlog #75). Set `PRAGMA busy_timeout = 5000` in connection setup. Already using
WAL mode. Add retry wrapper for critical write paths.

### L2-D4: Lamport versioning for conflict resolution (Omegon)

**Source**: `omegon-memory/src/sqlite.rs` — `version` field on facts

**Pattern**: Every mutation increments `version: u64` (Lamport clock). On import:
`if incoming.version <= stored.version { skip }`. Higher version always wins.
Deterministic conflict resolution without timestamps.

**Rust adaptation**: Add `version: u64` to `MemoryItem`. Increment on every store/update.
On hive sync or import: compare versions, higher wins. Prevents lost updates in
multi-agent scenarios.

---

## M2-evo — Overnight Evolution

### M2-D1: Reinforcement-extended half-life (Omegon decay.rs)

**Source**: `omegon-memory/src/decay.rs`

**Pattern**: Each reinforcement (access/retrieval) extends the half-life exponentially:
`halfLife = base × (factor ^ (count-1))`. Frequently accessed facts persist longer.
Unused facts decay faster.

**Rust adaptation**: memd has rehearsal_count on entities. Wire into decay calculation:
higher rehearsal_count → slower decay. Currently decay is flat (21d/0.12). Use Omegon's
formula for calibrated decay.

### M2-D2: Episode narrative extraction (Omegon)

**Source**: `omegon-memory/src/types.rs` — `Episode` struct

**Schema**:
```rust
pub struct Episode {
    pub id: String,
    pub mind: String,
    pub title: String,
    pub narrative: String,
    pub date: String,
    pub session_id: String,
    pub created_at: String,
}
```
Plus `episode_facts` junction table linking episodes to facts, and `episodes_fts`
for full-text search on narratives.

**Rust adaptation**: memd evolution loop should consolidate session events into episodic
narratives. Store as a special memory kind or in a new episodes table. Links to the
facts that were active during that episode.

---

## N2 — Integrations Polish

### N2-D1: Thin harness adapter pattern (supermemory)

**Source**: `supermemory/packages/tools/src/` — 5 adapters

**Pattern**: 3-step for every framework:
1. Accept framework-specific config
2. Wrap shared core (validate, create client, build text)
3. Return framework-specific wrapper

Each adapter is <200 lines. Core logic centralized in `shared/`.

**Rust adaptation**: memd harness packs should follow same pattern:
- `memd-harness-codex/`: thin adapter for Codex AGENTS.md
- `memd-harness-claude/`: thin adapter for Claude Code hooks
- `memd-harness-opencode/`: thin adapter for OpenCode
- Core logic stays in `memd-client` crate.

### N2-D2: Turn-scoped LRU cache (supermemory)

**Source**: `supermemory/packages/tools/src/shared/cache.ts`

**Pattern**: LRU cache (max 100 entries) keyed by `containerTag:threadId:mode:normalizedMessage`.
Prevents duplicate API calls within same agent turn. No TTL — cleared between turns.

**Rust adaptation**: memd client should cache retrieval results within a single command
invocation. Key: `project:namespace:intent:query_hash`. Max 100 entries. Cleared on
each new CLI invocation.

### N2-D3: Skill pack as versioned instruction file (Smriti)

**Source**: `smriti/.claude/skills/smriti/SKILL.md`

**Pattern**: Versioned instruction file installed via CLI. Includes workflow heuristics,
anti-patterns ("when NOT to checkpoint"), multi-agent etiquette. Teaches agents how
to use the memory system correctly.

**Rust adaptation**: memd harness packs should include instruction files that teach
agents: when to remember, when to correct, when NOT to checkpoint, how to read wake
packets, multi-agent etiquette for hive sessions.

### N2-D4: MCP server with multiple transport modes (Omegon)

**Source**: `omegon/src/plugins/mcp.rs` — `McpServerConfig`

**Pattern**: MCP servers support 4 transport modes: local process, OCI container,
Docker gateway, Styrene mesh. Config per server: url, command, args, env, image,
mount_cwd, network, timeout.

**Rust adaptation**: memd MCP integration should support at minimum: local process
(stdio) and HTTP transport. Configuration in `.memd/config.json`.

---

## Cross-Phase: Direct Code Lifts from Omegon (Rust)

These are patterns where we can literally copy Omegon's Rust code with minimal adaptation:

| Pattern | Omegon Source | memd Target | Phase |
|---------|--------------|-------------|-------|
| Content hash dedup | `types.rs:content_hash()` | `checkpoint_with_bundle_defaults()` | B2 |
| Decay profiles | `decay.rs:DecayProfile` | `working/mod.rs` decay calc | B2, M2 |
| FTS5 with triggers | `sqlite.rs` schema | `memd-server` migrations | H2 |
| RRF merge | `sqlite.rs:rrf_merge()` | `memd-sidecar` scoring | H2 |
| Worktree create | `worktree.rs:create_smart()` | `memd-client` hive | J2 |
| Busy timeout | `sqlite.rs` PRAGMA | `memd-server` DB init | L2 |
| Lamport versioning | `sqlite.rs` version field | `MemoryItem` schema | L2 |
| Edge reinforcement | `sqlite.rs` edges table | `memory_entity_links` | E2 |
| Episode struct | `types.rs:Episode` | new episodic table | M2 |
| AST chunking | `omegon-codescan` crate | ingestion pipeline | F2 |
| Secrets redaction | `omegon-secrets` crate | harness packs | N2 |
| IPC envelope | `omegon-traits` crate | memd IPC (future) | N2 |

---

## Cross-Phase: Pattern Lifts from Smriti (Python → Rust translation)

| Pattern | Smriti Source | memd Target | Phase |
|---------|-------------|-------------|-------|
| Advisory claims | `WorkClaim` model | `hive_claims` enrichment | J2 |
| Freshness check | `FreshnessInfo` response | hive freshness API | L2 |
| Divergence detection | `DivergenceSummary` | hive board divergence | L2 |
| Sequence isolation | `TurnEvent.sequence_number` | continuity capsule | C2 |
| Immutable checkpoints | `CommitModel` design | correction audit trail | D2 |
| Compact state brief | `--compact` flag | `memd state --compact` | K2 |
| Skill pack | `SKILL.md` | harness instruction files | N2 |
| Branch disposition | `branch_disposition` | session lifecycle | L2 |

---

## Cross-Phase: Pattern Lifts from mempalace (Python → Rust translation)

| Pattern | mempalace Source | memd Target | Phase |
|---------|-----------------|-------------|-------|
| Hash manifest | `palace.py:file_already_mined()` | ingestion manifest table | F2 |
| 4-priority routing | `miner.py:detect_room()` | lane auto-activation | G2 |
| 0.15 cosine dedup | `dedup.py` | storage-time dedup | B2 |
| 4-layer context | `layers.py` | wake packet layers | B2 |
| Query sanitization | `query_sanitizer.py` | search input cleaning | H2 |
| Three-stage repair | `repair.py` | `memd repair` command | D2 |
| Type-specific extraction | `general_extractor.py` | ingestion kind assignment | F2 |
| Temporal validity | `knowledge_graph.py` | atlas edges | E2 |

---

## Cross-Phase: Pattern Lifts from supermemory (TypeScript → Rust translation)

| Pattern | supermemory Source | memd Target | Phase |
|---------|-------------------|-------------|-------|
| Priority dedup | `tools-shared.ts` | wake packet assembly | B2 |
| Adapter pattern | `packages/tools/` | harness packs | N2 |
| Turn cache | `shared/cache.ts` | client retrieval cache | N2 |
| Graph component | `packages/memory-graph/` | dashboard atlas | I2 |
| Profile split | `shared/types.ts` | wake packet rendering | B2, I2 |
| Version chains | `types.ts` | correction history | D2 |
| Data-driven graph | `memory-graph/` | dashboard architecture | I2 |
