// src/analysis/v2/patterns/concurrency.rs
//! Concurrency pattern detection: C03, C04

use crate::types::Violation;
use tree_sitter::Node;

pub use super::concurrency_lock::detect_c03;
pub use super::concurrency_sync::detect_c04;

/// Detects concurrency-related violations in Rust code.
#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut violations = Vec::new();
    violations.extend(detect_c03(source, root));
    violations.extend(detect_c04(source, root));
    violations
}