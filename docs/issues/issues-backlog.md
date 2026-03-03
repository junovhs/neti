# BACKLOG Issues

---

## [32] Config hygiene: audit and wire preferences
**Status:** OPEN
**Files:** `src/config/types.rs`, `src/cli/handlers/mod.rs`
Preferences like `auto_promote`, `progress_bars` exist but may not be fully wired. Audit each preference — either implement it with observable behavior and test, or remove to avoid misleading UX.
**Resolution:**

---

## [33] Remove or expose `cli/audit.rs`
**Status:** OPEN
**Files:** `src/cli/audit.rs`, `src/cli/args.rs`
Module exists but `Audit` isn't in `Commands` enum. Either wire it up or delete until ready.
**Resolution:**

---

## [10] Governance-grade clippy integration
**Status:** OPEN
**Files:** `src/verification/runner.rs`, `src/config/types.rs`, tests
Make clippy a first-class governance stage with tiering (warn vs fail) via config and stable parsing (not brittle string matching).
**Resolution:**

---

## [24] Root src/ cleanup and domain consolidation
**Status:** OPEN
**Files:** `src/discovery.rs`, `src/file_class.rs`, `src/project.rs`, `src/detection.rs`, `src/constants.rs`, `src/reporting.rs`, `src/lib.rs`
Consolidate filesystem/project discovery into a `workspace` module. Move `src/reporting.rs` to `src/reporting/mod.rs` for consistency. Reduce top-level sprawl.
**Resolution:**

---

## [34] Add `neti rules` catalog command
**Status:** OPEN
**Files:** `src/cli/args.rs`, `src/reporting/guidance.rs`
List all rule codes with severity, confidence semantics, thresholds, fix guidance, and suppression syntax. Searchable in-product catalog reduces friction.
**Resolution:**

---

## [35] Add SARIF output format
**Status:** OPEN
**Files:** `src/reporting/mod.rs`, `src/cli/args.rs`
SARIF enables GitHub/GitLab PR annotations. Map rule code → ruleId, confidence → level, file/line → region. Keep JSON as canonical; SARIF is derived.
**Resolution:**

---

## [36] Add `neti init` scaffolding command
**Status:** OPEN
**Files:** `src/cli/args.rs`, `src/project.rs`
Generate `neti.toml` + `.netiignore` based on detected project type. Support `--strict` vs `--lenient` presets. Optionally generate `CHAT-PROTOCOL.md`.
**Resolution:**

---

## [37] LCOM4 miscalibrated for delegation patterns
**Status:** OPEN
**Files:** `src/analysis/structural.rs`, `src/config/mod.rs`
The "Touch fields for LCOM4" hacks indicate the metric is too sensitive to pure delegators. Either exempt config/CLI modules, adjust computation for delegation patterns, or add `// neti:allow(LCOM4)` mechanism.
**Resolution:**

---

## [21] Python `LangSemantics` table
**Status:** OPEN
**Files:** `src/lang/semantics.rs`
Add Python semantics: `test_` prefix, `list`/`dict`/`set` heap types, `index`/`find` search methods, `len` length method, `for_statement`/`while_statement` loops.
**Resolution:**

---

## [22] TypeScript `LangSemantics` table
**Status:** OPEN
**Files:** `src/lang/semantics.rs`
Add TypeScript semantics: `describe`/`it`/`test` markers, `Array`/`Map`/`Set` heap types, `find`/`indexOf` search methods, `length` property.
**Resolution:**

---

## [12] Cross-language regression suite
**Status:** OPEN
**Files:** `tests/` (new), CI config
Create test fixtures:
- `tests/fixtures/rust/` — syntax suppressions, L03 tiers, P04 numeric iteration
- `tests/fixtures/python/` — equivalent patterns once [21] lands
- `tests/fixtures/typescript/` — equivalent patterns once [22] lands

Same rules should fire on equivalent patterns across languages.
**Resolution:**

---

## [38] HTML monolith analysis and split recommendations
**Status:** OPEN
**Files:** `src/cli/args.rs`, `src/cli/handlers/mod.rs`, `src/analysis/html.rs` (new), `src/graph/locality/` (extend)

Single-file HTML applications with inline `<script>` and `<style>` blocks are common entry points for projects that outgrow their structure. Neti should analyze these monoliths and recommend splits.

**Required:**
1. Parse HTML with tree-sitter-html, extract `<script>` and `<style>` blocks
2. Run existing JS/TS definition extraction on script content
3. Cluster functions by shared variable access (reuse coupling analysis)
4. Cluster CSS rules by selector patterns (layout vs components vs theme)
5. Emit split recommendations with rationale

**Command:** `neti split <file.html>`

**Output format:**
```
Analysis:
  <script> block: 847 lines, 12 functions, 3 cohesion clusters
  <style> block: 234 lines, 2 distinct concerns

Recommended structure:
  src/
    js/
      auth.js      ← login(), logout(), checkSession() [share userState]
      ui.js        ← render(), update() [share domRefs]
    css/
      layout.css   ← structural selectors
      components.css ← .btn, .card, .modal
    index.html     ← markup only
```

**Resolution:**

---

## [39] Inline script/style analysis without split recommendation
**Status:** OPEN
**Files:** `src/analysis/engine.rs`, `src/analysis/html.rs` (new)

Before full split recommendations ([38]), support basic `neti scan` on HTML files:
- Extract `<script>` content, run JS/TS pattern detectors
- Report violations with correct line offsets (HTML line + script offset)
- Flag monolith size thresholds (e.g., >200 lines of inline JS triggers suggestion)

This enables governance on legacy HTML without requiring immediate refactoring.

**Resolution:**

---

## [40] CSS cohesion analysis
**Status:** OPEN
**Files:** `src/analysis/css.rs` (new), `src/lang.rs`

Add tree-sitter-css support for:
- Selector clustering (group by prefix: `.nav-*`, `.btn-*`, layout selectors)
- Specificity warnings (overly specific selectors indicate coupling)
- Dead code detection (selectors with no HTML match in same file)
- Variable locality (`--var` definitions and usage proximity)

Required for [38] split recommendations and standalone CSS governance.

**Resolution:**

---

## [41] SWUM-style identifier expansion for JS/TS
**Status:** OPEN
**Files:** `src/analysis/naming.rs` (extend), `src/lang.rs`

Port SEMMAP's SWUM identifier expansion to Neti for:
- Naming convention checks (camelCase vs snake_case consistency)
- Verb-first function name suggestions
- Acronym detection and expansion hints

Low priority — nice-to-have for style governance.

**Resolution:**

---

## [42] Report file optimized for LLM consumption
**Status:** OPEN
**Files:** `src/reporting/rich.rs`, `src/cli/handlers/check_report.rs`

`neti-report.txt` is the primary interface between Neti and whatever AI is doing the fixing. Audit for LLM-friendliness:

1. **Front-load the verdict.** First line: `PASS` or `FAIL (N errors, M warnings)`. Not buried after headers.
2. **Group by file, then by rule.** LLMs fix file-by-file.
3. **Inline the fix hint** with each violation (WHY/FIX from guidance.rs), not in a separate educational section. The AI needs fix context adjacent to the error.
4. **Strip ANSI.** Report file must never contain terminal color escapes.
5. **Cap length.** If violations exceed 200, summarize tail as `... and N more`. Context windows are finite.

Test: token count before/after on a noisy real-world scan.

**Resolution:**

---

## [43] Summary one-liner as last line of output
**Status:** OPEN
**Files:** `src/cli/handlers/mod.rs`

Always print a single summary as the very last line of `neti check`:

```
neti: PASS (247 files, 0 errors, 2 warnings) in 6.1s
```
or
```
neti: FAIL (3 errors, 1 warning) — see neti-report.txt
```

Console output gets truncated by agents and chatbot UIs. If the summary is last, it's always visible. The user (or AI) sees pass/fail immediately and knows whether to dig into the report.

**Resolution:**

---

## [44] Stage timing in report
**Status:** OPEN
**Files:** `src/cli/handlers/mod.rs`, `src/cli/handlers/check_report.rs`

Add timing to each stage:
```
Scan:      0.8s (247 files, 3 violations)
Locality:  1.2s (189 edges, 0 violations)
Commands:  4.1s (2/2 passed)
Total:     6.1s
```

Humans see if a stage is unexpectedly slow. Agents learn what's cheap vs expensive. Makes output look professional in CI logs.

**Resolution:**

---

## [45] Human-friendly violation messages alongside machine codes
**Status:** OPEN
**Files:** `src/reporting/rich.rs`, `src/reporting/guidance.rs`

Every violation should have two voices:

**Machine line** (for AI parsing):
```
FAIL src/foo.rs:42 [COMPLEXITY-01] cognitive_complexity=18 limit=15
```

**Human line** (for the person watching):
```
  → This function is doing too many things. Try splitting it into smaller pieces.
```

The machine line uses stable codes. The human line reads like advice from a mentor. Both appear in the report. The AI parses the first, the person reads the second. Neither needs to understand the other's line.

**Resolution:**

---

## [46] `neti check --changed-only` for incremental feedback
**Status:** OPEN
**Files:** `src/cli/args.rs`, `src/analysis/engine.rs`, `src/cli/handlers/mod.rs`

Scan only files changed since a git ref (default: `HEAD`). Uses `git diff --name-only` internally.

When an agent edits 3 files and runs `neti check`, scanning the whole codebase is wasteful. This keeps the report focused on what actually changed.

`--changed-only` diffs against HEAD. `--since <ref>` diffs against arbitrary ref.

Important: locality must still analyze the full graph (can't detect cycles from partial view). Only scan-level checks (complexity, patterns) get scoped.

**Resolution:**

---

## [47] Ungoverned file type warning
**Status:** OPEN
**Files:** `src/analysis/engine.rs`, `src/reporting/rich.rs`

After a scan, if the project contains file types Neti can't analyze yet, emit one info line:

```
info: 47 .py files and 23 .ts files not yet covered by Neti
```

Prevents false confidence from a clean scan that only checked half the codebase. Honest about coverage gaps. Also serves as organic signal for which language parity work matters most.

**Resolution:**

---

## [48] Exit code contract
**Status:** OPEN
**Files:** `src/exit.rs`, `README.md`, tests

Document and test exact exit code semantics:
- `0` — all stages pass
- `1` — violations found
- `2` — config error or runtime failure

Agents and CI key off exit codes. If these aren't stable and tested, integrations break silently. Integration tests should assert each code for known scenarios.

**Resolution:**

---

## [49] Zero-config first run
**Status:** OPEN
**Files:** `src/config/io.rs`, `src/project.rs`, `src/cli/handlers/mod.rs`

`neti check` with no `neti.toml` should do something useful:
- Auto-detect project type from root markers (Cargo.toml, package.json, go.mod, etc.)
- Apply sensible defaults for detected language (Rust → clippy in commands, complexity limits, locality warn mode)
- Run scan + locality + detected commands
- Print a note: `Using auto-detected defaults for Rust project. Run neti init to customize.`

The user's first experience should be: `cd my-project && neti check` → useful results. No config file, no docs, no flags. If they want to tune, `neti init` scaffolds the config. But the default path just works.

**Resolution:**

---

## [50] Configurable report path
**Status:** OPEN
**Files:** `src/config/types.rs`, `src/cli/handlers/mod.rs`

Allow `neti.toml` to specify report location:
```toml
[output]
report_path = ".neti/report.txt"
```

Default stays `neti-report.txt` at root. Some teams don't want governance artifacts in root. Some agents expect output in specific locations. Small config, real QoL.

**Resolution:**

---

## [51] `neti rules` catalog command
**Status:** OPEN
**Files:** `src/cli/args.rs`, `src/reporting/guidance.rs`

(Moved from backlog [34] — renumbered for continuity.)

List all rule codes with severity, confidence, thresholds, fix guidance, and examples. Searchable catalog inside the tool itself. When the report says `[P01]` and the user wants to understand why, `neti rules P01` gives the full explanation without leaving the terminal.

**Resolution:**
