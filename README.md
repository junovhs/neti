# SlopChop

SlopChop is a **High-Integrity Systems Architect** tool designed to enforce a strict structural boundary for repositories. It acts as a deterministic gatekeeper for code quality and a surgical sandbox for change ingestion.

SlopChop treats your repository as a **transactional filesystem**. It ensures that code stays modular, reviewable, and predictable, even when subjected to large-scale automated refactors or untrusted AI-generated patches.

---

## The Three Laws of SlopChop

SlopChop enforces three core structural constraints by default. These are configurable but opinionated:

### 1. The Law of Atomicity (File Size)
*   **Constraint:** Files MUST stay below a token-based threshold (default: 2000).
*   **Rationale:** Large files ("God Objects") are the primary source of technical debt. They confuse AI context windows, make code reviews impossible, and hide side effects. SlopChop nudges you toward modularity.

### 2. The Law of Complexity (Function Shape)
*   **Constraint:** Cyclomatic Complexity <= 8, Nesting Depth <= 3, Argument Count <= 5.
*   **Rationale:** Logic should be linear and shallow. Deep nesting and high branching factors increase cognitive load and hallucination rates in automated tools. SlopChop scans ASTs via `tree-sitter` to enforce these limits.

### 3. The Law of Paranoia (Safety)
*   **Constraint:** Discourage or block unhandled crash paths (`.unwrap()`, `.expect()`) and require explicit justification for `unsafe` code blocks.
*   **Rationale:** In a high-integrity environment, "it shouldn't happen" is not a valid strategy. SlopChop ensures error handling is explicit and panic-free.

---

## Core Architecture: The Staged Workspace

SlopChop implements an **Implicit Staged Workspace**. This is its most significant safety feature. 

Unlike traditional tools that write directly to your source files, `slopchop apply` never touches your real files initially. Instead, it creates a **Shadow Worktree** in `.slopchop/stage/worktree/`.

### The Loop:
1.  **Stage:** You `apply` a change payload. SlopChop writes it to the sandbox and records a `base_hash` of the workspace.
2.  **Verify:** You run `check`. SlopChop executes your test suite and its own scan **inside the sandbox**.
3.  **Promote:** Run `slopchop apply --promote`. SlopChop verifies that the workspace files haven't changed since Step 1 (**Split-Brain Protection**) before atomically moving files.

This ensures your repository never ends in a "half-broken" state. If a change fails verification, you simply `reset` the stage or patch the sandbox.

---

## The `XSC7XSC` Protocol

SlopChop uses the **Sequence Sigil** (`XSC7XSC`) to separate protocol instructions from code content.

### Why not Markdown?
Standard markdown code blocks (```) are fragile. AI UIs often collapse them, and renderers can corrupt content containing internal backticks. 

The `XSC7XSC` sigil is:
*   **Markdown-Inert:** It will not trigger formatting changes in any renderer.
*   **Shell-Safe:** It contains no characters that trigger shell expansion or pipes.
*   **Unique:** Statistically impossible to appear in normal code.

### Protocol Format:
```text
 XSC7XSC PLAN XSC7XSC
 GOAL: Refactor the auth module.
 CHANGES:
 1. Move login logic to login.rs.
 XSC7XSC END [SIGIL]

 XSC7XSC MANIFEST XSC7XSC
 src/auth.rs
 src/login.rs [NEW]
 XSC7XSC END [SIGIL]

 XSC7XSC FILE XSC7XSC src/auth.rs SHA256:7dab58e6...
 // file content...
 XSC7XSC END [SIGIL]
 
 XSC7XSC PATCH XSC7XSC src/auth.rs
 BASE_SHA256: 7dab58e6...
 MAX_MATCHES: 1
 LEFT_CTX: ...
 OLD: ...
 RIGHT_CTX: ...
 NEW: ...
 XSC7XSC END [SIGIL]
```

---

## Surfaces & Commands

### `slopchop scan` (Structural Scan)
A fast, deterministic scan for Law violations.
*   **Deterministic:** Always produces the same results for the same input.
*   **Actionable:** Outputs compiler-grade error messages with file/line/column pointers.

### `slopchop check` (The Gate)
The ultimate gatekeeper command. It runs:
1.  Your configured commands (formatters, linters, tests).
2.  The SlopChop structural scan.
*   **Context Aware:** If a stage exists, it runs checks inside the stage. If not, it runs them in your workspace.

### `slopchop apply` (Hardened Ingestion)
Applies a protocol payload from clipboard, stdin, or file.
*   **Atomic:** All files apply together or none do.
*   **Surgical:** Validates path safety, blocks traversal (`../`), and prevents writes to sensitive dirs (`.git`, `.env`).
*   **Modes:**
    *   `FILE`: Replaces entire file content.
    *   `PATCH`: Context-anchored surgical editing with hash verification.
*   **Options:** 
    *   `--reset`: Wipe the sandbox and start fresh.
    *   `--promote`: Commit verified staged changes to the real repo.
    *   `--sanitize`: Strip UI/Markdown artifacts (Default for Clipboard).
    *   `--strict`: Disable sanitization (use for raw file pipes).

### `slopchop pack` (AI Context Generation)
Knits your repository into a single high-density context file (`context.txt`).
*   **Focus Mode:** Use `--focus <file>` to provide full source for target files while automatically providing "skeletons" (signatures only) for all dependencies.
*   **Token Efficient:** Dramatically reduces context size by stripping function bodies from peripheral code.
*   **Globbing Tip**: To select multiple files (e.g., all markdown), let your shell do the work. Do **not** quote the wildcard.
    *   ✅ `slopchop pack --focus docs/*.md`
    *   ❌ `slopchop pack --focus "docs/*.md"` (Passes literal asterisk; finds nothing)

### `slopchop audit` (Refactor Radar)
Searches for repo-wide duplication and consolidation opportunities.
*   **Fingerprinting:** Uses Weisfeiler-Lehman AST fingerprinting to find structurally similar code even if variable names differ.
*   **Impact Scoring:** Estimates lines saved and difficulty for every cleanup opportunity.

---

## Configuration (`slopchop.toml`)

SlopChop generates a project-specific config if none exists.

```toml
[rules]
max_file_tokens = 2000
max_cyclomatic_complexity = 8
max_nesting_depth = 3
max_function_args = 5
max_function_words = 5

# Paths to ignore for specific laws
ignore_naming_on = ["tests", "spec"]
ignore_tokens_on = ["README.md", "lock", "slopchop_pivot_brief.md"]

[preferences]
theme = "Cyberpunk" # NASA, Corporate, or Cyberpunk
auto_copy = true    # Auto-copy context.txt to clipboard after pack
auto_format = false # Run project formatter after apply
backup_retention = 5
progress_bars = true
require_plan = false # Force payloads to include a PLAN block

[commands]
# Commands run during 'slopchop check'
check = [
  "cargo clippy --all-targets -- -D warnings",
  "cargo test",
]
fix = ["cargo fmt"]
```

---

## Why not just Clippy/ESLint?

Standard linters are excellent for **local syntax and idiom correctness**. SlopChop operates at a different layer:

1.  **Repo-Level Shape:** Linters rarely care if a file has 5000 lines. SlopChop enforces the "Shape of the Repo" to ensure it remains reviewable.
2.  **Ingestion Safety:** Clippy cannot protect you from a malicious or broken multi-file patch. SlopChop's `apply` pipeline is a hardened transaction engine.
3.  **Context Management:** SlopChop understands the relationship between your structural limits and AI context windows, providing tools like `pack` to manage that boundary.

---

## Installation

### Local Build
```bash
cargo install --path .
```

### Hygiene & Maintenance
To ensure your `.gitignore` correctly shields SlopChop's internal sandbox and backups:
```bash
slopchop clean
```

To wipe the current sandbox if you want to abort a staged change:
```bash
slopchop apply --reset
```

---

## Latest: v1.3.2 - High Integrity & Locality v2

SlopChop v1.3.2 is the "God Tier Integrity" release:
*   **Split-Brain Protection:** Blocks promotion if workspace files were manually modified after staging.
*   **Hardened Sandbox:** Fixed S03 (Null byte paths) and I01 (Greedy sigils) vulnerabilities.
*   **Locality v2:** Automated module layer inference and cycle detection to keep your architecture linear.
*   **Physical Stress Testing:** 100% verified coverage across adversarial edge cases.

SlopChop v1.1.0 adds robust protection against "copy-paste slop":
*   **Parser Resilience:** Tolerates indentation (`> `) and artifacts from chat UIs.
*   **Sanitization:** Automatically strips UI-injected markdown code fences from payloads.
*   **Enhanced Diagnostics:** PATCH failures now show "Did you mean?" hints and visual diffs.