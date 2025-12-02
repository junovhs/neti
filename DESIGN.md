# Warden Design Document

> **Audience:** Developers (human or AI) working on or extending Warden.  
> **See also:** [README.md](README.md) for user guide, [ROADMAP.md](ROADMAP.md) for feature tracking.

---

## Table of Contents

1. [Vision & Philosophy](#vision--philosophy)
2. [Architecture Overview](#architecture-overview)
3. [The Three Laws](#the-three-laws)
4. [The Nabla Protocol](#the-nabla-protocol)
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
│   ├── extractor.rs   # Nabla format parsing
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
│   └── mod.rs         # File discovery, skeleton, Nabla output
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
                                                 AI response (Nabla)
                                                         │
                                                         ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│    extractor    │────►│    validator    │────►│     writer      │
│ (parse Nabla)   │     │ (safety checks) │     │ (atomic write)  │
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

## The Nabla Protocol

### Why Not Markdown Fences?

AI models frequently mess up markdown code fences:
- Nested fences get escaped wrong: ` ```rust ` inside ` ``` ` 
- Some models emit fences with wrong language tags
- Closing fences get matched incorrectly with earlier opens

The `∇∇∇` and `∆∆∆` symbols:
- Never appear in normal code
- Unambiguous start/end delimiters
- Visually distinctive
- Don't require escape sequences

### Format Specification

```
∇∇∇ PLAN ∇∇∇
GOAL: What you're doing
CHANGES:
1. First change
2. Second change
∆∆∆

∇∇∇ MANIFEST ∇∇∇
src/file1.rs
src/file2.rs [NEW]
src/old.rs [DELETE]
∆∆∆

∇∇∇ src/file1.rs ∇∇∇
// Complete file content
// No truncation allowed
∆∆∆

∇∇∇ src/file2.rs ∇∇∇
// Another complete file
∆∆∆
```

### Block Types

| Block | Purpose | Required |
|-------|---------|----------|
| `PLAN` | Human-readable summary for review | Recommended |
| `MANIFEST` | Declares all files being touched | Optional but validated |
| File paths | Actual file content | Required |

### Markers

| Marker | Meaning |
|--------|---------|
| `[NEW]` | File doesn't exist, will be created |
| `[DELETE]` | File will be removed |
| *(none)* | File exists, will be updated |

### The Contract

1. Every file in MANIFEST must have a corresponding Nabla block (unless DELETE)
2. File content must be **complete** — no `// ...` or "remaining code"
3. Paths must be relative, no traversal (`../`), no absolute paths
4. No touching sensitive files (`.env`, `.git/`, etc.)

---

## Adoption Tiers

Warden can be adopted incrementally:

### Tier 1: Structural Linting Only

Use Warden as a code quality scanner without any AI integration.
```bash
warden              # Scan for violations
warden check        # Run tests/linters
```

**What you get:** The Three Laws enforcement, complexity metrics, consistent standards across the team.

**Who it's for:** Teams that want code quality tooling but aren't using AI assistants yet.

### Tier 2: AI-Assisted Development

Add the pack/apply loop for AI coding sessions.
```bash
warden pack         # Generate context for AI
warden apply        # Validate and apply AI responses
```

**What you get:** Automatic rejection of oversized/complex/truncated AI output, backup system, auto-commit on success, structured rejection messages.

**Who it's for:** Individual developers or teams actively using AI coding assistants.

### Tier 3: Full Traceability

Add roadmap management and audit enforcement.
```bash
warden roadmap audit --strict   # Verify test coverage
```

**What you get:** Every feature tied to a test, programmatic progress tracking, unified apply (code + roadmap in one paste), CI enforcement of the test contract.

**Who it's for:** Teams that want rigorous test traceability and project management integration.

Teams can start at Tier 1 and graduate upward as trust in the workflow grows.

---

## Analysis Engine

### Tree-sitter Integration

Warden uses [tree-sitter](https://tree-sitter.github.io/) for structural code analysis. This provides:
- Language-agnostic AST access
- Incremental parsing (though we don't use it yet)
- Battle-tested grammars

### Supported Languages

| Language | Complexity | Skeleton | Notes |
|----------|:----------:|:--------:|-------|
| Rust | ✅ | ✅ | + `.unwrap()`/`.expect()` detection |
| TypeScript | ✅ | ✅ | Shared with JavaScript |
| JavaScript | ✅ | ✅ | ESM and CJS |
| Python | ✅ | ✅ | |
| Go | — | — | Project detection only |
| Others | — | — | Token counting only |

### Query Architecture

```rust
// src/analysis/ast.rs

// Each language has three queries:
struct LanguageQueries {
    language: Language,      // tree-sitter grammar
    naming: Query,           // Finds function/method names
    complexity: Query,       // Counts branching constructs
    banned: Option<Query>,   // Language-specific bans (Rust only)
}
```

Example complexity query (Rust):
```
(if_expression) @branch
(match_expression) @branch  
(for_expression) @branch
(while_expression) @branch
(binary_expression operator: "&&") @branch
(binary_expression operator: "||") @branch
```

### Analysis Flow

```rust
// src/analysis/mod.rs

pub struct RuleEngine { config: Config }

impl RuleEngine {
    pub fn scan(&self, files: Vec<PathBuf>) -> ScanReport {
        files.par_iter()                          // Parallel via rayon
            .filter_map(|path| self.analyze_file(path))
            .collect()
    }
    
    fn analyze_file(&self, path: &Path) -> Option<FileReport> {
        // 1. Check for warden:ignore
        // 2. Token count (Law of Atomicity)
        // 3. AST analysis (Law of Complexity, Paranoia)
        // 4. Return FileReport with violations
    }
}
```

---

## Apply System

### The Pipeline

```
Clipboard ──► Extract ──► Validate ──► Backup ──► Write ──► Verify ──► Commit
                │            │           │          │          │          │
                │            │           │          │          │          ▼
                │            │           │          │          │     git commit/push
                │            │           │          │          ▼
                │            │           │          │     Run check commands
                │            │           │          ▼
                │            │           │     Write files atomically
                │            │           ▼
                │            │     Backup existing files to .warden_apply_backup/
                │            ▼
                │     Path safety, truncation detection, manifest validation
                ▼
          Parse Nabla blocks, extract PLAN, MANIFEST, files
```

### Validation Rules

**Path Safety:**
- No `../` traversal
- No absolute paths (`/etc/passwd`, `C:\Windows`)
- No sensitive files (`.env`, `.ssh/`, `.aws/`, `.git/`)
- No hidden files (except `.gitignore`, `.wardenignore`)
- No overwriting `ROADMAP.md` (protected)

**Content Safety:**
- No truncation markers (`// ...`, `/* ... */`, `# ...`)
- No lazy phrases ("rest of implementation", "remaining code")
- No empty files
- Files must match MANIFEST declaration

### Backup System

Before any write:
```
.warden_apply_backup/
└── 1699876543/           # Unix timestamp
    └── src/
        └── modified.rs   # Original content preserved
```

**Recovery:** If apply fails mid-write, original files are in backup.

### Verification

After successful writes, Warden runs configured check commands:

```toml
[commands]
check = [
    "cargo clippy --all-targets -- -D warnings",
    "cargo test"
]
```

- **All pass:** Auto-commit and push
- **Any fail:** Generate rejection message, copy to clipboard

### Escape Hatch (Planned)

For situations where strict rejection causes friction (AI failing repeatedly, deadline pressure), a future `warden apply --force` flag will:

1. Apply the code despite violations
2. Mark affected files with `// warden:quarantine`
3. Add them to a temporary ignore list
4. Report quarantined files in subsequent `warden` scans

This allows forward progress while maintaining visibility into technical debt. See ROADMAP.md v0.7.0 "Escape Hatches."

### Git Integration

On verification pass:
```rust
fn commit_and_push(message: &str) -> Result<()> {
    git add -A
    git commit -m "{prefix}{message}"
    git push
}
```

The commit message comes from the PLAN block's GOAL line.

---

## Pack & Context System

### The Problem

AI context windows are finite. You can't send your entire codebase for every request.

**Current solution:** Focus mode
```bash
warden pack src/apply/mod.rs
```
- Target file: full content
- All other files: skeletonized (signatures only)

### Skeleton System

Converts implementation to signatures:

**Before:**
```rust
pub fn validate_user(input: &UserInput) -> Result<User, ValidationError> {
    let email = input.email.trim();
    if email.is_empty() {
        return Err(ValidationError::EmptyEmail);
    }
    // ... 40 more lines
}
```

**After:**
```rust
pub fn validate_user(input: &UserInput) -> Result<User, ValidationError> { ... }
```

**Implementation:** Tree-sitter finds function bodies and replaces with `{ ... }` (Rust), `...` (Python), or `{ ... }` (JS/TS).

### Prompt Generation

Every `warden pack` output includes:
1. **Header:** System prompt with The Three Laws, current limits, Nabla instructions
2. **Violations:** Any existing rule violations (priority fix required)
3. **Files:** Codebase content in Nabla format
4. **Footer:** Constraint reminder

The AI receives not just code, but the rules it must follow.

---

## Smart Context (v0.8-0.9)

> **Status:** Designed, not yet implemented. See ROADMAP.md v0.8.0-v0.9.0.

### The Vision

Current flow:
```
You ──► [whole codebase] ──► AI ──► [fixes]
```

Future flow:
```
You ──► [minimal map] ──► AI ──► [context request] ──► You ──► [focused context] ──► AI ──► [fixes]
```

**The AI becomes a navigator, not just a consumer.**

### Warden Map

A lightweight structural overview (~50-100 lines):

```
WARDEN CODEBASE MAP
==================

src/
  analysis/     [4 files, 1.2k tokens]  → Code quality checks
  apply/        [8 files, 2.8k tokens]  → AI response parsing
  roadmap/      [9 files, 3.1k tokens]  → Task tracking
  graph/        [3 files, 0.6k tokens]  → Dependency extraction

CLUSTERS (auto-detected):
  [apply-system]    src/apply/* → uses: types.rs, config.rs, roadmap/mod.rs
  [roadmap-system]  src/roadmap/* → uses: types.rs
```

### Error-Driven Packing

For compiler errors, **skip AI reasoning entirely**:

```bash
cargo clippy 2>&1 | warden pack --from-errors
```

The compiler already tells you which files matter:
```
error[E0382]: use of moved value
  --> src/apply/writer.rs:45:9
```

Warden parses this, packs `src/apply/writer.rs` (and maybe its dependencies).

### Cluster Packing

```bash
warden pack --cluster apply-system
```

Packs all files in the cluster, in dependency order, with skeleton for boundary files.

### Trace Packing

```bash
warden pack --trace src/apply/mod.rs --depth 2
```

Walks the import graph 2 hops in both directions:
- What does `mod.rs` import? (dependencies)
- What imports `mod.rs`? (dependents)

Packs that subgraph, topologically sorted.

### Context Ordering

**Why it matters:** AI comprehension improves when dependencies come before dependents.

```
# BAD: Random order
src/apply/mod.rs        # Uses types.rs - but AI hasn't seen it yet
src/types.rs            # Too late!

# GOOD: Topological order  
src/types.rs            # Leaf node, no deps
src/apply/types.rs      # Uses types.rs (already seen)
src/apply/mod.rs        # Uses both (already seen)
```

### AI Context Protocol

Future: AI can request specific context:

```
CONTEXT_REQUEST:
  cluster: apply-system
  with_tests: true
```

Or in natural language:
> "I need the apply system files. Run `warden pack --cluster apply-system --with-tests`"

---

## Roadmap System

### Purpose

The roadmap isn't just documentation—it's a **contract**:
- Every `[x]` feature has a `<!-- test: path::function -->` anchor
- `warden roadmap audit` verifies anchors resolve to real tests
- This enforces that "done" means "tested"

### Programmatic Updates

AI can update the roadmap via commands:

```
===ROADMAP===
CHECK "task-slug"
ADD "**New task**" AFTER "existing-task"
UPDATE "task-slug" "**New text**"
NOTE "section" "Additional info"
===ROADMAP===
```

### Unified Apply

When you run `warden apply`, it handles BOTH:
1. Code files (Nabla blocks)
2. Roadmap updates (`===ROADMAP===` block)

**One paste updates everything atomically.**

### Why Not Split Roadmap Into Its Own Tool?

We considered this. Arguments for splitting:
- Roadmap is useful beyond Warden users
- Cleaner separation of concerns

Arguments against (winning):
- **Unified apply is the killer feature** — one paste updates code AND progress
- Splitting requires two pastes, breaking flow
- The value is in the integration

**Decision:** Keep integrated. Extract `roadmap_core` as internal library if needed, but don't ship separately until users explicitly request it.

### Hardening (Post-Mortem)

During development, a batch of 110 roadmap commands failed catastrophically:
- 90+ commands failed due to cascading AFTER dependencies
- Slug mismatches caused silent failures
- Partial application left orphaned tasks

**Lessons learned:**
1. Validate ALL targets before executing ANY commands
2. Echo generated slugs so users know what to reference
3. All-or-nothing execution (atomic batch)
4. CHAIN command eliminates AFTER guessing for sequences

See ROADMAP.md v0.7.0 "Roadmap Hardening" for implementation tasks.

---

## Security Model

### Threat Model

**Attacker:** Malicious or confused AI generating dangerous file operations.

**Attack surface:**
- Path traversal (`../../../etc/passwd`)
- Sensitive file overwrite (`.env`, SSH keys)
- Code injection via truncation markers

### Defenses

| Threat | Defense |
|--------|---------|
| Path traversal | Block any path containing `..` |
| Absolute paths | Block paths starting with `/` or `C:\` |
| Sensitive files | Blocklist: `.env`, `.ssh/`, `.aws/`, `.gnupg/`, `id_rsa`, `credentials` |
| Hidden files | Block `.*` except `.gitignore`, `.wardenignore` |
| Backup overwrite | Block `.warden_apply_backup/` |
| Truncation | Detect `// ...`, `/* ... */`, `# ...`, lazy phrases |
| Empty files | Reject zero-content files |
| Protected files | Block `ROADMAP.md` overwrites (use roadmap commands instead) |

### Non-Goals

- Sandboxing execution (trust the user's environment)
- Network isolation (AI responses are text, not executable)
- Encryption (files are plaintext on disk anyway)

---

## Key Decisions & Rationale

### Why Rust?

- **Performance:** Parallel file analysis via rayon
- **Reliability:** No runtime crashes from null/undefined
- **Tree-sitter bindings:** First-class Rust support
- **Single binary:** Easy distribution, no dependencies
- **Dogfooding:** Warden enforces Rust best practices on itself

### Why Tree-sitter Over LSP?

- **No server overhead:** Parse on-demand, no background process
- **Language-agnostic queries:** Same query syntax for all languages
- **Incremental not needed:** We parse once per command, not on every keystroke
- **Simpler deployment:** No language server installation required

### Why CLI Over VS Code Extension?

- **Editor-agnostic:** Works with Vim, Emacs, VS Code, anything
- **Composable:** Pipes, scripts, CI integration
- **Maintainable:** One codebase, not per-editor plugins
- **AI-friendly:** Command-line is the universal interface

### Why Nabla Over Markdown?

- **Unambiguous:** No fence-escape issues
- **Distinctive:** `∇∇∇` never appears in code
- **Simple:** No language tags, just path and content
- **Parseable:** Regex-friendly delimiters

### Why Reject Instead of Fix?

- **Teaching:** AI learns constraints through failure
- **Safety:** Auto-fix could mask deeper problems
- **Simplicity:** Rejection logic is stateless
- **Trust:** User sees exactly what AI generated

### Why Git Integration?

- **Atomicity:** Commit represents "AI task complete"
- **Undo:** `git revert` is the recovery mechanism
- **History:** Track AI contributions over time
- **Workflow:** Push triggers CI, PR, deployment

---

## Module Map

### Core Libraries Used

| Crate | Purpose |
|-------|---------|
| `tree-sitter` | AST parsing |
| `tree-sitter-rust/python/typescript` | Language grammars |
| `tiktoken-rs` | Token counting (OpenAI tokenizer) |
| `clap` | CLI argument parsing |
| `serde` + `toml` | Configuration |
| `walkdir` | File system traversal |
| `rayon` | Parallel iteration |
| `regex` | Pattern matching |
| `colored` | Terminal output |
| `ratatui` + `crossterm` | TUI dashboard |
| `anyhow` + `thiserror` | Error handling |

### Internal Module Dependencies

```
lib.rs (warden_core)
    ├── analysis ──► config, types, tokens
    ├── apply ────► config, types, clipboard, roadmap
    ├── pack ─────► config, discovery, analysis, skeleton, prompt, clipboard
    ├── roadmap ──► clipboard
    ├── discovery ► config
    └── tui ──────► analysis, types, config
```

---

## Testing Philosophy

### The Contract

From ROADMAP.md Philosophy:
> Every `[x]` feature MUST have a `<!-- test: path::function -->` reference

This is enforced by `warden roadmap audit --strict`.

### Test Organization

```
tests/
├── unit_*.rs           # Pure function tests, no I/O
├── integration_*.rs    # Multi-module tests, temp directories
├── cli_*.rs            # Full command invocation tests
└── security_*.rs       # Attack vector validation
```

### Naming Convention

Test functions should match feature slugs from ROADMAP.md:
```rust
// ROADMAP: - [x] **Block ../ traversal** <!-- test: tests/security_validation.rs::test_traversal_blocked -->

#[test]
fn test_traversal_blocked() {
    // ...
}
```

### What We Test

- **Happy paths:** Normal usage works
- **Rejection paths:** Invalid input is caught with correct error
- **Security:** Every blocked path type has explicit test
- **Edge cases:** Empty files, Unicode paths, deep nesting

### What We Don't Test

- Platform-specific clipboard (manual verification)
- Git operations in CI (mocked or skipped)
- TUI rendering (visual inspection)

---

## Future Considerations

### Language Additions

Adding a new language requires:
1. Add `tree-sitter-{lang}` dependency
2. Write complexity query (branching constructs)
3. Write naming query (function definitions)
4. Write skeleton cleaner (body replacement)
5. Add to language detection in `analysis/ast.rs`

Estimated effort: 2-4 hours per language.

### Performance

Current: ~1-2 seconds for medium codebase (1000 files).

If needed:
- Incremental analysis (cache unchanged files)
- Parallel tree-sitter parsing (currently sequential per file)
- Memory-mapped file reading

Not prioritized because current speed is acceptable.

### Distribution

Planned for v1.0.0:
- crates.io publication
- Homebrew formula (macOS)
- Scoop/Winget (Windows)
- AUR package (Arch Linux)
- GitHub Releases with prebuilt binaries

### What We're NOT Building

| Feature | Reason |
|---------|--------|
| VS Code Extension | IDE lock-in, maintenance burden |
| Watch mode | Complexity without clear benefit |
| Markdown fallback | Enforce format discipline |
| Auto-fix | Warden rejects, doesn't repair |
| LSP server | Overkill for our use case |
| Multi-repo | One project at a time |
| Cloud service | Local-first philosophy |

---

## Contributing

See ROADMAP.md for current priorities. The `🔄 CURRENT` version marker indicates active development.

Before submitting:
1. Run `warden` (must pass own rules)
2. Run `cargo clippy --all-targets -- -D warnings -D clippy::pedantic`
3. Run `cargo test`
4. Ensure new features have `<!-- test: -->` anchors in ROADMAP.md

---

*Last updated: 2025*
