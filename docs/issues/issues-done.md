# DONE Issues

---

## [50] Extract shared `omni-ast` crate from SEMMAP
**Status:** DONE
**Files:** `Cargo.toml`, `Cargo.lock`, `src/lib.rs`, `src/lang.rs`, `src/graph/imports.rs`, `omni-ast/Cargo.toml`, `omni-ast/src/lib.rs`, `omni-ast/src/harvester.rs`, `omni-ast/src/harvester_tree.rs`, `omni-ast/src/harvester_signatures.rs`, `omni-ast/src/taxonomy.rs`, `omni-ast/src/taxonomy_rules.rs`, `omni-ast/src/semantics.rs`, `omni-ast/src/types.rs`, `omni-ast/src/doc_extractor.rs`, `omni-ast/src/doc_filter.rs`, `omni-ast/src/language/`, `omni-ast/src/swum/`
**Labels:** Architecture, Language Support, Integrations
**Depends on:** none

**Problem:** NETI needed a shared multi-language analysis crate built from SEMMAP's reusable core so future rule work could stop depending on SEMMAP-internal modules and start targeting a publishable shared engine.

**Fix:**

1. Create a standalone shared crate, `omni-ast`, suitable for publication on crates.io.
2. Port SEMMAP Stage 1 harvesting (`SemanticFingerprint`) into the shared crate.
3. Port SEMMAP taxonomy evaluation (`SemanticBadges` and taxonomy rules) into the shared crate.
4. Port the SEMMAP language modules for Rust, Go, Python, C++, and JS/TS into the shared crate.
5. Port the SEMMAP SWUM engine into the shared crate.
6. Integrate the crate into at least one live NETI path so the extraction is proven in production code rather than parked in the workspace.

**Resolution:** Extracted a new workspace crate, `omni-ast`, and moved SEMMAP's shared analysis core into it under NETI's `tree-sitter 0.23` stack. The crate now contains the Stage 1 harvester, taxonomy engine and rules, SWUM engine, documentation extractors, shared dependency types, and language modules for Rust, Go, Python, JS/TS, and C++. I chose this boundary because it isolates reusable AST harvesting and semantic enrichment logic from SEMMAP-specific rendering concerns while giving NETI a publishable shared engine. NETI now depends on and re-exports `omni-ast`, uses `SemanticLanguage` in `src/lang.rs`, and consumes `omni-ast` import extraction in `src/graph/imports.rs`, which proves the shared crate is live in the production path rather than just parked in the workspace. Added extraction and regression coverage in `omni-ast` for harvesting, taxonomy behavior, cross-language import extraction, JS monorepo resolution, Python relative imports, and C++ symbol/doc extraction. Verified with `cargo test -p omni-ast` (9 tests passed) and `neti check`; verification commands passed and the only remaining `neti check` finding was the pre-existing LCOM4 warning in `src/types/command.rs`. Follow-on adoption work, where NETI becomes primarily driven by shared semantic queries rather than Rust-specific detector logic, remains open in [51].

---

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
**Status:** DONE
**Files:** `src/graph/locality/mod.rs`, `src/cli/locality.rs`, `src/cli/handlers/mod.rs`, `src/config/locality.rs`, `src/reporting.rs`
`check_locality_silent()` exists "for pipeline use" but `handle_check` doesn't call it. The `[rules.locality]` config exists but isn't enforced.

Required: Make locality a first-class stage in `neti check` gated by config mode (`off`/`warn`/`error`). Include locality results in JSON output and `neti-report.txt`.
**Resolution:** Made locality a first-class stage in neti check pipeline (Scan → Locality → Commands, 3 stages). Gated by [rules.locality] mode: "off" short-circuits, "warn" reports but doesn't block, "error" blocks on violations or cycles. Refactored check_locality_silent() to accept &Config and return a structured LocalityReport containing violation details (from → to, distance, target role) and cycle paths — not just counts. Added LocalityReport and LocalityViolation types in src/types/locality.rs. Added locality: Option<LocalityReport> to CheckReport JSON output. neti-report.txt now includes a NETI LOCALITY REPORT section showing mode, edge count, each violation with paths and distance, cycle paths, and pass/fail result. Interactive mode shows a locality scorecard with colored output. Extracted report-building and scorecard display into src/cli/handlers/check_report.rs to keep mod.rs under token limit. Twelve integration tests in tests/check_locality_test.rs verify JSON structure, field types, mode gating (off/warn/error), report file content, and overall pass/fail integration.

---

## [11] Python and TypeScript pattern detection parity
**Status:** DESCOPED
**Files:** N/A
Superseded by [17]-[22] which break this into concrete implementation steps.
**Resolution:** Descoped — covered by LangSemantics issues [17]-[22].

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
