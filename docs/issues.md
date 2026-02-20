# issues-0003: Post-Launch Work

---

## FORMAT (DO NOT MODIFY)

**Status values:** `OPEN`, `IN PROGRESS`, `DONE`, `DESCOPED`
**Issue format:**

```
## [N] Title
**Status:** OPEN
**Files:** list of files to modify
Description of the task.
**Resolution:** (fill when DONE) What was done, any notes.
```

**Instructions:**

* Work issues in order you feel is most important.
* Update status as you go
* Add **Resolution:** when completing
* Don't modify this FORMAT section
* Content below the line is the work.

---

## [1] Smart output compression for neti-report.txt

**Status:** OPEN
**Files:** `src/verification/runner.rs`, potentially new `src/reporting/compression.rs`

Clippy and other tools can output 10,000 lines for what amounts to 2 distinct issues repeated across many call sites. AI agents waste context window on this noise. `neti-report.txt` should be maximally informative but succinct.

Approach:

* Group violations/errors by kind (error code + message template), not by occurrence
* For each kind: show the first 2-3 instances with full context, then a count of remaining occurrences with just file:line references
* Example:

  ```
  E0308 (mismatched types) — 47 occurrences across 12 files
    src/foo.rs:42  expected `String`, found `&str`
    src/bar.rs:88  expected `String`, found `&str`
    ... and 45 more (src/baz.rs:12, src/qux.rs:55, ...)
  ```
* Must work for ANY command output in the `[commands]` pipeline — not clippy-specific
* Write compression logic as a shared utility

**Resolution:**

---

## [2] Locality integration into standard scan pipeline

**Status:** OPEN
**Files:** `src/graph/locality/` (all), `src/cli/locality.rs`, `src/config/locality.rs`

Locality has a massive subgraph (classifier, coupling, cycles, distance, edges, exemptions, layers, validator, types, report, analysis/metrics, analysis/violations) but feels separate from the main scan pipeline.

Questions to resolve:

* Should locality be a first-class `neti check` phase (runs automatically), or remain an opt-in `neti locality` command?
* Is the implementation providing value proportional to its code size?
* Can locality violations be surfaced as regular scan violations (same format, same report) rather than a separate report?

Recommendation: integrate locality results into the standard scan report format. Keep the graph engine as-is but unify the output.

**Resolution:**

---

## [3] Mutation testing strategy: de-scope `neti mutate` or make it truly cross-language

**Status:** OPEN
**Files:** `src/mutate/` (all), `src/cli/mutate_handler.rs`, `src/verification/runner.rs`, docs/README (where mutate is described), tests

`neti mutate` exists today, but Neti’s direction is language-agnostic governance. Mutation testing is valuable, but we need a clear strategy:

**Decision A — Remove/De-scope `neti mutate` (recommended default):**

* Mark `neti mutate` as deprecated and/or remove it.
* Prefer best-in-class ecosystem tools per language (e.g., Rust: `cargo-mutants`).
* Add “mutation testing hooks” via `[commands]` so teams can run their tool of choice and still get unified reporting in `neti-report.txt`.
* Ensure output compression (Issue [1]) supports mutation tool output well.

**Decision B — Make `neti mutate` real and cross-language:**
If we keep it, it must be a generic orchestration layer:

* Define a language-agnostic mutation interface:

  * discover target files
  * generate/apply mutations
  * run configured test command(s)
  * record killed/survived mutants + timing
* Provide per-language adapters:

  * Rust adapter could wrap `cargo-mutants` (or implement minimal native mutations)
  * Python adapter wraps `mutmut`/`cosmic-ray`
  * JS/TS adapter wraps `Stryker`
* Output a consistent report section with:

  * overall kill rate
  * survivors grouped by mutation operator + file:line
  * links to reproduction commands
* Add a baseline target (e.g., 70%+ kill rate) **only** for Neti’s own core logic tests (not for arbitrary repos).

**Deliverables:**

1. Pick A or B and document the rationale.
2. Implement chosen path.
3. Add regression tests around the mutate pipeline (or deprecation/removal behavior).
4. Ensure results flow into the standard `neti check` report format.

**Resolution:**


---

## [4] Python and TypeScript pattern detection parity

**Status:** OPEN
**Files:** `src/analysis/patterns/` (all), `src/lang.rs`

Rust has full pattern detection. Python and TypeScript are marked "Partial" in the README. Close the gap on the most impactful patterns:

* Concurrency patterns where applicable
* Security patterns (injection, credential exposure)
* Performance patterns (allocation in loops, linear search in loops)
* Idiomatic patterns per language

**Resolution:**

---

## [5] Concurrency rule C03: distinguish sync Mutex vs async mutexes to reduce false positives

**Status:** OPEN
**Files:** `src/analysis/patterns/concurrency*.rs`, `src/analysis/patterns/mod.rs`, potentially `src/lang.rs`, tests

Neti currently flags “MutexGuard held across `.await`” (C03) in codebases that use async-aware mutexes (e.g., `tokio::sync::Mutex`, `futures_util::lock::Mutex`). This produces false positives and overly severe guidance (“critical deadlock risk”) for async locks.

Required behavior:

* Detect whether the guard is from `std::sync::Mutex`/`parking_lot::Mutex` (sync) vs `tokio::sync::Mutex` / `futures_util::lock::Mutex` (async)
* For sync mutex guards across `.await`: keep as high severity (real deadlock/thread-block risk)
* For async mutex guards across `.await`: downgrade severity and adjust guidance to async-appropriate risks (head-of-line blocking, reentrancy hazards, starvation), and/or propose more specific patterns like “awaiting I/O while holding async lock”
* Add regression tests using real-world examples (Dioxus fullstack websocket payloads and CLI logging patterns)

Notes:

* If type resolution is unavailable, add pragmatic heuristics: import path checks, fully-qualified type names in tokens, or pattern match on `.lock().await` vs `.lock()`.

**Resolution:**

---

## [6] Command injection rule X02: reduce false positives for tokio::process::Command and non-shell execution

**Status:** OPEN
**Files:** `src/analysis/patterns/security.rs`, tests

Neti flags `tokio::process::Command::new(binary_path)` (and similar `std::process::Command`) as “command injection” even when arguments are passed via `.arg(...)` and no shell is invoked. This is often not injection; the realistic risk is executable provenance (PATH hijacking, untrusted download integrity, unexpected resolution).

Required behavior:

* Differentiate “shell invocation” vs “direct exec”

  * Shell invocation examples: `sh -c ...`, `cmd /C ...`, `powershell -Command ...`, `Command::new("sh").arg("-c")...`, etc.
  * Direct exec with `.arg(...)` should not be classified as injection by default
* Introduce improved taxonomy/guidance:

  * “Shell injection” (high severity) when shell is involved or untrusted string becomes a shell command
  * “Untrusted executable provenance” (warn) when `Command::new` uses dynamic path/name and resolution may be unsafe (PATH, downloads), with suggestions like allowlists, absolute paths, signature checks, or controlled install locations
* Add targeted regression tests using Dioxus CLI Tailwind integration patterns (`tokio::process::Command` + `.arg(...)`) to verify correct classification

**Resolution:**

---

## [7] Rust parser correctness: eliminate “Syntax error detected” on valid Rust

**Status:** OPEN
**Files:** `src/analysis/ast.rs`, `src/analysis/syntax.rs`, `src/lang.rs`, `Cargo.toml` (tree-sitter deps), tests

Neti currently emits LAW OF INTEGRITY “Syntax error detected” for valid Rust constructs (range patterns `0..`, `1..`, `Some(2..)`, inner attributes like `#![doc(...)]`, C-string literals `c"..."`, cfg-gated destructuring, etc). This is a trust-killer and blocks any attempt to use Neti on modern Rust.

Required behavior:

* Upgrade Rust parsing to fully support modern Rust syntax (Rust 1.90+).
* If Neti cannot parse a file due to unsupported grammar, it must report a **non-violation diagnostic** (“parser unsupported for this syntax”) and must not claim a syntax error.
* Add a **soundness guard**: if `cargo check` (or `cargo metadata` + a compile probe) succeeds for the workspace, Neti must never emit “syntax error” for Rust source files (unless the file is outside the crate graph and truly malformed).

Deliverables:

* Update tree-sitter Rust grammar and any node-kind mappings used by Neti.
* Add regression tests for:

  * inner attributes (`#![doc(...)]`)
  * range patterns (`0..`, `24..`, `..=2`, `Some(2..)`)
  * C-string literals (`c"main"`)
  * cfg-conditional fields/destructuring
* Add a “parser unsupported” diagnostics path distinct from “syntax error”.

**Resolution:**

---

## [8] Proof-based indexing analysis: stop flagging safe indexing (arrays, iterators with invariants, chunks_exact)

**Status:** OPEN
**Files:** `src/analysis/patterns/logic.rs` (L03), `src/analysis/ast.rs`, tests

Neti’s L03 “Index `[0]` without bounds check” is currently over-broad. It flags cases that are provably safe (fixed-size arrays, `chunks_exact(2)` indexing `[0]`/`[1]`, known-invariant code paths) and generates noise.

Required behavior:

* Do not flag indexing on **fixed-size arrays** when the index is a constant within bounds.
* Do not flag indexing into slices produced by APIs that guarantee length, e.g.:

  * `chunks_exact(N)` → indexing `[0..N-1]` is safe inside the closure
  * `split_at(N)` → first slice length is `N`
  * `array_chunks::<N>()` / `chunks_exact` patterns (where applicable)
* For ambiguous cases, keep the warning, but provide a more precise suggestion:

  * “prove non-empty with an assertion” (e.g., `debug_assert!(!x.is_empty())`) or refactor.

Deliverables:

* Implement small, composable proofs/invariants:

  * “known fixed length container”
  * “chunks_exact(N) closure implies length >= N”
* Add regression tests using Dioxus examples:

  * `chunks_exact(2).map(|a| u16::from_le_bytes([a[0], a[1]]))` should be **clean**
  * fixed array indexing like `seed[0] = 1` should be **clean**
* Keep L03 strong only when a panic is genuinely plausible.

**Resolution:**

---

## [9] Fix L02 rule logic: remove incorrect off-by-one guidance and replace with correct boundary heuristics

**Status:** OPEN
**Files:** `src/analysis/patterns/logic.rs` (L02), tests

Neti flags correct boundary guards (e.g. `if index >= len { return None; }`) with guidance that suggests it’s suspicious. That’s backwards; it produces false positives and bad advice.

Required behavior:

* L02 should target *actual* boundary ambiguity patterns, such as:

  * loops with `0..=len()` or indexing with `<= len()` comparisons that can reach `len`
  * patterns where `len()` is used in an inclusive range for indexing
* Do not flag `index >= len()` guards—they’re canonical.

Deliverables:

* Replace L02 with a smaller set of precise triggers.
* Add tests to ensure:

  * `if idx >= v.len() { return None; } v[idx]` is not flagged
  * `for i in 0..=v.len() { v[i] }` *is* flagged

**Resolution:**

---

## [10] Safety rule robustness: recognize SAFETY justifications and avoid penalizing well-documented unsafe

**Status:** OPEN
**Files:** `src/analysis/safety.rs`, `src/analysis/patterns/safety.rs` (if split), tests

Requiring `// SAFETY:` is good, but the rule should be robust and ergonomic: it should recognize nearby justifications and avoid false negatives in common formatting patterns.

Required behavior:

* Accept `// SAFETY:` comments that are:

  * immediately above the `unsafe` block
  * on the same line
  * within a small window (e.g., within the preceding 2 lines) if separated by blank line or short comment
* If an unsafe block is inside a function documented with a `# Safety` section, allow an opt-in mode to satisfy the requirement (still strict, but not brittle).

Deliverables:

* Improve detection of SAFETY comment adjacency.
* Add tests with common Rust idioms (including code formatted by rustfmt).
* Ensure Dioxus cases with correct justifications stop being flagged once a proper comment exists.

**Resolution:**

---

## [11] Meaningful performance signals: downgrade/disable heuristic-only P01/P02 unless allocation is proven

**Status:** OPEN
**Files:** `src/analysis/patterns/performance.rs`, tests

Neti’s P01/P02 heuristics fire frequently in real codebases (cloning Arcs, cheap string conversions, etc). To be antifragile, Neti should only escalate when it can establish material cost, otherwise it becomes “perf lint spam.”

Required behavior:

* For `.clone()` in loop:

  * Only escalate when the cloned type is likely heap-owning (e.g., `String`, `Vec`, `HashMap`, `Box`, `Rc`), or when clone is nested in an inner hot loop and cannot be hoisted.
  * For `Arc::clone` / `Rc::clone`, default to INFO/WARN unless additional evidence exists.
* For `.to_string()`/string conversions in loops:

  * Treat as WARN only when inside loop and result is not immediately consumed in a way that requires allocation.
* Always include “why we think this is expensive” in analysis.

Deliverables:

* Add lightweight type heuristics based on token/path (no full type inference required).
* Add tests that:

  * don’t flag `Arc::clone` in loop by default
  * do flag `String::to_string()` in loop when clearly alloc-heavy

**Resolution:**

---

## [12] Governance-grade clippy integration: separate “lint hygiene” from “merge blockers”

**Status:** OPEN
**Files:** `src/verification/runner.rs`, `src/cli/audit.rs` (if relevant), docs/README, tests

You’ve discovered that `cargo clippy -- -D warnings` on mature crates can explode due to lints the project does not treat as deny-by-default. Neti needs a robust approach: **Neti should not force an ecosystem-wide policy**, but it should support governance.

Required behavior:

* Provide first-class support for two tiers of command enforcement:

  1. **Blockers**: compile/test failures, security-critical patterns, explicit deny-lists
  2. **Hygiene**: clippy pedantic/nursery/etc as non-blocking or separately summarized
* Allow commands to be marked as `mode = "error" | "warn"` in Neti config (without special-casing any repo).
* Ensure the report compression (Issue [1]) groups these clearly.

Deliverables:

* Extend `[commands]` schema to allow severity / mode per command.
* Report should clearly show:

  * “BLOCKING” sections
  * “NON-BLOCKING / HYGIENE” sections
* Add tests on runner behavior.

**Resolution:**

---

## [13] Asset and non-source file classification: stop applying code laws to non-code artifacts

**Status:** OPEN
**Files:** `src/discovery.rs`, `src/lang.rs`, `src/analysis/engine.rs` (or wherever file typing occurs), tests

Neti is currently applying LAW OF ATOMICITY and other code-focused checks to large artifacts like HTML, JSON schema, and bundled/minified JS. That’s not a “large repo special case”—it’s a correctness issue: those aren’t code units governed by the same rules.

Required behavior:

* Classify files into:

  * **source code** (Rust/Python/TS/etc)
  * **config** (toml/yaml)
  * **assets** (html/json/svg/minified js/css/etc)
* Only apply structural laws (tokens, cognitive complexity, naming, etc.) to **source code** by default.
* Still allow scanning assets for specific checks where meaningful (e.g., secrets detection), but **do not** treat them as atomicity violations.

Deliverables:

* Implement file classification centrally.
* Add tests:

  * `.html` and `.json` do not trigger token-limit violations
  * Rust/TS/Py still do
  * secrets scanning can still include assets if enabled

**Resolution:**

---

## [14] Dioxus calibration gate: prove “all green means real green” with a locked regression suite

**Status:** OPEN
**Files:** `tests/` (new), `src/analysis/*`, `src/discovery.rs`, CI config (optional)

Once the above fixes land, we need a stable regression target to prevent backsliding. Dioxus is a perfect real-world corpus.

Required behavior:

* Add a regression test harness that runs Neti scan against a pinned Dioxus snapshot (or a small curated fixture extracted from Dioxus that includes the problematic constructs).
* The suite should assert:

  * no “syntax error” false positives
  * no C03/X02 misclassifications
  * no asset token-limit violations
  * indexing proofs behave correctly

Deliverables:

* Add fixtures (small extracted files) to avoid huge repo vendoring.
* Add CI job that runs these regression tests.

**Resolution:**

---

### Notes on ordering

If you want to get to “Dioxus green OR real problems” fastest, the order is:

1. **[7] parser correctness** (everything depends on this)
2. **[13] file classification** (removes asset noise correctly)
3. **[8]/[9] indexing + boundary correctness** (removes lots of false positives)
4. **[5]/[6] (already in your list)** C03/X02 correctness
5. **[12] command tiering** + **[1] compression** (quality of life + governance)
