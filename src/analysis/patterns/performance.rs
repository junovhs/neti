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
//! - Any clone inside `#[test]` functions or `#[cfg(test)]` modules
//!
//! ## P02 — String conversion in loop
//!
//! Only escalate for `.to_string()` / `.to_owned()` when the receiver is
//! statically likely to allocate (e.g., a `&str` literal or a field with a
//! string type). Conversions on loop variables are often the caller's intent.
//!
//! ## P06 — Linear search in loop
//!
//! Skipped inside `#[test]` functions or `#[cfg(test)]` modules — test code
//! routinely does small-collection linear scans for verification purposes.

use super::get_capture_node;
use crate::types::{Confidence, Violation, ViolationDetails};
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
        (for_expression pattern: _ @pat body: (block) @body) @loop
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

        let in_test = is_test_context(source, body);

        if !in_test {
            check_p01(source, body, loop_var.as_deref(), out);
        }
        check_p02(source, body, loop_var.as_deref(), out);
        check_p04(body, out);
        if !in_test {
            check_p06(source, body, out);
        }
    }
}

/// Returns `true` if the node is inside a `#[test]` function or `#[cfg(test)]` module.
fn is_test_context(source: &str, node: Node) -> bool {
    let mut cur = node;
    for _ in 0..30 {
        let Some(p) = cur.parent() else { break };

        match p.kind() {
            "function_item" => {
                if has_test_attribute(source, p) {
                    return true;
                }
            }
            "mod_item" => {
                if has_cfg_test_attribute(source, p) {
                    return true;
                }
            }
            "source_file" => break,
            _ => {}
        }

        cur = p;
    }
    false
}

/// Checks if a function node has a `#[test]` attribute.
fn has_test_attribute(source: &str, fn_node: Node) -> bool {
    if let Some(parent) = fn_node.parent() {
        let mut cursor = parent.walk();
        let mut prev_was_attr = false;
        for child in parent.children(&mut cursor) {
            if child.id() == fn_node.id() && prev_was_attr {
                return true;
            }
            if child.kind() == "attribute_item" {
                let text = child.utf8_text(source.as_bytes()).unwrap_or("");
                if text.contains("#[test]") {
                    prev_was_attr = true;
                    continue;
                }
            }
            prev_was_attr = false;
        }
    }

    let fn_start = fn_node.start_byte();
    if fn_start > 100 {
        let prefix = &source[fn_start.saturating_sub(100)..fn_start];
        if prefix.contains("#[test]") {
            return true;
        }
    } else if fn_start > 0 {
        let prefix = &source[..fn_start];
        if prefix.contains("#[test]") {
            if let Some(pos) = prefix.rfind("#[test]") {
                let between = &prefix[pos..];
                let non_attr = between.lines().skip(1).any(|l| {
                    let t = l.trim();
                    !t.is_empty()
                        && !t.starts_with("#[")
                        && !t.starts_with("fn ")
                        && !t.starts_with("pub ")
                        && !t.starts_with("async ")
                });
                if !non_attr {
                    return true;
                }
            }
        }
    }

    false
}

/// Checks if a module node has a `#[cfg(test)]` attribute.
fn has_cfg_test_attribute(source: &str, mod_node: Node) -> bool {
    if let Some(parent) = mod_node.parent() {
        let mut cursor = parent.walk();
        let mut prev_was_cfg_test = false;
        for child in parent.children(&mut cursor) {
            if child.id() == mod_node.id() && prev_was_cfg_test {
                return true;
            }
            if child.kind() == "attribute_item" {
                let text = child.utf8_text(source.as_bytes()).unwrap_or("");
                if text.contains("#[cfg(test)]") {
                    prev_was_cfg_test = true;
                    continue;
                }
            }
            prev_was_cfg_test = false;
        }
    }

    let mod_start = mod_node.start_byte();
    let look_back = mod_start.min(50);
    if look_back > 0 {
        let prefix = &source[mod_start - look_back..mod_start];
        if prefix.contains("#[cfg(test)]") {
            return true;
        }
    }

    false
}

fn check_p01(source: &str, body: Node, loop_var: Option<&str>, out: &mut Vec<Violation>) {
    let q = r#"(call_expression function: (field_expression
        value: (_) @recv field: (field_identifier) @m)
        (#eq? @m "clone")) @call"#;
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else {
        return;
    };
    let idx_call = query.capture_index_for_name("call");
    let idx_recv = query.capture_index_for_name("recv");

    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, body, source.as_bytes()) {
        let Some(call) = get_capture_node(&m, idx_call) else {
            continue;
        };
        let recv = get_capture_node(&m, idx_recv).and_then(|c| c.utf8_text(source.as_bytes()).ok());

        let call_text = call.utf8_text(source.as_bytes()).unwrap_or("");
        if is_arc_or_rc_clone(call_text) {
            continue;
        }

        if should_skip_clone(source, call, recv, loop_var) {
            continue;
        }

        let recv_text = recv.unwrap_or("");
        if !looks_heap_owning(recv_text) {
            continue;
        }

        // Determine confidence: if receiver contains `[` (indexed access) or
        // starts with `self.`, the type is ambiguous — could be generic
        let (confidence, reason) = if recv_text.contains('[') || recv_text.contains("self.") {
            (
                Confidence::Medium,
                Some(
                    "clone is on an indexed or field expression — type may be cheap to clone"
                        .to_string(),
                ),
            )
        } else {
            (Confidence::High, None)
        };

        let mut v = Violation::with_details(
            call.start_position().row + 1,
            "Detected `.clone()` inside a loop".into(),
            "P01",
            ViolationDetails {
                function_name: None,
                analysis: vec![
                    format!("Receiver `{recv_text}` appears to be a heap-owning type."),
                    "Clone inside a loop allocates on every iteration.".into(),
                ],
                suggestion: Some(
                    "Hoist clone before the loop, or use Arc if shared ownership is needed.".into(),
                ),
            },
        );
        v.confidence = confidence;
        v.confidence_reason = reason;
        out.push(v);
    }
}

/// Returns `true` for `Arc::clone(x)` and `Rc::clone(x)`.
fn is_arc_or_rc_clone(call_text: &str) -> bool {
    call_text.starts_with("Arc::clone")
        || call_text.starts_with("Rc::clone")
        || call_text.contains("::Arc::clone")
        || call_text.contains("::Rc::clone")
}

/// Returns `true` if the receiver name looks like a heap-owning type.
fn looks_heap_owning(recv: &str) -> bool {
    let r = recv.trim();
    if r.is_empty() {
        return true;
    }

    if r.chars().next().is_some_and(|c| c.is_uppercase()) {
        return true;
    }

    let lower = r.to_lowercase();
    let heap_keywords = [
        "string",
        "vec",
        "map",
        "set",
        "box",
        "rc",
        "bufwriter",
        "bytes",
        "name",
        "text",
        "data",
        "list",
        "array",
        "items",
        "buffer",
        "cache",
    ];

    if heap_keywords.iter().any(|&k| lower.contains(k)) {
        return true;
    }

    r.len() > 2
}

fn should_skip_clone(source: &str, call: Node, recv: Option<&str>, lv: Option<&str>) -> bool {
    if is_ownership_sink(source, call) {
        return true;
    }
    if let (Some(r), Some(v)) = (recv, lv) {
        let r_trim = r.trim();
        let v_trim = v.trim();
        if !r_trim.is_empty()
            && !v_trim.is_empty()
            && (r_trim == v_trim || r_trim.starts_with(&format!("{v_trim}.")))
        {
            return true;
        }
    }
    false
}

fn is_ownership_sink(source: &str, node: Node) -> bool {
    let mut cur = node;
    for _ in 0..5 {
        let Some(p) = cur.parent() else { break };
        if p.kind() == "call_expression" {
            if let Some(func) = p.child_by_field_name("function") {
                let txt = func.utf8_text(source.as_bytes()).unwrap_or("");
                if txt.ends_with(".insert")
                    || txt.ends_with(".push")
                    || txt.ends_with(".entry")
                    || txt.ends_with(".extend")
                {
                    return true;
                }
            }
        }
        if p.kind() == "expression_statement" || p.kind() == "let_declaration" {
            break;
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
            let mut v = Violation::with_details(
                child.start_position().row + 1,
                "Nested loop (O(n²)) detected".into(),
                "P04",
                ViolationDetails {
                    function_name: None,
                    analysis: vec!["Quadratic complexity — scales poorly with input size.".into()],
                    suggestion: Some("Refactor with a lookup map to achieve O(n).".into()),
                },
            );
            v.confidence = Confidence::Medium;
            v.confidence_reason = Some("inner loop may be bounded to a small constant".into());
            out.push(v);
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
            let mut v = Violation::with_details(
                cap.node.start_position().row + 1,
                "Linear search inside loop — O(n·m) complexity".into(),
                "P06",
                ViolationDetails {
                    function_name: None,
                    analysis: vec!["Each outer iteration performs a full inner scan.".into()],
                    suggestion: Some("Pre-build a HashSet or HashMap for O(1) lookup.".into()),
                },
            );
            v.confidence = Confidence::Medium;
            v.confidence_reason =
                Some("linear scan may be intentional algorithm or bounded collection".into());
            out.push(v);
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
    fn p01_skipped_in_test_function() {
        let code = r#"
            #[test]
            fn test_sampling() {
                for _ in 0..1000 {
                    let picked = iter.clone().choose(r).unwrap();
                    process(picked);
                }
            }
        "#;
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "P01"),
            "P01 must not fire inside #[test] functions"
        );
    }

    #[test]
    fn p01_skipped_in_cfg_test_module() {
        let code = r#"
            #[cfg(test)]
            mod tests {
                fn helper() {
                    for _ in 0..10 {
                        let s = name.clone();
                        process(s);
                    }
                }
            }
        "#;
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "P01"),
            "P01 must not fire inside #[cfg(test)] modules"
        );
    }

    #[test]
    fn p06_skipped_in_test_function() {
        let code = r#"
            #[test]
            fn test_position() {
                for i in 0..4 {
                    let pos = arr.iter().position(|&x| x == i).unwrap();
                    assert!(pos < 4);
                }
            }
        "#;
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "P06"),
            "P06 must not fire inside #[test] functions"
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
        assert!(!looks_heap_owning("i"));
        assert!(!looks_heap_owning("x"));
    }
}
