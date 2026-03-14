// src/analysis/patterns/performance.rs
//! Performance anti-patterns: P01, P02, P04, P06
//!
//! # Escalation Philosophy
//!
//! P01/P02 must only fire when we can make a reasonable argument that the
//! allocation is *material*. Blanket "clone in loop" flags generate lint spam
//! and train developers to ignore Neti. The goal is signal, not volume.

#[path = "performance_p01.rs"]
mod performance_p01;
#[path = "performance_p02.rs"]
mod performance_p02;
#[path = "performance_p04p06.rs"]
mod performance_p04p06;

use std::path::Path;
use tree_sitter::Node;

use crate::types::Violation;
use omni_ast::{semantics_for, Concept, LangSemantics, SemanticContext, SemanticLanguage};

#[must_use]
pub fn detect(source: &str, root: Option<Node>, path: &Path) -> Vec<Violation> {
    if should_skip(path) {
        return Vec::new();
    }

    let Some(language) = path
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(SemanticLanguage::from_ext)
    else {
        return Vec::new();
    };

    match (language, root) {
        (SemanticLanguage::Rust, Some(root)) => {
            let mut out = Vec::new();
            detect_loops(source, root, &mut out);
            out
        }
        _ => detect_shared_semantics(source, path, language),
    }
}

fn should_skip(path: &Path) -> bool {
    let s = path.to_string_lossy();
    s.contains("/cli/")
        || s.contains("/ui/")
        || s.contains("/tui/")
        || s.contains("reporting")
        || s.contains("messages")
        || s.contains("analysis/")
        || s.contains("audit/")
        || s.contains("pack/")
        || s.contains("signatures/")
        || s.ends_with("main.rs")
}

fn detect_shared_semantics(source: &str, path: &Path, language: SemanticLanguage) -> Vec<Violation> {
    let semantics = semantics_for(language);
    let context = SemanticContext::from_source(source).with_path(path);

    if semantics.is_test_context(&context) {
        return Vec::new();
    }

    let detects_nested_lookup =
        semantics.has_concept(Concept::Loop, &context) && semantics.has_concept(Concept::Lookup, &context);

    if !detects_nested_lookup {
        return Vec::new();
    }

    vec![performance_p04p06::shared_p06_violation(
        first_lookup_line(source, language),
        "Shared semantics identified looped lookup in a non-Rust file.".into(),
    )]
}

fn detect_loops(source: &str, root: Node, out: &mut Vec<Violation>) {
    use super::get_capture_node;
    use super::performance_test_ctx::is_test_context;
    use tree_sitter::{Query, QueryCursor};

    let q = r"
        (for_expression pattern: _ @pat body: (block) @body) @loop
        (while_expression body: (block) @body) @loop
        (loop_expression body: (block) @body) @loop
    ";
    let Ok(query) = Query::new(&tree_sitter_rust::LANGUAGE.into(), q) else {
        return;
    };
    let language = SemanticLanguage::Rust;
    let idx_pat = query.capture_index_for_name("pat");
    let idx_body = query.capture_index_for_name("body");

    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, root, source.as_bytes()) {
        let loop_var = extract_loop_var(source, &m, idx_pat);

        let Some(body) = get_capture_node(&m, idx_body) else {
            continue;
        };

        let in_test = is_test_context(source, body, language);

        if !in_test {
            performance_p01::check_p01(source, body, loop_var.as_deref(), language, out);
        }
        performance_p02::check_p02(source, body, loop_var.as_deref(), out);
        performance_p04p06::check_p04(body, out);
        if !in_test {
            performance_p04p06::check_p06(source, body, language, out);
        }
    }
}

fn extract_loop_var(
    source: &str,
    m: &tree_sitter::QueryMatch,
    idx_pat: Option<u32>,
) -> Option<String> {
    use super::get_capture_node;
    let node = get_capture_node(m, idx_pat)?;
    let text = node.utf8_text(source.as_bytes()).ok()?;
    Some(
        text.split([',', '('])
            .next()
            .unwrap_or(text)
            .trim()
            .to_string(),
    )
}

fn first_lookup_line(source: &str, language: SemanticLanguage) -> usize {
    let needles = match language {
        SemanticLanguage::Rust => &[".find(", ".position(", ".contains(", ".get("][..],
        SemanticLanguage::Python => &[" in ", ".index(", ".get(", ".count("][..],
        SemanticLanguage::JavaScript | SemanticLanguage::TypeScript => {
            &[".find(", ".findIndex(", ".includes(", ".indexOf(", ".get(", ".has("][..]
        }
        SemanticLanguage::Go => &["contains(", "map["][..],
        SemanticLanguage::Cpp => &[".find(", ".contains(", "std::find("][..],
        SemanticLanguage::Swift => &[".contains(", ".firstIndex(", ".first(where:"][..],
    };

    source
        .lines()
        .position(|line| needles.iter().any(|needle| line.contains(needle)))
        .map_or(1, |idx| idx + 1)
}
