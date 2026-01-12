// src/analysis/v2/patterns/idiomatic.rs
//! Idiomatic patterns: I01, I02

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};
use super::get_capture_node;

#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut out = Vec::new();
    detect_i01(source, root, &mut out);
    detect_i02(source, root, &mut out);
    out
}

/// I01: Manual From impl that could use derive
fn detect_i01(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r"(impl_item) @impl";
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else { return };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let Some(impl_node) = m.captures.first().map(|c| c.node) else { continue };

        let text = impl_node.utf8_text(source.as_bytes()).unwrap_or("");
        if !text.contains("impl From<") || !text.contains("for ") { continue }
        if text.contains("Error") { continue }
        if text.contains("if ") || text.contains("match ") { continue }
        if text.matches(';').count() > 2 { continue }

        out.push(Violation::with_details(
            impl_node.start_position().row + 1,
            "Manual `From` impl".into(),
            "I01",
            ViolationDetails {
                function_name: None,
                analysis: vec!["Consider `#[derive(From)]` from derive_more.".into()],
                suggestion: Some("Use derive_more::From if applicable.".into()),
            }
        ));
    }
}

/// I02: Match arms with duplicate bodies
fn detect_i02(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r"(match_expression body: (match_block) @block) @match";
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else { return };
    let idx_match = query.capture_index_for_name("match");
    let idx_block = query.capture_index_for_name("block");
    
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let match_node = get_capture_node(&m, idx_match);
        let block = get_capture_node(&m, idx_block);

        let (Some(match_node), Some(block)) = (match_node, block) else { continue };

        if let Some(dup) = find_dup_arms(source, block) {
            out.push(Violation::with_details(
                match_node.start_position().row + 1,
                "Duplicate match arm bodies".into(),
                "I02",
                ViolationDetails {
                    function_name: None,
                    analysis: vec![format!("Duplicate: `{}`", truncate(&dup, 30))],
                    suggestion: Some("Combine: `A | B => body`".into()),
                }
            ));
        }
    }
}

fn find_dup_arms(source: &str, block: Node) -> Option<String> {
    let mut bodies: Vec<String> = Vec::new();
    let mut cursor = block.walk();

    for child in block.children(&mut cursor) {
        if child.kind() != "match_arm" { continue }
        if let Some(body) = child.child_by_field_name("value") {
            let text = body.utf8_text(source.as_bytes()).unwrap_or("");
            let norm = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if norm.len() < 5 { continue }
            // P06: Linear search here is unavoidable as `bodies` is small (match arms)
            // and we need value equality check.
            if bodies.contains(&norm) { return Some(norm) }
            bodies.push(norm);
        }
    }
    None
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() } else { format!("{}...", &s[..max]) }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_and_detect(code: &str) -> Vec<Violation> {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_rust::language()).unwrap();
        let tree = parser.parse(code, None).unwrap();
        detect(code, tree.root_node())
    }

    #[test]
    fn i01_flag_simple_from() {
        let code = "impl From<String> for MyType { fn from(s: String) -> Self { Self { v: s } } }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "I01"));
    }

    #[test]
    fn i01_skip_error_from() {
        let code = "impl From<io::Error> for MyError { fn from(e: io::Error) -> Self { Self(e) } }";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "I01"));
    }

    #[test]
    fn i02_flag_duplicate_arms() {
        let code = "fn f(x: i32) { match x { 1 => do_thing(), 2 => do_thing(), _ => other() } }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "I02"));
    }

    #[test]
    fn i02_skip_unique_arms() {
        let code = "fn f(x: i32) { match x { 1 => one(), 2 => two(), _ => other() } }";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "I02"));
    }
}