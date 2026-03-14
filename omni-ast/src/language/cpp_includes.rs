use crate::types::DepKind;
use std::collections::{BTreeSet, HashSet};
use std::path::Path;

const BUILD_ONLY_INCLUDE_NAMES: &[&str] = &[
    "pch.h",
    "stdafx.h",
    "framework.h",
    "targetver.h",
    "resource.h",
];

pub fn extract_imports(
    content: &str,
    source_path: &str,
    known_paths: &HashSet<&str>,
) -> Vec<(String, DepKind)> {
    let base_dir = Path::new(source_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    find_quoted_includes(content)
        .into_iter()
        .flat_map(|include| resolve_include(base_dir, include, known_paths))
        .map(|path| (path, DepKind::Import))
        .collect()
}

pub fn is_build_only_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/").to_lowercase();
    let filename = normalized.rsplit('/').next().unwrap_or(&normalized);
    BUILD_ONLY_INCLUDE_NAMES.contains(&filename)
        || filename.starts_with("pch.")
        || filename.starts_with("stdafx.")
        || filename.contains("precomp")
        || filename.contains("precompiled")
}

fn find_quoted_includes(content: &str) -> Vec<&str> {
    let Ok(re) = regex::Regex::new(r#"(?m)^\s*#\s*include\s*\"([^\"]+)\""#) else {
        return Vec::new();
    };
    re.captures_iter(content)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str()))
        .collect()
}

fn resolve_include(
    base_dir: &Path,
    include: &str,
    known_paths: &HashSet<&str>,
) -> BTreeSet<String> {
    let normalized_include = normalize_rel_path(Path::new(include));
    let joined = normalize_rel_path(&base_dir.join(include));
    known_paths
        .iter()
        .filter(|known| {
            let known = **known;
            known == joined
                || known == normalized_include
                || known.ends_with(&format!("/{normalized_include}"))
        })
        .map(|known| (*known).to_owned())
        .collect()
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
