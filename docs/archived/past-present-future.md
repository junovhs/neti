# Past / Present / Future

**Status:** Canonical (living snapshot)  
**Last updated:** 2025-12-28  
**Canonical policy:** This document states the current operational reality and the single next action.

---

## 1) Past (What changed recently)

**v1.2.x: The Law of Locality was added.**

- Stability Classifier computing fan-in (Cₐ), fan-out (Cₑ), Instability (I), and Skew (K)
- Node identity classification (StableHub, VolatileLeaf, IsolatedDeadwood, GodModule)
- Universal Locality Algorithm validating dependency edges against distance thresholds
- CLI integration via `slopchop scan --locality`

**However:** The feature surface area grew beyond what's been validated in production use. A consolidation phase is now required.

---

## 2) Present (Where we are right now)

**Status:** CONSOLIDATION PHASE

### The Problem

SlopChop has accumulated features faster than they've been validated. The tool currently has:
- One active self-violation (`src/apply/parser.rs` exceeds 2000 tokens)
- Overlapping commands (`trace` duplicates `pack --focus`)
- Unused/untrusted features (`fix`, `dashboard`)
- A new feature (locality) that hasn't been battle-tested

A tool that enforces structural integrity must itself be structurally sound.

### Core Commands (Battle-Tested)

| Command | Purpose | Status |
|---------|---------|--------|
| `scan` | Structural violation detection | ✅ Stable |
| `check` | Gate (external commands + scan) | ✅ Stable |
| `apply` | Staged ingestion with XSC7XSC protocol | ✅ Stable |
| `pack` | AI context generation | ✅ Stable (needs trace absorption) |

### Experimental Commands (Keep but Mark)

| Command | Purpose | Status |
|---------|---------|--------|
| `scan --locality` | Topological integrity scanning | ⚠️ Advisory mode only |
| `signatures` | Type-surface maps for AI | ⚠️ Experimental |
| `map` | Repository visualization | ⚠️ Experimental |
| `audit` | Code duplication detection | ⚠️ Experimental |

### Commands to Remove

| Command | Reason |
|---------|--------|
| `trace` | Redundant with `pack --focus` |
| `fix` | Unused, modifies files unpredictably |
| `prompt` | Not core, can be done via `pack --prompt` |
| `dashboard` | Unused except for config; over-promised visualization |

### Current Violations

```
FILE: src/apply/parser.rs | LAW OF ATOMICITY | File size is 2018 tokens (Limit: 2000)
```

This must be fixed before any public release.

---

## 3) Future (What we do next)

**We are entering the Consolidation Era (v1.3.x).**

### Objectives

1. **Self-Compliance:** Fix all violations in SlopChop itself
2. **Surface Reduction:** Remove `trace`, `fix`, `prompt`, reduce `dashboard`
3. **Feature Absorption:** Merge useful `trace` concepts into `pack`
4. **Locality Stabilization:** Set `mode = "warn"` as default, validate thresholds
5. **Prescriptive Violations:** Make error messages actionable, not just descriptive
6. **TypeScript Hardening:** Improve TS import resolution for real-world projects

### Immediate Next Action

**Fix `src/apply/parser.rs` to be under 2000 tokens.**

This is symbolic and practical. SlopChop cannot ship while violating its own laws.

---

## 4) Non-Goals (What we are NOT doing)

- **Python support:** Not a real use case yet. Depth over breadth.
- **Test coverage enforcement:** Belongs in the separate Roadmap project.
- **Advanced visualization:** Dashboard dreams are deferred indefinitely.
- **Method B optimization:** Signatures/trace experiments are frozen, not expanded.
