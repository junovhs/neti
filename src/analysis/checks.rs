// src/analysis/checks.rs
//! AST-based complexity and style checks.

mod banned;
mod complexity;
mod naming;
mod syntax;

use tree_sitter::Node;

use crate::config::RuleConfig;

pub use banned::check_banned;
pub use complexity::check_metrics;
pub use naming::check_naming;
pub use syntax::check_syntax;

/// Context for running checks on a single file.
pub struct CheckContext<'a> {
    pub root: Node<'a>,
    pub source: &'a str,
    pub filename: &'a str,
    pub config: &'a RuleConfig,
}
