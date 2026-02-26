# issues-0003: Post-Launch Work
---
## FORMAT (DO NOT MODIFY)
**Status values:** `OPEN`, `IN PROGRESS`, `DONE`, `DESCOPED`
**Issue format:**
```
## [N] Title
**Status:** OPEN
**Files:** list of files to modify
Description of the task.
**Resolution:** (fill when DONE) What was done, any notes.
```
**Instructions:**
* Work issues in order you feel is most important.
* Update status as you go
* Add **Resolution:** when completing
* Don't modify this FORMAT section
* Content below the line is the work.
---

## [1] L03: Fixed-size array indexing produces false positives
**Status:** DONE
**Files:** `src/analysis/patterns/logic.rs`
**Resolution:** Added `is_fixed_size_array_access()` with three detection strategies. Reduced rand L03 from 13 to 4.

---

## [2] I02: Type-blind duplicate arm detection flags unfuseable match arms
**Status:** DONE
**Files:** `src/analysis/patterns/idiomatic.rs`
**Resolution:** Rewrote `patterns_have_incompatible_types()` to handle simple and tuple variant destructuring. Eliminates 9 false positives.

---

## [3] Syntax error on `#![doc(...)]` inner attribute
**Status:** DONE
**Files:** `src/analysis/checks/syntax.rs`
**Resolution:** Added `is_inside_inner_attribute()` to catch tree-sitter error nodes on interior content of attributes.

---

## [4] P01/P06: Skip test functions and intentional patterns
**Status:** DONE
**Files:** `src/analysis/patterns/performance.rs`
**Resolution:** Added `is_test_context()` walking ancestors for `#[test]` and `#[cfg(test)]`. P01/P06 skipped in test code. Extracted to `performance_test_ctx.rs` to fix complexity violation.

---

## [5] I01: Suggesting derive_more to zero-dependency crates
**Status:** DONE
**Files:** `src/analysis/patterns/idiomatic.rs`
**Resolution:** I01 is now an INFO suggestion with softened language ("consider derive_more if already using proc macros").

---

## [6] Confidence tiers: violations must declare what Neti knows vs what it guesses
**Status:** DONE
**Files:** `src/types.rs`, `src/reporting.rs`, all pattern detectors
**Resolution:** Added `Confidence` enum (High/Medium/Info) to `Violation`. Updated reporter to group by rule, deduplicate with back-references, and show static educational content (WHY/FIX/SUPPRESS) on first occurrence. Wired correct confidence tiers into all 8 pattern detector files. Rand scan now shows 6 errors, 6 warnings, 2 suggestions (instead of 14 undifferentiated errors).

---

## [7] Heuristic sharpening: Default to Medium confidence for untrackable variables
**Status:** DONE
**Files:** `src/analysis/patterns/logic.rs`, `src/analysis/patterns/performance.rs`
**Resolution:** Split logic.rs into `logic.rs` + `logic_helpers.rs` + `logic_proof.rs`. Split performance.rs into `performance.rs` + `performance_test_ctx.rs`. Implemented three-state L03 confidence classification (cross-scope → Medium, found declaration → High, not found → Medium). Implemented P01 confidence split (heap keyword → High, heuristic keyword → Medium, long-name-only → Medium). Fixed `check_p04` to walk descendants (was only checking direct children). Added 15+ mutant-killing tests. Ran cargo-mutants, triaged survivors, killed all meaningful logic-gap mutants.

---

## [16] HOTFIX: `neti-report.txt` regression and pipeline integration
**Status:** DONE
**Files:** `src/cli/handlers/mod.rs`, `src/cli/handlers/scan_report.rs`, `src/reporting.rs`
**Resolution:** Fixed the regression where `neti check` was overwriting `neti-report.txt` exclusively with external linter output. `neti check` now correctly acts as the master verification pipeline, running `neti scan` (AST/structural checks) first, then running the external verification commands. Formatted output for both passes is cleanly appended in memory and written in high fidelity to `neti-report.txt`.

---

## [23] HOTFIX: Governance compliance (Atomicity, Coupling, Cohesion)
**Status:** DONE
**Files:** `src/analysis/patterns/*`, `src/cli/handlers/mod.rs`, `src/analysis/engine.rs`, `src/verification/mod.rs`, `src/graph/rank/builder.rs`
**Resolution:** Rescued a botched refactor. Split test suites out of 8 rule files into `_test.rs` modules to satisfy the Law of Atomicity (<2000 tokens). Refactored `Engine` into free functions to fix CBO/SFOUT coupling violations. Fixed `CommandResult` LCOM4/AHF encapsulation by introducing `CommandResultInner` with explicit getter methods. Fixed P02/P04 looping violations in graph building. Codebase is now 100% green with zero `neti:allow` suppressions.

---

## [24] Root src/ cleanup and Domain consolidation
**Status:** OPEN
**Files:** `src/discovery.rs`, `src/file_class.rs`, `src/project.rs`, `src/detection.rs`, `src/constants.rs`, `src/reporting.rs`, `src/lib.rs`, `src/workspace/mod.rs`
Consolidate filesystem/project discovery files into a `workspace` module. Move `src/reporting.rs` to `src/reporting/mod.rs` for module style consistency.
**Resolution:**

---

## [8] Safety rule robustness: recognize SAFETY justifications
**Status:** OPEN
**Files:** `src/analysis/safety.rs`, tests
Requiring `// SAFETY:` is good, but the rule should recognize nearby justifications.
**Resolution:**

---

## [9] Locality integration into standard scan pipeline
**Status:** OPEN
**Files:** `src/graph/locality/` (all), `src/cli/locality.rs`, `src/config/locality.rs`
**Resolution:**

---

## [10] Governance-grade clippy integration
**Status:** OPEN
**Files:** `src/verification/runner.rs`, `src/cli/audit.rs`, `src/config/types.rs`, tests
**Resolution:**

---

## [11] Python and TypeScript pattern detection parity
**Status:** OPEN
**Files:** `src/analysis/patterns/` (all), `src/lang.rs`
**Resolution:**

---

## [12] Regression suite
**Status:** OPEN
**Files:** `tests/` (new), `src/analysis/*`, CI config
**Resolution:**

---

## [13] Syntax false positives on `&raw const` / `&raw mut` (Rust 1.82+)
**Status:** DONE
**Files:** `src/analysis/checks/syntax.rs`
Added `is_raw_pointer_syntax()` with three detection strategies: error node text
starts with `raw const`/`raw mut`; lone `raw` token whose parent contains the
full pattern; ancestor walk (≤5 levels, stops at statement/item boundaries)
finding a node whose text contains `&raw const`/`&raw mut`. Wired into
`is_known_unsupported_construct`. lazuli syntax errors drop from 42 to 0.
**Resolution:** Added `is_raw_pointer_syntax()` with three detection strategies: error node text starts with `raw const`/`raw mut`; lone `raw` token whose parent contains the full pattern; ancestor walk (≤5 levels, stops at statement/item boundaries) finding a node whose text contains `&raw const`/`&raw mut`. Wired into `is_known_unsupported_construct`. lazuli syntax errors: 42 → 0.

---

## [14] L03 volume: deduplicate or suppress unresolvable `self.field[0]`
**Status:** OPEN
**Files:** `src/analysis/patterns/logic.rs`, `src/analysis/patterns/logic_helpers.rs`
L03 produces 110 warnings on the lazuli scan, almost all on `self.field[0]` where the field type can't be traced across struct boundaries. These are correctly Medium confidence but the sheer volume drowns out actionable findings.

Two possible approaches (pick one):
* **Deduplicate per-field:** Report `self.regs[N]` once per struct, not once per access site. Back-reference subsequent occurrences.
* **Suppress unresolvable:** If confidence reason is "cannot trace type through field access" and the receiver is `self.field`, don't report at all — the signal-to-noise ratio is too low.

Target: lazuli L03 count drops from 110 to ~10-15 (only non-self, non-field indexing).

**Resolution:**

---

## [15] P04 false positives on 2D numeric iteration
**Status:** OPEN
**Files:** `src/analysis/patterns/performance.rs`
P04 (nested loop) fires 34 times on lazuli, mostly on tile-based texture decode (`gxtex`) and matrix operations. Nested `for x in 0..w { for y in 0..h }` with numeric range bounds is intentional 2D iteration, not algorithmic inefficiency.

Required behavior:
* If both the outer and inner loop use numeric range patterns (`0..N`, `0..=N`, range expressions), downgrade to Info or skip entirely
* Only flag nested loops where the inner loop iterates over a collection that could be replaced with a lookup structure

**Resolution:**

---

### Calibration results (rand v0.10.0)

| Round | Errors | Warnings | Suggestions | Total |
|-------|--------|----------|-------------|-------|
| Initial | 57 | 0 | 0 | 57 |
| Config tuning | 38 | 0 | 0 | 38 |
| False positive fixes | 19 | 0 | 0 | 19 |
| Test skipping | 14 | 0 | 0 | 14 |
| Confidence Tiers | 6 | 6 | 2 | 14 |
| **Heuristic Sharpening + Mutant Hardening** | **4** | **17** | **2** | **23** |

Note: violation count increased from 14 to 23 because `check_p04` descendant walking now correctly finds nested loops that were previously invisible. All new findings are genuine.

### Calibration results (lazuli emulator)

| Category | Count | Assessment |
|----------|-------|------------|
| LAW OF PARANOIA (unsafe) | 43 | **Actionable** |
| LAW OF INTEGRITY (syntax) | 42 | **False positive** — `&raw const` parser gap [13] |
| L03 (index) | 110 | **Noisy** — mostly `self.field[0]` [14] |
| P04 (nested loop) | 34 | **Noisy** — mostly 2D tile iteration [15] |
| I02 (duplicate arms) | 28 | Real but low-priority |
| CBO/SFOUT/LCOM4/AHF | ~80 | Expected for emulator domain |
| M03/M04 (naming) | 12 | **Actionable** |
| P02/R07 | 9 | **Actionable** |
| Complexity | 10 | **Actionable** |
| Atomicity | 17 | Real |

**Signal-to-noise: ~17% actionable, 10% false positive, 73% true-but-noisy.**
**Target after [13][14][15]: ~75% actionable, 0% false positive, 25% domain-expected.**

### Priority order (revised)
1. **[13] `&raw const` syntax suppression** — kills 42 false positives, zero tolerance for FPs
2. **[14] L03 volume control** — kills ~95 noisy warnings
3. **[15] P04 numeric range suppression** — kills ~25 noisy warnings
4. **[8] Safety rule robustness** — correctness fix for the 43 real PARANOIA findings
5. **[9]-[10] Infrastructure** — locality, clippy tiers
6. **[11] Language parity**
7. **[12] Regression suite**

## Fully language-specific (would need rewrite for each language)

**`syntax.rs`** — `is_inside_inner_attribute` is pure Rust AST node checking. The `&raw const` fix will be too. Python/TS have completely different syntax error patterns. This one is inherently per-language and that's fine — syntax checking *should* be language-specific.

**`performance_test_ctx.rs`** — Looks for `#[test]` and `#[cfg(test)]` by walking Rust AST node kinds. Python needs `def test_`, `@pytest.mark`. TypeScript needs `describe(`, `it(`, `test(`. This is currently hardcoded Rust logic but the *concept* (skip test code) is universal. Should be a language-provided predicate: `lang.is_test_context(node)`.

**`safety.rs`** — Checks for `// SAFETY:` on `unsafe` blocks. Python doesn't have `unsafe`. TypeScript doesn't either. This is genuinely Rust-only (and C/C++/Zig eventually). Fine as-is.

**`logic_proof.rs`** — `is_fixed_size_array_access`, `find_struct_field_array_size`, `extract_impl_type_name`, `find_param_array_size` — all deeply Rust-specific. Parses `impl` blocks, Rust struct syntax, Rust array type annotations. Python/TS don't have fixed-size arrays in the same way. This entire module is a Rust-only optimization for reducing L03 noise. That's acceptable — it's a precision enhancer, not a core detector.

## Rust-specific but shouldn't be

**`logic_helpers.rs` — `is_index_variable`** — Hardcoded list: `i`, `j`, `k`, `n`, `idx`, plus substring checks for `index`, `pos`, `ptr`, `offset`, `cursor`. These names are universal across languages. The *list* is fine, but it's embedded in a Rust-only module. Should be shared.

**`logic_helpers.rs` — `has_matching_parameter`** — Parses Rust parameter syntax textually (`strip_prefix("mut ")`, `strip_prefix("&mut ")`). Python params look like `def f(v: list[int])`. TS looks like `function f(v: number[])`. This needs to be a language-provided function.

**`logic_helpers.rs` — `decl_matches_variable`** — Looks for `let` and `let mut`. Python uses bare assignment. TS uses `let`, `const`, `var`. Should match on tree-sitter node kinds (`let_declaration` vs `assignment` vs `variable_declaration`) rather than string parsing.

**`performance.rs` — `looks_heap_owning`** — Checks for `String`, `Vec`, `HashMap`, `Box`, `Rc`, `Arc` via substring matching on variable names. In Python the heap types are `list`, `dict`, `set`, `str`. In TS they're `Array`, `Map`, `Set`, `string`. The entire approach of guessing types from variable names is language-specific *and* unreliable. This is the weakest heuristic in the codebase.

**`performance.rs` — `is_arc_or_rc_clone`** — Checks for `Arc::clone` and `Rc::clone` string patterns. Pure Rust idiom. Python/TS don't have reference-counted clone patterns. This should be a language-provided "cheap clone" predicate.

**`performance.rs` — `should_skip` path filter** — Hardcoded path substrings (`/cli/`, `/ui/`, `reporting`, `analysis/`). This isn't even Rust-specific, it's *Neti*-specific. Probably shouldn't exist at all — it's suppressing detections on Neti's own code structure.

## Actually language-agnostic (work as-is across languages)

**`logic.rs` — L02 detection** — Looks for `<=`/`>=` with `.len()`. The `.len()` part is Rust/Python but the concept (off-by-one on collection length) is universal. Would need `lang.length_methods()` returning `["len", "length", "size", "count"]` per language. Minor adaptation.

**`logic.rs` — L03 `detect_index_zero`** — Looks for `[0]` index expressions. This is syntactically identical in Rust, Python, TS, C, Go — square bracket indexing is nearly universal. The *detection* is language-agnostic. The *confidence classification* is Rust-specific (because it calls into `logic_proof.rs`).

**`logic.rs` — L03 `detect_first_last_unwrap`** — `.first().unwrap()` / `.last().unwrap()`. Rust-specific method names but the pattern (accessor-then-force-unwrap) exists everywhere. Python: `list[0]` without guard. TS: `arr[0]!` or `arr.at(0)!`. Could be generalized with language-provided "unsafe accessor" patterns.

**`performance.rs` — P02** — `.to_string()` / `.to_owned()` in loops. Rust-specific method names. Python equivalent would be `str()` or `f""` in loops. TS would be `String()` or template literals. Same concept, different surface syntax. Needs language-provided "allocation method" list.

**`performance.rs` — P04** — Nested loop detection. Walks tree-sitter loop node kinds. These are different per language (`for_expression` vs `for_statement` vs `for_in_statement`) but tree-sitter query patterns handle this naturally. The *logic* is language-agnostic, just needs the right node kind names from the grammar.

**`performance.rs` — P06** — `.find()` / `.position()` in loops. Linear search method names differ by language (Python: `index()`, `find()`. TS: `find()`, `indexOf()`). Same pattern, needs language-provided method list.

**`concurrency.rs` / `concurrency_lock.rs` / `concurrency_sync.rs`** — Mutex-across-await, undocumented sync primitives. These are Rust/async-specific in their current form but the concepts exist in Python (`asyncio.Lock`) and TS (`Mutex` patterns in worker threads). Moderately portable.

**`semantic.rs`** — M03 (getter with `&mut self`), M04 (`is_*` returning non-bool), M05 (`calculate_*` with mutation). M03's `&mut self` is Rust-specific but the concept (getter that mutates) exists in every language. M04/M05 are pure naming convention checks — fully language-agnostic.

**`idiomatic.rs`** — I01 (manual From impl) is Rust-only. I02 (duplicate match arms) works on any language with pattern matching (Rust, Python `match`, TS with switch).

**Structural metrics** (`structural.rs`, `scope.rs`, `inspector.rs`) — LCOM4, CBO, SFOUT, AHF. These are computed from scope extraction which is Rust-specific in implementation, but the *metrics* are language-agnostic OO concepts.

## Summary

| Category | Files | Status |
|---|---|---|
| Inherently per-language (fine) | syntax.rs, safety.rs, logic_proof.rs | No change needed |
| Should be language-configured | performance_test_ctx.rs, logic_helpers.rs (`has_matching_parameter`, `decl_matches_variable`), performance.rs (`looks_heap_owning`, `is_arc_or_rc_clone`, `should_skip`) | Needs `Lang` trait methods |
| Needs method/keyword tables per language | L02 (`.len()`), P02 (`.to_string()`), P06 (`.find()`), L03 (`.first().unwrap()`) | Move to `lang.rs` lookup tables |
| Already agnostic | P04, L03 index detection, M03/M04/M05, I02, all structural metrics | Works now |

The biggest structural debt is that `performance.rs` is about 60% Rust-specific heuristics pretending to be general logic. The cleanest path for [11] would be to define a trait or config on `Lang` that provides: test context detection, heap type keywords, cheap clone patterns, allocation method names, and length method names. Then the detectors become genuinely language-agnostic and each language just fills in its table.

# issues.md — Language Agnosticism Retrofit

## STATUS OVERVIEW

The analysis engine works. The architecture is clean. The problem is narrow:
**detection logic, surface syntax, and semantic vocabulary are tangled in ~8 files.**
Everything else (graph, locality, config, CLI, spinner, reporting) is already language-agnostic.

This is a refactor, not a rewrite.

---

## PHASE 0 — Precision fixes (Rust-only, no architecture change)

These kill false positives NOW and teach us which code is inherently per-language.

---

## [13] Syntax false positives on `&raw const` / `&raw mut` (Rust 1.82+)
**Status:** OPEN
**Files:** `src/analysis/checks/syntax.rs`

42 false positives on lazuli. Tree-sitter error nodes on valid modern Rust syntax.
Same pattern as the `#![doc(...)]` fix — detect error nodes inside `&raw const expr` / `&raw mut expr` and suppress.

**Acceptance:** lazuli syntax errors drop from 42 to 0.

---

## [14] L03 volume: deduplicate `self.field[N]` indexing
**Status:** OPEN
**Files:** `src/analysis/patterns/logic.rs`, `src/analysis/patterns/logic_helpers.rs`

110 L03 warnings on lazuli, almost all `self.field[0]` where field type is untraceable.
Deduplicate per-field per-struct: report `self.regs[N]` once, back-reference subsequent sites.

**Acceptance:** lazuli L03 drops from 110 to ~10-15.

---

## [15] P04 false positives on 2D numeric iteration
**Status:** OPEN
**Files:** `src/analysis/patterns/performance.rs`

34 P04 hits on lazuli, mostly `for x in 0..w { for y in 0..h }`.
If both loops use numeric range bounds, downgrade to Info or skip.

**Acceptance:** lazuli P04 drops from 34 to ~5.

---

## [8] Safety rule: recognize nearby SAFETY justifications
**Status:** OPEN
**Files:** `src/analysis/safety.rs`

The rule requires `// SAFETY:` but should recognize justifications within 3 lines above the unsafe block, not just immediately adjacent.

**Acceptance:** lazuli PARANOIA findings become precise — every remaining one is genuinely unjustified.

---

## PHASE 1 — The abstraction layer

This is the structural fix. Define `LangSemantics` so detectors query language-provided tables instead of hardcoding Rust vocabulary.

---

## [17] Define `LangSemantics` trait and Rust implementation
**Status:** OPEN
**Files:** `src/lang.rs` (modify), `src/lang/semantics.rs` (new)

Create the abstraction that decouples detectors from language-specific knowledge:

```rust
pub struct LangSemantics {
    // Test context detection
    pub test_markers: &'static [TestPattern],

    // Type vocabulary
    pub heap_types: &'static [&'static str],
    pub lock_types: &'static [&'static str],

    // Method vocabulary
    pub alloc_methods: &'static [&'static str],
    pub linear_search_methods: &'static [&'static str],
    pub length_methods: &'static [&'static str],
    pub unsafe_accessors: &'static [&'static str],
    pub clone_methods: &'static [&'static str],

    // Syntax vocabulary
    pub let_node_kinds: &'static [&'static str],
    pub loop_node_kinds: &'static [&'static str],
    pub param_strip_prefixes: &'static [&'static str],
    pub mut_receiver_pattern: Option<&'static str>,
}

pub enum TestPattern {
    Attribute(&'static str),      // #[test], #[cfg(test)]
    FunctionPrefix(&'static str), // def test_
    CallExpression(&'static str), // describe(, it(, test(
}
```

Populate the Rust table with all values currently hardcoded in detectors.
Add `Lang::semantics(&self) -> &LangSemantics`.

**Acceptance:** `cargo test` passes. No behavior change. Rust table returns identical values to what's currently hardcoded.

---

## [18] Wire `LangSemantics` into performance detectors
**Status:** OPEN
**Files:** `src/analysis/patterns/performance.rs`, `src/analysis/patterns/performance_test_ctx.rs`

Replace all hardcoded vocabulary:
- `looks_heap_owning()` → queries `semantics.heap_types`
- `is_arc_or_rc_clone()` → queries `semantics.clone_methods`
- P02 `.to_string()` / `.to_owned()` → queries `semantics.alloc_methods`
- P06 `.find()` / `.position()` → queries `semantics.linear_search_methods`
- P04 loop detection → queries `semantics.loop_node_kinds`
- Test context → queries `semantics.test_markers`

Delete `should_skip()` entirely. If findings are false positives, fix heuristics. If real, don't hide them.

**Acceptance:** All performance tests pass. `should_skip` is gone. Detectors take `&LangSemantics` parameter.

---

## [19] Wire `LangSemantics` into logic detectors
**Status:** OPEN
**Files:** `src/analysis/patterns/logic.rs`, `src/analysis/patterns/logic_helpers.rs`

Replace hardcoded vocabulary:
- L02 `.len()` → queries `semantics.length_methods`
- `has_matching_parameter()` Rust-specific prefix stripping → queries `semantics.param_strip_prefixes`
- `decl_matches_variable()` `let`/`let mut` matching → queries `semantics.let_node_kinds`

Leave `logic_proof.rs` alone — it's a Rust-only precision enhancer and that's correct. Gate it behind `Lang::Rust` check.

**Acceptance:** Logic tests pass. `logic.rs` and `logic_helpers.rs` have zero hardcoded Rust method names.

---

## [20] Wire `LangSemantics` into remaining detectors
**Status:** OPEN
**Files:** `src/analysis/patterns/semantic.rs`, `src/analysis/patterns/concurrency.rs`, `src/analysis/patterns/concurrency_lock.rs`, `src/analysis/patterns/concurrency_sync.rs`

- M03 `&mut self` → queries `semantics.mut_receiver_pattern`
- Concurrency lock types → queries `semantics.lock_types`

**Acceptance:** All pattern detectors receive `&LangSemantics`. No detector file contains hardcoded language-specific method or type names (except `logic_proof.rs` which is explicitly Rust-gated).

---

## PHASE 2 — Language tables

With the abstraction in place, adding a language is just filling in a table.

---

## [21] Python `LangSemantics` table
**Status:** OPEN
**Files:** `src/lang/semantics.rs` (add Python table)

```rust
pub static PYTHON: LangSemantics = LangSemantics {
    test_markers: &[
        TestPattern::FunctionPrefix("test_"),
        TestPattern::Attribute("pytest.mark"),
    ],
    heap_types: &["list", "dict", "set", "str", "bytearray"],
    lock_types: &["asyncio.Lock", "threading.Lock", "Lock"],
    alloc_methods: &["str(", "list(", "dict(", "copy("],
    linear_search_methods: &["index", "find", "count"],
    length_methods: &["len"],
    unsafe_accessors: &[],
    clone_methods: &["copy(", "deepcopy("],
    let_node_kinds: &["assignment", "augmented_assignment"],
    loop_node_kinds: &["for_statement", "while_statement"],
    param_strip_prefixes: &[],
    mut_receiver_pattern: None,
};
```

**Acceptance:** `neti scan` on a Python project produces findings using Python vocabulary. No Rust false positives (no `.len()`, no `Vec`, no `#[test]`).

---

## [22] TypeScript `LangSemantics` table
**Status:** OPEN
**Files:** `src/lang/semantics.rs` (add TypeScript table)

```rust
pub static TYPESCRIPT: LangSemantics = LangSemantics {
    test_markers: &[
        TestPattern::CallExpression("describe"),
        TestPattern::CallExpression("it"),
        TestPattern::CallExpression("test"),
    ],
    heap_types: &["Array", "Map", "Set", "string", "object"],
    lock_types: &[],
    alloc_methods: &["String(", "Array.from(", "JSON.stringify(", "JSON.parse("],
    linear_search_methods: &["find", "findIndex", "indexOf", "includes"],
    length_methods: &["length"],
    unsafe_accessors: &["at(0)"],
    clone_methods: &["structuredClone(", "slice("],
    let_node_kinds: &["variable_declaration", "lexical_declaration"],
    loop_node_kinds: &["for_statement", "while_statement", "for_in_statement"],
    param_strip_prefixes: &[],
    mut_receiver_pattern: None,
};
```

**Acceptance:** `neti scan` on a TypeScript project produces findings using TS vocabulary.

---

## PHASE 3 — Validation and infrastructure

---

## [12] Cross-language regression suite
**Status:** OPEN
**Files:** `tests/` (new)

Create test fixtures for each language:
- `tests/fixtures/rust/` — known-good Rust files with expected findings
- `tests/fixtures/python/` — equivalent patterns in Python
- `tests/fixtures/typescript/` — equivalent patterns in TypeScript

Each fixture file should trigger exactly the same rule set (P01, P04, L03, etc.) to prove detectors are language-agnostic.

**Acceptance:** `cargo test` runs all three language fixtures. Same rules fire on equivalent patterns across languages.
