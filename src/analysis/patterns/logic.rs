// src/analysis/patterns/logic.rs
//! Logic boundary patterns: L02 (off-by-one risk), L03 (unchecked index).
//!
//! See `logic_helpers` for shared utilities and `logic_proof` for
//! fixed-size array proof logic.

#[path = "logic_l02.rs"]
mod logic_l02;
#[path = "logic_l03.rs"]
mod logic_l03;

use crate::types::Violation;
use tree_sitter::Node;

// Re-export so `super::logic_helpers` inside the child modules resolves correctly
pub(super) use super::logic_helpers;
pub(super) use super::logic_proof;

#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut out = Vec::new();
    logic_l02::detect_l02(source, root, &mut out);
    logic_l03::detect_l03(source, root, &mut out);
    out
}
