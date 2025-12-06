The current README is solid, but it's now slightly out of date given the major features we just shipped (`signatures`, `focus` mode, and the new prompt workflow).

We should update it to reflect the **"Self-Driving Context"** workflow and the new commands.

Here is the plan:
1.  **Add `slopchop signatures`** to the Context Tools section.
2.  **Update "The Workflow"** to include the new 2-step context loop (Signatures -> Pack).
3.  **Update "Adoption Tiers"** to mention the new capabilities.
4.  **Mention `slopchop map`** properly.

**GOAL:** Update README.md to reflect v0.8.0 features.
**CHANGES:**
1. Update `README.md` with new commands and workflow explanation.

#__SLOPCHOP_PLAN__#
GOAL: Update README.md with v0.8.0 features.
CHANGES:
1. Rewrite "The Workflow" section to include Signatures/Architect mode.
2. Update Commands reference.
#__SLOPCHOP_END__#

#__SLOPCHOP_MANIFEST__#
README.md
#__SLOPCHOP_END__#

#__SLOPCHOP_FILE__# README.md
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
* **No truncation** — When AI writes // ... rest of implementation, that's not code. That's giving up. Rejected.

The result: code that's modular, testable, and honest. Not because AI is smart, but because you refused to let it be lazy.
This tool is the proof. It passes its own rules.

---

## What Is This?

SlopChop is the bridge between your AI chat and your codebase.

You (and I) love coding with traditional chat interfaces. The conversation is where the thinking happens. But the last mile **sucks**:

- Copy code, miss a bracket, **broken file**
- AI gives you `// rest of implementation`, **deletes your code**
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

**You:** "I'm getting error X. Here is the map."
**AI:** "I see the issue. It's likely in `src/config.rs`. Please pack that file."

### 3. Surgery (The Pack)
The AI tells you what it needs. You execute.
```bash
slopchop pack src/config.rs --copy
```
*Copies full source code of that file to your clipboard.*

**You:** "Here is the file."
**AI:** [Responds with corrected code in SlopChop format]

### 4. Application
You copy the AI's response.
```bash
slopchop apply
```
*Validates constraints, runs tests, and commits changes automatically.*

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
| `slopchop signatures` | Generate Type Map (The "Header File") |
| `slopchop map` | Show directory tree & sizes |
| `slopchop map --deps` | Show dependency graph visual |
| `slopchop trace <file>` | Trace dependencies deep |
| `slopchop prompt` | Generate system prompt |

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
#__SLOPCHOP_END__#
```

SlopChop parses this, validates it, writes files atomically, runs tests, commits on success.

If AI uses markdown fences or truncates code, rejected.

---

## Who Is This For?

**AI-Native Builders**

You've fully embraced AI coding. You're not scared of it, you're not skeptical of it. You use it daily. Your pain is the copy-paste friction and the quality inconsistency.

If you think AI code is categorically bad: this isn't for you.

If you think AI code needs guardrails to be good: welcome.

---

## The Proof

This tool was built by a product designer chatting with various AI models in chat interfaces. 

It's ~10,000 lines of Rust across 50+ files. It has tree-sitter parsing, a TUI dashboard, dependency graph analysis, and a roadmap system with test traceability.

Run `slopchop` on this repo. It passes its own rules.

That's the point.

---

## Adoption Tiers

### Tier 1: Quality Scanner
```bash
slopchop          # find violations
slopchop check    # run tests
```
Use it as a linter. No AI required.

### Tier 2: AI Workflow
```bash
slopchop signatures # get the map
slopchop pack       # get the code
slopchop apply      # land the changes
```
The core loop.

### Tier 3: Full System
```bash
slopchop roadmap audit    # test traceability
```
For serious projects.

---

## FAQ

**Is this like Cursor?**

No. Cursor replaces your editor with an AI-integrated IDE. SlopChop doesn't touch your editor. It bridges the gap between any chat UI and your existing workflow. Use it with Claude.ai, ChatGPT, local LLMs, whatever.

**Is this like Copilot?**

No. Copilot is autocomplete. SlopChop is for the conversational workflow where you discuss architecture, debug together, and get back complete files.

**Why Rust?**

Fast, single binary, no runtime dependencies, great tree-sitter support, and the tool enforces the same discipline on itself.

**Can I use this with languages other than Rust?**

Yes. Complexity analysis works for Rust, TypeScript, JavaScript, and Python. Token limits and truncation detection work for any file type.

---

## Chop the Slop

AI generates more code than ever. Most of it is slop.

You can reject AI entirely. You can accept slop and drown in tech debt. Or you can chop it.

```
slopchop apply
```

---

*MIT License*

#__SLOPCHOP_END__#
