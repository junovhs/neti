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
use omni_ast::{semantics_for, LangSemantics, SemanticContext, SemanticLanguage};
use std::path::Path;
use tree_sitter::Node;

// Re-export so `super::logic_helpers` inside the child modules resolves correctly
pub(super) use super::logic_helpers;
pub(super) use super::logic_proof;

#[must_use]
pub fn detect(source: &str, root: Option<Node>, path: &Path) -> Vec<Violation> {
    let Some(language) = path
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(SemanticLanguage::from_ext)
    else {
        return Vec::new();
    };

    if language != SemanticLanguage::Rust {
        return detect_shared_semantics(source, language);
    }

    let Some(root) = root else {
        return Vec::new();
    };

    let mut out = Vec::new();
    logic_l02::detect_l02(source, root, &mut out);
    logic_l03::detect_l03(source, root, &mut out);
    out
}

fn detect_shared_semantics(source: &str, language: SemanticLanguage) -> Vec<Violation> {
    let semantics = semantics_for(language);
    let mut out = Vec::new();

    for (line_idx, line) in source.lines().enumerate() {
        if !semantics.has_length_boundary_risk(&SemanticContext::from_source(line)) {
            continue;
        }

        out.push(Violation::simple(
            line_idx + 1,
            "Boundary uses collection length with an inclusive operator — possible off-by-one"
                .into(),
            "L02",
        ));
    }

    out
}
