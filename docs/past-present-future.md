# Past / Present / Future

**Status:** Canonical (living snapshot)  
**Last updated:** 2026-01-03 (v1.3.4)  
**Canonical policy:** This document states the current operational reality and the single next action.

---

## 1) Past (What changed recently)

**v1.3.4: UX Polish & Configuration.**
- **`slopchop config`**: Interactive TUI editor for settings.
- **Paste-Back Restoration**: Verification failures now auto-copy structured AI feedback.
- **Pack Fix**: Fixed `slopchop pack` ignoring auto-copy preferences.

**v1.3.3: Cross-Platform Patch Reliability.**
- Fixed critical CRLF hash flip-flopping bug on Windows.
- Fixed multi-patch verification logic.
- Consolidated `compute_sha256()` with normalization.

**v1.3.2: Patch Security & Stress Test Hardening.**
- Fixed critical vulnerabilities: S03 (Null Byte in Path) and I01 (Sigil Injection).
- Verified semantic matcher robustness.

**v1.3.0: Locality v2 & Consolidation.**
- **Locality v2:** Cycle detection, auto-hub detection, and layer inference.
- Deleted ~2000 lines of unused code (TUI, trace).
- Modularized analysis checks.

---

## 2) Present (Where we are right now)

**Status:** STABLE - v1.3.4 Released

SlopChop is now feature-complete for the v1.3 cycle. The patching workflow is robust, configuration is interactive, and the "Paste-Back" loop is restored and verified.

### Core Commands

| Command | Purpose |
|---------|---------|
| `scan` | Structural violation detection |
| `check` | Gate (external commands + scan) |
| `apply` | Staged ingestion with XSC7XSC protocol |
| `pack` | AI context generation |
| `config` | Interactive settings editor |
| `clean` | Remove artifacts |

### Experimental Commands

| Command | Purpose | Notes |
|---------|---------|-------|
| `scan --locality` | Topological integrity scanning | Works but has false positives |
| `audit` | Code duplication detection | |
| `map` | Repository visualization | |
| `signatures` | Type-surface maps for AI | |

### Known Issues

1. **Locality False Positives**: The directional coupling heuristic can sometimes flag legitimate dependency cycles in tightly coupled modules (e.g., `parser` <-> `ast`).

---

## 3) Future (What we do next)

### v1.4.0: Distribution & Ecology (Next Cycle)

| Feature | Description | Priority |
|---------|-------------|----------|
| **Installers** | Scoop (Windows), Homebrew (macOS), Shell script (Linux) | High |
| **Locality Validation** | Run on 3-5 external Rust repos to battle-test heuristics | High |
| **TypeScript Imports** | Better path alias and index file resolution | Medium |
| **`mode = "error"`** | Switch locality to blocking mode default once validated | Medium |

### Long Term

| Feature | Description |
|---------|-------------|
| **LSP Server** | Real-time "Law" violation flagging in IDE |
| **Pre-commit Hook** | Lightweight hook integration |

---

## 4) Non-Goals (What we are NOT doing)

These were considered and deliberately rejected:

| Feature | Reason |
|---------|--------|
| **History/generations** | Stage is ephemeral by design. Users wipe it immediately after promote. |
| **75% PATCH threshold** | Micromanaging. AI can decide when to use PATCH vs FILE. |
| **META block** | Redundant. BASE_SHA256 per-patch matches context requirements. |
| **Python support (Deep)** | Basic parsing exists, but full type inference is out of scope. |
| **Advanced visualization** | Dashboard was bloat. Deleted. |

---

## 5) Architecture Notes

### The Config Command

The new `config` command uses a minimal `crossterm` implementation (~250 lines) instead of the heavy `ratatui` dependency used in previous versions. This aligns with the "Complexity <= 8" law by keeping the UI logic linear and shallow.