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

/// Heap-type keywords used for High confidence classification.
pub(super) const HEAP_KEYWORDS: &[&str] = &[
    "string",
    "vec",
    "map",
    "set",
    "box",
    "rc",
    "bufwriter",
    "bytes",
    "buffer",
];

/// Heuristic keywords that suggest heap ownership but are less certain.
pub(super) const HEURISTIC_KEYWORDS: &[&str] =
    &["name", "text", "data", "list", "array", "items", "cache"];

#[must_use]
pub fn detect(source: &str, root: Node, path: &Path) -> Vec<Violation> {
    if should_skip(path) {
        return Vec::new();
    }
    let mut out = Vec::new();
    detect_loops(source, root, &mut out);
    out
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
    let idx_pat = query.capture_index_for_name("pat");
    let idx_body = query.capture_index_for_name("body");

    let mut cursor = QueryCursor::new();
    for m in cursor.matches(&query, root, source.as_bytes()) {
        let loop_var = get_capture_node(&m, idx_pat)
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .map(|s| s.split([',', '(']).next().unwrap_or(s).trim().to_string());

        let Some(body) = get_capture_node(&m, idx_body) else {
            continue;
        };

        let in_test = is_test_context(source, body);

        if !in_test {
            performance_p01::check_p01(source, body, loop_var.as_deref(), out);
        }
        performance_p02::check_p02(source, body, loop_var.as_deref(), out);
        performance_p04p06::check_p04(body, out);
        if !in_test {
            performance_p04p06::check_p06(source, body, out);
        }
    }
}
