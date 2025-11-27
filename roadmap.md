# Warden Protocol Roadmap

## Current State: v0.5.0 (Bulletproof Apply) ✓

The core loop is hardened:
- **Nabla Protocol**: Robust file delimiters (`∇∇∇`).
- **Plan Protocol**: Interactive confirmation before writing.
- **Git Integration**: Atomic commits per apply.
- **Self-hosting**: Warden enforces its own rules.

---

## v0.6.0 — Context Intelligence (The Saccade Merge)

**Theme:** The "Map vs. Territory" Architecture. Solve the "Lost in the Middle" problem.

### The Skeletonizer (Ported from Saccade)
- [ ] **Port `parser.rs` from Saccade**
  - Integrate Tree-sitter-based stripping of function bodies.
  - Keep structs, enums, trait signatures, and function signatures.
  - Goal: Reduce file size by ~70-90% while retaining API visibility.
  
- [ ] **knit --skeleton**
  - Generates a context file where *every* file is skeletonized.
  - Useful for "high level architectural planning" with the AI.

### Smart Knitting (Context Slicing)
- [ ] **Dependency Graphing (Saccade Stage 1)**
  - Implement Tree-sitter queries to find `mod`, `use`, `import`, and `require`.
  - Build a lightweight graph of local file dependencies.

- [ ] **knit src/main.rs --smart**
  - **The Territory:** Includes full source code of `src/main.rs` and its *immediate* imports.
  - **The Map:** Includes *skeletons* of the rest of the project (or at least the rest of the module).
  - *Result:* AI has deep focus on the task, broad awareness of the project, but low token count.

### The "Generate-Then-Structure" Workflow
- [ ] **Decoupled Reasoning**
  - Update `warden apply` to handle a two-step generation process if we move to an agentic loop later.
  - Step 1: Generate Plan (Natural Language).
  - Step 2: Generate Code (Strict XML).
  - *Reduces the cognitive load of formatting on the AI's reasoning capabilities.*

---

## v0.7.0 — Verification & Safety

**Theme:** Trust the tool, verify the AI.

### Property-Based Testing (The Dream)
- [ ] **warden gen-test <file>**
  - Uses AI to write *Property-Based Tests* (`proptest` for Rust, `hypothesis` for Python).
  - Prompt: "Analyze this code. Write a property test that asserts invariants. Do not write unit tests."
  - Automatically saves to `tests/warden_props_<name>.rs`.
  - *Moves verification from "it compiles" to "it is mathematically sound".*

### Smarter Analysis (Refined)

- [ ] **Function-level violation reporting**  
  Not just "file has violations" but detailed breakdown:
  
  *Learn from the patterns. Understand WHY it's complex.*

- [ ] **Incremental scanning**  
  Only re-analyze changed files:
  - Track file mtimes in `.warden_cache`
  - Or use `git status` to find modified files
  - Full rescan on config change

---

## v0.8.0 — Ecosystem & Polish

**Theme:** CI/CD and tooling integration.

- [ ] **Test suite**
  - Unit tests for each module
  - Integration tests: knit → apply → verify flow

- [ ] **Performance benchmarks**
  - Scan time vs file count

- [ ] **CLI stability guarantee**
  - Document all flags and subcommands

---

## v1.0.0 — Release

- [ ] Published to **crates.io**
- [ ] **Homebrew**
- [ ] **Scoop/Winget**

---

## Principles

1. **Reject bad input, don't fix it**  
   Warden is a gatekeeper, not a fixer.

2. **Git is the undo system**  
   Don't reinvent version control.

3. **Explicit > Magic**  
   If AI doesn't follow the format, fail loudly.

4. **Learn from violations**  
   Error messages should teach, not just complain.

5. **Eat your own dogfood**  
   Warden must pass its own rules.