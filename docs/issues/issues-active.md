# ACTIVE Issues

## Label Set

Use only these labels across active and backlog issues:
`Accuracy`, `Config`, `CLI`, `Reporting`, `AI Workflow`, `Adoption`, `Architecture`, `Cleanup`, `Language Support`, `Detection Rules`, `Testing`, `Performance`, `Safety`, `Branching`, `Web Stack`, `Integrations`

## Priority Theme

Current roadmap focus: harmonize NETI with SEMMAP by extracting a shared multi-language analysis crate, `omni-ast`, and refactoring NETI to consume semantic concepts instead of Rust-specific syntax checks.

---

## [51] Adopt `omni-ast` as NETI's primary semantic engine
**Status:** OPEN
**Files:** `src/lang.rs`, `src/graph/imports.rs`, `src/analysis/patterns/`, shared `omni-ast` semantics modules
**Labels:** Architecture, Adoption, Language Support, Integrations
**Depends on:** [50]

**Problem:** The shared crate now exists and NETI consumes pieces of it, but NETI is not yet primarily governed by `omni-ast` semantics. Detector behavior still largely lives in Rust-specific rule code, which means the extraction milestone is real while the strategic adoption milestone remains open.

**Fix:**

1. Define the shared semantic contract NETI detectors should consume as their main interface.
2. Move detector-facing language and concept knowledge behind `omni-ast` instead of keeping it embedded in NETI rules.
3. Replace detector-local Rust syntax assumptions with shared semantic queries in the main rule paths.
4. Prove the adoption with cross-language rule coverage showing one rule can execute through the shared semantic layer.

**Resolution:**

---

## [17] Implement `LangSemantics` in the new shared crate
**Status:** OPEN
**Files:** `src/lang.rs`, `src/lang/semantics.rs` (new or moved), shared `omni-ast` semantics module
**Labels:** Architecture, Language Support, Detection Rules
**Depends on:** [51]

**Problem:** NETI needs a stable semantic interface above raw syntax. That interface should live in the shared crate so both projects can reason about concepts such as test context, heap allocation, mutation, locking, and exported roles without duplicating language-specific logic.

**Fix:**

1. Define a `LangSemantics` trait in the shared crate.
2. Make the trait answer the semantic queries NETI detectors need, such as `is_test_context()` and `has_concept(Concept::HeapAllocation)`.
3. Map SEMMAP-style badges and concepts onto that trait surface.
4. Expose the interface so NETI detectors can query semantics without directly handling AST syntax.

**Resolution:**

---

## [41] Port SEMMAP SWUM expansion to Neti naming rules
**Status:** OPEN
**Files:** `../semmap/src/swum/`, `src/analysis/naming.rs`, shared `omni-ast` SWUM module
**Labels:** Language Support, Detection Rules, Architecture
**Depends on:** [51]

**Problem:** NETI naming guidance is still shallow and language-specific. SEMMAP already has a SWUM engine that can expand identifiers into verb-intent phrases, which is the right foundation for cross-language naming rules.

**Fix:**

1. Port SEMMAP's SWUM engine into `omni-ast`.
2. Replace NETI naming-rule heuristics with SWUM-backed semantic expansion where practical.
3. Use the shared engine to reason about verbs, themes, acronyms, and intent across languages.
4. Add tests proving naming analysis behavior works through the shared interface instead of language-local hacks.

**Resolution:**

---

## [18] Wire shared semantics into performance detectors
**Status:** OPEN
**Files:** `src/analysis/patterns/performance.rs`, `src/analysis/patterns/performance_test_ctx.rs`
**Labels:** Architecture, Language Support, Detection Rules, Performance
**Depends on:** [17]

**Problem:** Performance detectors still rely on Rust-shaped vocabulary and local heuristics. They should operate on shared semantic concepts so one rule can run against Rust, Python, Go, and JS/TS through the same interface.

**Fix:**

1. Replace detector-local vocabulary checks with queries against shared semantics.
2. Express rules in terms of concepts such as allocation, lookup, collection iteration, and test context.
3. Remove path-based skip heuristics that exist only to compensate for weak semantics.
4. Add tests showing the same detector intent can run across multiple languages through the shared layer.

**Resolution:**

---

## [19] Wire shared semantics into logic detectors
**Status:** OPEN
**Files:** `src/analysis/patterns/logic.rs`, `src/analysis/patterns/logic_helpers.rs`
**Labels:** Architecture, Language Support, Detection Rules
**Depends on:** [17]

**Problem:** Logic detectors currently mix rule intent with Rust-specific syntax assumptions. That blocks cross-language governance and makes the rules harder to reason about.

**Fix:**

1. Rewrite logic detectors to query shared semantics rather than raw syntax vocabulary.
2. Preserve Rust-only proof helpers only where they materially improve precision.
3. Keep any Rust-specific precision layer clearly gated on language, not embedded in the core rule definition.
4. Add regression coverage proving shared semantics drives the rule while precision enhancers remain optional.

**Resolution:**

---

## [20] Wire shared semantics into concurrency and remaining detectors
**Status:** OPEN
**Files:** `src/analysis/patterns/semantic.rs`, `src/analysis/patterns/concurrency.rs`, `src/analysis/patterns/concurrency_lock.rs`, `src/analysis/patterns/concurrency_sync.rs`
**Labels:** Architecture, Language Support, Detection Rules, Safety
**Depends on:** [17]

**Problem:** Lock-type knowledge, mutation receiver patterns, and other semantic cues still live inline inside NETI detectors. That leaves the cross-language abstraction incomplete.

**Fix:**

1. Move lock and sync concepts behind the shared semantics layer.
2. Move mutation and state-change concepts behind the shared semantics layer.
3. Remove remaining detector-local Rust vocabulary where the shared crate can own it.
4. Extend tests to prove these rule families work through semantic concepts rather than syntax matching.

**Resolution:**

---

## [21] Populate Python semantics in the shared crate
**Status:** OPEN
**Files:** shared `omni-ast` semantics tables
**Labels:** Language Support, Architecture, Detection Rules
**Depends on:** [17]

**Problem:** Python needs a first-class semantics table in the shared crate before cross-language detector execution can be credible.

**Fix:**

1. Add Python test-context semantics.
2. Add Python heap, lookup, length, mutation, and loop concepts.
3. Map Python syntax and library vocabulary onto the shared concept model.
4. Verify NETI rules can consume Python semantics through the same detector queries used for Rust.

**Resolution:**

---

## [22] Populate TypeScript semantics in the shared crate
**Status:** OPEN
**Files:** shared `omni-ast` semantics tables
**Labels:** Language Support, Architecture, Detection Rules, Web Stack
**Depends on:** [17]

**Problem:** TypeScript and JavaScript need shared semantics coverage so NETI rules can execute over web code through the same concept interface.

**Fix:**

1. Add JS/TS test-context semantics.
2. Add JS/TS heap, lookup, length, mutation, and loop concepts.
3. Map JS/TS library and syntax vocabulary onto the shared concept model.
4. Verify NETI rules can consume JS/TS semantics through the common detector interface.

**Resolution:**
