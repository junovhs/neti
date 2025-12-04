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

**Stable (v0.7.x):** The Three Laws, Warden Protocol, apply/reject loop, and roadmap system are fully implemented and tested.

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

Copy the AI's response (in Warden Protocol format), then:

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

**Step 2:** AI responds in Warden Protocol format

```
#__WARDEN_PLAN__#
GOAL: Add input validation to login handler
CHANGES:
1. Create validation module
2. Add email format check
3. Add password strength check