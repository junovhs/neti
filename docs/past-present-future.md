# Past / Present / Future

**Status:** STABLE - v1.5.0
**Last updated:** 2026-01-08

---

## 1) Past

**Branch Migration (v1.5.0)**

Replaced the custom staging directory with a lightweight git branch workflow (`slopchop-work`).
- Fixed UTF-8 panic in feedback generation.
- Removed ~1700 lines of legacy stage code.
- `apply` is now atomic and safe via git.

**Mutation Testing Incident (v1.4.0)**

Unsupervised `slopchop mutate` run corrupted codebase. Operators mutated, discovery broke, infinite loops introduced. Hard reset to bf46c8e.

**Root Cause:** File restoration is unreliable in both `slopchop mutate` and `cargo-mutants`.

**TypeScript Testing (2026-01-07)**

Exposed stage system flaws:
1. UTF-8 panic on Unicode tool output
2. Verification deadlock (can't fix files that get reverted)
3. Config catch-22 (can't fix config that validation depends on)

---

## 2) Present

**v1.5.0 — Stable & Clean**

### Works
- `scan`, `check`, `pack`, `map`
- `apply` (Branch-based)
- `mutate` (CLI integrated, experimental)

### Focus: Scan v2.0
Cyclomatic complexity is a weak predictor. We are moving to **Cognitive Complexity** and **AST Pattern Matching**.

See `docs/SCAN_V2_SPEC.md` for the full specification.

---

## 3) Future

### Work Order

| # | Task | Doc | Status |
|---|------|-----|--------|
| 1 | Scan v2.0 | `docs/SCAN_V2_SPEC.md` | **Next** |
| 2 | Mutate Safety | `docs/MUTATE_INTEGRATION.md` | Pending |

**Immediate Next Step:** Begin implementation of Scan v2.0 infrastructure (LCOM4, Cognitive Complexity).

---

## 4) Non-Goals

- Replacing the compiler
- Complex merging
- Working without git
- Being a linter (we're a constraint system)