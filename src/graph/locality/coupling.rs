// src/graph/locality/coupling.rs
//! Afferent and Efferent coupling computation.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::types::Coupling;

/// Computes coupling metrics for all files in a dependency graph.
pub fn compute_coupling<'a, I>(edges: I) -> HashMap<PathBuf, Coupling>
where
    I: Iterator<Item = (&'a Path, &'a Path)>,
{
    let mut afferent: HashMap<PathBuf, usize> = HashMap::new();
    let mut efferent: HashMap<PathBuf, usize> = HashMap::new();
    let mut all_nodes: HashSet<PathBuf> = HashSet::new();

    for (from, to) in edges {
        let from_buf = from.to_path_buf();
        let to_buf = to.to_path_buf();

        all_nodes.insert(from_buf.clone());
        all_nodes.insert(to_buf.clone());

        *efferent.entry(from_buf).or_insert(0) += 1;
        *afferent.entry(to_buf).or_insert(0) += 1;
    }

    all_nodes
        .into_iter()
        .map(|path| {
            let ca = afferent.get(&path).copied().unwrap_or(0);
            let ce = efferent.get(&path).copied().unwrap_or(0);
            (path, Coupling::new(ca, ce))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_coupling() {
        let edges: &[(&Path, &Path)] = &[
            (Path::new("a.rs"), Path::new("hub.rs")),
            (Path::new("b.rs"), Path::new("hub.rs")),
            (Path::new("c.rs"), Path::new("hub.rs")),
            (Path::new("hub.rs"), Path::new("types.rs")),
        ];

        let coupling = compute_coupling(edges.iter().copied());

        let hub = coupling
            .get(Path::new("hub.rs"))
            .cloned()
            .unwrap_or_default();
        assert_eq!(hub.afferent(), 3);
        assert_eq!(hub.efferent(), 1);
        assert!(hub.skew() > 0.0);

        let a = coupling
            .get(Path::new("a.rs"))
            .cloned()
            .unwrap_or_default();
        assert_eq!(a.afferent(), 0);
        assert_eq!(a.efferent(), 1);
        assert!(a.skew() < 0.0);
    }
}
