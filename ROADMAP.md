# Warden Protocol Roadmap

## Philosophy

**Source of Truth:** This roadmap is the authoritative registry of all Warden features.

**The Contract:**
1. Every `[x]` feature MUST have a `<!-- test: path::function -->` reference
2. Every referenced test MUST exist and pass
3. `warden roadmap audit --strict` enforces this before any commit
4. Features without tests use `[no-test]` (docs, config, UI-only)

**Versioning:**
- v0.x.0 = Development milestones
- v1.0.0 = Production release

---

## How to Read This File

- **Users:** Skim version headlines and the Principles section.
- **Contributors:** Look at the `🔄 CURRENT` version's unchecked tasks.
- **AI Assistants:** Never rewrite this file directly. Emit `===ROADMAP===` command blocks only.

**Current Status (v0.7.x):** Warden fully supports the Three Laws, Nabla protocol, and basic roadmap management. Smart Context (dependency graphs, cluster packing) is planned for v0.8-0.9.

---

## v0.1.0 — Foundation ✅

*Core infrastructure and project structure.*

### Token Counting
- [x] **Tokenizer initialization (cl100k_base)** <!-- test: tests/unit_tokens.rs::test_tokenizer_available -->
- [x] **Token count function** <!-- test: tests/unit_tokens.rs::test_count_basic -->
- [x] **Token limit check** <!-- test: tests/unit_tokens.rs::test_exceeds_limit -->
- [x] **Graceful fallback on init failure** <!-- test: tests/unit_tokens.rs::test_fallback_returns_zero -->

### Project Detection
- [x] **Rust project detection (Cargo.toml)** <!-- test: tests/unit_project.rs::test_detect_rust -->
- [x] **Node project detection (package.json)** <!-- test: tests/unit_project.rs::test_detect_node -->
- [x] **Python project detection** <!-- test: tests/unit_project.rs::test_detect_python -->
- [x] **Go project detection (go.mod)** <!-- test: tests/unit_project.rs::test_detect_go -->
- [x] **Unknown project fallback** <!-- test: tests/unit_project.rs::test_detect_unknown -->

### Configuration
- [x] **TOML config loading** <!-- test: tests/unit_config.rs::test_load_toml -->
- [x] **Default rule values** <!-- test: tests/unit_config.rs::test_defaults -->
- [x] **Command list parsing** <!-- test: tests/unit_config.rs::test_command_list -->
- [x] **.wardenignore loading** <!-- test: tests/unit_config.rs::test_wardenignore -->
- [x] **Auto-config generation** [no-test] *(side effect on first run)*

---

## v0.2.0 — The 3 Laws ✅

*Structural analysis enforcement.*

### Law of Atomicity
- [x] **File token counting** <!-- test: tests/integration_core.rs::test_atomicity_clean_file_passes -->
- [x] **Token limit violation** <!-- test: tests/integration_core.rs::test_atomicity_large_file_fails -->
- [x] **Token exemption patterns** <!-- test: tests/unit_config.rs::test_ignore_tokens_on -->

### Law of Complexity — Cyclomatic
- [x] **Rust complexity query (if/match/for/while/&&/||)** <!-- test: tests/integration_core.rs::test_complexity_boundary_check -->
- [x] **Complexity violation detection** <!-- test: tests/integration_core.rs::test_complexity_construct_match -->
- [x] **JS/TS complexity query** <!-- test: tests/unit_analysis.rs::test_js_complexity -->
- [x] **Python complexity query** <!-- test: tests/unit_analysis.rs::test_python_complexity -->

### Law of Complexity — Nesting Depth
- [x] **Depth calculation (block/body traversal)** <!-- test: tests/integration_core.rs::test_nesting_boundary -->
- [x] **Deep nesting violation** <!-- test: tests/integration_core.rs::test_nesting_boundary -->

### Law of Complexity — Arity
- [x] **Parameter counting** <!-- test: tests/integration_core.rs::test_arity_boundary -->
- [x] **High arity violation** <!-- test: tests/integration_core.rs::test_arity_boundary -->

### Law of Complexity — Naming
- [x] **Snake_case word counting** <!-- test: tests/unit_analysis.rs::test_snake_case_words -->
- [x] **CamelCase word counting** <!-- test: tests/unit_analysis.rs::test_camel_case_words -->
- [x] **Naming ignore patterns** <!-- test: tests/unit_config.rs::test_ignore_naming_on -->

### Law of Paranoia (Rust)
- [x] **Banned call query (.unwrap/.expect)** <!-- test: tests/integration_core.rs::test_paranoia_unwrap_fails -->
- [x] **.expect() detection** <!-- test: tests/integration_core.rs::test_paranoia_expect_fails -->
- [x] **Safe alternatives allowed (.unwrap_or)** <!-- test: tests/integration_core.rs::test_paranoia_safe_alternatives_pass -->

### File Ignores
- [x] **warden:ignore (C-style //)** <!-- test: tests/integration_core.rs::test_warden_ignore_skips_file -->
- [x] **warden:ignore (Hash-style #)** <!-- test: tests/unit_analysis.rs::test_warden_ignore_hash -->
- [x] **warden:ignore (HTML-style)** <!-- test: tests/unit_analysis.rs::test_warden_ignore_html -->

---

## v0.3.0 — Apply System ✅

*AI response parsing and file writing.*

### Nabla Format Extraction
- [x] **Header detection (∇∇∇ path ∇∇∇)** <!-- test: tests/integration_apply.rs::test_extract_single_file -->
- [x] **Footer detection (∆∆∆)** <!-- test: tests/integration_apply.rs::test_extract_single_file -->
- [x] **Path extraction from header** <!-- test: tests/integration_apply.rs::test_extract_single_file -->
- [x] **Content extraction** <!-- test: tests/integration_apply.rs::test_extract_single_file -->
- [x] **Multiple file extraction** <!-- test: tests/integration_apply.rs::test_extract_multiple_files -->
- [x] **MANIFEST block skipping** <!-- test: tests/integration_apply.rs::test_extract_skips_manifest -->
- [x] **PLAN block extraction** <!-- test: tests/integration_apply.rs::test_extract_plan -->
- [x] **Malformed block handling** <!-- test: tests/unit_extractor.rs::test_malformed_block_skipped -->

### Manifest Parsing
- [x] **Manifest block detection** <!-- test: tests/unit_manifest.rs::test_parse_manifest -->
- [x] **[NEW] marker detection** <!-- test: tests/unit_manifest.rs::test_new_marker -->
- [x] **[DELETE] marker detection** <!-- test: tests/unit_manifest.rs::test_delete_marker -->
- [x] **Default Update operation** <!-- test: tests/unit_manifest.rs::test_default_update -->

### File Writing
- [x] **Parent directory creation** <!-- test: tests/unit_writer.rs::test_creates_parent_dirs -->
- [x] **File content writing** <!-- test: tests/unit_writer.rs::test_writes_content -->
- [x] **Delete operation** <!-- test: tests/unit_writer.rs::test_delete_file -->
- [x] **Written files tracking** <!-- test: tests/unit_writer.rs::test_tracks_written -->

### Backup System
- [x] **Backup directory creation** <!-- test: tests/integration_backup.rs::test_backup_dir_created -->
- [x] **Timestamp subfolder** <!-- test: tests/integration_backup.rs::test_timestamp_folder -->
- [x] **Existing file backup** <!-- test: tests/integration_backup.rs::test_existing_backed_up -->
- [x] **New file skip (no backup needed)** <!-- test: tests/integration_backup.rs::test_new_file_no_backup -->
- [x] **Backup path structure preserved** <!-- test: tests/integration_backup.rs::test_path_structure -->

---

## v0.4.0 — Safety & Validation ✅

*Path security and content validation.*

### Path Safety — Traversal
- [x] **Block ../ traversal** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_traversal -->
- [x] **Block .. prefix** <!-- test: tests/security_validation.rs::test_traversal_blocked -->

### Path Safety — Absolute
- [x] **Block Unix absolute (/)** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_absolute -->
- [x] **Block Windows absolute (C:)** <!-- test: tests/security_validation.rs::test_absolute_paths_blocked -->

### Path Safety — Sensitive
- [x] **Block .git/** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_git -->
- [x] **Block .env** <!-- test: tests/security_validation.rs::test_sensitive_paths_blocked -->
- [x] **Block .ssh/** <!-- test: tests/security_validation.rs::test_sensitive_paths_blocked -->
- [x] **Block .aws/** <!-- test: tests/security_validation.rs::test_sensitive_paths_blocked -->
- [x] **Block .gnupg/** <!-- test: tests/unit_validator.rs::test_gnupg_blocked -->
- [x] **Block id_rsa** <!-- test: tests/unit_validator.rs::test_id_rsa_blocked -->
- [x] **Block credentials** <!-- test: tests/unit_validator.rs::test_credentials_blocked -->
- [x] **Block backup directory** <!-- test: tests/unit_validator.rs::test_backup_dir_blocked -->

### Path Safety — Hidden Files
- [x] **Block hidden files (.*)** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_hidden -->
- [x] **Allow . and .. segments** <!-- test: tests/security_validation.rs::test_valid_paths_allowed -->

### Path Safety — Protected Files
- [x] **Block ROADMAP.md rewrite** <!-- test: tests/protection_roadmap.rs::test_roadmap_rewrite_is_blocked -->
- [x] **Case-insensitive protection** <!-- test: tests/protection_roadmap.rs::test_roadmap_rewrite_blocked_case_insensitive -->

### Truncation Detection
- [x] **Pattern: // ...** <!-- test: tests/integration_apply.rs::test_truncation_detects_ellipsis_comment -->
- [x] **Pattern: /* ... */** <!-- test: tests/unit_validator.rs::test_block_comment_ellipsis -->
- [x] **Pattern: # ...** <!-- test: tests/unit_validator.rs::test_hash_ellipsis -->
- [x] **Pattern: "rest of" phrases** <!-- test: tests/unit_validator.rs::test_lazy_phrase_rest_of -->
- [x] **Pattern: "remaining" phrases** <!-- test: tests/unit_validator.rs::test_lazy_phrase_remaining -->
- [x] **warden:ignore bypass** <!-- test: tests/integration_apply.rs::test_truncation_allows_warden_ignore -->
- [x] **Empty file rejection** <!-- test: tests/integration_apply.rs::test_truncation_detects_empty_file -->
- [x] **Line number in error** <!-- test: tests/unit_validator.rs::test_line_number_reported -->

### Valid Paths
- [x] **Normal paths accepted** <!-- test: tests/integration_apply.rs::test_path_safety_allows_valid -->
- [x] **Nested src paths accepted** <!-- test: tests/security_validation.rs::test_valid_paths_allowed -->

---

## v0.5.0 — Pack & Context ✅

*Context generation for AI consumption.*

### Pack Core
- [x] **File discovery integration** <!-- test: tests/integration_pack.rs::test_nabla_delimiters_are_unique -->
- [x] **Nabla format output** <!-- test: tests/integration_pack.rs::test_nabla_format_structure -->
- [x] **Token count display** <!-- test: tests/unit_pack.rs::test_token_count_shown -->
- [x] **File write to context.txt** <!-- test: tests/unit_pack.rs::test_writes_context_file -->

### Pack Options
- [x] **--stdout output** <!-- test: tests/unit_pack.rs::test_stdout_option -->
- [x] **--copy to clipboard** <!-- test: tests/unit_pack.rs::test_copy_option -->
- [x] **--noprompt excludes header** <!-- test: tests/unit_pack.rs::test_noprompt -->
- [x] **--git-only mode** <!-- test: tests/unit_pack.rs::test_git_only -->
- [x] **--no-git mode** <!-- test: tests/unit_pack.rs::test_no_git -->
- [x] **--code-only mode** <!-- test: tests/unit_pack.rs::test_code_only -->
- [x] **--verbose progress** [no-test] *(output only)*

### Prompt Generation
- [x] **System prompt header** <!-- test: tests/integration_pack.rs::test_prompt_includes_laws -->
- [x] **Law of Atomicity in prompt** <!-- test: tests/integration_pack.rs::test_prompt_includes_limits -->
- [x] **Law of Complexity in prompt** <!-- test: tests/integration_pack.rs::test_prompt_includes_limits -->
- [x] **Nabla format instructions** <!-- test: tests/integration_pack.rs::test_prompt_includes_nabla_instructions -->
- [x] **Footer reminder** <!-- test: tests/integration_pack.rs::test_reminder_is_concise -->
- [x] **Violation injection** <!-- test: tests/unit_pack_violations.rs::test_violations_injected -->

### Skeleton System
- [x] **Rust body → { ... }** <!-- test: tests/integration_skeleton.rs::test_clean_rust_basic -->
- [x] **Rust nested functions** <!-- test: tests/integration_skeleton.rs::test_clean_rust_nested -->
- [x] **Python body → ...** <!-- test: tests/integration_skeleton.rs::test_clean_python -->
- [x] **TypeScript/JS body** <!-- test: tests/integration_skeleton.rs::test_clean_typescript -->
- [x] **Arrow function support** <!-- test: tests/integration_skeleton.rs::test_clean_typescript -->
- [x] **Unsupported passthrough** <!-- test: tests/integration_skeleton.rs::test_clean_unsupported_extension -->

### Focus Mode
- [x] **--skeleton all files** <!-- test: tests/integration_pack.rs::test_pack_skeleton_integration -->
- [x] **--target focus mode** <!-- test: tests/integration_pack.rs::test_smart_context_focus_mode -->
- [x] **Target full, rest skeleton** <!-- test: tests/integration_pack.rs::test_smart_context_focus_mode -->

### File Path Clipboard
- [x] **Copy file path for attachment** [no-test] *(platform-specific side effect)*

---

## v0.6.0 — Roadmap System ✅

*Programmatic roadmap management.*

### Roadmap Parsing
- [x] **Title extraction (# Title)** <!-- test: tests/integration_roadmap.rs::test_parse_simple_roadmap -->
- [x] **Section heading detection** <!-- test: tests/integration_roadmap.rs::test_parse_simple_roadmap -->
- [x] **Task checkbox detection** <!-- test: tests/integration_roadmap.rs::test_parse_extracts_tasks -->
- [x] **Task status: pending** <!-- test: tests/integration_roadmap.rs::test_parse_extracts_tasks -->
- [x] **Task status: complete** <!-- test: tests/integration_roadmap.rs::test_parse_extracts_tasks -->
- [x] **Stats calculation** <!-- test: tests/integration_roadmap.rs::test_stats_are_correct -->
- [x] **Test anchor extraction** <!-- test: tests/unit_roadmap.rs::test_anchor_extraction -->
- [x] **Task path generation** <!-- test: tests/integration_roadmap.rs::test_find_task_by_path -->
- [x] **Compact state display** <!-- test: tests/integration_roadmap.rs::test_compact_state_format -->

### Slugification
- [x] **Lowercase conversion** <!-- test: tests/integration_roadmap.rs::test_slugify_basic -->
- [x] **Special char to dash** <!-- test: tests/integration_roadmap.rs::test_slugify_special_chars -->
- [x] **Number preservation** <!-- test: tests/integration_roadmap.rs::test_slugify_preserves_numbers -->

### Command Parsing
- [x] **===ROADMAP=== block detection** <!-- test: tests/integration_roadmap.rs::test_parse_extracts_from_larger_text -->
- [x] **CHECK command** <!-- test: tests/integration_roadmap.rs::test_parse_check_command -->
- [ ] **UNCHECK command** <!-- test: tests/integration_roadmap.rs::test_parse_uncheck_command -->
- [x] **ADD command** <!-- test: tests/integration_roadmap.rs::test_parse_multiple_commands -->
- [x] **ADD with AFTER** <!-- test: tests/integration_roadmap.rs::test_parse_add_with_after -->
- [x] **UPDATE command** <!-- test: tests/unit_roadmap.rs::test_update_command -->
- [x] **NOTE command** <!-- test: tests/unit_roadmap.rs::test_note_command -->
- [x] **MOVE command** <!-- test: tests/unit_roadmap.rs::test_move_command -->
- [x] **Comment skipping** <!-- test: tests/integration_roadmap.rs::test_parse_ignores_comments -->
- [x] **Summary generation** <!-- test: tests/integration_roadmap.rs::test_summary_format -->

### Roadmap CLI
- [x] **roadmap init** <!-- test: tests/cli_roadmap.rs::test_init_creates_file -->
- [x] **roadmap prompt** <!-- test: tests/cli_roadmap.rs::test_prompt_generates -->
- [x] **roadmap apply** <!-- test: tests/cli_roadmap.rs::test_apply_from_clipboard -->
- [x] **roadmap show** <!-- test: tests/cli_roadmap.rs::test_show_tree -->
- [x] **roadmap tasks** <!-- test: tests/cli_roadmap.rs::test_tasks_list -->
- [x] **roadmap tasks --pending** <!-- test: tests/cli_roadmap.rs::test_tasks_pending_filter -->
- [x] **roadmap tasks --complete** <!-- test: tests/cli_roadmap.rs::test_tasks_complete_filter -->
- [x] **roadmap audit** <!-- test: tests/cli_roadmap.rs::test_audit_runs -->

### Unified Apply
- [x] **Detect ===ROADMAP=== in apply** <!-- test: tests/integration_apply.rs::test_unified_apply_roadmap -->
- [x] **Apply roadmap + files together** <!-- test: tests/integration_apply.rs::test_unified_apply_combined -->

---

## v0.7.0 — Test Traceability 🔄 CURRENT

*Enforce the contract: every feature has verified tests.*

### Parser Hardening
- [x] **Empty task ID filtering** <!-- test: tests/unit_parser.rs::test_empty_id_skipped -->
- [x] **Task ID collision detection** <!-- test: tests/unit_parser.rs::test_id_collision_resolved -->
- [x] **Anchor-based task matching** <!-- test: tests/unit_parser.rs::test_anchor_id_extraction -->
- [x] **Smart UPDATE inference (vs DELETE+ADD)** <!-- test: tests/unit_diff.rs::test_text_change_is_update -->

### Audit System
- [x] **Scan completed tasks** <!-- test: tests/integration_audit.rs::test_scans_completed_only -->
- [x] **[no-test] skip** <!-- test: tests/integration_audit.rs::test_no_test_skipped -->
- [x] **Explicit anchor verification** <!-- test: tests/integration_audit.rs::test_explicit_anchor_verified -->
- [x] **Missing test file detection** <!-- test: tests/integration_audit.rs::test_missing_file_detected -->
- [x] ****Missing test function detection** <!-- test: tests/unit_audit.rs::test_missing_function_detection -->** <!-- test: tests/unit_audit.rs::test_missing_function_detection -->**
- [ ] **Test execution verification (cargo test)**
- [x] ****Exit code 1 on any failure** [no-test] *(CLI exit behavior)***
- [x] ****--strict mode (all must pass)** [no-test] *(CLI flag)***

### Self-Hosting
- [x] **Warden passes own rules** <!-- test: tests/integration_self_host.rs::test_warden_passes_own_rules -->

### Test Naming Convention
- [ ] **Feature ID → test function mapping**
- [x] ****Audit validates naming convention** <!-- test: tests/unit_audit.rs::test_naming_convention_mismatch -->**
- [ ] **Traceability" "Enforce test-first registration during AI sessions"**
- [ ] **Traceability" "Block feature completion without matching test file"**
- [ ] **Traceability" "warden register <feature> command (pre-declares intent)"**
- [ ] **Traceability" "warden verify-session command (audit current work)"**
- [ ] **Traceability" "Incremental roadmap sync (update after each file)"**
- [ ] **Traceability" "Session manifest tracking (files touched this session)"**

### Roadmap Hardening
- [ ] **SECTION command (create version headers)** <!-- test: tests/unit_roadmap_cmd.rs::test_section_command -->
- [ ] **SUBSECTION command (create ### headers)** <!-- test: tests/unit_roadmap_cmd.rs::test_subsection_command -->
- [ ] **CHAIN command (sequential adds)** <!-- test: tests/unit_roadmap_cmd.rs::test_chain_command -->
- [ ] **AFTER PREVIOUS keyword** <!-- test: tests/unit_roadmap_cmd.rs::test_after_previous -->
- [ ] **AFTER TEXT "exact" match** <!-- test: tests/unit_roadmap_cmd.rs::test_after_text_exact -->
- [ ] **AFTER LINE N match** <!-- test: tests/unit_roadmap_cmd.rs::test_after_line_number -->
- [ ] **IN "section/subsection" location** <!-- test: tests/unit_roadmap_cmd.rs::test_in_location -->
- [ ] **Slug echo on ADD (show generated slug)** <!-- test: tests/unit_roadmap_cmd.rs::test_slug_echo -->
- [ ] **Pre-validation: all AFTER targets exist** <!-- test: tests/unit_roadmap_validate.rs::test_after_target_exists -->
- [ ] **Pre-validation: no slug collisions** <!-- test: tests/unit_roadmap_validate.rs::test_slug_collision -->
- [ ] **Pre-validation: no circular AFTER chains** <!-- test: tests/unit_roadmap_validate.rs::test_circular_detection -->
- [ ] **Batch dependency resolution (topological sort)** <!-- test: tests/unit_roadmap_validate.rs::test_batch_topo_sort -->
- [ ] **Fuzzy match suggestions on AFTER miss** <!-- test: tests/unit_roadmap_validate.rs::test_fuzzy_suggest -->
- [ ] **Dry-run mode (--dry-run flag)** <!-- test: tests/cli_roadmap.rs::test_apply_dry_run -->
- [ ] **Atomic file write (temp → rename)** <!-- test: tests/unit_roadmap_write.rs::test_atomic_write -->
- [ ] **Backup creation (.md.bak)** <!-- test: tests/unit_roadmap_write.rs::test_backup_created -->
- [ ] **All-or-nothing execution (rollback on error)** <!-- test: tests/integration_roadmap.rs::test_rollback_on_error -->
- [ ] **Verbose plan output** <!-- test: tests/cli_roadmap.rs::test_verbose_plan -->

### Escape Hatches
- [ ] **warden apply --force flag** <!-- test: tests/cli_apply.rs::test_force_flag -->
- [ ] **Quarantine mode (// warden:quarantine marker)** <!-- test: tests/integration_apply.rs::test_quarantine_marker -->
- [ ] **Quarantine report in warden scan** <!-- test: tests/integration_core.rs::test_quarantine_report -->

---

## v0.8.0 — Dependency Graph ⚡ HIGH PRIORITY

> **Why this matters:** Efficient context management is what makes AI assistance scale. Understanding file relationships enables surgical context packing.

*Build the import graph for smart context generation.*

### Import Extraction — Rust
- [x] **Rust use declaration extraction** <!-- test: tests/unit_graph.rs::test_rust_use_extraction -->
- [ ] **Rust mod declaration extraction** <!-- test: tests/unit_graph.rs::test_rust_mod_extraction -->
- [ ] **Rust re-export handling (pub use)** <!-- test: tests/unit_graph.rs::test_rust_reexport -->
- [ ] **Rust crate:: path resolution** <!-- test: tests/unit_graph.rs::test_rust_crate_path -->
- [ ] **Rust self:: path resolution** <!-- test: tests/unit_graph.rs::test_rust_self_path -->
- [ ] **Rust super:: path resolution** <!-- test: tests/unit_graph.rs::test_rust_super_path -->
- [ ] **Rust mod.rs index resolution** <!-- test: tests/unit_graph.rs::test_rust_mod_index -->

### Import Extraction — Python
- [x] **Python import statement extraction** <!-- test: tests/unit_graph.rs::test_python_import -->
- [x] **Python from...import extraction** <!-- test: tests/unit_graph.rs::test_python_from_import -->
- [ ] **Python relative import handling (.module)** <!-- test: tests/unit_graph.rs::test_python_relative -->
- [ ] **Python __init__.py resolution** <!-- test: tests/unit_graph.rs::test_python_init -->

### Import Extraction — TypeScript/JavaScript
- [x] **TypeScript import extraction** <!-- test: tests/unit_graph.rs::test_ts_import -->
- [ ] **TypeScript require() extraction** <!-- test: tests/unit_graph.rs::test_ts_require -->
- [ ] **TypeScript re-export extraction** <!-- test: tests/unit_graph.rs::test_ts_reexport -->
- [ ] **JavaScript ESM/CJS detection** <!-- test: tests/unit_graph.rs::test_js_module_type -->
- [ ] **TypeScript index.ts resolution** <!-- test: tests/unit_graph.rs::test_ts_index -->
- [ ] **TypeScript path alias resolution (tsconfig)** <!-- test: tests/unit_graph.rs::test_ts_path_alias -->

### Import Resolution — General
- [ ] **Relative path resolution (./)** <!-- test: tests/unit_resolver.rs::test_relative_path -->
- [ ] **Parent path resolution (../)** <!-- test: tests/unit_resolver.rs::test_parent_path -->
- [ ] **External dependency detection (skip)** <!-- test: tests/unit_resolver.rs::test_external_skipped -->

### Graph Construction
- [x] **Graph node creation (file → vertex)** <!-- test: tests/unit_graph_build.rs::test_node_creation -->
- [x] **Graph edge creation (import → directed edge)** <!-- test: tests/unit_graph_build.rs::test_edge_creation -->
- [x] **Reverse index construction (importers)** <!-- test: tests/unit_graph_build.rs::test_reverse_index -->
- [ ] **Cycle detection (tarjan or DFS)** <!-- test: tests/unit_graph_build.rs::test_cycle_detection -->
- [ ] **Hub file detection (high in-degree)** <!-- test: tests/unit_graph_build.rs::test_hub_detection -->
- [ ] **Leaf file detection (zero out-degree)** <!-- test: tests/unit_graph_build.rs::test_leaf_detection -->
- [ ] **Orphan file detection (zero in-degree)** <!-- test: tests/unit_graph_build.rs::test_orphan_detection -->
- [ ] **Graph serialization (debug output)** <!-- test: tests/unit_graph_build.rs::test_serialization -->

### Graph Traversal
- [ ] **BFS traversal from entry point** <!-- test: tests/unit_graph_walk.rs::test_bfs -->
- [ ] **DFS traversal from entry point** <!-- test: tests/unit_graph_walk.rs::test_dfs -->
- [ ] **Depth-limited traversal (--depth N)** <!-- test: tests/unit_graph_walk.rs::test_depth_limit -->
- [ ] **Forward walk (dependencies)** <!-- test: tests/unit_graph_walk.rs::test_forward -->
- [ ] **Reverse walk (dependents)** <!-- test: tests/unit_graph_walk.rs::test_reverse -->
- [ ] **Bidirectional walk (both)** <!-- test: tests/unit_graph_walk.rs::test_bidirectional -->
- [ ] **Subgraph extraction** <!-- test: tests/unit_graph_walk.rs::test_subgraph -->
- [ ] **Topological sort for output ordering** <!-- test: tests/unit_graph_walk.rs::test_topo_sort -->

### Cluster Detection
- [ ] **Directory-based cluster inference** <!-- test: tests/unit_cluster.rs::test_dir_cluster -->
- [ ] **Strongly connected component detection** <!-- test: tests/unit_cluster.rs::test_scc -->
- [ ] **Cluster boundary identification** <!-- test: tests/unit_cluster.rs::test_boundary -->
- [ ] **Cross-cluster edge detection** <!-- test: tests/unit_cluster.rs::test_cross_cluster -->
- [ ] **Cluster size metrics (files, tokens)** <!-- test: tests/unit_cluster.rs::test_metrics -->
- [ ] **warden.toml [clusters] definition** <!-- test: tests/unit_cluster.rs::test_toml_clusters -->
- [ ] **// warden:cluster(name) annotation** <!-- test: tests/unit_cluster.rs::test_annotation -->

---

## v0.9.0 — Smart Context ⚡ HIGH PRIORITY

> **Ship incrementally:** Start with `--from-errors` (highest ROI), then add cluster/trace packing.

*Give AI exactly what it needs, nothing more.*

### Warden Map Command
- [x] **warden map basic output** <!-- test: tests/cli_map.rs::test_map_basic -->
- [x] **Directory tree with file counts** <!-- test: tests/cli_map.rs::test_map_tree -->
- [ ] **Cluster summary display** <!-- test: tests/cli_map.rs::test_map_clusters -->
- [ ] **--deps flag (show dependency arrows)** <!-- test: tests/cli_map.rs::test_map_deps -->
- [ ] **--stats flag (token counts per cluster)** <!-- test: tests/cli_map.rs::test_map_stats -->
- [ ] **--json flag (machine-readable map)** <!-- test: tests/cli_map.rs::test_map_json -->
- [ ] **Module description extraction (//! or docstring)** <!-- test: tests/cli_map.rs::test_map_docs -->
- [ ] **Entry point detection (main.rs, lib.rs, index.ts)** <!-- test: tests/cli_map.rs::test_entry_point -->

### Error-Driven Packing
- [ ] **warden pack --from-errors flag** <!-- test: tests/integration_error_pack.rs::test_from_errors_flag -->
- [ ] **Cargo/rustc error parsing** <!-- test: tests/unit_error_parse.rs::test_cargo_errors -->
- [ ] **Clippy warning parsing** <!-- test: tests/unit_error_parse.rs::test_clippy_warnings -->
- [ ] **TypeScript/tsc error parsing** <!-- test: tests/unit_error_parse.rs::test_tsc_errors -->
- [ ] **Python traceback parsing** <!-- test: tests/unit_error_parse.rs::test_python_traceback -->
- [ ] **ESLint output parsing** <!-- test: tests/unit_error_parse.rs::test_eslint_output -->
- [ ] **File path extraction from errors** <!-- test: tests/unit_error_parse.rs::test_path_extraction -->
- [ ] **Line number extraction from errors** <!-- test: tests/unit_error_parse.rs::test_line_extraction -->
- [ ] **Unique file deduplication** <!-- test: tests/unit_error_parse.rs::test_dedup -->
- [ ] **Auto-include test files for src errors** <!-- test: tests/integration_error_pack.rs::test_auto_tests -->
- [ ] **Piped input support (cargo clippy 2>&1 |)** <!-- test: tests/integration_error_pack.rs::test_piped -->
- [ ] **--from-clipboard-errors flag** <!-- test: tests/integration_error_pack.rs::test_clipboard_errors -->

### Cluster Packing
- [ ] **warden pack --cluster NAME flag** <!-- test: tests/integration_cluster_pack.rs::test_cluster_flag -->
- [ ] **Cluster resolution by name** <!-- test: tests/integration_cluster_pack.rs::test_by_name -->
- [ ] **Cluster resolution by directory path** <!-- test: tests/integration_cluster_pack.rs::test_by_dir -->
- [ ] **--with-tests flag (include test files)** <!-- test: tests/integration_cluster_pack.rs::test_with_tests -->
- [ ] **--with-boundary flag (skeleton boundary files)** <!-- test: tests/integration_cluster_pack.rs::test_with_boundary -->
- [ ] **--no-boundary flag (exclude boundary files)** <!-- test: tests/integration_cluster_pack.rs::test_no_boundary -->
- [ ] **Multiple cluster inclusion (--cluster a --cluster b)** <!-- test: tests/integration_cluster_pack.rs::test_multi_cluster -->

### Trace Packing
- [ ] **warden pack --trace PATH flag** <!-- test: tests/integration_trace_pack.rs::test_trace_flag -->
- [ ] **--depth N limit for trace** <!-- test: tests/integration_trace_pack.rs::test_depth_limit -->
- [ ] **--forward flag (dependencies only)** <!-- test: tests/integration_trace_pack.rs::test_forward -->
- [ ] **--reverse flag (dependents only)** <!-- test: tests/integration_trace_pack.rs::test_reverse -->
- [ ] **Default: bidirectional trace** <!-- test: tests/integration_trace_pack.rs::test_bidirectional -->
- [ ] **Trace + skeleton hybrid output** <!-- test: tests/integration_trace_pack.rs::test_hybrid -->
- [ ] **Multiple trace roots (--trace a --trace b)** <!-- test: tests/integration_trace_pack.rs::test_multi_trace -->

### Context Ordering
- [ ] **Dependency-first ordering (topological)** <!-- test: tests/unit_ordering.rs::test_topo_order -->
- [ ] **Leaf files appear first** <!-- test: tests/unit_ordering.rs::test_leaves_first -->
- [ ] **Target/focus file appears last** <!-- test: tests/unit_ordering.rs::test_target_last -->
- [ ] **Circular dependency handling (break arbitrarily)** <!-- test: tests/unit_ordering.rs::test_cycle_break -->
- [ ] **Shared dependency hoisting** <!-- test: tests/unit_ordering.rs::test_hoisting -->

### AI Context Protocol
- [ ] **CONTEXT_REQUEST format specification** [no-test] *(documentation)*
- [ ] **AI can emit cluster requests** <!-- test: tests/integration_fulfill.rs::test_cluster_request -->
- [ ] **AI can emit trace requests** <!-- test: tests/integration_fulfill.rs::test_trace_request -->
- [ ] **AI can emit file requests** <!-- test: tests/integration_fulfill.rs::test_file_request -->
- [ ] **warden fulfill command (parse AI request)** <!-- test: tests/integration_fulfill.rs::test_fulfill_command -->
- [ ] **Request validation (cluster/file exists)** <!-- test: tests/integration_fulfill.rs::test_validation -->

---

## v0.10.0 — Validation Hardening

*Catch more AI failure modes.*

### Markdown Rejection
- [ ] **Block triple backticks (```)** <!-- test: tests/integration_apply.rs::test_rejects_markdown_fences -->
- [ ] **Block tilde fences (~~~)** <!-- test: tests/integration_apply.rs::test_rejects_tilde_fences -->
- [ ] **Markdown fence rejection rationale** [no-test] *(documentation: AI escape issues)*

### Brace Balancing
- [ ] **Detect unbalanced {** <!-- test: tests/integration_apply.rs::test_detects_unbalanced_open_brace -->
- [ ] **Detect unbalanced }** <!-- test: tests/integration_apply.rs::test_detects_unbalanced_close_brace -->
- [ ] **Detect unbalanced [** <!-- test: tests/integration_apply.rs::test_detects_unbalanced_bracket -->
- [ ] **Detect unbalanced (** <!-- test: tests/integration_apply.rs::test_detects_unbalanced_paren -->
- [ ] **Brace balance algorithm selection** [no-test] *(design decision)*
- [ ] **String literal exclusion from brace count** <!-- test: tests/unit_brace.rs::test_string_exclusion -->
- [ ] **Comment exclusion from brace count** <!-- test: tests/unit_brace.rs::test_comment_exclusion -->

---

## v0.11.0 — CI/CD Integration

*Machine-readable output and automation.*

### Output Formats
- [ ] **--format json** <!-- test: tests/cli_format.rs::test_json_output -->
- [ ] **SARIF output for GitHub** <!-- test: tests/cli_format.rs::test_sarif_output -->

### Git Hooks
- [ ] **warden hook install** <!-- test: tests/cli_hooks.rs::test_hook_install -->
- [ ] **Pre-commit hook script** <!-- test: tests/cli_hooks.rs::test_precommit_runs -->

### Exit Codes
- [ ] **Exit 0 on clean** <!-- test: tests/cli_exit.rs::test_exit_0_clean -->
- [ ] **Exit 1 on violations** <!-- test: tests/cli_exit.rs::test_exit_1_violations -->
- [ ] **Exit 2 on error** <!-- test: tests/cli_exit.rs::test_exit_2_error -->

### CI Templates
- [ ] **GitHub Actions workflow template** <!-- test: tests/cli_ci.rs::test_github_template -->
- [ ] **GitLab CI template** <!-- test: tests/cli_ci.rs::test_gitlab_template -->
- [ ] **warden init --ci flag (generate workflow)** <!-- test: tests/cli_ci.rs::test_init_ci -->
- [ ] **Fail-fast vs report-all modes** <!-- test: tests/cli_ci.rs::test_fail_modes -->
- [ ] **Annotation output for GitHub PR comments** <!-- test: tests/cli_ci.rs::test_annotations -->

---

## v0.12.0 — Graph Visualization

*See your codebase structure.*

### Visualization Formats
- [ ] **warden graph command** <!-- test: tests/cli_graph.rs::test_graph_command -->
- [ ] **DOT format export (Graphviz)** <!-- test: tests/cli_graph.rs::test_dot_export -->
- [ ] **Mermaid format export** <!-- test: tests/cli_graph.rs::test_mermaid_export -->
- [ ] **--cluster-only flag (show clusters, not files)** <!-- test: tests/cli_graph.rs::test_cluster_only -->
- [ ] **--highlight PATH flag (color specific subgraph)** <!-- test: tests/cli_graph.rs::test_highlight -->
- [ ] **Interactive HTML export (D3.js)** <!-- test: tests/cli_graph.rs::test_html_export -->
- [ ] **Terminal ASCII graph (small projects)** <!-- test: tests/cli_graph.rs::test_ascii_graph -->

---

## v0.13.0 — Legacy Adoption

*Make Warden adoptable in existing codebases.*

### Baseline System
- [ ] **warden baseline command (snapshot current violations)** <!-- test: tests/cli_baseline.rs::test_baseline_creation -->
- [ ] **Baseline file format (.warden-baseline.json)** <!-- test: tests/unit_baseline.rs::test_baseline_format -->
- [ ] **Baseline comparison mode (only report new violations)** <!-- test: tests/integration_baseline.rs::test_baseline_comparison -->
- [ ] **--baseline flag for warden scan** <!-- test: tests/cli_baseline.rs::test_baseline_flag -->
- [ ] **Auto-generate warden:ignore for existing violations** <!-- test: tests/cli_baseline.rs::test_auto_ignore -->
- [ ] **Gradual tightening guide** [no-test] *(documentation)*

---

## v1.0.0 — Release

*Production-ready distribution.*

### Distribution
- [ ] **Published to crates.io** [no-test]
- [ ] **Homebrew formula** [no-test]
- [ ] **Scoop/Winget packages** [no-test]

### Documentation
- [ ] **Documentation site** [no-test]
- [ ] **Logo and branding** [no-test]
- [ ] **README finalized** [no-test]
- [ ] **CHANGELOG.md generation** [no-test]
- [ ] **CONTRIBUTING.md guide** [no-test]
- [ ] **Security policy (SECURITY.md)** [no-test]

### Polish
- [ ] **License audit (dependency licenses)** <!-- test: tests/release.rs::test_license_audit -->
- [ ] **Binary size optimization** [no-test]
- [ ] **Startup time benchmarking** [no-test]
- [ ] **Cross-compilation CI (linux/mac/windows)** [no-test]

---

## Principles

1. **Every [x] feature has a verified test** — No exceptions (except [no-test])
2. **Reject bad input, don't fix it** — Warden is a gatekeeper
3. **Git is the undo system** — Don't reinvent version control
4. **Explicit > Magic** — Fail loudly on format violations
5. **Containment over craftsmanship** — Constraints are safety, not style
6. **Self-hosting** — Warden passes its own rules
7. **Context is king** — Give AI exactly what it needs, nothing more
8. **Graph over glob** — Understand structure, don't just pattern match
9. **Errors are context** — Parse failures to understand scope

---

## Not Doing

- **VS Code Extension** — IDE lock-in, maintenance burden
- **Watch mode** — Complexity without clear benefit
- **Markdown fallback parsing** — Enforce format discipline
- **"Smart" fixing** — Warden rejects, doesn't repair
- **Full LSP implementation** — Use tree-sitter queries, not language servers
- **Multi-repo support** — One project at a time
- **Incremental graph updates** — Rebuild on each run (fast enough)