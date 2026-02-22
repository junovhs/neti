//! Performance anti-patterns: P01, P02, P04, P06
//!
//! # Escalation Philosophy
//!
//! P01/P02 must only fire when we can make a reasonable argument that the
//! allocation is *material*. Blanket "clone in loop" flags generate lint spam
//! and train developers to ignore Neti. The goal is signal, not volume.

use super::get_capture_node;
use super::performance_test_ctx::is_test_context;
use crate::types::{Confidence, Violation, ViolationDetails};
use std::path::Path;
use tree_sitter::{Node, Query, QueryCursor};

/// Heap-type keywords used for High confidence classification.
const HEAP_KEYWORDS: &[&str] = &[
    "string",
    "vec",
    "map",
    "set",
    "box",
    "rc",
    "bufwriter",
    "bytes",
    "buffer",
];

/// Heuristic keywords that suggest heap ownership but are less certain.
const HEURISTIC_KEYWORDS: &[&str] = &["name", "text", "data", "list", "array", "items", "cache"];

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

// ── P01 ─────────────────────────────────────────────────────────────────────

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

        let (confidence, reason) = classify_p01_confidence(recv_text);

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

/// Classify P01 confidence based on how certain we are the receiver is heap-owning.
fn classify_p01_confidence(recv: &str) -> (Confidence, Option<String>) {
    let r = recv.trim();

    if r.contains('[') || r.contains("self.") {
        return (
            Confidence::Medium,
            Some(
                "clone is on an indexed or field expression — type may be cheap to clone"
                    .to_string(),
            ),
        );
    }

    if r.chars().next().is_some_and(|c| c.is_uppercase()) {
        return (Confidence::High, None);
    }

    let lower = r.to_lowercase();

    if HEAP_KEYWORDS.iter().any(|&k| lower.contains(k)) {
        return (Confidence::High, None);
    }

    if HEURISTIC_KEYWORDS.iter().any(|&k| lower.contains(k)) {
        return (
            Confidence::Medium,
            Some(format!(
                "receiver `{r}` matches a heuristic keyword but type is unverified"
            )),
        );
    }

    (
        Confidence::Medium,
        Some(format!(
            "receiver `{r}` has no known heap-type indicator — type inference incomplete"
        )),
    )
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
    if HEAP_KEYWORDS.iter().any(|&k| lower.contains(k)) {
        return true;
    }
    if HEURISTIC_KEYWORDS.iter().any(|&k| lower.contains(k)) {
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

// ── P02 ─────────────────────────────────────────────────────────────────────

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

// ── P04 ─────────────────────────────────────────────────────────────────────

fn check_p04(body: Node, out: &mut Vec<Violation>) {
    find_nested_loops(body, out);
}

fn find_nested_loops(node: Node, out: &mut Vec<Violation>) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
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
        } else {
            find_nested_loops(child, out);
        }
    }
}

// ── P06 ─────────────────────────────────────────────────────────────────────

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

    // ── P01 basics ───────────────────────────────────────────────────────

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
        assert!(parse_and_detect(code).iter().all(|v| v.law != "P01"));
    }

    #[test]
    fn p01_does_not_flag_rc_clone() {
        let code = r#"
            use std::rc::Rc;
            fn f(shared: Rc<Data>) {
                for _ in 0..10 {
                    let handle = Rc::clone(&shared);
                    use_it(handle);
                }
            }
        "#;
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "P01"),
            "Rc::clone is a ref-count increment and must not be flagged"
        );
    }

    #[test]
    fn p01_does_not_flag_short_receiver() {
        // "x" is 1 char — looks_heap_owning returns false for len <= 2
        let code = r#"
            fn f() {
                for _ in 0..10 {
                    let y = x.clone();
                    process(y);
                }
            }
        "#;
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "P01"),
            "single-char receiver must not trigger P01"
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
        assert!(parse_and_detect(code).iter().all(|v| v.law != "P01"));
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
        assert!(parse_and_detect(code).iter().all(|v| v.law != "P01"));
    }

    #[test]
    fn p01_skipped_when_clone_into_push() {
        let code = r#"
            fn f(items: &[String]) {
                let mut out = vec![];
                for _ in 0..10 {
                    out.push(name_string.clone());
                }
            }
        "#;
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "P01"),
            ".push() is an ownership sink — clone into push should be skipped"
        );
    }

    #[test]
    fn p01_skipped_when_clone_into_entry() {
        let code = r#"
            fn f() {
                let mut map = std::collections::HashMap::new();
                for _ in 0..10 {
                    map.entry(name_string.clone());
                }
            }
        "#;
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "P01"),
            ".entry() is an ownership sink — clone into entry should be skipped"
        );
    }

    #[test]
    fn p01_skipped_when_clone_into_extend() {
        let code = r#"
            fn f() {
                let mut out = vec![];
                for _ in 0..10 {
                    out.extend(name_list.clone());
                }
            }
        "#;
        assert!(
            parse_and_detect(code).iter().all(|v| v.law != "P01"),
            ".extend() is an ownership sink — clone into extend should be skipped"
        );
    }

    // ── P01 confidence tiers ─────────────────────────────────────────────

    #[test]
    fn p01_high_confidence_for_known_heap_keyword() {
        let code = r#"
            fn f() {
                for _ in 0..10 {
                    let s = name_string.clone();
                    process(s);
                }
            }
        "#;
        let violations: Vec<_> = parse_and_detect(code)
            .into_iter()
            .filter(|v| v.law == "P01")
            .collect();
        assert!(!violations.is_empty());
        assert_eq!(violations[0].confidence, Confidence::High);
    }

    #[test]
    fn p01_medium_confidence_for_generic_long_name() {
        let code = r#"
            fn f() {
                for _ in 0..10 {
                    let s = cumulative_weight.clone();
                    process(s);
                }
            }
        "#;
        let violations: Vec<_> = parse_and_detect(code)
            .into_iter()
            .filter(|v| v.law == "P01")
            .collect();
        assert!(!violations.is_empty());
        assert_eq!(violations[0].confidence, Confidence::Medium);
    }

    #[test]
    fn p01_medium_confidence_for_indexed_access() {
        let code = r#"
            fn f(items: &[String]) {
                for i in 0..10 {
                    let s = self.weights[i].clone();
                    process(s);
                }
            }
        "#;
        let violations: Vec<_> = parse_and_detect(code)
            .into_iter()
            .filter(|v| v.law == "P01")
            .collect();
        assert!(!violations.is_empty());
        assert_eq!(violations[0].confidence, Confidence::Medium);
    }

    #[test]
    fn p01_high_confidence_for_uppercase_receiver() {
        let code = r#"
            fn f() {
                for _ in 0..10 {
                    let s = MyStruct.clone();
                    process(s);
                }
            }
        "#;
        let violations: Vec<_> = parse_and_detect(code)
            .into_iter()
            .filter(|v| v.law == "P01")
            .collect();
        assert!(!violations.is_empty());
        assert_eq!(violations[0].confidence, Confidence::High);
    }

    // ── P02 ──────────────────────────────────────────────────────────────

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

    // ── P04 ──────────────────────────────────────────────────────────────

    #[test]
    fn p04_flags_nested_loop() {
        let code = r#"
            fn f(matrix: &[Vec<i32>]) {
                for row in matrix {
                    for val in row {
                        process(val);
                    }
                }
            }
        "#;
        let violations: Vec<_> = parse_and_detect(code)
            .into_iter()
            .filter(|v| v.law == "P04")
            .collect();
        assert!(!violations.is_empty(), "nested loop must trigger P04");
        assert_eq!(violations[0].confidence, Confidence::Medium);
    }

    // ── P06 ──────────────────────────────────────────────────────────────

    #[test]
    fn p06_flags_find_in_loop() {
        let code = r#"
            fn f(needles: &[i32], haystack: &[i32]) {
                for needle in needles {
                    let found = haystack.iter().find(|&&x| x == *needle);
                    process(found);
                }
            }
        "#;
        let violations: Vec<_> = parse_and_detect(code)
            .into_iter()
            .filter(|v| v.law == "P06")
            .collect();
        assert!(!violations.is_empty(), "find() in loop must trigger P06");
        assert_eq!(violations[0].confidence, Confidence::Medium);
    }

    #[test]
    fn p06_flags_position_in_loop() {
        let code = r#"
            fn f(values: &[i32]) {
                for i in 0..10 {
                    let pos = values.iter().position(|&x| x == i);
                    process(pos);
                }
            }
        "#;
        assert!(
            parse_and_detect(code).iter().any(|v| v.law == "P06"),
            "position() in loop must trigger P06"
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
        assert!(parse_and_detect(code).iter().all(|v| v.law != "P06"));
    }

    // ── Helpers ──────────────────────────────────────────────────────────

    #[test]
    fn looks_heap_owning_identifies_string() {
        assert!(looks_heap_owning("name_string"));
        assert!(looks_heap_owning("SomeStruct"));
        assert!(!looks_heap_owning("i"));
        assert!(!looks_heap_owning("x"));
    }

    #[test]
    fn is_arc_or_rc_detects_variants() {
        assert!(is_arc_or_rc_clone("Arc::clone(&x)"));
        assert!(is_arc_or_rc_clone("Rc::clone(&x)"));
        assert!(is_arc_or_rc_clone("std::sync::Arc::clone(&x)"));
        assert!(!is_arc_or_rc_clone("name.clone()"));
    }
}
