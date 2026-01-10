// src/analysis/v2/patterns/performance.rs
//! Performance anti-patterns: P01, P02, P04, P06
//! Tuned for high-signal detection of "Loop-Carried" costs.

use crate::types::{Violation, ViolationDetails};
use std::path::Path;
use tree_sitter::{Node, Query, QueryCursor};

/// Detects performance violations in Rust code.
#[must_use]
pub fn detect(source: &str, root: Node, path: &Path) -> Vec<Violation> {
    if should_skip_perf(path) {
        return Vec::new();
    }

    let mut violations = Vec::new();
    detect_loops(source, root, &mut violations);
    violations
}

fn should_skip_perf(path: &Path) -> bool {
    let s = path.to_string_lossy();
    // 1. Skip IO-bound CLI/UI code
    if s.contains("/cli/") || s.contains("/ui/") || s.contains("/tui/") {
        return true;
    }
    if s.contains("reporting") || s.contains("messages") {
        return true;
    }
    
    // 2. Skip Self-Scanning Meta-Noise
    // The analysis engine allocates strings to report violations.
    // This is "Business Logic" for a linter, not a performance bug.
    if s.contains("analysis/") {
        return true;
    }

    // 3. Skip Batch Tools
    // Audit/Pack/Signatures are one-off batch processes.
    // We prioritize the Core (Graph/Apply) and Discovery.
    if s.contains("audit/") || s.contains("pack/") || s.contains("signatures/") {
        return true;
    }

    if s.ends_with("main.rs") {
        return true;
    }

    // NOTE: `graph/` and `apply/` are NOT skipped. 
    // They are the Core Runtime. Hotspots there matter.

    false
}

/// Scans for loop-related performance issues.
/// - P01: Clone in loop (Deep copy hotspot)
/// - P02: Allocation in loop (Heap thrashing)
/// - P04: Nested iteration (O(n^2))
/// - P06: Linear search in loop (O(n*m))
fn detect_loops(source: &str, root: Node, out: &mut Vec<Violation>) {
    let query_str = r"
        (for_expression body: (block) @body) @loop
        (while_expression body: (block) @body) @loop
        (loop_expression body: (block) @body) @loop
    ";

    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else { return; };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(body_node) = m.captures.iter().find(|c| c.index == 1).map(|c| c.node) {
            check_p01_clone(source, body_node, out);
            check_p02_alloc(source, body_node, out);
            check_p04_nested(source, body_node, out);
            check_p06_linear(source, body_node, out);
        }
    }
}

/// P01: `.clone()` in loop
/// This is a proxy for "Loop-Carried Allocation".
fn check_p01_clone(source: &str, body: Node, out: &mut Vec<Violation>) {
    let query_str = r#"(call_expression function: (field_expression field: (field_identifier) @method (#eq? @method "clone"))) @call"#;
    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else { return; };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, body, source.as_bytes()) {
        if let Some(cap) = m.captures.first() {
            let row = cap.node.start_position().row + 1;
            out.push(Violation::with_details(
                row,
                "Detected `.clone()` inside a loop".to_string(),
                "P01",
                ViolationDetails {
                    function_name: None,
                    analysis: vec!["Deep cloning inside a loop creates O(n) allocation pressure.".into()],
                    suggestion: Some("Hoist the clone, borrow the data, or use `Cow`/`Arc`.".into()),
                }
            ));
        }
    }
}

/// P02: Allocations inside loop
/// Targets: `to_string`, `to_owned`, `with_capacity`.
/// (Note: `Vec::new()` is excluded as it is non-allocating in Rust).
/// (Note: `format!` is excluded as it is often used in benign I/O contexts).
fn check_p02_alloc(source: &str, body: Node, out: &mut Vec<Violation>) {
    // 1. Method calls that imply allocation
    let method_q = r#"
        (call_expression
            function: (field_expression field: (field_identifier) @method)
            (#match? @method "^(to_string|to_owned|into_owned)$")) @call
    "#;
    
    // 2. Explicit capacity allocation
    let cap_q = r#"
        (call_expression
            function: (scoped_identifier name: (identifier) @method)
            (#eq? @method "with_capacity")) @call
    "#;

    run_alloc_query(source, body, method_q, "String conversion", out);
    run_alloc_query(source, body, cap_q, "Pre-allocation", out);
}

fn run_alloc_query(source: &str, body: Node, q_str: &str, label: &str, out: &mut Vec<Violation>) {
    let Ok(query) = Query::new(tree_sitter_rust::language(), q_str) else { return; };
    let mut cursor = QueryCursor::new();
    
    for m in cursor.matches(&query, body, source.as_bytes()) {
        if let Some(cap) = m.captures.last() {
            let row = cap.node.start_position().row + 1;
            out.push(Violation::with_details(
                row,
                format!("{label} inside loop"),
                "P02",
                ViolationDetails {
                    function_name: None,
                    analysis: vec!["Heap allocation inside a hot path.".into()],
                    suggestion: Some("Hoist allocation or reuse a buffer.".into()),
                }
            ));
        }
    }
}

/// P04: Nested loops
fn check_p04_nested(_source: &str, body: Node, out: &mut Vec<Violation>) {
    let mut cursor = body.walk();
    for child in body.children(&mut cursor) {
        let kind = child.kind();
        if matches!(kind, "for_expression" | "while_expression" | "loop_expression") {
            let row = child.start_position().row + 1;
            out.push(Violation::with_details(
                row,
                "Nested loop detected (O(n²) complexity)".to_string(),
                "P04",
                ViolationDetails {
                    function_name: None,
                    analysis: vec!["Nested iteration usually implies quadratic complexity.".into()],
                    suggestion: Some("Flatten the loop, use a lookup map (O(1)), or use iterator chains.".into()),
                }
            ));
        }
    }
}

/// P06: Linear search in loop
/// Targets: `.find()`, `.position()`, `.rposition()`
/// (Note: `.contains()` is excluded because it is O(1) for Sets/Maps, reducing noise).
fn check_p06_linear(source: &str, body: Node, out: &mut Vec<Violation>) {
    let query_str = r#"
        (call_expression 
            function: (field_expression field: (field_identifier) @method) 
            (#match? @method "^(find|position|rposition)$")) @search
    "#;
    
    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else { return; };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, body, source.as_bytes()) {
        if let Some(cap) = m.captures.last() {
            let row = cap.node.start_position().row + 1;
            let text = cap.node.utf8_text(source.as_bytes()).unwrap_or("method");
            out.push(Violation::with_details(
                row,
                format!("Linear search `.{text}()` inside loop"),
                "P06",
                ViolationDetails {
                    function_name: None,
                    analysis: vec![
                        "Searching a collection inside a loop is O(n*m).".into(),
                        "Iterating to find an element is a linear operation.".into()
                    ],
                    suggestion: Some("Use a `HashSet` or `BTreeSet` for lookups, or index the data first.".into()),
                }
            ));
        }
    }
}