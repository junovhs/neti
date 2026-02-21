// src/analysis/patterns/logic.rs
//! Logic boundary patterns: L02 (off-by-one risk), L03 (unchecked index).
//!
//! # L02 Design
//!
//! L02 flags `<=`/`>=` comparisons with `.len()` that risk an off-by-one
//! panic. Crucially, it must NOT flag canonical bounds guards:
//!
//!   ```rust
//!   if idx >= v.len() { return None; } // safe guard — do NOT flag
//!   ```
//!
//! Only the dangerous direction (where idx could reach len as an array index)
//! should produce a violation:
//!
//!   ```rust
//!   if i <= v.len() { process(v[i]); } // idx could equal len — DO flag
//!   ```
//!
//! # L03 Design
//!
//! L03 flags `[0]` and `.first().unwrap()` without a bounds proof. It must
//! NOT flag indexing that is provably safe due to iterator invariants:
//!
//!   ```rust
//!   slice.chunks_exact(2).map(|a| u16::from_le_bytes([a[0], a[1]]));
//!   //                                                  ^^^^  safe
//!   ```

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut out = Vec::new();
    detect_l02(source, root, &mut out);
    detect_l03(source, root, &mut out);
    out
}

// ── L02 ─────────────────────────────────────────────────────────────────────

/// L02: Boundary ambiguity — `<=` or `>=` with `.len()` in the dangerous direction.
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

/// Returns `true` if this boundary comparison is provably safe (should not flag).
///
/// Safe cases:
/// - One side is a literal (threshold check, e.g. `v.len() >= 5`)
/// - `idx >= v.len()` — canonical early-return guard (idx is OUT of bounds)
/// - `v.len() <= idx` — same guard, reversed
///
/// Flaggable cases:
/// - `idx <= v.len()` — idx could equal len (off-by-one risk)
/// - `v.len() >= idx` — same risk, reversed
fn is_safe_boundary(node: Node, source: &str) -> bool {
    let left = node.child_by_field_name("left");
    let right = node.child_by_field_name("right");

    // Literal on either side → threshold check, always safe
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
        // Pattern: X OP v.len()
        // `>=`: x >= v.len() → guard ("x is out of bounds") — SAFE
        // `<=`: x <= v.len() → x could equal len — DANGEROUS
        if op == ">=" {
            return true;
        }
        return !is_index_variable(left_text);
    }

    if left_text.contains(".len()") {
        // Pattern: v.len() OP X
        // `<=`: v.len() <= x → same as x >= v.len() → guard — SAFE
        // `>=`: v.len() >= x → same as x <= v.len() → DANGEROUS
        if op == "<=" {
            return true;
        }
        return !is_index_variable(right_text);
    }

    // Ambiguous — err on the side of silence
    true
}

fn extract_op(full_text: &str) -> &str {
    // Order matters: check `<=` before `<`, `>=` before `>`
    if full_text.contains("<=") {
        "<="
    } else if full_text.contains(">=") {
        ">="
    } else {
        ""
    }
}

fn is_literal(node: Option<Node>) -> bool {
    node.is_some_and(|n| n.kind() == "integer_literal" || n.kind() == "float_literal")
}

fn is_index_variable(name: &str) -> bool {
    let n = name.trim();
    // Common short index names
    n == "i"
        || n == "j"
        || n == "k"
        || n == "n"
        || n == "idx"
        || n.contains("index")
        || n.contains("pos")
        || n.contains("ptr")
        || n.contains("offset")
        || n.contains("cursor")
}

// ── L03 ─────────────────────────────────────────────────────────────────────

/// L03: Unchecked `[0]` or `.first()/.last().unwrap()`.
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

        // Explicit guard: caller already checks .is_empty() or .len()
        if has_explicit_guard(source, idx_node) {
            continue;
        }

        // Proof by iterator invariant: inside chunks_exact / array_chunks
        if has_chunks_exact_context(source, idx_node) {
            continue;
        }

        out.push(Violation::with_details(
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
        ));
    }
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
        if !text.contains(".first()") && !text.contains(".last()") {
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

/// Returns `true` if a `.len()` / `.is_empty()` guard is visible in the
/// ancestor chain — meaning the caller already proved non-emptiness.
fn has_explicit_guard(source: &str, node: Node) -> bool {
    let mut cur = node;
    for _ in 0..10 {
        let Some(p) = cur.parent() else { break };
        let text = p.utf8_text(source.as_bytes()).unwrap_or("");
        if text.contains(".len()") || text.contains(".is_empty()") {
            return true;
        }
        if p.kind() == "if_expression" && text.contains('!') && text.contains("is_empty") {
            return true;
        }
        cur = p;
    }
    false
}

/// Returns `true` if the node is inside a `chunks_exact(N)` or `array_chunks()`
/// iterator — meaning the chunk length is guaranteed by the iterator contract.
///
/// Example (safe — do NOT flag):
/// ```rust
/// data.chunks_exact(2).map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]));
/// ```
fn has_chunks_exact_context(source: &str, node: Node) -> bool {
    let mut cur = node;
    for _ in 0..25 {
        let Some(p) = cur.parent() else { break };
        let text = p.utf8_text(source.as_bytes()).unwrap_or("");
        if text.contains("chunks_exact(") || text.contains("array_chunks(") {
            return true;
        }
        // Stop at source-file boundary
        if p.kind() == "source_file" {
            break;
        }
        cur = p;
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

    // ── L02 ──────────────────────────────────────────────────────────────

    #[test]
    fn l02_flag_lte_len() {
        // idx could equal len — off-by-one risk
        let code = "fn f(v: &[i32], i: usize) -> bool { i <= v.len() }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "L02"));
    }

    #[test]
    fn l02_flag_len_gte_idx() {
        // same risk, reversed: v.len() >= i means i could equal len
        let code = "fn f(v: &[i32], i: usize) -> bool { v.len() >= i }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "L02"));
    }

    #[test]
    fn l02_skip_threshold() {
        // Literal comparison — threshold check, never an indexing risk
        let code = "fn f(v: &[i32]) -> bool { v.len() >= 5 }";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "L02"));
    }

    #[test]
    fn l02_skip_max_var() {
        // Non-index variable on RHS — threshold-style check
        let code = "fn f(v: &[i32], max: usize) -> bool { v.len() <= max }";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "L02"));
    }

    #[test]
    fn l02_skip_guard_idx_gte_len() {
        // Canonical early-return bounds guard — must NOT be flagged
        let code = "fn f(v: &[i32], idx: usize) -> Option<i32> { if idx >= v.len() { return None; } Some(v[0]) }";
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "L02"),
            "idx >= v.len() is a canonical guard and must not be flagged"
        );
    }

    #[test]
    fn l02_skip_guard_len_lte_idx() {
        // Same guard, reversed: v.len() <= idx — also safe
        let code = "fn f(v: &[i32], idx: usize) -> Option<i32> { if v.len() <= idx { return None; } Some(v[0]) }";
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "L02"),
            "v.len() <= idx is a guard and must not be flagged"
        );
    }

    // ── L03 ──────────────────────────────────────────────────────────────

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

    #[test]
    fn l03_skip_chunks_exact_index() {
        // Safe: chunk length is guaranteed by chunks_exact(2)
        let code = r"
            fn f(data: &[u8]) -> Vec<u16> {
                data.chunks_exact(2)
                    .map(|a| u16::from_le_bytes([a[0], a[1]]))
                    .collect()
            }
        ";
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "L03"),
            "chunks_exact indexing is provably safe and must not be flagged"
        );
    }
}
