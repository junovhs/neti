// src/analysis/patterns/logic_l03.rs
//! L03: Unchecked index access (`[0]`, `.first().unwrap()`, etc.).

use crate::types::{Confidence, Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

use super::logic_helpers::{
    can_find_local_declaration, has_chunks_exact_context, has_explicit_guard,
};
use super::logic_proof::{extract_receiver, is_fixed_size_array_access};

#[cfg(test)]
#[path = "logic_l03_test.rs"]
mod tests;

pub(super) fn detect_l03(source: &str, root: Node, out: &mut Vec<Violation>) {
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
