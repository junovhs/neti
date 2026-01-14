# SlopChop: Past, Present, Future

**Date:** 2026-01-14  
**Version:** v1.8.0 (The "Pack Pivot" Release)

---

## What Was Done Previously (v1.7.0)

### Governance Profiles
- Introduced `application` vs `systems` profiles after stress-testing against `thubo` (high-performance network pipeline)
- Systems mode relaxes structural metrics while tightening safety checks (the "Inversion Principle")
- Implemented per-file heuristic detection based on `unsafe`, `Atomic`, `repr(C)`, `no_std`

### UX & Agent Ergonomics
- **Flight Recorder:** `slopchop-report.txt` persists full scan results for AI agent consumption
- **Rich Snippets:** rustc-style violation reporting with line numbers and underlines
- **Adaptive TUI:** Coarse/fine stepping in configuration editor

### Transactional Workflow
- Goal-aware commits: `GOAL:` from PLAN blocks persists through to merge commit messages
- Clean `apply` → `check` → `promote` pipeline

---

## What Was Decided Today (v1.8.0 Direction)

### The Pack Pivot

**Discovery:** Testing showed that a minimal context payload (repo map + dependency arrows + semantic descriptions) at ~3k tokens enables AI to request exactly the files needed for a task. An AI correctly identified 4 specific files for a feature implementation with 9/10 confidence.

**Conclusion:** The existing pack system (PageRank ranking, signature extraction, skeletonization) is unnecessary complexity. A simpler approach works better.

**Decision:** Delete the old pack system. Replace with semantic descriptions.

| Delete | Tokens | Rationale |
|--------|--------|-----------|
| `src/signatures/` | ~2.2k | Replaced by semantic descriptions |
| `src/graph/rank/` | ~3.5k | PageRank not needed for file selection |
| `src/pack/` | ~4k | Complete replacement |
| `src/skeleton.rs` | ~800 | AI can request full files directly |
| **Total** | **~10.5k** | |

| Add | Tokens | Description |
|-----|--------|-------------|
| `semantics.toml` loader | ~200 | Read one-line descriptions from config |
| `slopchop map --semantic` | ~150 | Include descriptions in map output |
| D01 pattern | ~150 | Flag files missing descriptions |
| **Total** | **~500** | |

**Net result:** Delete 10k tokens, add 500. Codebase shrinks AND feature improves.

See `pack-pivot.md` for full rationale and implementation plan.

### Structural Metrics: Keep, Don't Cut

Initial instinct was to cut LCOM4, AHF, CBO, SFOUT because case studies showed noise. Deeper analysis revealed:

- The noise was a **calibration problem**, not a fundamental flaw
- Structural metrics catch drift that accumulates across AI coding sessions
- Each session is a "rotating consultant" who doesn't see the whole arc

**Decision:** Keep structural metrics. Add small-codebase gate:
```
if total_files < 10 OR total_tokens < 5000:
    skip LCOM4, AHF, CBO, SFOUT
```

### TypeScript: Deferred

Tree-sitter infrastructure exists for TypeScript, but all tuning work has been Rust-focused. 

**Decision:** Ship Rust-first. TypeScript support follows once Rust governance is rock-solid. This aligns with the philosophy of doing one thing well before expanding scope.

### Positioning Clarity

**Tagline:** "The architectural compiler for AI-assisted Rust development"

SlopChop is a verification layer, not a generation tool:
- Rust's compiler rejects type-unsafe code
- Clippy rejects unidiomatic code
- Tests reject functionally incorrect code
- **SlopChop rejects structurally-unsound code**

Code bounces off until it passes. Then it enters the codebase.

---

## Current Status

| Category | Status | Notes |
|----------|--------|-------|
| **Governance Engine** | [OK] Stable | 23 patterns, 6 metrics |
| **Profiles** | [OK] Active | `application` / `systems` working |
| **Transactional Workflow** | [OK] Complete | apply → check → promote |
| **Flight Recorder** | [OK] Live | Agent-readable reports |
| **Tests** | [OK] Pass | 68 unit tests |
| **Self-Scan** | [OK] Clean | 0 violations at CC=15 |

---

## v1.8.0 Implementation Plan

### Phase 1: Add Semantic System
- [ ] Add `[semantics]` section to config parser
- [ ] Generate initial `semantics.toml` for SlopChop codebase
- [ ] Implement `--semantic` flag for `slopchop map`
- [ ] Implement D01 pattern (missing descriptions)

### Phase 2: Delete Old Pack System
- [ ] Remove `src/signatures/`
- [ ] Remove `src/graph/rank/`
- [ ] Remove `src/pack/`
- [ ] Remove `src/skeleton.rs`
- [ ] Remove CLI commands: `pack`, `signatures`

### Phase 3: Calibration
- [ ] Implement small-codebase detection for structural metrics
- [ ] Test against variety of repo sizes to validate threshold

### Phase 4: Documentation
- [ ] Update README.md (new version drafted)
- [ ] Update AGENT-README.md with new workflow
- [ ] Archive old pack documentation

---

## Future Roadmap

### v1.9: Stress Testing
- Scan top-tier Rust repositories: `tokio`, `polars`, `ripgrep`, `axum`
- Identify where SlopChop fails on idiomatic high-performance patterns
- Refine systems profile thresholds based on findings

### v2.0: TypeScript Support
- Map deferred patterns (async races, event listener leaks) to tree-sitter-typescript
- Implement `tsconfig.json` path resolution
- Full pattern coverage parity with Rust

### Research Track
- Semantic cost analysis: infer function call cost from call tree
- Auto-flag high-frequency functions missing `#[inline]`

---

## Key Files

```
src/
├── analysis/
│   ├── file_analysis.rs    # Systems detection heuristics
│   └── v2/
│       ├── cognitive.rs    # Cognitive complexity engine
│       └── patterns/       # All 23 AST patterns
├── apply/
│   ├── executor.rs         # Goal persistence
│   └── verification.rs     # Flight recorder
├── cli/
│   ├── handlers.rs         # Command implementations
│   └── config_ui/          # TUI configuration
├── config/
│   └── types.rs            # Will add semantics support
└── map.rs                  # Will add --semantic flag
```

---

## Reference Documents

| Document | Purpose |
|----------|---------|
| `SCAN_V2_SPEC.md` | Authoritative specification for governance engine |
| `pack-pivot.md` | Full rationale for the pack system replacement |
| `context-research.md` | Academic research backing progressive disclosure |
| `case-study-thubo.md` | Systems profile origin story |
| `personal-philosophy.md` | Development principles and tech stack |
