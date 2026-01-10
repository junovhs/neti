// src/analysis/v2/patterns/performance.rs
//! Performance anti-patterns: P01, P02, P04, P06

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

/// Detects performance violations in Rust code.
#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut violations = Vec::new();
    detect_loops(source, root, &mut violations);
    violations
}

/// Scans for loop-related performance issues.
/// - P01: Clone in loop
/// - P02: Allocation in loop
/// - P04: Nested iteration
/// - P06: Linear search in loop
fn detect_loops(source: &str, root: Node, out: &mut Vec<Violation>) {
    // Matches for, while, and loop expressions
    let query_str = r"
        (for_expression body: (block) @body) @loop
        (while_expression body: (block) @body) @loop
        (loop_expression body: (block) @body) @loop
    ";

    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else { return; };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        // We capture @loop at index 0, @body at index 1
        if let Some(body_node) = m.captures.iter().find(|c| c.index == 1).map(|c| c.node) {
            check_p01_clone(source, body_node, out);
            check_p02_alloc(source, body_node, out);
            check_p04_nested(source, body_node, out);
            check_p06_linear(source, body_node, out);
        }
    }
}

/// P01: `.clone()` in loop
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
                    analysis: vec!["Cloning inside a loop can be a major performance bottleneck.".into()],
                    suggestion: Some("Consider borrowing, using `Rc`/`Arc`, or moving out of the loop.".into()),
                }
            ));
        }
    }
}

/// P02: `Vec::new()` or `String::new()` in loop
fn check_p02_alloc(source: &str, body: Node, out: &mut Vec<Violation>) {
    let query_str = r#"
        (call_expression
            function: (scoped_identifier
                path: (identifier) @type
                name: (identifier) @method
                (#match? @type "^(Vec|String)$")
                (#eq? @method "new"))) @alloc
    "#;
    
    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else { return; };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, body, source.as_bytes()) {
        if let Some(cap) = m.captures.last() { // @alloc
            let row = cap.node.start_position().row + 1;
            // Get type name for message
            let type_name = m.captures.iter()
                .find(|c| c.index == 0) // @type
                .and_then(|c| c.node.utf8_text(source.as_bytes()).ok())
                .unwrap_or("Collection");

            out.push(Violation::with_details(
                row,
                format!("Allocation of `{type_name}::new()` inside loop"),
                "P02",
                ViolationDetails {
                    function_name: None,
                    analysis: vec!["Repeated allocation in a loop causes memory pressure.".into()],
                    suggestion: Some("Move allocation outside loop or use `with_capacity` if needed.".into()),
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
                    suggestion: Some("Consider flattening, using a lookup map (O(1)), or iterator chains.".into()),
                }
            ));
        }
    }
}

/// P06: Linear search (`.contains`) in loop
fn check_p06_linear(source: &str, body: Node, out: &mut Vec<Violation>) {
    // Matches .contains()
    let query_str = r#"(call_expression function: (field_expression field: (field_identifier) @method (#eq? @method "contains"))) @search"#;
    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else { return; };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, body, source.as_bytes()) {
        if let Some(cap) = m.captures.last() {
            let row = cap.node.start_position().row + 1;
            out.push(Violation::with_details(
                row,
                "Linear search `.contains()` inside loop".to_string(),
                "P06",
                ViolationDetails {
                    function_name: None,
                    analysis: vec![
                        "Searching a collection inside a loop is O(n*m).".into(),
                        "If the collection is large, this is a bottleneck.".into()
                    ],
                    suggestion: Some("Use a `HashSet` or `BTreeSet` for O(1) or O(log n) lookups.".into()),
                }
            ));
        }
    }
}