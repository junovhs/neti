// src/analysis/patterns/idiomatic.rs
//! Idiomatic patterns: I01, I02.
//!
//! I01: manual `From` impls that could use derive_more.
//! I02: duplicate match arm bodies that could use `A | B => body`.

#[path = "idiomatic_i01.rs"]
mod idiomatic_i01;
#[path = "idiomatic_i02.rs"]
mod idiomatic_i02;

use crate::types::Violation;
use tree_sitter::Node;

#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut out = Vec::new();
    idiomatic_i01::detect_i01(source, root, &mut out);
    idiomatic_i02::detect_i02(source, root, &mut out);
    out
}
