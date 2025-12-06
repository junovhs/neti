// src/skeleton.rs
use crate::lang::Lang;
use std::path::Path;
use tree_sitter::{Language, Parser, Query, QueryCursor};

/// Reduces code to its structural skeleton (signatures only).
///
/// # Arguments
/// * `path` - The file path (used for language detection).
/// * `content` - The full source code.
///
/// # Returns
/// The skeletonized code, or the original content if language is unsupported.
#[must_use]
pub fn clean(path: &Path, content: &str) -> String {
    let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
        return content.to_string();
    };

    let Some(lang) = Lang::from_ext(ext) else {
        return content.to_string();
    };

    let query_str = lang.q_skeleton();
    let replacement = lang.skeleton_replacement();
    let grammar = lang.grammar();
    let query = compile_query(grammar, query_str);

    apply_skeleton(content, grammar, &query, replacement)
}

fn apply_skeleton(source: &str, lang: Language, query: &Query, replacement: &str) -> String {
    let mut parser = Parser::new();
    if parser.set_language(lang).is_err() {
        return source.to_string();
    }

    let Some(tree) = parser.parse(source, None) else {
        return source.to_string();
    };

    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(query, tree.root_node(), source.as_bytes());

    let mut ranges = Vec::new();
    for m in matches {
        for capture in m.captures {
            ranges.push(capture.node.byte_range());
        }
    }

    // Filter nested ranges: if Range A contains Range B, we only want A.
    // We want the outermost bodies to be replaced.
    let root_ranges = filter_nested_ranges(ranges);

    replace_ranges(source, &root_ranges, replacement)
}

fn filter_nested_ranges(mut ranges: Vec<std::ops::Range<usize>>) -> Vec<std::ops::Range<usize>> {
    // Sort by start position
    ranges.sort_by_key(|r| r.start);

    let mut result: Vec<std::ops::Range<usize>> = Vec::new();
    let mut i = 0;
    while i < ranges.len() {
        let current = &ranges[i];
        
        // Check if this range is contained by any already added range.
        if let Some(last) = result.last() {
            if last.end >= current.end {
                // Current is inside Last. Skip Current.
                i += 1;
                continue;
            }
        }
        
        result.push(current.clone());
        i += 1;
    }
    result
}

fn replace_ranges(source: &str, ranges: &[std::ops::Range<usize>], replacement: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let mut last_pos = 0;

    for range in ranges {
        // Push text before the body
        if range.start > last_pos {
            result.push_str(&source[last_pos..range.start]);
        }
        
        // Push replacement
        result.push_str(replacement);
        
        // Advance
        last_pos = range.end;
    }

    // Append trailing content
    if last_pos < source.len() {
        result.push_str(&source[last_pos..]);
    }

    result
}

fn compile_query(lang: Language, pattern: &str) -> Query {
    match Query::new(lang, pattern) {
        Ok(q) => q,
        Err(e) => panic!("Invalid skeleton query: {e}"),
    }
}