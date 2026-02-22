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
**Resolution:** Added `is_test_context()` walking ancestors for `#[test]` and `#[cfg(test)]`. P01/P06 skipped in test code.

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
**Status:** OPEN
**Files:** `src/analysis/patterns/logic.rs`, `src/analysis/patterns/performance.rs`
The rand scan revealed that P01 and L03 still default to `High` (Error) when they simply *can't find* a variable declaration. This leaves some noise classified as Errors.

Required behavior:
* **L03:** If `is_fixed_size_array_access` fails because it cannot find the declaration at all (e.g. variable defined in a different scope/closure), default to **Medium** confidence ("Cannot verify variable origin"). Only stay High if we *find* the declaration and confirm it is a Slice/Vec.
* **P01:** If `looks_heap_owning` is true but we cannot find the declaration to confirm the type, default to **Medium** confidence ("Type inference incomplete"). Only stay High for known heap types (`String`, `Vec`) or clearly resolved structs.

Deliverables:
* Update `logic.rs` to distinguish "found and unsafe" vs "not found"
* Update `performance.rs` to downgrade confidence for local variables without clear type resolution
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

### Calibration results (rand v0.10.0)

| Round | Errors | Warnings | Suggestions | Total |
|-------|--------|----------|-------------|-------|
| Initial | 57 | 0 | 0 | 57 |
| Config tuning | 38 | 0 | 0 | 38 |
| False positive fixes | 19 | 0 | 0 | 19 |
| Test skipping | 14 | 0 | 0 | 14 |
| **Confidence Tiers** | **6** | **6** | **2** | **14** |

**Remaining 6 Errors breakdown:**
* 1 Real Safety issue (`slice.rs`)
* 1 P01 local variable clone (`cumulative_weight`) — needs Heuristic Sharpening [7]
* 4 L03 indexing (`xoshiro`, `bench`) — needs Heuristic Sharpening [7]

**Target state after [7]:** 1 Error (the real one), 11 Warnings, 2 Suggestions.

### Priority order
1. **[7] Heuristic sharpening** — perfects the rand output (1 real error)
2. **[8] Safety rule robustness** — small correctness fix
3. **[9]-[10] Infrastructure** — locality, clippy tiers
4. **[11] Language parity**
5. **[12] Regression suite**
