// src/analysis/patterns/performance_p02.rs
//! P02: String conversion (`.to_string()` / `.to_owned()`) inside a loop.

use super::super::get_capture_node;
use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

pub(super) fn check_p02(
    source: &str,
    body: Node,
    loop_var: Option<&str>,
    out: &mut Vec<Violation>,
) {
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::types::Violation;
    use std::path::Path;
    use tree_sitter::Parser;

    fn parse_and_detect(code: &str) -> Vec<Violation> {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_rust::language()).unwrap();
        let tree = parser.parse(code, None).unwrap();
        super::super::detect(code, tree.root_node(), Path::new("src/lib.rs"))
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
}
