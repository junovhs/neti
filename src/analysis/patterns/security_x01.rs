// src/analysis/patterns/security_x01.rs
//! X01: SQL Injection — format!() used to build SQL strings.

use super::super::get_capture_node;
use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

pub(super) fn detect_x01_sql(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r#"(macro_invocation macro: (identifier) @mac (token_tree) @args (#eq? @mac "format")) @fmt"#;
    let Ok(query) = Query::new(&tree_sitter_rust::LANGUAGE.into(), q) else {
        return;
    };
    let idx_args = query.capture_index_for_name("args");
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(arg_node) = get_capture_node(&m, idx_args) {
            let args = arg_node.utf8_text(source.as_bytes()).unwrap_or("");
            if is_suspicious_sql(args) {
                out.push(Violation::with_details(
                    arg_node.start_position().row + 1,
                    "Potential SQL Injection".into(),
                    "X01",
                    ViolationDetails {
                        function_name: None,
                        analysis: vec!["Formatting into SQL bypasses parameterization.".into()],
                        suggestion: Some("Use parameterized queries.".into()),
                    },
                ));
            }
        }
    }
}

fn is_suspicious_sql(text: &str) -> bool {
    let upper = text.to_uppercase();
    let has_sql = upper.contains("SELECT ")
        || upper.contains("INSERT INTO ")
        || upper.contains("UPDATE ")
        || upper.contains("DELETE FROM ");
    let has_interp = text.contains("{}") || text.contains("{:");
    has_sql && has_interp
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::types::Violation;
    use tree_sitter::Parser;

    fn parse_and_detect(code: &str) -> Vec<Violation> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
        let tree = parser.parse(code, None).unwrap();
        super::super::detect(code, tree.root_node())
    }

    #[test]
    fn x01_flags_sql_format() {
        let code = r#"fn q(id: i32) { let _ = format!("SELECT * FROM users WHERE id = {}", id); }"#;
        assert!(parse_and_detect(code).iter().any(|v| v.law == "X01"));
    }
}
