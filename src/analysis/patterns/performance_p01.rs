// src/analysis/patterns/performance_p01.rs
//! P01: `.clone()` inside a loop.

use super::super::get_capture_node;
use super::{HEAP_KEYWORDS, HEURISTIC_KEYWORDS};
use crate::types::{Confidence, Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

#[cfg(test)]
#[path = "performance_p01_test.rs"]
mod tests;

pub(super) fn check_p01(
    source: &str,
    body: Node,
    loop_var: Option<&str>,
    out: &mut Vec<Violation>,
) {
    let q = r#"(call_expression function: (field_expression
        value: (_) @recv field: (field_identifier) @m)
        (#eq? @m "clone")) @call"#;
    let Ok(query) = Query::new(&tree_sitter_rust::LANGUAGE.into(), q) else {
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

pub(super) fn is_arc_or_rc_clone(call_text: &str) -> bool {
    call_text.starts_with("Arc::clone")
        || call_text.starts_with("Rc::clone")
        || call_text.contains("::Arc::clone")
        || call_text.contains("::Rc::clone")
}

pub(super) fn looks_heap_owning(recv: &str) -> bool {
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
