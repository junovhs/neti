// src/graph/rank/mod.rs
pub mod builder;
pub mod graph;
pub mod pagerank;
pub mod tags;
pub mod queries;

pub use graph::RepoGraph;
use std::path::Path;

/// Orchestrates graph construction and ranking.
pub struct GraphEngine;

impl GraphEngine {
    #[must_use]
    pub fn build(files: &[(std::path::PathBuf, String)]) -> RepoGraph {
        let data = builder::build_data(files);
        let ranks = pagerank::compute(&data.edges, &data.all_files, None);
        RepoGraph::new(data.tags, data.defines, data.references, ranks)
    }

    pub fn focus_on(graph: &mut RepoGraph, anchor: &Path) {
        let (edges, all_files) = builder::rebuild_topology(&graph.defines, &graph.references);
        graph.ranks = pagerank::compute(&edges, &all_files, Some(&anchor.to_path_buf()));
    }
}
