# MemoryKind Taxonomy — Type Routing Contract

> Normative taxonomy for the twelve MemoryKind types used by memd's router.
> Cited by lookup `--explain-route` and F5 typed-retrieval benchmark.

Owners: `phase-f5-plan.md`, `phase-g5-plan.md`.

Status: **active** — F5 defines the router contract; G5 extends taxonomy
to include cross-harness scoring.

---

## 1. Kinds at a glance

| Kind | Domain | Example | Router hint |
|------|--------|---------|------------|
| **Fact** | Assertable ground truth | "The API returns 200 on success" | data, result, observation |
| **Decision** | Commitments + choices | "We chose PostgreSQL over MongoDB" | decided, selected, agreed |
| **Preference** | Stylistic + subjective choices | "Use snake_case for Rust variables" | prefer, convention, style |
| **Runbook** | Procedural step sequences | "To deploy: 1) build 2) test 3) ship" | steps, process, how-to |
| **Procedural** | Code patterns + idioms | "Wrap JSON with serde_json::Value" | pattern, idiom, template |
| **SelfModel** | Agent self-knowledge | "I am a Rust specialist with 5y exp" | identity, capability, role |
| **Topology** | System architecture + maps | "Auth service at 10.0.1.50:8080" | architecture, layout, service |
| **Status** | Current state snapshots | "Service is healthy; 3 alerts firing" | current, now, state |
| **LiveTruth** | Real-time runtime state | "Memory budget: 1200 chars used / 2000 available" | live, current, now |
| **Pattern** | Recurring themes + cycles | "This user always asks about deployment on Fridays" | trend, recurring, cycle |
| **Constraint** | Guardrails + limits | "Functions must return in <100ms" | must, cannot, limit, gate |
| **Correction** | Fact amendments + refutations | "Previous: 'API returns 404'; Actual: returns 200" | correction, amended, fixed |

---

## 2. Routing heuristics

The router uses query shape to infer probable MemoryKind(s). Per-kind scoring
feeds the confusion matrix in F5.

### Query shape → kind mapping

#### Assertional queries (Fact)
- Phrases: "what is", "did", "occurred", "observed", "returns", "equals"
- Example: "What does the API return on failure?"
- Expected kind: **Fact**

#### Volitional queries (Decision)
- Phrases: "decided", "chose", "why did we", "what did we pick", "agreed on"
- Example: "Why did we choose PostgreSQL?"
- Expected kind: **Decision**

#### Stylistic queries (Preference)
- Phrases: "convention", "style", "prefer", "our way", "how do we write"
- Example: "How do we write error messages?"
- Expected kind: **Preference**

#### Process queries (Runbook)
- Phrases: "steps", "how to", "process", "sequence", "checklist", "first", "then"
- Example: "What are the steps to deploy?"
- Expected kind: **Runbook**

#### Idiom queries (Procedural)
- Phrases: "pattern", "idiom", "implement", "code", "library", "function"
- Example: "How do we implement JSON parsing?"
- Expected kind: **Procedural**

#### Identity queries (SelfModel)
- Phrases: "who are you", "what can you", "your role", "expertise", "background"
- Example: "What is your experience with Rust?"
- Expected kind: **SelfModel**

#### Architecture queries (Topology)
- Phrases: "where is", "architecture", "service", "IP", "port", "endpoint", "infrastructure"
- Example: "Where is the auth service deployed?"
- Expected kind: **Topology**

#### Current-state queries (Status)
- Phrases: "right now", "currently", "status", "health", "up", "down", "alert"
- Example: "Is the service up right now?"
- Expected kind: **Status**

#### Live budget queries (LiveTruth)
- Phrases: "memory", "budget", "remaining", "available", "used", "quota"
- Example: "How much memory is left in the budget?"
- Expected kind: **LiveTruth**

#### Trend queries (Pattern)
- Phrases: "always", "usually", "tends to", "pattern", "recurring", "frequency"
- Example: "When does this user usually ask questions?"
- Expected kind: **Pattern**

#### Boundary queries (Constraint)
- Phrases: "must", "cannot", "limit", "max", "min", "rule", "constraint", "guarantee"
- Example: "What is the maximum response time?"
- Expected kind: **Constraint**

#### Correction queries (Correction)
- Phrases: "actually", "not", "wrong", "fixed", "amended", "mistake", "was wrong"
- Example: "Actually, the API returns 200, not 404."
- Expected kind: **Correction**

---

## 3. Confusion matrix boundaries

F5 validates:
- **correct_type_rate@1**: ≥ 0.85 (top result must match expected kind 85% of time)
- **per_kind_min_rate**: ≥ 0.75 (every kind individually ≥75%)
- **wrong_type_ratio**: ≤ 0.05 (hallucinated wrong kinds ≤5%)

---

## 4. Reference implementations

### Scorer (F5.3)
```rust
impl CorrectTypeScorer {
    pub fn score_result(&self, expected: &str, actual: &str) -> f64 {
        if expected == actual { 1.0 } else { 0.0 }
    }
}
```

### Router (future: lookup --explain-route)
```rust
fn explain_route(query: &str) -> Vec<MemoryKind> {
    // Tokenize query, match against heuristics above.
    // Return sorted by confidence: [Decision, Fact] (if ambiguous).
}
```

---

## 5. Exit criteria for F5

1. Taxonomy card complete + linked in phase doc.
2. All 12 kinds with descriptions + routing hints.
3. Heuristic mappings testable (Test 9 `taxonomy_card_round_trip`).
4. Used as ground truth by scorer + runner fixtures.
