//! Monorepo-aware bare import resolution for JavaScript/TypeScript.

use std::collections::HashSet;

const EXTS: &[&str] = &["ts", "tsx", "js", "jsx", "mjs"];
const SRC_SUBDIRS: &[&str] = &["src", "lib", "dist", ""];

pub fn collect_package_roots(known_paths: &HashSet<&str>) -> Vec<String> {
    let mut roots: Vec<String> = known_paths
        .iter()
        .filter_map(|p| {
            let mut parts = p.splitn(3, '/');
            let first = parts.next()?;
            let second = parts.next()?;
            if first == "packages" {
                Some(format!("{first}/{second}"))
            } else {
                None
            }
        })
        .collect();
    roots.sort();
    roots.dedup();
    roots
}

pub fn resolve_bare(
    spec: &str,
    known_paths: &HashSet<&str>,
    pkg_roots: &[String],
) -> Option<String> {
    let without_scope = strip_scope(spec);

    for root in pkg_roots {
        let pkg_name = root.split('/').next_back().unwrap_or("");
        let sub_path = match without_scope.strip_prefix(pkg_name) {
            Some(rest) => rest.trim_start_matches('/'),
            None => continue,
        };
        if let Some(hit) = try_root_subdirs(root, sub_path, known_paths) {
            return Some(hit);
        }
    }

    None
}

fn strip_scope(spec: &str) -> &str {
    spec.strip_prefix('@')
        .and_then(|s| s.split_once('/').map(|(_, rest)| rest))
        .unwrap_or(spec)
}

fn try_root_subdirs(root: &str, sub_path: &str, known_paths: &HashSet<&str>) -> Option<String> {
    SRC_SUBDIRS.iter().find_map(|src| {
        let base = if src.is_empty() {
            root.to_owned()
        } else {
            format!("{root}/{src}")
        };
        let prefix = if sub_path.is_empty() {
            base
        } else {
            format!("{base}/{sub_path}")
        };
        probe_extensions(&prefix, known_paths)
    })
}

fn probe_extensions(prefix: &str, known_paths: &HashSet<&str>) -> Option<String> {
    EXTS.iter()
        .map(|ext| format!("{prefix}.{ext}"))
        .find(|c| known_paths.contains(c.as_str()))
        .or_else(|| {
            EXTS.iter()
                .map(|ext| format!("{prefix}/index.{ext}"))
                .find(|c| known_paths.contains(c.as_str()))
        })
}
