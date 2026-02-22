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
**Status:** OPEN
**Files:** `src/analysis/patterns/logic.rs`
L03 flags `self.s[0]` on `[u32; 4]`, `seed[0]` on `[u8; 32]`, and similar constant-index access on fixed-size arrays. These cannot panic. This is the single largest false positive source — 13 violations on the rand crate alone.

Required behavior:
* When the receiver is a fixed-size array (`[T; N]`), and the index is a constant literal less than N, suppress the violation.
* Detection approach: walk up from the index expression to find the variable/field declaration. If the declaration has array type syntax `[T; N]`, extract N and compare against the index constant.
* Also suppress `self.field[0]` when the struct field is declared as a fixed-size array (requires walking to the struct definition or recognizing the pattern heuristically).
* Keep L03 active for dynamic indices and slice/Vec indexing where bounds are unknown.

Deliverables:
* Implement `is_fixed_size_array_access()` in logic.rs
* Add tests:
  * `let seed = [0u8; 32]; seed[0] = 1;` — NOT flagged
  * `let v: Vec<i32> = vec![1]; v[0]` — still flagged
  * `self.s[0]` where `s: [u32; 4]` — NOT flagged
**Resolution:**

---

## [2] I02: Type-blind duplicate arm detection flags unfuseable match arms
**Status:** OPEN
**Files:** `src/analysis/patterns/idiomatic.rs`
I02 flags match arms as "duplicate" when the arm bodies are textually identical but the bindings have different concrete types. Example from rand's `IndexVec`:
```rust
match self {
    IndexVec::U32(v) => v.len(),  // v: Vec<u32>
    IndexVec::U64(v) => v.len(),  // v: Vec<u64>
}
```
Neti suggests `U32(v) | U64(v) => v.len()` which does not compile — you cannot bind different types to the same name in a combined pattern. This produced 9 false positives on rand alone.

Required behavior:
* When an enum has variants wrapping different inner types, skip the duplicate-arm check.
* Heuristic: if the enum definition shows variants with different type parameters (e.g., `U32(Vec<u32>)` vs `U64(Vec<u64>)`), the arms are structurally forced to be separate.
* Minimum viable fix: if the match arms destructure enum variants with bindings, and the enum has more than one variant with different payloads, suppress I02.
* More precise fix: check if the variants' inner types differ at the AST level.

Deliverables:
* Update I02 detection to skip type-incompatible arms
* Add tests with `enum Foo { A(u32), B(u64) }` match — NOT flagged
* Keep I02 active for genuinely fuseable arms (same variant type or no destructuring)
**Resolution:**

---

## [3] Syntax error on `#![doc(...)]` inner attribute — Issue [7] regression
**Status:** OPEN
**Files:** `src/analysis/checks/syntax.rs`
`#![doc(html_logo_url = "...")]` in rand's `src/lib.rs` triggers "Syntax error detected" (LAW OF INTEGRITY). Issue [7] added `is_known_unsupported_construct()` for inner attributes, but it's not catching `#![doc(...)]` — likely the recognizer only matches `#![allow` or `#![cfg` patterns.

Required behavior:
* The inner attribute recognizer must match ANY `#![...]` form, not just specific attribute names.
* Fix the pattern to check for `#![` prefix generically.
* Verify against: `#![doc(...)]`, `#![allow(...)]`, `#![cfg_attr(...)]`, `#![feature(...)]`, `#![warn(...)]`, `#![deny(...)]`.

Deliverables:
* Fix the recognizer pattern
* Add regression test for `#![doc(html_logo_url = "...")]`
* Verify existing `#![allow(...)]` test still passes
**Resolution:**

---

## [4] P01: Skip test functions and intentional iterator clone patterns
**Status:** OPEN
**Files:** `src/analysis/patterns/performance.rs`
P01 flags `iter.clone().choose(r)` inside test loops. The clone is intentional — consume-once iterator APIs require a fresh iterator each iteration to test sampling distributions. This produced 3 false positives on rand's test code.

Additionally, P01 flags structurally necessary clones on generic types (e.g., `self.cumulative_weights[i].clone()` where `W: Clone`) where the clone cannot be hoisted because the value changes each iteration. 4 more false positives on rand.

Required behavior:
* Skip P01 inside `#[test]` functions entirely, or at minimum inside `#[cfg(test)]` modules.
* For non-test code: recognize when a clone is on an indexed collection element inside a loop where the index varies — the clone is structurally necessary and cannot be hoisted.

Deliverables:
* Add `is_test_context()` check that walks ancestors for `#[test]` or `#[cfg(test)]`
* Add test: clone in `#[test]` fn — NOT flagged
* Add test: clone of `vec[i]` inside loop with varying `i` — NOT flagged (or reduced severity)
**Resolution:**

---

## [5] P06: Floyd's algorithm and small-collection linear search are false positives
**Status:** OPEN
**Files:** `src/analysis/patterns/performance.rs`
P06 flags `.position()` inside loops without considering whether the linear scan IS the algorithm (Floyd's sampling) or operates on a trivially small collection (4-element test arrays). 4 false positives on rand.

Required behavior:
* Skip P06 inside `#[test]` / `#[cfg(test)]` contexts (same as [4]).
* Consider adding a heuristic: if the collection being scanned is demonstrably small (literal array, or bounded by a small constant), suppress or downgrade.
* For algorithmic cases (Floyd's), there's no good automated fix — but at minimum, test code should not be flagged.

Deliverables:
* Share the `is_test_context()` implementation with [4]
* Add test: `.position()` in test loop — NOT flagged
**Resolution:**

---

## [6] I01: Suggesting derive_more to zero-dependency crates is tone-deaf
**Status:** OPEN
**Files:** `src/analysis/patterns/idiomatic.rs`
I01 suggests `derive_more::From` for manual `From` impls. This is actively harmful advice for foundational crates (rand, serde, tokio) that intentionally maintain zero proc-macro dependencies. 2 violations on rand.

Required behavior:
* Downgrade I01 from error to info/suggestion severity.
* OR: add context to the suggestion: "Consider `derive_more::From` if your crate already uses proc macros."
* OR: suppress I01 when the impl is for a core/std trait (From, Into, TryFrom) and the crate has no proc-macro dependencies in Cargo.toml.

The simplest fix is severity downgrade — I01 is a style preference, not a correctness issue.

Deliverables:
* Change I01 severity or add qualifier to suggestion text
* Verify rand's manual From impls no longer appear as errors
**Resolution:**

---

## [7] Smart output compression for neti-report.txt
**Status:** OPEN
**Files:** `src/verification/runner.rs`, potentially new `src/reporting/compression.rs`
Clippy and other tools can output 10,000 lines for what amounts to 2 distinct issues repeated across many call sites. AI agents waste context window on this noise. `neti-report.txt` should be maximally informative but succinct.

Approach:
* Group violations/errors by kind (error code + message template), not by occurrence
* For each kind: show the first 2-3 instances with full context, then a count of remaining occurrences with just file:line references
* Must work for ANY command output in the `[commands]` pipeline — not clippy-specific
* Write compression logic as a shared utility
**Resolution:**

---

## [8] Safety rule robustness: recognize SAFETY justifications and avoid penalizing well-documented unsafe
**Status:** OPEN
**Files:** `src/analysis/safety.rs`, tests
Requiring `// SAFETY:` is good, but the rule should be robust and ergonomic: it should recognize nearby justifications and avoid false negatives in common formatting patterns.

Required behavior:
* Accept `// SAFETY:` comments that are:
  * immediately above the `unsafe` block
  * on the same line
  * within a small window (e.g., within the preceding 2 lines) if separated by blank line or short comment
* If an unsafe block is inside a function documented with a `# Safety` section, allow an opt-in mode to satisfy the requirement.

Deliverables:
* Improve detection of SAFETY comment adjacency
* Add tests with common Rust idioms (including code formatted by rustfmt)
**Resolution:**

---

## [9] Locality integration into standard scan pipeline
**Status:** OPEN
**Files:** `src/graph/locality/` (all), `src/cli/locality.rs`, `src/config/locality.rs`
Locality has a massive subgraph but feels separate from the main scan pipeline.

Questions to resolve:
* Should locality be a first-class `neti check` phase or remain opt-in `neti locality`?
* Can locality violations be surfaced as regular scan violations (same format, same report)?

Recommendation: integrate locality results into the standard scan report format. Keep the graph engine as-is but unify the output.
**Resolution:**

---

## [10] Governance-grade clippy integration: separate "lint hygiene" from "merge blockers"
**Status:** OPEN
**Files:** `src/verification/runner.rs`, `src/cli/audit.rs`, `src/config/types.rs`, tests
Required behavior:
* Two tiers of command enforcement: **blockers** (compile/test failures) and **hygiene** (clippy pedantic as non-blocking)
* Commands marked as `mode = "error" | "warn"` in neti.toml
* Report clearly separates blocking vs non-blocking sections

Deliverables:
* Extend `[commands]` schema for severity/mode per command
* Add tests on runner behavior
**Resolution:**

---

## [11] Python and TypeScript pattern detection parity
**Status:** OPEN
**Files:** `src/analysis/patterns/` (all), `src/lang.rs`
Rust has full pattern detection. Python and TypeScript are marked "Partial" in the README. Close the gap on the most impactful patterns:
* Security patterns (injection, credential exposure)
* Performance patterns (allocation in loops, linear search in loops)
* Idiomatic patterns per language
**Resolution:**

---

## [12] Regression suite: prove scan correctness against real-world crates
**Status:** OPEN
**Files:** `tests/` (new), `src/analysis/*`, CI config
Once [1]-[5] land, lock down correctness with a regression harness against pinned real-world code (rand, Dioxus, or curated fixtures extracted from them).

The suite should assert:
* No syntax error false positives on valid Rust
* No L03 on fixed-size arrays
* No I02 on type-incompatible match arms
* No P01/P06 in test code
* No C03/X02 misclassifications

Deliverables:
* Add curated fixture files reproducing known false positives
* Add CI job running regression tests
**Resolution:**

---

### Priority order
1. **[1] L03 fixed-size arrays** — 13 false positives, single fix
2. **[2] I02 type-blind matching** — 9 false positives, single fix
3. **[3] `#![doc]` regression** — 1 false positive, trivial fix
4. **[4]+[5] Test context skipping** — 7 false positives, shared `is_test_context()` implementation
5. **[6] I01 severity** — 2 false positives, trivial
6. **[7]-[10] Infrastructure** — compression, safety, locality, clippy tiers
7. **[11] Language parity** — feature expansion
8. **[12] Regression suite** — lock it all down
