// src/analysis/patterns/performance_p04p06.rs
//! P04: Nested loop (O(n²)) and P06: linear search inside loop.

use crate::types::{Confidence, Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

// ── P04 ─────────────────────────────────────────────────────────────────────

pub(super) fn check_p04(body: Node, out: &mut Vec<Violation>) {
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

pub(super) fn check_p06(source: &str, body: Node, out: &mut Vec<Violation>) {
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
    use crate::types::{Confidence, Violation};
    use std::path::Path;
    use tree_sitter::Parser;

    fn parse_and_detect(code: &str) -> Vec<Violation> {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_rust::language()).unwrap();
        let tree = parser.parse(code, None).unwrap();
        super::super::detect(code, tree.root_node(), Path::new("src/lib.rs"))
    }

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
}
