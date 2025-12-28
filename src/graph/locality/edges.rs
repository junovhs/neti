// src/graph/locality/edges.rs
//! Edge collection for locality analysis.
//!
//! Encapsulates import extraction and resolution so callers don't need
//! to reach into graph internals.

use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::graph::imports;
use crate::graph::resolver;

/// Collects all dependency edges from the given source files.
///
/// # Arguments
/// * `root` - Project root directory
/// * `files` - Source files to analyze
///
/// # Returns
/// Vec of (from, to) edges with paths relative to root.
///
/// # Errors
/// Returns error if file reading fails.
pub fn collect(root: &Path, files: &[PathBuf]) -> Result<Vec<(PathBuf, PathBuf)>> {
    let mut edges = Vec::new();

    for file in files {
        let file_edges = collect_file_edges(root, file)?;
        edges.extend(file_edges);
    }

    Ok(edges)
}

fn collect_file_edges(root: &Path, file: &Path) -> Result<Vec<(PathBuf, PathBuf)>> {
    let content = std::fs::read_to_string(file)?;
    let raw_imports = imports::extract(file, &content);

    let edges = raw_imports
        .iter()
        .filter_map(|import_str| {
            resolver::resolve(root, file, import_str).map(|resolved| {
                let from = normalize(file, root);
                let to = normalize(&resolved, root);
                (from, to)
            })
        })
        .collect();

    Ok(edges)
}

fn normalize(path: &Path, root: &Path) -> PathBuf {
    path.strip_prefix(root)
        .map_or_else(|_| path.to_path_buf(), Path::to_path_buf)
}