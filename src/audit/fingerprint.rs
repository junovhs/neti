// src/audit/fingerprint.rs
//! Structural fingerprinting with control flow graph awareness.
//!
//! Implements semantic fingerprinting detecting:
//! - Level 3: Identical algorithms with different variable names
//! - Level 4: Equivalent control flow with different syntax

use super::cfg::{
    normalize, BRANCH_NODES, CFG_NODES, EXIT_NODES, IDENTIFIER_KINDS, LOOP_NODES, STRUCTURAL_TOKENS,
};
use super::types::Fingerprint;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tree_sitter::Node;

/// Computes a structural fingerprint for an AST node.
#[must_use]
pub fn compute(node: Node, source: &[u8]) -> Fingerprint {
    let mut state = FingerprintState::new();
    state.visit(node, source, 0);
    state.finalize()
}

struct FingerprintState {
    struct_hasher: DefaultHasher,
    cfg_hasher: DefaultHasher,
    max_depth: usize,
    node_count: usize,
    branch_count: usize,
    loop_count: usize,
    exit_count: usize,
}

impl FingerprintState {
    fn new() -> Self {
        Self {
            struct_hasher: DefaultHasher::new(),
            cfg_hasher: DefaultHasher::new(),
            max_depth: 0,
            node_count: 0,
            branch_count: 0,
            loop_count: 0,
            exit_count: 0,
        }
    }

    fn visit(&mut self, node: Node, source: &[u8], depth: usize) {
        self.node_count += 1;
        self.max_depth = self.max_depth.max(depth);

        let kind = node.kind();
        self.count_cfg_elements(kind);
        self.hash_node(kind, node, source, depth);
        self.visit_children(node, source, depth);
    }

    fn count_cfg_elements(&mut self, kind: &str) {
        if BRANCH_NODES.contains(&kind) {
            self.branch_count += 1;
        }
        if LOOP_NODES.contains(&kind) {
            self.loop_count += 1;
        }
        if EXIT_NODES.contains(&kind) {
            self.exit_count += 1;
        }
    }

    fn hash_node(&mut self, kind: &str, node: Node, source: &[u8], depth: usize) {
        0xDEAD_BEEF_u64.wrapping_add(depth as u64).hash(&mut self.struct_hasher);
        kind.hash(&mut self.struct_hasher);

        if should_hash_text(kind, node, source) {
            if let Ok(text) = node.utf8_text(source) {
                text.hash(&mut self.struct_hasher);
            }
        }

        if CFG_NODES.contains(&kind) {
            depth.hash(&mut self.cfg_hasher);
            normalize(kind).hash(&mut self.cfg_hasher);
        }
    }

    fn visit_children(&mut self, node: Node, source: &[u8], depth: usize) {
        (node.child_count() as u64).hash(&mut self.struct_hasher);
        for (i, child) in node.children(&mut node.walk()).enumerate() {
            (i as u64).hash(&mut self.struct_hasher);
            self.visit(child, source, depth + 1);
        }
    }

    fn finalize(self) -> Fingerprint {
        Fingerprint {
            hash: self.struct_hasher.finish(),
            cfg_hash: self.cfg_hasher.finish(),
            depth: self.max_depth,
            node_count: self.node_count,
            branch_count: self.branch_count,
            loop_count: self.loop_count,
            exit_count: self.exit_count,
        }
    }
}

fn should_hash_text(kind: &str, node: Node, source: &[u8]) -> bool {
    if IDENTIFIER_KINDS.contains(&kind) {
        return false;
    }
    if STRUCTURAL_TOKENS.contains(&kind) {
        return true;
    }
    node.utf8_text(source)
        .is_ok_and(|text| STRUCTURAL_TOKENS.contains(&text))
}

/// Computes similarity between two fingerprints.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn similarity(a: &Fingerprint, b: &Fingerprint) -> f64 {
    if a.hash == b.hash {
        return 1.0;
    }
    if a.cfg_hash == b.cfg_hash {
        return 0.85 + (structural_similarity(a, b) * 0.15);
    }
    if cfg_metrics_match(a, b) {
        return 0.95;
    }
    cfg_similarity(a, b) * 0.6 + structural_similarity(a, b) * 0.4
}

fn cfg_metrics_match(a: &Fingerprint, b: &Fingerprint) -> bool {
    a.branch_count == b.branch_count
        && a.loop_count == b.loop_count
        && a.exit_count == b.exit_count
}

#[allow(clippy::cast_precision_loss)]
fn cfg_similarity(a: &Fingerprint, b: &Fingerprint) -> f64 {
    let branch = metric_sim(a.branch_count, b.branch_count);
    let loops = metric_sim(a.loop_count, b.loop_count);
    let exits = metric_sim(a.exit_count, b.exit_count);
    branch * 0.5 + loops * 0.3 + exits * 0.2
}

#[allow(clippy::cast_precision_loss)]
fn structural_similarity(a: &Fingerprint, b: &Fingerprint) -> f64 {
    metric_sim(a.depth, b.depth) * 0.3 + metric_sim(a.node_count, b.node_count) * 0.7
}

#[allow(clippy::cast_precision_loss)]
fn metric_sim(a: usize, b: usize) -> f64 {
    let max = a.max(b) as f64;
    if max == 0.0 { 1.0 } else { 1.0 - (a as f64 - b as f64).abs() / max }
}

/// Extracts fingerprinted units from a parsed file.
#[must_use]
pub fn extract_units(
    source: &str,
    tree: &tree_sitter::Tree,
) -> Vec<(String, &'static str, usize, usize, Fingerprint)> {
    let mut units = Vec::new();
    extract_from_node(tree.root_node(), source.as_bytes(), &mut units);
    units
}

fn extract_from_node(
    node: Node,
    source: &[u8],
    units: &mut Vec<(String, &'static str, usize, usize, Fingerprint)>,
) {
    if let Some(unit_kind) = match_unit_kind(node.kind()) {
        if let Some(name) = extract_name(node, source) {
            let fp = compute(node, source);
            let start = node.start_position().row + 1;
            let end = node.end_position().row + 1;
            units.push((name, unit_kind, start, end, fp));
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
    node.child_by_field_name("name")?.utf8_text(source).ok().map(String::from)
}