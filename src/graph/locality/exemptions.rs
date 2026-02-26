// src/graph/locality/exemptions.rs
//! Smart structural exemptions for Rust module patterns.
//!
//! Distinguishes legitimate vertical routing from sideways spaghetti.

use std::path::Path;

/// Checks if an edge is a structural Rust pattern that should be auto-exempted.
#[must_use]
pub fn is_structural_pattern(from: &Path, to: &Path) -> bool {
    is_crate_root(from)
        || is_parent_reexport(from, to)
        || is_child_to_parent(from, to)
        || is_vertical_routing(from, to)
        || is_shared_infrastructure(to)
}

/// lib.rs can import anything - it's the crate root.
fn is_crate_root(from: &Path) -> bool {
    from.file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|name| name == "lib.rs" || name == "main.rs")
}

/// mod.rs re-exporting direct children is structural.
fn is_parent_reexport(from: &Path, to: &Path) -> bool {
    let Some(from_name) = from.file_name().and_then(|s| s.to_str()) else {
        return false;
    };

    if from_name != "mod.rs" {
        return false;
    }

    let Some(from_dir) = from.parent() else {
        return false;
    };

    to.starts_with(from_dir)
}

/// Child importing from parent mod.rs for re-exports.
fn is_child_to_parent(from: &Path, to: &Path) -> bool {
    let Some(to_name) = to.file_name().and_then(|s| s.to_str()) else {
        return false;
    };

    if to_name != "mod.rs" {
        return false;
    }

    let Some(to_dir) = to.parent() else {
        return false;
    };

    from.starts_with(to_dir)
}

/// Vertical routing: files in the same module subtree.
fn is_vertical_routing(from: &Path, to: &Path) -> bool {
    let from_module = get_top_module(from);
    let to_module = get_top_module(to);

    match (from_module, to_module) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// Shared infrastructure: files directly under src/ (not in subdirs).
/// These are intentional cross-cutting utilities like types.rs, skeleton.rs.
fn is_shared_infrastructure(to: &Path) -> bool {
    let parts: Vec<_> = to.components().collect();

    // Pattern: src/<file>.rs (exactly 2 components after any prefix)
    let src_idx = parts.iter().position(|c| c.as_os_str() == "src");

    match src_idx {
        Some(idx) => {
            // Should be exactly: src/<file>.rs
            // parts[idx] = "src", parts[idx+1] = "<file>.rs", no more
            let remaining = parts.len() - idx - 1;
            remaining == 1
        }
        None => false,
    }
}

fn get_top_module(path: &Path) -> Option<String> {
    let parts: Vec<_> = path.components().collect();

    let src_idx = parts.iter().position(|c| c.as_os_str() == "src")?;

    parts
        .get(src_idx + 1)
        .and_then(|c| c.as_os_str().to_str())
        .map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_crate_root() {
        assert!(is_crate_root(Path::new("src/lib.rs")));
        assert!(is_crate_root(Path::new("src/main.rs")));
        assert!(!is_crate_root(Path::new("src/types.rs")));
    }

    #[test]
    fn test_parent_reexport() {
        let from = Path::new("src/graph/mod.rs");
        let to = Path::new("src/graph/rank/mod.rs");
        assert!(is_parent_reexport(from, to));
    }

    #[test]
    fn test_vertical_routing() {
        let a = Path::new("src/apply/writer.rs");
        let b = Path::new("src/apply/types.rs");
        assert!(is_vertical_routing(a, b));

        let c = Path::new("src/cli/handlers.rs");
        assert!(!is_vertical_routing(a, c));
    }

    #[test]
    fn test_shared_infrastructure() {
        assert!(is_shared_infrastructure(Path::new("src/types.rs")));
        assert!(is_shared_infrastructure(Path::new("src/skeleton.rs")));
        assert!(!is_shared_infrastructure(Path::new("src/apply/mod.rs")));
        assert!(!is_shared_infrastructure(Path::new("src/graph/imports.rs")));
    }
}
