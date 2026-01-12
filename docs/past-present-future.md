# SlopChop Scan v2: Past, Present, Future

**Date:** 2026-01-12
**Version:** v1.6.0 (The "High-Integrity" Release)

---

## What Was Done This Session (The Great Refactor)

### 1. Architecture Cleanup ("The Chop")
- **Deleted `src/apply/patch`:** Removed surgical patching (V0/V1) in favor of whole-file replacement. Eliminated context-drift risks.
- **Deleted `src/audit`:** Removed static dead-code/similarity analysis. Replaced by `slopchop mutate` logic and external tools.
- **Refactored `ScanEngineV2`:** Split into `worker.rs` (IO/Parsing) and `inspector.rs` (Metrics) to fix CBO/SFOUT violations in the engine itself.
- **Fixed Config UI:** Repaired TUI rendering artifacts and casting panics.

### 2. Pattern Tuning & Expansion
- **Tuned P03 (N+1):** Restricted to explicit DB verbs (`fetch`, `query`) to eliminate noise on `HashMap::get`.
- **Tuned L02 (Boundary):** Restricted to index variables (`i`, `idx`) to allow valid threshold checks (`len >= 5`).
- **Added X06 (Dangerous Config):** Detects `dangerous()`, `verify_none`, `danger_accept_invalid_certs`.
- **Added X07 (Unbounded Deser):** Detects `bincode::deserialize` (allocation bomb risk).
- **Added I05 (Global Mutation):** Detects `std::env::set_var` in library code.

---

## Current Pattern Coverage (Rust)

| Category | IDs | Status |
|----------|-----|--------|
| **State** | S01, S02, S03 | �o. Stable |
| **Concurrency** | C03, C04 | �o. High Signal |
| **Security** | X01, X02, X03, **X06**, **X07** | �o. Production Ready |
| **Performance** | P01, P02, P03 (Tuned), P04, P06 | �o. Tuned |
| **Semantic** | M03, M04, M05 | �o. Stable |
| **Resource** | R07 | �o. Stable |
| **Idiomatic** | I01, I02, **I05** | �o. Stable |
| **Logic** | L02 (Tuned), L03 | �o. Low Noise |

**Total: 23 active patterns**

---

## Triage Report (Missing IDs)

### 1. Deferred (TypeScript Support)
These patterns define the roadmap for adding TS/JS support.
- **C01, C02, C05:** Async race conditions, floating promises (JS-specific).
- **R01, R02, R05:** Event listener leaks, RxJS subscriptions, spread operators.
- **X04, X05:** Prototype pollution, `JSON.parse` safety.

### 2. Dropped (Rust handled / Noise)
These were in the spec but cut during implementation.
- **S04 (Impure Function):** Requires deep Data Flow Analysis (too expensive).
- **S05 (Deep Mutation):** Handled by Rust Borrow Checker.
- **R03, R04 (Loop Alloc):** Merged into `P01` / `P02`.
- **R06 (Unbounded):** Requires proving `Vec` is never cleared (impossible with AST).
- **M01, M02 (Docs/Unused):** Handled by `rustc` / `clippy` lints.
- **L01 (Untested):** Replaced by `slopchop mutate` (Mutation testing).

---

## Next Session Priorities

1.  **TypeScript Implementation:**
    - Map the Deferred patterns above to `tree-sitter-typescript` queries.

2.  **Mutation Testing Polish:**
    - Since we deleted static dead code analysis, we need to ensure `slopchop mutate` is easy to use for verifying test coverage.

3.  **Documentation:**
    - Update `README.md` to reflect the removal of `audit` and `patch`.

---

## Key Files (The Core)

```
src/analysis/v2/
|-- mod.rs          # Engine Entry
|-- engine.rs       # Orchestration
|-- worker.rs       # Parsing/IO
|-- inspector.rs    # Scope Metrics (LCOM4, CBO)
`-- patterns/       # AST Logic
    |-- security.rs
    |-- performance.rs
    |-- concurrency_lock.rs
    |-- concurrency_sync.rs
    |-- db_patterns.rs
    |-- logic.rs
    |-- semantic.rs
    `-- idiomatic.rs
```