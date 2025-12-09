# SlopChop

**AI writes slop. You chop it down til its clean.**

---

## The Story

SlopChop came from one idea: stop trying to make AI perfect, just refuse its bad output.

Most of the thinking around “AI for code” is top-down: predict every failure mode, design heuristics for all of them. SlopChop is a bottom-up, antifragile bet instead. Treat AI as a noisy generator, then put a hard filter in front of your repo.

I learned this as a production artist with early AI image gen: generate hundreds, throw away almost everything, keep the few frames that are gold, and manually composite one verified, high-quality design. SlopChop is that same filter for code.

It doesn’t try to foresee every way AI can mess up. It enforces a few hard rules at the boundary. If code is too complex, too big, truncated, or unsafe, it simply doesn’t land.


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

## The Workflow

SlopChop teaches the AI to navigate your repo through a simple loop: **Map → Pack → Apply**.

### The Loop

```mermaid
flowchart TD
    subgraph Loop
        M[🗺️ Map<br><i>you show codebase</i>]
        P[📦 Pack<br><i>AI asks, you provide</i>]
        A[⚡ Apply<br><i>you land or reject</i>]
    end
    
    M --> P --> A
    A -->|"✗ Rejected"| P
    A -->|"✓ Committed"| E(( ))
````

#### 1. Map — Show the AI your codebase

```bash
slopchop signatures
```

Copies a high-level map of every type and function to your clipboard.

> **You:** "I'm getting error X. Here's my codebase."
> **AI:** "I see the issue. It's likely in `src/config.rs`. Can you pack that file?"

#### 2. Pack — Run the command it gives you, to give the AI what it wants

```bash
slopchop pack --focus src/config.rs
```

Copies the full file + skeletons of its dependencies.

> **You:** **pastes**
> **AI:** **responds with fixed code in SlopChop format**

#### 3. Apply — Land the changes (or reject the slop)

Copy the AI's **entire** response, then:

```bash
slopchop apply
```

If clean: tests and lints run, changes commit.
If slop is detected:

```text
✗ REJECTED
- src/auth/login.rs: complexity 12 (max 8)
- src/auth/login.rs: detected "// ..." truncation

[error automatically copied to clipboard]
```

Paste the error back. AI fixes it. Repeat.

---

**The AI learns your (configurable) standards through rejection + automatic corrections.**

---

## The Killer Feature: Watch Mode (Coming Soon / Experimental)

```bash
slopchop watch
```

Runs in the background. Watches your clipboard.

1. You copy from your AI of choice
2. Notification: "3 files ready. ⌘⇧L to apply"
3. Press hotkey
4. Done. Never left the browser.

---

## The Three Laws

SlopChop enforces structural constraints. These are what keep AI code from becoming spaghetti.

### Law of Atomicity

Files must be small enough to review.

```text
max_file_tokens = 2000  (~500 lines)
```

### Law of Complexity

Functions must be simple enough to test.

```text
max_cyclomatic_complexity = 8
max_nesting_depth = 3
max_function_args = 5
```

### Law of Paranoia (Rust)

No hidden crash paths.

```text
.unwrap()      → rejected
.expect()      → rejected
.unwrap_or()   → allowed
?              → allowed
```

---

## Consolidation Audit (GOD TIER)

Once your code passes the basic laws, you still have a different problem: **duplication and dead weight**.

```bash
slopchop audit
```

This runs a structural audit over your repo:

* Finds **near-duplicate functions** and suggests enum-based consolidation.
* Flags **repeated patterns** (formatting, error wrapping, spawn/pipe/wait, etc.).
* Builds a **call graph** to look for dead code (entrypoints that are never reached).
* Estimates how many **lines you could delete or merge**.

Example:

```text
📊 SUMMARY

   Files analyzed:    114
   Code units found:  661
   Similarity clusters: 22
   Dead code units:     0
   Repeated patterns:   683

   💡 987 lines could potentially be removed/consolidated
```

And it surfaces concrete opportunities:

```text
1. [HIGH] 4 similar functions: q_complexity, q_imports, q_defs, q_exports
   📈 ~72 lines | difficulty: 1/5 | confidence: 100% | score: 72.0
   📁 src/lang.rs
   💡 Consolidate these 4 functions into a single parameterized implementation

   🤖 GOD TIER PLAN:
   │ #[derive(Debug, Clone, Copy)]
   │ pub enum QueryKind {
   │     Complexity,
   │     Imports,
   │     Defs,
   │     Exports,
   │ }
   │
   │ pub fn query(&self, kind: QueryKind) -> &'static str {
   │     match (self, kind) {
   │         (Self::Rust, QueryKind::Complexity) => todo!(),
   │         (Self::Rust, QueryKind::Imports)    => todo!(),
   │         (Self::Rust, QueryKind::Defs)       => todo!(),
   │         (Self::Rust, QueryKind::Exports)    => todo!(),
   │     }
   │ }
```

You can hand-roll the refactor, or feed that “GOD TIER PLAN” back into your AI and let it do the mechanical work.

---

## Installation

```bash
cargo install --path .
```

Then:

```bash
slopchop config  # interactive setup
```

Or just run `slopchop` and it auto-generates config.

---

## Commands

### Core Workflow

| Command                        | What it does                                           |
| ------------------------------ | ------------------------------------------------------ |
| `slopchop`                     | Scan codebase for violations                           |
| `slopchop apply`               | Apply AI response from clipboard                       |
| `slopchop pack <file>`         | Pack specific file (full source)                       |
| `slopchop pack --focus <file>` | Pack file + skeleton of dependencies                   |
| `slopchop audit`               | Analyze repo for duplication & dead-code opportunities |

### Context Tools

| Command                 | What it does                                |
| ----------------------- | ------------------------------------------- |
| `slopchop signatures`   | Generate Map (Header + Signatures + Footer) |
| `slopchop map`          | Show directory tree & sizes                 |
| `slopchop map --deps`   | Show dependency graph visual                |
| `slopchop trace <file>` | Trace dependencies deep                     |
| `slopchop prompt`       | Generate system prompt text                 |

### Project Management

> Experimental project-management commands. API may change.

| Command                  | What it does           |
| ------------------------ | ---------------------- |
| `slopchop roadmap show`  | Display progress       |
| `slopchop roadmap apply` | Update roadmap from AI |
| `slopchop roadmap audit` | Verify test coverage   |

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

```text
#__SLOPCHOP_FILE__# src/auth/login.rs
pub fn login(creds: &Credentials) -> Result<Session, AuthError> {
    // complete implementation
    // no truncation
}
```

You copy the whole response, run `slopchop apply`, and let the tool decide if it lands or gets chopped.

```

