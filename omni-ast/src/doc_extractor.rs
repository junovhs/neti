//! Extracts documentation comments from source files.

use crate::doc_filter;
use std::path::Path;

pub fn extract_doc_comment(content: &str) -> Option<String> {
    extract_module_doc(content).or_else(|| extract_first_item_doc(content))
}

pub fn extract_doc_comment_for_file(file: &Path, content: &str) -> Option<String> {
    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "rs" => extract_doc_comment(content),
        "ts" | "tsx" | "js" | "jsx" | "mjs" => crate::language::javascript::extract_doc(content),
        "py" => crate::language::python::extract_doc(content),
        "go" => crate::language::go::extract_doc(content),
        "c" | "cc" | "cpp" | "cxx" | "h" | "hh" | "hpp" | "hxx" => {
            crate::language::cpp::extract_doc(content)
        }
        _ => None,
    }
}

fn extract_module_doc(content: &str) -> Option<String> {
    let mut doc_lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("//!") {
            doc_lines.push(rest.trim());
        } else if !trimmed.is_empty() && !trimmed.starts_with("//") {
            break;
        }
    }

    (!doc_lines.is_empty()).then(|| collapse_doc_lines(&doc_lines))
}

fn extract_first_item_doc(content: &str) -> Option<String> {
    let mut doc_lines = Vec::new();
    let mut in_doc_block = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix("///") {
            if !rest.starts_with('/') {
                doc_lines.push(rest.trim());
                in_doc_block = true;
            }
        } else if in_doc_block {
            break;
        }
    }

    if doc_lines.is_empty() {
        return None;
    }

    let doc = collapse_doc_lines(&doc_lines);
    (!doc_filter::looks_like_item_doc(&doc)).then_some(doc)
}

pub fn collapse_doc_lines(lines: &[&str]) -> String {
    let cleaned: Vec<&str> = lines
        .iter()
        .map(|line| {
            let l = line.trim();
            if l.starts_with('#') {
                l.trim_start_matches('#').trim()
            } else {
                l
            }
        })
        .collect();

    let joined = cleaned.join(" ");
    let trimmed = joined.trim();

    if let Some(idx) = trimmed.find(". ") {
        return format!("{}.", &trimmed[..idx]);
    }

    if trimmed.ends_with('.') {
        trimmed.to_owned()
    } else {
        format!("{trimmed}.")
    }
}
