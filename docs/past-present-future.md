# Past / Present / Future

**Status:** STABLE - v1.4.0  
**Last updated:** 2026-01-08  

---

## 1) Past

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

**v1.4.0 — Functional with critical issues**

### Works
- `scan`, `check`, `pack`, `map`
- 3 Laws enforcement

### Broken

| Issue | Severity |
|-------|----------|
| UTF-8 panic | Critical |
| Mutate file restoration | Critical |
| Stage system | Architectural |

### Weak

`scan` uses cyclomatic complexity (poor defect predictor) and misses entire bug categories.

---

## 3) Future

### Work Order

| # | Task | Doc |
|---|------|-----|
| 1 | UTF-8 panic fix | `docs/utf8-panic.md` |
| 2 | Branch migration | `docs/branch-migration.md` |
| 3 | Scan v2.0 | `docs/SCAN_V2_SPEC.md` |
| 4 | Mutation testing fixed | `docs/MUTATE_INTEGRATION.md` |

One at a time. Full attention on each.

---

## 4) Non-Goals

- Replacing the compiler
- Complex merging
- Working without git
- Being a linter (we're a constraint system)
