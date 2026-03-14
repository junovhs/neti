//! Python import extraction and dependency analysis.

use crate::types::DepKind;
use regex::Regex;
use std::collections::HashSet;
use std::path::Path;

pub fn extract_import_strings(content: &str) -> Vec<String> {
    let Some(re) = Regex::new(r"(?m)^(?:import\s+(\w+)|from\s+(\w[\w.]*)\s+import)").ok() else {
        return Vec::new();
    };

    re.captures_iter(content)
        .filter_map(|cap| {
            cap.get(1)
                .or_else(|| cap.get(2))
                .map(|m| m.as_str().to_string())
        })
        .collect()
}

pub fn extract_imports(
    content: &str,
    source_path: &str,
    known_paths: &HashSet<&str>,
) -> Vec<(String, DepKind)> {
    let mut deps = Vec::new();

    let pkg_dir = Path::new(source_path)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();

    extract_relative_imports(content, &pkg_dir, known_paths, &mut deps);
    extract_absolute_imports(content, known_paths, &mut deps);

    deps
}

fn extract_relative_imports(
    content: &str,
    pkg_dir: &str,
    known_paths: &HashSet<&str>,
    deps: &mut Vec<(String, DepKind)>,
) {
    let Some(re) = Regex::new(r"from\s+(\.+)(\w*)\s+import\s+(\w+(?:\s*,\s*\w+)*)").ok() else {
        return;
    };

    let new_deps: Vec<(String, DepKind)> = re
        .captures_iter(content)
        .flat_map(|cap| {
            let dots = cap.get(1).map_or("", |m| m.as_str());
            let module = cap.get(2).map_or("", |m| m.as_str());
            let up = dots.len().saturating_sub(1);
            let base = go_up(pkg_dir, up);
            resolve_relative_cap(
                module,
                cap.get(3).map_or("", |m| m.as_str()),
                &base,
                known_paths,
            )
        })
        .collect();

    deps.extend(new_deps);
}

fn resolve_relative_cap(
    module: &str,
    names_str: &str,
    base: &str,
    known_paths: &HashSet<&str>,
) -> Vec<(String, DepKind)> {
    if module.is_empty() {
        names_str
            .split(',')
            .map(str::trim)
            .filter_map(|name| {
                let candidate = if base.is_empty() {
                    format!("{name}.py")
                } else {
                    format!("{base}/{name}.py")
                };
                known_paths
                    .contains(candidate.as_str())
                    .then_some((candidate, DepKind::Import))
            })
            .collect()
    } else {
        let candidate = if base.is_empty() {
            format!("{module}.py")
        } else {
            format!("{base}/{module}.py")
        };
        if known_paths.contains(candidate.as_str()) {
            vec![(candidate, DepKind::Import)]
        } else {
            Vec::new()
        }
    }
}

fn go_up(dir: &str, levels: usize) -> String {
    if levels == 0 || dir.is_empty() {
        return dir.to_owned();
    }
    let mut parts: Vec<&str> = dir.split('/').collect();
    for _ in 0..levels {
        if !parts.is_empty() {
            parts.pop();
        }
    }
    parts.join("/")
}

fn extract_absolute_imports(
    content: &str,
    known_paths: &HashSet<&str>,
    deps: &mut Vec<(String, DepKind)>,
) {
    let Some(re) = Regex::new(r"(?m)^(?:import\s+(\w+)|from\s+(\w[\w.]*)\s+import)").ok() else {
        return;
    };

    let new_deps: Vec<(String, DepKind)> = re
        .captures_iter(content)
        .filter_map(|cap| {
            let module = cap.get(1).or_else(|| cap.get(2)).map_or("", |m| m.as_str());
            let top = module.split('.').next().unwrap_or(module);
            if is_stdlib(top) {
                return None;
            }
            resolve_absolute_module(module, top, known_paths)
        })
        .collect();

    deps.extend(new_deps);
}

fn resolve_absolute_module(
    module: &str,
    top: &str,
    known_paths: &HashSet<&str>,
) -> Option<(String, DepKind)> {
    let path_parts: Vec<&str> = module.split('.').collect();

    for end in (1..=path_parts.len()).rev() {
        let Some(slice) = path_parts.get(..end) else {
            continue;
        };
        let candidate = format!("{}.py", slice.join("/"));
        if known_paths.contains(candidate.as_str()) {
            return Some((candidate, DepKind::Import));
        }
        let init = format!("{}/__init__.py", slice.join("/"));
        if known_paths.contains(init.as_str()) {
            return Some((init, DepKind::Import));
        }
    }

    let candidate = format!("{top}.py");
    known_paths
        .contains(candidate.as_str())
        .then_some((candidate, DepKind::Import))
}

fn is_stdlib(module: &str) -> bool {
    const STDLIB: &[&str] = &[
        "abc",
        "argparse",
        "array",
        "ast",
        "asyncio",
        "bisect",
        "collections",
        "concurrent",
        "contextlib",
        "copy",
        "csv",
        "dataclasses",
        "datetime",
        "dbm",
        "decimal",
        "dis",
        "email",
        "enum",
        "fnmatch",
        "fractions",
        "functools",
        "gc",
        "glob",
        "gzip",
        "hashlib",
        "heapq",
        "html",
        "http",
        "importlib",
        "inspect",
        "io",
        "itertools",
        "json",
        "linecache",
        "logging",
        "math",
        "multiprocessing",
        "operator",
        "os",
        "pathlib",
        "pickle",
        "pprint",
        "queue",
        "random",
        "re",
        "secrets",
        "shelve",
        "shutil",
        "signal",
        "socket",
        "sqlite3",
        "statistics",
        "string",
        "struct",
        "subprocess",
        "sys",
        "tarfile",
        "tempfile",
        "textwrap",
        "threading",
        "time",
        "tokenize",
        "traceback",
        "types",
        "typing",
        "unittest",
        "urllib",
        "weakref",
        "xml",
        "zipfile",
    ];
    STDLIB.contains(&module)
}

pub fn extract_doc(content: &str) -> Option<String> {
    let mut in_doc = false;
    let mut doc_lines = Vec::new();
    let mut quote_type = "";

    for line in content.lines() {
        let trimmed = line.trim();
        if !in_doc {
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("\"\"\"") {
                quote_type = "\"\"\"";
                in_doc = true;
                if let Some((internal, _)) = rest.split_once("\"\"\"") {
                    doc_lines.push(internal);
                    return Some(collapse_python_doc(&doc_lines));
                }
                doc_lines.push(rest);
            } else if let Some(rest) = trimmed.strip_prefix("'''") {
                quote_type = "'''";
                in_doc = true;
                if let Some((internal, _)) = rest.split_once("'''") {
                    doc_lines.push(internal);
                    return Some(collapse_python_doc(&doc_lines));
                }
                doc_lines.push(rest);
            } else {
                return None;
            }
        } else if let Some((internal, _)) = line.split_once(quote_type) {
            doc_lines.push(internal);
            return Some(collapse_python_doc(&doc_lines));
        } else {
            doc_lines.push(line);
        }
    }
    None
}

fn collapse_python_doc(lines: &[&str]) -> String {
    let first = lines.iter().map(|l| l.trim()).find(|l| !l.is_empty());
    if let Some(trimmed) = first {
        if let Some(idx) = trimmed.find(". ") {
            let mut s = trimmed[..idx + 1].to_string();
            if !s.ends_with('.') {
                s.push('.');
            }
            return s;
        } else if let Some(idx) = trimmed.find(".\t") {
            let mut s = trimmed[..idx + 1].to_string();
            if !s.ends_with('.') {
                s.push('.');
            }
            return s;
        } else if trimmed.ends_with('.') {
            return trimmed.to_string();
        } else {
            return format!("{trimmed}.");
        }
    }
    String::new()
}
