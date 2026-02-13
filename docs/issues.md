# issues-0002: Nim Integration + Consolidation Sweep
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
- Content below the line is the work. When done, archive in docs/archive and create next issues doc.
---

## Context

This issue set covers two tracks:
- **Track A (Carried from issues-0001):** Consolidation, cleanup, and refocusing of SlopChop core.
- **Track B (New):** Tier-1 Nim language support per `docs/SlopChop_Nim_Tier1_Spec.md`.

Track A should be completed first where items directly affect Track B (e.g., cutting v1 remnants, reframing the analysis engine). Items are ordered by dependency, not priority number.

---

## [1] Reframe v2 analysis engine as THE analysis engine
**Status:** OPEN
**Files:** `src/analysis/v2/` (all), `src/analysis/mod.rs`, `src/analysis/ast.rs`, `src/analysis/engine.rs`, `src/analysis/logic.rs`, `src/analysis/file_analysis.rs`, `src/analysis/checks.rs`, `src/analysis/metrics.rs`, `src/analysis/safety.rs`
**Blocking:** [9], [10], [11]

Remove all traces of "v2" naming. The v2 engine IS the engine. This must land before Nim integration because new Nim code should not be written into a `/v2/` namespace that's about to be renamed.

Tasks:
- Rename `src/analysis/v2/` contents into `src/analysis/` proper (or a clean namespace like `src/analysis/engine/`)
- Remove or absorb any remaining v1 analysis code (`ast.rs`, `checks.rs`, `metrics.rs`, `safety.rs`, `engine.rs`, `logic.rs`, `file_analysis.rs`) — determine what's still live vs dead
- Update all internal imports and references
- Update the scan spec doc to remove "v2" language
- Run `slopchop check` against self to verify nothing broke

**Resolution:**

---

## [2] Cut packing functionality
**Status:** OPEN
**Files:** `src/pack/mod.rs`, `src/pack/docs.rs`, `src/pack/focus.rs`, `src/pack/formats.rs`, `src/pack/xml_format.rs`, `src/cli/handlers/mod.rs` (handle_pack), `src/cli/args.rs` (pack command definition)
**Blocking:** [3]

SEMMAP replaces packing. Remove all pack-related code:
- Delete `src/pack/` module entirely
- Remove `handle_pack` from CLI handlers
- Remove `PackArgs` and pack command from `src/cli/args.rs`
- Remove pack-related exports from `src/lib.rs`
- Clean up any `FocusContext`, `OutputFormat`, `PackOptions` references elsewhere

**Resolution:**

---

## [3] Cut slopchop map and map --deps
**Status:** OPEN
**Files:** `src/map.rs`, `src/cli/handlers/mod.rs` (handle_map), `src/cli/args.rs` (map command definition)
**Depends on:** [2]

Separate app handles this better now. Remove:
- Delete `src/map.rs`
- Remove `handle_map` from CLI handlers
- Remove map command from args
- Remove map-related exports from `src/lib.rs`
- Clean up `src/discovery.rs` if it was only used by map (check — it may still be used by scan/analysis)

**Resolution:**

---

## [4] Evaluate prompt mechanisms post-pack removal
**Status:** OPEN
**Files:** `src/prompt.rs`, `AGENT-README.md`
**Depends on:** [2], [3]

With pack and map cut, the prompt generation system (`src/prompt.rs`) may have dead code paths or references to removed features. Evaluate:
- Is `src/prompt.rs` still needed at all, or does `AGENT-README.md` in root fully replace it?
- If keeping prompt.rs: strip references to pack, context generation, and anything that assumes the old chat-based workflow
- If the autonomous protocol markdown (AGENT-README.md) is the primary interface, ensure it's the source of truth and prompt.rs just generates/updates it
- Update prompt content for Nim support (see Nim spec Section 6)

**Resolution:**

---

## [5] Smart clippy/compiler output compression
**Status:** OPEN
**Files:** `src/cli/handlers/scan_report.rs`, `src/apply/report_writer.rs`, potentially new `src/reporting/compression.rs`

The problem: clippy (and soon `nim check`) can output 10,000 lines for what amounts to 2 distinct issues repeated across many call sites. AI agents waste context window on this noise. `slopchop-report.txt` should be maximally informative but succinct.

Approach:
- Group violations/errors by **kind** (error code + message template), not by occurrence
- For each kind: show the first 2-3 instances with full context, then a count of remaining occurrences with just file:line references
- Example output format:
  ```
  E0308 (mismatched types) — 47 occurrences across 12 files
    src/foo.rs:42  expected `String`, found `&str`
    src/bar.rs:88  expected `String`, found `&str`
    ... and 45 more (src/baz.rs:12, src/qux.rs:55, ...)
  
  W0599 (clippy::clone_on_copy) — 3 occurrences
    src/utils.rs:14  ...
    src/utils.rs:28  ...
    src/utils.rs:91  ...
  ```
- This must work for ANY command output in the `[commands]` check pipeline — not just clippy. When Nim's `nim check` joins, it benefits automatically.
- Write compression logic as a shared utility, not clippy-specific

**Resolution:**

---

## [6] Clipboard character encoding issues
**Status:** OPEN
**Files:** `src/clipboard/mod.rs`, `src/clipboard/linux.rs`, `src/clipboard/macos.rs`, `src/clipboard/windows.rs`, `src/clipboard/utils.rs`, `src/apply/parser.rs`, `src/apply/blocks.rs`

Recurring issue: clipboard transfers corrupt trailing characters, especially in `Cargo.toml` files. Last character often becomes `#`, carriage return errors appear.

Investigation needed:
- Audit the clipboard pipeline: copy → clipboard buffer → paste → parser
- Check for `\r\n` vs `\n` normalization issues (especially Windows → Linux clipboard)
- Check for null byte / EOF marker injection by clipboard tools
- Check if `xclip`/`xsel`/`pbcopy` append trailing newlines or characters
- Add explicit trailing-whitespace/character sanitization in `src/apply/parser.rs` before content is processed
- Add a test that round-trips a known `Cargo.toml` through the clipboard pipeline and asserts byte-identical output

Note: If the apply system is eventually separated ([7]), this fix still applies to whatever module owns clipboard I/O.

**Resolution:**

---

## [7] Evaluate apply/check separation
**Status:** OPEN
**Files:** Architectural decision — no immediate code changes

Question: should `slopchop apply` and `slopchop check` be separate binaries, separate crates in a workspace, or remain unified?

Considerations:
- Apply is about receiving AI output, parsing protocol, writing files, managing branches
- Check is about scanning, analysis, running commands, reporting
- They share: config, project detection, exit codes, reporting format
- Chat-based users (you) want them tightly coupled — apply then auto-checks
- Agentic users want check standalone — agent writes files directly, then runs check
- CI users only need check
- Recommendation: keep as one binary with clean internal module boundaries. The apply system is the "ingest" layer, check is the "verify" layer. They compose but don't depend on each other's internals. This is already mostly true architecturally.

Decision to make: keep unified or split. Document rationale either way.

**Resolution:**

---

## [8] Revisit law of locality elevation
**Status:** OPEN
**Files:** `src/graph/locality/` (all), `src/cli/locality.rs`, `src/config/locality.rs`

Locality has a massive subgraph (classifier, coupling, cycles, distance, edges, exemptions, layers, validator, types, report, analysis/metrics, analysis/violations). It feels weirdly separate from the main scan pipeline.

Questions to resolve:
- Should locality be a first-class `slopchop check` phase (runs automatically like clippy), or an opt-in `slopchop locality` command?
- Is the current implementation actually providing value proportional to its code size?
- Can the locality violations be surfaced as regular scan violations (same format, same report) rather than having their own separate report?
- For Nim support: the edge collection (`src/graph/locality/edges.rs`) and import resolution (`src/graph/imports.rs`, `src/graph/resolver.rs`) need Nim import syntax support. This is simpler if locality is integrated into the standard pipeline.

Recommendation: integrate locality results into the standard scan report format. Keep the graph engine as-is but unify the output so it doesn't feel like a separate tool.

**Resolution:**

---

## [9] Nim: Grammar validation and node discovery (GATE)
**Status:** OPEN
**Files:** `Cargo.toml`, `tests/nim_grammar.rs` (new)
**Depends on:** [1]
**Blocking:** [10], [11], [12], [13], [14], [15]
**Spec ref:** Nim Spec Phase 1

This is the gating step. Everything else depends on this.

Tasks:
- Add `tree-sitter-nim = { git = "https://github.com/alaviss/tree-sitter-nim", tag = "0.6.2" }` to Cargo.toml
- Verify it compiles with existing tree-sitter version
- Write and run `test_nim_grammar_node_discovery` — this prints the actual AST for a sample Nim file
- Document discovered node names for: proc declarations, if/elif/case, import statements, type declarations, exported symbols (the `*` postfix), routine bodies
- Compare discovered names against the queries in the Nim spec Section 3.2
- Update spec queries to match reality
- Write query compilation tests for all 6 query categories
- **STOP HERE if queries don't compile. Do not proceed to [10]+.**

**Resolution:**

---

## [10] Nim: Language integration
**Status:** OPEN
**Files:** `src/lang.rs`, `src/constants.rs`, `src/project.rs`, `src/config/types.rs`
**Depends on:** [1], [9]
**Spec ref:** Nim Spec Sections 3.1, 5

Tasks:
- Add `Nim` variant to `Lang` enum
- Implement `from_ext` for `.nim` / `.nims`
- Implement `grammar()` returning `tree_sitter_nim::language()`
- Implement `skeleton_replacement()` returning `"\n  discard"`
- Add Nim queries to the QUERIES array (using corrected node names from [9])
- Update `CODE_EXT_PATTERN` in constants.rs
- Add `Nim` to `ProjectType` enum
- Implement `.nimble` file detection
- Implement `nim.cfg` / `config.nims` fallback detection
- Add default commands: `nim check`, `nimpretty --check`, `nimble test`
- Implement tool presence detection for `nimpretty`

**Resolution:**

---

## [11] Nim: Paranoia checks
**Status:** OPEN
**Files:** `src/analysis/checks/nim_checks.rs` (new), `src/analysis/ast.rs`
**Depends on:** [1], [9], [10]
**Spec ref:** Nim Spec Section 4

Create `nim_checks.rs` with detection for all 10+ unsafe constructs:

- [ ] `check_cast` — `cast[T](x)`, AST-based with text fallback
- [ ] `check_addr` — `addr()` / `unsafeAddr()`, AST with text fallback
- [ ] `check_ptr_types` — `ptr T` / `pointer` type declarations
- [ ] `check_emit_pragma` — `{.emit.}` inline C injection (text-based)
- [ ] `check_asm_statement` — `asm` blocks, AST with text fallback
- [ ] `check_raw_memory_ops` — `copyMem`, `moveMem`, `zeroMem`, `equalMem`, `cmpMem`
- [ ] `check_manual_alloc` — `alloc`, `alloc0`, `dealloc`, `realloc`, `create(T)` with uppercase heuristic
- [ ] `check_disabled_checks` — `{.push checks:off.}` and 9 other pragma variants
- [ ] `check_noinit` — `{.noinit.}` pragma
- [ ] `check_global_pragma` — `{.global.}` on local `var`/`let`
- [ ] `has_safety_comment` — shared helper, checks current line and line above for `# SAFETY:`
- [ ] Wire up `check_nim_specifics` dispatch in `ast.rs`
- [ ] Profile-aware severity: systems mode relaxes ptr/cast/addr/alloc/memory/noinit to warnings; emit/asm/disabled-checks remain errors

**Resolution:**

---

## [12] Nim: Naming conventions
**Status:** OPEN
**Files:** `src/analysis/checks/naming.rs`
**Depends on:** [10]
**Spec ref:** Nim Spec Section 8

Add Nim naming convention checks:
- Types/objects/enums: must start uppercase (PascalCase)
- Procs/funcs/methods/iterators/converters: must start lowercase (camelCase)
- Exception: backtick-quoted operators (`\`+\``, `\`$\``) are exempt
- Wire into the naming check dispatcher with `Lang::Nim` guard

**Resolution:**

---

## [13] Nim: Strict config generator
**Status:** OPEN
**Files:** `src/project.rs` (or new `src/nim_config.rs`), `src/cli/args.rs`, `src/cli/dispatch.rs`
**Depends on:** [10]
**Spec ref:** Nim Spec Section 5.4

Implement `slopchop init --nim`:
- Generate `nim.cfg` with strict defaults: `--mm:arc`, `--warningAsError:on`, `--checks:on`, `--panics:on`, `--styleCheck:error`, all runtime checks enabled
- Generate `slopchop.toml` with Nim-appropriate defaults
- Generate `.slopchopignore` with Nim-specific entries (`nimcache/`, `nimblecache/`)
- If `slopchop init` already exists, extend it with `--nim` flag or auto-detect from project type

**Resolution:**

---

## [14] Nim: Test suite
**Status:** OPEN
**Files:** `tests/nim_grammar.rs` (extend from [9]), `tests/nim_paranoia.rs` (new), `tests/fixtures/nim/paranoia.nim` (new), `tests/fixtures/nim/false_positive.nim` (new)
**Depends on:** [11]
**Spec ref:** Nim Spec Section 9

- [ ] Query compilation tests pass for all 6 categories (naming, complexity, imports, defs, exports, skeleton)
- [ ] Paranoia test: ≥9 violations from unsafe code fixture
- [ ] False positive test: 0 violations from safe Nim code fixture
- [ ] SAFETY comment suppression: violations disappear when `# SAFETY:` is present
- [ ] Smoke test: `slopchop scan` on a minimal Nimble project produces correct output
- [ ] `slopchop check` on Nimble project runs `nimble test` (or emits tool-not-found warning)
- [ ] `slopchop init --nim` generates valid `nim.cfg` and `slopchop.toml`

**Resolution:**

---

## [15] Nim: Autonomous protocol and prompt updates
**Status:** OPEN
**Files:** `src/prompt.rs`, `AGENT-README.md`
**Depends on:** [4], [11]
**Spec ref:** Nim Spec Sections 6, 10

- Update prompt Law of Paranoia section with Nim rules
- Add Nim-specific guidance to autonomous protocol / AGENT-README.md
- Include: no `cast[]` without `# SAFETY:`, no `{.emit.}`, no disabled runtime checks, prefer `ref` over `ptr`, respect `nim.cfg` as governance

**Resolution:**

---

## [16] Nim: V2 metrics exclusion guard
**Status:** OPEN
**Files:** `src/analysis/v2/worker.rs` (or wherever it lives after [1] rename)
**Depends on:** [1], [10]
**Spec ref:** Nim Spec Section 7

Ensure the analysis engine guard clause excludes Nim from LCOM4/CBO/AHF/SFOUT scope extraction. Nim scope extraction (mapping `type Foo = object` + procs-with-Foo-first-param to cohesion scopes) is a future Phase 2 item.

- Add `Lang::Nim` to the `if lang != Lang::Rust` early-return guard
- Ensure Nim files still get basic violations (token count, complexity, nesting, args) just not structural metrics

**Resolution:**

---

## [17] Nim: Import resolver for locality graph
**Status:** OPEN
**Files:** `src/graph/imports.rs`, `src/graph/resolver.rs`
**Depends on:** [8], [10]

Extend the import extraction and resolution to understand Nim's import syntax:
- `import std/strutils` → resolves to stdlib module
- `import mymodule` → resolves to `mymodule.nim` in project
- `from os import paramCount` → resolves to `os` module
- `include helpers` → resolves to `helpers.nim` (textual include)
- `import pkg/somelib` → Nimble package dependency

This enables the law of locality graph to work with Nim projects. The edge collection, coupling computation, and cycle detection are all language-agnostic once edges are resolved.

**Resolution:**

---

## Dependency Graph

```
[1] Reframe v2 engine ─────────┬──→ [9] Grammar validation (GATE)
                                │         │
[2] Cut packing ──→ [3] Cut map │         ├──→ [10] Language integration
         │                      │         │         │
         └──→ [4] Prompt eval ──┘         │         ├──→ [11] Paranoia checks ──→ [14] Tests
                    │                     │         │         │
                    └─────────────────────┼─────────┼──→ [15] Protocol updates
                                          │         │
[5] Output compression (independent)      │         ├──→ [12] Naming conventions
                                          │         │
[6] Clipboard fix (independent)           │         ├──→ [13] Strict config gen
                                          │         │
[7] Apply/check eval (independent)        │         └──→ [16] V2 metrics guard
                                          │
[8] Locality revisit ────────────────────────────→ [17] Nim import resolver
```

Items [5], [6], [7] are independent and can be worked in parallel with everything else.
