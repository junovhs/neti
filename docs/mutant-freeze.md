# MUTATION-FREEZE: The Path to 80% Kill Rate

**Status:** ACTIVE — All other work paused
**Created:** 2026-01-22
**Target:** 80% mutation kill rate
**Exit Criteria:** Kill rate ≥ 80% with documented exclusions

---

## Executive Summary

SlopChop's mutation testing baseline is **12% kill rate** (300 caught / 2475 total). This is unacceptable for a "high-integrity governance engine." All feature work is frozen until we reach **80% kill rate**.

This document is the **north star** for all mutation-related work. Follow it sequentially. Do not skip phases. Verify progress after each phase.

---

## Current State (2026-01-22)

| Metric | Value |
|--------|-------|
| Total Mutants | 2,475 |
| Caught | 300 (12.1%) |
| Missed | 1,963 (79.3%) |
| Unviable | 210 (8.5%) |
| Timeout | 2 (0.1%) |

### Where Tests Exist (Caught Mutants)

These modules have SOME coverage — tests exist but need extension:

```
src/analysis/v2/cognitive.rs        — 15 caught, ~5 missed
src/analysis/v2/patterns/logic.rs   — 30 caught, 20 missed  
src/analysis/v2/patterns/semantic.rs — 10 caught, ~10 missed
src/analysis/v2/patterns/idiomatic.rs — 12 caught, ~8 missed
src/analysis/v2/patterns/resource.rs — 6 caught, ~4 missed
src/graph/locality/validator.rs     — 12 caught, ~15 missed
src/graph/locality/exemptions.rs    — 8 caught, ~10 missed
src/graph/locality/types.rs         — 14 caught, ~5 missed
src/graph/imports.rs                — 14 caught, ~5 missed
src/graph/tsconfig.rs               — 10 caught, ~8 missed
src/apply/blocks.rs                 — 10 caught, ~5 missed
src/apply/parser.rs                 — 1 caught, ~40 missed (CRITICAL GAP)
src/lang.rs                         — 10 caught, ~5 missed
src/map.rs                          — 10 caught, ~15 missed
```

### Where Tests DON'T Exist (Zero Catches)

These modules have NO unit test coverage:

```
CRITICAL (Core Features):
  src/tokens.rs                     — ~8 missed, 0 caught
  src/skeleton.rs                   — ~15 missed, 0 caught
  src/pack/focus.rs                 — ~20 missed, 0 caught
  src/graph/rank/pagerank.rs        — ~15 missed, 0 caught
  src/graph/locality/distance.rs    — ~10 missed, 0 caught
  src/graph/locality/cycles.rs      — ~15 missed, 0 caught
  src/apply/validator.rs            — ~40 missed, 0 caught

HIGH (Pattern Detectors):
  src/analysis/v2/patterns/state.rs       — ~20 missed
  src/analysis/v2/patterns/security.rs    — ~25 missed
  src/analysis/v2/patterns/performance.rs — ~30 missed
  src/analysis/v2/patterns/concurrency_lock.rs — ~15 missed
  src/analysis/v2/patterns/concurrency_sync.rs — ~10 missed
  src/analysis/v2/patterns/db_patterns.rs — ~15 missed

MEDIUM (Graph Infrastructure):
  src/graph/resolver.rs             — ~20 missed
  src/graph/locality/coupling.rs    — ~10 missed
  src/graph/locality/classifier.rs  — ~12 missed
  src/graph/rank/builder.rs         — ~25 missed

EXCLUDED (External Boundaries):
  src/branch.rs                     — 28 missed (git shell)
  src/clean.rs                      — 13 missed (fs + git)
  src/clipboard/**                  — ~15 missed (OS APIs)
  src/spinner/**                    — ~30 missed (terminal)
  src/cli/handlers/**               — ~100 missed (integration)
  src/mutate/**                     — ~40 missed (being deleted)
```

---

## Phase 0: Establish Baseline with Exclusions

**Duration:** 1 hour
**Goal:** Create honest baseline by excluding genuinely untestable code

### Tasks

1. Copy `.mutants.toml` to repository root
2. Run mutation testing with exclusions:
   ```bash
   cargo mutants --package slopchop
   ```
3. Record new baseline numbers
4. Commit exclusion file with message: `chore: add mutation testing exclusions`

### Expected Outcome

| Metric | Before | After Exclusions |
|--------|--------|------------------|
| Total Mutants | 2,475 | ~2,050 |
| Missed | 1,963 | ~1,550 |
| Kill Rate | 12% | ~24% |

### Verification

```bash
# Must see reduced mutant count
cargo mutants --list | wc -l
# Should be ~2050, not 2475
```

---

## Phase 1: Critical Path — Zero Coverage Modules

**Duration:** 2-3 days
**Goal:** Add tests to core algorithms with ZERO existing coverage
**Expected Kills:** ~200 mutants

These are the highest-ROI targets — core features with no tests at all.

### 1.1 tokens.rs (~8 mutants)

**Location:** `src/tokens.rs`
**Functions to test:**
- `count_tokens(text: &str) -> usize`
- `exceeds_limit(text: &str, limit: usize) -> bool`

**Test file:** `src/tokens.rs` (add `#[cfg(test)]` module) or `tests/tokens_test.rs`

**Test cases:**
```
TC-TOK-01: Empty string returns 0 tokens
TC-TOK-02: Single word returns 1 token
TC-TOK-03: Known string returns expected count (use tiktoken reference)
TC-TOK-04: Unicode text tokenizes correctly
TC-TOK-05: exceeds_limit returns false when under
TC-TOK-06: exceeds_limit returns true when over
TC-TOK-07: exceeds_limit boundary case (exactly at limit)
TC-TOK-08: Large text performance (sanity check, not assertion)
```

### 1.2 skeleton.rs (~15 mutants)

**Location:** `src/skeleton.rs`
**Functions to test:**
- `skeletonize(source: &str, lang: Lang) -> String`
- Body replacement logic

**Test cases:**
```
TC-SKEL-01: Rust function body replaced with { ... }
TC-SKEL-02: Rust impl block bodies replaced
TC-SKEL-03: Python function body replaced with ...
TC-SKEL-04: TypeScript function body replaced
TC-SKEL-05: Nested functions handled correctly
TC-SKEL-06: Struct/class definitions preserved
TC-SKEL-07: Comments and attributes preserved
TC-SKEL-08: Empty function stays empty (edge case)
TC-SKEL-09: Multiline signatures preserved
TC-SKEL-10: Invalid syntax returns original (graceful degradation)
```

### 1.3 pack/focus.rs (~20 mutants)

**Location:** `src/pack/focus.rs`
**Functions to test:**
- Foveal/peripheral classification
- Token budget allocation
- Neighbor expansion

**Test cases:**
```
TC-FOCUS-01: Single focal file gets full content
TC-FOCUS-02: Neighbors get skeleton treatment
TC-FOCUS-03: Token budget respected
TC-FOCUS-04: Depth=0 returns only focal file
TC-FOCUS-05: Depth=1 includes direct dependencies
TC-FOCUS-06: Depth=2 includes transitive dependencies
TC-FOCUS-07: Circular dependencies don't infinite loop
TC-FOCUS-08: Missing files handled gracefully
TC-FOCUS-09: Budget overflow truncates peripheral, not focal
TC-FOCUS-10: Empty graph returns just focal file
```

### 1.4 graph/rank/pagerank.rs (~15 mutants)

**Location:** `src/graph/rank/pagerank.rs`
**Functions to test:**
- `compute(graph, damping, iterations) -> HashMap<Node, f64>`
- Convergence logic
- Personalization (anchoring)

**Test cases:**
```
TC-PR-01: Single node graph returns 1.0
TC-PR-02: Two nodes, one edge: source < target rank
TC-PR-03: Cycle of 3 nodes: equal ranks
TC-PR-04: Star topology: center has highest rank
TC-PR-05: Damping factor affects distribution
TC-PR-06: More iterations = more convergence
TC-PR-07: Disconnected components handled
TC-PR-08: Personalization boosts anchor's neighbors
TC-PR-09: Empty graph returns empty map
TC-PR-10: Self-loops handled correctly
```

### 1.5 graph/locality/distance.rs (~10 mutants)

**Location:** `src/graph/locality/distance.rs`
**Functions to test:**
- `compute_distance(from: &Path, to: &Path) -> usize`
- LCA (Lowest Common Ancestor) finding

**Test cases:**
```
TC-DIST-01: Same file = distance 0
TC-DIST-02: Sibling files = distance 2
TC-DIST-03: Parent-child = distance 1
TC-DIST-04: Cousin files (shared grandparent) = distance 4
TC-DIST-05: Deeply nested vs root = correct count
TC-DIST-06: Different path separators normalized
TC-DIST-07: Relative vs absolute paths handled
TC-DIST-08: Non-existent paths don't panic
```

### 1.6 graph/locality/cycles.rs (~15 mutants)

**Location:** `src/graph/locality/cycles.rs`
**Functions to test:**
- `detect_cycles(edges) -> Vec<Vec<PathBuf>>`
- DFS traversal logic

**Test cases:**
```
TC-CYC-01: No edges = no cycles
TC-CYC-02: Linear chain = no cycles
TC-CYC-03: Self-loop detected
TC-CYC-04: Two-node cycle detected
TC-CYC-05: Three-node cycle detected
TC-CYC-06: Multiple disjoint cycles all found
TC-CYC-07: Figure-8 (two cycles sharing node) handled
TC-CYC-08: Large cycle (10+ nodes) detected
TC-CYC-09: DAG with convergence (not a cycle) passes
TC-CYC-10: Cycle membership is complete (all nodes listed)
```

### 1.7 apply/validator.rs (~40 mutants) — CRITICAL

**Location:** `src/apply/validator.rs`
**Functions to test:**
- Manifest validation
- File existence checks
- Path security validation
- SHA256 verification

**Test cases:**
```
TC-VAL-01: Valid manifest passes
TC-VAL-02: Empty manifest rejected
TC-VAL-03: Duplicate paths rejected
TC-VAL-04: Path traversal (../) rejected
TC-VAL-05: Absolute paths rejected (security)
TC-VAL-06: Reserved keywords rejected (PLAN, MANIFEST, etc.)
TC-VAL-07: Missing required files flagged
TC-VAL-08: SHA mismatch detected
TC-VAL-09: NEW operation on existing file = warning
TC-VAL-10: DELETE operation on missing file = warning
TC-VAL-11: UPDATE operation validates original exists
TC-VAL-12: Whitespace-only paths rejected
TC-VAL-13: Valid special characters in paths allowed
TC-VAL-14: Symlink paths handled safely
TC-VAL-15: Case sensitivity respected (or not, per OS)
```

### 1.8 apply/parser.rs (EXTEND — only 1 caught) (~40 mutants)

**Location:** `src/apply/parser.rs`
**Current state:** Only tests empty result
**Functions to test:**
- `parse(input: &str) -> Result<Vec<Block>>`
- XSC7XSC sigil recognition
- Block type detection
- Content extraction

**Test cases:**
```
TC-PARSE-01: Empty input returns empty vec
TC-PARSE-02: Single FILE block parsed correctly
TC-PARSE-03: Single PLAN block parsed correctly
TC-PARSE-04: Single MANIFEST block parsed correctly
TC-PARSE-05: Multiple blocks in sequence
TC-PARSE-06: Block with SHA256 hash extracted
TC-PARSE-07: Markdown fence stripping (```rust)
TC-PARSE-08: Blockquote prefix stripping (> )
TC-PARSE-09: Nested sigils in content don't confuse parser
TC-PARSE-10: Unclosed block returns error
TC-PARSE-11: Missing END marker returns error
TC-PARSE-12: Wrong END marker returns error
TC-PARSE-13: Path extraction from FILE header
TC-PARSE-14: Whitespace handling in content
TC-PARSE-15: Very large block (100KB) handled
TC-PARSE-16: Binary-looking content preserved
TC-PARSE-17: Unicode in path and content
TC-PARSE-18: Windows line endings (CRLF)
TC-PARSE-19: Mixed line endings
TC-PARSE-20: Interleaved PLAN and FILE blocks
```

### Phase 1 Verification

After completing all Phase 1 tests:

```bash
cargo test  # All tests pass
cargo mutants --package slopchop
```

**Expected state:**
| Metric | After Phase 0 | After Phase 1 |
|--------|---------------|---------------|
| Missed | ~1,550 | ~1,350 |
| Caught | ~500 | ~700 |
| Kill Rate | ~24% | ~34% |

---

## Phase 2: Pattern Detector Tests

**Duration:** 2-3 days
**Goal:** Test all V2 AST pattern detectors
**Expected Kills:** ~200 mutants

Each pattern detector follows the same structure:
- Input: Source code string + tree-sitter AST
- Output: `Vec<Violation>`
- Test: Positive cases (should detect) + Negative cases (should NOT detect)

### 2.1 patterns/state.rs (S01, S02, S03) — ~20 mutants

**Test cases:**
```
# S01: Global mutable declaration
TC-S01-POS-01: `static mut X: i32 = 0;` detected
TC-S01-POS-02: `static mut` in nested module detected
TC-S01-NEG-01: `static X: i32 = 0;` (immutable) not flagged
TC-S01-NEG-02: `const X: i32 = 0;` not flagged

# S02: Exported mutable
TC-S02-POS-01: `pub static X: Mutex<i32>` detected
TC-S02-NEG-01: `pub static X: &str` (const-like) not flagged
TC-S02-NEG-02: `static X: Mutex<i32>` (private) not flagged

# S03: Suspicious global container
TC-S03-POS-01: `lazy_static! { static ref X: Mutex<Vec<_>> }` detected
TC-S03-POS-02: `OnceCell<HashMap>` detected
TC-S03-NEG-01: `lazy_static! { static ref CONFIG: &str }` not flagged
```

### 2.2 patterns/security.rs (X01-X07) — ~25 mutants

**Test cases:**
```
# X01: SQL Injection
TC-X01-POS-01: `format!("SELECT * FROM users WHERE id = {}", id)` detected
TC-X01-POS-02: String concatenation in query detected
TC-X01-NEG-01: Parameterized query `query("SELECT * FROM users WHERE id = ?", &[id])` not flagged

# X02: Command Injection  
TC-X02-POS-01: `Command::new("sh").arg(user_input)` detected
TC-X02-NEG-01: `Command::new("ls").arg("-la")` (literal) not flagged

# X03: Hardcoded Secret
TC-X03-POS-01: `let api_key = "sk-1234567890abcdef"` detected
TC-X03-POS-02: `const TOKEN: &str = "ghp_xxxx"` detected
TC-X03-NEG-01: `let key = get_from_env("API_KEY")` not flagged
TC-X03-NEG-02: `let key = ""` (empty) not flagged

# X06: Dangerous Config
TC-X06-POS-01: `.danger_accept_invalid_certs(true)` detected
TC-X06-POS-02: `verify: false` in TLS config detected
TC-X06-NEG-01: `.danger_accept_invalid_certs(false)` not flagged

# X07: Unbounded Deserialization
TC-X07-POS-01: `bincode::deserialize(&bytes)` detected
TC-X07-NEG-01: `bincode::options().with_limit(1024).deserialize()` not flagged
```

### 2.3 patterns/performance.rs (P01-P06) — ~30 mutants

**Test cases:**
```
# P01: Clone in loop
TC-P01-POS-01: `for x in items { let y = x.clone(); }` detected
TC-P01-NEG-01: `let y = x.clone(); for x in items { }` (outside loop) not flagged
TC-P01-NEG-02: `vec.push(x.clone())` (ownership sink) not flagged

# P02: Allocation in loop
TC-P02-POS-01: `for _ in 0..n { let s = String::new(); }` detected
TC-P02-POS-02: `for _ in 0..n { let v = Vec::new(); }` detected
TC-P02-NEG-01: `let s = String::new(); for _ in 0..n { }` not flagged

# P03: N+1 Query
TC-P03-POS-01: `for id in ids { db.query("SELECT...", id); }` detected
TC-P03-NEG-01: `Atomic::load` in loop not flagged (allowlist)
TC-P03-NEG-02: `Arc::clone` in loop not flagged (allowlist)

# P04: Nested Loop
TC-P04-POS-01: `for x in xs { for y in ys { } }` detected (O(n²))
TC-P04-NEG-01: `for x in xs { } for y in ys { }` (sequential) not flagged

# P06: Linear Search in Loop
TC-P06-POS-01: `for x in xs { if ys.contains(&x) { } }` detected
TC-P06-POS-02: `for x in xs { ys.iter().find(|y| y == x); }` detected
TC-P06-NEG-01: HashSet lookup in loop not flagged
```

### 2.4 patterns/concurrency_lock.rs (C03) — ~15 mutants

**Test cases:**
```
# C03: Lock across await
TC-C03-POS-01: `let guard = mutex.lock(); foo().await;` detected
TC-C03-POS-02: `let _g = self.state.lock(); bar().await;` detected
TC-C03-NEG-01: `{ let guard = mutex.lock(); } foo().await;` (dropped before await) not flagged
TC-C03-NEG-02: `let guard = async_mutex.lock().await;` (async-aware mutex) not flagged per spec note
TC-C03-NEG-03: Non-async function with MutexGuard not flagged
```

### 2.5 patterns/concurrency_sync.rs (C04) — ~10 mutants

**Test cases:**
```
# C04: Undocumented sync primitive
TC-C04-POS-01: `struct Foo { state: Arc<Mutex<Vec<u8>>> }` without doc comment detected
TC-C04-NEG-01: `/// Protected state\n state: Arc<Mutex<Vec<u8>>>` not flagged
TC-C04-NEG-02: `// Internal sync\n state: Arc<Mutex<_>>` not flagged
TC-C04-NEG-03: `Arc<String>` (no Mutex) not flagged
```

### 2.6 patterns/db_patterns.rs (P03 detailed) — ~15 mutants

Extend P03 tests with more database patterns:
```
TC-DB-01: `sqlx::query` in loop detected
TC-DB-02: `diesel::load` in loop detected  
TC-DB-03: `fetch_one` in loop detected
TC-DB-04: `execute` in loop detected
TC-DB-05: Batched query outside loop not flagged
```

### Phase 2 Verification

```bash
cargo test
cargo mutants --package slopchop
```

**Expected state:**
| Metric | After Phase 1 | After Phase 2 |
|--------|---------------|---------------|
| Missed | ~1,350 | ~1,150 |
| Caught | ~700 | ~900 |
| Kill Rate | ~34% | ~44% |

---

## Phase 3: Extend Existing Test Coverage

**Duration:** 2 days
**Goal:** Fill gaps in modules that have SOME coverage
**Expected Kills:** ~150 mutants

### 3.1 patterns/logic.rs — Extend (~20 missed after Phase 2)

Current tests catch basic cases. Add edge cases:

```
# L02 edge cases
TC-L02-EDGE-01: `if i <= arr.len()` with non-index var not flagged
TC-L02-EDGE-02: `if buffer.len() >= 1024` (threshold check) not flagged
TC-L02-EDGE-03: `if idx <= slice.len() - 1` still flagged

# L03 edge cases  
TC-L03-EDGE-01: `arr[0]` with preceding `if !arr.is_empty()` not flagged
TC-L03-EDGE-02: `arr.first().unwrap()` with `assert!(!arr.is_empty())` not flagged
TC-L03-EDGE-03: Guard in different scope still flagged
```

### 3.2 graph/locality/validator.rs — Extend (~15 missed)

```
TC-VAL-EDGE-01: Hub exemption applied correctly
TC-VAL-EDGE-02: Distance threshold boundary (exactly at limit)
TC-VAL-EDGE-03: Multiple violations on same edge
TC-VAL-EDGE-04: Exemption + violation on same file
```

### 3.3 graph/locality/exemptions.rs — Extend (~10 missed)

```
TC-EXEMPT-01: main.rs importing anything is exempt
TC-EXEMPT-02: mod.rs re-exporting children is exempt
TC-EXEMPT-03: lib.rs importing is exempt
TC-EXEMPT-04: test file importing from parent is exempt
TC-EXEMPT-05: Non-exempt sideways import still flagged
```

### 3.4 map.rs — Extend (~15 missed)

```
TC-MAP-01: format_size handles bytes correctly
TC-MAP-02: format_size handles KB correctly
TC-MAP-03: format_size handles MB correctly
TC-MAP-04: format_tokens handles small counts
TC-MAP-05: format_tokens handles large counts with 'k' suffix
TC-MAP-06: Tree building with nested directories
TC-MAP-07: Tree building with flat structure
```

### Phase 3 Verification

**Expected state:**
| Metric | After Phase 2 | After Phase 3 |
|--------|---------------|---------------|
| Missed | ~1,150 | ~1,000 |
| Caught | ~900 | ~1,050 |
| Kill Rate | ~44% | ~51% |

---

## Phase 4: Graph Infrastructure Tests

**Duration:** 2 days
**Goal:** Test remaining graph/rank and graph/locality modules
**Expected Kills:** ~150 mutants

### 4.1 graph/resolver.rs (~20 mutants)

**Functions:** Path resolution from import strings

```
TC-RES-01: Rust `use crate::foo` resolves to src/foo.rs
TC-RES-02: Rust `use crate::foo` resolves to src/foo/mod.rs
TC-RES-03: Rust `use super::bar` resolves correctly
TC-RES-04: TypeScript relative import `./utils` resolves
TC-RES-05: TypeScript index file `./utils` -> `./utils/index.ts`
TC-RES-06: TypeScript path alias `@/components` resolves
TC-RES-07: Python relative import `.foo` resolves
TC-RES-08: Non-existent import returns None
TC-RES-09: External crate/package returns None
```

### 4.2 graph/locality/coupling.rs (~10 mutants)

```
TC-COUP-01: Afferent (fan-in) counts incoming edges
TC-COUP-02: Efferent (fan-out) counts outgoing edges
TC-COUP-03: Isolated node has (0, 0)
TC-COUP-04: Hub has high afferent
TC-COUP-05: Leaf has high efferent
```

### 4.3 graph/locality/classifier.rs (~12 mutants)

```
TC-CLASS-01: High fan-in, low fan-out = Stable Hub
TC-CLASS-02: Low fan-in, high fan-out = Volatile Leaf
TC-CLASS-03: High both = God Module
TC-CLASS-04: Low both = Deadwood (or Normal)
TC-CLASS-05: Threshold boundaries tested
```

### 4.4 graph/rank/builder.rs (~25 mutants)

```
TC-BUILD-01: Empty repo returns empty graph
TC-BUILD-02: Single file with no imports
TC-BUILD-03: Two files, one imports other
TC-BUILD-04: Import cycle creates bidirectional edge
TC-BUILD-05: External imports ignored
TC-BUILD-06: Symbol-level edge weights accumulated
```

### Phase 4 Verification

**Expected state:**
| Metric | After Phase 3 | After Phase 4 |
|--------|---------------|---------------|
| Missed | ~1,000 | ~850 |
| Caught | ~1,050 | ~1,200 |
| Kill Rate | ~51% | ~59% |

---

## Phase 5: Refactor for Testability

**Duration:** 2-3 days
**Goal:** Extract pure logic from I/O-heavy modules
**Expected Kills:** ~150 mutants

### 5.1 config/mod.rs — Extract Pure Logic

**Current:** `Config::load()` reads from filesystem
**Refactor to:**
```rust
impl Config {
    // Pure - can be unit tested
    pub fn from_toml_str(content: &str) -> Result<Self>
    pub fn validate(&self) -> Result<()>
    
    // I/O - stays excluded
    pub fn load(path: &Path) -> Result<Self>
}
```

**Tests:**
```
TC-CFG-01: from_toml_str parses valid config
TC-CFG-02: from_toml_str rejects invalid TOML
TC-CFG-03: from_toml_str applies defaults for missing fields
TC-CFG-04: validate rejects negative thresholds
TC-CFG-05: validate rejects invalid patterns
```

### 5.2 discovery.rs — Extract Filter Predicates

**Current:** Mixes filesystem walking with filtering logic
**Refactor to:**
```rust
// Pure - can be unit tested
pub fn should_include(path: &Path, config: &Config) -> bool
pub fn matches_ignore_pattern(path: &Path, patterns: &[String]) -> bool

// I/O - stays excluded
pub fn discover(root: &Path, config: &Config) -> Vec<PathBuf>
```

**Tests:**
```
TC-DISC-01: should_include rejects .git paths
TC-DISC-02: should_include rejects node_modules
TC-DISC-03: should_include accepts .rs files
TC-DISC-04: matches_ignore_pattern handles globs
TC-DISC-05: matches_ignore_pattern handles exact matches
```

### 5.3 apply/executor.rs — Extract Pure Validation

**Current:** Mixes git operations with validation logic
**Refactor to:**
```rust
// Pure - can be unit tested
pub fn validate_execution_plan(blocks: &[Block], manifest: &Manifest) -> Result<()>
pub fn compute_file_operations(blocks: &[Block]) -> Vec<FileOp>

// I/O - stays excluded
pub fn execute(blocks: &[Block]) -> Result<()>
```

### Phase 5 Verification

**Expected state:**
| Metric | After Phase 4 | After Phase 5 |
|--------|---------------|---------------|
| Missed | ~850 | ~700 |
| Caught | ~1,200 | ~1,350 |
| Kill Rate | ~59% | ~66% |

---

## Phase 6: Analysis V2 Infrastructure

**Duration:** 2 days
**Goal:** Test remaining V2 analysis infrastructure
**Expected Kills:** ~100 mutants

### 6.1 analysis/v2/metrics.rs (~20 mutants)

**Functions:** LCOM4, CBO, AHF, SFOUT calculations

```
TC-LCOM4-01: Single method struct = LCOM4 of 1
TC-LCOM4-02: Two methods sharing field = LCOM4 of 1
TC-LCOM4-03: Two methods, no shared fields = LCOM4 of 2
TC-LCOM4-04: Complex connected components calculated correctly

TC-CBO-01: No external deps = CBO of 0
TC-CBO-02: One external type = CBO of 1
TC-CBO-03: Multiple uses of same type = CBO of 1 (not counted twice)

TC-AHF-01: All private fields = AHF of 100%
TC-AHF-02: All public fields = AHF of 0%
TC-AHF-03: Mixed visibility calculated correctly

TC-SFOUT-01: No external calls = SFOUT of 0
TC-SFOUT-02: Calls to multiple external functions counted
```

### 6.2 analysis/v2/scope.rs (~15 mutants)

```
TC-SCOPE-01: Struct extracted with all fields
TC-SCOPE-02: Impl methods associated with struct
TC-SCOPE-03: Field access in method body recorded
TC-SCOPE-04: External calls in method body recorded
```

### 6.3 analysis/v2/rust.rs and rust_impl.rs (~25 mutants)

```
TC-RUST-01: Struct definition extracted
TC-RUST-02: Enum definition extracted
TC-RUST-03: Impl block associated with type
TC-RUST-04: Generic types handled
TC-RUST-05: Derive macros captured
TC-RUST-06: Self references tracked
```

### Phase 6 Verification

**Expected state:**
| Metric | After Phase 5 | After Phase 6 |
|--------|---------------|---------------|
| Missed | ~700 | ~600 |
| Caught | ~1,350 | ~1,450 |
| Kill Rate | ~66% | ~71% |

---

## Phase 7: Final Push & Triage

**Duration:** 2-3 days
**Goal:** Reach 80% kill rate, document remainder
**Expected Kills:** ~150 mutants + triage

### 7.1 Remaining High-Value Targets

Review `mutants.out/missed.txt` for:
- Functions with 5+ missed mutants (high impact)
- Simple predicates/comparisons (easy wins)
- Mathematical operations (+ to -, etc.)

### 7.2 Equivalent Mutant Documentation

Some mutants are mathematically equivalent and cannot be killed:
- `x + 1` vs `x * 1` when x is always 1
- `||` vs `&&` when one operand is always true
- Return value changes that don't affect callers

Document these in `MUTATION-EQUIVALENTS.md`.

### 7.3 Accepted Gaps Documentation

For mutants that would require excessive test infrastructure:
- Document in `MUTATION-ACCEPTED.md`
- Include rationale for each
- Mark for future integration test coverage

### Phase 7 Verification

**Expected state:**
| Metric | After Phase 6 | After Phase 7 |
|--------|---------------|---------------|
| Missed | ~600 | ~400 |
| Caught | ~1,450 | ~1,650 |
| Kill Rate | ~71% | **~80%** |

---

## Success Criteria

### Exit Gate

The MUTATION-FREEZE is complete when:

1. [ ] `cargo mutants` reports ≥80% kill rate
2. [ ] `.mutants.toml` exclusions are justified and documented
3. [ ] `MUTATION-EQUIVALENTS.md` documents equivalent mutants
4. [ ] `MUTATION-ACCEPTED.md` documents accepted gaps with rationale
5. [ ] All tests pass: `cargo test`
6. [ ] Clippy clean: `cargo clippy`
7. [ ] SlopChop self-scan clean: `slopchop scan`

### Progress Tracking

After each phase, record in this document:

```
## Progress Log

### Phase 0 - [DATE]
- Baseline after exclusions: X mutants, Y% kill rate
- Commit: [SHA]

### Phase 1 - [DATE]  
- After critical path tests: X mutants, Y% kill rate
- Tests added: N
- Commit: [SHA]

[etc.]
```

---

## Appendix A: Test File Locations

Prefer in-module `#[cfg(test)]` for unit tests:

```
src/tokens.rs              -> src/tokens.rs (add #[cfg(test)] mod tests)
src/skeleton.rs            -> src/skeleton.rs (add #[cfg(test)] mod tests)
src/pack/focus.rs          -> src/pack/focus.rs (add #[cfg(test)] mod tests)
src/graph/rank/pagerank.rs -> src/graph/rank/pagerank.rs (add #[cfg(test)] mod tests)
src/graph/locality/*.rs    -> Add #[cfg(test)] mod tests in each
src/analysis/v2/patterns/* -> Add #[cfg(test)] mod tests in each
src/apply/parser.rs        -> Extend existing tests
src/apply/validator.rs     -> src/apply/validator.rs (add #[cfg(test)] mod tests)
```

For tests requiring test fixtures, use `tests/` directory:
```
tests/fixtures/            -> Sample source files for parsing
tests/parser_test.rs       -> Integration-style parser tests
tests/pattern_test.rs      -> Pattern detection with real source
```

---

## Appendix B: Test Helpers to Create

### B.1 AST Test Helper

```rust
// tests/common/mod.rs
pub fn parse_rust(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(tree_sitter_rust::language()).unwrap();
    parser.parse(source, None).unwrap()
}

pub fn violations_for(source: &str, detector: fn(&str, &Tree) -> Vec<Violation>) -> Vec<Violation> {
    let tree = parse_rust(source);
    detector(source, &tree)
}
```

### B.2 Graph Test Helper

```rust
pub fn simple_graph(edges: &[(&str, &str)]) -> RepoGraph {
    let mut builder = GraphBuilder::new();
    for (from, to) in edges {
        builder.add_edge(PathBuf::from(from), PathBuf::from(to));
    }
    builder.build()
}
```

---

## Appendix C: Mutation Testing Commands

```bash
# Full run (slow, ~3 hours)
cargo mutants --package slopchop

# Quick check specific file
cargo mutants --package slopchop -f src/tokens.rs

# List mutants without running
cargo mutants --package slopchop --list

# Check specific function
cargo mutants --package slopchop -f src/tokens.rs -F count_tokens

# Parallel execution (faster)
cargo mutants --package slopchop -j 4
```

---

## Appendix D: Quick Reference — Mutant Categories

| Status | Meaning | Action |
|--------|---------|--------|
| **caught** | Test failed when mutant applied | ✅ Good |
| **missed** | Tests still pass with mutant | ❌ Need test |
| **unviable** | Mutant doesn't compile | Ignore |
| **timeout** | Test hung | Check for infinite loops |

---

*This document is the single source of truth for mutation testing work. Update the Progress Log as you complete each phase. Do not deviate from the plan without updating this document first.*
