# SlopChop

**The architectural compiler for AI-assisted Rust development.**

Rust's compiler rejects code that's memory-unsafe. Clippy rejects code that's unidiomatic. Tests reject code that's functionally wrong. SlopChop rejects code that's *structurally* unsound—files too large for AI to handle atomically, complexity that invites bugs, coupling that makes changes risky.

If it doesn't pass `slopchop check`, it doesn't enter the codebase. The AI bounces off and tries again, just like it bounces off rustc until the types are right.

## The Problem

AI-generated code has specific failure modes:

- **Giant files.** AI will happily write 2,000 lines in one file. Good luck getting it to surgically edit line 847 later.
- **Creeping complexity.** Each fix adds "just one more" nested condition. Nobody notices until the function is unmaintainable.
- **Coupling.** Module A imports B imports C imports A. Refactoring becomes surgery.
- **The classics.** N+1 queries, clones in loops, mutex guards held across await points—patterns AI reproduces confidently.

These aren't type errors. They're structural problems that compound over time.

## The Solution

SlopChop runs a governance check before code enters your codebase:

```
$ slopchop check

  ✓ cargo clippy (14.2s)
  ✓ cargo test (8.7s)

  ┌─────────────────────────────────────────────┐
  │ SLOPCHOP GOVERNANCE                         │
  ├─────────────────────────────────────────────┤
  │ Files scanned     18                        │
  │ Total tokens      12,847                    │
  │ Violations        2                         │
  └─────────────────────────────────────────────┘

  src/engine.rs:142
  │ P01: Clone inside hot loop
  │ Consider moving the clone outside the loop or using references.

  src/handlers.rs
  │ File exceeds 2,000 tokens (2,341)
  │ Split into smaller, focused modules.

  ✗ Governance check failed
```

## Current Features (v1.7)

**Metrics — quantitative thresholds**

| Metric | Default | What it catches |
|--------|---------|-----------------|
| File tokens | 2,000 | God files that AI can't edit atomically |
| Cognitive complexity | 15 | Functions too complex to reason about |
| Nesting depth | 3 | Arrow code that hides bugs |
| Function arguments | 5 | Weak abstraction boundaries |
| LCOM4 | 1 | Classes doing too many unrelated things |
| CBO | 9 | Modules coupled to too many others |

**Patterns — AST-based detection (23 active)**

SlopChop parses your code and detects specific anti-patterns:

| Category | Examples |
|----------|----------|
| State | Global mutables, exported mutable statics |
| Concurrency | Mutex guard held across `.await`, undocumented sync primitives |
| Security | SQL injection, command injection, hardcoded secrets, dangerous TLS config |
| Performance | Clone in loop, allocation in loop, N+1 queries, nested loops, linear search in loop |
| Semantic | Getter that mutates, `is_*` returning non-bool |
| Idiomatic | Global mutation via `std::env::set_var` |
| Logic | Boundary ambiguity (`<= .len()`), unchecked indexing |

**Governance Profiles**

Different code has different physics. A CLI app and a lock-free queue have fundamentally different constraints.

```toml
# slopchop.toml
profile = "application"  # or "systems"
```

| | application | systems |
|---|-------------|---------|
| Philosophy | Maintainability first | Throughput first |
| File tokens | 2,000 | 10,000 |
| Complexity | 15 | 50 |
| Structural metrics | Enabled | Disabled |
| Safety checks | Standard | Escalated |

The `systems` profile relaxes structural limits while *tightening* safety requirements—because systems code trades abstraction for performance but must be paranoid about memory.

**Flight Recorder**

Every `slopchop check` writes full results to `slopchop-report.txt`. Untruncated, machine-parseable, no terminal formatting. Useful for CI pipelines, agent loops, or just grepping later.

**Transactional Workflow**

```
$ slopchop apply    # Creates a working branch, applies changes
$ slopchop check    # Validates the changes
$ slopchop promote  # Merges with goal-aware commit message
```

The `apply` command reads a PLAN block from your AI's response and extracts the stated goal. When you `promote`, that goal becomes the merge commit message. Cleaner git history without writing commit messages yourself.

## Installation

```
cargo install slopchop
```

Or build from source:

```
git clone https://github.com/junovhs/slopchop
cd slopchop
cargo build --release
```

## Configuration

Run `slopchop config` for an interactive TUI, or edit `slopchop.toml` directly:

```toml
[rules]
max_file_tokens = 2000
max_cognitive_complexity = 15
max_nesting_depth = 3
max_function_args = 5
max_lcom4 = 1
max_cbo = 9

[rules.safety]
require_safety_comment = true

[commands]
check = [
    "cargo clippy --all-targets -- -D warnings",
    "cargo test",
]

[preferences]
auto_copy = true
```

## Aspirational / Roadmap

These features are planned but not yet implemented:

**Small-codebase detection.** Structural metrics (LCOM4, CBO, AHF) are meaningless for a 6-file project. SlopChop should auto-skip them when `total_files < 10` or `total_tokens < 5000`.

**TypeScript support.** The tree-sitter infrastructure exists, but pattern coverage and tuning are Rust-first for now. TypeScript will follow once Rust governance is rock-solid.

**Per-directory profiles.** `src/core/` as `systems`, `src/api/` as `application`, in the same repo.

## What SlopChop Is Not

**Not a linter.** Clippy handles Rust-specific lints better than SlopChop ever could. SlopChop defers to it (and runs it as part of `check`).

**Not a formatter.** Use `rustfmt`.

**Not a test framework.** SlopChop runs your tests but doesn't replace them.

**Not a context/packing tool.** SlopChop verifies AI *output*. Feeding context *to* AI is a different problem.

## License

MIT OR Apache-2.0
