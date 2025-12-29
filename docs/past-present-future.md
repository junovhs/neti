# Past / Present / Future

**Status:** Canonical (living snapshot)  
**Last updated:** 2025-12-28  
**Canonical policy:** This document states the current operational reality and the single next action.

---

## 1) Past (What changed recently)

**v1.2.x: The Law of Locality was added.**
- Stability Classifier computing fan-in, fan-out, Instability, and Skew
- Node identity classification (StableHub, VolatileLeaf, IsolatedDeadwood, GodModule)
- Universal Locality Algorithm validating dependency edges
- CLI integration via `slopchop scan --locality`

**v1.3.0: Consolidation Era completed.**
- Fixed self-violation: `src/apply/parser.rs` split into `parser.rs` + `blocks.rs`
- Removed 4 commands: `trace`, `fix`, `prompt`, `dashboard`
- Deleted ~2000 lines of unused code (`src/trace/`, `src/tui/`)
- Prescriptive violations: errors now include ANALYSIS and SUGGESTION sections
- Locality scan respects `mode = "warn"` (doesn't block verification)
- Modularized analysis checks into `checks/naming.rs`, `checks/complexity.rs`, `checks/banned.rs`

---

## 2) Present (Where we are right now)

**Status:** STABLE - Ready for Locality v2

SlopChop passes all its own checks. The codebase is clean and consolidated.

### Core Commands

| Command | Purpose |
|---------|---------|
| `scan` | Structural violation detection |
| `check` | Gate (external commands + scan) |
| `apply` | Staged ingestion with XSC7XSC protocol |
| `pack` | AI context generation |
| `clean` | Remove artifacts |

### Experimental Commands

| Command | Purpose | Notes |
|---------|---------|-------|
| `scan --locality` | Topological integrity scanning | Works but has false positives |
| `audit` | Code duplication detection | |
| `map` | Repository visualization | |
| `signatures` | Type-surface maps for AI | |

### Known Issue

The locality scanner requires manual hub config and produces false positives for legitimate layered architectures. This is addressed in the approved proposal.

---

## 3) Future (What we do next)

### Immediate Next Action

**Implement Locality v2 per `docs/locality-v2-proposal.md`**

| Phase | Description | Outcome |
|-------|-------------|---------|
| 0 | Cycle detection | Hard error on dependency cycles |
| 1 | Auto-hub detection | No manual hub config needed |
| 2 | Directional coupling | Only flag bidirectional, not layering |
| 3 | Layer inference | Auto-compute architectural layers |

**Success criteria:** `slopchop scan --locality` produces zero false positives with zero config.

**Estimated effort:** 6-10 hours total

### After Locality v2

- Validate on external Rust projects
- Consider `mode = "error"` as default once battle-tested
- TypeScript import resolution improvements (if needed)

---

## 4) Non-Goals (What we are NOT doing)

- **Python support:** Not a real use case yet
- **Test coverage enforcement:** Separate tooling
- **Advanced visualization:** Dashboard is dead
- **Method B optimization:** Signatures/map experiments frozen