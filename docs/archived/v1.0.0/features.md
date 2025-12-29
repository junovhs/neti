SlopChop is a **High-Integrity Systems Architect**. It is not just a linter; it is a transactional gatekeeper that sits between AI agents (or developers) and your codebase to prevent degradation, bugs, and "slop."

---

A.

## 1. The Core Philosophy: The 3 Laws
SlopChop enforces three structural laws by default. These are checked every time you run `scan` or `check`.

1.  **Law of Atomicity:** Files must be small (default < 2000 tokens). Large files are rejected.
2.  **Law of Complexity:** Logic must be simple.
    *   Cyclomatic Complexity ≤ 8
    *   Nesting Depth ≤ 3
    *   Function Arguments ≤ 5
3.  **Law of Paranoia:** Safety is mandatory.
    *   Bans `.unwrap()` and `.expect()` (in Rust).
    *   Requires `// SAFETY:` comments for any `unsafe` blocks.

---

## 2. The Architecture: Staged Workspace
This is the security boundary. SlopChop **never** writes AI-generated code directly to your working files.

1.  **The Stage:** When you run `apply`, changes are written to a shadow copy of your repo located at `.slopchop/stage/worktree/`.
2.  **Verification:** You run tests and checks *inside* this shadow stage.
3.  **Promotion:** Only when you are satisfied do you run `apply --promote`, which transactionally moves the files to your real folder.
4.  **Rollback:** If a promotion fails halfway, SlopChop auto-reverts to the previous state using internal backups.

---

## 3. The `XSC7XSC` Protocol
SlopChop listens for specific text patterns (usually copied to your clipboard). It ignores Markdown fences to prevent rendering errors.

*   **`XSC7XSC PLAN XSC7XSC`**: A summary of intent (for human review).
*   **`XSC7XSC MANIFEST XSC7XSC`**: A strict list of files to be touched. If a file is in a FILE/PATCH block but not here, it is rejected.
*   **`XSC7XSC FILE XSC7XSC`**: Replaces an entire file.
*   **`XSC7XSC PATCH XSC7XSC`**: Surgical editing (see below).

### Surgical Patching (V1 Engine)
The patch engine is context-anchored and cryptographically locked.
*   **`BASE_SHA256`**: The patch includes the hash of the file it expects. If you changed the file, the patch rejects (prevents "stale overwrite").
*   **Context Anchors**: It requires `LEFT_CTX`, `OLD`, and `RIGHT_CTX`. It verifies this sequence appears exactly **once** in the file. If it appears 0 times or >1 times, it rejects.
*   **Diagnostics**: If a patch fails, SlopChop runs a probe to find "Did you mean this region?" and shows you the diff.

---

## 4. Command Reference

### Core Loop
*   **`slopchop apply`**
    *   Reads payload from **Clipboard** (default), Stdin, or File.
    *   Applies changes to the **Stage**.
    *   **`--promote`**: Moves staged files to real workspace.
    *   **`--reset`**: Destroys the stage (undo).
    *   **`-c` / `--check`**: Runs `check` immediately after applying.
    *   **`--dry-run`**: Validates the payload without writing anything.

*   **`slopchop check`**
    *   Runs your configured linters/tests (e.g., `cargo test`, `eslint`).
    *   Runs the SlopChop structural `scan`.
    *   **Smart Context:** If a stage exists, it runs checks inside the stage. If not, it runs them in your workspace.

*   **`slopchop scan`**
    *   Runs *only* the SlopChop structural analysis (The 3 Laws).
    *   Fast and deterministic.

### Context Generation (for AI)
*   **`slopchop pack`**
    *   Generates a high-density `context.txt` of your codebase.
    *   **`--focus <file>`**: The killer feature. Puts the target file in full view, but compresses all other files into "Skeletons" (signatures only, no bodies) to save tokens.
    *   **`--noprompt`**: Excludes the system prompt instructions (raw code only).
    *   **`--copy`**: Copies result to clipboard.
    *   **Globbing Tip**: To select multiple files (e.g., all markdown), let your shell do the work. Do **not** quote the wildcard.
        *   ✅ `slopchop pack --focus docs/*.md`
        *   ❌ `slopchop pack --focus "docs/*.md"` (Passes literal asterisk; finds nothing)

*   **`slopchop prompt`**
    *   Outputs the raw System Prompt (The 3 Laws + Protocol instructions) to paste into an LLM.

### Analysis & Insights
*   **`slopchop audit`**
    *   **Refactor Radar:** Scans the codebase for structural duplication.
    *   Uses AST Fingerprinting (ignores variable names, looks at logic shape).
    *   Detects Dead Code and Repeated Patterns.
    *   Outputs a scored list of refactoring opportunities.

*   **`slopchop trace <file>`**
    *   Generates a dependency graph for a specific file.
    *   Shows what imports it, and what it imports.

*   **`slopchop map`**
    *   Prints a tree view of the repository structure with token counts and complexity metrics per file.

*   **`slopchop signatures`**
    *   Extracts only types, function signatures, and structs from the codebase. Useful for giving an AI a high-level map of the system without implementation details.

### Maintenance
*   **`slopchop fix`**
    *   Runs the configured auto-fix command (e.g., `cargo fmt`, `eslint --fix`).
*   **`slopchop clean`**
    *   Updates `.gitignore` to ensure SlopChop's internal files (`.slopchop/`) are ignored.
    *   Removes `context.txt`.
*   **`slopchop dashboard`** (or just `slopchop --ui`)
    *   Launches a Terminal User Interface (TUI) to visualize violations, file sizes, and config.

---

## 5. Configuration (`slopchop.toml`)

You can tune the laws per project.

*   **`[rules]`**: Set specific limits for tokens, complexity, depth, and args.
    *   `ignore_naming_on`: List files/dirs where long names are okay (e.g., `tests`).
    *   `ignore_tokens_on`: List files allowed to be huge (e.g., `lock` files).
*   **`[commands]`**:
    *   `check`: The list of shell commands to run during `slopchop check`.
    *   `fix`: The command to run for `slopchop fix`.
*   **`[preferences]`**:
    *   `auto_copy`: Automatically copy `context.txt` to clipboard after packing.
    *   `backup_retention`: How many promotion backups to keep.

## 6. Observability
*   **`.slopchop/events.jsonl`**: A machine-readable log of every action taken by the tool (Applies, Checks, Promotes, Resets). Useful for auditing agent behavior.

B.

### 1. The "Smart Clipboard" (Anti-Hallucination)
*   **Feature:** Context overflow protection.
*   **Behavior:** When you run `slopchop pack --copy`, it checks the token count.
    *   **Small (< 2k tokens):** Copies raw text to clipboard.
    *   **Huge (> 2k tokens):** It writes the content to a temporary file (`/tmp/slopchop_clipboard_...`) and copies the **file handle** to your clipboard.
*   **Why:** Pasting 50k tokens of text into an LLM chatbox often breaks the UI or gets silently truncated. Pasting it as a *file attachment* ensures the LLM reads 100% of the context perfectly.

### 2. The TUI "Interceptor" Mode
*   **Command:** `slopchop dashboard` (or `--ui`)
*   **Feature:** Background Clipboard Watcher (`src/tui/watcher.rs`).
*   **Behavior:** While the dashboard is open, it watches your system clipboard. If you copy a block of text containing `XSC7XSC` protocol markers from an AI (e.g., ChatGPT generates a patch), the Dashboard lights up:
    > **🚀 XSC7XSC PAYLOAD DETECTED**
    > **Press 'a' to apply.**
*   **Why:** It creates a "Command Center" workflow where you don't even need to type `slopchop apply` in a separate terminal. You just chat with the AI, copy the response, and hit 'a' in the SlopChop dashboard.

### 3. Polyglot AST Support
I mentioned "AST scanning," but specifically, SlopChop has `tree-sitter` parsers compiled in for:
*   **Rust** (First-class support: safety checks, banned methods, full complexity metrics).
*   **Python** (Complexity checks, function/class definitions).
*   **TypeScript / JavaScript / TSX** (Imports, exports, function complexity).
*   **Go** (Basic detection).
*   **Generic Fallback:** For other languages, it falls back to line-based heuristics so `pack` doesn't crash, though "Complexity" laws won't be enforced as strictly.

### 4. Auto-Detection (Zero Config)
*   **Feature:** `src/project.rs` detection logic.
*   **Behavior:** If you run SlopChop in a fresh repo without a `slopchop.toml`, it sniffs the environment:
    *   Sees `Cargo.toml`? -> Sets default check to `cargo clippy`.
    *   Sees `package.json`? -> Sets default check to `npm test` / `eslint`.
    *   Sees `requirements.txt`? -> Sets default check to `ruff`.
*   **Why:** You can drop `slopchop` into almost any repo and it provides value immediately without spending 10 minutes writing config files.

### 5. `.slopchopignore`
*   **Feature:** Specific exclusion logic.
*   **Behavior:** SlopChop respects standard `.gitignore`, but sometimes you want the AI to see files that Git ignores (or vice versa). You can create a `.slopchopignore` file to strictly control what `scan` and `pack` see, independent of Git.

### 6. Audit "Plan Generation"
*   **Feature:** `src/audit/codegen.rs`.
*   **Behavior:** When `slopchop audit` finds duplication, it doesn't just complain. It attempts to **write Rust code** to solve it.
    *   If it sees 3 functions that differ only by a string literal, it suggests an `enum` + a refactored function signature to unify them.
    *   It prints this suggestion in the audit report.

### 7. Trace Budgeting
*   **Command:** `slopchop trace <file> --budget <tokens>`
*   **Feature:** Token economics.
*   **Behavior:** If you trace a file with a massive dependency tree, SlopChop calculates the `PageRank` (importance) of every dependency. It fills the output context starting with the most important files until it hits the `--budget` limit, then cuts off the rest. It ensures you get the *most relevant* context for a fixed context window size.

### 8. Map Dependencies
*   **Command:** `slopchop map --deps`
*   **Feature:** Visual Dependency Graph.
*   **Behavior:** The standard `map` is a file tree. Adding `--deps` changes the visualization to show outgoing dependency arrows (`->`) for each file in the tree, derived from the import graph.

C.

### 1. `pack --format xml` (Claude Optimization)
*   **Feature:** XML Output Mode.
*   **Command:** `slopchop pack --format xml`
*   **Behavior:** Instead of the standard `XSC7XSC` sigil format, it wraps files in standard XML tags:
    ```xml
    <documents>
      <document path="src/main.rs"><![CDATA[ ... code ... ]]></document>
    </documents>
    ```
*   **Why:** Some models (specifically Anthropic's Claude) are trained to pay closer attention to XML-structured data than custom delimiters. If you are using Claude, this flag essentially "jailbreaks" its attention span.

### 2. Holographic Topological Ordering
*   **Feature:** `src/signatures/ordering.rs`
*   **Behavior:** When you run `slopchop signatures`, it doesn't just dump files alphabetically.
    1.  It builds a dependency graph of your code.
    2.  It runs a **Topological Sort** to find the "root" files (types/structs used by everyone else).
    3.  It puts those root files at the **top** of the output.
*   **Why:** This ensures the AI reads the *definitions* of your types before it reads the functions that *use* them. It mimics how a human compiler (or a human brain) learns a codebase, reducing hallucination of non-existent methods.

### 3. Symlink Jailbreak Protection
*   **Feature:** `src/apply/writer.rs` -> `check_symlink_escape`
*   **Behavior:** Before writing *any* file to the stage, SlopChop resolves the path. If the path contains a symlink that points **outside** the repository root (e.g., `src/link -> /etc/passwd`), it hard-rejects the write.
*   **Why:** Malicious or hallucinating AIs might try to overwrite system files via symlink traversal. SlopChop creates a chroot-like guarantee without needing actual OS privileges.

### 4. "Smart" Skeletonization
*   **Feature:** `src/skeleton.rs`
*   **Behavior:** When you use `pack --focus target.rs`, the other files are "skeletonized." This isn't a regex hack.
    *   It uses `tree-sitter` to parse the AST.
    *   It identifies function bodies (`{ ... }`) vs signatures.
    *   It recursively hollows out nested logic but keeps the structural shape intact.
*   **Why:** Regex-based removal breaks on unmatched braces or complex nesting. AST-based removal guarantees the resulting skeleton is syntactically valid (usually), so the AI understands the structure perfectly while saving 90% of the tokens.

### 5. Standardized Exit Codes
*   **Feature:** `src/exit.rs`
*   **Behavior:** SlopChop returns specific integer exit codes for specific failure modes, making it safe for CI/CD scripting:
    *   `0`: Success
    *   `1`: Generic Error (IO/Config)
    *   `2`: Invalid Input (Parser failed)
    *   `3`: Safety Violation (Path traversal attempt)
    *   `4`: Patch Failure (Hash mismatch/Ambiguity)
    *   `5`: Promote Failure (Write error)
    *   `6`: Check Failed (Lints/Tests failed)

### 6. The "System Bell"
*   **Feature:** `system_bell` in `slopchop.toml`
*   **Behavior:** If set to `true`, the terminal will emit an audible beep (`\x07`) when a long-running task (like a massive scan or pack) completes.
*   **Why:** For the developer who tabs away while the AI generates a massive refactor plan.
