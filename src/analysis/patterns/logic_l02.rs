// src/analysis/patterns/logic_l02.rs
//! L02: Boundary uses `<=`/`>=` with `.len()` — possible off-by-one.

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

use super::logic_helpers::{is_index_variable, is_literal};

pub(super) fn detect_l02(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r"(binary_expression) @cmp";
    let Ok(query) = Query::new(&tree_sitter_rust::LANGUAGE.into(), q) else {
        return;
    };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let Some(cmp) = m.captures.first().map(|c| c.node) else {
            continue;
        };

        let text = cmp.utf8_text(source.as_bytes()).unwrap_or("");
        if !text.contains(".len()") {
            continue;
        }
        if !text.contains("<=") && !text.contains(">=") {
            continue;
        }
        if is_safe_boundary(cmp, source) {
            continue;
        }

        out.push(Violation::with_details(
            cmp.start_position().row + 1,
            "Boundary uses `<=`/`>=` with `.len()` — possible off-by-one".into(),
            "L02",
            ViolationDetails {
                function_name: None,
                analysis: vec![
                    "Indices are 0..len-1. Comparing with `<= len` can reach `len`.".into(),
                ],
                suggestion: Some("Use `< len` for index upper bounds.".into()),
            },
        ));
    }
}

fn is_safe_boundary(node: Node, source: &str) -> bool {
    let left = node.child_by_field_name("left");
    let right = node.child_by_field_name("right");

    if is_literal(left) || is_literal(right) {
        return true;
    }

    let full_text = node.utf8_text(source.as_bytes()).unwrap_or("");
    let op = extract_op(full_text);

    let left_text = left
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("");
    let right_text = right
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("");

    if right_text.contains(".len()") {
        if op == ">=" {
            return true;
        }
        return !is_index_variable(left_text);
    }

    if left_text.contains(".len()") {
        if op == "<=" {
            return true;
        }
        return !is_index_variable(right_text);
    }

    true
}

fn extract_op(full_text: &str) -> &str {
    if full_text.contains("<=") {
        "<="
    } else if full_text.contains(">=") {
        ">="
    } else {
        ""
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::types::Violation;
    use tree_sitter::Parser;

    fn parse_and_detect(code: &str) -> Vec<Violation> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(code, None).unwrap();
        super::super::detect(code, tree.root_node())
    }

    #[test]
    fn l02_flag_lte_len() {
        let code = "fn f(v: &[i32], i: usize) -> bool { i <= v.len() }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "L02"));
    }

    #[test]
    fn l02_flag_len_gte_idx() {
        let code = "fn f(v: &[i32], i: usize) -> bool { v.len() >= i }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "L02"));
    }

    #[test]
    fn l02_skip_threshold() {
        let code = "fn f(v: &[i32]) -> bool { v.len() >= 5 }";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "L02"));
    }

    #[test]
    fn l02_skip_max_var() {
        let code = "fn f(v: &[i32], max: usize) -> bool { v.len() <= max }";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "L02"));
    }

    #[test]
    fn l02_skip_guard_idx_gte_len() {
        let code = "fn f(v: &[i32], idx: usize) -> Option<i32> { if idx >= v.len() { return None; } Some(v[0]) }";
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "L02"),
            "idx >= v.len() is a canonical guard and must not be flagged"
        );
    }

    #[test]
    fn l02_skip_guard_len_lte_idx() {
        let code = "fn f(v: &[i32], idx: usize) -> Option<i32> { if v.len() <= idx { return None; } Some(v[0]) }";
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "L02"),
            "v.len() <= idx is a guard and must not be flagged"
        );
    }
}
