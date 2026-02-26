// src/mutate/discovery.rs
//! Discovers mutation points in source files using tree-sitter.
//!
//! Walks the AST to find operators, booleans, and other mutable constructs.

use crate::lang::Lang;
use crate::mutate::mutations::{get_mutation, MutationPoint};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tree_sitter::{Node, Parser};

/// Discovers all mutation points in a single file.
///
/// # Errors
/// Returns error if file cannot be read or parsed.
pub fn discover_mutations(path: &Path) -> Result<Vec<MutationPoint>> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let Some(lang) = Lang::from_ext(ext) else {
        return Ok(Vec::new()); // Skip unsupported files
    };

    let source =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;

    let mut parser = Parser::new();
    parser
        .set_language(&lang.grammar())
        .context("Failed to set parser language")?;

    let tree = parser
        .parse(&source, None)
        .context("Failed to parse file")?;

    let mut points = Vec::new();
    collect_mutations(tree.root_node(), &source, path, &mut points);

    Ok(points)
}

/// Recursively walks the AST collecting mutation points.
fn collect_mutations(node: Node, source: &str, path: &Path, out: &mut Vec<MutationPoint>) {
    // Check if this node is a mutable operator/literal
    if let Some(point) = check_node(node, source, path) {
        out.push(point);
    }

    // Recurse into children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_mutations(child, source, path, out);
    }
}

/// Checks if a node represents a mutable point.
fn check_node(node: Node, source: &str, path: &Path) -> Option<MutationPoint> {
    let kind = node.kind();

    // Target operator nodes and boolean literals
    if !is_mutable_kind(kind) {
        return None;
    }

    let text = node.utf8_text(source.as_bytes()).ok()?;
    let (mutated, mutation_kind) = get_mutation(text)?;

    Some(MutationPoint {
        file: path.to_path_buf(),
        line: node.start_position().row + 1,
        column: node.start_position().column + 1,
        byte_start: node.start_byte(),
        byte_end: node.end_byte(),
        original: text.to_string(),
        mutated: mutated.to_string(),
        kind: mutation_kind,
    })
}

/// Returns true if this AST node kind might contain a mutable operator.
fn is_mutable_kind(kind: &str) -> bool {
    matches!(
        kind,
        // Operators (varies by language)
        "==" | "!=" | "<" | ">" | "<=" | ">="
            | "&&" | "||"
            | "+" | "-" | "*" | "/"
            | "and" | "or"
            // Boolean literals
            | "true" | "false"
            | "boolean" | "True" | "False"
            // Generic operator captures from tree-sitter
            | "binary_operator"
            | "comparison_operator"
            | "boolean_operator"
    )
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_discover_finds_operators() {
        let source = "fn test() { x == 1 && y > 2 }";
        let mut parser = Parser::new();
        parser.set_language(&Lang::Rust.grammar()).ok();
        let tree = parser.parse(source, None).expect("parse");

        let mut points = Vec::new();
        let path = PathBuf::from("test.rs");
        collect_mutations(tree.root_node(), source, &path, &mut points);

        // Should find at least == and >
        assert!(!points.is_empty());
    }
}
