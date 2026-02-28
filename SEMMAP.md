# project -- Semantic Map

**Purpose:** architectural linter and code quality gate for CI

## Legend

`[ENTRY]` Application entry point

`[CORE]` Core business logic

`[TYPE]` Data structures and types

`[UTIL]` Utility functions

## Layer 0 -- Config

`Cargo.toml`
Rust package manifest and dependencies.

`mutants.out/lock.json`
Configuration for lock.

`mutants.out/mutants.json`
Configuration for mutants.

`mutants.out/outcomes.json`
Configuration for outcomes.

`neti.toml`
Configuration for neti.

`src/cli/config_ui/editor.rs`
Implements config editor.
→ Exports: run_config_editor, EditResult, EventResult, set_modified

`src/cli/config_ui/items.rs`
Configuration items that can be edited.
→ Exports: ConfigItem, cycle_enum, toggle_boolean, get_value

`src/cli/config_ui/logic.rs`
Processes editor.
→ Exports: move_selection, run_editor

`src/cli/config_ui/render.rs`
Implements draw functionality.
→ Exports: draw

`src/config/io.rs`
Gets the toml config.
→ Exports: apply_project_defaults, process_ignore_line, save_to_file, load_toml_config

`src/config/locality.rs`
Configuration for the Law of Locality enforcement.
→ Exports: is_error_mode, to_validator_config, LocalityConfig, is_enabled

`src/config/types.rs`
Implements rule config.
→ Exports: CommandEntry, NetiToml, into_vec, RuleConfig

`src/graph/tsconfig.rs`
Parser for tsconfig.json / jsconfig.json path mappings.
→ Exports: TsConfig, load, resolve

## Layer 1 -- Core

`src/analysis/mod.rs`
Core analysis logic (The "Rule Engine").

`src/analysis/patterns/mod.rs`
AST pattern detection for violations.
→ Exports: get_capture_node, detect_all

`src/bin/neti.rs`
Orchestrates `clap`, `colored`, `neti_core`.

`src/cli/args.rs`
Implements cli functionality.
→ Exports: Cli, Commands

`src/cli/config_ui/mod.rs`
Orchestrates `editor`.

`src/cli/handlers/mod.rs`
Core analysis command handlers.
→ Exports: get_repo_root, handle_check, handle_scan

`src/cli/mod.rs`
CLI command handlers.

`src/config/mod.rs`
Gets the local config.
→ Exports: process_ignore_line, load_local_config, save_to_file, parse_toml

`src/graph/defs/mod.rs`
Extracts symbol DEFINITIONS from source files using tree-sitter.

`src/graph/locality/analysis/mod.rs`
Deep topology analysis: categorize violations, find patterns, suggest fixes.
→ Exports: TopologyAnalysis, analyze

`src/graph/locality/mod.rs`
Law of Locality enforcement for topological integrity.

`src/graph/mod.rs`
Module definitions for mod.

`src/graph/rank/mod.rs`
Orchestrates graph construction and ranking.
→ Exports: GraphEngine, focus_on

`src/lib.rs`
Library root and public exports.

`src/main.rs`
Placeholder file.

`src/mutate/mod.rs`
Cross-language mutation testing [EXPERIMENTAL].
→ Exports: MutateOptions, MutateReport

`src/spinner/mod.rs`
Triptych HUD (Head-Up Display) for process execution feedback.
→ Exports: start

`src/types/mod.rs`
Confidence level for a violation — how certain Neti is that this is a real problem.
→ Exports: is_small_codebase, has_blocking_errors, clean_file_count, with_details

`src/verification/mod.rs`
External command verification pipeline.
→ Exports: VerificationReport, failed_count, passed_count, total_commands

## Layer 2 -- Domain

`src/analysis/aggregator.rs`
Aggregation logic for analysis results.
→ Exports: FileAnalysis, Aggregator, ingest, merge

`src/analysis/ast.rs`
Implements analyzer functionality.
→ Exports: AnalysisResult, Analyzer, analyze

`src/analysis/checks.rs`
AST-based complexity and style checks.
→ Exports: CheckContext

`src/analysis/checks/banned.rs`
Banned construct checks (Law of Paranoia).
→ Exports: check_banned

`src/analysis/checks/complexity.rs`
Complexity metrics checks (Law of Complexity).
→ Exports: check_metrics

`src/analysis/checks/naming.rs`
Function naming checks (Law of Complexity).
→ Exports: check_naming

`src/analysis/checks/syntax.rs`
AST-level syntax error and malformed node detection.
→ Exports: check_syntax

`src/analysis/cognitive.rs`
Cognitive Complexity metric implementation.
→ Exports: CognitiveAnalyzer, calculate

`src/analysis/deep.rs`
Deep analysis runner.
→ Exports: DeepAnalyzer, compute_violations

`src/analysis/engine.rs`
Main execution logic for the `Neti` analysis engine.
→ Exports: scan_with_progress, Engine, scan

`src/analysis/extract.rs`
Rust scope extraction logic (Structs/Enums/Fields).
→ Exports: RustExtractor, extract_scopes

`src/analysis/extract_impl.rs`
Rust impl/method extraction logic.
→ Exports: extract

`src/analysis/metrics.rs`
Implements calculate complexity.
→ Exports: calculate_max_depth, count_arguments, calculate_complexity

`src/analysis/patterns/concurrency.rs`
Concurrency pattern detection: C03, C04.
→ Exports: detect

`src/analysis/patterns/concurrency_lock.rs`
C03: `MutexGuard` held across `.await`  Severity Tiers  **Sync mutex (std::sync::Mutex, parking_lot::Mutex) — HIGH confidence** Holding a sync guard across `.await` blocks the OS thread, starving the executor, and can deadlock if another task tries to acquire the same lock.
→ Exports: detect_c03

`src/analysis/patterns/concurrency_sync.rs`
C04: Undocumented synchronization primitives.
→ Exports: detect_c04

`src/analysis/patterns/db_patterns.rs`
Database anti-patterns: P03 (N+1 queries).
→ Exports: detect

`src/analysis/patterns/idiomatic.rs`
Idiomatic patterns: I01, I02.
→ Exports: detect

`src/analysis/patterns/idiomatic_i01.rs`
I01: Manual `From` implementations that could use `derive_more::From`.

`src/analysis/patterns/idiomatic_i02.rs`
I02: Duplicate match arm bodies that could be combined using `A | B => body`.

`src/analysis/patterns/logic.rs`
Logic boundary patterns: L02 (off-by-one risk), L03 (unchecked index).
→ Exports: detect

`src/analysis/patterns/logic_l02.rs`
L02: Boundary uses `<=`/`>=` with `.len()` — possible off-by-one.

`src/analysis/patterns/logic_l03.rs`
L03: Unchecked index access (`[0]`, `.first().unwrap()`, etc.).

`src/analysis/patterns/logic_proof.rs`
Fixed-size array proof helpers for L03.
→ Exports: is_fixed_size_array_access, extract_receiver

`src/analysis/patterns/performance.rs`
Performance anti-patterns: P01, P02, P04, P06  Escalation Philosophy  P01/P02 must only fire when we can make a reasonable argument that the allocation is *material*.
→ Exports: detect

`src/analysis/patterns/performance_p01.rs`
P01: `.clone()` inside a loop.

`src/analysis/patterns/performance_p02.rs`
P02: String conversion (`.to_string()` / `.to_owned()`) inside a loop.

`src/analysis/patterns/performance_p04p06.rs`
P04: Nested loop (O(n²)) and P06: linear search inside loop.

`src/analysis/patterns/resource.rs`
Resource patterns: R07 (missing flush).
→ Exports: detect

`src/analysis/patterns/security.rs`
Security patterns: X01 (SQL injection), X02 (command injection), X03 (hardcoded secrets).
→ Exports: detect

`src/analysis/patterns/security_x01.rs`
X01: SQL Injection — format!() used to build SQL strings.

`src/analysis/patterns/security_x02.rs`
X02: Command / Shell Injection.

`src/analysis/patterns/security_x03.rs`
X03: Hardcoded secrets (keys, tokens, passwords) in let/const bindings.

`src/analysis/patterns/semantic.rs`
Semantic patterns: M03, M04, M05.
→ Exports: detect

`src/analysis/patterns/state.rs`
State pattern detection: S01, S02, S03.
→ Exports: detect

`src/analysis/safety.rs`
Validates safety.
→ Exports: check_safety

`src/analysis/scope.rs`
Implements add method.
→ Exports: validate_record, FieldInfo, has_behavior, is_enum

`src/analysis/structural.rs`
Structural metrics calculation (LCOM4, CBO, SFOUT, AHF).
→ Exports: calculate_max_sfout, ScopeMetrics, calculate_ahf, calculate_cbo

`src/analysis/visitor.rs`
AST Visitor for analysis.
→ Exports: AstVisitor, extract_scopes

`src/analysis/worker.rs`
Worker module for file parsing and analysis.
→ Exports: is_ignored, scan_file

`src/branch.rs`
Git branch workflow for AI agents.
→ Exports: count_modified_files, on_work_branch, work_branch_name, PromoteResult

`src/clean.rs`
Processes .
→ Exports: run

`src/cli/audit.rs`
CLI handlers for the consolidation audit command.
→ Exports: AuditCliOptions, handle

`src/cli/dispatch.rs`
Command dispatch logic extracted from binary to reduce main function size.
→ Exports: execute

`src/cli/git_ops.rs`
Handlers for Git-based workflow operations (branch, promote, abort).
→ Exports: handle_abort, handle_branch, handle_promote

`src/cli/handlers/check_report.rs`
Report building and scorecard display for `neti check`.
→ Exports: build_report_text, print_commands_scorecard, print_locality_scorecard

`src/cli/handlers/scan_report.rs`
Scan report display formatting.
→ Exports: aggregate_by_law, build_summary_string, print

`src/cli/locality.rs`
Handler for locality scanning.
→ Exports: is_locality_blocking, check_locality_silent, run_locality_check, LocalityResult

`src/cli/mutate_handler.rs`
Processes mutate.
→ Exports: handle_mutate

`src/constants.rs`
Shared constants for file filtering and pattern matching.
→ Exports: should_prune

`src/detection.rs`
Detects build systems.
→ Exports: BuildSystemType, detect_build_systems, Detector

`src/discovery.rs`
Implements discover functionality.
→ Exports: group_by_directory, discover

`src/events.rs`
Machine-readable event logging for audit trails.
→ Exports: EventKind, EventLogger, NetiEvent

`src/exit.rs`
Standardized process exit codes for `Neti`.
→ Exports: NetiExit, code

`src/file_class.rs`
File classification: distinguishes source code from config, assets, and data.
→ Exports: FileKind, is_governed, secrets_applicable, classify

`src/graph/defs/extract.rs`
Parses .
→ Exports: DefKind, Definition

`src/graph/defs/queries.rs`
Gets the config.
→ Exports: DefExtractor, get_config

`src/graph/imports.rs`
Parses .
→ Exports: extract

`src/graph/locality/analysis/metrics.rs`
Finds hub candidates.
→ Exports: compute_module_coupling, GodModuleInfo, find_god_modules, find_hub_candidates

`src/graph/locality/analysis/violations.rs`
Categories of locality violations.
→ Exports: CategorizedViolation, ViolationKind, categorize_violation, description

`src/graph/locality/classifier.rs`
Node classification based on coupling metrics.
→ Exports: ClassifierConfig, classify

`src/graph/locality/coupling.rs`
Afferent and Efferent coupling computation.
→ Exports: compute_coupling

`src/graph/locality/cycles.rs`
Cycle detection for the Law of Locality.
→ Exports: detect_cycles

`src/graph/locality/distance.rs`
Dependency Distance calculator via Lowest Common Ancestor (LCA).
→ Exports: compute_distance, find_lca

`src/graph/locality/edges.rs`
Edge collection for locality analysis.
→ Exports: collect

`src/graph/locality/exemptions.rs`
Smart structural exemptions for Rust module patterns.
→ Exports: is_structural_pattern

`src/graph/locality/layers.rs`
Layer inference for the Law of Locality.
→ Exports: check_layer_violation, infer_layers

`src/graph/locality/report.rs`
Rich output formatting for locality analysis.
→ Exports: print_full_report

`src/graph/locality/types.rs`
Core types for the Law of Locality enforcement system.
→ Exports: allows_far_deps, routes_to_hub, NodeIdentity, PassReason

`src/graph/locality/validator.rs`
The Universal Locality Algorithm - Judgment Pass.
→ Exports: ValidationReport, ValidatorConfig, check_cohesion, is_clean

`src/graph/rank/builder.rs`
Graph construction logic: extraction and edge building.
→ Exports: rebuild_topology, GraphData, build_data

`src/graph/rank/graph.rs`
The dependency graph structure and query interface.
→ Exports: is_hub, ranked_files, RepoGraph, graph_tags

`src/graph/rank/pagerank.rs`
PageRank` algorithm implementation for file ranking.
→ Exports: compute

`src/graph/rank/queries.rs`
Gets the ranked files.
→ Exports: get_graph_tags, get_ranked_files, collect_dependencies, collect_dependents

`src/graph/rank/tags.rs`
Tag types representing definitions and references.
→ Exports: Tag, TagKind

`src/graph/resolver.rs`
Implements resolve functionality.
→ Exports: resolve

`src/lang.rs`
Implements q complexity.
→ Exports: from_ext, QueryKind, skeleton_replacement, q_complexity

`src/lang_queries.rs`
Implements lang queries.

`src/mutate/discovery.rs`
Discovers mutation points in source files using tree-sitter.
→ Exports: discover_mutations

`src/mutate/mutations.rs`
Mutation types and application logic.
→ Exports: MutationKind, MutationPoint, apply_mutation, get_mutation

`src/mutate/report.rs`
Report formatting for mutation test results.
→ Exports: format_json, format_progress, format_summary, format_survivors

`src/mutate/runner.rs`
Parallel mutation test runner.
→ Exports: RunnerConfig, run_mutations, MutationResult, MutationSummary

`src/project.rs`
Detects project type from current directory.
→ Exports: ProjectType, generate_toml, is_typescript, npx_cmd

`src/reporting.rs`
Console output formatting for scan results.
→ Exports: print_json

`src/reporting/console.rs`
Prints a formatted scan report to stdout with confidence tiers and deduplication.
→ Exports: print_report

`src/reporting/guidance.rs`
Static educational guidance per rule code.

`src/reporting/rich.rs`
Formats report string for output.
→ Exports: build_rich_report, format_report_string

`src/reporting/shared.rs`
Orchestrates `crate`.

`src/skeleton.rs`
Reduces code to its structural skeleton (signatures only).
→ Exports: clean

`src/spinner/client.rs`
Client for sending updates to the spinner.
→ Exports: set_macro_step, set_micro_status, step_micro_progress, SpinnerClient

`src/spinner/controller.rs`
Lifecycle controller for the spinner thread.
→ Exports: SpinnerController, stop

`src/spinner/handle.rs`
Thread management for the spinner.
→ Exports: SpinnerHandle, spawn, stop

`src/spinner/render.rs`
HUD rendering logic.
→ Exports: run_hud_loop

`src/spinner/safe_hud.rs`
Thread-safe wrapper for HUD state.
→ Exports: SafeHud, completion_info, modify, snapshot

`src/spinner/state.rs`
HUD state management.
→ Exports: step_micro_progress, set_macro_step, set_micro_status, completion_info

`src/tokens.rs`
The tokenizer encoding (`cl100k_base`, used by GPT-4/3.5-turbo).
→ Exports: exceeds_limit, is_available, Tokenizer, count

`src/types/command.rs`
Result of an external command execution.
→ Exports: CommandResult, duration_ms, exit_code, error_count

`src/types/locality.rs`
Types for locality (Law of Locality) reporting.
→ Exports: LocalityReport, LocalityViolation

`src/verification/runner.rs`
Command execution and output capture.
→ Exports: run_commands

## Layer 3 -- Utilities

`src/analysis/patterns/logic_helpers.rs`
Shared helpers for L02/L03 logic pattern detection.
→ Exports: can_find_local_declaration, has_chunks_exact_context, decl_matches_variable, has_explicit_guard

`src/analysis/patterns/logic_proof_helpers.rs`
Helper routines for extracting and verifying array sizes in scope boundaries.

`src/utils.rs`
Implements compute sha256.
→ Exports: compute_sha256

## Layer 4 -- Tests

`src/analysis/checks/syntax_test.rs`
Orchestrates `crate`, `super`, `tree_sitter`.

`src/analysis/inspector.rs`
Inspection logic for scopes (Metrics application).
→ Exports: Inspector, inspect, new

`src/analysis/patterns/concurrency_lock_test.rs`
Orchestrates `super`, `tokio`, `tree_sitter`.

`src/analysis/patterns/idiomatic_i02_test.rs`
Orchestrates `Idx`, `super`, `tree_sitter`.

`src/analysis/patterns/logic_helpers_test.rs`
Orchestrates `super`, `tree_sitter`.

`src/analysis/patterns/logic_l03_test.rs`
Orchestrates `super`, `tree_sitter`.

`src/analysis/patterns/logic_proof_test.rs`
Orchestrates `super`, `tree_sitter`.

`src/analysis/patterns/performance_p01_test.rs`
Orchestrates `crate`, `super`, `tree_sitter`.

`src/analysis/patterns/performance_test_ctx.rs`
Test context detection for pattern detectors.
→ Exports: is_test_context

`src/analysis/patterns/security_x02_test.rs`
Orchestrates `super`, `tree_sitter`.

`src/graph/locality/tests.rs`
Integration tests for locality analysis — part 1.

`src/graph/locality/tests/part2.rs`
Integration tests for locality analysis — part 2.

`tests/check_json_test.rs`
Integration test: `neti check --json` must emit valid JSON to stdout.

`tests/check_locality_test.rs`
Integration test: locality integration in `neti check` pipeline.

`tests/command_parsing_test.rs`
Integration test: command parsing with shell-words.

