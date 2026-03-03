# ACTIVE Issues

---

## [8] Safety rule: recognize nearby SAFETY justifications
**Status:** OPEN
**Files:** `src/analysis/safety.rs`, tests
Current rule requires `// SAFETY:` immediately adjacent. Should recognize justifications within 3 lines above the `unsafe` block, or immediately inside the block header.

Add tests proving "nearby is OK" and "distant is not."
**Resolution:**

---

## [15] P04 false positives on 2D numeric iteration
**Status:** OPEN
**Files:** `src/analysis/patterns/performance_p04p06.rs`, tests
P04 fires on `for x in 0..w { for y in 0..h }` — intentional 2D iteration, not algorithmic inefficiency.

Required: If both loops use numeric range patterns, downgrade to Info or skip. Only flag nested loops where inner iterates a collection that suggests lookup optimization.
**Resolution:**

---

## [29] Wire `write_fix_packet` and `auto_copy` preferences
**Status:** OPEN
**Files:** `src/cli/handlers/mod.rs`, `src/config/types.rs`, `src/reporting.rs`
These preferences exist in config/UI but aren't implemented. Critical for AI loop workflows.

Wire into check failure path: write `format_report_string()` to configured path, optionally copy to clipboard.
**Resolution:**

---

## [30] Baseline + suppression system for staged adoption
**Status:** OPEN
**Files:** `src/config/types.rs`, `src/reporting.rs`, `src/cli/handlers/mod.rs`, `src/types.rs`
Every governance tool needs escape hatches for legacy codebases.

Required:
- `neti baseline` generates snapshot of current violations
- Future runs enforce "no regressions" until baseline updated
- Inline suppressions `// neti:allow(CODE) reason` with required reason text
- Suppressions visible in report output
**Resolution:**

---

## [31] Make branch workflow configurable
**Status:** OPEN
**Files:** `src/branch.rs`, `src/config/types.rs`
Hard-coded `neti-work`, `main`, and squash merge blocks teams with different conventions.

Add config: `work_branch_name`, `base_branch_name`, `merge_mode` (squash/merge/rebase), `commit_message_template`. Keep current behavior as default.
**Resolution:**

---

## [17] Define `LangSemantics` trait and Rust implementation
**Status:** OPEN
**Files:** `src/lang.rs`, `src/lang/semantics.rs` (new)
Create abstraction decoupling detectors from language-specific knowledge:
- Test markers (attributes, function prefixes, call expressions)
- Type vocabulary (heap types, lock types)
- Method vocabulary (alloc, linear search, length, clone)
- Syntax vocabulary (let node kinds, loop node kinds, param prefixes)

Populate Rust table with all values currently hardcoded. Add `Lang::semantics()` method.
**Resolution:**

---

## [18] Wire `LangSemantics` into performance detectors
**Status:** OPEN
**Files:** `src/analysis/patterns/performance.rs`, `src/analysis/patterns/performance_test_ctx.rs`
Replace hardcoded vocabulary with `LangSemantics` queries. Delete `should_skip()` path filter — fix heuristics instead of hiding findings.
**Resolution:**

---

## [19] Wire `LangSemantics` into logic detectors
**Status:** OPEN
**Files:** `src/analysis/patterns/logic.rs`, `src/analysis/patterns/logic_helpers.rs`
Replace hardcoded vocabulary. Keep `logic_proof.rs` as Rust-only precision enhancer, gated by `Lang::Rust` check.
**Resolution:**

---

## [20] Wire `LangSemantics` into remaining detectors
**Status:** OPEN
**Files:** `src/analysis/patterns/semantic.rs`, `src/analysis/patterns/concurrency.rs`, `src/analysis/patterns/concurrency_lock.rs`, `src/analysis/patterns/concurrency_sync.rs`
Move lock types, mutation receiver patterns behind `LangSemantics`.
**Resolution:**
