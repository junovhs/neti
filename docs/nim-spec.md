# Final Specification: Tier-1 Nim Support for SlopChop

**Version:** 1.0 (Initial)  
**Status:** Implementation-Ready

---

## 1. Objectives

1. **Syntactic Parity:** Enable SlopChop to parse `.nim` files, calculate complexity, and enforce naming conventions using Tree-sitter.
2. **Law of Paranoia:** Implement Nim-specific bans on unsafe constructs (`cast[]`, `addr`, `ptr`, `{.emit.}`, raw memory ops) — the Nim equivalents of Rust's `unsafe` blocks.
3. **Strict Mode Enforcement:** Detect use of runtime check disabling pragmas (`{.push checks:off.}`, `{.boundChecks:off.}`, etc.) and flag them as paranoia violations unless annotated.
4. **Cross-Platform Workflow:** Configure command generation to support Nimble projects and detect available tooling (`nimlangserver`, `nimpretty`).

---

## 2. Dependencies & Discovery

### 2.1 Cargo Dependencies

**File:** `Cargo.toml`

The most mature Nim grammar is `alaviss/tree-sitter-nim` (58 stars, Rust bindings, parses all of Nim, latest release 0.6.2). This is **not on crates.io** and must be pulled from Git.

```toml
[dependencies]
# ... existing
tree-sitter-nim = { git = "https://github.com/alaviss/tree-sitter-nim", tag = "0.6.2" }
```

> **Note:** This grammar requires ~7GiB of memory to generate from `grammar.js`. The pre-built C source in the repo means we compile C, not regenerate — so this is only relevant if you fork/modify the grammar. Verify compatibility with your pinned `tree-sitter` version before implementation.

> **Risk:** If the Rust bindings in `alaviss/tree-sitter-nim` don't expose a standard `language()` function compatible with your tree-sitter version, you may need to write a thin wrapper or vendor the C source. Test this in Phase 1 before proceeding.

### 2.2 File Discovery

**File:** `src/constants.rs`

Update the regex to include `.nim` and `.nims` (Nim script):

```rust
pub const CODE_EXT_PATTERN: &str = r"(?i)\.(rs|go|py|js|jsx|ts|tsx|java|c|cpp|h|hpp|cs|php|rb|sh|sql|html|css|scss|json|toml|yaml|md|swift|nim|nims)$";
```

---

## 3. Language Definition & Queries

**File:** `src/lang.rs`

### 3.1 Enum Update

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    Rust,
    Python,
    TypeScript,
    Swift,
    Nim,  // NEW
}

impl Lang {
    #[must_use]
    pub fn from_ext(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Self::Rust),
            "py" => Some(Self::Python),
            "ts" | "tsx" | "js" | "jsx" => Some(Self::TypeScript),
            "swift" => Some(Self::Swift),
            "nim" | "nims" => Some(Self::Nim),  // NEW
            _ => None,
        }
    }

    #[must_use]
    pub fn grammar(self) -> Language {
        match self {
            Self::Rust => tree_sitter_rust::language(),
            Self::Python => tree_sitter_python::language(),
            Self::TypeScript => tree_sitter_typescript::language_typescript(),
            Self::Swift => tree_sitter_swift::language(),
            Self::Nim => tree_sitter_nim::language(),  // NEW
        }
    }
    
    #[must_use]
    pub fn skeleton_replacement(self) -> &'static str {
        match self {
            Self::Python => "\n    ...",
            Self::Nim => "\n  discard",  // NEW — Nim uses indentation, `discard` is idiomatic empty body
            _ => "{ ... }",
        }
    }
}
```

### 3.2 Tree-Sitter Queries

Expand `QUERIES` to `[[&str; 6]; 5]`:

```rust
// [Rust, Python, TypeScript, Swift, Nim] x [Naming, Complexity, Imports, Defs, Exports, Skeleton]
const QUERIES: [[&str; 6]; 5] = [
    // ... existing Rust, Python, TypeScript, Swift entries ...
    
    // Nim (Index 4)
    [
        // Naming — proc, func, method, iterator, template, macro names
        r#"
            (proc_declaration name: (identifier) @name)
            (func_declaration name: (identifier) @name)
            (method_declaration name: (identifier) @name)
            (iterator_declaration name: (identifier) @name)
            (template_declaration name: (identifier) @name)
            (macro_declaration name: (identifier) @name)
            (converter_declaration name: (identifier) @name)
        "#,
        
        // Complexity — branches and boolean operators
        r#"
            (if_statement) @branch
            (elif_branch) @branch
            (when_statement) @branch
            (for_statement) @branch
            (while_statement) @branch
            (case_statement) @branch
            (of_branch) @branch
            (except_branch) @branch
            (try_statement) @branch
            (binary_expression operator: ["and" "or"]) @branch
        "#,
        
        // Imports
        r#"
            (import_statement (identifier) @import)
            (import_statement (expression_list (identifier) @import))
            (from_statement module: (identifier) @import)
            (include_statement (identifier) @import)
        "#,
        
        // Defs (Signatures)
        r#"
            (proc_declaration name: (identifier) @name) @sig
            (func_declaration name: (identifier) @name) @sig
            (method_declaration name: (identifier) @name) @sig
            (iterator_declaration name: (identifier) @name) @sig
            (template_declaration name: (identifier) @name) @sig
            (macro_declaration name: (identifier) @name) @sig
            (converter_declaration name: (identifier) @name) @sig
            (type_section) @sig
            (const_section) @sig
        "#,
        
        // Exports — Nim uses `*` postfix for public symbols
        r#"
            (proc_declaration name: (exported_symbol) @name) @export
            (func_declaration name: (exported_symbol) @name) @export
            (method_declaration name: (exported_symbol) @name) @export
            (iterator_declaration name: (exported_symbol) @name) @export
            (template_declaration name: (exported_symbol) @name) @export
            (macro_declaration name: (exported_symbol) @name) @export
            (type_declaration name: (exported_symbol) @name) @export
        "#,
        
        // Skeleton (routine bodies to strip)
        r#"
            (proc_declaration body: (statement_list) @body)
            (func_declaration body: (statement_list) @body)
            (method_declaration body: (statement_list) @body)
            (iterator_declaration body: (statement_list) @body)
            (template_declaration body: (statement_list) @body)
            (macro_declaration body: (statement_list) @body)
        "#,
    ],
];
```

> **Critical:** These node names are hypothetical based on typical tree-sitter-nim patterns. The `alaviss/tree-sitter-nim` grammar may use different node names (e.g., `routine_declaration` instead of `proc_declaration`, or `accent_quoted` for backtick-quoted identifiers). **Run query compilation tests immediately after adding — if any fail, inspect the actual grammar with `tree-sitter parse` on a sample `.nim` file and adjust node names accordingly.** This is the single highest-risk item in the spec.

---

## 4. Analysis Logic Updates

### 4.1 AST Analysis Dispatch

**File:** `src/analysis/ast.rs`

Add Nim dispatch in `run_analysis`:

```rust
if lang == Lang::Nim {
    nim_checks::check_nim_specifics(&ctx, &mut violations);
}
```

### 4.2 The "Law of Paranoia" — Nim Unsafe Constructs

**File:** `src/analysis/checks/nim_checks.rs` (NEW FILE)

Nim's unsafe surface is broader than Swift's but well-defined. The Nim manual explicitly lists these as unsafe:

| Construct | Risk | Equivalent in Rust |
|-----------|------|--------------------|
| `cast[T](x)` | Reinterpret bits, type safety destroyed | `std::mem::transmute` |
| `addr(x)` | Takes untraced pointer, can dangle | Raw pointer `&raw` |
| `unsafeAddr(x)` | Same as addr but for `let` bindings | Raw pointer to immutable |
| `ptr T` | Untraced (manual) pointer type | `*mut T` / `*const T` |
| `pointer` | Untyped raw pointer | `*mut c_void` |
| `{.emit: "...".}` | Inline C/C++ code injection | `extern "C"` + `unsafe` |
| `asm` statement | Inline assembly | `asm!()` |
| `copyMem` / `moveMem` / `zeroMem` | Raw memory manipulation | `std::ptr::copy` |
| `alloc` / `alloc0` / `dealloc` | Manual memory management | `std::alloc::alloc` |
| `{.push checks:off.}` | Disables runtime safety checks | Compiler flags |
| `{.boundChecks:off.}` | Disables bounds checking | `get_unchecked()` |
| `{.overflowChecks:off.}` | Disables overflow checking | `wrapping_add()` etc. |
| `{.noinit.}` | Uninitialized memory | `MaybeUninit` |
| `{.global.}` on local var | Hidden global mutable state | `static mut` |

```rust
//! Nim-specific safety checks: unsafe casts, raw pointers, inline C,
//! raw memory operations, and disabled runtime checks.

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

/// Entry point for Nim safety checks.
pub fn check_nim_specifics(
    ctx: &CheckContext,
    violations: &mut Vec<Violation>,
) {
    check_cast(ctx, violations);
    check_addr(ctx, violations);
    check_ptr_types(ctx, violations);
    check_emit_pragma(ctx, violations);
    check_asm_statement(ctx, violations);
    check_raw_memory_ops(ctx, violations);
    check_manual_alloc(ctx, violations);
    check_disabled_checks(ctx, violations);
    check_noinit(ctx, violations);
    check_global_pragma(ctx, violations);
}

// ---------------------------------------------------------------------------
// 4.2.1  cast[T](x) — Unsafe type cast
// ---------------------------------------------------------------------------

/// Detects `cast[T](expr)` — Nim's equivalent of transmute.
fn check_cast(ctx: &CheckContext, out: &mut Vec<Violation>) {
    // Try AST-based detection first
    let query_str = "(cast_expression) @cast";
    if let Ok(query) = Query::new(ctx.lang.grammar(), query_str) {
        let mut cursor = QueryCursor::new();
        for m in cursor.matches(&query, ctx.root, ctx.source.as_bytes()) {
            for cap in m.captures {
                let node = cap.node;
                if !has_safety_comment(ctx.source, node.start_position().row) {
                    out.push(Violation::with_details(
                        node.start_position().row + 1,
                        "Unsafe `cast` detected".to_string(),
                        "PARANOIA",
                        ViolationDetails {
                            function_name: None,
                            analysis: vec![
                                "cast[] reinterprets the bit pattern and destroys type safety.".into(),
                            ],
                            suggestion: Some(
                                "Use type conversion (e.g., `int(x)`) for safe conversions. \
                                 If cast is required, add a `# SAFETY:` comment explaining why.".into()
                            ),
                        },
                    ));
                }
            }
        }
        return;
    }
    
    // Fallback: text-based scan
    check_cast_fallback(ctx, out);
}

fn check_cast_fallback(ctx: &CheckContext, out: &mut Vec<Violation>) {
    for (line_num, line) in ctx.source.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') { continue; }
        // Match `cast[` pattern — the opening bracket distinguishes from the keyword alone
        if line.contains("cast[") {
            if !has_safety_comment(ctx.source, line_num) {
                out.push(Violation::simple(
                    line_num + 1,
                    "Unsafe `cast` detected".to_string(),
                    "PARANOIA",
                ));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 4.2.2  addr / unsafeAddr — Untraced pointer creation
// ---------------------------------------------------------------------------

/// Detects `addr(x)` and `unsafeAddr(x)` — creates dangling pointer risk.
fn check_addr(ctx: &CheckContext, out: &mut Vec<Violation>) {
    // Try AST-based detection
    let query_str = r#"
        (call function: (identifier) @fn_name
            (#match? @fn_name "^(addr|unsafeAddr)$"))
    "#;
    
    if let Ok(query) = Query::new(ctx.lang.grammar(), query_str) {
        let mut cursor = QueryCursor::new();
        let fn_idx = query.capture_index_for_name("fn_name").unwrap_or(0);
        for m in cursor.matches(&query, ctx.root, ctx.source.as_bytes()) {
            for cap in m.captures {
                if cap.index == fn_idx {
                    let node = cap.node;
                    let text = node.utf8_text(ctx.source.as_bytes()).unwrap_or("");
                    if !has_safety_comment(ctx.source, node.start_position().row) {
                        out.push(Violation::with_details(
                            node.start_position().row + 1,
                            format!("Unsafe `{text}` detected — creates untraced pointer"),
                            "PARANOIA",
                            ViolationDetails {
                                function_name: None,
                                analysis: vec![
                                    "addr/unsafeAddr returns an untraced pointer that can dangle.".into(),
                                    "The pointer may outlive the variable it points to.".into(),
                                ],
                                suggestion: Some(
                                    "Use `ref` for traced references. If addr is required for FFI, \
                                     add a `# SAFETY:` comment explaining lifetime guarantees.".into()
                                ),
                            },
                        ));
                    }
                }
            }
        }
        return;
    }
    
    // Fallback: text-based
    check_addr_fallback(ctx, out);
}

fn check_addr_fallback(ctx: &CheckContext, out: &mut Vec<Violation>) {
    let patterns = ["addr(", "addr ", "unsafeAddr(", "unsafeAddr "];
    for (line_num, line) in ctx.source.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') { continue; }
        for pat in &patterns {
            if trimmed.contains(pat) && !has_safety_comment(ctx.source, line_num) {
                let keyword = if pat.starts_with("unsafe") { "unsafeAddr" } else { "addr" };
                out.push(Violation::simple(
                    line_num + 1,
                    format!("Unsafe `{keyword}` detected — creates untraced pointer"),
                    "PARANOIA",
                ));
                break;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 4.2.3  ptr T / pointer — Raw pointer types
// ---------------------------------------------------------------------------

/// Detects `ptr T` and `pointer` type declarations.
/// Only flags type DECLARATIONS, not usage of existing ptr-typed variables.
fn check_ptr_types(ctx: &CheckContext, out: &mut Vec<Violation>) {
    // Try AST-based: look for ptr type expressions in type definitions
    let query_str = r#"
        (ptr_type) @ptr
    "#;
    
    if let Ok(query) = Query::new(ctx.lang.grammar(), query_str) {
        let mut cursor = QueryCursor::new();
        for m in cursor.matches(&query, ctx.root, ctx.source.as_bytes()) {
            for cap in m.captures {
                let node = cap.node;
                if !has_safety_comment(ctx.source, node.start_position().row) {
                    out.push(Violation::with_details(
                        node.start_position().row + 1,
                        "Untraced pointer type (`ptr`) detected".to_string(),
                        "PARANOIA",
                        ViolationDetails {
                            function_name: None,
                            analysis: vec![
                                "ptr creates an untraced (manually managed) pointer.".into(),
                                "Memory must be manually allocated and freed.".into(),
                            ],
                            suggestion: Some(
                                "Use `ref` for garbage-collected references. If ptr is needed \
                                 for FFI or hardware access, add a `# SAFETY:` comment.".into()
                            ),
                        },
                    ));
                }
            }
        }
        return;
    }

    // Fallback: detect `ptr ` in type sections
    for (line_num, line) in ctx.source.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') { continue; }
        // Match `: ptr `, `= ptr `, `ptr T` in type contexts
        if (trimmed.contains(": ptr ") || trimmed.contains("= ptr ") || trimmed.contains(": pointer"))
            && !has_safety_comment(ctx.source, line_num)
        {
            out.push(Violation::simple(
                line_num + 1,
                "Untraced pointer type (`ptr`/`pointer`) detected".to_string(),
                "PARANOIA",
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// 4.2.4  {.emit.} — Inline C/C++ code injection
// ---------------------------------------------------------------------------

/// Detects `{.emit: "...".}` pragma — injects raw C/C++ into output.
fn check_emit_pragma(ctx: &CheckContext, out: &mut Vec<Violation>) {
    // This is best done text-based since the pragma is a simple pattern.
    // AST may represent it as a generic pragma_expression.
    for (line_num, line) in ctx.source.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') { continue; }
        if trimmed.contains("{.emit") || trimmed.contains("{. emit") {
            if !has_safety_comment(ctx.source, line_num) {
                out.push(Violation::with_details(
                    line_num + 1,
                    "Inline C code injection (`{.emit.}`) detected".to_string(),
                    "PARANOIA",
                    ViolationDetails {
                        function_name: None,
                        analysis: vec![
                            "{.emit.} injects raw C/C++ code, bypassing all Nim safety guarantees.".into(),
                            "Nim cannot type-check, bounds-check, or manage memory for emitted code.".into(),
                        ],
                        suggestion: Some(
                            "Use Nim's FFI (`importc`, `importcpp`) for C interop. \
                             If emit is unavoidable, add a `# SAFETY:` comment \
                             explaining what the emitted code does and why it's safe.".into()
                        ),
                    },
                ));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 4.2.5  asm — Inline assembly
// ---------------------------------------------------------------------------

/// Detects `asm` statements.
fn check_asm_statement(ctx: &CheckContext, out: &mut Vec<Violation>) {
    let query_str = "(asm_statement) @asm";
    if let Ok(query) = Query::new(ctx.lang.grammar(), query_str) {
        let mut cursor = QueryCursor::new();
        for m in cursor.matches(&query, ctx.root, ctx.source.as_bytes()) {
            for cap in m.captures {
                let node = cap.node;
                if !has_safety_comment(ctx.source, node.start_position().row) {
                    out.push(Violation::with_details(
                        node.start_position().row + 1,
                        "Inline assembly (`asm`) detected".to_string(),
                        "PARANOIA",
                        ViolationDetails {
                            function_name: None,
                            analysis: vec![
                                "asm blocks bypass all of Nim's safety guarantees.".into(),
                            ],
                            suggestion: Some(
                                "Add a `# SAFETY:` comment explaining the assembly's correctness.".into()
                            ),
                        },
                    ));
                }
            }
        }
        return;
    }
    
    // Fallback
    for (line_num, line) in ctx.source.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') { continue; }
        if trimmed.starts_with("asm ") || trimmed == "asm" || trimmed.starts_with("asm\"") {
            if !has_safety_comment(ctx.source, line_num) {
                out.push(Violation::simple(
                    line_num + 1,
                    "Inline assembly (`asm`) detected".to_string(),
                    "PARANOIA",
                ));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 4.2.6  copyMem / moveMem / zeroMem — Raw memory operations
// ---------------------------------------------------------------------------

/// Detects raw memory manipulation functions.
fn check_raw_memory_ops(ctx: &CheckContext, out: &mut Vec<Violation>) {
    let dangerous_procs = [
        "copyMem", "moveMem", "zeroMem", "equalMem",
        "cmpMem",
    ];
    
    for (line_num, line) in ctx.source.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') { continue; }
        for proc_name in &dangerous_procs {
            if trimmed.contains(&format!("{proc_name}("))
                || trimmed.contains(&format!("{proc_name} "))
            {
                if !has_safety_comment(ctx.source, line_num) {
                    out.push(Violation::with_details(
                        line_num + 1,
                        format!("Raw memory operation `{proc_name}` detected"),
                        "PARANOIA",
                        ViolationDetails {
                            function_name: None,
                            analysis: vec![
                                format!("{proc_name} operates on raw memory without bounds checking."),
                                "Buffer overflows, use-after-free, and data corruption are possible.".into(),
                            ],
                            suggestion: Some(
                                "Use Nim's standard collections and copy semantics. \
                                 If raw memory ops are needed for performance, add a `# SAFETY:` comment.".into()
                            ),
                        },
                    ));
                }
                break;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 4.2.7  alloc / alloc0 / dealloc — Manual memory management
// ---------------------------------------------------------------------------

/// Detects manual allocation — memory that the GC won't track.
fn check_manual_alloc(ctx: &CheckContext, out: &mut Vec<Violation>) {
    let alloc_procs = [
        "alloc", "alloc0", "dealloc", "realloc", "resize",
        "allocShared", "allocShared0", "deallocShared", "reallocShared",
        "createShared", "freeShared",
        "create", // create(T) allocates untraced memory
    ];
    
    for (line_num, line) in ctx.source.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') { continue; }
        for proc_name in &alloc_procs {
            // Match `alloc(` or `alloc0(` specifically to avoid false positives
            // like `allocator` or `allocation`
            if trimmed.contains(&format!("{proc_name}(")) {
                // Extra check: `create` is common — only flag if it looks like memory alloc
                // i.e., skip if it's something like `createWindow(` or `createFile(`
                if *proc_name == "create" {
                    // `create(SomeType)` is the dangerous pattern
                    // Heuristic: next char after `create(` is uppercase = likely type alloc
                    if let Some(idx) = trimmed.find("create(") {
                        let after = &trimmed[idx + 7..];
                        if !after.starts_with(|c: char| c.is_uppercase()) {
                            continue;
                        }
                    }
                }
                
                if !has_safety_comment(ctx.source, line_num) {
                    out.push(Violation::with_details(
                        line_num + 1,
                        format!("Manual memory allocation `{proc_name}` detected"),
                        "PARANOIA",
                        ViolationDetails {
                            function_name: None,
                            analysis: vec![
                                format!("{proc_name} allocates untraced memory outside the GC."),
                                "You are responsible for calling dealloc — leaks and use-after-free are possible.".into(),
                            ],
                            suggestion: Some(
                                "Use `ref` types and `new()` for GC-managed allocation. \
                                 If manual allocation is needed for FFI/performance, \
                                 add a `# SAFETY:` comment with dealloc strategy.".into()
                            ),
                        },
                    ));
                }
                break;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 4.2.8  Disabled runtime checks — the silent killers
// ---------------------------------------------------------------------------

/// Detects pragmas that disable Nim's runtime safety checks.
/// These are the Nim equivalent of compiling Rust with `--cfg unsafe_code`.
fn check_disabled_checks(ctx: &CheckContext, out: &mut Vec<Violation>) {
    let dangerous_pragmas = [
        ("checks:off",          "ALL runtime checks"),
        ("boundChecks:off",     "array/seq bounds checking"),
        ("overflowChecks:off",  "integer overflow checking"),
        ("rangeChecks:off",     "range/subrange checking"),
        ("nilChecks:off",       "nil dereference checking"),
        ("nanChecks:off",       "NaN/Inf checking"),
        ("infChecks:off",       "infinity checking"),
        ("assertions:off",      "assertion enforcement"),
        ("fieldChecks:off",     "case object field checking"),
        ("objChecks:off",       "object conversion checking"),
    ];
    
    for (line_num, line) in ctx.source.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') { continue; }
        
        for (pragma, description) in &dangerous_pragmas {
            // Match both `{.pragma.}` and `{.push pragma.}` forms
            // Normalize spaces for matching
            let normalized = trimmed.replace(' ', "");
            if normalized.contains(pragma) {
                if !has_safety_comment(ctx.source, line_num) {
                    out.push(Violation::with_details(
                        line_num + 1,
                        format!("Runtime safety disabled: {description}"),
                        "PARANOIA",
                        ViolationDetails {
                            function_name: None,
                            analysis: vec![
                                format!("Disabling {description} removes Nim's runtime safety net."),
                                "Crashes become undefined behavior instead of catchable exceptions.".into(),
                            ],
                            suggestion: Some(
                                format!("Remove `{pragma}` unless profiling proves it's a bottleneck. \
                                         If needed, add a `# SAFETY:` comment with benchmarks justifying it.")
                            ),
                        },
                    ));
                }
                break;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 4.2.9  {.noinit.} — Uninitialized memory
// ---------------------------------------------------------------------------

/// Detects `{.noinit.}` pragma — variable used without initialization.
fn check_noinit(ctx: &CheckContext, out: &mut Vec<Violation>) {
    for (line_num, line) in ctx.source.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') { continue; }
        if trimmed.contains("{.noinit.}") || trimmed.contains("{.noInit.}") {
            if !has_safety_comment(ctx.source, line_num) {
                out.push(Violation::with_details(
                    line_num + 1,
                    "Uninitialized variable (`{.noinit.}`) detected".to_string(),
                    "PARANOIA",
                    ViolationDetails {
                        function_name: None,
                        analysis: vec![
                            "noinit skips zero-initialization, leaving memory in an undefined state.".into(),
                            "Reading before writing is undefined behavior.".into(),
                        ],
                        suggestion: Some(
                            "Remove noinit and let Nim zero-initialize. If performance-critical, \
                             add a `# SAFETY:` comment proving the variable is always written before read.".into()
                        ),
                    },
                ));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 4.2.10  {.global.} on local variables — Hidden mutable state
// ---------------------------------------------------------------------------

/// Detects `{.global.}` pragma on local variables — hidden global mutable state.
fn check_global_pragma(ctx: &CheckContext, out: &mut Vec<Violation>) {
    for (line_num, line) in ctx.source.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') { continue; }
        // {.global.} is only dangerous on local variables inside procs
        // We detect it broadly and let users annotate with SAFETY if intentional
        if (trimmed.contains("{.global.}") || trimmed.contains("{.global.}"))
            && (trimmed.starts_with("var ") || trimmed.starts_with("let "))
        {
            if !has_safety_comment(ctx.source, line_num) {
                out.push(Violation::with_details(
                    line_num + 1,
                    "Hidden global state (`{.global.}` on local variable)".to_string(),
                    "PARANOIA",
                    ViolationDetails {
                        function_name: None,
                        analysis: vec![
                            "{.global.} makes a local variable persist across calls like a static.".into(),
                            "This creates hidden mutable state that's invisible at the call site.".into(),
                        ],
                        suggestion: Some(
                            "Move the variable to module scope where it's visible, \
                             or pass it as a parameter. If global is intentional (e.g., caching), \
                             add a `# SAFETY:` comment.".into()
                        ),
                    },
                ));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Safety comment detection (shared helper)
// ---------------------------------------------------------------------------

/// Checks if the line (or the line above) has a `# SAFETY:` comment.
/// This mirrors Rust's `// SAFETY:` convention for `unsafe` blocks.
fn has_safety_comment(source: &str, line_idx: usize) -> bool {
    let lines: Vec<&str> = source.lines().collect();
    
    // Check current line for inline safety comment
    if let Some(line) = lines.get(line_idx) {
        if line.contains("# SAFETY:") || line.contains("# safety:") {
            return true;
        }
    }
    
    // Check line above for block safety comment
    if line_idx > 0 {
        if let Some(line) = lines.get(line_idx - 1) {
            let trimmed = line.trim();
            if trimmed.starts_with("# SAFETY:") || trimmed.starts_with("# safety:") {
                return true;
            }
        }
    }
    
    false
}
```

### 4.3 Profile Behavior: Systems vs Application

When `profile = "systems"` is active, the following Nim paranoia checks should be **relaxed to warnings** (not errors) while retaining the `# SAFETY:` comment requirement:

- `cast[]`
- `addr` / `unsafeAddr`
- `ptr T` / `pointer`
- `alloc` / `dealloc`
- `copyMem` / `moveMem` / `zeroMem`
- `{.noinit.}`

The following should **remain errors even in systems mode** (escalated safety):

- `{.emit.}` (inline C is never safe to leave undocumented)
- Disabled runtime checks (`checks:off`, etc.)
- `asm` statements

This mirrors the Swift spec's principle and the existing Rust profile behavior where `unsafe` blocks are allowed in systems mode but must always have `// SAFETY:` comments.

### 4.4 Metrics Compatibility

**File:** `src/analysis/ast.rs`

Update argument counting to recognize Nim's parameter syntax:

```rust
fn count_arguments(node: Node, source: &str) -> usize {
    // ... existing logic ...
    
    // Add Nim parameter support
    // Nim uses `formal_parameters` or `parameter_list` depending on grammar version
    let param_kinds = [
        "parameters", "formal_parameters", "parameter_clause", 
        "parameter_list", "routine_param",  // Nim-specific
    ];
    
    // ... rest of implementation ...
}
```

> **Note:** Nim allows parameter grouping: `proc foo(a, b, c: int)` counts as 3 arguments, not 1. Ensure the argument counter descends into grouped parameters.

---

## 5. Project Detection & Commands

**File:** `src/project.rs`

### 5.1 Project Type Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    Rust,
    Node,
    Python,
    Go,
    SwiftSpm,
    SwiftXcode,
    Nim,       // NEW
    Unknown,
}
```

### 5.2 Detection Logic

```rust
impl ProjectType {
    pub fn detect_in(path: &Path) -> Self {
        // ... existing checks for Rust, Node, Python, Go, Swift ...
        
        // Nim: look for .nimble file or nim.cfg
        if Self::has_nimble_file(path) {
            return Self::Nim;
        }
        
        // Nim fallback: nim.cfg or config.nims in root
        if path.join("nim.cfg").exists() || path.join("config.nims").exists() {
            return Self::Nim;
        }
        
        Self::Unknown
    }
    
    fn has_nimble_file(root: &Path) -> bool {
        if let Ok(entries) = std::fs::read_dir(root) {
            return entries
                .filter_map(Result::ok)
                .any(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "nimble")
                        .unwrap_or(false)
                });
        }
        false
    }
    
    /// Returns true if the project type uses Nim.
    pub fn is_nim(self) -> bool {
        matches!(self, Self::Nim)
    }
}
```

### 5.3 Default Commands

```rust
fn commands_section(project: ProjectType) -> String {
    match project {
        // ... existing Rust, Node, Python, Go, Swift ...
        
        ProjectType::Nim => {
            let test_cmd = "nimble test";
            let lint = tool_or_warning("nimpretty", "nimpretty --check .");
            
            // Determine compiler flags based on SlopChop profile
            // In application mode, enforce strict checks
            let compile_check = "nim check --hints:on --warnings:on src/*.nim";
            
            format!(
                r#"[commands]
check = ["{compile_check}", "{lint}", "{test_cmd}"]"#
            )
        }
        
        ProjectType::Unknown => { /* ... existing ... */ }
    }
}
```

### 5.4 Strict Nim Configuration Generator

**New feature:** `slopchop init --nim` generates a `nim.cfg` with strict defaults.

**File:** `src/project.rs` (or new `src/nim_config.rs`)

```rust
/// Generates a strict nim.cfg for SlopChop-governed projects.
pub fn generate_strict_nim_cfg() -> String {
    r#"# nim.cfg — SlopChop Strict Mode
# Generated by `slopchop init --nim`
# Philosophy: Maximum safety, opt-out with justification.

# Memory management: ARC (deterministic, no GC pauses, no tracing)
--mm:arc

# Treat warnings as errors
--warningAsError:on

# Enable all runtime checks (SlopChop will flag any code that disables these)
--checks:on
--boundChecks:on
--overflowChecks:on
--rangeChecks:on
--nilChecks:on
--assertions:on

# Style enforcement
--styleCheck:error

# Panics instead of exceptions for defects (like Rust's panic)
--panics:on

# Hint control (keep useful, suppress noise)
--hint:XDeclaredButNotUsed:on
--hint:ConvFromXtoItselfNotNeeded:on

# Performance: enable dangerous_off only in release builds
# --danger is NEVER used in SlopChop governed projects
"#.to_string()
}
```

This is the "blessed configuration" concept — `slopchop init --nim` doesn't just generate a `slopchop.toml`, it also generates the `nim.cfg` that makes Nim behave like a strict, safety-first language. Combined with SlopChop's paranoia checks, this gets you significantly closer to Rust-level discipline.

---

## 6. Prompt Engineering

**File:** `src/prompt.rs`

Update `build_system_prompt` to include Nim:

```rust
fn build_system_prompt(config: &RuleConfig) -> String {
    format!(
        r"SYSTEM MANDATE: THE SLOPCHOP PROTOCOL
// ... existing content ...

LAW OF PARANOIA:
- Rust: No `.unwrap()` or `.expect()`. Use `Result`/`Option` combinators.
- Swift: No force unwrap (`!`), force try (`try!`), or force cast (`as!`).
         Use `if let`, `guard let`, `try?`, `as?`, or `do/catch`.
- Nim:  No `cast[]`, `addr`, `ptr`, `{{.emit.}}`, `copyMem`, or disabled checks.
         Use type conversions, `ref`, GC-managed memory, and Nim's FFI pragmas.
         Every unsafe construct MUST have a `# SAFETY:` comment or SlopChop rejects it.

// ... rest of prompt ...
"
    )
}
```

---

## 7. V2 Metrics Exclusion

**File:** `src/analysis/v2/worker.rs`

Ensure the guard clause excludes Nim (V2 metrics like LCOM4/CBO need Nim-specific scope extraction that doesn't exist yet):

```rust
pub fn scan_file(path: &Path) -> Option<FileAnalysis> {
    let source = std::fs::read_to_string(path).ok()?;
    let lang = Lang::from_ext(path.extension()?.to_str()?)?;
    
    // V2 metrics only for Rust (Swift/Nim need language-specific extractors)
    if lang != Lang::Rust {
        return Some(FileAnalysis {
            violations: vec![],
            scopes: HashMap::new(),
            path_str: path.to_string_lossy().to_string(),
        });
    }
    
    // ... existing V2 analysis for Rust ...
}
```

> **Future:** Nim V2 scope extraction is a natural Phase 2 item. Nim's `type` sections with `object` definitions map reasonably well to the LCOM4/CBO model — a type with methods (procs that take it as first param) is analogous to a struct with impl block.

---

## 8. Nim-Specific Naming Convention Check

**File:** `src/analysis/checks/naming.rs` (UPDATE)

Nim has a unique identifier normalization rule: `fooBar`, `foo_bar`, and `foobar` are **the same identifier**. SlopChop should enforce a consistent style.

```rust
/// Nim naming convention check.
/// Nim's style guide recommends:
/// - Types: PascalCase
/// - Procs/funcs/variables: camelCase
/// - Constants: camelCase (controversial — some use UPPER_CASE)
///
/// SlopChop enforces: the FIRST character's case must match the convention.
/// We don't fight Nim's case insensitivity — we just enforce consistency.
fn check_nim_naming(name: &str, kind: &str) -> Option<String> {
    if name.is_empty() { return None; }
    
    let first = name.chars().next()?;
    
    match kind {
        "type" | "object" | "enum" | "tuple" => {
            if !first.is_uppercase() {
                return Some(format!(
                    "Type `{name}` should be PascalCase (start with uppercase)"
                ));
            }
        }
        "proc" | "func" | "method" | "iterator" | "converter" => {
            // Exception: operators in backticks like `+`, `$`, `==`
            if name.starts_with('`') { return None; }
            if !first.is_lowercase() {
                return Some(format!(
                    "Routine `{name}` should be camelCase (start with lowercase)"
                ));
            }
        }
        _ => {}
    }
    
    None
}
```

---

## 9. Testing & Verification

### 9.1 Query Compilation Test

**File:** `tests/nim_grammar.rs` (NEW)

```rust
use tree_sitter::{Parser, Query};

#[test]
fn test_nim_queries_compile() {
    let lang = tree_sitter_nim::language();
    
    // Test a subset of queries — adjust node names based on actual grammar
    let queries = [
        // Basic parsing
        "(proc_declaration) @proc",
        "(if_statement) @branch",
        "(import_statement) @import",
    ];
    
    for q in queries {
        let result = Query::new(lang, q);
        assert!(
            result.is_ok(),
            "Query failed to compile: {q}\nError: {:?}",
            result.err()
        );
    }
}

#[test]
fn test_nim_parse_basic() {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_nim::language()).unwrap();
    
    let source = r#"
import std/strutils

type
  User* = object
    name: string
    age: int

proc greet*(user: User): string =
  result = "Hello, " & user.name
"#;
    
    let tree = parser.parse(source, None);
    assert!(tree.is_some(), "Failed to parse Nim source");
}

#[test]
fn test_nim_grammar_node_discovery() {
    // This test helps discover actual node names in the grammar.
    // Run with `cargo test -- --nocapture` to see the AST structure.
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_nim::language()).unwrap();
    
    let source = r#"
proc foo(a, b: int): string =
  if a > b:
    return "yes"
  else:
    return "no"
"#;
    
    let tree = parser.parse(source, None).unwrap();
    let root = tree.root_node();
    
    // Print AST for inspection
    fn print_tree(node: tree_sitter::Node, source: &str, indent: usize) {
        let spacing = "  ".repeat(indent);
        let text = if node.child_count() == 0 {
            format!(" {:?}", node.utf8_text(source.as_bytes()).unwrap_or(""))
        } else {
            String::new()
        };
        eprintln!("{}{}{}", spacing, node.kind(), text);
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                print_tree(child, source, indent + 1);
            }
        }
    }
    
    print_tree(root, source, 0);
}
```

> **Important:** The `test_nim_grammar_node_discovery` test is a **development aid**. Run it first to discover actual node names, then update all queries in Section 3.2 to match. This is the Phase 1 gating step.

### 9.2 Paranoia Detection Test

**File:** `tests/fixtures/nim/paranoia.nim`

```nim
# This file tests SlopChop's Law of Paranoia for Nim

import std/strutils

# --- SHOULD BE FLAGGED (no SAFETY comment) ---

proc unsafeCast() =
  var x: int = 42
  let y = cast[float](x)          # FLAGGED: cast without SAFETY

proc unsafeAddr() =
  var x = 10
  let p = addr(x)                  # FLAGGED: addr without SAFETY

proc unsafePtr() =
  type Node = ptr object            # FLAGGED: ptr type without SAFETY
    data: int

proc unsafeEmit() =
  {.emit: "printf(\"hello\");".}   # FLAGGED: emit without SAFETY

proc unsafeMemory() =
  var buf: array[100, byte]
  copyMem(addr buf[0], addr buf[50], 50)  # FLAGGED: copyMem and addr

proc unsafeAlloc() =
  let p = alloc(100)               # FLAGGED: manual alloc without SAFETY

proc disabledChecks() =
  {.push boundChecks:off.}         # FLAGGED: disabled bounds checking
  var a: array[5, int]
  echo a[10]
  {.pop.}

proc uninitVar() =
  var x {.noinit.}: int            # FLAGGED: noinit without SAFETY

proc hiddenGlobal() =
  var counter {.global.} = 0       # FLAGGED: hidden global state


# --- SHOULD NOT BE FLAGGED (has SAFETY comment) ---

proc safeCast() =
  var x: int = 42
  # SAFETY: Converting int bit pattern to float for serialization.
  # The value is immediately validated after conversion.
  let y = cast[float](x)

proc safeAddr() =
  var x = 10
  # SAFETY: Pointer is used only within this scope for FFI call.
  let p = addr(x)

proc safeAlloc() =
  # SAFETY: Allocating buffer for C library interop. Freed in defer block.
  let p = alloc0(256)
  defer: dealloc(p)

# --- SHOULD NOT BE FLAGGED (safe constructs) ---

proc normalCode() =
  let x: int = 42
  let y = float(x)                 # Safe type conversion, not cast
  if not true: discard             # Negation, not unsafe
  let s = "hello"
  echo s.toUpperAscii()
```

**File:** `tests/nim_paranoia.rs`

```rust
#[test]
fn test_nim_paranoia_detection() {
    let source = include_str!("fixtures/nim/paranoia.nim");
    
    let violations = analyze_nim_source(source);
    
    let paranoia: Vec<_> = violations
        .iter()
        .filter(|v| v.law == "PARANOIA")
        .collect();
    
    // Count expected violations from the "SHOULD BE FLAGGED" section
    // cast, addr, ptr, emit, copyMem, addr (in copyMem line), alloc,
    // boundChecks:off, noinit, global = ~10 violations
    assert!(
        paranoia.len() >= 9,
        "Expected at least 9 paranoia violations, got {}: {:?}",
        paranoia.len(),
        paranoia
    );
    
    // Verify no violations from SAFETY-commented code
    let safe_cast_line = source.lines()
        .position(|l| l.contains("proc safeCast"))
        .unwrap();
    
    let false_positives: Vec<_> = paranoia
        .iter()
        .filter(|v| v.row > safe_cast_line + 1)
        .collect();
    
    // The "SHOULD NOT BE FLAGGED" sections should produce 0 violations
    // (approximate — depends on exact line numbers)
    assert!(
        false_positives.is_empty(),
        "Found false positives in SAFETY-commented code: {:?}",
        false_positives
    );
}
```

### 9.3 False Positive Test: Safe Nim Constructs

**File:** `tests/fixtures/nim/false_positive.nim`

```nim
# These should NOT trigger any paranoia violations

import std/[os, strutils, sequtils]

# Safe type conversions (NOT cast)
let x = int(3.14)
let y = float(42)
let s = $42

# Normal if/not usage (NOT related to unsafe constructs)
if not fileExists("test.txt"):
  echo "not found"

# String containing "cast" / "addr" in comments or strings
# cast is a keyword that might appear in docs
let docs = "This function uses cast internally"  # NOT a cast

# `ref` types are safe (traced references)
type
  SafeNode = ref object
    data: string

# Normal proc with no unsafe constructs
proc add(a, b: int): int =
  result = a + b

# Seq operations (NOT raw memory)
var items = @[1, 2, 3]
items.add(4)
let total = items.foldl(a + b)
```

### 9.4 Local Smoke Test

```bash
# Setup
mkdir -p /tmp/slop-nim/src
cd /tmp/slop-nim

# Create nimble file
cat > slop_nim.nimble << 'EOF'
# Package
version       = "0.1.0"
author        = "test"
description   = "Test"
license       = "MIT"
srcDir        = "src"
bin           = @["main"]

# Dependencies
requires "nim >= 2.0.0"
EOF

# Create test file
cat > src/main.nim << 'EOF'
import std/strutils

# Should trigger: too many args (if limit is 5)
proc godProc(a, b, c, d, e, f: int): int =
  # Should trigger: PARANOIA — cast without SAFETY
  let x = cast[float](a)
  
  # Should trigger: PARANOIA — disabled checks
  {.push boundChecks:off.}
  var arr: array[5, int]
  let y = arr[b]
  {.pop.}
  
  # Should NOT trigger (negation, safe code)
  if not (a > 0):
    return 0
  
  result = a + b + c + d + e + f
EOF

# Run
/path/to/target/debug/slopchop scan
```

**Expected Output:**
- 1 max-args violation (6 > 5)
- 1 PARANOIA violation (`cast` without SAFETY)
- 1 PARANOIA violation (`boundChecks:off` without SAFETY)
- 0 false positives on `not` or normal code

---

## 10. Autonomous Protocol Update

**File:** Wherever `generate_autonomous_protocol()` lives

Add Nim-specific guidance to the Laws of Physics section:

```markdown
## Nim-Specific Rules
If working in a Nim project:
- **NEVER use `cast[]`** without a `# SAFETY:` comment.
- **NEVER use `{.emit.}`** — use Nim's FFI (`importc`, `importcpp`) instead.
- **NEVER disable runtime checks** (`{.push checks:off.}`) without a `# SAFETY:` comment and benchmarks.
- **NEVER use `addr`/`ptr`** when `ref` would work.
- **Prefer `--mm:arc`** — it's deterministic and GC-free.
- The project's `nim.cfg` is part of the governance. Don't modify it to weaken checks.
```

---

## 11. Implementation Checklist

### Phase 1: Grammar Validation (GATE — do not proceed if this fails)
- [ ] Add `tree-sitter-nim` Git dependency to `Cargo.toml`
- [ ] Verify it compiles with existing `tree-sitter` version
- [ ] Run `test_nim_grammar_node_discovery` to inspect actual AST node names
- [ ] **Update ALL queries in Section 3.2 to match actual grammar** — this is the critical step
- [ ] Run query compilation tests — **STOP if any fail**

### Phase 2: Language Integration
- [ ] Add `Nim` variant to `Lang` enum with `from_ext`, `grammar`, `skeleton_replacement`
- [ ] Update `CODE_EXT_PATTERN` to include `.nim` / `.nims`
- [ ] Add `Nim` to `ProjectType` enum
- [ ] Implement Nimble project detection
- [ ] Add Nim default commands (`nimble test`, `nimpretty`, `nim check`)
- [ ] Implement `generate_strict_nim_cfg()` for `slopchop init --nim`

### Phase 3: Paranoia Checks
- [ ] Create `src/analysis/checks/nim_checks.rs`
- [ ] Implement `check_cast` with AST + fallback
- [ ] Implement `check_addr` with AST + fallback
- [ ] Implement `check_ptr_types` with AST + fallback
- [ ] Implement `check_emit_pragma` (text-based)
- [ ] Implement `check_asm_statement` with AST + fallback
- [ ] Implement `check_raw_memory_ops` (text-based)
- [ ] Implement `check_manual_alloc` with `create()` heuristic
- [ ] Implement `check_disabled_checks` (text-based pragma scan)
- [ ] Implement `check_noinit` (text-based)
- [ ] Implement `check_global_pragma` (text-based)
- [ ] Implement `has_safety_comment` helper
- [ ] Wire up profile-aware severity (systems vs application)

### Phase 4: Polish
- [ ] Add Nim naming convention checks (PascalCase types, camelCase procs)
- [ ] Update prompt with Nim paranoia rules
- [ ] Update autonomous protocol with Nim-specific rules
- [ ] Ensure V2 guard clause excludes Nim
- [ ] Add Nim to `slopchop map` output formatting

### Phase 5: Test
- [ ] Grammar node discovery test runs and outputs AST
- [ ] Query compilation tests pass (all 6 query categories)
- [ ] Paranoia detection test: ≥9 violations from unsafe code
- [ ] False positive test: 0 violations from safe code
- [ ] SAFETY comment suppression works correctly
- [ ] Local smoke test: correct violation count
- [ ] `slopchop check` on Nimble project runs `nimble test` (or warning)
- [ ] `slopchop init --nim` generates correct `nim.cfg` and `slopchop.toml`

---

## 12. Future: Nim V2 Scope Extraction (Phase 2 Roadmap)

Nim's type system maps to LCOM4/CBO analysis, but requires a Nim-specific extractor:

| Nim Construct | Maps To |
|---------------|---------|
| `type Foo = object` + procs with `self: Foo` first param | Struct + impl (Rust) |
| `type Foo = ref object` | Class with reference semantics |
| `method` declarations | Virtual dispatch (polymorphism) |
| `import` / `from` / `include` | Module coupling edges |
| `export` markers (`*` postfix) | Public API surface |

This would enable SlopChop to compute LCOM4 (cohesion) and CBO (coupling) for Nim types, completing the structural analysis parity with Rust.

---

## Summary: Nim vs Swift Paranoia Comparison

| | Swift Spec | Nim Spec |
|---|-----------|----------|
| **Unsafe constructs** | 3 (`!`, `try!`, `as!`) | 10+ (cast, addr, ptr, emit, asm, memory ops, disabled checks, noinit, global) |
| **Detection method** | AST token + parent filtering | AST where possible + text-based fallback + pragma scanning |
| **Suppression** | Not supported | `# SAFETY:` comment (mirrors Rust convention) |
| **Profile-aware** | No | Yes (systems mode relaxes ptr/cast, escalates emit/checks) |
| **Config generation** | No | Yes (`slopchop init --nim` generates strict `nim.cfg`) |
| **Naming rules** | None | PascalCase types, camelCase routines |

Nim has a significantly larger unsafe surface area than Swift, which actually makes SlopChop *more valuable* for Nim — there's more to govern, and no existing tool does it.
