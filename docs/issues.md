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
**Resolution:** Added `is_fixed_size_array_access()` with three detection strategies: local variable declarations, struct fields via `self.field`, function parameters with array type annotation. Six new tests. Reduced rand L03 from 13 to 4 (remaining require cross-scope type tracking — tree-sitter floor).

---

## [2] I02: Type-blind duplicate arm detection flags unfuseable match arms
**Status:** DONE
**Files:** `src/analysis/patterns/idiomatic.rs`
**Resolution:** Rewrote `patterns_have_incompatible_types()` with `extract_variant_names()` handling both simple and tuple patterns. Also rewrote I01 to use text-based impl header matching. All 9 false positives on rand eliminated.

---

## [3] Syntax error on `#![doc(...)]` inner attribute
**Status:** DONE
**Files:** `src/analysis/checks/syntax.rs`
**Resolution:** Added `is_inside_inner_attribute()` which walks ancestor chain. Tree-sitter parses outer `#![doc(` but errors on interior content — ancestor walk catches this.

---

## [4] P01/P06: Skip test functions and intentional patterns
**Status:** DONE
**Files:** `src/analysis/patterns/performance.rs`
**Resolution:** Added `is_test_context()` walking ancestors for `#[test]` and `#[cfg(test)]`. P01 and P06 skipped in test context. Five new tests.

---

## [5] I01: Suggesting derive_more to zero-dependency crates
**Status:** DONE
**Files:** `src/analysis/patterns/idiomatic.rs`
**Resolution:** Changed I01 message to suggestive framing with "consider if already using proc macros" and "otherwise this is fine as-is".

---

## [6] Confidence tiers: violations must declare what Neti knows vs what it guesses
**Status:** OPEN
**Files:** `src/types.rs`, `src/reporting.rs`, all pattern detectors in `src/analysis/patterns/`, `src/analysis/checks/`, `src/analysis/safety.rs`

### Problem

Every violation currently says "Action required" with identical severity. On rand, 1 of 14 findings is a real error and 13 are noise the tool can't resolve. This makes Neti look untrustworthy. The tool must be honest about what it can prove vs what it's guessing.

### Design

#### Confidence enum

```rust
pub enum Confidence {
    /// Neti can prove this is wrong. Structural violation, missing required
    /// annotation, provable bounds error.
    High,
    /// Neti sees a suspicious pattern but cannot prove it's wrong. May require
    /// type information, algorithmic intent, or cross-scope analysis that
    /// tree-sitter cannot provide.
    Medium,
    /// Style observation. Not wrong, but could be improved. Take it or leave it.
    Info,
}
```

Add `confidence: Confidence` field to `Violation`. Default to `High` for backward compatibility — each detector explicitly downgrades where appropriate.

#### Confidence assignments per rule

**HIGH — Neti can prove it:**
| Rule | Condition |
|------|-----------|
| LAW OF INTEGRITY | Syntax error on unparseable code |
| LAW OF PARANOIA | Missing `// SAFETY:` comment on unsafe block |
| LAW OF ATOMICITY | File exceeds token limit (objective measurement) |
| L02 | `<=`/`>=` with `.len()` in dangerous direction |
| L03 | `[0]` on slice/Vec with no guard (Neti proved it's NOT a fixed array) |
| I02 | Duplicate match arms with compatible types (Neti proved they CAN fuse) |
| X01 | SQL string formatting (format!/concat with SQL keywords) |
| X02 (shell) | Shell invocation with dynamic argument |
| C03 (sync) | Sync mutex guard held across `.await` |
| M03/M04/M05 | Name-behavior mismatch (getter with mut, is_ returning String) |
| R07 | Writer dropped without flush |

**MEDIUM — pattern match, can't prove:**
| Rule | Condition | Why medium |
|------|-----------|------------|
| P01 | Clone in loop on generic/ambiguous receiver | Can't resolve generic type params |
| P01 | Clone in loop on `self.field[i]` | Can't determine if type is cheap |
| P06 | `.find()`/`.position()` in loop | Might be intentional algorithm |
| P04 | Nested loop | Might be bounded/intentional |
| L03 | `[0]` where declaration isn't visible | Can't trace through method returns |
| C03 (async) | Async mutex guard across `.await` | HOL blocking, not deadlock |
| X02 (provenance) | `Command::new(variable)` without shell | Risk is PATH, not injection |
| CBO | High coupling metric | Accurate measurement, context-dependent |
| LCOM4 | Low cohesion metric | Accurate measurement, may be intentional |
| S01/S02/S03 | State patterns | Heuristic detection |

**INFO — style suggestion:**
| Rule | Condition |
|------|-----------|
| I01 | Manual `From` impl |
| Naming | Function name word count |
| SFOUT | High fan-out (below error threshold) |

#### Reporter formatting

**HIGH:**
```
error: Unsafe block missing justification
  --> src/distr/slice.rs:100
   = LAW OF PARANOIA: Fix required
```

**MEDIUM:**
```
warn: Detected `.clone()` inside a loop
  --> src/distr/weighted/weighted_index.rs:191
   = P01: Review recommended — Neti cannot determine clone cost for generic types
```

**INFO:**
```
info: Manual `From` impl — consider derive_more if already using proc macros
  --> src/seq/index.rs:131
   = I01: Style suggestion
```

**Summary line:**
```
Neti found 1 error, 9 warnings, 2 suggestions (423ms)
```

#### Exit code behavior

- Any HIGH violation → exit 1 (blocks CI)
- Only MEDIUM/INFO → exit 0 (does not block CI)
- Configurable: `[rules] medium_blocks = true` to make MEDIUM also block

### Educational suggestion system (Layer architecture)

#### Layer 1: Rule explanation (language-agnostic, static)

Each rule has a fixed `why` and `fix` that applies regardless of language:

```rust
struct RuleGuidance {
    why: &'static str,
    fix: &'static str,
    suppress: &'static str, // templated: "// neti:allow({rule})" or "{rule} = \"warn\"" in neti.toml
}
```

Examples:
```
P01:
  why: "Cloning/copying inside a loop allocates on every iteration, scaling linearly with iteration count."
  fix: "Hoist the allocation before the loop, use a reference or borrow, or confirm the copy is cheap (primitives, small structs, reference-counted pointers)."

P06:
  why: "Linear search (.find/.position/.index) inside a loop produces O(n·m) complexity."
  fix: "Pre-build a lookup structure (HashSet/HashMap/dict/Set) for O(1) access, or confirm the inner collection is bounded to a small constant size."

L03:
  why: "Indexing without a bounds proof panics on empty or undersized collections."
  fix: "Use safe accessors (.first()/.get()), add an emptiness check, or prove the collection size is guaranteed by construction."

X01:
  why: "Building SQL from string formatting allows injection when inputs are user-controlled."
  fix: "Use parameterized queries (? placeholders) with your database driver's bind API."

C03:
  why: "Holding a lock guard across an await point blocks the executor (sync) or starves other tasks (async)."
  fix: "Scope the guard so it drops before the await, or extract the critical section into a synchronous helper."

I01:
  why: "Manual From implementations are boilerplate that can be generated."
  fix: "Use derive_more::From if your project already depends on proc macros. Manual impls are fine for zero-dependency crates."
```

Budget: ~15 rules × ~50 words each = ~750 words. Well within 10k token budget.

#### Layer 2: Detector-computed context (dynamic, already exists)

These are the `analysis` strings each detector already produces:
- "Receiver `self.cumulative_weights[i]` appears to be a heap-owning type."
- "Index varies per iteration — clone cannot be hoisted."
- "Each outer iteration performs a full inner scan."

No change needed. These stay as-is.

#### Layer 3: Confidence framing (automatic from tier)

- HIGH: "Fix required"
- MEDIUM: "Review recommended — {reason Neti can't prove it}"
- INFO: "Style suggestion"

The `{reason}` comes from a `confidence_reason: Option<String>` field on the violation, set by the detector. Examples:
- "Neti cannot resolve generic type parameters"
- "Linear scan may be intentional (bounded algorithm)"
- "Cannot trace type through method return"

#### Layer 4: Suppression (one template, all rules)

```
To suppress: // neti:allow({rule}) on the line, or {rule} = "warn" in neti.toml [rules]
```

Same for every rule, every language.

### Deduplication in reporter

When multiple violations share the same rule AND the same educational content (layers 1+4):

- **First occurrence:** Full output — all four layers
- **Subsequent:** Compact — just the location, layer 2 (specific facts), and "[N of M] — see first {rule} above"

Example output:
```
warn: Detected `.clone()` inside a loop [1 of 4]
  --> src/distr/weighted/weighted_index.rs:191
   = P01: Review recommended — Neti cannot determine clone cost for generic types
   |
   = ANALYSIS:
   |   Receiver `self.cumulative_weights[i]` appears to be a heap-owning type.
   |   Clone inside a loop allocates on every iteration.
   |
   = WHY: Cloning/copying inside a loop allocates on every iteration,
   |  scaling linearly with iteration count.
   |
   = FIX: Hoist the allocation before the loop, use a reference or borrow,
   |  or confirm the copy is cheap (primitives, small structs, ref-counted pointers).
   |
   = SUPPRESS: // neti:allow(P01) on the line, or P01 = "warn" in neti.toml

warn: Detected `.clone()` inside a loop [2 of 4]
  --> src/distr/weighted/weighted_index.rs:193
   = P01: Receiver `self.total_weight` — see first P01 above

warn: Detected `.clone()` inside a loop [3 of 4]
  --> src/distr/weighted/weighted_index.rs:226
   = P01: Receiver `self.cumulative_weights[i]` — see first P01 above

warn: Detected `.clone()` inside a loop [4 of 4]
  --> src/distr/weighted/weighted_index.rs:231
   = P01: Receiver `cumulative_weight` — see first P01 above
```

### Implementation order

1. **Add `Confidence` enum to `src/types.rs`** — add field to `Violation`, default High
2. **Set confidence in each detector** — go through every pattern detector and assign correct tier per the table above
3. **Update reporter** — format by tier (error/warn/info), change summary line, adjust exit code logic
4. **Add rule guidance registry** — static `RuleGuidance` per rule with `why`/`fix`/`suppress`
5. **Wire guidance into reporter** — show on first occurrence of each rule
6. **Add deduplication** — group same-rule violations, compact format for 2nd+

Steps 1-3 are the structural change. Steps 4-6 are the polish.

### What this looks like on rand after implementation

```
SCAN 56 files │ 124790 tokens

error: Unsafe block missing justification
  --> src/distr/slice.rs:100
   = LAW OF PARANOIA: Fix required
   |
   = ANALYSIS: unsafe block has no // SAFETY: comment
   = WHY: Unsafe blocks must document their safety invariants...
   = FIX: Add a // SAFETY: comment explaining why the invariants hold.
   = SUPPRESS: // neti:allow(paranoia) or paranoia = "warn" in neti.toml

warn: Detected `.clone()` inside a loop [1 of 4]
  --> src/distr/weighted/weighted_index.rs:191
   = P01: Review recommended — cannot determine clone cost for generic types
   = [full educational block]

warn: Detected `.clone()` inside a loop [2-4 of 4]
  --> weighted_index.rs:193, weighted_index.rs:226, weighted_index.rs:231
   = P01: see first P01 above

warn: Linear search inside loop [1 of 2]
  --> src/seq/mod.rs:76
   = P06: Review recommended — linear scan may be intentional algorithm
   = [full educational block]

warn: Linear search inside loop [2 of 2]
  --> src/seq/index.rs:452
   = P06: see first P06 above

warn: Index `[0]` without bounds check [1 of 4]
  --> src/rngs/xoshiro128plusplus.rs:57
   = L03: Review recommended — cannot trace type through method return
   = [full educational block]

warn: Index `[0]` without bounds check [2-4 of 4]
  --> shuffle.rs:29, shuffle.rs:45, shuffle.rs:56
   = L03: see first L03 above

warn: Class 'IndexVec' is tightly coupled (CBO: 16)
  --> src/seq/index.rs:34
   = CBO: Review recommended — accurate measurement, may be expected for core types

info: Manual `From` impl [1 of 2]
  --> src/seq/index.rs:131
   = I01: Style suggestion
   = [full educational block]

info: Manual `From` impl [2 of 2]
  --> src/seq/index.rs:139
   = I01: see first I01 above

Neti found 1 error, 11 warnings, 2 suggestions (423ms)
```

That's 14 violations compressed into a scannable report that an AI agent or developer can triage in seconds. The one real error jumps out. The warnings are honest about their limits. The suggestions don't pretend to be demands.

---

## [7] Safety rule robustness: recognize SAFETY justifications
**Status:** OPEN
**Files:** `src/analysis/safety.rs`, tests
Requiring `// SAFETY:` is good, but the rule should recognize nearby justifications.
**Resolution:**

---

## [8] Locality integration into standard scan pipeline
**Status:** OPEN
**Files:** `src/graph/locality/` (all), `src/cli/locality.rs`, `src/config/locality.rs`
**Resolution:**

---

## [9] Governance-grade clippy integration
**Status:** OPEN
**Files:** `src/verification/runner.rs`, `src/cli/audit.rs`, `src/config/types.rs`, tests
**Resolution:**

---

## [10] Python and TypeScript pattern detection parity
**Status:** OPEN
**Files:** `src/analysis/patterns/` (all), `src/lang.rs`
**Resolution:**

---

## [11] Regression suite
**Status:** OPEN
**Files:** `tests/` (new), `src/analysis/*`, CI config
**Resolution:**

---

### Calibration results

**rand crate (v0.10.0) — final: 14 violations (from 57 initial)**

| Round | Violations | What changed |
|-------|-----------|--------------|
| 0 | 57 | Initial scan with default config |
| 1 | 38 | Config tuning (token limit 5000, LCOM4 3) |
| 2 | 19 | L03 fixed arrays, I02 type-blind, `#![doc]` syntax |
| 3 | 14 | Test context skipping (P01/P06), I01 softened |

**Remaining 14 breakdown:**
* 1 real finding (missing SAFETY comment) → HIGH
* 4 need type inference (generic clone) → MEDIUM
* 2 need algorithmic intent (Floyd's sampling) → MEDIUM
* 2 style observations (I01) → INFO
* 1 accurate metric (CBO) → MEDIUM
* 4 cross-scope proof needed (L03) → MEDIUM

**Floor:** 13 of 14 remaining violations are at the limit of tree-sitter analysis without type information. Going lower requires rustc/rust-analyzer integration or per-project suppression.

### Priority order
1. **[6] Confidence tiers** — structural foundation for everything else
2. **[7] Safety rule robustness** — small scope, pure correctness
3. **[8]-[9] Infrastructure** — locality, clippy tiers
4. **[10] Language parity** — feature expansion
5. **[11] Regression suite** — lock it all down
