# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2025-12-26

### Added

- **Transport Hardening**: parser now tolerates common AI artifacts on sigil lines (indentation, list markers, blockquotes) to prevent copy-paste failures from chat UIs.
- **Sanitization Mode**: `slopchop apply --sanitize` (default for clipboard) strips UI-injected markdown code fences from `FILE` blocks in non-markdown files. Use `--strict` to disable.
- **Enhanced Patch Diagnostics**: `PATCH` failures with 0 matches now display a bounded visual diff summary and a "Did you mean?" probe to help locate the correct context.
- **Indentation Tolerance**: `PATCH` metadata (like `BASE_SHA256`) is now parsed correctly even if indented.

## [1.0.0] - 2025-12-25

### Added

- **Staged Workspace Architecture**: All changes via `slopchop apply` are now written to `.slopchop/stage/worktree/` by default, never directly to your repository. This ensures your codebase is never left in a broken state.
- **Transactional Promotion**: Use `slopchop apply --promote` to atomically move verified changes from the stage to your real workspace, with automatic backup and rollback on failure.
- **Context-Anchored Patching**: New `PATCH` block format with `BASE_SHA256` verification prevents stale overwrites. Patches require exact context matching to apply.
- **Audit Trail**: All operations are logged to `.slopchop/events.jsonl` for traceability.
- **XSC7XSC Protocol**: A markdown-inert, shell-safe sigil protocol for AI-generated payloads that won't be corrupted by renderers.
- **AST-Based Analysis**: Uses tree-sitter for Rust, Python, and TypeScript to enforce complexity limits.
- **Weisfeiler-Lehman Fingerprinting**: `slopchop audit` finds structurally similar code even when variable names differ.
- **Holographic Signatures**: `slopchop signatures` generates type-surface maps with PageRank-based importance ranking.
- **Focus Mode**: `slopchop pack --focus <file>` provides full source for target files with automatic skeleton generation for dependencies.

### The Three Laws

SlopChop enforces three core structural constraints:

1. **Law of Atomicity**: Files must stay below 2000 tokens (configurable)
2. **Law of Complexity**: Cyclomatic complexity ≤ 8, nesting depth ≤ 3, arguments ≤ 5
3. **Law of Paranoia**: Discourages `.unwrap()` and `.expect()`, requires `// SAFETY:` comments for unsafe blocks

### Commands

- `slopchop scan` - Run structural analysis
- `slopchop check` - Run full verification pipeline (your tests + scan)
- `slopchop apply` - Apply changes to stage
- `slopchop apply --promote` - Promote staged changes to workspace
- `slopchop apply --reset` - Clear the staging area
- `slopchop pack` - Generate AI context file
- `slopchop audit` - Find consolidation opportunities
- `slopchop trace` - Smart context generation with dependency tracking
- `slopchop map` - Show repository structure
- `slopchop signatures` - Generate type-surface map
- `slopchop fix` - Run configured fix commands
- `slopchop prompt` - Generate system prompt for AI assistants
- `slopchop dashboard` - Interactive TUI

### Breaking Changes from 0.x

- `slopchop apply` no longer writes directly to your workspace. Use `--promote` to move changes from stage.
- The patch format has changed from simple SEARCH/REPLACE to context-anchored blocks with hash verification.

### Migration from 0.x

If you have existing workflows:

1. After `slopchop apply`, changes are now in `.slopchop/stage/worktree/`
2. Run `slopchop check` to verify
3. Run `slopchop apply --promote` to move to workspace

---

## [0.9.0] - 2025-12-20

### Added

- Initial staged workspace implementation
- Parser hardening with reserved-name protection
- Surgical patching with ambiguity rejection

### Changed

- Verification now runs against stage if present

---

## [0.8.0] - 2025-12-15

### Added

- Basic apply command
- Structural scanning
- Token counting with tiktoken

---

*For older versions, see the git history.*