// src/graph/rank/graph.rs
//! The dependency graph structure and query interface.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// The dependency graph and ranker.
#[derive(Clone)]
pub struct RepoGraph {
    pub(crate) tags: Vec<crate::graph::rank::tags::Tag>,
    pub(crate) defines: HashMap<String, HashSet<PathBuf>>,
    /// References map: Symbol -> Set of files that reference it.
    /// Changed from `Vec` to `HashSet` to ensure O(1) lookups and fix P06 violations.
    pub(crate) references: HashMap<String, HashSet<PathBuf>>,
    pub(crate) ranks: HashMap<PathBuf, f64>,
}


impl RepoGraph {
    /// Creates a new graph container.
    #[must_use]
    pub fn new(
        tags: Vec<crate::graph::rank::tags::Tag>,
        defines: HashMap<String, HashSet<PathBuf>>,
        references: HashMap<String, HashSet<PathBuf>>,
        ranks: HashMap<PathBuf, f64>,
    ) -> Self {
        Self { tags, defines, references, ranks }
    }

    /// Returns files ranked by importance.
    #[must_use]
    pub fn ranked_files(&self) -> Vec<(PathBuf, f64)> {
        crate::graph::rank::queries::get_ranked_files(self)
    }

    /// Returns files directly connected to the anchor.
    #[must_use]
    pub fn neighbors(&self, anchor: &Path) -> Vec<PathBuf> {
        crate::graph::rank::queries::get_neighbors(self, anchor)
    }

    /// Returns files that this file depends on.
    #[must_use]
    pub fn dependencies(&self, anchor: &Path) -> Vec<PathBuf> {
        crate::graph::rank::queries::get_dependencies(self, anchor)
    }

    /// Returns files that depend on this file.
    #[must_use]
    pub fn dependents(&self, anchor: &Path) -> Vec<PathBuf> {
        crate::graph::rank::queries::get_dependents(self, anchor)
    }

    /// Returns definition tags only.
    #[must_use]
    pub fn graph_tags(&self) -> Vec<crate::graph::rank::tags::Tag> {
        crate::graph::rank::queries::get_graph_tags(self)
    }

    /// Returns true if this file is a hub.
    #[must_use]
    pub fn is_hub(&self, anchor: &Path, threshold: usize) -> bool {
        self.dependents(anchor).len() >= threshold
    }
}