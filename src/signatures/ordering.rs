// src/signatures/ordering.rs
//! Topological ordering for holographic signatures.

use crate::graph::rank::RepoGraph;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

/// Computes topological order: files with no dependencies first.
///
/// Falls back to `PageRank` order for cycles or disconnected nodes.
#[must_use]
pub fn topological_order(graph: &RepoGraph, all_files: &[PathBuf]) -> Vec<PathBuf> {
    let file_set: HashSet<_> = all_files.iter().cloned().collect();
    let (in_degree, adjacency) = build_graph_structures(graph, &file_set);

    let mut result = kahn_sort(&in_degree, &adjacency);
    append_remaining(&mut result, all_files, graph);

    result
}

fn build_graph_structures(
    graph: &RepoGraph,
    file_set: &HashSet<PathBuf>,
) -> (HashMap<PathBuf, usize>, HashMap<PathBuf, Vec<PathBuf>>) {
    let mut in_degree: HashMap<PathBuf, usize> = HashMap::new();
    let mut adjacency: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

    for file in file_set {
        in_degree.entry(file.clone()).or_insert(0);
        adjacency.entry(file.clone()).or_default();
    }

    for file in file_set {
        for dep in graph.dependencies(file) {
            if file_set.contains(&dep) {
                adjacency.entry(dep.clone()).or_default().push(file.clone());
                *in_degree.entry(file.clone()).or_default() += 1;
            }
        }
    }

    (in_degree, adjacency)
}

fn kahn_sort(
    in_degree: &HashMap<PathBuf, usize>,
    adjacency: &HashMap<PathBuf, Vec<PathBuf>>,
) -> Vec<PathBuf> {
    let mut degrees = in_degree.clone();
    let mut queue: VecDeque<PathBuf> = degrees
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(p, _)| p.clone())
        .collect();

    let mut result = Vec::new();

    while let Some(node) = queue.pop_front() {
        result.push(node.clone());

        let Some(neighbors) = adjacency.get(&node) else {
            continue;
        };

        for neighbor in neighbors {
            let Some(deg) = degrees.get_mut(neighbor) else {
                continue;
            };
            *deg = deg.saturating_sub(1);
            if *deg == 0 {
                queue.push_back(neighbor.clone());
            }
        }
    }

    result
}

fn append_remaining(result: &mut Vec<PathBuf>, all_files: &[PathBuf], graph: &RepoGraph) {
    let in_result: HashSet<_> = result.iter().cloned().collect();

    let mut remaining: Vec<_> = all_files
        .iter()
        .filter(|f| !in_result.contains(*f))
        .cloned()
        .collect();

    let ranks: HashMap<_, _> = graph.ranked_files().into_iter().collect();
    remaining.sort_by(|a, b| {
        let ra = ranks.get(a).unwrap_or(&0.0);
        let rb = ranks.get(b).unwrap_or(&0.0);
        rb.partial_cmp(ra).unwrap_or(std::cmp::Ordering::Equal)
    });

    result.extend(remaining);
}