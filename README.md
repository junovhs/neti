# SlopChop

**AI writes slop. You chop it clean.**

---

## The Story

I'm mainly a product designer. This tool is the embodiment of a workflow I've developed over the last several years playing with LLMs. Every line came from whatever smarty-pants AI model was kicking ass that week.

While senior devs and teams can use this as a sort of structural/code quality linter (same rules, no AI required), the full suite is really geared at low/no-code folk like myself who are building entire projects through conversation with a traditional chat interface for the AI.

How does that even work?

By enforcing rules that keep AI output clean. Small files. Simple functions. No lazy truncation. The AI learns on the spot via rejection.
If vibe coding is building a house of cards, SlopChop is building a house of cards with glue. At every step, you refuse code that doesn't meet strict quality standards. You never have to "go back and fix something later" — that's where projects collapse. I mean, as much as possible, anyway.

Here's what that actually means in practice:

* **Small files** — AI reasons better over code it can see entirely. So does your reviewer (you).
* **Low complexity** — Deeply nested logic confuses AI. It confuses humans too. Keep it flat. Why not be draconian about it? If AI is writing the code, who cares if in the end you used fewer tokens and the outcome is cleaner.
* **No panic paths** — In Rust, .unwrap() is a hidden crash. SlopChop bans it, forcing proper error handling.
* **No truncation** — When AI writes // ... rest of implementation, that's not code. That's giving up. Rejected. <!-- slopchop:ignore -->

The result: code that's modular, testable, and honest. Not because AI is smart, but because you refused to let it be lazy.
This tool is the proof. It passes its own rules.

---

## What Is This?

SlopChop is the bridge between your AI chat and your codebase.

You (and I) love coding with traditional chat interfaces. The conversation is where the thinking happens. But the last mile **sucks**:

- Copy code, miss a bracket, **broken file**
- AI gives you `// rest of implementation`, **deletes your code** <!-- slopchop:ignore -->
- 300-line god function **you didn't ask for**
- Context window forgets everything between sessions

SlopChop **fixes all of this.**

---

## The Workflow: Self-Driving Context

SlopChop doesn't just check code; it teaches the AI how to navigate your repo.

### 1. Setup
Copy the system prompt into your AI settings (Claude Project / ChatGPT Instructions).
```bash
slopchop prompt --copy
```

### 2. Diagnosis (The Map)
When you have a bug or need a feature, but don't know where to start:
```bash
slopchop signatures
```
*Copies a high-level map of all types and functions to your clipboard.*
*Automatically includes the System Prompt & Rules.*

**You:** "I'm getting error X. Here is the map."
**AI:** "I see the issue. It's likely in `src/config.rs`. Please pack that file."

### 3. Surgery (The Pack)
The AI tells you what it needs. You execute.
```bash
slopchop pack --focus src/config.rs
```
*Copies `src/config.rs` (Full Source) + Dependencies (Skeletons).*
*Smart Copy automatically puts it in clipboard (or attaches file if huge).*

**You:** "Here is the context."
**AI:** [Responds with corrected code in SlopChop format]

### 4. Application
You copy the AI's response.
```bash
slopchop apply
```
*Validates constraints, runs tests, and commits changes automatically.*

If the AI gives you slop:

```
    slopchop apply
    
    ✗ REJECTED
    - src/auth/login.rs: complexity 12 (max 8)
    - src/auth/login.rs: detected "// ..." truncation <!-- slopchop:ignore -->
    
    [error copied to clipboard]
```

Paste the error back. AI apologizes. Fixes it. Resubmit.

**The AI learns your standards through rejection, not instruction.**

---

## The Killer Feature: Watch Mode (Coming Soon)

```
slopchop watch
```

Runs in background. Watches your clipboard.

1. You copy from Claude
2. Notification: "3 files ready. ⌘⇧L to land"
3. Press hotkey
4. Done. Never left the browser.

---

## The Three Laws

SlopChop enforces structural constraints. These are what keep AI code from becoming spaghetti.

### Law of Atomicity
Files must be small enough to review.
```
max_file_tokens = 2000  (~500 lines)
```

### Law of Complexity
Functions must be simple enough to test.
```
max_cyclomatic_complexity = 8
max_nesting_depth = 3
max_function_args = 5
```

### Law of Paranoia (Rust)
No hidden crash paths.
```
.unwrap()  → rejected
.expect()  → rejected
.unwrap_or() → allowed
?          → allowed
```

---

## Installation

```bash
cargo install --path .
```

Then:

```bash
slopchop --init    # interactive setup
```

Or just run `slopchop` and it auto-generates config.

---

## Commands

### Core Workflow

| Command | What it does |
|---------|--------------|
| `slopchop` | Scan codebase for violations |
| `slopchop apply` | Apply AI response from clipboard |
| `slopchop pack <file>` | Pack specific file (full source) |
| `slopchop pack --focus <file>` | Pack file + skeleton of dependencies |

### Context Tools

| Command | What it does |
|---------|--------------|
| `slopchop signatures` | Generate Map (Header + Signatures + Footer) |
| `slopchop map` | Show directory tree & sizes |
| `slopchop map --deps` | Show dependency graph visual |
| `slopchop trace <file>` | Trace dependencies deep |
| `slopchop prompt` | Generate system prompt text |

### Project Management

| Command | What it does |
|---------|--------------|
| `slopchop roadmap show` | Display progress |
| `slopchop roadmap apply` | Update roadmap from AI |
| `slopchop roadmap audit` | Verify test coverage |

---

## Configuration

`slopchop.toml`:

```toml
[rules]
max_file_tokens = 2000
max_cyclomatic_complexity = 8
max_nesting_depth = 3
max_function_args = 5

[commands]
check = ["cargo test", "cargo clippy -- -D warnings"]
fix = "cargo fmt"
```

---

## The Format

AI outputs code in this format:

```
#__SLOPCHOP_FILE__# src/auth/login.rs
pub fn login(creds: &Credentials) -> Result<Session, AuthError> {
    // complete implementation
    // no truncation
}