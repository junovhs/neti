# Warden Protocol Roadmap

## Philosophy

**The Two-Layer Model:**
1. **Files** (~2k tokens) - The organizational unit. Right-sized for context windows.
2. **Contracts** - The semantic unit. Machine-verifiable intent at symbol level.

**Why Constraints Matter for AI:**
- Cyclomatic complexity limits bound hallucination surface
- Nesting depth limits prevent AI losing track of scope  
- Function length limits fight attention degradation
- File size limits respect context window economics

These aren't style preferences. They're **containment protocols**.

---

## Current State: v0.4.1

- [x] Core loop: knit → chat → apply → verify
- [x] Self-hosting (Warden passes its own rules)
- [x] Path safety validation (traversal, absolute, sensitive, hidden)
- [x] Markdown block rejection
- [x] Backup system

---

## v0.5.0 — Bulletproof Apply

**Theme:** If it applies, it's valid. If it's invalid, it rejects hard.

### Validation Hardening

- [x] **Path safety validation**
  Blocks: `../` traversal, absolute paths, `.git/`, `.env`, `.ssh/`, `.aws/`, hidden files.

- [x] **Markdown block rejection**
  Rejects fenced code blocks in file content.

- [ ] **Truncation detection**
  Reject obviously incomplete files:
  - Unbalanced braces/brackets (language-aware)
  - Truncation markers: `// ...`, `// rest of file`, `// etc`
  - Files ending mid-statement: trailing `{`, `,`, `(`, `=`
  
  *Zero false positives. If Warden rejects, it was broken.*

- [ ] **Robust Delimiter Protocol (Nabla Format)**
  Replace fragile XML with high-entropy Unicode fences:
  
      ∇∇∇ src/main.rs ∇∇∇
      fn main() {}
      ∆∆∆
  
  Benefits:
  - Never interpreted as HTML/Markdown
  - Never appears in real code
  - Trivial to regex
  - AI can't confuse output with format

### Workflow Enhancement

- [ ] **Error injection in knit**
  When `knit --prompt` runs, append current violations:
  
      ═══════════════════════════════════════════════════════════════════
      CURRENT VIOLATIONS (FIX THESE)
      ═══════════════════════════════════════════════════════════════════
      
      src/validator.rs:42 [COMPLEXITY] Score 12 (max 5)
      src/lib.rs:1 [ATOMICITY] 2341 tokens (max 2000)
  
  *AI sees what's broken. AI fixes it.*

- [ ] **`warden apply --commit`**
  On success: `git add .` → auto-generate commit message → commit.
  
  *If it passes validation, commit it. Git is your undo.*

---

## v0.6.0 — The Contract Protocol

**Theme:** Trust but Verify (Programmatically).

AI must declare intent before writing code. Warden verifies the output matches the contract.

### The Contract DSL

Grammar:

    ACTION TYPE PATH[:SYMBOL] [ASSERTIONS]

Actions: `CREATE`, `UPDATE`, `DELETE`, `REFACTOR`
Types: `FILE`, `FN`, `STRUCT`, `ENUM`, `TRAIT`, `IMPL`

### Supported Assertions

| Keyword | Meaning | Example |
|---------|---------|---------|
| `complexity` | Cyclomatic complexity | `ASSERT complexity <= 5` |
| `depth` | Max nesting level | `ASSERT depth <= 2` |
| `args` | Function arity | `ASSERT args <= 3` |
| `lines` | Line count | `ASSERT lines < 50` |
| `tokens` | Token count | `ASSERT tokens < 500` |
| `contains` | Text/regex presence | `ASSERT contains "Result<"` |
| `public` | Visibility | `ASSERT public == true` |

### Example Contract

    ∇∇∇ CONTRACT ∇∇∇
    GOAL: Refactor parser for clarity
    
    REFACTOR FN src/parser.rs:parse_header
        ASSERT complexity <= 4
        ASSERT depth <= 1
    
    CREATE STRUCT src/types.rs:Header
        ASSERT public == true
    
    UPDATE FILE src/lib.rs
        ASSERT tokens < 2000
    ∆∆∆

### Execution Logic

1. **Parse Contract** → `Vec<Intent>`
2. **Parse Payload** → Tree-sitter AST (in memory, before write)
3. **Symbol Resolution** → Find declared symbols in AST
4. **Metric Validation** → Run metrics, compare to assertions
5. **Scope Creep Detection** → Flag undeclared modifications

Contract Breach Examples:
- "Function `parse_header` not found in output"
- "Complexity is 8, contract requires <= 4"
- "Undeclared modification: `fn risky_logic` was changed"

---

## v0.7.0 — Context Intelligence

**Theme:** The Map vs. Territory problem.

### The Skeletonizer

Strip function bodies, keep signatures:

    // Full (Territory)
    pub fn process(data: &[u8]) -> Result<Output> {
        let parsed = parse(data)?;
        validate(&parsed)?;
        transform(parsed)
    }
    
    // Skeleton (Map)
    pub fn process(data: &[u8]) -> Result<Output> { ... }

- [ ] **`knit --skeleton`** - All files skeletonized
- [ ] **`knit src/main.rs --smart`** - Full code for target + skeletons for rest

### Dependency Graphing

- [ ] Parse `mod`, `use`, `import`, `require` statements
- [ ] Build local dependency graph
- [ ] Auto-include dependencies in context

---

## v0.8.0 — Verification & Safety

**Theme:** Beyond "it compiles."

### Property-Based Testing

- [ ] **`warden gen-test <file>`**
  AI writes property tests (proptest/hypothesis), not unit tests.
  
      "Assert invariants. What must ALWAYS be true?"

### Function-Level Reporting

    src/engine.rs
    
      fn process_batch() [Line 45]
      ├─ Complexity: 14 (max 5)
      ├─ Depth: 5 (max 2)
      ├─ Contributing factors:
      │   ├─ 3 nested ifs (lines 52, 58, 61)
      │   └─ 2 complex match guards (lines 67, 89)
      └─ Suggestion: Extract inner match

### Incremental Scanning

- [ ] Track file mtimes in `.warden_cache`
- [ ] Use `git status` for changed files
- [ ] Full rescan on config change

---

## v0.9.0 — Ecosystem

**Theme:** CI/CD integration.

- [ ] `warden --format json` - Machine-readable output
- [ ] SARIF output for GitHub Code Scanning
- [ ] `warden hook install` - Pre-commit hook
- [ ] GitHub Action for PR checks
- [ ] Documented exit codes

---

## v1.0.0 — Release

- [ ] Published to crates.io
- [ ] Homebrew formula
- [ ] Scoop/Winget packages
- [ ] Documentation site
- [ ] Logo and branding

---

## Future

### AI-Native Linting
- Global state detection (`static mut`, singletons)
- Impure function warnings (returns value, takes no args)
- Deep inheritance check (> 1 level)

### Metrics Dashboard
SQLite backend. Complexity trends over time. Codebase health charts.

### Session Branches
`warden session start` → timestamped branch
`warden apply --commit` → atomic commits
`warden session merge` → squash to main

---

## Not Doing

- **VS Code Extension** - IDE lock-in, maintenance burden
- **Watch mode** - Complexity without clear benefit
- **Markdown fallback parsing** - Enforce format discipline
- **"Smart" fixing** - Warden rejects, doesn't repair

---

## Principles

1. **Reject bad input, don't fix it**
   Warden is a gatekeeper, not a fixer.

2. **Git is the undo system**
   Don't reinvent version control.

3. **Explicit > Magic**
   If AI doesn't follow format, fail loudly.

4. **Containment over craftsmanship**
   For AI, constraints aren't style—they're safety.

5. **Eat your own dogfood**
   Warden must pass its own rules.

6. **The dream: perfect modularity**
   Take any file to AI, bring it back, it slots in perfectly.
   Contracts make this verifiable.
