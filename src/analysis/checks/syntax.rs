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
//! the violation entirely.

use tree_sitter::Node;

use crate::types::Violation;

use super::CheckContext;

#[cfg(test)]
#[path = "syntax_test.rs"]
mod tests;

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
fn is_known_unsupported_construct(node: Node, source: &str) -> bool {
    let text = node.utf8_text(source.as_bytes()).unwrap_or("").trim();

    if is_c_string_literal(text) {
        return true;
    }
    if is_open_range_pattern(text) {
        return true;
    }
    if is_suffixed_numeric_literal(text) {
        return true;
    }
    if text.starts_with("#![") {
        return true;
    }
    if is_inside_inner_attribute(node, source) {
        return true;
    }

    false
}

/// Returns `true` if the error node is inside an inner attribute (`#![...]`).
fn is_inside_inner_attribute(node: Node, source: &str) -> bool {
    let mut cur = node;
    for _ in 0..10 {
        let Some(p) = cur.parent() else { break };

        if p.kind() == "attribute_item" || p.kind() == "inner_attribute_item" {
            return true;
        }

        let parent_text = p.utf8_text(source.as_bytes()).unwrap_or("");
        if parent_text.trim_start().starts_with("#![") {
            return true;
        }

        if matches!(p.kind(), "token_tree" | "meta_arguments") {
            if let Some(gp) = p.parent() {
                let gp_text = gp.utf8_text(source.as_bytes()).unwrap_or("");
                if gp_text.trim_start().starts_with("#![")
                    || gp.kind() == "attribute_item"
                    || gp.kind() == "inner_attribute_item"
                {
                    return true;
                }
            }
        }

        if matches!(
            p.kind(),
            "function_item"
                | "struct_item"
                | "enum_item"
                | "impl_item"
                | "mod_item"
                | "source_file"
        ) {
            break;
        }

        cur = p;
    }
    false
}

fn is_c_string_literal(text: &str) -> bool {
    text.starts_with("c\"")
        || text.starts_with("c'")
        || text.starts_with("cr\"")
        || text.starts_with("cr#\"")
}

fn is_open_range_pattern(text: &str) -> bool {
    let Some(prefix) = text.strip_suffix("..") else {
        return false;
    };
    let bare = strip_numeric_suffix(prefix.trim());
    bare.chars().all(|c| c.is_ascii_digit() || c == '_')
}

fn is_suffixed_numeric_literal(text: &str) -> bool {
    const SUFFIXES: &[&str] = &[
        "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
        "f32", "f64",
    ];
    SUFFIXES.iter().any(|s| {
        if let Some(digits) = text.strip_suffix(s) {
            !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit() || c == '_')
        } else {
            false
        }
    })
}

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
