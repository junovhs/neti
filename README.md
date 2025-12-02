# Warden

**A code quality gatekeeper for AI-assisted development**

Warden creates a feedback loop between your codebase and AI coding assistants. It packages your code with configurable quality constraints, validates AI responses, and only commits changes that pass your rules.

Instead of manually reviewing every AI-generated file, Warden automatically rejects malformed output and asks the AI to try again. When everything passes, it commits and pushes. You stay in flow.

```
warden pack → AI → warden apply → ✅ Pass → commit
                        ↓
                   ❌ Fail → rejection copied → paste back to AI → retry
```

## Current Status

**Stable (v0.7.x):** The Three Laws, Nabla protocol, apply/reject loop, and roadmap system are fully implemented and tested.

**In Development (v0.8+):** Smart Context features (dependency graphs, error-driven packing, cluster isolation) are designed but not yet implemented. See [ROADMAP.md](ROADMAP.md).

For architecture, protocol specs, and design philosophy, see [DESIGN.md](DESIGN.md).

---

## Installation

```bash
git clone https://github.com/yourusername/warden.git
cd warden
cargo install --path .
```

Verify:
```bash
warden --version
```

### Clipboard Dependencies

Warden uses system clipboard for seamless copy/paste:

| Platform | Utility | Install |
|----------|---------|---------|
| macOS | `pbcopy` / `pbpaste` | Built-in |
| Linux (X11) | `xclip` | `apt install xclip` |
| Linux (Wayland) | `wl-copy` / `wl-paste` | `apt install wl-clipboard` |
| Windows | PowerShell | Built-in |

If clipboard isn't available, Warden displays file paths for manual copy.

---

## Quick Start

### 1. Initialize

```bash
cd your-project
warden --init
```

This creates `warden.toml` with sensible defaults for your project type.

### 2. Scan

```bash
warden
```

Reports any violations of the configured rules (file size, complexity, nesting, etc.).

### 3. Pack for AI

```bash
warden pack
```

Generates `context.txt` with your codebase and copies the file path to clipboard. Attach it to your AI conversation.

### 4. Apply AI Response

Copy the AI's response (in Nabla format), then:

```bash
warden apply
```

Warden validates the response, writes files, runs your checks, and commits if everything passes.

---

## The AI Loop (Example)

Here's how a typical AI coding session works with Warden:

**Step 1:** Pack your codebase
```bash
warden pack
# → Generates context.txt, copies path to clipboard
# → Attach to your AI conversation
```

**Step 2:** AI responds in Nabla format

```
∇∇∇ PLAN ∇∇∇
GOAL: Add input validation to login handler
CHANGES:
1. Create validation module
2. Add email format check
3. Add password strength check
∆∆∆

∇∇∇ MANIFEST ∇∇∇
src/auth/validate.rs [NEW]
src/auth/mod.rs
∆∆∆

∇∇∇ src/auth/validate.rs ∇∇∇
use regex::Regex;

pub fn validate_email(email: &str) -> Result<(), &'static str> {
    let re = Regex::new(r"^[^@]+@[^@]+\.[^@]+$").unwrap();
    if re.is_match(email) {
        Ok(())
    } else {
        Err("Invalid email format")
    }
}

pub fn validate_password(password: &str) -> Result<(), &'static str> {
    if password.len() >= 8 {
        Ok(())
    } else {
        Err("Password must be at least 8 characters")
    }
}
∆∆∆

∇∇∇ src/auth/mod.rs ∇∇∇
pub mod validate;
pub mod login;
∆∆∆
```

**Step 3:** Apply the response

```bash
warden apply
```

**If validation passes:** Files are written, tests run, changes committed and pushed.

**If validation fails:** Warden copies a rejection message to your clipboard:

```
The previous output was rejected by the Warden Protocol.

VALIDATION ERRORS:
- src/auth/validate.rs: Banned: '.unwrap()'. Use '?' or 'unwrap_or'.

Please provide corrected files using the NABLA PROTOCOL (∇∇∇ ... ∆∆∆).
```

**Step 4:** Paste the rejection back to the AI. It will fix the issue and retry.

The AI learns through rejection. After a few cycles, it internalizes the constraints.

---

## Configuration

Warden is configured via `warden.toml`:

```toml
[rules]
max_file_tokens = 2000              # Files must be reviewable
max_cyclomatic_complexity = 8       # Functions must be testable
max_nesting_depth = 3               # Logic must be followable
max_function_args = 5               # Functions must be focused

# Skip rules for certain files
ignore_tokens_on = [".md", ".lock", ".json"]
ignore_naming_on = ["tests", "spec"]

[commands]
check = [
    "cargo clippy --all-targets -- -D warnings",
    "cargo test"
]
fix = "cargo fmt"
```

### Language-Specific Examples

**Rust:**
```toml
[commands]
check = ["cargo clippy --all-targets -- -D warnings", "cargo test"]
fix = "cargo fmt"
```

**Node/TypeScript:**
```toml
[commands]
check = ["npm test", "npx eslint src/"]
fix = "npx prettier --write src/"
```

**Python:**
```toml
[commands]
check = ["pytest", "ruff check ."]
fix = "ruff check --fix ."
```

### Ignoring Files

**Project-wide:** Create `.wardenignore` (same syntax as `.gitignore`)
```
target/
node_modules/
*.generated.ts
```

**Per-file:** Add a comment at the top of the file
```rust
// warden:ignore
```
```python
# warden:ignore
```

---

## Commands Reference

| Command | Description |
|---------|-------------|
| `warden` | Scan codebase and report violations |
| `warden --init` | Launch configuration wizard |
| `warden --ui` | Interactive TUI dashboard |
| `warden pack` | Generate `context.txt` for AI |
| `warden pack --copy` | Copy content directly to clipboard |
| `warden pack --skeleton` | Compress all files to signatures only |
| `warden pack <file>` | Focus on one file, skeleton the rest |
| `warden apply` | Validate and apply AI response from clipboard |
| `warden check` | Run configured test/lint commands |
| `warden fix` | Run configured fix commands |
| `warden prompt` | Output the system prompt |

### Roadmap Commands

| Command | Description |
|---------|-------------|
| `warden roadmap init` | Create ROADMAP.md template |
| `warden roadmap show` | Display roadmap as tree |
| `warden roadmap tasks` | List all tasks |
| `warden roadmap tasks --pending` | List incomplete tasks |
| `warden roadmap audit` | Verify test anchors exist |
| `warden roadmap audit --strict` | Fail if any anchor is broken |

---

## Roadmap & Test Traceability

Warden includes a roadmap system that ties features to tests:

```markdown
- [x] **Token counting** <!-- test: tests/unit_tokens.rs::test_count_basic -->
- [ ] **Cluster packing**
```

The `<!-- test: ... -->` anchor creates a contract: "done" means "tested."

```bash
warden roadmap audit --strict   # Verify all completed tasks have passing tests
```

For maximum rigor, add audit to your check pipeline:

```toml
[commands]
check = [
    "cargo test",
    "warden roadmap audit --strict"
]
```

See [ROADMAP.md](ROADMAP.md) for the full feature contract.

---

## Language Support

| Language | Complexity Analysis | Skeleton Mode | Notes |
|----------|:-------------------:|:-------------:|-------|
| Rust | ✅ | ✅ | + `.unwrap()` detection |
| TypeScript | ✅ | ✅ | |
| JavaScript | ✅ | ✅ | |
| Python | ✅ | ✅ | |
| Go | — | — | Token counting only |
| Other | — | — | Token counting only |

---

## Adoption Tiers

Warden can be adopted incrementally:

### Tier 1: Structural Linting
Use Warden as a code quality scanner, no AI integration required.

```bash
warden              # Scan for violations
warden check        # Run tests/linters
```

### Tier 2: AI-Assisted Development
Add the pack/apply loop for AI coding sessions.

```bash
warden pack         # Generate context for AI
warden apply        # Validate and apply AI responses
```

### Tier 3: Full Traceability
Add roadmap management with audit enforcement.

```bash
warden roadmap audit --strict   # Every feature must have a test
```

Start at Tier 1, graduate upward as trust in the workflow grows.

---

## What's Coming

**v0.8 — Dependency Graph:** Import extraction, cluster detection, understanding which files relate to each other.

**v0.9 — Smart Context:** Error-driven packing (parse compiler errors → pack relevant files), cluster packing (`--cluster apply-system`), trace packing (`--trace src/mod.rs --depth 2`).

**v0.10+ — Validation Hardening:** Markdown fence rejection, brace balancing, stricter content validation.

See [ROADMAP.md](ROADMAP.md) for the complete plan.

---

## License

MIT — See [LICENSE](LICENSE)
