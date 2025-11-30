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
- [x] **Command string parsing** <!-- test: tests/unit_config.rs::test_command_single -->
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
- [x] **Rust complexity query (if/match/for/while/&&/||)** <!-- test: tests/integration_core.rs::test_complexity_simple_function_passes -->
- [x] **Complexity violation detection** <!-- test: tests/integration_core.rs::test_complexity_branchy_function_fails -->
- [x] **JS/TS complexity query** <!-- test: tests/unit_analysis.rs::test_js_complexity -->
- [x] **Python complexity query** <!-- test: tests/unit_analysis.rs::test_python_complexity -->

### Law of Complexity — Nesting Depth
- [x] **Depth calculation (block/body traversal)** <!-- test: tests/integration_core.rs::test_nesting_shallow_passes -->
- [x] **Deep nesting violation** <!-- test: tests/integration_core.rs::test_nesting_deep_fails -->

### Law of Complexity — Arity
- [x] **Parameter counting** <!-- test: tests/integration_core.rs::test_arity_few_args_passes -->
- [x] **High arity violation** <!-- test: tests/integration_core.rs::test_arity_many_args_fails -->

### Law of Complexity — Naming
- [x] **Snake_case word counting** <!-- test: tests/unit_analysis.rs::test_snake_case_words -->
- [x] **CamelCase word counting** <!-- test: tests/unit_analysis.rs::test_camel_case_words -->
- [x] **Naming ignore patterns** <!-- test: tests/unit_config.rs::test_ignore_naming_on -->

### Law of Paranoia (Rust)
- [x] **Banned call query (.unwrap/.expect)** <!-- test: tests/integration_core.rs::test_paranoia_unwrap_fails -->
- [x] **.expect() detection** <!-- test: tests/integration_core.rs::test_paranoia_expect_fails -->
- [x] **Safe alternatives allowed (.unwrap_or)** <!-- test: tests/integration_core.rs::test_paranoia_no_unwrap_passes -->

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
- [x] **Violation injection <!-- test: tests/unit_pack_violations.rs::test_violations_injected -->**

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
- [x] **UNCHECK command** <!-- test: tests/integration_roadmap.rs::test_parse_multiple_commands -->
- [x] **ADD command** <!-- test: tests/integration_roadmap.rs::test_parse_multiple_commands -->
- [x] **ADD with AFTER** <!-- test: tests/integration_roadmap.rs::test_parse_add_with_after -->
- [x] **DELETE command** <!-- test: tests/unit_roadmap.rs::test_delete_command -->
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

### Audit System
- [x] **Scan completed tasks** <!-- test: tests/integration_audit.rs::test_scans_completed_only -->
- [x] **[no-test] skip** <!-- test: tests/integration_audit.rs::test_no_test_skipped -->
- [x] **Explicit anchor verification** <!-- test: tests/integration_audit.rs::test_explicit_anchor_verified -->
- [x] **Missing test file detection** <!-- test: tests/integration_audit.rs::test_missing_file_detected -->
- [ ] **Missing test function detection**
- [ ] **Test execution verification (cargo test)**
- [ ] **Exit code 1 on any failure**
- [ ] **--strict mode (all must pass)**

### Self-Hosting
- [ ] **Warden passes own rules** <!-- test: tests/integration_self_host.rs::test_warden_passes_own_rules -->

### Test Naming Convention
- [ ] **Feature ID → test function mapping**
- [ ] **Audit validates naming convention**

---

## v0.8.0 — Missing Features

*Features claimed but not implemented.*

### Markdown Rejection
- [ ] **Block triple backticks (```)** <!-- test: tests/integration_apply.rs::test_rejects_markdown_fences -->
- [ ] **Block tilde fences (~~~)** <!-- test: tests/integration_apply.rs::test_rejects_tilde_fences -->

### Brace Balancing
- [ ] **Detect unbalanced {** <!-- test: tests/integration_apply.rs::test_detects_unbalanced_open_brace -->
- [ ] **Detect unbalanced }** <!-- test: tests/integration_apply.rs::test_detects_unbalanced_close_brace -->
- [ ] **Detect unbalanced [** <!-- test: tests/integration_apply.rs::test_detects_unbalanced_bracket -->
- [ ] **Detect unbalanced (** <!-- test: tests/integration_apply.rs::test_detects_unbalanced_paren -->

---

## v0.9.0 — CI/CD Integration

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

---

## Principles

1. **Every [x] feature has a verified test** — No exceptions (except [no-test])
2. **Reject bad input, don't fix it** — Warden is a gatekeeper
3. **Git is the undo system** — Don't reinvent version control
4. **Explicit > Magic** — Fail loudly on format violations
5. **Containment over craftsmanship** — Constraints are safety, not style
6. **Self-hosting** — Warden passes its own rules

---

## Not Doing

- **VS Code Extension** — IDE lock-in, maintenance burden
- **Watch mode** — Complexity without clear benefit
- **Markdown fallback parsing** — Enforce format discipline
- **"Smart" fixing** — Warden rejects, doesn't repair
