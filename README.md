# Neti

**The gatekeeper. Code that doesn't pass Neti doesn't enter your codebase.**

In Sumerian mythology, Neti was the guardian of the underworld gate — the one who decided what passed and what was turned back. That's exactly what this tool does.

Neti is a **structural governance and code quality enforcement engine** built for the era of AI-assisted development. It sits between AI agents (or developers) and your codebase, running a comprehensive battery of static analysis, structural metrics, pattern detection, and your own custom verification commands. If the code doesn't pass, it doesn't merge. The AI tries again.

```
$ neti check

✖ error: Function 'parse_response' has 8 args (Max: 5)
  --> src/client.rs:42
  = LAW OF COMPLEXITY: Action required
    Group related parameters into a struct or options object

✖ error: File size is 2847 tokens (Limit: 2000)
  --> src/handlers/mod.rs:1
  = LAW OF ATOMICITY: Action required
    Split the file. Create submodules.

✖ error: MutexGuard held across `.await` point
  --> src/worker.rs:88
  = C03: Action required
    Drop the guard before the await, or use an async-aware lock.

FAILED — 3 violations. See neti-report.txt for full output.
```

---

## Three Ways to Use Neti

Neti is designed to fit into your workflow however you work with AI — from casual chat sessions to fully autonomous agents to production CI pipelines.

### 1. Chat Loop (Human in the Loop)

You're working with an AI assistant in a chat interface. The AI delivers code, you apply it, Neti checks it, you paste the report back. The AI fixes violations and redelivers. Repeat until green.

```bash
# AI delivers files → you apply them → you run:
neti check

# Paste neti-report.txt back to the AI
# AI fixes violations and redelivers
# Repeat until green
```

The `CHAT-PROTOCOL.md` in any Neti-governed repo contains the full protocol for this workflow, including how to structure AI instructions, how many files to deliver per batch, and how to converge on green in minimum turns.

The key insight: every failed check costs real money in API tokens. Neti's report format is designed to give the AI maximum signal in minimum tokens — structured, untruncated, prescriptive.

### 2. Autonomous Agent Loop (No Human Required)

An AI agent runs the full loop itself. It creates a sandbox branch, makes changes, runs `neti check`, reads the report, fixes violations, and only promotes to main when the gate is green. No human intervention until the task is complete.

```bash
# Agent runs this loop autonomously:
neti branch          # Create isolated sandbox
# ... agent makes changes ...
neti check           # Run the full gate
# ... agent reads neti-report.txt, fixes violations ...
neti promote         # Only runs when check is GREEN
git push
```

The `AGENT-README.md` in any Neti-governed repo contains the complete autonomous protocol — the exact loop, the laws, what constitutes dishonorable behavior (bypassing the sandbox, silencing violations with `#[allow(...)]`, promoting without a green check).

### 3. CI Pipeline (Always On)

Neti runs in your GitHub Actions, GitLab CI, or any other pipeline. Structural violations block merges the same way a failing test would. The gate never sleeps.

```yaml
# .github/workflows/neti.yml
- name: Install Neti
  run: cargo install neti

- name: Run Neti Check
  run: neti check
```

`neti check` exits with a non-zero code on any violation. It integrates cleanly with any CI system that respects exit codes.

---

## What Neti Actually Checks

This is not a simple linter. Neti runs a multi-layer analysis pipeline across every file in your codebase.

### Structural Limits (Configurable)

Hard limits that block merges when exceeded:

| Metric | What It Catches | Default |
| :--- | :--- | :--- |
| **File Token Count** | Files too large for AI or humans to reason about atomically | 2,000 tokens |
| **Cognitive Complexity** | Functions too complex to safely modify or understand | ≤ 25 |
| **Nesting Depth** | Logic nested so deep it becomes unreadable | ≤ 3 levels |
| **Function Arity** | Functions with too many parameters | ≤ 5 args |
| **Function Name Length** | Overly verbose or meaninglessly short names | ≤ 10 words |
| **LCOM4** | Structs/classes doing too many unrelated things — split them | = 1 |
| **CBO** | Modules coupled to too many others — reduce dependencies | ≤ 9 |
| **SFOUT** | Structural fan-out — god module detection | ≤ 7 |
| **AHF** | Attribute Hiding Factor — encapsulation discipline | ≥ 60% |

### Pattern Detection (AST-Level)

Neti parses your actual AST using Tree-sitter and detects specific anti-patterns by category:

**Concurrency**
- `C03` — `MutexGuard` held across an `.await` point (deadlock waiting to happen)
- `C04` — Undocumented synchronization primitives

**Security**
- `X01` — SQL injection surface patterns
- `X02` — Credential and secret exposure
- `X03` — Unsafe input handling

**Performance**
- `P01` — Unnecessary cloning in hot paths
- `P02` — Excessive allocation patterns
- `P04` — Inefficient iteration
- `P06` — String formatting anti-patterns

**Logic**
- `L02` — Boundary ambiguity (`<=`/`>=` with `.len()`, off-by-one surface)
- `L03` — Additional logic correctness patterns

**State**
- `S01`, `S02`, `S03` — State management violations and anti-patterns

**Resource**
- `R07` — Missing flush on buffered writers

**Semantic**
- `M03`, `M04`, `M05` — Semantic correctness patterns

**Idiomatic**
- `I01`, `I02` — Language-idiomatic patterns

**Database**
- `P03` — N+1 query patterns

**Safety**
- Unsafe blocks without `// SAFETY:` justification comments
- Optional: ban `unsafe` entirely

**Syntax**
- AST-level syntax error detection
- Missing or malformed nodes

**Naming**
- Function naming convention enforcement by language

### Deep Structural Analysis (Graph-Level)

For codebases above a minimum size threshold, Neti computes full dependency graph metrics:

- **Law of Locality** — Dependency distance analysis using Lowest Common Ancestor. Flags cross-boundary coupling that violates your intended layering. Configurable as warn or error.
- **Cycle Detection** — Finds circular dependencies in your module graph.
- **PageRank** — Identifies the most structurally critical files in the codebase. Useful for understanding what's load-bearing.
- **Hub Detection** — Finds modules with unusually high afferent coupling (everything depends on them).
- **God Module Detection** — Finds modules that do too much.
- **Layer Violation Detection** — Enforces that your dependency direction matches your intended architecture (e.g. `ui → domain → infra`, never `infra → domain`).
- **Coupling Entropy** — Measures overall topological health of the codebase.

### Your Own Commands

`neti check` also runs whatever you put in `[commands]` — clippy, your test suite, biome, ruff, go vet, anything. The output of all commands is captured and written to `neti-report.txt` alongside the structural analysis. One command, one report, one green/red answer.

```toml
[commands]
check = [
    "cargo clippy --all-targets -- -D warnings -W clippy::pedantic -W clippy::unwrap_used -W clippy::expect_used -W clippy::indexing_slicing -A clippy::struct_excessive_bools -A clippy::module_name_repetitions -A clippy::missing_errors_doc -A clippy::must_use_candidate",
    "cargo test",
]
```

---

## The Report File

Every `neti check` writes full results to `neti-report.txt`. This is not a log file — it's the **contract**.

- **Never truncated.** Terminal output gets cut off. The report does not.
- **No ANSI formatting.** Machine-readable by default.
- **Prescriptive.** Every violation includes what went wrong, where, why it matters, and what to do about it.
- **Complete.** Compiler errors, linter output, test failures, and structural violations — all in one place.

AI agents are instructed to read `neti-report.txt`, not stdout. This is intentional.

---

## Installation

```bash
cargo install neti
```

Or build from source:

```bash
git clone https://github.com/junovhs/neti
cd neti
cargo install --path .
```

---

## Configuration

Run `neti config` for an interactive TUI editor, or edit `neti.toml` directly:

```toml
[rules]
max_file_tokens = 2000
max_cognitive_complexity = 25
max_nesting_depth = 3
max_function_args = 5
max_function_words = 10
max_lcom4 = 1
min_ahf = 60.0
max_cbo = 9
max_sfout = 7

[rules.safety]
require_safety_comment = true
ban_unsafe = false

[rules.locality]
max_distance = 4
l1_threshold = 2
mode = "warn"   # or "error" to make locality violations block merges

[preferences]
auto_copy = true
progress_bars = true
backup_retention = 5

[commands]
check = [
    "cargo clippy --all-targets -- -D warnings -W clippy::pedantic -W clippy::unwrap_used -W clippy::expect_used -W clippy::indexing_slicing",
    "cargo test",
]
fix = ["cargo fmt"]
```

Neti auto-detects your project type (Rust, Node, Python, Go) and generates sensible defaults if no `neti.toml` exists.

---

## Language Support

| Language | Complexity | Naming | Pattern Detection | Structural Metrics |
| :--- | :--- | :--- | :--- | :--- |
| Rust | ✅ | ✅ | ✅ Full | ✅ Full |
| Python | ✅ | ✅ | Partial | — |
| TypeScript / JavaScript | ✅ | ✅ | Partial | — |

---

## What Neti Is Not

**Not a linter.** Clippy, ESLint, Ruff — Neti runs them as part of `check` but doesn't replace them. They handle language-specific lints better than any general tool could.

**Not a test framework.** Neti runs your tests. It doesn't write them.

**Not a formatter.** `neti fix` triggers your formatter. Neti doesn't format itself.

**Not a context packing tool.** Neti verifies AI output. Feeding context to AI is a different problem — see [SEMMAP](https://semmap.dev).

---

## Why This Exists

AI coding assistants are extraordinarily capable and extraordinarily bad at maintaining architectural discipline over time. They'll pass your unit tests while quietly making your codebase harder to change, harder to understand, and more fragile. They optimize for "works now" and produce the kind of code that looks fine in a single PR review but compounds into spaghetti across dozens of sessions.

Neti exists because the feedback loop that keeps human developers honest — code review, accumulated taste, architectural intuition — doesn't naturally exist for AI agents. So we make it mechanical. You define what "good" looks like. Neti enforces it. The AI can't ship slop if the gate won't open.

---

## License

MIT OR Apache-2.0

---

*Neti — a [SEMMAP Labs](https://semmap.dev) project*
