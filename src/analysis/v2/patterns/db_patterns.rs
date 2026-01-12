// src/analysis/v2/patterns/db_patterns.rs
//! Database anti-patterns: P03 (N+1 queries)

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};
use super::get_capture_node;

#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut out = Vec::new();
    detect_p03(source, root, &mut out);
    out
}

/// P03: N+1 Query - DB call inside loop using loop variable
fn detect_p03(source: &str, root: Node, out: &mut Vec<Violation>) {
    let loop_q = r"
        (for_expression pattern: (_) @pat body: (block) @body) @loop
        (while_expression body: (block) @body) @loop
    ";

    let Ok(query) = Query::new(tree_sitter_rust::language(), loop_q) else { return };
    let idx_pat = query.capture_index_for_name("pat");
    let idx_body = query.capture_index_for_name("body");
    
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let loop_var_node = get_capture_node(&m, idx_pat);
        let body_node = get_capture_node(&m, idx_body);

        let (Some(var_node), Some(body)) = (loop_var_node, body_node) else { continue };
        
        if let Ok(var_text) = var_node.utf8_text(source.as_bytes()) {
            let loop_var = extract_loop_var(var_text);
            check_db_calls(source, body, &loop_var, out);
        }
    }
}

fn extract_loop_var(pattern: &str) -> String {
    pattern.trim().trim_start_matches('(').split(',').next().unwrap_or(pattern).trim().to_string()
}

fn check_db_calls(source: &str, body: Node, loop_var: &str, out: &mut Vec<Violation>) {
    let patterns = [
        r#"(call_expression function: (field_expression field: (field_identifier) @m)
            (#match? @m "^(fetch_one|fetch_all|fetch_optional|execute|query|query_as|execute_many)$")) @call"#,
        r#"(call_expression function: (field_expression field: (field_identifier) @m)
            (#match? @m "^(load|get_result|get_results)$")) @call"#,
        r#"(call_expression function: (field_expression field: (field_identifier) @m)
            (#match? @m "^(find_by|save|delete|update)$")) @call"#,
    ];

    for pattern in patterns {
        check_pattern(source, body, pattern, loop_var, out);
    }
}

fn check_pattern(source: &str, body: Node, pattern: &str, loop_var: &str, out: &mut Vec<Violation>) {
    let Ok(query) = Query::new(tree_sitter_rust::language(), pattern) else { return };
    let idx_call = query.capture_index_for_name("call");
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, body, source.as_bytes()) {
        let Some(call) = get_capture_node(&m, idx_call) else { continue };
        let call_text = call.utf8_text(source.as_bytes()).unwrap_or("");

        if !call_text.contains(loop_var) { continue }
        if is_likely_safe_method(call_text) { continue }

        let method = call_text.split('.').next_back()
            .and_then(|s| s.split('(').next())
            .unwrap_or("query");

        out.push(Violation::with_details(
            call.start_position().row + 1,
            format!("Potential N+1 query: `{method}` in loop"),
            "P03",
            ViolationDetails {
                function_name: None,
                analysis: vec![
                    "DB call inside loop causes N+1 queries.".into(),
                    format!("Loop variable `{loop_var}` used in call."),
                ],
                suggestion: Some("Batch the query or use JOIN/IN.".into()),
            }
        ));
    }
}

fn is_likely_safe_method(text: &str) -> bool {
    text.contains(".iter()") ||
    text.contains(".into_iter()") ||
    text.contains(".chars()") ||
    text.contains(".lines()") ||
    text.contains(".unwrap_or") ||
    text.contains(".map(") ||
    text.contains(".get(") ||
    text.contains(".find(")
}