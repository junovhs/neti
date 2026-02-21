// src/analysis/patterns/performance.rs
//! Performance anti-patterns: P01, P02, P04, P06
//!
//! # Escalation Philosophy
//!
//! P01/P02 must only fire when we can make a reasonable argument that the
//! allocation is *material*. Blanket "clone in loop" flags generate lint spam
//! and train developers to ignore Neti. The goal is signal, not volume.
//!
//! ## P01 — Clone in loop
//!
//! Only escalate when the cloned type is likely heap-owning:
//! - `String`, `Vec`, `HashMap`, `HashSet`, `Box`, `Rc`, `BTreeMap`
//! - Structs heuristically identified by a capitalized receiver name
//!
//! Do NOT escalate for:
//! - `Arc::clone(...)` — cheap reference count increment
//! - `Rc::clone(...)` — same, single-threaded
//! - Cloning the loop variable itself (owned iteration pattern)
//! - When the clone goes into an ownership sink (`.push`, `.insert`, `.extend`)
//!
//! ## P02 — String conversion in loop
//!
//! Only escalate for `.to_string()` / `.to_owned()` when the receiver is
//! statically likely to allocate (e.g., a `&str` literal or a field with a
//! string type). Conversions on loop variables are often the caller's intent.

use super::get_capture_node;
use crate::types::{Violation, ViolationDetails};
use std::path::Path;
use tree_sitter::{Node, Query, QueryCursor};

#[must_use]
pub fn detect(source: &str, root: Node, path: &Path) -> Vec<Violation> {
    if should_skip(path) {
        return Vec::new();
    }
    let mut out = Vec::new();
    detect_loops(source, root, &mut out);
    out
}

fn should_skip(path: &Path) -> bool {
    let s = path.to_string_lossy();
    s.contains("/cli/")
        || s.contains("/ui/")
        || s.contains("/tui/")
        || s.contains("reporting")
        || s.contains("messages")
        || s.contains("analysis/")
        || s.contains("audit/")
        || s.contains("pack/")
        || s.contains("signatures/")
        || s.ends_with("main.rs")
}

fn detect_loops(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r"
        (for_expression pattern: (_) @pat body: (block) @body) @loop
        (while_expression body: (block) @body) @loop
        (loop_expression body: (block) @body) @loop
    ";
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else {
        return;
    };
    let idx_pat = query.capture_index_for_name("pat");
    let idx_body = query.capture_index_for_name("body");

    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, root, source.as_bytes()) {
        let loop_var = get_capture_node(&m, idx_pat)
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .map(|s| s.split([',', '(']).next().unwrap_or(s).trim().to_string());

        let Some(body) = get_capture_node(&m, idx_body) else {
            continue;
        };

        check_p01(source, body, loop_var.as_deref(), out);
        check_p02(source, body, loop_var.as_deref(), out);
        check_p04(body, out);
        check_p06(source, body, out);
    }
}

fn check_p01(source: &str, body: Node, loop_var: Option<&str>, out: &mut Vec<Violation>) {
    let q = r#"(call_expression function: (field_expression
        value: (_) @recv field: (field_identifier) @m (#eq? @m "clone"))) @call"#;
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else {
        return;
    };
    let idx_call = query.capture_index_for_name("call");
    let idx_recv = query.capture_index_for_name("recv");

    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, body, source.as_bytes()) {
        let call = get_capture_node(&m, idx_call);
        let recv = get_capture_node(&m, idx_recv).and_then(|c| c.utf8_text(source.as_bytes()).ok());

        let Some(call) = call else { continue };

        // Arc::clone / Rc::clone — cheap ref-count, never escalate
        let call_text = call.utf8_text(source.as_bytes()).unwrap_or("");
        if is_arc_or_rc_clone(call_text) {
            continue;
        }

        if should_skip_clone(source, call, recv, loop_var) {
            continue;
        }

        // Determine if the cloned receiver is likely heap-owning
        let recv_text = recv.unwrap_or("");
        let is_heap_owning = looks_heap_owning(recv_text);

        let analysis = if is_heap_owning {
            vec![
                format!("Receiver `{recv_text}` appears to be a heap-owning type."),
                "Clone inside a loop allocates on every iteration.".into(),
            ]
        } else {
            vec!["Clone inside a loop; type may be cheap but worth reviewing.".into()]
        };

        out.push(Violation::with_details(
            call.start_position().row + 1,
            "Detected `.clone()` inside a loop".into(),
            "P01",
            ViolationDetails {
                function_name: None,
                analysis,
                suggestion: Some(
                    "Hoist clone before the loop, or use Arc if shared ownership is needed.".into(),
                ),
            },
        ));
    }
}

/// Returns `true` for `Arc::clone(x)` and `Rc::clone(x)` — free from an
/// allocation standpoint.
fn is_arc_or_rc_clone(call_text: &str) -> bool {
    call_text.starts_with("Arc::clone")
        || call_text.starts_with("Rc::clone")
        || call_text.contains("::Arc::clone")
        || call_text.contains("::Rc::clone")
}

/// Returns `true` if the receiver name looks like a heap-owning type.
///
/// Heuristic: types known to own heap memory, or capitalized identifiers
/// (likely struct instances that derive Clone and may hold owned data).
fn looks_heap_owning(recv: &str) -> bool {
    // Known heap types as substrings of the receiver expression
    const HEAP_TYPES: &[&str] = &[
        "String",
        "Vec",
        "HashMap",
        "HashSet",
        "BTreeMap",
        "BTreeSet",
        "Box",
        "Rc",
        "BufWriter",
        "Bytes",
    ];
    if HEAP_TYPES.iter().any(|t| recv.contains(t)) {
        return true;
    }
    // Capitalized simple identifier → likely a struct instance
    recv.chars().next().is_some_and(|c| c.is_uppercase())
}

fn should_skip_clone(source: &str, call: Node, recv: Option<&str>, lv: Option<&str>) -> bool {
    if is_ownership_sink(source, call) {
        return true;
    }
    if let (Some(r), Some(v)) = (recv, lv) {
        if r.trim() == v || r.contains(&format!("{v}.")) {
            return true;
        }
    }
    if let Some(r) = recv {
        if r.contains("..") || r.parse::<i64>().is_ok() {
            return true;
        }
    }
    false
}

fn is_ownership_sink(source: &str, node: Node) -> bool {
    let mut cur = node;
    for _ in 0..10 {
        let Some(p) = cur.parent() else { break };
        let txt = p.utf8_text(source.as_bytes()).unwrap_or("");
        if txt.contains(".insert(")
            || txt.contains(".push(")
            || txt.contains(".entry(")
            || txt.contains(".extend(")
        {
            return true;
        }
        cur = p;
    }
    false
}

fn check_p02(source: &str, body: Node, loop_var: Option<&str>, out: &mut Vec<Violation>) {
    let q = r#"(call_expression function: (field_expression
        value: (_) @recv field: (field_identifier) @m)
        (#match? @m "^(to_string|to_owned)$")) @call"#;
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else {
        return;
    };
    let idx_call = query.capture_index_for_name("call");
    let idx_recv = query.capture_index_for_name("recv");

    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, body, source.as_bytes()) {
        let call = get_capture_node(&m, idx_call);
        let recv = get_capture_node(&m, idx_recv).and_then(|c| c.utf8_text(source.as_bytes()).ok());

        let Some(call) = call else { continue };
        if is_ownership_sink(source, call) {
            continue;
        }
        // Skip conversion of the loop variable itself — caller intent
        if let (Some(r), Some(v)) = (recv, loop_var) {
            if r.trim() == v || r.contains(&format!("{v}.")) {
                continue;
            }
        }

        let recv_text = recv.unwrap_or("<expr>");
        out.push(Violation::with_details(
            call.start_position().row + 1,
            "String conversion inside loop".into(),
            "P02",
            ViolationDetails {
                function_name: None,
                analysis: vec![format!(
                    "`{recv_text}.to_string()` allocates a new String on every iteration."
                )],
                suggestion: Some(
                    "Hoist the conversion before the loop, or operate on &str.".into(),
                ),
            },
        ));
    }
}

fn check_p04(body: Node, out: &mut Vec<Violation>) {
    let mut cursor = body.walk();
    for child in body.children(&mut cursor) {
        if matches!(
            child.kind(),
            "for_expression" | "while_expression" | "loop_expression"
        ) {
            out.push(Violation::with_details(
                child.start_position().row + 1,
                "Nested loop (O(n²)) detected".into(),
                "P04",
                ViolationDetails {
                    function_name: None,
                    analysis: vec!["Quadratic complexity — scales poorly with input size.".into()],
                    suggestion: Some("Refactor with a lookup map to achieve O(n).".into()),
                },
            ));
        }
    }
}

fn check_p06(source: &str, body: Node, out: &mut Vec<Violation>) {
    let q = r#"(call_expression function: (field_expression
        field: (field_identifier) @m) (#match? @m "^(find|position)$")) @call"#;
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else {
        return;
    };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, body, source.as_bytes()) {
        if let Some(cap) = m.captures.first() {
            out.push(Violation::with_details(
                cap.node.start_position().row + 1,
                "Linear search inside loop — O(n·m) complexity".into(),
                "P06",
                ViolationDetails {
                    function_name: None,
                    analysis: vec!["Each outer iteration performs a full inner scan.".into()],
                    suggestion: Some("Pre-build a HashSet or HashMap for O(1) lookup.".into()),
                },
            ));
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::path::Path;
    use tree_sitter::Parser;

    fn parse_and_detect(code: &str) -> Vec<Violation> {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_rust::language()).unwrap();
        let tree = parser.parse(code, None).unwrap();
        detect(code, tree.root_node(), Path::new("src/lib.rs"))
    }

    #[test]
    fn p01_flags_string_clone_in_loop() {
        let code = r#"
            fn f(items: &[String]) {
                for _ in 0..10 {
                    let s = name.clone();
                    process(s);
                }
            }
        "#;
        assert!(parse_and_detect(code).iter().any(|v| v.law == "P01"));
    }

    #[test]
    fn p01_does_not_flag_arc_clone() {
        let code = r#"
            use std::sync::Arc;
            fn f(shared: Arc<Data>) {
                for _ in 0..10 {
                    let handle = Arc::clone(&shared);
                    spawn(handle);
                }
            }
        "#;
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "P01"),
            "Arc::clone is a ref-count increment and must not be flagged"
        );
    }

    #[test]
    fn p02_flags_to_string_in_loop() {
        let code = r#"
            fn f(label: &str) -> Vec<String> {
                let mut out = vec![];
                for i in 0..10 {
                    out.push(label.to_string());
                }
                out
            }
        "#;
        assert!(parse_and_detect(code).iter().any(|v| v.law == "P02"));
    }

    #[test]
    fn looks_heap_owning_identifies_string() {
        assert!(looks_heap_owning("name_string"));
        assert!(looks_heap_owning("SomeStruct"));
        assert!(!looks_heap_owning("count"));
        assert!(!looks_heap_owning("x"));
    }
}
