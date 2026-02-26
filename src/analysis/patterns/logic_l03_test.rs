// src/analysis/patterns/logic_l03_test.rs

use super::*;
use tree_sitter::Parser;

fn parse_and_detect(code: &str) -> Vec<Violation> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_rust::language()).unwrap();
    let tree = parser.parse(code, None).unwrap();
    let mut violations = Vec::new();
    detect_l03(code, tree.root_node(), &mut violations);
    violations
}

#[test]
fn l03_flag_index_zero() {
    let code = "fn f(v: &[i32]) -> i32 { v[0] }";
    let violations: Vec<_> = parse_and_detect(code)
        .into_iter()
        .filter(|v| v.law == "L03")
        .collect();
    assert!(!violations.is_empty());
    assert_eq!(violations[0].confidence, Confidence::High);
}

#[test]
fn l03_skip_with_empty_check() {
    let code = "fn f(v: &[i32]) -> i32 { if !v.is_empty() { v[0] } else { 0 } }";
    assert!(parse_and_detect(code).iter().all(|v| v.law != "L03"));
}

#[test]
fn l03_skip_with_len_check() {
    let code = "fn f(v: &[i32]) -> i32 { if v.len() > 0 { v[0] } else { 0 } }";
    assert!(
        parse_and_detect(code).iter().all(|v| v.law != "L03"),
        "v.len() guard must suppress L03"
    );
}

#[test]
fn l03_flag_first_unwrap() {
    let code = "fn f(v: &[i32]) -> i32 { *v.first().unwrap() }";
    assert!(parse_and_detect(code).iter().any(|v| v.law == "L03"));
}

#[test]
fn l03_flag_last_unwrap() {
    let code = "fn f(v: &[i32]) -> i32 { *v.last().unwrap() }";
    assert!(
        parse_and_detect(code).iter().any(|v| v.law == "L03"),
        ".last().unwrap() must be flagged"
    );
}

#[test]
fn l03_no_flag_without_unwrap() {
    let code = "fn f(v: &[i32]) -> Option<&i32> { v.first() }";
    assert!(
        parse_and_detect(code).iter().all(|v| v.law != "L03"),
        ".first() without .unwrap() must not be flagged"
    );
}

#[test]
fn l03_skip_chunks_exact_index() {
    let code = r"
        fn f(data: &[u8]) -> Vec<u16> {
            data.chunks_exact(2)
                .map(|a| u16::from_le_bytes([a[0], a[1]]))
                .collect()
        }
    ";
    assert!(parse_and_detect(code).iter().all(|v| v.law != "L03"));
}

#[test]
fn l03_skip_fixed_array_repeat() {
    let code = r"
        fn f() {
            let mut seed = [0u8; 32];
            seed[0] = 1;
        }
    ";
    assert!(parse_and_detect(code).iter().all(|v| v.law != "L03"));
}

#[test]
fn l03_skip_fixed_array_literal() {
    let code = r"
        fn f() -> i32 {
            let arr = [1, 2, 3];
            arr[0]
        }
    ";
    assert!(parse_and_detect(code).iter().all(|v| v.law != "L03"));
}

#[test]
fn l03_skip_struct_field_array() {
    let code = r"
        struct Rng {
            s: [u32; 4],
        }
        impl Rng {
            fn next(&mut self) -> u32 {
                let res = self.s[0].wrapping_add(self.s[3]);
                res
            }
        }
    ";
    assert!(parse_and_detect(code).iter().all(|v| v.law != "L03"));
}

#[test]
fn l03_skip_typed_param_array() {
    let code = r"
        fn process(buf: [u8; 4]) -> u8 {
            buf[0]
        }
    ";
    assert!(parse_and_detect(code).iter().all(|v| v.law != "L03"));
}

#[test]
fn l03_still_flags_vec_index() {
    let code = r"
        fn f(v: Vec<i32>) -> i32 {
            v[0]
        }
    ";
    assert!(parse_and_detect(code).iter().any(|v| v.law == "L03"));
}

#[test]
fn l03_still_flags_slice_index() {
    let code = "fn f(v: &[i32]) -> i32 { v[0] }";
    assert!(parse_and_detect(code).iter().any(|v| v.law == "L03"));
}

#[test]
fn l03_medium_confidence_for_unfound_variable() {
    let code = r"
        fn f() -> i32 {
            data[0]
        }
    ";
    let violations: Vec<_> = parse_and_detect(code)
        .into_iter()
        .filter(|v| v.law == "L03")
        .collect();
    assert!(!violations.is_empty(), "should flag data[0]");
    assert_eq!(violations[0].confidence, Confidence::Medium);
}

#[test]
fn l03_high_confidence_for_found_vec() {
    let code = r"
        fn f() -> i32 {
            let v = vec![1, 2, 3];
            v[0]
        }
    ";
    let violations: Vec<_> = parse_and_detect(code)
        .into_iter()
        .filter(|v| v.law == "L03")
        .collect();
    assert!(!violations.is_empty());
    assert_eq!(violations[0].confidence, Confidence::High);
}

#[test]
fn l03_medium_confidence_for_self_field() {
    let code = r"
        struct Foo { items: Vec<i32> }
        impl Foo {
            fn first(&self) -> i32 {
                self.items[0]
            }
        }
    ";
    let violations: Vec<_> = parse_and_detect(code)
        .into_iter()
        .filter(|v| v.law == "L03")
        .collect();
    assert!(!violations.is_empty());
    assert_eq!(violations[0].confidence, Confidence::Medium);
}

#[test]
fn l03_high_confidence_for_param_slice() {
    let code = "fn f(v: &[i32]) -> i32 { v[0] }";
    let violations: Vec<_> = parse_and_detect(code)
        .into_iter()
        .filter(|v| v.law == "L03")
        .collect();
    assert!(!violations.is_empty());
    assert_eq!(violations[0].confidence, Confidence::High);
}

#[test]
fn l03_medium_confidence_for_method_return() {
    let code = "fn f() -> i32 { get_data()[0] }";
    let violations: Vec<_> = parse_and_detect(code)
        .into_iter()
        .filter(|v| v.law == "L03")
        .collect();
    assert!(!violations.is_empty(), "should flag method return indexing");
    assert_eq!(
        violations[0].confidence,
        Confidence::Medium,
        "method return receiver should be Medium"
    );
}
