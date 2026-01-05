# Past / Present / Future

**Status:** ACTIVE DEV - v1.3.5
**Last updated:** 2026-01-04 (v1.3.5)
**Canonical policy:** This document states the current operational reality and the single next action.

---

## 1) Past (What changed recently)

**v1.3.4: UX Polish & Config.**
- **Interactive Config:** `slopchop config` TUI using crossterm.
- **Workflow Repairs:** Fixed `pack` auto-copy, restored paste-back packet.
- **Maintenance:** Removed `ratatui` dependency, wired `auto_promote` preference.

**v1.3.3: Cross-Platform Reliability.**
- Unified hash normalization (CRLF fix).
- Multi-patch verification stability.

---

## 2) Present (Where we are right now)

**Status:** ACTIVE DEVELOPMENT (v1.3.5)

We are building the "Agentic Interfaces" to allow SlopChop to integrate with Roadmap and autonomous agents.

### Core Commands

| Command | Purpose | Status |
|---------|---------|--------|
| `scan` | Structural violation detection | Stable |
| `check` | Gate (external commands + scan) | Stable |
| `apply` | Staged ingestion | Stable |
| `config` | Interactive settings | Stable |
| `sabotage` | Mutation testing generator | **Planned (v1.3.5)** |

### Known Issues
- None blocking.

---

## 3) Future (What we do next)

### v1.3.5: Agentic Interfaces & Trust Engine

| Feature | Description | Priority |
|---------|-------------|----------|
| **JSON Output** | `--json` for `check`/`scan`. Critical for Roadmap integration. | High |
| **Sabotage** | `slopchop sabotage <file>`. Breaks code in stage to verify tests. | High |
| **Holographic Spec** | `pack --spec`. Extracts doc comments into architectural markdown. | Medium |

### v1.4.0: Distribution
- Installers (Scoop, Homebrew).

---

## 4) Non-Goals
- **Python Support:** Frozen.
- **Advanced Viz:** Dashboard deleted.
- **Git Hooks:** Not core.

---

## 5) Architecture Notes

### The Trust Engine (v1.3.5+)
We are introducing `slopchop sabotage` to solve the "Fox Guarding the Henhouse" problem with AI-generated tests. By automating the breakage of code within the safety of the Staged Workspace, we can mathematically prove if a test suite is valid or vacuous.
