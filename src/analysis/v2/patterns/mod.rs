// src/analysis/v2/patterns/mod.rs
//! AST pattern detection for state and concurrency violations.

pub mod state;
pub mod concurrency;
pub mod concurrency_lock;
pub mod concurrency_sync;
pub mod idiomatic;

use crate::types::Violation;
use crate::lang::Lang;
use std::path::Path;
use tree_sitter::Parser;

/// Runs all pattern detections on a file and returns violations.
#[must_use]
pub fn detect_all(path: &Path, source: &str) -> Vec<Violation> {
    let Some(lang) = get_rust_lang(path) else {
        return Vec::new();
    };

    let Some(tree) = parse_source(source, lang) else {
        return Vec::new();
    };

    let root = tree.root_node();
    let mut violations = Vec::new();

    violations.extend(state::detect(source, root));
    violations.extend(concurrency::detect(source, root));
    violations.extend(idiomatic::detect(source, root));

    violations
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