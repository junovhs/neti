# SlopChop Design Document

> **Audience:** Developers (human or AI) working on or extending SlopChop.  
> **See also:** [README.md](README.md) for user guide.

---

## Table of Contents

1. [Vision & Philosophy](#vision--philosophy)
2. [Architecture Overview](#architecture-overview)
3. [The Three Laws](#the-three-laws)
4. [The SlopChop Protocol](#the-slopchop-protocol)
5. [Analysis Engine](#analysis-engine)
6. [Apply System](#apply-system)
7. [Context Generation](#context-generation)
8. [Dependency Graph](#dependency-graph)
9. [Roadmap System (V2)](#roadmap-system-v2)
10. [TUI Dashboard](#tui-dashboard)
11. [Security Model](#security-model)
12. [Key Decisions & Rationale](#key-decisions--rationale)
13. [Module Map](#module-map)
14. [Testing Philosophy](#testing-philosophy)
15. [Future Work](#future-work)

---

## Vision & Philosophy

### The Problem

AI coding assistants are powerful but unreliable. They:
- Generate files too large to review meaningfully
- Produce complex functions that can't be tested in isolation
- Truncate code with `// ...` or "rest of implementation"
- Escape markdown fences incorrectly, corrupting output
- Have no memory of project constraints between sessions

### The Solution

**SlopChop is a gatekeeper, not a fixer.** It creates a feedback loop:

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│   slopchop pack ──► AI ──► slopchop apply ──► verify ──► commit │
│        ▲                         │                              │
│        │                         ▼                              │
│        └────── rejection ◄───── FAIL                            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

When AI output violates constraints:
1. SlopChop rejects the entire response
2. Generates a structured error message
3. Copies it to clipboard for pasting back to AI
4. AI corrects and resubmits

**The AI learns the constraints through rejection, not instruction.**

### Core Principles

| # | Principle | Meaning |
|---|-----------|---------|
| 1 | **Reject bad input, don't fix it** | SlopChop is a gatekeeper, not a linter with autofix |
| 2 | **Git is the undo system** | Don't reinvent version control. Commit on success |
| 3 | **Explicit > Magic** | Fail loudly on format violations |
| 4 | **Context is king** | Give AI exactly what it needs, nothing more |
| 5 | **Graph over glob** | Understand structure, don't just pattern match |
| 6 | **Self-hosting** | SlopChop passes its own rules |

### What SlopChop Is NOT

- **Not a linter** — It doesn't suggest fixes, it rejects
- **Not an IDE plugin** — CLI-first, composable with any editor
- **Not AI-specific** — The constraints help human reviewers too
- **Not prescriptive about style** — Cares about size and complexity, not formatting

---

## Architecture Overview

```
src/
├── analysis/          # The Three Laws enforcement
│   ├── ast.rs         # Tree-sitter query compilation
│   ├── checks.rs      # Violation detection logic
│   ├── metrics.rs     # Complexity, depth, arity calculations
│   └── mod.rs         # RuleEngine orchestration
│
├── apply/             # AI response → filesystem
│   ├── extractor.rs   # Protocol parsing
│   ├── manifest.rs    # MANIFEST block parsing
│   ├── validator.rs   # Path safety, truncation detection
│   ├── writer.rs      # Atomic file writes with backup
│   ├── verification.rs# Post-apply check commands
│   ├── git.rs         # Git commit/push operations
│   ├── messages.rs    # Error message formatting
│   ├── types.rs       # ApplyContext, ApplyOutcome types
│   └── mod.rs         # Orchestration and flow control
│
├── graph/             # Dependency analysis
│   ├── imports.rs     # Import extraction per language
│   ├── resolver.rs    # Import → file path resolution
│   ├── defs/          # Definition extraction
│   │   ├── extract.rs # Symbol extraction from source
│   │   ├── queries.rs # Tree-sitter queries for symbols
│   │   └── mod.rs
│   └── rank/          # PageRank-based importance
│       ├── graph.rs   # RepoGraph structure
│       ├── pagerank.rs# PageRank algorithm
│       ├── tags.rs    # Tag kinds and definitions
│       └── mod.rs
│
├── pack/              # Context generation for AI
│   ├── formats.rs     # Output format handling
│   ├── focus.rs       # Focus mode computation
│   └── mod.rs         # Pack orchestration
│
├── trace/             # Smart context generation
│   ├── options.rs     # TraceOptions configuration
│   ├── output.rs      # Trace output rendering
│   ├── runner.rs      # Trace execution logic
│   └── mod.rs
│
├── roadmap_v2/        # TOML-based task management
│   ├── types.rs       # TaskStore, Task, Section types
│   ├── parser.rs      # Command parsing
│   ├── executor.rs    # Command execution
│   ├── storage.rs     # TOML serialization
│   └── cli/           # Subcommand handlers
│       ├── display.rs # Output formatting
│       ├── handlers.rs# Command implementations
│       └── mod.rs
│
├── tui/               # Terminal UI
│   ├── dashboard/     # Main dashboard with tabs
│   │   ├── mod.rs
│   │   ├── state.rs   # DashboardApp state
│   │   └── ui.rs      # Ratatui rendering
│   ├── config/        # Configuration editor
│   │   ├── components.rs
│   │   ├── helpers.rs
│   │   ├── state.rs   # ConfigApp state
│   │   └── view.rs
│   ├── view/          # Scan results viewer
│   │   ├── components.rs
│   │   └── layout.rs
│   ├── watcher.rs     # Clipboard monitoring
│   ├── runner.rs      # Terminal setup/restore
│   ├── state.rs       # App state for scan view
│   └── mod.rs
│
├── clipboard/         # Cross-platform clipboard
│   ├── linux.rs       # xclip/wl-copy + WSL support
│   ├── macos.rs       # pbcopy/pbpaste
│   ├── windows.rs     # PowerShell clipboard
│   ├── platform.rs    # Platform detection
│   ├── temp.rs        # Temp file management for large content
│   └── mod.rs         # smart_copy logic
│
├── cli/               # CLI command handlers
│   ├── handlers.rs    # All command implementations
│   └── mod.rs
│
├── config/            # Configuration management
│   ├── io.rs          # File I/O, TOML parsing
│   ├── types.rs       # Config, RuleConfig, Preferences
│   └── mod.rs
│
├── lang.rs            # Language detection and queries
├── skeleton.rs        # Code → signatures compression
├── signatures.rs      # Type surface map generation
├── discovery.rs       # File enumeration (git + walk)
├── tokens.rs          # tiktoken integration
├── prompt.rs          # System prompt generation
├── reporting.rs       # Scan report formatting
├── spinner.rs         # Loading indicator
├── project.rs         # Project type detection
├── wizard.rs          # Interactive config wizard
├── clean.rs           # Cleanup utilities
├── constants.rs       # Global constants
├── detection.rs       # File type detection
├── error.rs           # Error types
├── types.rs           # Shared types (Violation, FileReport, ScanReport)
└── lib.rs             # Public API (slopchop_core)
```

### Data Flow

```
User runs "slopchop pack --focus file.rs"
         │
         ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│    discovery    │────►│      graph      │────►│      pack       │
│   (find files)  │     │  (build deps)   │     │ (generate ctx)  │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                         │
                                                         ▼
                                                 context.txt + prompt
                                                         │
                                                    [TO AI]
                                                         │
                                                         ▼
                                                 AI response (Protocol)
                                                         │
                                                         ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│    extractor    │────►│    validator    │────►│     writer      │
│ (parse blocks)  │     │ (safety checks) │     │ (atomic write)  │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                         │
                                                         ▼
                                                 ┌───────────────┐
                                                 │ verification  │
                                                 │ (cargo test)  │
                                                 └───────────────┘
                                                         │
                                    ┌────────────────────┴────────────────────┐
                                    ▼                                         ▼
                              [PASS: commit]                          [FAIL: reject]
                                    │                                         │
                                    ▼                                         ▼
                              git commit/push                      copy feedback to clipboard
```

---

## The Three Laws

SlopChop enforces structural constraints. These are configurable but opinionated defaults.

### Law of Atomicity

**Files must be small enough to reason about.**

```toml
[rules]
max_file_tokens = 2000  # ~500 lines of code
```

**Why:** Large files can't be meaningfully reviewed. AI-generated code tends toward monolithic files. Forcing small files creates natural modularity.

### Law of Complexity

**Functions must be simple enough to test.**

```toml
[rules]
max_cyclomatic_complexity = 8   # Branches per function
max_nesting_depth = 3           # if/for/while depth
max_function_args = 5           # Parameter count
max_function_words = 5          # Words in function name
```

**Why:** 
- High complexity = hard to test exhaustively
- Deep nesting = hard to follow control flow
- Many arguments = function doing too much
- Long names = unclear responsibility

**Implementation:** Tree-sitter queries count:
- Complexity: `if`, `match`, `for`, `while`, `&&`, `||`
- Depth: Nested `block` and `body` nodes
- Arity: Children of `parameters`/`arguments` nodes

### Law of Paranoia (Rust-specific)

**No panic paths in production code.**

```rust
// REJECTED
let value = thing.unwrap();
let other = thing.expect("msg");

// ALLOWED
let value = thing.unwrap_or(default);
let value = thing.unwrap_or_else(|| compute());
let value = thing?;
```

**Why:** `.unwrap()` and `.expect()` are hidden crash paths. In production, explicit error handling is safer.

**Implementation:** Tree-sitter query matches `call_expression` where method is `unwrap` or `expect`.

---

## The SlopChop Protocol

### Why Not Markdown Fences?

AI models frequently mess up markdown code fences:
- Nested fences get escaped wrong
- Closing fences match incorrectly
- Language tags vary unpredictably

The `#__SLOPCHOP_FILE__#` and `#__SLOPCHOP_END__#` delimiters:
- Never appear in normal code
- Unambiguous start/end
- Don't require escape sequences
- Machine-parseable

### Format Specification

```
#__SLOPCHOP_PLAN__#
GOAL: What you're doing
CHANGES:
1. First change
2. Second change
#__SLOPCHOP_END__#

#__SLOPCHOP_MANIFEST__#
src/file1.rs
src/file2.rs [NEW]
src/old.rs [DELETE]
#__SLOPCHOP_END__#

#__SLOPCHOP_FILE__# src/file1.rs
// Complete file content
// No truncation allowed
#__SLOPCHOP_END__#

===ROADMAP===
CHECK
id = task-id
ADD
id = new-task
text = New feature
section = v0.2.0
===ROADMAP===
```

### Block Types

| Block | Purpose | Required |
|-------|---------|----------|
| `PLAN` | Human-readable summary | Recommended |
| `MANIFEST` | Declares all files being touched | Optional but validated |
| File blocks | Actual file content | Required |
| `ROADMAP` | Task updates | Optional |

### Markers

| Marker | Meaning |
|--------|---------|
| `[NEW]` | File doesn't exist, will be created |
| `[DELETE]` | File will be removed |
| *(none)* | File exists, will be updated |

---

## Analysis Engine

### Language Support

| Language | Complexity | Skeleton | Imports | Banned Patterns |
|----------|:----------:|:--------:|:-------:|:---------------:|
| Rust | ✅ | ✅ | ✅ | `.unwrap()/.expect()` |
| TypeScript | ✅ | ✅ | ✅ | — |
| JavaScript | ✅ | ✅ | ✅ | — |
| Python | ✅ | ✅ | ✅ | — |

Languages are defined in `src/lang.rs` with their tree-sitter grammars and query patterns.

### Query Architecture

Each language defines:
- `q_naming()` — Function definition queries
- `q_complexity()` — Branch/loop queries  
- `q_skeleton()` — Function body queries for skeletonization
- `q_exports()` — Public API queries for signatures
- `q_banned()` — Optional banned pattern queries (Rust only)

---

## Apply System

### The Pipeline

```
Clipboard ──► Extract ──► Validate ──► Backup ──► Write ──► Verify ──► Commit
```

### Validation Rules

**Path Safety:**
- No `../` traversal
- No absolute paths
- No sensitive directories (`.git`, `.env`, `.ssh`, `.aws`)
- No hidden files (except `.gitignore`, `.slopchopignore`, `.github`)

**Protected Files:**
- `ROADMAP.md` — Use roadmap commands instead
- `slopchop.toml`, `Cargo.lock`, `package-lock.json`

**Content Safety:**
- No truncation markers (`// ...`, `/* ... */`, `# ...`)
- No lazy phrases ("rest of implementation", "remaining code")
- No empty files
- No markdown fences in non-markdown files

### Backup System

Before any write:
```
.slopchop_apply_backup/
└── {timestamp}/
    └── src/
        └── modified.rs
```

### Git Integration

On verification pass:
1. Stage all changes (`git add -A`)
2. Commit with PLAN's GOAL as message
3. Push to remote

Failed verifications leave changes uncommitted with error copied to clipboard.

---

## Context Generation

### Commands

| Command | Output |
|---------|--------|
| `slopchop signatures` | Type map of all exports (skeletonized) |
| `slopchop pack <file>` | Full file content |
| `slopchop pack --focus <file>` | Full file + dependency skeletons |
| `slopchop trace <file>` | Full file + dependency graph visualization |

### Smart Copy

For context > 2000 tokens:
1. Write to temp file
2. Copy file handle to clipboard (not text)
3. User can paste as file attachment

This prevents clipboard overflow and enables larger context windows.

### Skeleton System

Converts implementation to signatures:

**Before:**
```rust
pub fn validate(input: &str) -> Result<User> {
    let email = input.trim();
    if email.is_empty() {
        return Err(ValidationError::Empty);
    }
    // ... 40 more lines
}
```

**After:**
```rust
pub fn validate(input: &str) -> Result<User> { ... }
```

---

## Dependency Graph

### PageRank Ranking

The `graph/rank` module builds a dependency graph and ranks files by importance using PageRank:

1. **Extract definitions** — Functions, structs, types from each file
2. **Extract references** — Imports and usages
3. **Build edges** — File A imports from File B = edge A→B
4. **Compute PageRank** — Iterate until convergence

Files with high fan-in (many dependents) rank higher.

### Focus Mode

`slopchop pack --focus file.rs`:
1. Re-runs PageRank with anchor file as seed
2. Includes full content of anchor
3. Includes skeletonized content of neighbors
4. Respects token budget

---

## Roadmap System (V2)

### Storage Format

Tasks are stored in `tasks.toml`:

```toml
[meta]
title = "Project Roadmap"
description = ""

[[sections]]
id = "v0.1.0"
title = "v0.1.0"
status = "complete"
order = 0

[[tasks]]
id = "feature-x"
text = "Implement feature X"
status = "done"
section = "v0.1.0"
test = "tests/feature_x.rs::test_feature"
```

### Commands

AI can update the roadmap via the `===ROADMAP===` block:

```
===ROADMAP===
CHECK
id = feature-x

ADD
id = feature-y
text = New feature
section = v0.2.0
===ROADMAP===
```

| Command | Syntax |
|---------|--------|
| `CHECK` | Mark task done |
| `UNCHECK` | Mark task pending |
| `ADD` | Create new task |
| `UPDATE` | Modify task fields |
| `DELETE` | Remove task |

### Unified Apply

`slopchop apply` handles both code files and roadmap updates atomically.

---

## TUI Dashboard

### Tabs

| Tab | Purpose |
|-----|---------|
| **Dashboard** | Live scan status, recent activity |
| **Roadmap** | Task list with filtering |
| **Config** | Interactive settings editor |
| **Logs** | System log stream |

### Watcher (In Progress)

`src/tui/watcher.rs` polls clipboard for SlopChop payloads:
- Detects `#__SLOPCHOP_FILE__#` markers
- Sends `PayloadDetected` event to TUI
- Enables future "watch mode" hotkey application

---

## Security Model

### Threat Model

**Attacker:** Malicious or confused AI generating dangerous file operations.

### Defenses

| Threat | Defense |
|--------|---------|
| Path traversal | Block `..` in any path component |
| Absolute paths | Block `/` or `C:\` prefixes |
| Sensitive files | Blocklist: `.env`, `.ssh/`, `.aws/`, `.gnupg/`, `credentials` |
| Hidden files | Block `.*` except allowlist |
| Backup overwrite | Block `.slopchop_apply_backup/` |
| Truncation | Detect comment patterns and lazy phrases |
| Protected files | Block config/lock file overwrites |

---

## Key Decisions & Rationale

### Why Rust?

- **Performance:** Parallel file analysis via rayon
- **Reliability:** No null pointer crashes
- **Tree-sitter:** First-class Rust bindings
- **Single binary:** Easy distribution
- **Dogfooding:** SlopChop enforces its own rules on itself

### Why Tree-sitter Over LSP?

- **No server overhead:** Parse on-demand
- **Language-agnostic queries:** Same patterns for all languages
- **Simpler deployment:** No language server installation

### Why CLI Over IDE Plugin?

- **Editor-agnostic:** Works everywhere
- **Composable:** Pipes, scripts, CI
- **Maintainable:** One codebase

### Why Custom Protocol Over Markdown?

- **Unambiguous:** No fence-escape issues
- **Distinctive:** Delimiters never appear in code
- **Parseable:** Clean regex patterns

### Why Reject Instead of Fix?

- **Teaching:** AI learns through failure
- **Safety:** Auto-fix could mask deeper problems
- **Simplicity:** Rejection is stateless

---

## Module Map

### Core Dependencies

| Crate | Purpose |
|-------|---------|
| `tree-sitter` | AST parsing |
| `tree-sitter-{rust,python,typescript}` | Language grammars |
| `tiktoken-rs` | Token counting |
| `clap` | CLI parsing |
| `serde` + `toml` | Configuration |
| `walkdir` | File traversal |
| `rayon` | Parallelism |
| `regex` | Pattern matching |
| `colored` | Terminal colors |
| `ratatui` + `crossterm` | TUI |
| `anyhow` + `thiserror` | Error handling |

---

## Testing Philosophy

### What We Test

- **Happy paths:** Normal usage works
- **Rejection paths:** Invalid input caught with correct error
- **Security:** Every blocked path type has explicit test
- **Edge cases:** Empty files, Unicode, deep nesting

### What We Skip

- Platform-specific clipboard (manual verification)
- Git operations in CI (mocked or skipped)
- TUI rendering (visual inspection)

---

## Future Work

### Watch Mode

`slopchop watch` — Background clipboard monitoring with hotkey application.

The watcher infrastructure exists (`src/tui/watcher.rs`). Remaining work:
- Global hotkey registration
- System notification integration
- Diff preview modal

### Additional Languages

Adding a language requires:
1. Add `tree-sitter-{lang}` dependency
2. Add variant to `Lang` enum in `lang.rs`
3. Implement query methods
4. Add to language detection

### Distribution

Planned for v1.0:
- crates.io publication
- Homebrew formula
- GitHub Releases with binaries

---

*Last updated: 2025-06*
