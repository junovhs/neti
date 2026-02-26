// src/analysis/patterns/idiomatic_i01.rs
//! I01: Manual `From` implementations that could use `derive_more::From`.
//!
//! Reported as INFO — style suggestion, not a correctness issue.

use crate::types::{Confidence, Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

/// Detects manual `From` implementations. Reported as INFO — style suggestion.
pub(super) fn detect_i01(source: &str, root: Node, out: &mut Vec<Violation>) {
    let q = r"(impl_item) @impl";
    let Ok(query) = Query::new(tree_sitter_rust::language(), q) else {
        return;
    };

    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, root, source.as_bytes()) {
        let Some(impl_node) = m.captures.first().map(|c| c.node) else {
            continue;
        };

        let impl_text = impl_node.utf8_text(source.as_bytes()).unwrap_or("");
        let header = impl_text.lines().next().unwrap_or("");

        if !is_from_impl(header) {
            continue;
        }

        if is_error_from_impl(impl_text) {
            continue;
        }

        let mut v = Violation::with_details(
            impl_node.start_position().row + 1,
            "Manual `From` impl — consider derive_more if already using proc macros".into(),
            "I01",
            ViolationDetails {
                function_name: None,
                analysis: vec![
                    "This `From` impl could be generated with `#[derive(From)]`.".into(),
                    "Note: many crates intentionally avoid proc-macro dependencies.".into(),
                ],
                suggestion: Some(
                    "Use derive_more::From if your crate already uses proc macros. Otherwise this is fine as-is.".into(),
                ),
            },
        );
        v.confidence = Confidence::Info;
        out.push(v);
    }
}

fn is_from_impl(header: &str) -> bool {
    let trimmed = header.trim();
    let after_impl = match trimmed.strip_prefix("impl") {
        Some(rest) => rest.trim_start(),
        None => return false,
    };
    after_impl.starts_with("From<") || after_impl.starts_with("From ")
}

fn is_error_from_impl(impl_text: &str) -> bool {
    let lower = impl_text.to_lowercase();
    lower.contains("error") || lower.contains("err")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::types::{Confidence, Violation};
    use tree_sitter::Parser;

    fn parse_and_detect(code: &str) -> Vec<Violation> {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_rust::language()).unwrap();
        let tree = parser.parse(code, None).unwrap();
        super::super::detect(code, tree.root_node())
    }

    #[test]
    fn i01_flag_simple_from() {
        let code = r#"
            impl From<String> for MyType {
                fn from(s: String) -> Self { MyType(s) }
            }
        "#;
        let vs = parse_and_detect(code);
        assert!(vs.iter().any(|v| v.law == "I01"));
        assert_eq!(
            vs.iter().find(|v| v.law == "I01").unwrap().confidence,
            Confidence::Info
        );
    }

    #[test]
    fn i01_skip_error_from() {
        let code = r#"
            impl From<IoError> for MyError {
                fn from(e: IoError) -> Self { MyError::Io(e) }
            }
        "#;
        assert!(parse_and_detect(code).iter().all(|v| v.law != "I01"));
    }

    #[test]
    fn i01_message_is_suggestion_not_error() {
        let code = r#"
            impl From<String> for MyType {
                fn from(s: String) -> Self { MyType(s) }
            }
        "#;
        let violations = parse_and_detect(code);
        let i01 = violations.iter().find(|v| v.law == "I01").unwrap();
        assert!(
            i01.message.contains("consider"),
            "I01 message should be suggestive, not imperative"
        );
    }
}
