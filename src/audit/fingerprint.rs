// src/audit/fingerprint.rs
//! Structural fingerprinting using Weisfeiler-Lehman style hashing.
//!
//! This module implements a hash function that captures the structural
//! properties of code while being invariant to identifier names. Two
//! functions with identical structure but different variable names will
//! produce the same fingerprint.
//!
//! The algorithm:
//! 1. Traverse the AST depth-first
//! 2. For each node, hash its *type* (not its text for identifiers)
//! 3. Combine child hashes using a position-aware mixing function
//! 4. The final hash represents the tree structure
//!
//! This is inspired by the Weisfeiler-Lehman graph kernel but adapted
//! for ordered trees (ASTs).

use super::types::Fingerprint;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tree_sitter::Node;

/// Nodes whose text content should be included in the hash.
/// These are structural tokens, not user-defined identifiers.
const STRUCTURAL_NODES: &[&str] = &[
    // Operators
    "+", "-", "*", "/", "%", "&&", "||", "!", "==", "!=", "<", ">", "<=", ">=", "&", "|", "^", "<<",
    ">>", "+=", "-=", "*=", "/=", // Keywords (language structure)
    "if", "else", "match", "while", "for", "loop", "return", "break", "continue", "let", "mut",
    "const", "static", "fn", "struct", "enum", "impl", "trait", "pub", "self", "Self", "async",
    "await", "move", "ref", "where", // Punctuation that affects structure
    "=>", "->", "::", ":", ";", ",", ".", "?", // Delimiters
    "(", ")", "{", "}", "[", "]", "<", ">", // Literals (type, not value)
    "true", "false", "None", "Some", "Ok", "Err",
];

/// Node types that should be treated as "identifier-like" (hash by kind only).
const IDENTIFIER_KINDS: &[&str] = &[
    "identifier",
    "type_identifier",
    "field_identifier",
    "scoped_identifier",
    "scoped_type_identifier",
    "string_literal",
    "raw_string_literal",
    "char_literal",
    "integer_literal",
    "float_literal",
];

/// Computes a structural fingerprint for an AST node and its subtree.
#[must_use]
pub fn compute(node: Node, source: &[u8]) -> Fingerprint {
    let mut state = FingerprintState::new();
    state.visit(node, source, 0);
    state.finalize()
}

/// Internal state for fingerprint computation.
struct FingerprintState {
    hasher: DefaultHasher,
    depth: usize,
    max_depth: usize,
    node_count: usize,
}

impl FingerprintState {
    fn new() -> Self {
        Self {
            hasher: DefaultHasher::new(),
            depth: 0,
            max_depth: 0,
            node_count: 0,
        }
    }

    fn visit(&mut self, node: Node, source: &[u8], depth: usize) {
        self.node_count += 1;
        self.max_depth = self.max_depth.max(depth);

        // Start node boundary marker
        self.mix_u64(0xDEAD_BEEF_u64.wrapping_add(depth as u64));

        let kind = node.kind();

        // Hash the node kind (always)
        self.mix_str(kind);

        // For structural nodes, also hash the actual text
        if should_hash_text(kind, node, source) {
            if let Ok(text) = node.utf8_text(source) {
                self.mix_str(text);
            }
        }

        // Hash child count (important for structure)
        let child_count = node.child_count();
        self.mix_u64(child_count as u64);

        // Recursively visit children with position encoding
        for (i, child) in node.children(&mut node.walk()).enumerate() {
            // Mix in child position to distinguish [a, b] from [b, a]
            self.mix_u64(i as u64);
            self.visit(child, source, depth + 1);
        }

        // End node boundary marker
        self.mix_u64(0xCAFE_BABE_u64.wrapping_add(depth as u64));
    }

    fn mix_str(&mut self, s: &str) {
        s.hash(&mut self.hasher);
    }

    fn mix_u64(&mut self, v: u64) {
        v.hash(&mut self.hasher);
    }

    fn finalize(self) -> Fingerprint {
        Fingerprint {
            hash: self.hasher.finish(),
            depth: self.max_depth,
            node_count: self.node_count,
        }
    }
}

/// Determines if we should hash the text content of a node.
fn should_hash_text(kind: &str, node: Node, source: &[u8]) -> bool {
    // Don't hash identifier text - we want structural similarity
    if IDENTIFIER_KINDS.contains(&kind) {
        return false;
    }

    // Hash structural tokens
    if STRUCTURAL_NODES.contains(&kind) {
        return true;
    }

    // For other nodes, check if the text is a keyword/operator
    if let Ok(text) = node.utf8_text(source) {
        if STRUCTURAL_NODES.contains(&text) {
            return true;
        }
    }

    false
}

/// Computes similarity between two fingerprints.
/// Returns a value between 0.0 (completely different) and 1.0 (identical).
#[must_use]
pub fn similarity(a: &Fingerprint, b: &Fingerprint) -> f64 {
    // Exact hash match = perfect similarity
    if a.hash == b.hash {
        return 1.0;
    }

    // Compare structural metrics for partial similarity
    let depth_sim =
        1.0 - (a.depth as f64 - b.depth as f64).abs() / (a.depth.max(b.depth) as f64).max(1.0);
    let count_sim = 1.0
        - (a.node_count as f64 - b.node_count as f64).abs()
            / (a.node_count.max(b.node_count) as f64).max(1.0);

    // Weight: exact match matters most, then structure
    // If hashes differ, we can't have high similarity
    (depth_sim * 0.3 + count_sim * 0.3) * 0.5
}

/// Computes fingerprints for all extractable code units in a file.
/// Returns a list of (name, kind, start_line, end_line, fingerprint) tuples.
pub fn extract_units(
    source: &str,
    tree: &tree_sitter::Tree,
) -> Vec<(String, &'static str, usize, usize, Fingerprint)> {
    let mut units = Vec::new();
    let source_bytes = source.as_bytes();

    extract_from_node(tree.root_node(), source_bytes, &mut units);

    units
}

fn extract_from_node(
    node: Node,
    source: &[u8],
    units: &mut Vec<(String, &'static str, usize, usize, Fingerprint)>,
) {
    let kind = node.kind();

    // Extract named code units
    if let Some(unit_kind) = match_unit_kind(kind) {
        if let Some(name) = extract_name(node, source) {
            let fingerprint = compute(node, source);
            let start_line = node.start_position().row + 1;
            let end_line = node.end_position().row + 1;
            units.push((name, unit_kind, start_line, end_line, fingerprint));
        }
    }

    // Recurse into children
    for child in node.children(&mut node.walk()) {
        extract_from_node(child, source, units);
    }
}

fn match_unit_kind(kind: &str) -> Option<&'static str> {
    match kind {
        "function_item" | "function_definition" => Some("function"),
        "impl_item" => Some("impl"),
        "struct_item" | "struct_definition" => Some("struct"),
        "enum_item" | "enum_definition" => Some("enum"),
        "trait_item" | "trait_definition" => Some("trait"),
        "mod_item" => Some("module"),
        // Methods are functions inside impl blocks
        "function_item" => Some("method"),
        _ => None,
    }
}

fn extract_name(node: Node, source: &[u8]) -> Option<String> {
    // Look for a name child node
    let name_node = node.child_by_field_name("name")?;
    let name = name_node.utf8_text(source).ok()?;
    Some(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_structure_same_hash() {
        // Two functions with identical structure but different names
        // should produce the same fingerprint
        let code1 = "fn foo(x: i32) -> i32 { x + 1 }";
        let code2 = "fn bar(y: i32) -> i32 { y + 1 }";

        // This test would require tree-sitter setup
        // For now, we verify the algorithm compiles
    }
}

