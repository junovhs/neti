// src/analysis/patterns/security_x03.rs
//! X03: Hardcoded secrets (keys, tokens, passwords) in let/const bindings.

use super::super::get_capture_node;
use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

pub(super) fn detect_x03_secrets(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r#"
        (let_declaration pattern: (identifier) @name value: (string_literal) @value
            (#match? @name "(?i)(key|secret|token|password|auth)")) @decl
        (const_item name: (identifier) @name value: (string_literal) @value
            (#match? @name "(?i)(key|secret|token|password|auth)")) @const
    "#;

    let Ok(query) = Query::new(&tree_sitter_rust::LANGUAGE.into(), q) else {
        return;
    };
    let idx_value = query.capture_index_for_name("value");
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(val) = get_capture_node(&m, idx_value) {
            let text = val.utf8_text(source.as_bytes()).unwrap_or("");
            if text.contains("placeholder")
                || text.contains("example")
                || text.contains("test")
                || text.contains("dummy")
                || text.len() < 5
            {
                continue;
            }
            out.push(Violation::with_details(
                val.start_position().row + 1,
                "Potential hardcoded secret".into(),
                "X03",
                ViolationDetails {
                    function_name: None,
                    analysis: vec!["Secrets should come from environment.".into()],
                    suggestion: Some("Use `std::env::var()`.".into()),
                },
            ));
        }
    }
}
