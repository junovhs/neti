// src/signatures/docs.rs
//! Doc comment extraction for holographic signatures.

use std::ops::Range;

/// Prefixes that indicate a doc comment or attribute we want to preserve.
const DOC_PREFIXES: &[&str] = &[
    "///",        // Rust doc comment
    "//!",        // Rust inner doc
    "#[doc",      // Rust doc attribute
    "#![doc",     // Rust inner doc attribute
    "/**",        // JSDoc start
    "*/",         // JSDoc end
    "#[must_use", // Common attribute
    "#[derive",   // Common attribute
];

/// Expands all ranges to include doc comments above each item.
pub fn expand_ranges_for_docs(content: &str, ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    let line_starts = build_line_starts(content);
    ranges
        .into_iter()
        .map(|r| expand_for_docs(content, r, &line_starts))
        .collect()
}

/// Builds a table of byte offsets where each line starts.
fn build_line_starts(source: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(source.match_indices('\n').map(|(i, _)| i + 1))
        .collect()
}

/// Expands a range backward to include doc comments above the item.
fn expand_for_docs(source: &str, range: Range<usize>, line_starts: &[usize]) -> Range<usize> {
    let start_line = line_starts
        .iter()
        .rposition(|&offset| offset <= range.start)
        .unwrap_or(0);

    let mut first_line = start_line;

    for line_idx in (0..start_line).rev() {
        let line = get_line(source, line_starts, line_idx);
        let trimmed = line.trim();

        if is_doc_comment(trimmed) {
            first_line = line_idx;
        } else if !trimmed.is_empty() {
            break;
        }
        // Empty lines: just keep scanning upward
    }

    let new_start = line_starts.get(first_line).copied().unwrap_or(range.start);
    new_start..range.end
}

/// Gets the text of a specific line by index.
fn get_line<'a>(source: &'a str, line_starts: &[usize], line_idx: usize) -> &'a str {
    let start = line_starts.get(line_idx).copied().unwrap_or(0);
    let end = line_starts
        .get(line_idx + 1)
        .copied()
        .unwrap_or(source.len());
    &source[start..end]
}

/// Checks if a line is a doc comment or attribute we want to preserve.
fn is_doc_comment(trimmed: &str) -> bool {
    if DOC_PREFIXES.iter().any(|p| trimmed.starts_with(p)) {
        return true;
    }
    // JSDoc continuation line (starts with *)
    trimmed.starts_with('*')
}