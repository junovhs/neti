# issues: Post-Launch Work
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

# CRITICAL — Contract Bugs & Correctness

## [26] `neti check --json` must emit JSON to stdout
**Status:** DONE
**Files:** `src/cli/handlers/mod.rs`, `src/types.rs`, `src/reporting.rs`
The `--json` flag exists but `handle_check` writes text to `neti-report.txt` instead of emitting JSON to stdout. This breaks CI integrations and agent automation.

Required: Output a coherent `CheckReport` JSON payload containing scan report, command results, overall pass/fail, and exit code. Keep `neti-report.txt` behavior unchanged for non-JSON mode.
**Resolution:** handle_check now constructs a CheckReport containing ScanReport, converted command results, and overall pass/fail, then emits it via reporting::print_json() to stdout when --json is passed. neti-report.txt is still written in both modes. Extracted shared helpers (build_report_text, convert_commands, append_command_result) and split handle_check into handle_check_json/handle_check_interactive to eliminate duplicated report-building code. Five integration tests in tests/check_json_test.rs verify JSON structure, required fields, type correctness, and exit code consistency — running in isolated temp directories with empty [commands] to prevent recursive cargo test invocation.

---

## [27] Unify duplicate `CommandResult` types
**Status:** DONE
**Files:** `src/types.rs`, `src/verification/mod.rs`, `src/verification/runner.rs`
Two different `CommandResult` structs exist: `types::CommandResult` (stdout/stderr separated, `exit_code: i32`) and `verification::CommandResult` (combined output, `exit_code: Option<i32>`, plus `passed`).

Consolidate to one canonical type with: `argv`, `exit_code: i32`, `stdout`, `stderr`, `duration_ms`, `passed`. Remove status inference from formatted output text.
**Resolution:** Consolidated two duplicate CommandResult types into one canonical type in src/types.rs with fields: command, passed, exit_code: i32, stdout, stderr, duration_ms. The passed field is now derived from exit_code == 0 at construction time, not inferred from parsing output text. src/verification/mod.rs now re-exports crate::types::CommandResult instead of defining its own. src/verification/runner.rs captures stdout/stderr separately at execution time and gets exit code directly from output.status.code(). Removed the convert_commands() bridge function from src/cli/handlers/mod.rs — the unified type flows through directly. Removed orphaned output field from VerificationReport. Six integration tests verify field presence, type correctness, pass/fail derivation, stdout capture, and duration semantics.

---

## [28] Command execution uses naive `split_whitespace`
**Status:** DONE
**Files:** `src/verification/runner.rs`, `src/config/types.rs`
Commands with quoted arguments or spaces break. `run_single_command()` uses `split_whitespace()` which cannot handle `cargo clippy -- -D "some flag"`.

Fix: Use shell-words parser, or support structured TOML form `argv = ["cargo", "clippy", "--all-targets"]`. Store exit code as data at execution time — don't parse it from output text.
**Resolution:** Added shell-words = "1.1" dependency and replaced naive split_whitespace() in run_single_command() with shell_words::split(), which implements POSIX shell-style word splitting (double quotes, single quotes, backslash escapes). Unclosed quotes now return a clear parse error with exit code -1 instead of silently mangling arguments. Commands like cargo clippy -- -D "some flag" are now parsed correctly. The existing CommandEntry::List TOML form continues to work unchanged. Split src/types.rs into src/types/mod.rs + src/types/command.rs to resolve a token-count governance violation. 15 unit tests in src/verification/runner.rs cover shell parsing edge cases (quoting styles, empty input, parse errors, pipeline behavior). 6 integration tests in tests/command_parsing_test.rs verify quoted commands end-to-end through the compiled binary via neti check --json.

---

## [9] Locality integration into `neti check` pipeline
**Status:** OPEN
**Files:** `src/graph/locality/mod.rs`, `src/cli/locality.rs`, `src/cli/handlers/mod.rs`, `src/config/locality.rs`, `src/reporting.rs`
`check_locality_silent()` exists "for pipeline use" but `handle_check` doesn't call it. The `[rules.locality]` config exists but isn't enforced.

Required: Make locality a first-class stage in `neti check` gated by config mode (`off`/`warn`/`error`). Include locality results in JSON output and `neti-report.txt`.
**Resolution:**

---

# HIGH PRIORITY — Precision & Adoption

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

# MEDIUM PRIORITY — Infrastructure & Polish

## [32] Config hygiene: audit and wire preferences
**Status:** OPEN
**Files:** `src/config/types.rs`, `src/cli/handlers/mod.rs`
Preferences like `auto_promote`, `progress_bars` exist but may not be fully wired. Audit each preference — either implement it with observable behavior and test, or remove to avoid misleading UX.
**Resolution:**

---

## [33] Remove or expose `cli/audit.rs`
**Status:** OPEN
**Files:** `src/cli/audit.rs`, `src/cli/args.rs`
Module exists but `Audit` isn't in `Commands` enum. Either wire it up or delete until ready.
**Resolution:**

---

## [10] Governance-grade clippy integration
**Status:** OPEN
**Files:** `src/verification/runner.rs`, `src/config/types.rs`, tests
Make clippy a first-class governance stage with tiering (warn vs fail) via config and stable parsing (not brittle string matching).
**Resolution:**

---

## [24] Root src/ cleanup and domain consolidation
**Status:** OPEN
**Files:** `src/discovery.rs`, `src/file_class.rs`, `src/project.rs`, `src/detection.rs`, `src/constants.rs`, `src/reporting.rs`, `src/lib.rs`
Consolidate filesystem/project discovery into a `workspace` module. Move `src/reporting.rs` to `src/reporting/mod.rs` for consistency. Reduce top-level sprawl.
**Resolution:**

---

## [34] Add `neti rules` catalog command
**Status:** OPEN
**Files:** `src/cli/args.rs`, `src/reporting/guidance.rs`
List all rule codes with severity, confidence semantics, thresholds, fix guidance, and suppression syntax. Searchable in-product catalog reduces friction.
**Resolution:**

---

## [35] Add SARIF output format
**Status:** OPEN
**Files:** `src/reporting/mod.rs`, `src/cli/args.rs`
SARIF enables GitHub/GitLab PR annotations. Map rule code → ruleId, confidence → level, file/line → region. Keep JSON as canonical; SARIF is derived.
**Resolution:**

---

## [36] Add `neti init` scaffolding command
**Status:** OPEN
**Files:** `src/cli/args.rs`, `src/project.rs`
Generate `neti.toml` + `.netiignore` based on detected project type. Support `--strict` vs `--lenient` presets. Optionally generate `CHAT-PROTOCOL.md`.
**Resolution:**

---

## [37] LCOM4 miscalibrated for delegation patterns
**Status:** OPEN
**Files:** `src/analysis/structural.rs`, `src/config/mod.rs`
The "Touch fields for LCOM4" hacks indicate the metric is too sensitive to pure delegators. Either exempt config/CLI modules, adjust computation for delegation patterns, or add `// neti:allow(LCOM4)` mechanism.
**Resolution:**

---

# LANGUAGE PARITY — Phase 1 (Abstraction)

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

---

# LANGUAGE PARITY — Phase 2 (Tables)

## [21] Python `LangSemantics` table
**Status:** OPEN
**Files:** `src/lang/semantics.rs`
Add Python semantics: `test_` prefix, `list`/`dict`/`set` heap types, `index`/`find` search methods, `len` length method, `for_statement`/`while_statement` loops.
**Resolution:**

---

## [22] TypeScript `LangSemantics` table
**Status:** OPEN
**Files:** `src/lang/semantics.rs`
Add TypeScript semantics: `describe`/`it`/`test` markers, `Array`/`Map`/`Set` heap types, `find`/`indexOf` search methods, `length` property.
**Resolution:**

---

## [11] Python and TypeScript pattern detection parity
**Status:** DESCOPED
**Files:** N/A
Superseded by [17]-[22] which break this into concrete implementation steps.
**Resolution:** Descoped — covered by LangSemantics issues [17]-[22].

---

# VALIDATION

## [12] Cross-language regression suite
**Status:** OPEN
**Files:** `tests/` (new), CI config
Create test fixtures:
- `tests/fixtures/rust/` — syntax suppressions, L03 tiers, P04 numeric iteration
- `tests/fixtures/python/` — equivalent patterns once [21] lands
- `tests/fixtures/typescript/` — equivalent patterns once [22] lands

Same rules should fire on equivalent patterns across languages.
**Resolution:**

---

# COMPLETED

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
**Resolution:** Added `Confidence` enum (High/Medium/Info) to `Violation`. Updated reporter to group by rule, deduplicate with back-references, and show static educational content (WHY/FIX/SUPPRESS) on first occurrence. Wired correct confidence tiers into all 8 pattern detector files.

---

## [7] Heuristic sharpening: Default to Medium confidence for untrackable variables
**Status:** DONE
**Files:** `src/analysis/patterns/logic.rs`, `src/analysis/patterns/performance.rs`
**Resolution:** Split logic.rs into `logic.rs` + `logic_helpers.rs` + `logic_proof.rs`. Split performance.rs into `performance.rs` + `performance_test_ctx.rs`. Implemented three-state L03 confidence classification. Implemented P01 confidence split. Fixed `check_p04` to walk descendants. Added 15+ mutant-killing tests.

---

## [13] Syntax false positives on `&raw const` / `&raw mut` (Rust 1.82+)
**Status:** DONE
**Files:** `src/analysis/checks/syntax.rs`
**Resolution:** Added `is_raw_pointer_syntax()` with three detection strategies. Wired into `is_known_unsupported_construct`. lazuli syntax errors: 42 → 0.

---

## [14] L03 volume: deduplicate `self.field[0]` indexing
**Status:** DONE
**Files:** `src/analysis/patterns/logic_l03.rs`
**Resolution:** Added `seen_self_fields: HashSet<String>` threaded through `detect_index_zero`. First occurrence reported; subsequent sites suppressed. lazuli L03: 110 → ~10-15.

---

## [16] HOTFIX: `neti-report.txt` regression and pipeline integration
**Status:** DONE
**Files:** `src/cli/handlers/mod.rs`, `src/cli/handlers/scan_report.rs`, `src/reporting.rs`
**Resolution:** Fixed regression where `neti check` overwrote report with only external linter output. Now runs `neti scan` first, then external commands, appends both to report.

---

## [23] HOTFIX: Governance compliance (Atomicity, Coupling, Cohesion)
**Status:** DONE
**Files:** `src/analysis/patterns/*`, `src/cli/handlers/mod.rs`, `src/analysis/engine.rs`, `src/verification/mod.rs`, `src/graph/rank/builder.rs`
**Resolution:** Split test suites into `_test.rs` modules. Refactored `Engine` into free functions. Fixed `CommandResult` encapsulation. Fixed P02/P04 looping violations. Codebase 100% green.

---

## [25] HOTFIX: Tree-sitter v0.23 migration and self-governance compliance
**Status:** DONE
**Files:** `src/analysis/patterns/mod.rs`, `src/analysis/checks/syntax_test.rs`, `src/analysis/ast.rs`, `src/analysis/extract.rs`, `src/analysis/extract_impl.rs`, `src/analysis/patterns/idiomatic_i02.rs`, `src/analysis/patterns/logic_helpers.rs`, `src/analysis/patterns/logic_proof_helpers.rs`, `src/analysis/patterns/performance.rs`, `src/analysis/patterns/semantic.rs`, `src/analysis/structural.rs`, `src/analysis/checks/naming.rs`, `src/cli/handlers/scan_report.rs`, `src/lang.rs`, `src/graph/defs/queries.rs`, `src/graph/defs/extract.rs`
**Resolution:**
1. Tree-sitter API migration: Added `&` before `lang.grammar()` calls
2. P02 fixes (8 sites): Hoisted `.to_string()` out of loops
3. P04 fixes (8 sites): Eliminated nested loops via `flat_map()`, pre-computed pairs, `extend()`
4. P06 allow (1): `clean.find(':')` — char search on single string
5. P03 allow (1): `Lang::Swift.query(kind)` — array index lookup
6. Swift ABI: Made `DefExtractor::get_config()` return `Option`, tests skip gracefully

Final state: 0 errors, 0 warnings, 149 tests passing. Only 2 legitimate `neti:allow` suppressions with documented justification.

---

# CALIBRATION DATA

### rand v0.10.0

| Round | Errors | Warnings | Suggestions | Total |
|-------|--------|----------|-------------|-------|
| Initial | 57 | 0 | 0 | 57 |
| Config tuning | 38 | 0 | 0 | 38 |
| False positive fixes | 19 | 0 | 0 | 19 |
| Test skipping | 14 | 0 | 0 | 14 |
| Confidence Tiers | 6 | 6 | 2 | 14 |
| Heuristic Sharpening | 4 | 17 | 2 | 23 |

### lazuli emulator

| Category | Count | Assessment |
|----------|-------|------------|
| LAW OF PARANOIA | 43 | Actionable |
| LAW OF INTEGRITY | 0 | Fixed by [13] |
| L03 | ~12 | Fixed by [14] |
| P04 | 34 | Noisy — [15] pending |
| I02 | 28 | Real but low-priority |
| Structural | ~80 | Expected for domain |
| M03/M04 | 12 | Actionable |
| P02/R07 | 9 | Actionable |
| Complexity | 10 | Actionable |
| Atomicity | 17 | Real |
