# Past / Present / Future
**Status:** Canonical (living snapshot)
**Last updated:** 2025-12-28
**Canonical policy:** This document states the current operational reality and the single next action.
---
## 1) Past (What changed recently)
**v1.2.x: The Law of Locality is Live.**
*   **Stability Classifier:** Computes fan-in (Cₐ), fan-out (Cₑ), Instability (I), and Skew (K) for all files.
*   **Node Identity:** Files are classified as StableHub, VolatileLeaf, IsolatedDeadwood, GodModule, or Standard.
*   **Universal Locality Algorithm:** Validates every dependency edge against distance thresholds with smart exemptions for structural Rust patterns (lib.rs, mod.rs re-exports, vertical routing, shared infrastructure).
*   **Actionable Analysis:** Violations are categorized (ENCAPSULATION_BREACH, MISSING_HUB, TIGHT_COUPLING, SIDEWAYS_DEP) with specific refactoring suggestions.
*   **Topology Metrics:** Module coupling visualization and Topological Entropy score.
*   **CLI Integration:** `slopchop scan --locality` with configurable thresholds in `[rules.locality]`.
---
## 2) Present (Where we are right now)
**Status:** STABLE (Enforcement + Intelligence)
### Operator-visible contract
*   `slopchop apply` (Stage + Decontaminate)
*   `slopchop check` (Verify stage or workspace)
*   `slopchop apply --promote` (Atomic commit)
*   `slopchop pack --focus <file>` (High-density context with SHA snapshots)
*   `slopchop scan` (Law of Complexity enforcement)
*   `slopchop scan --locality` (Law of Locality enforcement)
### System Posture
SlopChop enforces both **structural integrity** (complexity, naming, nesting) and **topological integrity** (locality, coupling, hub/leaf classification). The scanner produces actionable output, not just warnings.
### Current Topology (slopchop scan --locality)
*   169 edges, 162 passed, 7 failed (4.1% entropy)
*   2 Hub candidates: `clipboard/mod.rs`, `stage/mod.rs`
*   1 Tight coupling: `cli/handlers.rs` → `apply/mod.rs`
---
## 3) Future (What we do next)
We are entering the **Multi-Language Era (v1.3.x)**.
### Objectives
*   **TypeScript Support:** Extend locality scanning to TS/JS projects. The infrastructure is language-agnostic; we need import extraction and resolution for TS.
*   **Python Support:** Same pattern — import extraction via tree-sitter.
*   **Wire Locality into Check:** Make locality violations block `slopchop check` when `mode = "error"`.
### Immediate Next Action
**Add TypeScript import resolution to `graph/resolver.rs` and test locality scanning on a TS project.**
