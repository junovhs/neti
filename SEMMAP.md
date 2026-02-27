# project -- Semantic Map

**Purpose:** architectural linter and code quality gate for CI

## Legend

`[ENTRY]` Application entry point

`[CORE]` Core business logic

`[TYPE]` Data structures and types

`[UTIL]` Utility functions

## Layer 0 -- Config

`Cargo.toml`
Rust package manifest and dependencies. Centralizes project configuration.

`mutants.out/lock.json`
Configuration for lock. Centralizes project configuration.

`mutants.out/mutants.json`
Configuration for mutants. Centralizes project configuration.

`mutants.out/outcomes.json`
Configuration for outcomes. Centralizes project configuration.

`neti.toml`
Configuration for neti. Centralizes project configuration.

`src/cli/config_ui/editor.rs`
Module providing `ConfigEditor`, `EditResult`, `EventResult`. Centralizes project configuration.
→ Exports: ConfigEditor, EditResult, EventResult, config, config_mut, items, new, run, run_config_editor, selected, set_modified, set_selected

`src/cli/config_ui/items.rs`
Configuration items that can be edited. Centralizes project configuration.
→ Exports: ConfigItem, all, cycle_enum, get_number, get_value, label, set_number, toggle_boolean

`src/cli/config_ui/logic.rs`
Module providing `move_selection`, `run_editor`. Centralizes project configuration.
→ Exports: move_selection, run_editor

`src/cli/config_ui/render.rs`
Module providing `draw`. Centralizes project configuration.
→ Exports: draw

`src/config/io.rs`
Module providing `apply_project_defaults`, `load_ignore_file`, `load_toml_config`. Centralizes project configuration.
→ Exports: apply_project_defaults, load_ignore_file, load_toml_config, parse_toml, process_ignore_line, save_to_file

`src/config/locality.rs`
Configuration for the Law of Locality enforcement. Centralizes project configuration.
→ Exports: LocalityConfig, is_enabled, is_error_mode, to_validator_config

`src/config/types.rs`
Module providing `CommandEntry`, `Config`, `NetiToml`. Centralizes project configuration.
→ Exports: CommandEntry, Config, NetiToml, Preferences, RuleConfig, SafetyConfig, into_vec

`src/graph/tsconfig.rs`
Parser for tsconfig.json / jsconfig.json path mappings. Centralizes project configuration.
→ Exports: TsConfig, load, resolve

## Layer 1 -- Core

`src/analysis/mod.rs`
Core analysis logic (The "Rule Engine"). Supports application functionality.

`src/analysis/patterns/mod.rs`
AST pattern detection for violations. Supports application functionality.
→ Exports: detect_all, get_capture_node

`src/bin/neti.rs`
Orchestrates `clap`, `colored`, `neti_core`. Defines command-line interface.

`src/cli/args.rs`
Module providing `Cli`, `Commands`. Defines command-line interface.
→ Exports: Cli, Commands

`src/cli/config_ui/mod.rs`
Orchestrates `editor`. Centralizes project configuration.

`src/cli/handlers/mod.rs`
Core analysis command handlers. Supports application functionality.
→ Exports: get_repo_root, handle_check, handle_scan

`src/cli/mod.rs`
CLI command handlers. Supports application functionality.

`src/config/mod.rs`
Module providing `load`, `load_local_config`, `new`. Centralizes project configuration.
→ Exports: load, load_local_config, new, parse_toml, process_ignore_line, save, save_to_file, validate

`src/graph/defs/mod.rs`
Extracts symbol DEFINITIONS from source files using tree-sitter. Supports application functionality.

`src/graph/locality/analysis/mod.rs`
Deep topology analysis: categorize violations, find patterns, suggest fixes. Supports application functionality.
→ Exports: TopologyAnalysis, analyze

`src/graph/locality/mod.rs`
Law of Locality enforcement for topological integrity. Supports application functionality.

`src/graph/mod.rs`
Module definitions for mod. Supports application functionality.

`src/graph/rank/mod.rs`
Orchestrates graph construction and ranking. Supports application functionality.
→ Exports: GraphEngine, build, focus_on

`src/lib.rs`
Library root and public exports. Provides application entry point.

`src/main.rs`
Placeholder file. Provides application entry point.

`src/mutate/mod.rs`
Cross-language mutation testing [EXPERIMENTAL]. Supports application functionality.
→ Exports: MutateOptions, MutateReport, run

`src/spinner/mod.rs`
Triptych HUD (Head-Up Display) for process execution feedback. Supports application functionality.
→ Exports: start

`src/verification/mod.rs`
External command verification pipeline. Supports application functionality.
→ Exports: VerificationReport, failed_count, new, passed_count, run, total_commands, total_errors, total_warnings

## Layer 2 -- Domain

`src/analysis/aggregator.rs`
Aggregation logic for analysis results. Supports application functionality.
→ Exports: Aggregator, FileAnalysis, ingest, merge, new

`src/analysis/ast.rs`
Module providing `AnalysisResult`, `Analyzer`, `analyze`. Supports application functionality.
→ Exports: AnalysisResult, Analyzer, analyze, new

`src/analysis/checks.rs`
AST-based complexity and style checks. Supports application functionality.
→ Exports: CheckContext

`src/analysis/checks/banned.rs`
Banned construct checks (Law of Paranoia). Supports application functionality.
→ Exports: check_banned

`src/analysis/checks/complexity.rs`
Complexity metrics checks (Law of Complexity). Supports application functionality.
→ Exports: check_metrics

`src/analysis/checks/naming.rs`
Function naming checks (Law of Complexity). Supports application functionality.
→ Exports: check_naming

`src/analysis/checks/syntax.rs`
AST-level syntax error and malformed node detection. Supports application functionality.
→ Exports: check_syntax

`src/analysis/cognitive.rs`
Cognitive Complexity metric implementation. Supports application functionality.
→ Exports: CognitiveAnalyzer, calculate

`src/analysis/deep.rs`
Deep analysis runner. Supports application functionality.
→ Exports: DeepAnalyzer, compute_violations, new

`src/analysis/engine.rs`
Main execution logic for the `Neti` analysis engine. Supports application functionality.
→ Exports: Engine, scan, scan_with_progress

`src/analysis/extract.rs`
Rust scope extraction logic (Structs/Enums/Fields). Supports application functionality.
→ Exports: RustExtractor, extract_scopes

`src/analysis/extract_impl.rs`
Rust impl/method extraction logic. Supports application functionality.
→ Exports: extract

`src/analysis/metrics.rs`
Module providing `calculate_complexity`, `calculate_max_depth`, `count_arguments`. Supports application functionality.
→ Exports: calculate_complexity, calculate_max_depth, count_arguments

`src/analysis/patterns/concurrency.rs`
Concurrency pattern detection: C03, C04. Supports application functionality.
→ Exports: detect

`src/analysis/patterns/concurrency_lock.rs`
C03: `MutexGuard` held across `.await`  Severity Tiers  **Sync mutex (std::sync::Mutex, parking_lot::Mutex) — HIGH confidence** Holding a sync guard across `.await` blocks the OS thread, starving the executor, and can deadlock if another task tries to acquire the same lock. Supports application functionality.
→ Exports: detect_c03

`src/analysis/patterns/concurrency_sync.rs`
C04: Undocumented synchronization primitives. Supports application functionality.
→ Exports: detect_c04

`src/analysis/patterns/db_patterns.rs`
Database anti-patterns: P03 (N+1 queries). Supports application functionality.
→ Exports: detect

`src/analysis/patterns/idiomatic.rs`
Idiomatic patterns: I01, I02. Supports application functionality.
→ Exports: detect

`src/analysis/patterns/idiomatic_i01.rs`
I01: Manual `From` implementations that could use `derive_more::From`. Supports application functionality.

`src/analysis/patterns/idiomatic_i02.rs`
I02: Duplicate match arm bodies that could be combined using `A | B => body`. Supports application functionality.

`src/analysis/patterns/logic.rs`
Logic boundary patterns: L02 (off-by-one risk), L03 (unchecked index). Supports application functionality.
→ Exports: detect

`src/analysis/patterns/logic_l02.rs`
L02: Boundary uses `<=`/`>=` with `.len()` — possible off-by-one. Supports application functionality.

`src/analysis/patterns/logic_l03.rs`
L03: Unchecked index access (`[0]`, `.first().unwrap()`, etc.). Supports application functionality.

`src/analysis/patterns/logic_proof.rs`
Fixed-size array proof helpers for L03. Supports application functionality.
→ Exports: extract_receiver, is_fixed_size_array_access

`src/analysis/patterns/performance.rs`
Performance anti-patterns: P01, P02, P04, P06  Escalation Philosophy  P01/P02 must only fire when we can make a reasonable argument that the allocation is *material*. Supports application functionality.
→ Exports: detect

`src/analysis/patterns/performance_p01.rs`
P01: `.clone()` inside a loop. Supports application functionality.

`src/analysis/patterns/performance_p02.rs`
P02: String conversion (`.to_string()` / `.to_owned()`) inside a loop. Supports application functionality.

`src/analysis/patterns/performance_p04p06.rs`
P04: Nested loop (O(n²)) and P06: linear search inside loop. Supports application functionality.

`src/analysis/patterns/resource.rs`
Resource patterns: R07 (missing flush). Supports application functionality.
→ Exports: detect

`src/analysis/patterns/security.rs`
Security patterns: X01 (SQL injection), X02 (command injection), X03 (hardcoded secrets). Supports application functionality.
→ Exports: detect

`src/analysis/patterns/security_x01.rs`
X01: SQL Injection — format!() used to build SQL strings. Supports application functionality.

`src/analysis/patterns/security_x02.rs`
X02: Command / Shell Injection. Supports application functionality.

`src/analysis/patterns/security_x03.rs`
X03: Hardcoded secrets (keys, tokens, passwords) in let/const bindings. Supports application functionality.

`src/analysis/patterns/semantic.rs`
Semantic patterns: M03, M04, M05. Supports application functionality.
→ Exports: detect

`src/analysis/patterns/state.rs`
State pattern detection: S01, S02, S03. Supports application functionality.
→ Exports: detect

`src/analysis/safety.rs`
Module providing `check_safety`. Supports application functionality.
→ Exports: check_safety

`src/analysis/scope.rs`
Module providing `FieldInfo`, `Method`, `Scope`. Supports application functionality.
→ Exports: FieldInfo, Method, Scope, add_derive, add_field, add_method, derives, fields, has_behavior, has_derives, is_enum, methods, name, new, new_enum, row, validate_record

`src/analysis/structural.rs`
Structural metrics calculation (LCOM4, CBO, SFOUT, AHF). Supports application functionality.
→ Exports: ScopeMetrics, calculate_ahf, calculate_cbo, calculate_lcom4, calculate_max_sfout

`src/analysis/visitor.rs`
AST Visitor for analysis. Supports application functionality.
→ Exports: AstVisitor, extract_scopes, new

`src/analysis/worker.rs`
Worker module for file parsing and analysis. Supports application functionality.
→ Exports: is_ignored, scan_file

`src/branch.rs`
Git branch workflow for AI agents. Supports application functionality.
→ Exports: BranchResult, PromoteResult, abort, count_modified_files, init_branch, on_work_branch, promote, work_branch_name

`src/clean.rs`
Module providing `run`. Supports application functionality.
→ Exports: run

`src/cli/audit.rs`
CLI handlers for the consolidation audit command. Supports application functionality.
→ Exports: AuditCliOptions, handle

`src/cli/dispatch.rs`
Command dispatch logic extracted from binary to reduce main function size. Supports application functionality.
→ Exports: execute

`src/cli/git_ops.rs`
Handlers for Git-based workflow operations (branch, promote, abort). Supports application functionality.
→ Exports: handle_abort, handle_branch, handle_promote

`src/cli/handlers/scan_report.rs`
Scan report display formatting. Supports application functionality.
→ Exports: aggregate_by_law, build_summary_string, print

`src/cli/locality.rs`
Handler for locality scanning. Supports application functionality.
→ Exports: LocalityResult, check_locality_silent, handle_locality, is_locality_blocking, run_locality_check

`src/cli/mutate_handler.rs`
Module providing `handle_mutate`. Supports application functionality.
→ Exports: handle_mutate

`src/constants.rs`
Shared constants for file filtering and pattern matching. Supports application functionality.
→ Exports: should_prune

`src/detection.rs`
Detects build systems. Supports application functionality.
→ Exports: BuildSystemType, Detector, detect_build_systems, new

`src/discovery.rs`
Module providing `discover`, `group_by_directory`. Parses input into structured data.
→ Exports: discover, group_by_directory

`src/events.rs`
Machine-readable event logging for audit trails. Supports application functionality.
→ Exports: EventKind, EventLogger, NetiEvent, log, new

`src/exit.rs`
Standardized process exit codes for `Neti`. Supports application functionality.
→ Exports: NetiExit, code, exit

`src/file_class.rs`
File classification: distinguishes source code from config, assets, and data. Supports application functionality.
→ Exports: FileKind, classify, is_governed, secrets_applicable

`src/graph/defs/extract.rs`
Module providing `DefKind`, `Definition`, `extract`. Supports application functionality.
→ Exports: DefKind, Definition, extract

`src/graph/defs/queries.rs`
Module providing `DefExtractor`, `get_config`. Supports application functionality.
→ Exports: DefExtractor, get_config

`src/graph/imports.rs`
Module providing `extract`. Supports application functionality.
→ Exports: extract

`src/graph/locality/analysis/metrics.rs`
Module providing `GodModuleInfo`, `HubCandidate`, `compute_module_coupling`. Supports application functionality.
→ Exports: GodModuleInfo, HubCandidate, compute_module_coupling, find_god_modules, find_hub_candidates

`src/graph/locality/analysis/violations.rs`
Categories of locality violations. Supports application functionality.
→ Exports: CategorizedViolation, ViolationKind, categorize_violation, description, label, suggest

`src/graph/locality/classifier.rs`
Node classification based on coupling metrics. Supports application functionality.
→ Exports: ClassifierConfig, classify

`src/graph/locality/coupling.rs`
Afferent and Efferent coupling computation. Supports application functionality.
→ Exports: compute_coupling

`src/graph/locality/cycles.rs`
Cycle detection for the Law of Locality. Supports application functionality.
→ Exports: detect_cycles

`src/graph/locality/distance.rs`
Dependency Distance calculator via Lowest Common Ancestor (LCA). Supports application functionality.
→ Exports: compute_distance, find_lca

`src/graph/locality/edges.rs`
Edge collection for locality analysis. Supports application functionality.
→ Exports: collect

`src/graph/locality/exemptions.rs`
Smart structural exemptions for Rust module patterns. Supports application functionality.
→ Exports: is_structural_pattern

`src/graph/locality/layers.rs`
Layer inference for the Law of Locality. Supports application functionality.
→ Exports: check_layer_violation, infer_layers

`src/graph/locality/report.rs`
Rich output formatting for locality analysis. Supports application functionality.
→ Exports: print_full_report

`src/graph/locality/types.rs`
Core types for the Law of Locality enforcement system. Defines domain data structures.
→ Exports: Coupling, EdgeVerdict, LocalityEdge, NodeIdentity, PassReason, allows_far_deps, instability, is_local, label, new, routes_to_hub, skew, total

`src/graph/locality/validator.rs`
The Universal Locality Algorithm - Judgment Pass. Supports application functionality.
→ Exports: ValidationReport, ValidatorConfig, check_cohesion, is_clean, validate_edge, validate_graph

`src/graph/rank/builder.rs`
Graph construction logic: extraction and edge building. Supports application functionality.
→ Exports: GraphData, build_data, rebuild_topology

`src/graph/rank/graph.rs`
The dependency graph structure and query interface. Supports application functionality.
→ Exports: RepoGraph, dependencies, dependents, graph_tags, is_hub, neighbors, new, ranked_files

`src/graph/rank/pagerank.rs`
PageRank` algorithm implementation for file ranking. Supports application functionality.
→ Exports: compute

`src/graph/rank/queries.rs`
Module providing `collect_dependencies`, `collect_dependents`, `get_dependencies`. Supports application functionality.
→ Exports: collect_dependencies, collect_dependents, get_dependencies, get_dependents, get_graph_tags, get_neighbors, get_ranked_files

`src/graph/rank/tags.rs`
Tag types representing definitions and references. Supports application functionality.
→ Exports: Tag, TagKind

`src/graph/resolver.rs`
Module providing `resolve`. Supports application functionality.
→ Exports: resolve

`src/lang.rs`
Module providing `Lang`, `QueryKind`, `from_ext`. Supports application functionality.
→ Exports: Lang, QueryKind, from_ext, grammar, q_complexity, q_defs, q_exports, q_imports, q_naming, q_skeleton, query, skeleton_replacement

`src/lang_queries.rs`
Implements lang queries. Supports application functionality.

`src/mutate/discovery.rs`
Discovers mutation points in source files using tree-sitter. Supports application functionality.
→ Exports: discover_mutations

`src/mutate/mutations.rs`
Mutation types and application logic. Supports application functionality.
→ Exports: MutationKind, MutationPoint, apply_mutation, get_mutation, symbol

`src/mutate/report.rs`
Report formatting for mutation test results. Supports application functionality.
→ Exports: format_json, format_progress, format_summary, format_survivors

`src/mutate/runner.rs`
Parallel mutation test runner. Supports application functionality.
→ Exports: MutationResult, MutationSummary, RunnerConfig, python, run_mutations, rust, summarize, typescript

`src/project.rs`
Detects project type from current directory. Supports application functionality.
→ Exports: ProjectType, Strictness, detect, detect_in, generate_toml, is_typescript, npx_cmd

`src/reporting.rs`
Console output formatting for scan results. Supports application functionality.
→ Exports: print_json

`src/reporting/console.rs`
Prints a formatted scan report to stdout with confidence tiers and deduplication. Supports application functionality.
→ Exports: print_report

`src/reporting/guidance.rs`
Static educational guidance per rule code. Supports application functionality.

`src/reporting/rich.rs`
Module providing `build_rich_report`, `format_report_string`. Supports application functionality.
→ Exports: build_rich_report, format_report_string

`src/reporting/shared.rs`
Orchestrates `crate`. Supports application functionality.

`src/skeleton.rs`
Reduces code to its structural skeleton (signatures only). Supports application functionality.
→ Exports: clean

`src/spinner/client.rs`
Client for sending updates to the spinner. Supports application functionality.
→ Exports: SpinnerClient, new, push_log, set_macro_step, set_micro_status, step_micro_progress, tick

`src/spinner/controller.rs`
Lifecycle controller for the spinner thread. Supports application functionality.
→ Exports: SpinnerController, new, stop

`src/spinner/handle.rs`
Thread management for the spinner. Supports application functionality.
→ Exports: SpinnerHandle, spawn, stop

`src/spinner/render.rs`
HUD rendering logic. Formats data for output.
→ Exports: run_hud_loop

`src/spinner/safe_hud.rs`
Thread-safe wrapper for HUD state. Supports application functionality.
→ Exports: SafeHud, completion_info, modify, new, snapshot

`src/spinner/state.rs`
HUD state management. Supports application functionality.
→ Exports: HudSnapshot, HudState, completion_info, new, push_log, set_finished, set_macro_step, set_micro_status, snapshot, step_micro_progress, tick

`src/tokens.rs`
The tokenizer encoding (`cl100k_base`, used by GPT-4/3.5-turbo). Supports application functionality.
→ Exports: Tokenizer, count, exceeds_limit, is_available

`src/types.rs`
Confidence level for a violation — how certain Neti is that this is a real problem. Defines domain data structures.
→ Exports: CheckReport, CommandResult, Confidence, FileReport, ScanReport, Violation, ViolationDetails, clean_file_count, command, duration_ms, error_count, exit_code, has_blocking_errors, has_errors, is_clean, is_small_codebase, label, new, output, passed, prefix, simple, stderr, stdout, suggestion_count, violation_count, warning_count, with_details

`src/verification/runner.rs`
Command execution and output capture. Supports application functionality.
→ Exports: run_commands

## Layer 3 -- Utilities

`src/analysis/patterns/logic_helpers.rs`
Shared helpers for L02/L03 logic pattern detection. Provides reusable helper functions.
→ Exports: can_find_local_declaration, decl_matches_variable, has_chunks_exact_context, has_explicit_guard, has_matching_parameter, is_index_variable, is_literal

`src/analysis/patterns/logic_proof_helpers.rs`
Helper routines for extracting and verifying array sizes in scope boundaries. Provides reusable helper functions.

`src/utils.rs`
Module providing `compute_sha256`. Provides reusable helper functions.
→ Exports: compute_sha256

## Layer 4 -- Tests

`src/analysis/checks/syntax_test.rs`
Orchestrates `crate`, `super`, `tree_sitter`. Verifies correctness.

`src/analysis/inspector.rs`
Inspection logic for scopes (Metrics application). Verifies correctness.
→ Exports: Inspector, inspect, new

`src/analysis/patterns/concurrency_lock_test.rs`
Orchestrates `super`, `tokio`, `tree_sitter`. Verifies correctness.

`src/analysis/patterns/idiomatic_i02_test.rs`
Orchestrates `Idx`, `super`, `tree_sitter`. Verifies correctness.

`src/analysis/patterns/logic_helpers_test.rs`
Orchestrates `super`, `tree_sitter`. Verifies correctness.

`src/analysis/patterns/logic_l03_test.rs`
Orchestrates `super`, `tree_sitter`. Verifies correctness.

`src/analysis/patterns/logic_proof_test.rs`
Orchestrates `super`, `tree_sitter`. Verifies correctness.

`src/analysis/patterns/performance_p01_test.rs`
Orchestrates `crate`, `super`, `tree_sitter`. Verifies correctness.

`src/analysis/patterns/performance_test_ctx.rs`
Test context detection for pattern detectors. Verifies correctness.
→ Exports: is_test_context

`src/analysis/patterns/security_x02_test.rs`
Orchestrates `super`, `tree_sitter`. Verifies correctness.

`src/graph/locality/tests.rs`
Integration tests for locality analysis — part 1. Verifies correctness.

`src/graph/locality/tests/part2.rs`
Integration tests for locality analysis — part 2. Verifies correctness.

`tests/check_json_test.rs`
Integration test: `neti check --json` must emit valid JSON to stdout. Verifies correctness.

