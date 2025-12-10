// src/audit/fingerprint.rs
//! Structural fingerprinting using Weisfeiler-Lehman style hashing.
//!
//! This module implements a hash function that captures the structural
//! properties of code while being invariant to identifier names. Two
//! functions with identical structure but different variable names will
//! produce the same fingerprint.

use super::types::Fingerprint;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tree_sitter::Node;

/// Nodes whose text content should be included in the hash.
const STRUCTURAL_NODES: &[&str] = &[
    "+", "-", "*", "/", "%", "&&", "||", "!", "==", "!=", "<", ">", "<=", ">=", "&", "|", "^",
    "<<", ">>", "+=", "-=", "*=", "/=", "if", "else", "match", "while", "for", "loop", "return",
    "break", "continue", "let", "mut", "const", "static", "fn", "struct", "enum", "impl", "trait",
    "pub", "self", "Self", "async", "await", "move", "ref", "where", "=>", "->", "::", ":", ";",
    ",", ".", "?", "(", ")", "{", "}", "[", "]", "true", "false", "None", "Some", "Ok", "Err",
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

struct FingerprintState {
    hasher: DefaultHasher,
    max_depth: usize,
    node_count: usize,
}

impl FingerprintState {
    fn new() -> Self {
        Self {
            hasher: DefaultHasher::new(),
            max_depth: 0,
            node_count: 0,
        }
    }

    fn visit(&mut self, node: Node, source: &[u8], depth: usize) {
        self.node_count += 1;
        self.max_depth = self.max_depth.max(depth);

        self.mix_u64(0xDEAD_BEEF_u64.wrapping_add(depth as u64));

        let kind = node.kind();
        self.mix_str(kind);

        if should_hash_text(kind, node, source) {
            if let Ok(text) = node.utf8_text(source) {
                self.mix_str(text);
            }
        }

        let child_count = node.child_count();
        self.mix_u64(child_count as u64);

        for (i, child) in node.children(&mut node.walk()).enumerate() {
            self.mix_u64(i as u64);
            self.visit(child, source, depth + 1);
        }

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

fn should_hash_text(kind: &str, node: Node, source: &[u8]) -> bool {
    if IDENTIFIER_KINDS.contains(&kind) {
        return false;
    }

    if STRUCTURAL_NODES.contains(&kind) {
        return true;
    }

    if let Ok(text) = node.utf8_text(source) {
        if STRUCTURAL_NODES.contains(&text) {
            return true;
        }
    }

    false
}

/// Computes similarity between two fingerprints.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn similarity(a: &Fingerprint, b: &Fingerprint) -> f64 {
    if a.hash == b.hash {
        return 1.0;
    }

    let max_depth = a.depth.max(b.depth) as f64;
    let depth_sim = 1.0 - (a.depth as f64 - b.depth as f64).abs() / max_depth.max(1.0);

    let max_count = a.node_count.max(b.node_count) as f64;
    let count_sim = 1.0 - (a.node_count as f64 - b.node_count as f64).abs() / max_count.max(1.0);

    // FIXED: Was capped at 0.3 for non-exact matches ((x*0.3 + y*0.3) * 0.5).
    // Now returns proper 0.0-1.0 range for near-duplicates.
    depth_sim * 0.5 + count_sim * 0.5
}

/// Computes fingerprints for all extractable code units in a file.
#[must_use]
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

    if let Some(unit_kind) = match_unit_kind(kind) {
        if let Some(name) = extract_name(node, source) {
            let fingerprint = compute(node, source);
            let start_line = node.start_position().row + 1;
            let end_line = node.end_position().row + 1;
            units.push((name, unit_kind, start_line, end_line, fingerprint));
        }
    }

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
        _ => None,
    }
}

fn extract_name(node: Node, source: &[u8]) -> Option<String> {
    let name_node = node.child_by_field_name("name")?;
    let name = name_node.utf8_text(source).ok()?;
    Some(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity_math() {
        let fp1 = Fingerprint {
            hash: 1,
            depth: 10,
            node_count: 50,
        };
        // Identical metrics, different hash -> should be 1.0 struct sim
        let fp2 = Fingerprint {
            hash: 2,
            depth: 10,
            node_count: 50,
        };
        assert!((similarity(&fp1, &fp2) - 1.0).abs() < f64::EPSILON);

        // Half depth -> 0.5 depth_sim * 0.5 weight = 0.25
        // Same count -> 1.0 count_sim * 0.5 weight = 0.5
        // Total = 0.75
        let fp3 = Fingerprint {
            hash: 3,
            depth: 5,
            node_count: 50,
        };
        assert!((similarity(&fp1, &fp3) - 0.75).abs() < f64::EPSILON);
    }
}