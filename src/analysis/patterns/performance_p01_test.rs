// src/analysis/patterns/performance_p01_test.rs

use super::*;
use crate::types::{Confidence, Violation};
use std::path::Path;
use tree_sitter::Parser;

fn parse_and_detect(code: &str) -> Vec<Violation> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
    let tree = parser.parse(code, None).unwrap();
    super::super::detect(code, tree.root_node(), Path::new("src/lib.rs"))
}

#[test]
fn p01_flags_string_clone_in_loop() {
    let code = r#"
        fn f(items: &[String]) {
            for _ in 0..10 {
                let s = name.clone();
                process(s);
            }
        }
    "#;
    assert!(parse_and_detect(code).iter().any(|v| v.law == "P01"));
}

#[test]
fn p01_does_not_flag_arc_clone() {
    let code = r#"
        use std::sync::Arc;
        fn f(shared: Arc<Data>) {
            for _ in 0..10 {
                let handle = Arc::clone(&shared);
                spawn(handle);
            }
        }
    "#;
    assert!(parse_and_detect(code).iter().all(|v| v.law != "P01"));
}

#[test]
fn p01_does_not_flag_rc_clone() {
    let code = r#"
        use std::rc::Rc;
        fn f(shared: Rc<Data>) {
            for _ in 0..10 {
                let handle = Rc::clone(&shared);
                use_it(handle);
            }
        }
    "#;
    assert!(
        parse_and_detect(code).iter().all(|v| v.law != "P01"),
        "Rc::clone is a ref-count increment and must not be flagged"
    );
}

#[test]
fn p01_does_not_flag_short_receiver() {
    let code = r#"
        fn f() {
            for _ in 0..10 {
                let y = x.clone();
                process(y);
            }
        }
    "#;
    assert!(
        parse_and_detect(code).iter().all(|v| v.law != "P01"),
        "single-char receiver must not trigger P01"
    );
}

#[test]
fn p01_skipped_in_test_function() {
    let code = r#"
        #[test]
        fn test_sampling() {
            for _ in 0..1000 {
                let picked = iter.clone().choose(r).unwrap();
                process(picked);
            }
        }
    "#;
    assert!(parse_and_detect(code).iter().all(|v| v.law != "P01"));
}

#[test]
fn p01_skipped_in_cfg_test_module() {
    let code = r#"
        #[cfg(test)]
        mod tests {
            fn helper() {
                for _ in 0..10 {
                    let s = name.clone();
                    process(s);
                }
            }
        }
    "#;
    assert!(parse_and_detect(code).iter().all(|v| v.law != "P01"));
}

#[test]
fn p01_skipped_when_clone_into_push() {
    let code = r#"
        fn f(items: &[String]) {
            let mut out = vec![];
            for _ in 0..10 {
                out.push(name_string.clone());
            }
        }
    "#;
    assert!(
        parse_and_detect(code).iter().all(|v| v.law != "P01"),
        ".push() is an ownership sink — clone into push should be skipped"
    );
}

#[test]
fn p01_skipped_when_clone_into_entry() {
    let code = r#"
        fn f() {
            let mut map = std::collections::HashMap::new();
            for _ in 0..10 {
                map.entry(name_string.clone());
            }
        }
    "#;
    assert!(
        parse_and_detect(code).iter().all(|v| v.law != "P01"),
        ".entry() is an ownership sink — clone into entry should be skipped"
    );
}

#[test]
fn p01_skipped_when_clone_into_extend() {
    let code = r#"
        fn f() {
            let mut out = vec![];
            for _ in 0..10 {
                out.extend(name_list.clone());
            }
        }
    "#;
    assert!(
        parse_and_detect(code).iter().all(|v| v.law != "P01"),
        ".extend() is an ownership sink — clone into extend should be skipped"
    );
}

#[test]
fn p01_high_confidence_for_known_heap_keyword() {
    let code = r#"
        fn f() {
            for _ in 0..10 {
                let s = name_string.clone();
                process(s);
            }
        }
    "#;
    let violations: Vec<_> = parse_and_detect(code)
        .into_iter()
        .filter(|v| v.law == "P01")
        .collect();
    assert!(!violations.is_empty());
    assert_eq!(violations[0].confidence, Confidence::High);
}

#[test]
fn p01_medium_confidence_for_generic_long_name() {
    let code = r#"
        fn f() {
            for _ in 0..10 {
                let s = cumulative_weight.clone();
                process(s);
            }
        }
    "#;
    let violations: Vec<_> = parse_and_detect(code)
        .into_iter()
        .filter(|v| v.law == "P01")
        .collect();
    assert!(!violations.is_empty());
    assert_eq!(violations[0].confidence, Confidence::Medium);
}

#[test]
fn p01_medium_confidence_for_indexed_access() {
    let code = r#"
        fn f(items: &[String]) {
            for i in 0..10 {
                let s = self.weights[i].clone();
                process(s);
            }
        }
    "#;
    let violations: Vec<_> = parse_and_detect(code)
        .into_iter()
        .filter(|v| v.law == "P01")
        .collect();
    assert!(!violations.is_empty());
    assert_eq!(violations[0].confidence, Confidence::Medium);
}

#[test]
fn p01_high_confidence_for_uppercase_receiver() {
    let code = r#"
        fn f() {
            for _ in 0..10 {
                let s = MyStruct.clone();
                process(s);
            }
        }
    "#;
    let violations: Vec<_> = parse_and_detect(code)
        .into_iter()
        .filter(|v| v.law == "P01")
        .collect();
    assert!(!violations.is_empty());
    assert_eq!(violations[0].confidence, Confidence::High);
}

#[test]
fn looks_heap_owning_identifies_string() {
    assert!(looks_heap_owning("name_string"));
    assert!(looks_heap_owning("SomeStruct"));
    assert!(!looks_heap_owning("i"));
    assert!(!looks_heap_owning("x"));
}

#[test]
fn is_arc_or_rc_detects_variants() {
    assert!(is_arc_or_rc_clone("Arc::clone(&x)"));
    assert!(is_arc_or_rc_clone("Rc::clone(&x)"));
    assert!(is_arc_or_rc_clone("std::sync::Arc::clone(&x)"));
    assert!(!is_arc_or_rc_clone("name.clone()"));
}
