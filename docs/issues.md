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
- Work issues in order you feel is most important.
- Update status as you go
- Add **Resolution:** when completing
- Don't modify this FORMAT section
- Content below the line is the work.
---

## [1] Smart output compression for neti-report.txt
**Status:** OPEN
**Files:** `src/verification/runner.rs`, potentially new `src/reporting/compression.rs`

Clippy and other tools can output 10,000 lines for what amounts to 2 distinct issues repeated across many call sites. AI agents waste context window on this noise. `neti-report.txt` should be maximally informative but succinct.

Approach:
- Group violations/errors by kind (error code + message template), not by occurrence
- For each kind: show the first 2-3 instances with full context, then a count of remaining occurrences with just file:line references
- Example:
  ```
  E0308 (mismatched types) — 47 occurrences across 12 files
    src/foo.rs:42  expected `String`, found `&str`
    src/bar.rs:88  expected `String`, found `&str`
    ... and 45 more (src/baz.rs:12, src/qux.rs:55, ...)
  ```
- Must work for ANY command output in the `[commands]` pipeline — not clippy-specific
- Write compression logic as a shared utility

**Resolution:**

---

## [2] Locality integration into standard scan pipeline
**Status:** OPEN
**Files:** `src/graph/locality/` (all), `src/cli/locality.rs`, `src/config/locality.rs`

Locality has a massive subgraph (classifier, coupling, cycles, distance, edges, exemptions, layers, validator, types, report, analysis/metrics, analysis/violations) but feels separate from the main scan pipeline.

Questions to resolve:
- Should locality be a first-class `neti check` phase (runs automatically), or remain an opt-in `neti locality` command?
- Is the implementation providing value proportional to its code size?
- Can locality violations be surfaced as regular scan violations (same format, same report) rather than a separate report?

Recommendation: integrate locality results into the standard scan report format. Keep the graph engine as-is but unify the output.

**Resolution:**

---

## [3] Mutation testing kill rate
**Status:** OPEN
**Files:** `src/mutate/` (all), tests

Current mutation testing exists but kill rate is unknown. Target: 70%+ kill rate. This requires meaningful test coverage of core analysis logic — engine phase separation, violation detection contracts, `Violation`/`FileReport` constructors.

Tasks:
- Run `neti mutate` against self, establish baseline kill rate
- Add contract tests for core types
- Add regression tests for two-phase engine invariant (parallel local → sequential deep)
- Iterate until 70%+ kill rate achieved

**Resolution:**

---

## [4] Python and TypeScript pattern detection parity
**Status:** OPEN
**Files:** `src/analysis/patterns/` (all), `src/lang.rs`

Rust has full pattern detection. Python and TypeScript are marked "Partial" in the README. Close the gap on the most impactful patterns:
- Concurrency patterns where applicable
- Security patterns (injection, credential exposure)
- Performance patterns (allocation in loops, linear search in loops)
- Idiomatic patterns per language

**Resolution:**





scratchpad need to fix later: 

getting some C03 violations on Dioxus as false positives from Neti. Which is actually useful to know — it means Neti needs a smarter check that distinguishes std::sync::Mutex from tokio::sync::Mutex / futures_util::lock::Mutex.



Neti incorrectly flags `futures_util::lock::Mutex` (async mutex) as `std::sync::MutexGuard held across .await`, reporting “critical deadlock risk” in async code (e.g., Dioxus `packages/fullstack/src/payloads/websocket.rs`). Detection should distinguish sync vs async mutexes and downgrade/adjust guidance for async locks (reentrancy/starvation vs thread-blocking deadlock).
