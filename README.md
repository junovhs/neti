# SlopChop

SlopChop is a CLI that enforces a **configurable structural integrity boundary** for a repository—so code stays modular, reviewable, and predictable as the project and team scale.

It does this in two complementary ways:

1. **Structural governance (fast, deterministic):** scan the repo for violations of “shape” constraints (size, complexity, panic paths, etc.).
2. **Optional hardened change ingestion:** safely apply **large, automated, or otherwise untrusted multi-file edits** with manifest integrity, path safety, and transactional rollback.

SlopChop works well in normal developer workflows (local and CI). It also happens to be an excellent safety harness for AI-assisted coding, but it does not require an LLM.

---

## Why SlopChop exists

Most repos already run formatters, linters, and tests. Those tools are necessary, but they do not reliably enforce **repo-level structural standards**:

* Keeping files small enough to review and reason about
* Keeping functions simple enough to test in isolation
* Preventing hidden crash paths (`unwrap` / `expect`) where forbidden
* Enforcing consistent “shape” in a growing codebase (and resisting entropy)

SlopChop’s job is to make those constraints **explicit, configurable, and continuously enforced**.

---

## Surfaces

SlopChop currently provides:

1. `slopchop` — structural integrity scan (“Three Laws” defaults)
2. `slopchop check` — one gate command (runs your configured commands + SlopChop scan)
3. `slopchop apply` — hardened application of multi-file change payloads (optional)
4. `slopchop audit` — consolidation / refactor radar (optional)

> Note: `slopchop roadmap` is planned to be split into a separate crate/binary and treated as an optional integration via `slopchop check`.

---

## Quick start

### Install (local dev)

```bash
cargo install --path .
```

### Generate config

Run `slopchop` once in a repo. If `slopchop.toml` is missing, SlopChop will generate it.

### Run the structural scan

```bash
slopchop
```

### Run the full gate (commands + scan)

```bash
slopchop check
```

---

## 1) `slopchop`: structural integrity scan (fast, deterministic)

`slopchop` scans the repository for violations of a few structural constraints. Output is designed to be actionable and CI-friendly:

```text
❯ slopchop
error: High Complexity: Score is 9. Hard to test. (Max: 8)
  --> core/captions/src/lib.rs:92:1
   |
   = LAW OF COMPLEXITY: Action required

error: High Arity: Function takes 7 arguments. Use a Struct. (Max: 5)
  --> core/encoder/src/gif.rs:11:1
   |
   = LAW OF COMPLEXITY: Action required

error: Banned: '.expect()'. Use '?' or 'unwrap_or'.
  --> ui/src/main.rs:17:1
   |
   = LAW OF PARANOIA: Action required

❌ SlopChop found 12 violations in 98ms.
```

### Default rule set (“Three Laws”)

These defaults are opinionated but configurable:

* **Law of Atomicity:** keep files small (token-based threshold)
* **Law of Complexity:** cap cyclomatic complexity, nesting depth, and arity
* **Law of Paranoia (Rust-oriented):** discourage hidden crash paths (`unwrap`/`expect`) in configured scopes

The intent is not style policing. The intent is keeping the codebase “shape” compatible with review, testing, and maintenance.

### Practical outcomes

In practice, these rules tend to nudge a repo toward:

* Smaller modules and clearer boundaries
* Fewer “god functions” and deep nesting
* More explicit error handling
* Faster, higher-quality reviews (less “wall of code”)

---

## 2) `slopchop check`: one command to gate the branch

`slopchop check` is a **gate**, not a build system. It runs:

1. Your configured commands (formatters/linters/tests/etc.)
2. The SlopChop structural scan (`slopchop`)

It returns non-zero on failure and is designed for:

* CI pipelines (required PR checks)
* Local hooks (`pre-push`, `pre-commit`)
* Agent loops (repeatable gate when automated changes are being proposed)

### What `check` is not

SlopChop does not aim to replace:

* toolchain pinning / MSRV management
* feature matrix build orchestration
* semver/API checking
* dependency vulnerability/license tooling

Those remain best handled by dedicated tools. `check` is a thin, consistent entry point to run *your* commands plus SlopChop’s policy gate.

### Example `slopchop.toml` for `check`

```toml
[commands]
check = [
  "cargo clippy --all-targets -- -D warnings -W clippy::pedantic",
  "cargo test",
]
fix = ["cargo fmt"]
```

Then:

```bash
slopchop check
```

---

## 3) `slopchop apply`: hardened change ingestion (optional)

`slopchop apply` is designed for safely applying **large multi-file changes** that come from an unreliable or untrusted source. In practice, that often includes AI-generated patches—but the core design is broader:

* apply a proposed change bundle from clipboard, stdin, or a file
* refuse partial or inconsistent multi-file updates
* prevent path traversal and “write outside repo” bugs
* rollback automatically on failures
* optionally run verification commands before any git operations

### When is this useful (non-AI cases included)?

Use cases where developers tend to appreciate a hardened “apply” workflow:

* **Bulk refactors that touch many files** (module moves, large-scale renames, reorganizations)
* **Changes produced by automation** (scripts/codemods/codegen) where you want strict integrity gates
* **Accepting large change proposals through a text channel** (tickets, chats, docs) where “copy/paste and hope” is risky
* **Any workflow where partial application is unacceptable** (the repo must end either fully updated or untouched)

### Apply pipeline safety properties

If you use `apply`, it is designed to be safe on untrusted input:

* **Manifest integrity:** no partial multi-file updates

  * every declared file must be present
  * no undeclared file blocks are allowed
* **Transactional writes + rollback:** repo ends in pre-apply or post-apply state, never “half applied”
* **Path safety:** blocks traversal, absolute paths, and sensitive locations
* **Symlink escape protection:** prevents writes outside repo root via symlinks
* **Content safety checks:** blocks truncation markers and incomplete outputs
* **Verification hooks:** run your commands before committing/pushing

### Protocol format (high level)

SlopChop’s protocol is designed to be machine-parseable and robust (avoids markdown fence ambiguity):

* `PLAN` (recommended)
* `MANIFEST` (mandatory in hardened mode)
* one `FILE` block per updated file
* optional `ROADMAP` blocks (pending split-out)

The protocol is intentionally strict. On invalid input, SlopChop rejects the entire apply.

### Git operations (optional)

`apply` can optionally stage/commit/push *after verification passes*, but these behaviors should be treated as opt-in in public-facing defaults.

If enabled, SlopChop can produce commit messages derived from the PLAN, and (optionally) preserve attempt history in commit bodies. This is powerful for automated workflows, but should remain configurable and conservative.

---

## 4) `slopchop audit`: consolidation and refactor radar (optional)

`slopchop audit` searches for repo-wide consolidation opportunities:

* repeated idioms/patterns across many files
* near-duplicate functions/structs/enums
* test consolidation candidates
* rough impact estimates (lines removable), difficulty, confidence

Treat it as a prioritization tool: it does not force refactors. It helps you spot “we have 12 versions of the same idea” problems early.

---

## Configuration (`slopchop.toml`)

A representative configuration (matching current behavior and concepts):

```toml
[rules]
max_file_tokens = 2000
max_cyclomatic_complexity = 8
max_nesting_depth = 3
max_function_args = 5
max_function_words = 5

ignore_naming_on = ["tests", "spec"]
ignore_tokens_on = ["README.md", "lock"]

[preferences]
theme = "Cyberpunk"
auto_copy = true
auto_format = false
auto_commit = true
auto_push = true
commit_prefix = "AI: "
allow_dirty_git = false
backup_retention = 5
progress_bars = true
require_plan = false

[commands]
check = [
  "cargo clippy --all-targets -- -D warnings -W clippy::pedantic",
  "cargo test",
]
fix = ["cargo fmt"]
```

### Notes on rules and heuristics

* Some rules are “hard boundaries” by design (complexity/arity/unsafe paths as configured).
* Some checks may be best treated as “smells” (warn/info) rather than “errors,” depending on repo culture. For example, function-name length heuristics can be useful as a signal without being a build breaker.

---

## CI integration

In CI, SlopChop is typically used as a required PR gate:

* Run `slopchop check` as part of your pipeline
* Fail the job if it returns non-zero
* Require the job to pass before merge

The advantage is consistency: the same gate can run locally, in hooks, and in CI.

---

## Local workflow integration

### Recommended local habits

* Run `slopchop` during development when working on a module with active changes
* Run `slopchop check` before pushing or opening a PR
* Optionally wire `slopchop check` into a `pre-push` hook

### Watch mode (planned/optional)

A watch mode can make SlopChop feel like a continuous “integrity guardrail”:

* run the fast structural scan when files change
* surface new violations immediately
* avoid running heavy test pipelines on every save
* keep `check` for explicit gating moments (pre-push / CI)

---

## Why not just clippy?

Clippy is excellent, but it is not designed to enforce repo-level structural “shape” constraints:

* SlopChop applies consistent structural budgets across the whole repository.
* SlopChop can enforce constraints that are not lint-shaped (file token limits, cross-file structural duplication signals, hardened apply integrity).
* SlopChop is designed to act as a strict boundary for automated multi-file changes, not just a linter.

In practice, SlopChop and Clippy complement each other: Clippy catches many correctness and idiom issues; SlopChop enforces structural governance and ingestion safety.

---

## Design goals

* Deterministic, scriptable behavior
* Conservative failure modes (reject invalid input rather than “best-effort apply”)
* Composable in CI and local workflows
* Language-aware structural metrics via tree-sitter
* Optional automation support without coupling correctness to an LLM

---

## Command overview

### Structural governance

* `slopchop` — scan repo for structural violations

### Gate

* `slopchop check` — run configured commands + SlopChop scan, fail on violations

### Hardened apply (optional)

* `slopchop apply` — transactional apply of a strict patch payload
* `slopchop pack --focus <file>` — generate focused context (full focus + skeleton deps)

### Refactoring radar (optional)

* `slopchop audit` — duplication + consolidation opportunities

---

## Project status and roadmap

* Core structural scan: implemented
* `check` pipeline: implemented (thin gate pattern)
* Apply hardening: implemented (manifest integrity, rollback, path safety, verification hooks)
* Audit mode: implemented (consolidation radar)
* Roadmap system: planned split-out into separate crate/binary
* Watch mode: planned (leveraging existing watcher infrastructure)
