# Warden Design Document

> **Audience:** Developers (human or AI) working on or extending Warden.  
> **See also:** [README.md](README.md) for user guide, [ROADMAP.md](ROADMAP.md) for feature tracking.

---

## Table of Contents

1. [Vision & Philosophy](#vision--philosophy)
2. [Architecture Overview](#architecture-overview)
3. [The Three Laws](#the-three-laws)
4. [The Warden Protocol](#the-warden-protocol)
5. [Analysis Engine](#analysis-engine)
6. [Apply System](#apply-system)
7. [Pack & Context System](#pack--context-system)
8. [Smart Context (v0.8-0.9)](#smart-context-v08-09)
9. [Roadmap System](#roadmap-system)
10. [Security Model](#security-model)
11. [Key Decisions & Rationale](#key-decisions--rationale)
12. [Module Map](#module-map)
13. [Testing Philosophy](#testing-philosophy)
14. [Future Considerations](#future-considerations)

---

## Vision & Philosophy

### The Problem

AI coding assistants are powerful but unreliable. They:
- Generate files too large to review meaningfully
- Produce complex functions that can't be tested in isolation
- Truncate code with `// ...` or "rest of implementation"
- Escape markdown fences incorrectly, corrupting output
- Have no memory of project constraints between sessions

Developers end up manually reviewing every line, defeating the productivity gains.

### The Solution

**Warden is a gatekeeper, not a fixer.** It creates a feedback loop:

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│   warden pack ──► AI ──► warden apply ──► verify ──► commit    │
│        ▲                      │                                 │
│        │                      ▼                                 │
│        └────── rejection ◄── FAIL                               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

When AI output violates constraints:
1. Warden rejects the entire response
2. Generates a structured error message
3. Copies it to clipboard for pasting back to AI
4. AI corrects and resubmits

**The AI learns the constraints through rejection, not instruction.**

### Core Principles

| # | Principle | Meaning |
|---|-----------|---------|
| 1 | **Every feature has a verified test** | No exceptions. The roadmap enforces this. |
| 2 | **Reject bad input, don't fix it** | Warden is a gatekeeper, not a linter with autofix. |
| 3 | **Git is the undo system** | Don't reinvent version control. Commit on success. |
| 4 | **Explicit > Magic** | Fail loudly on format violations. |
| 5 | **Containment over craftsmanship** | Constraints are safety, not style. |
| 6 | **Self-hosting** | Warden passes its own rules. |
| 7 | **Context is king** | Give AI exactly what it needs, nothing more. |
| 8 | **Graph over glob** | Understand structure, don't just pattern match. |
| 9 | **Errors are context** | Parse failures to understand scope. |

### What Warden Is NOT

- **Not a linter** — It doesn't suggest fixes, it rejects
- **Not an IDE plugin** — It's CLI-first, composable with any editor
- **Not AI-specific** — The constraints help human reviewers too
- **Not prescriptive about style** — It cares about size and complexity, not formatting

---

## Architecture Overview

```
src/
├── analysis/          # The Three Laws enforcement (tree-sitter)
│   ├── ast.rs         # Language-specific query compilation
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
│   ├── git.rs         # Commit and push automation
│   └── mod.rs         # Orchestration and flow control
│
├── graph/             # Dependency analysis (partially implemented)
│   ├── imports.rs     # Import extraction per language
│   ├── resolver.rs    # Import → file path resolution
│   └── mod.rs
│
├── pack/              # Context generation for AI
│   └── mod.rs         # File discovery, skeleton, Protocol output
│
├── roadmap/           # Programmatic roadmap management
│   ├── parser.rs      # Markdown → structured data
│   ├── commands.rs    # CHECK, ADD, UPDATE, etc.
│   ├── audit/         # Test traceability verification
│   └── cli.rs         # Subcommand handlers
│
├── skeleton/          # Code compression (full → signatures)
│   └── mod.rs         # Tree-sitter body removal
│
├── tui/               # Interactive dashboard
│   ├── state.rs       # App state management
│   └── view/          # Ratatui rendering
│
├── config.rs          # warden.toml loading
├── discovery.rs       # File enumeration (git + walk)
├── tokens.rs          # tiktoken integration
├── types.rs           # Shared types (Violation, FileReport, etc.)
├── prompt.rs          # System prompt generation
├── clipboard.rs       # Cross-platform clipboard
└── lib.rs             # Public API (warden_core)
```

### Data Flow

```
User runs "warden pack"
         │
         ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│    discovery    │────►│    analysis     │────►│      pack       │
│   (find files)  │     │  (check rules)  │     │ (generate ctx)  │
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
│ (parse Blocks)  │     │ (safety checks) │     │ (atomic write)  │
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

Warden enforces structural constraints inspired by code review best practices. These are configurable but opinionated defaults.

### Law of Atomicity

**Files must be small enough to reason about.**

```toml
[rules]
max_file_tokens = 2000  # Default: ~500 lines of code
```

**Why:** A 5000-token file can't be meaningfully reviewed. AI-generated code especially tends toward monolithic files. Forcing small files creates natural modularity.

**Escape hatch:** `ignore_tokens_on = [".lock", ".md"]`

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

**Why:** `.unwrap()` and `.expect()` are fine for prototyping but represent silent panic paths. In production, explicit error handling is safer.

**Implementation:** Tree-sitter query matches `call_expression` where method is `unwrap` or `expect`.

### Law of Clarity (Naming)

**Function names should reveal intent.**
```toml
[rules]
max_function_words = 5   # Words in function name
```

**Why:** A function named `validate_user_input_and_send_email_notification_async` is doing too much. Short names force single responsibility.

**Implementation:** Tree-sitter extracts function names, then counts words by splitting on `_` (snake_case) or uppercase boundaries (CamelCase).

**Note:** In violation reports, this appears as `LAW OF BLUNTNESS` — a reminder that good names are blunt about what a function does.

---

## The Warden Protocol

### Why Not Markdown Fences?

AI models frequently mess up markdown code fences:
- Nested fences get escaped wrong: ` ```rust ` inside ` ``` ` 
- Some models emit fences with wrong language tags
- Closing fences get matched incorrectly with earlier opens

The custom delimiters:
- Never appear in normal code
- Unambiguous start/end delimiters
- Visually distinctive
- Don't require escape sequences

### Format Specification

```
#__WARDEN_PLAN__#
GOAL: What you're doing
CHANGES:
1. First change
2. Second change