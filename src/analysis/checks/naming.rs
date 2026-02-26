// src/analysis/checks/naming.rs
//! Function naming checks (Law of Complexity).

use tree_sitter::{Query, QueryCursor, QueryMatch};

use crate::types::{Violation, ViolationDetails};

use super::CheckContext;

/// Checks for naming violations (function name word count).
pub fn check_naming(ctx: &CheckContext, query: &Query, out: &mut Vec<Violation>) {
    for pattern in &ctx.config.ignore_naming_on {
        if ctx.filename.contains(pattern) {
            return;
        }
    }

    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(query, ctx.root, ctx.source.as_bytes());

    for m in matches {
        process_match(&m, ctx, out);
    }
}

fn process_match(m: &QueryMatch, ctx: &CheckContext, out: &mut Vec<Violation>) {
    for capture in m.captures {
        if let Ok(name) = capture.node.utf8_text(ctx.source.as_bytes()) {
            check_word_count(name, capture.node, ctx, out);
        }
    }
}

fn check_word_count(
    name: &str,
    node: tree_sitter::Node,
    ctx: &CheckContext,
    out: &mut Vec<Violation>,
) {
    let word_count = count_words(name);
    if word_count <= ctx.config.max_function_words {
        return;
    }
    let owned_name = name.to_string();
    let details = ViolationDetails {
        function_name: Some(owned_name),
        analysis: vec![format!("Name has {word_count} words")],
        suggestion: Some(suggest_shorter_name(name)),
    };
    out.push(Violation::with_details(
        node.start_position().row + 1,
        format!(
            "Function name '{name}' has {word_count} words (Max: {})",
            ctx.config.max_function_words
        ),
        "LAW OF COMPLEXITY",
        details,
    ));
}

fn suggest_shorter_name(name: &str) -> String {
    let words: Vec<&str> = split_name_words(name);
    if words.len() <= 3 {
        return "Consider a more concise name".to_string();
    }
    format!(
        "Consider abbreviating. Core: '{}'",
        words.iter().take(3).copied().collect::<Vec<_>>().join("_")
    )
}

fn split_name_words(name: &str) -> Vec<&str> {
    if name.contains('_') {
        name.split('_').filter(|s| !s.is_empty()).collect()
    } else {
        split_camel_case(name)
    }
}

fn split_camel_case(name: &str) -> Vec<&str> {
    let mut words = Vec::new();
    let mut start = 0;
    let chars: Vec<char> = name.chars().collect();

    for (i, &c) in chars.iter().enumerate().skip(1) {
        if c.is_uppercase() && start < i {
            words.push(&name[start..i]);
            start = i;
        }
    }
    if start < name.len() {
        words.push(&name[start..]);
    }
    words
}

fn count_words(name: &str) -> usize {
    if name.contains('_') {
        name.split('_').filter(|s| !s.is_empty()).count()
    } else {
        count_camel_words(name)
    }
}

fn count_camel_words(name: &str) -> usize {
    let mut count = 0;
    let mut prev_upper = true;
    for c in name.chars() {
        if c.is_uppercase() && !prev_upper {
            count += 1;
        }
        prev_upper = c.is_uppercase();
    }
    count + 1
}
