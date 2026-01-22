// src/pack/focus.rs
//! Focus mode computation for foveal/peripheral packing.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use crate::graph::rank::{RepoGraph, GraphEngine};

/// Computes foveal (full) and peripheral (skeleton) file sets.
#[must_use]
pub fn compute_sets(
    all_files: &[PathBuf],
    focus: &[PathBuf],
    depth: usize,
) -> (HashSet<PathBuf>, HashSet<PathBuf>) {
    let contents = read_files(all_files);
    let graph = build_graph(&contents);
    let all_set: HashSet<_> = all_files.iter().cloned().collect();

    let foveal = collect_foveal(focus, &all_set);
    let peripheral = collect_peripheral(&foveal, &graph, &all_set, depth);

    (foveal, peripheral)
}

fn read_files(files: &[PathBuf]) -> HashMap<PathBuf, String> {
    files
        .iter()
        .filter_map(|p| fs::read_to_string(p).ok().map(|c| (p.clone(), c)))
        .collect()
}

fn build_graph(contents: &HashMap<PathBuf, String>) -> RepoGraph {
    let file_vec: Vec<_> = contents
        .iter()
        .map(|(p, c)| (p.clone(), c.clone()))
        .collect();
    GraphEngine::build(&file_vec)
}

fn collect_foveal(focus: &[PathBuf], all_set: &HashSet<PathBuf>) -> HashSet<PathBuf> {
    focus
        .iter()
        .filter(|f| all_set.contains(*f))
        .cloned()
        .collect()
}

fn collect_peripheral(
    foveal: &HashSet<PathBuf>,
    graph: &RepoGraph,
    all_set: &HashSet<PathBuf>,
    depth: usize,
) -> HashSet<PathBuf> {
    let mut peripheral = HashSet::new();
    let mut frontier = foveal.clone();

    for _ in 0..depth {
        let next = expand_frontier(&frontier, foveal, &peripheral, graph, all_set);
        peripheral.extend(next.iter().cloned());
        frontier = next;
    }

    peripheral
}

fn expand_frontier(
    frontier: &HashSet<PathBuf>,
    foveal: &HashSet<PathBuf>,
    peripheral: &HashSet<PathBuf>,
    graph: &RepoGraph,
    all_set: &HashSet<PathBuf>,
) -> HashSet<PathBuf> {
    let mut next = HashSet::new();

    for anchor in frontier {
        for neighbor in graph.neighbors(anchor) {
            if should_include(&neighbor, foveal, peripheral, all_set) {
                next.insert(neighbor);
            }
        }
    }

    next
}

fn should_include(
    path: &PathBuf,
    foveal: &HashSet<PathBuf>,
    peripheral: &HashSet<PathBuf>,
    all_set: &HashSet<PathBuf>,
) -> bool {
    !foveal.contains(path) && !peripheral.contains(path) && all_set.contains(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> PathBuf { PathBuf::from(s) }

    fn manual_graph(edges: &[(&str, &str)]) -> RepoGraph {
        let mut defines: HashMap<String, HashSet<PathBuf>> = HashMap::new();
        let mut references: HashMap<String, HashSet<PathBuf>> = HashMap::new();

        for (src, dst) in edges {
            // src depends on dst. 
            // Implementation: src refs "SYM", dst defines "SYM".
            let sym = format!("SYM_{dst}");
            
            references.entry(sym.clone())
                .or_default()
                .insert(p(src));
                
            defines.entry(sym)
                .or_default()
                .insert(p(dst));
        }

        RepoGraph::new(
            Vec::new(), // tags
            defines,
            references,
            HashMap::new(), // ranks
        )
    }

    #[test]
    fn test_collect_foveal_logic() {
        let all_files: HashSet<_> = vec![p("a.rs"), p("b.rs"), p("c.rs")].into_iter().collect();
        let focus = vec![p("a.rs"), p("x.rs")]; 

        let foveal = collect_foveal(&focus, &all_files);
        assert!(foveal.contains(&p("a.rs")));
        assert!(!foveal.contains(&p("x.rs"))); 
        assert!(!foveal.contains(&p("b.rs")));
    }

    #[test]
    fn test_collect_peripheral_logic() {
        // Graph: a -> b -> c
        let graph = manual_graph(&[
            ("a.rs", "b.rs"),
            ("b.rs", "c.rs"),
        ]);
        let all_files: HashSet<_> = vec![p("a.rs"), p("b.rs"), p("c.rs")].into_iter().collect();
        
        let cases = vec![
            (
                "Depth 0",
                vec!["a.rs"],
                0,
                vec![]
            ),
            (
                "Depth 1",
                vec!["a.rs"],
                1,
                vec!["b.rs"]
            ),
            (
                "Depth 2",
                vec!["a.rs"],
                2,
                vec!["b.rs", "c.rs"]
            ),
        ];

        for (desc, foveal_strs, depth, expected_strs) in cases {
            let foveal: HashSet<_> = foveal_strs.iter().map(|s| p(s)).collect();
            let peripheral = collect_peripheral(&foveal, &graph, &all_files, depth);
            
            assert_eq!(peripheral.len(), expected_strs.len(), "{desc}: Count mismatch. \nFoveal: {foveal:?}\nPeripheral: {peripheral:?}");
            for exp in expected_strs {
                assert!(peripheral.contains(&p(exp)), "{desc}: Missing {exp}");
            }
            
            // Peripheral should never contain foveal
            for f in &foveal {
                assert!(!peripheral.contains(f), "{desc}: Leaked foveal {f:?}");
            }
        }
    }

    #[test]
    fn test_should_include_logic() {
        let foveal: HashSet<_> = vec![p("f")].into_iter().collect();
        let peripheral: HashSet<_> = vec![p("p")].into_iter().collect();
        let all_set: HashSet<_> = vec![p("f"), p("p"), p("other")].into_iter().collect();

        assert!(should_include(&p("other"), &foveal, &peripheral, &all_set));
        assert!(!should_include(&p("f"), &foveal, &peripheral, &all_set));
        assert!(!should_include(&p("p"), &foveal, &peripheral, &all_set));
        assert!(!should_include(&p("missing"), &foveal, &peripheral, &all_set));
    }
}
