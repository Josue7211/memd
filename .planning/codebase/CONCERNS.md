# Codebase Concerns

## Primary Product Concern

`memd` is supposed to be a memory system / memory OS for agents, but the current
codebase does not yet prove the closed loop that matters most:

1. ingest memory
2. persist it durably
3. retrieve it in the hot path for the current task
4. let that retrieval override stale assumptions
5. visibly change later behavior

The repository has substantial machinery for storage, resume rendering,
evaluation, research loops, coordination, and planning, but the observed product
failure is that important memories are still easy to miss in practice.

## Concern 1: Storage And Recall Are Too Loosely Coupled

- `README.md` presents a strong product promise around `memd resume`,
  `memd remember`, and durable cross-session memory, but the recall guarantee is
  weaker than the surface suggests.
- The main retrieval hot path is assembled in
  `crates/memd-server/src/main.rs` via `build_context()`.
- `build_context()` does prepend `MemoryKind::LiveTruth`, but after that it
  mostly appends other active records by score; it does not clearly enforce a
  robust “this newer memory suppresses the stale one” behavior for general
  memory recall.
- `crates/memd-server/src/working.rs` depends on `build_context()` and then
  compacts what it receives; if retrieval is incomplete, compaction just makes
  incomplete recall more efficient.

Why this matters:

- If the retrieval set is wrong, the memory OS fails even if storage and
  compaction look healthy.

## Concern 2: Live Truth Is Narrower Than The Product Need

- The concrete live-truth ingest path currently visible in
  `crates/memd-client/src/main.rs` is `sync_recent_repo_live_truth()`.
- That path summarizes repo changes from git state and stores a compact record.
- This is useful for edit-awareness, but it is not the same thing as general
  memory durability.
- The codebase does not show an equally direct, automatic, first-class ingest
  path for “the user corrected the system” or “this conversational memory must
  be recalled later.”

Why this matters:

- The current live-truth implementation helps with repo freshness more than it
  helps with durable behavioral memory.
- That leaves a gap between “fresh local repo state” and “the agent actually
  remembers what the user told it.”

## Concern 3: Planning And Evaluation Are Ahead Of Core Product Proof

- `.planning/STATE.md` and `.planning/ROADMAP.md` assert many capabilities as
  complete or advanced.
- The repo includes extensive work for scenario harnesses, composite scoring,
  bounded experiment loops, research telemetry, and capability-roadmap layers.
- However, the core memory product bar still appears under-verified: “remember
  an important thing and use it later when it matters.”

Why this matters:

- The codebase risks measuring the sophistication of memory infrastructure
  before proving the reliability of memory recall.
- This creates planning/status drift: docs can honestly describe implemented
  surfaces while still overstating practical memory quality.

## Concern 4: The Main Client Entrypoint Is Carrying Too Much System Behavior

- `crates/memd-client/src/main.rs` is very large and contains CLI dispatch,
  resume orchestration, loop logic, repo-diff ingestion, bundle state syncing,
  coordination summaries, and multiple feature slices.
- That makes the memory failure hard to localize because ingest, retrieval
  requests, prompt shaping, and side effects are not isolated cleanly enough.

Why this matters:

- When a product guarantee fails, debugging should quickly answer:
  - Was the memory never stored?
  - Was it stored but not retrieved?
  - Was it retrieved but not rendered?
  - Was it rendered but ignored by the agent integration?
- The current client layout makes that harder than it should be.

## Concern 5: Tests Are More Aligned To Subsystems Than To The Product Contract

- The repository has healthy Rust test coverage and currently passes package
  tests, but passing tests did not prevent the practical memory failure.
- Existing tests appear stronger around individual behaviors, scenario
  scaffolds, and artifact generation than around the end-to-end contract of
  memory recall.
- A key missing bar is a product-level failing test for:
  - store a memory
  - start a later resume/retrieval flow
  - confirm the returned context contains that memory
  - confirm stale conflicting memory is demoted or suppressed

Why this matters:

- Without product-contract tests, the codebase can stay green while the memory
  OS still feels broken to users.

## Concern 6: Memory Surfaces May Be Inert Unless Explicitly Invoked

- The project strongly depends on explicit surfaces such as `memd resume`,
  bundle files like `.memd/MEMD_MEMORY.md`, generated agent launchers, and
  harness-specific integration steps.
- If the active agent loop does not automatically call the right `memd` surface,
  stored memory becomes passive state on disk instead of active recall.

Why this matters:

- A memory OS that only works when the operator manually performs the right
  sequence is still too fragile for the product claim.

## Concern 7: Correction Learning Is Still Framed As Future-Oriented

- The planning docs explicitly say user corrections still need to become learned
- operating policy.
- That wording is a direct warning that the current system still permits the
  next response to fall back to stale assumptions.

Relevant files:

- `.planning/STATE.md`
- `docs/live-truth.md`
- `docs/superpowers/plans/2026-04-06-ceiling-memd-live-truth.md`

Why this matters:

- This is not just a bug; it is already acknowledged in the project’s own
  planning language as unfinished product behavior.

## Highest-Risk Debugging Targets

1. `crates/memd-client/src/main.rs`
   - verify which commands and flows actually store durable memory
   - isolate whether conversational corrections ever enter storage
2. `crates/memd-server/src/main.rs`
   - inspect `build_context()` retrieval and suppression behavior
3. `crates/memd-server/src/working.rs`
   - confirm working-memory compaction is not hiding bad retrieval selection
4. `crates/memd-client/src/render.rs`
   - verify whether important recalled memory is rendered prominently enough to
     affect downstream agent behavior
5. `README.md` and `.planning/STATE.md`
   - reduce mismatch between documented promise and demonstrated reliability

## Recommended Next Move

Do not start with more roadmap expansion.

Start with a deep product debugging pass that proves or falsifies the core loop:

1. create one durable memory in a controlled test fixture
2. run the exact later resume/retrieval path
3. inspect whether that memory is returned
4. add a conflicting stale memory
5. verify suppression / precedence
6. verify the rendered prompt surface exposes the winning memory first

Until that passes reliably, `memd` should be treated as a partially functioning
memory substrate rather than a dependable memory OS.
