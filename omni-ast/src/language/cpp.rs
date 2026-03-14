//! C and C++ import, doc, and export extraction.

use crate::doc_extractor::collapse_doc_lines;
use crate::doc_filter;
use regex::Regex;
use std::collections::BTreeSet;
use std::path::Path;

#[path = "cpp_includes.rs"]
mod includes;

const HEADER_EXTENSIONS: &[&str] = &["h", "hh", "hpp", "hxx"];
const MFC_HANDLER_PREFIXES: &[&str] = &[
    "On",
    "PreCreate",
    "DoDataExchange",
    "AssertValid",
    "Dump",
    "WindowProc",
];
const TYPE_RE: &str = r"(?m)^\s*(?:class|struct|enum(?:\s+class)?|namespace)\s+([A-Za-z_]\w*)";
const FN_RE: &str = r"(?:inline\s+|constexpr\s+|extern\s+|virtual\s+|friend\s+|template\s*<[^>]+>\s*)*(?:[A-Za-z_]\w*|[A-Za-z_]\w*::[A-Za-z_]\w*|auto|const|unsigned|signed|short|long|void|bool|char|float|double)[^\n;{}=]*?\b([A-Za-z_]\w*)\s*\(";
const CPP_KEYWORDS: &[&str] = &[
    "if", "for", "while", "switch", "catch", "return", "delete", "new", "sizeof",
];

pub use includes::{extract_imports, is_build_only_path};

pub fn extract_import_strings(content: &str) -> Vec<String> {
    let Ok(re) = Regex::new(r#"(?m)^\s*#\s*include\s*["<]([^">]+)[">]"#) else {
        return Vec::new();
    };
    re.captures_iter(content)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

pub fn extract_doc(content: &str) -> Option<String> {
    let mut slash_lines: Vec<&str> = Vec::new();
    let mut iter = content.lines().peekable();
    while let Some(line) = iter.peek().copied() {
        let trimmed = line.trim();
        if should_skip_prelude_line(trimmed, slash_lines.is_empty()) {
            iter.next();
            continue;
        }
        if trimmed.starts_with("/**") {
            return extract_block_doc(&mut iter);
        }
        if let Some(doc_line) = slash_doc_line(trimmed) {
            slash_lines.push(doc_line);
            iter.next();
            continue;
        }
        break;
    }
    collapse_cpp_doc(&slash_lines)
}

fn should_skip_prelude_line(trimmed: &str, slash_lines_empty: bool) -> bool {
    if trimmed.is_empty() {
        return slash_lines_empty;
    }
    slash_lines_empty
        && (trimmed.starts_with("#pragma once")
            || trimmed.starts_with("#ifndef")
            || trimmed.starts_with("#define"))
}

fn slash_doc_line(trimmed: &str) -> Option<&str> {
    if let Some(rest) = trimmed.strip_prefix("///") {
        return Some(rest.trim());
    }
    let rest = trimmed.strip_prefix("//")?;
    let body = rest.trim();
    (!body.is_empty()).then_some(body)
}

pub fn extract_exports(content: &str, source_path: &Path) -> BTreeSet<String> {
    let mut exports = collect_matches(content, TYPE_RE);
    exports.extend(collect_function_exports(
        content,
        is_header_path(source_path),
    ));
    exports.retain(|name| !CPP_KEYWORDS.contains(&name.as_str()));
    exports
}

pub fn primary_symbol(content: &str, source_path: &Path) -> Option<String> {
    collect_matches(content, TYPE_RE)
        .into_iter()
        .find(|name| !CPP_KEYWORDS.contains(&name.as_str()))
        .or_else(|| {
            collect_function_exports(content, is_header_path(source_path))
                .into_iter()
                .find(|name| !CPP_KEYWORDS.contains(&name.as_str()))
        })
}

pub fn is_header_path(path: &Path) -> bool {
    HEADER_EXTENSIONS.contains(&path.extension().and_then(|e| e.to_str()).unwrap_or(""))
}

fn collect_matches(content: &str, pattern: &str) -> BTreeSet<String> {
    let Ok(re) = Regex::new(pattern) else {
        return BTreeSet::new();
    };
    re.captures_iter(content)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_owned()))
        .collect()
}

fn collect_function_exports(content: &str, is_header: bool) -> BTreeSet<String> {
    let Ok(re) = Regex::new(FN_RE) else {
        return BTreeSet::new();
    };
    let mut names: BTreeSet<&str> = BTreeSet::new();
    let mut brace_depth = 0i32;

    for line in content.lines() {
        let trimmed = line.trim();
        if brace_depth == 0 && !trimmed.starts_with("static ") && !trimmed.starts_with('#') {
            if let Some(name) = re
                .captures(trimmed)
                .and_then(|cap| cap.get(1).map(|m| m.as_str()))
            {
                if is_semantic_function_line(trimmed, name, is_header) {
                    names.insert(name);
                }
            }
        }
        brace_depth += trimmed.chars().filter(|&c| c == '{').count() as i32;
        brace_depth -= trimmed.chars().filter(|&c| c == '}').count() as i32;
        brace_depth = brace_depth.max(0);
    }

    names.into_iter().map(str::to_owned).collect()
}

fn is_semantic_function_line(line: &str, name: &str, is_header: bool) -> bool {
    !line.contains("::")
        && !MFC_HANDLER_PREFIXES
            .iter()
            .any(|prefix| name.starts_with(prefix))
        && (is_header || line.contains('{'))
}

fn extract_block_doc<'a, I>(iter: &mut std::iter::Peekable<I>) -> Option<String>
where
    I: Iterator<Item = &'a str>,
{
    let mut lines = Vec::new();
    let mut first = true;
    for line in iter.by_ref() {
        let trimmed = line.trim();
        let body = if first {
            first = false;
            trimmed.trim_start_matches("/**").trim()
        } else {
            trimmed
        };
        if let Some((before_close, _)) = body.split_once("*/") {
            let cleaned = before_close.trim().trim_start_matches('*').trim();
            if !cleaned.is_empty() {
                lines.push(cleaned);
            }
            break;
        }
        let cleaned = body.trim_start_matches('*').trim();
        if !cleaned.is_empty() {
            lines.push(cleaned);
        }
    }
    collapse_cpp_doc(&lines)
}

fn collapse_cpp_doc(lines: &[&str]) -> Option<String> {
    if lines.is_empty() {
        return None;
    }
    let collapsed = collapse_doc_lines(lines);
    (!doc_filter::looks_like_item_doc(&collapsed)).then_some(collapsed)
}
