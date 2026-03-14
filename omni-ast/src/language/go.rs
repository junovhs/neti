use crate::doc_extractor::collapse_doc_lines;
use crate::doc_filter;
use crate::types::DepKind;
use std::collections::HashSet;

pub fn extract_import_strings(content: &str) -> Vec<String> {
    let mut imports = Vec::new();
    let mut in_block = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("import (") {
            in_block = true;
            continue;
        }
        if in_block {
            if trimmed == ")" {
                in_block = false;
                continue;
            }
            if let Some(path) = extract_quoted_string(trimmed) {
                imports.push(path);
            }
        } else if let Some(rest) = trimmed.strip_prefix("import ") {
            if let Some(path) = extract_quoted_string(rest) {
                imports.push(path);
            }
        }
    }

    imports
}

pub fn extract_doc(content: &str) -> Option<String> {
    let mut collected: Vec<&str> = Vec::new();
    let mut result = None;
    let mut in_block = false;

    for line in content.lines() {
        if result.is_some() {
            break;
        }
        let trimmed = line.trim();

        if in_block {
            if trimmed.contains("*/") {
                in_block = false;
            }
            continue;
        }

        if trimmed.is_empty() {
            collected.clear();
            continue;
        }

        if trimmed.starts_with("/*") {
            if !trimmed.contains("*/") {
                in_block = true;
            }
            collected.clear();
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("//") {
            let body = rest.trim();
            if is_go_directive_or_copyright(body) {
                continue;
            }
            if !body.is_empty() {
                collected.push(body);
            }
        } else if trimmed.starts_with("package ") || trimmed.starts_with("package\t") {
            result = Some(process_package_doc(&collected));
        } else {
            result = Some(None);
        }
    }

    result.flatten()
}

fn is_go_directive_or_copyright(body: &str) -> bool {
    body.starts_with("+build")
        || body.starts_with("go:build")
        || body.starts_with("go:generate")
        || body.to_lowercase().contains("copyright")
}

fn process_package_doc(collected: &[&str]) -> Option<String> {
    if collected.is_empty() {
        return None;
    }
    let collapsed = collapse_doc_lines(collected);
    if !doc_filter::looks_like_item_doc(&collapsed) {
        Some(collapsed)
    } else {
        None
    }
}

pub fn extract_imports(
    content: &str,
    module_name: &Option<String>,
    known_paths: &HashSet<&str>,
) -> Vec<(String, DepKind)> {
    let Some(module) = module_name.as_deref() else {
        return Vec::new();
    };

    let imported_dirs = collect_imported_dirs(content, module);
    if imported_dirs.is_empty() {
        return Vec::new();
    }

    let is_test_file = file_is_test(content);
    resolve_known_paths(known_paths, &imported_dirs, is_test_file)
}

fn collect_imported_dirs(content: &str, module: &str) -> HashSet<String> {
    let mut dirs = HashSet::new();
    let mut in_block = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("import (") {
            in_block = true;
            continue;
        }
        if in_block {
            if trimmed == ")" {
                in_block = false;
                continue;
            }
            if let Some(dir) = quoted_local_dir(trimmed, module) {
                dirs.insert(dir);
            }
        } else if let Some(rest) = trimmed.strip_prefix("import ") {
            if let Some(dir) = quoted_local_dir(rest, module) {
                dirs.insert(dir);
            }
        }
    }

    dirs
}

fn file_is_test(content: &str) -> bool {
    content.contains("_test.go")
        || content
            .lines()
            .any(|l| l.trim().starts_with("package ") && l.contains("_test"))
}

fn resolve_known_paths(
    known_paths: &HashSet<&str>,
    imported_dirs: &HashSet<String>,
    is_test_file: bool,
) -> Vec<(String, DepKind)> {
    let mut results = Vec::new();
    for kp in known_paths {
        if !is_test_file && kp.ends_with("_test.go") {
            continue;
        }
        let dir = dir_of(kp);
        if imported_dirs.contains(dir) {
            results.push((kp.to_string(), DepKind::Import));
        }
    }
    results
}

fn dir_of(path: &str) -> &str {
    match path.rfind('/') {
        Some(idx) => &path[..idx],
        None => "",
    }
}

fn quoted_local_dir(s: &str, module: &str) -> Option<String> {
    let path = extract_quoted_string(s)?;
    get_local_dir(&path, module)
}

fn extract_quoted_string(s: &str) -> Option<String> {
    let s = s.trim();
    if s.is_empty() || s.starts_with("//") {
        return None;
    }
    if let Some((_, rest)) = s.split_once('"') {
        if let Some((path, _)) = rest.split_once('"') {
            return Some(path.to_string());
        }
    }
    None
}

fn get_local_dir(path: &str, module: &str) -> Option<String> {
    if let Some(rest) = path.strip_prefix(module) {
        if rest.is_empty() || rest == "/" {
            return Some(String::new());
        }
        if let Some(stripped) = rest.strip_prefix('/') {
            return Some(stripped.to_string());
        }
    }
    None
}
