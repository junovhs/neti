// src/graph/locality/distance.rs
//! Dependency Distance calculator via Lowest Common Ancestor (LCA).

use std::path::{Path, PathBuf};

/// Computes D(a, b) = (depth(a) - depth(LCA)) + (depth(b) - depth(LCA)).
#[must_use]
pub fn compute_distance(from: &Path, to: &Path) -> usize {
    let from_components: Vec<_> = from.components().collect();
    let to_components: Vec<_> = to.components().collect();

    let lca_depth = find_lca_depth(&from_components, &to_components);
    let from_depth = from_components.len();
    let to_depth = to_components.len();

    (from_depth - lca_depth) + (to_depth - lca_depth)
}

#[allow(clippy::indexing_slicing)] // Guarded: loop bound is min(a.len(), b.len())
fn find_lca_depth<T: PartialEq>(a: &[T], b: &[T]) -> usize {
    let mut depth = 0;
    let min_len = a.len().min(b.len());

    for i in 0..min_len {
        if a[i] == b[i] {
            depth = i + 1;
        } else {
            break;
        }
    }
    depth
}

/// Returns the LCA path between two files.
#[must_use]
pub fn find_lca(from: &Path, to: &Path) -> PathBuf {
    let from_components: Vec<_> = from.components().collect();
    let to_components: Vec<_> = to.components().collect();

    let lca_depth = find_lca_depth(&from_components, &to_components);

    from_components
        .iter()
        .take(lca_depth)
        .fold(PathBuf::new(), |mut acc, c| {
            acc.push(c);
            acc
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_directory() {
        let a = Path::new("src/apply/parser.rs");
        let b = Path::new("src/apply/types.rs");
        assert_eq!(compute_distance(a, b), 2);
    }

    #[test]
    fn test_sibling_directories() {
        let a = Path::new("src/tui/view.rs");
        let b = Path::new("src/apply/parser.rs");
        assert_eq!(compute_distance(a, b), 4);
    }

    #[test]
    fn test_deep_hierarchy() {
        let a = Path::new("src/tui/dashboard/widgets/sidebar.rs");
        let b = Path::new("src/apply/patch/v1/context.rs");
        assert_eq!(compute_distance(a, b), 8);
    }
}
