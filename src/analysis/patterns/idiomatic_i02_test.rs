// src/analysis/patterns/idiomatic_i02_test.rs

use super::*;
use tree_sitter::Parser;

fn parse_and_detect(code: &str) -> Vec<Violation> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_rust::language()).unwrap();
    let tree = parser.parse(code, None).unwrap();
    let mut violations = Vec::new();
    detect_i02(code, tree.root_node(), &mut violations);
    violations
}

#[test]
fn i02_flag_duplicate_arms() {
    let code = r#"
        fn f(x: Option<i32>) -> &str {
            match x {
                Some(_) => "yes",
                None => "yes",
            }
        }
    "#;
    assert!(parse_and_detect(code).iter().any(|v| v.law == "I02"));
}

#[test]
fn i02_skip_unique_arms() {
    let code = r#"
        fn f(x: Option<i32>) -> &str {
            match x {
                Some(_) => "yes",
                None => "no",
            }
        }
    "#;
    assert!(parse_and_detect(code).iter().all(|v| v.law != "I02"));
}

#[test]
fn i02_skip_different_variant_types() {
    let code = r#"
        enum IndexVec {
            U32(Vec<u32>),
            U64(Vec<u64>),
        }
        impl IndexVec {
            fn len(&self) -> usize {
                match self {
                    IndexVec::U32(v) => v.len(),
                    IndexVec::U64(v) => v.len(),
                }
            }
        }
    "#;
    assert!(
        parse_and_detect(code).iter().all(|v| v.law != "I02"),
        "Different variant types must not be flagged as fuseable"
    );
}

#[test]
fn i02_skip_tuple_match_different_variants() {
    let code = r#"
        enum Idx { U32(Vec<u32>), U64(Vec<u64>) }
        impl PartialEq for Idx {
            fn eq(&self, other: &Self) -> bool {
                use Idx::*;
                match (self, other) {
                    (U32(v1), U32(v2)) => v1 == v2,
                    (U64(v1), U64(v2)) => v1 == v2,
                    _ => false,
                }
            }
        }
    "#;
    assert!(
        parse_and_detect(code).iter().all(|v| v.law != "I02"),
        "Tuple match with different variant types must not be flagged"
    );
}

#[test]
fn i02_still_flags_same_variant_duplicates() {
    let code = r#"
        fn f(x: i32) -> &str {
            match x {
                1 => "same",
                2 => "same",
                _ => "other",
            }
        }
    "#;
    assert!(
        parse_and_detect(code).iter().any(|v| v.law == "I02"),
        "Literal patterns with same body should still be flagged"
    );
}
