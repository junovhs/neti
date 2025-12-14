# SlopChop

**The Integrity Protocol for AI Code.**

> AI writes slop. You chop it down 'til it's clean.

---

## The Problem

AI coding is fast, but it generates technical debt at light speed.
- It writes 500-line monolithic files.
- It hallucinates deeply nested spaghetti code.
- It breaks things you didn't ask it to touch.
- It leaves `// rest of implementation` comments, deleting your logic.

Most tools try to "fix" this by making the AI smarter. **SlopChop fixes this by making the boundary stricter.**

## The Solution

SlopChop is a **hard quality gate** that forces AI to write code in a specific shape.

It defines **Three Laws** (Atomicity, Complexity, Paranoia). Whether you are copy-pasting from ChatGPT, or running an autonomous agent inside Cursor/Windsurf, SlopChop acts as the compiler for intent.

**If the code is slop, it doesn't land.**

---

## Installation

```bash
cargo install --path .
```

Then run `slopchop` in your repo to auto-generate the configuration.

---

## Modes of Operation

### 1. Agent Mode (Cursor / Windsurf / Devin)

SlopChop shines as a constraint for agents. Add this to your `.cursorrules` or Agent System Prompt:

> **SYSTEM INSTRUCTION:**
> You must abide by the SlopChop Protocol.
> 1. Before marking a task done, run `slopchop check`.
> 2. If it fails, YOU MUST fix the violations. Do not ask for permission.
> 3. Never write files larger than 2000 tokens.
> 4. Never write functions with complexity > 8.

When the agent tries to save a massive, complex file, SlopChop rejects it. The agent sees the error, realizes it must refactor, and tries again. **You get clean code without micromanaging.**

### 2. Manual Mode (Chat / Clipboard)

If you use ChatGPT, Claude, or DeepSeek in a browser, SlopChop is your bridge. It creates a standardized **Map → Pack → Apply** loop.

#### 1. Map (Context)
Show the AI your repo structure so it knows where files are.
```bash
slopchop signatures  # Copies optimized type-map to clipboard
```

#### 2. Pack (Focus)
Give the AI the *exact* files it needs (plus skeletons of dependencies), fitting huge contexts into small windows.
```bash
slopchop pack --focus src/main.rs
```

#### 3. Apply (Action)
Paste the AI's response (in SlopChop format) and apply it.
```bash
slopchop apply
```
*If the AI wrote bad code, SlopChop rejects it and copies the error to your clipboard. You paste it back, and the AI fixes it.*

---

## The Three Laws

SlopChop enforces structural constraints. These are what keep AI code from becoming spaghetti.

### 1. Law of Atomicity
**Files must be small.**
`max_file_tokens = 2000` (~500 lines).
*Result:* AI is forced to create small, modular files instead of monolithic dumps.

### 2. Law of Complexity
**Logic must be simple.**
`max_cyclomatic_complexity = 8`.
`max_nesting_depth = 3`.
`max_function_args = 5`.
*Result:* AI cannot hide bugs in deep nesting. It must write linear, testable logic.

### 3. Law of Paranoia
**No hidden crashes.**
Blocks `.unwrap()`, `.expect()`, and unchecked assumptions.
*Result:* Production-grade error handling is enforced by default.

---

## V1.0 Safety Net

SlopChop is designed to be run blindly on AI output without fear.

*   **Mandatory Integrity:** If the AI hallucinates a file update but doesn't provide the code, SlopChop rejects the **entire** batch. No silent partial updates.
*   **Atomic Rollback:** If *any* part of an operation fails (IO error, lint failure), the **entire** repo rolls back to the exact state before you ran `apply`.
*   **Symlink Protection:** Prevents AI from writing outside the repo root via symlink attacks.

---

## Command Reference

### Core Workflow

| Command | Description |
| :--- | :--- |
| `slopchop` | Scan the current directory for violations. |
| `slopchop check` | Run verification pipeline (useful for CI/Agents). Returns exit code 1 on failure. |
| `slopchop apply` | Apply code from clipboard (Standard Flow). |
| `slopchop pack <file>` | Prepare full file content for LLM. |
| `slopchop pack --focus <file>` | Pack file + *skeletons* of dependencies (Smart Context). |
| `slopchop audit` | Find duplication and dead code (God Tier analysis). |

### Apply Flags

Safety and automation controls for `slopchop apply`:

| Flag | Description |
| :--- | :--- |
| `--dry-run` | Validate integrity and show plan without writing files. |
| `--force` | Skip interactive confirmation prompts (for scripts). |
| `--no-commit` | Apply changes but do not commit to git. |
| `--no-push` | Commit changes locally but do not push to remote. |
| `--stdin` | Read payload from standard input instead of clipboard. |

### Context & Discovery

| Command | Description |
| :--- | :--- |
| `slopchop signatures` | Generate high-level map (Header + Signatures + Footer). |
| `slopchop map` | Show directory tree & token counts. |
| `slopchop map --deps` | Show dependency graph visual. |
| `slopchop trace <file>` | Trace dependencies deep (recursive analysis). |
| `slopchop prompt` | Generate the system prompt text to teach AI the protocol. |

---

## Configuration

`slopchop.toml` lets you tune the strictness.

```toml
[rules]
# The 3 Laws
max_file_tokens = 2000
max_cyclomatic_complexity = 8
max_nesting_depth = 3
max_function_args = 5

[preferences]
auto_copy = true        # Copy output to clipboard automatically
auto_commit = true      # Git commit on success
auto_push = false       # Git push (Disabled by default in V1.0)
backup_retention = 5    # Keep last 5 backups

[commands]
# Commands to run during verification
check = ["cargo test", "cargo clippy -- -D warnings"]
fix = ["cargo fmt"]
```

---

## The Protocol Format

To use SlopChop, the AI must output code in this format. (Run `slopchop prompt` to get the instructions to paste to your AI).

```text
#__SLOPCHOP_PLAN__#
GOAL: Add login feature
CHANGES:
1. Implement auth logic
#__SLOPCHOP_END__#

#__SLOPCHOP_MANIFEST__#
src/auth/login.rs [NEW]
src/main.rs
#__SLOPCHOP_END__#

#__SLOPCHOP_FILE__# src/auth/login.rs
pub fn login(creds: &Credentials) -> Result<Session, AuthError> {
    // complete implementation
    // no truncation
}
#__SLOPCHOP_END__#
```
