# DONE Issues

---

## [51] Adopt `omni-ast` as NETI's primary semantic engine
**Status:** DONE
**Files:** `src/lang.rs`, `src/analysis/patterns/mod.rs`, `src/analysis/patterns/performance.rs`, `src/analysis/patterns/performance_p01.rs`, `src/analysis/patterns/performance_p04p06.rs`, `src/analysis/patterns/performance_test_ctx.rs`, `src/analysis/patterns/logic.rs`, `src/analysis/patterns/logic_l02.rs`, `src/analysis/patterns/logic_l03.rs`, `src/analysis/patterns/logic_helpers.rs`, shared `omni-ast` semantics modules
**Labels:** Architecture, Adoption, Language Support, Integrations
**Depends on:** [50]

**Problem:** The shared crate now exists and NETI consumes pieces of it, but NETI is not yet primarily governed by `omni-ast` semantics. Detector behavior still largely lives in Rust-specific rule code, which means the extraction milestone is real while the strategic adoption milestone remains open.

**Fix:**

1. Define the shared semantic contract NETI detectors should consume as their main interface.
2. Move detector-facing language and concept knowledge behind `omni-ast` instead of keeping it embedded in NETI rules.
3. Replace detector-local Rust syntax assumptions with shared semantic queries in the main rule paths.
4. Prove the adoption with cross-language rule coverage showing one rule can execute through the shared semantic layer.

**Resolution:** Turned `omni-ast` semantics into the live detector contract NETI now consumes. `LangSemantics` gained stable detector-facing queries for concept checks, length-boundary risk, collection access risk, front-access unwrap risk, and guard detection, backed by `SharedSemantics`, split logic/performance semantic tables, and path-aware `SemanticContext`. I chose to keep Rust tree-sitter structure only for precision and line attribution while moving vocabulary and semantic rule predicates behind `omni-ast`; that makes the shared crate the governing semantic layer without forcing a one-shot rewrite of all AST extraction.

NETI's performance and logic detector families now route their core rule predicates through shared semantics. Rust performance/logic paths still use AST traversal for precision, but no longer own the semantic vocabulary locally. `detect_all()` also now executes shared-semantic rule paths for non-Rust files, with verified cross-language coverage for P06 and L02 on Python and TypeScript inputs. Verification used `neti check`: `cargo clippy --all-targets --no-deps -- -D warnings` passed, `cargo test` passed, and the only remaining non-zero condition was the pre-existing `LCOM4` warning on `src/types/command.rs`. Commands run: `semmap generate`, `semmap trace src/lang.rs`, `semmap trace src/analysis/patterns/performance.rs`, `semmap trace src/analysis/patterns/logic.rs`, `neti check`.

---

## [17] Implement `LangSemantics` in the new shared crate
**Status:** DONE
**Files:** `src/lang.rs`, `omni-ast/src/lib.rs`, `omni-ast/src/semantics.rs`, `omni-ast/src/semantics_engine.rs`, `omni-ast/src/semantics_queries.rs`, `omni-ast/src/semantics_logic_queries.rs`, `omni-ast/src/semantics_tables.rs`, `omni-ast/src/semantics_logic_tables.rs`
**Labels:** Architecture, Language Support, Detection Rules
**Depends on:** [51]

**Problem:** NETI needs a stable semantic interface above raw syntax. That interface should live in the shared crate so both projects can reason about concepts such as test context, heap allocation, mutation, locking, and exported roles without duplicating language-specific logic.

**Fix:**

1. Define a `LangSemantics` trait in the shared crate.
2. Make the trait answer the semantic queries NETI detectors need, such as `is_test_context()` and `has_concept(Concept::HeapAllocation)`.
3. Map SEMMAP-style badges and concepts onto that trait surface.
4. Expose the interface so NETI detectors can query semantics without directly handling AST syntax.

**Resolution:** Implemented `LangSemantics` as NETI's shared semantic contract inside `omni-ast`, with exported `SharedSemantics` instances created through `semantics_for()`. The trait now answers concept-level questions (`TestContext`, `HeapAllocation`, `Lookup`, `Length`, `Mutation`, `Locking`, `Loop`, `ExportedApi`) plus detector-facing logic queries for boundary risk, unguarded collection access, front unwrap access, and guarding collection checks. SEMMAP fingerprint and badge data now feed the same interface through `SemanticContext`, which carries harvested metadata, source text, and optional path hints. NETI consumes the trait directly in detector paths, so the semantic surface is no longer hypothetical. Verified by `neti check`; command stages passed and only the pre-existing `src/types/command.rs` warning remained.

---

## [18] Wire shared semantics into performance detectors
**Status:** DONE
**Files:** `src/analysis/patterns/performance.rs`, `src/analysis/patterns/performance_p01.rs`, `src/analysis/patterns/performance_p04p06.rs`, `src/analysis/patterns/performance_test_ctx.rs`
**Labels:** Architecture, Language Support, Detection Rules, Performance
**Depends on:** [17]

**Problem:** Performance detectors still rely on Rust-shaped vocabulary and local heuristics. They should operate on shared semantic concepts so one rule can run against Rust, Python, Go, and JS/TS through the same interface.

**Fix:**

1. Replace detector-local vocabulary checks with queries against shared semantics.
2. Express rules in terms of concepts such as allocation, lookup, collection iteration, and test context.
3. Remove path-based skip heuristics that exist only to compensate for weak semantics.
4. Add tests showing the same detector intent can run across multiple languages through the shared layer.

**Resolution:** Rewired the performance detector family to consume `LangSemantics` directly. Test-context detection, heap-allocation classification, and lookup-in-loop checks now all route through `omni-ast` semantic queries instead of local Rust keyword tables. Rust keeps AST-driven loop extraction for precision, while non-Rust files execute the shared-semantic P06 path through `detect_all()`. Added regression coverage proving the same P06 intent now runs on Python and TypeScript through the shared layer. Verified with `neti check`; command stages passed and the only remaining warning was pre-existing in `src/types/command.rs`.

---

## [19] Wire shared semantics into logic detectors
**Status:** DONE
**Files:** `src/analysis/patterns/logic.rs`, `src/analysis/patterns/logic_l02.rs`, `src/analysis/patterns/logic_l03.rs`, `src/analysis/patterns/logic_helpers.rs`
**Labels:** Architecture, Language Support, Detection Rules
**Depends on:** [17]

**Problem:** Logic detectors currently mix rule intent with Rust-specific syntax assumptions. That blocks cross-language governance and makes the rules harder to reason about.

**Fix:**

1. Rewrite logic detectors to query shared semantics rather than raw syntax vocabulary.
2. Preserve Rust-only proof helpers only where they materially improve precision.
3. Keep any Rust-specific precision layer clearly gated on language, not embedded in the core rule definition.
4. Add regression coverage proving shared semantics drives the rule while precision enhancers remain optional.

**Resolution:** Logic detectors now consume shared semantic queries for their core predicates. L02's boundary-risk and length-side detection moved behind `LangSemantics`, while L03 now uses shared semantic checks for index access, front unwrap access, and guard detection; Rust-only array/chunks proof helpers remain as explicit precision layers. `detect_all()` now runs a shared-semantic L02 path for non-Rust files, with regression tests covering Python and TypeScript off-by-one detection. Verified by `neti check`; `cargo clippy` and `cargo test` both passed, and no touched-path scan errors remained.

---

## [21] Populate Python semantics in the shared crate
**Status:** DONE
**Files:** `omni-ast/src/semantics.rs`, `omni-ast/src/semantics_queries.rs`, `omni-ast/src/semantics_logic_queries.rs`, `omni-ast/src/semantics_tables.rs`, `omni-ast/src/semantics_logic_tables.rs`
**Labels:** Language Support, Architecture, Detection Rules
**Depends on:** [17]

**Problem:** Python needs a first-class semantics table in the shared crate before cross-language detector execution can be credible.

**Fix:**

1. Add Python test-context semantics.
2. Add Python heap, lookup, length, mutation, and loop concepts.
3. Map Python syntax and library vocabulary onto the shared concept model.
4. Verify NETI rules can consume Python semantics through the same detector queries used for Rust.

**Resolution:** Added Python semantic coverage for test context, heap allocation, lookup, length, mutation, loop detection, and shared logic-rule predicates in `omni-ast`. NETI now consumes those semantics in live detector paths: Python files can trigger shared-semantic P06 and L02 through `detect_all()`, backed by regression tests. Verified with `neti check`; command stages passed and the only remaining warning was the unrelated `src/types/command.rs` LCOM4 warning.

---

## [22] Populate TypeScript semantics in the shared crate
**Status:** DONE
**Files:** `omni-ast/src/semantics.rs`, `omni-ast/src/semantics_queries.rs`, `omni-ast/src/semantics_logic_queries.rs`, `omni-ast/src/semantics_tables.rs`, `omni-ast/src/semantics_logic_tables.rs`
**Labels:** Language Support, Architecture, Detection Rules, Web Stack
**Depends on:** [17]

**Problem:** TypeScript and JavaScript need shared semantics coverage so NETI rules can execute over web code through the same concept interface.

**Fix:**

1. Add JS/TS test-context semantics.
2. Add JS/TS heap, lookup, length, mutation, and loop concepts.
3. Map JS/TS library and syntax vocabulary onto the shared concept model.
4. Verify NETI rules can consume JS/TS semantics through the common detector interface.

**Resolution:** Expanded `omni-ast` with JS/TS semantic tables covering test contexts, heap allocation, lookup, length, mutation, loops, and shared logic/performance rule predicates. NETI now exercises those semantics through `detect_all()`, with TypeScript regression coverage for both P06 and L02 on the shared-semantic execution path. Verified with `neti check`; verification commands passed and only the pre-existing `src/types/command.rs` warning remained.

---

## [41] Port SEMMAP SWUM expansion to Neti naming rules
**Status:** DONE
**Files:** `src/analysis/checks/naming.rs`, `omni-ast/src/swum/mod.rs`, `omni-ast/src/swum/splitter.rs`, `omni-ast/src/swum/verb_patterns.rs`
**Labels:** Language Support, Detection Rules, Architecture
**Depends on:** [51]

**Problem:** NETI naming guidance is still shallow and language-specific. SEMMAP already has a SWUM engine that can expand identifiers into verb-intent phrases, which is the right foundation for cross-language naming rules.

**Fix:**

1. Port SEMMAP's SWUM engine into `omni-ast`.
2. Replace NETI naming-rule heuristics with SWUM-backed semantic expansion where practical.
3. Use the shared engine to reason about verbs, themes, acronyms, and intent across languages.
4. Add tests proving naming analysis behavior works through the shared interface instead of language-local hacks.

**Resolution:** Adopted the existing `omni-ast` SWUM engine into NETI's naming rule path. `src/analysis/checks/naming.rs` now uses shared `split_identifier()` and `expand_identifier()` instead of maintaining a local snake/camel splitter, so acronym boundaries, verbs, themes, and intent all come from the shared SWUM surface. The naming violation payload now includes SWUM's interpretation of the identifier and builds rename guidance from shared word splits rather than NETI-only heuristics. Added regression tests covering acronym-heavy camelCase, snake_case counting, and SWUM-backed suggestion text. Verified with `neti check`; both command stages passed and the only remaining non-zero condition was the pre-existing `LCOM4` warning in `src/types/command.rs`.

---

## [20] Wire shared semantics into concurrency and remaining detectors
**Status:** DONE
**Files:** `src/analysis/patterns/semantic.rs`, `src/analysis/patterns/concurrency.rs`, `src/analysis/patterns/concurrency_lock.rs`, `src/analysis/patterns/concurrency_sync.rs`, `omni-ast/src/semantics.rs`, `omni-ast/src/semantics_engine.rs`, `omni-ast/src/semantics_concurrency_queries.rs`, `omni-ast/src/semantics_tables.rs`
**Labels:** Architecture, Language Support, Detection Rules, Safety
**Depends on:** [17]

**Problem:** Lock-type knowledge, mutation receiver patterns, and other semantic cues still live inline inside NETI detectors. That leaves the cross-language abstraction incomplete.

**Fix:**

1. Move lock and sync concepts behind the shared semantics layer.
2. Move mutation and state-change concepts behind the shared semantics layer.
3. Remove remaining detector-local Rust vocabulary where the shared crate can own it.
4. Extend tests to prove these rule families work through semantic concepts rather than syntax matching.

**Resolution:** Moved the remaining concurrency and mutation cue ownership behind `LangSemantics`. `omni-ast` now exposes `is_async_locking_context()` and expanded Rust mutation semantics so NETI no longer hardcodes async-lock import families or `&mut self` mutation markers inside detector-local helpers. C03 now uses shared locking concepts for lock detection and shared async-lock classification for severity selection; C04 now detects synchronization fields through shared locking semantics instead of inline `Arc<Mutex>`/`Arc<RwLock>` string checks; M03/M05 now use shared mutation semantics rather than local `contains(\"mut\")` checks on self parameters. I kept AST traversal only where it provides structural evidence such as async function shape, await span, and field locations. Verified with `neti check`; `cargo clippy --all-targets --no-deps -- -D warnings` passed, `cargo test` passed, and the only remaining non-zero condition was the pre-existing `LCOM4` warning in `src/types/command.rs`.

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
