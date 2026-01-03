# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.3.4] - 2026-01-03

### Added
- **Interactive Configuration**: Added `slopchop config` command for TUI-based setting management (using crossterm).
- **Fix Packet Persistence**: Added option to write AI fix packets to file (`write_fix_packet = true`) instead of clipboard.
- **Config Fields**: Added `auto_promote`, `write_fix_packet`, `fix_packet_path` to preferences.

### Fixed
- **Paste-Back UX**: Restored auto-copy of AI feedback to clipboard when verification fails.
- **Auto-Copy Logic**: Fixed bug where `slopchop pack` ignored the `auto_copy` preference from config.
- **Command Output**: Verification failure now generates a structured, copy-pasteable report for the AI.

## [1.3.3] - 2026-01-03

### Fixed
- **CRLF Hash Normalization**: Fixed critical bug where CRLF and LF files produced different SHA256 hashes, causing patch verification to fail unpredictably on Windows.
- **Multi-Patch Verification**: Fixed bug where multiple patches to the same file in a single payload would fail. Hash is now only verified on the first patch to each file.

### Changed
- **Unified Hash Function**: Consolidated duplicate `compute_hash()` implementations into single `compute_sha256()` with line ending normalization.
- **Patch API**: Added `apply_with_options(skip_hash)` to support chained patches without re-verification.

### Added
- **Hash Stability Tests**: Added `test_eol_normalization` and `test_hash_stability` unit tests.

## [1.3.2] - 2026-01-02

### Added
- **Split-Brain Protection**: Promotion now performs a pre-flight "base hash" verification to ensure workspace files haven't been manually modified during the staging period.
- **Stress Testing Suite**: Completed physical verification of 42 adversarial cases (paths, sigils, concurrency).

### Fixed
- **S03 (Path Traversal)**: Hardened path validator with byte-level null character detection to prevent truncation attacks.
- **I01 (Sigil Injection)**: Redesigned the parser to use prefixed terminator sigils, preventing greedy matching of embedded protocol markers.

## [1.3.1] - 2026-01-01

### Added
- **Locality v2**: Automated layer inference (toposort-based) and cycle detection for module-level dependencies.
- **Directional Coupling**: Enhanced coupling heuristics to distinguish between intentional directional dependencies and problematic bidirectional cycles.

## [1.3.0] - 2025-12-29

### Added
- **Prescriptive Violations**: Error output now includes ANALYSIS and SUGGESTION sections.
- **Modular Checks**: Split analysis into `checks/naming.rs`, `checks/complexity.rs`, `checks/banned.rs`.

### Removed
- **Commands**: Removed `trace`, `fix`, `prompt`, `dashboard` commands.
- **Code**: Deleted ~2000 lines of unused code (`src/trace/`, `src/tui/`).

### Fixed
- **Self-Violation**: Split `src/apply/parser.rs` into `parser.rs` + `blocks.rs` to comply with token limits.

## [1.1.0] - 2025-12-26

### Added
- **Transport Hardening**: Parser now tolerates common AI artifacts on sigil lines (indentation, list markers, blockquotes) to prevent copy-paste failures from chat UIs.
- **Sanitization Mode**: `slopchop apply --sanitize` (default for clipboard) strips UI-injected markdown code fences from `FILE` blocks in non-markdown files. Use `--strict` to disable.
- **Enhanced Patch Diagnostics**: `PATCH` failures with 0 matches now display a bounded visual diff summary and a "Did you mean?" probe to help locate the correct context.
- **Indentation Tolerance**: `PATCH` metadata (like `BASE_SHA256`) is now parsed correctly even if indented.

## [1.0.0] - 2025-12-25

### Added
- **Staged Workspace Architecture**: All changes via `slopchop apply` are now written to `.slopchop/stage/worktree/` by default.
- **Transactional Promotion**: Use `slopchop apply --promote` to atomically move verified changes from the stage to your real workspace.
- **Context-Anchored Patching**: New `PATCH` block format with `BASE_SHA256` verification prevents stale overwrites.
- **Audit Trail**: All operations are logged to `.slopchop/events.jsonl` for traceability.
- **XSC7XSC Protocol**: A markdown-inert, shell-safe sigil protocol for AI-generated payloads.
- **AST-Based Analysis**: Uses tree-sitter for Rust, Python, and TypeScript to enforce complexity limits.
- **Focus Mode**: `slopchop pack --focus <file>` provides full source for target files with automatic skeleton generation for dependencies.

### Breaking Changes from 0.x
- `slopchop apply` no longer writes directly to your workspace. Use `--promote` to move changes from stage.
- The patch format has changed from simple SEARCH/REPLACE to context-anchored blocks with hash verification.

---

## [0.9.0] - 2025-12-20

### Added
- Initial staged workspace implementation
- Parser hardening with reserved-name protection
- Surgical patching with ambiguity rejection

---

## [0.8.0] - 2025-12-15

### Added
- Basic apply command
- Structural scanning
- Token counting with tiktoken

---

*For older versions, see the git history.*