# SlopChop Scan v2.0 Specification

**Status:** Implementation In Progress
**Date:** 2026-01-09
**Philosophy:** Do the hard thing first. No shortcuts. Real solutions.

---

## What We're Replacing

1.  **Legacy `slopchop scan` checks:**
    - File tokens (Atomicity Law)
    - Cyclomatic complexity (Replacing with Cognitive)
    - Nesting depth
    - Function arguments
    - `.unwrap()` / `.expect()` (Paranoia Law)

2.  **Legacy `slopchop audit` pattern registry:**
    - We are deprecating the separate pattern engine in `audit` and moving high-signal idiom detection into `Scan v2`.

**Problem:** Cyclomatic complexity is a weak predictor. Research shows Cognitive Complexity correlates more strongly with defect density. Furthermore, "Audit" patterns belonged in the safety/linting pipeline (Scan), not the deduplication pipeline.

---

## The Bug Categories

| Category | Current Coverage | Target |
|----------|------------------|--------|
| Syntax | N/A (compiler) | N/A |
| Type | N/A (compiler) | N/A |
| Logic | ❌ None | Pattern flags + mutation testing |
| Off-by-one | ❌ None | Boundary pattern detection |
| State | ✅ Partial | Metrics + AST patterns |
| Concurrency | ✅ Partial | Async race patterns |
| Resource | ❌ None | Leak patterns |
| Security | ❌ None | Injection + secrets |
| Performance | ❌ None | Loop anti-patterns |
| Semantic | ❌ None | Name/behavior alignment |
| Idiomatic | ❌ None | Rust-specific best practices (migrated from Audit) |

---

## Metrics

These are computed values with thresholds.

| Metric | Threshold | Category | Notes |
|--------|-----------|----------|-------|
| **File Tokens** | > 2000 | Atomicity | [DONE] Keep |
| **Cognitive Complexity** | > 15 | Complexity | [DONE] Replaces cyclomatic |
| **Nesting Depth** | > 3 | Complexity | [DONE] Keep |
| **Function Args** | > 5 | Complexity | [DONE] Keep |
| **LCOM4** | > 1 | State | [DONE] Class should be split |
| **AHF** | < 60% | State | [DONE] State is leaking |
| **CBO** | > 9 | State | [DONE] Defect predictor |
| **SFOUT** | > 7 | Performance | [DONE] Architectural bottleneck |

---

## AST Patterns

Boolean checks. Either the pattern exists or it doesn't.

### State

| ID | Pattern | Rust | TypeScript |
|----|---------|------|------------|
| S01 | Global mutable declaration | `static mut` | Top-level `let` |
| S02 | Exported mutable | `pub static` (non-const) | `export let` |
| S03 | Suspicious global container | `lazy_static<Mutex<Vec/HashMap>>` | `const arr = []` at module scope |
| S04 | Impure function | Reads identifier not in params/locals | Same |
| S05 | Deep mutation of param | N/A (borrow checker) | `param.prop = x` |

### Concurrency

| ID | Pattern | Rust | TypeScript |
|----|---------|------|------------|
| C01 | Async race gap | N/A | `read(v) → await → write(v)` |
| C02 | Floating promise | N/A | Async call without await/assignment |
| C03 | Lock across await | `MutexGuard` live across `.await` | N/A |
| C04 | Undocumented sync primitive | `Arc<Mutex<T>>` without comment | N/A |
| C05 | Check-then-act | N/A | `if(v) { await; mutate(v) }` |

### Resource

| ID | Pattern | Rust | TypeScript |
|----|---------|------|------------|
| R01 | Unbalanced listener | N/A | `addEventListener` without `removeEventListener` |
| R02 | Floating subscription | N/A | `.subscribe()` without `takeUntil`/stored handle |
| R03 | Allocation in loop | `Vec::new()` / `String::new()` in loop | `new` / `fetch` in loop |
| R04 | Clone in loop | `.clone()` in `for` | N/A |
| R05 | Spread in loop | N/A | `{...obj}` or `[...arr]` in loop |
| R06 | Unbounded growth | `.push()` in loop, no `.clear()` | Same |
| R07 | Missing flush | `BufWriter` without `.flush()` | N/A |

### Security

| ID | Pattern | Rust | TypeScript |
|----|---------|------|------------|
| X01 | SQL concatenation | `format!("SELECT...{}", var)` | Template literal + SELECT + var |
| X02 | Command injection | `Command::new().arg(user_var)` | `exec(var)` / `spawn(var)` |
| X03 | Hardcoded secret | `const.*KEY.*=.*[A-Za-z0-9]{20,}` | Same |
| X04 | Unsafe parse | N/A | `JSON.parse()` without try/catch |
| X05 | Prototype pollution | N/A | `obj[userInput] = value` |

### Performance

| ID | Pattern | Rust | TypeScript |
|----|---------|------|------------|
| P01 | Clone in loop | `.clone()` in iteration | N/A |
| P02 | Allocation in loop | `Vec::new()` in loop | `new Object()` in loop |
| P03 | N+1 query | DB call in loop with iterator param | Same |
| P04 | Nested iteration | `for x { for y { if x == y } }` | Same |
| P05 | Repeated linear search | Multiple `.find()` same collection | Same |
| P06 | Linear search in loop | `.contains()` in loop | `.includes()` in loop |
| P07 | String abuse | `.to_string()` on literal/primitive | N/A |

### Semantic

| ID | Pattern | Rust | TypeScript |
|----|---------|------|------------|
| M01 | Missing doc on public | `pub fn` without `///` | `export function` without JSDoc |
| M02 | Dead parameter | Param never read in body | Same |
| M03 | Getter with mutation | `get_*` / `is_*` that writes | Same |
| M04 | Name/return mismatch | `is_*` returns non-bool | Same |
| M05 | Side-effecting calculation | `calculate_*` / `compute_*` that writes state | Same |

### Idiomatic (Merged from Audit)

| ID | Pattern | Rust | TypeScript |
|----|---------|------|------------|
| I01 | Manual From impl | `impl From` manually (use `thiserror`/`derive`) | N/A |
| I02 | Match duplication | Identical bodies in `match` arms | Same |
| I03 | If-Let Pattern | `if let Some(x) = y` (use `map`/`?` if simple) | N/A |
| I04 | Manual Display | `impl Display` manually (use `thiserror`/`derive`) | N/A |

### Logic & Off-by-One

| ID | Pattern | Rust | TypeScript |
|----|---------|------|------------|
| L01 | Untested public | `pub fn` without `#[test]` nearby | `export` without `.test.ts` |
| L02 | Boundary ambiguity | `..` vs `..=` near loop | `<` vs `<=` with `.length` |
| L03 | Unchecked first access | `.first()` / `[0]` without len check | Same |

---

## Configuration

```toml
[scan]
# Metrics
file_tokens_max = 2000
cognitive_complexity_max = 15
nesting_depth_max = 3
function_args_max = 5
lcom4_max = 1
ahf_min = 60
cbo_max = 9
sfout_max = 7

# Enable/disable categories
[scan.categories]
state = true
concurrency = true
resource = true
security = true
performance = true
semantic = true
logic = true
idiomatic = true

# Per-pattern overrides
[scan.patterns]
S01 = "error"    # Global mutable
S02 = "warn"     # Exported mutable
C02 = "error"    # Floating promise
I01 = "warn"     # Manual From impl
```

---

## Research References

All patterns and thresholds backed by:
- SonarSource Cognitive Complexity paper
- Chidamber & Kemerer OO metrics (CBO, LCOM)
- MOOD suite (AHF)
- NodeRacer (async races)
- JSWhiz (listener leaks)
- TruffleHog (secrets)
- CodeQL (injection patterns)
- See `/docs/research/` for full reports

---

## Notes

- Implementation will be incremental, one pattern at a time
- Each pattern gets full attention, tested thoroughly before moving on
- No "good enough" implementations — each check should be as correct as possible
- Cross-file analysis infrastructure built first, then individual checks leverage it
- Tree-sitter is the foundation for all AST work
