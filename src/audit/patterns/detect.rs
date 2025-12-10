use super::{CustomPattern, PatternMatch};
use crate::lang::Lang;
use std::path::Path;
use tree_sitter::{Query, QueryCursor};

/// Detects patterns in a file.
#[must_use]
pub fn detect_in_file(
    source: &str,
    file: &Path,
    tree: &tree_sitter::Tree,
    lang: tree_sitter::Language,
) -> Vec<PatternMatch> {
    let mut matches = Vec::new();
    let source_bytes = source.as_bytes();

    // Determine language from file extension to pick correct queries
    let ext = file.extension().and_then(|s| s.to_str()).unwrap_or("");
    let detected_lang = Lang::from_ext(ext);

    for template in super::registry::PATTERNS {
        // Select query based on language
        let query_str = match detected_lang {
            Some(Lang::Python) => template.python_query,
            // Default to Rust/Generic for others, or skip if None
            _ => Some(template.rust_query),
        };

        let Some(q_str) = query_str else {
            continue;
        };

        let Ok(query) = Query::new(lang, q_str) else {
            continue;
        };

        let mut cursor = QueryCursor::new();
        let query_matches = cursor.matches(&query, tree.root_node(), source_bytes);

        for qm in query_matches {
            if let Some(capture) = qm.captures.last() {
                let node = capture.node;
                let start_line = node.start_position().row + 1;
                let end_line = node.end_position().row + 1;

                let matched_text = node
                    .utf8_text(source_bytes)
                    .unwrap_or("")
                    .chars()
                    .take(100)
                    .collect::<String>();

                matches.push(PatternMatch {
                    pattern_name: template.name.to_string(),
                    file: file.to_path_buf(),
                    start_line,
                    end_line,
                    matched_text,
                });
            }
        }
    }

    matches
}

/// Detects a custom pattern in a file.
#[must_use]
pub fn detect_custom(
    pattern: &CustomPattern,
    source: &str,
    file: &Path,
    tree: &tree_sitter::Tree,
    lang: tree_sitter::Language,
) -> Vec<PatternMatch> {
    let Ok(query) = Query::new(lang, &pattern.query) else {
        return Vec::new();
    };

    let mut matches = Vec::new();
    let source_bytes = source.as_bytes();
    let mut cursor = QueryCursor::new();

    for qm in cursor.matches(&query, tree.root_node(), source_bytes) {
        if let Some(capture) = qm.captures.last() {
            let node = capture.node;
            matches.push(PatternMatch {
                pattern_name: pattern.name.clone(),
                file: file.to_path_buf(),
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
                matched_text: node
                    .utf8_text(source_bytes)
                    .unwrap_or("")
                    .chars()
                    .take(100)
                    .collect(),
            });
        }
    }

    matches
}