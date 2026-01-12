// src/analysis/v2/patterns/db_patterns.rs
//! Database anti-patterns: P03 (N+1 queries)

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor, QueryCapture};

#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut out = Vec::new();
    detect_p03(source, root, &mut out);
    out
}

fn cap_name<'a>(query: &'a Query, cap: &QueryCapture) -> &'a str {
    query.capture_names().get(cap.index as usize).map_or("", String::as_str)
}

/// P03: N+1 Query - DB call inside loop using loop variable
fn detect_p03(source: &str, root: Node, out: &mut Vec<Violation>) {
    let loop_q = r"
        (for_expression pattern: (_) @pat body: (block) @body) @loop
        (while_expression body: (block) @body) @loop
    ";

    let Ok(query) = Query::new(tree_sitter_rust::language(), loop_q) else { return };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        let loop_var = m.captures.iter().find(|c| c.index == 0)
            .and_then(|c| c.node.utf8_text(source.as_bytes()).ok())
            .map(extract_loop_var);

        let Some(body) = m.captures.iter()
            .find(|c| cap_name(&query, c) == "body")
            .map(|c| c.node) else { continue };

        let Some(loop_var) = loop_var else { continue };
        check_db_calls(source, body, &loop_var, out);
    }
}

fn extract_loop_var(pattern: &str) -> String {
    pattern.trim().trim_start_matches('(').split(',').next().unwrap_or(pattern).trim().to_string()
}

fn check_db_calls(source: &str, body: Node, loop_var: &str, out: &mut Vec<Violation>) {
    let patterns = [
        r#"(call_expression function: (field_expression field: (field_identifier) @m)
            (#match? @m "^(fetch_one|fetch_all|fetch_optional|execute)$")) @call"#,
        r#"(call_expression function: (field_expression field: (field_identifier) @m)
            (#match? @m "^(load|get_result|first)$")) @call"#,
        r#"(call_expression function: (field_expression field: (field_identifier) @m)
            (#match? @m "^(query|find|find_by|get|select)$")) @call"#,
    ];

    for pattern in patterns {
        check_pattern(source, body, pattern, loop_var, out);
    }
}

fn check_pattern(source: &str, body: Node, pattern: &str, loop_var: &str, out: &mut Vec<Violation>) {
    let Ok(query) = Query::new(tree_sitter_rust::language(), pattern) else { return };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, body, source.as_bytes()) {
        let call = m.captures.iter().find(|c| cap_name(&query, c) == "call").map(|c| c.node);
        let Some(call) = call else { continue };

        let call_text = call.utf8_text(source.as_bytes()).unwrap_or("");
        if !call_text.contains(loop_var) { continue }

        let method = call_text.split('.').next_back()
            .and_then(|s| s.split('(').next())
            .unwrap_or("query");

        out.push(Violation::with_details(
            call.start_position().row + 1,
            format!("N+1 query: `{method}` in loop"),
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

    #[test]
    fn p03_flag_fetch_in_loop() {
        let code = "async fn f(ids: Vec<i32>) { for id in ids { db.fetch_all(id).await; } }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "P03"));
    }

    #[test]
    fn p03_flag_query_in_loop() {
        let code = "fn f(ids: &[i32]) { for id in ids { repo.find_by(id); } }";
        assert!(parse_and_detect(code).iter().any(|v| v.law == "P03"));
    }

    #[test]
    fn p03_skip_unrelated_call() {
        let code = "fn f(items: &[Item]) { for item in items { item.calculate(); } }";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "P03"));
    }

    #[test]
    fn p03_skip_no_loop_var() {
        let code = "async fn f(items: Vec<Item>) { let cfg = db.fetch_one(1).await; for item in items { process(item); } }";
        assert!(parse_and_detect(code).iter().all(|v| v.law != "P03"));
    }
}
