# The Context Pivot: A Research-Backed Roadmap for SlopChop

**Status:** Draft / Strategic
**Date:** 2026-01-21
**Objective:** Transition SlopChop from a *Verification Engine* to a *Context Engine*.

---

## 1. The Core Insight
Current AI coding assistants (like Aider or Cursor) optimize for **Recall** (finding everything potentially relevant) at the expense of **Precision** (token cost). They often dump full files when signatures would suffice.

Research (e.g., *KGCompass*, *RepoHyper*) suggests the optimal interaction model is **Progressive Disclosure**:
1.  **Map**: High-level structural overview.
2.  **Skeleton**: Interface contracts (signatures) of relevant dependencies.
3.  **Flesh**: Full implementation details of *only* the focal files.

SlopChop will enforce this via the **Three-Turn Protocol**.

---

## 2. The Three-Turn Protocol

### Turn 1: The Seed (~2k tokens)
**User Input:** "I have a bug in payments."
**System Output:** A **Ranked Repository Map**.
*   **Content:** File tree + Symbol Signatures (no bodies).
*   **Ranking:** PageRank or Call-Graph centrality.
*   **Goal:** Allow the AI to identify *which* files contain the logic without reading them.

### Turn 2: The Skeleton Request (~6k tokens)
**User Input:** "The bug is likely in `src/payments/processor.rs`. Give me context."
**System Output:** A **Focus Pack (Depth 1)**.
*   **Focal File:** `src/payments/processor.rs` (Full Content).
*   **Neighbors:** `src/payments/gateway.rs`, `src/users/model.rs` (Signatures Only).
*   **Mechanism:** Graph traversal. If `processor.rs` calls `gateway.rs`, include `gateway`'s signatures.
*   **Goal:** Provide enough context to compile/type-check the fix without wasting tokens on neighbor implementation details.

### Turn 3: The Precision Strike (~20k tokens)
**User Input:** "I need to change how `gateway.submit()` handles errors."
**System Output:** Targeted Expansion.
*   **Content:** `src/payments/gateway.rs` (Full Content).
*   **Goal:** The AI now has the full implementation of the dependency it decided to modify.

---

## 3. Technical Implementation

### A. Skeleton Operators (The "Chop")
We need a robust way to strip function bodies while preserving structural validity (so the AI understands types).

```rust
// Original
fn calculate(x: i32) -> i32 {
    let y = x * 2;
    y + 1
}

// Skeleton (Rust)
fn calculate(x: i32) -> i32 { ... }

// Skeleton (Python)
def calculate(x: int) -> int:
    ...
```

*   **Action:** Enhance `src/skeleton.rs` (already exists) to support AST-based stripping for Rust, Python, TS.

### B. Graph-Aware Packing
The `pack` command must support graph traversal.

```bash
# Current
slopchop pack --focus src/main.rs

# Target
slopchop pack --focus src/main.rs --depth 1 --fidelity skeleton
```

*   **Action:** Update `src/pack/focus.rs` to leverage `src/graph/` for neighbor discovery.

### C. The "Zoom" Capability
Interactive refinement of the context window.

*   `slopchop slice src/lib.rs:20-50`: Extract specific line ranges.
*   `slopchop slice --symbol MyStruct`: Extract only that struct and its impls.

---

## 4. Immediate Next Steps

1.  **Refine `skeleton.rs`**: Ensure it produces valid syntax that LLMs recognize as "implementation hidden".
2.  **Enhance `RepoGraph`**: Ensure `src/graph/` correctly identifies imports/calls for traversal.
3.  **Update CLI**: Add `--depth` and `--neighbors` flags to `slopchop pack`.

---

## 5. Success Metrics

*   **Token Efficiency:** Fix a bug using <20% of the tokens required by a "dump all" approach.
*   **Solve Rate:** Maintain high fix rates by ensuring no *required* context is hidden (skeletons provide type info).
*   **Latency:** Pack generation must be near-instant (<500ms).