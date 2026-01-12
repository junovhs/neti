// src/graph/locality/cycles.rs
//! Cycle detection for the Law of Locality.
//!
//! Simply put: dependency cycles are architectural errors.
//! This module detects them using Depth-First Search (DFS).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Detects cycles in a dependency graph.
/// Returns a list of cycles, where each cycle is a list of nodes involved.
pub fn detect_cycles<'a, I>(edges: I) -> Vec<Vec<PathBuf>>
where
    I: Iterator<Item = (&'a Path, &'a Path)>,
{
    let mut adjacency: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    let mut nodes = HashSet::new();

    for (from, to) in edges {
        adjacency
            .entry(from.to_path_buf())
            .or_default()
            .push(to.to_path_buf());
        nodes.insert(from.to_path_buf());
        nodes.insert(to.to_path_buf());
    }

    let mut state = DfsState {
        visited: HashSet::new(),
        recursion_stack: HashSet::new(),
        path_stack: Vec::new(),
        cycles: Vec::new(),
    };

    // To ensure deterministic output for testing
    let mut sorted_nodes: Vec<_> = nodes.into_iter().collect();
    sorted_nodes.sort();

    for node in sorted_nodes {
        if !state.visited.contains(&node) {
            dfs(&node, &adjacency, &mut state);
        }
    }

    state.cycles
}

struct DfsState {
    visited: HashSet<PathBuf>,
    recursion_stack: HashSet<PathBuf>,
    path_stack: Vec<PathBuf>,
    cycles: Vec<Vec<PathBuf>>,
}

fn dfs(
    node: &PathBuf,
    adjacency: &HashMap<PathBuf, Vec<PathBuf>>,
    state: &mut DfsState,
) {
    state.visited.insert(node.clone());
    state.recursion_stack.insert(node.clone());
    state.path_stack.push(node.clone());

    if let Some(neighbors) = adjacency.get(node) {
        process_neighbors(neighbors, adjacency, state);
    }

    state.recursion_stack.remove(node);
    state.path_stack.pop();
}

fn process_neighbors(
    neighbors: &[PathBuf],
    adjacency: &HashMap<PathBuf, Vec<PathBuf>>,
    state: &mut DfsState,
) {
    let mut sorted_neighbors = neighbors.to_vec();
    sorted_neighbors.sort();

    for neighbor in sorted_neighbors {
        visit_neighbor(neighbor, adjacency, state);
    }
}

fn visit_neighbor(
    neighbor: PathBuf,
    adjacency: &HashMap<PathBuf, Vec<PathBuf>>,
    state: &mut DfsState,
) {
    if !state.visited.contains(&neighbor) {
        dfs(&neighbor, adjacency, state);
    } else if state.recursion_stack.contains(&neighbor) {
        record_cycle(neighbor, state);
    }
}

#[allow(clippy::indexing_slicing)] // Guarded: pos is from position() returning Some
fn record_cycle(
    neighbor: PathBuf,
    state: &mut DfsState,
) {
    if let Some(pos) = state.path_stack.iter().position(|x| x == &neighbor) {
        let mut cycle = state.path_stack[pos..].to_vec();
        cycle.push(neighbor); // Close the loop visually
        state.cycles.push(cycle);
    }
}



#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn test_no_cycles() {
        let edges = vec![
            (Path::new("a"), Path::new("b")),
            (Path::new("b"), Path::new("c")),
        ];
        let cycles = detect_cycles(edges.into_iter());
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_simple_cycle() {
        let edges = vec![
            (Path::new("a"), Path::new("b")),
            (Path::new("b"), Path::new("a")),
        ];
        let cycles = detect_cycles(edges.into_iter());
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 3); // a -> b -> a
    }

    #[test]
    fn test_diamond_dag_no_cycle() {
        let edges = vec![
            (Path::new("a"), Path::new("b")),
            (Path::new("a"), Path::new("c")),
            (Path::new("b"), Path::new("d")),
            (Path::new("c"), Path::new("d")),
        ];
        let cycles = detect_cycles(edges.into_iter());
        assert!(cycles.is_empty());
    }
}
