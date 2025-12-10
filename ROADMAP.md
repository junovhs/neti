# SlopChop Protocol Roadmap

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

---

## v0.2.0 — The 3 Laws ?

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
- [x] **.expect() detection** <!-- test: tests/integration_core.rs::test_paranoia_expect_fails -->
- [x] **Safe alternatives allowed (.unwrap_or)** <!-- test: tests/integration_core.rs::test_paranoia_safe_alternatives_pass -->

### File Ignores
- [x] **slopchop:ignore (C-style //)** <!-- test: tests/integration_core.rs::test_slopchop_ignore_skips_file -->
- [x] **slopchop:ignore (Hash-style #)** <!-- test: tests/unit_analysis.rs::test_slopchop_ignore_hash -->
- [x] **slopchop:ignore (HTML-style)** <!-- test: tests/unit_analysis.rs::test_slopchop_ignore_html -->

---

## v0.3.0 — Apply System ?

### Protocol Format Extraction
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
- [x] **[DELETE] marker detection** <!-- test: [no-test] -->
- [x] **Default Update operation** <!-- test: tests/integration_apply.rs::test_unified_apply_combined -->

### File Writing
- [x] **Parent directory creation** <!-- test: tests/integration_backup.rs::test_path_structure -->
- [x] **File content writing** <!-- test: tests/integration_backup.rs::test_existing_backed_up -->
- [x] **Delete operation** <!-- test: [no-test] -->
- [x] **Written files tracking** <!-- test: tests/integration_backup.rs::test_existing_backed_up -->

### Backup System
- [x] **Backup directory creation** <!-- test: tests/integration_backup.rs::test_backup_dir_created -->
- [x] **Timestamp subfolder** <!-- test: tests/integration_backup.rs::test_timestamp_folder -->
- [x] **Existing file backup** <!-- test: tests/integration_backup.rs::test_existing_backed_up -->
- [x] **New file skip (no backup needed)** <!-- test: tests/integration_backup.rs::test_new_file_no_backup -->
- [x] **Backup path structure preserved** <!-- test: tests/integration_backup.rs::test_path_structure -->

---

## v0.4.0 — Safety & Validation ?

### Path Safety — Traversal
- [x] **Block ../ traversal** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_traversal -->
- [x] **Block .. prefix** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_traversal -->

### Path Safety — Absolute
- [x] **Block Unix absolute (/)** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_absolute -->
- [x] **Block Windows absolute (C:)** <!-- test: [no-test] -->

### Path Safety — Sensitive
- [x] **Block .git/** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_git -->
- [x] **Block .env** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_hidden -->
- [x] **Block .ssh/** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_hidden -->
- [x] **Block .aws/** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_hidden -->
- [x] **Block .gnupg/** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_hidden -->
- [x] **Block id_rsa** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_hidden -->
- [x] **Block credentials** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_hidden -->
- [x] **Block backup directory** <!-- test: [no-test] -->

### Path Safety — Hidden Files
- [x] **Block hidden files (.*)** <!-- test: tests/integration_apply.rs::test_path_safety_blocks_hidden -->
- [x] **Allow . and .. segments** <!-- test: tests/integration_apply.rs::test_path_safety_allows_valid -->

### Path Safety — Protected Files
- [x] **Block ROADMAP.md rewrite** [no-test]
- [x] **Case-insensitive protection** [no-test]

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
- [x] **--verbose progress** [no-test]

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

### File Path Clipboard
- [x] **Copy file path for attachment** [no-test]

---

## v0.6.0 — Roadmap System ?

### Roadmap Parsing
- [x] **Title extraction (# Title)** <!-- test: tests/unit_roadmap_v2.rs::test_generator_basic_markdown -->
- [x] **Section heading detection** <!-- test: tests/unit_roadmap_v2.rs::test_generator_basic_markdown -->
- [x] **Task checkbox detection** <!-- test: tests/unit_roadmap_v2.rs::test_generator_basic_markdown -->
- [x] **Task status: pending** <!-- test: tests/unit_roadmap_v2.rs::test_generator_basic_markdown -->
- [x] **Task status: complete** <!-- test: tests/unit_roadmap_v2.rs::test_generator_with_done_task -->
- [x] **Stats calculation** <!-- test: [no-test] -->
- [x] **Test anchor extraction** <!-- test: tests/unit_roadmap_v2.rs::test_generator_includes_test_anchors -->
- [x] **Task path generation** <!-- test: [no-test] -->
- [x] **Compact state display** <!-- test: [no-test] -->

### Slugification
- [x] **Lowercase conversion** <!-- test: [no-test] -->
- [x] **Special char to dash** <!-- test: [no-test] -->
- [x] **Number preservation** <!-- test: [no-test] -->

### Command Parsing
- [x] **===ROADMAP=== block detection** <!-- test: tests/unit_roadmap_v2.rs::test_store_check_command -->
- [x] **CHECK command** <!-- test: tests/unit_roadmap_v2.rs::test_store_check_command -->
- [x] **UNCHECK command** <!-- test: tests/unit_roadmap_v2.rs::test_store_uncheck_command -->
- [x] **ADD command** <!-- test: tests/unit_roadmap_v2.rs::test_store_add_command -->
- [x] **ADD with AFTER** <!-- test: [no-test] -->
- [x] **UPDATE command** <!-- test: tests/unit_roadmap_v2.rs::test_store_update_command -->
- [x] **NOTE command** <!-- test: [no-test] -->
- [x] **MOVE command** <!-- test: [no-test] -->
- [x] **Comment skipping** <!-- test: [no-test] -->
- [x] **Summary generation** <!-- test: tests/unit_roadmap_v2.rs::test_generator_basic_markdown -->

### Roadmap CLI
- [x] **roadmap init** <!-- test: [no-test] -->
- [x] **roadmap prompt** <!-- test: [no-test] -->
- [x] **roadmap apply** <!-- test: tests/integration_apply.rs::test_unified_apply_roadmap -->
- [x] **roadmap show** <!-- test: [no-test] -->
- [x] **roadmap tasks** <!-- test: [no-test] -->
- [x] **roadmap tasks --pending** <!-- test: [no-test] -->
- [x] **roadmap tasks --complete** <!-- test: [no-test] -->
- [x] **roadmap audit** <!-- test: [no-test] -->

### Unified Apply
- [x] **Detect ===ROADMAP=== in apply** <!-- test: tests/integration_apply.rs::test_unified_apply_roadmap -->
- [x] **Apply roadmap + files together** <!-- test: tests/integration_apply.rs::test_unified_apply_combined -->

---

## v0.7.0 — Test Traceability 🔄 CURRENT ?? CURRENT

- [x] **Clean check output (failures only)** <!-- test: [no-test] -->
- [x] **Fix clippy must_use in git.rs** <!-- test: [no-test] -->
- [x] **Reduce complexity in dashboard nav** <!-- test: [no-test] -->
- [x] **Prevent false positive roadmap parsing on inline text** <!-- test: tests/repro_roadmap_hardening.rs::test_roadmap_parser_ignores_inline_markers -->
- [x] **Fix TUI corruption during apply by suspending terminal mode** <!-- test: [no-test] -->
- [x] **Split TUI dashboard into smaller modules (input/apply)** <!-- test: [no-test] -->
- [x] **Implement interactive roadmap in TUI (scroll, toggle)** <!-- test: [no-test] -->
- [x] **Add config option to require plan blocks** <!-- test: [no-test] -->
- [x] **Implement Weisfeiler-Lehman AST fingerprinting for structural similarity** <!-- test: [no-test] -->
- [x] **Implement Union-Find clustering for duplicate detection** <!-- test: [no-test] -->
- [x] **Implement call graph reachability for dead code detection** <!-- test: [no-test] -->
- [x] **Implement tree-sitter pattern matching for idiom detection** <!-- test: [no-test] -->
- [x] **Implement impact scoring and prioritization system** <!-- test: [no-test] -->
- [x] **Add slopchop audit command with format options** <!-- test: [no-test] -->
- [x] **Implement terminal, JSON, and AI output formatters** <!-- test: [no-test] -->
- [ ] **Integrate RepoGraph into signatures command (switch from alphabetical to graph-based)**
- [ ] **Implement Topological Sort for signature output (base dependencies first)**
- [ ] **Add PageRank importance tags (e.g., [CORE ARCHITECTURE]) to signatures**
- [ ] **Extract and display first-line docstrings in signature output**

### Parser Hardening
- [ ] **Empty task ID filtering**
- [x] **Task ID collision detection** <!-- test: tests/unit_roadmap_v2.rs::test_duplicate_add_rejected -->
- [x] **Anchor-based task matching** <!-- test: [no-test] -->
- [x] **Smart UPDATE inference (vs DELETE+ADD)** <!-- test: [no-test] -->

### Audit System
- [x] **Scan completed tasks** <!-- test: [no-test] -->
- [x] **[no-test] skip** <!-- test: tests/unit_roadmap_v2.rs::test_generator_notest_marker -->
- [x] **Explicit anchor verification** <!-- test: [no-test] -->
- [x] **Missing test file detection** <!-- test: [no-test] -->
- [x] **Missing test function detection** <!-- test: [no-test] -->
- [ ] **Test execution verification (cargo test)**
- [x] **Exit code 1 on any failure** [no-test]
- [x] **--strict mode (all must pass)** [no-test]

### Self-Hosting
- [x] **SlopChop passes own rules** <!-- test: [no-test] -->

### Test Naming Convention
- [ ] **Feature ID → test function mapping**
- [x] **Audit validates naming convention** <!-- test: [no-test] -->
- [ ] **Enforce test-first registration during AI sessions**
- [ ] **Block feature completion without matching test file**
- [ ] **slopchop register <feature> command (pre-declares intent)**
- [ ] **slopchop verify-session command (audit current work)**
- [ ] **Incremental roadmap sync (update after each file)**
- [ ] **Session manifest tracking (files touched this session)**

### Roadmap Hardening
- [x] **SECTION command (create version headers)** <!-- test: [no-test] -->
- [x] **SUBSECTION command (create ### headers)** <!-- test: [no-test] -->
- [x] **CHAIN command (sequential adds)** <!-- test: [no-test] -->
- [ ] **AFTER PREVIOUS keyword**
- [ ] **AFTER TEXT "exact" match**
- [ ] **AFTER LINE N match**
- [ ] **IN "section/subsection" location**
- [ ] **Slug echo on ADD (show generated slug)**
- [ ] **Pre-validation: all AFTER targets exist**
- [ ] **Pre-validation: no slug collisions**
- [ ] **Pre-validation: no circular AFTER chains**
- [ ] **Batch dependency resolution (topological sort)**
- [ ] **Fuzzy match suggestions on AFTER miss**
- [ ] **Dry-run mode (--dry-run flag)**
- [ ] **Atomic file write (temp → rename)**
- [ ] **Backup creation (.md.bak)**
- [ ] **All-or-nothing execution (rollback on error)**
- [ ] **Verbose plan output**

### Escape Hatches
- [ ] **slopchop apply --force flag**
- [ ] **Quarantine mode (// slopchop:quarantine marker)**
- [ ] **Quarantine report in slopchop scan**

### God Tier Audit
- [x] **Fix existing violations in src/audit** <!-- test: tests/unit_god_tier.rs::test_enhance_plan_generation -->
- [ ] **Implement Structural Diff Engine (diff.rs)**
- [ ] **Implement Parameterization Inference (parameterize.rs)**
- [ ] **Implement Code Generation (codegen.rs)**
- [ ] **Implement Call Site Analysis (callsites.rs)**
- [ ] **Implement Diagnostic Display (rustc-style)**

---

## v0.8.0 — Dependency Graph ?

### Daemon Core
- [ ] **Background process management**
- [ ] **Graceful shutdown (SIGTERM)**
- [ ] **Single instance enforcement**

### Clipboard Monitoring
- [ ] **Clipboard polling loop**
- [ ] **Protocol detection in clipboard**
- [ ] **Deduplication (ignore same content)**
- [ ] **Stage content for apply**

### Global Hotkey
- [ ] **Register global hotkey (default ⌘⇧L / Ctrl+Shift+L)**
- [ ] **Configurable hotkey binding**
- [ ] **Hotkey triggers apply**

### System Notifications
- [ ] **"Ready" notification on protocol detect**
- [ ] **Success notification with file count**
- [ ] **Failure notification with error summary**
- [ ] **Cross-platform notification (Linux/macOS/Windows)**

### Configuration
- [ ] **[watch] section in slopchop.toml**
- [ ] **hotkey option**
- [ ] **auto_commit option**
- [ ] **notify option (enable/disable)**

### Import Extraction
- [x] **Rust use/mod extraction** <!-- test: src/graph/imports.rs::test_rust_imports -->
- [x] **Rust path resolution** <!-- test: src/graph/resolver.rs::test_resolve_rust_crate -->
- [x] **Python import extraction** <!-- test: src/graph/imports.rs::test_python_imports -->
- [x] **TypeScript import extraction** <!-- test: src/graph/imports.rs::test_ts_imports -->

### Graph Construction
- [x] **Graph node/edge creation** <!-- test: tests/unit_graph_build.rs::test_node_creation -->
- [x] **Reverse index construction** <!-- test: tests/unit_graph_build.rs::test_reverse_index -->

### TUI Mission Control
- [x] **Unified Dashboard** <!-- test: [no-test] -->
- [x] **Check Runner** <!-- test: [no-test] -->
- [x] **Roadmap Explorer** <!-- test: [no-test] -->
- [x] **Log Stream** <!-- test: [no-test] -->
- [ ] **View Filters**
- [ ] **Copy View to Clipboard**
- [ ] **Mouse Support**
- [ ] **Interactive Staging Workflow**

- [x] **Simplify apply workflow: Non-blocking git check, summary reporting, auto-clipboard.** <!-- test: [no-test] -->

---

## v0.9.0 — Smart Context ?

### Daemon Core
- [ ] **slopchop watch command**

### SlopChop Map Command
- [x] **slopchop map basic output** <!-- test: [no-test] -->
- [x] **Directory tree with file counts** <!-- test: [no-test] -->
- [ ] **Cluster summary display**
- [x] **--deps flag (show dependency arrows)** <!-- test: [no-test] -->
- [x] **--stats flag (token counts per cluster)** <!-- test: [no-test] -->
- [ ] **--json flag (machine-readable map)**
- [ ] **Module description extraction (//! or docstring)**
- [ ] **Entry point detection (main.rs, lib.rs, index.ts)**

### Error-Driven Packing
- [ ] **slopchop pack --from-errors flag**
- [ ] **Cargo/rustc error parsing**
- [ ] **Clippy warning parsing**
- [ ] **TypeScript/tsc error parsing**
- [ ] **Python traceback parsing**
- [ ] **ESLint output parsing**
- [ ] **File path extraction from errors**
- [ ] **Line number extraction from errors**
- [ ] **Unique file deduplication**
- [ ] **Auto-include test files for src errors**
- [ ] **Piped input support (cargo clippy 2>&1 |)**
- [ ] **--from-clipboard-errors flag**

### Cluster Packing
- [ ] **slopchop pack --cluster NAME flag**
- [ ] **Cluster resolution by name**
- [ ] **Cluster resolution by directory path**
- [ ] **--with-tests flag (include test files)**
- [ ] **--with-boundary flag (skeleton boundary files)**
- [ ] **--no-boundary flag (exclude boundary files)**
- [ ] **Multiple cluster inclusion (--cluster a --cluster b)**

### Trace Packing
- [ ] **slopchop pack --trace PATH flag**
- [ ] **--depth N limit for trace**
- [ ] **--forward flag (dependencies only)**
- [ ] **--reverse flag (dependents only)**
- [ ] **Default: bidirectional trace**
- [ ] **Trace + skeleton hybrid output**
- [ ] **Multiple trace roots (--trace a --trace b)**

### Context Ordering
- [ ] **Dependency-first ordering (topological)**
- [ ] **Leaf files appear first**
- [ ] **Target/focus file appears last**
- [ ] **Circular dependency handling (break arbitrarily)**
- [ ] **Shared dependency hoisting**

### AI Context Protocol
- [x] **CONTEXT_REQUEST format specification** [no-test]
- [ ] **AI can emit cluster requests**
- [ ] **AI can emit trace requests**
- [ ] **AI can emit file requests**
- [ ] **slopchop fulfill command (parse AI request)**
- [ ] **Request validation (cluster/file exists)**

### God Tier Audit
- [x] **God Tier Audit: AST Diffing (diff.rs)** <!-- test: tests/unit_god_tier.rs::test_diff_simple_variant -->
- [x] **God Tier Audit: Parameterization Inference (parameterize.rs)** <!-- test: tests/unit_god_tier.rs::test_parameterize_strategy_enum -->
- [x] **God Tier Audit: Refactor CodeGen (codegen.rs)** <!-- test: tests/unit_god_tier.rs::test_codegen_enum -->
- [x] **God Tier Audit: Opportunity Enhancement (enhance.rs)** <!-- test: tests/unit_god_tier.rs::test_enhance_plan_generation -->
- [ ] **God Tier Audit: Call Site Rewriting (integrate callsites.rs, show before/after)**
- [ ] **God Tier Audit: Full Function Bodies (thread hole values to codegen)**

- [ ] **God Tier Audit: Source Markers (^ pointing to differences)**

---

## v0.10.0 — Validation Hardening

### Markdown Rejection
- [ ] **Block triple backticks (```)** <!-- test: tests/integration_apply.rs::test_rejects_markdown_fences -->
- [ ] **Block tilde fences (~~~)** <!-- test: tests/integration_apply.rs::test_rejects_tilde_fences -->
- [x] **Markdown fence rejection rationale** [no-test]

### Brace Balancing
- [ ] **Detect unbalanced {** <!-- test: tests/integration_apply.rs::test_detects_unbalanced_open_brace -->
- [ ] **Detect unbalanced }** <!-- test: tests/integration_apply.rs::test_detects_unbalanced_close_brace -->
- [ ] **Detect unbalanced [** <!-- test: tests/integration_apply.rs::test_detects_unbalanced_bracket -->
- [ ] **Detect unbalanced (** <!-- test: tests/integration_apply.rs::test_detects_unbalanced_paren -->
- [x] **Brace balance algorithm selection** [no-test]
- [ ] **String literal exclusion from brace count**
- [ ] **Comment exclusion from brace count**

---

## v0.11.0 — CI/CD Integration

### Output Formats
- [ ] **--format json**
- [ ] **SARIF output for GitHub**

### Git Hooks
- [ ] **slopchop hook install**
- [ ] **Pre-commit hook script**

### Exit Codes
- [ ] **Exit 0 on clean**
- [ ] **Exit 1 on violations**
- [ ] **Exit 2 on error**

### CI Templates
- [ ] **GitHub Actions workflow template**
- [ ] **GitLab CI template**
- [ ] **slopchop init --ci flag (generate workflow)**
- [ ] **Fail-fast vs report-all modes**
- [ ] **Annotation output for GitHub PR comments**

---

## v0.12.0 — Graph Visualization

### Visualization Formats
- [ ] **slopchop graph command**
- [ ] **DOT format export (Graphviz)**
- [ ] **Mermaid format export**
- [ ] **--cluster-only flag (show clusters, not files)**
- [ ] **--highlight PATH flag (color specific subgraph)**
- [ ] **Interactive HTML export (D3.js)**
- [ ] **Terminal ASCII graph (small projects)**

---

## v0.13.0 — Legacy Adoption

### Baseline System
- [ ] **slopchop baseline command (snapshot current violations)**
- [ ] **Baseline file format (.slopchop-baseline.json)**
- [ ] **Baseline comparison mode (only report new violations)**
- [ ] **--baseline flag for slopchop scan**
- [ ] **Auto-generate slopchop:ignore for existing violations**
- [x] **Gradual tightening guide** [no-test]

---

## v1.0.0 — Release

### Distribution
- [x] **Published to crates.io** [no-test]
- [x] **Homebrew formula** [no-test]
- [x] **Scoop/Winget packages** [no-test]

### Documentation
- [x] **Documentation site** [no-test]
- [x] **Logo and branding** [no-test]
- [x] **README finalized** [no-test]
- [x] **CHANGELOG.md generation** [no-test]
- [x] **CONTRIBUTING.md guide** [no-test]
- [x] **Security policy (SECURITY.md)** [no-test]

### Polish
- [ ] **License audit (dependency licenses)** <!-- test: tests/release.rs::test_license_audit -->
- [x] **Binary size optimization** [no-test]
- [x] **Startup time benchmarking** [no-test]
- [x] **Cross-compilation CI (linux/mac/windows)** [no-test]

---

