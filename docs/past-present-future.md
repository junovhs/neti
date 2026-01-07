# Past / Present / Future

**Status:** STABLE - v1.4.0  
**Last updated:** 2026-01-07  
**Canonical policy:** This document states the current operational reality. All previous history is archived in `docs/archived/`.

---

## 1) Past (What changed recently)

**v1.4.1: Mutation Testing Incident (Rolled Back)**

An unsupervised overnight run of `cargo-mutants` corrupted the codebase:
- Multiple files had operators mutated (`||` → `&&`, `>=` → `<`, etc.)
- Discovery broke completely (returned 0 files)
- One mutation caused infinite loop in tests
- Corrupted state was accidentally committed to main

**Resolution:** Hard reset to bf46c8e, force pushed. Lessons documented in `docs/MUTATION_TESTING_PLAN.md`.

**Key Lesson:** Never run mutation testing unsupervised. Always verify with `git diff` before committing.

**v1.4.0: Agentic Stage & Nuclear Sync (Completed)**
- **Stage Management:** Added `slopchop stage` for explicit sandbox initialization.
- **Nuclear Sync:** Added `apply --sync` for direct-to-root mirroring.
- **Heuristic Nag:** Added passive advisory for high edit volume.

**v1.3.5: Agentic Interfaces (Completed)**
- **JSON Output:** Added `--json` to `scan` and `check` for machine-readable verification.
- **Sabotage:** Added `slopchop sabotage <file>` to safe-test test suite robustness in stage.
- **Holographic Spec:** Added `--format spec` to `pack` to extract `///` comments into Markdown.

**v1.3.4: UX Polish (Completed)**
- **Config UI:** Added `slopchop config` interactive TUI.
- **Workflow:** Restored paste-back AI feedback and fixed `pack` auto-copy.

---

## 2) Present (Where we are right now)

**Status:** STABLE (v1.4.0) but with known issues

The custom stage system works but has proven problematic:
- Discovery broke when running from stage directory
- Antigravity couldn't access stage (blocked by .gitignore)
- ~1700 lines of code duplicating what git already does
- Custom bugs to maintain

### Real-World Testing: TypeScript/Windows (2026-01-07)

Testing on a TypeScript project with Biome exposed critical flaws:

**1. Panic Bug:** `verification.rs` slices UTF-8 strings by byte index. Biome's Unicode box characters (`━`) caused: `byte index 1000 is not a char boundary`

**2. Verification Deadlock:** Stage is binary (all checks pass OR revert). TypeScript linters fail on formatting, but you can't run `biome format --write` because the file was reverted. Infinite loop.

**3. Config Catch-22:** Can't fix `biome.json` because fixing it requires verification to pass, which requires valid `biome.json`.

**Root Cause:** Stage model assumes all checks are equal. In reality:
- **Rust:** `cargo check` ignores whitespace
- **TypeScript:** `biome check` fails on line endings, import order

Git branches fix this because files persist even when checks fail. You can iterate.

### Operational State
- SlopChop passes all internal 3-Law checks.
- Core functionality (scan, check, pack, map) is solid.
- Stage is fundamentally flawed for TypeScript workflows.

---

## 3) Future (What we do next)

### v1.4.1: Hotfix (Immediate)

| Fix | Description |
|-----|-------------|
| **UTF-8 Panic** | Use `floor_char_boundary()` in verification.rs string truncation |

### v1.5.0: Git Branch Migration (Planned)

**The Problem:** The custom stage system is 1700 lines of code that duplicates git branch functionality, with more bugs and less compatibility. The transactional model is too rigid for TypeScript linters.

**The Solution:** Replace `src/stage/` with `src/branch.rs` (~150 lines) wrapping git commands.

| Delete (~1700 lines) | Replace with |
|---------------------|--------------|
| `src/stage/copy.rs` | — |
| `src/stage/manager.rs` | — |
| `src/stage/promote.rs` | — |
| `src/stage/state.rs` | — |
| `src/stage/sync.rs` | — |
| `src/stage/mod.rs` | `src/branch.rs` (~150 lines) |

**New Commands:**

| Old Command | New Command | What it does |
|-------------|-------------|--------------|
| `slopchop stage --force` | `slopchop branch` | `git checkout -b slopchop-work` |
| `slopchop apply --sync` | `slopchop promote` | Merge work branch to main |
| `slopchop apply --reset` | `slopchop abort` | Delete work branch, return to main |

**What stays the same:**
- `slopchop check` — runs on current branch
- `slopchop scan`, `pack`, `map` — unchanged
- Advisory nag — reads from `git status` instead of `state.json`

**Benefits:**
- -1550 lines of code
- No custom bugs (git is battle-tested)
- Works with all tools (Antigravity, Claude Code, etc.)
- Rollback is just `git checkout .`

### v1.5.x: Mutation Testing (After Branch Migration)

| Feature | Priority |
|---------|----------|
| Wire up `src/mutate/` to CLI | High |
| Per-mutation timeout (30s) | High |
| `--fail-fast` flag | High |
| Git stash/restore safety | High |
| Return value mutations | Medium |

**Safety Protocol:** Mutation testing must run on a branch, never main.

### v1.6.0: Configurable Check Severity (Future)

Allow `slopchop.toml` to classify checks as blockers vs warnings:

```toml
[commands]
check = [
    { cmd = "cargo clippy", severity = "error" },
    { cmd = "biome check", severity = "warn" },
]
```

- **error:** Fails `slopchop check`, blocks promote
- **warn:** Prints warning, allows promote

This lets formatting failures be fixed iteratively without blocking the workflow.

---

## 4) Non-Goals

- Complex 3-way merging (use git directly)
- Working without git (git is infrastructure, not a dependency)
- Parallel mutation testing (requires workspace copies, not worth complexity)

---

## 5) Architecture Notes

### Git-Based Workflow (v1.5.0+)

```
main (source of truth)
  │
  └── slopchop-work (AI workspace)
        │
        ├── AI edits files
        ├── slopchop check
        └── slopchop promote → merges to main
```

### Why Git, Not GitHub

Git is:
- Open source, local-first
- Installed everywhere
- Not going anywhere
- Infrastructure, not a third-party service

GitHub is a service built on git. We depend on git (the tool), not GitHub (the company).
