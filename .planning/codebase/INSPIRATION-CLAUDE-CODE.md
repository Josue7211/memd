# Claude Code Source Extraction

Date: 2026-04-09

Source build path:
- `/home/josue/Documents/projects/claude-code-source-build`

## Thesis

This source build is not just an agent-team product. It is a full terminal-first assistant runtime with:

- build-time source-map reconstruction
- Ink/React terminal UI
- session persistence and resume
- memory compaction and auto-consolidation
- IDE integration
- worktree orchestration
- bridge / remote-control session management
- tool, command, and skill registries
- plugins, MCP, and remote integrations
- analytics, feature flags, and packaging controls
- task orchestration across local, remote, teammate, and background flows

For `memd`, the useful lesson is the composition model:

- one runtime
- many capability surfaces
- explicit gates
- compact visible state
- recoverable sessions
- background consolidation
- inspectable coordination

## What The Source Build Teaches

### 1. Build and packaging are first-class

Relevant surfaces:

- `README.md`
- `package.json`
- `scripts/build-cli.mjs`
- `source/cli.js.map`
- `source/native-addons/`
- `source/runtime-vendor/`

Takeaway for `memd`:

- treat the runtime bundle as a product artifact, not just a codebase
- keep a reproducible build path
- make platform-specific assets explicit
- separate source reconstruction from runtime behavior

### 2. Session memory is a real subsystem

Relevant surfaces:

- `source/src/services/SessionMemory/sessionMemory.ts`
- `source/src/services/SessionMemory/sessionMemoryUtils.ts`
- `source/src/services/compact/autoCompact.ts`
- `source/src/services/compact/sessionMemoryCompact.ts`
- `source/src/screens/ResumeConversation.tsx`
- `source/src/history.ts`
- `source/src/context.ts`

Takeaway for `memd`:

- memory is not a blob, it is a live maintenance loop
- compaction is a policy, not a one-time export
- resume should rebuild useful context automatically
- session memory needs thresholds, freshness, and background updates

### 3. Auto-consolidation matters

Relevant surfaces:

- `source/src/services/autoDream/autoDream.ts`
- `source/src/services/autoDream/consolidationPrompt.ts`
- `source/src/services/autoDream/consolidationLock.ts`

Takeaway for `memd`:

- recurring background consolidation is a strong pattern
- gate order matters: cheap checks first, expensive checks later
- lock / throttle / scan gating prevents repeated churn
- the system should improve memory without interrupting the main turn

### 4. IDE integration is part of the memory model

Relevant surfaces:

- `source/src/hooks/useIDEIntegration.tsx`
- `source/src/hooks/useIdeConnectionStatus.tsx`
- `source/src/hooks/useIdeSelection.tsx`
- `source/src/hooks/useIdeLogging.ts`
- `source/src/hooks/useIDEAtMentioned.ts`

Takeaway for `memd`:

- IDE state is not just a convenience feature
- editor connection status should feed the live context model
- selection / at-mention / connection awareness are useful memory signals
- auto-connect behavior should be explicit and gated

### 5. Worktrees are a coordination primitive

Relevant surfaces:

- `source/src/utils/worktree.ts`
- `source/src/hooks/useWorktree*` and worktree-related utilities
- `source/src/tools/EnterWorktreeTool/*`
- `source/src/tools/ExitWorktreeTool/*`

Takeaway for `memd`:

- worktrees are not just git plumbing
- they are an isolation boundary for parallel agent work
- session identity should include worktree context
- creating, resuming, and leaving a worktree should be explicit

### 6. Bridge / remote-control is a product surface

Relevant surfaces:

- `source/src/bridge/createSession.ts`
- `source/src/bridge/sessionRunner.ts`
- `source/src/bridge/bridgeMain.ts`
- `source/src/bridge/remoteBridgeCore.ts`
- `source/src/entrypoints/agentSdkTypes.ts`

Takeaway for `memd`:

- remote session creation is a core workflow, not an integration detail
- title, environment, branch, and source context belong in the session object
- session archive/fetch/create are lifecycle operations
- remote bridges need compatibility layers and explicit trust boundaries

### 7. The tool and command registries are capability catalogs

Relevant surfaces:

- `source/src/tools.ts`
- `source/src/commands.ts`
- `source/src/tools/*`
- `source/src/commands/*`
- `source/src/skills/bundled/*`

Takeaway for `memd`:

- capabilities should be enumerable and filterable
- feature flags should control tool exposure cleanly
- command/tool registries are useful as memory objects themselves
- explicit catalogs make it easier to reason about what the runtime can do

### 8. Tasks are broader than “agent teams”

Relevant surfaces:

- `source/src/tasks/types.ts`
- `source/src/tasks/*`
- `source/src/tasks/pillLabel.ts`

Task classes visible in the build include:

- local shell tasks
- local agent tasks
- remote agent tasks
- in-process teammate tasks
- workflow tasks
- monitor tasks
- dream tasks

Takeaway for `memd`:

- agent teams are only one slice of the product
- task orchestration is broader: local, remote, teammate, and maintenance loops
- the UI should surface task type, status, and background activity cleanly

### 9. Analytics and feature flags are used as product controls

Relevant surfaces:

- `source/src/services/analytics/*`
- `source/src/services/feature flags`
- `source/src/constants/betas.ts`
- `source/src/constants/product.ts`
- `source/src/utils/worktreeModeEnabled.ts`

Takeaway for `memd`:

- feature flags are part of the architecture, not a side concern
- product behavior is gated at build and runtime
- analytics can be used to steer memory/coordination experiments
- `memd` should keep its own feature switches visible and inspectable

### 10. Native and platform-specific capabilities are integrated, not hidden

Relevant surfaces:

- `source/native-addons/*`
- `source/src/services/voice/*`
- `source/src/services/mcp/*`
- `source/src/services/plugins/*`
- `source/src/services/computer-use*`

Takeaway for `memd`:

- voice, image, and computer-use are not separate products
- they are capability surfaces attached to the same assistant runtime
- the product can scale across platforms if the core abstractions stay clean

## Highest-Value Borrowings For `memd`

### Tier 1: Must borrow

1. Session memory as an active maintenance loop
- automatic extraction
- background consolidation
- freshness / threshold gates
- explicit resume/recovery flow

2. Worktree-aware session isolation
- worktree as a first-class session boundary
- parallel work without collapsing state
- explicit enter / exit behavior

3. Capability catalogs
- enumerate tools, commands, and skills
- make capabilities discoverable and filterable
- treat the catalog as a memory object

4. Bridge / remote session lifecycle
- create
- fetch
- archive
- resume
- trust / permission boundaries

### Tier 2: Strong borrow

5. IDE integration
- auto-connect
- connection status
- selection-aware context
- editor-driven memory cues

6. Background tasks dashboard
- task type
- running / pending / completed
- teammate / remote / maintenance distinctions

7. Auto-compaction and prompt discipline
- thresholds
- turn-scoped caches
- compact visible state
- preserve provenance while shortening output

8. Analytics / feature-flag control plane
- gated experiments
- runtime toggles
- product measurement hooks

### Tier 3: Optional borrow

9. Voice / audio / computer-use
- useful if `memd` expands into richer operator surfaces

10. Plugin packaging
- useful if `memd` becomes a multi-harness product

## What To Avoid Copying Blindly

- source-map reconstruction complexity unless `memd` truly needs it
- startup side effects that are hard to reason about
- giant feature-flag sprawl without clear ownership
- hidden state mutations that are not inspectable
- coupling the core product too tightly to terminal UI details
- platform-native dependencies unless the capability payoff is real

## Memd Opportunity Shortlist

1. Make memory maintenance automatic
- keep `memd` from being a passive store
- add scheduled or turn-triggered consolidation

2. Treat session and worktree as one coordinate system
- a session should know where it lives
- a worktree should know which session owns it

3. Expose capability catalogs
- tool/command/skill inventory
- visible, searchable, filterable

4. Add explicit lifecycle operations
- create, resume, archive, rebind, retire

5. Add editor-aware context signals
- IDE connection and selection become memory inputs

6. Build a visible task taxonomy
- local, remote, teammate, and maintenance tasks should not all look the same

7. Keep performance and gating visible
- freshness, confidence, and thresholds should be part of the product story

## Bottom Line

The Claude Code source build shows a product architecture that combines:

- compact terminal UX
- memory maintenance
- session recovery
- coordination primitives
- capability catalogs
- feature gating
- workspace isolation
- editor integration

That is the real extraction target for `memd`.

Agent teams are one feature. The useful pattern is the whole runtime.
