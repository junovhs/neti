# Development Philosophy & Technical Stack

**Author:** Spencer Nunamaker  
**Date:** 2026-01-14  
**Purpose:** Reference document for AI assistants and collaborators

---

## Core Principles

### 1. The Goal Informs the Process

Before solving a problem, ask: *What are we actually trying to accomplish?*

AI systems (and humans) often optimize for the wrong objective. Instead of asking "what's the goal?", they ask "what's the fastest way to clear this block?" The objective quietly shifts from achieving the right outcome to merely unblocking progress—regardless of downstream consequences.

This manifests as:
- Tests that always pass but validate nothing
- Features deleted because they produce warnings
- Constraints bypassed instead of properly addressed

**The right questions:**
1. What's the end goal?
2. Does this obstacle exist for a reason?
3. How do we serve that goal while remaining efficient?
4. Are we solving the right problem without creating new ones?

When pattern detection produces noise, the answer isn't deletion. The answer is calibration. Noise signals incomplete tuning, not a flawed concept.

### 2. Robust Solutions Over Quick Fixes

When addressing a problem, the first instinct should be: *How do I solve this in a way that stands the test of time, prevents recurrence, and enables solving the next problem more easily?*

Each fix should leave the codebase better than before. Build tools that make building the next tool easier. This compounds over time.

### 3. Zero Tolerance for Debt

Technical debt, deferred maintenance, accumulated entropy—these slow everything down.

Whether maintaining software, hardware, or any complex system: **regular maintenance beats deferred cleanup**. Small, continuous improvements prevent the need for large, risky refactors.

**Applied:**
- No "we'll fix it later" without a concrete timeline
- Refactor incrementally, not in large batches
- Address noise at the source rather than suppressing symptoms

### 4. Clarity Over Cleverness

Verbose but readable code is preferable to dense, clever abstractions.

Code that can be understood in six months beats an elegant ten-line solution that requires reverse-engineering. Domain-specific languages are powerful but introduce maintenance burden.

**Applied:**
- Explicit over implicit
- Readable over clever
- If code requires extensive comments, consider refactoring

### 5. Small, Atomic Units

File atomicity enables better AI-assisted development:
- Smaller files are easier to reason about completely
- AI can emit full file replacements without surgical edits
- Small units have focused responsibilities
- The codebase naturally decomposes into understandable pieces

**The guideline:** Files under 2,000 tokens (~400 lines). Larger files should be split by responsibility.

This principle scales to the application level. Smaller, focused tools often outperform monolithic systems.

---

## The Rust Decision

### The Compiler as Verification Layer

Early experience with AI-assisted Rust development revealed a pattern: the compiler would reject generated code, the AI would revise, and this cycle would repeat several times. Initially frustrating, this process consistently produced working code once compilation succeeded.

This contrasts with dynamically-typed languages where generated code often compiles (or runs) but fails at runtime with subtle bugs.

**The insight:** Rust's compiler acts as a verification layer. Code that passes the compiler has already survived rigorous type and memory safety checks. "If it compiles, it is significantly more likely to work correctly."

This doesn't catch logic errors, but it eliminates entire categories of bugs before runtime.

### The Verification Stack

| Layer | Rejects | Guarantees |
|-------|---------|------------|
| Rust compiler | Type/memory unsoundness | Memory safety, type correctness |
| Clippy | Anti-patterns | Idiomatic code |
| Tests | Behavioral regressions | Functionality matches specification |
| SlopChop | Structural decay | File atomicity, bounded complexity, controlled coupling |

Each layer is a gate. Code must pass all gates before entering the codebase.

---

## Technical Stack

### Objective

A unified technology stack capable of targeting Linux, Windows, macOS, iOS, Android, desktop applications, and web applications—using a single language.

### Current Stack

| Layer | Technology | Rationale |
|-------|------------|-----------|
| Language | **Rust** | Memory safety, performance, cross-platform compilation |
| Desktop UI | **Dioxus** | Rust-native, compiles to native binaries |
| Web UI | **Dioxus** | Same codebase compiles to WebAssembly |
| Mobile | **Dioxus** | iOS/Android support (maturing) |
| Styling | **HTML + CSS** | Mature, expressive, well-tooled design system |
| CLI | **clap** | Standard Rust CLI framework |
| Parsing | **tree-sitter** | Multi-language AST parsing |

### Design Language

HTML and CSS remain the styling layer. This is intentional—they represent decades of refinement in layout and visual design primitives. Dioxus preserves access to this ecosystem while maintaining Rust-native application logic.

### Excluded Technologies

| Technology | Reason |
|------------|--------|
| JavaScript/TypeScript | Rust covers all target platforms |
| Electron | Resource overhead; Dioxus compiles native |
| Complex state frameworks | Prefer simple state in small files |

---

## AI-Assisted Development Model

### Trust and Autonomy

There is an inverse relationship between autonomy and trust in AI-assisted development:

| Mode | Autonomy | Trust Level | Verification Need |
|------|----------|-------------|-------------------|
| Chat-based | Low (human in loop) | Higher | Standard |
| Agentic (Cursor, etc.) | High (autonomous) | Lower | Elevated |

Greater autonomy requires stronger verification. This is why SlopChop exists.

### The Session Model

Each AI coding session operates as an independent consultant:
- No memory of previous architectural decisions
- No awareness of ongoing technical direction
- Fresh context each interaction

This model requires:
1. **Documentation** — Explicit trails of decisions and rationale
2. **Goal-aware commits** — Capture intent, not just changes
3. **Automated governance** — Structural checks that persist across sessions

### The Senior Engineer Insight

Research indicates senior engineers accept more AI-generated code than juniors. The differentiator is **verification capability**:
- Seniors write precise, unambiguous prompts
- Seniors decompose work into appropriate units
- Seniors have heuristics to evaluate correctness quickly
- Juniors can generate code but struggle to verify quality

SlopChop externalizes these verification heuristics as automated checks.

### Progressive Disclosure

The optimal context strategy for AI assistance:

1. **Initial payload** (~3k tokens): Repository map with semantic descriptions
2. **AI request**: Specific files identified as relevant
3. **Targeted context** (~15-20k tokens): Exactly what's needed
4. **Total for large codebase**: ~20-30k tokens vs. 500k+ raw

This achieves ~95% token reduction while maintaining effectiveness. See `context-research.md` for academic validation.

---

## Architectural Boundaries

### Tool Separation

| Tool | Responsibility | Data Flow |
|------|----------------|-----------|
| **SlopChop** | Verification and governance | Validates AI *output* |
| **Context tooling** | Progressive disclosure | Prepares AI *input* |

These are complementary but distinct concerns. Combining them creates unclear boundaries and competing objectives.

### SlopChop's Role

SlopChop functions as an "architectural compiler" for AI-assisted development:
- Rust's compiler rejects type-unsafe code
- SlopChop rejects structurally-unsound code

Structural soundness includes:
- File sizes within atomic editing range
- Function complexity within reasoning limits
- Module coupling within refactoring tolerance

Code that fails `slopchop check` does not enter the codebase.

---

## Decision Framework

When evaluating options:

1. **Goal alignment** — Does this serve the actual objective?
2. **Robustness** — Will this solution persist without rework?
3. **Compounding value** — Does this enable future improvements?
4. **Debt avoidance** — Are we deferring cost or eliminating it?
5. **Clarity** — Can this be understood without extensive context?
6. **Atomicity** — Can this be smaller and more focused?

---

## Summary

- **Language:** Rust for all platforms
- **Verification:** Multi-layer gates (compiler → clippy → tests → SlopChop)
- **Architecture:** Small files, clear boundaries, zero debt
- **AI workflow:** Progressive disclosure, strong verification
- **Philosophy:** Goal-driven, robust, clear, atomic
