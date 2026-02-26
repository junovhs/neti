// src/analysis/checks/syntax_test.rs

use super::*;
use crate::config::RuleConfig;
use crate::lang::Lang;
use tree_sitter::Parser;

fn run_syntax_check(code: &str, filename: &str) -> Vec<Violation> {
    let mut parser = Parser::new();
    parser.set_language(&Lang::Rust.grammar()).unwrap();
    let tree = parser.parse(code, None).unwrap();
    let config = RuleConfig::default();
    let ctx = CheckContext {
        root: tree.root_node(),
        source: code,
        filename,
        config: &config,
    };
    let mut violations = Vec::new();
    check_syntax(&ctx, &mut violations);
    violations
}

#[test]
fn test_rust_error() {
    let code = "fn main() { let x = ; }";
    assert!(!run_syntax_check(code, "test.rs").is_empty());
}

#[test]
fn test_valid_rust() {
    let code = "fn main() { let x = 5; }";
    assert!(run_syntax_check(code, "test.rs").is_empty());
}

#[test]
fn test_c_string_literal_recognized() {
    assert!(is_c_string_literal(r#"c"hello""#));
    assert!(is_c_string_literal("c'h'"));
    assert!(is_c_string_literal(r#"cr"raw""#));
    assert!(!is_c_string_literal(r#""normal""#));
}

#[test]
fn test_open_range_pattern_recognized() {
    assert!(is_open_range_pattern("0.."));
    assert!(is_open_range_pattern("24.."));
    assert!(is_open_range_pattern("100.."));
    assert!(!is_open_range_pattern("0..=5"));
    assert!(!is_open_range_pattern("..10"));
    assert!(!is_open_range_pattern("abc"));
}

#[test]
fn test_suffixed_literal_recognized() {
    assert!(is_suffixed_numeric_literal("24u8"));
    assert!(is_suffixed_numeric_literal("100usize"));
    assert!(is_suffixed_numeric_literal("5i32"));
    assert!(!is_suffixed_numeric_literal("abc"));
    assert!(!is_suffixed_numeric_literal("24"));
}

#[test]
fn test_inner_attribute_recognized() {
    assert!(is_known_unsupported_construct_from_text("#![doc(hidden)]"));
    assert!(is_known_unsupported_construct_from_text(
        "#![doc(html_logo_url = \"https://example.com\")]"
    ));
}

#[test]
fn test_inner_attribute_content_suppressed() {
    let code = r#"
#![doc(
html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk.png",
html_favicon_url = "https://www.rust-lang.org/favicon.ico"
)]

fn main() {}
"#;
    let violations = run_syntax_check(code, "test.rs");
    assert!(
        violations.is_empty(),
        "Inner attribute #![doc(...)] content must not produce syntax errors, got: {violations:?}"
    );
}

fn is_known_unsupported_construct_from_text(text: &str) -> bool {
    is_c_string_literal(text)
        || is_open_range_pattern(text)
        || is_suffixed_numeric_literal(text)
        || text.starts_with("#![")
}
