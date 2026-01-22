# SlopChop: Past, Present, Future
**Date:** 2026-01-21
**Version:** 1.6.0

---

## Executive Summary

SlopChop is evolving from a **Verification Engine** (linting, governance) into a **Context Engine** (intelligent context packaging for LLM-assisted development). This session implemented **small codebase auto-detection** to reduce false positives on tiny projects, validating the approach against the git-trek case study.

---

## What Was Done This Session

### Small Codebase Auto-Detection

**Problem:** Structural metrics (LCOM4, CBO, AHF, SFOUT) generate noise on small projects. A 6-file TUI app doesn't need modularity governance — it needs to work.

**Solution:** Skip deep analysis when `src/` file count < 10.

| File | Change |
|------|--------|
| `src/analysis/v2/engine.rs` | Added `SMALL_CODEBASE_THRESHOLD = 10`, `count_source_files()`, `is_source_file()`. Core logic extracted to helpers to satisfy SFOUT limit. |
| `src/analysis/v2/mod.rs` | Re-exported `is_small_codebase()`, `small_codebase_threshold()` |
| `src/cli/handlers/scan_report.rs` | Added visual indicator when small codebase mode active |
| `src/types.rs` | Added `is_small_codebase()` method to `ScanReport` |

**Key Design Decisions:**
1. Count only `src/` files (excludes tests, benches, examples)
2. Threshold is 10 files (based on case-study-gittrek.md recommendation)
3. When triggered, returns empty HashMap from deep analysis — no LCOM4/CBO/AHF/SFOUT violations generated

**Validation:** Tested on git-trek. AHF (0.0%) and CBO (14) violations on `App` struct now suppressed. Only real issues remain (syntax error in `test.rs`, token limits on docs).

---

## Current State

### Passing
- ✅ 144 files clean (SlopChop self-scan)
- ✅ 76 tests passing
- ✅ Clippy clean
- ✅ Locality: 98.9% health (2 known violations)

### Architecture Overview

```
slopchop/
├── src/
│   ├── analysis/           # Rule engine, AST analysis
│   │   ├── v2/             # Scan V2 engine (LCOM4, CBO, patterns)
│   │   │   ├── engine.rs   # ← Small codebase detection lives here
│   │   │   ├── inspector.rs
│   │   │   ├── metrics.rs
│   │   │   └── patterns/   # AST pattern detectors (S, C, R, X, P, M, I, L)
│   │   └── checks/         # Legacy checks (complexity, naming, syntax)
│   ├── pack/               # Context packaging for LLMs
│   │   ├── focus.rs        # Foveal/peripheral computation
│   │   ├── formats.rs      # XSC7XSC sigil format
│   │   └── xml_format.rs   # XML output format
│   ├── graph/              # Dependency graph, PageRank, locality
│   │   ├── rank/           # PageRank implementation
│   │   └── locality/       # Topological health analysis
│   ├── cli/                # Command handlers
│   └── spinner/            # Progress UI
```

### Known Issues (Not Blocking)

| Issue | Status | Notes |
|-------|--------|-------|
| Locality violations (2) | Accepted | `apply→cli` encapsulation breach, `cli→spinner` sideways dep |
| `slopchop map --deps` slow on large repos | Known | O(n²) in graph construction |

---

## Strategic Context: The Context Engine Pivot

### The Vision (from `pack-pivot.md` and `context-research.md`)

Current AI coding assistants optimize for **Recall** (dump everything) at the expense of **Precision** (token cost). Research shows the optimal model is **Progressive Disclosure**:

```
Turn 1: Map (~2k tokens)     → "Where might the bug be?"
Turn 2: Skeleton (~6k)       → "Give me context to understand it"  
Turn 3: Full (~20k)          → "Now I need the implementation"
```

SlopChop already has the infrastructure:
- `slopchop map` — Repository structure with PageRank
- `slopchop pack --focus FILE --depth N` — Foveal (full) + peripheral (skeleton)
- `slopchop signatures` — Type-surface extraction

**Gap:** These exist but aren't well-integrated or discoverable.

### Research-Backed Improvements (from `context-research.md`)

| Current (Aider-style) | Research-Optimal | Priority |
|-----------------------|------------------|----------|
| Static PageRank | Task-aware ranking (embed query, re-rank by similarity) | Medium |
| Whole-file granularity | Symbol slicing (`--symbol MyStruct`) | Medium |
| One-shot commands | Interactive zoom (`expand`, `slice`) | Low |
| No backward slicing | Line-range extraction with dataflow | Low (hard) |

### Governance Profiles (from `case-study-thubo.md` and `SCAN_V2_SPEC.md`)

Different software has different physics:

| Profile | Philosophy | Structural Metrics | Safety Checks |
|---------|------------|-------------------|---------------|
| `application` (default) | Maintainability > Performance | Strict | Standard |
| `systems` | Throughput > Abstraction | Relaxed/Disabled | Escalated |

**The Inversion Principle:** Systems code trades abstraction for performance but must be paranoid about memory safety. Relax LCOM4/CBO/AHF, escalate unsafe/transmute checks.

**Implementation:** Add `profile = "systems"` to `slopchop.toml`. When set:
- `max_file_tokens = 10000`
- `max_cognitive_complexity = 50`
- `max_lcom4 = 100` (effectively disabled)
- `max_cbo = 100` (effectively disabled)
- Require `// SAFETY:` comments on all unsafe blocks

---

## Next Session Priorities

### 1. Profile System (HIGH)

**Why:** Thubo case study showed 70+ violations that were domain-appropriate architectural decisions. Explicit profiles give users control.

**Implementation:**
```toml
# slopchop.toml
profile = "systems"  # or "application" (default)
```

**Files to modify:**
- `src/config/types.rs` — Add `profile: Profile` field
- `src/analysis/file_analysis.rs` — Use profile to adjust thresholds
- `src/analysis/v2/inspector.rs` — Skip structural metrics in systems mode

**Reference:** `case-study-thubo.md`, `SCAN_V2_SPEC.md` section "Governance Profiles"

### 2. Governance Hint (MEDIUM)

**Why:** When violation density is high, suggest profile adjustment instead of overwhelming users.

**Trigger:** `violations > (files × 3)`

**Output:**
```
⚠ High violation density: 47 violations across 8 files (5.9 per file)

  If this is a high-performance systems project, consider:
    profile = "systems"

  Run `slopchop config` to adjust.
```

**Files to modify:**
- `src/cli/handlers/scan_report.rs` — Add hint logic after violation summary

### 3. Pack Command Discoverability (MEDIUM)

**Why:** The Three-Turn Protocol infrastructure exists but users don't know about it.

**Actions:**
- Improve `slopchop pack --help` with examples
- Add `slopchop pack --focus FILE` example to README
- Consider `slopchop focus FILE` alias (simpler UX)

### 4. Skeleton Refinement (LOW)

**Why:** Research (RQ6) says skeletons must produce valid syntax LLMs recognize.

**Current:** `fn foo() { ... }` — Good for Rust.

**Check:** Validate Python (`def foo(): ...`) and TypeScript output are idiomatic.

**File:** `src/skeleton.rs`, `src/lang.rs` (skeleton_replacement)

---

## Reference Documents

| Document | Purpose |
|----------|---------|
| `context-research.md` | 12 research questions + Agentless vs Agentic comparison + Aider analysis |
| `pack-pivot.md` | Three-Turn Protocol spec, skeleton operators, zoom capability |
| `case-study-thubo.md` | Systems code governance, profile system design, P03 allowlist |
| `case-study-gittrek.md` | Small codebase detection rationale |
| `SCAN_V2_SPEC.md` | Full pattern catalog, metric definitions, profile thresholds |

---

## Commands Reference

```bash
# Governance
slopchop check          # Full verification (scan + clippy + test + locality)
slopchop scan           # Structural analysis only
slopchop scan --locality # Topological health analysis

# Context Packaging
slopchop pack                           # Full codebase context
slopchop pack --focus src/main.rs       # Focused context (main.rs full, neighbors skeleton)
slopchop pack --focus src/main.rs --depth 2  # Expand neighbor radius
slopchop map                            # Repository structure
slopchop map --deps                     # With dependency arrows
slopchop signatures                     # Type-surface only (architect mode)

# Workflow
slopchop branch         # Create slopchop-work branch
slopchop apply          # Apply AI changes from clipboard
slopchop apply -c       # Apply with verification
slopchop promote        # Merge to main (squash)
slopchop config         # Interactive config editor
```

---

## Metrics Cheat Sheet

| Metric | Default Limit | What It Catches |
|--------|---------------|-----------------|
| File Tokens | 2000 | God files |
| Cognitive Complexity | 25 | Tangled logic |
| Nesting Depth | 3 | Deep conditionals |
| Function Args | 5 | Bloated signatures |
| LCOM4 | 1 | Incohesive classes |
| AHF | ≥60% | Leaking state |
| CBO | 9 | Tight coupling |
| SFOUT | 7 | High fan-out |

**Small Codebase Mode:** When `src/` has <10 files, LCOM4/CBO/AHF/SFOUT are skipped.

---

## Test Commands

```bash
# In slopchop repo
cargo test                              # 76 tests
cargo clippy --all-targets -- -D warnings -W clippy::pedantic
slopchop check                          # Self-verification

# Validate small codebase detection
cd ~/git-trek && slopchop scan          # Should skip structural metrics
```

---

## Architecture Notes

### Why Structural Metrics Live in V2 Engine

The V2 engine (`src/analysis/v2/`) performs **global** analysis that requires seeing all files:
- LCOM4: Requires method→field access graph per struct
- CBO: Requires cross-file dependency counting
- AHF: Requires field visibility analysis

The V1 checks (`src/analysis/checks/`) are **local** per-file analysis:
- Complexity, nesting, arity — single file, single pass

### Why Small Codebase Detection is in engine.rs

The decision to skip structural metrics must happen **before** aggregation begins. Placing it at the entry point of `run_analysis()` means we never waste cycles building scope graphs for tiny projects.

### The XSC7XSC Sigil Protocol

The `XSC7XSC` markers are a DNA-like sequence chosen to be:
1. Extremely unlikely to appear in real code
2. Easy for regex to find
3. Visually distinctive

Format:
```
XSC7XSC FILE XSC7XSC path/to/file.rs SHA256:abc123
<content>
XSC7XSC END XSC7XSC
```

This enables `slopchop apply` to parse AI responses reliably.

---

## Session Log

1. Reviewed research docs (`context-research.md`, `pack-pivot.md`, case studies)
2. Identified small codebase detection as immediate value add
3. Implemented in `engine.rs` — initial version counted all files
4. Hit SFOUT violation — extracted to module-level functions
5. Hit clippy `ptr_arg` — changed `&PathBuf` to `&Path`
6. Refined heuristic to count only `src/` files (excludes tests)
7. Validated on git-trek — AHF/CBO noise eliminated
8. Documented for next session handoff

**Total changes:** 4 files, ~100 lines added

---

*"Governance is a social contract between you and your code. The tool should enforce the contract you chose, not guess what contract you wanted."* — case-study-thubo.md
