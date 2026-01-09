# SlopChop Scan v2.0 Specification

**Status:** Planning  
**Date:** 2026-01-08  
**Philosophy:** Do the hard thing first. No shortcuts. Real solutions.

---

## What We're Replacing

Current `slopchop scan` checks:
- File tokens (Atomicity Law)
- Cyclomatic complexity
- Nesting depth
- Function arguments
- `.unwrap()` / `.expect()` (Paranoia Law)

**Problem:** Cyclomatic complexity is a weak predictor. Research shows Cognitive Complexity correlates more strongly with defect density and developer comprehension time. We're also missing entire bug categories.

---

## The Bug Categories

| Category | Current Coverage | Target |
|----------|------------------|--------|
| Syntax | N/A (compiler) | N/A |
| Type | N/A (compiler) | N/A |
| Logic | ❌ None | Pattern flags + mutation testing |
| Off-by-one | ❌ None | Boundary pattern detection |
| State | ❌ None | Metrics + AST patterns |
| Concurrency | ❌ None | Async race patterns |
| Resource | ❌ None | Leak patterns |
| Security | ❌ None | Injection + secrets |
| Performance | ❌ None | Loop anti-patterns |
| Semantic | ❌ None | Name/behavior alignment |

---

## Metrics

These are computed values with thresholds.

| Metric | Threshold | Category | Notes |
|--------|-----------|----------|-------|
| **File Tokens** | > 2000 | Atomicity | Keep |
| **Cognitive Complexity** | > 15 | Complexity | Replaces cyclomatic |
| **Nesting Depth** | > 3 | Complexity | Keep |
| **Function Args** | > 5 | Complexity | Keep |
| **LCOM4** | > 1 | State | Class should be split |
| **AHF** | < 60% | State | State is leaking |
| **CBO** | > 9 | State | Defect predictor |
| **SFOUT** | > 7 | Performance | Architectural bottleneck |
| **Author Entropy** | > 0.8 | State | No clear owner (git) |

### Metric Definitions

**Cognitive Complexity:** Measures mental effort to understand code. Penalizes nesting more heavily than sequential branches. Shorthand notation (ternary) penalized less than full if/else.

**LCOM4:** Model class as undirected graph. Nodes = methods. Edge exists if methods share a field or one calls the other. LCOM4 = number of connected components. Value > 1 means the class is doing multiple unrelated things.

**AHF (Attribute Hiding Factor):** Percentage of fields that are private. `AHF = private_fields / total_fields × 100`. Low AHF means state is accessible from outside, increasing coupling.

**CBO (Coupling Between Objects):** Count of distinct classes/modules this class depends on or is depended upon by. High CBO predicts defects.

**SFOUT (Structural Fan-Out):** Number of outgoing calls/dependencies from a function or module. High fan-out = bottleneck, ripple effects on change.

**Author Entropy:** Shannon entropy of commit authorship. `H = -Σ(p_i × log2(p_i))` where p_i is proportion of changes by author i. High entropy = no clear owner = defects.

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

### Semantic

| ID | Pattern | Rust | TypeScript |
|----|---------|------|------------|
| M01 | Missing doc on public | `pub fn` without `///` | `export function` without JSDoc |
| M02 | Dead parameter | Param never read in body | Same |
| M03 | Getter with mutation | `get_*` / `is_*` that writes | Same |
| M04 | Name/return mismatch | `is_*` returns non-bool | Same |
| M05 | Side-effecting calculation | `calculate_*` / `compute_*` that writes state | Same |

### Logic & Off-by-One

| ID | Pattern | Rust | TypeScript |
|----|---------|------|------------|
| L01 | Untested public | `pub fn` without `#[test]` nearby | `export` without `.test.ts` |
| L02 | Boundary ambiguity | `..` vs `..=` near loop | `<` vs `<=` with `.length` |
| L03 | Unchecked first access | `.first()` / `[0]` without len check | Same |

---

## Work Breakdown

Ordered by difficulty. Hardest first.

### Cross-File Analysis

**LCOM4**
- Build method-field graph per class/struct/impl
- Requires: parsing all methods, tracking field access, building adjacency
- Count connected components
- Challenge: Rust's impl blocks separate from struct definition

**AHF**
- Track visibility of all fields across codebase
- Requires: full project scan, visibility modifiers
- Calculate percentage private

**CBO**
- Build dependency graph between modules
- Count edges per node
- Requires: import/use resolution, cross-file

**Author Entropy**
- Shell out to git blame
- Parse output, calculate Shannon entropy per file/module
- Requires: git integration

**N+1 Detection**
- Must know what constitutes a "database call"
- Requires: configurable sink list or heuristics (function names, return types)
- Track if called inside loop with iterator-derived param

### Async Analysis

**Async Race Gap (C01)**
- Parse async function body
- Track variable reads before await
- Check if same variable written after await
- Challenge: control flow branches

**Floating Promise (C02)**
- Find async call expressions
- Check if result is awaited, assigned, or returned
- Relatively straightforward AST check

**Check-Then-Act (C05)**
- Find if-statement checking variable
- Scan body for await
- Check if same variable mutated after await

**Lock Across Await (C03)**
- Track MutexGuard bindings
- Check if binding scope spans an await
- Requires: lifetime-like analysis

### Listener/Subscription Tracking

**Unbalanced Listener (R01)**
- Find addEventListener calls
- Track the handler reference
- Scan for removeEventListener with same handler
- Challenge: handler might be inline vs named

**Floating Subscription (R02)**
- Find .subscribe() calls
- Check for takeUntil pipe or stored return value
- Angular/RxJS specific patterns

### Single-File Metrics

**Cognitive Complexity**
- Traverse AST
- Increment for: if, else, switch, for, while, catch, &&, ||, nested breaks
- Apply nesting multiplier
- Well-documented algorithm (SonarSource paper)

**SFOUT**
- Count distinct call targets in function
- Count distinct imports/uses in module
- Straightforward counting

### Pattern Matching

**State Patterns (S01-S05)**
- S01/S02: Check declaration scope + mutability modifier
- S03: Check type annotation for container types
- S04: Build scope, check identifier resolution
- S05: Check assignment target is parameter access

**Security Patterns (X01-X05)**
- X01/X02: Pattern match dangerous function + string interpolation
- X03: Regex on const declarations with high-entropy strings
- X04: Find JSON.parse not in try block
- X05: Find bracket notation with non-literal key

**Performance Patterns (P01-P06)**
- Find loop constructs
- Check body for specific patterns (clone, new, etc.)
- Mostly straightforward AST queries

**Semantic Patterns (M01-M05)**
- M01: Check for doc comment before pub/export
- M02: Collect params, scan body for reads
- M03-M05: Parse function name, check body/return type

**Resource Patterns (R03-R07)**
- Loop body checks similar to performance
- R07: Track BufWriter binding, check for flush before scope end

**Logic Patterns (L01-L03)**
- L01: Scan for test attributes/files
- L02: Flag comparison operators near loop bounds
- L03: Check for length/is_empty guard before index

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
author_entropy_max = 0.8

# Enable/disable categories
[scan.categories]
state = true
concurrency = true
resource = true
security = true
performance = true
semantic = true
logic = true

# Per-pattern overrides
[scan.patterns]
S01 = "error"    # Global mutable
S02 = "warn"     # Exported mutable
C02 = "error"    # Floating promise
X03 = "error"    # Hardcoded secret
M01 = "warn"     # Missing doc
```

---

## Output Format

```
$ slopchop scan

src/lib.rs
  ├─ [S01] Global mutable state: `static mut COUNTER` (line 42)
  ├─ [P01] Clone in loop: `.clone()` inside `for` (line 87)
  └─ [M01] Missing doc: `pub fn process_data` (line 112)

src/handlers.rs
  ├─ [LCOM4] Class cohesion: 3 connected components (should be 1)
  ├─ [CC] Cognitive complexity: 23 (max 15) in `handle_request`
  └─ [X01] SQL concatenation: `format!("SELECT...")` (line 56)

src/utils.ts
  ├─ [C02] Floating promise: async call without await (line 34)
  ├─ [R01] Unbalanced listener: addEventListener without remove (line 78)
  └─ [S04] Impure function: reads `config` not passed as param (line 91)

Summary:
  Errors: 4
  Warnings: 5
  Files scanned: 23
```

---

## What This Replaces

| Old | New |
|-----|-----|
| Cyclomatic complexity | Cognitive complexity |
| Just counting metrics | Metrics + 35 AST patterns |
| Rust-focused | Rust + TypeScript parity |
| Single-file only | Cross-file analysis for coupling |
| No security checks | Injection, secrets, pollution |
| No concurrency checks | Async race patterns |
| No semantic checks | Name/behavior alignment |

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
