// src/analysis/v2/patterns/mod.rs
//! AST pattern detection for violations.

pub mod state;
pub mod concurrency;
pub mod concurrency_lock;
pub mod concurrency_sync;
pub mod performance;
pub mod security;
pub mod semantic;
pub mod resource;
pub mod db_patterns;
pub mod idiomatic;
pub mod logic;

use crate::types::Violation;
use crate::lang::Lang;
use std::path::Path;
use tree_sitter::Parser;

/// Runs all pattern detections on a file.
#[must_use]
pub fn detect_all(path: &Path, source: &str) -> Vec<Violation> {
    let Some(lang) = get_rust_lang(path) else { return Vec::new() };
    let Some(tree) = parse_source(source, lang) else { return Vec::new() };

    let root = tree.root_node();
    let mut out = Vec::new();

    out.extend(state::detect(source, root));
    out.extend(concurrency::detect(source, root));
    out.extend(performance::detect(source, root, path));
    out.extend(db_patterns::detect(source, root));
    out.extend(security::detect(source, root));
    out.extend(semantic::detect(source, root));
    out.extend(resource::detect(source, root));
    out.extend(idiomatic::detect(source, root));
    out.extend(logic::detect(source, root));

    out
}

fn get_rust_lang(path: &Path) -> Option<Lang> {
    let ext = path.extension()?.to_str()?;
    match Lang::from_ext(ext) {
        Some(Lang::Rust) => Some(Lang::Rust),
        _ => None,
    }
}

fn parse_source(source: &str, lang: Lang) -> Option<tree_sitter::Tree> {
    let mut parser = Parser::new();
    parser.set_language(lang.grammar()).ok()?;
    parser.parse(source, None)
}

/// Helper to get a node from a capture by index.
/// Centralized to reduce duplication and fix lifetime issues.
#[must_use]
pub fn get_capture_node<'a>(
    m: &tree_sitter::QueryMatch<'_, 'a>,
    idx: Option<u32>,
) -> Option<tree_sitter::Node<'a>> {
    let i = idx?;
    for c in m.captures {
        if c.index == i {
            return Some(c.node);
        }
    }
    None
}