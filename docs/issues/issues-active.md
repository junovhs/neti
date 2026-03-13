# ACTIVE Issues

## Label Set

Use only these labels across active and backlog issues:
`Accuracy`, `Config`, `CLI`, `Reporting`, `AI Workflow`, `Adoption`, `Architecture`, `Cleanup`, `Language Support`, `Detection Rules`, `Testing`, `Performance`, `Safety`, `Branching`, `Web Stack`, `Integrations`

---

## [8] Safety rule: recognize nearby SAFETY justifications
**Status:** OPEN
**Files:** `src/analysis/safety.rs`, tests
**Labels:** Safety, Accuracy, Detection Rules, Testing
**Depends on:** none

**Problem:** The current rule requires a `// SAFETY:` comment to be immediately adjacent to an `unsafe` block. That is stricter than how humans actually document safety reasoning, and it creates false positives when the justification appears a few lines above the block or directly inside the block header.

**Fix:**

1. Accept `// SAFETY:` comments within 3 lines above the `unsafe` block.
2. Accept a justification immediately inside the block header when it clearly documents that block.
3. Keep rejecting distant or ambiguous comments.
4. Add tests proving "nearby is OK" and "distant is not."

**Resolution:**

---

## [15] P04 false positives on 2D numeric iteration
**Status:** OPEN
**Files:** `src/analysis/patterns/performance_p04p06.rs`, tests
**Labels:** Accuracy, Performance, Detection Rules, Testing
**Depends on:** none

**Problem:** P04 currently fires on patterns like `for x in 0..w { for y in 0..h }`, which are often intentional 2D numeric iteration rather than evidence of avoidable lookup inefficiency.

**Fix:**

1. Detect when both loops are simple numeric range iteration.
2. Skip the finding entirely, or downgrade it to `Info`, for that 2D numeric case.
3. Keep flagging nested loops where the inner loop iterates a collection that suggests a lookup optimization.
4. Add tests covering both the intentional 2D case and a true positive collection-iteration case.

**Resolution:**

---

## [29] Wire `write_fix_packet` and `auto_copy` preferences
**Status:** OPEN
**Files:** `src/cli/handlers/mod.rs`, `src/config/types.rs`, `src/reporting.rs`
**Labels:** AI Workflow, Reporting, Config, CLI
**Depends on:** none

**Problem:** `write_fix_packet` and `auto_copy` exist in config and UI surfaces but are not implemented in the actual check failure path. That makes the AI-fix loop look supported without delivering the behavior.

**Fix:**

1. On `neti check` failure, write `format_report_string()` output to the configured report path.
2. When `auto_copy` is enabled, copy the fix packet to the clipboard after generation.
3. Keep the behavior opt-in through config.
4. Verify the file output and clipboard path both work from the real failure flow.

**Resolution:**

---

## [30] Baseline + suppression system for staged adoption
**Status:** OPEN
**Files:** `src/config/types.rs`, `src/reporting.rs`, `src/cli/handlers/mod.rs`, `src/types.rs`
**Labels:** Adoption, Reporting, Config, CLI, Detection Rules
**Depends on:** none

**Problem:** Neti needs staged-adoption escape hatches for legacy repositories. Without a baseline and explicit suppressions, teams either absorb a large migration cost immediately or avoid adoption entirely.

**Fix:**

1. Add `neti baseline` to snapshot the current violation set.
2. Enforce "no regressions" on future runs until the baseline is intentionally refreshed.
3. Support inline suppressions using `// neti:allow(CODE) reason`.
4. Require a human-readable reason for every suppression.
5. Surface baseline and suppression effects clearly in report output.

**Resolution:**

---

## [31] Make branch workflow configurable
**Status:** OPEN
**Files:** `src/branch.rs`, `src/config/types.rs`
**Labels:** Branching, Config, CLI
**Depends on:** none

**Problem:** The branch workflow is hard-coded around `neti-work`, `main`, and squash merge semantics. That blocks teams whose branch naming, base branch, or merge policy differs.

**Fix:**

1. Add config for `work_branch_name`.
2. Add config for `base_branch_name`.
3. Add config for `merge_mode` with `squash`, `merge`, and `rebase`.
4. Add config for `commit_message_template`.
5. Preserve current behavior as the default when config is absent.

**Resolution:**

---

## [17] Define `LangSemantics` trait and Rust implementation
**Status:** OPEN
**Files:** `src/lang.rs`, `src/lang/semantics.rs` (new)
**Labels:** Architecture, Language Support, Detection Rules
**Depends on:** none

**Problem:** Detector logic still hardcodes language-specific vocabulary directly in rule implementations. That couples the detection engine to Rust-specific details and makes multi-language support brittle.

**Fix:**

1. Create a `LangSemantics` abstraction that exposes language-specific knowledge through one interface.
2. Cover test markers, type vocabulary, method vocabulary, and syntax vocabulary.
3. Populate a Rust semantics table with the values currently hardcoded across detectors.
4. Add `Lang::semantics()` so detectors can query the abstraction.

**Resolution:**

---

## [18] Wire `LangSemantics` into performance detectors
**Status:** OPEN
**Files:** `src/analysis/patterns/performance.rs`, `src/analysis/patterns/performance_test_ctx.rs`
**Labels:** Architecture, Language Support, Detection Rules, Performance
**Depends on:** [17]

**Problem:** Performance detectors still embed language-specific vocabulary and path-based exceptions. That makes the rules less portable and hides precision problems behind `should_skip()`.

**Fix:**

1. Replace hardcoded vocabulary lookups with `LangSemantics` queries.
2. Remove the `should_skip()` path filter.
3. Tighten heuristics so precision comes from the rule logic rather than file-path exclusion.
4. Update or extend tests to prove the detectors still behave correctly after the refactor.

**Resolution:**

---

## [19] Wire `LangSemantics` into logic detectors
**Status:** OPEN
**Files:** `src/analysis/patterns/logic.rs`, `src/analysis/patterns/logic_helpers.rs`
**Labels:** Architecture, Language Support, Detection Rules
**Depends on:** [17]

**Problem:** Logic detectors also hardcode language-specific terms and syntax assumptions, which prevents the rule family from scaling cleanly beyond Rust.

**Fix:**

1. Replace hardcoded vocabulary with `LangSemantics` queries.
2. Keep `logic_proof.rs` as a Rust-only precision enhancer.
3. Gate Rust-only proof logic behind an explicit `Lang::Rust` check.
4. Update tests to ensure the refactor preserves existing Rust precision.

**Resolution:**

---

## [20] Wire `LangSemantics` into remaining detectors
**Status:** OPEN
**Files:** `src/analysis/patterns/semantic.rs`, `src/analysis/patterns/concurrency.rs`, `src/analysis/patterns/concurrency_lock.rs`, `src/analysis/patterns/concurrency_sync.rs`
**Labels:** Architecture, Language Support, Detection Rules, Safety
**Depends on:** [17]

**Problem:** The remaining detector families still keep lock-type and mutation-pattern knowledge inline, leaving the language abstraction incomplete.

**Fix:**

1. Move lock types behind `LangSemantics`.
2. Move mutation receiver patterns behind `LangSemantics`.
3. Remove remaining detector-local Rust vocabulary where the abstraction can own it.
4. Extend tests for the affected detector families.

**Resolution:**
