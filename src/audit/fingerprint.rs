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
///
/// Delegates to `similarity_core` to keep this file small.
#[must_use]
pub fn similarity(a: &Fingerprint, b: &Fingerprint) -> f64 {
    crate::audit::similarity_core::calculate_similarity(a, b)
}