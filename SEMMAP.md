# project -- Semantic Map

**Purpose:** Architectural linter and context engineering

## Legend

`[ENTRY]` Application entry point

`[CORE]` Core business logic

`[TYPE]` Data structures and types

`[UTIL]` Utility functions

## Layer 0 -- Config

`Cargo.toml`
Rust package manifest and dependencies. Centralizes project configuration.

`slopchop.toml`
Configuration for slopchop. Centralizes project configuration.

`src/cli/config_ui/editor.rs`
Runs the interactive editor. Centralizes project configuration.
→ Exports: ConfigEditor, EditResult, EventResult, config, config_mut, items, new, run, run_config_editor, selected, set_modified, set_selected

`src/cli/config_ui/items.rs`
Configuration items that can be edited. Centralizes project configuration.
→ Exports: ConfigItem, all, cycle_enum, get_number, get_value, label, set_number, toggle_boolean

`src/cli/config_ui/logic.rs`
Runs the editor event loop. Centralizes project configuration.
→ Exports: move_selection, run_editor

`src/cli/config_ui/mod.rs`
Orchestrates `editor`. Centralizes project configuration.

`src/cli/config_ui/render.rs`
Renders the configuration UI  # Errors Returns error if terminal manipulation fails. Centralizes project configuration.
→ Exports: draw

`src/config/io.rs`
Saves the configuration to the file system. Centralizes project configuration.
→ Exports: apply_project_defaults, load_ignore_file, load_toml_config, parse_toml, process_ignore_line, save_to_file

`src/config/locality.rs`
Configuration for the Law of Locality enforcement. Centralizes project configuration.
→ Exports: LocalityConfig, is_enabled, is_error_mode, to_validator_config

`src/config/mod.rs`
Creates a new config and loads local settings (`slopchop.toml`, `.slopchopignore`). Centralizes project configuration.
→ Exports: load, load_local_config, new, parse_toml, process_ignore_line, save, save_to_file, validate

`src/config/types.rs`
Module providing `CommandEntry`, `Config`, `Preferences`. Centralizes project configuration.
→ Exports: CommandEntry, Config, Preferences, RuleConfig, SafetyConfig, SlopChopToml, into_vec

`src/graph/tsconfig.rs`
Parser for tsconfig.json / jsconfig.json path mappings. Centralizes project configuration.
→ Exports: TsConfig, load, resolve

## Layer 1 -- Core

`src/analysis/mod.rs`
Core analysis logic (The "Rule Engine"). Supports application functionality.

`src/analysis/patterns/mod.rs`
AST pattern detection for violations. Supports application functionality.
→ Exports: detect_all, get_capture_node

`src/apply/mod.rs`
Executes the apply operation based on user input. Supports application functionality.
→ Exports: print_result, process_input, run_apply, run_promote

`src/apply/verification/mod.rs`
Verification pipeline orchestration. Supports application functionality.
→ Exports: run_verification_pipeline

`src/bin/slopchop.rs`
Orchestrates `clap`, `colored`, `slopchop_core`. Defines command-line interface.

`src/cli/args.rs`
Run structural checks on the codebase. Defines command-line interface.
→ Exports: ApplyArgs, Cli, Commands

`src/cli/handlers/mod.rs`
Core analysis command handlers. Supports application functionality.
→ Exports: PackArgs, get_repo_root, handle_check, handle_map, handle_pack, handle_scan, handle_signatures

`src/cli/mod.rs`
CLI command handlers. Supports application functionality.

`src/clipboard/mod.rs`
Smartly copies text or file handles based on size. Supports application functionality.
→ Exports: copy_file_path, copy_to_clipboard, read_clipboard, smart_copy

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
Application entry point. Provides application entry point.

`src/mutate/mod.rs`
Cross-language mutation testing [EXPERIMENTAL]. Supports application functionality.
→ Exports: MutateOptions, MutateReport, run

`src/pack/mod.rs`
Markdown specification generated from doc comments. Defines command-line interface.
→ Exports: FocusContext, OutputFormat, PackOptions, generate_content, run, run_with_progress

`src/signatures/mod.rs`
Holographic signature map generator. Supports application functionality.
→ Exports: SignatureOptions, run

`src/spinner/mod.rs`
Triptych HUD (Head-Up Display) for process execution feedback. Supports application functionality.
→ Exports: start

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
Checks for syntax errors or missing nodes in the AST. Supports application functionality.
→ Exports: check_syntax

`src/analysis/cognitive.rs`
Cognitive Complexity metric implementation. Supports application functionality.
→ Exports: CognitiveAnalyzer, calculate

`src/analysis/deep.rs`
Deep analysis runner. Supports application functionality.
→ Exports: DeepAnalyzer, compute_violations, new

`src/analysis/engine.rs`
Main execution logic for the `SlopChop` analysis engine. Supports application functionality.
→ Exports: Engine, new, scan, scan_with_progress

`src/analysis/extract.rs`
Rust scope extraction logic (Structs/Enums/Fields). Supports application functionality.
→ Exports: RustExtractor, extract_scopes

`src/analysis/extract_impl.rs`
Rust impl/method extraction logic. Supports application functionality.
→ Exports: extract

`src/analysis/metrics.rs`
Calculates the nesting depth of a node. Supports application functionality.
→ Exports: calculate_complexity, calculate_max_depth, count_arguments

`src/analysis/patterns/concurrency.rs`
Concurrency pattern detection: C03, C04. Supports application functionality.
→ Exports: detect

`src/analysis/patterns/concurrency_lock.rs`
C03: `MutexGuard` held across `.await`. Supports application functionality.
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

`src/analysis/patterns/logic.rs`
Logic patterns: L02, L03. Supports application functionality.
→ Exports: detect

`src/analysis/patterns/performance.rs`
Performance anti-patterns: P01, P02, P04, P06. Supports application functionality.
→ Exports: detect

`src/analysis/patterns/resource.rs`
Resource patterns: R07 (missing flush). Supports application functionality.
→ Exports: detect

`src/analysis/patterns/security.rs`
Security patterns: X01, X02, X03. Supports application functionality.
→ Exports: detect

`src/analysis/patterns/semantic.rs`
Semantic patterns: M03, M04, M05. Supports application functionality.
→ Exports: detect

`src/analysis/patterns/state.rs`
State pattern detection: S01, S02, S03. Supports application functionality.
→ Exports: detect

`src/analysis/safety.rs`
Checks for unsafe blocks and ensures they have justification comments. Supports application functionality.
→ Exports: check_safety

`src/analysis/scope.rs`
Represents a cohesion and coupling scope (Class, Struct+Impl, Enum). Supports application functionality.
→ Exports: FieldInfo, Method, Scope, add_derive, add_field, add_method, derives, fields, has_behavior, has_derives, is_enum, methods, name, new, new_enum, row, validate_record

`src/analysis/structural.rs`
Structural metrics calculation (LCOM4, CBO, SFOUT, AHF). Supports application functionality.
→ Exports: ScopeMetrics, calculate_ahf, calculate_cbo, calculate_lcom4, calculate_max_sfout

`src/analysis/types.rs`
A single violation detected during analysis. Defines domain data structures.
→ Exports: CheckReport, CommandResult, FileReport, ScanReport, Violation, ViolationDetails, clean_file_count, has_errors, is_clean, is_small_codebase, simple, violation_count, with_details

`src/analysis/visitor.rs`
AST Visitor for analysis. Supports application functionality.
→ Exports: AstVisitor, extract_scopes, new

`src/analysis/worker.rs`
Worker module for file parsing and analysis. Supports application functionality.
→ Exports: is_ignored, scan_file

`src/apply/advisory.rs`
Threshold for triggering the high edit volume advisory. Supports application functionality.
→ Exports: maybe_print_edit_advisory

`src/apply/backup.rs`
Creates a backup of the files listed in the manifest. Supports application functionality.
→ Exports: cleanup_old_backups, create_backup, perform_rollback

`src/apply/blocks.rs`
Block creation and content sanitization for the XSC7XSC protocol. Supports application functionality.
→ Exports: clean_block_content, create_block, validate_path_keyword

`src/apply/executor.rs`
Handles the execution of apply actions with automatic branch management. Supports application functionality.
→ Exports: apply_to_stage_transaction, confirm, run_promote_standalone

`src/apply/manifest.rs`
Parses the delivery manifest body lines. Parses input into structured data.
→ Exports: parse_manifest, parse_manifest_body

`src/apply/messages.rs`
Finds the largest valid char boundary <= idx. Supports application functionality.
→ Exports: format_ai_rejection, generate_ai_feedback, print_ai_feedback, print_outcome

`src/apply/parser.rs`
Strict parser for the `SlopChop` XSC7XSC protocol. Parses input into structured data.
→ Exports: parse

`src/apply/patch.rs`
Surgical patch application logic. Supports application functionality.
→ Exports: apply, apply_with_options

`src/apply/process_runner.rs`
External command execution with streaming output. Supports application functionality.
→ Exports: CommandRunner, new, run

`src/apply/processor.rs`
Validates and applies a string payload containing a plan, manifest and files. Supports application functionality.
→ Exports: process_input, run_promote_standalone

`src/apply/report_writer.rs`
Logic for generating the verification report file. Supports application functionality.
→ Exports: write_check_report

`src/apply/types.rs`
Types for the apply module. Defines domain data structures.
→ Exports: ApplyContext, ApplyInput, ApplyOutcome, Block, CheckReport, CommandResult, FileContent, FileReport, ManifestEntry, Operation, ScanReport, Violation, ViolationDetails, clean_file_count, has_errors, is_clean, is_small_codebase, new, simple, violation_count, with_details

`src/apply/validator.rs`
Module providing `validate`. Supports application functionality.
→ Exports: validate

`src/apply/verification/report_display.rs`
Verification report display formatting. Supports application functionality.
→ Exports: print_report

`src/apply/writer.rs`
Writes changes (updates, new files, deletes) to disk atomically. Supports application functionality.
→ Exports: write_files

`src/branch.rs`
Git branch workflow for AI agents. Supports application functionality.
→ Exports: BranchResult, PromoteResult, abort, count_modified_files, init_branch, on_work_branch, promote, work_branch_name

`src/clean.rs`
Runs the clean command: removes context.txt and ensures gitignore. Supports application functionality.
→ Exports: run

`src/cli/apply_handler.rs`
Handler for the apply command. Supports application functionality.
→ Exports: handle_apply

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
→ Exports: aggregate_by_law, print

`src/cli/locality.rs`
Handler for locality scanning. Supports application functionality.
→ Exports: LocalityResult, check_locality_silent, handle_locality, is_locality_blocking, run_locality_check

`src/cli/mutate_handler.rs`
Handles the mutate command. Supports application functionality.
→ Exports: handle_mutate

`src/clipboard/linux.rs`
Copies the file at the given path to the clipboard. Supports application functionality.
→ Exports: copy_file_handle, perform_copy, perform_read

`src/clipboard/macos.rs`
Copies the file at the given path to the clipboard. Supports application functionality.
→ Exports: copy_file_handle, perform_copy, perform_read

`src/clipboard/platform.rs`
Platform-specific clipboard operations. Supports application functionality.

`src/clipboard/temp.rs`
Writes content to a temporary file. Supports application functionality.
→ Exports: cleanup_temp_files, write_to_temp

`src/clipboard/windows.rs`
Copies the file at the given path to the clipboard. Supports application functionality.
→ Exports: copy_file_handle, perform_copy, perform_read

`src/constants.rs`
Shared constants for file filtering and pattern matching. Supports application functionality.
→ Exports: should_prune

`src/detection.rs`
Detects build systems. Supports application functionality.
→ Exports: BuildSystemType, Detector, detect_build_systems, new

`src/discovery.rs`
Runs the file discovery pipeline. Parses input into structured data.
→ Exports: discover, group_by_directory

`src/error.rs`
Error handling - just use anyhow everywhere. Defines error types and handling.

`src/events.rs`
Machine-readable event logging for audit trails. Supports application functionality.
→ Exports: EventKind, EventLogger, SlopChopEvent, log, new

`src/exit.rs`
Standardized process exit codes for `SlopChop`. Supports application functionality.
→ Exports: SlopChopExit, code, exit

`src/graph/defs/extract.rs`
A symbol definition found in source code. Supports application functionality.
→ Exports: DefKind, Definition, extract

`src/graph/defs/queries.rs`
Module providing `DefExtractor`, `get_config`. Supports application functionality.
→ Exports: DefExtractor, get_config

`src/graph/imports.rs`
Extracts raw import strings from the given file content. Supports application functionality.
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
Resolves an import string to a likely file path on disk. Supports application functionality.
→ Exports: resolve

`src/lang.rs`
Module providing `Lang`, `QueryKind`, `from_ext`. Supports application functionality.
→ Exports: Lang, QueryKind, from_ext, grammar, q_complexity, q_defs, q_exports, q_imports, q_naming, q_skeleton, query, skeleton_replacement

`src/map.rs`
Repository map generation with tree-style visualization. Supports application functionality.
→ Exports: generate

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

`src/pack/docs.rs`
Logic for extracting documentation from source code. Supports application functionality.
→ Exports: extract_from_path

`src/pack/focus.rs`
Focus mode computation for foveal/peripheral packing. Supports application functionality.
→ Exports: compute_sets

`src/pack/formats.rs`
Packs files into the `SlopChop` format with focus awareness. Formats data for output.
→ Exports: pack_slopchop_focus, pack_spec

`src/pack/xml_format.rs`
XML output format logic. Formats data for output.
→ Exports: pack_xml_focus

`src/project.rs`
Detects project type from current directory. Supports application functionality.
→ Exports: ProjectType, Strictness, detect, detect_in, generate_toml, is_typescript, npx_cmd

`src/prompt.rs`
Current protocol version for AI compatibility tracking. Supports application functionality.
→ Exports: PromptGenerator, generate, generate_reminder, generate_short, new, wrap_header

`src/reporting.rs`
Console output formatting for scan results. Supports application functionality.
→ Exports: format_report_string, print_json, print_report

`src/signatures/docs.rs`
Doc comment extraction for holographic signatures. Supports application functionality.
→ Exports: expand_ranges_for_docs

`src/signatures/ordering.rs`
Topological ordering for holographic signatures. Supports application functionality.
→ Exports: topological_order

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
A single violation detected during analysis. Defines domain data structures.
→ Exports: CheckReport, CommandResult, FileReport, ScanReport, Violation, ViolationDetails, clean_file_count, has_errors, is_clean, is_small_codebase, simple, violation_count, with_details

## Layer 3 -- Utilities

`src/clipboard/utils.rs`
Pipes input text to a command's stdin. Provides reusable helper functions.
→ Exports: pipe_to_cmd

`src/utils.rs`
Computes SHA256 hash of content with normalized line endings. Provides reusable helper functions.
→ Exports: compute_sha256

## Layer 4 -- Tests

`src/analysis/inspector.rs`
Inspection logic for scopes (Metrics application). Verifies correctness.
→ Exports: Inspector, inspect, new

`src/apply/validator_tests.rs`
Orchestrates `super`. Verifies correctness.

`src/graph/locality/tests.rs`
Integration tests for locality analysis. Verifies correctness.

