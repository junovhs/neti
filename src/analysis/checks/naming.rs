// src/analysis/checks/naming.rs
//! Function naming checks (Law of Complexity).

use omni_ast::swum::{expand_identifier, split_identifier};
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
    let words = split_identifier(name);
    let word_count = words.len();
    if word_count <= ctx.config.max_function_words {
        return;
    }
    let owned_name = name.to_string();
    let details = ViolationDetails {
        function_name: Some(owned_name),
        analysis: vec![
            format!("Name has {word_count} words"),
            format!("SWUM reads it as: {}", expand_identifier(name)),
        ],
        suggestion: Some(suggest_shorter_name(name, &words)),
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

fn suggest_shorter_name(name: &str, words: &[String]) -> String {
    if words.len() <= 3 {
        return format!(
            "Consider a more concise name. SWUM reads it as: {}",
            expand_identifier(name)
        );
    }
    let core = words
        .iter()
        .take(3)
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("_");
    format!(
        "Consider abbreviating to '{core}'. SWUM reads it as: {}",
        expand_identifier(name)
    )
}

#[cfg(test)]
fn count_words(name: &str) -> usize {
    split_identifier(name).len()
}

#[cfg(test)]
mod tests {
    use super::{count_words, suggest_shorter_name};
    use omni_ast::swum::split_identifier;

    #[test]
    fn count_words_uses_swum_for_acronyms() {
        assert_eq!(count_words("parseHTTPResponseBody"), 4);
    }

    #[test]
    fn count_words_uses_swum_for_snake_case() {
        assert_eq!(count_words("render_user_profile_card"), 4);
    }

    #[test]
    fn suggestion_reuses_swum_summary() {
        let words = split_identifier("parseHTTPResponseBody");
        let suggestion = suggest_shorter_name("parseHTTPResponseBody", &words);

        assert!(suggestion.contains("parse_http_response"));
        assert!(suggestion.contains("Parses http response body."));
    }
}
