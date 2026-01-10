// src/analysis/v2/patterns/idiomatic.rs
//! Idiomatic Rust patterns: I01, I02, I03, I04

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

/// Detects idiomatic violations in Rust code.
#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut violations = Vec::new();
    detect_i01(source, root, &mut violations);
    detect_i02(source, root, &mut violations);
    detect_i03(source, root, &mut violations);
    detect_i04(source, root, &mut violations);
    violations
}

/// I01: Manual `impl From` (suggest `thiserror` or `derive_more`)
fn detect_i01(source: &str, root: Node, out: &mut Vec<Violation>) {
    let query_str = r#"
        (impl_item
            trait: (generic_type
                type: (type_identifier) @trait
                (#eq? @trait "From"))
            body: (declaration_list
                (function_item
                    name: (identifier) @fn
                    (#eq? @fn "from")))) @impl
    "#;
    
    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else { return; };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(cap) = m.captures.iter().find(|c| c.index == 1) {
            let row = cap.node.start_position().row + 1;
            out.push(Violation::with_details(
                row,
                "Manual `impl From` detected".to_string(),
                "I01",
                ViolationDetails {
                    function_name: None,
                    analysis: vec!["Manual error conversion code is boilerplate.".into()],
                    suggestion: Some("Consider using `#[derive(thiserror::Error)]` or `derive_more`.".into()),
                }
            ));
        }
    }
}

/// I02: Match arms with identical bodies (duplication)
fn detect_i02(source: &str, root: Node, out: &mut Vec<Violation>) {
    let query_str = r"(match_expression body: (match_block) @block)";
    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else { return; };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        for cap in m.captures {
            check_match_arms(source, cap.node, out);
        }
    }
}

fn check_match_arms(source: &str, block_node: Node, out: &mut Vec<Violation>) {
    let mut cursor = block_node.walk();
    let mut seen_bodies: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for child in block_node.children(&mut cursor) {
        if child.kind() == "match_arm" {
            process_arm_body(source, child, &mut seen_bodies, out);
        }
    }
}

fn process_arm_body(
    source: &str,
    arm_node: Node,
    seen: &mut std::collections::HashMap<String, usize>,
    out: &mut Vec<Violation>
) {
    let Some(body_node) = arm_node.child_by_field_name("value") else { return; };
    
    // Skip small/trivial bodies to avoid noise (e.g. `None => {}`)
    if body_node.end_byte() - body_node.start_byte() < 5 { return; }

    let body_text = body_node.utf8_text(source.as_bytes()).unwrap_or("").trim().to_string();
    
    if let Some(&first_line) = seen.get(&body_text) {
        let row = arm_node.start_position().row + 1;
        out.push(Violation::with_details(
            row,
            "Identical match arm body detected".to_string(),
            "I02",
            ViolationDetails {
                function_name: None,
                analysis: vec![format!("This body duplicates the one at line {first_line}.")],
                suggestion: Some("Combine arms using `|` pattern: `PatA | PatB => ...`".into()),
            }
        ));
    } else {
        seen.insert(body_text, arm_node.start_position().row + 1);
    }
}

/// I03: `if let Some(x)` pattern (suggest `?` or `map`)
fn detect_i03(source: &str, root: Node, out: &mut Vec<Violation>) {
    let query_str = r#"
        (if_expression
            condition: (let_condition
                pattern: (tuple_struct_pattern
                    type: (identifier) @typ
                    (#eq? @typ "Some")))) @if_expr
    "#;
    
    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else { return; };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(cap) = m.captures.last() {
            let row = cap.node.start_position().row + 1;
            out.push(Violation::with_details(
                row,
                "Verbose `if let Some` detected".to_string(),
                "I03",
                ViolationDetails {
                    function_name: None,
                    analysis: vec!["Checking Option validity manually.".into()],
                    suggestion: Some("Use idiomatic `?` propagation, `.map()`, or `.unwrap_or()`.".into()),
                }
            ));
        }
    }
}

/// I04: Manual `impl Display` (suggest `derive_more` or `thiserror`)
fn detect_i04(source: &str, root: Node, out: &mut Vec<Violation>) {
    let query_str = r#"
        (impl_item
            trait: (type_identifier) @trait
            (#eq? @trait "Display")
            body: (declaration_list
                (function_item
                    body: (block
                        (expression_statement
                            (macro_invocation
                                macro: (identifier) @mac
                                (#match? @mac "^write"))))))) @impl
    "#;

    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else { return; };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(cap) = m.captures.last() {
            let row = cap.node.start_position().row + 1;
            out.push(Violation::with_details(
                row,
                "Manual `impl Display` detected".to_string(),
                "I04",
                ViolationDetails {
                    function_name: None,
                    analysis: vec!["Manual Display implementation using write!.".into()],
                    suggestion: Some("Consider `#[derive(derive_more::Display)]` or `thiserror`.".into()),
                }
            ));
        }
    }
}