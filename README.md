# SlopChop

**Bring rigor to your conversations with AI.**

SlopChop is a command-line tool designed for developers who love the conversational workflow of AI (ChatGPT, Claude, DeepSeek) but hate the friction of moving code back and forth.

It doesn't try to replace the Chat UI with a headless agent. It respects that **chatting is thinking**. The back-and-forth is where the architecture happens.

The problem isn't the conversation; it's the **delivery**.
- You paste code, and it breaks because of bad Markdown escaping.
- The AI writes a 300-line "God Function" that works but ruins your architecture.
- The AI gets lazy and gives you `// ... rest of implementation`.
- You lose track of what you've actually finished vs. what you just talked about.

**SlopChop bridges the gap between the chat window and your compiler.** It ensures that when you decide to apply the AI's advice, it is **Deterministic**, **Atomic**, and **Safe**.

---

## The Philosophy: Chat-Driven Development

We believe that **slowing down** to articulate your problem to an AI is a feature, not a bug. It forces you to think. But once the thinking is done, the *doing* should be instant and error-free.

SlopChop acts as a **Gatekeeper**.

1. **Pack** your context (smartly filtered) to the clipboard.
2. **Converse** with the AI in your browser.
3. **Apply** the result via a rigorous protocol.

If the AI produces garbage, SlopChop rejects it and tells the AI *exactly why*.

---

## How SlopChop Changes the Game

### 1. The Protocol (Certainty in Transport)
Markdown code blocks (` ```rust `) are fragile. AIs mess them up constantly.

SlopChop teaches the AI a specific, delimiter-based protocol (`#__SLOPCHOP_FILE__#`). When you run `slopchop apply`, it doesn't just regex-match text; it parses a structured manifest.

**Result:** No more copy-paste errors. No more "files ending in the middle." If the protocol is valid, the file lands.

### 2. The Three Laws (Enforced Modularity)
AIs love to write spaghetti code. They don't have to maintain it—you do. SlopChop enforces architectural discipline at the gate.

If an AI tries to give you a complex function:
> **SlopChop:** "Error: Function `process_data` has a Cyclomatic Complexity of 12 (Max: 8). Refactor."

You paste that error back to the chat. The AI apologizes and breaks the function into three smaller, testable helpers.

**Result:** You don't just get working code; you get **clean, atomic code**.

### 3. The Anti-Lazy Defense (No Truncation)
We've all seen it:
```rust
fn complicated_logic() {
    // ... existing logic ... // slopchop:ignore
    new_logic();
}
```
If you paste this, you delete your source code.

SlopChop detects these "lazy markers" (comments, placeholders, ellipses) and **rejects the file immediately**, forcing the AI to provide the complete, compile-ready source.

### 4. Shared Memory (The Roadmap)
Chats are ephemeral. Context windows forget.

SlopChop maintains a `ROADMAP.md` that serves as the project's long-term memory. The AI can read it to know where we are, and issue commands to update it:

```
===ROADMAP===
CHECK "auth-login"
ADD "auth-logout" AFTER "auth-login"
===ROADMAP===
```

**Result:** One paste updates your code **and** your project status.

### 5. Smart Context (Dependency-Aware Packing)
Not all files are equal. SlopChop understands your codebase's structure.

```bash
slopchop trace src/apply/mod.rs --depth 2
```

SlopChop walks the import graph, packs your anchor file in full, and includes dependencies as skeletons—optimizing the AI's context window.

---

## A Typical Session

**You:** "I need to fix the login bug. Let's look at the auth module."
```bash
slopchop pack src/auth/
# Context is now on your clipboard.
```

**You (in ChatGPT):** [Paste Context] "The login handler is failing when..."

**AI:** "I see the issue. Here is the fix."
*[AI outputs a SlopChop Protocol block]*

**You:** [Copy Response]
```bash
slopchop apply
```

**SlopChop:**
```text
❌ Validation Failed
- src/auth/login.rs: High Complexity: Score is 10 (Max: 8).
- src/auth/login.rs: Detected lazy truncation marker: '// ...'.
```
*[SlopChop copies this error report to your clipboard]*

**You (in ChatGPT):** [Paste Error]

**AI:** "Apologies. I will refactor to reduce complexity and provide the full file."
*[AI outputs corrected code]*

**You:** [Copy Response]
```bash
slopchop apply
```

**SlopChop:**
```text
✅ Apply successful!
   ✓ src/auth/login.rs
   ✓ src/auth/helpers.rs [NEW]
   
running tests... passed.
git committing... done.
```

You just refactored a module without writing a line of code, and you have **certainty** that it meets your quality standards.

---

## Installation

```bash
cargo install --path .
```

(Requires Rust toolchain. Supports Linux, macOS, and Windows via WSL.)

### Quick Start

```bash
# Initialize configuration (interactive wizard)
slopchop --init

# Or let SlopChop auto-detect and create slopchop.toml
slopchop
```

---

## Commands

### Core Workflow

| Command | Description |
|---------|-------------|
| `slopchop` | Scan codebase for violations |
| `slopchop --ui` | Interactive TUI dashboard |
| `slopchop pack [options]` | Pack context for AI consumption |
| `slopchop apply` | Apply AI response from clipboard |
| `slopchop check` | Run configured check commands |
| `slopchop fix` | Run configured fix commands |

### Smart Context

| Command | Description |
|---------|-------------|
| `slopchop trace <FILE>` | Trace dependencies from anchor file |
| `slopchop map [--deps]` | Show repository structure map |
| `slopchop context [--copy]` | Generate context map |
| `slopchop prompt [--copy]` | Generate system prompt |

### Configuration & Maintenance

| Command | Description |
|---------|-------------|
| `slopchop --init` | Interactive configuration wizard |
| `slopchop config` | TUI configuration editor |
| `slopchop clean [--commit]` | Clean backup files |

### Roadmap Management

| Command | Description |
|---------|-------------|
| `slopchop roadmap show` | Display roadmap tree |
| `slopchop roadmap tasks` | List all tasks |
| `slopchop roadmap apply` | Apply roadmap commands |
| `slopchop roadmap audit` | Verify test coverage |
| `slopchop roadmap prompt` | Generate roadmap prompt |

---

## Pack Options

```bash
slopchop pack [OPTIONS]

Options:
  -s, --stdout         Output to stdout instead of file
  -c, --copy           Copy to clipboard
      --noprompt       Exclude system prompt header
      --format <FMT>   Output format: text, json (default: text)
      --skeleton       Skeletonize all files (signatures only)
      --git-only       Only include git-tracked files
      --no-git         Include all files regardless of git
      --code-only      Exclude markdown and config files
  -v, --verbose        Show progress
      --target <FILE>  Focus on specific file (full content)
  -f, --focus <FILE>   Additional focus files
      --depth <N>      Dependency trace depth (default: 1)
```

### Focus Mode

```bash
# Target file in full, everything else skeletonized
slopchop pack --target src/apply/mod.rs

# Multiple focus files
slopchop pack --focus src/apply/mod.rs --focus src/types.rs
```

---

## Trace Command

```bash
slopchop trace <FILE> [OPTIONS]

Options:
  -d, --depth <N>    Dependency depth (default: 2)
  -b, --budget <N>   Token budget (default: 4000)
```

Traces dependencies from an anchor file, generating optimized context:
- **Anchor**: Full content
- **Direct dependencies**: Skeletonized
- **Indirect dependencies**: Skeletonized

---

## Configuration (`slopchop.toml`)

SlopChop is opinionated, but you can negotiate.

```toml
[rules]
max_file_tokens = 2000              # Keep files small
max_cyclomatic_complexity = 8       # Keep logic simple
max_nesting_depth = 3               # Keep indentation flat
max_function_args = 5               # Keep interfaces clean
max_function_words = 5              # Keep names focused
ignore_tokens_on = [".lock", ".md"] # Skip token checks
ignore_naming_on = ["tests"]        # Skip naming checks

[commands]
# SlopChop runs these before committing. If they fail, the apply is rejected.
check = ["cargo test", "cargo clippy --all-targets -- -D warnings"]
fix = "cargo fmt"
```

---

## The Protocol Format

AI outputs follow this structure:

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

#__SLOPCHOP_FILE__# src/file2.rs
// Another complete file
#__SLOPCHOP_END__#
```

### Block Types

| Block | Purpose |
|-------|---------|
| `PLAN` | Human-readable summary |
| `MANIFEST` | Declares all files being touched |
| File paths | Actual file content |

### Markers

| Marker | Meaning |
|--------|---------|
| `[NEW]` | File will be created |
| `[DELETE]` | File will be removed |
| *(none)* | File will be updated |

---

## The Three Laws

### Law of Atomicity
Files must be small enough to reason about.
- Default: 2000 tokens (~500 lines)
- Enforced: Reject files exceeding limit

### Law of Complexity
Functions must be simple enough to test.
- Cyclomatic complexity ≤ 8
- Nesting depth ≤ 3
- Function arguments ≤ 5
- Function name words ≤ 5

### Law of Paranoia (Rust)
No panic paths in production code.
- `.unwrap()` → Rejected
- `.expect()` → Rejected
- `.unwrap_or()` → Allowed
- `?` operator → Allowed

---

## "Is this an Agent?"

No. Agents (like Devin) try to be the pilot. **SlopChop keeps you as the pilot.**

SlopChop is the navigation system and the safety interlocks. It allows you to use the most powerful LLMs available (which are currently chat-based) without sacrificing the integrity of your local codebase.

---

## Adoption Tiers

### Tier 1: Structural Linting Only
```bash
slopchop              # Scan for violations
slopchop check        # Run tests/linters
```
Use SlopChop as a code quality scanner without AI integration.

### Tier 2: AI-Assisted Development
```bash
slopchop pack         # Generate context for AI
slopchop apply        # Apply AI responses
```
Add the pack/apply loop for AI coding sessions.

### Tier 3: Full Traceability
```bash
slopchop roadmap audit --strict
```
Every feature tied to a test, programmatic progress tracking.

---

*MIT License*
