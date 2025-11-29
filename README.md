# 🛡️ Warden Protocol

**Structural governance for AI-assisted development.**

> *"The rules are like the seat belts in a car: Initially, using them is perhaps a little uncomfortable, but after a while, it becomes second nature, and not using them is unimaginable."*
> — Gerard J. Holzmann, NASA/JPL

Warden enforces **NASA "Power of 10" Rules** adapted for the AI coding era. It's not a style linter—it's a containment system that keeps AI-generated code verifiable, modular, and sane.

    cargo install --path .

---

## The Problem

You paste 50KB into Claude. It generates a 400-line function with 6 levels of nesting. Now you're debugging something neither you nor the AI can reason about.

AI-generated code **drifts**:
- Functions bloat
- Complexity creeps
- Context windows overflow
- Hallucinations compound

Warden stops this at the source.

---

## The 3 Laws

### 1. Law of Atomicity
Files must be **< 2000 tokens**.

Small files fit in context windows. Small files are verifiable. Small files can be taken to an AI in isolation and brought back—they just slot in.

### 2. Law of Complexity
- **Cyclomatic Complexity:** ≤ 10 per function
- **Nesting Depth:** ≤ 3 levels
- **Function Arguments:** ≤ 5 parameters

These aren't style preferences. They're **containment protocols**. Low complexity bounds the hallucination surface. Shallow nesting prevents AI losing track of scope.

### 3. Law of Paranoia
- No `.unwrap()` or `.expect()` (Rust)
- Fallible operations must return `Result`

The type system is your ally. Don't let AI lie to the compiler.

---

## The Dream

Take any file to a fresh AI conversation. Work on it. Bring it back.

**It slots in perfectly. Every time. Guaranteed.**

This is the ultimate dream of Warden: modularity so strict that files become interchangeable, verifiable units.

---

## Quick Start

    cd your-project
    warden              # Scan for violations (auto-creates warden.toml)
    knit --prompt       # Generate context.txt for AI

That's it. Warden detects your project type (Rust/Node/Python) and configures itself.

---

## The Workflow

Warden is a closed-loop system for AI development.

### 1. Generate Context

    knit --prompt

Creates `context.txt` containing:
- Your codebase (filtered, deduplicated)
- The Warden Protocol system prompt
- Current violations (AI sees what to fix)
- Token count

### 2. Chat with AI

`context.txt` will be generated and applied to your clipboard, paste the file into Claude/GPT/Gemini. Ask for changes.

The AI responds with structured output:

    ∇∇∇ src/lib.rs ∇∇∇
    // complete file contents
    ∆∆∆
    
    ∇∇∇ src/new_module.rs ∇∇∇
    // complete file contents
    ∆∆∆

### 3. Apply Changes

    warden apply

This:
- Extracts file blocks from clipboard
- **Validates paths** (blocks traversal, sensitive files, hidden files)
- **Rejects truncated output** (unbalanced braces, `// ...` markers)
- **Rejects markdown artifacts** (no fenced code blocks in source)
- Creates timestamped backup
- Writes files atomically
- On failure: copies AI-friendly error to clipboard (future plans to make this configurable)

### 4. Verify

    warden          # Structural analysis
    warden check    # Run your linter (clippy, biome, ruff)

If violations exist, exit code is non-zero. CI-friendly.

### 5. Iterate

If `warden apply` fails, the error is already in your clipboard. Paste it back to AI, get corrected output, repeat.

---

## Commands

| Command | Description |
|---------|-------------|
| `warden` | Run structural scan |
| `warden --ui` | Interactive TUI dashboard |
| `warden --init` | Create/regenerate warden.toml |
| `warden apply` | Apply AI response from clipboard |
| `warden apply --dry-run` | Validate without writing |
| `warden apply --commit` | Apply and git commit |
| `warden check` | Run configured linter |
| `warden fix` | Run configured formatter |
| `warden prompt` | Print system prompt |
| `warden prompt -c` | Copy system prompt to clipboard |
| `knit` | Generate context.txt |
| `knit --prompt` | Include system prompt |
| `knit --skeleton` | Signatures only (coming soon) |
| `knit --stdout` | Output to stdout |
| `warden roadmap init` | Create new ROADMAP.md |
| `warden roadmap prompt` | Copy AI teaching prompt to clipboard |
| `warden roadmap apply` | Apply roadmap commands from clipboard |
| `warden roadmap show` | Display roadmap status |
| `warden roadmap tasks` | List tasks with paths |

---

## Configuration

Warden auto-generates `warden.toml` based on project type:

**Rust:**

    [rules]
    max_file_tokens = 2000
    max_cyclomatic_complexity = 5
    max_nesting_depth = 2
    max_function_args = 5
    
    [commands]
    check = "cargo clippy --all-targets -- -D warnings -D clippy::pedantic"
    fix = "cargo fmt"

**Node/TypeScript:**

    [commands]
    check = "npx @biomejs/biome check src/"
    fix = "npx @biomejs/biome check --write src/"

**Python:**

    [commands]
    check = "ruff check ."
    fix = "ruff check --fix ."

### Tuning Strictness

Strict (for greenfield):

    [rules]
    max_file_tokens = 1500
    max_cyclomatic_complexity = 4
    max_nesting_depth = 2

Relaxed (for legacy adoption):

    [rules]
    max_file_tokens = 3000
    max_cyclomatic_complexity = 10
    max_nesting_depth = 4

---

## Safety Features

### Path Safety Validation

Warden blocks dangerous paths before they touch disk:

| Threat | Example | Status |
|--------|---------|--------|
| Directory traversal | `../etc/passwd` | Blocked |
| Absolute paths | `/etc/passwd`, `C:\Windows` | Blocked |
| Git internals | `.git/config` | Blocked |
| Secrets | `.env`, `.ssh/`, `.aws/` | Blocked |
| Hidden files | `.secrets`, `.private` | Blocked |

### Truncation Detection

AI output often gets cut off. Warden catches:
- Unbalanced `{}`, `[]`, `()`
- Truncation markers: `// ...`, `// rest of file`
- Files ending mid-statement

### Markdown Rejection

Chat interfaces love to wrap code in fenced blocks. Warden rejects any file containing triple backticks or tildes—these corrupt source files.

### Atomic Backups

Before any write: `.warden_apply_backup/TIMESTAMP/`

Your original files are always preserved.

---

## The Nabla Format

XML tags get mangled by chat interfaces. Warden uses Unicode delimiters that never appear in real code:

    ∇∇∇ path/to/file.rs ∇∇∇
    fn main() {
        println!("Hello");
    }
    ∆∆∆

- `∇∇∇` (nabla) opens a file block
- `∆∆∆` (delta) closes it
- Never interpreted as HTML
- Never rendered as markdown
- Trivial to parse

---

## TUI Dashboard

    warden --ui

| Key | Action |
|-----|--------|
| `j/k` | Navigate files |
| `s` | Cycle sort (name/size/errors) |
| `f` | Toggle error filter |
| `q` | Quit |

---

## Roadmap Management

Warden includes AI-friendly roadmap management. Instead of AI rewriting your entire roadmap, it sends surgical commands.

    warden roadmap init          # Create ROADMAP.md
    warden roadmap prompt        # Copy AI instructions to clipboard
    warden roadmap apply         # Apply commands from clipboard

### The Workflow

1. Run `warden roadmap prompt`
2. Paste to AI, describe what changed
3. AI responds with commands:
```
===ROADMAP===
CHECK truncation-detection
ADD v0.5.0 "New feature" AFTER truncation-detection
NOTE auth-system "Needs refactor"
===END===
```

4. Run `warden roadmap apply`

### Commands

| Command | Example |
|---------|---------|
| `CHECK` | `CHECK task-name` |
| `UNCHECK` | `UNCHECK task-name` |
| `ADD` | `ADD v0.1.0 "New task"` |
| `ADD AFTER` | `ADD v0.1.0 "Task" AFTER other-task` |
| `DELETE` | `DELETE old-task` |
| `UPDATE` | `UPDATE task "New description"` |
| `NOTE` | `NOTE task "Implementation note"` |

Tasks are identified by slugified names. `Truncation detection` becomes `truncation-detection`.

## Coming Soon: The Contract Protocol

AI will declare intent before writing code. Warden verifies the output matches.

    ∇∇∇ CONTRACT ∇∇∇
    GOAL: Refactor parser for clarity
    
    REFACTOR FN src/parser.rs:parse_header
        ASSERT complexity <= 4
        ASSERT depth <= 1
    
    CREATE STRUCT src/types.rs:Header
        ASSERT public == true
    
    UPDATE FILE src/lib.rs
        ASSERT tokens < 2000
    ∆∆∆

If AI hallucinates complex code or touches undeclared files, the contract fails. No human judgment needed—pass/fail.

---

## Why These Rules?

For humans, complexity limits are debatable style choices.

For AI, they're **containment protocols**:

| Metric | Human Value | AI Value |
|--------|-------------|----------|
| Cyclomatic Complexity | Debatable | **Critical** - bounds hallucination surface |
| Nesting Depth | Readability | **Critical** - AI loses scope tracking |
| Function Length | Preference | **Critical** - attention degrades |
| File Size | Organization | **Critical** - context economics |

We don't enforce low complexity because it makes "better" code. We enforce it because it makes **verifiable** code.

---

## Languages

Currently supported:
- **Rust** - Full analysis (complexity, nesting, arity, safety)
- **TypeScript/JavaScript** - Full analysis
- **Python** - Full analysis

Coming soon: Go, C/C++, Java/Kotlin

---

## Philosophy

Warden exists because AI-assisted development needs **constraints**, not suggestions.

The original Power of 10 rules were for life-critical systems where bugs kill people. We're not building flight software—but we are building systems that must remain comprehensible through hundreds of AI-human iterations.

**Principles:**

1. **Reject bad input, don't fix it** — Warden is a gatekeeper, not a fixer.

2. **Git is the undo system** — Don't reinvent version control.

3. **Explicit > Magic** — If AI doesn't follow format, fail loudly.

4. **Containment over craftsmanship** — For AI, constraints aren't style. They're safety.

5. **Eat your own dogfood** — Warden enforces its own rules on its own codebase.

---

## Self-Hosting

Warden is self-hosting. Run `warden` in this repo:

    ✅ All Clear. Scanned 24011 tokens in 133ms.

The tool passes its own rules.

---

## License

MIT

---

## Links

- [Roadmap](roadmap.md)
- [NASA Power of 10 Rules](https://en.wikipedia.org/wiki/The_Power_of_10:_Rules_for_Developing_Safety-Critical_Code)

---

*Complexity is the enemy. Warden is the checkpoint.*
