//! Logic boundary patterns: L02 (off-by-one risk), L03 (unchecked index).
//!
//! See `logic_helpers` for shared utilities and `logic_proof` for
//! fixed-size array proof logic.

use crate::types::{Confidence, Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

use super::logic_helpers::{
    can_find_local_declaration, has_chunks_exact_context, has_explicit_guard, is_index_variable,
    is_literal,
};
use super::logic_proof::{extract_receiver, is_fixed_size_array_access};

#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut out = Vec::new();
    detect_l02(source, root, &mut out);
    detect_l03(source, root, &mut out);
    out
}

// ── L02 ─────────────────────────────────────────────────────────────────────

fn detect_l02(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r"(binary_expression) @cmp";
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else {
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

// ── L03 ─────────────────────────────────────────────────────────────────────

fn detect_l03(source: &str, root: Node, out: &mut Vec<Violation>) {
    detect_index_zero(source, root, out);
    detect_first_last_unwrap(source, root, out);
}

fn detect_index_zero(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r"(index_expression) @idx";
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else {
        return;
    };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let Some(idx_node) = m.captures.first().map(|c| c.node) else {
            continue;
        };

        let text = idx_node.utf8_text(source.as_bytes()).unwrap_or("");
        if !text.ends_with("[0]") {
            continue;
        }
        if has_explicit_guard(source, idx_node) {
            continue;
        }
        if has_chunks_exact_context(source, idx_node) {
            continue;
        }
        if is_fixed_size_array_access(source, idx_node, root) {
            continue;
        }

        let receiver = extract_receiver(text);
        let (confidence, reason) = classify_l03_confidence(source, idx_node, receiver);

        let mut v = Violation::with_details(
            idx_node.start_position().row + 1,
            "Index `[0]` without bounds check".into(),
            "L03",
            ViolationDetails {
                function_name: None,
                analysis: vec!["Panics on empty collection.".into()],
                suggestion: Some(
                    "Use `.first()` and handle `None`, or check `.is_empty()` first.".into(),
                ),
            },
        );
        v.confidence = confidence;
        v.confidence_reason = reason;
        out.push(v);
    }
}

/// Determine L03 confidence by distinguishing three epistemic states:
///
/// 1. Receiver is `self.field` or contains a method call → Medium (cross-scope)
/// 2. Receiver is a simple local variable and we found its declaration → High
/// 3. Receiver is a simple local variable but we can't find a declaration → Medium
fn classify_l03_confidence(
    source: &str,
    node: Node,
    receiver: &str,
) -> (Confidence, Option<String>) {
    if receiver.contains("self.") || receiver.contains('(') {
        return (
            Confidence::Medium,
            Some("cannot trace type through field access or method return".to_string()),
        );
    }

    if !receiver.contains('.') {
        if can_find_local_declaration(source, node, receiver) {
            return (Confidence::High, None);
        }
        return (
            Confidence::Medium,
            Some("cannot find variable declaration to verify type".to_string()),
        );
    }

    (
        Confidence::Medium,
        Some("cannot trace type through field access".to_string()),
    )
}

fn detect_first_last_unwrap(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r"(call_expression) @call";
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else {
        return;
    };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let Some(call) = m.captures.first().map(|c| c.node) else {
            continue;
        };

        let text = call.utf8_text(source.as_bytes()).unwrap_or("");
        let has_first_or_last = text.contains(".first()") || text.contains(".last()");
        if !has_first_or_last {
            continue;
        }
        if !text.contains(".unwrap()") {
            continue;
        }
        if has_explicit_guard(source, call) {
            continue;
        }

        out.push(Violation::with_details(
            call.start_position().row + 1,
            "`.first()/.last().unwrap()` without guard".into(),
            "L03",
            ViolationDetails {
                function_name: None,
                analysis: vec!["Panics on empty collection.".into()],
                suggestion: Some("Use `?` or check `.is_empty()`.".into()),
            },
        ));
    }
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

    // ── L02 ──────────────────────────────────────────────────────────────

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

    // ── L03 basics ───────────────────────────────────────────────────────

    #[test]
    fn l03_flag_index_zero() {
        let code = "fn f(v: &[i32]) -> i32 { v[0] }";
        let violations: Vec<_> = parse_and_detect(code)
            .into_iter()
            .filter(|v| v.law == "L03")
            .collect();
        assert!(!violations.is_empty());
        assert_eq!(violations[0].confidence, Confidence::High);
    }

    #[test]
    fn l03_skip_with_empty_check() {
        let code = "fn f(v: &[i32]) -> i32 { if !v.is_empty() { v[0] } else { 0 } }";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "L03"));
    }

    #[test]
    fn l03_skip_with_len_check() {
        let code = "fn f(v: &[i32]) -> i32 { if v.len() > 0 { v[0] } else { 0 } }";
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "L03"),
            "v.len() guard must suppress L03"
        );
    }

    #[test]
    fn l03_flag_first_unwrap() {
        let code = "fn f(v: &[i32]) -> i32 { *v.first().unwrap() }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "L03"));
    }

    #[test]
    fn l03_flag_last_unwrap() {
        let code = "fn f(v: &[i32]) -> i32 { *v.last().unwrap() }";
        assert!(
            parse_and_detect(code).iter().any(|v| v.law == "L03"),
            ".last().unwrap() must be flagged"
        );
    }

    #[test]
    fn l03_no_flag_without_unwrap() {
        let code = "fn f(v: &[i32]) -> Option<&i32> { v.first() }";
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "L03"),
            ".first() without .unwrap() must not be flagged"
        );
    }

    #[test]
    fn l03_skip_chunks_exact_index() {
        let code = r"
            fn f(data: &[u8]) -> Vec<u16> {
                data.chunks_exact(2)
                    .map(|a| u16::from_le_bytes([a[0], a[1]]))
                    .collect()
            }
        ";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "L03"));
    }

    // ── L03 fixed-size array ─────────────────────────────────────────────

    #[test]
    fn l03_skip_fixed_array_repeat() {
        let code = r"
            fn f() {
                let mut seed = [0u8; 32];
                seed[0] = 1;
            }
        ";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "L03"));
    }

    #[test]
    fn l03_skip_fixed_array_literal() {
        let code = r"
            fn f() -> i32 {
                let arr = [1, 2, 3];
                arr[0]
            }
        ";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "L03"));
    }

    #[test]
    fn l03_skip_struct_field_array() {
        let code = r"
            struct Rng {
                s: [u32; 4],
            }
            impl Rng {
                fn next(&mut self) -> u32 {
                    let res = self.s[0].wrapping_add(self.s[3]);
                    res
                }
            }
        ";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "L03"));
    }

    #[test]
    fn l03_skip_typed_param_array() {
        let code = r"
            fn process(buf: [u8; 4]) -> u8 {
                buf[0]
            }
        ";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "L03"));
    }

    #[test]
    fn l03_still_flags_vec_index() {
        let code = r"
            fn f(v: Vec<i32>) -> i32 {
                v[0]
            }
        ";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "L03"));
    }

    #[test]
    fn l03_still_flags_slice_index() {
        let code = "fn f(v: &[i32]) -> i32 { v[0] }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "L03"));
    }

    // ── L03 confidence tiers ─────────────────────────────────────────────

    #[test]
    fn l03_medium_confidence_for_unfound_variable() {
        let code = r"
            fn f() -> i32 {
                data[0]
            }
        ";
        let violations: Vec<_> = parse_and_detect(code)
            .into_iter()
            .filter(|v| v.law == "L03")
            .collect();
        assert!(!violations.is_empty(), "should flag data[0]");
        assert_eq!(violations[0].confidence, Confidence::Medium);
    }

    #[test]
    fn l03_high_confidence_for_found_vec() {
        let code = r"
            fn f() -> i32 {
                let v = vec![1, 2, 3];
                v[0]
            }
        ";
        let violations: Vec<_> = parse_and_detect(code)
            .into_iter()
            .filter(|v| v.law == "L03")
            .collect();
        assert!(!violations.is_empty());
        assert_eq!(violations[0].confidence, Confidence::High);
    }

    #[test]
    fn l03_medium_confidence_for_self_field() {
        let code = r"
            struct Foo { items: Vec<i32> }
            impl Foo {
                fn first(&self) -> i32 {
                    self.items[0]
                }
            }
        ";
        let violations: Vec<_> = parse_and_detect(code)
            .into_iter()
            .filter(|v| v.law == "L03")
            .collect();
        assert!(!violations.is_empty());
        assert_eq!(violations[0].confidence, Confidence::Medium);
    }

    #[test]
    fn l03_high_confidence_for_param_slice() {
        let code = "fn f(v: &[i32]) -> i32 { v[0] }";
        let violations: Vec<_> = parse_and_detect(code)
            .into_iter()
            .filter(|v| v.law == "L03")
            .collect();
        assert!(!violations.is_empty());
        assert_eq!(violations[0].confidence, Confidence::High);
    }

    #[test]
    fn l03_medium_confidence_for_method_return() {
        let code = "fn f() -> i32 { get_data()[0] }";
        let violations: Vec<_> = parse_and_detect(code)
            .into_iter()
            .filter(|v| v.law == "L03")
            .collect();
        assert!(!violations.is_empty(), "should flag method return indexing");
        assert_eq!(
            violations[0].confidence,
            Confidence::Medium,
            "method return receiver should be Medium"
        );
    }
}
