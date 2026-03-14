//! JavaScript/TypeScript import extraction and path resolution.

mod monorepo;

use crate::doc_extractor::collapse_doc_lines;
use crate::types::DepKind;
use monorepo::{collect_package_roots, resolve_bare};
use std::collections::HashSet;
use std::path::Path;

pub fn extract_import_strings(content: &str) -> Vec<String> {
    find_all_specifiers(content)
        .into_iter()
        .map(str::to_string)
        .collect()
}

pub fn extract_imports(
    content: &str,
    source_path: &str,
    known_paths: &HashSet<&str>,
) -> Vec<(String, DepKind)> {
    let base_dir = Path::new(source_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));

    let pkg_roots = collect_package_roots(known_paths);

    let mut deps = Vec::new();

    for spec in find_all_specifiers(content) {
        let resolved = if is_relative(spec) {
            resolve_path(base_dir, spec, known_paths)
        } else {
            resolve_bare(spec, known_paths, &pkg_roots)
        };
        if let Some(path) = resolved {
            deps.push((path, DepKind::Import));
        }
    }

    deps
}

fn is_relative(spec: &str) -> bool {
    spec.starts_with('.') || spec.starts_with('/')
}

fn find_all_specifiers(content: &str) -> Vec<&str> {
    let mut results = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        ["from ", "require(", "import(", "import "]
            .iter()
            .for_each(|starter| {
                trimmed.match_indices(starter).for_each(|(pos, _)| {
                    let after_start = pos + starter.len();
                    if let Some(spec) = extract_quoted_specifier(trimmed, after_start) {
                        results.push(spec);
                    }
                });
            });
    }

    results
}

fn extract_quoted_specifier(line: &str, start: usize) -> Option<&str> {
    let rest = line.get(start..)?.trim();

    let (quote, after_quote) = if let Some(s) = rest.strip_prefix('\'') {
        ('\'', s)
    } else if let Some(s) = rest.strip_prefix('"') {
        ('"', s)
    } else {
        return None;
    };

    let (spec, _) = after_quote.split_once(quote)?;

    if spec.is_empty() || spec.starts_with("node:") {
        return None;
    }

    Some(spec)
}

fn resolve_path(base: &Path, relative: &str, known_paths: &HashSet<&str>) -> Option<String> {
    let rel = relative.split(['?', '#']).next().unwrap_or(relative);
    let joined = base.join(rel);

    if joined.extension().is_some() {
        let candidate = normalize_rel_path(&joined);
        return known_paths
            .contains(candidate.as_str())
            .then_some(candidate);
    }

    const EXTS: &[&str] = &["ts", "tsx", "js", "jsx", "mjs", "css"];

    for ext in EXTS {
        let mut p = base.join(rel);
        p.set_extension(ext);
        let candidate = normalize_rel_path(&p);
        if known_paths.contains(candidate.as_str()) {
            return Some(candidate);
        }
    }

    for ext in EXTS {
        let mut p = base.join(rel).join("index");
        p.set_extension(ext);
        let candidate = normalize_rel_path(&p);
        if known_paths.contains(candidate.as_str()) {
            return Some(candidate);
        }
    }

    None
}

fn normalize_rel_path(path: &Path) -> String {
    let raw = path.to_string_lossy();
    let mut parts: Vec<&str> = Vec::new();

    for seg in raw.split(['/', '\\']) {
        match seg {
            "" | "." => {}
            ".." => {
                let _ = parts.pop();
            }
            _ => parts.push(seg),
        }
    }

    parts.join("/")
}

pub fn extract_doc(content: &str) -> Option<String> {
    let mut lines = content.lines();
    let mut collected: Vec<String> = Vec::new();
    let mut inside_block = false;

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        if inside_block {
            return collect_jsdoc_body(trimmed, &mut lines, &mut collected);
        }

        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with("/*") && !trimmed.starts_with("/**") {
            skip_block_comment(trimmed, &mut lines);
            continue;
        }

        if !trimmed.starts_with("/**") {
            return None;
        }

        let after = trimmed.strip_prefix("/**").unwrap_or("");
        if let Some((before_close, _)) = after.split_once("*/") {
            let body = before_close.trim();
            if let Some(clean) = clean_jsdoc_line(body) {
                collected.push(clean);
            }
            return collapse_cleaned_doc(&collected);
        }

        if let Some(clean) = clean_jsdoc_line(after) {
            collected.push(clean);
        }
        inside_block = true;
    }

    collapse_cleaned_doc(&collected)
}

fn collect_jsdoc_body(
    first_trimmed: &str,
    lines: &mut std::str::Lines<'_>,
    collected: &mut Vec<String>,
) -> Option<String> {
    if let Some(before_close) = first_trimmed.find("*/") {
        let body = first_trimmed.get(..before_close).unwrap_or("");
        if let Some(clean) = clean_jsdoc_line(body) {
            collected.push(clean);
        }
        return collapse_cleaned_doc(collected);
    }
    if let Some(clean) = clean_jsdoc_line(first_trimmed) {
        collected.push(clean);
    }

    for line in lines {
        let trimmed = line.trim();
        if let Some((before_close_str, _)) = trimmed.split_once("*/") {
            let block = before_close_str.trim();
            if let Some(clean) = clean_jsdoc_line(block) {
                collected.push(clean);
            }
            return collapse_cleaned_doc(collected);
        }
        if let Some(clean) = clean_jsdoc_line(trimmed) {
            collected.push(clean);
        }
    }

    collapse_cleaned_doc(collected)
}

fn skip_block_comment(first_trimmed: &str, lines: &mut std::str::Lines<'_>) {
    if first_trimmed.contains("*/") {
        return;
    }
    for line in lines.by_ref() {
        if line.contains("*/") {
            return;
        }
    }
}

fn clean_jsdoc_line(line: &str) -> Option<String> {
    let mut trimmed = line.trim();
    trimmed = trimmed.strip_prefix('*').unwrap_or(trimmed).trim();

    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with('@') {
        return None;
    }

    Some(trimmed.to_owned())
}

fn collapse_cleaned_doc(lines: &[String]) -> Option<String> {
    let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    let collapsed = collapse_doc_lines(&refs);
    (collapsed.trim() != ".").then_some(collapsed)
}
