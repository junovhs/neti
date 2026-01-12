// src/analysis/v2/patterns/logic.rs
//! Logic patterns: L02, L03

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut out = Vec::new();
    detect_l02(source, root, &mut out);
    detect_l03(source, root, &mut out);
    out
}

/// L02: Boundary ambiguity - `<=` or `>=` with `.len()`
fn detect_l02(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r"(binary_expression) @cmp";
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else { return };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let cmp = m.captures.first().map(|c| c.node);
        let Some(cmp) = cmp else { continue };

        let text = cmp.utf8_text(source.as_bytes()).unwrap_or("");
        if !text.contains(".len()") { continue }
        if !text.contains("<=") && !text.contains(">=") { continue }

        out.push(Violation::with_details(
            cmp.start_position().row + 1,
            "Boundary uses `<=`/`>=` with `.len()`".into(),
            "L02",
            ViolationDetails {
                function_name: None,
                analysis: vec!["May cause off-by-one. Indices are 0..len-1.".into()],
                suggestion: Some("Usually want `< len` not `<= len`.".into()),
            }
        ));
    }
}

/// L03: Unchecked `[0]` or `.first().unwrap()`
fn detect_l03(source: &str, root: Node, out: &mut Vec<Violation>) {
    detect_index_zero(source, root, out);
    detect_first_unwrap(source, root, out);
}

fn detect_index_zero(source: &str, root: Node, out: &mut Vec<Violation>) {
    // Walk all index_expression nodes
    let q = r"(index_expression) @idx";
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else { return };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let idx_node = m.captures.first().map(|c| c.node);
        let Some(idx_node) = idx_node else { continue };

        let text = idx_node.utf8_text(source.as_bytes()).unwrap_or("");
        // Check if indexing with literal 0
        if !text.ends_with("[0]") { continue }
        if has_guard(source, idx_node) { continue }

        out.push(Violation::with_details(
            idx_node.start_position().row + 1,
            "Index `[0]` without bounds check".into(),
            "L03",
            ViolationDetails {
                function_name: None,
                analysis: vec!["Panics on empty collection.".into()],
                suggestion: Some("Use `.first()` or check `.is_empty()`.".into()),
            }
        ));
    }
}

fn detect_first_unwrap(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r"(call_expression) @call";
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else { return };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let call = m.captures.first().map(|c| c.node);
        let Some(call) = call else { continue };

        let text = call.utf8_text(source.as_bytes()).unwrap_or("");
        if !text.contains(".first()") && !text.contains(".last()") { continue }
        if !text.contains(".unwrap()") { continue }
        if has_guard(source, call) { continue }

        out.push(Violation::with_details(
            call.start_position().row + 1,
            "`.first()/.last().unwrap()` without guard".into(),
            "L03",
            ViolationDetails {
                function_name: None,
                analysis: vec!["Panics on empty collection.".into()],
                suggestion: Some("Use `?` or check `.is_empty()`.".into()),
            }
        ));
    }
}

fn has_guard(source: &str, node: Node) -> bool {
    let mut cur = node;
    for _ in 0..10 {
        if let Some(p) = cur.parent() {
            let text = p.utf8_text(source.as_bytes()).unwrap_or("");
            if text.contains(".len()") || text.contains(".is_empty()") { return true }
            if p.kind() == "if_expression" && text.contains('!') && text.contains("is_empty") {
                return true;
            }
            cur = p;
        } else { break }
    }
    false
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
    fn l02_flag_lte_len() {
        let code = "fn f(v: &[i32], i: usize) -> bool { i <= v.len() }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "L02"));
    }

    #[test]
    fn l03_flag_index_zero() {
        let code = "fn f(v: &[i32]) -> i32 { v[0] }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "L03"));
    }

    #[test]
    fn l03_skip_with_empty_check() {
        let code = "fn f(v: &[i32]) -> i32 { if !v.is_empty() { v[0] } else { 0 } }";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "L03"));
    }

    #[test]
    fn l03_flag_first_unwrap() {
        let code = "fn f(v: &[i32]) -> i32 { *v.first().unwrap() }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "L03"));
    }
}
