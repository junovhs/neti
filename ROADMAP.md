# SlopChop Protocol Roadmap

The path to a hardened v1.0.0 trust boundary.

---

## v0.1.0 — Foundation ?

### Token Counting
- [x] **Tokenizer initialization (cl100k_base)** <!-- test: tests/unit_tokens.rs::test_tokenizer_available -->
- [x] **Token count function** <!-- test: tests/unit_tokens.rs::test_count_basic -->
- [x] **Token limit check** <!-- test: tests/unit_tokens.rs::test_exceeds_limit -->

### Project Detection
- [x] **Rust project detection (Cargo.toml)** <!-- test: tests/unit_project.rs::test_detect_rust -->
- [x] **Node project detection (package.json)** <!-- test: tests/unit_project.rs::test_detect_node -->
- [x] **Python project detection** <!-- test: tests/unit_project.rs::test_detect_python -->
- [x] **Go project detection (go.mod)** <!-- test: tests/unit_project.rs::test_detect_go -->

### Configuration
- [x] **TOML config loading** <!-- test: tests/unit_config.rs::test_load_toml -->
- [x] **Default rule values** <!-- test: tests/unit_config.rs::test_defaults -->
- [x] **Command list parsing** <!-- test: tests/unit_config.rs::test_command_list -->
- [x] **.slopchopignore loading** <!-- test: tests/unit_config.rs::test_slopchopignore -->
- [x] **Auto-config generation** [no-test]

### New Group
- [ ] **A brand new feature** <!-- test: tests/new.rs::test_new -->

- [ ] **First new task**
- [ ] **Second new task (after previous)**
- [ ] **Task added after text match**
- [ ] **Task at specific line position**

---

## v0.2.0 — The 3 Laws ?

### Law of Atomicity
- [x] **File token counting** <!-- test: tests/integration_core.rs::test_atomicity_clean_file_passes -->
- [x] **Token limit violation** <!-- test: tests/integration_core.rs::test_atomicity_large_file_fails -->
- [x] **Token exemption patterns** <!-- test: tests/unit_config.rs::test_ignore_tokens_on -->

### Law of Complexity
- [x] **Rust complexity query (if/match/for/while/&&/||)** <!-- test: tests/integration_core.rs::test_complexity_boundary_check -->
- [x] **Complexity violation detection** <!-- test: tests/integration_core.rs::test_complexity_construct_match -->
- [x] **JS/TS complexity query** <!-- test: tests/unit_analysis.rs::test_js_complexity -->
- [x] **Python complexity query** <!-- test: tests/unit_analysis.rs::test_python_complexity -->
- [x] **Depth calculation (block/body traversal)** <!-- test: tests/integration_core.rs::test_nesting_boundary -->
- [x] **Deep nesting violation** <!-- test: tests/integration_core.rs::test_nesting_boundary -->
- [x] **Parameter counting** <!-- test: tests/integration_core.rs::test_arity_boundary -->
- [x] **High arity violation** <!-- test: tests/integration_core.rs::test_arity_boundary -->
- [x] **Snake_case word counting** <!-- test: tests/unit_analysis.rs::test_snake_case_words -->
- [x] **CamelCase word counting** <!-- test: tests/unit_analysis.rs::test_camel_case_words -->
- [x] **Naming ignore patterns** <!-- test: tests/unit_config.rs::test_ignore_naming_on -->

### Law of Paranoia
- [x] **.expect() detection** <!-- test: tests/integration_core.rs::test_paranoia_expect_fails -->
- [x] **Safe alternatives allowed (.unwrap_or)** <!-- test: tests/integration_core.rs::test_paranoia_safe_alternatives_pass -->

### File Ignores
- [x] **slopchop:ignore (C-style //)** <!-- test: tests/integration_core.rs::test_slopchop_ignore_skips_file -->
- [x] **slopchop:ignore (Hash-style #)** <!-- test: tests/unit_analysis.rs::test_slopchop_ignore_hash -->
- [x] **slopchop:ignore (HTML-style)** <!-- test: tests/unit_analysis.rs::test_slopchop_ignore_html -->

---

## v0.3.0 — Apply System ?

### Protocol Format
- [x] **Header detection (#__SLOPCHOP_FILE__#)** <!-- test: tests/integration_apply.rs::test_extract_single_file -->
- [x] **Footer detection (#__SLOPCHOP_END__#)** <!-- test: tests/integration_apply.rs::test_extract_single_file -->
- [x] **Path extraction from header** <!-- test: tests/integration_apply.rs::test_extract_single_file -->
- [x] **Content extraction** <!-- test: tests/integration_apply.rs::test_extract_single_file -->
- [x] **Multiple file extraction** <!-- test: tests/integration_apply.rs::test_extract_multiple_files -->
- [x] **MANIFEST block skipping** <!-- test: tests/integration_apply.rs::test_extract_skips_manifest -->
- [x] **PLAN block extraction** <!-- test: tests/integration_apply.rs::test_extract_plan -->
- [x] **Malformed block handling** <!-- test: tests/integration_apply.rs::test_extract_single_file -->

### Manifest Parsing
- [x] **Manifest block detection** <!-- test: tests/integration_apply.rs::test_extract_skips_manifest -->
- [x] **[NEW] marker detection** <!-- test: tests/integration_apply.rs::test_unified_apply_combined -->
- [x] **[DELETE] marker detection** <!-- test: tests/integration_apply.rs::test_delete_marker_detection -->
- [x] **Default Update operation** <!-- test: tests/integration_apply.rs::test_unified_apply_combined -->

### File Writing
- [x] **Parent directory creation** <!-- test: tests/integration_backup.rs::test_path_structure -->
- [x] **File content writing** <!-- test: tests/integration_backup.rs::test_existing_backed_up -->
- [x] **Delete operation** <!-- test: tests/integration_apply.rs::test_delete_operation -->
- [x] **Written files tracking** <!-- test: tests/integration_backup.rs::test_existing_backed_up -->

### Backup System
- [x] **Backup directory creation** <!-- test: tests/integration_backup.rs::test_backup_dir_created -->
- [x] **Timestamp subfolder** <!-- test: tests/integration_backup.rs::test_timestamp_folder -->
- [x] **Existing file backup** <!-- test: tests/integration_backup.rs::test_existing_backed_up -->
- [x] **New file skip (no backup needed)** <!-- test: tests/integration_backup.rs::test_new_file_no_backup -->
- [x] **Backup path structure preserved** <!-- test: tests/integration_backup.rs::test_path_structure -->

---

## v0.4.0 — Safety & Validation ?

### Path Safety
- [x] **Block ../ traversal** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_traversal -->
- [x] **Block .. prefix** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_traversal -->
- [x] **Block Unix absolute (/)** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_absolute -->
- [x] **Block Windows absolute (C:)** <!-- test: tests/integration_apply.rs::test_block_windows_absolute -->
- [x] **Block .git/** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_git -->
- [x] **Block .env** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_hidden -->
- [x] **Block .ssh/** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_hidden -->
- [x] **Block .aws/** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_hidden -->
- [x] **Block .gnupg/** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_hidden -->
- [x] **Block id_rsa** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_hidden -->
- [x] **Block credentials** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_hidden -->
- [x] **Block backup directory** <!-- test: tests/integration_apply.rs::test_block_backup_directory -->
- [x] **Block hidden files (.*)** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_hidden -->
- [x] **Allow . and .. segments** <!-- test: tests/integration_apply.rs::test_path_safety_allows_valid -->

### Truncation Detection
- [x] **Pattern: // ...** <!-- test: tests/integration_apply.rs::test_truncation_detects_ellipsis_comment -->
- [x] **Pattern: /* ... */** <!-- test: tests/integration_apply.rs::test_truncation_detects_ellipsis_comment -->
- [x] **Pattern: # ...** <!-- test: tests/integration_apply.rs::test_truncation_detects_ellipsis_comment -->
- [x] **Pattern: "rest of" phrases** <!-- test: tests/integration_apply.rs::test_truncation_detects_ellipsis_comment -->
- [x] **Pattern: "remaining" phrases** <!-- test: tests/integration_apply.rs::test_truncation_detects_ellipsis_comment -->
- [x] **slopchop:ignore bypass** <!-- test: tests/integration_apply.rs::test_truncation_allows_slopchop_ignore -->
- [x] **Empty file rejection** <!-- test: tests/integration_apply.rs::test_truncation_detects_empty_file -->
- [x] **Line number in error** <!-- test: tests/integration_apply.rs::test_truncation_detects_ellipsis_comment -->

### Valid Paths
- [x] **Normal paths accepted** <!-- test: tests/integration_apply.rs::test_path_safety_allows_valid -->
- [x] **Nested src paths accepted** <!-- test: tests/integration_apply.rs::test_path_safety_allows_valid -->

---

## v0.5.0 — Pack & Context ?

### Pack Core
- [x] **File discovery integration** <!-- test: tests/integration_pack.rs::test_nabla_delimiters_are_unique -->
- [x] **Protocol format output** <!-- test: tests/integration_pack.rs::test_nabla_format_structure -->
- [x] **Token count display** <!-- test: tests/unit_pack.rs::test_token_count_shown -->
- [x] **File write to context.txt** <!-- test: tests/unit_pack.rs::test_writes_context_file -->

### Pack Options
- [x] **--stdout output** <!-- test: tests/unit_pack.rs::test_stdout_option -->
- [x] **--copy to clipboard** <!-- test: tests/unit_pack.rs::test_copy_option -->
- [x] **--noprompt excludes header** <!-- test: tests/unit_pack.rs::test_noprompt -->
- [x] **--git-only mode** <!-- test: tests/unit_pack.rs::test_git_only -->
- [x] **--no-git mode** <!-- test: tests/unit_pack.rs::test_no_git -->
- [x] **--code-only mode** <!-- test: tests/unit_pack.rs::test_code_only -->

### Prompt Generation
- [x] **System prompt header** <!-- test: tests/integration_pack.rs::test_prompt_includes_laws -->
- [x] **Law of Atomicity in prompt** <!-- test: tests/integration_pack.rs::test_prompt_includes_limits -->
- [x] **Law of Complexity in prompt** <!-- test: tests/integration_pack.rs::test_prompt_includes_limits -->
- [x] **Protocol format instructions** <!-- test: tests/integration_pack.rs::test_prompt_includes_nabla_instructions -->
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

---

## v0.6.0 — Roadmap System ?

### Roadmap Parsing
- [x] **Title extraction (# Title)** <!-- test: tests/unit_roadmap_v2.rs::test_generator_basic_markdown -->
- [x] **Section heading detection** <!-- test: tests/unit_roadmap_v2.rs::test_generator_basic_markdown -->
- [x] **Task checkbox detection** <!-- test: tests/unit_roadmap_v2.rs::test_generator_basic_markdown -->
- [x] **Task status: pending** <!-- test: tests/unit_roadmap_v2.rs::test_generator_basic_markdown -->
- [x] **Task status: complete** <!-- test: tests/unit_roadmap_v2.rs::test_generator_with_done_task -->
- [x] **Stats calculation** <!-- test: tests/unit_roadmap_v2_extra.rs::test_stats_calculation -->
- [x] **Test anchor extraction** <!-- test: tests/unit_roadmap_v2.rs::test_generator_includes_test_anchors -->
- [x] **Task path generation** <!-- test: tests/unit_roadmap_v2_extra.rs::test_task_path_generation -->
- [x] **Compact state display** <!-- test: tests/unit_roadmap_v2_extra.rs::test_compact_state_display -->

### Slugification
- [x] **Lowercase conversion** <!-- test: tests/unit_roadmap_v2_extra.rs::test_lowercase_conversion -->
- [x] **Special char to dash** <!-- test: tests/unit_roadmap_v2_extra.rs::test_special_char_to_dash -->
- [x] **Number preservation** <!-- test: tests/unit_roadmap_v2_extra.rs::test_number_preservation -->

### Command Parsing
- [x] **===ROADMAP=== block detection** <!-- test: tests/unit_roadmap_v2.rs::test_store_check_command -->
- [x] **CHECK command** <!-- test: tests/unit_roadmap_v2.rs::test_store_check_command -->
- [x] **UNCHECK command** <!-- test: tests/unit_roadmap_v2.rs::test_store_uncheck_command -->
- [x] **ADD command** <!-- test: tests/unit_roadmap_v2.rs::test_store_add_command -->
- [x] **ADD with AFTER** <!-- test: tests/unit_roadmap_v2_extra.rs::test_add_with_after -->
- [x] **UPDATE command** <!-- test: tests/unit_roadmap_v2.rs::test_store_update_command -->
- [x] **NOTE command** <!-- test: tests/unit_roadmap_v2_extra.rs::test_note_command -->
- [x] **MOVE command** <!-- test: tests/unit_roadmap_v2_extra.rs::test_move_command -->
- [x] **Comment skipping** <!-- test: tests/unit_roadmap_v2_extra.rs::test_comment_skipping -->
- [x] **Summary generation** <!-- test: tests/unit_roadmap_v2.rs::test_generator_basic_markdown -->

### Roadmap CLI
- [x] **roadmap init** <!-- test: tests/unit_cli_roadmap.rs::test_roadmap_init -->
- [x] **roadmap prompt** <!-- test: tests/unit_cli_roadmap.rs::test_roadmap_prompt -->
- [x] **roadmap apply** <!-- test: tests/integration_apply.rs::test_unified_apply_roadmap -->
- [x] **roadmap show** <!-- test: tests/unit_cli_roadmap.rs::test_roadmap_show -->
- [x] **roadmap tasks** <!-- test: tests/unit_cli_roadmap.rs::test_roadmap_tasks -->
- [x] **roadmap tasks --pending** <!-- test: tests/unit_cli_roadmap.rs::test_roadmap_tasks_pending -->
- [x] **roadmap tasks --complete** <!-- test: tests/unit_cli_roadmap.rs::test_roadmap_tasks_complete -->
- [x] **roadmap audit** <!-- test: tests/unit_cli_roadmap.rs::test_roadmap_audit -->

### Unified Apply
- [x] **Detect ===ROADMAP=== in apply** <!-- test: tests/integration_apply.rs::test_unified_apply_roadmap -->
- [x] **Apply roadmap + files together** <!-- test: tests/integration_apply.rs::test_unified_apply_combined -->

---

## v0.7.0 — Legacy Baseline ?

- [x] **Fix clippy must_use in git.rs** <!-- test: [no-test] -->
- [x] **Reduce complexity in dashboard nav** <!-- test: [no-test] -->
- [x] **Prevent false positive roadmap parsing on inline text** <!-- test: tests/repro_roadmap_hardening.rs::test_roadmap_parser_ignores_inline_markers -->
- [x] **Fix TUI corruption during apply by suspending terminal mode** <!-- test: [no-test] -->
- [x] **Split TUI dashboard into smaller modules (input/apply)** <!-- test: [no-test] -->
- [x] **Implement interactive roadmap in TUI (scroll, toggle)** <!-- test: [no-test] -->
- [x] **Add config option to require plan blocks** <!-- test: [no-test] -->
- [x] **Implement Weisfeiler-Lehman AST fingerprinting** <!-- test: [no-test] -->
- [x] **Implement Union-Find clustering for duplicate detection** <!-- test: [no-test] -->
- [x] **Implement call graph reachability for dead code** <!-- test: [no-test] -->
- [x] **Implement tree-sitter pattern matching for idiom detection** <!-- test: [no-test] -->
- [x] **Implement impact scoring and prioritization system** <!-- test: [no-test] -->
- [x] **Add slopchop audit command with format options** <!-- test: [no-test] -->
- [x] **Implement terminal, JSON, and AI output formatters** <!-- test: [no-test] -->
- [x] **Fix existing violations in src/audit** <!-- test: tests/unit_god_tier.rs::test_enhance_plan_generation -->
- [x] **God Tier Audit: AST Diffing (diff.rs)** <!-- test: tests/unit_god_tier.rs::test_diff_simple_variant -->
- [x] **God Tier Audit: Parameterization Inference** <!-- test: tests/unit_god_tier.rs::test_parameterize_strategy_enum -->
- [x] **God Tier Audit: Refactor CodeGen** <!-- test: tests/unit_god_tier.rs::test_codegen_enum -->
- [x] **God Tier Audit: Opportunity Enhancement** <!-- test: tests/unit_god_tier.rs::test_enhance_plan_generation -->
- [x] **Integrate RepoGraph into signatures command** <!-- test: tests/signatures_test.rs::test_holographic_signatures_graph -->
- [x] **Implement Topological Sort for signature output** <!-- test: tests/signatures_test.rs::test_holographic_signatures_topo_sort -->
- [x] **Add PageRank importance tags to signatures** <!-- test: tests/signatures_test.rs::test_holographic_signatures_pagerank -->
- [x] **Extract and display first-line docstrings in signatures** <!-- test: tests/signatures_test.rs::test_holographic_signatures_docstrings -->
- [x] **Empty task ID filtering** <!-- test: tests/signatures_test.rs::test_empty_task_id_filtering -->
- [x] **Task ID collision detection** <!-- test: tests/unit_roadmap_v2.rs::test_duplicate_add_rejected -->
- [x] **Clean check output (failures only)** <!-- test: tests/unit_cli_roadmap.rs::test_clean_check_output -->
- [x] **[no-test] skip** <!-- test: tests/unit_roadmap_v2.rs::test_generator_notest_marker -->
- [x] **Explicit anchor verification** <!-- test: tests/unit_cli_roadmap.rs::test_explicit_anchor_verification -->
- [x] **Missing test file detection** <!-- test: tests/unit_cli_roadmap.rs::test_missing_test_file_detection -->
- [x] **Missing test function detection** <!-- test: tests/unit_cli_roadmap.rs::test_missing_test_function_detection -->
- [x] **Audit validates naming convention** <!-- test: tests/unit_cli_roadmap.rs::test_audit_validates_naming -->
- [x] **SECTION command (create version headers)** <!-- test: tests/unit_roadmap_v2_extra.rs::test_section_command -->
- [x] **SUBSECTION command (create ### headers)** <!-- test: tests/unit_roadmap_v2_extra.rs::test_subsection_command -->
- [x] **CHAIN command (sequential adds)** <!-- test: tests/unit_roadmap_v2_extra.rs::test_chain_command -->
- [x] **AFTER PREVIOUS keyword** <!-- test: src/roadmap_v2/parser.rs::tests::test_add_after_previous -->
- [x] **AFTER TEXT "exact" match** <!-- test: src/roadmap_v2/parser.rs::tests::test_add_after_text -->
- [x] **AFTER LINE N match** <!-- test: src/roadmap_v2/parser.rs::tests::test_add_after_line -->
- [x] **Pre-validation: all AFTER targets exist** <!-- test: [no-test] -->
- [x] **Pre-validation: no slug collisions** <!-- test: [no-test] -->
- [x] **Pre-validation: no circular AFTER chains** <!-- test: [no-test] -->
- [x] **Fuzzy match suggestions on AFTER miss** <!-- test: [no-test] -->
- [x] **Atomic file write (temp → rename)** <!-- test: tests/integration_backup.rs::test_backup_dir_created -->
- [x] **Fix fingerprint math** <!-- test: src/audit/fingerprint.rs::test_similarity_math -->
- [x] **Wire up language detection for pattern queries** <!-- test: [no-test] -->
- [x] **Delete src/audit/mod_append.rs** <!-- test: [no-test] -->
- [x] **Extract node matching logic in enhance.rs** <!-- test: [no-test] -->
- [x] **Final polish of God Tier Audit** <!-- test: [no-test] -->

---

## v0.7.1 — Truth & Hardening 🔄 CURRENT ?? CURRENT

### Truth
- [x] **Re-baseline roadmap (tasks.toml overhaul)** <!-- test: [no-test] -->

### Validation
- [x] **Reject markdown fences in non-markdown files** <!-- test: tests/integration_apply.rs::test_rejects_markdown_fences -->
- [x] **Roadmap audit execution (--exec)** <!-- test: tests/unit_cli_roadmap.rs::test_roadmap_audit -->

---

## v0.7.2 — Apply Transactionality

### Manifest
- [ ] **Manifest integrity: Enforce all NEW/UPDATE files exist in extracted**
- [ ] **Manifest integrity: Enforce DELETE entries have no extracted content**

### Filesystem
- [x] **Symlink escape protection** <!-- test: tests/integration_apply.rs::test_block_windows_absolute -->

### Transactionality
- [ ] **Apply-level rollback (restore from backup on failure)**
- [ ] **Backup retention cleanup (bounded growth)**
- [ ] **Dry-run mode (show plan + diff summary)**

### Stability
- [ ] **Remove panics from AST analysis queries**

---

## v0.7.3 — CLI & Git Controls

### Verification
- [ ] **Verification pipeline supports quoted args**
- [ ] **Clear stage reporting (which check failed, why)**

### Git Automation
- [ ] **Honor preferences.auto_commit and auto_push**
- [ ] **CLI overrides (--no-commit, --no-push)**

### CLI Polish
- [ ] **Standardize exit codes (0/1/2)**

---

## v0.7.4 — Self-Hosting Lock

### Self-Hosting
- [x] **Remove .unwrap()/.expect() from tests** <!-- test: tests/unit_god_tier.rs::test_enhance_plan_generation -->
- [ ] **Ensure codebase meets token/complexity constraints**

### CI/CD
- [ ] **CI baseline (GitHub Actions workflow)**

---

## v0.8.0 — Release Candidate

### Polish
- [ ] **Lock CLI help text + examples**
- [ ] **Harden error messages (apply rejection UX)**

### Documentation
- [ ] **Final doc pass (README/DESIGN/CHANGELOG)**

---

## v1.0.0 — Release

### Distribution
- [ ] **Publish to crates.io**
- [ ] **Homebrew formula**
- [ ] **Windows install path (Scoop/Winget)**
- [ ] **GitHub Releases binaries**

### Hygiene
- [ ] **License + dependency license audit**

---

## v2.0.0 — Productivity Suite (Daemon, Viz, Audit)

### Daemon
- [ ] **slopchop watch command (Daemon)**
- [ ] **Global hotkey registration**
- [ ] **System notifications**

### Visualization
- [ ] **slopchop graph command (Visualization)**

### Context
- [ ] **Error-driven packing (Smart Context)**

### Audit
- [ ] **God Tier Audit: Rewriting (call-sites, bodies)**

### Adoption
- [ ] **Legacy adoption baseline mode**

---

