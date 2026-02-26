// src/analysis/checks/banned.rs
//! Banned construct checks (Law of Paranoia).

use std::path::Path;
use tree_sitter::{Query, QueryCursor, QueryMatch};

use crate::types::{Violation, ViolationDetails};

use super::CheckContext;

/// Checks for banned constructs (`.unwrap()` and `.expect()` calls).
pub fn check_banned(ctx: &CheckContext, banned_query: &Query, out: &mut Vec<Violation>) {
    if is_test_file(ctx.filename) {
        return;
    }

    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(banned_query, ctx.root, ctx.source.as_bytes());

    for m in matches {
        process_match(&m, ctx, out);
    }
}

fn is_test_file(filename: &str) -> bool {
    let path = Path::new(filename);
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| name.contains("test") || name.contains("spec"))
}

fn process_match(m: &QueryMatch, ctx: &CheckContext, out: &mut Vec<Violation>) {
    for capture in m.captures {
        if let Ok(text) = capture.node.utf8_text(ctx.source.as_bytes()) {
            let row = capture.node.start_position().row + 1;
            let kind = capture.node.kind();
            if is_banned_call(kind, text) {
                add_violation(text, row, out);
            }
        }
    }
}

fn is_banned_call(kind: &str, text: &str) -> bool {
    kind == "identifier" && (text == "unwrap" || text == "expect")
}

fn add_violation(text: &str, row: usize, out: &mut Vec<Violation>) {
    let suggestion = if text == "unwrap" {
        "Use `?` operator or `ok_or_else()` to propagate errors"
    } else {
        "Use `?` with `context()` or `with_context()` for better errors"
    };

    let details = ViolationDetails {
        function_name: None,
        analysis: vec![format!("Found `.{text}()` call")],
        suggestion: Some(suggestion.to_string()),
    };

    out.push(Violation::with_details(
        row,
        format!("Banned: '.{text}()' found. Use ? or proper error handling."),
        "LAW OF PARANOIA",
        details,
    ));
}
