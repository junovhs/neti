// src/graph/resolver.rs
use crate::graph::tsconfig::TsConfig;
use std::path::{Path, PathBuf};

/// Resolves an import string to a likely file path on disk.
#[must_use]
pub fn resolve(project_root: &Path, current_file: &Path, import_str: &str) -> Option<PathBuf> {
    let ext = current_file.extension().and_then(|s| s.to_str())?;

    match ext {
        "rs" => resolve_rust(project_root, current_file, import_str),
        "ts" | "tsx" | "js" | "jsx" => resolve_ts(project_root, current_file, import_str),
        "py" => resolve_python(project_root, import_str),
        _ => None,
    }
}

fn resolve_rust(root: &Path, current: &Path, import: &str) -> Option<PathBuf> {
    if let Some(rest) = import.strip_prefix("crate::") {
        return resolve_crate_path(root, rest);
    }
    if import.starts_with("super::") {
        return resolve_super_path(current, import);
    }
    if let Some(rest) = import.strip_prefix("self::") {
        return resolve_self_path(current, rest);
    }
    if !import.contains("::") {
        return resolve_sibling_path(current, import);
    }
    None
}

fn resolve_crate_path(root: &Path, rest: &str) -> Option<PathBuf> {
    let parts: Vec<&str> = rest.split("::").collect();
    check_rust_variations(&root.join("src"), &parts)
}

fn resolve_super_path(current: &Path, import: &str) -> Option<PathBuf> {
    let mut parts: Vec<&str> = import.split("::").collect();
    let mut dir = current.parent()?;

    while parts.first() == Some(&"super") {
        parts.remove(0);
        dir = dir.parent()?;
    }

    if parts.is_empty() { return None; }
    check_rust_variations(dir, &parts)
}

fn resolve_self_path(current: &Path, rest: &str) -> Option<PathBuf> {
    let parts: Vec<&str> = rest.split("::").collect();
    check_rust_variations(current.parent()?, &parts)
}

fn resolve_sibling_path(current: &Path, import: &str) -> Option<PathBuf> {
    check_rust_variations(current.parent()?, &[import])
}

fn check_rust_variations(base: &Path, parts: &[&str]) -> Option<PathBuf> {
    let mut current = base.to_path_buf();
    for part in parts {
        current.push(part);
    }

    let file_path = current.with_extension("rs");
    if file_path.exists() { return Some(file_path); }

    let mod_path = current.join("mod.rs");
    if mod_path.exists() { return Some(mod_path); }

    None
}

fn resolve_ts(root: &Path, current: &Path, import: &str) -> Option<PathBuf> {
    if import.starts_with('.') {
        return resolve_relative_ts(current, import);
    }
    if is_node_module(import) {
        return None;
    }
    TsConfig::load(root).and_then(|cfg| cfg.resolve(import))
}

fn resolve_relative_ts(current: &Path, import: &str) -> Option<PathBuf> {
    let parent = current.parent()?;
    let path = parent.join(import);
    check_ts_file(&path).or_else(|| check_ts_index(&path))
}

/// Heuristic: bare specifiers without path separators are likely `node_modules`.
fn is_node_module(import: &str) -> bool {
    if import.starts_with('@') {
        return import.splitn(3, '/').count() <= 2;
    }
    !import.contains('/')
}

fn check_ts_file(path: &Path) -> Option<PathBuf> {
    if path.is_file() { return Some(path.to_path_buf()); }

    for ext in &["ts", "tsx", "js", "jsx", "json"] {
        let p = path.with_extension(ext);
        if p.exists() { return Some(p); }
    }
    None
}

fn check_ts_index(path: &Path) -> Option<PathBuf> {
    if !path.is_dir() { return None; }

    for ext in &["ts", "tsx", "js", "jsx"] {
        let p = path.join(format!("index.{ext}"));
        if p.exists() { return Some(p); }
    }
    None
}

fn resolve_python(root: &Path, import: &str) -> Option<PathBuf> {
    if import.starts_with('.') { return None; }

    let parts: Vec<&str> = import.split('.').collect();
    let mut current = root.to_path_buf();
    for part in &parts {
        current.push(part);
    }

    let file_path = current.with_extension("py");
    if file_path.exists() { return Some(file_path); }

    let init_path = current.join("__init__.py");
    if init_path.exists() { return Some(init_path); }

    None
}
