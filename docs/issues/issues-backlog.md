# BACKLOG Issues

## Label Set

Use only these labels across active and backlog issues:
`Accuracy`, `Config`, `CLI`, `Reporting`, `AI Workflow`, `Adoption`, `Architecture`, `Cleanup`, `Language Support`, `Detection Rules`, `Testing`, `Performance`, `Safety`, `Branching`, `Web Stack`, `Integrations`

---

## [32] Config hygiene: audit and wire preferences
**Status:** OPEN
**Files:** `src/config/types.rs`, `src/cli/handlers/mod.rs`
**Labels:** Config, Cleanup, CLI
**Depends on:** none

**Problem:** Several preferences such as `auto_promote` and `progress_bars` appear to exist in config surfaces without a clear, verified runtime effect. That creates misleading UX and hidden maintenance debt.

**Fix:**

1. Audit every config preference exposed to users.
2. For each preference, either implement observable behavior with tests or remove it.
3. Eliminate any config entry that looks supported but is actually dead.
4. Document the final supported preference set through code and tests.

**Resolution:**

---

## [33] Remove or expose `cli/audit.rs`
**Status:** OPEN
**Files:** `src/cli/audit.rs`, `src/cli/args.rs`
**Labels:** CLI, Cleanup
**Depends on:** none

**Problem:** `src/cli/audit.rs` exists, but `Audit` is not exposed through the `Commands` enum. That leaves dead or half-finished CLI surface area in the tree.

**Fix:**

1. Decide whether `audit` is a real command or not.
2. If yes, wire it into argument parsing and the command dispatch path.
3. If no, remove the dormant module until the feature is ready.
4. Add or update tests so the command surface matches the codebase.

**Resolution:**

---

## [10] Governance-grade clippy integration
**Status:** OPEN
**Files:** `src/verification/runner.rs`, `src/config/types.rs`, tests
**Labels:** Integrations, Config, Testing
**Depends on:** none

**Problem:** Clippy should be a first-class governance stage, but the integration needs stable parsing and configurable severity instead of brittle string matching.

**Fix:**

1. Promote clippy to an explicit verification stage.
2. Support config-driven warn-vs-fail behavior.
3. Parse clippy output using a stable approach rather than ad hoc string matching.
4. Add tests proving severity handling and parser stability.

**Resolution:**

---

## [24] Root `src/` cleanup and domain consolidation
**Status:** OPEN
**Files:** `src/discovery.rs`, `src/file_class.rs`, `src/project.rs`, `src/detection.rs`, `src/constants.rs`, `src/reporting.rs`, `src/lib.rs`
**Labels:** Architecture, Cleanup
**Depends on:** none

**Problem:** Too many unrelated concerns live directly under `src/`, which weakens discoverability and muddies domain boundaries.

**Fix:**

1. Consolidate filesystem and project discovery into a `workspace` module.
2. Move `src/reporting.rs` to `src/reporting/mod.rs` for consistency with the rest of the tree.
3. Reduce top-level sprawl so responsibilities are grouped by domain rather than history.
4. Verify imports and public module boundaries remain coherent after the move.

**Resolution:**

---

## [34] Add `neti rules` catalog command
**Status:** OPEN
**Files:** `src/cli/args.rs`, `src/reporting/guidance.rs`
**Labels:** CLI, Reporting, Detection Rules
**Depends on:** none

**Problem:** Users have no in-product way to browse rule codes, severity semantics, thresholds, fix guidance, or suppression syntax. That raises adoption friction and pushes people into docs hunting.

**Fix:**

1. Add a `neti rules` command.
2. List all rule codes with severity and confidence semantics.
3. Include thresholds, fix guidance, and suppression syntax.
4. Make the catalog searchable or filterable enough to be useful in real workflows.

**Resolution:**

---

## [35] Add SARIF output format
**Status:** OPEN
**Files:** `src/reporting/mod.rs`, `src/cli/args.rs`
**Labels:** Reporting, Integrations
**Depends on:** none

**Problem:** SARIF output is needed for GitHub and GitLab annotations, but Neti currently only provides its own report formats.

**Fix:**

1. Add SARIF as an output option.
2. Map Neti rule code to SARIF `ruleId`.
3. Map confidence and severity to SARIF levels.
4. Map file and line information into SARIF regions.
5. Keep Neti JSON as the canonical internal representation and derive SARIF from it.

**Resolution:**

---

## [36] Add `neti init` scaffolding command
**Status:** OPEN
**Files:** `src/cli/args.rs`, `src/project.rs`
**Labels:** CLI, Config, Adoption
**Depends on:** [49]

**Problem:** Users need a fast path to a sensible initial configuration once they decide to customize Neti beyond default behavior.

**Fix:**

1. Add `neti init`.
2. Generate `neti.toml` and `.netiignore` based on detected project type.
3. Support `--strict` and `--lenient` presets.
4. Optionally generate `CHAT-PROTOCOL.md`.

**Resolution:**

---

## [37] LCOM4 miscalibrated for delegation patterns
**Status:** OPEN
**Files:** `src/analysis/structural.rs`, `src/config/mod.rs`
**Labels:** Accuracy, Detection Rules, Architecture
**Depends on:** [30]

**Problem:** Existing "touch fields for LCOM4" hacks suggest the metric is too sensitive to pure delegation patterns, which reduces trust in the signal.

**Fix:**

1. Revisit the LCOM4 calculation for delegator-heavy modules.
2. Either exempt the affected module categories, improve the computation, or rely on an explicit suppression path.
3. Prefer a real metric fix over a blanket carve-out when practical.
4. Add tests proving the chosen approach distinguishes delegation from low cohesion.

**Resolution:**

---

## [21] Python `LangSemantics` table
**Status:** OPEN
**Files:** `src/lang/semantics.rs`
**Labels:** Language Support, Architecture, Detection Rules
**Depends on:** [17]

**Problem:** The language abstraction needs a Python semantics table before detector behavior can be extended coherently to Python codebases.

**Fix:**

1. Add Python test markers such as the `test_` prefix.
2. Add heap-type vocabulary such as `list`, `dict`, and `set`.
3. Add search and length vocabulary such as `index`, `find`, and `len`.
4. Add loop and syntax markers such as `for_statement` and `while_statement`.

**Resolution:**

---

## [22] TypeScript `LangSemantics` table
**Status:** OPEN
**Files:** `src/lang/semantics.rs`
**Labels:** Language Support, Architecture, Detection Rules, Web Stack
**Depends on:** [17]

**Problem:** The language abstraction also needs a TypeScript semantics table before detector reuse can extend cleanly to JS/TS projects.

**Fix:**

1. Add test markers such as `describe`, `it`, and `test`.
2. Add heap-type vocabulary such as `Array`, `Map`, and `Set`.
3. Add search vocabulary such as `find` and `indexOf`.
4. Add length-property semantics such as `length`.

**Resolution:**

---

## [12] Cross-language regression suite
**Status:** OPEN
**Files:** `tests/` (new), CI config
**Labels:** Testing, Language Support, Detection Rules
**Depends on:** [15], [21], [22]

**Problem:** Neti needs a fixture-backed regression suite that proves equivalent rules behave consistently across supported languages.

**Fix:**

1. Add Rust fixtures covering syntax suppressions, L03 tiers, and P04 numeric iteration.
2. Add Python fixtures once [21] lands.
3. Add TypeScript fixtures once [22] lands.
4. Assert that equivalent patterns produce equivalent findings across languages where the rule intent matches.

**Resolution:**

---

## [38] HTML monolith analysis and split recommendations
**Status:** OPEN
**Files:** `src/cli/args.rs`, `src/cli/handlers/mod.rs`, `src/analysis/html.rs` (new), `src/graph/locality/` (extend)
**Labels:** Web Stack, Architecture, CLI, Reporting
**Depends on:** [39], [40]

**Problem:** Large single-file HTML apps with inline `<script>` and `<style>` blocks are common, but Neti cannot currently analyze them or recommend structural splits.

**Fix:**

1. Parse HTML with `tree-sitter-html` and extract `<script>` and `<style>` blocks.
2. Run existing JS or TS definition extraction on script content.
3. Cluster functions by shared variable access, reusing coupling analysis where possible.
4. Cluster CSS rules by selector patterns such as layout, components, and theme.
5. Add `neti split <file.html>` and emit split recommendations with rationale.

**Resolution:**

---

## [39] Inline script/style analysis without split recommendation
**Status:** OPEN
**Files:** `src/analysis/engine.rs`, `src/analysis/html.rs` (new)
**Labels:** Web Stack, Detection Rules, Architecture
**Depends on:** none

**Problem:** Before Neti can recommend structural splits for HTML monoliths, it needs baseline support for scanning inline scripts and styles with correct source mapping.

**Fix:**

1. Extract `<script>` content during scan.
2. Run JS or TS pattern detectors against the extracted script.
3. Report violations using correct line offsets back into the HTML file.
4. Add monolith-size suggestions, such as warning when inline JS grows past a threshold.

**Resolution:**

---

## [40] CSS cohesion analysis
**Status:** OPEN
**Files:** `src/analysis/css.rs` (new), `src/lang.rs`
**Labels:** Web Stack, Detection Rules, Language Support
**Depends on:** none

**Problem:** CSS currently has no dedicated analysis support, which limits both standalone governance and any future HTML split recommendations.

**Fix:**

1. Add `tree-sitter-css` support.
2. Implement selector clustering by prefix and concern.
3. Add specificity warnings to catch overly coupled selectors.
4. Add dead-code detection for selectors with no HTML match in the same file.
5. Add variable locality analysis for `--var` definitions and usage proximity.

**Resolution:**

---

## [41] SWUM-style identifier expansion for JS/TS
**Status:** OPEN
**Files:** `src/analysis/naming.rs` (extend), `src/lang.rs`
**Labels:** Web Stack, Language Support, Detection Rules
**Depends on:** [22]

**Problem:** JS and TS naming analysis would benefit from SWUM-style identifier expansion so naming guidance can reason about verbs, acronyms, and intent rather than raw token shape.

**Fix:**

1. Port SEMMAP's SWUM-style identifier expansion into Neti's naming analysis.
2. Use it for naming convention consistency checks.
3. Add verb-first function name suggestions.
4. Add acronym detection and expansion hints.

**Resolution:**

---

## [42] Report file optimized for LLM consumption
**Status:** OPEN
**Files:** `src/reporting/rich.rs`, `src/cli/handlers/check_report.rs`
**Labels:** AI Workflow, Reporting
**Depends on:** [29]

**Problem:** `neti-report.txt` is the primary handoff surface between Neti and an AI agent doing remediation work, but the current structure is not explicitly optimized for that use case.

**Fix:**

1. Front-load the verdict with `PASS` or `FAIL`.
2. Group findings by file and then by rule.
3. Inline fix hints next to each violation rather than isolating them elsewhere.
4. Guarantee the report file is ANSI-free.
5. Cap report length and summarize the tail when the finding count is very large.

**Resolution:**

---

## [43] Summary one-liner as last line of output
**Status:** OPEN
**Files:** `src/cli/handlers/mod.rs`
**Labels:** CLI, Reporting, AI Workflow
**Depends on:** none

**Problem:** Console output often gets truncated in agent tooling and chat UIs. If the final line is not a concise verdict, the important result is easy to miss.

**Fix:**

1. Always print a single summary line as the last line of `neti check`.
2. Include pass/fail, key counts, and runtime.
3. On failure, point directly to `neti-report.txt`.
4. Keep the format stable so humans and tools can both rely on it.

**Resolution:**

---

## [44] Stage timing in report
**Status:** OPEN
**Files:** `src/cli/handlers/mod.rs`, `src/cli/handlers/check_report.rs`
**Labels:** Reporting, CLI, Performance
**Depends on:** none

**Problem:** Neti does not currently make stage timing visible, which hides slow paths and makes CI output less informative.

**Fix:**

1. Add timing for scan, locality, commands, and total runtime.
2. Include useful per-stage counts alongside the timing output.
3. Surface the data in a format that works both in terminals and CI logs.

**Resolution:**

---

## [45] Human-friendly violation messages alongside machine codes
**Status:** OPEN
**Files:** `src/reporting/rich.rs`, `src/reporting/guidance.rs`
**Labels:** Reporting, AI Workflow
**Depends on:** none

**Problem:** Neti findings need to serve two audiences at once: machine parsing and human comprehension. Right now the report does not clearly separate those two voices.

**Fix:**

1. Keep a stable machine-readable line for every violation.
2. Add a human-readable explanation line immediately after it.
3. Ensure the human line reads like actionable guidance rather than raw metadata.
4. Keep both lines adjacent so people and tools can consume the same report.

**Resolution:**

---

## [46] `neti check --changed-only` for incremental feedback
**Status:** OPEN
**Files:** `src/cli/args.rs`, `src/analysis/engine.rs`, `src/cli/handlers/mod.rs`
**Labels:** CLI, Performance, AI Workflow
**Depends on:** none

**Problem:** Full-repo scans are expensive when an agent has only changed a few files. Incremental feedback should be faster and more focused where that is semantically safe.

**Fix:**

1. Add `--changed-only` and `--since <ref>` flags.
2. Scope scan-level checks to files changed since the target ref.
3. Keep locality analysis full-graph because partial locality analysis would be misleading.
4. Document the exact contract so users know what is and is not incremental.

**Resolution:**

---

## [47] Ungoverned file type warning
**Status:** OPEN
**Files:** `src/analysis/engine.rs`, `src/reporting/rich.rs`
**Labels:** Reporting, Adoption, Language Support
**Depends on:** none

**Problem:** A clean scan can create false confidence when large parts of the repo use file types Neti does not analyze yet.

**Fix:**

1. Count file types that Neti cannot currently govern.
2. Emit a concise informational summary after the scan.
3. Make the wording explicit that coverage is incomplete, not clean.

**Resolution:**

---

## [48] Exit code contract
**Status:** OPEN
**Files:** `src/exit.rs`, `README.md`, tests
**Labels:** CLI, Integrations, Testing
**Depends on:** none

**Problem:** CI systems and agents depend on stable exit code semantics. If those semantics are undocumented or untested, integrations become fragile.

**Fix:**

1. Document exact exit code meanings for success, violations, and runtime/config failure.
2. Add tests that assert each exit code in a known scenario.
3. Keep the contract stable once published.

**Resolution:**

---

## [49] Zero-config first run
**Status:** OPEN
**Files:** `src/config/io.rs`, `src/project.rs`, `src/cli/handlers/mod.rs`
**Labels:** Adoption, Config, CLI
**Depends on:** none

**Problem:** The first-run experience should be useful even when the user has not created `neti.toml`. Requiring manual setup before the first meaningful check adds avoidable friction.

**Fix:**

1. Auto-detect project type from root markers such as `Cargo.toml`, `package.json`, or `go.mod`.
2. Apply sensible defaults based on the detected language or ecosystem.
3. Run scan, locality, and detected commands with those defaults.
4. Print a note explaining that auto-detected defaults are in effect and `neti init` can customize them.

**Resolution:**
