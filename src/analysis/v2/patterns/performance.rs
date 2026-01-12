// src/analysis/v2/patterns/performance.rs
//! Performance anti-patterns: P01, P02, P04, P06

use crate::types::{Violation, ViolationDetails};
use std::path::Path;
use tree_sitter::{Node, Query, QueryCursor};
use super::get_capture_node;

#[must_use]
pub fn detect(source: &str, root: Node, path: &Path) -> Vec<Violation> {
    if should_skip(path) { return Vec::new(); }
    let mut out = Vec::new();
    detect_loops(source, root, &mut out);
    out
}

fn should_skip(path: &Path) -> bool {
    let s = path.to_string_lossy();
    s.contains("/cli/") || s.contains("/ui/") || s.contains("/tui/")
        || s.contains("reporting") || s.contains("messages")
        || s.contains("analysis/") || s.contains("audit/")
        || s.contains("pack/") || s.contains("signatures/")
        || s.ends_with("main.rs")
}

fn detect_loops(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r"
        (for_expression pattern: (_) @pat body: (block) @body) @loop
        (while_expression body: (block) @body) @loop
        (loop_expression body: (block) @body) @loop
    ";
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else { return };
    let idx_pat = query.capture_index_for_name("pat");
    let idx_body = query.capture_index_for_name("body");
    
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let loop_var = get_capture_node(&m, idx_pat)
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .map(|s| s.split([',', '(']).next().unwrap_or(s).trim().to_string());

        let Some(body) = get_capture_node(&m, idx_body) else { continue };

        check_p01(source, body, loop_var.as_deref(), out);
        check_p02(source, body, loop_var.as_deref(), out);
        check_p04(body, out);
        check_p06(source, body, out);
    }
}

fn check_p01(source: &str, body: Node, loop_var: Option<&str>, out: &mut Vec<Violation>) {
    let q = r#"(call_expression function: (field_expression
        value: (_) @recv field: (field_identifier) @m (#eq? @m "clone"))) @call"#;
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else { return };
    let idx_call = query.capture_index_for_name("call");
    let idx_recv = query.capture_index_for_name("recv");
    
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, body, source.as_bytes()) {
        let call = get_capture_node(&m, idx_call);
        let recv = get_capture_node(&m, idx_recv)
            .and_then(|c| c.utf8_text(source.as_bytes()).ok());

        let Some(call) = call else { continue };
        if should_skip_clone(source, call, recv, loop_var) { continue }

        out.push(Violation::with_details(
            call.start_position().row + 1,
            "Detected `.clone()` inside a loop".into(),
            "P01",
            ViolationDetails {
                function_name: None,
                analysis: vec!["Clone appears hoistable.".into()],
                suggestion: Some("Hoist clone or use Arc.".into()),
            }
        ));
    }
}

fn should_skip_clone(source: &str, call: Node, recv: Option<&str>, lv: Option<&str>) -> bool {
    if is_ownership_sink(source, call) { return true }
    if let (Some(r), Some(v)) = (recv, lv) {
        if r.trim() == v || r.contains(&format!("{v}.")) { return true }
    }
    if let Some(r) = recv {
        if r.contains("..") || r.parse::<i64>().is_ok() { return true }
    }
    false
}

fn is_ownership_sink(source: &str, node: Node) -> bool {
    let mut cur = node;
    for _ in 0..10 {
        if let Some(p) = cur.parent() {
            let txt = p.utf8_text(source.as_bytes()).unwrap_or("");
            if txt.contains(".insert(") || txt.contains(".push(")
                || txt.contains(".entry(") || txt.contains(".extend(") {
                return true;
            }
            cur = p;
        } else { break }
    }
    false
}

fn check_p02(source: &str, body: Node, loop_var: Option<&str>, out: &mut Vec<Violation>) {
    let q = r#"(call_expression function: (field_expression
        value: (_) @recv field: (field_identifier) @m)
        (#match? @m "^(to_string|to_owned)$")) @call"#;
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else { return };
    let idx_call = query.capture_index_for_name("call");
    let idx_recv = query.capture_index_for_name("recv");
    
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, body, source.as_bytes()) {
        let call = get_capture_node(&m, idx_call);
        let recv = get_capture_node(&m, idx_recv)
            .and_then(|c| c.utf8_text(source.as_bytes()).ok());

        let Some(call) = call else { continue };
        if is_ownership_sink(source, call) { continue }
        if let (Some(r), Some(v)) = (recv, loop_var) {
            if r.trim() == v || r.contains(&format!("{v}.")) { continue }
        }

        out.push(Violation::with_details(
            call.start_position().row + 1,
            "String conversion inside loop".into(),
            "P02",
            ViolationDetails {
                function_name: None,
                analysis: vec!["Allocation appears hoistable.".into()],
                suggestion: Some("Hoist allocation.".into()),
            }
        ));
    }
}

fn check_p04(body: Node, out: &mut Vec<Violation>) {
    let mut cursor = body.walk();
    for child in body.children(&mut cursor) {
        if matches!(child.kind(), "for_expression" | "while_expression" | "loop_expression") {
            out.push(Violation::with_details(
                child.start_position().row + 1,
                "Nested loop (O(nA�))".into(),
                "P04",
                ViolationDetails {
                    function_name: None,
                    analysis: vec!["Quadratic complexity.".into()],
                    suggestion: Some("Use a lookup map.".into()),
                }
            ));
        }
    }
}

fn check_p06(source: &str, body: Node, out: &mut Vec<Violation>) {
    let q = r#"(call_expression function: (field_expression
        field: (field_identifier) @m) (#match? @m "^(find|position)$")) @call"#;
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else { return };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, body, source.as_bytes()) {
        if let Some(cap) = m.captures.first() {
            out.push(Violation::with_details(
                cap.node.start_position().row + 1,
                "Linear search in loop".into(),
                "P06",
                ViolationDetails {
                    function_name: None,
                    analysis: vec!["O(n*m) complexity.".into()],
                    suggestion: Some("Use HashSet.".into()),
                }
            ));
        }
    }
}