# Past / Present / Future

**Status:** ACTIVE DEV - v1.4.0
**Last updated:** 2026-01-04
**Canonical policy:** This document states the current operational reality. All previous history is archived in `docs/archived/`.

---

## 1) Past (What changed recently)

**v1.3.5: Agentic Interfaces (Completed)**
- **JSON Output:** Added `--json` to `scan` and `check` for machine-readable verification.
- **Sabotage:** Added `slopchop sabotage <file>` to safe-test test suite robustness in stage.
- **Holographic Spec:** Added `--format spec` to `pack` to extract `///` comments into Markdown.

**v1.3.4: UX Polish (Completed)**
- **Config UI:** Added `slopchop config` interactive TUI.
- **Workflow:** Restored paste-back AI feedback and fixed `pack` auto-copy.

---

## 2) Present (Where we are right now)

**Status:** RE-ARCHITECTING WORKFLOW (v1.4.0)

We are moving away from "Clipboard Payloads" as the primary workflow for autonomous agents. We are establishing the **"Agent-in-Stage"** protocol where agents edit the sandbox directly.

### Operational State
- SlopChop passes all internal 3-Law checks.
- Machine-readable JSON output is stable for Roadmap integration.

---

## 3) Future (What we do next)

### v1.4.0: The Agentic Stage

| Feature | Description | Priority |
|---------|-------------|----------|
| **`slopchop stage`** | Constructor command to initialize/reset the sandbox. | High |
| **`apply --sync`** | Nuclear Promote: Mirror the stage to root and clear. | High |
| **Heuristic Nag** | In-terminal warnings in `check` for high-volume edits. | Medium |
| **Path Protection** | `slopchop.toml` rule to block edits to critical files. | Medium |

---

## 4) Non-Goals
- Complex 3-way merging (Use `--sync` + Git instead).
- Manual manifest writing in Agent mode.

---

## 5) Architecture Notes
### The Shadow Worktree
The stage folder `.slopchop/stage/worktree/` is now considered the **Primary Working Directory** for AI Agents. The root directory is the **Source of Truth** managed by the human via `--sync` and `git commit`.
