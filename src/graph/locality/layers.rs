// src/graph/locality/layers.rs
//! Layer inference for the Law of Locality.
//!
//! Automatically infers architectural layers from the dependency graph.
//! A valid architecture must be a DAG where dependencies only flow downwards.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::types::LocalityEdge;
use super::analysis::ViolationKind;

/// Infers layers from a set of edges.
/// Returns a map of Path -> Layer Index.
pub fn infer_layers<'a, I>(edges: I) -> HashMap<PathBuf, usize>
where
    I: Iterator<Item = (&'a Path, &'a Path)> + Clone,
{
    let (nodes, dependencies) = build_dependency_map(edges);

    let mut layers: HashMap<PathBuf, usize> = HashMap::new();
    let mut current_layer_idx = 0;
    
    loop {
        let next_layer = find_next_layer(&nodes, &layers, &dependencies);

        if next_layer.is_empty() {
             handle_remaining_nodes(&nodes, &mut layers, current_layer_idx);
             break;
        }

        for node in next_layer {
            layers.insert(node, current_layer_idx);
        }
        
        if layers.len() == nodes.len() {
            break;
        }

        current_layer_idx += 1;
    }

    layers
}

fn build_dependency_map<'a, I>(edges: I) -> (HashSet<PathBuf>, HashMap<PathBuf, HashSet<PathBuf>>)
where
    I: Iterator<Item = (&'a Path, &'a Path)>,
{
    let mut dependencies: HashMap<PathBuf, HashSet<PathBuf>> = HashMap::new();
    let mut nodes = HashSet::new();

    for (from, to) in edges {
        nodes.insert(from.to_path_buf());
        nodes.insert(to.to_path_buf());
        if from != to {
            dependencies.entry(from.to_path_buf()).or_default().insert(to.to_path_buf());
        }
    }
    (nodes, dependencies)
}

fn find_next_layer(
    nodes: &HashSet<PathBuf>,
    layers: &HashMap<PathBuf, usize>,
    dependencies: &HashMap<PathBuf, HashSet<PathBuf>>,
) -> Vec<PathBuf> {
    let assigned_nodes: HashSet<_> = layers.keys().cloned().collect();
    let mut next_layer = Vec::new();

    for node in nodes {
        if layers.contains_key(node) {
            continue;
        }

        let is_ready = if let Some(deps) = dependencies.get(node) {
            deps.iter().all(|d| assigned_nodes.contains(d))
        } else {
            true
        };

        if is_ready {
            next_layer.push(node.clone());
        }
    }
    next_layer
}

fn handle_remaining_nodes(
    nodes: &HashSet<PathBuf>,
    layers: &mut HashMap<PathBuf, usize>,
    current_layer_idx: usize,
) {
     for node in nodes {
         if !layers.contains_key(node) {
             layers.insert(node.clone(), current_layer_idx + 1);
         }
     }
}

/// Checks if an edge violates layer ordering (must go down, never up).
/// Returns Some(violation) if the edge points from Lower -> Higher layer.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn check_layer_violation(
    edge: &LocalityEdge,
    layers: &HashMap<PathBuf, usize>,
) -> Option<ViolationKind> {
    let from_layer = layers.get(&edge.from).copied().unwrap_or(usize::MAX);
    let to_layer = layers.get(&edge.to).copied().unwrap_or(usize::MAX);

    if from_layer < to_layer {
        return Some(ViolationKind::UpwardDep);
    }

    None
}
