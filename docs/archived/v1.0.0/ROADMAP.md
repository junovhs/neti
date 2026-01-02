# SlopChop Protocol Roadmap

The path to a hardened v1.0.0 trust boundary.

---

## v0.1.0 — Foundation ✅

### Token Counting
- [x] **Tokenizer initialization (cl100k_base)** <!-- test: tests/unit_tokens.rs::test_tokenizer_available -->
- [x] **Token count function** <!-- test: tests/unit_tokens.rs::test_count_basic -->
- [x] **Token limit check** <!-- test: tests/unit_tokens.rs::test_exceeds_limit -->

### Project Detection
- [x] **Rust project detection (Cargo.toml)** <!-- test: tests/unit_project.rs::test_detection_cases -->
- [x] **Node project detection (package.json)** <!-- test: tests/unit_project.rs::test_detection_cases -->
- [x] **Python project detection** <!-- test: tests/unit_project.rs::test_detection_cases -->
- [x] **Go project detection (go.mod)** <!-- test: tests/unit_project.rs::test_detection_cases -->

### Configuration
- [x] **TOML config loading** <!-- test: tests/unit_config.rs::test_load_toml -->
- [x] **Default rule values** <!-- test: tests/unit_config.rs::test_defaults -->
- [x] **Command list parsing** <!-- test: tests/unit_config.rs::test_command_list -->
- [x] **.slopchopignore loading** <!-- test: tests/unit_config.rs::test_slopchopignore -->
- [x] **Auto-config generation** [no-test]

---

## v0.2.0 — The 3 Laws ✅

### Law of Atomicity
- [x] **File token counting** <!-- test: tests/integration_core.rs::test_atomicity_boundary -->
- [x] **Token limit violation** <!-- test: tests/integration_core.rs::test_atomicity_boundary -->
- [x] **Token exemption patterns** <!-- test: tests/unit_config.rs::test_ignore_tokens_on -->

### Law of Complexity
- [x] **Rust complexity query** <!-- test: tests/integration_core.rs::test_complexity_boundary -->
- [x] **JS/TS complexity query** <!-- test: tests/unit_analysis.rs::test_js_complexity -->
- [x] **Python complexity query** <!-- test: tests/unit_analysis.rs::test_python_complexity -->
- [x] **Depth calculation** <!-- test: tests/integration_core.rs::test_nesting_boundary -->
- [x] **High arity violation** <!-- test: tests/integration_core.rs::test_arity_boundary -->
- [x] **Snake_case word counting** <!-- test: tests/unit_analysis.rs::test_snake_case_words -->
- [x] **CamelCase word counting** <!-- test: tests/unit_analysis.rs::test_camel_case_words -->
- [x] **Naming ignore patterns** <!-- test: tests/unit_config.rs::test_ignore_naming_on -->

### Law of Paranoia
- [x] **.unwrap()/.expect() detection** <!-- test: tests/integration_core.rs::test_paranoia_check -->
- [x] **Unsafe code justification requirement** <!-- test: src/analysis/safety.rs -->

### File Ignores
- [x] **slopchop:ignore (C-style //)** <!-- test: tests/unit_analysis.rs::check_ignore -->
- [x] **slopchop:ignore (Hash-style #)** <!-- test: tests/unit_analysis.rs::test_slopchop_ignore_hash -->
- [x] **slopchop:ignore (HTML-style)** <!-- test: tests/unit_analysis.rs::test_slopchop_ignore_html -->

---

## v0.3.0 — The XSC7XSC Protocol ✅

### Protocol Format
- [x] **Header detection (XSC7XSC FILE XSC7XSC)** <!-- test: tests/integration_apply.rs::test_extract_single_file -->
- [x] **Footer detection (XSC7XSC END XSC7XSC)** <!-- test: tests/integration_apply.rs::test_extract_single_file -->
- [x] **PLAN block extraction** <!-- test: tests/integration_apply.rs::test_extract_plan -->
- [x] **MANIFEST block extraction** <!-- test: tests/integration_apply.rs::test_unified_apply_combined -->

### Manifest Parsing
- [x] **[NEW] marker detection** <!-- test: tests/integration_apply.rs::test_unified_apply_combined -->
- [x] **[DELETE] marker detection** <!-- test: tests/integration_apply.rs::test_delete_marker_detection -->
- [x] **Update operation (default)** <!-- test: tests/integration_apply.rs::test_unified_apply_combined -->

---

## v0.4.0 — Safety & Validation ✅

### Path Safety
- [x] **Block ../ traversal** <!-- test: tests/integration_apply.rs::test_security_boundaries -->
- [x] **Block absolute paths (Unix/Windows)** <!-- test: tests/integration_apply.rs::test_security_boundaries -->
- [x] **Block sensitive dirs (.git, .ssh, .env)** <!-- test: tests/integration_apply.rs::test_security_boundaries -->
- [x] **Symlink escape protection** <!-- test: src/apply/writer.rs::check_symlink_escape -->

### Truncation Detection
- [x] **Pattern: // ... detection** <!-- test: tests/integration_apply.rs::test_truncation_detects_ellipsis_comment -->
- [x] **Empty file rejection** <!-- test: tests/integration_apply.rs::test_truncation_detects_empty_file -->
- [x] **Markdown fence detection in code** <!-- test: tests/integration_apply.rs::test_rejects_markdown_fences -->

---

## v0.5.0 — Pack & Context ✅

### Core Packing
- [x] **Repo knitting to context.txt** <!-- test: tests/integration_pack.rs -->
- [x] **Smart Copy (Clipboard vs Temp File)** <!-- test: src/clipboard/mod.rs -->
- [x] **--noprompt mode** <!-- test: tests/integration_pack.rs::test_reminder_is_concise -->

### Skeleton & Focus
- [x] **Skeletonization (Body stripping)** <!-- test: tests/integration_skeleton.rs -->
- [x] **Foveal/Peripheral focus mode** <!-- test: tests/integration_pack.rs::test_smart_context_focus_mode -->
- [x] **Topological ordering in context** <!-- test: tests/signatures_test.rs::test_holographic_signatures_topo_sort -->

---

## v0.6.0 — The Great Pivot ✅

### The Purge
- [x] **Decouple Git dependency** <!-- status: Decoupled -->
- [x] **Remove Roadmap V2 bloat** <!-- status: Deleted -->
- [x] **Refactor Audit for the 3 Laws** <!-- test: tests/unit_god_tier.rs -->

### Audit System
- [x] **AST Diffing** <!-- test: tests/unit_god_tier.rs::test_diff_simple_variant -->
- [x] **Duplicate clustering (Union-Find)** <!-- test: src/audit/similarity.rs -->
- [x] **Impact scoring** <!-- test: src/audit/scoring.rs -->

---

## v0.7.0 — Staged Workspace (Foundation) ✅

### Shadow Worktree
- [x] **Implicit stage creation** <!-- test: tests/integration_stage_lifecycle.rs::test_stage_creates_worktree_dir -->
- [x] **Isolation (Exclude .slopchop from stage)** <!-- test: tests/integration_stage_lifecycle.rs::test_stage_does_not_copy_slopchop_into_itself -->
- [x] **Apply redirects writes to stage** <!-- test: tests/integration_stage_lifecycle.rs::test_apply_writes_to_stage_not_real_workspace -->

### Stage State
- [x] **Persistent Stage ID** <!-- test: tests/integration_stage_lifecycle.rs::test_stage_id_persists_across_loads -->
- [x] **Touched path tracking** <!-- test: tests/integration_stage_lifecycle.rs::test_stage_tracks_written_paths -->
- [x] **Apply count tracking** <!-- test: tests/integration_stage_lifecycle.rs::test_apply_count_increments -->

---

## v0.8.0 — Transactional Integrity ✅

### Promotion Mechanics
- [x] **Atomic Promotion (Stage -> Repo)** <!-- test: tests/integration_stage_promote.rs::test_promote_only_applies_touched_paths -->
- [x] **Deletion support in promotion** <!-- test: tests/integration_stage_promote.rs::test_promote_handles_deletions -->
- [x] **Pre-promotion backup** <!-- test: tests/integration_stage_promote.rs::test_promote_creates_backup -->
- [x] **Promotion Rollback on failure** <!-- test: tests/integration_stage_promote.rs::test_promote_rollback_on_failure -->

### Workflow Integration
- [x] **Stage-aware `check` (uses stage cwd)** <!-- test: tests/integration_stage_lifecycle.rs::test_effective_cwd_uses_stage_when_present -->
- [x] **Stage-aware `pack` (reflects staged changes)** <!-- test: src/pack/mod.rs -->
- [x] **Manual Stage Reset (`--reset`)** <!-- test: tests/integration_stage_lifecycle.rs::test_stage_reset_removes_everything -->

---

## v0.9.0 — The Scalpel (Hardening) ✅

### Parser Hardening
- [x] **Enum-based Block Tokenizer**
- [x] **Strict Block Validation** (Reject unknown KINDs)
- [x] **Reserved Name Protection** (Block files named "MANIFEST", "PLAN", etc.)

### Surgical PATCH Blocks
- [x] **PATCH block extraction** (strict SEARCH/REPLACE form)
- [x] **Base SHA256 Verification** (Prevent stale patches)
- [x] **Exact Match Engine** (Reject if match is ambiguous or missing)
- [ ] **Context-Anchored PATCH (LEFT_CTX / OLD / RIGHT_CTX / NEW)** (Canonical v1.0 format)
- [ ] **Auto-Fallback to FILE** (Deferred)


---

## v1.0.0 — Production Ready ⏳ NEXT

### Patch UX
- [ ] **"Did you mean?" Diagnostics** (On match failure)
- [ ] **Visual Patch Diff Summary**

### Polish
- [ ] **Machine-Readable Event Log** (.slopchop/events.jsonl)
- [ ] **CLI Polish & Exit Code Standardization**
- [ ] **Distribution** (Scoop, Winget, Homebrew)
