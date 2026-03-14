# project -- Semantic Map

**Purpose:** architectural linter and code quality gate for CI

## Legend

`[ENTRY]` Application entry point

`[CORE]` Core business logic

`[TYPE]` Data structures and types

`[UTIL]` Utility functions

`[HOTSPOT]` High fan-in file imported by 4+ others - request this file early in any task

`[GLOBAL-UTIL]` High fan-in utility imported from 3+ distinct domains

`[DOMAIN-CONTRACT]` Shared contract imported mostly by one subsystem

`[ROLE:model]` Primary domain model or state-holding data structure.

`[ROLE:controller]` Coordinates commands, events, or request handling.

`[ROLE:rendering]` Produces visual output or drawing behavior.

`[ROLE:view]` Represents a reusable UI view or presentation component.

`[ROLE:dialog]` Implements dialog-oriented interaction flow.

`[ROLE:config]` Defines configuration loading or configuration schema behavior.

`[ROLE:os-integration]` Bridges the application to OS-specific APIs or services.

`[ROLE:utility]` Provides cross-cutting helper logic without owning core flow.

`[ROLE:bootstrap]` Initializes the application or wires subsystem startup.

`[ROLE:build-only]` Supports the build toolchain rather than runtime behavior.

`[COUPLING:pure]` Logic stays within the language/runtime without external surface coupling.

`[COUPLING:mixed]` Blends pure logic with side effects or boundary interactions.

`[COUPLING:ui-coupled]` Depends directly on UI framework, rendering, or windowing APIs.

`[COUPLING:os-coupled]` Depends directly on operating-system services or platform APIs.

`[COUPLING:build-only]` Only relevant during build, generation, or compilation steps.

`[BEHAVIOR:owns-state]` Maintains durable in-memory state for a subsystem.

`[BEHAVIOR:mutates]` Changes application or model state in response to work.

`[BEHAVIOR:renders]` Produces rendered output, drawing commands, or visual layout.

`[BEHAVIOR:dispatches]` Routes commands, events, or control flow to other units.

`[BEHAVIOR:observes]` Listens to callbacks, notifications, or external signals.

`[BEHAVIOR:persists]` Reads from or writes to durable storage.

`[BEHAVIOR:spawns-worker]` Creates background workers, threads, or async jobs.

`[BEHAVIOR:sync-primitives]` Coordinates execution with locks, channels, or wait primitives.

`[SURFACE:filesystem]` Touches filesystem paths, files, or directory traversal.

`[SURFACE:ntfs]` Uses NTFS-specific filesystem semantics or metadata.

`[SURFACE:win32]` Touches Win32 platform APIs or Windows-native handles.

`[SURFACE:shell]` Integrates with shell commands, shell UX, or command launch surfaces.

`[SURFACE:clipboard]` Reads from or writes to the system clipboard.

`[SURFACE:gdi]` Uses GDI drawing primitives or related graphics APIs.

`[SURFACE:control]` Represents or manipulates widget/control surfaces.

`[SURFACE:view]` Represents a view-level presentation surface.

`[SURFACE:dialog]` Represents a dialog/window interaction surface.

`[SURFACE:document]` Represents document-oriented editing or display surfaces.

`[SURFACE:frame]` Represents application frame/window chrome surfaces.

## Layer 0 -- Config

`Cargo.toml`
Workspace configuration.

`README.md`
Project overview and usage guide.

`SEMMAP.md`
Generated semantic map.

`neti.toml`
Configuration for neti.

`src/cli/config_ui/editor.rs`
Implements config editor.
Exports: run_config_editor, EditResult, EventResult, set_modified

`src/cli/config_ui/items.rs`
Configuration items that can be edited.
Exports: ConfigItem, cycle_enum, toggle_boolean, get_value

`src/cli/config_ui/logic.rs`
Processes editor.
Exports: move_selection, run_editor

`src/cli/config_ui/render.rs`
Implements draw functionality.
Exports: draw

`src/config/io.rs`
Gets the toml config.
Exports: apply_project_defaults, process_ignore_line, save_to_file, load_toml_config

`src/config/locality.rs`
Configuration for the Law of Locality enforcement.
Exports: is_error_mode, to_validator_config, LocalityConfig, is_enabled

`src/config/types.rs`
Implements command entry.
Exports: CommandEntry, NetiToml, into_vec, RuleConfig

`src/graph/tsconfig.rs`
Parser for tsconfig.json / jsconfig.json path mappings.
Exports: TsConfig, load, resolve
Touch: Contains inline Rust tests alongside runtime code.

`v0.1.8-focus.md`
Support file for v0.1.8-focus.

## Layer 1 -- Domain (Engine)

`src/analysis/aggregator.rs`
Aggregation logic for analysis results.
Exports: FileAnalysis, Aggregator, ingest, merge

`src/analysis/ast.rs`
Implements analysis result.
Exports: AnalysisResult, Analyzer, analyze

`src/analysis/checks.rs`
AST-based complexity and style checks.
Exports: CheckContext, check_banned, check_metrics, check_naming

`src/analysis/checks/banned.rs`
Banned construct checks (Law of Paranoia).
Exports: check_banned

`src/analysis/checks/naming.rs`
Function naming checks (Law of Complexity).
Exports: check_naming
Touch: Contains inline Rust tests alongside runtime code.

`src/analysis/deep.rs`
Deep analysis runner.
Exports: DeepAnalyzer, compute_violations

`src/analysis/engine.rs`
Main execution logic for the `Neti` analysis engine.
Exports: scan_with_progress, Engine, scan

`src/analysis/extract.rs`
Rust scope extraction logic (Structs/Enums/Fields).
Exports: RustExtractor, extract_scopes

`src/analysis/inspector.rs`
Inspection logic for scopes (Metrics application).
Exports: Inspector, inspect, new

`src/analysis/patterns/concurrency.rs`
Concurrency pattern detection: C03, C04.
Exports: detect_c03, detect_c04, detect

`src/analysis/patterns/concurrency_lock.rs`
C03: `MutexGuard` held across `.await`  Severity Tiers  **Sync mutex (std::sync::Mutex, parking_lot::Mutex) — HIGH confidence** Holding a sync guard across `.await` blocks the OS thread, starving the executor, and can deadlock if another task tries to acquire the same lock.
Exports: detect_c03
Touch: Contains inline Rust tests alongside runtime code.

`src/analysis/patterns/concurrency_sync.rs`
C04: Undocumented synchronization primitives.
Exports: detect_c04

`src/analysis/patterns/db_patterns.rs`
Database anti-patterns: P03 (N+1 queries).
Exports: detect

`src/analysis/patterns/idiomatic.rs`
Idiomatic patterns: I01, I02.
Exports: detect

`src/analysis/patterns/idiomatic_i01.rs`
I01: Manual `From` implementations that could use `derive_more::From`.
Touch: Contains inline Rust tests alongside runtime code.

`src/analysis/patterns/idiomatic_i02.rs`
I02: Duplicate match arm bodies that could be combined using `A | B => body`.
Touch: Contains inline Rust tests alongside runtime code.

`src/analysis/patterns/logic.rs`
Logic boundary patterns: L02 (off-by-one risk), L03 (unchecked index).
Exports: detect

`src/analysis/patterns/logic_l02.rs`
L02: Boundary uses `<=`/`>=` with `.len()` — possible off-by-one.
Touch: Contains inline Rust tests alongside runtime code.

`src/analysis/patterns/logic_l03.rs`
L03: Unchecked index access (`[0]`, `.first().unwrap()`, etc.).
Touch: Contains inline Rust tests alongside runtime code.

`src/analysis/patterns/logic_proof.rs`
Fixed-size array proof helpers for L03.
Exports: is_fixed_size_array_access, extract_receiver
Touch: Contains inline Rust tests alongside runtime code.

`src/analysis/patterns/performance.rs`
Performance anti-patterns: P01, P02, P04, P06  Escalation Philosophy  P01/P02 must only fire when we can make a reasonable argument that the allocation is *material*.
Exports: detect

`src/analysis/patterns/performance_p02.rs`
P02: String conversion (`.to_string()` / `.to_owned()`) inside a loop.
Touch: Contains inline Rust tests alongside runtime code.

`src/analysis/patterns/resource.rs`
Resource patterns: R07 (missing flush).
Exports: detect
Touch: Contains inline Rust tests alongside runtime code.

`src/analysis/patterns/security.rs`
Security patterns: X01 (SQL injection), X02 (command injection), X03 (hardcoded secrets).
Exports: detect

`src/analysis/patterns/security_x01.rs`
X01: SQL Injection — format!() used to build SQL strings.
Touch: Contains inline Rust tests alongside runtime code.

`src/analysis/patterns/security_x02.rs`
X02: Command / Shell Injection.
Touch: Contains inline Rust tests alongside runtime code.

`src/analysis/patterns/security_x03.rs`
X03: Hardcoded secrets (keys, tokens, passwords) in let/const bindings.

`src/analysis/patterns/semantic.rs`
Semantic patterns: M03, M04, M05.
Exports: detect
Touch: Contains inline Rust tests alongside runtime code.

`src/analysis/patterns/state.rs`
State pattern detection: S01, S02, S03.
Exports: detect

`src/analysis/safety.rs`
Validates safety.
Exports: check_safety

`src/analysis/scope.rs`
Implements add method.
Exports: validate_record, FieldInfo, has_behavior, is_enum

`src/analysis/structural.rs`
Structural metrics calculation (LCOM4, CBO, SFOUT, AHF).
Exports: calculate_max_sfout, ScopeMetrics, calculate_ahf, calculate_cbo

`src/analysis/visitor.rs`
AST Visitor for analysis.
Exports: AstVisitor, extract_scopes

`src/analysis/worker.rs`
Worker module for file parsing and analysis.
Exports: is_ignored, scan_file

`src/branch.rs`
Git branch workflow for AI agents.
Exports: count_modified_files, on_work_branch, work_branch_name, PromoteResult
Touch: Contains inline Rust tests alongside runtime code.

`src/clean.rs`
Implements run functionality.
Exports: run

`src/cli/audit.rs`
CLI handlers for the consolidation audit command.
Exports: AuditCliOptions, handle

`src/cli/dispatch.rs`
Command dispatch logic extracted from binary to reduce main function size.
Exports: execute

`src/cli/git_ops.rs`
Handlers for Git-based workflow operations (branch, promote, abort).
Exports: handle_abort, handle_branch, handle_promote

`src/cli/handlers/check_report.rs`
Report building and scorecard display for `neti check`.
Exports: build_report_text, print_commands_scorecard, print_locality_scorecard

`src/cli/locality.rs`
Handler for locality scanning.
Exports: is_locality_blocking, check_locality_silent, run_locality_check, LocalityResult

`src/cli/mutate_handler.rs`
Processes mutate.
Exports: handle_mutate

`src/constants.rs`
Shared constants for file filtering and pattern matching.
Exports: should_prune

`src/detection.rs`
Detects build systems.
Exports: BuildSystemType, detect_build_systems, Detector

`src/events.rs`
Machine-readable event logging for audit trails.
Exports: EventKind, EventLogger, NetiEvent

`src/exit.rs`
Standardized process exit codes for `Neti`. [HOTSPOT]
Exports: NetiExit, code

`src/file_class.rs`
File classification: distinguishes source code from config, assets, and data.
Exports: FileKind, is_governed, secrets_applicable, classify
Touch: Contains inline Rust tests alongside runtime code.

`src/graph/defs/extract.rs`
Implements def kind.
Exports: DefKind, Definition
Touch: Contains inline Rust tests alongside runtime code.

`src/graph/defs/queries.rs`
Gets the config.
Exports: DefExtractor, get_config

`src/graph/imports.rs`
Implements extract functionality.
Exports: extract
Touch: Contains inline Rust tests alongside runtime code.

`src/graph/locality/analysis/metrics.rs`
Finds hub candidates.
Exports: compute_module_coupling, GodModuleInfo, find_god_modules, find_hub_candidates

`src/graph/locality/analysis/violations.rs`
Categories of locality violations.
Exports: CategorizedViolation, ViolationKind, categorize_violation, description

`src/graph/locality/classifier.rs`
Node classification based on coupling metrics.
Exports: ClassifierConfig, classify
Touch: Contains inline Rust tests alongside runtime code.

`src/graph/locality/coupling.rs`
Afferent and Efferent coupling computation.
Exports: compute_coupling
Touch: Contains inline Rust tests alongside runtime code.

`src/graph/locality/cycles.rs`
Cycle detection for the Law of Locality.
Exports: detect_cycles
Touch: Contains inline Rust tests alongside runtime code.

`src/graph/locality/distance.rs`
Dependency Distance calculator via Lowest Common Ancestor (LCA).
Exports: compute_distance, find_lca
Touch: Contains inline Rust tests alongside runtime code.

`src/graph/locality/edges.rs`
Edge collection for locality analysis.
Exports: collect

`src/graph/locality/exemptions.rs`
Smart structural exemptions for Rust module patterns.
Exports: is_structural_pattern
Touch: Contains inline Rust tests alongside runtime code.

`src/graph/locality/layers.rs`
Layer inference for the Law of Locality.
Exports: check_layer_violation, infer_layers

`src/graph/locality/report.rs`
Rich output formatting for locality analysis.
Exports: print_full_report

`src/graph/locality/types.rs`
Core types for the Law of Locality enforcement system. [TYPE]
Exports: allows_far_deps, routes_to_hub, NodeIdentity, PassReason

`src/graph/locality/validator.rs`
The Universal Locality Algorithm - Judgment Pass.
Exports: ValidationReport, ValidatorConfig, check_cohesion, is_clean

`src/graph/rank/builder.rs`
Graph construction logic: extraction and edge building.
Exports: rebuild_topology, GraphData, build_data

`src/graph/rank/graph.rs`
The dependency graph structure and query interface.
Exports: is_hub, ranked_files, RepoGraph, graph_tags

`src/graph/rank/pagerank.rs`
PageRank` algorithm implementation for file ranking.
Exports: compute
Touch: Contains inline Rust tests alongside runtime code.

`src/graph/rank/queries.rs`
Gets the ranked files.
Exports: get_graph_tags, get_ranked_files, collect_dependencies, collect_dependents

`src/graph/rank/tags.rs`
Tag types representing definitions and references.
Exports: Tag, TagKind

`src/graph/resolver.rs`
Implements resolve functionality.
Exports: resolve

`src/mutate/discovery.rs`
Discovers mutation points in source files using tree-sitter.
Exports: discover_mutations
Touch: Contains inline Rust tests alongside runtime code.

`src/mutate/mutations.rs`
Mutation types and application logic.
Exports: MutationKind, MutationPoint, apply_mutation, get_mutation
Touch: Contains inline Rust tests alongside runtime code.

`src/mutate/runner.rs`
Parallel mutation test runner.
Exports: RunnerConfig, run_mutations, MutationResult, MutationSummary

`src/project.rs`
Detects project type from current directory.
Exports: ProjectType, generate_toml, is_typescript, npx_cmd

`src/reporting.rs`
Console output formatting for scan results. [HOTSPOT] [GLOBAL-UTIL]
Exports: build_rich_report, format_report_string, print_json, print_report

`src/reporting/console.rs`
Prints a formatted scan report to stdout with confidence tiers and deduplication.
Exports: print_report

`src/reporting/guidance.rs`
Static educational guidance per rule code.

`src/reporting/rich.rs`
Formats report string for output.
Exports: build_rich_report, format_report_string

`src/reporting/shared.rs`
Implements shared functionality.

`src/skeleton.rs`
Reduces code to its structural skeleton (signatures only).
Exports: clean
Touch: Contains inline Rust tests alongside runtime code.

`src/spinner/client.rs`
Client for sending updates to the spinner.
Exports: set_macro_step, set_micro_status, step_micro_progress, SpinnerClient

`src/spinner/controller.rs`
Lifecycle controller for the spinner thread.
Exports: SpinnerController, stop

`src/spinner/handle.rs`
Thread management for the spinner.
Exports: SpinnerHandle, spawn, stop

`src/spinner/safe_hud.rs`
Thread-safe wrapper for HUD state.
Exports: SafeHud, completion_info, modify, snapshot

`src/spinner/state.rs`
HUD state management.
Exports: step_micro_progress, set_macro_step, set_micro_status, completion_info

`src/tokens.rs`
The tokenizer encoding (`cl100k_base`, used by GPT-4/3.5-turbo).
Exports: exceeds_limit, is_available, Tokenizer, count

`src/types/command.rs`
Result of an external command execution. [TYPE]
Exports: CommandResult, duration_ms, exit_code, error_count
Touch: Contains inline Rust tests alongside runtime code.

`src/types/locality.rs`
Types for locality (Law of Locality) reporting. [TYPE]
Exports: LocalityReport, LocalityViolation

`src/verification/runner.rs`
Command execution and output capture.
Exports: run_commands
Touch: Contains inline Rust tests alongside runtime code.

## Layer 2 -- Adapters / Infra

`src/analysis/checks/complexity.rs`
Complexity metrics checks (Law of Complexity). [CORE]
Exports: check_metrics

`src/analysis/checks/syntax.rs`
AST-level syntax error and malformed node detection. [CORE]
Exports: check_syntax
Touch: Contains inline Rust tests alongside runtime code.

`src/analysis/cognitive.rs`
Cognitive Complexity metric implementation. [CORE]
Exports: CognitiveAnalyzer, calculate
Touch: Contains inline Rust tests alongside runtime code.

`src/analysis/extract_impl.rs`
Rust impl/method extraction logic. [CORE]
Exports: extract

`src/analysis/metrics.rs`
Implements calculate complexity. [CORE]
Exports: calculate_max_depth, count_arguments, calculate_complexity

`src/analysis/patterns/logic_helpers.rs`
Shared helpers for L02/L03 logic pattern detection. [UTIL]
Exports: can_find_local_declaration, has_chunks_exact_context, decl_matches_variable, has_explicit_guard
Touch: Contains inline Rust tests alongside runtime code.

`src/analysis/patterns/logic_proof_helpers.rs`
Helper routines for extracting and verifying array sizes in scope boundaries. [UTIL]

`src/analysis/patterns/performance_p01.rs`
P01: `.clone()` inside a loop. [CORE]
Touch: Contains inline Rust tests alongside runtime code.

`src/analysis/patterns/performance_p04p06.rs`
P04: Nested loop (O(n²)) and P06: linear search inside loop. [CORE]
Touch: Contains inline Rust tests alongside runtime code.

`src/cli/handlers/scan_report.rs`
Scan report display formatting. [CORE]
Exports: aggregate_by_law, build_summary_string, print

`src/discovery.rs`
Provides shared discovery used across multiple domains. [UTIL] [HOTSPOT] [GLOBAL-UTIL]
Exports: group_by_directory, discover

`src/lang.rs`
Provides shared lang used across multiple domains. [HOTSPOT] [GLOBAL-UTIL]
Exports: from_semantic_language, QueryKind, from_ext, skeleton_replacement
Touch: Contains inline Rust tests alongside runtime code.

`src/lang_queries.rs`
Implements lang queries.

`src/mutate/report.rs`
Report formatting for mutation test results. [CORE]
Exports: format_json, format_progress, format_summary, format_survivors

`src/spinner/render.rs`
HUD rendering logic. [UTIL]
Exports: run_hud_loop

`src/utils.rs`
Implements compute sha256. [UTIL]
Exports: compute_sha256

## Layer 3 -- App / Entrypoints

`src/analysis/mod.rs`
Core analysis logic (The "Rule Engine"). [ENTRY] [HOTSPOT] [GLOBAL-UTIL]
Exports: FileAnalysis, extract_impl, Engine, aggregator

`src/analysis/patterns/mod.rs`
AST pattern detection for violations. [ENTRY]
Exports: get_capture_node, performance_test_ctx, db_patterns, detect_all
Touch: Contains inline Rust tests alongside runtime code.

`src/bin/neti.rs`
Implements neti functionality.

`src/cli/args.rs`
Implements Cli functionality.
Exports: Cli, Commands

`src/cli/config_ui/mod.rs`
Re-exports the public API surface. [ENTRY]
Exports: run_config_editor, items, logic, render

`src/cli/handlers/mod.rs`
Core analysis command handlers. [CORE]
Exports: get_repo_root, handle_check, scan_report, handle_scan

`src/cli/mod.rs`
CLI command handlers. [ENTRY]
Exports: config_ui, git_ops, mutate_handler, args

`src/config/mod.rs`
Provides shared mod used across multiple domains. [ENTRY] [HOTSPOT] [GLOBAL-UTIL]
Exports: process_ignore_line, save_to_file, load_local_config, parse_toml

`src/graph/defs/mod.rs`
Extracts symbol DEFINITIONS from source files using tree-sitter. [ENTRY]

`src/graph/locality/analysis/mod.rs`
Deep topology analysis: categorize violations, find patterns, suggest fixes. [ENTRY]
Exports: TopologyAnalysis, analyze, metrics, violations

`src/graph/locality/mod.rs`
Law of Locality enforcement for topological integrity. [ENTRY]
Exports: is_structural_pattern, collect_edges, compute_coupling, compute_distance
Touch: Contains inline Rust tests alongside runtime code.

`src/graph/mod.rs`
Re-exports the public API surface. [ENTRY] [HOTSPOT] [GLOBAL-UTIL]
Exports: defs, imports, locality, rank

`src/graph/rank/mod.rs`
Orchestrates graph construction and ranking. [ENTRY]
Exports: focus_on, GraphEngine, RepoGraph, builder

`src/lib.rs`
Re-exports the public API surface. [ENTRY]
Exports: file_class, omni_ast, analysis, branch

`src/main.rs`
Placeholder file. [ENTRY]

`src/mutate/mod.rs`
Cross-language mutation testing [EXPERIMENTAL]. [ENTRY] [HOTSPOT] [GLOBAL-UTIL]
Exports: MutateOptions, MutateReport, discovery, mutations

`src/spinner/mod.rs`
Triptych HUD (Head-Up Display) for process execution feedback. [ENTRY]
Exports: safe_hud, SpinnerClient, SpinnerController, render

`src/types/mod.rs`
Confidence level for a violation — how certain Neti is that this is a real problem. [TYPE] [HOTSPOT] [GLOBAL-UTIL]
Exports: is_small_codebase, has_blocking_errors, clean_file_count, CommandResult

`src/verification/mod.rs`
External command verification pipeline. [ENTRY]
Exports: CommandResult, VerificationReport, failed_count, passed_count

## Layer 4 -- Tests

`src/analysis/checks/syntax_test.rs`
Tests for crate.

`src/analysis/patterns/performance_test_ctx.rs`
Test context detection for pattern detectors.
Exports: is_test_context

`src/analysis/patterns/` (7 files: 7 .rs)
Representative: src/analysis/patterns/concurrency_lock_test.rs, src/analysis/patterns/idiomatic_i02_test.rs

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

## Subprojects

### omni-ast

`omni-ast/Cargo.toml`
Workspace configuration.

`omni-ast/src/harvester_signatures.rs`
Implements harvester signatures.

`omni-ast/src/semantics.rs`
Implements semantic language.
Exports: LangSemantics, from_ext, SemanticContext, SemanticLanguage

`omni-ast/src/semantics_concurrency_queries.rs`
Implements semantics concurrency queries.

`omni-ast/src/semantics_engine.rs`
Implements shared semantics.
Exports: from_source, with_path, SharedSemantics, semantics_for

`omni-ast/src/semantics_logic_queries.rs`
Implements semantics logic queries.

`omni-ast/src/semantics_logic_tables.rs`
Implements semantics logic tables.

`omni-ast/src/semantics_queries.rs`
Implements semantics queries.
Touch: Contains inline Rust tests alongside runtime code.

`omni-ast/src/semantics_tables.rs`
Implements semantics tables.

`omni-ast/src/taxonomy.rs`
Stage 3 semantic badge evaluation.
Exports: SemanticBadges, load_taxonomy, evaluate, Taxonomy
Touch: Contains inline Rust tests alongside runtime code.

`omni-ast/src/types.rs`
Implements dep kind. [TYPE]
Exports: DepKind

`omni-ast/src/doc_extractor.rs`
Extracts documentation comments from source files.
Exports: extract_doc_comment_for_file, collapse_doc_lines, extract_doc_comment

`omni-ast/src/doc_filter.rs`
Heuristics to reject item-level doc comments mistaken for module docs.
Exports: looks_like_item_doc

`omni-ast/src/harvester.rs`
Stage 1 semantic signal harvesting. [UTIL]
Exports: SemanticFingerprint, parse_config, with_exports
Touch: Contains inline Rust tests alongside runtime code.

`omni-ast/src/harvester_tree.rs`
Implements harvester tree. [CORE]

`omni-ast/src/language/cpp.rs`
C and C++ import, doc, and export extraction. [UTIL]
Exports: is_header_path, extract_import_strings, primary_symbol, extract_doc

`omni-ast/src/language/cpp_includes.rs`
Parses imports.
Exports: is_build_only_path, extract_imports

`omni-ast/src/language/go.rs`
Parses import strings.
Exports: extract_import_strings, extract_doc, extract_imports

`omni-ast/src/language/javascript.rs`
JavaScript/TypeScript import extraction and path resolution.
Exports: extract_import_strings, extract_doc, extract_imports

`omni-ast/src/language/javascript/monorepo.rs`
Monorepo-aware bare import resolution for JavaScript/TypeScript.
Exports: collect_package_roots, resolve_bare

`omni-ast/src/language/rust.rs`
Rust import extraction and dependency analysis. [UTIL]
Exports: has_inline_tests, extract_crate_imports, extract_mod_declarations, extract_modules

`omni-ast/src/swum/splitter.rs`
Identifier splitting for SWUM.
Exports: split_identifier

`omni-ast/src/swum/verb_patterns.rs`
Verb pattern expansion for SWUM.
Exports: expand_verb_pattern

`omni-ast/src/taxonomy_rules.rs`
Implements Taxonomy functionality. [UTIL]
Exports: Taxonomy

`omni-ast/src/language/mod.rs`
Implements resolve semantic exports. [ENTRY]
Exports: extract_doc_comment_for_file, has_rust_inline_tests, extract_import_strings, resolve_primary_symbol
Touch: Contains inline Rust tests alongside runtime code.

`omni-ast/src/language/python.rs`
Python import extraction and dependency analysis.
Exports: extract_import_strings, extract_doc, extract_imports

`omni-ast/src/lib.rs`
Re-exports the public API surface. [ENTRY]
Exports: DepKind, harvester, language, semantics

`omni-ast/src/swum/mod.rs`
SWUM (Software Word Usage Model) for identifier expansion. [ENTRY]
Exports: expand_verb_pattern, summarize_exports, split_identifier, expand_identifier


## DependencyGraph

```yaml
DependencyGraph:
  # --- Entrypoints ---
  main.rs, neti.rs:
    Imports: []
    ImportedBy: []
  src/lib.rs:
    Imports: [branch.rs, clean.rs, cli/mod.rs, config/mod.rs, constants.rs, detection.rs, events.rs, exit.rs, file_class.rs, graph/mod.rs, lang.rs, mutate/mod.rs, project.rs, reporting.rs, skeleton.rs, spinner/mod.rs, src/analysis/mod.rs, src/discovery.rs, tokens.rs, types/mod.rs, utils.rs, verification/mod.rs]
    ImportedBy: []
  # --- High Fan-In Hotspots ---
  config/mod.rs:
    Imports: [config/locality.rs, config/types.rs, constants.rs, io.rs, types/mod.rs]
    ImportedBy: [ast.rs, checks.rs, cli/locality.rs, complexity.rs, config_ui/logic.rs, config_ui/render.rs, deep.rs, editor.rs, engine.rs, handlers/mod.rs, imports.rs, inspector.rs, items.rs, mutate/mod.rs, safety.rs, src/discovery.rs, src/lib.rs, syntax_test.rs, verification/mod.rs, worker.rs]
  exit.rs:
    Imports: []
    ImportedBy: [cli/locality.rs, dispatch.rs, git_ops.rs, handlers/mod.rs, mutate_handler.rs, src/lib.rs]
  graph/mod.rs:
    Imports: [defs/mod.rs, imports.rs, locality/mod.rs, rank/mod.rs, resolver.rs, tsconfig.rs]
    ImportedBy: [builder.rs, cli/locality.rs, config/locality.rs, edges.rs, locality/analysis/metrics.rs, locality/analysis/mod.rs, rank/queries.rs, resolver.rs, src/lib.rs, violations.rs]
  lang.rs:
    Imports: []
    ImportedBy: [ast.rs, defs/extract.rs, defs/queries.rs, imports.rs, mutate/discovery.rs, patterns/mod.rs, skeleton.rs, src/lib.rs, syntax_test.rs, visitor.rs, worker.rs]
  mutate/mod.rs:
    Imports: [config/mod.rs, mutate/discovery.rs, mutate/report.rs, mutate/runner.rs, mutations.rs, project.rs, src/discovery.rs]
    ImportedBy: [mutate/discovery.rs, mutate/report.rs, mutate/runner.rs, mutate_handler.rs, src/lib.rs]
  reporting.rs:
    Imports: [console.rs, guidance.rs, rich.rs, shared.rs]
    ImportedBy: [check_report.rs, console.rs, handlers/mod.rs, rich.rs, src/lib.rs]
  src/analysis/mod.rs:
    Imports: [aggregator.rs, analysis/extract.rs, ast.rs, checks.rs, cognitive.rs, deep.rs, engine.rs, extract_impl.rs, inspector.rs, patterns/mod.rs, safety.rs, scope.rs, src/analysis/metrics.rs, structural.rs, visitor.rs, worker.rs]
    ImportedBy: [handlers/mod.rs, layers.rs, locality/report.rs, safety.rs, scan_report.rs, src/lib.rs, types/mod.rs]
  src/discovery.rs:
    Imports: [config/mod.rs, constants.rs]
    ImportedBy: [cli/locality.rs, handlers/mod.rs, mutate/mod.rs, src/lib.rs]
  types/mod.rs:
    Imports: [command.rs, src/analysis/mod.rs, types/locality.rs]
    ImportedBy: [aggregator.rs, ast.rs, banned.rs, check_report.rs, classifier.rs, cli/locality.rs, complexity.rs, concurrency.rs, concurrency_lock.rs, concurrency_sync.rs, config/mod.rs, console.rs, coupling.rs, db_patterns.rs, deep.rs, engine.rs, handlers/mod.rs, idiomatic.rs, idiomatic_i01.rs, idiomatic_i02.rs, inspector.rs, io.rs, layers.rs, logic_l02.rs, logic_l03.rs, naming.rs, patterns/logic.rs, patterns/mod.rs, patterns/state.rs, performance.rs, performance_p01.rs, performance_p01_test.rs, performance_p02.rs, performance_p04p06.rs, resource.rs, rich.rs, safety.rs, scan_report.rs, security.rs, security_x01.rs, security_x02.rs, security_x03.rs, semantic.rs, shared.rs, src/lib.rs, syntax.rs, validator.rs, verification/mod.rs, verification/runner.rs, worker.rs]
  # --- Layer 0 -- Config ---
  Cargo.toml, README.md, SEMMAP.md, neti.toml, v0.1.8-focus.md:
    Imports: []
    ImportedBy: []
  config/locality.rs:
    Imports: [graph/mod.rs]
    ImportedBy: [config/mod.rs]
  config/types.rs:
    Imports: []
    ImportedBy: [config/mod.rs]
  config_ui/logic.rs, config_ui/render.rs, editor.rs, items.rs:
    Imports: [config/mod.rs]
    ImportedBy: [config_ui/mod.rs]
  io.rs:
    Imports: [project.rs, types/mod.rs]
    ImportedBy: [config/mod.rs]
  tsconfig.rs:
    Imports: []
    ImportedBy: [graph/mod.rs]
  # --- Layer 1 -- Domain (Engine) ---
  aggregator.rs:
    Imports: [types/mod.rs]
    ImportedBy: [src/analysis/mod.rs]
  analysis/extract.rs, scope.rs, structural.rs:
    Imports: []
    ImportedBy: [src/analysis/mod.rs]
  ast.rs:
    Imports: [config/mod.rs, lang.rs, types/mod.rs]
    ImportedBy: [src/analysis/mod.rs]
  audit.rs:
    Imports: []
    ImportedBy: []
  banned.rs, naming.rs:
    Imports: [types/mod.rs]
    ImportedBy: [checks.rs]
  branch.rs:
    Imports: []
    ImportedBy: [git_ops.rs, src/lib.rs]
  builder.rs, rank/queries.rs:
    Imports: [graph/mod.rs]
    ImportedBy: [rank/mod.rs]
  check_report.rs:
    Imports: [cli/mod.rs, reporting.rs, types/mod.rs, verification/mod.rs]
    ImportedBy: [handlers/mod.rs]
  checks.rs:
    Imports: [banned.rs, complexity.rs, config/mod.rs, naming.rs, syntax.rs]
    ImportedBy: [src/analysis/mod.rs]
  classifier.rs, coupling.rs, validator.rs:
    Imports: [types/mod.rs]
    ImportedBy: [locality/mod.rs]
  clean.rs, detection.rs, events.rs:
    Imports: []
    ImportedBy: [src/lib.rs]
  cli/locality.rs:
    Imports: [config/mod.rs, exit.rs, graph/mod.rs, src/discovery.rs, types/mod.rs]
    ImportedBy: [cli/mod.rs]
  client.rs, controller.rs, handle.rs, safe_hud.rs, spinner/state.rs:
    Imports: []
    ImportedBy: [spinner/mod.rs]
  command.rs, types/locality.rs:
    Imports: []
    ImportedBy: [types/mod.rs]
  concurrency.rs, concurrency_lock.rs, concurrency_sync.rs, db_patterns.rs, idiomatic.rs, patterns/logic.rs, patterns/state.rs, performance.rs, resource.rs, security.rs, semantic.rs:
    Imports: [types/mod.rs]
    ImportedBy: [patterns/mod.rs]
  console.rs, rich.rs:
    Imports: [reporting.rs, types/mod.rs]
    ImportedBy: [reporting.rs]
  constants.rs:
    Imports: []
    ImportedBy: [config/mod.rs, src/discovery.rs, src/lib.rs]
  cycles.rs, distance.rs, exemptions.rs, locality/types.rs:
    Imports: []
    ImportedBy: [locality/mod.rs]
  deep.rs, engine.rs, inspector.rs:
    Imports: [config/mod.rs, types/mod.rs]
    ImportedBy: [src/analysis/mod.rs]
  defs/extract.rs, defs/queries.rs:
    Imports: [lang.rs]
    ImportedBy: [defs/mod.rs]
  dispatch.rs:
    Imports: [exit.rs]
    ImportedBy: [cli/mod.rs]
  edges.rs:
    Imports: [graph/mod.rs]
    ImportedBy: [locality/mod.rs]
  file_class.rs, tokens.rs:
    Imports: []
    ImportedBy: [src/lib.rs, worker.rs]
  git_ops.rs:
    Imports: [branch.rs, exit.rs]
    ImportedBy: [cli/mod.rs]
  graph.rs, pagerank.rs, tags.rs:
    Imports: []
    ImportedBy: [rank/mod.rs]
  guidance.rs:
    Imports: []
    ImportedBy: [reporting.rs]
  idiomatic_i01.rs, idiomatic_i02.rs, logic_l02.rs, logic_l03.rs, performance_p02.rs, security_x01.rs, security_x02.rs, security_x03.rs:
    Imports: [types/mod.rs]
    ImportedBy: []
  imports.rs:
    Imports: [config/mod.rs, lang.rs]
    ImportedBy: [graph/mod.rs]
  layers.rs:
    Imports: [src/analysis/mod.rs, types/mod.rs]
    ImportedBy: [locality/mod.rs]
  locality/analysis/metrics.rs, violations.rs:
    Imports: [graph/mod.rs]
    ImportedBy: [locality/analysis/mod.rs]
  locality/report.rs:
    Imports: [src/analysis/mod.rs]
    ImportedBy: [locality/mod.rs]
  logic_proof.rs:
    Imports: []
    ImportedBy: [patterns/mod.rs]
  mutate/discovery.rs:
    Imports: [lang.rs, mutate/mod.rs]
    ImportedBy: [mutate/mod.rs]
  mutate/runner.rs:
    Imports: [mutate/mod.rs]
    ImportedBy: [mutate/mod.rs]
  mutate_handler.rs:
    Imports: [cli/mod.rs, exit.rs, mutate/mod.rs]
    ImportedBy: [cli/mod.rs]
  mutations.rs:
    Imports: []
    ImportedBy: [mutate/mod.rs]
  project.rs:
    Imports: []
    ImportedBy: [io.rs, mutate/mod.rs, src/lib.rs]
  resolver.rs:
    Imports: [graph/mod.rs]
    ImportedBy: [graph/mod.rs]
  safety.rs:
    Imports: [config/mod.rs, src/analysis/mod.rs, types/mod.rs]
    ImportedBy: [src/analysis/mod.rs]
  shared.rs:
    Imports: [types/mod.rs]
    ImportedBy: [reporting.rs]
  skeleton.rs:
    Imports: [lang.rs]
    ImportedBy: [src/lib.rs]
  verification/runner.rs:
    Imports: [types/mod.rs]
    ImportedBy: [verification/mod.rs]
  visitor.rs:
    Imports: [lang.rs]
    ImportedBy: [src/analysis/mod.rs]
  worker.rs:
    Imports: [config/mod.rs, file_class.rs, lang.rs, tokens.rs, types/mod.rs]
    ImportedBy: [src/analysis/mod.rs]
  # --- Layer 2 -- Adapters / Infra ---
  cognitive.rs, extract_impl.rs, src/analysis/metrics.rs:
    Imports: []
    ImportedBy: [src/analysis/mod.rs]
  complexity.rs:
    Imports: [config/mod.rs, types/mod.rs]
    ImportedBy: [checks.rs]
  lang_queries.rs, logic_proof_helpers.rs:
    Imports: []
    ImportedBy: []
  logic_helpers.rs:
    Imports: []
    ImportedBy: [patterns/mod.rs]
  mutate/report.rs:
    Imports: [mutate/mod.rs]
    ImportedBy: [mutate/mod.rs]
  performance_p01.rs, performance_p04p06.rs:
    Imports: [types/mod.rs]
    ImportedBy: []
  scan_report.rs:
    Imports: [src/analysis/mod.rs, types/mod.rs]
    ImportedBy: [handlers/mod.rs]
  spinner/render.rs:
    Imports: []
    ImportedBy: [spinner/mod.rs]
  syntax.rs:
    Imports: [types/mod.rs]
    ImportedBy: [checks.rs]
  utils.rs:
    Imports: []
    ImportedBy: [src/lib.rs]
  # --- Layer 3 -- App / Entrypoints ---
  args.rs:
    Imports: []
    ImportedBy: [cli/mod.rs]
  cli/mod.rs:
    Imports: [args.rs, cli/locality.rs, config_ui/mod.rs, dispatch.rs, git_ops.rs, handlers/mod.rs, mutate_handler.rs]
    ImportedBy: [check_report.rs, mutate_handler.rs, src/lib.rs]
  config_ui/mod.rs:
    Imports: [config_ui/logic.rs, config_ui/render.rs, editor.rs, items.rs]
    ImportedBy: [cli/mod.rs]
  defs/mod.rs:
    Imports: [defs/extract.rs, defs/queries.rs]
    ImportedBy: [graph/mod.rs]
  handlers/mod.rs:
    Imports: [check_report.rs, config/mod.rs, exit.rs, reporting.rs, scan_report.rs, spinner/mod.rs, src/analysis/mod.rs, src/discovery.rs, types/mod.rs, verification/mod.rs]
    ImportedBy: [cli/mod.rs]
  locality/analysis/mod.rs:
    Imports: [graph/mod.rs, locality/analysis/metrics.rs, violations.rs]
    ImportedBy: [locality/mod.rs]
  locality/mod.rs:
    Imports: [classifier.rs, coupling.rs, cycles.rs, distance.rs, edges.rs, exemptions.rs, layers.rs, locality/analysis/mod.rs, locality/report.rs, locality/types.rs, tests.rs, validator.rs]
    ImportedBy: [graph/mod.rs]
  patterns/mod.rs:
    Imports: [concurrency.rs, concurrency_lock.rs, concurrency_sync.rs, db_patterns.rs, idiomatic.rs, lang.rs, logic_helpers.rs, logic_proof.rs, patterns/logic.rs, patterns/state.rs, performance.rs, performance_test_ctx.rs, resource.rs, security.rs, semantic.rs, types/mod.rs]
    ImportedBy: [src/analysis/mod.rs]
  rank/mod.rs:
    Imports: [builder.rs, graph.rs, pagerank.rs, rank/queries.rs, tags.rs]
    ImportedBy: [graph/mod.rs]
  spinner/mod.rs:
    Imports: [client.rs, controller.rs, handle.rs, safe_hud.rs, spinner/render.rs, spinner/state.rs]
    ImportedBy: [handlers/mod.rs, src/lib.rs]
  verification/mod.rs:
    Imports: [config/mod.rs, types/mod.rs, verification/runner.rs]
    ImportedBy: [check_report.rs, handlers/mod.rs, src/lib.rs]
  # --- Tests ---
  check_json_test.rs, check_locality_test.rs, command_parsing_test.rs, concurrency_lock_test.rs, idiomatic_i02_test.rs, logic_helpers_test.rs, logic_l03_test.rs, logic_proof_test.rs, security_x02_test.rs:
    Imports: []
    ImportedBy: []
  part2.rs:
    Imports: []
    ImportedBy: [tests.rs]
  performance_p01_test.rs:
    Imports: [types/mod.rs]
    ImportedBy: []
  performance_test_ctx.rs:
    Imports: []
    ImportedBy: [patterns/mod.rs]
  syntax_test.rs:
    Imports: [config/mod.rs, lang.rs]
    ImportedBy: []
  tests.rs:
    Imports: [part2.rs]
    ImportedBy: [locality/mod.rs]
  # --- Subproject -- omni-ast ---
  cpp.rs, go.rs, python.rs, rust.rs:
    Imports: []
    ImportedBy: [language/mod.rs]
  cpp_includes.rs, harvester_signatures.rs, harvester_tree.rs, omni-ast/Cargo.toml, semantics_concurrency_queries.rs, semantics_engine.rs, semantics_logic_queries.rs, semantics_logic_tables.rs, semantics_queries.rs, semantics_tables.rs, taxonomy_rules.rs:
    Imports: []
    ImportedBy: []
  doc_extractor.rs, doc_filter.rs, harvester.rs, semantics.rs, src/types.rs, taxonomy.rs:
    Imports: []
    ImportedBy: [omni-ast/src/lib.rs]
  javascript.rs:
    Imports: [monorepo.rs]
    ImportedBy: [language/mod.rs]
  language/mod.rs:
    Imports: [cpp.rs, go.rs, javascript.rs, python.rs, rust.rs]
    ImportedBy: [omni-ast/src/lib.rs]
  monorepo.rs:
    Imports: []
    ImportedBy: [javascript.rs]
  omni-ast/src/lib.rs:
    Imports: [doc_extractor.rs, doc_filter.rs, harvester.rs, language/mod.rs, semantics.rs, src/types.rs, swum/mod.rs, taxonomy.rs]
    ImportedBy: []
  splitter.rs, verb_patterns.rs:
    Imports: []
    ImportedBy: [swum/mod.rs]
  swum/mod.rs:
    Imports: [splitter.rs, verb_patterns.rs]
    ImportedBy: [omni-ast/src/lib.rs]
```
