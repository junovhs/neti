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
