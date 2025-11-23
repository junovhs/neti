# 🛡️ Warden Protocol

**Architecture Governance for the AI Era.**

> *"We do not ask the AI to write good code. We enforce good code via mechanical constraints."*

Warden is a local toolchain designed to enforce **Code With Intent (POT)**. It solves the "Context Drift" and "Hallucination" problems common in AI coding by enforcing strict structural discipline (Atomicity, Naming, Safety) before code is committed.

## The Ecosystem

This repository contains two binaries that share a single logic core:

1.  **`warden` (The Enforcer):** A structural linter that rejects bloat, complexity, and unsafe code.
2.  **`knit` (The Messenger):** A smart context-packer that serializes your repository for AI consumption, respecting Warden's filters automatically.

---

## 1. The Warden (Linter)

Warden does not check if your code works. It checks if your code is **maintainable**. It enforces the "3 Laws" of this architecture:

### The 3 Laws
1.  **The Law of Atomicity (Anti-Bloat)**
    *   **Rule:** No file may exceed **200 lines of code**.
    *   **Goal:** Forces modularity. Small files fit in AI context windows perfectly and reduce regression errors.
2.  **The Law of Bluntness (Naming)**
    *   **Rule:** Function names must be **≤ 3 words** (e.g., `fetchUser` ✅, `fetchUserAndSaveToDb` ❌).
    *   **Goal:** Enforces Single Responsibility Principle (SRP). If you can't name it simply, split it.
3.  **The Law of Paranoia (Safety)**
    *   **Rule:** Logic files must contain explicit error handling (`Result`, `try/catch`, `Option`).
    *   **Goal:** Prevents "Silent Failures."

### Usage
```bash
# Run inside any Git repo
warden

# Force scan ignored files
warden --no-git

# Verbose mode (see exactly what it checks)
warden -v
```

**Bypass:** To intentionally skip a file (e.g., a UI component with no logic), add this comment to the top of the file:
```rust
// warden:ignore
```

---

## 2. Knit (Context Packer)

Knit is the bridge between your filesystem and the LLM. It stitches your "Atomic" files into a single text stream with clear headers.

### Features
*   **Smart Defaults:** Automatically strips noise (`node_modules`, `target`, `_assets`, `lockfiles`, `tests`, `docs`). You get the **Kernel** of the code, not the fluff.
*   **Shared Brain:** Uses the exact same ignore logic as Warden.
*   **Security:** Filters out secrets (`.env`, keys) and binaries (`.png`, `.exe`) automatically.

### Usage
```bash
# Generates a clean 'context.txt' in the current folder
knit

# Pipe directly to clipboard (Mac)
knit --stdout | pbcopy

# Pipe directly to clipboard (Linux)
knit --stdout | xclip -selection c
```

---

## ⚙️ Configuration

Warden and Knit work out-of-the-box with "Smart Defaults" (ignoring `dist`, `build`, `assets`, etc).

To add custom excludes for a specific project, create a `.wardenignore` file in the project root:

```text
# .wardenignore
legacy_code/
experiment.rs
scripts/
```

---

## 🚀 Installation

Requires Rust (`cargo`).

```bash
# Clone and install globally
git clone https://github.com/yourusername/warden.git
cd warden
cargo install --path .
```

**Recommended Shell Aliases:**
Add these to your `.zshrc` or `.bashrc` for the full workflow:

```bash
# Mac
alias gcp="knit --stdout | pbcopy && echo '📋 Context copied.'"

# Linux
# alias gcp="knit --stdout | xclip -selection c && echo '📋 Context copied.'"
```

---

## 🤖 The AI System Prompt

To make the AI obey Warden, paste this into your System Prompt / Custom Instructions:

```text
ROLE: High-Integrity Systems Architect.
CONTEXT: You are coding inside a strict "Code With Intent" environment enforced by a binary linter called Warden.

THE 3 LAWS (Non-Negotiable):
1. LAW OF ATOMICITY (No Monoliths):
   - Files MUST be < 200 Lines of Code.
   - If a file grows too large, split it immediately.
   - React/UI: Split VIEW (Component.tsx) from LOGIC (useComponent.ts).

2. LAW OF PARANOIA (Safety First):
   - Functions MUST use explicit error handling (Result, try/catch, Option). No silent failures.
   - If a component is pure UI (visuals only), add "// warden:ignore" at the top.

3. LAW OF BLUNTNESS (Naming):
   - Function names Max 3 words (e.g., `fetchData` is good; `fetchDataAndProcess` is bad).

OPERATIONAL PROTOCOL:
1. Scan: Read the provided context.
2. Generate: Output WHOLE FILES with the filename in a header. Do not use diffs.
3. Verify: Ask yourself: "Will Warden reject this?" before printing.
```

---

**License:** MIT  
**Philosophy:** Code With Intent (POT)
