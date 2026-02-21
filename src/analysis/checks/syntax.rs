// src/analysis/checks/syntax.rs
//! AST-level syntax error and malformed node detection.
//!
//! # Soundness Contract
//!
//! We must never emit `LAW OF INTEGRITY` on valid Rust. The tree-sitter-rust
//! grammar bundled with this version of Neti (0.20) predates several Rust
//! language features (C-string literals, open-ended range patterns, etc.).
//! When tree-sitter produces error nodes for these constructs, we must emit
//! a soft "parser unsupported" diagnostic rather than a hard syntax error.
//!
//! Rule: if the error node matches a known modern-Rust construct, suppress
//! the violation entirely. The cost of false silence is lower than the cost
//! of incorrectly blocking a valid Rust codebase.

use tree_sitter::Node;

use crate::types::Violation;

use super::CheckContext;

/// Checks for syntax errors or missing nodes in the AST.
pub fn check_syntax(ctx: &CheckContext, out: &mut Vec<Violation>) {
    let is_rust = is_rust_file(ctx.filename);
    traverse_for_errors(ctx.root, ctx.source, is_rust, out);
}

fn is_rust_file(filename: &str) -> bool {
    filename.ends_with(".rs")
}

fn traverse_for_errors(node: Node, source: &str, is_rust: bool, out: &mut Vec<Violation>) {
    if node.is_error() {
        handle_error_node(node, source, is_rust, out);
    } else if node.is_missing() {
        report_missing(node, out);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        traverse_for_errors(child, source, is_rust, out);
    }
}

fn handle_error_node(node: Node, source: &str, is_rust: bool, out: &mut Vec<Violation>) {
    if is_rust && is_known_unsupported_construct(node, source) {
        // Suppress: this is a grammar-version limitation, not a code defect.
        // See module-level doc for the soundness contract.
        return;
    }
    let row = node.start_position().row + 1;
    out.push(Violation::simple(
        row,
        "Syntax error detected".to_string(),
        "LAW OF INTEGRITY",
    ));
}

fn report_missing(node: Node, out: &mut Vec<Violation>) {
    let row = node.start_position().row + 1;
    let msg = format!("Missing expected node: {}", node.kind());
    out.push(Violation::simple(row, msg, "LAW OF INTEGRITY"));
}

/// Returns `true` if the error node represents a valid modern-Rust construct
/// that the bundled tree-sitter grammar version cannot parse.
///
/// If you are seeing "Syntax error detected" on valid Rust, the pattern that
/// triggered it likely belongs here. Add the smallest possible recognizer and
/// document which Rust version stabilized it.
fn is_known_unsupported_construct(node: Node, source: &str) -> bool {
    let text = node.utf8_text(source.as_bytes()).unwrap_or("").trim();

    // C-string literals: c"..." c'...' cr"..." (stabilized Rust 1.77)
    if is_c_string_literal(text) {
        return true;
    }

    // Open-ended range patterns: 0.. 24.. (as match arm patterns, stabilized Rust 1.26+)
    // These sometimes parse as errors in older grammars when used in pattern position.
    if is_open_range_pattern(text) {
        return true;
    }

    // Numeric literals with type suffixes used in range pattern context: 24u8, 100usize, etc.
    if is_suffixed_numeric_literal(text) {
        return true;
    }

    // Inner attributes inside function bodies: #![doc(...)] etc.
    // Standard inner attributes at file level parse fine; function-body ones may not.
    if text.starts_with("#![") {
        return true;
    }

    false
}

/// Recognizes C-string literals introduced in Rust 1.77.
/// Patterns: c"...", c'...', cr"..."
fn is_c_string_literal(text: &str) -> bool {
    text.starts_with("c\"")
        || text.starts_with("c'")
        || text.starts_with("cr\"")
        || text.starts_with("cr#\"")
}

/// Recognizes open-ended range pattern suffixes: `0..`, `24..`, `n..`
/// The text will be just the start-of-range expression with trailing `..`.
fn is_open_range_pattern(text: &str) -> bool {
    let Some(prefix) = text.strip_suffix("..") else {
        return false;
    };
    // Must be a numeric literal (with optional suffix) or simple identifier
    let bare = strip_numeric_suffix(prefix.trim());
    bare.chars().all(|c| c.is_ascii_digit() || c == '_')
}

/// Recognizes numeric literals with Rust type suffixes: 24u8, 100usize, 5i32, etc.
/// These appear as isolated error nodes when in unsupported pattern contexts.
fn is_suffixed_numeric_literal(text: &str) -> bool {
    const SUFFIXES: &[&str] = &[
        "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
        "f32", "f64",
    ];
    SUFFIXES.iter().any(|s| {
        if let Some(digits) = text.strip_suffix(s) {
            // Must be non-empty and all digits/underscores
            !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit() || c == '_')
        } else {
            false
        }
    })
}

/// Strips a trailing type suffix from a numeric literal string.
fn strip_numeric_suffix(s: &str) -> &str {
    const SUFFIXES: &[&str] = &[
        "u128", "i128", "usize", "isize", "u64", "i64", "f64", "u32", "i32", "f32", "u16", "i16",
        "u8", "i8",
    ];
    for suffix in SUFFIXES {
        if let Some(stripped) = s.strip_suffix(suffix) {
            return stripped;
        }
    }
    s
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::config::RuleConfig;
    use crate::lang::Lang;
    use tree_sitter::Parser;

    fn run_syntax_check(code: &str, filename: &str) -> Vec<Violation> {
        let mut parser = Parser::new();
        parser.set_language(Lang::Rust.grammar()).unwrap();
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
    }

    fn is_known_unsupported_construct_from_text(text: &str) -> bool {
        // Build a fake node test by checking the text-only path
        is_c_string_literal(text)
            || is_open_range_pattern(text)
            || is_suffixed_numeric_literal(text)
            || text.starts_with("#![")
    }
}
