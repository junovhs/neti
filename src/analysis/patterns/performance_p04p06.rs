// src/analysis/patterns/performance_p04p06.rs
//! P04: Nested loop (O(n²)) and P06: linear search inside loop.

use crate::types::{Confidence, Violation, ViolationDetails};
use omni_ast::{semantics_for, Concept, LangSemantics, SemanticContext, SemanticLanguage};
use tree_sitter::Node;

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

pub(super) fn check_p06(source: &str, body: Node, language: SemanticLanguage, out: &mut Vec<Violation>) {
    let Some(body_text) = body.utf8_text(source.as_bytes()).ok() else {
        return;
    };

    if detects_linear_lookup(language, body_text) {
        out.push(shared_p06_violation(
            body.start_position().row + 1,
            "Each outer iteration performs a full inner scan.".into(),
        ));
    }
}

fn detects_linear_lookup(language: SemanticLanguage, loop_body: &str) -> bool {
    let semantics = semantics_for(language);
    semantics.has_concept(Concept::Lookup, &SemanticContext::from_source(loop_body))
}

pub(super) fn shared_p06_violation(row: usize, analysis: String) -> Violation {
    let mut v = Violation::with_details(
        row,
        "Linear search inside loop — O(n·m) complexity".into(),
        "P06",
        ViolationDetails {
            function_name: None,
            analysis: vec![analysis],
            suggestion: Some("Pre-build a HashSet or HashMap for O(1) lookup.".into()),
        },
    );
    v.confidence = Confidence::Medium;
    v.confidence_reason =
        Some("linear scan may be intentional algorithm or bounded collection".into());
    v
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::types::{Confidence, Violation};
    use omni_ast::SemanticLanguage;
    use std::path::Path;
    use tree_sitter::Parser;

    fn parse_and_detect(code: &str) -> Vec<Violation> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(code, None).unwrap();
        super::super::detect(code, Some(tree.root_node()), Path::new("src/lib.rs"))
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

    #[test]
    fn shared_semantics_drive_p06_across_languages() {
        assert!(super::detects_linear_lookup(
            SemanticLanguage::Rust,
            "let found = haystack.iter().find(|&&x| x == needle);"
        ));
        assert!(super::detects_linear_lookup(
            SemanticLanguage::Python,
            "if needle in haystack:\n    hits.append(needle)\n"
        ));
        assert!(super::detects_linear_lookup(
            SemanticLanguage::TypeScript,
            "const found = haystack.find((value) => value === needle);"
        ));
    }
}
