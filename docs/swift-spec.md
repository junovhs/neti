# Final Specification: Tier-1 Swift Support for SlopChop

**Version:** 2.0 (Consolidated)  
**Status:** Implementation-Ready

---

## 1. Objectives

1. **Syntactic Parity:** Enable SlopChop to parse `.swift` files, calculate complexity, and enforce naming conventions using Tree-sitter.
2. **Law of Paranoia:** Implement Swift-specific bans on unsafe constructs (`!`, `try!`, `as!`) equivalent to Rust's `.unwrap()`.
3. **Linux-First Workflow:** Configure command generation to support local linting on Linux while deferring build/test logic to CI for Xcode projects.

---

## 2. Dependencies & Discovery

### 2.1 Cargo Dependencies

**File:** `Cargo.toml`

```toml
[dependencies]
# ... existing
tree-sitter-swift = "0.3.4"  # Pinned for tree-sitter 0.20 compatibility
```

> **Note:** Verify 0.3.4 compatibility before implementation. If unavailable, you may need to update `tree-sitter` across the board.

### 2.2 File Discovery

**File:** `src/constants.rs`

Update the regex to include `.swift`:

```rust
pub const CODE_EXT_PATTERN: &str = r"(?i)\.(rs|go|py|js|jsx|ts|tsx|java|c|cpp|h|hpp|cs|php|rb|sh|sql|html|css|scss|json|toml|yaml|md|swift)$";
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
    Swift,  // NEW
}

impl Lang {
    #[must_use]
    pub fn from_ext(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Self::Rust),
            "py" => Some(Self::Python),
            "ts" | "tsx" | "js" | "jsx" => Some(Self::TypeScript),
            "swift" => Some(Self::Swift),  // NEW
            _ => None,
        }
    }

    #[must_use]
    pub fn grammar(self) -> Language {
        match self {
            Self::Rust => tree_sitter_rust::language(),
            Self::Python => tree_sitter_python::language(),
            Self::TypeScript => tree_sitter_typescript::language_typescript(),
            Self::Swift => tree_sitter_swift::language(),  // NEW
        }
    }
    
    #[must_use]
    pub fn skeleton_replacement(self) -> &'static str {
        match self {
            Self::Python => "\n    ...",
            Self::Swift => "{ ... }",  // NEW
            _ => "{ ... }",
        }
    }
}
```

### 3.2 Tree-Sitter Queries

Expand `QUERIES` to `[[&str; 6]; 4]`:

```rust
// [Rust, Python, TypeScript, Swift] x [Naming, Complexity, Imports, Defs, Exports, Skeleton]
const QUERIES: [[&str; 6]; 4] = [
    // ... existing Rust, Python, TypeScript entries ...
    
    // Swift (Index 3)
    [
        // Naming
        "(function_declaration name: (simple_identifier) @name)",
        
        // Complexity
        r#"
            (if_statement) @branch
            (guard_statement) @branch
            (for_statement) @branch
            (while_statement) @branch
            (repeat_while_statement) @branch
            (catch_clause) @branch
            (switch_statement) @branch
            (ternary_expression) @branch
            (binary_expression operator: ["&&" "||"]) @branch
        "#,
        
        // Imports (captures module name, normalize complex forms in Rust)
        r#"
            (import_declaration (identifier) @import)
            (import_declaration (scoped_identifier) @import)
        "#,
        
        // Defs (Signatures)
        r#"
            (function_declaration name: (simple_identifier) @name) @sig
            (class_declaration name: (type_identifier) @name) @sig
            (struct_declaration name: (type_identifier) @name) @sig
            (enum_declaration name: (type_identifier) @name) @sig
            (protocol_declaration name: (type_identifier) @name) @sig
            (extension_declaration) @sig
            (init_declaration) @sig
        "#,
        
        // Exports (visibility-based)
        r#"
            (function_declaration (visibility_modifier) @vis) @export
            (class_declaration (visibility_modifier) @vis) @export
            (struct_declaration (visibility_modifier) @vis) @export
            (enum_declaration (visibility_modifier) @vis) @export
            (protocol_declaration (visibility_modifier) @vis) @export
        "#,
        
        // Skeleton (function bodies to strip)
        r#"
            (function_declaration body: (code_block) @body)
            (init_declaration body: (code_block) @body)
            (deinit_declaration body: (code_block) @body)
        "#,
    ],
];
```

> **Critical:** These node names are based on tree-sitter-swift 0.3.x. Run query compilation tests immediately after adding to verify they match your pinned version.

---

## 4. Analysis Logic Updates

### 4.1 AST Analysis Dispatch

**File:** `src/analysis/ast.rs`

Add Swift dispatch in `run_analysis`:

```rust
if lang == Lang::Swift {
    swift_checks::check_swift_specifics(&ctx, &mut violations);
}
```

### 4.2 The "Law of Paranoia" — Token-Based Detection

**File:** `src/analysis/checks/swift_checks.rs` (NEW FILE)

This is the critical section. Swift's unsafe constructs must be detected via **token/operator nodes with parent-context filtering**, not high-level expression nodes.

```rust
//! Swift-specific safety checks: force unwrap, force try, force cast.

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

/// Entry point for Swift safety checks.
pub fn check_swift_specifics(
    ctx: &CheckContext,
    violations: &mut Vec<Violation>,
) {
    check_force_unwrap(ctx, violations);
    check_force_try(ctx, violations);
    check_force_cast(ctx, violations);
}

/// Detects postfix `!` (force unwrap), excluding prefix `!` (negation).
fn check_force_unwrap(ctx: &CheckContext, out: &mut Vec<Violation>) {
    // Query for bang tokens
    let query_str = "(bang) @bang";
    let Ok(query) = Query::new(ctx.lang.grammar(), query_str) else {
        return;
    };
    
    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, ctx.root, ctx.source.as_bytes()) {
        for cap in m.captures {
            let node = cap.node;
            
            // Filter: only flag POSTFIX unwrap, not PREFIX negation
            if is_postfix_unwrap(node, ctx.source) {
                out.push(Violation::with_details(
                    node.start_position().row + 1,
                    "Force unwrap (`!`) detected".to_string(),
                    "PARANOIA",
                    ViolationDetails {
                        function_name: None,
                        analysis: vec![
                            "Force unwrap will crash if the value is nil.".into(),
                        ],
                        suggestion: Some(
                            "Use `if let`, `guard let`, or `??` with a default value.".into()
                        ),
                    },
                ));
            }
        }
    }
}

/// Determines if a `bang` node is a postfix unwrap (not prefix negation).
fn is_postfix_unwrap(node: Node, source: &str) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    
    // Postfix unwrap contexts (adjust based on actual grammar inspection)
    let postfix_kinds = [
        "force_unwrap_expression",
        "postfix_expression", 
        "optional_chaining_expression",
        // Add more as discovered from grammar inspection
    ];
    
    if postfix_kinds.contains(&parent.kind()) {
        return true;
    }
    
    // Heuristic fallback: check if `!` follows an identifier/expression
    // (i.e., `foo!` not `!foo`)
    if let Some(prev_sibling) = node.prev_sibling() {
        let prev_kind = prev_sibling.kind();
        // If preceded by an identifier or closing paren/bracket, it's postfix
        if matches!(prev_kind, "simple_identifier" | ")" | "]" | "self") {
            return true;
        }
    }
    
    false
}

/// Detects `try!` (force try).
fn check_force_try(ctx: &CheckContext, out: &mut Vec<Violation>) {
    // Query for try operators
    let query_str = "(try_operator) @try";
    let Ok(query) = Query::new(ctx.lang.grammar(), query_str) else {
        // Fallback: try alternative node name
        check_force_try_fallback(ctx, out);
        return;
    };
    
    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, ctx.root, ctx.source.as_bytes()) {
        for cap in m.captures {
            let node = cap.node;
            let text = node.utf8_text(ctx.source.as_bytes()).unwrap_or("");
            
            if text == "try!" {
                out.push(Violation::with_details(
                    node.start_position().row + 1,
                    "Force try (`try!`) detected".to_string(),
                    "PARANOIA",
                    ViolationDetails {
                        function_name: None,
                        analysis: vec![
                            "Force try will crash if the expression throws.".into(),
                        ],
                        suggestion: Some(
                            "Use `try?` for optional result, or `do/catch` for error handling.".into()
                        ),
                    },
                ));
            }
        }
    }
}

/// Fallback if `try_operator` doesn't exist in grammar.
fn check_force_try_fallback(ctx: &CheckContext, out: &mut Vec<Violation>) {
    // Text-based scan as last resort
    for (line_num, line) in ctx.source.lines().enumerate() {
        if line.contains("try!") && !line.trim_start().starts_with("//") {
            out.push(Violation::simple(
                line_num + 1,
                "Force try (`try!`) detected".to_string(),
                "PARANOIA",
            ));
        }
    }
}

/// Detects `as!` (force cast).
fn check_force_cast(ctx: &CheckContext, out: &mut Vec<Violation>) {
    // Query for as operators
    let query_str = "(as_operator) @as";
    let Ok(query) = Query::new(ctx.lang.grammar(), query_str) else {
        // Fallback: try alternative node name
        check_force_cast_fallback(ctx, out);
        return;
    };
    
    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, ctx.root, ctx.source.as_bytes()) {
        for cap in m.captures {
            let node = cap.node;
            let text = node.utf8_text(ctx.source.as_bytes()).unwrap_or("");
            
            if text == "as!" {
                out.push(Violation::with_details(
                    node.start_position().row + 1,
                    "Force cast (`as!`) detected".to_string(),
                    "PARANOIA",
                    ViolationDetails {
                        function_name: None,
                        analysis: vec![
                            "Force cast will crash if the type doesn't match.".into(),
                        ],
                        suggestion: Some(
                            "Use `as?` for conditional cast, or check type with `is` first.".into()
                        ),
                    },
                ));
            }
        }
    }
}

/// Fallback if `as_operator` doesn't exist in grammar.
fn check_force_cast_fallback(ctx: &CheckContext, out: &mut Vec<Violation>) {
    for (line_num, line) in ctx.source.lines().enumerate() {
        // Match ` as! ` to avoid false positives in comments/strings
        if line.contains(" as! ") && !line.trim_start().starts_with("//") {
            out.push(Violation::simple(
                line_num + 1,
                "Force cast (`as!`) detected".to_string(),
                "PARANOIA",
            ));
        }
    }
}
```

### 4.3 Metrics Compatibility

**File:** `src/analysis/ast.rs` (or wherever `count_arguments` lives)

Update argument counting to recognize Swift's parameter syntax:

```rust
fn count_arguments(node: Node, source: &str) -> usize {
    // ... existing logic ...
    
    // Add Swift parameter support
    let param_kinds = ["parameters", "formal_parameters", "parameter_clause", "parameter_list"];
    
    // ... rest of implementation ...
}
```

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
    SwiftSpm,    // NEW: Linux-compatible Swift Package Manager
    SwiftXcode,  // NEW: macOS/CI only
    Unknown,
}
```

### 5.2 Detection Logic

```rust
impl ProjectType {
    pub fn detect() -> Self {
        Self::detect_in(&std::env::current_dir().unwrap_or_default())
    }
    
    pub fn detect_in(path: &Path) -> Self {
        // ... existing checks for Rust, Node, Python, Go ...
        
        // Swift SPM: Package.swift in root
        if path.join("Package.swift").exists() {
            return Self::SwiftSpm;
        }
        
        // Swift Xcode: .xcodeproj or .xcworkspace (search max depth 3)
        if Self::has_xcode_project(path) {
            return Self::SwiftXcode;
        }
        
        Self::Unknown
    }
    
    fn has_xcode_project(root: &Path) -> bool {
        use walkdir::WalkDir;
        
        WalkDir::new(root)
            .max_depth(3)
            .into_iter()
            .filter_map(Result::ok)
            .any(|e| {
                let name = e.file_name().to_string_lossy();
                name.ends_with(".xcodeproj") || name.ends_with(".xcworkspace")
            })
    }
}
```

### 5.3 Default Commands with Tool Presence Detection

```rust
fn commands_section(project: ProjectType) -> String {
    match project {
        // ... existing Rust, Node, Python, Go ...
        
        ProjectType::SwiftSpm => {
            let lint = tool_or_warning("swiftlint", "swiftlint lint --strict");
            let fix = tool_or_warning("swift-format", "swift-format format -i -r .");
            let test = "swift test";
            
            format!(
                r#"[commands]
check = ["{lint}", "{test}"]
fix = "{fix}""#
            )
        }
        
        ProjectType::SwiftXcode => {
            let lint = tool_or_warning("swiftlint", "swiftlint lint --strict");
            
            format!(
                r#"[commands]
check = ["{lint}", "echo '⚠️  Push to CI for Xcode build/test (xcodebuild not available locally)'"]
fix = "echo 'Configure swift-format or swiftlint --fix manually'""#
            )
        }
        
        ProjectType::Unknown => { /* ... existing ... */ }
    }
}

/// Returns the command if the tool exists, or a warning echo if not.
fn tool_or_warning(tool: &str, command: &str) -> String {
    if tool_exists(tool) {
        command.to_string()
    } else {
        format!("echo '⚠️  {tool} not found. Install it or configure commands manually.'")
    }
}

fn tool_exists(name: &str) -> bool {
    std::process::Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
```

---

## 6. Prompt Engineering

**File:** `src/prompt.rs`

Update `build_system_prompt` to include Swift in the "LAW OF PARANOIA" section:

```rust
fn build_system_prompt(config: &RuleConfig) -> String {
    // ... existing setup ...
    
    format!(
        r"SYSTEM MANDATE: THE SLOPCHOP PROTOCOL
// ... existing content ...

LAW OF PARANOIA:
- Rust: No `.unwrap()` or `.expect()`. Use `Result`/`Option` combinators.
- Swift: No force unwrap (`!`), force try (`try!`), or force cast (`as!`). 
         Use `if let`, `guard let`, `try?`, `as?`, or `do/catch`.

// ... rest of prompt ...
"
    )
}
```

---

## 7. V2 Metrics Exclusion

**File:** `src/analysis/v2/worker.rs`

Ensure the guard clause remains:

```rust
pub fn scan_file(path: &Path) -> Option<FileAnalysis> {
    let source = std::fs::read_to_string(path).ok()?;
    let lang = Lang::from_ext(path.extension()?.to_str()?)?;
    
    // V2 metrics only for Rust (Swift extensions break LCOM4/CBO)
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

---

## 8. Backup System Removal

**Files:** Multiple

The `.slopchop_apply_backup` system is obsolete now that the workflow uses git branches (`slopchop-work`) for rollback.

### 8.1 Remove the backup module

Delete `src/apply/backup.rs`.

### 8.2 Update `src/apply/mod.rs`

```rust
// Remove this line:
// pub mod backup;
```

### 8.3 Update `src/apply/writer.rs`

Remove calls to `backup::create_backup()` and `backup::cleanup_old_backups()`.

### 8.4 Update `src/config.rs`

Remove `backup_retention` from preferences (or keep it and ignore it for backward compatibility).

### 8.5 Update `.gitignore` template

Remove `.slopchop_apply_backup/` from any generated gitignore.

---

## 9. Testing & Verification

### 9.1 Query Compilation Test

**File:** `tests/swift_grammar.rs` (NEW)

```rust
use tree_sitter::{Parser, Query};

#[test]
fn test_swift_queries_compile() {
    let lang = tree_sitter_swift::language();
    
    let queries = [
        // Naming
        "(function_declaration name: (simple_identifier) @name)",
        // Complexity (subset)
        "(if_statement) @branch",
        "(guard_statement) @branch",
        // Imports
        "(import_declaration (identifier) @import)",
        // Bang (for force unwrap detection)
        "(bang) @bang",
    ];
    
    for q in queries {
        let result = Query::new(lang, q);
        assert!(result.is_ok(), "Query failed to compile: {q}\nError: {:?}", result.err());
    }
}

#[test]
fn test_swift_parse_basic() {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_swift::language()).unwrap();
    
    let source = r#"
        import Foundation
        
        public class User {
            func parse(data: Data) {
                let x = try! JSONSerialization.jsonObject(with: data)
                if x == nil { return }
            }
        }
    "#;
    
    let tree = parser.parse(source, None);
    assert!(tree.is_some(), "Failed to parse Swift source");
}
```

### 9.2 Force Unwrap False Positive Test

**File:** `tests/fixtures/swift/false_positive.swift`

```swift
// This file tests that prefix negation is NOT flagged as force unwrap

func testNegation(flag: Bool, opt: Int?) {
    // These should NOT be flagged (prefix negation):
    if !flag { return }
    guard !flag else { return }
    let inverted = !flag
    
    // These SHOULD be flagged (postfix force unwrap):
    let value = opt!
    print(opt!)
}
```

**File:** `tests/swift_paranoia.rs`

```rust
#[test]
fn test_force_unwrap_vs_negation() {
    let source = include_str!("fixtures/swift/false_positive.swift");
    
    let violations = analyze_swift_source(source);
    
    // Should find exactly 2 violations (the two `opt!` usages)
    let unwrap_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.message.contains("Force unwrap"))
        .collect();
    
    assert_eq!(
        unwrap_violations.len(), 
        2, 
        "Expected 2 force unwrap violations, got {}: {:?}",
        unwrap_violations.len(),
        unwrap_violations
    );
    
    // Verify line numbers point to the correct lines (the `opt!` ones)
    for v in &unwrap_violations {
        assert!(v.row >= 10, "Violation at line {} is in the negation section", v.row);
    }
}
```

### 9.3 Local Smoke Test (Linux)

```bash
# Setup
mkdir -p /tmp/slop-swift/src
cd /tmp/slop-swift
touch Package.swift

# Create test file
cat > src/Bad.swift << 'EOF'
import Foundation

// Should trigger max args (if limit is 5)
func godFunction(a: Int, b: Int, c: Int, d: Int, e: Int, f: Int) {
    // Should trigger PARANOIA
    let danger = someOptional!
    let crash = try! riskyOperation()
    let cast = something as! SpecificType
    
    // Should NOT trigger (negation, not unwrap)
    if !flag { return }
}
EOF

# Run
/path/to/target/debug/slopchop scan
```

**Expected Output:**
- 3 PARANOIA violations (`!`, `try!`, `as!`)
- 1 max-args violation (6 > 5)
- 0 false positives on `!flag`

### 9.4 CI Verification (for Xcode projects)

Create `.github/workflows/ios.yml`:

```yaml
name: iOS CI

on:
  push:
    paths:
      - '**/*.swift'
      - '*.xcodeproj/**'
      - '*.xcworkspace/**'

jobs:
  build-and-test:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Select Xcode
        run: sudo xcode-select -s /Applications/Xcode_15.2.app
      
      - name: Build
        run: xcodebuild build -scheme MyApp -destination 'platform=iOS Simulator,name=iPhone 15'
      
      - name: Test
        run: xcodebuild test -scheme MyApp -destination 'platform=iOS Simulator,name=iPhone 15'
```

---

## 10. Implementation Checklist

### Phase 1: Foundation
- [ ] Add `tree-sitter-swift = "0.3.4"` to `Cargo.toml`
- [ ] Verify it compiles with existing `tree-sitter = "0.20"`
- [ ] Add `Swift` variant to `Lang` enum
- [ ] Add Swift queries to `QUERIES` array
- [ ] Run query compilation tests — **STOP if any fail**

### Phase 2: Detection
- [ ] Update `CODE_EXT_PATTERN` to include `.swift`
- [ ] Add `SwiftSpm` and `SwiftXcode` to `ProjectType`
- [ ] Implement project detection logic
- [ ] Add tool presence checking for `swiftlint`/`swift-format`

### Phase 3: Analysis
- [ ] Create `src/analysis/checks/swift_checks.rs`
- [ ] Implement token-based force unwrap detection with negation filtering
- [ ] Implement force try detection with fallback
- [ ] Implement force cast detection with fallback
- [ ] Wire up `check_swift_specifics` in `ast.rs`

### Phase 4: Polish
- [ ] Update prompt with Swift paranoia rules
- [ ] Ensure V2 guard clause excludes Swift
- [ ] Remove backup system (optional but recommended)

### Phase 5: Test
- [ ] Query compilation tests pass
- [ ] False positive test passes (`!flag` not flagged)
- [ ] Local smoke test shows correct violations
- [ ] `slopchop check` on SPM project runs `swiftlint` (or warning)

---

## Summary of Changes from Original Spec

| Section | Original | Final |
|---------|----------|-------|
| 4.2 Banned Constructs | AST expression nodes (`try_expression`, etc.) | Token/operator nodes (`bang`, `try_operator`, `as_operator`) + parent-context filtering |
| 4.2 Messages | Generic "Unsafe construct (`!`)" | Construct-specific messages |
| 4.2 Fallbacks | None | Text-based fallbacks if queries fail |
| 5.3 Commands | Assumed tools exist | Tool presence detection with warnings |
| 8 (NEW) | N/A | Backup system removal |
| 9.2 (NEW) | N/A | False positive test for `!flag` |
